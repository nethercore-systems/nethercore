//! Row and note processing at tick 0

use nether_tracker::{TrackerEffect, TrackerNote};

use super::super::utils::{note_to_period, xm_sample_finetune};
use super::super::{FADE_IN_SAMPLES, TrackerEngine, raw_tracker_handle};
use super::VOLUME_FADEOUT_MAX;
use crate::audio::Sound;

impl TrackerEngine {
    fn resolve_it_effect(&mut self, ch_idx: usize, effect: TrackerEffect) -> TrackerEffect {
        if self.is_it_format {
            // D/K/L share the complete main-column slide byte, including fine mode.
            let slide = match effect {
                TrackerEffect::VolumeSlide { up, down }
                | TrackerEffect::TonePortaVolSlide { vol_up: up, vol_down: down, .. }
                | TrackerEffect::VibratoVolSlide { vol_up: up, vol_down: down, .. } => Some((up << 4) | down),
                TrackerEffect::FineVolumeUp(value) => Some((value << 4) | 15),
                TrackerEffect::FineVolumeDown(value) => Some(0xf0 | value),
                _ => None,
            };
            if let Some(value) = slide {
                if value != 0 { self.channels[ch_idx].last_volume_slide = value; }
                let value = self.channels[ch_idx].last_volume_slide;
                return match effect {
                    TrackerEffect::TonePortaVolSlide { porta, .. } => TrackerEffect::TonePortaVolSlide { porta, vol_up: value >> 4, vol_down: value & 15 },
                    TrackerEffect::VibratoVolSlide { vib_speed, vib_depth, .. } => TrackerEffect::VibratoVolSlide { vib_speed, vib_depth, vol_up: value >> 4, vol_down: value & 15 },
                    _ => TrackerEffect::VolumeSlide { up: value >> 4, down: value & 15 },
                };
            }
            let tempo = match effect {
                TrackerEffect::SetTempo(value) | TrackerEffect::TempoSlideDown(value) => Some(value),
                TrackerEffect::TempoSlideUp(value) => Some(0x10 | value),
                _ => None,
            };
            if let Some(value) = tempo {
                let memory = &mut self.channels[ch_idx].last_it_tempo;
                if value != 0 { *memory = value; }
                // Resolve before row timing controls so T00 can restore a tempo,
                // not just its slide magnitude. T10 remains a zero upward slide.
                return match *memory {
                    0..=15 => TrackerEffect::TempoSlideDown(*memory),
                    16..=31 => TrackerEffect::TempoSlideUp(*memory & 15),
                    _ => TrackerEffect::SetTempo(*memory),
                };
            }
        }
        let TrackerEffect::ItExtended(param) = effect else {
            return effect;
        };
        if !self.is_it_format {
            return TrackerEffect::None;
        }
        let param = if param == 0 {
            self.channels[ch_idx].last_it_extended
        } else {
            self.channels[ch_idx].last_it_extended = param;
            param
        };
        TrackerEffect::from_it_extended(param)
    }

    fn resolve_it_row_note(&mut self, handle: u32, ch_idx: usize, mut note: TrackerNote) -> TrackerNote {
        let empty_mapping = note.has_note() && note.has_instrument()
            && self.modules.get(raw_tracker_handle(handle) as usize).and_then(|m| m.as_ref())
                .filter(|m| m.module.format.contains(nether_tracker::FormatFlags::IS_IT_FORMAT | nether_tracker::FormatFlags::INSTRUMENTS))
                .and_then(|m| m.module.instruments.get(note.instrument as usize - 1))
                .and_then(|i| i.note_sample_table.get(note.note as usize - 1))
                .is_some_and(|(_, sample)| *sample == 0);
        if empty_mapping {
            // IT ignores the entire cell before effect memory and timing, but
            // retains the pending instrument for the next musical note.
            self.channels[ch_idx].last_it_instrument = note.instrument;
            return TrackerNote::default();
        }
        note.effect = self.resolve_it_effect(ch_idx, note.effect);
        note
    }

    /// Internal version that accesses module by handle to avoid borrow issues
    pub(crate) fn process_row_tick0_internal(&mut self, handle: u32, sounds: &[Option<Sound>]) {
        // Get module data - need to access by index to work around borrow checker
        let raw_handle = raw_tracker_handle(handle);
        let (num_channels, pattern_info, is_it, old_effects, link_g, xm_amiga_slides) = {
            let loaded = match self
                .modules
                .get(raw_handle as usize)
                .and_then(|m| m.as_ref())
            {
                Some(m) => m,
                None => return,
            };
            let pattern = match loaded.module.pattern_at_order(self.current_order) {
                Some(p) => p,
                None => return,
            };

            // Check format flags (affects effect processing)
            let is_it = loaded
                .module
                .format
                .contains(nether_tracker::FormatFlags::IS_IT_FORMAT);
            let old_effects = loaded
                .module
                .format
                .contains(nether_tracker::FormatFlags::OLD_EFFECTS);
            // The source bit means Compatible Gxx: enabling it SEPARATES memory.
            let link_g = is_it
                && !loaded
                    .module
                    .format
                    .contains(nether_tracker::FormatFlags::LINK_G_MEMORY);

            // Collect note data for this row
            let mut notes = Vec::new();
            for ch_idx in 0..loaded.module.num_channels as usize {
                if let Some(note) = pattern.get_note(self.current_row, ch_idx as u8) {
                    notes.push((ch_idx, *note));
                }
            }
            (
                loaded.module.num_channels,
                notes,
                is_it,
                old_effects,
                link_g,
                !is_it && !loaded.module.format.contains(nether_tracker::FormatFlags::LINEAR_SLIDES),
            )
        };

        // Store format flags for use in effect processing
        self.is_it_format = is_it;
        self.old_effects_mode = old_effects;
        self.link_g_memory = link_g;

        let mut pattern_info = pattern_info;
        for (ch_idx, note) in &mut pattern_info {
            *note = self.resolve_it_row_note(handle, *ch_idx, *note);
        }
        self.resolved_row_notes = pattern_info.clone();
        self.resolved_row_key = Some((raw_tracker_handle(handle), self.current_order, self.current_row));

        // Reset tempo slide (only active during the row it appears on)
        self.tempo_slide = 0;

        // Reset per-row effect state for all channels before processing
        // XM/IT effects only apply during the row they appear on
        for ch_idx in 0..num_channels as usize {
            if self.channels[ch_idx].arpeggio_active || self.channels[ch_idx].vibrato_active || (!is_it && self.channels[ch_idx].auto_vibrato_depth > 0) {
                self.channels[ch_idx].period = self.channels[ch_idx].base_period;
            }
            self.channels[ch_idx].reset_row_effects();
            self.channels[ch_idx].xm_amiga_slides = xm_amiga_slides;
        }

        // IT uses the first SEx, XM the last EEx, including a zero parameter.
        // Tick delays add across channels; retain them for all row repetitions.
        let mut row_delay = None;
        self.fine_pattern_delay = 0;
        for (ch_idx, note) in pattern_info {
            for effect in [note.volume_effect, note.effect] {
                match effect {
                    TrackerEffect::PatternDelay(rows) if row_delay.is_none() || !is_it => {
                        row_delay = Some(rows);
                    }
                    TrackerEffect::FinePatternDelay(ticks) => {
                        self.fine_pattern_delay += u16::from(ticks);
                    }
                    _ => {}
                }
            }
            self.process_note_internal(ch_idx, &note, handle, sounds);
        }
        self.pattern_delay = row_delay.unwrap_or(0);
        if is_it {
            for channel in &mut self.channels[..num_channels as usize] {
                channel.advance_it_modulation(true, old_effects);
            }
        }
        self.advance_xm_auto_vibrato();
    }

    /// IT fine E/F slides run again on delayed-row first ticks, unlike volume-column slides.
    pub(super) fn repeat_fine_pitch_slides(&mut self, handle: u32) {
        if !self.is_it_format {
            return;
        }
        for (ch_idx, note) in self.row_notes(raw_tracker_handle(handle), self.current_order, self.current_row) {
            if let TrackerEffect::PortamentoUp(value) | TrackerEffect::PortamentoDown(value) =
                note.effect
            {
                let value = if value == 0 {
                    self.channels[ch_idx].last_porta_up as u16
                } else {
                    value
                };
                if value >= 0xe0 {
                    self.process_unified_effect_tick0(ch_idx, &note.effect, 0, 0);
                }
            }
        }
    }

    fn process_volume_column_tick0(&mut self, ch_idx: usize, effect: &TrackerEffect, note: u8, instrument: u8) {
        if !self.is_it_format {
            if *effect == TrackerEffect::PanningLeftOnTicks && self.channels[ch_idx].xm_legacy_retrigger {
                self.channels[ch_idx].xm_volume_column_pan_slide = self.channels[ch_idx].panning_slide;
                return;
            }
            if let TrackerEffect::PanningSlide { left, right } = *effect {
                let slide = right as i8 - left as i8;
                self.channels[ch_idx].xm_volume_column_pan_slide = slide;
                if self.channels[ch_idx].xm_legacy_retrigger {
                    if slide != 0 { self.channels[ch_idx].panning_slide = slide; }
                    self.channels[ch_idx].xm_volume_column_pan_slide = self.channels[ch_idx].panning_slide;
                }
                return;
            }
            if let TrackerEffect::VolumeSlide { up, down } = *effect {
                self.channels[ch_idx].xm_volume_column_slide = up as i8 - down as i8;
                return;
            }
        }
        self.process_unified_effect_tick0(ch_idx, effect, note, instrument);
    }

    /// Internal note processing; returns whether this cell reset voice envelopes.
    pub(super) fn process_note_internal(
        &mut self,
        ch_idx: usize,
        note: &TrackerNote,
        handle: u32,
        _sounds: &[Option<Sound>],
    ) -> bool {
        use super::super::channels::NNA_CUT;
        use super::CHANNEL_VOLUME_MAX;

        let mut envelopes_reset = false;
        let raw_handle = raw_tracker_handle(handle);
        let format = self
            .modules
            .get(raw_handle as usize)
            .and_then(|m| m.as_ref())
            .map_or(nether_tracker::FormatFlags::empty(), |m| m.module.format);
        let is_it = format.contains(nether_tracker::FormatFlags::IS_IT_FORMAT);
        self.channels[ch_idx].xm_short_restart_attack = !is_it
            && (!format.contains(nether_tracker::FormatFlags::XM_FT2_MIX)
                || format.contains(nether_tracker::FormatFlags::XM_LEGACY_RETRIGGER));
        self.channels[ch_idx].xm_legacy_retrigger = !is_it
            && format.contains(nether_tracker::FormatFlags::XM_LEGACY_RETRIGGER);
        // Defer the whole delayed cell. IT SD0 is SD1; unlike XM, IT's
        // volume-column effects stay with the delayed trigger.
        if is_it || note.has_note() || note.note == 0 {
            if let TrackerEffect::NoteDelay(delay) = note.effect {
                let delay = if is_it { delay.max(1) } else { delay };
                if delay != 0 {
                    if !is_it && (matches!(note.volume_effect, TrackerEffect::VolumeSlide { .. })
                        || (!self.channels[ch_idx].xm_legacy_retrigger && matches!(note.volume_effect, TrackerEffect::PanningSlide { .. } | TrackerEffect::PanningLeftOnTicks))) {
                        self.process_volume_column_tick0(ch_idx, &note.volume_effect, 0, 0);
                    }
                    self.channels[ch_idx].note_delay_tick = delay;
                    self.channels[ch_idx].delayed_xm_note = Some((*note, handle));
                    return false;
                }
            }
        }
        let sample_mode = is_it && !format.contains(nether_tracker::FormatFlags::INSTRUMENTS);
        let has_active_note =
            self.channels[ch_idx].note_on && self.channels[ch_idx].sample_handle != 0;
        // A rate-stopped voice retains note identity, but has no sample to crossfade.
        let starts_xm_attack = !is_it && (!has_active_note || self.channels[ch_idx].xm_loop_stopped);
        let valid_instrument = note.instrument.checked_sub(1).is_some_and(|index| {
            self.modules
                .get(raw_handle as usize)
                .and_then(|m| m.as_ref())
                .is_some_and(|loaded| (index as usize) < if sample_mode {
                    loaded.module.samples.len()
                } else {
                    loaded.module.instruments.len()
                })
        });
        let mut note = *note;
        let mut xm_pending_instrument = false;
        let xm_standalone_selection = !is_it
            && note.note == 0
            && note.has_instrument()
            && note.effect != TrackerEffect::KeyOff;
        if !is_it {
            if note.has_instrument() {
                self.channels[ch_idx].last_xm_instrument = note.instrument;
                if note.note == 0 && self.channels[ch_idx].instrument != 0 {
                    // XM remembers a standalone selection, but reloads the active
                    // voice now. A later instrument-less note changes sample only.
                    note.instrument = self.channels[ch_idx].instrument;
                }
            } else if note.has_note() && self.channels[ch_idx].last_xm_instrument != 0
                && self.channels[ch_idx].last_xm_instrument != self.channels[ch_idx].instrument {
                note.instrument = self.channels[ch_idx].last_xm_instrument;
                xm_pending_instrument = true;
            }
        }
        if is_it && !sample_mode {
            if note.has_instrument() {
                self.channels[ch_idx].last_it_instrument = note.instrument;
            } else if note.has_note()
                && self.channels[ch_idx].last_it_instrument != self.channels[ch_idx].instrument
            {
                note.instrument = self.channels[ch_idx].last_it_instrument;
            }
        }
        if is_it && !sample_mode && note.has_note() {
            let instrument = if note.has_instrument() { note.instrument } else { self.channels[ch_idx].instrument };
            let empty_slot = self.modules.get(raw_handle as usize)
                .and_then(|m| m.as_ref())
                .and_then(|m| instrument.checked_sub(1).and_then(|i| m.module.instruments.get(i as usize)))
                .and_then(|i| i.note_sample_table.get((note.note - 1) as usize))
                .is_some_and(|(_, sample)| *sample == 0);
            if empty_slot {
                // Ignore the note/instrument, not co-located volume and effects.
                note.note = 0;
                note.instrument = 0;
            }
        }
        if is_it && !sample_mode {
            let selected = self.channels[ch_idx].last_it_instrument;
            let invalid = selected != 0 && self.modules.get(raw_handle as usize)
                .and_then(|m| m.as_ref())
                .is_some_and(|m| selected as usize > m.module.instruments.len());
            if (note.note == 0 || note.has_note()) && invalid {
                // Invalid selections persist across instrument-less notes, but
                // do not replace the active voice or discard volume/effects.
                note.note = 0;
                note.instrument = 0;
            }
        }
        // IT instrument-only changes retrigger; the same instrument needs a stopped voice.
        let restarts_instrument_note = is_it
            && note.note == 0
            && note.has_instrument()
            && valid_instrument
            && (!has_active_note || note.instrument != self.channels[ch_idx].instrument)
            && self.channels[ch_idx].current_note > 0;
        let incoming_instrument = if note.has_instrument() {
            note.instrument
        } else {
            self.channels[ch_idx].instrument
        };
        let xm_mapped = !is_it && self.modules.get(raw_handle as usize)
            .and_then(|m| m.as_ref()).is_some_and(|m| !m.module.samples.is_empty());
        let it_mapped_sample = if (is_it || xm_mapped) && !sample_mode {
            self.modules
                .get(raw_handle as usize)
                .and_then(|m| m.as_ref())
                .and_then(|loaded| {
                    let original_note = if note.has_note() {
                        note.note
                    } else if restarts_instrument_note {
                        self.channels[ch_idx].current_note
                    } else {
                        return None;
                    };
                    let instrument = loaded
                        .module
                        .instruments
                        .get(incoming_instrument.checked_sub(1)? as usize)?;
                    let (mapped_note, sample_number) = *instrument
                        .note_sample_table
                        .get(original_note.checked_sub(1)? as usize)?;
                    let sample_index = sample_number.checked_sub(1)? as usize;
                    Some((
                        mapped_note.saturating_add(1),
                        loaded.sound_handles.get(sample_index).copied().unwrap_or(0),
                        loaded.module.samples.get(sample_index)?.clone(),
                        sample_index,
                    ))
                })
        } else {
            None
        };
        let tone_porta_note_continues_current = note.has_note()
            && has_active_note
            && [note.volume_effect, note.effect].iter().any(|effect| {
                matches!(
                    effect,
                    TrackerEffect::TonePortamento(_) | TrackerEffect::TonePortaVolSlide { .. }
                )
            });

        if tone_porta_note_continues_current {
            let finetune = self.channels[ch_idx].finetune;
            let relative_note = if sample_mode || (is_it && it_mapped_sample.is_some()) {
                -12
            } else {
                self.modules
                    .get(raw_handle as usize)
                    .and_then(|m| m.as_ref())
                    .and_then(|m| {
                        m.module
                            .instruments
                            .get(self.channels[ch_idx].instrument.saturating_sub(1) as usize)
                    })
                    .map_or(0, |instr| instr.sample_relative_note)
            };
            let pitch_note = it_mapped_sample
                .as_ref()
                .map(|(mapped_note, _, _, _)| *mapped_note)
                .unwrap_or(note.note);
            let effective_note = (pitch_note as i16 + relative_note as i16).clamp(1, 96) as u8;
            if is_it {
                self.channels[ch_idx].current_note = note.note;
            } else {
                self.channels[ch_idx].xm_retrigger_note = effective_note;
            }
            self.channels[ch_idx].xm_porta_target_reached = false;
            self.channels[ch_idx].target_period = note_to_period(effective_note, finetune)
                - if is_it {0.0} else {f32::from(self.channels[ch_idx].xm_finetune_delta) / 2.0};
        }

        // Compatible Gxx also retains the active sample on command-only cells.
        // Retain the active sample selection for subsequent instrument-less notes.
        let retain_sample = sample_mode && has_active_note
            && format.contains(nether_tracker::FormatFlags::LINK_G_MEMORY)
            && [note.volume_effect, note.effect].iter().any(|effect| matches!(
                effect, TrackerEffect::TonePortamento(_) | TrackerEffect::TonePortaVolSlide { .. }
            ));

        let sample_special = sample_mode
            && matches!(note.note, TrackerNote::NOTE_OFF | TrackerNote::NOTE_CUT | TrackerNote::NOTE_FADE)
            && !format.contains(nether_tracker::FormatFlags::OLD_EFFECTS);
        if sample_special && note.has_instrument() {
            if let Some(sample) = self.modules.get(raw_handle as usize)
                .and_then(|m| m.as_ref())
                .and_then(|m| m.module.samples.get(note.instrument as usize - 1))
            {
                self.channels[ch_idx].volume = sample.default_volume as f32 / CHANNEL_VOLUME_MAX;
            }
        }

        // Handle instrument change
        // IT header bit 0x20 is Compatible Gxx (legacy name LINK_G_MEMORY).
        // Without it, sample-mode portamento switches samples without retriggering.
        let porta_sample_switch = tone_porta_note_continues_current
            && sample_mode
            && !format.contains(nether_tracker::FormatFlags::LINK_G_MEMORY);
        if note.has_instrument() && !sample_special && (!tone_porta_note_continues_current || porta_sample_switch) {
            let instr_idx = (note.instrument - 1) as usize;
            if !retain_sample
                && !(is_it && matches!(note.note, TrackerNote::NOTE_OFF | TrackerNote::NOTE_CUT | TrackerNote::NOTE_FADE))
                && (porta_sample_switch || !(is_it && (note.has_note() || restarts_instrument_note)))
            {
                self.channels[ch_idx].instrument = note.instrument;
            }

            // Get sound handle and instrument data
            let (
                sound_handle,
                loop_start,
                loop_end,
                loop_type,
                sample_is_stereo,
                sustain_loop_start,
                sustain_loop_end,
                sustain_loop_type,
                finetune,
                default_volume,
            ) = {
                let loaded = match self
                    .modules
                    .get(raw_handle as usize)
                    .and_then(|m| m.as_ref())
                {
                    Some(m) => m,
                    None => return false,
                };
                let sound_handle = loaded.sound_handles.get(instr_idx).copied().unwrap_or(0);
                if sample_mode {
                    if let Some(sample) = loaded.module.samples.get(instr_idx) {
                        let loop_type = match sample.loop_type {
                            nether_tracker::LoopType::None => 0,
                            nether_tracker::LoopType::Forward => 1,
                            nether_tracker::LoopType::PingPong => 2,
                        };
                        let sustain_loop_type = match sample.sustain_loop_type {
                            nether_tracker::LoopType::None => 0,
                            nether_tracker::LoopType::Forward => 1,
                            nether_tracker::LoopType::PingPong => 2,
                        };
                        (
                            sound_handle,
                            sample.loop_begin,
                            sample.loop_end,
                            loop_type,
                            sample.is_stereo,
                            sample.sustain_loop_begin,
                            sample.sustain_loop_end,
                            sustain_loop_type,
                            0,
                            sample.default_volume as f32 / CHANNEL_VOLUME_MAX,
                        )
                    } else {
                        (sound_handle, 0, 0, 0, false, 0, 0, 0, 0, 1.0)
                    }
                } else if (is_it || (xm_mapped && note.note == 0)) && !loaded.module.samples.is_empty() {
                    let channel = &self.channels[ch_idx];
                    // IT special notes recall the selected sample's volume,
                    // without changing the playing instrument/sample/envelopes.
                    let special_volume = matches!(note.note, TrackerNote::NOTE_OFF | TrackerNote::NOTE_CUT | TrackerNote::NOTE_FADE)
                        .then(|| loaded.module.instruments.get(instr_idx)
                            .and_then(|i| channel.current_note.checked_sub(1).and_then(|n| i.note_sample_table.get(n as usize)))
                            .and_then(|(_, sample)| sample.checked_sub(1))
                            .and_then(|s| loaded.module.samples.get(s as usize))
                            .map(|s| s.default_volume as f32 / CHANNEL_VOLUME_MAX))
                        .flatten();
                    (
                        channel.sample_handle,
                        channel.sample_loop_start,
                        channel.sample_loop_end,
                        channel.sample_loop_type,
                        channel.sample_is_stereo,
                        channel.sample_sustain_loop_start,
                        channel.sample_sustain_loop_end,
                        channel.sample_sustain_loop_type,
                        channel.finetune,
                        if xm_mapped {
                            channel.sample_index.and_then(|index| loaded.module.samples.get(index))
                                .map_or(channel.volume, |sample| sample.default_volume as f32 / CHANNEL_VOLUME_MAX)
                        } else {
                            special_volume.unwrap_or(channel.volume)
                        },
                    )
                } else if let Some(instr) = loaded.module.instruments.get(instr_idx) {
                    // Get sample metadata from TrackerInstrument
                    let loop_type = match instr.sample_loop_type {
                        nether_tracker::LoopType::None => 0,
                        nether_tracker::LoopType::Forward => 1,
                        nether_tracker::LoopType::PingPong => 2,
                    };
                    (
                        sound_handle,
                        instr.sample_loop_start,
                        instr.sample_loop_end,
                        loop_type,
                        instr.sample_is_stereo,
                        0,
                        0,
                        0,
                        instr.sample_finetune,
                        instr
                            .sample_default_volume
                            .map_or(1.0, |volume| volume as f32 / CHANNEL_VOLUME_MAX),
                    )
                } else {
                    (sound_handle, 0, 0, 0, false, 0, 0, 0, 0, 1.0)
                }
            };

            if porta_sample_switch && self.channels[ch_idx].sample_index != Some(instr_idx) {
                // IT pitched sample swaps restart position (OpenMPT Snd_fx.cpp:3130).
                self.channels[ch_idx].sample_pos = 0.0;
                self.channels[ch_idx].xm_loop_stopped = false;
            }
            if !retain_sample {
                self.channels[ch_idx].sample_handle = sound_handle;
                self.channels[ch_idx].sample_loop_start = loop_start;
                self.channels[ch_idx].sample_loop_end = loop_end;
                self.channels[ch_idx].sample_loop_type = loop_type;
                self.channels[ch_idx].sample_is_stereo = sample_is_stereo;
                self.channels[ch_idx].sample_sustain_loop_start = sustain_loop_start;
                self.channels[ch_idx].sample_sustain_loop_end = sustain_loop_end;
                self.channels[ch_idx].sample_sustain_loop_type = sustain_loop_type;
                self.channels[ch_idx].finetune = finetune;
                if !xm_pending_instrument {
                    self.channels[ch_idx].volume = default_volume;
                }
                if sample_mode {
                    self.channels[ch_idx].sample_index = Some(instr_idx);
                    self.channels[ch_idx].instrument_global_volume = CHANNEL_VOLUME_MAX as u8;
                }
            }
        }

        // IT instrument-mode porta changes the mapped sample without retriggering.
        // Same-sample changes keep direction/loop state, but clear key-off/fade;
        // a released sustain remains released through the selector below.
        let porta_instrument_switch = tone_porta_note_continues_current
            && is_it
            && !sample_mode
            && note.has_instrument();
        if porta_instrument_switch {
            if let Some((_, sound_handle, sample, sample_index)) = it_mapped_sample.as_ref() {
                let channel = &mut self.channels[ch_idx];
                let sample_changed = channel.sample_index != Some(*sample_index);
                if channel.instrument != incoming_instrument {
                    channel.key_off = false;
                    channel.note_fade = false;
                }
                channel.instrument = incoming_instrument;
                // Compatible Gxx retains the playing sample across instrument changes.
                if sample_changed && !format.contains(nether_tracker::FormatFlags::LINK_G_MEMORY) {
                    channel.sample_handle = *sound_handle;
                    channel.sample_index = Some(*sample_index);
                    // IT sample swaps on a pitched porta cell restart sample position.
                    channel.sample_pos = 0.0;
                    channel.xm_loop_stopped = false;
                    channel.sample_loop_start = sample.loop_begin;
                    channel.sample_loop_end = sample.loop_end;
                    channel.sample_loop_type = match sample.loop_type {
                        nether_tracker::LoopType::None => 0,
                        nether_tracker::LoopType::Forward => 1,
                        nether_tracker::LoopType::PingPong => 2,
                    };
                    channel.sample_sustain_loop_start = sample.sustain_loop_begin;
                    channel.sample_sustain_loop_end = sample.sustain_loop_end;
                    channel.sample_sustain_loop_type = match sample.sustain_loop_type {
                        nether_tracker::LoopType::None => 0,
                        nether_tracker::LoopType::Forward => 1,
                        nether_tracker::LoopType::PingPong => 2,
                    };
                    channel.sample_direction = 1;
                }
            }
        }

        // IT's Old Effects Note Off can select a new envelope while the
        // existing sample keeps running. Sample sustain releases independently.
        let old_effects_off = is_it && !sample_mode && note.is_note_off()
            && note.has_instrument() && format.contains(nether_tracker::FormatFlags::OLD_EFFECTS);
        if old_effects_off {
            if let Some(instrument) = self.modules.get(raw_handle as usize)
                .and_then(|m| m.as_ref())
                .and_then(|m| m.module.instruments.get(note.instrument as usize - 1))
            {
                let channel = &mut self.channels[ch_idx];
                if channel.instrument != note.instrument {
                    channel.nna = match instrument.nna {
                        nether_tracker::NewNoteAction::Cut => 0,
                        nether_tracker::NewNoteAction::Continue => 1,
                        nether_tracker::NewNoteAction::NoteOff => 2,
                        nether_tracker::NewNoteAction::NoteFade => 3,
                    };
                }
                channel.instrument = note.instrument;
                channel.instrument_fadeout_rate = instrument.fadeout;
                let loop_ticks = |env: Option<&nether_tracker::TrackerEnvelope>| env
                    .filter(|e| e.has_loop()).and_then(|e| Some((
                        e.points.get(e.loop_begin as usize)?.0,
                        e.points.get(e.loop_end as usize)?.0)));
                let vol = instrument.volume_envelope.as_ref();
                let pan = instrument.panning_envelope.as_ref();
                let pitch = instrument.pitch_envelope.as_ref();
                channel.volume_envelope_end = vol.and_then(|e| e.points.last()).map(|p| p.0);
                channel.volume_envelope_zero_end = vol.and_then(|e| e.points.last())
                    .and_then(|&(tick, value)| (value == 0).then_some(tick));
                channel.volume_envelope_enabled = vol.is_some_and(|e| e.is_enabled());
                channel.volume_envelope_sustain_loop = vol.and_then(|e| e.sustain_ticks());
                channel.volume_envelope_loop = loop_ticks(vol);
                channel.panning_envelope_enabled = pan.is_some_and(|e| e.is_enabled());
                channel.panning_envelope_sustain_loop = pan.and_then(|e| e.sustain_ticks());
                channel.panning_envelope_loop = loop_ticks(pan);
                channel.pitch_envelope_enabled = pitch.is_some_and(|e| e.is_enabled() && !e.is_filter());
                channel.pitch_envelope_sustain_loop = pitch.and_then(|e| e.sustain_ticks());
                channel.pitch_envelope_loop = loop_ticks(pitch);
                channel.reset_filter_envelope(pitch);
                channel.apply_instrument_filter_defaults(instrument);
                let porta = [note.volume_effect, note.effect].iter().any(|e| matches!(e,
                    TrackerEffect::TonePortamento(_) | TrackerEffect::TonePortaVolSlide { .. }));
                let mapped_sample = channel.current_note.checked_sub(1)
                    .and_then(|n| instrument.note_sample_table.get(n as usize))
                    .and_then(|(_, s)| s.checked_sub(1)).map(usize::from);
                if mapped_sample != channel.sample_index {
                    let index = if porta && format.contains(nether_tracker::FormatFlags::LINK_G_MEMORY) {
                        channel.sample_index
                    } else { mapped_sample };
                    if let Some(pan) = self.modules.get(raw_handle as usize).and_then(|m| m.as_ref())
                        .and_then(|m| index.and_then(|i| m.module.samples.get(i)))
                        .and_then(|s| s.default_pan)
                    {
                        channel.panning = pan.min(64) as f32 / 32.0 - 1.0;
                        channel.surround = false;
                    }
                }
                if !porta || format.contains(nether_tracker::FormatFlags::LINK_G_MEMORY) {
                    channel.volume_envelope_pos = 0;
                    channel.panning_envelope_pos = 0;
                    channel.pitch_envelope_pos = 0;
                    channel.filter_envelope_pos = 0;
                    channel.envelope_started = 0;
                    channel.volume_fadeout = 65535;
                }
            }
        }

        // FT2 retriggers envelope/release state on an instrument-only cell,
        // but keeps the currently playing sample and its position.
        if xm_standalone_selection && has_active_note {
            let channel = &mut self.channels[ch_idx];
            channel.key_off = false;
            channel.sample_sustain_released = false;
            channel.note_fade = false;
            channel.volume_envelope_pos = 0;
            channel.panning_envelope_pos = 0;
            channel.panning_envelope_frozen = false;
            channel.pitch_envelope_pos = 0;
            channel.filter_envelope_pos = 0;
            channel.envelope_started = 0;
            channel.volume_fadeout = VOLUME_FADEOUT_MAX as u16;
            channel.auto_vibrato_sweep = 0;
            channel.auto_vibrato_pos = 0;
            envelopes_reset = true;
        }

        // Handle note
        if (note.has_note() || restarts_instrument_note) && !tone_porta_note_continues_current {
            // Get module channel count and NNA data for processing
            let (num_channels, nna_data) = {
                let loaded = match self
                    .modules
                    .get(raw_handle as usize)
                    .and_then(|m| m.as_ref())
                {
                    Some(m) => m,
                    None => return false,
                };
                let instr_idx = incoming_instrument.saturating_sub(1) as usize;
                let nna_data = if let Some(instr) = loaded.module.instruments.get(instr_idx) {
                    let nna = match instr.nna {
                        nether_tracker::NewNoteAction::Cut => 0,
                        nether_tracker::NewNoteAction::Continue => 1,
                        nether_tracker::NewNoteAction::NoteOff => 2,
                        nether_tracker::NewNoteAction::NoteFade => 3,
                    };
                    let dct = match instr.dct {
                        nether_tracker::DuplicateCheckType::Off => 0,
                        nether_tracker::DuplicateCheckType::Note => 1,
                        nether_tracker::DuplicateCheckType::Sample => 2,
                        nether_tracker::DuplicateCheckType::Instrument => 3,
                    };
                    let dca = match instr.dca {
                        nether_tracker::DuplicateCheckAction::Cut => 0,
                        nether_tracker::DuplicateCheckAction::NoteOff => 1,
                        nether_tracker::DuplicateCheckAction::NoteFade => 2,
                    };
                    Some((nna, dct, dca))
                } else {
                    None
                };
                (loaded.module.num_channels as usize, nna_data)
            };

            let retained_sample_volume = if sample_mode && !note.has_instrument() {
                self.modules
                    .get(raw_handle as usize)
                    .and_then(|m| m.as_ref())
                    .and_then(|m| {
                        m.module
                            .samples
                            .get(self.channels[ch_idx].instrument.saturating_sub(1) as usize)
                    })
                    .map(|sample| sample.default_volume as f32 / CHANNEL_VOLUME_MAX)
            } else {
                None
            };

            // Process NNA: handle the currently playing note before triggering new one
            // This is an IT feature - move the old note to a background channel based on NNA setting
            if self.is_it_format {
                // Dispose of the displaced voice using its own current action.
                let nna = self.channels[ch_idx].nna;
                self.handle_nna(ch_idx, num_channels, nna);
                self.channels[ch_idx].nna = nna_data.map(|(nna, _, _)| nna).unwrap_or(NNA_CUT);

                // Process duplicate check against background channels
                if let Some((_, dct, dca)) = nna_data {
                    let sample_handle = it_mapped_sample
                        .as_ref()
                        .map(|(_, handle, _, _)| *handle)
                        .unwrap_or(self.channels[ch_idx].sample_handle);
                    self.process_duplicate_check(
                        num_channels,
                        ch_idx,
                        dct,
                        dca,
                        note.note,
                        sample_handle,
                        it_mapped_sample.as_ref().map(|(_, _, _, index)| *index),
                        incoming_instrument,
                    );
                }
            }

            self.channels[ch_idx].instrument = incoming_instrument;
            if let Some((_, sound_handle, sample, sample_index)) = it_mapped_sample.as_ref() {
                let channel = &mut self.channels[ch_idx];
                channel.sample_handle = *sound_handle;
                channel.sample_index = Some(*sample_index);
                if (is_it || note.has_instrument()) && !xm_pending_instrument {
                    channel.volume = sample.default_volume as f32 / CHANNEL_VOLUME_MAX;
                }
                channel.sample_loop_start = sample.loop_begin;
                channel.sample_loop_end = sample.loop_end;
                channel.sample_loop_type = match sample.loop_type {
                    nether_tracker::LoopType::None => 0,
                    nether_tracker::LoopType::Forward => 1,
                    nether_tracker::LoopType::PingPong => 2,
                };
                channel.sample_is_stereo = sample.is_stereo;
                channel.sample_sustain_loop_start = sample.sustain_loop_begin;
                channel.sample_sustain_loop_end = sample.sustain_loop_end;
                channel.sample_sustain_loop_type = match sample.sustain_loop_type {
                    nether_tracker::LoopType::None => 0,
                    nether_tracker::LoopType::Forward => 1,
                    nether_tracker::LoopType::PingPong => 2,
                };
            } else if xm_mapped {
                self.channels[ch_idx].sample_handle = 0;
                self.channels[ch_idx].sample_index = None;
                self.channels[ch_idx].sample_is_stereo = false;
            }

            if let Some(volume) = retained_sample_volume {
                self.channels[ch_idx].volume = volume;
            }

            // Fetch all instrument data we need for note trigger
            let instr_data = {
                let loaded = match self
                    .modules
                    .get(raw_handle as usize)
                    .and_then(|m| m.as_ref())
                {
                    Some(m) => m,
                    None => return false,
                };
                let instr_idx = incoming_instrument.saturating_sub(1) as usize;
                if sample_mode {
                    loaded.module.samples.get(instr_idx).map(|sample| {
                        let loop_type = match sample.loop_type {
                            nether_tracker::LoopType::None => 0,
                            nether_tracker::LoopType::Forward => 1,
                            nether_tracker::LoopType::PingPong => 2,
                        };
                        let sustain_loop_type = match sample.sustain_loop_type {
                            nether_tracker::LoopType::None => 0,
                            nether_tracker::LoopType::Forward => 1,
                            nether_tracker::LoopType::PingPong => 2,
                        };
                        (
                            0,
                            sample.loop_begin,
                            sample.loop_end,
                            loop_type,
                            sample.is_stereo,
                            sample.sustain_loop_begin,
                            sample.sustain_loop_end,
                            sustain_loop_type,
                            0,
                            0,
                            0,
                            0,
                            -12,
                            0,
                            false,
                            None,
                            None,
                            false,
                            None,
                            None,
                        )
                    })
                } else if let Some(instr) = loaded.module.instruments.get(instr_idx) {
                    // Extract envelope data from TrackerEnvelope
                    let (vol_env_enabled, vol_env_sustain, vol_env_loop) =
                        if let Some(ref env) = instr.volume_envelope {
                            let enabled = env.is_enabled();
                            let sustain = env.sustain_ticks();
                            let loop_range = if env.has_loop() {
                                let start = env
                                    .points
                                    .get(env.loop_begin as usize)
                                    .map(|(tick, _)| *tick)
                                    .unwrap_or(0);
                                let end = env
                                    .points
                                    .get(env.loop_end as usize)
                                    .map(|(tick, _)| *tick)
                                    .unwrap_or(0);
                                Some((start, end))
                            } else {
                                None
                            };
                            (enabled, sustain, loop_range)
                        } else {
                            (false, None, None)
                        };
                    let (pan_env_enabled, pan_env_sustain, pan_env_loop) =
                        if let Some(ref env) = instr.panning_envelope {
                            let enabled = env.is_enabled();
                            let sustain = env.sustain_ticks();
                            let loop_range = if env.has_loop() {
                                let start = env
                                    .points
                                    .get(env.loop_begin as usize)
                                    .map(|(tick, _)| *tick)
                                    .unwrap_or(0);
                                let end = env
                                    .points
                                    .get(env.loop_end as usize)
                                    .map(|(tick, _)| *tick)
                                    .unwrap_or(0);
                                Some((start, end))
                            } else {
                                None
                            };
                            (enabled, sustain, loop_range)
                        } else {
                            (false, None, None)
                        };

                    let sample_data = if let Some((_, _, sample, _)) = it_mapped_sample.as_ref() {
                        (
                            0,
                            sample.loop_begin,
                            sample.loop_end,
                            match sample.loop_type {
                                nether_tracker::LoopType::None => 0,
                                nether_tracker::LoopType::Forward => 1,
                                nether_tracker::LoopType::PingPong => 2,
                            },
                            sample.is_stereo,
                            sample.sustain_loop_begin,
                            sample.sustain_loop_end,
                            match sample.sustain_loop_type {
                                nether_tracker::LoopType::None => 0,
                                nether_tracker::LoopType::Forward => 1,
                                nether_tracker::LoopType::PingPong => 2,
                            },
                            sample.vibrato_type,
                            sample.vibrato_depth,
                            sample.vibrato_rate,
                            sample.vibrato_speed,
                            if is_it { -12 } else { 0 },
                        )
                    } else if is_it && !loaded.module.samples.is_empty() {
                        let channel = &self.channels[ch_idx];
                        (
                            channel.finetune,
                            channel.sample_loop_start,
                            channel.sample_loop_end,
                            channel.sample_loop_type,
                            channel.sample_is_stereo,
                            channel.sample_sustain_loop_start,
                            channel.sample_sustain_loop_end,
                            channel.sample_sustain_loop_type,
                            channel.auto_vibrato_type,
                            channel.auto_vibrato_depth,
                            channel.auto_vibrato_rate,
                            channel.auto_vibrato_sweep_len,
                            0,
                        )
                    } else {
                        (
                            instr.sample_finetune,
                            instr.sample_loop_start,
                            instr.sample_loop_end,
                            match instr.sample_loop_type {
                                nether_tracker::LoopType::None => 0,
                                nether_tracker::LoopType::Forward => 1,
                                nether_tracker::LoopType::PingPong => 2,
                            },
                            instr.sample_is_stereo,
                            0,
                            0,
                            0,
                            instr.auto_vibrato_type,
                            instr.auto_vibrato_depth,
                            instr.auto_vibrato_rate,
                            instr.auto_vibrato_sweep,
                            instr.sample_relative_note,
                        )
                    };
                    Some((
                        sample_data.0,
                        sample_data.1,
                        sample_data.2,
                        sample_data.3,
                        sample_data.4,
                        sample_data.5,
                        sample_data.6,
                        sample_data.7,
                        sample_data.8,
                        sample_data.9,
                        sample_data.10,
                        sample_data.11,
                        sample_data.12,
                        instr.fadeout,
                        vol_env_enabled,
                        vol_env_sustain,
                        vol_env_loop,
                        pan_env_enabled,
                        pan_env_sustain,
                        pan_env_loop,
                    ))
                } else {
                    None
                }
            };

            let (
                finetune,
                loop_start,
                loop_end,
                loop_type,
                sample_is_stereo,
                sustain_loop_start,
                sustain_loop_end,
                sustain_loop_type,
                vib_type,
                vib_depth,
                vib_rate,
                vib_sweep,
                relative_note,
                fadeout_rate,
                vol_env_enabled,
                vol_env_sustain,
                vol_env_loop,
                pan_env_enabled,
                pan_env_sustain,
                pan_env_loop,
            ) = instr_data.unwrap_or((
                0, 0, 0, 0, false, 0, 0, 0, 0, 0, 0, 0, 0, 0, false, None, None, false, None, None,
            ));

            let (source_tuning, source_finetune, forward_loop_start, forward_loop_limit) = if is_it { (0, 0, 0.0, 0.0) } else {
                it_mapped_sample.as_ref().map(|(_,_,sample,_)| (sample.xm_source_tuning, sample.xm_source_finetune, sample.xm_forward_loop_start, sample.xm_forward_loop_limit))
                    .or_else(|| self.modules.get(raw_handle as usize).and_then(|m| m.as_ref())
                        .and_then(|m| m.module.instruments.get(incoming_instrument.checked_sub(1)? as usize))
                        .map(|i| (i.xm_source_tuning, i.xm_source_finetune, i.xm_forward_loop_start, i.xm_forward_loop_limit))).unwrap_or((0, 0, 0.0, 0.0))
            };
            let channel = &mut self.channels[ch_idx];
            channel.xm_source_tuning = source_tuning;
            channel.xm_source_finetune = source_finetune;
            channel.xm_forward_loop_start = forward_loop_start;
            channel.xm_forward_loop_limit = forward_loop_limit;
            channel.xm_finetune_delta = 0;
            // Original mono/stereo startup controls at 44.1/48/96 kHz use
            // the measured XM attack when there is no voice to crossfade.
            channel.xm_restart_attack = u32::from(starts_xm_attack);
            channel.note_on = true;
            channel.key_off = false;
            channel.sample_sustain_released = false;
            channel.note_fade = false;
            channel.sample_pos = 0.0;
            channel.xm_source_position = 0.0;
            channel.xm_loop_stopped = false;
            channel.sample_direction = 1;
            // XM note-only cells restart PCM, not the instrument envelopes.
            if is_it || note.has_instrument() {
                channel.volume_envelope_pos = 0;
                channel.envelope_started = 0;
                envelopes_reset = true;
                channel.panning_envelope_pos = 0;
                channel.panning_envelope_frozen = false;
                channel.volume_fadeout = VOLUME_FADEOUT_MAX as u16;
            }
            channel.fade_out_samples = 0; // Cancel any fade-out
            channel.fade_in_samples = FADE_IN_SAMPLES; // Start fade-in for crossfade

            // Reset vibrato/tremolo on new note
            if channel.vibrato_waveform < 4 {
                channel.vibrato_pos = 0;
            }
            if channel.tremolo_waveform < 4 {
                channel.tremolo_pos = 0;
            }

            // Restart the remembered note, not its pitch after previous slides.
            if note.has_note() {
                channel.current_note = note.note;
            }
            let pitch_note = it_mapped_sample
                .as_ref()
                .map(|(mapped_note, _, _, _)| *mapped_note)
                .unwrap_or(channel.current_note);
            let effective_note = (pitch_note as i16 + relative_note as i16).clamp(1, 96) as u8;
            channel.xm_retrigger_note = if is_it {0} else {effective_note};
            channel.base_period = note_to_period(effective_note, finetune);
            if !is_it {
                channel.xm_source_finetune = xm_sample_finetune(source_finetune, channel.xm_legacy_retrigger);
                channel.xm_finetune_delta = i16::from(channel.xm_source_finetune) - i16::from(source_finetune);
            }
            if !is_it && note.has_note() {
                if let TrackerEffect::SetFinetune(value) = note.effect {
                    // E5x replaces sample finetune, not its relative-note transpose.
                    // PCM already contains source tuning: apply only the difference.
                    channel.xm_finetune_delta = i16::from(value) - i16::from(source_finetune);
                    channel.xm_source_finetune = value;
                }
            }
            channel.base_period -= f32::from(channel.xm_finetune_delta) / 2.0;
            channel.period = channel.base_period;
            // IT cancels the old Gxx target on a fresh non-portamento note.
            if is_it {
                channel.target_period = 0.0;
            }
            channel.finetune = finetune;
            channel.sample_loop_start = loop_start;
            channel.sample_loop_end = loop_end;
            channel.sample_loop_type = loop_type;
            channel.sample_is_stereo = sample_is_stereo;
            channel.sample_sustain_loop_start = sustain_loop_start;
            channel.sample_sustain_loop_end = sustain_loop_end;
            channel.sample_sustain_loop_type = sustain_loop_type;

            channel.volume_envelope_zero_end = self
                .modules
                .get(raw_handle as usize)
                .and_then(|m| m.as_ref())
                .and_then(|m| {
                    m.module
                        .instruments
                        .get(incoming_instrument.saturating_sub(1) as usize)
                })
                .and_then(|instr| instr.volume_envelope.as_ref())
                .and_then(|env| env.points.last())
                .and_then(|&(tick, value)| (value == 0).then_some(tick));
            let instrument = self.modules.get(raw_handle as usize).and_then(|m| m.as_ref())
                .and_then(|m| m.module.instruments.get(incoming_instrument.saturating_sub(1) as usize));
            if is_it && !sample_mode {
                let sample_global = self.modules.get(raw_handle as usize).and_then(|m| m.as_ref())
                    .and_then(|m| channel.sample_index.and_then(|i| m.module.samples.get(i)))
                    .map_or(64, |s| s.global_volume);
                channel.reset_instrument_swing(instrument, sample_global, ch_idx);
                channel.instrument_global_volume = instrument.map_or(64, |i| i.global_volume);
            }
            channel.volume_envelope_escape_loop = !is_it && instrument
                .and_then(|i| i.volume_envelope.as_ref())
                .is_some_and(|e| e.has_sustain() && e.has_loop() && e.sustain_end == e.loop_end);
            channel.panning_envelope_escape_loop = !is_it && instrument
                .and_then(|i| i.panning_envelope.as_ref())
                .is_some_and(|e| e.has_sustain() && e.has_loop() && e.sustain_end == e.loop_end);
            channel.volume_envelope_end = instrument
                .and_then(|i| i.volume_envelope.as_ref())
                .and_then(|e| e.points.last()).map(|&(tick, _)| tick);
            // Copy envelope settings from instrument
            channel.volume_envelope_enabled = vol_env_enabled;
            channel.volume_envelope_sustain_loop = vol_env_sustain;
            channel.volume_envelope_loop = vol_env_loop;
            channel.instrument_fadeout_rate = fadeout_rate;

            channel.panning_envelope_enabled = pan_env_enabled;
            channel.panning_envelope_sustain_loop = pan_env_sustain;
            channel.panning_envelope_loop = pan_env_loop;

            // The live note path must initialize pitch envelopes too; trigger_note
            // is not used here. Keep filter routing separate from pitch modulation.
            let pitch_env = self.modules.get(raw_handle as usize)
                .and_then(|m| m.as_ref())
                .and_then(|m| m.module.instruments.get(incoming_instrument.saturating_sub(1) as usize))
                .and_then(|i| i.pitch_envelope.as_ref())
                .filter(|e| !sample_mode && !e.is_filter());
            channel.pitch_envelope_enabled = pitch_env.is_some_and(|e| e.is_enabled());
            channel.pitch_envelope_pos = 0;
            channel.pitch_envelope_value = 0.0;
            channel.pitch_envelope_sustain_loop = pitch_env.and_then(|e| e.sustain_ticks());
            channel.pitch_envelope_loop = pitch_env.filter(|e| e.has_loop()).and_then(|e| {
                Some((e.points.get(e.loop_begin as usize)?.0, e.points.get(e.loop_end as usize)?.0))
            });

            channel.reset_filter_envelope(instrument.and_then(|i| i.pitch_envelope.as_ref()).filter(|_| !sample_mode));

            if is_it && !sample_mode && let Some(instrument) = instrument {
                channel.apply_instrument_filter_defaults(instrument);
            }

            // Copy auto-vibrato settings from instrument
            channel.auto_vibrato_pos = 0;
            channel.auto_vibrato_sweep = 0;
            channel.auto_vibrato_type = vib_type;
            channel.auto_vibrato_depth = vib_depth;
            channel.auto_vibrato_rate = vib_rate;
            channel.auto_vibrato_sweep_len = vib_sweep;
        } else if note.is_note_cut() {
            self.channels[ch_idx].note_on = false;
            self.channels[ch_idx].volume = 0.0;
        } else if note.is_note_off() {
            let channel = &mut self.channels[ch_idx];
            if !is_it && !channel.volume_envelope_enabled {
                channel.note_fade = true;
                // Unlike K00, only volume-setting commands exempt note 97 from mute.
                if note.instrument == 0 && note.volume == 0
                    && !matches!(note.volume_effect, TrackerEffect::SetVolume(_))
                    && !matches!(note.effect, TrackerEffect::SetVolume(_))
                {
                    channel.volume = 0.0;
                } else {
                    channel.key_off = true;
                    channel.sample_sustain_released = true;
                }
            } else {
                channel.key_off = true;
                channel.sample_sustain_released = true;
            }
        } else if note.is_note_fade() && is_it && !sample_mode {
            self.channels[ch_idx].note_fade = true;
        }

        if old_effects_off && ![note.volume_effect, note.effect].iter().any(|e| matches!(e,
            TrackerEffect::TonePortamento(_) | TrackerEffect::TonePortaVolSlide { .. }))
        {
            self.channels[ch_idx].key_off = false;
            self.channels[ch_idx].note_fade = false;
        }

        // IT applies defaults on note changes (including Gxx), not instrument-only cells.
        // Sample pan takes precedence; explicit row effects execute afterward.
        if is_it && (note.has_note() || restarts_instrument_note) {
            if let Some(loaded) = self
                .modules
                .get(raw_handle as usize)
                .and_then(|m| m.as_ref())
            {
                let channel = &mut self.channels[ch_idx];
                let sample_pan = channel
                    .sample_index
                    .and_then(|index| loaded.module.samples.get(index))
                    .and_then(|sample| sample.default_pan);
                let instrument_pan = if sample_mode {
                    None
                } else {
                    channel
                        .instrument
                        .checked_sub(1)
                        .and_then(|index| loaded.module.instruments.get(index as usize))
                        .and_then(|instrument| instrument.default_pan)
                };
                if let Some(pan) = sample_pan.or(instrument_pan) {
                    channel.panning = pan.min(64) as f32 / 32.0 - 1.0;
                    channel.surround = false;
                }
            }
        }

        // FT2 reloads sample panning with an explicit instrument; row effects win.
        if !is_it && note.instrument > 0 && !xm_pending_instrument {
            if let Some(pan) = self.modules.get(raw_handle as usize)
                .and_then(|m| m.as_ref())
                .and_then(|m| {
                    if xm_mapped {
                        self.channels[ch_idx].sample_index.and_then(|index| m.module.samples.get(index))
                            .and_then(|sample| sample.default_pan)
                    } else {
                        m.module.instruments.get(self.channels[ch_idx].instrument.saturating_sub(1) as usize)
                            .and_then(|instrument| instrument.default_pan)
                    }
                })
            {
                self.channels[ch_idx].surround = false;
                self.channels[ch_idx].panning = pan as f32 / 128.0 - 1.0;
            }
        }

        // Handle volume column (TrackerNote has volume directly as 0-64)
        if note.volume > 0 {
            self.channels[ch_idx].volume = note.volume as f32 / CHANNEL_VOLUME_MAX;
        }

        // Both columns execute; volume-column commands precede the main column.
        self.process_volume_column_tick0(ch_idx, &note.volume_effect, note.note, note.instrument);
        // FT2 K00 with an instrument or volume command fades instead of muting.
        if !is_it && note.effect == TrackerEffect::KeyOff
            && !self.channels[ch_idx].volume_envelope_enabled
            && (note.instrument != 0 || note.volume > 0 || note.volume_effect != TrackerEffect::None)
        {
            self.channels[ch_idx].note_fade = true;
            self.channels[ch_idx].key_off = true;
            self.channels[ch_idx].sample_sustain_released = true;
        } else {
            self.process_unified_effect_tick0(ch_idx, &note.effect, note.note, note.instrument);
        }
        envelopes_reset
    }
}
