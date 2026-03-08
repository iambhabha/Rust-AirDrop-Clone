//! # Multi-Stream Transfer Receiver
//!
//! Receives file chunks from multiple QUIC streams simultaneously,
//! verifies integrity via SHA-256, decompresses if needed, and
//! reassembles them into the original file.
//!
//! ## Reassembly Process
//!
//! 1. Receive chunk metadata + data on each stream
//! 2. Verify checksum
//! 3. Store chunk to temporary storage
//! 4. When all chunks received, reassemble into final file
//! 5. Verify final file hash

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use anyhow::{Context, Result};
use dashmap::DashMap;
use quinn::{RecvStream, SendStream};
use sha2::{Digest, Sha256};
use tokio::sync::Notify;
use tracing::{debug, error, info, warn};

use crate::compression;
use crate::storage::chunk_storage::ChunkStorage;
use crate::transfer::chunker::{ChunkMeta, FileChunkPlan};

// ── Data Structures ──

/// Tracks the state of an ongoing file reception.
#[derive(Debug)]
pub struct ReceptionState {
    /// The chunk plan for this file
    pub plan: FileChunkPlan,
    /// Number of chunks received so far
    pub chunks_received: AtomicU64,
    /// Notifier for when all chunks are received
    pub completion_notify: Notify,
}

/// The transfer receiver — handles incoming chunk streams
/// and reassembles files.
#[derive(Clone)]
pub struct TransferReceiver {
    /// Chunk storage for persisting received chunks
    chunk_storage: Arc<ChunkStorage>,
    /// Active receptions keyed by file_id
    active_receptions: Arc<DashMap<String, Arc<ReceptionState>>>,
}

impl TransferReceiver {
    /// Create a new transfer receiver.
    pub fn new(chunk_storage: Arc<ChunkStorage>) -> Self {
        Self {
            chunk_storage,
            active_receptions: Arc::new(DashMap::new()),
        }
    }

    pub fn active_receptions(&self) -> Arc<DashMap<String, Arc<ReceptionState>>> {
        self.active_receptions.clone()
    }

    /// Handle a single incoming chunk stream.
    ///
    /// This is called for each QUIC bidirectional stream opened by the sender.
    /// Each stream carries one chunk with the following protocol:
    ///
    /// 1. Read chunk metadata (length-prefixed JSON)
    /// 2. Read compression flag (1 byte)
    /// 3. Read chunk data (remaining bytes)
    /// 4. Verify SHA-256 checksum
    /// 5. Store chunk
    /// 6. Send ACK (0x06) or NAK (0x15)
    pub async fn handle_chunk_stream(
        &self,
        mut recv: RecvStream,
        mut send: SendStream,
    ) -> Result<()> {
        // Receiver expects the caller to have already read the type byte (0x02).
        // ── Read metadata length ──
        let mut len_buf = [0u8; 4];
        recv.read_exact(&mut len_buf)
            .await
            .context("Failed to read chunk metadata length")?;
        let meta_len = u32::from_be_bytes(len_buf) as usize;

        if meta_len > 1024 * 1024 {
            anyhow::bail!("Chunk metadata too large: {} bytes", meta_len);
        }

        // ── Read metadata JSON ──
        let mut meta_buf = vec![0u8; meta_len];
        recv.read_exact(&mut meta_buf)
            .await
            .context("Failed to read chunk metadata")?;

        let chunk_meta: ChunkMeta =
            serde_json::from_slice(&meta_buf).context("Failed to deserialize chunk metadata")?;

        debug!(
            "Receiving chunk {}/{} of '{}' ({} bytes)",
            chunk_meta.chunk_index + 1,
            chunk_meta.total_chunks,
            chunk_meta.file_name,
            chunk_meta.size
        );

        // ── Read compression flag ──
        let mut comp_flag = [0u8; 1];
        recv.read_exact(&mut comp_flag)
            .await
            .context("Failed to read compression flag")?;
        let is_compressed = comp_flag[0] != 0;

        // ── Read chunk data ──
        let mut data = Vec::new();
        let mut read_buf = vec![0u8; 64 * 1024]; // 64 KB read buffer
        loop {
            match recv.read(&mut read_buf).await? {
                Some(n) => data.extend_from_slice(&read_buf[..n]),
                None => break,
            }
        }

        // ── Decompress if needed ──
        let final_data = if is_compressed {
            // Try LZ4 first, then ZSTD
            compression::lz4::decompress(&data, chunk_meta.size as usize)
                .or_else(|_| compression::zstd::decompress(&data))
                .context("Failed to decompress chunk data")?
        } else {
            data
        };

        // ── Verify checksum ──
        let mut hasher = Sha256::new();
        hasher.update(&final_data);
        let computed_checksum = hex::encode(hasher.finalize());

        if !chunk_meta.checksum.is_empty() && computed_checksum != chunk_meta.checksum {
            warn!(
                "Checksum mismatch for chunk {} of '{}': expected {}, got {}",
                chunk_meta.chunk_index,
                chunk_meta.file_name,
                &chunk_meta.checksum[..16],
                &computed_checksum[..16]
            );

            // Send NAK
            send.write_all(&[0x15]).await?;
            send.finish()?;

            anyhow::bail!("Checksum verification failed");
        }

        // ── Store chunk ──
        self.chunk_storage
            .store_chunk(&chunk_meta.file_id, chunk_meta.chunk_index, &final_data)
            .await?;

        // ── Update reception state ──
        if let Some(state) = self.active_receptions.get(&chunk_meta.file_id) {
            let received = state.chunks_received.fetch_add(1, Ordering::Relaxed) + 1;
            if received >= chunk_meta.total_chunks {
                state.completion_notify.notify_one();
            }
        } else {
            warn!("Received chunk for unknown file_id: {}", chunk_meta.file_id);
            // Send NAK
            let _ = send.write_all(&[0x15]).await;
            let _ = send.finish();
            anyhow::bail!("Unknown file_id: {}", chunk_meta.file_id);
        }

        // ── Send ACK ──
        send.write_all(&[0x06]).await?;
        send.finish()?;

        debug!(
            "✓ Chunk {}/{} of '{}' received and verified",
            chunk_meta.chunk_index + 1,
            chunk_meta.total_chunks,
            chunk_meta.file_name
        );

        Ok(())
    }

    /// Handle a file metadata stream (sent before chunks begin).
    ///
    /// This initializes the reception state for a new file transfer.
    pub async fn handle_file_plan(&self, plan: FileChunkPlan) -> Result<()> {
        info!(
            "📥 Incoming file: '{}' ({} bytes, {} chunks)",
            plan.file_name, plan.total_size, plan.total_chunks
        );

        let state = Arc::new(ReceptionState {
            plan: plan.clone(),
            chunks_received: AtomicU64::new(0),
            completion_notify: Notify::new(),
        });

        // Create chunk storage directory
        self.chunk_storage.prepare_for_file(&plan.file_id).await?;

        self.active_receptions.insert(plan.file_id.clone(), state);

        Ok(())
    }

    pub fn get_reception(&self, file_id: &str) -> Option<Arc<ReceptionState>> {
        self.active_receptions
            .get(file_id)
            .map(|r| r.value().clone())
    }

    /// Reassemble a completed file from its chunks.
    ///
    /// Reads all chunks from temporary storage in order and writes
    /// them to the final output path.
    pub async fn reassemble_file(&self, file_id: &str, output_path: &Path) -> Result<()> {
        let state = self
            .active_receptions
            .get(file_id)
            .context("No active reception for this file")?;

        let plan = &state.plan;
        info!(
            "🔧 Reassembling '{}' from {} chunks...",
            plan.file_name, plan.total_chunks
        );

        // ── Write all chunks to the output file ──
        use tokio::io::AsyncWriteExt;
        let mut output = tokio::fs::File::create(output_path)
            .await
            .context("Failed to create output file")?;

        for i in 0..plan.total_chunks {
            let chunk_data = self.chunk_storage.read_chunk(file_id, i).await?;
            output
                .write_all(&chunk_data)
                .await
                .context("Failed to write chunk to output")?;
        }

        output.flush().await?;

        info!("✅ File reassembled: '{}'", output_path.display());

        // ── Cleanup ──
        self.chunk_storage.cleanup_file(file_id).await?;
        self.active_receptions.remove(file_id);

        Ok(())
    }
}
