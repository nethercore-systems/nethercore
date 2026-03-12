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
use serde::Deserialize;

use nethercore_core::Console;
use nethercore_core::app::player::sanitize_game_id;
use nethercore_core::app::{LoadedRom, RomLoader, StandaloneConfig, run_standalone};
use nethercore_shared::local::LocalGameManifest;
use nethercore_shared::{
    MAX_ROM_BYTES, MAX_WASM_BYTES, ZX_ROM_FORMAT, is_safe_game_id, read_file_with_limit,
};
use zx_common::{ZXDataPack, ZXRom};

use crate::console::NethercoreZX;

/// Player configuration passed from CLI
pub type PlayerConfig = StandaloneConfig;

/// ROM loader for Nethercore ZX
///
/// Handles both .nczx ROM files (with metadata and datapacks) and raw .wasm files.
pub struct ZXRomLoader;

const MAX_MANIFEST_BYTES: u64 = 64 * 1024;

#[derive(Default)]
struct RawWasmMetadata {
    game_name: String,
    game_id: String,
    render_mode: u8,
}

#[derive(Default, Deserialize)]
struct RawWasmNetherManifest {
    #[serde(default)]
    game: RawWasmGameManifest,
}

#[derive(Default, Deserialize)]
struct RawWasmGameManifest {
    #[serde(default)]
    id: String,
    #[serde(default)]
    title: String,
    render_mode: Option<u8>,
}

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

            let metadata = raw_wasm_metadata_from_path(path, &fallback_name);

            Ok(LoadedRom {
                code: wasm,
                console: NethercoreZX::with_datapack_and_render_mode(None, metadata.render_mode),
                game_name: metadata.game_name,
                game_id: metadata.game_id,
            })
        }
    }
}

fn raw_wasm_metadata_from_path(path: &Path, fallback_name: &str) -> RawWasmMetadata {
    let fallback_game_id = sanitize_game_id(fallback_name);
    let mut resolved_game_id = false;
    let mut metadata = RawWasmMetadata {
        game_name: fallback_name.to_string(),
        game_id: fallback_game_id.clone(),
        render_mode: 0,
    };

    let Some(parent) = path.parent() else {
        return metadata;
    };

    // Prefer manifest.json next to the ROM.
    let manifest_path = parent.join("manifest.json");
    if manifest_path.is_file() {
        if let Ok(bytes) = read_file_with_limit(&manifest_path, MAX_MANIFEST_BYTES)
            && let Ok(manifest) = serde_json::from_slice::<LocalGameManifest>(&bytes)
        {
            if !manifest.title.is_empty() {
                metadata.game_name = manifest.title;
            }
            if !manifest.id.is_empty() && is_safe_game_id(&manifest.id) {
                metadata.game_id = manifest.id;
                resolved_game_id = true;
            }
        }
    }

    if let Some(manifest) = raw_wasm_nether_manifest_from_path(path) {
        if !manifest.game.title.is_empty() {
            metadata.game_name = manifest.game.title;
        }
        if !manifest.game.id.is_empty() && is_safe_game_id(&manifest.game.id) {
            metadata.game_id = manifest.game.id;
            resolved_game_id = true;
        }
        if let Some(render_mode) = manifest.game.render_mode {
            metadata.render_mode = render_mode.min(3);
        }
    }

    // Next, accept the parent directory name if it is already safe.
    if !resolved_game_id
        && let Some(dir_name) = parent.file_name().and_then(|s| s.to_str())
        && is_safe_game_id(dir_name)
    {
        metadata.game_id = dir_name.to_string();
    }

    // Finally, sanitize the file stem.
    metadata
}

fn raw_wasm_nether_manifest_from_path(path: &Path) -> Option<RawWasmNetherManifest> {
    let parent = path.parent()?;
    for ancestor in parent.ancestors() {
        let manifest_path = ancestor.join("nether.toml");
        if !manifest_path.is_file() {
            continue;
        }
        let bytes = read_file_with_limit(&manifest_path, MAX_MANIFEST_BYTES).ok()?;
        let text = std::str::from_utf8(&bytes).ok()?;
        return toml::from_str::<RawWasmNetherManifest>(text).ok();
    }
    None
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

    use nethercore_core::Console;
    use nethercore_core::app::RomLoader;
    use nethercore_shared::local::LocalGameManifest;
    use tempfile::tempdir;

    use crate::state::ZXFFIState;

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

    #[test]
    fn zx_loader_uses_ancestor_nether_toml_for_raw_wasm_render_mode() {
        let tmp = tempdir().unwrap();
        let project_dir = tmp.path().join("examples").join("epu-showcase");
        let wasm_dir = project_dir.join("target").join("wasm32-unknown-unknown").join("release");
        fs::create_dir_all(&wasm_dir).unwrap();

        fs::write(
            project_dir.join("nether.toml"),
            "[game]\nid = \"epu-showcase\"\ntitle = \"EPU Showcase\"\nrender_mode = 2\n",
        )
        .unwrap();

        let wasm_path = wasm_dir.join("epu_showcase.wasm");
        fs::write(&wasm_path, [0_u8, 1, 2, 3]).unwrap();

        let loaded = ZXRomLoader::load_rom(&wasm_path).unwrap();
        assert_eq!(loaded.game_id, "epu-showcase");
        assert_eq!(loaded.game_name, "EPU Showcase");

        let mut state = ZXFFIState::default();
        loaded.console.initialize_ffi_state(&mut state);
        assert_eq!(state.init_config.render_mode, 2);
    }
}
