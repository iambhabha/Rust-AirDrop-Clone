//! # Home Screen
//!
//! Main dashboard showing device status, nearby devices count,
//! and quick actions for sending/receiving files.

use crate::app::AppState;
use std::sync::Arc;

/// Home screen state for the UI.
#[derive(Debug, Clone)]
pub struct HomeScreen {
    /// Current device name
    pub device_name: String,
    /// Current device ID (truncated for display)
    pub device_id_short: String,
    /// Number of nearby discovered devices
    pub nearby_count: usize,
    /// Number of active transfers
    pub active_transfers: usize,
    /// Current network status
    pub network_status: String,
    /// Whether the QUIC server is running
    pub server_running: bool,
}

impl HomeScreen {
    /// Create the home screen state from application state.
    pub async fn from_state(state: &Arc<AppState>) -> Self {
        let devices = state.nearby_devices.read().await;
        let metrics = state.network_monitor.get_metrics().await;

        let network_status = if metrics.throughput_bps > 0 {
            format!(
                "{:.1} MB/s | {:.0}ms latency",
                metrics.throughput_bps as f64 / (1024.0 * 1024.0),
                metrics.latency_ms
            )
        } else {
            "Idle".into()
        };

        Self {
            device_name: state.device_name.clone(),
            device_id_short: state.device_id[..8].to_string(),
            nearby_count: devices.len(),
            active_transfers: 0,
            network_status,
            server_running: true,
        }
    }

    /// Render the home screen to the terminal.
    pub fn render(&self) {
        println!("╔══════════════════════════════════════════╗");
        println!("║            ⚡ FastShare                  ║");
        println!("╠══════════════════════════════════════════╣");
        println!("║ Device: {:<32} ║", self.device_name);
        println!("║ ID: {:<36} ║", self.device_id_short);
        println!(
            "║ Server: {:<32} ║",
            if self.server_running {
                "✅ Running"
            } else {
                "❌ Stopped"
            }
        );
        println!("║ Network: {:<31} ║", self.network_status);
        println!("║ Nearby Devices: {:<24} ║", self.nearby_count);
        println!("║ Active Transfers: {:<22} ║", self.active_transfers);
        println!("╠══════════════════════════════════════════╣");
        println!("║  [S] Send File    [D] View Devices       ║");
        println!("║  [H] History      [Q] Quit               ║");
        println!("╚══════════════════════════════════════════╝");
    }
}
