//! # Transfer Resume System
//!
//! Persists transfer state to disk so that interrupted transfers can
//! be resumed from where they left off. This is critical for large
//! file transfers (100GB+) where re-transferring from scratch would
//! waste significant time and bandwidth.
//!
//! ## State Format
//!
//! Transfer state is stored as JSON in:
//! `~/.fastshare/resume/<file_id>.json`
//!
//! It tracks which chunks have been successfully received and verified.

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use tokio::fs;
use tracing::{debug, info};

// ── Data Structures ──

/// Persistent transfer state for resume capability.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferState {
    /// Unique file transfer identifier
    pub file_id: String,
    /// Original file name
    pub file_name: String,
    /// Total file size in bytes
    pub total_size: u64,
    /// Chunk size used for the transfer
    pub chunk_size: u64,
    /// Total number of chunks
    pub total_chunks: u64,
    /// Set of chunk indices that have been successfully received
    pub received_chunks: HashSet<u64>,
    /// Remote device ID
    pub remote_device_id: String,
    /// Timestamp when the transfer started
    pub started_at: String,
    /// Timestamp of the last update
    pub updated_at: String,
}

impl TransferState {
    /// Create a new transfer state for a fresh transfer.
    pub fn new(
        file_id: String,
        file_name: String,
        total_size: u64,
        chunk_size: u64,
        total_chunks: u64,
        remote_device_id: String,
    ) -> Self {
        let now = chrono::Utc::now().to_rfc3339();
        Self {
            file_id,
            file_name,
            total_size,
            chunk_size,
            total_chunks,
            received_chunks: HashSet::new(),
            remote_device_id,
            started_at: now.clone(),
            updated_at: now,
        }
    }

    /// Mark a chunk as successfully received.
    pub fn mark_chunk_received(&mut self, chunk_index: u64) {
        self.received_chunks.insert(chunk_index);
        self.updated_at = chrono::Utc::now().to_rfc3339();
    }

    /// Check if the transfer is complete (all chunks received).
    pub fn is_complete(&self) -> bool {
        self.received_chunks.len() as u64 >= self.total_chunks
    }

    /// Get the list of missing chunk indices.
    pub fn missing_chunks(&self) -> Vec<u64> {
        (0..self.total_chunks)
            .filter(|i| !self.received_chunks.contains(i))
            .collect()
    }

    /// Get the completion percentage.
    pub fn progress_percent(&self) -> f64 {
        if self.total_chunks == 0 {
            return 100.0;
        }
        (self.received_chunks.len() as f64 / self.total_chunks as f64) * 100.0
    }

    /// Get the number of bytes received so far.
    pub fn bytes_received(&self) -> u64 {
        let full_chunks = self.received_chunks.len().saturating_sub(1) as u64;
        let last_chunk_size = self.total_size % self.chunk_size;
        let has_last = self.received_chunks.contains(&(self.total_chunks - 1));

        let full_bytes = full_chunks * self.chunk_size;
        if has_last && last_chunk_size > 0 {
            full_bytes + last_chunk_size
        } else {
            full_bytes + if has_last { self.chunk_size } else { 0 }
        }
    }
}

/// The resume manager handles persisting and loading transfer states.
pub struct ResumeManager {
    /// Directory where resume states are saved
    state_dir: PathBuf,
}

impl ResumeManager {
    /// Create a new resume manager.
    ///
    /// Creates the state directory if it doesn't exist.
    pub async fn new() -> Result<Self> {
        let state_dir = Self::get_state_dir()?;
        fs::create_dir_all(&state_dir)
            .await
            .context("Failed to create resume state directory")?;

        Ok(Self { state_dir })
    }

    /// Get the resume state directory path.
    fn get_state_dir() -> Result<PathBuf> {
        let home = dirs::data_local_dir()
            .or_else(|| dirs::home_dir())
            .context("Could not determine home directory")?;

        Ok(home.join(".fastshare").join("resume"))
    }

    /// Save a transfer state to disk.
    pub async fn save_state(&self, state: &TransferState) -> Result<()> {
        let path = self.state_dir.join(format!("{}.json", state.file_id));
        let json =
            serde_json::to_string_pretty(state).context("Failed to serialize transfer state")?;

        fs::write(&path, json)
            .await
            .context("Failed to write resume state")?;

        debug!(
            "Resume state saved for '{}' ({:.1}% complete)",
            state.file_name,
            state.progress_percent()
        );

        Ok(())
    }

    /// Load a transfer state from disk.
    pub async fn load_state(&self, file_id: &str) -> Result<Option<TransferState>> {
        let path = self.state_dir.join(format!("{}.json", file_id));

        if !path.exists() {
            return Ok(None);
        }

        let json = fs::read_to_string(&path)
            .await
            .context("Failed to read resume state")?;
        let state: TransferState =
            serde_json::from_str(&json).context("Failed to deserialize resume state")?;

        info!(
            "📋 Loaded resume state for '{}': {}/{} chunks ({:.1}%)",
            state.file_name,
            state.received_chunks.len(),
            state.total_chunks,
            state.progress_percent()
        );

        Ok(Some(state))
    }

    /// Delete a transfer state (called after successful completion).
    pub async fn delete_state(&self, file_id: &str) -> Result<()> {
        let path = self.state_dir.join(format!("{}.json", file_id));
        if path.exists() {
            fs::remove_file(&path)
                .await
                .context("Failed to delete resume state")?;
        }
        Ok(())
    }

    /// List all incomplete transfers.
    pub async fn list_incomplete(&self) -> Result<Vec<TransferState>> {
        let mut states = Vec::new();

        let mut entries = fs::read_dir(&self.state_dir).await?;
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.extension().map_or(false, |ext| ext == "json") {
                if let Ok(json) = fs::read_to_string(&path).await {
                    if let Ok(state) = serde_json::from_str::<TransferState>(&json) {
                        if !state.is_complete() {
                            states.push(state);
                        }
                    }
                }
            }
        }

        Ok(states)
    }
}
