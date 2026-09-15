//! ROM loader implementation for Nethercore ZX
//!
//! This module provides the `ZXRomLoader` which implements the `RomLoader` trait
//! from nethercore-core, allowing the library system to work with .nczx ROM files.

use std::path::Path;

use anyhow::{Context, Result};
use nethercore_core::library::{DataDirProvider, LocalGame, RomLoader, RomMetadata};
use nethercore_shared::{MAX_ROM_BYTES, ZX_ROM_FORMAT, is_safe_game_id, read_file_with_limit};

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
        let bytes = read_file_with_limit(rom_path, MAX_ROM_BYTES)
            .with_context(|| format!("Failed to read ROM file: {}", rom_path.display()))?;

        let rom = ZXRom::from_bytes(&bytes)
            .with_context(|| format!("Failed to load NCZX ROM: {}", rom_path.display()))?;

        let data_dir = data_dir_provider
            .data_dir()
            .context("Data directory not available")?;
        self.install_validated(&rom, &bytes, &data_dir)
    }
}

impl ZXRomLoader {
    /// Shared native install/update path. The cartridge is authoritative;
    /// manifest and thumbnail files are rebuildable library caches.
    pub fn install_bytes(&self, bytes: &[u8], data_dir: &Path) -> Result<LocalGame> {
        anyhow::ensure!(
            bytes.len() as u64 <= MAX_ROM_BYTES,
            "ROM exceeds size limit"
        );
        let rom = ZXRom::from_bytes(bytes).context("Invalid NCZX cartridge")?;
        self.install_validated(&rom, bytes, data_dir)
    }

    fn install_validated(&self, rom: &ZXRom, bytes: &[u8], data_dir: &Path) -> Result<LocalGame> {
        anyhow::ensure!(
            is_safe_game_id(&rom.metadata.id),
            "Invalid game id in ROM metadata: '{}'",
            rom.metadata.id
        );
        let engine = nethercore_core::wasm::WasmEngine::new()?;
        engine
            .load_module(&rom.code)
            .context("Invalid cartridge WASM")?;
        let manifest = serde_json::to_vec_pretty(&rom.to_local_manifest())?;
        let game_dir = data_dir.join("games").join(&rom.metadata.id);
        std::fs::create_dir_all(&game_dir)?;
        let rom_path = game_dir.join("rom.nczx");
        nethercore_core::library::rom::atomic_write(&rom_path, bytes)
            .context("Failed to replace installed cartridge")?;
        // Commit point: cache failures do not misreport a successful installation.
        for (name, contents) in std::iter::once(("manifest.json", manifest.as_slice())).chain(
            rom.thumbnail
                .as_deref()
                .map(|thumbnail| ("thumbnail.png", thumbnail)),
        ) {
            if let Err(error) =
                nethercore_core::library::rom::atomic_write(&game_dir.join(name), contents)
            {
                eprintln!("Installed cartridge; could not refresh {name}: {error}");
            }
        }
        Ok(LocalGame {
            id: rom.metadata.id.clone(),
            title: rom.metadata.title.clone(),
            author: rom.metadata.author.clone(),
            version: rom.metadata.version.clone(),
            rom_path,
            console_type: ZX_ROM_FORMAT.console_type.into(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ZXMetadata;
    use nethercore_shared::netplay::NetplayMetadata;
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

    #[test]
    fn shipping_install_retains_full_cart_and_reloads_without_stale_manifest() {
        let dir = TempDir::new().unwrap();
        let provider = TestDataDirProvider {
            path: dir.path().into(),
        };
        let mut rom = create_test_rom();
        let mut data = crate::ZXDataPack::new();
        data.data.push(crate::PackedData {
            id: "level".into(),
            data: b"level-A".to_vec(),
        });
        rom.data_pack = Some(data);
        let path = dir.path().join("source.nczx");
        std::fs::write(&path, rom.to_bytes().unwrap()).unwrap();
        let installed = ZXRomLoader.install(&path, &provider).unwrap();
        let readback = ZXRom::from_bytes(&std::fs::read(&installed.rom_path).unwrap()).unwrap();
        assert_eq!(
            readback.data_pack.unwrap().find_data("level").unwrap().data,
            b"level-A"
        );
        assert_eq!(readback.metadata.render_mode, rom.metadata.render_mode);
        std::fs::write(
            installed.rom_path.parent().unwrap().join("manifest.json"),
            b"stale",
        )
        .unwrap();
        let mut registry = nethercore_core::library::RomLoaderRegistry::new();
        registry.register(Box::new(ZXRomLoader));
        let games = nethercore_core::library::get_local_games_with_loaders(&provider, &registry);
        assert_eq!(games.len(), 1);
        assert_eq!(games[0].version, rom.metadata.version);
        assert_eq!(games[0].rom_path, installed.rom_path);
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
                netplay: NetplayMetadata {
                    max_players: 1,
                    ..Default::default()
                },
            },
            code: b"\0asm\x01\x00\x00\x00".to_vec(),
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
        assert!(game_dir.join("rom.nczx").exists());
        assert!(game_dir.join("thumbnail.png").exists());
        assert!(game_dir.join("manifest.json").exists());
    }

    #[test]
    fn test_install_rejects_invalid_game_id() {
        let loader = ZXRomLoader;

        let temp_dir = TempDir::new().unwrap();
        let provider = TestDataDirProvider {
            path: temp_dir.path().to_path_buf(),
        };

        let mut rom = create_test_rom();
        rom.metadata.id = "../evil".to_string();
        let rom_bytes = rom.to_bytes().unwrap();
        let rom_path = temp_dir.path().join("bad-id.nczx");
        std::fs::write(&rom_path, rom_bytes).unwrap();

        let result = loader.install(&rom_path, &provider);
        assert!(result.is_err());
    }
}
