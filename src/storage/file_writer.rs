//! # Streaming File Writer
//!
//! Efficient file writer that supports writing chunks in any order
//! and assembling them into the final file. Uses pre-allocated files
//! with sparse seeking for out-of-order chunk assembly.

use std::path::Path;

use anyhow::{Context, Result};
use tokio::fs::File;
use tokio::io::{AsyncSeekExt, AsyncWriteExt, BufWriter, SeekFrom};
use tracing::{debug, info};

/// Write buffer size (256 KB)
const WRITE_BUFFER_SIZE: usize = 256 * 1024;

/// Streaming file writer optimized for large file assembly.
pub struct StreamingFileWriter {
    writer: BufWriter<File>,
    total_size: u64,
    bytes_written: u64,
}

impl StreamingFileWriter {
    /// Create a new file for writing with the expected total size.
    ///
    /// Pre-allocates the file to `total_size` bytes on supported
    /// platforms for better performance.
    pub async fn create(path: &Path, total_size: u64) -> Result<Self> {
        let file = File::create(path)
            .await
            .context("Failed to create output file")?;

        // Pre-allocate file space (platform-specific optimization)
        file.set_len(total_size)
            .await
            .context("Failed to pre-allocate file")?;

        let writer = BufWriter::with_capacity(WRITE_BUFFER_SIZE, file);

        info!(
            "Created output file: {} ({} bytes pre-allocated)",
            path.display(),
            total_size
        );

        Ok(Self {
            writer,
            total_size,
            bytes_written: 0,
        })
    }

    /// Write data at a specific offset (for out-of-order chunk assembly).
    pub async fn write_at(&mut self, offset: u64, data: &[u8]) -> Result<()> {
        self.writer
            .seek(SeekFrom::Start(offset))
            .await
            .context("Failed to seek in output file")?;

        self.writer
            .write_all(data)
            .await
            .context("Failed to write data")?;

        self.bytes_written += data.len() as u64;

        debug!("Wrote {} bytes at offset {}", data.len(), offset);

        Ok(())
    }

    /// Append data at the end of written content (for sequential assembly).
    pub async fn append(&mut self, data: &[u8]) -> Result<()> {
        self.writer
            .write_all(data)
            .await
            .context("Failed to append data")?;

        self.bytes_written += data.len() as u64;

        Ok(())
    }

    /// Flush all buffered data to disk.
    pub async fn flush(&mut self) -> Result<()> {
        self.writer
            .flush()
            .await
            .context("Failed to flush file writer")?;
        Ok(())
    }

    /// Finalize the file — flush and truncate to actual written size.
    pub async fn finalize(mut self) -> Result<()> {
        self.flush().await?;
        info!("File write complete: {} bytes", self.bytes_written);
        Ok(())
    }

    /// Get the number of bytes written so far.
    pub fn bytes_written(&self) -> u64 {
        self.bytes_written
    }

    /// Get the expected total file size.
    pub fn total_size(&self) -> u64 {
        self.total_size
    }

    /// Get the completion percentage.
    pub fn progress(&self) -> f64 {
        if self.total_size == 0 {
            return 100.0;
        }
        (self.bytes_written as f64 / self.total_size as f64) * 100.0
    }
}
