//! Local game library management
//!
//! This module provides functions for managing locally downloaded games.
//! Games are stored in the platform's data directory under a `games` subdirectory.
//!
//! Supported formats:
//! - ROM files in the games directory (detected via RomLoaderRegistry)
//! - Subdirectories with `manifest.json` and `rom.wasm` (development, backward compatibility)

use emberware_shared::{LocalGameManifest, ZX_ROM_FORMAT};
use std::path::{Path, PathBuf};

use super::DataDirProvider;
use super::rom::RomLoaderRegistry;

/// A locally cached game with its metadata and ROM path.
#[derive(Debug, Clone)]
pub struct LocalGame {
    /// Unique game identifier
    pub id: String,
    /// Display title of the game
    pub title: String,
    /// Game author's name
    pub author: String,
    /// Version string (semantic versioning recommended)
    #[allow(dead_code)] // Will be displayed in UI
    pub version: String,
    /// Path to the WASM ROM file
    pub rom_path: PathBuf,
    /// Console type identifier ("z", "classic", etc.)
    pub console_type: String,
}

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
fn get_games_from_dir(games_dir: &Path, registry: Option<&RomLoaderRegistry>) -> Vec<LocalGame> {
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
                let rom_bytes = std::fs::read(&path).ok()?;
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

/// Checks if a game is cached locally.
///
/// Returns `true` if the game's ROM file exists at the expected path.
#[allow(dead_code)] // Public API for download flow
pub fn is_cached(provider: &dyn DataDirProvider, game_id: &str) -> bool {
    provider
        .data_dir()
        .map(|dir| dir.join("games").join(game_id).join("rom.wasm").exists())
        .unwrap_or(false)
}

/// Internal: Check if a game is cached in a specific directory.
/// Extracted for testability.
#[cfg(test)]
fn is_cached_in_dir(games_dir: &Path, game_id: &str) -> bool {
    games_dir.join(game_id).join("rom.wasm").exists()
}

/// Deletes a cached game from the local filesystem.
///
/// Removes the entire game directory including manifest and ROM.
/// Returns `Ok(())` even if the game doesn't exist.
pub fn delete_game(provider: &dyn DataDirProvider, game_id: &str) -> std::io::Result<()> {
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
fn delete_game_in_dir(games_dir: &Path, game_id: &str) -> std::io::Result<()> {
    let game_dir = games_dir.join(game_id);
    if game_dir.exists() {
        std::fs::remove_dir_all(game_dir)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    // =============================================================
    // Helper function to create a test game directory
    // =============================================================

    fn create_test_game(games_dir: &Path, id: &str, title: &str, author: &str, version: &str) {
        let game_dir = games_dir.join(id);
        fs::create_dir_all(&game_dir).expect("failed to create test game directory");

        let manifest = serde_json::json!({
            "id": id,
            "title": title,
            "author": author,
            "version": version,
            "downloaded_at": "2024-01-01T00:00:00Z",
            "console_type": "z"
        });

        fs::write(
            game_dir.join("manifest.json"),
            serde_json::to_string(&manifest).expect("failed to serialize test manifest"),
        )
        .expect("failed to write test manifest.json");

        // Create a dummy ROM file
        fs::write(game_dir.join("rom.wasm"), b"dummy wasm content")
            .expect("failed to write test rom.wasm");
    }

    // =============================================================
    // get_games_from_dir tests
    // =============================================================

    #[test]
    fn test_get_games_from_empty_dir() {
        let temp_dir = TempDir::new().unwrap();
        let games = get_games_from_dir(temp_dir.path(), None);
        assert!(games.is_empty());
    }

    #[test]
    fn test_get_games_from_nonexistent_dir() {
        let games = get_games_from_dir(Path::new("/nonexistent/path/that/does/not/exist"), None);
        assert!(games.is_empty());
    }

    #[test]
    fn test_get_games_single_game() {
        let temp_dir = TempDir::new().unwrap();
        create_test_game(temp_dir.path(), "my-game", "My Game", "Dev Name", "1.0.0");

        let games = get_games_from_dir(temp_dir.path(), None);
        assert_eq!(games.len(), 1);
        assert_eq!(games[0].id, "my-game");
        assert_eq!(games[0].title, "My Game");
        assert_eq!(games[0].author, "Dev Name");
        assert_eq!(games[0].version, "1.0.0");
        assert!(games[0].rom_path.ends_with("rom.wasm"));
    }

    #[test]
    fn test_get_games_multiple_games() {
        let temp_dir = TempDir::new().unwrap();
        create_test_game(temp_dir.path(), "game-1", "Game One", "Author A", "1.0.0");
        create_test_game(temp_dir.path(), "game-2", "Game Two", "Author B", "2.0.0");
        create_test_game(temp_dir.path(), "game-3", "Game Three", "Author C", "3.0.0");

        let games = get_games_from_dir(temp_dir.path(), None);
        assert_eq!(games.len(), 3);

        // Verify all games are present (order may vary)
        let ids: Vec<&str> = games.iter().map(|g| g.id.as_str()).collect();
        assert!(ids.contains(&"game-1"));
        assert!(ids.contains(&"game-2"));
        assert!(ids.contains(&"game-3"));
    }

    #[test]
    fn test_get_games_skips_files_not_directories() {
        let temp_dir = TempDir::new().unwrap();
        create_test_game(
            temp_dir.path(),
            "valid-game",
            "Valid Game",
            "Author",
            "1.0.0",
        );

        // Create a file (not a directory) in the games dir
        fs::write(temp_dir.path().join("not-a-dir.txt"), "some content").unwrap();

        let games = get_games_from_dir(temp_dir.path(), None);
        assert_eq!(games.len(), 1);
        assert_eq!(games[0].id, "valid-game");
    }

    #[test]
    fn test_get_games_skips_missing_manifest() {
        let temp_dir = TempDir::new().unwrap();
        create_test_game(
            temp_dir.path(),
            "valid-game",
            "Valid Game",
            "Author",
            "1.0.0",
        );

        // Create a game directory without manifest
        let invalid_dir = temp_dir.path().join("invalid-game");
        fs::create_dir_all(&invalid_dir).unwrap();
        fs::write(invalid_dir.join("rom.wasm"), b"dummy").unwrap();

        let games = get_games_from_dir(temp_dir.path(), None);
        assert_eq!(games.len(), 1);
        assert_eq!(games[0].id, "valid-game");
    }

    #[test]
    fn test_get_games_skips_invalid_json_manifest() {
        let temp_dir = TempDir::new().unwrap();
        create_test_game(
            temp_dir.path(),
            "valid-game",
            "Valid Game",
            "Author",
            "1.0.0",
        );

        // Create a game directory with invalid JSON manifest
        let invalid_dir = temp_dir.path().join("invalid-json");
        fs::create_dir_all(&invalid_dir).unwrap();
        fs::write(invalid_dir.join("manifest.json"), "not valid json {{{").unwrap();

        let games = get_games_from_dir(temp_dir.path(), None);
        assert_eq!(games.len(), 1);
        assert_eq!(games[0].id, "valid-game");
    }

    #[test]
    fn test_get_games_skips_incomplete_manifest() {
        let temp_dir = TempDir::new().unwrap();
        create_test_game(
            temp_dir.path(),
            "valid-game",
            "Valid Game",
            "Author",
            "1.0.0",
        );

        // Create a game directory with incomplete manifest (missing required fields)
        let invalid_dir = temp_dir.path().join("incomplete");
        fs::create_dir_all(&invalid_dir).unwrap();
        fs::write(
            invalid_dir.join("manifest.json"),
            r#"{"id": "incomplete"}"#, // Missing title, author, version, downloaded_at
        )
        .unwrap();

        let games = get_games_from_dir(temp_dir.path(), None);
        assert_eq!(games.len(), 1);
        assert_eq!(games[0].id, "valid-game");
    }

    #[test]
    fn test_get_games_rom_path_correct() {
        let temp_dir = TempDir::new().unwrap();
        create_test_game(temp_dir.path(), "path-test", "Path Test", "Author", "1.0.0");

        let games = get_games_from_dir(temp_dir.path(), None);
        assert_eq!(games.len(), 1);

        let expected_rom_path = temp_dir.path().join("path-test").join("rom.wasm");
        assert_eq!(games[0].rom_path, expected_rom_path);
    }

    // =============================================================
    // is_cached_in_dir tests
    // =============================================================

    #[test]
    fn test_is_cached_not_present() {
        let temp_dir = TempDir::new().unwrap();
        assert!(!is_cached_in_dir(temp_dir.path(), "nonexistent-game"));
    }

    #[test]
    fn test_is_cached_directory_only() {
        let temp_dir = TempDir::new().unwrap();
        let game_dir = temp_dir.path().join("partial-game");
        fs::create_dir_all(&game_dir).unwrap();

        // No rom.wasm file
        assert!(!is_cached_in_dir(temp_dir.path(), "partial-game"));
    }

    #[test]
    fn test_is_cached_with_rom() {
        let temp_dir = TempDir::new().unwrap();
        let game_dir = temp_dir.path().join("cached-game");
        fs::create_dir_all(&game_dir).unwrap();
        fs::write(game_dir.join("rom.wasm"), b"wasm content").unwrap();

        assert!(is_cached_in_dir(temp_dir.path(), "cached-game"));
    }

    #[test]
    fn test_is_cached_complete_game() {
        let temp_dir = TempDir::new().unwrap();
        create_test_game(
            temp_dir.path(),
            "complete-game",
            "Complete",
            "Author",
            "1.0.0",
        );

        assert!(is_cached_in_dir(temp_dir.path(), "complete-game"));
    }

    // =============================================================
    // delete_game_in_dir tests
    // =============================================================

    #[test]
    fn test_delete_nonexistent_game() {
        let temp_dir = TempDir::new().unwrap();
        // Should not error when deleting a game that doesn't exist
        let result = delete_game_in_dir(temp_dir.path(), "nonexistent");
        assert!(result.is_ok());
    }

    #[test]
    fn test_delete_existing_game() {
        let temp_dir = TempDir::new().unwrap();
        create_test_game(temp_dir.path(), "to-delete", "To Delete", "Author", "1.0.0");

        // Verify game exists
        assert!(temp_dir.path().join("to-delete").exists());

        // Delete the game
        let result = delete_game_in_dir(temp_dir.path(), "to-delete");
        assert!(result.is_ok());

        // Verify game is gone
        assert!(!temp_dir.path().join("to-delete").exists());
    }

    #[test]
    fn test_delete_removes_all_contents() {
        let temp_dir = TempDir::new().unwrap();
        create_test_game(temp_dir.path(), "full-game", "Full Game", "Author", "1.0.0");

        let game_dir = temp_dir.path().join("full-game");

        // Add some extra files
        fs::write(game_dir.join("extra.txt"), "extra content").unwrap();
        fs::create_dir_all(game_dir.join("subdir")).unwrap();
        fs::write(game_dir.join("subdir").join("nested.txt"), "nested").unwrap();

        // Delete the game
        let result = delete_game_in_dir(temp_dir.path(), "full-game");
        assert!(result.is_ok());

        // Everything should be gone
        assert!(!game_dir.exists());
    }

    #[test]
    fn test_delete_leaves_other_games_intact() {
        let temp_dir = TempDir::new().unwrap();
        create_test_game(temp_dir.path(), "keep-this", "Keep This", "Author", "1.0.0");
        create_test_game(
            temp_dir.path(),
            "delete-this",
            "Delete This",
            "Author",
            "1.0.0",
        );

        // Delete one game
        let result = delete_game_in_dir(temp_dir.path(), "delete-this");
        assert!(result.is_ok());

        // Other game should still exist
        assert!(temp_dir.path().join("keep-this").exists());
        assert!(!temp_dir.path().join("delete-this").exists());
    }

    // =============================================================
    // Integration-style tests (using internal helpers)
    // =============================================================

    #[test]
    fn test_full_workflow_add_list_delete() {
        let temp_dir = TempDir::new().unwrap();

        // Initially empty
        let games = get_games_from_dir(temp_dir.path(), None);
        assert!(games.is_empty());

        // Add a game
        create_test_game(temp_dir.path(), "workflow-game", "Workflow", "Dev", "1.0.0");

        // Should now appear in list
        let games = get_games_from_dir(temp_dir.path(), None);
        assert_eq!(games.len(), 1);
        assert!(is_cached_in_dir(temp_dir.path(), "workflow-game"));

        // Delete the game
        delete_game_in_dir(temp_dir.path(), "workflow-game").unwrap();

        // Should be gone from list
        let games = get_games_from_dir(temp_dir.path(), None);
        assert!(games.is_empty());
        assert!(!is_cached_in_dir(temp_dir.path(), "workflow-game"));
    }

    #[test]
    fn test_special_characters_in_game_id() {
        let temp_dir = TempDir::new().unwrap();

        // Game IDs with various characters (avoiding path separators)
        create_test_game(
            temp_dir.path(),
            "game-with-dashes",
            "Dashes",
            "Author",
            "1.0.0",
        );
        create_test_game(
            temp_dir.path(),
            "game_with_underscores",
            "Underscores",
            "Author",
            "1.0.0",
        );
        create_test_game(temp_dir.path(), "game.with.dots", "Dots", "Author", "1.0.0");

        let games = get_games_from_dir(temp_dir.path(), None);
        assert_eq!(games.len(), 3);

        // All should be cached
        assert!(is_cached_in_dir(temp_dir.path(), "game-with-dashes"));
        assert!(is_cached_in_dir(temp_dir.path(), "game_with_underscores"));
        assert!(is_cached_in_dir(temp_dir.path(), "game.with.dots"));
    }

    #[test]
    fn test_unicode_in_game_metadata() {
        let temp_dir = TempDir::new().unwrap();

        // Create a game with unicode in title and author
        let game_dir = temp_dir.path().join("unicode-game");
        fs::create_dir_all(&game_dir).unwrap();

        let manifest = serde_json::json!({
            "id": "unicode-game",
            "title": "日本語ゲーム",
            "author": "开发者",
            "version": "1.0.0",
            "downloaded_at": "2024-01-01T00:00:00Z"
        });

        fs::write(
            game_dir.join("manifest.json"),
            serde_json::to_string(&manifest).unwrap(),
        )
        .unwrap();
        fs::write(game_dir.join("rom.wasm"), b"dummy").unwrap();

        let games = get_games_from_dir(temp_dir.path(), None);
        assert_eq!(games.len(), 1);
        assert_eq!(games[0].title, "日本語ゲーム");
        assert_eq!(games[0].author, "开发者");
    }

    #[test]
    fn test_empty_strings_in_manifest() {
        let temp_dir = TempDir::new().unwrap();

        let game_dir = temp_dir.path().join("empty-strings");
        fs::create_dir_all(&game_dir).unwrap();

        let manifest = serde_json::json!({
            "id": "",
            "title": "",
            "author": "",
            "version": "",
            "downloaded_at": "2024-01-01T00:00:00Z"
        });

        fs::write(
            game_dir.join("manifest.json"),
            serde_json::to_string(&manifest).unwrap(),
        )
        .unwrap();
        fs::write(game_dir.join("rom.wasm"), b"dummy").unwrap();

        let games = get_games_from_dir(temp_dir.path(), None);
        assert_eq!(games.len(), 1);
        assert_eq!(games[0].id, "");
        assert_eq!(games[0].title, "");
    }

    #[test]
    fn test_very_long_game_id() {
        let temp_dir = TempDir::new().unwrap();
        let long_id = "a".repeat(200);

        create_test_game(temp_dir.path(), &long_id, "Long ID Game", "Author", "1.0.0");

        let games = get_games_from_dir(temp_dir.path(), None);
        assert_eq!(games.len(), 1);
        assert_eq!(games[0].id, long_id);
        assert!(is_cached_in_dir(temp_dir.path(), &long_id));
    }
}
