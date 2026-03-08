//! # FastShare
//!
//! Ultra-high-performance peer-to-peer distributed file transfer system.
//!
//! ## Architecture
//!
//! FastShare is built around several core subsystems:
//!
//! - **Network Discovery**: Multi-protocol device discovery (mDNS, UDP broadcast, multicast)
//! - **QUIC Transport**: Secure, multiplexed connections via Quinn
//! - **Transfer Engine**: Parallel chunk-based file transfer with adaptive streaming
//! - **Distributed Transfer**: Multi-peer chunk distribution for maximum throughput
//! - **Network Optimizer**: Real-time metrics and adaptive stream tuning
//! - **Storage Engine**: Streaming I/O for files of any size (100GB+)
//! - **Security**: TLS encryption, transfer confirmation, QR pairing

pub mod app;
pub mod compression;
pub mod distributed;
pub mod network;
pub mod optimizer;
pub mod qr;
pub mod security;
pub mod storage;
pub mod transfer;
pub mod ui;
pub mod wifi_direct;

use std::sync::atomic::{AtomicBool, Ordering};

/// Global flag: whether to verify chunk checksums.
/// Off by default for max speed. Can be toggled from Flutter settings.
pub static CHECKSUM_ENABLED: AtomicBool = AtomicBool::new(true);

/// Global flag: whether to enable compression.
/// Off by default for high-speed networks, on for slow networks.
pub static COMPRESSION_ENABLED: AtomicBool = AtomicBool::new(false);

/// Check if checksum verification is enabled.
#[inline]
pub fn is_checksum_enabled() -> bool {
    CHECKSUM_ENABLED.load(Ordering::Relaxed)
}

/// Enable or disable checksum verification.
pub fn set_checksum_enabled(enabled: bool) {
    CHECKSUM_ENABLED.store(enabled, Ordering::Relaxed);
}

/// Check if compression is enabled.
#[inline]
pub fn is_compression_enabled() -> bool {
    COMPRESSION_ENABLED.load(Ordering::Relaxed)
}

/// Enable or disable compression.
pub fn set_compression_enabled(enabled: bool) {
    COMPRESSION_ENABLED.store(enabled, Ordering::Relaxed);
}
