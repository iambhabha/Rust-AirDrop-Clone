//! # Nearby Devices Screen
//!
//! Displays discovered devices with their metadata and status.

use crate::network::discovery::DeviceInfo;

/// Devices screen for the UI.
pub struct DevicesScreen {
    /// List of discovered devices
    pub devices: Vec<DeviceInfo>,
}

impl DevicesScreen {
    /// Create a new devices screen.
    pub fn new(devices: Vec<DeviceInfo>) -> Self {
        Self { devices }
    }

    /// Render the devices list to the terminal.
    pub fn render(&self) {
        println!("╔══════════════════════════════════════════════════════╗");
        println!("║                📡 Nearby Devices                    ║");
        println!("╠══════════════════════════════════════════════════════╣");

        if self.devices.is_empty() {
            println!("║  No devices found. Scanning...                      ║");
        } else {
            for (i, device) in self.devices.iter().enumerate() {
                let icon = match device.device_type.as_str() {
                    "phone" => "📱",
                    "tablet" => "📱",
                    "laptop" => "💻",
                    "desktop" => "🖥️",
                    _ => "📟",
                };
                println!(
                    "║  [{}] {} {} ({}) - {}:{} [{:<6}] ║",
                    i + 1,
                    icon,
                    truncate_string(&device.device_name, 15),
                    device.device_type,
                    device.ip_address,
                    device.port,
                    device.max_bandwidth
                );
            }
        }

        println!("╠══════════════════════════════════════════════════════╣");
        println!("║  [number] Select device  [R] Refresh  [B] Back      ║");
        println!("╚══════════════════════════════════════════════════════╝");
    }

    /// Get a device by its display index (1-based).
    pub fn get_device(&self, index: usize) -> Option<&DeviceInfo> {
        if index > 0 && index <= self.devices.len() {
            Some(&self.devices[index - 1])
        } else {
            None
        }
    }
}

/// Truncate a string to a maximum length with ellipsis.
fn truncate_string(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        format!("{:<width$}", s, width = max_len)
    } else {
        format!("{:.width$}...", s, width = max_len - 3)
    }
}
