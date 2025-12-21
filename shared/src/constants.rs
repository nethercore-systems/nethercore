//! Centralized constants for the Nethercore platform.
//!
//! This module provides single-source-of-truth constants used across
//! both the platform backend and console clients.

/// Bearer token prefix for Authorization headers.
///
/// Used when constructing or parsing Authorization headers:
/// ```ignore
/// let header = format!("{}{}", BEARER_PREFIX, token);
/// ```
pub const BEARER_PREFIX: &str = "Bearer ";

/// Local development API base URL.
///
/// Used for development/testing when running the backend locally.
pub const LOCAL_DEV_BASE_URL: &str = "http://localhost:3000";

/// Local development frontend URL.
///
/// Used for CORS configuration in development mode.
pub const LOCAL_FRONTEND_URL: &str = "http://localhost:4321";

/// Production API/website URL.
///
/// Used for CORS configuration and production URLs.
pub const PRODUCTION_URL: &str = "https://nethercore.systems";
