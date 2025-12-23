//! ROM format specifications for Nethercore fantasy consoles.
//!
//! This module defines the `RomFormat` struct which serves as the single source of truth
//! for all ROM-related constants (file extensions, magic bytes, asset extensions).
//!
//! # Example
//!
//! ```
//! use nethercore_shared::ZX_ROM_FORMAT;
//!
//! // Get the ROM file extension
//! assert_eq!(ZX_ROM_FORMAT.extension, "nczx");
//!
//! // Check magic bytes
//! assert_eq!(ZX_ROM_FORMAT.magic, b"NCZX");
//!
//! // Get asset extensions
//! assert_eq!(ZX_ROM_FORMAT.mesh_ext, "nczxmesh");
//! ```

use crate::console::nethercore_zx_specs;

/// ROM format specification for a fantasy console.
///
/// Defines the file format constants used for ROM files and assets.
/// Each console has its own static `RomFormat` instance.
#[derive(Debug, Clone, Copy)]
pub struct RomFormat {
    /// Console type identifier (e.g., "zx")
    ///
    /// Used in manifests and registry to identify the console type.
    pub console_type: &'static str,

    /// ROM file extension without dot (e.g., "nczx")
    pub extension: &'static str,

    /// Magic bytes at start of ROM file (4 bytes)
    pub magic: &'static [u8; 4],

    /// Format version for backward compatibility
    pub version: u32,

    /// Mesh file extension (e.g., "nczxmesh")
    pub mesh_ext: &'static str,

    /// Texture file extension (e.g., "nczxtex")
    pub texture_ext: &'static str,

    /// Sound file extension (e.g., "nczxsnd")
    pub sound_ext: &'static str,

    /// Skeleton file extension (e.g., "nczxskel")
    pub skeleton_ext: &'static str,

    /// Animation file extension (e.g., "nczxanim")
    pub animation_ext: &'static str,
}

impl RomFormat {
    /// Create a new ROM format specification.
    #[allow(clippy::too_many_arguments)]
    pub const fn new(
        console_type: &'static str,
        extension: &'static str,
        magic: &'static [u8; 4],
        version: u32,
        mesh_ext: &'static str,
        texture_ext: &'static str,
        sound_ext: &'static str,
        skeleton_ext: &'static str,
        animation_ext: &'static str,
    ) -> Self {
        Self {
            console_type,
            extension,
            magic,
            version,
            mesh_ext,
            texture_ext,
            sound_ext,
            skeleton_ext,
            animation_ext,
        }
    }
}

/// Nethercore ZX ROM format specification.
///
/// This is the single source of truth for all ZX ROM format constants:
/// - Console type: `zx`
/// - ROM extension: `.nczx`
/// - Magic bytes: `NCZX`
/// - Asset extensions: `.nczxmesh`, `.nczxtex`, `.nczxsnd`, `.nczxskel`, `.nczxanim`
pub const ZX_ROM_FORMAT: RomFormat = RomFormat::new(
    nethercore_zx_specs().console_type,
    "nczx",
    b"NCZX",
    1,
    "nczxmesh",
    "nczxtex",
    "nczxsnd",
    "nczxskel",
    "nczxanim",
);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zx_rom_format_console_type() {
        assert_eq!(ZX_ROM_FORMAT.console_type, "zx");
    }

    #[test]
    fn test_zx_rom_format_extension() {
        assert_eq!(ZX_ROM_FORMAT.extension, "nczx");
    }

    #[test]
    fn test_zx_rom_format_magic() {
        assert_eq!(ZX_ROM_FORMAT.magic, b"NCZX");
        assert_eq!(ZX_ROM_FORMAT.magic.len(), 4);
    }

    #[test]
    fn test_zx_rom_format_version() {
        assert_eq!(ZX_ROM_FORMAT.version, 1);
    }

    #[test]
    fn test_zx_asset_extensions() {
        assert_eq!(ZX_ROM_FORMAT.mesh_ext, "nczxmesh");
        assert_eq!(ZX_ROM_FORMAT.texture_ext, "nczxtex");
        assert_eq!(ZX_ROM_FORMAT.sound_ext, "nczxsnd");
        assert_eq!(ZX_ROM_FORMAT.skeleton_ext, "nczxskel");
        assert_eq!(ZX_ROM_FORMAT.animation_ext, "nczxanim");
    }
}
