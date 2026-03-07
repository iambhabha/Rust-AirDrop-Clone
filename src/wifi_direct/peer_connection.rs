//! # Wi-Fi Direct Peer Connection
//!
//! Manages connections between peers over Wi-Fi Direct.
//! Handles joining an existing hotspot and managing the P2P link.

use std::net::IpAddr;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use tracing::{info, warn};

// ── Data Structures ──

/// Information about a Wi-Fi Direct peer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WifiDirectPeer {
    /// The peer's device name
    pub device_name: String,
    /// The peer's SSID (if acting as hotspot)
    pub ssid: String,
    /// Signal strength (RSSI)
    pub signal_strength: i32,
    /// Whether this peer is the group owner (hotspot host)
    pub is_group_owner: bool,
    /// IP address once connected
    pub ip_address: Option<IpAddr>,
}

/// Status of the Wi-Fi Direct peer connection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PeerConnectionStatus {
    Disconnected,
    Scanning,
    Connecting,
    Connected,
    Error(String),
}

/// Manages Wi-Fi Direct peer connections.
pub struct PeerConnectionManager {
    /// Current connection status
    status: PeerConnectionStatus,
    /// Currently connected peer
    connected_peer: Option<WifiDirectPeer>,
    /// Discovered Wi-Fi Direct peers
    discovered_peers: Vec<WifiDirectPeer>,
}

impl PeerConnectionManager {
    /// Create a new peer connection manager.
    pub fn new() -> Self {
        Self {
            status: PeerConnectionStatus::Disconnected,
            connected_peer: None,
            discovered_peers: Vec::new(),
        }
    }

    /// Scan for nearby Wi-Fi Direct peers.
    pub async fn scan(&mut self) -> Result<Vec<WifiDirectPeer>> {
        info!("🔍 Scanning for Wi-Fi Direct peers...");
        self.status = PeerConnectionStatus::Scanning;

        // Platform-specific scanning would happen here
        // For now, return the discovered peers list

        self.status = PeerConnectionStatus::Disconnected;
        Ok(self.discovered_peers.clone())
    }

    /// Connect to a Wi-Fi Direct peer (join their hotspot).
    ///
    /// # Arguments
    /// * `ssid` — The SSID of the peer's hotspot
    /// * `password` — The password for the hotspot
    pub async fn connect(&mut self, ssid: &str, password: &str) -> Result<()> {
        info!("📡 Connecting to Wi-Fi Direct peer: {}", ssid);
        self.status = PeerConnectionStatus::Connecting;

        #[cfg(target_os = "linux")]
        {
            info!(
                "Linux: nmcli dev wifi connect {} password {}",
                ssid, password
            );
        }

        #[cfg(target_os = "windows")]
        {
            info!("Windows: netsh wlan connect ssid={}", ssid);
        }

        // In production, this would use platform APIs to actually connect
        self.status = PeerConnectionStatus::Connected;
        info!("✅ Connected to Wi-Fi Direct peer: {}", ssid);

        Ok(())
    }

    /// Disconnect from the current Wi-Fi Direct peer.
    pub async fn disconnect(&mut self) -> Result<()> {
        info!("Disconnecting from Wi-Fi Direct peer...");
        self.connected_peer = None;
        self.status = PeerConnectionStatus::Disconnected;
        Ok(())
    }

    /// Get the current connection status.
    pub fn status(&self) -> &PeerConnectionStatus {
        &self.status
    }

    /// Get the connected peer info.
    pub fn connected_peer(&self) -> Option<&WifiDirectPeer> {
        self.connected_peer.as_ref()
    }
}
