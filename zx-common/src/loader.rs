//! ROM loader implementation for Nethercore ZX
//!
//! This module provides the `ZXRomLoader` which implements the `RomLoader` trait
//! from nethercore-core, allowing the library system to work with .nczx ROM files.

use std::path::Path;

use anyhow::{Context, Result};
use nethercore_core::library::{DataDirProvider, LocalGame, RomLoader, RomMetadata};
use nethercore_shared::ZX_ROM_FORMAT;

use crate::ZXRom;

/// ROM loader for Nethercore ZX (.nczx) files.
///
/// This loader handles the ZX console's ROM format, which contains:
/// - WASM code (compiled game)
/// - Optional data pack (bundled assets)
/// - Metadata (title, author, version)
/// - Optional thumbnail and screenshots
pub struct ZXRomLoader;

impl RomLoader for ZXRomLoader {
    fn extension(&self) -> &'static str {
        ZX_ROM_FORMAT.extension
    }

    fn console_type(&self) -> &'static str {
        ZX_ROM_FORMAT.console_type
    }

    fn load_metadata(&self, bytes: &[u8]) -> Result<RomMetadata> {
        let rom = ZXRom::from_bytes(bytes)?;
        Ok(RomMetadata {
            id: rom.metadata.id,
            title: rom.metadata.title,
            author: rom.metadata.author,
            version: rom.metadata.version,
        })
    }

    fn can_load(&self, bytes: &[u8]) -> bool {
        bytes.len() >= 4 && &bytes[0..4] == ZX_ROM_FORMAT.magic
    }

    fn install(
        &self,
        rom_path: &Path,
        data_dir_provider: &dyn DataDirProvider,
    ) -> Result<LocalGame> {
        // 1. Load and validate ROM
        let bytes = std::fs::read(rom_path)
            .with_context(|| format!("Failed to read ROM file: {}", rom_path.display()))?;

        let rom = ZXRom::from_bytes(&bytes)
            .with_context(|| format!("Failed to load NCZX ROM: {}", rom_path.display()))?;

        // 2. Get game directory
        let games_dir = data_dir_provider
            .data_dir()
            .ok_or_else(|| anyhow::anyhow!("Data directory not available"))?
            .join("games");

        let game_dir = games_dir.join(&rom.metadata.id);

        // 3. Create game directory
        std::fs::create_dir_all(&game_dir)
            .with_context(|| format!("Failed to create game directory: {}", game_dir.display()))?;

        // 4. Extract WASM code
        std::fs::write(game_dir.join("rom.wasm"), &rom.code).with_context(|| {
            format!(
                "Failed to write WASM code to: {}",
                game_dir.join("rom.wasm").display()
            )
        })?;

        // 5. Extract thumbnail ONLY (screenshots stay in ROM to save disk space)
        if let Some(ref thumb) = rom.thumbnail {
            std::fs::write(game_dir.join("thumbnail.png"), thumb).with_context(|| {
                format!(
                    "Failed to write thumbnail to: {}",
                    game_dir.join("thumbnail.png").display()
                )
            })?;
        }

        // 6. Write manifest.json for backward compatibility with existing library system
        let manifest = rom.to_local_manifest();
        std::fs::write(
            game_dir.join("manifest.json"),
            serde_json::to_string_pretty(&manifest)?,
        )
        .with_context(|| {
            format!(
                "Failed to write manifest to: {}",
                game_dir.join("manifest.json").display()
            )
        })?;

        // 7. Return LocalGame
        Ok(LocalGame {
            id: rom.metadata.id.clone(),
            title: rom.metadata.title.clone(),
            author: rom.metadata.author.clone(),
            version: rom.metadata.version.clone(),
            rom_path: game_dir.join("rom.wasm"),
            console_type: ZX_ROM_FORMAT.console_type.to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ZXMetadata;
    use std::path::PathBuf;
    use tempfile::TempDir;

    /// Test data directory provider
    struct TestDataDirProvider {
        path: PathBuf,
    }

    impl DataDirProvider for TestDataDirProvider {
        fn data_dir(&self) -> Option<PathBuf> {
            Some(self.path.clone())
        }
    }

    fn create_test_rom() -> ZXRom {
        ZXRom {
            version: ZX_ROM_FORMAT.version,
            metadata: ZXMetadata {
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
            code: b"\0asm\x01\x00\x00\x00test code".to_vec(),
            data_pack: None,
            thumbnail: Some(b"fake png data".to_vec()),
            screenshots: vec![],
        }
    }

    #[test]
    fn test_extension() {
        let loader = ZXRomLoader;
        assert_eq!(loader.extension(), ZX_ROM_FORMAT.extension);
    }

    #[test]
    fn test_console_type() {
        let loader = ZXRomLoader;
        assert_eq!(loader.console_type(), ZX_ROM_FORMAT.console_type);
    }

    #[test]
    fn test_can_load_valid() {
        let loader = ZXRomLoader;
        let rom = create_test_rom();
        let bytes = rom.to_bytes().unwrap();
        assert!(loader.can_load(&bytes));
    }

    #[test]
    fn test_can_load_invalid() {
        let loader = ZXRomLoader;
        assert!(!loader.can_load(b"invalid"));
        assert!(!loader.can_load(b""));
        assert!(!loader.can_load(b"NC")); // Too short
    }

    #[test]
    fn test_load_metadata() {
        let loader = ZXRomLoader;
        let rom = create_test_rom();
        let bytes = rom.to_bytes().unwrap();

        let metadata = loader.load_metadata(&bytes).unwrap();
        assert_eq!(metadata.id, "test-game");
        assert_eq!(metadata.title, "Test Game");
        assert_eq!(metadata.author, "Test Author");
        assert_eq!(metadata.version, "1.0.0");
    }

    #[test]
    fn test_install() {
        let loader = ZXRomLoader;

        // Create temp directory
        let temp_dir = TempDir::new().unwrap();
        let provider = TestDataDirProvider {
            path: temp_dir.path().to_path_buf(),
        };

        // Create ROM file
        let rom = create_test_rom();
        let rom_bytes = rom.to_bytes().unwrap();
        let rom_path = temp_dir.path().join("test-game.nczx");
        std::fs::write(&rom_path, rom_bytes).unwrap();

        // Install ROM
        let result = loader.install(&rom_path, &provider);
        assert!(result.is_ok());

        let local_game = result.unwrap();
        assert_eq!(local_game.id, "test-game");
        assert_eq!(local_game.title, "Test Game");
        assert_eq!(local_game.console_type, ZX_ROM_FORMAT.console_type);

        // Check files were created
        let game_dir = temp_dir.path().join("games").join("test-game");
        assert!(game_dir.join("rom.wasm").exists());
        assert!(game_dir.join("thumbnail.png").exists());
        assert!(game_dir.join("manifest.json").exists());
    }
}
