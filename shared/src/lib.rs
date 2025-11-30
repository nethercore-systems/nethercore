//! Shared types for Emberware fantasy console platform.
//!
//! This crate provides types shared between the Emberware platform backend API
//! and console clients (Emberware Z, etc.). All types are serializable with serde
//! for JSON API communication.
//!
//! # Type Categories
//!
//! - **API Response Types**: [`Game`], [`Author`], [`GamesResponse`], [`RomUrlResponse`], [`VersionResponse`]
//! - **Auth Types**: [`User`], [`AuthResponse`], [`ApiError`]
//! - **Local Types**: [`LocalGameManifest`] for cached game metadata
//! - **Request Types**: [`RegisterRequest`], [`LoginRequest`], [`CreateGameRequest`], etc.
//!
//! # Example
//!
//! ```ignore
//! use emberware_shared::{Game, GamesResponse};
//!
//! // Parse a games list response from the API
//! let json = r#"{"games": [], "total": 0, "page": 1, "limit": 10}"#;
//! let response: GamesResponse = serde_json::from_str(json).unwrap();
//! assert_eq!(response.total, 0);
//! ```

use serde::{Deserialize, Serialize};

// === API Response Types ===

/// A game author (developer) profile.
///
/// Contains minimal author information embedded in [`Game`] responses.
/// For full author profiles, query the author endpoint directly.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Author {
    /// Unique author identifier (UUID).
    pub id: String,
    /// Display name chosen by the author.
    pub username: String,
}

/// A game published on the Emberware platform.
///
/// Games are WASM modules that run on Emberware fantasy consoles.
/// Each game has a unique ID and slug for URLs, along with metadata
/// like title, description, and screenshots.
///
/// # Fields
///
/// Required fields are always present. Optional fields are omitted
/// from JSON when `None` to reduce response size.
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
    /// URL to the game's icon image (64x64 PNG recommended).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon_url: Option<String>,
    /// URLs to screenshot images (up to 5, 1280x720 PNG recommended).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub screenshots: Option<Vec<String>>,
    /// ROM file size in bytes (for download progress).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rom_size: Option<u64>,
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
///
/// Used for library browsing. Supports cursor-based pagination
/// with configurable page size.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GamesResponse {
    /// Games on the current page.
    pub games: Vec<Game>,
    /// Total number of games matching the query.
    pub total: u32,
    /// Current page number (1-indexed).
    pub page: u32,
    /// Maximum games per page.
    pub limit: u32,
}

/// Temporary signed URL for ROM download.
///
/// ROM downloads use pre-signed URLs that expire after a short time
/// for security. Clients should begin download immediately after
/// receiving this response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RomUrlResponse {
    /// Pre-signed download URL (expires after `expires_at`).
    pub url: String,
    /// ISO 8601 timestamp when the URL expires.
    pub expires_at: String,
}

/// Current version information for a game's ROM.
///
/// Used to check if a cached ROM is outdated and needs re-downloading.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionResponse {
    /// Semantic version string (e.g., "1.2.3").
    pub version: String,
    /// ROM file size in bytes.
    pub rom_size: u64,
}

// === Auth Types ===

/// Authenticated user information.
///
/// Returned after successful login or registration.
/// Contains user profile data for display in the client UI.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    /// Unique user identifier (UUID).
    pub id: String,
    /// Display name chosen by the user.
    pub username: String,
    /// User's email address (for account recovery).
    pub email: String,
    /// ISO 8601 timestamp when the account was created.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
}

/// Successful authentication response.
///
/// Contains a JWT token for subsequent API requests and the
/// authenticated user's profile.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthResponse {
    /// JWT bearer token for Authorization header.
    pub token: String,
    /// Authenticated user profile.
    pub user: User,
}

// === Error Types ===

/// API error response.
///
/// All API errors return this structure with a human-readable message
/// and machine-readable error code. See [`error_codes`] for standard codes.
///
/// # Example
///
/// ```
/// use emberware_shared::ApiError;
///
/// let error = ApiError {
///     error: "Game not found".to_string(),
///     code: "NOT_FOUND".to_string(),
/// };
/// assert_eq!(format!("{}", error), "Game not found (NOT_FOUND)");
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiError {
    /// Human-readable error message.
    pub error: String,
    /// Machine-readable error code (see [`error_codes`]).
    pub code: String,
}

impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ({})", self.error, self.code)
    }
}

impl std::error::Error for ApiError {}

/// Standard API error codes.
///
/// These codes help clients handle errors programmatically without
/// parsing error messages.
pub mod error_codes {
    /// Authentication required but not provided or invalid.
    pub const UNAUTHORIZED: &str = "UNAUTHORIZED";
    /// User authenticated but lacks permission for this action.
    pub const FORBIDDEN: &str = "FORBIDDEN";
    /// Requested resource does not exist.
    pub const NOT_FOUND: &str = "NOT_FOUND";
    /// Request data failed validation (check error message for details).
    pub const VALIDATION_ERROR: &str = "VALIDATION_ERROR";
    /// Resource already exists (e.g., duplicate username).
    pub const CONFLICT: &str = "CONFLICT";
}

// === Local Types ===

/// Cached game metadata stored locally.
///
/// This is saved alongside downloaded ROMs in the local game cache.
/// Contains enough information to display the game in the library
/// without fetching from the API.
///
/// # Storage Location
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

// === Request Types ===

/// User registration request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterRequest {
    /// Email address (must be unique, used for login).
    pub email: String,
    /// Display name (must be unique, 3-32 characters).
    pub username: String,
    /// Password (minimum 8 characters).
    pub password: String,
}

/// User login request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginRequest {
    /// Email address used during registration.
    pub email: String,
    /// Account password.
    pub password: String,
}

/// Create a new game (developer endpoint).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateGameRequest {
    /// Game title (3-64 characters).
    pub title: String,
    /// Optional description (Markdown, max 4096 characters).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// Update an existing game (developer endpoint).
///
/// All fields are optional; only provided fields are updated.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateGameRequest {
    /// New title (3-64 characters).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// New description (Markdown, max 4096 characters).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Set published status (true = visible in library).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub published: Option<bool>,
}

/// Response after creating a new game.
///
/// Contains pre-signed upload URLs for the ROM and assets.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateGameResponse {
    /// Unique game identifier (UUID).
    pub id: String,
    /// Game title.
    pub title: String,
    /// Generated URL slug.
    pub slug: String,
    /// Pre-signed upload URLs for game assets.
    pub upload_urls: UploadUrls,
}

/// Pre-signed URLs for uploading game assets.
///
/// These URLs expire after 1 hour. Upload the ROM first,
/// then optionally upload icon and screenshots.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UploadUrls {
    /// Upload URL for the WASM ROM file (required, max 32MB).
    pub rom: String,
    /// Upload URL for the game icon (optional, 64x64 PNG).
    pub icon: String,
    /// Upload URLs for screenshots (optional, up to 5, 1280x720 PNG).
    pub screenshots: Vec<String>,
}

/// Generic success response.
///
/// Used for operations that don't return data, like DELETE requests.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuccessResponse {
    /// Always `true` on success.
    pub success: bool,
}
