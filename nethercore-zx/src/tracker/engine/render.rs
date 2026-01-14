//! Public rendering API methods

use super::super::utils::samples_per_tick;
use super::super::{MAX_TRACKER_CHANNELS, TrackerEngine, raw_tracker_handle};
use super::{CHANNEL_VOLUME_MAX, MAX_BPM, MIN_BPM, TRACKER_VOLUME_MAX};
use crate::audio::Sound;
use crate::state::tracker_flags;

impl TrackerEngine {
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

        // Early return if module not found
        if self
            .modules
            .get(raw_handle as usize)
            .and_then(|m| m.as_ref())
            .is_none()
        {
            return (0.0, 0.0);
        }

        let (left, right) = self.mix_channels(raw_handle, sounds, sample_rate);
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
        if state.handle == 0 || (state.flags & tracker_flags::PLAYING) == 0 {
            return (0.0, 0.0);
        }

        if (state.flags & tracker_flags::PAUSED) != 0 {
            return (0.0, 0.0);
        }

        // Process tick 0 at the start of a row
        if state.tick == 0 && state.tick_sample_pos == 0 {
            self.process_row_tick0_internal(state.handle, sounds);
        }

        let raw_handle = raw_tracker_handle(state.handle);

        // Early return if module not found
        if self
            .modules
            .get(raw_handle as usize)
            .and_then(|m| m.as_ref())
            .is_none()
        {
            return (0.0, 0.0);
        }

        let (left, right) = self.mix_channels(raw_handle, sounds, sample_rate);

        // Advance tick position
        state.tick_sample_pos += 1;
        let spt = samples_per_tick(state.bpm, sample_rate);

        if state.tick_sample_pos >= spt {
            state.tick_sample_pos = 0;
            state.tick += 1;

            if state.tick > 0 {
                self.process_tick(state.tick, state.speed);

                if self.tempo_slide != 0 {
                    let new_bpm =
                        (state.bpm as i16 + self.tempo_slide as i16).clamp(MIN_BPM, MAX_BPM) as u16;
                    state.bpm = new_bpm;
                }
            }

            let effective_speed = state.speed + self.fine_pattern_delay as u16;
            if state.tick >= effective_speed {
                state.tick = 0;
                self.fine_pattern_delay = 0;

                if self.pattern_delay > 0 {
                    if self.pattern_delay_count < self.pattern_delay {
                        self.pattern_delay_count += 1;
                        let vol = state.volume as f32 / TRACKER_VOLUME_MAX;
                        return (left * vol, right * vol);
                    } else {
                        self.pattern_delay = 0;
                        self.pattern_delay_count = 0;
                    }
                }

                state.row += 1;
                self.current_row = state.row;

                let (num_rows, song_length, restart_position) = {
                    let loaded = match self
                        .modules
                        .get(raw_handle as usize)
                        .and_then(|m| m.as_ref())
                    {
                        Some(m) => m,
                        None => {
                            return (
                                left * state.volume as f32 / TRACKER_VOLUME_MAX,
                                right * state.volume as f32 / TRACKER_VOLUME_MAX,
                            );
                        }
                    };
                    let num_rows = loaded
                        .module
                        .pattern_at_order(state.order_position)
                        .map(|p| p.num_rows)
                        .unwrap_or(CHANNEL_VOLUME_MAX as u16);
                    (
                        num_rows,
                        loaded.module.order_table.len() as u16,
                        loaded.module.restart_position,
                    )
                };

                if state.row >= num_rows {
                    state.row = 0;
                    state.order_position += 1;
                    self.current_order = state.order_position;
                    self.current_row = 0;

                    if state.order_position >= song_length {
                        if (state.flags & tracker_flags::LOOPING) != 0 {
                            state.order_position = restart_position;
                            self.current_order = restart_position;
                        } else {
                            state.flags &= !tracker_flags::PLAYING;
                        }
                    }
                }
            }
        }

        let vol = state.volume as f32 / TRACKER_VOLUME_MAX;
        (left * vol, right * vol)
    }

    /// Advance tracker positions without generating samples
    ///
    /// This is a lightweight version of `render_sample_and_advance` that advances
    /// all tracker state (tick, row, order positions, effect processing, envelope
    /// advancement) without performing the expensive sample mixing.
    ///
    /// Used in threaded audio mode to advance the main thread's tracker state
    /// without the cost of sample generation. The audio thread handles actual
    /// sample generation from its snapshot.
    ///
    /// # Performance
    ///
    /// This is ~10-20x faster than `render_sample_and_advance` because it skips:
    /// - Sample interpolation
    /// - Channel mixing
    /// - Envelope value lookups (just advances positions)
    /// - Panning calculations
    pub fn advance_positions(
        &mut self,
        state: &mut crate::state::TrackerState,
        sounds: &[Option<Sound>],
        samples_per_frame: u32,
        sample_rate: u32,
    ) {
        if state.handle == 0 || (state.flags & tracker_flags::PLAYING) == 0 {
            return;
        }

        if (state.flags & tracker_flags::PAUSED) != 0 {
            return;
        }

        let raw_handle = raw_tracker_handle(state.handle);

        // Early return if module not found
        if self
            .modules
            .get(raw_handle as usize)
            .and_then(|m| m.as_ref())
            .is_none()
        {
            return;
        }

        // Process each sample position worth of ticks
        for _ in 0..samples_per_frame {
            // Process tick 0 at the start of a row
            if state.tick == 0 && state.tick_sample_pos == 0 {
                self.process_row_tick0_internal(state.handle, sounds);
            }

            // Advance channel sample positions (without generating samples)
            self.advance_channel_sample_positions(raw_handle, sample_rate);

            // Advance tick position
            state.tick_sample_pos += 1;
            let spt = samples_per_tick(state.bpm, sample_rate);

            if state.tick_sample_pos >= spt {
                state.tick_sample_pos = 0;
                state.tick += 1;

                if state.tick > 0 {
                    self.process_tick(state.tick, state.speed);

                    if self.tempo_slide != 0 {
                        let new_bpm = (state.bpm as i16 + self.tempo_slide as i16)
                            .clamp(MIN_BPM, MAX_BPM) as u16;
                        state.bpm = new_bpm;
                    }
                }

                let effective_speed = state.speed + self.fine_pattern_delay as u16;
                if state.tick >= effective_speed {
                    state.tick = 0;
                    self.fine_pattern_delay = 0;

                    if self.pattern_delay > 0 {
                        if self.pattern_delay_count < self.pattern_delay {
                            self.pattern_delay_count += 1;
                            continue;
                        } else {
                            self.pattern_delay = 0;
                            self.pattern_delay_count = 0;
                        }
                    }

                    state.row += 1;
                    self.current_row = state.row;

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
                            .pattern_at_order(state.order_position)
                            .map(|p| p.num_rows)
                            .unwrap_or(CHANNEL_VOLUME_MAX as u16);
                        (
                            num_rows,
                            loaded.module.order_table.len() as u16,
                            loaded.module.restart_position,
                        )
                    };

                    if state.row >= num_rows {
                        state.row = 0;
                        state.order_position += 1;
                        self.current_order = state.order_position;
                        self.current_row = 0;

                        if state.order_position >= song_length {
                            if (state.flags & tracker_flags::LOOPING) != 0 {
                                state.order_position = restart_position;
                                self.current_order = restart_position;
                            } else {
                                state.flags &= !tracker_flags::PLAYING;
                                return;
                            }
                        }
                    }
                }
            }
        }
    }

    /// Advance all channel sample positions by one output sample
    ///
    /// This updates sample_pos for each active channel based on its period,
    /// without performing interpolation or mixing. This is the lightweight
    /// equivalent of what happens inside `sample_channel`.
    fn advance_channel_sample_positions(&mut self, raw_handle: u32, sample_rate: u32) {
        let num_channels = self
            .modules
            .get(raw_handle as usize)
            .and_then(|m| m.as_ref())
            .map(|m| m.module.num_channels as usize)
            .unwrap_or(0);

        // Process both regular and background channels
        for ch_idx in 0..MAX_TRACKER_CHANNELS {
            let channel = &mut self.channels[ch_idx];

            // Skip inactive channels in regular range
            if ch_idx < num_channels {
                if !channel.note_on || channel.sample_handle == 0 {
                    continue;
                }
            } else {
                // Background channel - check if audible
                if !channel.note_on || channel.sample_handle == 0 || channel.volume_fadeout == 0 {
                    continue;
                }
            }

            // Calculate frequency from period and advance sample position
            let freq = super::super::utils::period_to_frequency(channel.period) as f64;
            let advance = freq / sample_rate as f64;
            channel.sample_pos += advance * channel.sample_direction as f64;

            // Handle loop boundaries (same logic as sample_channel)
            if channel.sample_loop_type > 0 && channel.sample_loop_end > channel.sample_loop_start {
                let loop_start = channel.sample_loop_start as f64;
                let loop_end = channel.sample_loop_end as f64;

                if channel.sample_pos >= loop_end {
                    if channel.sample_loop_type == 2 {
                        // Ping-pong
                        channel.sample_direction = -1;
                        channel.sample_pos = loop_end - (channel.sample_pos - loop_end);
                    } else {
                        // Forward loop
                        channel.sample_pos = loop_start + (channel.sample_pos - loop_end);
                    }
                } else if channel.sample_pos < loop_start && channel.sample_direction < 0 {
                    // Ping-pong reverse
                    channel.sample_direction = 1;
                    channel.sample_pos = loop_start + (loop_start - channel.sample_pos);
                }
            }
        }
    }
}
