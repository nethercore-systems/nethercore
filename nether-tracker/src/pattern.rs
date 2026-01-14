//! Pattern and note data structures

use crate::effects::TrackerEffect;

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

    /// Check if this has a valid note (0 = no note)
    pub fn has_note(&self) -> bool {
        self.note > 0 && self.note <= Self::NOTE_MAX
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
