//! Authentication and error types.

use serde::{Deserialize, Serialize};

/// Authenticated user information.
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
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthResponse {
    /// JWT bearer token for Authorization header.
    pub token: String,
    /// Authenticated user profile.
    pub user: User,
}

/// Console linking code response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkCodeResponse {
    /// 6-character code for linking console to account.
    pub code: String,
    /// ISO 8601 timestamp when the code expires.
    pub expires_at: String,
}

/// API error response.
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
