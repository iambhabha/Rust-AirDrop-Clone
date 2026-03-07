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
