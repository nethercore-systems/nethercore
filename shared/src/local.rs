//! Local storage types.

use serde::{Deserialize, Serialize};

/// Cached game metadata stored locally.
///
/// Stored as `manifest.json` in `~/.emberware/games/{game_id}/`.
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
}
