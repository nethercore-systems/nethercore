//! Eurobeat XM generation - "Nether Fire"
//!
//! 155 BPM, D minor, 8 patterns, 7 instruments

mod constants;
mod generators;
mod patterns;

// Re-export public API
pub use generators::{generate_eurobeat_xm, generate_eurobeat_xm_embedded};
