//! # Receive File Screen
//!
//! Interface for incoming file transfer requests. Displays the
//! sender info, file metadata, and accept/reject options.

use crate::security::auth::TransferRequest;

/// Receive screen for incoming transfer requests.
pub struct ReceiveScreen {
    /// Pending transfer request
    pub request: Option<TransferRequest>,
    /// Whether we're currently receiving
    pub is_receiving: bool,
}

impl ReceiveScreen {
    /// Create a new receive screen.
    pub fn new() -> Self {
        Self {
            request: None,
            is_receiving: false,
        }
    }

    /// Set a pending transfer request.
    pub fn set_request(&mut self, request: TransferRequest) {
        self.request = Some(request);
    }

    /// Render the receive screen.
    pub fn render(&self) {
        println!("╔══════════════════════════════════════════╗");
        println!("║              📥 Receive File             ║");
        println!("╠══════════════════════════════════════════╣");

        if let Some(ref req) = self.request {
            println!("║                                            ║");
            println!("║  📱 {} wants to send a file       ║", req.sender_name);
            println!("║                                            ║");
            println!("║  File: {:<33} ║", req.file_name);
            println!("║  Size: {:<33} ║", format_size(req.file_size));
            println!("║  Type: {:<33} ║", req.file_type);
            println!("║                                            ║");

            if req.is_trusted {
                println!("║  🔒 Trusted Device                         ║");
            } else {
                println!("║  ⚠️  Unknown Device                        ║");
            }

            println!("║                                            ║");
            println!("╠══════════════════════════════════════════╣");
            println!("║    [A] Accept        [R] Reject           ║");
        } else {
            println!("║                                            ║");
            println!("║  Waiting for incoming transfers...         ║");
            println!("║                                            ║");
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
