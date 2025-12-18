//! ROM loading abstraction
//!
//! This module provides a trait for console-specific ROM loaders,
//! allowing the core library to work with any console's ROM format
//! without depending on console-specific crates.

use std::path::Path;

use anyhow::Result;

use super::{DataDirProvider, LocalGame};

/// Metadata extracted from a ROM file.
#[derive(Debug, Clone)]
pub struct RomMetadata {
    /// Unique game identifier
    pub id: String,
    /// Display title of the game
    pub title: String,
    /// Game author's name
    pub author: String,
    /// Version string
    pub version: String,
}

/// Trait for console-specific ROM loaders.
///
/// Each console type implements this trait to handle its own ROM format.
/// This allows the core library to work with any console's ROMs without
/// depending on console-specific crates.
///
/// # Example
///
/// ```ignore
/// // In zx-common crate:
/// pub struct ZRomLoader;
///
/// impl RomLoader for ZRomLoader {
///     fn extension(&self) -> &'static str { "ewzx" }
///     fn console_type(&self) -> &'static str { "zx" }
///     // ...
/// }
/// ```
pub trait RomLoader: Send + Sync {
    /// Get the file extension for this ROM format (without the dot).
    ///
    /// Example: `"ewzx"` for Emberware ZX ROMs.
    fn extension(&self) -> &'static str;

    /// Get the console type identifier.
    ///
    /// This matches the `console_type` field in game manifests.
    /// Example: `"z"` for Emberware ZX.
    fn console_type(&self) -> &'static str;

    /// Load metadata from ROM bytes without fully parsing the ROM.
    ///
    /// This is used for displaying game info in the library UI
    /// without loading the entire ROM into memory.
    fn load_metadata(&self, bytes: &[u8]) -> Result<RomMetadata>;

    /// Check if bytes appear to be a valid ROM for this loader.
    ///
    /// This should be a quick check (e.g., magic bytes) without
    /// fully parsing the ROM.
    fn can_load(&self, bytes: &[u8]) -> bool;

    /// Install a ROM file to the local game library.
    ///
    /// This extracts the ROM contents (WASM code, thumbnail, etc.)
    /// to the game directory and creates a manifest for the library.
    ///
    /// # Arguments
    ///
    /// * `rom_path` - Path to the ROM file
    /// * `data_dir_provider` - Provides the data directory path
    ///
    /// # Returns
    ///
    /// The installed `LocalGame` that can be launched immediately.
    fn install(
        &self,
        rom_path: &Path,
        data_dir_provider: &dyn DataDirProvider,
    ) -> Result<LocalGame>;
}

/// Registry of ROM loaders for all supported console types.
///
/// This allows the library to detect console types from ROM files
/// and install ROMs without knowing the specific console format.
pub struct RomLoaderRegistry {
    loaders: Vec<Box<dyn RomLoader>>,
}

impl RomLoaderRegistry {
    /// Create an empty registry.
    pub fn new() -> Self {
        Self {
            loaders: Vec::new(),
        }
    }

    /// Register a ROM loader.
    pub fn register(&mut self, loader: Box<dyn RomLoader>) {
        self.loaders.push(loader);
    }

    /// Find a loader that can handle the given bytes.
    pub fn find_loader(&self, bytes: &[u8]) -> Option<&dyn RomLoader> {
        self.loaders
            .iter()
            .find(|l| l.can_load(bytes))
            .map(|l| l.as_ref())
    }

    /// Find a loader by file extension.
    pub fn find_by_extension(&self, ext: &str) -> Option<&dyn RomLoader> {
        self.loaders
            .iter()
            .find(|l| l.extension() == ext)
            .map(|l| l.as_ref())
    }

    /// Find a loader by console type.
    pub fn find_by_console_type(&self, console_type: &str) -> Option<&dyn RomLoader> {
        self.loaders
            .iter()
            .find(|l| l.console_type() == console_type)
            .map(|l| l.as_ref())
    }

    /// Get all registered loaders.
    pub fn loaders(&self) -> &[Box<dyn RomLoader>] {
        &self.loaders
    }

    /// Get all supported file extensions.
    pub fn supported_extensions(&self) -> Vec<&'static str> {
        self.loaders.iter().map(|l| l.extension()).collect()
    }
}

impl Default for RomLoaderRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Install a ROM using the appropriate loader from the registry.
///
/// This function detects the ROM type and uses the correct loader to install it.
///
/// # Arguments
///
/// * `registry` - The ROM loader registry
/// * `rom_path` - Path to the ROM file
/// * `data_dir_provider` - Provides the data directory path
///
/// # Returns
///
/// The installed `LocalGame` or an error if the ROM type is not recognized.
pub fn install_rom(
    registry: &RomLoaderRegistry,
    rom_path: &Path,
    data_dir_provider: &dyn DataDirProvider,
) -> Result<LocalGame> {
    // Try to detect by extension first
    if let Some(ext) = rom_path.extension().and_then(|e| e.to_str())
        && let Some(loader) = registry.find_by_extension(ext)
    {
        return loader.install(rom_path, data_dir_provider);
    }

    // Fall back to reading bytes and checking magic
    let bytes = std::fs::read(rom_path)?;
    if let Some(loader) = registry.find_loader(&bytes) {
        return loader.install(rom_path, data_dir_provider);
    }

    anyhow::bail!(
        "Unknown ROM format: {}. Supported extensions: {}",
        rom_path.display(),
        registry.supported_extensions().join(", ")
    )
}
