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
use nethercore_core::app::player::sanitize_game_id;
use nethercore_core::app::{LoadedRom, RomLoader, StandaloneConfig, run_standalone};
use nethercore_shared::{MAX_ROM_BYTES, MAX_WASM_BYTES, ZX_ROM_FORMAT, is_safe_game_id, read_file_with_limit};
use nethercore_shared::local::LocalGameManifest;
use zx_common::{ZXDataPack, ZXRom};

use crate::console::NethercoreZX;

/// Player configuration passed from CLI
pub type PlayerConfig = StandaloneConfig;

/// ROM loader for Nethercore ZX
///
/// Handles both .nczx ROM files (with metadata and datapacks) and raw .wasm files.
pub struct ZXRomLoader;

const MAX_MANIFEST_BYTES: u64 = 64 * 1024;

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
            let rom_bytes = read_file_with_limit(path, MAX_ROM_BYTES)
                .context("Failed to read Nethercore ZX ROM file")?;

            let rom = ZXRom::from_bytes(&rom_bytes).context("Failed to parse Nethercore ZX ROM")?;

            // Use metadata title, fall back to file stem if empty
            let game_name = if rom.metadata.title.is_empty() {
                fallback_name.clone()
            } else {
                rom.metadata.title.clone()
            };

            let game_id = if !rom.metadata.id.is_empty() && is_safe_game_id(&rom.metadata.id) {
                rom.metadata.id.clone()
            } else {
                sanitize_game_id(&fallback_name)
            };

            // Create console with datapack
            let data_pack: Option<Arc<ZXDataPack>> = rom.data_pack.map(Arc::new);
            let render_mode = rom.metadata.render_mode.unwrap_or(0).min(3) as u8;
            let console = NethercoreZX::with_datapack_and_render_mode(data_pack, render_mode);

            Ok(LoadedRom {
                code: rom.code,
                console,
                game_name,
                game_id,
            })
        } else {
            // Raw WASM file - use file stem as name
            let wasm =
                read_file_with_limit(path, MAX_WASM_BYTES).context("Failed to read WASM file")?;

            let fallback_game_id = sanitize_game_id(&fallback_name);
            let game_id = wasm_game_id_from_path(path).unwrap_or(fallback_game_id);

            Ok(LoadedRom {
                code: wasm,
                console: NethercoreZX::new(),
                game_name: fallback_name,
                game_id,
            })
        }
    }
}

fn wasm_game_id_from_path(path: &Path) -> Option<String> {
    let parent = path.parent()?;

    // Prefer manifest.json next to the ROM.
    let manifest_path = parent.join("manifest.json");
    if manifest_path.is_file() {
        let bytes = read_file_with_limit(&manifest_path, MAX_MANIFEST_BYTES).ok()?;
        let manifest = serde_json::from_slice::<LocalGameManifest>(&bytes).ok()?;
        if !manifest.id.is_empty() && is_safe_game_id(&manifest.id) {
            return Some(manifest.id);
        }
    }

    // Next, accept the parent directory name if it is already safe.
    if let Some(dir_name) = parent.file_name().and_then(|s| s.to_str())
        && is_safe_game_id(dir_name)
    {
        return Some(dir_name.to_string());
    }

    // Finally, sanitize the file stem.
    let file_stem = path.file_stem()?.to_str()?;
    Some(sanitize_game_id(file_stem))
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

    run_standalone::<NethercoreZX, ZXRomLoader>(config)
}

#[cfg(test)]
mod tests {
    use std::fs;

    use nethercore_core::app::RomLoader;
    use nethercore_shared::local::LocalGameManifest;
    use tempfile::tempdir;

    use super::ZXRomLoader;

    #[test]
    fn zx_loader_prefers_manifest_game_id_for_rom_wasm() {
        let tmp = tempdir().unwrap();
        let game_dir = tmp.path().join("games").join("some-dir");
        fs::create_dir_all(&game_dir).unwrap();

        let wasm_path = game_dir.join("rom.wasm");
        fs::write(&wasm_path, [0_u8, 1, 2, 3]).unwrap();

        let manifest_id = "manifest-game-id".to_string();
        let manifest = LocalGameManifest {
            id: manifest_id.clone(),
            title: "Some Game".to_string(),
            author: "some-author".to_string(),
            version: "0.0.0".to_string(),
            downloaded_at: "2026-01-26T00:00:00Z".to_string(),
            console_type: "zx".to_string(),
        };
        let manifest_json = serde_json::to_vec(&manifest).unwrap();
        fs::write(game_dir.join("manifest.json"), manifest_json).unwrap();

        let loaded = ZXRomLoader::load_rom(&wasm_path).unwrap();
        assert_eq!(loaded.game_id, manifest_id);
    }
}
