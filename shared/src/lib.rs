//! Shared types for Nethercore fantasy console platform.
//!
//! This crate provides general types shared between the Nethercore platform backend API
//! and all console clients. All types are serializable with serde for JSON API communication.
//!
//! Console-specific types (like Z ROM formats, asset formats) belong in their respective
//! console crates (e.g., `zx-common` for Nethercore ZX).
//!
//! # Type Categories
//!
//! - **Console Types**: [`ConsoleSpecs`] for fantasy console specifications
//! - **API Response Types**: [`Game`], [`Author`], [`GamesResponse`], [`RomUrlResponse`], [`VersionResponse`]
//! - **Auth Types**: [`User`], [`AuthResponse`], [`ApiError`]
//! - **Local Types**: [`LocalGameManifest`] for cached game metadata
//! - **Request Types**: [`RegisterRequest`], [`LoginRequest`], [`CreateGameRequest`], etc.
//! - **Math Types**: [`BoneMatrix3x4`] for skeletal animation
//!
//! # Example
//!
//! ```ignore
//! use nethercore_shared::{Game, GamesResponse};
//!
//! // Parse a games list response from the API
//! let json = r#"{"games": [], "total": 0, "page": 1, "limit": 10}"#;
//! let response: GamesResponse = serde_json::from_str(json).unwrap();
//! assert_eq!(response.total, 0);
//! ```

// Module declarations
pub mod api;
pub mod auth;
pub mod console;
pub mod constants;
pub mod fs;
pub mod ids;
pub mod local;
pub mod math;
pub mod netplay;
pub mod requests;
pub mod rom_format;
pub mod screenshot;

// Re-export all public items for convenience
pub use api::*;
pub use auth::*;
pub use console::*;
pub use constants::*;
pub use fs::*;
pub use ids::*;
pub use local::*;
pub use math::*;
pub use netplay::*;
pub use requests::*;
pub use rom_format::*;
