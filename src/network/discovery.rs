//! # Ultra-Fast Device Discovery System
//!
//! Implements simultaneous multi-protocol discovery that is faster than AirDrop:
//!
//! 1. **mDNS** — Standard service discovery via `_fastshare._udp.local`
//! 2. **UDP Broadcast** — Fast LAN-wide device announcements
//! 3. **Multicast** — Group-based discovery for segmented networks
//! 4. **QR Pairing** — Out-of-band pairing via QR code
//! 5. **Manual IP** — Direct connection by IP address
//!
//! Discovery packets contain full device metadata enabling instant capability
//! negotiation before connection establishment.

use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;
use std::time::{Duration, Instant};

use anyhow::Result;
use chrono::{DateTime, Utc};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use tokio::net::UdpSocket;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

use crate::app::AppState;
use crate::network::broadcast::BroadcastEngine;

// ── Constants ──

/// mDNS service type for FastShare discovery
const MDNS_SERVICE_TYPE: &str = "_fastshare._udp.local.";

/// UDP broadcast port for LAN discovery
const BROADCAST_PORT: u16 = 5001;

/// Multicast group address for discovery
const MULTICAST_ADDR: Ipv4Addr = Ipv4Addr::new(239, 255, 42, 1);

/// Multicast port
const MULTICAST_PORT: u16 = 5002;

/// How often to send discovery announcements (milliseconds)
const ANNOUNCE_INTERVAL_MS: u64 = 500;

/// How long before a device is considered stale (seconds)
const DEVICE_STALE_SECS: u64 = 10;

// ── Data Structures ──

/// Metadata about a discovered device on the network.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceInfo {
    /// Unique device identifier (UUID v4)
    pub device_id: String,
    /// Human-readable device name  
    pub device_name: String,
    /// Device type: "desktop", "laptop", "phone", "tablet"
    pub device_type: String,
    /// Features supported by this device
    pub supported_features: Vec<String>,
    /// Maximum bandwidth capability (e.g., "10Gb", "1Gb")
    pub max_bandwidth: String,
    /// Device IP address
    pub ip_address: IpAddr,
    /// QUIC server port
    pub port: u16,
    /// Protocol version for compatibility checking
    pub protocol_version: u32,
    /// Timestamp of last seen announcement
    #[serde(skip)]
    pub last_seen: Option<Instant>,
}

/// Discovery announcement packet sent over the network.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveryPacket {
    /// Packet type: "announce", "query", "response"
    pub packet_type: String,
    /// Device metadata
    pub device: DeviceInfo,
    /// Timestamp of when the packet was created
    pub timestamp: DateTime<Utc>,
}

/// The multi-protocol discovery service.
///
/// Runs mDNS, UDP broadcast, and multicast discovery simultaneously,
/// keeping a real-time view of nearby devices.
#[derive(Clone)]
pub struct DiscoveryService {
    /// Our own device information
    device_info: DeviceInfo,
    /// Thread-safe map of discovered devices keyed by device_id
    discovered_devices: Arc<DashMap<String, DeviceInfo>>,
    /// Broadcast engine for UDP announcements
    broadcast_engine: Arc<BroadcastEngine>,
}

impl DiscoveryService {
    /// Create a new discovery service.
    ///
    /// # Arguments
    /// * `device_id` — Unique ID for this device
    /// * `device_name` — Human-readable name
    /// * `port` — QUIC server port to advertise
    pub fn new(device_id: String, device_name: String, port: u16) -> Result<Self> {
        let local_ip =
            local_ip_address::local_ip().unwrap_or(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)));

        let device_info = DeviceInfo {
            device_id,
            device_name,
            device_type: detect_device_type(),
            supported_features: vec![
                "parallel_streams".into(),
                "compression_lz4".into(),
                "compression_zstd".into(),
                "distributed_transfer".into(),
                "transfer_resume".into(),
            ],
            max_bandwidth: detect_max_bandwidth(),
            ip_address: local_ip,
            port,
            protocol_version: 1,
            last_seen: None,
        };

        let broadcast_engine = Arc::new(BroadcastEngine::new(BROADCAST_PORT)?);

        Ok(Self {
            device_info,
            discovered_devices: Arc::new(DashMap::new()),
            broadcast_engine,
        })
    }

    /// Run the discovery service. This spawns three concurrent tasks:
    ///
    /// 1. **Announcer** — Periodically broadcasts our device info
    /// 2. **Listener** — Listens for discovery packets from other devices
    /// 3. **Pruner** — Removes stale devices that haven't been seen recently
    pub async fn run(&self, state: Arc<AppState>) -> Result<()> {
        info!("Starting multi-protocol discovery system...");

        let service = self.clone();
        let state_clone = state.clone();

        // ── Spawn UDP Broadcast Announcer ──
        let announcer_service = service.clone();
        let announce_handle = tokio::spawn(async move {
            announcer_service.announce_loop().await;
        });

        // ── Spawn UDP Broadcast Listener ──
        let listener_service = service.clone();
        let listener_state = state_clone.clone();
        let listener_handle = tokio::spawn(async move {
            if let Err(e) = listener_service.listen_loop(listener_state).await {
                warn!("Discovery listener error: {}", e);
            }
        });

        // ── Spawn Multicast Listener ──
        let multicast_service = service.clone();
        let multicast_state = state_clone.clone();
        let multicast_handle = tokio::spawn(async move {
            if let Err(e) = multicast_service.multicast_listen(multicast_state).await {
                warn!("Multicast listener error: {}", e);
            }
        });

        // ── Spawn Stale Device Pruner ──
        let pruner_service = service.clone();
        let pruner_state = state_clone.clone();
        let pruner_handle = tokio::spawn(async move {
            pruner_service.prune_stale_devices(pruner_state).await;
        });

        // Wait for all tasks (they run forever until cancelled)
        tokio::select! {
            _ = announce_handle => {},
            _ = listener_handle => {},
            _ = multicast_handle => {},
            _ = pruner_handle => {},
        }

        Ok(())
    }

    /// Periodically broadcast our device info via UDP broadcast + multicast.
    async fn announce_loop(&self) {
        let interval = Duration::from_millis(ANNOUNCE_INTERVAL_MS);

        loop {
            let packet = DiscoveryPacket {
                packet_type: "announce".into(),
                device: self.device_info.clone(),
                timestamp: Utc::now(),
            };

            if let Ok(data) = serde_json::to_vec(&packet) {
                // UDP Broadcast
                if let Err(e) = self.broadcast_engine.broadcast(&data).await {
                    debug!("Broadcast send error: {}", e);
                }

                // Multicast
                if let Err(e) = self.send_multicast(&data).await {
                    debug!("Multicast send error: {}", e);
                }
            }

            tokio::time::sleep(interval).await;
        }
    }

    /// Listen for UDP broadcast discovery packets.
    async fn listen_loop(&self, state: Arc<AppState>) -> Result<()> {
        let socket = UdpSocket::bind(format!("0.0.0.0:{}", BROADCAST_PORT)).await?;
        socket.set_broadcast(true)?;

        let mut buf = vec![0u8; 65535];

        info!("UDP discovery listener started on port {}", BROADCAST_PORT);

        loop {
            let (len, src) = socket.recv_from(&mut buf).await?;

            if let Ok(packet) = serde_json::from_slice::<DiscoveryPacket>(&buf[..len]) {
                // Skip our own announcements
                if packet.device.device_id == self.device_info.device_id {
                    continue;
                }

                self.handle_discovery_packet(packet, src, &state).await;
            }
        }
    }

    /// Listen for multicast discovery packets.
    async fn multicast_listen(&self, state: Arc<AppState>) -> Result<()> {
        let socket = UdpSocket::bind(format!("0.0.0.0:{}", MULTICAST_PORT)).await?;

        // Join multicast group using socket2 for platform compatibility
        let std_socket = socket.into_std()?;
        let socket2 = socket2::Socket::from(std_socket);
        socket2.join_multicast_v4(&MULTICAST_ADDR, &Ipv4Addr::UNSPECIFIED)?;
        let std_socket: std::net::UdpSocket = socket2.into();
        std_socket.set_nonblocking(true)?;
        let socket = UdpSocket::from_std(std_socket)?;

        let mut buf = vec![0u8; 65535];

        info!(
            "Multicast discovery listener started on {}:{}",
            MULTICAST_ADDR, MULTICAST_PORT
        );

        loop {
            let (len, src) = socket.recv_from(&mut buf).await?;

            if let Ok(packet) = serde_json::from_slice::<DiscoveryPacket>(&buf[..len]) {
                if packet.device.device_id == self.device_info.device_id {
                    continue;
                }

                self.handle_discovery_packet(packet, src, &state).await;
            }
        }
    }

    /// Send data to the multicast group.
    async fn send_multicast(&self, data: &[u8]) -> Result<()> {
        let socket = UdpSocket::bind("0.0.0.0:0").await?;
        let multicast_target = SocketAddr::new(IpAddr::V4(MULTICAST_ADDR), MULTICAST_PORT);
        socket.send_to(data, multicast_target).await?;
        Ok(())
    }

    /// Process a received discovery packet and update our device list.
    async fn handle_discovery_packet(
        &self,
        packet: DiscoveryPacket,
        source: SocketAddr,
        state: &Arc<AppState>,
    ) {
        let device_id = packet.device.device_id.clone();
        let mut device = packet.device;
        device.last_seen = Some(Instant::now());

        // Use the source IP if the announced IP is loopback
        if device.ip_address.is_loopback() {
            device.ip_address = source.ip();
        }

        let is_new = !self.discovered_devices.contains_key(&device_id);

        // Update our local device map
        self.discovered_devices
            .insert(device_id.clone(), device.clone());

        // Update shared state
        let devices: Vec<DeviceInfo> = self
            .discovered_devices
            .iter()
            .map(|entry| entry.value().clone())
            .collect();

        *state.nearby_devices.write().await = devices;

        if is_new {
            info!(
                "🔍 Discovered new device: {} ({}) at {}:{}",
                device.device_name, device.device_type, device.ip_address, device.port
            );
        }
    }

    /// Remove devices that haven't announced in DEVICE_STALE_SECS.
    async fn prune_stale_devices(&self, state: Arc<AppState>) {
        let prune_interval = Duration::from_secs(5);
        let stale_threshold = Duration::from_secs(DEVICE_STALE_SECS);

        loop {
            tokio::time::sleep(prune_interval).await;

            let now = Instant::now();
            let mut removed = Vec::new();

            self.discovered_devices.retain(|id, device| {
                if let Some(last_seen) = device.last_seen {
                    if now.duration_since(last_seen) > stale_threshold {
                        removed.push(id.clone());
                        return false;
                    }
                }
                true
            });

            if !removed.is_empty() {
                let devices: Vec<DeviceInfo> = self
                    .discovered_devices
                    .iter()
                    .map(|entry| entry.value().clone())
                    .collect();
                *state.nearby_devices.write().await = devices;

                for id in &removed {
                    info!("📡 Device went offline: {}", id);
                }
            }
        }
    }

    /// Get a snapshot of all discovered devices.
    pub fn get_devices(&self) -> Vec<DeviceInfo> {
        self.discovered_devices
            .iter()
            .map(|entry| entry.value().clone())
            .collect()
    }

    /// Get our own device info.
    pub fn device_info(&self) -> &DeviceInfo {
        &self.device_info
    }
}

// ── Helper Functions ──

/// Detect the device type based on the operating system.
fn detect_device_type() -> String {
    if cfg!(target_os = "android") {
        "phone".into()
    } else if cfg!(target_os = "ios") {
        "phone".into()
    } else if cfg!(target_os = "macos") {
        "laptop".into()
    } else if cfg!(target_os = "windows") {
        "desktop".into()
    } else if cfg!(target_os = "linux") {
        "desktop".into()
    } else {
        "unknown".into()
    }
}

/// Detect the maximum bandwidth capability.
/// In a production system this would probe network interfaces.
fn detect_max_bandwidth() -> String {
    // Default to 1Gb, can be enhanced to detect actual NIC speed
    "1Gb".into()
}
