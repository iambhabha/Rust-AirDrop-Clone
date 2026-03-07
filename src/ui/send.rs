//! # Send File Screen
//!
//! Interface for selecting and sending files to nearby devices.

use crate::network::discovery::DeviceInfo;
use std::path::PathBuf;

/// Send file screen state.
pub struct SendScreen {
    /// Selected file path
    pub file_path: Option<PathBuf>,
    /// Selected file name
    pub file_name: String,
    /// Selected file size
    pub file_size: u64,
    /// Target device
    pub target_device: Option<DeviceInfo>,
    /// Whether compression is enabled
    pub compression_enabled: bool,
    /// Number of parallel streams
    pub parallel_streams: usize,
}

impl SendScreen {
    /// Create a new send screen.
    pub fn new() -> Self {
        Self {
            file_path: None,
            file_name: String::new(),
            file_size: 0,
            target_device: None,
            compression_enabled: true,
            parallel_streams: 8,
        }
    }

    /// Set the file to send.
    pub fn set_file(&mut self, path: PathBuf, name: String, size: u64) {
        self.file_path = Some(path);
        self.file_name = name;
        self.file_size = size;
    }

    /// Set the target device.
    pub fn set_target(&mut self, device: DeviceInfo) {
        self.target_device = Some(device);
    }

    /// Render the send screen.
    pub fn render(&self) {
        println!("╔══════════════════════════════════════════╗");
        println!("║              📤 Send File                ║");
        println!("╠══════════════════════════════════════════╣");

        if let Some(ref path) = self.file_path {
            println!("║ File: {:<34} ║", self.file_name);
            println!("║ Size: {:<34} ║", format_size(self.file_size));
            println!("║ Path: {:<34} ║", truncate_path(path, 34));
        } else {
            println!("║ No file selected                          ║");
        }

        println!("║                                            ║");

        if let Some(ref device) = self.target_device {
            println!("║ To: {:<36} ║", device.device_name);
            println!(
                "║ IP: {:<36} ║",
                format!("{}:{}", device.ip_address, device.port)
            );
        } else {
            println!("║ No target device selected                  ║");
        }

        println!("║                                            ║");
        println!(
            "║ Compression: {:<27} ║",
            if self.compression_enabled {
                "Enabled (LZ4)"
            } else {
                "Disabled"
            }
        );
        println!("║ Streams: {:<31} ║", self.parallel_streams);
        println!("╠══════════════════════════════════════════╣");
        println!("║  [Enter] Start Transfer  [B] Back         ║");
        println!("╚══════════════════════════════════════════╝");
    }
}

/// Format a byte size into human-readable form.
fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;
    const TB: u64 = GB * 1024;

    if bytes >= TB {
        format!("{:.2} TB", bytes as f64 / TB as f64)
    } else if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

/// Truncate a path for display.
fn truncate_path(path: &std::path::Path, max_len: usize) -> String {
    let s = path.to_string_lossy().to_string();
    if s.len() <= max_len {
        s
    } else {
        format!("...{}", &s[s.len() - max_len + 3..])
    }
}
