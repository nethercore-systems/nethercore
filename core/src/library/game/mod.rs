//! Local game library management
//!
//! This module provides functions for managing locally downloaded games.
//! Games are stored in the platform's data directory under a `games` subdirectory.
//!
//! Supported formats:
//! - ROM files in the games directory (detected via RomLoaderRegistry)
//! - Subdirectories with `manifest.json` and `rom.wasm` (development, backward compatibility)

use std::path::PathBuf;

mod scanning;
mod operations;

#[cfg(test)]
mod tests;

// Re-export public API
pub use scanning::{get_local_games, get_local_games_with_loaders};
pub use operations::{is_cached, delete_game};

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
