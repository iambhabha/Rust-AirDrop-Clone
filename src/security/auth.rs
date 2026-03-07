//! # Authentication & Transfer Authorization
//!
//! Handles transfer authorization — ensuring the user explicitly
//! accepts or rejects incoming file transfers before any data
//! is received.
//!
//! ## Security Model
//!
//! - All QUIC connections are TLS 1.3 encrypted
//! - Transfer requests require explicit user approval
//! - Optional device pairing for trusted repeat connections
//! - File metadata shown before acceptance

use std::collections::HashMap;
use std::sync::Arc;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use tokio::sync::{mpsc, oneshot, RwLock};
use tracing::{info, warn};

// ── Data Structures ──

/// An incoming transfer request that needs user approval.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferRequest {
    /// Unique request identifier
    pub request_id: String,
    /// Sender's device name
    pub sender_name: String,
    /// Sender's device ID
    pub sender_device_id: String,
    /// File name being sent
    pub file_name: String,
    /// File size in bytes
    pub file_size: u64,
    /// File type/MIME type (if known)
    pub file_type: String,
    /// Whether the sender is a paired/trusted device
    pub is_trusted: bool,
}

/// The user's response to a transfer request.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TransferResponse {
    /// User accepted the transfer
    Accept,
    /// User rejected the transfer
    Reject,
    /// Request timed out
    Timeout,
}

/// Manages transfer authorization and trusted device list.
pub struct AuthManager {
    /// Set of trusted device IDs (auto-accept from these)
    trusted_devices: RwLock<HashMap<String, String>>,
    /// Whether to auto-accept from trusted devices
    auto_accept_trusted: bool,
}

impl AuthManager {
    /// Create a new authentication manager.
    pub fn new() -> Self {
        Self {
            trusted_devices: RwLock::new(HashMap::new()),
            auto_accept_trusted: false,
        }
    }

    /// Check if a transfer should be auto-accepted.
    pub async fn should_auto_accept(&self, device_id: &str) -> bool {
        if !self.auto_accept_trusted {
            return false;
        }
        self.trusted_devices.read().await.contains_key(device_id)
    }

    /// Add a device to the trusted list.
    pub async fn trust_device(&self, device_id: String, device_name: String) {
        info!("🔒 Trusted device added: {} ({})", device_name, device_id);
        self.trusted_devices
            .write()
            .await
            .insert(device_id, device_name);
    }

    /// Remove a device from the trusted list.
    pub async fn untrust_device(&self, device_id: &str) {
        self.trusted_devices.write().await.remove(device_id);
    }

    /// Check if a device is trusted.
    pub async fn is_trusted(&self, device_id: &str) -> bool {
        self.trusted_devices.read().await.contains_key(device_id)
    }

    /// Get the list of trusted devices.
    pub async fn trusted_devices(&self) -> HashMap<String, String> {
        self.trusted_devices.read().await.clone()
    }

    /// Create a transfer request from transfer metadata.
    pub fn create_request(
        sender_name: &str,
        sender_device_id: &str,
        file_name: &str,
        file_size: u64,
    ) -> TransferRequest {
        TransferRequest {
            request_id: uuid::Uuid::new_v4().to_string(),
            sender_name: sender_name.into(),
            sender_device_id: sender_device_id.into(),
            file_name: file_name.into(),
            file_size,
            file_type: guess_file_type(file_name),
            is_trusted: false,
        }
    }

    /// Enable auto-accept for trusted devices.
    pub fn set_auto_accept(&mut self, enabled: bool) {
        self.auto_accept_trusted = enabled;
    }
}

/// Guess the file type from the file extension.
fn guess_file_type(file_name: &str) -> String {
    let ext = file_name.rsplit('.').next().unwrap_or("").to_lowercase();
    match ext.as_str() {
        "jpg" | "jpeg" | "png" | "gif" | "webp" | "bmp" => "image".into(),
        "mp4" | "mkv" | "avi" | "mov" | "webm" => "video".into(),
        "mp3" | "wav" | "flac" | "aac" | "ogg" => "audio".into(),
        "pdf" | "doc" | "docx" | "txt" | "rtf" => "document".into(),
        "zip" | "tar" | "gz" | "7z" | "rar" => "archive".into(),
        "exe" | "msi" | "dmg" | "deb" | "rpm" => "application".into(),
        _ => "file".into(),
    }
}
