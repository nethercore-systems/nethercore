//! Game operations (cache checking, deletion)
//!
//! This module provides functions for managing individual games,
//! including checking if they are cached and deleting them.

use nethercore_shared::is_safe_game_id;

use crate::library::DataDirProvider;

#[cfg(test)]
use std::path::Path;

/// Checks if a game is cached locally.
///
/// Returns `true` if the game's ROM file exists at the expected path.
#[allow(dead_code)] // Public API for download flow
pub fn is_cached(provider: &dyn DataDirProvider, game_id: &str) -> bool {
    if !is_safe_game_id(game_id) {
        return false;
    }
    provider
        .data_dir()
        .map(|dir| dir.join("games").join(game_id).join("rom.wasm").exists())
        .unwrap_or(false)
}

/// Internal: Check if a game is cached in a specific directory.
/// Extracted for testability.
#[cfg(test)]
pub(super) fn is_cached_in_dir(games_dir: &Path, game_id: &str) -> bool {
    games_dir.join(game_id).join("rom.wasm").exists()
}

/// Deletes a cached game from the local filesystem.
///
/// Removes the entire game directory including manifest and ROM.
/// Returns `Ok(())` even if the game doesn't exist.
pub fn delete_game(provider: &dyn DataDirProvider, game_id: &str) -> std::io::Result<()> {
    if !is_safe_game_id(game_id) {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("Invalid game id: '{}'", game_id),
        ));
    }
    if let Some(dir) = provider.data_dir() {
        let game_dir = dir.join("games").join(game_id);
        if game_dir.exists() {
            std::fs::remove_dir_all(game_dir)?;
        }
    }
    Ok(())
}

/// Internal: Delete a game from a specific directory.
/// Extracted for testability.
#[cfg(test)]
pub(super) fn delete_game_in_dir(games_dir: &Path, game_id: &str) -> std::io::Result<()> {
    if !is_safe_game_id(game_id) {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("Invalid game id: '{}'", game_id),
        ));
    }
    let game_dir = games_dir.join(game_id);
    if game_dir.exists() {
        std::fs::remove_dir_all(game_dir)?;
    }
    Ok(())
}
