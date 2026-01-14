//! Effect processing at tick 0 (row start)

use nether_tracker::TrackerEffect;

use super::super::utils::note_to_period;
use super::super::TrackerEngine;
use super::{CHANNEL_VOLUME_MAX, GLOBAL_VOLUME_MAX};

impl TrackerEngine {
    /// Process unified TrackerEffect at tick 0 (row start)
    pub(super) fn process_unified_effect_tick0(
        &mut self,
        ch_idx: usize,
        effect: &TrackerEffect,
        note_num: u8,
        _note_instrument: u8,
    ) {
        let channel = &mut self.channels[ch_idx];

        match effect {
            TrackerEffect::None => {}

            // Speed and Tempo (handled by caller via return value in legacy code)
            TrackerEffect::SetSpeed(_) | TrackerEffect::SetTempo(_) => {
                // These modify TrackerState, handled in FFI layer
            }

            TrackerEffect::TempoSlideUp(amount) => {
                self.tempo_slide = *amount as i8;
            }

            TrackerEffect::TempoSlideDown(amount) => {
                self.tempo_slide = -(*amount as i8);
            }

            // Pattern Flow (handled by caller)
            TrackerEffect::PositionJump(_) | TrackerEffect::PatternBreak(_) => {}
            TrackerEffect::PatternDelay(rows) => {
                if *rows > 0 && self.pattern_delay == 0 {
                    self.pattern_delay = *rows;
                }
            }
            TrackerEffect::PatternLoop(count) => {
                if *count == 0 {
                    channel.pattern_loop_row = self.current_row;
                } else if channel.pattern_loop_count == 0 {
                    channel.pattern_loop_count = *count;
                } else {
                    channel.pattern_loop_count -= 1;
                }
            }

            TrackerEffect::FinePatternDelay(ticks) => {
                self.fine_pattern_delay = *ticks;
            }

            TrackerEffect::HighSampleOffset(value) => {
                channel.sample_offset_high = *value;
            }

            // Volume Effects
            TrackerEffect::SetVolume(vol) => {
                channel.volume = ((*vol).min(64) as f32) / CHANNEL_VOLUME_MAX;
            }
            TrackerEffect::VolumeSlide { up, down } => {
                channel.volume_slide_active = true;
                let param = (*up << 4) | *down;
                if param != 0 {
                    channel.last_volume_slide = param;
                }
            }
            TrackerEffect::FineVolumeUp(val) => {
                channel.volume = (channel.volume + *val as f32 / CHANNEL_VOLUME_MAX).min(1.0);
            }
            TrackerEffect::FineVolumeDown(val) => {
                channel.volume = (channel.volume - *val as f32 / CHANNEL_VOLUME_MAX).max(0.0);
            }
            TrackerEffect::SetGlobalVolume(vol) => {
                self.global_volume = ((*vol).min(128) as f32) / GLOBAL_VOLUME_MAX;
            }
            TrackerEffect::GlobalVolumeSlide { up, down } => {
                let param = (*up << 4) | *down;
                if param != 0 {
                    self.last_global_vol_slide = param;
                }
            }
            TrackerEffect::FineGlobalVolumeUp(val) => {
                self.global_volume =
                    (self.global_volume + *val as f32 / CHANNEL_VOLUME_MAX).min(1.0);
            }
            TrackerEffect::FineGlobalVolumeDown(val) => {
                self.global_volume =
                    (self.global_volume - *val as f32 / CHANNEL_VOLUME_MAX).max(0.0);
            }
            TrackerEffect::SetChannelVolume(vol) => {
                channel.channel_volume = (*vol).min(64);
            }
            TrackerEffect::ChannelVolumeSlide { up, down } => {
                channel.channel_volume_slide_active = true;
                let param = (*up << 4) | *down;
                if param != 0 {
                    channel.channel_volume_slide = if *up > 0 { *up as i8 } else { -(*down as i8) };
                }
            }
            TrackerEffect::FineChannelVolumeUp(val) => {
                channel.channel_volume = channel.channel_volume.saturating_add(*val).min(64);
            }
            TrackerEffect::FineChannelVolumeDown(val) => {
                channel.channel_volume = channel.channel_volume.saturating_sub(*val);
            }

            // Pitch Effects
            TrackerEffect::PortamentoUp(val) => {
                channel.porta_up_active = true;
                let v = *val as u8;
                if v != 0 {
                    channel.last_porta_up = v;
                    if self.link_g_memory {
                        channel.shared_efg_memory = v;
                    }
                }
            }
            TrackerEffect::PortamentoDown(val) => {
                channel.porta_down_active = true;
                let v = *val as u8;
                if v != 0 {
                    channel.last_porta_down = v;
                    if self.link_g_memory {
                        channel.shared_efg_memory = v;
                    }
                }
            }
            TrackerEffect::FinePortaUp(val) => {
                let v = (*val as u8) & 0x0F;
                if v != 0 {
                    channel.last_fine_porta_up = v;
                }
                channel.period =
                    (channel.period - channel.last_fine_porta_up as f32 * 4.0).max(1.0);
            }
            TrackerEffect::FinePortaDown(val) => {
                let v = (*val as u8) & 0x0F;
                if v != 0 {
                    channel.last_fine_porta_down = v;
                }
                channel.period += channel.last_fine_porta_down as f32 * 4.0;
            }
            TrackerEffect::ExtraFinePortaUp(val) => {
                channel.period = (channel.period - *val as f32).max(1.0);
            }
            TrackerEffect::ExtraFinePortaDown(val) => {
                channel.period += *val as f32;
            }
            TrackerEffect::TonePortamento(speed) => {
                channel.tone_porta_active = true;
                let v = *speed as u8;
                if v != 0 {
                    channel.porta_speed = v;
                    if self.link_g_memory {
                        channel.shared_efg_memory = v;
                    }
                } else if self.link_g_memory && channel.shared_efg_memory != 0 {
                    channel.porta_speed = channel.shared_efg_memory;
                }
                if note_num > 0 && note_num <= 96 {
                    channel.target_period = note_to_period(note_num, channel.finetune);
                }
            }
            TrackerEffect::TonePortaVolSlide {
                porta: _,
                vol_up,
                vol_down,
            } => {
                channel.tone_porta_active = true;
                channel.volume_slide_active = true;
                let param = (*vol_up << 4) | *vol_down;
                if param != 0 {
                    channel.last_volume_slide = param;
                }
            }

            // Modulation Effects
            TrackerEffect::Vibrato { speed, depth } => {
                channel.vibrato_active = true;
                let param = (*speed << 4) | *depth;
                if param != 0 {
                    channel.last_vibrato = param;
                }
                let p = channel.last_vibrato;
                if p >> 4 != 0 {
                    channel.vibrato_speed = p >> 4;
                }
                if p & 0x0F != 0 {
                    channel.vibrato_depth = p & 0x0F;
                }
            }
            TrackerEffect::VibratoVolSlide {
                vib_speed: _,
                vib_depth: _,
                vol_up,
                vol_down,
            } => {
                channel.vibrato_active = true;
                channel.volume_slide_active = true;
                let param = (*vol_up << 4) | *vol_down;
                if param != 0 {
                    channel.last_volume_slide = param;
                }
            }
            TrackerEffect::FineVibrato { speed, depth } => {
                channel.vibrato_active = true;
                if *speed != 0 {
                    channel.vibrato_speed = *speed;
                }
                if *depth != 0 {
                    channel.vibrato_depth = (*depth).min(15);
                }
            }
            TrackerEffect::Tremolo { speed, depth } => {
                channel.tremolo_active = true;
                let param = (*speed << 4) | *depth;
                if param != 0 {
                    channel.last_tremolo = param;
                }
                let p = channel.last_tremolo;
                if p >> 4 != 0 {
                    channel.tremolo_speed = p >> 4;
                }
                if p & 0x0F != 0 {
                    channel.tremolo_depth = p & 0x0F;
                }
            }
            TrackerEffect::Tremor { ontime, offtime } => {
                channel.tremor_active = true;
                if *ontime != 0 || *offtime != 0 {
                    channel.tremor_on_ticks = *ontime;
                    channel.tremor_off_ticks = *offtime;
                }
                channel.tremor_counter = 0;
                channel.tremor_mute = false;
            }
            TrackerEffect::Arpeggio { note1, note2 } => {
                channel.arpeggio_active = true;
                channel.arpeggio_note1 = *note1;
                channel.arpeggio_note2 = *note2;
                channel.arpeggio_tick = 0;
            }

            // Panning Effects
            TrackerEffect::SetPanning(pan) => {
                channel.panning = (*pan as f32 / 64.0) * 2.0 - 1.0;
            }
            TrackerEffect::PanningSlide { left, right } => {
                channel.panning_slide_active = true;
                channel.panning_slide = (*right as i8) - (*left as i8);
            }
            TrackerEffect::FinePanningRight(amount) => {
                channel.panning = (channel.panning + *amount as f32 / 64.0).clamp(-1.0, 1.0);
            }
            TrackerEffect::FinePanningLeft(amount) => {
                channel.panning = (channel.panning - *amount as f32 / 64.0).clamp(-1.0, 1.0);
            }
            TrackerEffect::Panbrello { speed, depth } => {
                channel.panbrello_active = true;
                if *speed != 0 {
                    channel.panbrello_speed = *speed;
                }
                if *depth != 0 {
                    channel.panbrello_depth = *depth;
                }
            }

            // Sample Effects
            TrackerEffect::SampleOffset(offset) => {
                let high = (*offset >> 16) as u8;
                let low = ((*offset >> 8) & 0xFF) as u8;
                if low != 0 {
                    channel.last_sample_offset = low;
                }
                if high != 0 {
                    channel.sample_offset_high = high;
                }
                let full_offset = ((channel.sample_offset_high as u32) << 16)
                    | ((channel.last_sample_offset as u32) << 8);
                channel.sample_pos = full_offset as f64;
            }
            TrackerEffect::Retrigger {
                ticks,
                volume_change,
            } => {
                channel.retrigger_tick = *ticks;
                channel.retrigger_volume = *volume_change;
            }
            TrackerEffect::NoteCut(tick) => {
                channel.note_cut_tick = *tick;
            }
            TrackerEffect::NoteDelay(tick) => {
                channel.note_delay_tick = *tick;
                channel.delayed_note = note_num;
            }
            TrackerEffect::SetFinetune(val) => {
                channel.finetune = *val;
            }

            // Filter Effects (IT only)
            TrackerEffect::SetFilterCutoff(cutoff) => {
                channel.filter_cutoff = *cutoff as f32 / 127.0;
                channel.filter_dirty = true;
            }
            TrackerEffect::SetFilterResonance(res) => {
                channel.filter_resonance = *res as f32 / 127.0;
                channel.filter_dirty = true;
            }

            // Waveform Control
            TrackerEffect::VibratoWaveform(wf) => {
                channel.vibrato_waveform = *wf & 0x07;
            }
            TrackerEffect::TremoloWaveform(wf) => {
                channel.tremolo_waveform = *wf & 0x07;
            }
            TrackerEffect::PanbrelloWaveform(wf) => {
                channel.panbrello_waveform = *wf & 0x07;
            }

            // Other Effects
            TrackerEffect::SetEnvelopePosition(pos) => {
                channel.volume_envelope_pos = *pos as u16;
            }
            TrackerEffect::KeyOff => {
                channel.key_off = true;
            }
            TrackerEffect::SetGlissando(enabled) => {
                channel.glissando = *enabled;
            }

            // Sound Control Effects (IT S9x)
            TrackerEffect::SetSurround(enabled) => {
                channel.surround = *enabled;
            }
            TrackerEffect::SetSampleReverse(reversed) => {
                // S9F = play backwards, S9E = play forwards
                channel.sample_direction = if *reversed { -1 } else { 1 };
            }

            TrackerEffect::MultiRetrigNote { ticks, volume } => {
                channel.retrigger_tick = *ticks;
                channel.retrigger_mode = *volume;
                channel.retrigger_volume = match *volume {
                    1 => -1,
                    2 => -2,
                    3 => -4,
                    4 => -8,
                    5 => -16,
                    9 => 1,
                    10 => 2,
                    11 => 4,
                    12 => 8,
                    13 => 16,
                    _ => 0,
                };
            }
        }
    }
}
