//! State synchronization and seeking for rollback support

use crate::audio::Sound;
use crate::state::tracker_flags;

use super::super::state::RowStateCache;
use super::super::{TrackerEngine, raw_tracker_handle};

impl TrackerEngine {
    /// Sync engine state to rollback state
    ///
    /// Called at the start of each render cycle to detect if rollback occurred.
    pub fn sync_to_state(&mut self, state: &crate::state::TrackerState, sounds: &[Option<Sound>]) {
        if state.handle == 0 || (state.flags & tracker_flags::PLAYING) == 0 {
            return;
        }

        // Check if position diverged (rollback occurred)
        if self.current_order != state.order_position || self.current_row != state.row {
            self.seek_to_position(state.handle, state.order_position, state.row, sounds);
        }

        // Sync tick position
        self.current_tick = state.tick;
        self.tick_samples_rendered = state.tick_sample_pos;
    }

    /// Seek to a specific position, using cache when possible
    pub(super) fn seek_to_position(
        &mut self,
        handle: u32,
        target_order: u16,
        target_row: u16,
        sounds: &[Option<Sound>],
    ) {
        let raw_handle = raw_tracker_handle(handle);
        // Validate handle exists
        if self
            .modules
            .get(raw_handle as usize)
            .and_then(|m| m.as_ref())
            .is_none()
        {
            return;
        }

        // Try to find cached state
        if let Some((cached_pos, cached_state)) =
            self.row_cache.find_nearest(target_order, target_row)
        {
            // Restore from cache
            self.channels = *cached_state.channels.clone();
            self.global_volume = cached_state.global_volume;
            self.current_order = cached_pos.0;
            self.current_row = cached_pos.1;
        } else {
            // Full reset and replay from start
            self.reset();
        }

        // Fast-forward to target position by processing rows
        while self.current_order < target_order
            || (self.current_order == target_order && self.current_row < target_row)
        {
            // Process the row (tick 0 only for seeking)
            self.process_row_tick0_internal(handle, sounds);

            // Cache at intervals
            if RowStateCache::should_cache(self.current_row) {
                self.row_cache.store(
                    self.current_order,
                    self.current_row,
                    &self.channels,
                    self.global_volume,
                );
            }

            // Advance to next row - inline the logic to avoid borrow issues
            self.current_row += 1;

            // Get current pattern length and restart position
            let (num_rows, song_length, restart_position) = {
                let loaded = match self
                    .modules
                    .get(raw_handle as usize)
                    .and_then(|m| m.as_ref())
                {
                    Some(m) => m,
                    None => return,
                };
                let num_rows = loaded
                    .module
                    .pattern_at_order(self.current_order)
                    .map(|p| p.num_rows)
                    .unwrap_or(0);
                (
                    num_rows,
                    loaded.module.order_table.len() as u16,
                    loaded.module.restart_position,
                )
            };

            if num_rows == 0 {
                // No pattern at this order - end of song
                self.current_order = restart_position;
                self.current_row = 0;
            } else if self.current_row >= num_rows {
                // End of pattern
                self.current_order += 1;
                self.current_row = 0;

                if self.current_order >= song_length {
                    self.current_order = restart_position;
                }
            }
        }

        self.current_tick = 0;
    }
}
