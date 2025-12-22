//! Nethercore ZX ROM format (`.nczx`)
//!
//! Binary ROM format for Nethercore ZX games using bitcode serialization.
//! Each ROM contains game code, metadata, optional data pack, and preview assets.
//!
//! # Memory Model
//!
//! Nethercore ZX uses a **12MB ROM + 4MB RAM** memory model:
//! - ROM (Cartridge): 12 MB total (WASM code + assets via data pack)
//! - RAM: 4 MB WASM linear memory (code + heap + stack)
//! - VRAM: 4 MB GPU textures and mesh buffers
//!
//! Assets loaded via `rom_*` FFI go directly to VRAM/audio memory on the host,
//! bypassing WASM linear memory. Only handles (u32 IDs) live in game state.
//!
//! # Format Constants
//!
//! All ROM format constants are defined in [`nethercore_shared::ZX_ROM_FORMAT`]:
//! - Extension: `"nczx"`
//! - Magic bytes: `b"NCZX"`
//! - Version: `1`

use bitcode::{Decode, Encode};

use nethercore_shared::local::LocalGameManifest;
use nethercore_shared::ZX_ROM_FORMAT;

use super::z_data_pack::ZDataPack;

/// Complete Nethercore ZX ROM
///
/// This struct represents a complete game ROM for Nethercore ZX, including
/// all metadata, compiled WASM code, optional data pack, and preview assets.
///
/// # Memory Layout
///
/// ```text
/// ┌─────────────────────────────────────────────────────────────┐
/// │                   .nczx ROM File (≤12MB)                    │
/// ├─────────────────────────────────────────────────────────────┤
/// │  NCZX Header (4 bytes)                                      │
/// │  ├── Magic: "NCZX"                                          │
/// ├─────────────────────────────────────────────────────────────┤
/// │  ZRom (bitcode serialized)                                  │
/// │  ├── version: u32                                           │
/// │  ├── metadata: ZMetadata                                    │
/// │  ├── code: Vec<u8>         ← WASM bytecode (≤4MB)          │
/// │  ├── data_pack: Option<ZDataPack>  ← Bundled assets        │
/// │  ├── thumbnail: Option<Vec<u8>>                            │
/// │  └── screenshots: Vec<Vec<u8>>                             │
/// └─────────────────────────────────────────────────────────────┘
/// ```
#[derive(Debug, Clone, Encode, Decode)]
pub struct ZRom {
    /// ROM format version (currently 1)
    pub version: u32,

    /// Game metadata
    pub metadata: ZMetadata,

    /// Compiled WASM code (must fit in 4MB RAM)
    pub code: Vec<u8>,

    /// Optional data pack containing bundled assets
    ///
    /// Assets in the data pack are loaded via `rom_*` FFI functions and go
    /// directly to VRAM/audio memory, bypassing WASM linear memory.
    /// This enables efficient rollback (only 4MB RAM snapshotted).
    pub data_pack: Option<ZDataPack>,

    /// Optional thumbnail (256x256 PNG, extracted locally during installation)
    pub thumbnail: Option<Vec<u8>>,

    /// Optional screenshots (PNG bytes, max 5)
    ///
    /// These are stored in the ROM but NOT extracted during installation
    /// to save disk space. They can be displayed when viewing ROM info
    /// or on the platform game page.
    pub screenshots: Vec<Vec<u8>>,
}

/// Nethercore ZX specific metadata
#[derive(Debug, Clone, Encode, Decode)]
pub struct ZMetadata {
    // Core game info
    /// Game slug (e.g., "platformer")
    pub id: String,

    /// Display title
    pub title: String,

    /// Primary author/studio name (for display)
    pub author: String,

    /// Semantic version (e.g., "1.0.0")
    pub version: String,

    /// Game description
    pub description: String,

    /// Category tags
    pub tags: Vec<String>,

    // Platform integration (optional foreign keys)
    /// UUID linking to platform game record
    pub platform_game_id: Option<String>,

    /// UUID linking to platform user/studio
    pub platform_author_id: Option<String>,

    // Creation info
    /// ISO 8601 timestamp when ROM was created
    pub created_at: String,

    /// xtask version that created this ROM
    pub tool_version: String,

    // Z-specific settings
    /// Render mode: 0=Unlit, 1=Matcap, 2=PBR, 3=Hybrid
    pub render_mode: Option<u32>,

    /// Default resolution (e.g., "640x480")
    pub default_resolution: Option<String>,

    /// Target FPS
    pub target_fps: Option<u32>,
}

impl ZRom {
    /// Serialize ROM to bytes with magic header
    ///
    /// The output format is:
    /// - 4 bytes: Magic bytes "NCZX"
    /// - Remaining bytes: Bitcode-encoded ZRom struct
    pub fn to_bytes(&self) -> anyhow::Result<Vec<u8>> {
        let mut bytes = ZX_ROM_FORMAT.magic.to_vec();
        let encoded = bitcode::encode(self);
        bytes.extend(encoded);
        Ok(bytes)
    }

    /// Deserialize ROM from bytes and validate
    ///
    /// This checks magic bytes, deserializes the ROM, and runs validation.
    pub fn from_bytes(bytes: &[u8]) -> anyhow::Result<Self> {
        // Check magic bytes
        if bytes.len() < 4 || &bytes[0..4] != ZX_ROM_FORMAT.magic {
            anyhow::bail!(
                "Invalid NCZX magic bytes (expected: {:?})",
                std::str::from_utf8(ZX_ROM_FORMAT.magic).unwrap_or("NCZX")
            );
        }

        // Decode remaining bytes
        let rom: ZRom = bitcode::decode(&bytes[4..])
            .map_err(|e| anyhow::anyhow!("Failed to decode NCZX ROM: {}", e))?;

        // Validate
        rom.validate()?;

        Ok(rom)
    }

    /// Validate ROM structure
    ///
    /// Checks:
    /// - Version is supported
    /// - Required fields are present
    /// - WASM code has valid magic bytes
    pub fn validate(&self) -> anyhow::Result<()> {
        // Check version
        if self.version > ZX_ROM_FORMAT.version {
            anyhow::bail!(
                "Unsupported NCZX version: {} (max supported: {})",
                self.version,
                ZX_ROM_FORMAT.version
            );
        }

        // Check required fields
        if self.metadata.id.is_empty() {
            anyhow::bail!("Game ID cannot be empty");
        }
        if self.metadata.title.is_empty() {
            anyhow::bail!("Game title cannot be empty");
        }
        if self.metadata.author.is_empty() {
            anyhow::bail!("Game author cannot be empty");
        }
        if self.metadata.version.is_empty() {
            anyhow::bail!("Game version cannot be empty");
        }

        // Validate WASM magic bytes
        if self.code.len() < 4 {
            anyhow::bail!("WASM code too small (< 4 bytes)");
        }
        if &self.code[0..4] != b"\0asm" {
            anyhow::bail!("Invalid WASM code (missing \\0asm magic bytes)");
        }

        Ok(())
    }

    /// Convert to LocalGameManifest for library installation
    ///
    /// This creates a `manifest.json` compatible with the existing
    /// library system for backward compatibility.
    pub fn to_local_manifest(&self) -> LocalGameManifest {
        LocalGameManifest {
            id: self.metadata.id.clone(),
            title: self.metadata.title.clone(),
            author: self.metadata.author.clone(),
            version: self.metadata.version.clone(),
            downloaded_at: chrono::Utc::now().to_rfc3339(),
            console_type: ZX_ROM_FORMAT.console_type.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_rom() -> ZRom {
        ZRom {
            version: ZX_ROM_FORMAT.version,
            metadata: ZMetadata {
                id: "test-game".to_string(),
                title: "Test Game".to_string(),
                author: "Test Author".to_string(),
                version: "1.0.0".to_string(),
                description: "A test game".to_string(),
                tags: vec!["test".to_string()],
                platform_game_id: None,
                platform_author_id: None,
                created_at: "2025-01-01T00:00:00Z".to_string(),
                tool_version: "0.1.0".to_string(),
                render_mode: Some(2),
                default_resolution: Some("640x480".to_string()),
                target_fps: Some(60),
            },
            code: b"\0asm\x01\x00\x00\x00".to_vec(), // Valid WASM header
            data_pack: None,                         // No bundled assets for simple test
            thumbnail: None,
            screenshots: vec![],
        }
    }

    #[test]
    fn test_rom_roundtrip() {
        let rom = create_test_rom();
        let bytes = rom.to_bytes().unwrap();
        let decoded = ZRom::from_bytes(&bytes).unwrap();

        assert_eq!(decoded.version, rom.version);
        assert_eq!(decoded.metadata.id, rom.metadata.id);
        assert_eq!(decoded.metadata.title, rom.metadata.title);
        assert_eq!(decoded.code, rom.code);
    }

    #[test]
    fn test_magic_bytes() {
        let rom = create_test_rom();
        let bytes = rom.to_bytes().unwrap();

        // Check magic bytes
        assert_eq!(&bytes[0..4], ZX_ROM_FORMAT.magic);
    }

    #[test]
    fn test_invalid_magic_bytes() {
        let bad_bytes = b"BADMAGIC".to_vec();
        let result = ZRom::from_bytes(&bad_bytes);

        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Invalid NCZX magic bytes")
        );
    }

    #[test]
    fn test_validation_empty_id() {
        let mut rom = create_test_rom();
        rom.metadata.id = String::new();

        let result = rom.validate();
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Game ID cannot be empty")
        );
    }

    #[test]
    fn test_validation_invalid_wasm() {
        let mut rom = create_test_rom();
        rom.code = b"notw".to_vec(); // Invalid WASM

        let result = rom.validate();
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Invalid WASM code")
        );
    }

    #[test]
    fn test_to_local_manifest() {
        let rom = create_test_rom();
        let manifest = rom.to_local_manifest();

        assert_eq!(manifest.id, "test-game");
        assert_eq!(manifest.title, "Test Game");
        assert_eq!(manifest.author, "Test Author");
        assert_eq!(manifest.version, "1.0.0");
        assert_eq!(manifest.console_type, ZX_ROM_FORMAT.console_type);
    }
}
