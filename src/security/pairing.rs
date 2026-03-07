//! # Device Pairing
//!
//! Provides secure device pairing mechanisms:
//!
//! 1. **Temporary Pairing Key** — Short numeric code displayed on both devices
//! 2. **QR Authentication** — Scan QR code containing pairing data
//! 3. **Permanent Trust** — Remember paired devices for future sessions

use std::time::{Duration, Instant};

use anyhow::Result;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tracing::info;

// ── Constants ──

/// Default pairing key validity duration (5 minutes)
const PAIRING_KEY_VALIDITY_SECS: u64 = 300;

/// Length of the numeric pairing code
const PAIRING_CODE_LENGTH: usize = 6;

// ── Data Structures ──

/// A pairing session between two devices.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PairingSession {
    /// Unique session identifier
    pub session_id: String,
    /// Our device ID
    pub our_device_id: String,
    /// Remote device ID
    pub remote_device_id: String,
    /// The pairing code (displayed to user)
    pub pairing_code: String,
    /// Shared secret derived from the pairing
    pub shared_secret: Option<String>,
    /// When this session was created
    pub created_at: String,
    /// Whether pairing is confirmed
    pub confirmed: bool,
}

/// QR code pairing data structure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QrPairingData {
    /// Device ID of the QR generator
    pub device_id: String,
    /// Device name
    pub device_name: String,
    /// IP address
    pub ip_address: String,
    /// Port
    pub port: u16,
    /// One-time pairing token
    pub pairing_token: String,
    /// Expiration timestamp
    pub expires_at: String,
}

/// The pairing manager handles device pairing flows.
pub struct PairingManager {
    /// Our device ID
    device_id: String,
    /// Active pairing sessions
    sessions: Vec<PairingSession>,
}

impl PairingManager {
    /// Create a new pairing manager.
    pub fn new(device_id: String) -> Self {
        Self {
            device_id,
            sessions: Vec::new(),
        }
    }

    /// Generate a numeric pairing code.
    ///
    /// The code is a 6-digit number that both devices display.
    /// The user visually confirms both codes match.
    pub fn generate_pairing_code(&mut self, remote_device_id: &str) -> PairingSession {
        // Generate deterministic but unpredictable code
        let seed = format!(
            "{}:{}:{}",
            self.device_id,
            remote_device_id,
            chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0)
        );

        let mut hasher = Sha256::new();
        hasher.update(seed.as_bytes());
        let hash = hasher.finalize();

        // Use first bytes to create a 6-digit code
        let code_num = u32::from_be_bytes([hash[0], hash[1], hash[2], hash[3]]) % 1_000_000;
        let pairing_code = format!("{:06}", code_num);

        let session = PairingSession {
            session_id: uuid::Uuid::new_v4().to_string(),
            our_device_id: self.device_id.clone(),
            remote_device_id: remote_device_id.into(),
            pairing_code: pairing_code.clone(),
            shared_secret: None,
            created_at: chrono::Utc::now().to_rfc3339(),
            confirmed: false,
        };

        info!("🔐 Pairing code generated: {}", pairing_code);
        self.sessions.push(session.clone());

        session
    }

    /// Confirm a pairing by verifying the code matches.
    pub fn confirm_pairing(&mut self, session_id: &str, code: &str) -> Result<bool> {
        if let Some(session) = self
            .sessions
            .iter_mut()
            .find(|s| s.session_id == session_id)
        {
            if session.pairing_code == code {
                session.confirmed = true;

                // Derive shared secret from both device IDs + code
                let mut hasher = Sha256::new();
                hasher.update(session.our_device_id.as_bytes());
                hasher.update(session.remote_device_id.as_bytes());
                hasher.update(code.as_bytes());
                session.shared_secret = Some(hex::encode(hasher.finalize()));

                info!(
                    "✅ Pairing confirmed with device {}",
                    session.remote_device_id
                );
                Ok(true)
            } else {
                info!("❌ Pairing code mismatch");
                Ok(false)
            }
        } else {
            anyhow::bail!("Pairing session not found: {}", session_id)
        }
    }

    /// Generate QR pairing data for scanning.
    pub fn generate_qr_data(
        &self,
        device_name: &str,
        ip_address: &str,
        port: u16,
    ) -> QrPairingData {
        let token = uuid::Uuid::new_v4().to_string();
        let expires_at = (chrono::Utc::now()
            + chrono::Duration::seconds(PAIRING_KEY_VALIDITY_SECS as i64))
        .to_rfc3339();

        QrPairingData {
            device_id: self.device_id.clone(),
            device_name: device_name.into(),
            ip_address: ip_address.into(),
            port,
            pairing_token: token,
            expires_at,
        }
    }

    /// Clean up expired pairing sessions.
    pub fn cleanup_expired(&mut self) {
        // Remove sessions older than PAIRING_KEY_VALIDITY_SECS
        let cutoff =
            chrono::Utc::now() - chrono::Duration::seconds(PAIRING_KEY_VALIDITY_SECS as i64);
        self.sessions.retain(|s| {
            if let Ok(created) = chrono::DateTime::parse_from_rfc3339(&s.created_at) {
                created > cutoff
            } else {
                false
            }
        });
    }
}
