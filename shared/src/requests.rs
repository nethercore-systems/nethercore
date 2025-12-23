//! API request and response types.

use serde::{Deserialize, Serialize};

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
    /// Console type (e.g., "zx"). Required.
    pub console_type: String,
    /// Optional tags for categorization.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
    /// Content rating: "E" (Everyone), "T" (Teen), "M" (Mature 17+). Defaults to "E".
    #[serde(default = "default_content_rating")]
    pub content_rating: String,
    /// Content descriptor tags (e.g., "Violence", "Language").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_tags: Option<Vec<String>>,
}

fn default_content_rating() -> String {
    "E".to_string()
}

/// Update an existing game (developer endpoint).
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
    /// Update tags (replaces existing).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
    /// Update content rating: "E" (Everyone), "T" (Teen), "M" (Mature 17+).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_rating: Option<String>,
    /// Update content descriptor tags (replaces existing).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_tags: Option<Vec<String>>,
}

/// Response after creating a new game.
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
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UploadUrls {
    /// Upload URL for the WASM ROM file (required, max 32MB uncompressed).
    pub rom: String,
    /// Upload URL for the game icon (optional, 64x64 PNG).
    pub icon: String,
    /// Upload URLs for screenshots (optional, up to 5, 1280x720 PNG).
    pub screenshots: Vec<String>,
}

/// Generic success response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuccessResponse {
    /// Always `true` on success.
    pub success: bool,
}
