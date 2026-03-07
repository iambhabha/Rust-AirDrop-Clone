//! # QUIC Handshake & Capability Negotiation
//!
//! After a QUIC connection is established, the first bidirectional stream
//! performs a capability handshake. Both peers exchange their supported
//! features so the transfer engine can optimize accordingly.
//!
//! ## Handshake Flow
//!
//! ```text
//! Client                              Server
//!   |---- Capabilities JSON ---->       |
//!   |<--- Capabilities JSON -----       |
//!   |                                   |
//!   |  (negotiation complete,           |
//!   |   both sides know each            |
//!   |   other's capabilities)           |
//! ```

use anyhow::{Context, Result};
use quinn::{RecvStream, SendStream};
use serde::{Deserialize, Serialize};
use tracing::debug;

// ── Data Structures ──

/// Capabilities exchanged during the handshake.
///
/// This allows both sides to negotiate optimal transfer parameters
/// based on actual hardware and software capabilities.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Capabilities {
    /// Protocol version for compatibility checking
    pub protocol_version: u32,
    /// Maximum number of parallel QUIC streams
    pub max_streams: u32,
    /// Whether LZ4 compression is supported
    pub compression_lz4: bool,
    /// Whether ZSTD compression is supported
    pub compression_zstd: bool,
    /// Maximum chunk size in bytes this device can handle
    pub max_chunk_size: u64,
    /// Maximum bandwidth in Mbps (self-reported)
    pub max_bandwidth_mbps: u64,
    /// Whether distributed transfer is supported
    pub distributed_transfer: bool,
    /// Whether transfer resume is supported
    pub transfer_resume: bool,
    /// Device identifier
    pub device_id: String,
}

/// The result of a successful handshake — the negotiated parameters
/// that both sides will use for the transfer.
#[derive(Debug, Clone)]
pub struct NegotiatedParams {
    /// Agreed-upon number of parallel streams
    pub streams: u32,
    /// Agreed-upon chunk size in bytes
    pub chunk_size: u64,
    /// Which compression to use (None, "lz4", or "zstd")
    pub compression: Option<String>,
    /// Whether distributed transfer is available
    pub distributed: bool,
    /// Whether transfer resume is available
    pub resumable: bool,
}

// ── Functions ──

/// Get our own capabilities based on system configuration.
pub fn our_capabilities() -> Capabilities {
    Capabilities {
        protocol_version: 1,
        max_streams: 32,
        compression_lz4: true,
        compression_zstd: true,
        max_chunk_size: 32 * 1024 * 1024, // 32 MB
        max_bandwidth_mbps: 10_000,       // 10 Gbps
        distributed_transfer: true,
        transfer_resume: true,
        device_id: String::new(), // Filled in by caller
    }
}

/// Send our capabilities over a QUIC stream.
///
/// The capability JSON is length-prefixed with a 4-byte big-endian
/// length header for unambiguous framing.
pub async fn send_handshake(send: &mut SendStream, capabilities: &Capabilities) -> Result<()> {
    let data = serde_json::to_vec(capabilities).context("Failed to serialize capabilities")?;

    // Send length prefix (4 bytes, big-endian)
    let len_bytes = (data.len() as u32).to_be_bytes();
    send.write_all(&len_bytes)
        .await
        .context("Failed to send handshake length")?;

    // Send capability data
    send.write_all(&data)
        .await
        .context("Failed to send handshake data")?;

    debug!("Sent capabilities: {} bytes", data.len());
    Ok(())
}

/// Receive capabilities from a QUIC stream.
///
/// Reads the length-prefixed JSON capabilities message.
pub async fn receive_handshake(recv: &mut RecvStream) -> Result<Capabilities> {
    // Read length prefix
    let mut len_buf = [0u8; 4];
    recv.read_exact(&mut len_buf)
        .await
        .context("Failed to read handshake length")?;
    let len = u32::from_be_bytes(len_buf) as usize;

    // Sanity check — capabilities shouldn't be more than 64 KB
    if len > 65536 {
        anyhow::bail!("Handshake too large: {} bytes", len);
    }

    // Read capability data
    let mut data = vec![0u8; len];
    recv.read_exact(&mut data)
        .await
        .context("Failed to read handshake data")?;

    let capabilities: Capabilities =
        serde_json::from_slice(&data).context("Failed to deserialize capabilities")?;

    debug!("Received capabilities: {:?}", capabilities);
    Ok(capabilities)
}

/// Negotiate transfer parameters based on both sides' capabilities.
///
/// The negotiation follows a "minimum common denominator" approach:
/// - Streams: min of both sides' max_streams
/// - Chunk size: min of both sides' max_chunk_size
/// - Compression: prefer ZSTD > LZ4 > None
/// - Features: only enabled if both sides support them
pub fn negotiate(ours: &Capabilities, theirs: &Capabilities) -> NegotiatedParams {
    let streams = ours.max_streams.min(theirs.max_streams);
    let chunk_size = ours.max_chunk_size.min(theirs.max_chunk_size);

    // Prefer ZSTD for better compression ratio, fall back to LZ4 for speed
    let compression = if ours.compression_zstd && theirs.compression_zstd {
        Some("zstd".into())
    } else if ours.compression_lz4 && theirs.compression_lz4 {
        Some("lz4".into())
    } else {
        None
    };

    let distributed = ours.distributed_transfer && theirs.distributed_transfer;
    let resumable = ours.transfer_resume && theirs.transfer_resume;

    NegotiatedParams {
        streams,
        chunk_size,
        compression,
        distributed,
        resumable,
    }
}
