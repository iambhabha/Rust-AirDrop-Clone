//! # Streaming File Reader
//!
//! Zero-copy streaming file reader optimized for very large files (100GB+).
//! Uses async I/O to read file data without loading the entire file into memory.

use std::path::Path;

use anyhow::{Context, Result};
use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncSeekExt, BufReader, SeekFrom};
use tracing::debug;

/// Read buffer size (256 KB for optimal I/O throughput)
const READ_BUFFER_SIZE: usize = 256 * 1024;

/// Streaming file reader that reads data in chunks without
/// loading the entire file into memory.
pub struct StreamingFileReader {
    reader: BufReader<File>,
    file_size: u64,
    position: u64,
}

impl StreamingFileReader {
    /// Open a file for streaming reads.
    pub async fn open(path: &Path) -> Result<Self> {
        let file = File::open(path)
            .await
            .context("Failed to open file for reading")?;
        let metadata = file
            .metadata()
            .await
            .context("Failed to read file metadata")?;
        let file_size = metadata.len();

        let reader = BufReader::with_capacity(READ_BUFFER_SIZE, file);

        Ok(Self {
            reader,
            file_size,
            position: 0,
        })
    }

    /// Read a specific range of bytes from the file.
    ///
    /// Seeks to `offset` and reads `length` bytes.
    /// Returns the actual bytes read (may be less at EOF).
    pub async fn read_range(&mut self, offset: u64, length: usize) -> Result<Vec<u8>> {
        self.reader
            .seek(SeekFrom::Start(offset))
            .await
            .context("Failed to seek in file")?;
        self.position = offset;

        let mut buffer = vec![0u8; length];
        let bytes_read = self
            .reader
            .read(&mut buffer)
            .await
            .context("Failed to read from file")?;

        buffer.truncate(bytes_read);
        self.position += bytes_read as u64;

        debug!("Read {} bytes from offset {}", bytes_read, offset);

        Ok(buffer)
    }

    /// Read the next `length` bytes from the current position.
    pub async fn read_next(&mut self, length: usize) -> Result<Vec<u8>> {
        let mut buffer = vec![0u8; length];
        let bytes_read = self
            .reader
            .read(&mut buffer)
            .await
            .context("Failed to read from file")?;

        buffer.truncate(bytes_read);
        self.position += bytes_read as u64;

        Ok(buffer)
    }

    /// Get the total file size.
    pub fn file_size(&self) -> u64 {
        self.file_size
    }

    /// Get the current read position.
    pub fn position(&self) -> u64 {
        self.position
    }

    /// Check if we've reached the end of the file.
    pub fn is_eof(&self) -> bool {
        self.position >= self.file_size
    }

    /// Get the remaining bytes to read.
    pub fn remaining(&self) -> u64 {
        self.file_size.saturating_sub(self.position)
    }
}
