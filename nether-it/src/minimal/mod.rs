//! NCIT: Nethercore IT minimal format for ROM packing
//!
//! This module implements a highly optimized binary format that strips all
//! unnecessary IT overhead while preserving playback data. This is designed
//! exclusively for ROM storage where samples come from a separate data pack.
//!
//! # Format Overview (NCIT - Nethercore IT)
//!
//! ## Header (24 bytes)
//! ```text
//! 0x00  u8      num_channels (1-64)
//! 0x01  u16 LE  num_orders
//! 0x03  u16 LE  num_instruments
//! 0x05  u16 LE  num_samples
//! 0x07  u16 LE  num_patterns
//! 0x09  u8      initial_speed
//! 0x0A  u8      initial_tempo
//! 0x0B  u8      global_volume (0-128)
//! 0x0C  u8      mix_volume (0-128)
//! 0x0D  u16 LE  flags (IT flags)
//! 0x0F  u8      panning_separation (0-128)
//! 0x10  8 bytes reserved
//! ```
//!
//! ## Savings vs Standard IT
//!
//! - Removes 192-byte IT header (replaced with 24-byte NCIT header)
//! - Removes 547-byte instrument headers (replaced with ~15-50 bytes each)
//! - Removes 80-byte sample headers (replaced with ~7-27 bytes each)
//! - Note-sample tables compressed (240 bytes → 2-50 bytes typically)
//! - Pattern data unchanged (IT's packing is already efficient)
//!
//! **Total savings: ~75-80% reduction in metadata overhead**

mod pack;
mod parse;
mod legacy;

#[cfg(test)]
mod tests;

use std::io::{Cursor, Read, Write};

use crate::IT_MAGIC;
use crate::error::ItError;
use crate::module::ItModule;
use crate::parser::parse_it;

// Re-export public API
pub use pack::pack_ncit;
pub use parse::parse_ncit;
pub use legacy::{pack_it_minimal, strip_it_samples};

// =============================================================================
// Constants
// =============================================================================

/// NCIT header size in bytes
pub(crate) const NCIT_HEADER_SIZE: usize = 24;

/// Maximum envelope points we support
pub(crate) const MAX_ENVELOPE_POINTS: usize = 25;

// =============================================================================
// Instrument Flags (for NCIT format)
// =============================================================================

pub(crate) const INSTR_HAS_VOL_ENV: u8 = 0x01;
pub(crate) const INSTR_HAS_PAN_ENV: u8 = 0x02;
pub(crate) const INSTR_HAS_PITCH_ENV: u8 = 0x04;
pub(crate) const INSTR_HAS_FILTER: u8 = 0x08;
pub(crate) const INSTR_HAS_DEFAULT_PAN: u8 = 0x10;

// =============================================================================
// Sample Flags (for NCIT format)
// =============================================================================

pub(crate) const SAMPLE_HAS_LOOP: u8 = 0x01;
pub(crate) const SAMPLE_PINGPONG_LOOP: u8 = 0x02;
pub(crate) const SAMPLE_HAS_SUSTAIN: u8 = 0x04;
pub(crate) const SAMPLE_PINGPONG_SUSTAIN: u8 = 0x08;
pub(crate) const SAMPLE_HAS_PAN: u8 = 0x10;
pub(crate) const SAMPLE_HAS_VIBRATO: u8 = 0x20;

// =============================================================================
// Note-Sample Table Types
// =============================================================================

pub(crate) const TABLE_UNIFORM: u8 = 0;
pub(crate) const TABLE_SPARSE: u8 = 1;
pub(crate) const TABLE_FULL: u8 = 2;

// =============================================================================
// Auto-detection Wrapper
// =============================================================================

/// Parse IT file with auto-detection (NCIT or standard IT)
///
/// This function auto-detects the format by checking for the IT magic bytes.
/// If present, it parses as standard IT format; otherwise, it parses as NCIT.
///
/// # Arguments
/// * `data` - Raw binary data (either IT or NCIT format)
///
/// # Returns
/// * `Ok(ItModule)` - Parsed module
/// * `Err(ItError)` - Parse error
pub fn parse_it_minimal(data: &[u8]) -> Result<ItModule, ItError> {
    if data.len() >= 4 && &data[0..4] == IT_MAGIC {
        // Standard IT format - use full parser
        parse_it(data)
    } else {
        // Assume NCIT minimal format
        parse_ncit(data)
    }
}

// =============================================================================
// Helper Functions
// =============================================================================

pub(crate) fn write_u16<W: Write>(output: &mut W, val: u16) {
    output.write_all(&val.to_le_bytes()).unwrap();
}

pub(crate) fn write_u32<W: Write>(output: &mut W, val: u32) {
    output.write_all(&val.to_le_bytes()).unwrap();
}

pub(crate) fn read_u8(cursor: &mut Cursor<&[u8]>) -> Result<u8, ItError> {
    let mut buf = [0u8; 1];
    cursor
        .read_exact(&mut buf)
        .map_err(|_| ItError::UnexpectedEof)?;
    Ok(buf[0])
}

pub(crate) fn read_u16(cursor: &mut Cursor<&[u8]>) -> Result<u16, ItError> {
    let mut buf = [0u8; 2];
    cursor
        .read_exact(&mut buf)
        .map_err(|_| ItError::UnexpectedEof)?;
    Ok(u16::from_le_bytes(buf))
}

pub(crate) fn read_u32(cursor: &mut Cursor<&[u8]>) -> Result<u32, ItError> {
    let mut buf = [0u8; 4];
    cursor
        .read_exact(&mut buf)
        .map_err(|_| ItError::UnexpectedEof)?;
    Ok(u32::from_le_bytes(buf))
}
