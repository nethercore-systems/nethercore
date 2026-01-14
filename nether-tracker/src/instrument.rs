//! Instrument data structures and envelopes

use crate::sample::LoopType;

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
            if sample > 0 { Some(sample) } else { None }
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
