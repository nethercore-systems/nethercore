//! ROM loader registry and console registry.

use anyhow::Result;

use nethercore_core::library::{LocalGame, RomLoaderRegistry};

use zx_common::ZXRomLoader;

use super::helpers::{console_type_from_str, supported_console_types};
use super::launcher::PlayerOptions;
use super::player::{
    launch_game_by_id, launch_game_from_path, run_game_by_id, run_game_by_id_with_options,
    run_game_from_path, run_game_from_path_with_options,
};

use std::path::PathBuf;

/// Create a ROM loader registry with all supported console ROM loaders.
///
/// This registers loaders for all supported ROM formats:
/// - `.nczx` files for Nethercore ZX
pub fn create_rom_loader_registry() -> RomLoaderRegistry {
    let mut registry = RomLoaderRegistry::new();
    registry.register(Box::new(ZXRomLoader));
    // Future: registry.register(Box::new(ChromaRomLoader));
    registry
}

/// Registry of all available console types.
///
/// Provides lookup and validation for console types based on game manifests.
/// Uses player process spawning for game execution.
pub struct ConsoleRegistry {
    // No fields needed - all console types are compile-time known
}

impl ConsoleRegistry {
    /// Create a new registry.
    pub fn new() -> Self {
        Self {}
    }

    /// Launch a game using the appropriate player process.
    ///
    /// Spawns a new process and returns immediately.
    /// The library stays open while the game plays.
    pub fn launch_game(&self, game: &LocalGame) -> Result<()> {
        launch_game_by_id(game)
    }

    /// Run a game and wait for it to finish.
    ///
    /// Used when launching from CLI - no library UI is shown.
    pub fn run_game(&self, game: &LocalGame) -> Result<()> {
        run_game_by_id(game)
    }

    /// Run a game with options and wait for it to finish.
    ///
    /// Used when launching from CLI with flags like --fullscreen.
    pub fn run_game_with_options(&self, game: &LocalGame, options: &PlayerOptions) -> Result<()> {
        run_game_by_id_with_options(game, options)
    }

    /// Launch a game directly from a file path.
    ///
    /// Detects the console type from the file extension and spawns the player.
    pub fn launch_from_path(&self, path: PathBuf) -> Result<()> {
        launch_game_from_path(&path)
    }

    /// Run a game from a file path and wait for it to finish.
    ///
    /// Used when launching from CLI - no library UI is shown.
    pub fn run_from_path(&self, path: PathBuf) -> Result<()> {
        run_game_from_path(&path)
    }

    /// Run a game from a file path with options and wait for it to finish.
    pub fn run_from_path_with_options(&self, path: PathBuf, options: &PlayerOptions) -> Result<()> {
        run_game_from_path_with_options(&path, options)
    }

    /// Launch the library UI.
    ///
    /// This runs the library UI in the current process.
    /// Games are launched as separate player processes.
    pub fn launch_library(&self) -> Result<()> {
        // Library is console-agnostic - it shows all games and spawns appropriate players
        crate::app::run().map_err(|e| anyhow::anyhow!("Library error: {}", e))
    }

    /// Get all available console type strings.
    #[allow(dead_code)]
    pub fn available_consoles(&self) -> Vec<&'static str> {
        supported_console_types()
            .iter()
            .map(|ct| ct.as_str())
            .collect()
    }

    /// Check if a console type is supported.
    #[allow(dead_code)]
    pub fn supports(&self, console_type: &str) -> bool {
        console_type_from_str(console_type).is_some()
    }
}

impl Default for ConsoleRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nethercore_shared::ConsoleType;

    #[test]
    fn test_console_type_as_str() {
        assert_eq!(ConsoleType::ZX.as_str(), "zx");
    }

    #[test]
    fn test_registry_new() {
        let registry = ConsoleRegistry::new();
        // Should not panic and should be usable
        assert!(!registry.available_consoles().is_empty());
    }

    #[test]
    fn test_registry_supports_valid() {
        let registry = ConsoleRegistry::new();
        assert!(registry.supports("zx"));
    }

    #[test]
    fn test_registry_supports_invalid() {
        let registry = ConsoleRegistry::new();
        assert!(!registry.supports("invalid"));
        assert!(!registry.supports("chroma")); // No ROM format yet
        assert!(!registry.supports(""));
        assert!(!registry.supports("ZX")); // Case-sensitive
    }

    #[test]
    fn test_registry_available_consoles() {
        let registry = ConsoleRegistry::new();
        let consoles = registry.available_consoles();
        assert_eq!(consoles, vec!["zx"]);
    }

    #[test]
    fn test_registry_default() {
        let registry = ConsoleRegistry::default();
        assert!(registry.supports("zx"));
    }
}
