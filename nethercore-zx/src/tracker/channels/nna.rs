//! NNA (New Note Action) system implementation
//!
//! Handles IT module polyphony through background channels and duplicate checking.

use super::TrackerChannel;

// =============================================================================
// NNA Constants
// =============================================================================

/// New Note Action: Cut the previous note immediately
pub const NNA_CUT: u8 = 0;
/// New Note Action: Continue playing in background
pub const NNA_CONTINUE: u8 = 1;
/// New Note Action: Trigger note-off (release envelopes)
pub const NNA_NOTE_OFF: u8 = 2;
/// New Note Action: Start fadeout
pub const NNA_NOTE_FADE: u8 = 3;

/// Duplicate Check Type: No checking
pub const DCT_OFF: u8 = 0;
/// Duplicate Check Type: Check for same note
pub const DCT_NOTE: u8 = 1;
/// Duplicate Check Type: Check for same sample
pub const DCT_SAMPLE: u8 = 2;
/// Duplicate Check Type: Check for same instrument
pub const DCT_INSTRUMENT: u8 = 3;

/// Duplicate Check Action: Cut the duplicate
pub const DCA_CUT: u8 = 0;
/// Duplicate Check Action: Note-off the duplicate
pub const DCA_NOTE_OFF: u8 = 1;
/// Duplicate Check Action: Fade out the duplicate
pub const DCA_NOTE_FADE: u8 = 2;

impl TrackerChannel {
    /// Check if this channel is currently producing audible output
    ///
    /// A channel is audible if it has a note playing with non-zero volume
    /// and hasn't fully faded out.
    pub fn is_audible(&self) -> bool {
        self.note_on && self.sample_handle != 0 && self.volume_fadeout > 0
    }

    /// Check if this channel can be used as a background channel
    ///
    /// A background channel slot is available if it's either:
    /// - Not playing anything
    /// - Has fully faded out
    /// - Is a background channel with very low volume
    pub fn is_available_for_nna(&self) -> bool {
        !self.note_on || self.sample_handle == 0 || self.volume_fadeout == 0
    }

    /// Copy state to a background channel for NNA continuation
    ///
    /// This preserves all the playback state (position, envelopes, volume, etc.)
    /// so the note can continue playing in the background.
    pub fn copy_to_background(&self, parent_idx: u8) -> TrackerChannel {
        let mut bg = self.clone();
        bg.is_background = true;
        bg.parent_channel = parent_idx;
        bg
    }

    /// Apply NNA action to this channel
    ///
    /// Called when this channel needs to be "displaced" by a new note.
    /// Returns true if the channel should be moved to background,
    /// false if it should just be cut.
    pub fn apply_nna_action(&mut self, action: u8) -> bool {
        match action {
            NNA_CUT => {
                // Immediate cut - no background needed
                self.note_on = false;
                self.volume = 0.0;
                false
            }
            NNA_CONTINUE => {
                // Continue as-is in background
                true
            }
            NNA_NOTE_OFF => {
                // Trigger key-off, then move to background
                self.key_off = true;
                true
            }
            NNA_NOTE_FADE => {
                // Start fadeout (force key_off for envelope), then move to background
                self.key_off = true;
                // If no fadeout rate set, use a default fast fade
                if self.instrument_fadeout_rate == 0 {
                    self.instrument_fadeout_rate = 1024; // Fast fade
                }
                true
            }
            _ => false,
        }
    }

    /// Apply duplicate check action to this channel
    ///
    /// Called when this background channel matches a duplicate check.
    pub fn apply_dca(&mut self, action: u8) {
        match action {
            DCA_CUT => {
                self.note_on = false;
                self.volume = 0.0;
            }
            DCA_NOTE_OFF => {
                self.key_off = true;
            }
            DCA_NOTE_FADE => {
                self.key_off = true;
                if self.instrument_fadeout_rate == 0 {
                    self.instrument_fadeout_rate = 1024;
                }
            }
            _ => {}
        }
    }

    /// Check if this channel matches a duplicate check condition
    ///
    /// - DCT_NOTE: Same note value
    /// - DCT_SAMPLE: Same sample handle
    /// - DCT_INSTRUMENT: Same instrument number
    pub fn matches_duplicate_check(
        &self,
        dct: u8,
        note: u8,
        sample_handle: u32,
        instrument: u8,
    ) -> bool {
        if !self.is_audible() {
            return false;
        }

        match dct {
            DCT_OFF => false,
            DCT_NOTE => self.current_note == note,
            DCT_SAMPLE => self.sample_handle == sample_handle,
            DCT_INSTRUMENT => self.instrument == instrument,
            _ => false,
        }
    }
}
