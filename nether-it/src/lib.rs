//! Nether-IT: IT (Impulse Tracker) module format parser and writer for Nethercore
//!
//! This crate provides a pure Rust parser and writer for the Impulse Tracker IT format,
//! designed for use with Nethercore's fantasy consoles. It supports all major IT features
//! including NNA (New Note Actions), pitch/filter envelopes, and IT215 sample compression.
//!
//! # Key Features
//!
//! - **Pure Rust**: No external C/C++ dependencies
//! - **Full IT support**: NNA, envelopes, filters, 64 channels
//! - **Sample separation**: Designed to work with samples loaded from ROM
//! - **IT215 decompression**: Handles compressed samples
//! - **Writer**: Generate IT files with embedded samples for preview
//!
//! # IT Format Overview
//!
//! IT files contain:
//! - Header with song metadata (name, speed, tempo, channels)
//! - Pattern order table
//! - Instrument definitions with NNA settings and envelopes
//! - Sample definitions with loop and vibrato settings
//! - Pattern data (compressed channel-based format)
//! - Sample data (optionally IT215 compressed)
//!
//! # Usage
//!
//! ```ignore
//! use nether_it::{parse_it, ItModule};
//!
//! let it_data = std::fs::read("song.it").unwrap();
//! let module = parse_it(&it_data).unwrap();
//!
//! println!("Song: {}", module.name);
//! println!("Channels: {}", module.num_channels);
//! println!("Patterns: {}", module.num_patterns);
//! println!("Instruments: {}", module.num_instruments);
//! ```
//!
//! # Format Reference
//!
//! - Impulse Tracker Technical Specification (ITTECH.TXT)
//! - <https://github.com/schismtracker/schismtracker/wiki/ITTECH.TXT>

mod compression;
mod error;
mod extract;
mod minimal;
mod module;
mod parser;
mod writer;

pub use compression::{
    decompress_it215_16bit, decompress_it215_16bit_with_size, decompress_it215_8bit,
    decompress_it215_8bit_with_size,
};
pub use error::ItError;
pub use minimal::{pack_it_minimal, pack_ncit, parse_it_minimal, parse_ncit, strip_it_samples};
pub use module::{
    note_from_name, DuplicateCheckAction, DuplicateCheckType, ItEnvelope, ItEnvelopeFlags,
    ItFlags, ItInstrument, ItModule, ItNote, ItPattern, ItSample, ItSampleFlags, NewNoteAction,
};
pub use parser::{
    get_instrument_names, get_sample_names, load_sample_data, parse_it, parse_sample, SampleData,
    SampleInfo,
};
pub use extract::{extract_samples, ExtractedSample};
pub use writer::ItWriter;

// =============================================================================
// Constants
// =============================================================================

/// IT format magic string "IMPM"
pub const IT_MAGIC: &[u8; 4] = b"IMPM";

/// Instrument magic string "IMPI"
pub const INSTRUMENT_MAGIC: &[u8; 4] = b"IMPI";

/// Sample magic string "IMPS"
pub const SAMPLE_MAGIC: &[u8; 4] = b"IMPS";

/// Minimum compatible version we support (2.00)
pub const MIN_COMPATIBLE_VERSION: u16 = 0x0200;

/// Maximum number of channels supported
pub const MAX_CHANNELS: u8 = 64;

/// Maximum number of patterns supported
pub const MAX_PATTERNS: u16 = 256;

/// Maximum pattern length (rows)
pub const MAX_PATTERN_ROWS: u16 = 200;

/// Maximum instruments in an IT file
pub const MAX_INSTRUMENTS: u16 = 99;

/// Maximum samples in an IT file
pub const MAX_SAMPLES: u16 = 99;

/// Maximum orders in an IT file
pub const MAX_ORDERS: u16 = 256;

/// Maximum envelope points
pub const MAX_ENVELOPE_POINTS: usize = 25;

// =============================================================================
// Note Constants
// =============================================================================

/// Note value for "note cut" (===)
pub const NOTE_CUT: u8 = 254;

/// Note value for "note off" (^^^)
pub const NOTE_OFF: u8 = 255;

/// Note value for "note fade"
pub const NOTE_FADE: u8 = 253;

/// Minimum valid note (C-0)
pub const NOTE_MIN: u8 = 0;

/// Maximum valid note (B-9)
pub const NOTE_MAX: u8 = 119;

// =============================================================================
// Order Constants
// =============================================================================

/// Order value for "skip" (+++)
pub const ORDER_SKIP: u8 = 254;

/// Order value for "end" (---)
pub const ORDER_END: u8 = 255;

// =============================================================================
// Effect Constants
// =============================================================================

/// IT effect commands for reference
pub mod effects {
    /// Axx - Set tempo (speed in ticks per row)
    pub const SET_SPEED: u8 = b'A' - b'@';
    /// Bxx - Jump to order
    pub const POSITION_JUMP: u8 = b'B' - b'@';
    /// Cxx - Break to row in next pattern
    pub const PATTERN_BREAK: u8 = b'C' - b'@';
    /// Dxy - Volume slide
    pub const VOLUME_SLIDE: u8 = b'D' - b'@';
    /// Exx - Pitch slide down
    pub const PORTA_DOWN: u8 = b'E' - b'@';
    /// Fxx - Pitch slide up
    pub const PORTA_UP: u8 = b'F' - b'@';
    /// Gxx - Tone portamento
    pub const TONE_PORTA: u8 = b'G' - b'@';
    /// Hxy - Vibrato
    pub const VIBRATO: u8 = b'H' - b'@';
    /// Ixy - Tremor
    pub const TREMOR: u8 = b'I' - b'@';
    /// Jxy - Arpeggio
    pub const ARPEGGIO: u8 = b'J' - b'@';
    /// Kxy - Vibrato + volume slide
    pub const VIBRATO_VOL_SLIDE: u8 = b'K' - b'@';
    /// Lxy - Tone portamento + volume slide
    pub const TONE_PORTA_VOL_SLIDE: u8 = b'L' - b'@';
    /// Mxx - Set channel volume
    pub const SET_CHANNEL_VOLUME: u8 = b'M' - b'@';
    /// Nxy - Channel volume slide
    pub const CHANNEL_VOLUME_SLIDE: u8 = b'N' - b'@';
    /// Oxx - Sample offset
    pub const SAMPLE_OFFSET: u8 = b'O' - b'@';
    /// Pxy - Panning slide
    pub const PANNING_SLIDE: u8 = b'P' - b'@';
    /// Qxy - Retrigger note
    pub const RETRIGGER: u8 = b'Q' - b'@';
    /// Rxy - Tremolo
    pub const TREMOLO: u8 = b'R' - b'@';
    /// Sxy - Extended effects
    pub const EXTENDED: u8 = b'S' - b'@';
    /// Txx - Set tempo (BPM)
    pub const SET_TEMPO: u8 = b'T' - b'@';
    /// Uxy - Fine vibrato
    pub const FINE_VIBRATO: u8 = b'U' - b'@';
    /// Vxx - Set global volume
    pub const SET_GLOBAL_VOLUME: u8 = b'V' - b'@';
    /// Wxy - Global volume slide
    pub const GLOBAL_VOLUME_SLIDE: u8 = b'W' - b'@';
    /// Xxx - Set panning
    pub const SET_PANNING: u8 = b'X' - b'@';
    /// Yxy - Panbrello
    pub const PANBRELLO: u8 = b'Y' - b'@';
    /// Zxx - MIDI macro / filter
    pub const MIDI_MACRO: u8 = b'Z' - b'@';
}

/// Extended effect sub-commands (Sxy where x is the sub-command)
pub mod extended_effects {
    /// S0x - Set filter (obsolete)
    pub const SET_FILTER: u8 = 0x0;
    /// S1x - Set glissando control
    pub const GLISSANDO: u8 = 0x1;
    /// S2x - Set finetune
    pub const SET_FINETUNE: u8 = 0x2;
    /// S3x - Set vibrato waveform
    pub const VIBRATO_WAVEFORM: u8 = 0x3;
    /// S4x - Set tremolo waveform
    pub const TREMOLO_WAVEFORM: u8 = 0x4;
    /// S5x - Set panbrello waveform
    pub const PANBRELLO_WAVEFORM: u8 = 0x5;
    /// S6x - Fine pattern delay (extra ticks)
    pub const FINE_PATTERN_DELAY: u8 = 0x6;
    /// S7x - Instrument control
    pub const INSTRUMENT_CONTROL: u8 = 0x7;
    /// S8x - Set panning (coarse)
    pub const SET_PANNING_COARSE: u8 = 0x8;
    /// S9x - Sound control
    pub const SOUND_CONTROL: u8 = 0x9;
    /// SAx - High sample offset
    pub const HIGH_SAMPLE_OFFSET: u8 = 0xA;
    /// SBx - Pattern loop
    pub const PATTERN_LOOP: u8 = 0xB;
    /// SCx - Note cut
    pub const NOTE_CUT: u8 = 0xC;
    /// SDx - Note delay
    pub const NOTE_DELAY: u8 = 0xD;
    /// SEx - Pattern delay (rows)
    pub const PATTERN_DELAY: u8 = 0xE;
    /// SFx - Set active macro
    pub const SET_ACTIVE_MACRO: u8 = 0xF;
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constants() {
        assert_eq!(IT_MAGIC.len(), 4);
        assert_eq!(INSTRUMENT_MAGIC.len(), 4);
        assert_eq!(SAMPLE_MAGIC.len(), 4);
        assert!(MAX_CHANNELS <= 64);
        assert!(MAX_PATTERNS <= 256);
    }

    #[test]
    fn test_note_constants() {
        assert_eq!(NOTE_CUT, 254);
        assert_eq!(NOTE_OFF, 255);
        assert_eq!(NOTE_MIN, 0);
        assert_eq!(NOTE_MAX, 119);
    }

    #[test]
    fn test_effect_constants() {
        // IT effects are 1-indexed (A=1, B=2, etc.)
        assert_eq!(effects::SET_SPEED, 1);
        assert_eq!(effects::POSITION_JUMP, 2);
        assert_eq!(effects::SET_PANNING, 24);
    }
}
