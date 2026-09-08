//! State synchronization and seeking for rollback support

use crate::audio::Sound;
use crate::state::tracker_flags;

use super::super::state::{CachedControlState, RowStateCache};
use super::super::{TrackerEngine, raw_tracker_handle};

impl TrackerEngine {
    /// Sync engine state to rollback state
    ///
    /// Called at the start of each render cycle to detect if rollback occurred.
    pub fn sync_to_state(&mut self, state: &crate::state::TrackerState, sounds: &[Option<Sound>]) {
        self.sync_to_state_at_rate(state, sounds, 44100);
    }

    pub fn sync_to_state_at_rate(&mut self, state: &crate::state::TrackerState, sounds: &[Option<Sound>], sample_rate: u32) {
        if state.handle == 0 || (state.flags & tracker_flags::PLAYING) == 0 {
            return;
        }

        let diverged = self.sync_handle != state.handle
            || self.current_order != state.order_position
            || self.current_row != state.row
            || self.current_tick != state.tick
            || self.tick_samples_rendered != state.tick_sample_pos
            || self.current_sample_clock != state.rendered_samples();
        if diverged {
            if let Some(index) = self
                .rollback_cache
                .iter()
                .rposition(|(saved, _)| bytemuck::bytes_of(saved) == bytemuck::bytes_of(state))
            {
                let snapshot = self.rollback_cache[index].1.clone();
                self.apply_snapshot(&snapshot);
                self.rollback_cache.truncate(index + 1);
            } else if !self.reconstruct_playback(state, sounds, sample_rate) {
                return;
            }
        }
        self.sync_handle = state.handle;
        self.current_tick = state.tick;
        self.tick_samples_rendered = state.tick_sample_pos;
        // ponytail: retain 64 exact snapshots; older states replay the sample clock.
        if self
            .rollback_cache
            .back()
            .is_none_or(|(saved, _)| bytemuck::bytes_of(saved) != bytemuck::bytes_of(state))
        {
            self.rollback_cache.push_back((*state, self.snapshot()));
            if self.rollback_cache.len() > 64 {
                self.rollback_cache.pop_front();
            }
        }
    }

    /// Cold recovery uses playback's actual clock, including slides, delays and loops.
    fn reconstruct_playback(&mut self, target: &crate::state::TrackerState, sounds: &[Option<Sound>], sample_rate: u32) -> bool {
        let raw = raw_tracker_handle(target.handle);
        let Some(loaded) = self.modules.get(raw as usize).and_then(|m| m.as_ref()) else { return false; };
        let origin = if target.rendered_samples() == 0 {
            (target.order_position, target.row)
        } else {
            ((target._reserved[3] >> 16) as u16, target._reserved[3] as u16)
        };
        if origin != (0, 0) && loaded.module.pattern_at_order(origin.0).is_none_or(|p| origin.1 >= p.num_rows) {
            return false;
        }
        let mut replay = crate::state::TrackerState {
            handle: target.handle, flags: target.flags & !tracker_flags::PAUSED,
            speed: loaded.module.initial_speed as u16, bpm: loaded.module.initial_tempo as u16,
            volume: target.volume, ..Default::default()
        };
        self.reset();
        self.seek_to_position(target.handle, 0, 0, sounds);
        let rate = if target.rendered_samples() == 0 || target._reserved[2] == 0 { sample_rate } else { target._reserved[2] };
        let mut visited = std::collections::BTreeSet::new();
        while (replay.order_position, replay.row, replay.tick, replay.tick_sample_pos) != (origin.0, origin.1, 0, 0) {
            if replay.flags & tracker_flags::PLAYING == 0 { return false; }
            if replay.tick == 0 && replay.tick_sample_pos == 0 {
                let loops: Vec<_> = self.channels.iter().map(|c| (c.pattern_loop_row, c.pattern_loop_count)).collect();
                if !visited.insert((replay.order_position, replay.row, self.pattern_delay_count, self.xm_next_pattern_row, self.xm_loop_owner, loops)) { return false; }
            }
            // Row tick zero can change tempo: cross one sample before sizing the rest.
            let count = if replay.tick_sample_pos == 0 { 1 } else {
                super::super::samples_per_tick(replay.bpm, rate).saturating_sub(replay.tick_sample_pos).max(1)
            };
            self.advance_positions(&mut replay, sounds, count, rate);
        }
        replay.reset_sample_clock();
        self.current_sample_clock = 0;
        let mut remaining = target.rendered_samples();
        if remaining == 0 {
            // Legacy/explicit positions can supply a tick without an elapsed clock.
            for _ in 0..target.tick {
                self.advance_positions(&mut replay, sounds, 1, rate);
                let rest = super::super::samples_per_tick(replay.bpm, rate).saturating_sub(replay.tick_sample_pos);
                self.advance_positions(&mut replay, sounds, rest, rate);
            }
            remaining = u64::from(target.tick_sample_pos);
        }
        // ponytail: cold recovery replays from the origin; recent rollback uses exact snapshots.
        while remaining != 0 && replay.flags & tracker_flags::PLAYING != 0 {
            let count = remaining.min(u32::MAX as u64) as u32;
            self.advance_positions(&mut replay, sounds, count, rate);
            remaining -= u64::from(count);
        }
        self.current_sample_clock = target.rendered_samples();
        remaining == 0
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
        // Reject unreachable coordinates before replay: wrapping orders would
        // otherwise spin forever for an order/row beyond the module bounds.
        if self
            .modules
            .get(raw_handle as usize)
            .and_then(|m| m.as_ref())
            .is_none_or(|loaded| {
                (target_order != 0 || target_row != 0)
                    && loaded.module.pattern_at_order(target_order)
                        .is_none_or(|pattern| target_row >= pattern.num_rows)
            })
        {
            return;
        }

        // Try to find cached state
        if let Some((cached_pos, cached_state)) =
            self.row_cache
                .find_nearest(handle, target_order, target_row)
        {
            // Restore from cache
            self.channels = *cached_state.channels.clone();
            self.channel_mutes = cached_state.control.channel_mutes;
            self.global_volume = cached_state.control.global_volume;
            self.pattern_delay = cached_state.control.pattern_delay;
            self.pattern_delay_count = cached_state.control.pattern_delay_count;
            self.fine_pattern_delay = cached_state.control.fine_pattern_delay;
            self.xm_next_pattern_row = cached_state.control.xm_next_pattern_row;
            self.xm_loop_owner = cached_state.control.xm_loop_owner;
            self.last_global_vol_slide = cached_state.control.last_global_vol_slide;
            self.is_it_format = cached_state.control.is_it_format;
            self.old_effects_mode = cached_state.control.old_effects_mode;
            self.link_g_memory = cached_state.control.link_g_memory;
            self.tempo_slide = cached_state.control.tempo_slide;
            self.current_order = cached_pos.0;
            self.current_row = cached_pos.1;
            self.resolved_row_notes.clear();
            self.resolved_row_key = None;
        } else {
            // Full reset and replay from start
            self.reset();
            let module = &self.modules[raw_handle as usize].as_ref().unwrap().module;
            let it = module
                .format
                .contains(nether_tracker::FormatFlags::IS_IT_FORMAT);
            self.global_volume = module.global_volume as f32 / if it { 128.0 } else { 64.0 };
            for (index, channel) in self
                .channels
                .iter_mut()
                .take(module.num_channels as usize)
                .enumerate()
            {
                let pan = module.channel_pan[index];
                channel.panning = if pan & 127 <= 64 {
                    (pan & 127) as f32 / 32.0 - 1.0
                } else {
                    0.0
                };
                channel.surround = pan & 127 == 100;
                self.channel_mutes[index] = pan & 128 != 0;
                channel.channel_volume = module.channel_vol[index].min(64);
            }
        }

        // Fast-forward to target position by processing rows
        while self.current_order < target_order
            || (self.current_order == target_order && self.current_row < target_row)
        {
            // Use playback's IT skip/end-marker semantics before reading a pattern.
            let mut position = crate::state::TrackerState {
                handle,
                order_position: self.current_order,
                row: self.current_row,
                flags: tracker_flags::PLAYING,
                ..Default::default()
            };
            if !self.normalize_order(&mut position) {
                return;
            }
            self.current_order = position.order_position;
            self.current_row = position.row;
            if (self.current_order, self.current_row) >= (target_order, target_row) {
                break;
            }
            // Cache the state BEFORE this row; replay starts at its tick zero.
            // A post-row snapshot would apply fine effects twice on cache hits.
            if RowStateCache::should_cache(self.current_row) {
                self.row_cache.store(
                    handle,
                    self.current_order,
                    self.current_row,
                    &self.channels,
                    CachedControlState {
                        channel_mutes: self.channel_mutes,
                        global_volume: self.global_volume,
                        pattern_delay: self.pattern_delay,
                        pattern_delay_count: self.pattern_delay_count,
                        fine_pattern_delay: self.fine_pattern_delay,
                        xm_next_pattern_row: self.xm_next_pattern_row,
                        xm_loop_owner: self.xm_loop_owner,
                        last_global_vol_slide: self.last_global_vol_slide,
                        is_it_format: self.is_it_format,
                        old_effects_mode: self.old_effects_mode,
                        link_g_memory: self.link_g_memory,
                        tempo_slide: self.tempo_slide,
                    },
                );
            }

            // Process the row (tick 0 only for seeking).
            self.process_row_tick0_internal(handle, sounds);

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
