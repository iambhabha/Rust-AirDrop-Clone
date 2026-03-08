//! # Application Root
//!
//! Orchestrates all FastShare subsystems: discovery, QUIC server,
//! transfer engine, network optimizer, and the UI event loop.

use anyhow::Result;
use dashmap::DashMap;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::{broadcast, mpsc, oneshot, RwLock};
use tracing::info;
use uuid::Uuid;

use crate::distributed::peer_manager::PeerManager;
use crate::network::connection::QuicServer;
use crate::network::discovery::{DeviceInfo, DiscoveryService};
use crate::optimizer::network_monitor::NetworkMonitor;
use crate::storage::chunk_storage::ChunkStorage;
use crate::transfer::chunker::NetworkSpeed;
use crate::transfer::receiver::TransferReceiver;
use crate::transfer::sender::{TransferProgress, TransferSender};

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
    /// Registry for pending transfer Accept/Decline: file_id -> oneshot sender. Backend waits on receiver; GUI sends via this.
    pub pending_decisions: Arc<DashMap<String, oneshot::Sender<bool>>>,
    /// Current incoming transfer to show in UI: (file_id, from_addr, file_name). GUI reads this and responds via pending_decisions.
    /// Current incoming transfer to show in UI: (file_id, from_addr, file_name, total_files).
    pub pending_incoming_display: Arc<std::sync::Mutex<Option<(String, SocketAddr, String, u32)>>>,
    /// Current outgoing transfer progress for UI (file list, size, progress bar).
    pub transfer_progress: Arc<std::sync::Mutex<Option<crate::transfer::sender::TransferProgress>>>,
    /// Receiver for management of incoming transfers
    pub transfer_receiver: Arc<TransferReceiver>,
    /// History of completed transfers: (file_name, size, result, timestamp, is_incoming)
    pub transfer_history: Arc<std::sync::Mutex<Vec<TransferHistoryItem>>>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct TransferHistoryItem {
    pub file_name: String,
    pub size: u64,
    pub status: String, // "Success", "Failed", "Declined"
    pub timestamp: String,
    pub is_incoming: bool,
    pub saved_path: Option<String>,
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

        // ── Initialize Transfer Receiver ──
        let transfer_receiver = Arc::new(TransferReceiver::new(chunk_storage.clone()));

        let (shutdown_tx, _) = broadcast::channel(1);

        let history = Self::load_history(&download_path);

        let state = Arc::new(AppState {
            device_id,
            device_name,
            listen_addr,
            nearby_devices: Arc::new(RwLock::new(Vec::new())),
            network_monitor,
            peer_manager,
            chunk_storage,
            download_path,
            pending_decisions: Arc::new(DashMap::new()),
            pending_incoming_display: Arc::new(std::sync::Mutex::new(None)),
            transfer_progress: Arc::new(std::sync::Mutex::new(None)),
            transfer_receiver,
            transfer_history: Arc::new(std::sync::Mutex::new(history)),
        });

        Ok(Self {
            state,
            quic_server,
            discovery,
            shutdown_tx,
        })
    }

    fn get_history_path(base_path: &str) -> PathBuf {
        PathBuf::from(base_path).join("history.json")
    }

    fn load_history(download_path: &str) -> Vec<TransferHistoryItem> {
        let path = Self::get_history_path(download_path);
        if let Ok(data) = std::fs::read_to_string(path) {
            serde_json::from_str(&data).unwrap_or_default()
        } else {
            Vec::new()
        }
    }

    pub fn save_history(state: &AppState) {
        let path = Self::get_history_path(&state.download_path);
        if let Ok(history) = state.transfer_history.lock() {
            if let Ok(json) = serde_json::to_string_pretty(&*history) {
                let _ = std::fs::write(path, json);
            }
        }
    }

    /// Returns the device ID of this node.
    pub fn device_id(&self) -> &str {
        &self.state.device_id
    }

    /// Returns the QUIC server listen address.
    pub fn listen_addr(&self) -> SocketAddr {
        self.state.listen_addr
    }

    /// Explicitly trigger a network scan for nearby devices.
    pub async fn trigger_discovery_scan(&self) -> Result<()> {
        self.discovery.trigger_scan().await
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

/// Max retries for connecting to peer (e.g. phone may need a moment to accept).
const CONNECT_RETRIES: u32 = 3;
/// Delay between retries.
const CONNECT_RETRY_DELAY: std::time::Duration = std::time::Duration::from_secs(2);
/// Connection attempt timeout.
const CONNECT_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(15);

/// Runs a loop that receives send-file requests from the GUI and performs the transfer.
/// Call this in the same runtime as `App::run` (e.g. spawn it before `app.run().await`).
pub async fn run_send_loop(
    mut send_rx: mpsc::Receiver<(PathBuf, SocketAddr)>,
    quic_server: QuicServer,
    state: Arc<AppState>,
) {
    let sender = TransferSender::new();
    while let Some((file_path, peer_addr)) = send_rx.recv().await {
        if !file_path.exists() {
            tracing::error!("File not found: {}", file_path.display());
            continue;
        }

        if let Ok(mut guard) = state.transfer_progress.lock() {
            *guard = Some(TransferProgress {
                file_name: format!("Connecting to {}...", peer_addr.ip()),
                file_id: String::new(),
                total_bytes: 0,
                bytes_sent: 0,
                chunks_sent: 0,
                total_chunks: 1,
                current_file_index: 1,
                total_files: 1,
                throughput_bps: 0,
                eta_seconds: 0.0,
                complete: false,
            });
        }

        let mut last_err = None;
        for attempt in 1..=CONNECT_RETRIES {
            let connect_fut = quic_server.connect_and_handshake(peer_addr, state.clone());
            match tokio::time::timeout(CONNECT_TIMEOUT, connect_fut).await {
                Ok(Ok(connection)) => {
                    let state_for_cb = state.clone();
                    let progress_cb = Some(Box::new(move |p: TransferProgress| {
                        if let Ok(mut guard) = state_for_cb.transfer_progress.lock() {
                            *guard = Some(p);
                        }
                    })
                        as Box<dyn Fn(TransferProgress) + Send + Sync>);
                    if let Err(e) = sender
                        .send_file(
                            &connection,
                            &file_path,
                            NetworkSpeed::Normal,
                            1,
                            1,
                            None,
                            progress_cb,
                        )
                        .await
                    {
                        tracing::error!("Send file failed: {}", e);
                        let mut history = state.transfer_history.lock().unwrap();
                        history.push(TransferHistoryItem {
                            file_name: file_path.to_string_lossy().to_string(),
                            size: 0,
                            status: format!("Failed: {}", e),
                            timestamp: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
                            is_incoming: false,
                            saved_path: Some(file_path.to_string_lossy().to_string()),
                        });
                        App::save_history(&state);
                    } else {
                        info!("✅ File sent successfully: {}", file_path.display());
                        let mut history = state.transfer_history.lock().unwrap();
                        history.push(TransferHistoryItem {
                            file_name: file_path.to_string_lossy().to_string(),
                            size: 0,
                            status: "Success".into(),
                            timestamp: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
                            is_incoming: false,
                            saved_path: Some(file_path.to_string_lossy().to_string()),
                        });
                        App::save_history(&state);
                    }
                    if let Ok(mut guard) = state.transfer_progress.lock() {
                        *guard = None;
                    }
                    last_err = None;
                    break;
                }
                Ok(Err(e)) => {
                    last_err = Some(e);
                    if attempt < CONNECT_RETRIES {
                        tracing::warn!(
                            "Connect to {} failed (attempt {}/{}), retrying in {:?}...",
                            peer_addr,
                            attempt,
                            CONNECT_RETRIES,
                            CONNECT_RETRY_DELAY
                        );
                        tokio::time::sleep(CONNECT_RETRY_DELAY).await;
                    }
                }
                Err(_) => {
                    last_err = Some(anyhow::anyhow!(
                        "Connection timeout after {:?}",
                        CONNECT_TIMEOUT
                    ));
                    if attempt < CONNECT_RETRIES {
                        tracing::warn!(
                            "Connect to {} timed out (attempt {}/{}), retrying...",
                            peer_addr,
                            attempt,
                            CONNECT_RETRIES
                        );
                        tokio::time::sleep(CONNECT_RETRY_DELAY).await;
                    }
                }
            }
        }
        if let Some(e) = last_err {
            tracing::error!(
                "Connect to {} failed after {} attempts: {}",
                peer_addr,
                CONNECT_RETRIES,
                e
            );
            crate::ui::gui_bridge::set_backend_status(format!(
                "Failed to connect to {}: {}",
                peer_addr.ip(),
                e
            ));
            if let Ok(mut guard) = state.transfer_progress.lock() {
                *guard = None;
            }
        }
    }
}
