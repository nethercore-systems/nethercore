//! Standalone player for Emberware ZX
//!
//! This module provides a minimal player application that can run Emberware ZX ROM files
//! without the full library UI. Used by:
//! - `emberware-zx` binary (standalone player)
//! - `ember run` command (development)
//! - Library process spawning

use std::path::Path;
use std::sync::Arc;

use anyhow::{Context, Result};

use emberware_core::Console;
use emberware_core::app::{LoadedRom, RomLoader, StandaloneConfig, run_standalone};
use emberware_shared::ZX_ROM_FORMAT;
use zx_common::{ZDataPack, ZRom};

use crate::console::EmberwareZX;

/// Player configuration passed from CLI
pub type PlayerConfig = StandaloneConfig;

/// ROM loader for Emberware ZX
///
/// Handles both .ewz ROM files (with metadata and datapacks) and raw .wasm files.
pub struct ZXRomLoader;

impl RomLoader for ZXRomLoader {
    type Console = EmberwareZX;

    fn load_rom(path: &Path) -> Result<LoadedRom<EmberwareZX>> {
        // Fallback name from file stem
        let fallback_name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or(EmberwareZX::specs().name)
            .to_string();

        if path.extension().and_then(|e| e.to_str()) == Some(ZX_ROM_FORMAT.extension) {
            let rom_bytes = std::fs::read(path).context("Failed to read Emberware ZX ROM file")?;

            let rom = ZRom::from_bytes(&rom_bytes).context("Failed to parse Emberware ZX ROM")?;

            // Use metadata title, fall back to file stem if empty
            let game_name = if rom.metadata.title.is_empty() {
                fallback_name
            } else {
                rom.metadata.title.clone()
            };

            // Create console with datapack
            let data_pack: Option<Arc<ZDataPack>> = rom.data_pack.map(Arc::new);
            let console = EmberwareZX::with_datapack(data_pack);

            Ok(LoadedRom {
                code: rom.code,
                console,
                game_name,
            })
        } else {
            // Raw WASM file - use file stem as name
            let wasm = std::fs::read(path).context("Failed to read WASM file")?;

            Ok(LoadedRom {
                code: wasm,
                console: EmberwareZX::new(),
                game_name: fallback_name,
            })
        }
    }
}

/// Run the standalone player
pub fn run(config: PlayerConfig) -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .init();

    tracing::info!("Starting Emberware ZX player");
    tracing::info!("ROM: {}", config.rom_path.display());

    run_standalone::<EmberwareZX, ZXRomLoader>(config)
}
