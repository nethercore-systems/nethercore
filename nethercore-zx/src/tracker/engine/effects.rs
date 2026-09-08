//! Effect processing at tick 0 (row start)

use nether_tracker::TrackerEffect;

use super::super::TrackerEngine;
use super::super::utils::{slide_xm_period, xm_vibrato_period};

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
        if let TrackerEffect::PastNoteAction(action) = effect {
            for voice in &mut self.channels {
                if voice.is_background && voice.parent_channel as usize == ch_idx {
                    voice.apply_dca(*action);
                }
            }
            return;
        }
        let channel = &mut self.channels[ch_idx];

        // IT E/F memory includes the fine/extra-fine mode nibble; E00/F00
        // recalls the complete command, not a regular slide of that magnitude.
        if self.is_it_format
            && let TrackerEffect::PortamentoUp(value) | TrackerEffect::PortamentoDown(value) =
                effect
        {
            let up = matches!(effect, TrackerEffect::PortamentoUp(_));
            let value = if *value == 0 {
                channel.last_porta_up
            } else {
                *value as u8
            };
            channel.last_porta_up = value;
            channel.last_porta_down = value;
            if self.link_g_memory {
                channel.shared_efg_memory = value;
            }
            if value >= 0xE0 {
                let amount = (value & 15) as f32 * if value >= 0xF0 { 4.0 } else { 1.0 };
                channel.period = (channel.period + if up { -amount } else { amount }).max(1.0);
                channel.base_period = channel.period;
            } else if up {
                channel.porta_up_count = channel.porta_up_count.saturating_add(1);
            } else {
                channel.porta_down_count = channel.porta_down_count.saturating_add(1);
            }
            return;
        }

        let xm_pitch_change = !self.is_it_format && matches!(effect,
            TrackerEffect::FinePortaUp(_) | TrackerEffect::FinePortaDown(_)
            | TrackerEffect::ExtraFinePortaUp(_) | TrackerEffect::ExtraFinePortaDown(_));
        if xm_pitch_change { channel.period = channel.base_period; }
        match effect {
            TrackerEffect::None | TrackerEffect::ItExtended(_) | TrackerEffect::PastNoteAction(_) => {}
            TrackerEffect::SetNewNoteAction(action) => {
                channel.nna = (*action).min(3);
            }
            TrackerEffect::SetVolumeEnvelope(enabled) => {
                channel.volume_envelope_enabled = *enabled;
            }
            TrackerEffect::SetPitchEnvelope(enabled) => {
                // The source instrument selects pitch vs filter in the mixer.
                channel.pitch_envelope_enabled = *enabled;
                channel.filter_envelope_enabled = *enabled;
            }
            TrackerEffect::SetPanningEnvelope(enabled) => {
                channel.panning_envelope_enabled = *enabled;
            }

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
            TrackerEffect::PatternDelay(_) | TrackerEffect::FinePatternDelay(_) => {
                // Resolved across all columns/channels by row processing.
            }
            TrackerEffect::PatternLoop(count) => {
                if *count > 0 && !self.is_it_format && !channel.xm_legacy_retrigger {
                    self.xm_next_pattern_row = channel.pattern_loop_row;
                }
                if *count == 0 {
                    channel.pattern_loop_row = self.current_row;

                } else if !self.is_it_format && channel.xm_legacy_retrigger
                    && self.xm_loop_owner.is_some_and(|owner| owner != ch_idx) {
                    // Measured legacy flow serializes competing channel loops.
                } else if channel.pattern_loop_count == 0 {
                    channel.pattern_loop_count = *count;
                    if !self.is_it_format && channel.xm_legacy_retrigger { self.xm_loop_owner = Some(ch_idx); }
                } else {
                    channel.pattern_loop_count -= 1;
                    if !self.is_it_format && channel.xm_legacy_retrigger && channel.pattern_loop_count == 0 { self.xm_loop_owner = None; }
                }
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
                // IT ignores invalid Vxx; XM Gxx uses a 0..64 range and clamps.
                if !self.is_it_format || *vol <= 128 {
                    // Both converters emit the unified 0..128 command range.
                    self.global_volume = (*vol as f32 / GLOBAL_VOLUME_MAX).min(1.0);
                }
            }
            TrackerEffect::GlobalVolumeSlide { .. }
            | TrackerEffect::FineGlobalVolumeUp(_)
            | TrackerEffect::FineGlobalVolumeDown(_) => {
                let param = match *effect {
                    TrackerEffect::GlobalVolumeSlide { up, down } => (up << 4) | down,
                    TrackerEffect::FineGlobalVolumeUp(val) => (val << 4) | 15,
                    TrackerEffect::FineGlobalVolumeDown(val) => 0xF0 | val,
                    _ => unreachable!(),
                };
                if !self.is_it_format && channel.xm_legacy_retrigger {
                    if param != 0 { self.last_global_vol_slide = param; }
                    channel.global_volume_slide = self.last_global_vol_slide;
                } else if param != 0 {
                    channel.global_volume_slide = param;
                }
                let up = channel.global_volume_slide >> 4;
                let down = channel.global_volume_slide & 15;
                // IT fine slides execute on tick zero, including recalled W00.
                // XM gives the upper nibble priority and has no fine Hxy slides.
                if self.is_it_format && down == 15 && up != 0 {
                    self.global_volume = (self.global_volume + up as f32 / 128.0).min(1.0);
                } else if self.is_it_format && up == 15 && down != 0 {
                    self.global_volume = (self.global_volume - down as f32 / 128.0).max(0.0);
                } else {
                    channel.global_volume_slide_active = true;
                }
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
                channel.porta_up_count = channel.porta_up_count.saturating_add(1);
                let v = *val as u8;
                if v != 0 {
                    channel.last_porta_up = v;
                    if self.is_it_format {
                        channel.last_porta_down = v;
                    }
                    if self.link_g_memory {
                        channel.shared_efg_memory = v;
                    }
                }
            }
            TrackerEffect::PortamentoDown(val) => {
                channel.porta_down_count = channel.porta_down_count.saturating_add(1);
                let v = *val as u8;
                if v != 0 {
                    channel.last_porta_down = v;
                    if self.is_it_format {
                        channel.last_porta_up = v;
                    }
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
                channel.period = slide_xm_period(channel.period,
                    -(channel.last_fine_porta_up as f32) * 4.0, channel.xm_amiga_slides, channel.xm_source_tuning);
            }
            TrackerEffect::FinePortaDown(val) => {
                let v = (*val as u8) & 0x0F;
                if v != 0 {
                    channel.last_fine_porta_down = v;
                }
                channel.period = slide_xm_period(channel.period, channel.last_fine_porta_down as f32 * 4.0, channel.xm_amiga_slides, channel.xm_source_tuning);
            }
            TrackerEffect::ExtraFinePortaUp(val) => {
                channel.period = slide_xm_period(channel.period, -(*val as f32), channel.xm_amiga_slides, channel.xm_source_tuning);
            }
            TrackerEffect::ExtraFinePortaDown(val) => {
                channel.period = slide_xm_period(channel.period, *val as f32, channel.xm_amiga_slides, channel.xm_source_tuning);
            }
            TrackerEffect::TonePortamento(speed) => {
                channel.tone_porta_active = true;
                let v = *speed as u8;
                if *speed != 0 {
                    channel.porta_speed = *speed;
                    if self.link_g_memory {
                        channel.last_porta_up = v;
                        channel.last_porta_down = v;
                        channel.shared_efg_memory = v;
                    }
                } else if self.link_g_memory && channel.shared_efg_memory != 0 {
                    channel.porta_speed = u16::from(channel.shared_efg_memory);
                }
                // Row processing owns the mapped/transposed target period.
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
            TrackerEffect::VibratoSpeed(speed) => {
                if *speed != 0 {
                    channel.vibrato_speed = *speed;
                }
            }
            TrackerEffect::Vibrato { speed, depth } => {
                channel.vibrato_active = true;
                channel.it_fine_vibrato = false;
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
                channel.it_fine_vibrato = false;
                channel.volume_slide_active = true;
                let param = (*vol_up << 4) | *vol_down;
                if param != 0 {
                    channel.last_volume_slide = param;
                }
            }
            TrackerEffect::FineVibrato { speed, depth } => {
                channel.vibrato_active = true;
                channel.it_fine_vibrato = true;
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
                if !self.is_it_format {
                    channel.xm_tremolo_delta = super::super::utils::xm_tremolo_delta(channel.tremolo_depth, channel.tremolo_waveform, channel.tremolo_pos, channel.vibrato_pos);
                }
            }
            TrackerEffect::Tremor { ontime, offtime } => {
                channel.tremor_active = true;
                if *ontime != 0 || *offtime != 0 {
                    channel.tremor_on_ticks = *ontime;
                    channel.tremor_off_ticks = *offtime;
                }
                if note_num > 0 && note_num < 97 {
                    channel.tremor_counter = 0;
                    channel.tremor_mute = false;
                }
            }
            TrackerEffect::Arpeggio { note1, note2 } => {
                channel.arpeggio_active = true;
                let mut param = (*note1 << 4) | *note2;
                if self.is_it_format {
                    if param != 0 { channel.last_it_arpeggio = param; }
                    param = channel.last_it_arpeggio;
                }
                channel.arpeggio_note1 = param >> 4;
                channel.arpeggio_note2 = param & 15;
                channel.arpeggio_tick = 0;
                if !self.is_it_format { channel.period = channel.base_period; }
            }

            // Panning Effects
            TrackerEffect::SetPanning(pan) => {
                channel.surround = false;
                channel.panning = (*pan as f32 / 64.0) * 2.0 - 1.0;
            }
            TrackerEffect::PanningLeftOnTicks => { channel.panning_left_on_ticks = true; }
            TrackerEffect::PanningSlide { left, right } => {
                channel.panning_slide_active = true;
                if self.is_it_format {
                    channel.panning_slide = (*right as i8) - (*left as i8);
                } else if *right != 0 || *left != 0 {
                    channel.panning_slide = if *right != 0 { *right as i8 } else { -(*left as i8) };
                }
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
            // FT2 note-less cells and volume-column portamento suppress 9xx memory/seek.
            TrackerEffect::SampleOffset(_) if !self.is_it_format && (channel.tone_porta_active || (!channel.xm_legacy_retrigger && !(1..=96).contains(&note_num))) => {}
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
                if self.is_it_format {
                    channel.sample_pos = full_offset as f64;
                } else if (1..=96).contains(&note_num) {
                    // 9xx is authored in source-sample coordinates, not packed PCM.
                    // Legacy empty rows remember 9xx without moving the voice.
                    // Volume-portamento rows are excluded by the guard above.
                    let relative = (channel.xm_source_tuning / 128).clamp(-128, 127) as i8;
                    let fine = (channel.xm_source_tuning - i16::from(relative) * 128) as i8;
                    let rate = nether_xm::ExtractedSample::calculate_sample_rate(fine, relative);
                    channel.sample_pos = nether_tracker::convert_loop_points(rate, full_offset, 0).0 as f64;
                    channel.xm_source_position = f64::from(full_offset) * 22050.0 / f64::from(rate);
                }
            }
            TrackerEffect::Retrigger {
                ticks,
                volume_change,
            } => {
                if self.is_it_format {
                    let param = ((*volume_change as u8) << 4) | *ticks;
                    if param != 0 { channel.last_it_retrigger = param; }
                    if channel.last_it_retrigger == 0 { return; }
                    channel.retrigger_tick = (channel.last_it_retrigger & 15).max(1);
                    channel.retrigger_mode = channel.last_it_retrigger >> 4;
                    channel.retrigger_volume = match channel.retrigger_mode {
                        1..=5 => -(1i8 << (channel.retrigger_mode - 1)),
                        9..=13 => 1i8 << (channel.retrigger_mode - 9),
                        _ => 0,
                    };
                    if (1..=120).contains(&note_num) { channel.it_retrigger_count = channel.retrigger_tick; }
                    return;
                }
                channel.retrigger_tick = if !self.is_it_format && channel.xm_legacy_retrigger { (*ticks).max(1) } else { *ticks };
                if !self.is_it_format && !channel.xm_legacy_retrigger && *ticks == 0 {
                    channel.sample_pos = 0.0;
                    channel.xm_source_position = 0.0;
                    channel.xm_loop_stopped = false;
                    channel.volume_envelope_pos = 0;
                    channel.panning_envelope_pos = 0;
                    channel.panning_envelope_frozen = false;
                    channel.envelope_started = 0;
                }
                if !self.is_it_format { channel.retrigger_mode = 0; }
                channel.retrigger_volume = *volume_change;
            }
            TrackerEffect::NoteCut(tick) => {
                channel.note_cut_tick = *tick;
                if *tick == 0 && !self.is_it_format {
                    channel.volume = 0.0;
                }
            }
            TrackerEffect::NoteDelay(tick) => {
                channel.note_delay_tick = *tick;
                channel.delayed_note = note_num;
            }
            TrackerEffect::SetFinetune(val) => {
                if self.is_it_format { channel.finetune = *val; }
                // XM E5x is resolved at note initialization before baked-tuning compensation.
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
                // FT2 Lxx gates pan position on the *volume* sustain flag.
                if self.is_it_format || channel.volume_envelope_sustain_loop.is_some() {
                    channel.panning_envelope_pos = *pos as u16;
                    channel.pitch_envelope_pos = *pos as u16;
                }
            }
            TrackerEffect::KeyOffAt(tick) => { channel.key_off_tick = *tick; }
            TrackerEffect::KeyOff => {
                channel.key_off = true;
                channel.sample_sustain_released = true;
                if !self.is_it_format && !channel.volume_envelope_enabled {
                    channel.volume = 0.0;
                }
            }
            TrackerEffect::SetGlissando(enabled) => {
                channel.glissando = *enabled;
            }

            // Sound Control Effects (IT S9x)
            TrackerEffect::SetSurround(enabled) => {
                channel.surround = *enabled;
                if *enabled {
                    channel.panning = 0.0;
                }
            }
            TrackerEffect::SetSampleReverse(reversed) => {
                // S9F = play backwards, S9E = play forwards
                channel.sample_direction = if *reversed { -1 } else { 1 };
            }

            TrackerEffect::MultiRetrigNote { ticks, volume } => {
                let (ticks, volume) = if self.is_it_format { (*ticks, *volume) } else {
                    let ticks = if *ticks == 0 { channel.xm_retrigger_memory & 15 } else { *ticks };
                    let volume = if *volume == 0 { channel.xm_retrigger_memory >> 4 } else { *volume };
                    channel.xm_retrigger_memory = (volume << 4) | ticks;
                    channel.xm_multi_retrigger_active = true;
                    (ticks.max(1), volume)
                };
                channel.retrigger_tick = ticks;
                channel.retrigger_mode = volume;
                channel.retrigger_volume = match volume {
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
                if !self.is_it_format {
                    if (1..=96).contains(&note_num) { channel.xm_retrigger_count = if channel.xm_legacy_retrigger {0}else{1}; }
                    channel.advance_xm_retrigger();
                }
            }
        }
        if self.is_it_format && channel.volume_slide_active {
            let up = channel.last_volume_slide >> 4;
            let down = channel.last_volume_slide & 15;
            if down == 15 && up != 0 {
                channel.volume = (channel.volume + f32::from(up) / 64.0).min(1.0);
                channel.volume_slide_active = false;
            } else if up == 15 && down != 0 {
                channel.volume = (channel.volume - f32::from(down) / 64.0).max(0.0);
                channel.volume_slide_active = false;
            }
        }
        if xm_pitch_change { channel.base_period = channel.period; }
        if !self.is_it_format && channel.vibrato_active {
            channel.period = xm_vibrato_period(channel.base_period, channel.vibrato_depth,
                channel.vibrato_waveform, channel.vibrato_pos, channel.xm_amiga_slides, channel.xm_source_tuning);
        }
    }
}
