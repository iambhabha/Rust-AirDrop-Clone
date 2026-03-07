//! # Application Root
//!
//! Orchestrates all FastShare subsystems: discovery, QUIC server,
//! transfer engine, network optimizer, and the UI event loop.

use anyhow::Result;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use tracing::info;
use uuid::Uuid;

use crate::distributed::peer_manager::PeerManager;
use crate::network::connection::QuicServer;
use crate::network::discovery::{DeviceInfo, DiscoveryService};
use crate::optimizer::network_monitor::NetworkMonitor;
use crate::storage::chunk_storage::ChunkStorage;
use crate::transfer::receiver::TransferReceiver;
use crate::transfer::sender::TransferSender;

/// Shared application state accessible across all subsystems.
pub struct AppState {
    /// Unique device identifier for this node
    pub device_id: String,
    /// Human-readable device name
    pub device_name: String,
    /// QUIC server listening address
    pub listen_addr: SocketAddr,
    /// Currently discovered nearby devices
    pub nearby_devices: Arc<RwLock<Vec<DeviceInfo>>>,
    /// Network performance monitor
    pub network_monitor: Arc<NetworkMonitor>,
    /// Peer manager for distributed transfers
    pub peer_manager: Arc<PeerManager>,
    /// Chunk storage for temporary chunk persistence
    pub chunk_storage: Arc<ChunkStorage>,
    /// Download path for received files
    pub download_path: String,
}

/// Top-level application that ties all subsystems together.
pub struct App {
    pub state: Arc<AppState>,
    pub quic_server: QuicServer,
    pub discovery: DiscoveryService,
    /// Shutdown signal broadcaster
    shutdown_tx: broadcast::Sender<()>,
}

impl App {
    /// Create a new FastShare application instance.
    ///
    /// This initializes:
    /// 1. A unique device ID
    /// 2. The QUIC server (self-signed TLS)
    /// 3. The mDNS/UDP discovery service
    /// 4. The network performance monitor
    /// 5. The distributed peer manager
    /// 6. Chunk storage for temporary file chunks
    pub async fn new(download_path: String, temp_path: String) -> Result<Self> {
        let device_id = Uuid::new_v4().to_string();
        let device_name = hostname::get()
            .map(|h| h.to_string_lossy().to_string())
            .unwrap_or_else(|_| "Unknown".into());

        let listen_addr: SocketAddr = "0.0.0.0:5000".parse()?;

        // ── Initialize QUIC Server ──
        let quic_server = QuicServer::new(listen_addr).await?;
        info!("QUIC server initialized on {}", listen_addr);

        // ── Initialize Discovery ──
        let discovery =
            DiscoveryService::new(device_id.clone(), device_name.clone(), listen_addr.port())?;

        // ── Initialize Network Monitor ──
        let network_monitor = Arc::new(NetworkMonitor::new());

        // ── Initialize Peer Manager ──
        let peer_manager = Arc::new(PeerManager::new(device_id.clone()));

        // ── Initialize Chunk Storage ──
        let chunk_storage = Arc::new(ChunkStorage::with_path(std::path::PathBuf::from(temp_path)));

        let (shutdown_tx, _) = broadcast::channel(1);

        let state = Arc::new(AppState {
            device_id,
            device_name,
            listen_addr,
            nearby_devices: Arc::new(RwLock::new(Vec::new())),
            network_monitor,
            peer_manager,
            chunk_storage,
            download_path,
        });

        Ok(Self {
            state,
            quic_server,
            discovery,
            shutdown_tx,
        })
    }

    /// Returns the device ID of this node.
    pub fn device_id(&self) -> &str {
        &self.state.device_id
    }

    /// Returns the QUIC server listen address.
    pub fn listen_addr(&self) -> SocketAddr {
        self.state.listen_addr
    }

    /// Run the application. Spawns all subsystems concurrently:
    ///
    /// 1. QUIC server listener (accepts connections + streams)
    /// 2. Discovery broadcaster + listener
    /// 3. Network monitor (background metrics collection)
    /// 4. Ctrl-C handler for graceful shutdown
    pub async fn run(&self) -> Result<()> {
        let mut shutdown_rx = self.shutdown_tx.subscribe();
        let shutdown_tx = self.shutdown_tx.clone();

        // ── Spawn QUIC server accept loop ──
        let server = self.quic_server.clone();
        let state = self.state.clone();
        let quic_handle = tokio::spawn(async move {
            if let Err(e) = server.accept_loop(state).await {
                tracing::error!("QUIC server error: {}", e);
            }
        });

        // ── Spawn Discovery ──
        let discovery_state = self.state.clone();
        let discovery = self.discovery.clone();
        let discovery_handle = tokio::spawn(async move {
            if let Err(e) = discovery.run(discovery_state).await {
                tracing::error!("Discovery error: {}", e);
            }
        });

        // ── Spawn Network Monitor ──
        let monitor = self.state.network_monitor.clone();
        let monitor_handle = tokio::spawn(async move {
            monitor.run().await;
        });

        // ── Wait for Ctrl-C ──
        info!("FastShare is running. Press Ctrl+C to stop.");
        tokio::select! {
            _ = tokio::signal::ctrl_c() => {
                info!("Shutdown signal received...");
                let _ = shutdown_tx.send(());
            }
            _ = shutdown_rx.recv() => {
                info!("Shutdown broadcast received...");
            }
        }

        // ── Cleanup ──
        quic_handle.abort();
        discovery_handle.abort();
        monitor_handle.abort();

        Ok(())
    }
}
