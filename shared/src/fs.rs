//! Filesystem helpers shared across Nethercore tools and runtimes.

use std::path::Path;

use anyhow::{Context, Result};

/// Maximum allowed ROM size for reading into memory.
pub const MAX_ROM_BYTES: u64 = 512 * 1024 * 1024; // 512 MiB
/// Maximum allowed WASM size for reading into memory.
pub const MAX_WASM_BYTES: u64 = 128 * 1024 * 1024; // 128 MiB
/// Maximum allowed PNG size for thumbnails/screenshots.
pub const MAX_PNG_BYTES: u64 = 32 * 1024 * 1024; // 32 MiB

/// Read a file into memory with a size cap.
pub fn read_file_with_limit(path: &Path, max_bytes: u64) -> Result<Vec<u8>> {
    let metadata = std::fs::metadata(path)
        .with_context(|| format!("Failed to read file metadata: {}", path.display()))?;
    let len = metadata.len();
    if len > max_bytes {
        anyhow::bail!(
            "File too large: {} ({} bytes, max {} bytes)",
            path.display(),
            len,
            max_bytes
        );
    }
    std::fs::read(path).with_context(|| format!("Failed to read file: {}", path.display()))
}
