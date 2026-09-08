//! New Note Action (NNA) processing for IT modules
//!
//! Handles the displacement of currently playing notes when new notes are triggered,
//! moving them to background channels based on NNA settings.

use super::super::channels::DCT_OFF;
use super::super::{MAX_TRACKER_CHANNELS, TrackerEngine};

impl TrackerEngine {
    /// Find an available background channel for NNA
    ///
    /// Background channels are slots beyond the module's channel count
    /// that can hold notes displaced by NNA actions.
    ///
    /// Returns the index of an available channel, or None if all are busy.
    pub(super) fn find_background_channel(&self, num_channels: usize) -> Option<usize> {
        // Look for an available slot in the background channel range
        for idx in num_channels..MAX_TRACKER_CHANNELS {
            if self.channels[idx].is_available_for_nna() {
                return Some(idx);
            }
        }

        // If no free slot, find the quietest background channel to steal
        let mut quietest_idx = None;
        let mut quietest_vol: f32 = f32::MAX;

        for idx in num_channels..MAX_TRACKER_CHANNELS {
            let ch = &self.channels[idx];
            if ch.note_on {
                // Calculate effective volume
                let vol = ch.volume * (ch.volume_fadeout as f32 / 65535.0);
                if vol < quietest_vol {
                    quietest_vol = vol;
                    quietest_idx = Some(idx);
                }
            }
        }

        quietest_idx
    }

    /// Process duplicate check for a new note trigger
    ///
    /// Checks all background channels for duplicates matching the DCT criteria
    /// and applies the DCA action to matching channels.
    pub(super) fn process_duplicate_check(
        &mut self,
        num_channels: usize,
        parent_channel: usize,
        dct: u8,
        dca: u8,
        note: u8,
        sample_handle: u32,
        sample_index: Option<usize>,
        instrument: u8,
    ) {
        if dct == DCT_OFF {
            return;
        }

        // Check background channels for duplicates
        for idx in num_channels..MAX_TRACKER_CHANNELS {
            if self.channels[idx].is_background
                && self.channels[idx].parent_channel as usize == parent_channel
                && (dct != super::super::channels::DCT_SAMPLE
                    || (sample_index.is_some() && self.channels[idx].sample_index == sample_index))
                && self.channels[idx].matches_duplicate_check(dct, note, sample_handle, instrument)
            {
                self.channels[idx].apply_dca(dca);
            }
        }
    }

    /// Handle NNA for a channel when a new note is triggered
    ///
    /// If the channel has a note playing and NNA is not Cut, moves the
    /// current note to a background channel before the new note takes over.
    ///
    /// The `nna` parameter belongs to the displaced voice, before installing
    /// the incoming note's instrument defaults.
    ///
    /// Returns true if NNA processing occurred (note was moved to background).
    pub(super) fn handle_nna(&mut self, ch_idx: usize, num_channels: usize, nna: u8) -> bool {
        use super::super::channels::NNA_CUT;

        let channel = &self.channels[ch_idx];

        // Only process if channel has an audible note
        if !channel.is_audible() {
            return false;
        }

        // NNA_CUT doesn't need background channel
        if nna == NNA_CUT {
            self.channels[ch_idx].note_on = false;
            // The caller already selected the incoming note volume; do not erase it.
            return false;
        }

        // Find a background channel to move the note to
        if let Some(bg_idx) = self.find_background_channel(num_channels) {
            // Copy current channel state to background
            let bg_channel = self.channels[ch_idx].copy_to_background(ch_idx as u8);

            // Apply NNA action to the background copy
            self.channels[bg_idx] = bg_channel;
            self.channels[bg_idx].apply_nna_action(nna);

            return true;
        }

        // No background channel available - fall back to cut
        self.channels[ch_idx].note_on = false;
        // Preserve incoming note volume when stealing is unavailable, too.
        false
    }
}
