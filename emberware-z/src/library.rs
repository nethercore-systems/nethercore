//! Game library management - re-exports from core
//!
//! Emberware Z uses the console-agnostic game library from `emberware_core`.
//! This module provides convenient re-exports and Z-specific implementations.

use emberware_core::library::DataDirProvider;
use std::path::PathBuf;

// Re-export all library types and functions from core
pub use emberware_core::library::{delete_game, get_local_games, is_cached, LocalGame};

/// Z's implementation of DataDirProvider.
///
/// Uses the standard Emberware data directory from config.
pub struct ZDataDirProvider;

impl DataDirProvider for ZDataDirProvider {
    fn data_dir(&self) -> Option<PathBuf> {
        crate::config::data_dir()
    }
}
