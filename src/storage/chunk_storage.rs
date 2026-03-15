//! # Chunk Storage
//!
//! Temporary storage for file chunks during transfer.
//! Chunks are stored as individual files in a directory structure:
//!
//! ```text
//! temp/
//!   <file_id>/
//!     chunk_000000
//!     chunk_000001
//!     chunk_000002
//!     ...
//! ```
//!
//! This design allows:
//! - Out-of-order chunk reception
//! - Easy resume (just check which chunk files exist)
//! - Memory-efficient (chunks are on disk, not in RAM)

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use tokio::fs;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tracing::{debug, info};

/// Chunk storage manages temporary chunk files on disk.
pub struct ChunkStorage {
    /// Base directory for temporary chunk storage
    base_dir: PathBuf,
}

impl ChunkStorage {
    /// Create a new chunk storage in the system temp directory.
    pub fn new() -> Result<Self> {
        let base_dir = std::env::temp_dir().join("fastshare").join("chunks");
        Ok(Self { base_dir })
    }

    /// Create a new chunk storage at a specific path.
    pub fn with_path(base_dir: PathBuf) -> Self {
        Self { base_dir }
    }

    /// Prepare storage directory for a new file transfer.
    pub async fn prepare_for_file(&self, file_id: &str) -> Result<()> {
        let dir = self.base_dir.join(file_id);
        fs::create_dir_all(&dir)
            .await
            .context("Failed to create chunk storage directory")?;

        debug!("Chunk storage prepared for file: {}", file_id);
        Ok(())
    }

    /// Store a chunk to disk.
    pub async fn store_chunk(&self, file_id: &str, chunk_index: u64, data: &[u8]) -> Result<()> {
        let dir = self.base_dir.join(file_id);

        if !dir.exists() {
            fs::create_dir_all(&dir).await?;
        }

        let chunk_path = dir.join(format!("chunk_{:06}", chunk_index));

        // Faster than create + write_all + flush per chunk
        fs::write(&chunk_path, data)
            .await
            .context("Failed to write chunk data")?;

        debug!(
            "Stored chunk {} ({} bytes) at {}",
            chunk_index,
            data.len(),
            chunk_path.display()
        );
        Ok(())
    }

    /// Read a chunk from disk.
    pub async fn read_chunk(&self, file_id: &str, chunk_index: u64) -> Result<Vec<u8>> {
        let chunk_path = self
            .base_dir
            .join(file_id)
            .join(format!("chunk_{:06}", chunk_index));
        let data = fs::read(&chunk_path)
            .await
            .context("Failed to read chunk file")?;

        debug!("Read chunk {} ({} bytes)", chunk_index, data.len());
        Ok(data)
    }

    /// Check if a specific chunk exists.
    pub async fn has_chunk(&self, file_id: &str, chunk_index: u64) -> bool {
        let chunk_path = self
            .base_dir
            .join(file_id)
            .join(format!("chunk_{:06}", chunk_index));
        chunk_path.exists()
    }

    /// Get the list of received chunk indices for a file.
    pub async fn received_chunks(&self, file_id: &str) -> Result<Vec<u64>> {
        let dir = self.base_dir.join(file_id);
        if !dir.exists() {
            return Ok(Vec::new());
        }

        let mut chunks = Vec::new();
        let mut entries = fs::read_dir(&dir).await?;

        while let Some(entry) = entries.next_entry().await? {
            let name = entry.file_name().to_string_lossy().to_string();
            if let Some(index_str) = name.strip_prefix("chunk_") {
                if let Ok(index) = index_str.parse::<u64>() {
                    chunks.push(index);
                }
            }
        }

        chunks.sort();
        Ok(chunks)
    }

    /// Clean up all chunks for a completed file transfer.
    pub async fn cleanup_file(&self, file_id: &str) -> Result<()> {
        let dir = self.base_dir.join(file_id);
        if dir.exists() {
            fs::remove_dir_all(&dir)
                .await
                .context("Failed to clean up chunk directory")?;
            info!("Cleaned up chunk storage for file: {}", file_id);
        }
        Ok(())
    }

    /// Clean up all chunk storage (e.g., on shutdown).
    pub async fn cleanup_all(&self) -> Result<()> {
        if self.base_dir.exists() {
            fs::remove_dir_all(&self.base_dir)
                .await
                .context("Failed to clean up all chunk storage")?;
        }
        Ok(())
    }

    /// Get the total disk space used by chunks for a file.
    pub async fn disk_usage(&self, file_id: &str) -> Result<u64> {
        let dir = self.base_dir.join(file_id);
        if !dir.exists() {
            return Ok(0);
        }

        let mut total = 0u64;
        let mut entries = fs::read_dir(&dir).await?;

        while let Some(entry) = entries.next_entry().await? {
            if let Ok(metadata) = entry.metadata().await {
                total += metadata.len();
            }
        }

        Ok(total)
    }
}
