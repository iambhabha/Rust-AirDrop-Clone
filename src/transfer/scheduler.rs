//! # Stream Scheduler
//!
//! Distributes file chunks across multiple QUIC streams for load-balanced
//! parallel transfer. The scheduler optimizes chunk assignment to maximize
//! throughput and minimize transfer time.
//!
//! ## Scheduling Strategies
//!
//! - **Round Robin**: Simple even distribution across streams
//! - **Weighted**: Prioritize streams with lower latency/higher throughput
//! - **Adaptive**: Dynamically adjust based on real-time stream metrics

use std::collections::VecDeque;
use std::sync::Arc;

use anyhow::Result;
use tokio::sync::Mutex;
use tracing::debug;

use crate::transfer::chunker::ChunkMeta;

// ── Scheduling Strategy ──

/// Scheduling strategy for distributing chunks across streams.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScheduleStrategy {
    /// Even round-robin distribution
    RoundRobin,
    /// Weighted by stream capacity
    Weighted,
    /// Dynamically adjusted by metrics
    Adaptive,
}

/// A stream assignment — which chunk goes to which stream.
#[derive(Debug, Clone)]
pub struct StreamAssignment {
    /// Stream index (0 to N-1)
    pub stream_index: usize,
    /// The chunk to send on this stream
    pub chunk: ChunkMeta,
}

// ── Stream Scheduler ──

/// Distributes chunks across parallel QUIC streams.
///
/// The scheduler maintains a queue of pending chunks and assigns
/// them to streams as they become available.
pub struct StreamScheduler {
    /// Number of available streams
    num_streams: usize,
    /// Scheduling strategy
    strategy: ScheduleStrategy,
    /// Queue of chunks waiting to be assigned
    pending_chunks: Mutex<VecDeque<ChunkMeta>>,
    /// Current round-robin index
    current_stream: Mutex<usize>,
}

impl StreamScheduler {
    /// Create a new stream scheduler.
    ///
    /// # Arguments
    /// * `num_streams` — Number of parallel QUIC streams
    /// * `strategy` — How to distribute chunks across streams
    pub fn new(num_streams: usize, strategy: ScheduleStrategy) -> Self {
        Self {
            num_streams,
            strategy,
            pending_chunks: Mutex::new(VecDeque::new()),
            current_stream: Mutex::new(0),
        }
    }

    /// Queue chunks for scheduling.
    pub async fn add_chunks(&self, chunks: Vec<ChunkMeta>) {
        let mut queue = self.pending_chunks.lock().await;
        for chunk in chunks {
            queue.push_back(chunk);
        }
        debug!("Queued {} chunks for scheduling", queue.len());
    }

    /// Get the next chunk assignment.
    ///
    /// Returns `None` if no more chunks are pending.
    pub async fn next_assignment(&self) -> Option<StreamAssignment> {
        let mut queue = self.pending_chunks.lock().await;
        let chunk = queue.pop_front()?;

        let stream_index = match self.strategy {
            ScheduleStrategy::RoundRobin => {
                let mut current = self.current_stream.lock().await;
                let index = *current;
                *current = (*current + 1) % self.num_streams;
                index
            }
            ScheduleStrategy::Weighted => {
                // Weighted scheduling based on chunk index
                // In production, this would use stream metrics
                chunk.chunk_index as usize % self.num_streams
            }
            ScheduleStrategy::Adaptive => {
                // Adaptive scheduling — for now, falls back to round-robin
                // In production, this would query the network monitor
                let mut current = self.current_stream.lock().await;
                let index = *current;
                *current = (*current + 1) % self.num_streams;
                index
            }
        };

        Some(StreamAssignment {
            stream_index,
            chunk,
        })
    }

    /// Get all assignments for a batch of chunks.
    ///
    /// Returns a vector of (stream_index, chunks) grouped by stream.
    pub async fn schedule_all(&self, chunks: Vec<ChunkMeta>) -> Vec<Vec<ChunkMeta>> {
        let mut stream_queues: Vec<Vec<ChunkMeta>> =
            (0..self.num_streams).map(|_| Vec::new()).collect();

        for (i, chunk) in chunks.into_iter().enumerate() {
            let stream = match self.strategy {
                ScheduleStrategy::RoundRobin => i % self.num_streams,
                ScheduleStrategy::Weighted => i % self.num_streams,
                ScheduleStrategy::Adaptive => i % self.num_streams,
            };
            stream_queues[stream].push(chunk);
        }

        debug!(
            "Scheduled chunks across {} streams: {:?}",
            self.num_streams,
            stream_queues.iter().map(|q| q.len()).collect::<Vec<_>>()
        );

        stream_queues
    }

    /// Get the number of chunks still pending.
    pub async fn pending_count(&self) -> usize {
        self.pending_chunks.lock().await.len()
    }

    /// Get the number of streams.
    pub fn num_streams(&self) -> usize {
        self.num_streams
    }
}
