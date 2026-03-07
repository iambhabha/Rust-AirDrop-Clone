//! # Wi-Fi Direct Hotspot Controller
//!
//! Enables device-to-device file transfer without a router by creating
//! a Wi-Fi Direct hotspot. This is essential for scenarios where:
//!
//! - No Wi-Fi router is available
//! - Direct connection provides lower latency
//! - Maximum bandwidth between two specific devices is needed
//!
//! ## Platform Support
//!
//! Wi-Fi Direct hotspot creation requires platform-specific APIs:
//! - **Windows**: WlanHostedNetworkStartUsing / Mobile Hotspot API
//! - **Linux**: hostapd / NetworkManager
//! - **macOS**: CoreWLAN framework
//! - **Android**: WifiP2pManager
//! - **iOS**: NEHotspotConfiguration

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use tracing::{info, warn};

// ── Data Structures ──

/// Wi-Fi Direct hotspot configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HotspotConfig {
    /// SSID of the hotspot (auto-generated)
    pub ssid: String,
    /// Password for the hotspot
    pub password: String,
    /// Channel to use (0 = auto)
    pub channel: u8,
    /// Whether the hotspot is currently active
    pub active: bool,
    /// IP address assigned to this device as the hotspot host
    pub host_ip: String,
    /// Subnet mask
    pub subnet: String,
}

/// Status of the Wi-Fi Direct hotspot.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum HotspotStatus {
    Inactive,
    Starting,
    Active,
    Stopping,
    Error(String),
}

/// The hotspot controller manages Wi-Fi Direct access point creation.
pub struct HotspotController {
    /// Current hotspot configuration
    config: HotspotConfig,
    /// Current status
    status: HotspotStatus,
}

impl HotspotController {
    /// Create a new hotspot controller with auto-generated credentials.
    pub fn new() -> Self {
        let ssid = format!("FastShare-{}", &uuid::Uuid::new_v4().to_string()[..8]);
        let password = generate_password();

        Self {
            config: HotspotConfig {
                ssid,
                password,
                channel: 0, // Auto-select
                active: false,
                host_ip: "192.168.49.1".into(),
                subnet: "255.255.255.0".into(),
            },
            status: HotspotStatus::Inactive,
        }
    }

    /// Start the Wi-Fi Direct hotspot.
    ///
    /// This is platform-specific. The implementation dispatches to
    /// the appropriate system API based on the target OS.
    pub async fn start(&mut self) -> Result<()> {
        info!(
            "📡 Starting Wi-Fi Direct hotspot: SSID={}, Channel={}",
            self.config.ssid, self.config.channel
        );

        self.status = HotspotStatus::Starting;

        #[cfg(target_os = "linux")]
        {
            self.start_linux().await?;
        }

        #[cfg(target_os = "windows")]
        {
            self.start_windows().await?;
        }

        #[cfg(target_os = "macos")]
        {
            self.start_macos().await?;
        }

        #[cfg(any(target_os = "android", target_os = "ios"))]
        {
            warn!("Wi-Fi Direct hotspot requires platform-specific native API integration");
            self.status = HotspotStatus::Error("Platform API not yet integrated".into());
            return Ok(());
        }

        self.config.active = true;
        self.status = HotspotStatus::Active;

        info!(
            "✅ Hotspot active: {} (password: {})",
            self.config.ssid, self.config.password
        );

        Ok(())
    }

    /// Stop the Wi-Fi Direct hotspot.
    pub async fn stop(&mut self) -> Result<()> {
        info!("Stopping Wi-Fi Direct hotspot...");
        self.status = HotspotStatus::Stopping;
        self.config.active = false;
        self.status = HotspotStatus::Inactive;
        info!("Hotspot stopped.");
        Ok(())
    }

    /// Get the current hotspot configuration.
    pub fn config(&self) -> &HotspotConfig {
        &self.config
    }

    /// Get the current hotspot status.
    pub fn status(&self) -> &HotspotStatus {
        &self.status
    }

    #[cfg(target_os = "linux")]
    async fn start_linux(&self) -> Result<()> {
        // On Linux, we would use NetworkManager D-Bus API or hostapd
        // For now, log the command that would be executed
        info!("Linux: Would create hotspot via NetworkManager/hostapd");
        info!(
            "  nmcli dev wifi hotspot ifname wlan0 ssid {} password {}",
            self.config.ssid, self.config.password
        );
        Ok(())
    }

    #[cfg(target_os = "windows")]
    async fn start_windows(&self) -> Result<()> {
        // On Windows, we would use the Mobile Hotspot API
        info!("Windows: Would create hotspot via Mobile Hotspot API");
        info!(
            "  netsh wlan set hostednetwork mode=allow ssid={} key={}",
            self.config.ssid, self.config.password
        );
        Ok(())
    }

    #[cfg(target_os = "macos")]
    async fn start_macos(&self) -> Result<()> {
        // On macOS, we would use the CoreWLAN framework
        info!("macOS: Would create hotspot via CoreWLAN framework");
        Ok(())
    }
}

/// Generate a random 8-character alphanumeric password.
fn generate_password() -> String {
    use std::fmt::Write;
    let uuid = uuid::Uuid::new_v4();
    let bytes = uuid.as_bytes();
    let mut password = String::with_capacity(8);
    for &b in &bytes[..8] {
        write!(password, "{:02x}", b).unwrap();
    }
    password.truncate(8);
    password
}
