//! Local storage types.

use serde::{Deserialize, Serialize};

use crate::ZX_ROM_FORMAT;

/// Default console type for backward compatibility with old manifests.
fn default_console_type() -> String {
    ZX_ROM_FORMAT.console_type.to_string()
}

/// Cached game metadata stored locally.
///
/// Stored as `manifest.json` in `~/.nethercore/games/{game_id}/`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalGameManifest {
    /// Unique game identifier (matches API game ID).
    pub id: String,
    /// Game title for display.
    pub title: String,
    /// Author username for display.
    pub author: String,
    /// ROM version when downloaded.
    pub version: String,
    /// ISO 8601 timestamp when the ROM was downloaded.
    pub downloaded_at: String,
    /// Console type identifier ("zx", "chroma", etc.)
    ///
    /// Used by the unified launcher to determine which console to use.
    /// Defaults to "zx" for backward compatibility with old manifests.
    #[serde(default = "default_console_type")]
    pub console_type: String,
}
