//! FastShare — Ultra-High-Performance P2P File Transfer
//!
//! Entry point that initializes the Tokio async runtime, sets up logging,
//! discovery services, the QUIC server, and the transfer engine.

use anyhow::Result;
use std::thread;
use tracing::{info, Level};
use tracing_subscriber::EnvFilter;

use fastshare::app::{self, App};
use fastshare::ui::{gui, gui_bridge};

/// Main entry point. Starts the GUI and the FastShare background subsystems.
fn main() -> Result<()> {
    // ── Initialize Logging ──
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive(Level::INFO.into()))
        .with_target(true)
        .with_thread_ids(true)
        .init();

    info!("╔══════════════════════════════════════════════════╗");
    info!("║        FastShare v0.1.0 — P2P File Transfer      ║");
    info!("║   Ultra-High-Performance Distributed Transfer    ║");
    info!("╚══════════════════════════════════════════════════╝");

    let download_path: String = dirs::download_dir()
        .or_else(dirs::document_dir)
        .or_else(dirs::home_dir)
        .map(|p| p.join("FastShare").to_string_lossy().into_owned())
        .unwrap_or_else(|| ".".to_string());
    let temp_path = dirs::data_local_dir()
        .or_else(dirs::home_dir)
        .map(|p| p.join("FastShare").join("temp").to_string_lossy().into_owned())
        .unwrap_or_else(|| std::env::temp_dir().to_string_lossy().into_owned());

    let (send_tx, send_rx) = tokio::sync::mpsc::channel(32);

    // Spawn the Tokio runtime in a background thread
    thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime");
        rt.block_on(async {
            tokio::time::sleep(std::time::Duration::from_millis(500)).await;

            info!("Starting FastShare backend...");

            match App::new(download_path, temp_path).await {
                Ok(app) => {
                    info!("Device ID: {}", app.device_id());
                    info!("Listening on: {}", app.listen_addr());

                    gui_bridge::set_bridge(app.state.clone(), send_tx.clone());
                    let quic_server = app.quic_server.clone();
                    tokio::spawn(app::run_send_loop(send_rx, quic_server, app.state.clone()));

                    if let Err(e) = app.run().await {
                        tracing::error!("FastShare app error: {}", e);
                    }
                }
                Err(e) => {
                    tracing::error!("Failed to initialize FastShare app: {}", e);
                }
            }
        });
    });

    info!("Starting Dioxus GUI...");
    dioxus::launch(gui::app);

    info!("FastShare shut down gracefully.");
    Ok(())
}
