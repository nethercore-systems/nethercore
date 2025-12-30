//! Nether-Tracker: Unified tracker module format for Nethercore
//!
//! This crate provides format-agnostic tracker types that both XM and IT modules
//! can be converted to. This allows the playback engine to handle multiple formats
//! with a single unified implementation.
//!
//! # Design
//!
//! The unified format normalizes differences between XM and IT:
//! - Effect semantics are standardized
//! - Channel counts are unified (64 max)
//! - Envelopes are normalized
//! - NNA (New Note Actions) are represented uniformly
//!
//! # Architecture
//!
//! ```text
//! ┌──────────────────┐     ┌──────────────────┐
//! │  IT File (.it)   │     │  XM File (.xm)   │
//! └────────┬─────────┘     └────────┬─────────┘
//!          │                        │
//!     parse_it()               parse_xm()
//!          │                        │
//!          ▼                        ▼
//!     ┌────────────────────────────────────────┐
//!     │         TrackerModule (unified)         │
//!     │  - patterns: Vec<TrackerPattern>       │
//!     │  - instruments: Vec<TrackerInstrument> │
//!     │  - samples: Vec<TrackerSample>         │
//!     │  - format_flags: FormatFlags           │
//!     └────────────────────────────────────────┘
//!                      │
//!                      ▼
//!              TrackerEngine
//!         (plays any TrackerModule)
//! ```

mod convert_it;
mod convert_xm;
mod effects;

pub use convert_it::from_it_module;
pub use convert_xm::from_xm_module;
pub use effects::TrackerEffect;

// =============================================================================
// Unified Tracker Module
// =============================================================================

/// Unified tracker module format (agnostic to XM/IT origin)
#[derive(Debug, Clone)]
pub struct TrackerModule {
    /// Module name
    pub name: String,
    /// Number of channels used (1-64)
    pub num_channels: u8,
    /// Initial speed (ticks per row)
    pub initial_speed: u8,
    /// Initial tempo (BPM)
    pub initial_tempo: u8,
    /// Global volume (0-128)
    pub global_volume: u8,
    /// Pattern order table
    pub order_table: Vec<u8>,
    /// Pattern data
    pub patterns: Vec<TrackerPattern>,
    /// Instrument definitions
    pub instruments: Vec<TrackerInstrument>,
    /// Sample definitions
    pub samples: Vec<TrackerSample>,
    /// Format-specific flags
    pub format: FormatFlags,
    /// Optional song message
    pub message: Option<String>,
    /// Restart position for song looping (XM feature, IT uses 0)
    pub restart_position: u16,
}

impl TrackerModule {
    /// Get the pattern at the given order position
    pub fn pattern_at_order(&self, order: u16) -> Option<&TrackerPattern> {
        let pattern_idx = *self.order_table.get(order as usize)? as usize;
        if pattern_idx >= 254 {
            return None; // Skip or end marker
        }
        self.patterns.get(pattern_idx)
    }

    /// Check if linear frequency slides are used (vs Amiga)
    pub fn uses_linear_slides(&self) -> bool {
        self.format.contains(FormatFlags::LINEAR_SLIDES)
    }

    /// Check if this module uses instruments (vs samples-only)
    pub fn uses_instruments(&self) -> bool {
        self.format.contains(FormatFlags::INSTRUMENTS)
    }

    /// Check if this module uses old effects mode (S3M compatibility)
    pub fn uses_old_effects(&self) -> bool {
        self.format.contains(FormatFlags::OLD_EFFECTS)
    }

    /// Check if this module links G memory with E/F for portamento
    pub fn uses_link_g_memory(&self) -> bool {
        self.format.contains(FormatFlags::LINK_G_MEMORY)
    }
}

/// Format-specific flags
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct FormatFlags(u16);

impl FormatFlags {
    /// Use linear frequency slides (vs Amiga slides)
    pub const LINEAR_SLIDES: Self = Self(0x0001);
    /// Use instruments (vs samples-only mode)
    pub const INSTRUMENTS: Self = Self(0x0002);
    /// Original format was IT (vs XM)
    pub const IS_IT_FORMAT: Self = Self(0x0004);
    /// Original format was XM
    pub const IS_XM_FORMAT: Self = Self(0x0008);
    /// Use old effects (S3M compatibility - affects vibrato/tremolo depth)
    pub const OLD_EFFECTS: Self = Self(0x0010);
    /// Link G memory with E/F for portamento
    pub const LINK_G_MEMORY: Self = Self(0x0020);

    pub const fn empty() -> Self {
        Self(0)
    }

    pub const fn from_bits(bits: u16) -> Self {
        Self(bits)
    }

    pub const fn bits(&self) -> u16 {
        self.0
    }

    pub const fn contains(&self, other: Self) -> bool {
        (self.0 & other.0) == other.0
    }

    pub const fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }
}

impl std::ops::BitOr for FormatFlags {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
}

// =============================================================================
// Pattern Data
// =============================================================================

/// Tracker pattern
#[derive(Debug, Clone)]
pub struct TrackerPattern {
    /// Number of rows (1-256)
    pub num_rows: u16,
    /// Note data: [row][channel]
    pub notes: Vec<Vec<TrackerNote>>,
}

impl TrackerPattern {
    /// Get note at specific row and channel
    pub fn get_note(&self, row: u16, channel: u8) -> Option<&TrackerNote> {
        self.notes.get(row as usize)?.get(channel as usize)
    }

    /// Create an empty pattern
    pub fn empty(num_rows: u16, num_channels: u8) -> Self {
        let mut notes = Vec::with_capacity(num_rows as usize);
        for _ in 0..num_rows {
            notes.push(vec![TrackerNote::default(); num_channels as usize]);
        }
        Self { num_rows, notes }
    }
}

/// Single note/command in a pattern
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct TrackerNote {
    /// Note value (0-119 = C-0 to B-9, 254 = cut, 255 = off)
    pub note: u8,
    /// Instrument number (1-based, 0 = none)
    pub instrument: u8,
    /// Volume (0-64)
    pub volume: u8,
    /// Unified effect
    pub effect: TrackerEffect,
}

impl TrackerNote {
    pub const NOTE_CUT: u8 = 254;
    pub const NOTE_OFF: u8 = 255;
    pub const NOTE_MAX: u8 = 119;

    /// Check if this is a note-cut
    pub fn is_note_cut(&self) -> bool {
        self.note == Self::NOTE_CUT
    }

    /// Check if this is a note-off
    pub fn is_note_off(&self) -> bool {
        self.note == Self::NOTE_OFF
    }

    /// Check if this has a valid note
    pub fn has_note(&self) -> bool {
        self.note <= Self::NOTE_MAX
    }

    /// Check if this has an instrument
    pub fn has_instrument(&self) -> bool {
        self.instrument > 0
    }

    /// Check if there's an effect
    pub fn has_effect(&self) -> bool {
        !matches!(self.effect, TrackerEffect::None)
    }
}

// =============================================================================
// Instrument Data
// =============================================================================

/// Unified tracker instrument
#[derive(Debug, Clone)]
pub struct TrackerInstrument {
    /// Instrument name
    pub name: String,
    /// New Note Action (IT feature)
    pub nna: NewNoteAction,
    /// Duplicate check type (IT feature)
    pub dct: DuplicateCheckType,
    /// Duplicate check action (IT feature)
    pub dca: DuplicateCheckAction,
    /// Fadeout speed (0-1024)
    pub fadeout: u16,
    /// Global volume (0-64)
    pub global_volume: u8,
    /// Default panning (0-64), None if not set
    pub default_pan: Option<u8>,
    /// Note→sample mapping table (120 entries)
    /// Each entry: (transposed_note, sample_number)
    pub note_sample_table: [(u8, u8); 120],
    /// Volume envelope
    pub volume_envelope: Option<TrackerEnvelope>,
    /// Panning envelope
    pub panning_envelope: Option<TrackerEnvelope>,
    /// Pitch envelope (IT only)
    pub pitch_envelope: Option<TrackerEnvelope>,
    /// Initial filter cutoff (0-127, IT only)
    pub filter_cutoff: Option<u8>,
    /// Initial filter resonance (0-127, IT only)
    pub filter_resonance: Option<u8>,
    /// Pitch-pan separation (-32 to +32, IT only)
    pub pitch_pan_separation: i8,
    /// Pitch-pan center note (0-119, IT only)
    pub pitch_pan_center: u8,

    // =========================================================================
    // Sample metadata (XM stores these per-instrument, IT per-sample)
    // =========================================================================

    /// Sample loop start position (in samples)
    pub sample_loop_start: u32,
    /// Sample loop end position (in samples)
    pub sample_loop_end: u32,
    /// Sample loop type
    pub sample_loop_type: LoopType,
    /// Sample finetune (-128 to 127)
    pub sample_finetune: i8,
    /// Sample relative note (semitones offset)
    pub sample_relative_note: i8,

    // =========================================================================
    // Auto-vibrato settings (XM feature, applied automatically to notes)
    // =========================================================================

    /// Auto-vibrato waveform (0=sine, 1=square, 2=ramp down, 3=ramp up)
    pub auto_vibrato_type: u8,
    /// Auto-vibrato sweep (frames to reach full depth)
    pub auto_vibrato_sweep: u8,
    /// Auto-vibrato depth
    pub auto_vibrato_depth: u8,
    /// Auto-vibrato rate (speed)
    pub auto_vibrato_rate: u8,
}

impl Default for TrackerInstrument {
    fn default() -> Self {
        let mut note_sample_table = [(0u8, 0u8); 120];
        for (i, entry) in note_sample_table.iter_mut().enumerate() {
            entry.0 = i as u8;
            entry.1 = 1; // Sample 1
        }

        Self {
            name: String::new(),
            nna: NewNoteAction::Cut,
            dct: DuplicateCheckType::Off,
            dca: DuplicateCheckAction::Cut,
            fadeout: 0,
            global_volume: 64,
            default_pan: None,
            note_sample_table,
            volume_envelope: None,
            panning_envelope: None,
            pitch_envelope: None,
            filter_cutoff: None,
            filter_resonance: None,
            pitch_pan_separation: 0,
            pitch_pan_center: 60, // C-5
            // Sample metadata
            sample_loop_start: 0,
            sample_loop_end: 0,
            sample_loop_type: LoopType::None,
            sample_finetune: 0,
            sample_relative_note: 0,
            // Auto-vibrato
            auto_vibrato_type: 0,
            auto_vibrato_sweep: 0,
            auto_vibrato_depth: 0,
            auto_vibrato_rate: 0,
        }
    }
}

impl TrackerInstrument {
    /// Get the sample number for a given note
    pub fn sample_for_note(&self, note: u8) -> Option<u8> {
        if note < 120 {
            let (_, sample) = self.note_sample_table[note as usize];
            if sample > 0 {
                Some(sample)
            } else {
                None
            }
        } else {
            None
        }
    }
}

/// New Note Action (IT feature)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u8)]
pub enum NewNoteAction {
    /// Cut the previous note immediately
    #[default]
    Cut = 0,
    /// Continue playing in background
    Continue = 1,
    /// Release the previous note
    NoteOff = 2,
    /// Fade out the previous note
    NoteFade = 3,
}

/// Duplicate Check Type (IT feature)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u8)]
pub enum DuplicateCheckType {
    /// No duplicate checking
    #[default]
    Off = 0,
    /// Check for same note
    Note = 1,
    /// Check for same sample
    Sample = 2,
    /// Check for same instrument
    Instrument = 3,
}

/// Duplicate Check Action (IT feature)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u8)]
pub enum DuplicateCheckAction {
    /// Cut the duplicate note
    #[default]
    Cut = 0,
    /// Release the duplicate note
    NoteOff = 1,
    /// Fade out the duplicate note
    NoteFade = 2,
}

/// Unified envelope
#[derive(Debug, Clone)]
pub struct TrackerEnvelope {
    /// Envelope points: (tick, value)
    /// Value range depends on envelope type:
    /// - Volume: 0-64
    /// - Panning: -32 to +32 (stored as i8)
    /// - Pitch: -32 to +32 half-semitones (stored as i8)
    pub points: Vec<(u16, i8)>,
    /// Loop begin point index
    pub loop_begin: u8,
    /// Loop end point index
    pub loop_end: u8,
    /// Sustain loop begin point index
    pub sustain_begin: u8,
    /// Sustain loop end point index
    pub sustain_end: u8,
    /// Envelope flags
    pub flags: EnvelopeFlags,
}

impl Default for TrackerEnvelope {
    fn default() -> Self {
        Self {
            points: vec![(0, 64), (100, 64)],
            loop_begin: 0,
            loop_end: 0,
            sustain_begin: 0,
            sustain_end: 0,
            flags: EnvelopeFlags::ENABLED,
        }
    }
}

impl TrackerEnvelope {
    /// Check if envelope is enabled
    pub fn is_enabled(&self) -> bool {
        self.flags.contains(EnvelopeFlags::ENABLED)
    }

    /// Check if envelope has loop
    pub fn has_loop(&self) -> bool {
        self.flags.contains(EnvelopeFlags::LOOP)
    }

    /// Check if envelope has sustain loop
    pub fn has_sustain(&self) -> bool {
        self.flags.contains(EnvelopeFlags::SUSTAIN_LOOP)
    }

    /// Check if this is a filter envelope (for pitch envelope type)
    pub fn is_filter(&self) -> bool {
        self.flags.contains(EnvelopeFlags::FILTER)
    }

    /// Get interpolated value at a given tick
    pub fn value_at(&self, tick: u16) -> i8 {
        if self.points.is_empty() {
            return 64;
        }

        // Find surrounding points
        for i in 0..self.points.len().saturating_sub(1) {
            let (x1, y1) = self.points[i];
            let (x2, y2) = self.points[i + 1];

            if tick >= x1 && tick < x2 {
                // Linear interpolation
                if x2 == x1 {
                    return y1;
                }
                let dx = (x2 - x1) as f32;
                let dy = y2 as f32 - y1 as f32;
                let t = (tick - x1) as f32 / dx;
                return (y1 as f32 + dy * t) as i8;
            }
        }

        // Past the last point
        self.points.last().map(|(_, y)| *y).unwrap_or(64)
    }
}

/// Envelope flags
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct EnvelopeFlags(u8);

impl EnvelopeFlags {
    pub const ENABLED: Self = Self(0x01);
    pub const LOOP: Self = Self(0x02);
    pub const SUSTAIN_LOOP: Self = Self(0x04);
    pub const CARRY: Self = Self(0x08);
    pub const FILTER: Self = Self(0x80);

    pub const fn empty() -> Self {
        Self(0)
    }

    pub const fn from_bits(bits: u8) -> Self {
        Self(bits)
    }

    pub const fn bits(&self) -> u8 {
        self.0
    }

    pub const fn contains(&self, other: Self) -> bool {
        (self.0 & other.0) == other.0
    }
}

impl std::ops::BitOr for EnvelopeFlags {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
}

// =============================================================================
// Sample Data
// =============================================================================

/// Unified tracker sample
#[derive(Debug, Clone)]
pub struct TrackerSample {
    /// Sample name
    pub name: String,
    /// Global volume (0-64)
    pub global_volume: u8,
    /// Default volume (0-64)
    pub default_volume: u8,
    /// Default panning (0-64), None if not set
    pub default_pan: Option<u8>,
    /// Sample length in samples
    pub length: u32,
    /// Loop begin
    pub loop_begin: u32,
    /// Loop end
    pub loop_end: u32,
    /// Loop type
    pub loop_type: LoopType,
    /// C5 speed (sample rate for middle C)
    pub c5_speed: u32,
    /// Sustain loop begin
    pub sustain_loop_begin: u32,
    /// Sustain loop end
    pub sustain_loop_end: u32,
    /// Sustain loop type
    pub sustain_loop_type: LoopType,
}

impl Default for TrackerSample {
    fn default() -> Self {
        Self {
            name: String::new(),
            global_volume: 64,
            default_volume: 64,
            default_pan: None,
            length: 0,
            loop_begin: 0,
            loop_end: 0,
            loop_type: LoopType::None,
            c5_speed: 8363,
            sustain_loop_begin: 0,
            sustain_loop_end: 0,
            sustain_loop_type: LoopType::None,
        }
    }
}

/// Sample loop type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LoopType {
    /// No loop
    #[default]
    None,
    /// Forward loop
    Forward,
    /// Ping-pong (bidirectional) loop
    PingPong,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tracker_note_methods() {
        let note = TrackerNote {
            note: 48, // C-4
            instrument: 1,
            volume: 32,
            effect: TrackerEffect::None,
        };
        assert!(note.has_note());
        assert!(note.has_instrument());
        assert!(!note.is_note_off());
        assert!(!note.is_note_cut());

        let note_off = TrackerNote {
            note: TrackerNote::NOTE_OFF,
            ..Default::default()
        };
        assert!(note_off.is_note_off());
        assert!(!note_off.has_note());
    }

    #[test]
    fn test_envelope_interpolation() {
        let env = TrackerEnvelope {
            points: vec![(0, 64), (10, 32), (20, 0)],
            flags: EnvelopeFlags::ENABLED,
            ..Default::default()
        };

        assert_eq!(env.value_at(0), 64);
        assert_eq!(env.value_at(5), 48); // Midpoint between 64 and 32
        assert_eq!(env.value_at(10), 32);
        assert_eq!(env.value_at(15), 16); // Midpoint between 32 and 0
        assert_eq!(env.value_at(20), 0);
        assert_eq!(env.value_at(30), 0); // Past end
    }

    #[test]
    fn test_pattern_empty() {
        let pattern = TrackerPattern::empty(64, 8);
        assert_eq!(pattern.num_rows, 64);
        assert_eq!(pattern.notes.len(), 64);
        assert_eq!(pattern.notes[0].len(), 8);
    }
}
