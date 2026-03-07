//! # Adaptive Stream Controller
//!
//! Dynamically adjusts transfer parameters based on real-time network
//! metrics from the NetworkMonitor. The controller follows these rules:
//!
//! | Condition         | Action                          |
//! |-------------------|---------------------------------|
//! | Throughput ↑      | Increase stream count           |
//! | Packet loss ↑     | Reduce stream count             |
//! | Latency ↑         | Reduce chunk size               |
//! | CPU usage high    | Reduce parallel operations      |
//! | Network excellent | Maximize all parameters         |

use std::sync::Arc;
use std::time::Duration;

use tracing::{debug, info};

use crate::optimizer::network_monitor::{NetworkCondition, NetworkMonitor};

// ── Constants ──

/// Minimum number of parallel streams
const MIN_STREAMS: u32 = 2;

/// Maximum number of parallel streams
const MAX_STREAMS: u32 = 32;

/// Default number of parallel streams
const DEFAULT_STREAMS: u32 = 8;

/// Minimum chunk size (1 MB)
const MIN_CHUNK_SIZE: u64 = 1 * 1024 * 1024;

/// Maximum chunk size (32 MB)
const MAX_CHUNK_SIZE: u64 = 32 * 1024 * 1024;

/// Default chunk size (8 MB)
const DEFAULT_CHUNK_SIZE: u64 = 8 * 1024 * 1024;

/// How often to re-evaluate parameters (milliseconds)
const EVALUATION_INTERVAL_MS: u64 = 2000;

// ── Data Structures ──

/// Current recommended transfer parameters.
#[derive(Debug, Clone)]
pub struct TransferParams {
    /// Number of parallel QUIC streams to use
    pub stream_count: u32,
    /// Chunk size in bytes
    pub chunk_size: u64,
    /// Whether to enable compression
    pub compression_enabled: bool,
    /// Which compression algorithm to use
    pub compression_algo: String,
}

impl Default for TransferParams {
    fn default() -> Self {
        Self {
            stream_count: DEFAULT_STREAMS,
            chunk_size: DEFAULT_CHUNK_SIZE,
            compression_enabled: true,
            compression_algo: "lz4".into(),
        }
    }
}

/// The adaptive stream controller that tunes transfer parameters
/// in real-time based on network conditions.
pub struct AdaptiveStreamController {
    /// Reference to the network monitor
    monitor: Arc<NetworkMonitor>,
    /// Current parameters
    current_params: tokio::sync::RwLock<TransferParams>,
    /// Previous network condition (for change detection)
    last_condition: tokio::sync::RwLock<NetworkCondition>,
}

impl AdaptiveStreamController {
    /// Create a new adaptive controller.
    pub fn new(monitor: Arc<NetworkMonitor>) -> Self {
        Self {
            monitor,
            current_params: tokio::sync::RwLock::new(TransferParams::default()),
            last_condition: tokio::sync::RwLock::new(NetworkCondition::Good),
        }
    }

    /// Run the adaptive control loop.
    ///
    /// Periodically evaluates network metrics and adjusts transfer
    /// parameters accordingly.
    pub async fn run(&self) {
        let interval = Duration::from_millis(EVALUATION_INTERVAL_MS);

        loop {
            tokio::time::sleep(interval).await;
            self.evaluate_and_adjust().await;
        }
    }

    /// Get the current recommended transfer parameters.
    pub async fn get_params(&self) -> TransferParams {
        self.current_params.read().await.clone()
    }

    /// Evaluate current network conditions and adjust parameters.
    async fn evaluate_and_adjust(&self) {
        let metrics = self.monitor.get_metrics().await;
        let condition = self.monitor.get_condition().await;
        let throughput_trend = self.monitor.throughput_trend().await;

        let mut params = self.current_params.write().await;
        let mut last_condition = self.last_condition.write().await;

        let condition_changed = *last_condition != condition;
        *last_condition = condition;

        match condition {
            NetworkCondition::Excellent => {
                // Maximize performance
                params.stream_count = (params.stream_count + 2).min(MAX_STREAMS);
                params.chunk_size = MAX_CHUNK_SIZE;
                params.compression_algo = "lz4".into(); // Fast compression for speed
            }

            NetworkCondition::Good => {
                // Moderate optimization
                if throughput_trend > 5.0 {
                    params.stream_count = (params.stream_count + 1).min(MAX_STREAMS);
                }
                params.chunk_size = 16 * 1024 * 1024; // 16 MB
            }

            NetworkCondition::Fair => {
                // Conservative settings
                params.stream_count = DEFAULT_STREAMS;
                params.chunk_size = DEFAULT_CHUNK_SIZE;
                params.compression_algo = "zstd".into(); // Better compression ratio
            }

            NetworkCondition::Poor => {
                // Reduce load
                params.stream_count = (params.stream_count.saturating_sub(2)).max(MIN_STREAMS);
                params.chunk_size = 4 * 1024 * 1024; // 4 MB
                params.compression_algo = "zstd".into();
            }

            NetworkCondition::Critical => {
                // Minimal settings
                params.stream_count = MIN_STREAMS;
                params.chunk_size = MIN_CHUNK_SIZE;
                params.compression_algo = "zstd".into(); // Minimize data sent
            }
        }

        // High CPU usage → reduce parallel operations regardless of network
        if metrics.cpu_usage_percent > 90.0 {
            params.stream_count = (params.stream_count / 2).max(MIN_STREAMS);
        }

        if condition_changed {
            info!(
                "⚡ Network condition: {:?} → streams: {}, chunk: {} MB, compression: {}",
                condition,
                params.stream_count,
                params.chunk_size / (1024 * 1024),
                params.compression_algo
            );
        }
    }
}
