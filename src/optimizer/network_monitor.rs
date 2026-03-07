//! # Network Performance Monitor
//!
//! Continuously monitors network performance metrics and provides
//! real-time data to the adaptive stream controller for optimization.
//!
//! ## Tracked Metrics
//!
//! - **Throughput** — Bytes per second (rolling average)
//! - **Latency** — Round-trip time in milliseconds
//! - **Packet Loss** — Percentage of lost packets
//! - **CPU Usage** — System CPU utilization
//! - **Jitter** — Latency variance

use std::collections::VecDeque;
use std::sync::Arc;
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tracing::{debug, info};

// ── Constants ──

/// Number of samples to keep for rolling averages
const SAMPLE_WINDOW: usize = 60;

/// How often to update metrics (milliseconds)
const UPDATE_INTERVAL_MS: u64 = 1000;

// ── Data Structures ──

/// A snapshot of current network performance metrics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkMetrics {
    /// Current throughput in bytes per second
    pub throughput_bps: u64,
    /// Average latency in milliseconds
    pub latency_ms: f64,
    /// Packet loss percentage (0.0 - 100.0)
    pub packet_loss_percent: f64,
    /// CPU usage percentage (0.0 - 100.0)
    pub cpu_usage_percent: f64,
    /// Latency jitter in milliseconds
    pub jitter_ms: f64,
    /// Number of active QUIC streams
    pub active_streams: u32,
    /// Timestamp of this measurement
    #[serde(skip, default = "Instant::now")]
    pub timestamp: Instant,
}

impl Default for NetworkMetrics {
    fn default() -> Self {
        Self {
            throughput_bps: 0,
            latency_ms: 0.0,
            packet_loss_percent: 0.0,
            cpu_usage_percent: 0.0,
            jitter_ms: 0.0,
            active_streams: 0,
            timestamp: Instant::now(),
        }
    }
}

/// Network condition classification based on metrics.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NetworkCondition {
    /// Excellent: High throughput, low latency, no loss
    Excellent,
    /// Good: Decent throughput, moderate latency
    Good,
    /// Fair: Reduced throughput, some packet loss
    Fair,
    /// Poor: High latency, significant packet loss
    Poor,
    /// Critical: Very high loss, connection at risk
    Critical,
}

/// The network performance monitor.
///
/// Runs in the background, collecting and analyzing network metrics
/// to enable adaptive optimization of transfer parameters.
pub struct NetworkMonitor {
    /// Current metrics snapshot
    current_metrics: RwLock<NetworkMetrics>,
    /// Historical throughput samples for trend analysis
    throughput_history: RwLock<VecDeque<u64>>,
    /// Historical latency samples
    latency_history: RwLock<VecDeque<f64>>,
    /// Historical packet loss samples
    loss_history: RwLock<VecDeque<f64>>,
    /// Total bytes sent (for throughput calculation)
    bytes_counter: RwLock<u64>,
    /// Timestamp of last throughput calculation
    last_calc_time: RwLock<Instant>,
}

impl NetworkMonitor {
    /// Create a new network monitor.
    pub fn new() -> Self {
        Self {
            current_metrics: RwLock::new(NetworkMetrics::default()),
            throughput_history: RwLock::new(VecDeque::with_capacity(SAMPLE_WINDOW)),
            latency_history: RwLock::new(VecDeque::with_capacity(SAMPLE_WINDOW)),
            loss_history: RwLock::new(VecDeque::with_capacity(SAMPLE_WINDOW)),
            bytes_counter: RwLock::new(0),
            last_calc_time: RwLock::new(Instant::now()),
        }
    }

    /// Background monitoring loop.
    ///
    /// Periodically calculates throughput from the byte counter,
    /// maintains rolling averages, and updates the metrics snapshot.
    pub async fn run(&self) {
        let interval = Duration::from_millis(UPDATE_INTERVAL_MS);

        loop {
            tokio::time::sleep(interval).await;
            self.update_metrics().await;
        }
    }

    /// Record bytes transferred (called by the transfer engine).
    pub async fn record_bytes(&self, bytes: u64) {
        let mut counter = self.bytes_counter.write().await;
        *counter += bytes;
    }

    /// Record a latency measurement (called after receiving an ACK).
    pub async fn record_latency(&self, latency_ms: f64) {
        let mut history = self.latency_history.write().await;
        if history.len() >= SAMPLE_WINDOW {
            history.pop_front();
        }
        history.push_back(latency_ms);
    }

    /// Record a packet loss event.
    pub async fn record_packet_loss(&self, loss_percent: f64) {
        let mut history = self.loss_history.write().await;
        if history.len() >= SAMPLE_WINDOW {
            history.pop_front();
        }
        history.push_back(loss_percent);
    }

    /// Get the current network metrics snapshot.
    pub async fn get_metrics(&self) -> NetworkMetrics {
        self.current_metrics.read().await.clone()
    }

    /// Classify the current network condition.
    pub async fn get_condition(&self) -> NetworkCondition {
        let metrics = self.current_metrics.read().await;

        if metrics.packet_loss_percent > 10.0 {
            NetworkCondition::Critical
        } else if metrics.packet_loss_percent > 5.0 || metrics.latency_ms > 100.0 {
            NetworkCondition::Poor
        } else if metrics.packet_loss_percent > 1.0 || metrics.latency_ms > 50.0 {
            NetworkCondition::Fair
        } else if metrics.latency_ms > 10.0 {
            NetworkCondition::Good
        } else {
            NetworkCondition::Excellent
        }
    }

    /// Get the average throughput over the sample window.
    pub async fn average_throughput(&self) -> u64 {
        let history = self.throughput_history.read().await;
        if history.is_empty() {
            return 0;
        }
        history.iter().sum::<u64>() / history.len() as u64
    }

    /// Get the throughput trend (positive = increasing, negative = decreasing).
    pub async fn throughput_trend(&self) -> f64 {
        let history = self.throughput_history.read().await;
        if history.len() < 2 {
            return 0.0;
        }

        let recent: f64 = history.iter().rev().take(5).sum::<u64>() as f64 / 5.0;
        let older: f64 = history.iter().take(5).sum::<u64>() as f64 / 5.0;

        if older == 0.0 {
            return 0.0;
        }

        (recent - older) / older * 100.0
    }

    /// Internal: calculate and update all metrics.
    async fn update_metrics(&self) {
        // ── Calculate throughput ──
        let mut counter = self.bytes_counter.write().await;
        let mut last_time = self.last_calc_time.write().await;

        let elapsed = last_time.elapsed();
        let throughput = if elapsed.as_secs_f64() > 0.0 {
            (*counter as f64 / elapsed.as_secs_f64()) as u64
        } else {
            0
        };

        *counter = 0;
        *last_time = Instant::now();

        // ── Update throughput history ──
        {
            let mut history = self.throughput_history.write().await;
            if history.len() >= SAMPLE_WINDOW {
                history.pop_front();
            }
            history.push_back(throughput);
        }

        // ── Calculate averages ──
        let avg_latency = {
            let history = self.latency_history.read().await;
            if history.is_empty() {
                0.0
            } else {
                history.iter().sum::<f64>() / history.len() as f64
            }
        };

        let avg_loss = {
            let history = self.loss_history.read().await;
            if history.is_empty() {
                0.0
            } else {
                history.iter().sum::<f64>() / history.len() as f64
            }
        };

        let jitter = {
            let history = self.latency_history.read().await;
            if history.len() < 2 {
                0.0
            } else {
                let mean = history.iter().sum::<f64>() / history.len() as f64;
                let variance =
                    history.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / history.len() as f64;
                variance.sqrt()
            }
        };

        // ── Update current metrics ──
        let mut metrics = self.current_metrics.write().await;
        metrics.throughput_bps = throughput;
        metrics.latency_ms = avg_latency;
        metrics.packet_loss_percent = avg_loss;
        metrics.jitter_ms = jitter;
        metrics.timestamp = Instant::now();

        debug!(
            "📊 Network: {:.1} MB/s, {:.1}ms latency, {:.2}% loss",
            throughput as f64 / (1024.0 * 1024.0),
            avg_latency,
            avg_loss
        );
    }
}
