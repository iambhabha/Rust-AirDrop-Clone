//! # Transfer Progress Screen
//!
//! Real-time transfer progress display with throughput, ETA,
//! and per-stream statistics.

use crate::transfer::sender::TransferProgress;

/// Transfer progress screen.
pub struct TransferScreen {
    /// Current transfer progress
    pub progress: Option<TransferProgress>,
}

impl TransferScreen {
    /// Create a new transfer screen.
    pub fn new() -> Self {
        Self { progress: None }
    }

    /// Update the progress.
    pub fn update(&mut self, progress: TransferProgress) {
        self.progress = Some(progress);
    }

    /// Render the transfer progress.
    pub fn render(&self) {
        println!("╔══════════════════════════════════════════╗");
        println!("║           📊 Transfer Progress           ║");
        println!("╠══════════════════════════════════════════╣");

        if let Some(ref p) = self.progress {
            let percent = if p.total_bytes > 0 {
                (p.bytes_sent as f64 / p.total_bytes as f64) * 100.0
            } else {
                0.0
            };

            let bar_width = 30;
            let filled = (percent / 100.0 * bar_width as f64) as usize;
            let bar: String = "█".repeat(filled) + &"░".repeat(bar_width - filled);

            println!("║ File: {:<34} ║", p.file_name);
            println!("║                                            ║");
            println!("║ [{bar}] {percent:.1}%       ║");
            println!("║                                            ║");
            println!(
                "║ Sent: {} / {}       ║",
                format_size(p.bytes_sent),
                format_size(p.total_bytes)
            );
            println!(
                "║ Chunks: {} / {}                  ║",
                p.chunks_sent, p.total_chunks
            );
            println!(
                "║ Speed: {}/s                     ║",
                format_size(p.throughput_bps)
            );
            println!("║ ETA: {:.0}s                          ║", p.eta_seconds);

            if p.complete {
                println!("║                                            ║");
                println!("║        ✅ Transfer Complete!                ║");
            }
        } else {
            println!("║  No active transfer.                        ║");
        }

        println!("╚══════════════════════════════════════════╝");
    }
}

fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}
