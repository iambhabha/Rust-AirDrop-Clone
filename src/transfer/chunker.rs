//! # File Chunker
//!
//! Splits files into chunks for parallel transfer across multiple QUIC streams.
//! Each chunk includes metadata (index, checksum, size) for integrity verification
//! and reassembly on the receiver side.
//!
//! ## Adaptive Chunk Sizing
//!
//! | Network Speed | Chunk Size |
//! |---------------|-----------|
//! | < 100 Mbps    | 1 MB      |
//! | 100 - 999 Mbps| 2 MB      |
//! | 1 - 5 Gbps    | 4 MB      |
//! | > 5 Gbps      | 6 MB     |
//!
//! Larger chunks reduce per-chunk overhead. Checksum uses fast CRC32 instead
//! of SHA-256 for maximum throughput on local networks.

use std::path::Path;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncSeekExt, SeekFrom};
use tracing::{debug, info};
use uuid::Uuid;

// ── Constants ──

/// Default chunk size: 8 MB (Standard for modern WiFi)
pub const DEFAULT_CHUNK_SIZE: u64 = 8 * 1024 * 1024;

/// Fast network chunk size: 16 MB (Gbit networks)
pub const FAST_CHUNK_SIZE: u64 = 16 * 1024 * 1024;

/// Ultra-fast network chunk size: 32 MB (>5 Gbps or local loopback)
pub const ULTRA_FAST_CHUNK_SIZE: u64 = 32 * 1024 * 1024;

/// Slow network chunk size: 4 MB (Slow WiFi or mobile)
pub const SLOW_CHUNK_SIZE: u64 = 4 * 1024 * 1024;

// ── Data Structures ──

/// Network speed classification for adaptive chunk sizing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NetworkSpeed {
    Slow,      // < 100 Mbps
    Normal,    // 100 - 999 Mbps
    Fast,      // 1 - 5 Gbps
    UltraFast, // > 5 Gbps
}

/// Metadata about a single file chunk.
///
/// This is sent alongside (or before) the chunk data so the receiver
/// knows where to place it and can verify integrity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkMeta {
    /// Unique file transfer identifier
    pub file_id: String,
    /// Original file name
    pub file_name: String,
    /// Total file size in bytes
    pub total_file_size: u64,
    /// Index of this chunk (0-based)
    pub chunk_index: u64,
    /// Total number of chunks in the file
    pub total_chunks: u64,
    /// Byte offset of this chunk within the file
    pub offset: u64,
    /// Size of this chunk in bytes
    pub size: u64,
    /// SHA-256 checksum of this chunk's data
    pub checksum: String,
}

/// Information about a file that has been split into chunks.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileChunkPlan {
    /// Unique transfer identifier
    pub file_id: String,
    /// Original file name
    pub file_name: String,
    /// Total file size
    pub total_size: u64,
    /// Total number of files in this transfer session
    pub total_files: u32,
    /// Total size of all files in this batch
    pub total_batch_size: u64,
    /// Bytes sent in previous files of this batch
    pub batch_bytes_already_sent: u64,
    /// 1-based index of this file in the batch
    pub current_file_index: u32,
    /// Chunk size used for splitting
    pub chunk_size: u64,
    /// Total number of chunks
    pub total_chunks: u64,
    /// Metadata for each chunk
    pub chunks: Vec<ChunkMeta>,
}

/// The file chunker — splits files into chunks with checksums.
pub struct FileChunker {
    chunk_size: u64,
}

impl FileChunker {
    /// Create a new chunker with the specified chunk size.
    pub fn new(chunk_size: u64) -> Self {
        Self { chunk_size }
    }

    /// Create a chunker with adaptive chunk sizing based on network speed.
    pub fn adaptive(speed: NetworkSpeed) -> Self {
        let chunk_size = match speed {
            NetworkSpeed::Slow => SLOW_CHUNK_SIZE,
            NetworkSpeed::Normal => DEFAULT_CHUNK_SIZE,
            NetworkSpeed::Fast => FAST_CHUNK_SIZE,
            NetworkSpeed::UltraFast => ULTRA_FAST_CHUNK_SIZE,
        };
        Self { chunk_size }
    }

    /// Plan the chunking of a file without reading its contents.
    ///
    /// Returns a `FileChunkPlan` with metadata for each chunk.
    /// The actual chunk data is read on-demand during transfer
    /// to avoid loading large files into memory.
    pub async fn plan_file(
        &self,
        path: &Path,
        total_files: u32,
        current_file_index: u32,
        total_batch_size: u64,
        batch_bytes_already_sent: u64,
    ) -> Result<FileChunkPlan> {
        let metadata = tokio::fs::metadata(path)
            .await
            .context("Failed to read file metadata")?;
        let total_size = metadata.len();
        let file_name = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "unnamed".into());

        let file_id = Uuid::new_v4().to_string();
        let total_chunks = (total_size + self.chunk_size - 1) / self.chunk_size;

        info!(
            "📦 Chunking file '{}' ({} bytes) into {} chunks of {} bytes",
            file_name, total_size, total_chunks, self.chunk_size
        );

        // Calculate chunk metadata (checksums computed lazily during transfer)
        let mut chunks = Vec::with_capacity(total_chunks as usize);
        for i in 0..total_chunks {
            let offset = i * self.chunk_size;
            let size = std::cmp::min(self.chunk_size, total_size - offset);

            chunks.push(ChunkMeta {
                file_id: file_id.clone(),
                file_name: file_name.clone(),
                total_file_size: total_size,
                chunk_index: i,
                total_chunks,
                offset,
                size,
                checksum: String::new(), // Computed during read_chunk
            });
        }

        Ok(FileChunkPlan {
            file_id,
            file_name,
            total_size,
            total_files,
            total_batch_size,
            chunk_size: self.chunk_size,
            total_chunks,
            chunks,
            batch_bytes_already_sent,
            current_file_index,
        })
    }

    /// Read a specific chunk from a file and compute a fast CRC32 checksum.
    ///
    /// Uses seeking to read only the required bytes — never loads the
    /// entire file into memory. This is critical for 100GB+ files.
    ///
    /// Uses CRC32 instead of SHA-256 for ~10x faster checksumming on local
    /// networks where speed matters more than cryptographic integrity.
    pub async fn read_chunk(&self, path: &Path, chunk: &ChunkMeta) -> Result<(Vec<u8>, String)> {
        let mut file = File::open(path)
            .await
            .context("Failed to open file for chunk reading")?;

        // Seek to the chunk offset
        file.seek(SeekFrom::Start(chunk.offset))
            .await
            .context("Failed to seek to chunk offset")?;

        // Read chunk data
        let mut buffer = vec![0u8; chunk.size as usize];
        file.read_exact(&mut buffer)
            .await
            .context("Failed to read chunk data")?;

        // Compute CRC32 checksum ONLY if enabled in settings (skip = max speed)
        let checksum = if crate::is_checksum_enabled() {
            let cs = format!("{:08x}", crc32fast::hash(&buffer));
            debug!(
                "Read chunk {}/{}: {} bytes, crc32: {}",
                chunk.chunk_index + 1,
                chunk.total_chunks,
                buffer.len(),
                &cs
            );
            cs
        } else {
            debug!(
                "Read chunk {}/{}: {} bytes, checksum skipped (max speed mode)",
                chunk.chunk_index + 1,
                chunk.total_chunks,
                buffer.len(),
            );
            String::new() // Empty = receiver also skips verification
        };

        Ok((buffer, checksum))
    }

    /// Compute the CRC32 checksum of the entire file for final verification.
    pub async fn compute_file_hash(path: &Path) -> Result<String> {
        let mut file = File::open(path).await?;
        let mut hasher = crc32fast::Hasher::new();
        let mut buffer = vec![0u8; 4 * 1024 * 1024]; // 4 MB read buffer

        loop {
            let bytes_read = file.read(&mut buffer).await?;
            if bytes_read == 0 {
                break;
            }
            hasher.update(&buffer[..bytes_read]);
        }

        Ok(format!("{:08x}", hasher.finalize()))
    }

    /// Get the chunk size being used.
    pub fn chunk_size(&self) -> u64 {
        self.chunk_size
    }
}

/// Classify network speed in Mbps to a NetworkSpeed enum.
pub fn classify_network_speed(mbps: u64) -> NetworkSpeed {
    if mbps < 50 {
        NetworkSpeed::Slow
    } else if mbps < 500 {
        NetworkSpeed::Normal
    } else if mbps < 2000 {
        NetworkSpeed::Fast
    } else {
        NetworkSpeed::UltraFast
    }
}
