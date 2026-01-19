//! Console type registry for multi-console support
//!
//! This module provides the infrastructure for the unified launcher to
//! support multiple console types (ZX, Chroma, etc.) from a single binary.
//!
//! # Architecture
//!
//! The library spawns separate player processes for each console type:
//!
//! 1. `ConsoleType` (from nethercore_shared) represents known console identifiers
//! 2. `RomLoaderRegistry` manages ROM loaders for supported console types
//! 3. `launch_player()` spawns the appropriate player binary
//! 4. Each console has its own player binary (e.g., `nethercore-zx`)
//!
//! # Adding a New Console
//!
//! 1. Create player binary for the console (e.g., `nethercore-chroma`)
//! 2. Add a `ConsoleType` variant in nethercore_shared (e.g., `Chroma`)
//! 3. Add a ROM format entry in nethercore_shared::rom_format
//! 4. Update `player_binary_name()` to return the binary name
//! 5. Register the console's RomLoader in `create_rom_loader_registry()`
//!
//! # Benefits
//!
//! - Library has zero console-specific code
//! - Each console is fully isolated (crash isolation)
//! - Adding a new console requires no library changes
//! - Library can be replaced with any UI (web, native, CLI)

mod helpers;
mod launcher;
mod player;
mod rom_loader;

// Re-export public API
pub use launcher::{ConnectionMode, LaunchTarget, PlayerLauncher, PlayerOptions};
pub use player::{
    find_player_binary, launch_game_by_id, launch_game_by_id_with_options, launch_game_from_path,
    launch_player, launch_player_with_options, run_game_by_id, run_game_by_id_with_options,
    run_game_from_path, run_game_from_path_with_options, run_player, run_player_with_options,
};
pub use rom_loader::{ConsoleRegistry, create_rom_loader_registry};
