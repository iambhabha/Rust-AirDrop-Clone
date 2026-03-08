use chrono;
use fastshare::app::{App, AppState};
use fastshare::network::connection::QuicServer;
use fastshare::transfer::chunker::NetworkSpeed;
use fastshare::transfer::sender::{TransferProgress, TransferSender};
use serde::Serialize;
use std::net::SocketAddr;
use std::path::Path;
use std::sync::{Arc, Mutex};
use tokio::runtime::Runtime;

lazy_static::lazy_static! {
    static ref RUNTIME: Runtime = tokio::runtime::Runtime::new().unwrap();
    static ref GLOBAL_SERVER: Mutex<Option<QuicServer>> = Mutex::new(None);
    static ref GLOBAL_TRANSFER_PROGRESS: Mutex<Option<TransferProgress>> = Mutex::new(None);
    static ref GLOBAL_APP_STATE: Mutex<Option<Arc<AppState>>> = Mutex::new(None);
    static ref GLOBAL_DISCOVERY: Mutex<Option<fastshare::network::discovery::DiscoveryService>> = Mutex::new(None);
}

#[flutter_rust_bridge::frb(init)]
pub fn init_app() {
    // Initialize logging based on platform
    #[cfg(target_os = "android")]
    {
        android_logger::init_once(
            android_logger::Config::default()
                .with_max_level(tracing::log::LevelFilter::Info)
                .with_tag("fastshare_rust"),
        );
    }
    #[cfg(not(target_os = "android"))]
    {
        let _ = tracing_subscriber::fmt()
            .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
            .with_target(true)
            .try_init();
    }

    // Suppress verbose dependency logs
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var(
            "RUST_LOG",
            "fastshare=info,mdns_sd=error,polling=error,quinn=error,rustls=error",
        );
    }

    flutter_rust_bridge::setup_default_user_utils();
    tracing::info!("🚀 [FastShare] Rust Engine Initialized");
}

/// Start the background FastShare server and return some details
pub fn start_fastshare() -> String {
    let download_path = dirs::download_dir()
        .or_else(dirs::document_dir)
        .or_else(dirs::home_dir)
        .map(|p| p.join("FastShare").to_string_lossy().into_owned())
        .unwrap_or_else(|| ".".to_string());

    let temp_path = dirs::data_local_dir()
        .or_else(dirs::home_dir)
        .map(|p| {
            p.join("FastShare")
                .join("temp")
                .to_string_lossy()
                .into_owned()
        })
        .unwrap_or_else(|| std::env::temp_dir().to_string_lossy().into_owned());

    let result = RUNTIME.block_on(async {
        match App::new(download_path, temp_path).await {
            Ok(app) => {
                let id = app.device_id().to_string();
                let _addr = app.listen_addr();
                let local_ip = local_ip_address::list_afinet_netifas()
                    .ok()
                    .and_then(|ifs| {
                        ifs.into_iter()
                            .find(|(_, ip)| !ip.is_loopback() && ip.is_ipv4())
                            .map(|(_, ip)| ip)
                    })
                    .unwrap_or(std::net::IpAddr::V4(std::net::Ipv4Addr::new(127, 0, 0, 1)));

                // Save Global state
                *GLOBAL_SERVER.lock().unwrap() = Some(app.quic_server.clone());
                *GLOBAL_APP_STATE.lock().unwrap() = Some(app.state.clone());
                *GLOBAL_DISCOVERY.lock().unwrap() = Some(app.discovery.clone());

                tokio::spawn(async move {
                    let _ = app.run().await;
                });

                tracing::info!(
                    "📥 [FastShare] QUIC server listening on 0.0.0.0:5000 — ready to receive files"
                );
                format!("Online: {}\nID: {}", local_ip, id)
            }
            Err(e) => format!("Failed: {}", e),
        }
    });
    result
}

/// Send multiple files to a target IP
pub fn send_files_to_ip(file_paths: Vec<String>, target_ip: String) -> String {
    let server_opt = GLOBAL_SERVER.lock().unwrap().clone();
    if let Some(server) = server_opt {
        *GLOBAL_TRANSFER_PROGRESS.lock().unwrap() = None;
        let progress_cb: Arc<dyn Fn(TransferProgress) + Send + Sync> = Arc::new(|p| {
            if let Ok(mut g) = GLOBAL_TRANSFER_PROGRESS.lock() {
                *g = Some(p);
            }
        });
        let result = RUNTIME.block_on(do_send_files(server, file_paths, target_ip, progress_cb));
        *GLOBAL_TRANSFER_PROGRESS.lock().unwrap() = None;
        result
    } else {
        "Please Start Rust Engine first!".into()
    }
}

async fn do_send_files(
    server: QuicServer,
    file_paths: Vec<String>,
    target_ip: String,
    progress_cb: Arc<dyn Fn(TransferProgress) + Send + Sync>,
) -> String {
    let mut addr_str = target_ip;
    if !addr_str.contains(":") {
        addr_str = format!("{}:5000", addr_str);
    }
    let target_addr: SocketAddr = match addr_str.parse() {
        Ok(a) => a,
        Err(e) => return format!("Invalid IP address format: {}", e),
    };
    let state = match GLOBAL_APP_STATE.lock().unwrap().clone() {
        Some(s) => s,
        None => return "Engine not ready".into(),
    };
    let connection = match server
        .connect_and_handshake(target_addr, state.clone())
        .await
    {
        Ok(c) => {
            tracing::info!("📤 [FastShare] Handshake successful with {}", target_addr);
            c
        }
        Err(e) => {
            tracing::error!("❌ [FastShare] Connection failed to {}: {}", target_addr, e);
            return format!("Failed to connect and handshake: {}", e);
        }
    };
    let sender = TransferSender::new();
    let mut success_count = 0;
    let total = file_paths.len() as u32;
    for (idx, path) in file_paths.iter().enumerate() {
        let cb = progress_cb.clone();
        let progress_cb_opt = Some(Box::new(move |p: TransferProgress| cb(p))
            as Box<dyn Fn(TransferProgress) + Send + Sync>);
        tracing::info!(
            "📤 [FastShare] Sending file: {} ({}/{})",
            path,
            idx + 1,
            total
        );
        match sender
            .send_file(
                &connection,
                Path::new(path),
                NetworkSpeed::Fast,
                total,
                (idx + 1) as u32,
                None,
                progress_cb_opt,
            )
            .await
        {
            Ok(_) => {
                success_count += 1;
                // Add to history
                let mut history = state.transfer_history.lock().unwrap();
                history.push(fastshare::app::TransferHistoryItem {
                    file_name: path.clone(),
                    size: 0, // Should ideally get from plan
                    status: "Success".into(),
                    timestamp: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
                    is_incoming: false,
                });
            }
            Err(e) => {
                let err_msg = format!(
                    "Interrupted! Sent {}/{} files. Last error: {}",
                    success_count, total, e
                );
                // Add failure to history
                let mut history = state.transfer_history.lock().unwrap();
                history.push(fastshare::app::TransferHistoryItem {
                    file_name: path.clone(),
                    size: 0,
                    status: format!("Failed: {}", e),
                    timestamp: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
                    is_incoming: false,
                });
                return err_msg;
            }
        }
    }
    format!(
        "Success! Sent {}/{} files to {}",
        success_count, total, target_addr
    )
}

/// Get transfer history as JSON
pub fn get_transfer_history() -> String {
    let state_opt = GLOBAL_APP_STATE.lock().unwrap().clone();
    if let Some(state) = state_opt {
        let history = state.transfer_history.lock().unwrap();
        serde_json::to_string(&*history).unwrap_or_else(|_| "[]".into())
    } else {
        "[]".into()
    }
}

/// Trigger a network scan manually
pub fn trigger_discovery_scan() {
    let discovery_opt = GLOBAL_DISCOVERY.lock().unwrap().clone();
    if let Some(discovery) = discovery_opt {
        RUNTIME.block_on(async {
            let _ = discovery.trigger_scan().await;
        });
    }
}

/// Get a list of discovered nearby devices in JSON format
pub fn get_nearby_devices() -> String {
    let state_opt = GLOBAL_APP_STATE.lock().unwrap().clone();
    if let Some(state) = state_opt {
        RUNTIME.block_on(async {
            let devices = state.nearby_devices.read().await;
            serde_json::to_string(&*devices).unwrap_or_else(|_| "[]".into())
        })
    } else {
        "[]".into()
    }
}

/// Get pending incoming transfer for UI popup, or null JSON if none. Returns {"file_id","from_addr","file_name"} or "null".
pub fn get_pending_incoming() -> String {
    let state_opt = GLOBAL_APP_STATE.lock().unwrap().clone();
    if let Some(state) = state_opt {
        if let Ok(guard) = state.pending_incoming_display.lock() {
            if let Some((ref file_id, ref from_addr, ref file_name, total_files)) = *guard {
                tracing::info!(
                    "📥 [FastShare] Pending incoming: file={} from={} ({} files)",
                    file_name,
                    from_addr,
                    total_files
                );
                let obj = serde_json::json!({
                    "file_id": file_id,
                    "from_addr": format!("{}", from_addr),
                    "file_name": file_name,
                    "total_files": total_files,
                });
                return obj.to_string();
            }
        }
    }
    "null".to_string()
}

/// Respond to incoming transfer (Accept = true, Decline = false). Call from Flutter when user taps Accept/Decline.
pub fn respond_incoming(file_id: String, accept: bool) {
    tracing::info!(
        "📥 [FastShare] respond_incoming called for {} (Accept: {})",
        file_id,
        accept
    );
    if let Some(state) = GLOBAL_APP_STATE.lock().unwrap().clone() {
        if let Some((_, tx)) = state.pending_decisions.remove(&file_id) {
            let _ = tx.send(accept);
            tracing::info!(
                "📥 [FastShare] Sent decision {} to backend for {}",
                accept,
                file_id
            );
        } else {
            tracing::warn!(
                "⚠️ [FastShare] No pending decision found for file_id: {}",
                file_id
            );
        }
        if let Ok(mut guard) = state.pending_incoming_display.lock() {
            *guard = None;
        }
    }
}

/// Get progress of all active incoming transfers as JSON.
/// Returns a list of {"file_name", "progress", "total_bytes", "received_bytes"}
pub fn get_incoming_progress() -> String {
    let state_opt = GLOBAL_APP_STATE.lock().unwrap().clone();
    if let Some(state) = state_opt {
        let mut progress_list = Vec::new();
        for r in state.transfer_receiver.active_receptions().iter() {
            let s = r.value();
            let received_chunks = s.chunks_received.load(std::sync::atomic::Ordering::Relaxed);
            let progress = if s.plan.total_chunks > 0 {
                received_chunks as f64 / s.plan.total_chunks as f64
            } else {
                0.0
            };
            progress_list.push(serde_json::json!({
                "file_name": s.plan.file_name,
                "file_id": s.plan.file_id,
                "progress": progress,
                "total_bytes": s.plan.total_size,
                "received_bytes": received_chunks * s.plan.chunk_size, // Approximation
                "total_chunks": s.plan.total_chunks,
                "received_chunks": received_chunks,
            }));
        }
        serde_json::to_string(&progress_list).unwrap_or_else(|_| "[]".into())
    } else {
        "[]".into()
    }
}

#[derive(Serialize)]
struct TransferStatus {
    pub file_name: String,
    pub progress: f64,
    pub total_bytes: u64,
    pub bytes_sent: u64,
    pub complete: bool,
    pub throughput_bps: u64,
}

/// Get outgoing transfer progress as JSON
pub fn get_outgoing_progress() -> String {
    if let Ok(guard) = GLOBAL_TRANSFER_PROGRESS.lock() {
        if let Some(ref p) = *guard {
            return serde_json::to_string(p).unwrap_or_else(|_| "null".into());
        }
    }
    "null".into()
}
