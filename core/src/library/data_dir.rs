//! Data directory abstraction for console-specific paths

use std::path::PathBuf;

/// Trait for providing platform-specific data directory paths.
///
/// Each console implementation can provide its own data directory strategy.
/// This allows core library code to be console-agnostic while still accessing
/// the correct filesystem locations.
///
/// # Example
///
/// ```rust,ignore
/// use emberware_core::library::DataDirProvider;
/// use std::path::PathBuf;
///
/// struct ZDataDirProvider;
///
/// impl DataDirProvider for ZDataDirProvider {
///     fn data_dir(&self) -> Option<PathBuf> {
///         directories::ProjectDirs::from("io", "emberware", "emberware")
///             .map(|dirs| dirs.data_dir().to_path_buf())
///     }
/// }
/// ```
pub trait DataDirProvider: Send + Sync {
    /// Returns the platform-specific data directory path.
    ///
    /// This is where games are stored locally (typically `~/.emberware/games/`
    /// or platform equivalent).
    ///
    /// Returns `None` if the home directory cannot be determined or the
    /// platform doesn't support data directories.
    fn data_dir(&self) -> Option<PathBuf>;
}

// Note: Use `emberware_core::app::config::data_dir()` for the default path.
// Consoles provide their own DataDirProvider implementation (e.g., ZDataDirProvider).
