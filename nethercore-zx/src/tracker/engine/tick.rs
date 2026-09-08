//! Per-tick effect processing (called every tick except tick 0)

use super::super::utils::{
    apply_it_linear_slide, get_waveform_value, note_to_period, slide_xm_period, xm_sample_finetune,
    xm_vibrato_period,
};
use super::super::{MAX_TRACKER_CHANNELS, TrackerEngine};
use super::{CHANNEL_VOLUME_MAX, VOLUME_FADEOUT_MAX};

// IT loop ends are inclusive; sustain takes precedence over a normal loop.
// Keep the existing FT2 loop-before-sustain ordering separate.
fn advance_envelope(
    pos: u16,
    sustain: Option<(u16, u16)>,
    normal: Option<(u16, u16)>,
    released: bool,
    it: bool,
) -> u16 {
    if it {
        let next = u32::from(pos) + 1;
        let active = if !released {
            sustain.or(normal)
        } else {
            normal
        };
        if let Some((start, end)) = active
            && next > u32::from(end)
        {
            return start;
        }
        return next.min(u16::MAX as u32) as u16;
    }
    let held = !released && sustain.is_some_and(|(start, _)| pos >= start);
    let next = if held { pos } else { pos.saturating_add(1) };
    if let Some((start, end)) = normal
        && next >= end
    {
        return start;
    }
    next
}

impl TrackerEngine {
    /// Voice envelopes/fadeout advance at every completed tick, including row boundaries.
    pub(super) fn advance_envelopes(&mut self) {
        self.advance_envelopes_except(&[false; MAX_TRACKER_CHANNELS]);
    }

    fn advance_envelopes_except(&mut self, new_voices: &[bool; MAX_TRACKER_CHANNELS]) {
        for (index, channel) in self.channels.iter_mut().enumerate() {
            if !channel.note_on || new_voices[index] {
                continue;
            }
            // Volume envelope advancement
            if channel.volume_envelope_enabled {
                channel.envelope_started |= 1;
                channel.volume_envelope_pos = advance_envelope(
                    channel.volume_envelope_pos,
                    channel.volume_envelope_sustain_loop,
                    if !self.is_it_format && channel.key_off && channel.volume_envelope_escape_loop
                    {
                        None
                    } else {
                        channel.volume_envelope_loop
                    },
                    channel.key_off,
                    self.is_it_format,
                );
            }

            // IT retires voices past a silent terminal volume node, not at a
            // zero point inside a loop or while sustain holds the envelope.
            if self.is_it_format
                && channel.volume_envelope_enabled
                && channel.volume_envelope_loop.is_none()
                && channel
                    .volume_envelope_zero_end
                    .is_some_and(|end| channel.volume_envelope_pos > end)
            {
                channel.volume_fadeout = 0;
                channel.note_on = false;
            }

            // Panning envelope advancement
            if channel.panning_envelope_enabled && !channel.panning_envelope_frozen {
                channel.envelope_started |= 2;
                let previous = channel.panning_envelope_pos;
                channel.panning_envelope_pos = advance_envelope(
                    channel.panning_envelope_pos,
                    channel.panning_envelope_sustain_loop,
                    if !self.is_it_format && channel.key_off && channel.panning_envelope_escape_loop
                    {
                        None
                    } else {
                        channel.panning_envelope_loop
                    },
                    channel.key_off,
                    self.is_it_format,
                );
                channel.panning_envelope_frozen = !self.is_it_format
                    && !channel.key_off
                    && channel
                        .panning_envelope_sustain_loop
                        .is_some_and(|(start, end)| {
                            previous == end && channel.panning_envelope_pos == start
                        });
            }

            // Pitch envelope advancement (IT only)
            if channel.pitch_envelope_enabled {
                channel.envelope_started |= 4;
                channel.pitch_envelope_pos = advance_envelope(
                    channel.pitch_envelope_pos,
                    channel.pitch_envelope_sustain_loop,
                    channel.pitch_envelope_loop,
                    channel.key_off,
                    self.is_it_format,
                );
            }

            // Filter envelope advancement (IT only)
            if channel.filter_envelope_enabled {
                channel.envelope_started |= 8;
                channel.filter_envelope_pos = advance_envelope(
                    channel.filter_envelope_pos,
                    channel.filter_envelope_sustain_loop,
                    channel.filter_envelope_loop,
                    channel.key_off,
                    self.is_it_format,
                );
            }

            // IT lets a non-looping volume envelope finish before automatic
            // fading. Explicit fade, no envelope, and looped release fade now.
            if self.is_it_format
                && channel.volume_envelope_enabled
                && channel.volume_envelope_loop.is_none()
                && channel
                    .volume_envelope_end
                    .is_some_and(|end| channel.volume_envelope_pos > end)
            {
                channel.note_fade = true;
            }
            let release_fades = channel.key_off
                && (!self.is_it_format
                    || !channel.volume_envelope_enabled
                    || channel.volume_envelope_loop.is_some());
            if (release_fades || channel.note_fade) && channel.instrument_fadeout_rate > 0 {
                channel.volume_fadeout = channel
                    .volume_fadeout
                    .saturating_sub(channel.instrument_fadeout_rate);
                if channel.volume_fadeout == 0 {
                    channel.note_on = false;
                }
            }
        }
    }

    /// Original authored playback measurements: instrument waveforms use their
    /// own 256-tick phase coordinate and source-period depth, including tick zero.
    pub(super) fn advance_xm_auto_vibrato(&mut self) {
        if self.is_it_format {
            return;
        }
        for channel in &mut self.channels {
            if !channel.note_on || channel.auto_vibrato_depth == 0 {
                continue;
            }
            channel.auto_vibrato_pos =
                (channel.auto_vibrato_pos + u16::from(channel.auto_vibrato_rate)) & 255;
            if !channel.key_off {
                channel.auto_vibrato_sweep = channel.auto_vibrato_sweep.saturating_add(1);
            }
            let phase = channel.auto_vibrato_pos as u8;
            let wave = match channel.auto_vibrato_type {
                0 => -(f32::from(phase) * std::f32::consts::TAU / 256.0).sin(),
                1 => {
                    if phase < 128 {
                        -1.0
                    } else {
                        1.0
                    }
                }
                2 => f32::from(phase as i8) / 128.0,
                3 => f32::from(phase.wrapping_sub(1).wrapping_neg() as i8) / 128.0,
                _ => 0.0,
            };
            let sweep = if channel.key_off
                && channel.auto_vibrato_sweep < u16::from(channel.auto_vibrato_sweep_len)
            {
                0.0
            } else if channel.auto_vibrato_sweep_len == 0 {
                1.0
            } else {
                (f32::from(channel.auto_vibrato_sweep) / f32::from(channel.auto_vibrato_sweep_len))
                    .min(1.0)
            };
            let delta = (wave * f32::from(channel.auto_vibrato_depth) * sweep).trunc();
            channel.period = slide_xm_period(
                channel.period,
                delta,
                channel.xm_amiga_slides,
                channel.xm_source_tuning,
            );
        }
    }

    /// Process per-tick effects (called every tick except tick 0)
    pub fn process_tick(&mut self, tick: u16, speed: u16) {
        let mut new_voices = [false; MAX_TRACKER_CHANNELS];
        for (ch_idx, new_voice) in new_voices.iter_mut().enumerate() {
            let skip_delayed_volume_slide = !self.is_it_format
                && tick == self.channels[ch_idx].note_delay_tick as u16
                && self.channels[ch_idx]
                    .delayed_xm_note
                    .is_some_and(|(note, _)| note.instrument != 0);
            if self.channels[ch_idx].note_delay_tick as u16 == tick
                && let Some((mut note, handle)) = self.channels[ch_idx].delayed_xm_note.take()
            {
                self.channels[ch_idx].note_delay_tick = 0;
                // IT S6x extends the SDx boundary; XM keeps its base-speed limit.
                let delay_limit = if self.is_it_format {
                    speed.max(1).saturating_add(self.fine_pattern_delay)
                } else {
                    speed
                };
                if tick < delay_limit {
                    if !self.is_it_format
                        && note.note == 0
                        && !self.channels[ch_idx].xm_legacy_retrigger
                    {
                        note.note = self.channels[ch_idx].current_note;
                    }
                    note.effect = nether_tracker::TrackerEffect::None;
                    // FT2 EDx turns volume-column portamento into a fresh note.
                    if !self.is_it_format
                        && matches!(
                            note.volume_effect,
                            nether_tracker::TrackerEffect::TonePortamento(_)
                        )
                    {
                        note.volume_effect = nether_tracker::TrackerEffect::None;
                    }
                    // A real delayed retrigger must first render envelope tick zero.
                    // Non-retriggering portamento and displaced NNA voices still advance.
                    *new_voice = self.process_note_internal(ch_idx, &note, handle, &[]);
                }
            }
            let channel = &mut self.channels[ch_idx];
            if !channel.note_on {
                continue;
            }

            if channel.arpeggio_active
                || channel.vibrato_active
                || (!self.is_it_format && channel.auto_vibrato_depth > 0)
            {
                channel.period = channel.base_period;
            }
            // IT retains its existing tick ordering.
            if self.is_it_format
                && channel.arpeggio_active
                && (channel.arpeggio_note1 != 0 || channel.arpeggio_note2 != 0)
            {
                channel.arpeggio_tick = if self.is_it_format {
                    ((channel.arpeggio_tick as u16 + 1) % 3) as u8
                } else {
                    (speed.saturating_sub(tick) % 3) as u8
                };
                let note_offset = match channel.arpeggio_tick {
                    0 => 0,
                    1 => channel.arpeggio_note1,
                    _ => channel.arpeggio_note2,
                };
                let arp_period = channel.base_period - note_offset as f32 * 64.0;
                channel.period = arp_period.max(1.0);
            }

            // XM volume-column slide precedes, and does not replace, Axy/5xy/6xy.
            if !self.is_it_format && !skip_delayed_volume_slide {
                channel.volume = (channel.volume
                    + f32::from(channel.xm_volume_column_slide) / 64.0)
                    .clamp(0.0, 1.0);
            }
            // Volume slide
            if channel.volume_slide_active && !skip_delayed_volume_slide {
                let vol_slide = channel.last_volume_slide;
                if vol_slide != 0 {
                    let up = (vol_slide >> 4) as f32 / 64.0;
                    let down = (vol_slide & 0x0F) as f32 / 64.0;
                    if up > 0.0 {
                        channel.volume = (channel.volume + up).min(1.0);
                    } else {
                        channel.volume = (channel.volume - down).max(0.0);
                    }
                }
            }

            // Channel volume slide (IT only)
            if channel.channel_volume_slide_active && channel.channel_volume_slide != 0 {
                if channel.channel_volume_slide > 0 {
                    channel.channel_volume = channel
                        .channel_volume
                        .saturating_add(channel.channel_volume_slide as u8)
                        .min(64);
                } else {
                    channel.channel_volume = channel
                        .channel_volume
                        .saturating_sub((-channel.channel_volume_slide) as u8);
                }
            }

            // Portamento up
            if channel.porta_up_count != 0 && channel.last_porta_up != 0 {
                if self.is_it_format {
                    channel.period = apply_it_linear_slide(
                        channel.period,
                        channel.last_porta_up as i16 * channel.porta_up_count as i16,
                    );
                } else {
                    channel.period = slide_xm_period(
                        channel.period,
                        -(channel.last_porta_up as f32) * 4.0 * channel.porta_up_count as f32,
                        channel.xm_amiga_slides,
                        channel.xm_source_tuning,
                    );
                }
            }

            // Portamento down
            if channel.porta_down_count != 0 && channel.last_porta_down != 0 {
                if self.is_it_format {
                    channel.period = apply_it_linear_slide(
                        channel.period,
                        -(channel.last_porta_down as i16 * channel.porta_down_count as i16),
                    );
                } else {
                    channel.period = slide_xm_period(
                        channel.period,
                        channel.last_porta_down as f32 * 4.0 * channel.porta_down_count as f32,
                        channel.xm_amiga_slides,
                        channel.xm_source_tuning,
                    );
                }
            }

            // Tone portamento
            if channel.tone_porta_active && channel.target_period > 0.0 && channel.porta_speed > 0 {
                let diff = channel.target_period - channel.period;
                if self.is_it_format {
                    let slide = channel.porta_speed as i16;
                    if diff > 0.0 {
                        let new_period = apply_it_linear_slide(channel.period, -slide);
                        if new_period >= channel.target_period {
                            channel.period = channel.target_period;
                        } else {
                            channel.period = new_period;
                        }
                    } else if diff < 0.0 {
                        let new_period = apply_it_linear_slide(channel.period, slide);
                        if new_period <= channel.target_period {
                            channel.period = channel.target_period;
                        } else {
                            channel.period = new_period;
                        }
                    }
                } else if channel.xm_porta_target_reached {
                    let next = slide_xm_period(
                        channel.period,
                        channel.porta_speed as f32 * 4.0,
                        channel.xm_amiga_slides,
                        channel.xm_source_tuning,
                    );
                    channel.period = next.min(channel.target_period);
                } else if diff != 0.0 {
                    let speed = channel.porta_speed as f32 * 4.0;
                    let next = slide_xm_period(
                        channel.period,
                        speed.copysign(diff),
                        channel.xm_amiga_slides,
                        channel.xm_source_tuning,
                    );
                    channel.period = if diff > 0.0 {
                        next.min(channel.target_period)
                    } else {
                        next.max(channel.target_period)
                    };
                }
                if !self.is_it_format
                    && !channel.xm_legacy_retrigger
                    && diff < 0.0
                    && channel.period == channel.target_period
                {
                    channel.xm_porta_target_reached = true;
                }
            }

            if !self.is_it_format {
                channel.base_period = channel.period;
                if channel.arpeggio_active
                    && (channel.arpeggio_note1 != 0 || channel.arpeggio_note2 != 0)
                {
                    channel.arpeggio_tick = (speed.saturating_sub(tick) % 3) as u8;
                    let offset = match channel.arpeggio_tick {
                        0 => 0,
                        1 => channel.arpeggio_note1,
                        _ => channel.arpeggio_note2,
                    };
                    channel.period = (channel.base_period - f32::from(offset) * 64.0).max(1.0);
                }
            }

            // Vibrato
            if !self.is_it_format && channel.vibrato_active && channel.vibrato_depth > 0 {
                channel.period = xm_vibrato_period(
                    channel.base_period,
                    channel.vibrato_depth,
                    channel.vibrato_waveform,
                    channel.vibrato_pos,
                    channel.xm_amiga_slides,
                    channel.xm_source_tuning,
                );
                channel.vibrato_pos = channel.vibrato_pos.wrapping_add(channel.vibrato_speed << 2);
            }

            if self.is_it_format {
                channel.advance_it_modulation(false, self.old_effects_mode);
            }

            // Preserve the existing IT auto-vibrato path.
            if self.is_it_format && channel.auto_vibrato_depth > 0 {
                let auto_vib = get_waveform_value(
                    channel.auto_vibrato_type,
                    (channel.auto_vibrato_pos >> 2) as u8,
                );
                let sweep_factor = if channel.auto_vibrato_sweep_len > 0 {
                    let sweep_progress = channel.auto_vibrato_sweep as f32
                        / (channel.auto_vibrato_sweep_len as f32 * 256.0);
                    sweep_progress.min(1.0)
                } else {
                    1.0
                };
                let depth_scale = if self.is_it_format && !self.old_effects_mode {
                    32.0 / 15.0
                } else {
                    128.0 / 15.0
                };
                let delta =
                    auto_vib * channel.auto_vibrato_depth as f32 * sweep_factor * depth_scale;
                channel.period += delta;
                channel.auto_vibrato_pos = channel
                    .auto_vibrato_pos
                    .wrapping_add(channel.auto_vibrato_rate as u16);
                if channel.auto_vibrato_sweep < 65535 {
                    channel.auto_vibrato_sweep = channel.auto_vibrato_sweep.saturating_add(1);
                }
            }

            // Tremolo
            if !self.is_it_format && channel.tremolo_active && channel.tremolo_depth > 0 {
                channel.xm_tremolo_delta = super::super::utils::xm_tremolo_delta(
                    channel.tremolo_depth,
                    channel.tremolo_waveform,
                    channel.tremolo_pos,
                    channel.vibrato_pos,
                );
                channel.tremolo_pos = channel.tremolo_pos.wrapping_add(channel.tremolo_speed << 2);
            }

            // XM Rxy counts active ticks including tick zero; E9x and IT keep row timing.
            if !self.is_it_format && channel.xm_multi_retrigger_active {
                channel.advance_xm_retrigger();
            } else if !self.is_it_format
                && channel.retrigger_tick > 0
                && tick.is_multiple_of(channel.retrigger_tick as u16)
            {
                channel.retrigger_sample(false);
                if !self.is_it_format {
                    // E9 restores default sample tuning, including FT2 quantization.
                    let source = i16::from(channel.xm_source_finetune) - channel.xm_finetune_delta;
                    let restored = xm_sample_finetune(source as i8, channel.xm_legacy_retrigger);
                    let delta = i16::from(restored) - source;
                    // E9 restarts the remembered note, even during an unfinished porta.
                    let retrigger_note = if channel.xm_retrigger_note == 0 {
                        channel.current_note
                    } else {
                        channel.xm_retrigger_note
                    };
                    channel.period =
                        note_to_period(retrigger_note, channel.finetune) - f32::from(delta) / 2.0;
                    channel.base_period = channel.period;
                    // Measured legacy E9 refreshes the active target; FT2 retains it.
                    if channel.xm_legacy_retrigger
                        && channel.tone_porta_active
                        && channel.target_period > 0.0
                    {
                        channel.target_period = channel.base_period;
                        channel.xm_porta_target_reached = false;
                    }
                    channel.current_note = retrigger_note;
                    channel.xm_source_finetune = restored;
                    channel.xm_finetune_delta = delta;
                    // Measured E9x restarts envelopes; Rxy above preserves them.
                    channel.volume_envelope_pos = 0;
                    channel.panning_envelope_pos = 0;
                    channel.panning_envelope_frozen = false;
                    channel.envelope_started = 0;
                    *new_voice = true;
                }
            }

            if channel.panning_left_on_ticks && !skip_delayed_volume_slide {
                channel.panning = -1.0;
            }

            // XM columns execute independently before main-column Pxy.
            if !self.is_it_format
                && channel.xm_volume_column_pan_slide != 0
                && (channel.xm_legacy_retrigger || !skip_delayed_volume_slide)
            {
                let scale = if channel.xm_legacy_retrigger {
                    32.0
                } else {
                    128.0
                };
                let upper = if channel.xm_legacy_retrigger {
                    1.0
                } else {
                    127.0 / 128.0
                };
                channel.panning = (channel.panning
                    + channel.xm_volume_column_pan_slide as f32 / scale)
                    .clamp(-1.0, upper);
            }
            if channel.panning_slide_active && channel.panning_slide != 0 {
                let (scale, upper) = if self.is_it_format {
                    (255.0, 1.0)
                } else if channel.xm_legacy_retrigger {
                    (32.0, 1.0)
                } else {
                    (128.0, 127.0 / 128.0)
                };
                channel.panning =
                    (channel.panning + channel.panning_slide as f32 / scale).clamp(-1.0, upper);
            }

            // Tremor (IT)
            if channel.tremor_active && !self.is_it_format {
                // FT2 gates on nonzero ticks. Tick zero holds the previous gate,
                // and T00 recalls duration without restarting phase at each row.
                let duration = if channel.tremor_mute {
                    channel.tremor_off_ticks
                } else {
                    channel.tremor_on_ticks
                };
                if channel.tremor_counter > duration {
                    channel.tremor_mute = !channel.tremor_mute;
                    channel.tremor_counter = 0;
                }
                channel.tremor_counter = channel.tremor_counter.saturating_add(1);
            }

            // Panbrello (IT)
            if channel.panbrello_active && channel.panbrello_depth > 0 {
                let panbrello =
                    get_waveform_value(channel.panbrello_waveform, channel.panbrello_pos);
                let delta = panbrello * channel.panbrello_depth as f32 / 64.0;
                channel.panning = (channel.panning + delta).clamp(-1.0, 1.0);
                channel.panbrello_pos = channel.panbrello_pos.wrapping_add(channel.panbrello_speed);
            }

            // Note cut
            if channel.note_cut_tick > 0 && tick == channel.note_cut_tick as u16 {
                channel.volume = 0.0;
                if self.is_it_format {
                    channel.note_on = false;
                }
            }

            // Note delay
            if channel.note_delay_tick > 0 && tick == channel.note_delay_tick as u16 {
                if channel.delayed_note > 0 && channel.delayed_note <= 96 {
                    channel.sample_pos = 0.0;
                    channel.xm_source_position = 0.0;
                    channel.xm_loop_stopped = false;
                    channel.note_on = true;
                    channel.key_off = false;
                    channel.sample_sustain_released = false;
                    channel.note_fade = false;
                    channel.volume_envelope_pos = 0;
                    channel.envelope_started = 0;
                    channel.panning_envelope_pos = 0;
                    channel.panning_envelope_frozen = false;
                    channel.volume_fadeout = VOLUME_FADEOUT_MAX as u16;
                    if channel.vibrato_waveform < 4 {
                        channel.vibrato_pos = 0;
                    }
                    if channel.tremolo_waveform < 4 {
                        channel.tremolo_pos = 0;
                    }
                    channel.base_period = note_to_period(channel.delayed_note, channel.finetune);
                    channel.period = channel.base_period;
                }
                channel.note_delay_tick = 0;
            }

            // Key off timing
            if channel.key_off_tick > 0 && tick == channel.key_off_tick as u16 {
                channel.key_off = true;
                channel.sample_sustain_released = true;
                if !self.is_it_format && !channel.volume_envelope_enabled {
                    channel.volume = 0.0;
                }
            }

            // Volume column effects (per-tick)
            match channel.vol_col_effect {
                0x6 => {
                    channel.volume =
                        (channel.volume - channel.vol_col_param as f32 / 64.0).max(0.0);
                }
                0x7 => {
                    channel.volume =
                        (channel.volume + channel.vol_col_param as f32 / 64.0).min(1.0);
                }
                0xB => {
                    channel.vibrato_depth = channel.vol_col_param;
                }
                0xD => {
                    channel.panning =
                        (channel.panning - channel.vol_col_param as f32 / 16.0).clamp(-1.0, 1.0);
                }
                0xE => {
                    channel.panning =
                        (channel.panning + channel.vol_col_param as f32 / 16.0).clamp(-1.0, 1.0);
                }
                0xF => {}
                _ => {}
            }

            // Glissando is an output pitch quantizer; never round the slide accumulator.
            // Quantization is applied by sample_channel, preserving sub-semitone progress.
        }

        self.advance_xm_auto_vibrato();
        self.advance_envelopes_except(&new_voices);

        // Global volume slide
        for channel in &self.channels {
            if !channel.global_volume_slide_active || channel.is_background {
                continue;
            }
            let scale = if self.is_it_format {
                128.0
            } else {
                CHANNEL_VOLUME_MAX
            };
            let up = (channel.global_volume_slide >> 4) as f32 / scale;
            let down = (channel.global_volume_slide & 0x0F) as f32 / scale;
            if self.is_it_format && up > 0.0 && down > 0.0 {
                continue;
            }
            if up > 0.0 {
                self.global_volume = (self.global_volume + up).min(1.0);
            } else if down > 0.0 {
                self.global_volume = (self.global_volume - down).max(0.0);
            }
        }
    }
}

#[cfg(test)]
mod envelope_tests {
    use super::advance_envelope;
    #[test]
    fn inclusive_it_endpoints_and_legacy_xm_ordering() {
        assert_eq!(advance_envelope(2, None, Some((1, 3)), false, true), 3);
        assert_eq!(advance_envelope(3, None, Some((1, 3)), false, true), 1);
        assert_eq!(advance_envelope(2, None, Some((1, 3)), false, false), 1);
        assert_eq!(
            advance_envelope(3, Some((3, 3)), Some((0, 3)), false, true),
            3
        );
        assert_eq!(
            advance_envelope(3, Some((3, 3)), Some((0, 3)), true, true),
            0
        );
        assert_eq!(
            advance_envelope(u16::MAX, None, Some((1, u16::MAX)), false, true),
            1
        );
        assert_eq!(
            advance_envelope(u16::MAX, None, None, false, true),
            u16::MAX
        );
    }
}
