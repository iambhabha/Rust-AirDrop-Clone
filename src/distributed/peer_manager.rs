//! # Distributed Peer Manager
//!
//! Manages multiple peers participating in a distributed transfer.
//! When transferring large files, multiple nearby peers can act as
//! relay nodes to increase aggregate throughput beyond what a single
//! connection can achieve.
//!
//! ## Distributed Transfer Topology
//!
//! ```text
//! Sender ──→ Peer1 ──→ Receiver    (chunks 0-49)
//! Sender ──→ Peer2 ──→ Receiver    (chunks 50-99)
//! Sender ──→ Peer3 ──→ Receiver    (chunks 100-149)
//! Sender ─────────────→ Receiver   (chunks 150-199, direct)
//! ```
//!
//! Goal: Saturate all available network paths to achieve up to 10 GB/s
//! aggregate throughput across multiple simultaneous connections.

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;

use anyhow::{Context, Result};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

use crate::network::discovery::DeviceInfo;

// ── Constants ──

/// Maximum number of relay peers to use for a distributed transfer
const MAX_RELAY_PEERS: usize = 8;

/// Minimum file size to consider distributed transfer (100 MB)
const MIN_DISTRIBUTED_SIZE: u64 = 100 * 1024 * 1024;

// ── Data Structures ──

/// Status of a peer in the distributed transfer network.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum PeerStatus {
    /// Peer has been discovered but not yet connected.
    Discovered,
    /// QUIC connection is being established.
    Connecting,
    /// Peer is connected and ready for transfers.
    Connected,
    /// Peer is actively relaying chunks.
    Transferring,
    /// Peer has completed its assigned chunks.
    Completed,
    /// Peer encountered an error.
    Error(String),
    /// Peer has disconnected.
    Disconnected,
}

/// A peer participating in a distributed transfer.
#[derive(Debug, Clone)]
pub struct Peer {
    /// The device information from discovery
    pub device_info: DeviceInfo,
    /// Current status
    pub status: PeerStatus,
    /// The peer's QUIC server address
    pub addr: SocketAddr,
    /// Estimated bandwidth to this peer (bytes/sec)
    pub estimated_bandwidth: u64,
    /// Number of chunks assigned to this peer
    pub chunks_assigned: u64,
    /// Number of chunks completed by this peer
    pub chunks_completed: u64,
}

/// Assignment of chunk ranges to peers for distributed transfer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistributedPlan {
    /// File ID being transferred
    pub file_id: String,
    /// Total number of chunks
    pub total_chunks: u64,
    /// Assignments: peer_id → list of chunk indices
    pub assignments: HashMap<String, Vec<u64>>,
}

// ── Peer Manager ──

/// Manages peers for distributed file transfers.
///
/// The peer manager tracks available relay peers, estimates their
/// bandwidth capacity, and creates optimal distribution plans
/// that maximize aggregate throughput.
pub struct PeerManager {
    /// Our device ID
    device_id: String,
    /// Known peers keyed by device_id
    peers: DashMap<String, Peer>,
    /// Active distributed transfers
    active_transfers: DashMap<String, DistributedPlan>,
}

impl PeerManager {
    /// Create a new peer manager.
    pub fn new(device_id: String) -> Self {
        Self {
            device_id,
            peers: DashMap::new(),
            active_transfers: DashMap::new(),
        }
    }

    /// Register or update a peer from device discovery.
    pub fn register_peer(&self, device: DeviceInfo) {
        let addr = SocketAddr::new(device.ip_address, device.port);
        let peer = Peer {
            device_info: device.clone(),
            status: PeerStatus::Discovered,
            addr,
            estimated_bandwidth: estimate_bandwidth(&device.max_bandwidth),
            chunks_assigned: 0,
            chunks_completed: 0,
        };

        self.peers.insert(device.device_id.clone(), peer);
    }

    /// Remove a peer that has gone offline.
    pub fn remove_peer(&self, device_id: &str) {
        self.peers.remove(device_id);
    }

    /// Get all available peers (in Connected or Discovered status).
    pub fn available_peers(&self) -> Vec<Peer> {
        self.peers
            .iter()
            .filter(|p| matches!(p.status, PeerStatus::Discovered | PeerStatus::Connected))
            .map(|p| p.value().clone())
            .collect()
    }

    /// Check if a file is large enough to benefit from distributed transfer.
    pub fn should_distribute(file_size: u64) -> bool {
        file_size >= MIN_DISTRIBUTED_SIZE
    }

    /// Create a distributed transfer plan that assigns chunk ranges
    /// to multiple peers based on their estimated bandwidth.
    ///
    /// The algorithm:
    /// 1. Sort peers by estimated bandwidth (fastest first)
    /// 2. Assign chunks proportionally to bandwidth
    /// 3. Limit to MAX_RELAY_PEERS
    ///
    /// # Arguments
    /// * `file_id` — The transfer's file ID
    /// * `total_chunks` — Total number of chunks in the file
    /// * `receiver_id` — The final receiver's device ID (excluded from relay peers)
    pub fn create_distribution_plan(
        &self,
        file_id: &str,
        total_chunks: u64,
        receiver_id: &str,
    ) -> DistributedPlan {
        let mut available: Vec<Peer> = self
            .peers
            .iter()
            .filter(|p| {
                p.device_info.device_id != self.device_id
                    && p.device_info.device_id != receiver_id
                    && matches!(p.status, PeerStatus::Discovered | PeerStatus::Connected)
                    && p.device_info
                        .supported_features
                        .contains(&"distributed_transfer".into())
            })
            .map(|p| p.value().clone())
            .collect();

        // Sort by bandwidth (highest first)
        available.sort_by(|a, b| b.estimated_bandwidth.cmp(&a.estimated_bandwidth));

        // Limit to MAX_RELAY_PEERS
        available.truncate(MAX_RELAY_PEERS);

        let mut assignments: HashMap<String, Vec<u64>> = HashMap::new();

        if available.is_empty() {
            // No relay peers — all chunks go direct
            info!("No relay peers available, using direct transfer");
            let plan = DistributedPlan {
                file_id: file_id.into(),
                total_chunks,
                assignments,
            };
            self.active_transfers.insert(file_id.into(), plan.clone());
            return plan;
        }

        // Calculate total bandwidth for proportional assignment
        let total_bandwidth: u64 = available.iter().map(|p| p.estimated_bandwidth).sum();

        let mut chunk_index = 0u64;
        for (i, peer) in available.iter().enumerate() {
            let proportion = peer.estimated_bandwidth as f64 / total_bandwidth as f64;
            let num_chunks = if i == available.len() - 1 {
                // Last peer gets remaining chunks
                total_chunks - chunk_index
            } else {
                (total_chunks as f64 * proportion).ceil() as u64
            };

            let end = (chunk_index + num_chunks).min(total_chunks);
            let chunk_range: Vec<u64> = (chunk_index..end).collect();

            if !chunk_range.is_empty() {
                info!(
                    "📦 Assigned chunks {}-{} ({} chunks) to peer '{}' (est. {} MB/s)",
                    chunk_index,
                    end - 1,
                    chunk_range.len(),
                    peer.device_info.device_name,
                    peer.estimated_bandwidth / (1024 * 1024)
                );
                assignments.insert(peer.device_info.device_id.clone(), chunk_range);
            }

            chunk_index = end;
            if chunk_index >= total_chunks {
                break;
            }
        }

        let plan = DistributedPlan {
            file_id: file_id.into(),
            total_chunks,
            assignments,
        };

        self.active_transfers.insert(file_id.into(), plan.clone());
        plan
    }

    /// Update a peer's status.
    pub fn update_peer_status(&self, device_id: &str, status: PeerStatus) {
        if let Some(mut peer) = self.peers.get_mut(device_id) {
            peer.status = status;
        }
    }

    /// Get a peer by device ID.
    pub fn get_peer(&self, device_id: &str) -> Option<Peer> {
        self.peers.get(device_id).map(|p| p.value().clone())
    }

    /// Get the number of connected peers.
    pub fn peer_count(&self) -> usize {
        self.peers.len()
    }
}

/// Estimate bandwidth from the max_bandwidth string (e.g., "10Gb", "1Gb").
fn estimate_bandwidth(max_bandwidth: &str) -> u64 {
    let lower = max_bandwidth.to_lowercase();
    if lower.contains("10g") {
        10_000_000_000 / 8 // 10 Gbps in bytes/sec
    } else if lower.contains("5g") {
        5_000_000_000 / 8
    } else if lower.contains("2.5g") || lower.contains("2g") {
        2_500_000_000 / 8
    } else if lower.contains("1g") {
        1_000_000_000 / 8
    } else if lower.contains("100m") {
        100_000_000 / 8
    } else {
        100_000_000 / 8 // Default 100 Mbps
    }
}
