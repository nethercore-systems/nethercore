//! XM file parser
//!
//! This module provides functionality for parsing and manipulating XM (Extended Module) files.
//! It consists of:
//!
//! - `read` - Parsing XM files into structured data
//! - `write` - Writing/rebuilding XM files (primarily for stripping sample data)
//! - `tests` - Comprehensive test suite

mod read;
mod write;

#[cfg(test)]
mod tests;

// Re-export public API
pub use read::{get_instrument_names, parse_xm};
pub use write::strip_xm_samples;

// Re-export internal helpers for use within crate
pub(crate) use write::pack_pattern_data;
