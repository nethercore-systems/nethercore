//! Utility helpers for asset loading.

use anyhow::Result;
use sha2::{Digest, Sha256};
use zx_common::TrackerFormat;

use crate::manifest::AssetEntry;

/// Get required ID from asset entry, or error if missing
pub fn require_id<'a>(entry: &'a AssetEntry, asset_type: &str) -> Result<&'a str> {
    entry.id.as_deref().ok_or_else(|| {
        anyhow::anyhow!(
            "{} asset at '{}' is missing required 'id' field",
            asset_type,
            entry.path
        )
    })
}

/// Detect tracker format by magic bytes
pub fn detect_tracker_format(data: &[u8]) -> Option<TrackerFormat> {
    // Check for XM magic: "Extended Module: " (17 bytes)
    if data.len() >= 17 && &data[0..17] == b"Extended Module: " {
        return Some(TrackerFormat::Xm);
    }
    // Check for IT magic: "IMPM" (4 bytes)
    if data.len() >= 4 && &data[0..4] == b"IMPM" {
        return Some(TrackerFormat::It);
    }
    None
}

/// Sanitize XM instrument name to valid sound ID
///
/// Converts instrument names like "  My Kick!  " to "my_kick"
/// Empty names are auto-generated from tracker ID and instrument index
pub fn sanitize_name(name: &str, tracker_id: &str, index: u8) -> String {
    let trimmed = name.trim();

    if trimmed.is_empty() {
        return format!("{}_inst{}", tracker_id, index);
    }

    let sanitized = trimmed
        .to_lowercase()
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '_' || c == '-' {
                c
            } else {
                '_'
            }
        })
        .collect::<String>();

    // Collapse consecutive underscores and trim leading/trailing underscores
    let mut result = String::new();
    let mut prev_was_underscore = false;

    for c in sanitized.chars() {
        if c == '_' {
            if !prev_was_underscore {
                result.push(c);
            }
            prev_was_underscore = true;
        } else {
            result.push(c);
            prev_was_underscore = false;
        }
    }

    result.trim_matches('_').to_string()
}

/// Calculate SHA-256 hash of sample data for deduplication
pub fn hash_sample_data(data: &[i16]) -> [u8; 32] {
    let mut hasher = Sha256::new();

    // Hash the sample data as bytes
    let bytes = bytemuck::cast_slice::<i16, u8>(data);
    hasher.update(bytes);

    hasher.finalize().into()
}
