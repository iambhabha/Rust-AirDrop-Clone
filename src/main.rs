//! FastShare — Ultra-High-Performance P2P File Transfer
//!
//! Entry point that initializes the Tokio async runtime, sets up logging,
//! discovery services, the QUIC server, and the transfer engine.

use anyhow::Result;
use std::thread;
use tracing::{info, Level};
use tracing_subscriber::{fmt, EnvFilter};

use fastshare::app::App;
use fastshare::ui::gui;

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

    // Spawn the Tokio runtime in a background thread
    // This is necessary because Dioxus desktop event loop needs to run on the main thread.
    thread::spawn(|| {
        let rt = tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime");
        rt.block_on(async {
            // Wait a brief moment for the GUI to start
            tokio::time::sleep(std::time::Duration::from_millis(500)).await;

            info!("Starting FastShare backend...");

            // ── Create the application ──
            match App::new().await {
                Ok(app) => {
                    info!("Device ID: {}", app.device_id());
                    info!("Listening on: {}", app.listen_addr());

                    // ── Run the application ──
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

    // ── Launch Dioxus GUI (Blocks main thread) ──
    info!("Starting Dioxus GUI...");
    dioxus::launch(gui::app);

    info!("FastShare shut down gracefully.");
    Ok(())
}
