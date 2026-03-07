//! # QR Code Scanner
//!
//! Parses QR code data to extract pairing information.
//! On mobile platforms, the camera would be used for scanning.
//! On desktop, this processes QR code image files or clipboard data.

use anyhow::{Context, Result};
use tracing::info;

use crate::security::pairing::QrPairingData;

/// Parse QR code data from a JSON string.
///
/// This processes the raw data extracted from a QR code scan
/// and deserializes it into pairing data.
pub fn parse_qr_data(raw_data: &str) -> Result<QrPairingData> {
    let pairing_data: QrPairingData =
        serde_json::from_str(raw_data).context("Failed to parse QR code data")?;

    info!(
        "📷 QR code scanned: device '{}' at {}:{}",
        pairing_data.device_name, pairing_data.ip_address, pairing_data.port
    );

    // Check if the pairing token has expired
    if let Ok(expires) = chrono::DateTime::parse_from_rfc3339(&pairing_data.expires_at) {
        if expires < chrono::Utc::now() {
            anyhow::bail!("QR code pairing token has expired");
        }
    }

    Ok(pairing_data)
}

/// Validate that QR pairing data is still valid.
pub fn validate_qr_data(data: &QrPairingData) -> Result<bool> {
    // Check expiration
    if let Ok(expires) = chrono::DateTime::parse_from_rfc3339(&data.expires_at) {
        if expires < chrono::Utc::now() {
            return Ok(false);
        }
    }

    // Check required fields
    if data.device_id.is_empty() || data.ip_address.is_empty() || data.port == 0 {
        return Ok(false);
    }

    Ok(true)
}
