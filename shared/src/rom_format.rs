//! ROM format specifications for Emberware fantasy consoles.
//!
//! This module defines the `RomFormat` struct which serves as the single source of truth
//! for all ROM-related constants (file extensions, magic bytes, asset extensions).
//!
//! # Example
//!
//! ```
//! use emberware_shared::ZX_ROM_FORMAT;
//!
//! // Get the ROM file extension
//! assert_eq!(ZX_ROM_FORMAT.extension, "ewzx");
//!
//! // Check magic bytes
//! assert_eq!(ZX_ROM_FORMAT.magic, b"EWZX");
//!
//! // Get asset extensions
//! assert_eq!(ZX_ROM_FORMAT.mesh_ext, "ewzxmesh");
//! ```

/// ROM format specification for a fantasy console.
///
/// Defines the file format constants used for ROM files and assets.
/// Each console has its own static `RomFormat` instance.
#[derive(Debug, Clone, Copy)]
pub struct RomFormat {
    /// ROM file extension without dot (e.g., "ewzx")
    pub extension: &'static str,

    /// Magic bytes at start of ROM file (4 bytes)
    pub magic: &'static [u8; 4],

    /// Format version for backward compatibility
    pub version: u32,

    /// Mesh file extension (e.g., "ewzxmesh")
    pub mesh_ext: &'static str,

    /// Texture file extension (e.g., "ewzxtex")
    pub texture_ext: &'static str,

    /// Sound file extension (e.g., "ewzxsnd")
    pub sound_ext: &'static str,

    /// Skeleton file extension (e.g., "ewzxskel")
    pub skeleton_ext: &'static str,

    /// Animation file extension (e.g., "ewzxanim")
    pub animation_ext: &'static str,
}

impl RomFormat {
    /// Create a new ROM format specification.
    #[allow(clippy::too_many_arguments)]
    pub const fn new(
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

/// Emberware ZX ROM format specification.
///
/// This is the single source of truth for all ZX ROM format constants:
/// - ROM extension: `.ewzx`
/// - Magic bytes: `EWZX`
/// - Asset extensions: `.ewzxmesh`, `.ewzxtex`, `.ewzxsnd`, `.ewzxskel`, `.ewzxanim`
pub const ZX_ROM_FORMAT: RomFormat = RomFormat::new(
    "ewzx",
    b"EWZX",
    1,
    "ewzxmesh",
    "ewzxtex",
    "ewzxsnd",
    "ewzxskel",
    "ewzxanim",
);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zx_rom_format_extension() {
        assert_eq!(ZX_ROM_FORMAT.extension, "ewzx");
    }

    #[test]
    fn test_zx_rom_format_magic() {
        assert_eq!(ZX_ROM_FORMAT.magic, b"EWZX");
        assert_eq!(ZX_ROM_FORMAT.magic.len(), 4);
    }

    #[test]
    fn test_zx_rom_format_version() {
        assert_eq!(ZX_ROM_FORMAT.version, 1);
    }

    #[test]
    fn test_zx_asset_extensions() {
        assert_eq!(ZX_ROM_FORMAT.mesh_ext, "ewzxmesh");
        assert_eq!(ZX_ROM_FORMAT.texture_ext, "ewzxtex");
        assert_eq!(ZX_ROM_FORMAT.sound_ext, "ewzxsnd");
        assert_eq!(ZX_ROM_FORMAT.skeleton_ext, "ewzxskel");
        assert_eq!(ZX_ROM_FORMAT.animation_ext, "ewzxanim");
    }
}
