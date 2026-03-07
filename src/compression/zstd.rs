//! # ZSTD Compression
//!
//! ZSTD (Zstandard) provides a better compression ratio than LZ4
//! at the cost of slightly higher CPU usage. It's preferred for:
//!
//! - Slower networks (< 1 Gbps) where bandwidth savings matter
//! - Highly compressible data (text, logs, databases)
//! - When transfer time is dominated by network, not CPU
//!
//! Typical performance:
//! - Compression: 100-400 MB/s per core (level dependent)
//! - Decompression: 600-1000 MB/s per core
//! - Ratio: ~3:1 or better for typical data

use anyhow::{Context, Result};

/// Default ZSTD compression level.
/// Level 3 provides a good balance of speed and ratio.
const DEFAULT_LEVEL: i32 = 3;

/// Compress data using ZSTD.
///
/// Uses the default compression level (3) which provides
/// a good balance between compression speed and ratio.
pub fn compress(data: &[u8]) -> Result<Vec<u8>> {
    let compressed = zstd::encode_all(data, DEFAULT_LEVEL).context("ZSTD compression failed")?;
    Ok(compressed)
}

/// Compress data with a specific compression level (1-22).
///
/// Higher levels provide better compression at the cost of speed:
/// - Level 1-3: Fast compression (good for real-time)
/// - Level 4-9: Balanced
/// - Level 10-22: Maximum compression (good for archival)
pub fn compress_with_level(data: &[u8], level: i32) -> Result<Vec<u8>> {
    let level = level.clamp(1, 22);
    let compressed = zstd::encode_all(data, level).context("ZSTD compression failed")?;
    Ok(compressed)
}

/// Decompress ZSTD-compressed data.
pub fn decompress(data: &[u8]) -> Result<Vec<u8>> {
    let decompressed = zstd::decode_all(data).context("ZSTD decompression failed")?;
    Ok(decompressed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compress_decompress() {
        let data = b"The quick brown fox jumps over the lazy dog. ".repeat(1000);
        let compressed = compress(&data).unwrap();
        let decompressed = decompress(&compressed).unwrap();
        assert_eq!(data.as_slice(), decompressed.as_slice());
    }

    #[test]
    fn test_compression_levels() {
        let data = b"FastShare distributed transfer! ".repeat(10000);
        let fast = compress_with_level(&data, 1).unwrap();
        let balanced = compress_with_level(&data, 6).unwrap();
        let max = compress_with_level(&data, 19).unwrap();

        // Higher levels should produce smaller output
        assert!(max.len() <= balanced.len());
        assert!(balanced.len() <= fast.len());
    }
}
