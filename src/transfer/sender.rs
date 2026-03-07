//! # Parallel Stream Transfer Sender
//!
//! Sends file chunks across multiple QUIC streams simultaneously for
//! maximum throughput. The sender coordinates with the scheduler to
//! determine which chunks go on which streams.
//!
//! ## Transfer Flow
//!
//! ```text
//! File → Chunker → Scheduler → QUIC Stream 1 → chunk_0
//!                             → QUIC Stream 2 → chunk_1
//!                             → QUIC Stream 3 → chunk_2
//!                             → ...
//!                             → QUIC Stream N → chunk_N
//! ```
//!
//! Each stream sends:
//! 1. Chunk metadata (JSON, length-prefixed)
//! 2. Chunk data (raw bytes)
//! 3. Waits for ACK from receiver

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Instant;

use anyhow::{Context, Result};
use quinn::Connection;
use tokio::sync::{mpsc, Semaphore};
use tracing::{debug, error, info, warn};

use crate::compression;
use crate::transfer::chunker::{ChunkMeta, FileChunkPlan, FileChunker, NetworkSpeed};
use crate::transfer::resume::TransferState;
use crate::transfer::scheduler::StreamScheduler;

// ── Constants ──

/// Default number of parallel streams for sending
const DEFAULT_PARALLEL_STREAMS: usize = 8;

/// Maximum number of parallel streams
const MAX_PARALLEL_STREAMS: usize = 32;

// ── Data Structures ──

/// Progress callback type for tracking transfer progress.
pub type ProgressCallback = Box<dyn Fn(TransferProgress) + Send + Sync>;

/// Real-time transfer progress information.
#[derive(Debug, Clone)]
pub struct TransferProgress {
    /// File being transferred
    pub file_name: String,
    /// Unique transfer ID
    pub file_id: String,
    /// Total file size in bytes
    pub total_bytes: u64,
    /// Bytes transferred so far
    pub bytes_sent: u64,
    /// Number of chunks sent
    pub chunks_sent: u64,
    /// Total number of chunks
    pub total_chunks: u64,
    /// Current throughput in bytes per second
    pub throughput_bps: u64,
    /// Estimated time remaining in seconds
    pub eta_seconds: f64,
    /// Whether the transfer is complete
    pub complete: bool,
}

/// The parallel file transfer sender.
///
/// Manages chunking, scheduling, and sending of file data across
/// multiple QUIC streams for maximum throughput.
#[derive(Clone)]
pub struct TransferSender {
    /// Number of parallel streams to use
    parallel_streams: usize,
    /// Compression algorithm to use (None, "lz4", or "zstd")
    compression: Option<String>,
}

impl TransferSender {
    /// Create a new transfer sender with default settings.
    pub fn new() -> Self {
        Self {
            parallel_streams: DEFAULT_PARALLEL_STREAMS,
            compression: Some("lz4".into()),
        }
    }

    /// Create a sender with custom parallel stream count.
    pub fn with_streams(parallel_streams: usize) -> Self {
        let streams = parallel_streams.min(MAX_PARALLEL_STREAMS);
        Self {
            parallel_streams: streams,
            compression: Some("lz4".into()),
        }
    }

    /// Set the compression algorithm.
    pub fn set_compression(&mut self, compression: Option<String>) {
        self.compression = compression;
    }

    /// Send a file to a connected peer over the given QUIC connection.
    ///
    /// This is the main entry point for file transfer. It:
    /// 1. Plans the chunking of the file
    /// 2. Checks for a resume state (skip already-sent chunks)
    /// 3. Opens N parallel QUIC streams
    /// 4. Distributes chunks across streams via the scheduler
    /// 5. Reports progress via callback
    ///
    /// # Arguments
    /// * `connection` — Active QUIC connection to the receiver
    /// * `file_path` — Path to the file to send
    /// * `network_speed` — Detected network speed for adaptive chunking
    /// * `resume_state` — Optional resume state from a previous interrupted transfer
    /// * `progress_cb` — Optional callback for progress updates
    pub async fn send_file(
        &self,
        connection: &Connection,
        file_path: &Path,
        network_speed: NetworkSpeed,
        resume_state: Option<&TransferState>,
        progress_cb: Option<ProgressCallback>,
    ) -> Result<()> {
        let start_time = Instant::now();
        let file_path = file_path.to_path_buf();

        // ── Plan Chunks ──
        let chunker = FileChunker::adaptive(network_speed);
        let plan = chunker.plan_file(&file_path).await?;

        info!(
            "📤 Starting transfer: '{}' ({} bytes, {} chunks, {} streams)",
            plan.file_name, plan.total_size, plan.total_chunks, self.parallel_streams
        );

        // ── Determine which chunks to send (skip resumed ones) ──
        let chunks_to_send: Vec<ChunkMeta> = if let Some(state) = resume_state {
            plan.chunks
                .iter()
                .filter(|c| !state.received_chunks.contains(&c.chunk_index))
                .cloned()
                .collect()
        } else {
            plan.chunks.clone()
        };

        if chunks_to_send.is_empty() {
            info!("All chunks already sent (resumed transfer complete)");
            return Ok(());
        }

        info!(
            "Sending {} chunks ({} skipped from resume)",
            chunks_to_send.len(),
            plan.total_chunks as usize - chunks_to_send.len()
        );

        // ── Send File Metadata on first stream ──
        // The receiver needs to know about the file before chunks arrive
        let (mut meta_send, _meta_recv) = connection
            .open_bi()
            .await
            .context("Failed to open metadata stream")?;

        let file_meta_json = serde_json::to_vec(&plan).context("Failed to serialize file plan")?;
        let meta_len = (file_meta_json.len() as u32).to_be_bytes();
        meta_send.write_all(&[0x01]).await?; // Type 1: FileMetaData
        meta_send.write_all(&meta_len).await?;
        meta_send.write_all(&file_meta_json).await?;
        meta_send.finish()?;

        // ── Distribute Chunks Across Streams ──
        let bytes_sent = Arc::new(AtomicU64::new(0));
        let chunks_sent = Arc::new(AtomicU64::new(0));
        let semaphore = Arc::new(Semaphore::new(self.parallel_streams));

        let mut handles = Vec::new();

        for chunk_meta in chunks_to_send {
            let permit = semaphore
                .clone()
                .acquire_owned()
                .await
                .context("Semaphore closed")?;
            let connection = connection.clone();
            let file_path = file_path.clone();
            let bytes_sent = bytes_sent.clone();
            let chunks_sent = chunks_sent.clone();
            let chunker_size = chunker.chunk_size();
            let compression = self.compression.clone();

            let handle = tokio::spawn(async move {
                let result = send_chunk(
                    &connection,
                    &file_path,
                    &chunk_meta,
                    chunker_size,
                    compression.as_deref(),
                )
                .await;

                match &result {
                    Ok(sent_bytes) => {
                        bytes_sent.fetch_add(*sent_bytes, Ordering::Relaxed);
                        chunks_sent.fetch_add(1, Ordering::Relaxed);
                        debug!(
                            "✓ Chunk {}/{} sent ({} bytes)",
                            chunk_meta.chunk_index + 1,
                            chunk_meta.total_chunks,
                            sent_bytes
                        );
                    }
                    Err(e) => {
                        error!("✗ Chunk {} failed: {}", chunk_meta.chunk_index, e);
                    }
                }

                drop(permit);
                result
            });

            handles.push(handle);
        }

        // ── Wait for all chunks to complete ──
        let mut errors = Vec::new();
        for handle in handles {
            match handle.await {
                Ok(Ok(_)) => {}
                Ok(Err(e)) => errors.push(e),
                Err(e) => errors.push(anyhow::anyhow!("Task panicked: {}", e)),
            }
        }

        let elapsed = start_time.elapsed();
        let total_sent = bytes_sent.load(Ordering::Relaxed);
        let throughput_mbps = if elapsed.as_secs_f64() > 0.0 {
            (total_sent as f64 / elapsed.as_secs_f64()) / (1024.0 * 1024.0)
        } else {
            0.0
        };

        if errors.is_empty() {
            info!(
                "✅ Transfer complete: '{}' ({} bytes in {:.2}s, {:.1} MB/s)",
                plan.file_name,
                total_sent,
                elapsed.as_secs_f64(),
                throughput_mbps
            );
        } else {
            warn!(
                "⚠️ Transfer completed with {} errors out of {} chunks",
                errors.len(),
                plan.total_chunks
            );
            return Err(errors.remove(0));
        }

        // Fire completion progress callback
        if let Some(ref cb) = progress_cb {
            cb(TransferProgress {
                file_name: plan.file_name,
                file_id: plan.file_id,
                total_bytes: plan.total_size,
                bytes_sent: total_sent,
                chunks_sent: plan.total_chunks,
                total_chunks: plan.total_chunks,
                throughput_bps: (throughput_mbps * 1024.0 * 1024.0) as u64,
                eta_seconds: 0.0,
                complete: true,
            });
        }

        Ok(())
    }
}

/// Send a single chunk over a new QUIC bidirectional stream.
///
/// Protocol:
/// 1. Open a new bidirectional stream
/// 2. Send chunk metadata (length-prefixed JSON)
/// 3. Send chunk data (optionally compressed)
/// 4. Wait for ACK byte from receiver
/// 5. Close the stream
async fn send_chunk(
    connection: &Connection,
    file_path: &Path,
    chunk_meta: &ChunkMeta,
    chunk_size: u64,
    compression_algo: Option<&str>,
) -> Result<u64> {
    // ── Open a new stream for this chunk ──
    let (mut send, mut recv) = connection
        .open_bi()
        .await
        .context("Failed to open chunk stream")?;

    // ── Read chunk data from file ──
    let chunker = FileChunker::new(chunk_size);
    let (data, checksum) = chunker.read_chunk(file_path, chunk_meta).await?;

    // ── Optionally compress the data ──
    let (final_data, is_compressed) = match compression_algo {
        Some("lz4") => {
            let compressed = compression::lz4::compress(&data)?;
            // Only use compression if it actually reduces size
            if compressed.len() < data.len() {
                (compressed, true)
            } else {
                (data, false)
            }
        }
        Some("zstd") => {
            let compressed = compression::zstd::compress(&data)?;
            if compressed.len() < data.len() {
                (compressed, true)
            } else {
                (data, false)
            }
        }
        _ => (data, false),
    };

    // ── Build metadata with checksum ──
    let mut meta = chunk_meta.clone();
    meta.checksum = checksum;

    // ── Send metadata header ──
    // Format: [1 byte type][4 bytes meta_len][meta_json][1 byte compressed_flag][chunk_data]
    let meta_json = serde_json::to_vec(&meta)?;
    let meta_len = (meta_json.len() as u32).to_be_bytes();

    send.write_all(&[0x02]).await?; // Type 2: Chunk Data
    send.write_all(&meta_len).await?;
    send.write_all(&meta_json).await?;

    // Compression flag
    send.write_all(&[is_compressed as u8]).await?;

    // ── Send chunk data ──
    send.write_all(&final_data).await?;
    send.finish()?;

    // ── Wait for ACK ──
    let mut ack = [0u8; 1];
    recv.read_exact(&mut ack)
        .await
        .context("Failed to receive chunk ACK")?;

    if ack[0] != 0x06 {
        anyhow::bail!("Received NAK for chunk {}", chunk_meta.chunk_index);
    }

    Ok(final_data.len() as u64)
}
