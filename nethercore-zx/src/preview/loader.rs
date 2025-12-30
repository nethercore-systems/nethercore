//! ZX ROM loader for preview mode
//!
//! Loads Nethercore ZX ROM files and extracts data packs for asset preview.

use std::path::Path;

use anyhow::{Context, Result};
use zx_common::{ZXDataPack, ZXRom};

use nethercore_core::app::preview::{PreviewData as CorePreviewData, PreviewRomLoader as CorePreviewRomLoader};
use crate::console::NethercoreZX;
use super::viewers::ZXAssetViewer;
use super::{PreviewData, PreviewMetadata};

/// ROM loader for ZX preview mode
///
/// Handles loading `.nczx` ROM files and extracting their data packs
/// for inspection in the asset preview UI.
pub struct ZXPreviewLoader;

impl ZXPreviewLoader {
    /// Load a ROM file for preview
    ///
    /// Reads the ROM, parses it, and extracts the data pack and metadata
    /// needed for the preview UI.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the `.nczx` ROM file
    ///
    /// # Returns
    ///
    /// A `PreviewData` containing the data pack and metadata
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The file cannot be read
    /// - The ROM format is invalid
    /// - The ROM fails validation
    pub fn load_for_preview(path: &Path) -> Result<PreviewData<ZXDataPack>> {
        // Read the ROM file
        let rom_bytes = std::fs::read(path)
            .with_context(|| format!("Failed to read ROM file: {}", path.display()))?;

        // Parse the ROM
        let rom = ZXRom::from_bytes(&rom_bytes)
            .with_context(|| format!("Failed to parse ROM: {}", path.display()))?;

        // Extract the data pack (use default if none bundled)
        let data_pack = rom.data_pack.unwrap_or_default();

        // Extract metadata
        let metadata = PreviewMetadata {
            id: rom.metadata.id,
            title: rom.metadata.title,
            author: rom.metadata.author,
            version: rom.metadata.version,
        };

        Ok(PreviewData {
            data_pack,
            metadata,
        })
    }

    /// Check if a file is a valid ZX ROM without fully loading it
    ///
    /// This performs a quick magic byte check to validate the file format.
    pub fn is_valid_rom(path: &Path) -> bool {
        let Ok(file) = std::fs::File::open(path) else {
            return false;
        };

        use std::io::Read;
        let mut magic = [0u8; 4];
        let mut reader = std::io::BufReader::new(file);

        if reader.read_exact(&mut magic).is_err() {
            return false;
        }

        // Check for NCZX magic bytes
        &magic == b"NCZX"
    }

    /// Get basic ROM info without loading the full data pack
    ///
    /// This is useful for listing ROMs in a directory without
    /// loading all their assets into memory.
    pub fn peek_metadata(path: &Path) -> Result<PreviewMetadata> {
        let rom_bytes = std::fs::read(path)
            .with_context(|| format!("Failed to read ROM file: {}", path.display()))?;

        let rom = ZXRom::from_bytes(&rom_bytes)
            .with_context(|| format!("Failed to parse ROM: {}", path.display()))?;

        Ok(PreviewMetadata {
            id: rom.metadata.id,
            title: rom.metadata.title,
            author: rom.metadata.author,
            version: rom.metadata.version,
        })
    }
}

// Implement the core PreviewRomLoader trait
impl CorePreviewRomLoader for ZXPreviewLoader {
    type Console = NethercoreZX;
    type DataPack = ZXDataPack;
    type Viewer = ZXAssetViewer;

    fn load_for_preview(path: &Path) -> Result<CorePreviewData<Self::DataPack>> {
        // Delegate to the existing implementation
        let preview_data = ZXPreviewLoader::load_for_preview(path)?;

        // Convert to core PreviewData
        Ok(CorePreviewData {
            data_pack: preview_data.data_pack,
            metadata: nethercore_core::app::preview::PreviewMetadata {
                id: preview_data.metadata.id,
                title: preview_data.metadata.title,
                author: preview_data.metadata.author,
                version: preview_data.metadata.version,
            },
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_valid_rom_nonexistent() {
        assert!(!ZXPreviewLoader::is_valid_rom(Path::new("/nonexistent/path.nczx")));
    }
}
