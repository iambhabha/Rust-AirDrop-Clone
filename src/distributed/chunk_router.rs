//! # Distributed Chunk Router
//!
//! Routes file chunks through relay peers to the final receiver.
//! Each relay peer receives chunks from the sender and forwards
//! them to the receiver, multiplying the available network paths.
//!
//! ## Routing Flow
//!
//! ```text
//!                 ┌─ Relay Peer 1 ─┐
//! Sender ────────>│ chunks [0-49]  │────────> Receiver
//!                 └────────────────┘
//!                 ┌─ Relay Peer 2 ─┐
//! Sender ────────>│ chunks [50-99] │────────> Receiver
//!                 └────────────────┘
//! Sender ─────────────────────────────────> Receiver
//!                 (chunks [100-149], direct)
//! ```

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use tracing::{debug, error, info, warn};

use crate::distributed::peer_manager::{DistributedPlan, PeerManager, PeerStatus};
use crate::network::connection::QuicServer;
use crate::transfer::chunker::ChunkMeta;

// ── Data Structures ──

/// A routing instruction for a single chunk.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkRoute {
    /// The chunk to route
    pub chunk_index: u64,
    /// Route type: Direct or via relay
    pub route_type: RouteType,
    /// Target address for the chunk
    pub target_addr: SocketAddr,
    /// Final destination address (may differ from target if relayed)
    pub destination_addr: SocketAddr,
}

/// How a chunk is routed to its destination.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RouteType {
    /// Direct from sender to receiver
    Direct,
    /// Via a relay peer
    Relay {
        /// Device ID of the relay peer
        relay_device_id: String,
        /// Address of the relay peer
        relay_addr: SocketAddr,
    },
}

/// Control message sent to relay peers.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RelayMessage {
    /// Request the relay to forward received chunks to the destination
    StartRelay {
        file_id: String,
        destination_addr: SocketAddr,
        chunk_indices: Vec<u64>,
    },
    /// Stop relaying
    StopRelay { file_id: String },
    /// Status update
    RelayStatus {
        file_id: String,
        chunks_forwarded: u64,
    },
}

/// The chunk router creates routes for distributed transfer.
pub struct ChunkRouter {
    /// Reference to the peer manager
    peer_manager: Arc<PeerManager>,
}

impl ChunkRouter {
    /// Create a new chunk router.
    pub fn new(peer_manager: Arc<PeerManager>) -> Self {
        Self { peer_manager }
    }

    /// Create routing table for a distributed transfer.
    ///
    /// Takes a distribution plan (from PeerManager) and the receiver's
    /// address, and produces a complete routing table mapping each
    /// chunk to its route.
    pub fn create_routing_table(
        &self,
        plan: &DistributedPlan,
        receiver_addr: SocketAddr,
    ) -> Vec<ChunkRoute> {
        let mut routes = Vec::new();

        // Track which chunks are assigned to relay peers
        let mut assigned_chunks: HashMap<u64, String> = HashMap::new();
        for (peer_id, chunks) in &plan.assignments {
            for &chunk_idx in chunks {
                assigned_chunks.insert(chunk_idx, peer_id.clone());
            }
        }

        for chunk_idx in 0..plan.total_chunks {
            let route = if let Some(peer_id) = assigned_chunks.get(&chunk_idx) {
                // Route through relay peer
                if let Some(peer) = self.peer_manager.get_peer(peer_id) {
                    ChunkRoute {
                        chunk_index: chunk_idx,
                        route_type: RouteType::Relay {
                            relay_device_id: peer_id.clone(),
                            relay_addr: peer.addr,
                        },
                        target_addr: peer.addr,
                        destination_addr: receiver_addr,
                    }
                } else {
                    // Peer not found, fall back to direct
                    ChunkRoute {
                        chunk_index: chunk_idx,
                        route_type: RouteType::Direct,
                        target_addr: receiver_addr,
                        destination_addr: receiver_addr,
                    }
                }
            } else {
                // Direct route
                ChunkRoute {
                    chunk_index: chunk_idx,
                    route_type: RouteType::Direct,
                    target_addr: receiver_addr,
                    destination_addr: receiver_addr,
                }
            };

            routes.push(route);
        }

        let relay_count = routes
            .iter()
            .filter(|r| matches!(r.route_type, RouteType::Relay { .. }))
            .count();
        let direct_count = routes.len() - relay_count;

        info!(
            "📡 Routing table created: {} total chunks ({} direct, {} relayed)",
            routes.len(),
            direct_count,
            relay_count
        );

        routes
    }

    /// Set up relay peers by sending them relay instructions.
    ///
    /// Connects to each relay peer and instructs them to forward
    /// received chunks to the final receiver.
    pub async fn setup_relays(
        &self,
        plan: &DistributedPlan,
        quic_server: &QuicServer,
        receiver_addr: SocketAddr,
    ) -> Result<()> {
        for (peer_id, chunk_indices) in &plan.assignments {
            if let Some(peer) = self.peer_manager.get_peer(peer_id) {
                info!(
                    "Setting up relay through '{}' for {} chunks",
                    peer.device_info.device_name,
                    chunk_indices.len()
                );

                // Connect to the relay peer
                match quic_server.connect_to_peer(peer.addr).await {
                    Ok(connection) => {
                        // Send relay instructions on the first stream
                        let (mut send, _recv) = connection
                            .open_bi()
                            .await
                            .context("Failed to open relay control stream")?;

                        let msg = RelayMessage::StartRelay {
                            file_id: plan.file_id.clone(),
                            destination_addr: receiver_addr,
                            chunk_indices: chunk_indices.clone(),
                        };

                        let data = serde_json::to_vec(&msg)?;
                        let len = (data.len() as u32).to_be_bytes();
                        send.write_all(&len).await?;
                        send.write_all(&data).await?;
                        send.finish()?;

                        self.peer_manager
                            .update_peer_status(peer_id, PeerStatus::Connected);

                        info!(
                            "✓ Relay setup complete for peer '{}'",
                            peer.device_info.device_name
                        );
                    }
                    Err(e) => {
                        warn!(
                            "Failed to connect to relay peer '{}': {}",
                            peer.device_info.device_name, e
                        );
                        self.peer_manager
                            .update_peer_status(peer_id, PeerStatus::Error(e.to_string()));
                    }
                }
            }
        }

        Ok(())
    }
}
