//! Public rendering API methods

use super::super::utils::samples_per_tick;
use super::super::{TrackerEngine, raw_tracker_handle};
use super::{MAX_BPM, MIN_BPM, TRACKER_VOLUME_MAX};
use crate::audio::Sound;
use crate::state::tracker_flags;
use nether_tracker::{TrackerEffect, TrackerNote};

impl TrackerEngine {
    pub(super) fn row_notes(
        &self,
        raw_handle: u32,
        order: u16,
        row: u16,
    ) -> Vec<(usize, TrackerNote)> {
        if self.resolved_row_key == Some((raw_handle, order, row)) {
            return self.resolved_row_notes.clone();
        }
        let Some(loaded) = self
            .modules
            .get(raw_handle as usize)
            .and_then(|m| m.as_ref())
        else {
            return Vec::new();
        };
        let Some(pattern) = loaded.module.pattern_at_order(order) else {
            return Vec::new();
        };
        (0..loaded.module.num_channels as usize)
            .filter_map(|channel| {
                pattern
                    .get_note(row, channel as u8)
                    .map(|note| (channel, *note))
            })
            .collect()
    }

    fn apply_row_timing_controls(&mut self, state: &mut crate::state::TrackerState) {
        let raw_handle = raw_tracker_handle(state.handle);
        for (_, note) in self.row_notes(raw_handle, state.order_position, state.row) {
            for effect in [note.volume_effect, note.effect] {
                match effect {
                    TrackerEffect::SetSpeed(speed) if speed != 0 => state.speed = speed as u16,
                    TrackerEffect::SetTempo(tempo) if tempo != 0 => {
                        state.bpm = (tempo as i16).clamp(MIN_BPM, MAX_BPM) as u16;
                    }
                    _ => {}
                }
            }
        }
    }

    fn prepare_row(
        &mut self,
        state: &mut crate::state::TrackerState,
        sounds: &[Option<Sound>],
    ) -> bool {
        self.sync_handle = state.handle;
        if state.tick != 0 || state.tick_sample_pos != 0 {
            return true;
        }
        if self.pattern_delay_count != 0 {
            if !self.is_it_format {
                for channel in &mut self.channels {
                    if channel.auto_vibrato_depth > 0 || channel.vibrato_active {
                        channel.period = channel.base_period;
                        if channel.vibrato_active && channel.vibrato_depth > 0 {
                            channel.period = super::super::utils::xm_vibrato_period(
                                channel.base_period,
                                channel.vibrato_depth,
                                channel.vibrato_waveform,
                                channel.vibrato_pos,
                                channel.xm_amiga_slides,
                                channel.xm_source_tuning,
                            );
                            channel.vibrato_pos =
                                channel.vibrato_pos.wrapping_add(channel.vibrato_speed << 2);
                        }
                    }
                }
                self.advance_xm_auto_vibrato();
            }
            self.repeat_fine_pitch_slides(state.handle);
            // IT replays SDx on each SEx repetition, not other note cells.
            if self.is_it_format {
                for (channel, note) in self.row_notes(
                    raw_tracker_handle(state.handle),
                    state.order_position,
                    state.row,
                ) {
                    if matches!(note.effect, TrackerEffect::NoteDelay(_)) {
                        self.process_note_internal(channel, &note, state.handle, sounds);
                    }
                }
            }
            return true;
        }
        if !self.normalize_order(state) {
            return false;
        }
        self.current_order = state.order_position;
        self.current_row = state.row;

        self.process_row_tick0_internal(state.handle, sounds);
        self.apply_row_timing_controls(state);
        true
    }

    pub(super) fn normalize_order(&mut self, state: &mut crate::state::TrackerState) -> bool {
        let raw_handle = raw_tracker_handle(state.handle);
        let Some((song_length, restart_position, is_it)) = self
            .modules
            .get(raw_handle as usize)
            .and_then(|m| m.as_ref())
            .map(|loaded| {
                (
                    loaded.module.order_table.len() as u16,
                    loaded.module.restart_position,
                    loaded
                        .module
                        .format
                        .contains(nether_tracker::FormatFlags::IS_IT_FORMAT),
                )
            })
        else {
            return false;
        };

        let mut order = state.order_position;
        for _ in 0..=song_length {
            if order >= song_length {
                if (state.flags & tracker_flags::LOOPING) == 0 {
                    state.flags &= !tracker_flags::PLAYING;
                    return false;
                }
                order = restart_position;
                state.row = 0;
                continue;
            }

            let marker = self.modules[raw_handle as usize]
                .as_ref()
                .expect("validated tracker module")
                .module
                .order_table[order as usize];
            match marker {
                254 if is_it => {
                    order += 1;
                    state.row = 0;
                }
                255 if is_it => {
                    if (state.flags & tracker_flags::LOOPING) == 0 {
                        state.flags &= !tracker_flags::PLAYING;
                        return false;
                    }
                    order = restart_position;
                    state.row = 0;
                }
                _ => {
                    state.order_position = order;
                    self.current_order = order;
                    return true;
                }
            }
        }
        state.flags &= !tracker_flags::PLAYING;
        false
    }

    fn row_flow_commands(
        &self,
        raw_handle: u32,
        order: u16,
        row: u16,
    ) -> (Option<u8>, Option<u16>) {
        let is_it = self
            .modules
            .get(raw_handle as usize)
            .and_then(|m| m.as_ref())
            .is_some_and(|loaded| {
                loaded
                    .module
                    .format
                    .contains(nether_tracker::FormatFlags::IS_IT_FORMAT)
            });
        let mut jump = None;
        let mut break_row = None;
        for (_, note) in self.row_notes(raw_handle, order, row) {
            for effect in [note.volume_effect, note.effect] {
                match effect {
                    TrackerEffect::PositionJump(target) => {
                        jump = Some(target);
                        if !is_it {
                            break_row = None;
                        }
                    }
                    TrackerEffect::PatternBreak(target) => {
                        break_row = Some(if is_it {
                            target as u16
                        } else {
                            ((target >> 4) * 10 + (target & 0x0f)) as u16
                        });
                    }
                    _ => {}
                }
            }
        }
        (jump, break_row)
    }

    fn pattern_loop_target(&self, raw_handle: u32, order: u16, row: u16) -> Option<u16> {
        let mut target = None;
        for (channel, note) in self.row_notes(raw_handle, order, row) {
            if [note.volume_effect, note.effect]
                .iter()
                .any(|effect| matches!(effect, TrackerEffect::PatternLoop(count) if *count > 0))
                && self.channels[channel].pattern_loop_count > 0
            {
                target = Some(self.channels[channel].pattern_loop_row);
                if self.is_it_format || self.channels[channel].xm_legacy_retrigger {
                    return target;
                }
            }
        }
        target
    }

    fn advance_row(&mut self, state: &mut crate::state::TrackerState) {
        let raw_handle = raw_tracker_handle(state.handle);
        let delayed_xm = !self.is_it_format && self.pattern_delay > 0;
        if self.pattern_delay > 0 {
            if self.pattern_delay_count < self.pattern_delay {
                self.pattern_delay_count += 1;
                return;
            }
            self.pattern_delay = 0;
            self.pattern_delay_count = 0;
        }

        self.fine_pattern_delay = 0;
        let (jump, break_row) = self.row_flow_commands(raw_handle, state.order_position, state.row);
        if jump.is_some() || break_row.is_some() {
            state.order_position = jump
                .map(u16::from)
                .unwrap_or(state.order_position.saturating_add(1));
            state.row = break_row.unwrap_or(0);
            self.xm_next_pattern_row = 0;
            self.xm_loop_owner = None;
        } else if let Some(loop_row) =
            self.pattern_loop_target(raw_handle, state.order_position, state.row)
        {
            state.row = loop_row.saturating_add(u16::from(delayed_xm));
        } else {
            let rows = self
                .modules
                .get(raw_handle as usize)
                .and_then(|m| m.as_ref())
                .and_then(|loaded| loaded.module.pattern_at_order(state.order_position))
                .map(|pattern| pattern.num_rows)
                .unwrap_or(0);
            if state.row.saturating_add(1) < rows {
                state.row += 1;
            } else {
                state.order_position = state.order_position.saturating_add(1);
                state.row = if self.is_it_format {
                    0
                } else {
                    std::mem::take(&mut self.xm_next_pattern_row)
                };
                self.xm_loop_owner = None;
            }
        }

        self.current_order = state.order_position;
        self.current_row = state.row;
        self.normalize_order(state);
    }

    fn advance_clock(&mut self, state: &mut crate::state::TrackerState, sample_rate: u32) {
        state.advance_sample_clock(sample_rate);
        self.current_sample_clock = state.rendered_samples();
        state.tick_sample_pos += 1;
        let spt = samples_per_tick(state.bpm, sample_rate).max(1);
        if state.tick_sample_pos < spt {
            return;
        }

        state.tick_sample_pos = 0;
        state.tick += 1;
        let effective_speed = state.speed.max(1).saturating_add(self.fine_pattern_delay);
        if state.tick < effective_speed {
            self.process_tick(state.tick, state.speed);
            if self.is_it_format {
                // IT applies each channel's remembered T command in channel order.
                for (_, note) in self.row_notes(
                    raw_tracker_handle(state.handle),
                    state.order_position,
                    state.row,
                ) {
                    let delta = match note.effect {
                        TrackerEffect::TempoSlideUp(amount) => i16::from(amount),
                        TrackerEffect::TempoSlideDown(amount) => -i16::from(amount),
                        _ => 0,
                    };
                    state.bpm = (state.bpm as i16 + delta).clamp(MIN_BPM, MAX_BPM) as u16;
                }
            } else if self.tempo_slide != 0 {
                state.bpm =
                    (state.bpm as i16 + self.tempo_slide as i16).clamp(MIN_BPM, MAX_BPM) as u16;
            }
        } else {
            self.advance_envelopes();
            state.tick = 0;
            self.advance_row(state);
        }
    }

    /// Render one stereo sample from the tracker (read-only, no state advance)
    pub fn render_sample(
        &mut self,
        state: &crate::state::TrackerState,
        sounds: &[Option<Sound>],
        sample_rate: u32,
    ) -> (f32, f32) {
        if state.handle == 0 || (state.flags & tracker_flags::PLAYING) == 0 {
            return (0.0, 0.0);
        }

        if (state.flags & tracker_flags::PAUSED) != 0 {
            return (0.0, 0.0);
        }

        let raw_handle = raw_tracker_handle(state.handle);
        if self
            .modules
            .get(raw_handle as usize)
            .and_then(|m| m.as_ref())
            .is_none()
        {
            return (0.0, 0.0);
        }

        let (left, right) = self.mix_channels(
            raw_handle,
            sounds,
            sample_rate,
            samples_per_tick(state.bpm, sample_rate),
        );
        let vol = state.volume as f32 / TRACKER_VOLUME_MAX;
        (left * vol, right * vol)
    }

    /// Render one stereo sample and advance the tracker state
    pub fn render_sample_and_advance(
        &mut self,
        state: &mut crate::state::TrackerState,
        sounds: &[Option<Sound>],
        sample_rate: u32,
    ) -> (f32, f32) {
        if state.handle == 0
            || (state.flags & tracker_flags::PLAYING) == 0
            || (state.flags & tracker_flags::PAUSED) != 0
        {
            return (0.0, 0.0);
        }
        if !self.prepare_row(state, sounds) {
            return (0.0, 0.0);
        }

        let raw_handle = raw_tracker_handle(state.handle);
        let (left, right) = self.mix_channels(
            raw_handle,
            sounds,
            sample_rate,
            samples_per_tick(state.bpm, sample_rate),
        );
        self.advance_clock(state, sample_rate);

        self.current_tick = state.tick;
        self.tick_samples_rendered = state.tick_sample_pos;
        let vol = state.volume as f32 / TRACKER_VOLUME_MAX;
        (left * vol, right * vol)
    }

    /// Advance tracker positions without generating samples.
    pub fn advance_positions(
        &mut self,
        state: &mut crate::state::TrackerState,
        sounds: &[Option<Sound>],
        samples_per_frame: u32,
        sample_rate: u32,
    ) {
        if state.handle == 0
            || (state.flags & tracker_flags::PLAYING) == 0
            || (state.flags & tracker_flags::PAUSED) != 0
        {
            return;
        }

        let raw_handle = raw_tracker_handle(state.handle);
        if self
            .modules
            .get(raw_handle as usize)
            .and_then(|m| m.as_ref())
            .is_none()
        {
            return;
        }

        for _ in 0..samples_per_frame {
            if !self.prepare_row(state, sounds) {
                break;
            }
            // Reuse the mixer state transitions; discard only the output.
            // ponytail: silent advance costs a mix; optimize only with state-parity tests.
            let _ = self.process_channels::<false>(
                raw_handle,
                sounds,
                sample_rate,
                samples_per_tick(state.bpm, sample_rate),
            );
            self.advance_clock(state, sample_rate);
            if (state.flags & tracker_flags::PLAYING) == 0 {
                break;
            }
        }
        self.current_tick = state.tick;
        self.tick_samples_rendered = state.tick_sample_pos;
    }
}
