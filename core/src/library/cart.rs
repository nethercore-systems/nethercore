//! ROM installation to local library
//!
//! This module handles installing ROM files (`.ewz`, `.ewc`, etc.) to the
//! local game library. It deserializes ROMs, validates them, and extracts
//! their contents to the game directory.

use std::path::Path;

use anyhow::{Context, Result};
use z_common::ZRom;

use super::{DataDirProvider, LocalGame};

/// Install an Emberware Z ROM (.ewz)
///
/// This function:
/// 1. Loads and validates the ROM
/// 2. Extracts the WASM code and thumbnail to the game directory
/// 3. Creates a manifest.json for backward compatibility
///
/// # Arguments
///
/// * `rom_path` - Path to the .ewz ROM file
/// * `data_dir_provider` - Provides the data directory path
///
/// # Returns
///
/// The installed `LocalGame` that can be launched immediately.
///
/// # Errors
///
/// Returns an error if:
/// - The ROM cannot be read or is corrupted
/// - The ROM format is invalid
/// - The game directory cannot be created
pub fn install_z_rom(
    rom_path: &Path,
    data_dir_provider: &dyn DataDirProvider,
) -> Result<LocalGame> {
    // 1. Load and validate ROM
    let bytes = std::fs::read(rom_path)
        .with_context(|| format!("Failed to read ROM file: {}", rom_path.display()))?;

    let rom = ZRom::from_bytes(&bytes)
        .with_context(|| format!("Failed to load EWZ ROM: {}", rom_path.display()))?;

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
        console_type: "z".to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use tempfile::TempDir;
    use z_common::{EWZ_VERSION, ZMetadata, ZRom};

    /// Test data directory provider
    struct TestDataDirProvider {
        path: PathBuf,
    }

    impl DataDirProvider for TestDataDirProvider {
        fn data_dir(&self) -> Option<PathBuf> {
            Some(self.path.clone())
        }
    }

    fn create_test_rom() -> ZRom {
        ZRom {
            version: EWZ_VERSION,
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
            code: b"\0asm\x01\x00\x00\x00test code".to_vec(),
            data_pack: None,
            thumbnail: Some(b"fake png data".to_vec()),
            screenshots: vec![],
        }
    }

    #[test]
    fn test_install_z_rom() {
        // Create temp directory
        let temp_dir = TempDir::new().unwrap();
        let provider = TestDataDirProvider {
            path: temp_dir.path().to_path_buf(),
        };

        // Create ROM file
        let rom = create_test_rom();
        let rom_bytes = rom.to_bytes().unwrap();
        let rom_path = temp_dir.path().join("test-game.ewz");
        std::fs::write(&rom_path, rom_bytes).unwrap();

        // Install ROM
        let result = install_z_rom(&rom_path, &provider);
        assert!(result.is_ok());

        let local_game = result.unwrap();
        assert_eq!(local_game.id, "test-game");
        assert_eq!(local_game.title, "Test Game");
        assert_eq!(local_game.console_type, "z");

        // Check files were created
        let game_dir = temp_dir.path().join("games").join("test-game");
        assert!(game_dir.join("rom.wasm").exists());
        assert!(game_dir.join("thumbnail.png").exists());
        assert!(game_dir.join("manifest.json").exists());
    }

    #[test]
    fn test_install_unsupported_extension() {
        let temp_dir = TempDir::new().unwrap();
        let provider = TestDataDirProvider {
            path: temp_dir.path().to_path_buf(),
        };

        let bad_rom_path = temp_dir.path().join("game.bad");
        std::fs::write(&bad_rom_path, b"data").unwrap();

        let result = install_z_rom(&bad_rom_path, &provider);
        assert!(result.is_err());
        // Should fail with invalid EWZ magic bytes or failed to load since it's not a valid ROM
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("Invalid EWZ magic bytes") || err.contains("Failed to load EWZ ROM"),
            "Error was: {}",
            err
        );
    }
}
