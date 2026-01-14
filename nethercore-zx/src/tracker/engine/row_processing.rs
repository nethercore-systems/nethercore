//! Row and note processing at tick 0

use nether_tracker::TrackerNote;

use super::super::utils::note_to_period;
use super::super::{FADE_IN_SAMPLES, TrackerEngine, raw_tracker_handle};
use super::VOLUME_FADEOUT_MAX;
use crate::audio::Sound;

impl TrackerEngine {
    /// Internal version that accesses module by handle to avoid borrow issues
    pub(crate) fn process_row_tick0_internal(&mut self, handle: u32, sounds: &[Option<Sound>]) {
        // Get module data - need to access by index to work around borrow checker
        let raw_handle = raw_tracker_handle(handle);
        let (num_channels, pattern_info, is_it, old_effects, link_g) = {
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
            let link_g = loaded
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
            )
        };

        // Store format flags for use in effect processing
        self.is_it_format = is_it;
        self.old_effects_mode = old_effects;
        self.link_g_memory = link_g;

        // Reset tempo slide (only active during the row it appears on)
        self.tempo_slide = 0;

        // Reset per-row effect state for all channels before processing
        // XM/IT effects only apply during the row they appear on
        for ch_idx in 0..num_channels as usize {
            self.channels[ch_idx].reset_row_effects();
        }

        // Process each note
        for (ch_idx, note) in pattern_info {
            self.process_note_internal(ch_idx, &note, handle, sounds);
        }
    }

    /// Internal note processing that accesses module by handle
    pub(super) fn process_note_internal(
        &mut self,
        ch_idx: usize,
        note: &TrackerNote,
        handle: u32,
        _sounds: &[Option<Sound>],
    ) {
        use super::super::channels::NNA_CUT;
        use super::CHANNEL_VOLUME_MAX;

        let raw_handle = raw_tracker_handle(handle);
        // Handle instrument change
        if note.has_instrument() {
            let instr_idx = (note.instrument - 1) as usize;
            self.channels[ch_idx].instrument = note.instrument;

            // Get sound handle and instrument data
            let (sound_handle, loop_start, loop_end, loop_type, finetune) = {
                let loaded = match self
                    .modules
                    .get(raw_handle as usize)
                    .and_then(|m| m.as_ref())
                {
                    Some(m) => m,
                    None => return,
                };
                let sound_handle = loaded.sound_handles.get(instr_idx).copied().unwrap_or(0);
                // Get sample metadata from TrackerInstrument
                if let Some(instr) = loaded.module.instruments.get(instr_idx) {
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
                        instr.sample_finetune,
                    )
                } else {
                    (sound_handle, 0, 0, 0, 0)
                }
            };

            self.channels[ch_idx].sample_handle = sound_handle;
            self.channels[ch_idx].sample_loop_start = loop_start;
            self.channels[ch_idx].sample_loop_end = loop_end;
            self.channels[ch_idx].sample_loop_type = loop_type;
            self.channels[ch_idx].finetune = finetune;
            self.channels[ch_idx].volume = 1.0;
        }

        // Handle note
        if note.has_note() {
            // Get module channel count and NNA data for processing
            let (num_channels, nna_data) = {
                let loaded = match self
                    .modules
                    .get(raw_handle as usize)
                    .and_then(|m| m.as_ref())
                {
                    Some(m) => m,
                    None => return,
                };
                let instr_idx = (self.channels[ch_idx].instrument.saturating_sub(1)) as usize;
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

            // Process NNA: handle the currently playing note before triggering new one
            // This is an IT feature - move the old note to a background channel based on NNA setting
            if self.is_it_format {
                // Use NNA from the NEW instrument, not channel's stale state
                let nna = nna_data.map(|(nna, _, _)| nna).unwrap_or(NNA_CUT);
                self.handle_nna(ch_idx, num_channels, nna);

                // Process duplicate check against background channels
                if let Some((_, dct, dca)) = nna_data {
                    let sample_handle = self.channels[ch_idx].sample_handle;
                    let instrument = self.channels[ch_idx].instrument;
                    self.process_duplicate_check(
                        num_channels,
                        dct,
                        dca,
                        note.note,
                        sample_handle,
                        instrument,
                    );
                }
            }

            // Fetch all instrument data we need for note trigger
            let instr_data = {
                let loaded = match self
                    .modules
                    .get(raw_handle as usize)
                    .and_then(|m| m.as_ref())
                {
                    Some(m) => m,
                    None => return,
                };
                let instr_idx = (self.channels[ch_idx].instrument.saturating_sub(1)) as usize;
                if let Some(instr) = loaded.module.instruments.get(instr_idx) {
                    // Extract envelope data from TrackerEnvelope
                    let (vol_env_enabled, vol_env_sustain, vol_env_loop) =
                        if let Some(ref env) = instr.volume_envelope {
                            let enabled = env.is_enabled();
                            let sustain = if env.has_sustain() {
                                env.points
                                    .get(env.sustain_begin as usize)
                                    .map(|(tick, _)| *tick)
                            } else {
                                None
                            };
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
                            let sustain = if env.has_sustain() {
                                env.points
                                    .get(env.sustain_begin as usize)
                                    .map(|(tick, _)| *tick)
                            } else {
                                None
                            };
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

                    // Get sample metadata from TrackerInstrument
                    let loop_type = match instr.sample_loop_type {
                        nether_tracker::LoopType::None => 0u8,
                        nether_tracker::LoopType::Forward => 1u8,
                        nether_tracker::LoopType::PingPong => 2u8,
                    };
                    Some((
                        instr.sample_finetune,
                        instr.sample_loop_start,
                        instr.sample_loop_end,
                        loop_type,
                        instr.auto_vibrato_type,
                        instr.auto_vibrato_depth,
                        instr.auto_vibrato_rate,
                        instr.auto_vibrato_sweep,
                        instr.sample_relative_note,
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
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, false, None, None, false, None, None,
            ));

            let channel = &mut self.channels[ch_idx];
            channel.note_on = true;
            channel.key_off = false;
            channel.sample_pos = 0.0;
            channel.sample_direction = 1;
            channel.volume_envelope_pos = 0;
            channel.panning_envelope_pos = 0;
            channel.volume_fadeout = VOLUME_FADEOUT_MAX as u16;
            channel.fade_out_samples = 0; // Cancel any fade-out
            channel.fade_in_samples = FADE_IN_SAMPLES; // Start fade-in for crossfade

            // Reset vibrato/tremolo on new note
            if channel.vibrato_waveform < 4 {
                channel.vibrato_pos = 0;
            }
            if channel.tremolo_waveform < 4 {
                channel.tremolo_pos = 0;
            }

            // Apply sample relative note offset to pitch calculation
            // XM spec: RealNote = PatternNote + RelativeTone
            let effective_note = (note.note as i16 + relative_note as i16).clamp(1, 96) as u8;
            channel.base_period = note_to_period(effective_note, finetune);
            channel.period = channel.base_period;
            channel.finetune = finetune;
            channel.sample_loop_start = loop_start;
            channel.sample_loop_end = loop_end;
            channel.sample_loop_type = loop_type;

            // Copy envelope settings from instrument
            channel.volume_envelope_enabled = vol_env_enabled;
            channel.volume_envelope_sustain_tick = vol_env_sustain;
            channel.volume_envelope_loop = vol_env_loop;
            channel.instrument_fadeout_rate = fadeout_rate;

            channel.panning_envelope_enabled = pan_env_enabled;
            channel.panning_envelope_sustain_tick = pan_env_sustain;
            channel.panning_envelope_loop = pan_env_loop;

            // Copy auto-vibrato settings from instrument
            channel.auto_vibrato_pos = 0;
            channel.auto_vibrato_sweep = 0;
            channel.auto_vibrato_type = vib_type;
            channel.auto_vibrato_depth = vib_depth;
            channel.auto_vibrato_rate = vib_rate;
            channel.auto_vibrato_sweep_len = vib_sweep;
        } else if note.is_note_off() {
            self.channels[ch_idx].key_off = true;
        }

        // Handle volume column (TrackerNote has volume directly as 0-64)
        if note.volume > 0 {
            self.channels[ch_idx].volume = note.volume as f32 / CHANNEL_VOLUME_MAX;
        }

        // Handle effects (tick 0 processing)
        self.process_unified_effect_tick0(ch_idx, &note.effect, note.note, note.instrument);
    }
}
