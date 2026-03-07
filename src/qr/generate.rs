//! # QR Code Generation
//!
//! Generates QR codes containing pairing data for out-of-band
//! device discovery and authentication. The QR code encodes a
//! JSON payload with connection and pairing information.

use std::path::Path;

use anyhow::{Context, Result};
use qrcode::render::unicode;
use qrcode::QrCode;
use tracing::info;

use crate::security::pairing::QrPairingData;

/// Generate a QR code as a Unicode string (for terminal display).
///
/// This is useful for CLI usage where the QR code is displayed
/// directly in the terminal for scanning by a nearby device.
pub fn generate_terminal_qr(data: &QrPairingData) -> Result<String> {
    let json = serde_json::to_string(data).context("Failed to serialize QR data")?;

    let code = QrCode::new(json.as_bytes()).context("Failed to generate QR code")?;

    let string = code
        .render::<unicode::Dense1x2>()
        .dark_color(unicode::Dense1x2::Light)
        .light_color(unicode::Dense1x2::Dark)
        .build();

    Ok(string)
}

/// Generate a QR code and save it as a PNG image file.
///
/// The image can be displayed in the GUI or shared via other means.
pub fn generate_qr_image(data: &QrPairingData, output_path: &Path) -> Result<()> {
    let json = serde_json::to_string(data).context("Failed to serialize QR data")?;

    let code = QrCode::new(json.as_bytes()).context("Failed to generate QR code")?;

    let image = code
        .render::<image::Luma<u8>>()
        .quiet_zone(true)
        .min_dimensions(256, 256)
        .build();

    image
        .save(output_path)
        .context("Failed to save QR code image")?;

    info!("QR code saved to: {}", output_path.display());
    Ok(())
}

/// Generate a QR code as raw PNG bytes (for embedding in UI).
pub fn generate_qr_bytes(data: &QrPairingData) -> Result<Vec<u8>> {
    let json = serde_json::to_string(data).context("Failed to serialize QR data")?;

    let code = QrCode::new(json.as_bytes()).context("Failed to generate QR code")?;

    let image = code
        .render::<image::Luma<u8>>()
        .quiet_zone(true)
        .min_dimensions(256, 256)
        .build();

    let mut png_data = Vec::new();
    let mut cursor = std::io::Cursor::new(&mut png_data);
    image::DynamicImage::ImageLuma8(image)
        .write_to(&mut cursor, image::ImageFormat::Png)
        .context("Failed to encode QR code as PNG")?;

    Ok(png_data)
}
