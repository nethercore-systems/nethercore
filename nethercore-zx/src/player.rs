//! Standalone player for Nethercore ZX
//!
//! This module provides a minimal player application that can run Nethercore ZX ROM files
//! without the full library UI. Used by:
//! - `nethercore-zx` binary (standalone player)
//! - `nether run` command (development)
//! - Library process spawning

use std::path::Path;
use std::sync::Arc;

use anyhow::{Context, Result};

use nethercore_core::Console;
use nethercore_core::app::{LoadedRom, RomLoader, StandaloneConfig, run_standalone};
use nethercore_shared::ZX_ROM_FORMAT;
use zx_common::{ZDataPack, ZRom};

use crate::console::NethercoreZX;

/// Player configuration passed from CLI
pub type PlayerConfig = StandaloneConfig;

/// ROM loader for Nethercore ZX
///
/// Handles both .nczx ROM files (with metadata and datapacks) and raw .wasm files.
pub struct ZXRomLoader;

impl RomLoader for ZXRomLoader {
    type Console = NethercoreZX;

    fn load_rom(path: &Path) -> Result<LoadedRom<NethercoreZX>> {
        // Fallback name from file stem
        let fallback_name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or(NethercoreZX::specs().name)
            .to_string();

        if path.extension().and_then(|e| e.to_str()) == Some(ZX_ROM_FORMAT.extension) {
            let rom_bytes = std::fs::read(path).context("Failed to read Nethercore ZX ROM file")?;

            let rom = ZRom::from_bytes(&rom_bytes).context("Failed to parse Nethercore ZX ROM")?;

            // Use metadata title, fall back to file stem if empty
            let game_name = if rom.metadata.title.is_empty() {
                fallback_name
            } else {
                rom.metadata.title.clone()
            };

            // Create console with datapack
            let data_pack: Option<Arc<ZDataPack>> = rom.data_pack.map(Arc::new);
            let console = NethercoreZX::with_datapack(data_pack);

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
                console: NethercoreZX::new(),
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

    tracing::info!("Starting Nethercore ZX player");
    tracing::info!("ROM: {}", config.rom_path.display());

    run_standalone::<NethercoreZX, ZXRomLoader>(config)
}
