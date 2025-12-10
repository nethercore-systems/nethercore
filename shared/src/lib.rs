//! Shared types for Emberware fantasy console platform.
//!
//! This crate provides types shared between the Emberware platform backend API
//! and console clients (Emberware Z, etc.). All types are serializable with serde
//! for JSON API communication.
//!
//! # Type Categories
//!
//! - **Console Types**: [`ConsoleSpecs`] for fantasy console specifications
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

// Module declarations
pub mod api;
pub mod auth;
pub mod cart;
pub mod console;
pub mod formats;
pub mod local;
pub mod requests;

// Re-export all public items for convenience
pub use api::*;
pub use auth::*;
pub use cart::*;
pub use console::*;
pub use local::*;
pub use requests::*;
