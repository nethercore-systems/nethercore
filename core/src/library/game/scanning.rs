//! Game directory scanning and discovery
//!
//! This module handles scanning the games directory for installed games,
//! supporting both ROM files and directory-based games with manifests.

use nethercore_shared::{
    LocalGameManifest, MAX_ROM_BYTES, ZX_ROM_FORMAT, read_file_with_limit,
};
use std::path::Path;

use crate::library::{DataDirProvider, rom::RomLoaderRegistry};
use super::LocalGame;

/// Returns all locally cached games.
///
/// Scans the games directory for valid game folders (containing `manifest.json`).
/// Games with invalid or missing manifests are silently skipped.
///
/// Note: This version only scans directories with manifest.json files.
/// Use `get_local_games_with_loaders` to also detect ROM files directly.
pub fn get_local_games(provider: &dyn DataDirProvider) -> Vec<LocalGame> {
    let games_dir = match provider.data_dir() {
        Some(dir) => dir.join("games"),
        None => return vec![],
    };

    get_games_from_dir(&games_dir, None)
}

/// Returns all locally cached games, including ROM files detected by loaders.
///
/// Scans the games directory for:
/// 1. ROM files matching registered loader extensions (if registry provided)
/// 2. Directories with `manifest.json` (backward compatibility, development)
///
/// Games with invalid or missing data are silently skipped.
pub fn get_local_games_with_loaders(
    provider: &dyn DataDirProvider,
    registry: &RomLoaderRegistry,
) -> Vec<LocalGame> {
    let games_dir = match provider.data_dir() {
        Some(dir) => dir.join("games"),
        None => return vec![],
    };

    get_games_from_dir(&games_dir, Some(registry))
}

/// Internal: Scan a directory for games.
/// Extracted for testability.
///
/// Scans for:
/// 1. ROM files matching registered loader extensions (if registry provided)
/// 2. Directories with `manifest.json` (backward compatibility, development)
pub(super) fn get_games_from_dir(games_dir: &Path, registry: Option<&RomLoaderRegistry>) -> Vec<LocalGame> {
    let Ok(entries) = std::fs::read_dir(games_dir) else {
        return vec![];
    };

    entries
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let path = entry.path();

            // Check if this is a ROM file (if registry provided)
            if let Some(registry) = registry
                && path.is_file()
                && let Some(ext) = path.extension().and_then(|e| e.to_str())
                && let Some(loader) = registry.find_by_extension(ext)
            {
                // Load ROM metadata using the appropriate loader
                let rom_bytes = read_file_with_limit(&path, MAX_ROM_BYTES).ok()?;
                let metadata = loader.load_metadata(&rom_bytes).ok()?;

                return Some(LocalGame {
                    id: metadata.id,
                    title: metadata.title,
                    author: metadata.author,
                    version: metadata.version,
                    rom_path: path,
                    console_type: loader.console_type().to_string(),
                });
            }

            // Check if this is a game directory
            if path.is_dir() {
                let manifest_path = path.join("manifest.json");
                let manifest_content = std::fs::read_to_string(manifest_path).ok()?;
                let manifest: LocalGameManifest = serde_json::from_str(&manifest_content).ok()?;

                // Check for ROM file - try registered extensions first, fall back to .wasm
                let rom_path = if let Some(registry) = registry {
                    // Try each registered extension, then fall back to .wasm
                    let wasm_fallback = path.join("rom.wasm");
                    registry
                        .supported_extensions()
                        .iter()
                        .map(|ext| path.join(format!("rom.{}", ext)))
                        .find(|p| p.exists())
                        .or_else(|| wasm_fallback.exists().then_some(wasm_fallback))?
                } else {
                    // Without registry, check for known ROM extensions
                    // Try ZX ROM format first, then fall back to .wasm
                    let zx_rom_path = path.join(format!("rom.{}", ZX_ROM_FORMAT.extension));
                    let wasm_path = path.join("rom.wasm");
                    if zx_rom_path.exists() {
                        zx_rom_path
                    } else if wasm_path.exists() {
                        wasm_path
                    } else {
                        return None; // Skip games with missing ROM files
                    }
                };

                return Some(LocalGame {
                    id: manifest.id,
                    title: manifest.title,
                    author: manifest.author,
                    version: manifest.version,
                    rom_path,
                    console_type: manifest.console_type,
                });
            }

            None
        })
        .collect()
}
