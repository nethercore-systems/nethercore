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

// Re-export public items explicitly for clarity
pub use api::{Author, Game, GamesResponse, RomUrlResponse, VersionResponse};
pub use auth::{ApiError, AuthResponse, LinkCodeResponse, User, error_codes};
pub use console::{
    CONSOLES, ConsoleSpecs, ConsoleType, NETHERCORE_CHROMA_MEMORY_LIMIT,
    NETHERCORE_CHROMA_RESOLUTION, NETHERCORE_CHROMA_VRAM_LIMIT, NETHERCORE_ZX_RAM_LIMIT,
    NETHERCORE_ZX_RESOLUTION, NETHERCORE_ZX_ROM_LIMIT, NETHERCORE_ZX_TICK_RATES,
    NETHERCORE_ZX_VRAM_LIMIT, ParseConsoleTypeError, TickRate, get_console_specs,
    nethercore_chroma_specs, nethercore_zx_specs,
};
pub use constants::{BEARER_PREFIX, LOCAL_DEV_BASE_URL, LOCAL_FRONTEND_URL, PRODUCTION_URL};
pub use fs::{MAX_PNG_BYTES, MAX_ROM_BYTES, MAX_WASM_BYTES, read_file_with_limit};
pub use ids::is_safe_game_id;
pub use local::LocalGameManifest;
pub use math::BoneMatrix3x4;
pub use netplay::{NetplayMetadata, NetplayMismatch};
pub use requests::{
    CreateGameRequest, CreateGameResponse, LoginRequest, RegisterRequest, SuccessResponse,
    UpdateGameRequest, UploadUrls,
};
pub use rom_format::{
    ROM_FORMATS, RomFormat, ZX_ROM_FORMAT, get_console_type_by_extension,
    get_rom_format_by_console, get_rom_format_by_console_type, get_rom_format_by_extension,
};
pub use screenshot::{
    SCREENSHOT_SIGNATURE_KEYWORD, ScreenshotPayload, ScreenshotSignError, SignedScreenshot,
    compute_pixel_hash, sign_screenshot, verify_screenshot,
};
