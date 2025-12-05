//! API response types for Emberware platform.

use serde::{Deserialize, Serialize};

/// A game author (developer) profile.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Author {
    /// Unique author identifier (UUID).
    pub id: String,
    /// Display name chosen by the author.
    pub username: String,
}

/// A game published on the Emberware platform.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Game {
    /// Unique game identifier (UUID).
    pub id: String,
    /// Display title shown in the library.
    pub title: String,
    /// URL-safe identifier (lowercase, hyphens, e.g., "super-bounce").
    pub slug: String,
    /// Full game description (Markdown supported).
    pub description: String,
    /// Console type ("classic" or "z").
    pub console_type: String,
    /// URL to the game's icon image (64x64 PNG recommended).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon_url: Option<String>,
    /// URLs to screenshot images (up to 5, 1280x720 PNG recommended).
    /// Empty vec if no screenshots available.
    pub screenshots: Vec<String>,
    /// ROM file size in bytes (for download progress).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rom_size: Option<i64>,
    /// Semantic version of the current ROM (e.g., "1.2.3").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rom_version: Option<String>,
    /// Whether the game is publicly visible in the library.
    pub published: bool,
    /// The game's author.
    pub author: Author,
    /// ISO 8601 timestamp when the game was created.
    pub created_at: String,
    /// ISO 8601 timestamp when the game was last updated.
    pub updated_at: String,
}

/// Paginated list of games from the API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GamesResponse {
    /// Games on the current page.
    pub games: Vec<Game>,
    /// Total number of games matching the query.
    pub total: i64,
    /// Current page number (1-indexed).
    pub page: u32,
    /// Maximum games per page.
    pub limit: u32,
}

/// Temporary signed URL for ROM download.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RomUrlResponse {
    /// Pre-signed download URL (expires after `expires_at`).
    pub url: String,
    /// ISO 8601 timestamp when the URL expires.
    pub expires_at: String,
}

/// Current version information for a game's ROM.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionResponse {
    /// Semantic version string (e.g., "1.2.3"), or None if no ROM uploaded yet.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    /// ROM file size in bytes.
    pub rom_size: i64,
}
