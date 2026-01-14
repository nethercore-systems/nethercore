//! Minimal XM format for Nethercore ROM packing
//!
//! This module implements a highly optimized binary format that strips all
//! unnecessary XM overhead while preserving playback data. This is designed
//! exclusively for ROM storage where samples come from a separate data pack.
//!
//! # Format Overview (NCXM - Nethercore XM)
//!
//! ```text
//! [Header: 16 bytes]
//! - num_channels: u8
//! - num_patterns: u16 (LE)
//! - num_instruments: u16 (LE)
//! - song_length: u16 (LE)
//! - restart_position: u16 (LE)
//! - default_speed: u16 (LE)
//! - default_bpm: u16 (LE)
//! - flags: u8 (bit 0 = linear_frequency_table)
//! - reserved: [u8; 2]
//!
//! [Pattern Order Table: song_length bytes]
//! - order_table[0..song_length]
//!
//! [Patterns: variable]
//! For each pattern:
//!   - num_rows: u16 (LE)
//!   - packed_size: u16 (LE)
//!   - packed_data: [u8; packed_size]
//!
//! [Instruments: variable]
//! For each instrument:
//!   - flags: u8 (bits 0-1: envelope flags, bits 2-7: num_samples)
//!   - [if has_vol_env] volume envelope data
//!   - [if has_pan_env] panning envelope data
//!   - vibrato_type: u8
//!   - vibrato_sweep: u8
//!   - vibrato_depth: u8
//!   - vibrato_rate: u8
//!   - volume_fadeout: u16 (LE)
//!   - [if num_samples > 0] sample metadata (15 bytes)
//! ```
//!
//! # Savings
//!
//! Compared to standard XM format:
//! - Removes magic header (17 bytes)
//! - Removes module name (20 bytes)
//! - Removes tracker name (20 bytes)
//! - Removes version (2 bytes)
//! - Removes 0x1A marker (1 byte)
//! - Removes pattern order padding (~200 bytes)
//! - Removes instrument names (22 bytes × N)
//! - Removes sample names (22 bytes × N × M)
//! - Removes sample headers (40 bytes × total samples)
//! - Removes all sample data (handled separately)
//!
//! **Total savings: ~1,500-3,000 bytes per typical XM file**

mod io;
mod packer;
mod parser;

#[cfg(test)]
mod tests;

// Re-export public API
pub use packer::pack_xm_minimal;
pub use parser::parse_xm_minimal;

/// Header size in bytes
pub(crate) const HEADER_SIZE: usize = 16;

/// Maximum envelope points we support (XM spec allows 12)
pub(crate) const MAX_ENVELOPE_POINTS: usize = 12;
