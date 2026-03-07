//! # LZ4 Compression
//!
//! LZ4 provides extremely fast compression and decompression speeds,
//! making it ideal for high-throughput file transfers where CPU time
//! is the bottleneck rather than network bandwidth.
//!
//! LZ4 is preferred when:
//! - Network speed > 1 Gbps
//! - Low CPU overhead is critical
//! - Compression ratio is secondary to speed
//!
//! Typical performance:
//! - Compression: 500-800 MB/s per core
//! - Decompression: 1500-2000 MB/s per core
//! - Ratio: ~2:1 for typical data

use anyhow::{Context, Result};

/// Compress data using LZ4.
///
/// Returns the compressed bytes. If compression doesn't reduce
/// the size, the caller should send uncompressed data instead.
pub fn compress(data: &[u8]) -> Result<Vec<u8>> {
    let compressed = lz4_flex::compress_prepend_size(data);
    Ok(compressed)
}

/// Decompress LZ4-compressed data.
///
/// # Arguments
/// * `data` — Compressed data
/// * `max_output_size` — Maximum expected decompressed size (safety limit)
pub fn decompress(data: &[u8], max_output_size: usize) -> Result<Vec<u8>> {
    let decompressed = lz4_flex::decompress_size_prepended(data)
        .map_err(|e| anyhow::anyhow!("LZ4 decompression failed: {}", e))?;

    if decompressed.len() > max_output_size {
        anyhow::bail!(
            "LZ4 decompressed size {} exceeds maximum {}",
            decompressed.len(),
            max_output_size
        );
    }

    Ok(decompressed)
}

/// Get the compression ratio for a piece of data.
/// Returns a value < 1.0 if compression is beneficial.
pub fn compression_ratio(original_size: usize, compressed_size: usize) -> f64 {
    compressed_size as f64 / original_size as f64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compress_decompress() {
        let data = b"Hello, FastShare! ".repeat(1000);
        let compressed = compress(&data).unwrap();
        let decompressed = decompress(&compressed, data.len()).unwrap();
        assert_eq!(data.as_slice(), decompressed.as_slice());
    }

    #[test]
    fn test_compression_reduces_size() {
        let data = b"AAAAAAAAAA".repeat(10000);
        let compressed = compress(&data).unwrap();
        assert!(compressed.len() < data.len());
    }
}
