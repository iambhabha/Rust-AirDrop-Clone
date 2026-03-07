//! # Transfer History Screen
//!
//! Displays completed and failed transfers with timestamps
//! and transfer statistics.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// A completed transfer record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferRecord {
    /// File name transferred
    pub file_name: String,
    /// File size
    pub file_size: u64,
    /// Direction: "sent" or "received"
    pub direction: String,
    /// Remote device name
    pub remote_device: String,
    /// Transfer completion time
    pub timestamp: DateTime<Utc>,
    /// Transfer duration in seconds
    pub duration_secs: f64,
    /// Average throughput in bytes/sec
    pub throughput_bps: u64,
    /// Whether the transfer succeeded
    pub success: bool,
    /// Error message if failed
    pub error: Option<String>,
}

/// Transfer history screen.
pub struct HistoryScreen {
    /// List of transfer records
    pub records: Vec<TransferRecord>,
}

impl HistoryScreen {
    /// Create a new history screen.
    pub fn new() -> Self {
        Self {
            records: Vec::new(),
        }
    }

    /// Add a transfer record.
    pub fn add_record(&mut self, record: TransferRecord) {
        self.records.push(record);
    }

    /// Render the history screen.
    pub fn render(&self) {
        println!("╔══════════════════════════════════════════════════════════╗");
        println!("║                   📋 Transfer History                   ║");
        println!("╠══════════════════════════════════════════════════════════╣");

        if self.records.is_empty() {
            println!("║  No transfer history.                                    ║");
        } else {
            for record in self.records.iter().rev().take(10) {
                let icon = match (record.direction.as_str(), record.success) {
                    ("sent", true) => "📤✅",
                    ("sent", false) => "📤❌",
                    ("received", true) => "📥✅",
                    ("received", false) => "📥❌",
                    _ => "📎",
                };

                println!(
                    "║  {} {} ({}) → {} [{:.1} MB/s]    ║",
                    icon,
                    truncate(&record.file_name, 20),
                    format_size(record.file_size),
                    truncate(&record.remote_device, 12),
                    record.throughput_bps as f64 / (1024.0 * 1024.0),
                );
            }
        }

        println!("╠══════════════════════════════════════════════════════════╣");
        println!("║  [C] Clear History    [B] Back                          ║");
        println!("╚══════════════════════════════════════════════════════════╝");
    }

    /// Clear all history.
    pub fn clear(&mut self) {
        self.records.clear();
    }
}

fn format_size(bytes: u64) -> String {
    const MB: u64 = 1024 * 1024;
    const GB: u64 = MB * 1024;
    if bytes >= GB {
        format!("{:.1}GB", bytes as f64 / GB as f64)
    } else {
        format!("{:.1}MB", bytes as f64 / MB as f64)
    }
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        format!("{:<width$}", s, width = max)
    } else {
        format!("{:.width$}…", s, width = max - 1)
    }
}
