use fastshare::app::{App, AppState};
use fastshare::network::connection::QuicServer;
use fastshare::transfer::chunker::NetworkSpeed;
use fastshare::transfer::sender::TransferSender;
use std::net::SocketAddr;
use std::path::Path;
use std::sync::{Arc, Mutex};
use tokio::runtime::Runtime;

lazy_static::lazy_static! {
    static ref RUNTIME: Runtime = tokio::runtime::Runtime::new().unwrap();
    static ref GLOBAL_SERVER: Mutex<Option<QuicServer>> = Mutex::new(None);
    static ref GLOBAL_APP_STATE: Mutex<Option<Arc<AppState>>> = Mutex::new(None);
}

#[flutter_rust_bridge::frb(init)]
pub fn init_app() {
    flutter_rust_bridge::setup_default_user_utils();
}

/// Start the background FastShare server and return some details
pub fn start_fastshare(download_path: String, temp_path: String) -> String {
    let _ = RUNTIME.block_on(async {
        match App::new(download_path, temp_path).await {
            Ok(app) => {
                let id = app.device_id().to_string();
                let addr = app.listen_addr().to_string();

                // Save Global state
                *GLOBAL_SERVER.lock().unwrap() = Some(app.quic_server.clone());
                *GLOBAL_APP_STATE.lock().unwrap() = Some(app.state.clone());

                tokio::spawn(async move {
                    let _ = app.run().await;
                });

                format!("Started FastShare as {}\nListening on {}", id, addr)
            }
            Err(e) => format!("Failed: {}", e),
        }
    });
    "FastShare Backend Started!".into()
}

/// Send a file to a target IP
pub fn send_file_to_ip(file_path: String, target_ip: String) -> String {
    let server_opt = GLOBAL_SERVER.lock().unwrap().clone();
    if let Some(server) = server_opt {
        RUNTIME.block_on(async {
            // Append the default port if not provided
            let mut addr_str = target_ip.clone();
            if !addr_str.contains(":") {
                addr_str = format!("{}:5000", addr_str);
            }

            let target_addr: SocketAddr = match addr_str.parse() {
                Ok(a) => a,
                Err(e) => return format!("Invalid IP address format: {}", e),
            };

            // Connect to peer
            let connection = match server.connect_to_peer(target_addr).await {
                Ok(c) => c,
                Err(e) => return format!("Failed to connect to peer: {}", e),
            };

            // Send file
            let sender = TransferSender::new();
            match sender
                .send_file(
                    &connection,
                    Path::new(&file_path),
                    NetworkSpeed::Fast,
                    None,
                    None,
                )
                .await
            {
                Ok(_) => format!("Success! File {} sent to {}", file_path, target_ip),
                Err(e) => format!("Transfer failed: {}", e),
            }
        })
    } else {
        "Please Start Rust Engine first!".into()
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
