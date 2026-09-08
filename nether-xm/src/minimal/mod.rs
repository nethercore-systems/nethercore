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
//! - flags: u8 (bit 0 = linear_frequency_table; bit 1 = single-sample default-pan byte)
//! - byte 14: sample preamp when flag bit 4 is set; otherwise ignored (default 48)
//! - byte 15: reserved
//! - flag bit 3: FT2 pan law; absent = compatible linear pan law
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
//!   - [if num_samples > 0] sample metadata (12 bytes)
//!   - [if header bit 1 && num_samples == 1] sample default pan (1 byte, 0-255)
//!
//! The final sample-metadata byte was reserved in the original NCXM format.
//! It now stores `sample_default_volume + 1` for single-sample instruments;
//! `0` remains the backwards-compatible encoding for legacy full volume (64).
//! Values above 65 are invalid.
//!
//! Header bit 1 is an additive format flag: when set, each single-sample
//! instrument has one following full-range XM default-pan byte. Files without
//! the bit remain readable and retain no sample-pan override. Older runtimes
//! do not understand bit 1 and cannot safely read files that use it.
//! ```
//!
//! Header bit 2 adds a 96-byte local note/sample map followed by 13 bytes per
//! sample (LE u32 loop start/length; i8 finetune/relative note; u8 loop type,
//! volume, pan), after each nonempty instrument's legacy metadata. Samples are
//! flattened in instrument/local-slot order, including empty PCM slots.
//! Loop-type bit 7 marks interleaved stereo PCM; low bits retain the legacy
//! none/forward/ping-pong encoding, so existing mono artifacts are unchanged.
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

/// NCXM header flag: use the linear frequency table.
pub(crate) const FLAG_LINEAR_FREQUENCY: u8 = 0x01;
/// NCXM header flag: single-sample instruments carry a default-pan byte.
pub(crate) const FLAG_SAMPLE_DEFAULT_PAN: u8 = 0x02;
/// NCXM header flag: local sample maps and per-sample metadata follow each instrument.
pub(crate) const FLAG_SAMPLE_MAP: u8 = 0x04;
/// NCXM header flag: use the FT2-compatible pan and gain chain.
pub(crate) const FLAG_FT2_MIX: u8 = 0x08;
/// Header byte 14 carries explicit sample preamp (otherwise legacy default 48).
pub(crate) const FLAG_SAMPLE_PREAMP: u8 = 0x10;
/// Distinct legacy XM output scale; absent in old packed artifacts.
pub(crate) const FLAG_LEGACY_MIX: u8 = 0x20;
pub(crate) const FLAG_LEGACY_RETRIGGER: u8 = 0x40;
pub(crate) const SUPPORTED_FLAGS: u8 = FLAG_LINEAR_FREQUENCY
    | FLAG_SAMPLE_DEFAULT_PAN
    | FLAG_SAMPLE_MAP
    | FLAG_FT2_MIX
    | FLAG_SAMPLE_PREAMP
    | FLAG_LEGACY_MIX
    | FLAG_LEGACY_RETRIGGER;

/// Maximum envelope points we support (XM spec allows 12)
pub(crate) const MAX_ENVELOPE_POINTS: usize = 12;

/// Additive stereo marker in each existing sample loop-type byte.
pub(crate) const SAMPLE_STEREO: u8 = 0x80;
