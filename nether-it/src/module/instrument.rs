//! IT instrument structures and enums

use super::ItEnvelope;

/// New Note Action - what happens when a new note is played
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u8)]
pub enum NewNoteAction {
    /// Cut the previous note immediately
    #[default]
    Cut = 0,
    /// Continue playing the previous note in background
    Continue = 1,
    /// Release the previous note (key-off)
    NoteOff = 2,
    /// Fade out the previous note
    NoteFade = 3,
}

impl NewNoteAction {
    pub fn from_u8(val: u8) -> Self {
        match val {
            0 => Self::Cut,
            1 => Self::Continue,
            2 => Self::NoteOff,
            3 => Self::NoteFade,
            _ => Self::Cut,
        }
    }
}

/// Duplicate Check Type - when to check for duplicate notes
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

impl DuplicateCheckType {
    pub fn from_u8(val: u8) -> Self {
        match val {
            0 => Self::Off,
            1 => Self::Note,
            2 => Self::Sample,
            3 => Self::Instrument,
            _ => Self::Off,
        }
    }
}

/// Duplicate Check Action - what to do with duplicate notes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u8)]
pub enum DuplicateCheckAction {
    /// Cut the duplicate note
    #[default]
    Cut = 0,
    /// Release the duplicate note (key-off)
    NoteOff = 1,
    /// Fade out the duplicate note
    NoteFade = 2,
}

impl DuplicateCheckAction {
    pub fn from_u8(val: u8) -> Self {
        match val {
            0 => Self::Cut,
            1 => Self::NoteOff,
            2 => Self::NoteFade,
            _ => Self::Cut,
        }
    }
}

/// IT instrument metadata
#[derive(Debug, Clone)]
pub struct ItInstrument {
    /// Instrument name (max 26 chars)
    pub name: String,
    /// DOS filename (max 12 chars)
    pub filename: String,
    /// New Note Action
    pub nna: NewNoteAction,
    /// Duplicate Check Type
    pub dct: DuplicateCheckType,
    /// Duplicate Check Action
    pub dca: DuplicateCheckAction,
    /// Fadeout value (0-128, multiply by 8 for internal 0-1024)
    pub fadeout: u16,
    /// Pitch-Pan Separation (-32 to +32)
    pub pitch_pan_separation: i8,
    /// Pitch-Pan Center note (0-119)
    pub pitch_pan_center: u8,
    /// Global volume (0-128)
    pub global_volume: u8,
    /// Default panning (0-64), None if not enabled
    pub default_pan: Option<u8>,
    /// Random volume variation (0-100%)
    pub random_volume: u8,
    /// Random panning variation (0-64)
    pub random_pan: u8,
    /// Note-Sample-Keyboard table (120 entries)
    /// Each entry: (note_to_play, sample_number)
    pub note_sample_table: [(u8, u8); 120],
    /// Volume envelope
    pub volume_envelope: Option<ItEnvelope>,
    /// Panning envelope
    pub panning_envelope: Option<ItEnvelope>,
    /// Pitch/Filter envelope
    pub pitch_envelope: Option<ItEnvelope>,
    /// Initial filter cutoff (0-127), None if not set
    pub filter_cutoff: Option<u8>,
    /// Initial filter resonance (0-127), None if not set
    pub filter_resonance: Option<u8>,
    /// MIDI channel (0-16, 0 = disabled)
    pub midi_channel: u8,
    /// MIDI program (0-127)
    pub midi_program: u8,
    /// MIDI bank (0-16383)
    pub midi_bank: u16,
}

impl Default for ItInstrument {
    fn default() -> Self {
        // Default note-sample table: each note maps to itself with sample 1
        let mut note_sample_table = [(0u8, 0u8); 120];
        for (i, entry) in note_sample_table.iter_mut().enumerate() {
            entry.0 = i as u8; // Note plays as itself
            entry.1 = 1; // Use sample 1
        }

        Self {
            name: String::new(),
            filename: String::new(),
            nna: NewNoteAction::Cut,
            dct: DuplicateCheckType::Off,
            dca: DuplicateCheckAction::Cut,
            fadeout: 0,
            pitch_pan_separation: 0,
            pitch_pan_center: 60, // C-5
            global_volume: 128,
            default_pan: None,
            random_volume: 0,
            random_pan: 0,
            note_sample_table,
            volume_envelope: None,
            panning_envelope: None,
            pitch_envelope: None,
            filter_cutoff: None,
            filter_resonance: None,
            midi_channel: 0,
            midi_program: 0,
            midi_bank: 0,
        }
    }
}

impl ItInstrument {
    /// Get the sample number for a given note
    pub fn sample_for_note(&self, note: u8) -> Option<u8> {
        if note < 120 {
            let (_, sample) = self.note_sample_table[note as usize];
            if sample > 0 { Some(sample) } else { None }
        } else {
            None
        }
    }

    /// Get the transposed note for a given input note
    pub fn note_for_input(&self, note: u8) -> u8 {
        if note < 120 {
            self.note_sample_table[note as usize].0
        } else {
            note
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nna_from_u8() {
        assert_eq!(NewNoteAction::from_u8(0), NewNoteAction::Cut);
        assert_eq!(NewNoteAction::from_u8(1), NewNoteAction::Continue);
        assert_eq!(NewNoteAction::from_u8(2), NewNoteAction::NoteOff);
        assert_eq!(NewNoteAction::from_u8(3), NewNoteAction::NoteFade);
        assert_eq!(NewNoteAction::from_u8(99), NewNoteAction::Cut); // Invalid defaults to Cut
    }
}
