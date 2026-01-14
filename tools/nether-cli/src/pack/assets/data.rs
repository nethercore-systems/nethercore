//! Raw data file loading.

use anyhow::{Context, Result};
use zx_common::PackedData;

/// Load raw data from file
pub fn load_data(id: &str, path: &std::path::Path) -> Result<PackedData> {
    let data =
        std::fs::read(path).with_context(|| format!("Failed to load data: {}", path.display()))?;

    Ok(PackedData {
        id: id.to_string(),
        data,
    })
}
