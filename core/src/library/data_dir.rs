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
///         directories::ProjectDirs::from("io.emberware", "", "Emberware")
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

/// Default implementation of DataDirProvider using the standard Emberware data directory.
///
/// Uses `~/.emberware/` (or platform equivalent) for all consoles.
/// This is suitable for most use cases where consoles share the same data directory.
pub struct DefaultDataDirProvider;

impl DataDirProvider for DefaultDataDirProvider {
    fn data_dir(&self) -> Option<PathBuf> {
        crate::app::config::data_dir()
    }
}
