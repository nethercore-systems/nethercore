//! Console type registry for multi-console support
//!
//! This module provides the infrastructure for the unified launcher to
//! support multiple console types (Z, Classic, etc.) from a single binary.
//!
//! # Architecture
//!
//! The library spawns separate player processes for each console type:
//!
//! 1. `ConsoleType` enum represents all compile-time known console types
//! 2. `RomLoaderRegistry` manages ROM loaders for all console types
//! 3. `launch_player()` spawns the appropriate player binary
//! 4. Each console has its own player binary (e.g., `emberware-z`)
//!
//! # Adding a New Console
//!
//! 1. Create player binary for the console (e.g., `emberware-classic`)
//! 2. Add variant to `ConsoleType` enum (e.g., `Classic`)
//! 3. Update `as_str()` to return the manifest identifier (e.g., `"classic"`)
//! 4. Update `from_str()` to parse the identifier
//! 5. Update `all()` to include the new variant
//! 6. Update `player_binary_name()` to return the binary name
//! 7. Register the console's RomLoader in `create_rom_loader_registry()`
//!
//! # Benefits
//!
//! - Library has zero console-specific code
//! - Each console is fully isolated (crash isolation)
//! - Adding a new console requires no library changes
//! - Library can be replaced with any UI (web, native, CLI)

use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result};

use emberware_core::library::{LocalGame, RomLoaderRegistry};

use z_common::ZRomLoader;

/// Options to pass to the player process
#[derive(Debug, Clone, Default)]
pub struct PlayerOptions {
    /// Start in fullscreen mode
    pub fullscreen: bool,
    /// Enable debug overlay
    pub debug: bool,
}

/// Enum representing all available console types.
///
/// Uses static dispatch for zero-cost abstraction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ConsoleType {
    /// Emberware Z (PS1/N64 aesthetic)
    Z,
    // Future: Classic, Y, X, etc.
}

impl ConsoleType {
    /// Get the string identifier for this console type.
    ///
    /// This matches the `console_type` field in game manifests.
    pub fn as_str(&self) -> &'static str {
        match self {
            ConsoleType::Z => "z",
        }
    }

    /// Parse a console type from a string.
    ///
    /// Returns `None` if the string doesn't match any known console type.
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "z" => Some(ConsoleType::Z),
            _ => None,
        }
    }

    /// Get the ROM file extension for this console type.
    ///
    /// This is used when creating ROM files to determine the file extension.
    ///
    /// # Returns
    ///
    /// - `"ewz"` for Emberware Z
    /// - Future: `"ewc"` for Emberware Classic, etc.
    #[allow(dead_code)]
    pub fn rom_extension(&self) -> &'static str {
        match self {
            ConsoleType::Z => "ewz",
        }
    }

    /// Parse console type from ROM file extension.
    ///
    /// This allows detecting the console type from a ROM file's extension
    /// without needing to read the file contents.
    ///
    /// # Returns
    ///
    /// - `Some(ConsoleType::Z)` for `.ewz` files
    /// - `None` for unknown extensions
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext {
            "ewz" => Some(ConsoleType::Z),
            // Future: "ewc" => Some(ConsoleType::Classic),
            _ => None,
        }
    }

    /// Get all available console types.
    pub fn all() -> &'static [ConsoleType] {
        &[ConsoleType::Z]
    }

    /// Get the player binary name for this console type.
    ///
    /// This is the name of the executable that plays games for this console.
    pub fn player_binary_name(&self) -> &'static str {
        match self {
            ConsoleType::Z => "emberware-z",
            // Future: ConsoleType::Classic => "emberware-classic",
        }
    }
}

// =============================================================================
// Player Launching
// =============================================================================

/// Find the player binary for a console type.
///
/// Searches in order:
/// 1. Same directory as the library executable
/// 2. System PATH
///
/// Returns the full path to the player binary, or just the binary name
/// if it should be found in PATH.
pub fn find_player_binary(console_type: ConsoleType) -> PathBuf {
    let binary_name = console_type.player_binary_name();
    let exe_name = if cfg!(windows) {
        format!("{}.exe", binary_name)
    } else {
        binary_name.to_string()
    };

    // Try same directory as library executable
    if let Ok(exe) = std::env::current_exe()
        && let Some(dir) = exe.parent()
    {
        let player_path = dir.join(&exe_name);
        if player_path.exists() {
            return player_path;
        }
    }

    // Fall back to PATH
    PathBuf::from(exe_name)
}

/// Launch a game using the appropriate player process.
///
/// This spawns a new process for the player and returns immediately.
/// The library continues running while the game plays.
/// Use `run_player` if you want to wait for the player to finish.
pub fn launch_player(rom_path: &Path, console_type: ConsoleType) -> Result<()> {
    let player = find_player_binary(console_type);

    tracing::info!(
        "Launching player: {} {}",
        player.display(),
        rom_path.display()
    );

    Command::new(&player)
        .arg(rom_path)
        .spawn()
        .with_context(|| {
            format!(
                "Failed to launch player '{}'. Make sure it exists in the same directory as the library or in your PATH.",
                player.display()
            )
        })?;

    Ok(())
}

/// Run a game using the appropriate player process and wait for it to finish.
///
/// This is used when launching from CLI - the launcher process waits for the
/// player to exit before returning. No library UI is shown.
pub fn run_player(rom_path: &Path, console_type: ConsoleType) -> Result<()> {
    run_player_with_options(rom_path, console_type, &PlayerOptions::default())
}

/// Run a game with player options and wait for it to finish.
///
/// This is used when launching from CLI with flags like --fullscreen.
pub fn run_player_with_options(
    rom_path: &Path,
    console_type: ConsoleType,
    options: &PlayerOptions,
) -> Result<()> {
    let player = find_player_binary(console_type);

    tracing::info!(
        "Running player: {} {}{}{}",
        player.display(),
        rom_path.display(),
        if options.fullscreen {
            " --fullscreen"
        } else {
            ""
        },
        if options.debug { " --debug" } else { "" },
    );

    let mut cmd = Command::new(&player);
    cmd.arg(rom_path);

    if options.fullscreen {
        cmd.arg("--fullscreen");
    }
    if options.debug {
        cmd.arg("--debug");
    }

    let status = cmd.status().with_context(|| {
        format!(
            "Failed to run player '{}'. Make sure it exists in the same directory as the library or in your PATH.",
            player.display()
        )
    })?;

    if !status.success()
        && let Some(code) = status.code()
        && code != 0
    {
        // Exit code 0 is success, anything else is an error
        // But some exit codes are normal (e.g., user pressed ESC)
        tracing::debug!("Player exited with code: {}", code);
    }

    Ok(())
}

/// Launch a game by ID (spawns and returns immediately).
///
/// Looks up the game in the local games list and launches the appropriate player.
/// Used by the library UI when the user clicks Play.
pub fn launch_game_by_id(game: &LocalGame) -> Result<()> {
    let console_type = ConsoleType::parse(&game.console_type).ok_or_else(|| {
        anyhow::anyhow!(
            "Unknown console type: '{}'. Supported consoles: {}",
            game.console_type,
            ConsoleType::all()
                .iter()
                .map(|c| c.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        )
    })?;

    launch_player(&game.rom_path, console_type)
}

/// Run a game by ID and wait for it to finish.
///
/// Used when launching from CLI with a game ID argument.
pub fn run_game_by_id(game: &LocalGame) -> Result<()> {
    run_game_by_id_with_options(game, &PlayerOptions::default())
}

/// Run a game by ID with options and wait for it to finish.
pub fn run_game_by_id_with_options(game: &LocalGame, options: &PlayerOptions) -> Result<()> {
    let console_type = ConsoleType::parse(&game.console_type).ok_or_else(|| {
        anyhow::anyhow!(
            "Unknown console type: '{}'. Supported consoles: {}",
            game.console_type,
            ConsoleType::all()
                .iter()
                .map(|c| c.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        )
    })?;

    run_player_with_options(&game.rom_path, console_type, options)
}

/// Launch a game from a file path (spawns and returns immediately).
///
/// Detects the console type from the file extension.
/// Used by the library UI.
pub fn launch_game_from_path(path: &Path) -> Result<()> {
    let console_type = path
        .extension()
        .and_then(|e| e.to_str())
        .and_then(ConsoleType::from_extension)
        .ok_or_else(|| {
            anyhow::anyhow!(
                "Unknown ROM file type: {}. Supported extensions: .ewz",
                path.display()
            )
        })?;

    launch_player(path, console_type)
}

/// Run a game from a file path and wait for it to finish.
///
/// Detects the console type from the file extension.
/// Used when launching from CLI with a file path argument.
pub fn run_game_from_path(path: &Path) -> Result<()> {
    run_game_from_path_with_options(path, &PlayerOptions::default())
}

/// Run a game from a file path with options and wait for it to finish.
pub fn run_game_from_path_with_options(path: &Path, options: &PlayerOptions) -> Result<()> {
    let console_type = path
        .extension()
        .and_then(|e| e.to_str())
        .and_then(ConsoleType::from_extension)
        .ok_or_else(|| {
            anyhow::anyhow!(
                "Unknown ROM file type: {}. Supported extensions: .ewz",
                path.display()
            )
        })?;

    run_player_with_options(path, console_type, options)
}

// =============================================================================
// ROM Loader Registry
// =============================================================================

/// Create a ROM loader registry with all supported console ROM loaders.
///
/// This registers loaders for all supported ROM formats:
/// - `.ewz` files for Emberware Z
pub fn create_rom_loader_registry() -> RomLoaderRegistry {
    let mut registry = RomLoaderRegistry::new();
    registry.register(Box::new(ZRomLoader));
    // Future: registry.register(Box::new(ClassicRomLoader));
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
        // Library is now console-agnostic - it shows all games and spawns appropriate players
        crate::app::run().map_err(|e| anyhow::anyhow!("Library error: {}", e))
    }

    /// Get all available console type strings.
    #[allow(dead_code)]
    pub fn available_consoles(&self) -> Vec<&'static str> {
        ConsoleType::all().iter().map(|ct| ct.as_str()).collect()
    }

    /// Check if a console type is supported.
    #[allow(dead_code)]
    pub fn supports(&self, console_type: &str) -> bool {
        ConsoleType::parse(console_type).is_some()
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

    #[test]
    fn test_console_type_as_str() {
        assert_eq!(ConsoleType::Z.as_str(), "z");
    }

    #[test]
    fn test_console_type_parse_valid() {
        assert_eq!(ConsoleType::parse("z"), Some(ConsoleType::Z));
    }

    #[test]
    fn test_console_type_parse_invalid() {
        assert_eq!(ConsoleType::parse("invalid"), None);
        assert_eq!(ConsoleType::parse(""), None);
        assert_eq!(ConsoleType::parse("Z"), None); // Case-sensitive
        assert_eq!(ConsoleType::parse("classic"), None); // Not yet implemented
    }

    #[test]
    fn test_console_type_all() {
        let all = ConsoleType::all();
        assert_eq!(all.len(), 1);
        assert!(all.contains(&ConsoleType::Z));
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
        assert!(registry.supports("z"));
    }

    #[test]
    fn test_registry_supports_invalid() {
        let registry = ConsoleRegistry::new();
        assert!(!registry.supports("invalid"));
        assert!(!registry.supports("classic")); // Not yet implemented
        assert!(!registry.supports(""));
        assert!(!registry.supports("Z")); // Case-sensitive
    }

    #[test]
    fn test_registry_available_consoles() {
        let registry = ConsoleRegistry::new();
        let consoles = registry.available_consoles();
        assert_eq!(consoles, vec!["z"]);
    }

    #[test]
    fn test_registry_default() {
        let registry = ConsoleRegistry::default();
        assert!(registry.supports("z"));
    }

    #[test]
    fn test_console_type_player_binary_name() {
        assert_eq!(ConsoleType::Z.player_binary_name(), "emberware-z");
    }

    #[test]
    fn test_console_type_rom_extension() {
        assert_eq!(ConsoleType::Z.rom_extension(), "ewz");
    }

    #[test]
    fn test_console_type_from_extension_valid() {
        assert_eq!(ConsoleType::from_extension("ewz"), Some(ConsoleType::Z));
    }

    #[test]
    fn test_console_type_from_extension_invalid() {
        assert_eq!(ConsoleType::from_extension("invalid"), None);
        assert_eq!(ConsoleType::from_extension(""), None);
        assert_eq!(ConsoleType::from_extension("EWZ"), None); // Case-sensitive
        assert_eq!(ConsoleType::from_extension("ewc"), None); // Not yet implemented
    }
}
