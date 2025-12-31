//! Nether-XM: XM (Extended Module) tracker format parser for Nethercore
//!
//! This crate provides a pure Rust parser for the FastTracker 2 XM format,
//! designed for use with Nethercore's fantasy consoles. It extracts pattern
//! data and instrument metadata while allowing sample data to be managed
//! separately by the caller.
//!
//! # Key Features
//!
//! - **Pure Rust**: No external C/C++ dependencies
//! - **No-std compatible**: Can work without standard library (feature-gated)
//! - **Sample separation**: Designed to work with samples loaded from ROM
//! - **Minimal allocations**: Efficient parsing with pre-sized vectors
//!
//! # XM Format Overview
//!
//! XM files contain:
//! - Header with song metadata (name, speed, BPM, channels)
//! - Pattern data (note/effect sequences)
//! - Instruments with envelopes and sample metadata
//! - Sample data (which we optionally skip for ROM-based samples)
//!
//! # Usage
//!
//! ```ignore
//! use nether_xm::{parse_xm, XmModule};
//!
//! let xm_data = std::fs::read("song.xm").unwrap();
//! let module = parse_xm(&xm_data).unwrap();
//!
//! println!("Song: {}", module.name);
//! println!("Channels: {}", module.num_channels);
//! println!("Patterns: {}", module.num_patterns);
//! println!("Instruments: {}", module.num_instruments);
//!
//! // Get instrument names for ROM sample mapping
//! for instr in &module.instruments {
//!     println!("  Instrument: {}", instr.name);
//! }
//! ```
//!
//! # Format Reference
//!
//! - FastTracker 2 XM format specification v0104
//! - <https://github.com/milkytracker/MilkyTracker/blob/master/resources/reference/xm-form.txt>

mod error;
mod extract;
mod minimal;
mod module;
mod parser;

pub use error::XmError;
pub use extract::{ExtractedSample, extract_samples};
pub use minimal::{pack_xm_minimal, parse_xm_minimal};
pub use module::{XmEnvelope, XmInstrument, XmModule, XmNote, XmPattern};
pub use parser::{get_instrument_names, parse_xm, strip_xm_samples};

// =============================================================================
// Constants
// =============================================================================

/// XM format magic string
pub const XM_MAGIC: &[u8; 17] = b"Extended Module: ";

/// XM format version we support
pub const XM_VERSION: u16 = 0x0104;

/// Maximum number of channels supported
pub const MAX_CHANNELS: u8 = 32;

/// Maximum number of patterns supported
pub const MAX_PATTERNS: u16 = 256;

/// Maximum pattern length (rows)
pub const MAX_PATTERN_ROWS: u16 = 256;

/// Maximum instruments in an XM file
pub const MAX_INSTRUMENTS: u16 = 128;

/// Maximum samples per instrument
pub const MAX_SAMPLES_PER_INSTRUMENT: u8 = 16;

// =============================================================================
// Note Constants
// =============================================================================

/// Note value for "note off"
pub const NOTE_OFF: u8 = 97;

/// Minimum valid note (C-0)
pub const NOTE_MIN: u8 = 1;

/// Maximum valid note (B-7)
pub const NOTE_MAX: u8 = 96;

// =============================================================================
// Effect Constants
// =============================================================================

/// XM effect commands for reference
pub mod effects {
    /// 0xy - Arpeggio
    pub const ARPEGGIO: u8 = 0x00;
    /// 1xx - Portamento up
    pub const PORTA_UP: u8 = 0x01;
    /// 2xx - Portamento down
    pub const PORTA_DOWN: u8 = 0x02;
    /// 3xx - Tone portamento
    pub const TONE_PORTA: u8 = 0x03;
    /// 4xy - Vibrato
    pub const VIBRATO: u8 = 0x04;
    /// 5xy - Tone portamento + volume slide
    pub const TONE_PORTA_VOL_SLIDE: u8 = 0x05;
    /// 6xy - Vibrato + volume slide
    pub const VIBRATO_VOL_SLIDE: u8 = 0x06;
    /// 7xy - Tremolo
    pub const TREMOLO: u8 = 0x07;
    /// 8xx - Set panning
    pub const SET_PANNING: u8 = 0x08;
    /// 9xx - Sample offset
    pub const SAMPLE_OFFSET: u8 = 0x09;
    /// Axy - Volume slide
    pub const VOLUME_SLIDE: u8 = 0x0A;
    /// Bxx - Position jump
    pub const POSITION_JUMP: u8 = 0x0B;
    /// Cxx - Set volume
    pub const SET_VOLUME: u8 = 0x0C;
    /// Dxx - Pattern break
    pub const PATTERN_BREAK: u8 = 0x0D;
    /// Exy - Extended effects
    pub const EXTENDED: u8 = 0x0E;
    /// Fxx - Set speed/tempo
    pub const SET_SPEED_TEMPO: u8 = 0x0F;
    /// Gxx - Set global volume
    pub const SET_GLOBAL_VOLUME: u8 = 0x10;
    /// Hxy - Global volume slide
    pub const GLOBAL_VOLUME_SLIDE: u8 = 0x11;
    /// Kxx - Key off
    pub const KEY_OFF: u8 = 0x14;
    /// Lxx - Set envelope position
    pub const SET_ENVELOPE_POS: u8 = 0x15;
    /// Pxy - Panning slide
    pub const PANNING_SLIDE: u8 = 0x19;
    /// Rxy - Multi retrig note
    pub const MULTI_RETRIG: u8 = 0x1B;
    /// Xxx - Extra fine portamento
    pub const EXTRA_FINE_PORTA: u8 = 0x21;
}

/// Extended effect sub-commands (Exy where x is the sub-command)
pub mod extended_effects {
    /// E1x - Fine portamento up
    pub const FINE_PORTA_UP: u8 = 0x1;
    /// E2x - Fine portamento down
    pub const FINE_PORTA_DOWN: u8 = 0x2;
    /// E3x - Glissando control
    pub const GLISSANDO: u8 = 0x3;
    /// E4x - Vibrato waveform
    pub const VIBRATO_WAVEFORM: u8 = 0x4;
    /// E5x - Set finetune
    pub const SET_FINETUNE: u8 = 0x5;
    /// E6x - Pattern loop
    pub const PATTERN_LOOP: u8 = 0x6;
    /// E7x - Tremolo waveform
    pub const TREMOLO_WAVEFORM: u8 = 0x7;
    /// E8x - Set panning (coarse)
    pub const SET_PANNING_COARSE: u8 = 0x8;
    /// E9x - Retrigger note
    pub const RETRIG: u8 = 0x9;
    /// EAx - Fine volume slide up
    pub const FINE_VOLUME_UP: u8 = 0xA;
    /// EBx - Fine volume slide down
    pub const FINE_VOLUME_DOWN: u8 = 0xB;
    /// ECx - Note cut
    pub const NOTE_CUT: u8 = 0xC;
    /// EDx - Note delay
    pub const NOTE_DELAY: u8 = 0xD;
    /// EEx - Pattern delay
    pub const PATTERN_DELAY: u8 = 0xE;
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constants() {
        assert_eq!(XM_MAGIC.len(), 17);
        assert_eq!(XM_VERSION, 0x0104);
        assert!(MAX_CHANNELS <= 32);
    }

    #[test]
    fn test_note_constants() {
        assert_eq!(NOTE_OFF, 97);
        assert_eq!(NOTE_MIN, 1);
        assert_eq!(NOTE_MAX, 96);
    }
}
