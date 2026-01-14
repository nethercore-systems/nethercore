//! Per-tick effect processing (called every tick except tick 0)

use super::super::utils::{apply_it_linear_slide, get_waveform_value, note_to_period};
use super::super::{MAX_TRACKER_CHANNELS, TrackerEngine};
use super::{CHANNEL_VOLUME_MAX, VOLUME_FADEOUT_MAX};

impl TrackerEngine {
    /// Process per-tick effects (called every tick except tick 0)
    pub fn process_tick(&mut self, tick: u16, _speed: u16) {
        for ch_idx in 0..MAX_TRACKER_CHANNELS {
            let channel = &mut self.channels[ch_idx];
            if !channel.note_on {
                continue;
            }

            // Arpeggio
            if channel.arpeggio_active
                && (channel.arpeggio_note1 != 0 || channel.arpeggio_note2 != 0)
            {
                channel.arpeggio_tick = ((channel.arpeggio_tick as u16 + 1) % 3) as u8;
                let note_offset = match channel.arpeggio_tick {
                    0 => 0,
                    1 => channel.arpeggio_note1,
                    _ => channel.arpeggio_note2,
                };
                let arp_period = channel.base_period - note_offset as f32 * 64.0;
                channel.period = arp_period.max(1.0);
            }

            // Volume slide
            if channel.volume_slide_active {
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
            if channel.porta_up_active && channel.last_porta_up != 0 {
                if self.is_it_format {
                    channel.period =
                        apply_it_linear_slide(channel.period, channel.last_porta_up as i16);
                } else {
                    channel.period = (channel.period - channel.last_porta_up as f32 * 4.0).max(1.0);
                }
            }

            // Portamento down
            if channel.porta_down_active && channel.last_porta_down != 0 {
                if self.is_it_format {
                    channel.period =
                        apply_it_linear_slide(channel.period, -(channel.last_porta_down as i16));
                } else {
                    channel.period += channel.last_porta_down as f32 * 4.0;
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
                } else {
                    let speed = channel.porta_speed as f32 * 4.0;
                    if diff.abs() < speed {
                        channel.period = channel.target_period;
                    } else if diff > 0.0 {
                        channel.period += speed;
                    } else {
                        channel.period -= speed;
                    }
                }
            }

            // Vibrato
            if channel.vibrato_active && channel.vibrato_depth > 0 {
                let vibrato = get_waveform_value(channel.vibrato_waveform, channel.vibrato_pos);
                let depth_scale = if self.is_it_format && !self.old_effects_mode {
                    32.0 / 15.0
                } else {
                    128.0 / 15.0
                };
                let delta = vibrato * channel.vibrato_depth as f32 * depth_scale;
                channel.period = channel.base_period + delta;
                channel.vibrato_pos = channel.vibrato_pos.wrapping_add(channel.vibrato_speed << 2);
            }

            // Auto-vibrato
            if channel.auto_vibrato_depth > 0 {
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
            if channel.tremolo_active && channel.tremolo_depth > 0 {
                let tremolo = get_waveform_value(channel.tremolo_waveform, channel.tremolo_pos);
                let delta = tremolo * channel.tremolo_depth as f32 * 4.0 / 128.0;
                channel.volume = (channel.volume + delta).clamp(0.0, 1.0);
                channel.tremolo_pos = channel.tremolo_pos.wrapping_add(channel.tremolo_speed << 2);
            }

            // Retrigger
            if channel.retrigger_tick > 0 && tick.is_multiple_of(channel.retrigger_tick as u16) {
                channel.sample_pos = 0.0;
                match channel.retrigger_mode {
                    6 => channel.volume = (channel.volume * (2.0 / 3.0)).clamp(0.0, 1.0),
                    7 => channel.volume = (channel.volume * 0.5).clamp(0.0, 1.0),
                    14 => channel.volume = (channel.volume * 1.5).clamp(0.0, 1.0),
                    15 => channel.volume = (channel.volume * 2.0).clamp(0.0, 1.0),
                    _ => {
                        if channel.retrigger_volume != 0 {
                            channel.volume = (channel.volume
                                + channel.retrigger_volume as f32 / 64.0)
                                .clamp(0.0, 1.0);
                        }
                    }
                }
            }

            // Panning slide
            if channel.panning_slide_active && channel.panning_slide != 0 {
                channel.panning =
                    (channel.panning + channel.panning_slide as f32 / 255.0).clamp(-1.0, 1.0);
            }

            // Tremor (IT)
            if channel.tremor_active
                && (channel.tremor_on_ticks > 0 || channel.tremor_off_ticks > 0)
            {
                channel.tremor_counter = channel.tremor_counter.saturating_add(1);
                if channel.tremor_mute {
                    if channel.tremor_counter >= channel.tremor_off_ticks {
                        channel.tremor_mute = false;
                        channel.tremor_counter = 0;
                    }
                } else if channel.tremor_counter >= channel.tremor_on_ticks {
                    channel.tremor_mute = true;
                    channel.tremor_counter = 0;
                }
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
                channel.note_on = false;
            }

            // Note delay
            if channel.note_delay_tick > 0 && tick == channel.note_delay_tick as u16 {
                if channel.delayed_note > 0 && channel.delayed_note <= 96 {
                    channel.sample_pos = 0.0;
                    channel.note_on = true;
                    channel.key_off = false;
                    channel.volume_envelope_pos = 0;
                    channel.panning_envelope_pos = 0;
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

            // Glissando
            if channel.glissando && channel.target_period > 0.0 {
                channel.period = (channel.period / 64.0).round() * 64.0;
            }

            // Volume envelope advancement
            if channel.volume_envelope_enabled {
                let at_sustain = if let Some(sus_tick) = channel.volume_envelope_sustain_tick {
                    channel.volume_envelope_pos >= sus_tick && !channel.key_off
                } else {
                    false
                };
                if !at_sustain {
                    channel.volume_envelope_pos += 1;
                }
                if let Some((loop_start, loop_end)) = channel.volume_envelope_loop
                    && channel.volume_envelope_pos >= loop_end
                {
                    channel.volume_envelope_pos = loop_start;
                }
            }

            // Panning envelope advancement
            if channel.panning_envelope_enabled {
                let at_sustain = if let Some(sus_tick) = channel.panning_envelope_sustain_tick {
                    channel.panning_envelope_pos >= sus_tick && !channel.key_off
                } else {
                    false
                };
                if !at_sustain {
                    channel.panning_envelope_pos += 1;
                }
                if let Some((loop_start, loop_end)) = channel.panning_envelope_loop
                    && channel.panning_envelope_pos >= loop_end
                {
                    channel.panning_envelope_pos = loop_start;
                }
            }

            // Pitch envelope advancement (IT only)
            if channel.pitch_envelope_enabled {
                let at_sustain = if let Some(sus_tick) = channel.pitch_envelope_sustain_tick {
                    channel.pitch_envelope_pos >= sus_tick && !channel.key_off
                } else {
                    false
                };
                if !at_sustain {
                    channel.pitch_envelope_pos += 1;
                }
                if let Some((loop_start, loop_end)) = channel.pitch_envelope_loop
                    && channel.pitch_envelope_pos >= loop_end
                {
                    channel.pitch_envelope_pos = loop_start;
                }
            }

            // Filter envelope advancement (IT only)
            if channel.filter_envelope_enabled {
                let at_sustain = if let Some(sus_tick) = channel.filter_envelope_sustain_tick {
                    channel.filter_envelope_pos >= sus_tick && !channel.key_off
                } else {
                    false
                };
                if !at_sustain {
                    channel.filter_envelope_pos += 1;
                }
                if let Some((loop_start, loop_end)) = channel.filter_envelope_loop
                    && channel.filter_envelope_pos >= loop_end
                {
                    channel.filter_envelope_pos = loop_start;
                }
            }

            // Volume fadeout after key-off
            if channel.key_off && channel.instrument_fadeout_rate > 0 {
                channel.volume_fadeout = channel
                    .volume_fadeout
                    .saturating_sub(channel.instrument_fadeout_rate);
                if channel.volume_fadeout == 0 {
                    channel.note_on = false;
                }
            }
        }

        // Global volume slide
        if self.last_global_vol_slide != 0 {
            let up = (self.last_global_vol_slide >> 4) as f32 / CHANNEL_VOLUME_MAX;
            let down = (self.last_global_vol_slide & 0x0F) as f32 / CHANNEL_VOLUME_MAX;
            if up > 0.0 {
                self.global_volume = (self.global_volume + up).min(1.0);
            } else if down > 0.0 {
                self.global_volume = (self.global_volume - down).max(0.0);
            }
        }
    }
}
