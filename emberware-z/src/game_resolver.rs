//! Game ID resolution - re-exports from core
//!
//! Emberware Z uses the console-agnostic game resolver from `emberware_core`.

// Re-export all resolver types and functions from core
pub use emberware_core::library::{resolve_game_id, GameResolutionError};
