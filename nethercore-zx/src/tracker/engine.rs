//! Tracker engine implementation
//!
//! Core playback logic for the tracker engine including:
//! - State synchronization for rollback
//! - Row and tick processing
//! - Effect processing
//! - Audio rendering

use nether_tracker::{TrackerEffect, TrackerNote};

use super::channels::{DCT_OFF, NNA_CUT};
use super::state::RowStateCache;
use super::utils::{
    SINE_LUT, apply_channel_pan, apply_it_linear_slide, get_waveform_value, note_to_period,
    sample_channel, samples_per_tick,
};
use super::{FADE_IN_SAMPLES, MAX_TRACKER_CHANNELS, TrackerEngine, raw_tracker_handle};
use crate::audio::Sound;
use crate::state::tracker_flags;

// ============================================================================
// Tracker Audio Constants
// ============================================================================

/// Maximum volume level for volume envelopes (XM/IT spec)
const VOLUME_ENVELOPE_MAX: f32 = 64.0;

/// Maximum volume fadeout value (16-bit)
const VOLUME_FADEOUT_MAX: f32 = 65535.0;

/// Maximum tracker volume (8-bit state volume)
const TRACKER_VOLUME_MAX: f32 = 256.0;

/// Maximum channel volume (IT-style, 0-64)
const CHANNEL_VOLUME_MAX: f32 = 64.0;

/// Maximum global volume (IT-style, 0-128)
const GLOBAL_VOLUME_MAX: f32 = 128.0;

/// Panning envelope center value
const PAN_ENVELOPE_CENTER: f32 = 32.0;

/// Maximum panning note range
const PAN_NOTE_RANGE: f32 = 256.0;

/// Minimum BPM value
const MIN_BPM: i16 = 32;

/// Maximum BPM value
const MAX_BPM: i16 = 255;

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
    fn seek_to_position(
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

    // =========================================================================
    // NNA (New Note Action) Processing
    // =========================================================================

    /// Find an available background channel for NNA
    ///
    /// Background channels are slots beyond the module's channel count
    /// that can hold notes displaced by NNA actions.
    ///
    /// Returns the index of an available channel, or None if all are busy.
    fn find_background_channel(&self, num_channels: usize) -> Option<usize> {
        // Look for an available slot in the background channel range
        for idx in num_channels..MAX_TRACKER_CHANNELS {
            if self.channels[idx].is_available_for_nna() {
                return Some(idx);
            }
        }

        // If no free slot, find the quietest background channel to steal
        let mut quietest_idx = None;
        let mut quietest_vol: f32 = f32::MAX;

        for idx in num_channels..MAX_TRACKER_CHANNELS {
            let ch = &self.channels[idx];
            if ch.note_on {
                // Calculate effective volume
                let vol = ch.volume * (ch.volume_fadeout as f32 / 65535.0);
                if vol < quietest_vol {
                    quietest_vol = vol;
                    quietest_idx = Some(idx);
                }
            }
        }

        quietest_idx
    }

    /// Process duplicate check for a new note trigger
    ///
    /// Checks all background channels for duplicates matching the DCT criteria
    /// and applies the DCA action to matching channels.
    fn process_duplicate_check(
        &mut self,
        num_channels: usize,
        dct: u8,
        dca: u8,
        note: u8,
        sample_handle: u32,
        instrument: u8,
    ) {
        if dct == DCT_OFF {
            return;
        }

        // Check background channels for duplicates
        for idx in num_channels..MAX_TRACKER_CHANNELS {
            if self.channels[idx].matches_duplicate_check(dct, note, sample_handle, instrument) {
                self.channels[idx].apply_dca(dca);
            }
        }
    }

    /// Handle NNA for a channel when a new note is triggered
    ///
    /// If the channel has a note playing and NNA is not Cut, moves the
    /// current note to a background channel before the new note takes over.
    ///
    /// The `nna` parameter should come from the NEW instrument being triggered,
    /// not the channel's previous state.
    ///
    /// Returns true if NNA processing occurred (note was moved to background).
    fn handle_nna(&mut self, ch_idx: usize, num_channels: usize, nna: u8) -> bool {
        let channel = &self.channels[ch_idx];

        // Only process if channel has an audible note
        if !channel.is_audible() {
            return false;
        }

        // NNA_CUT doesn't need background channel
        if nna == NNA_CUT {
            self.channels[ch_idx].note_on = false;
            self.channels[ch_idx].volume = 0.0;
            return false;
        }

        // Find a background channel to move the note to
        if let Some(bg_idx) = self.find_background_channel(num_channels) {
            // Copy current channel state to background
            let bg_channel = self.channels[ch_idx].copy_to_background(ch_idx as u8);

            // Apply NNA action to the background copy
            self.channels[bg_idx] = bg_channel;
            self.channels[bg_idx].apply_nna_action(nna);

            return true;
        }

        // No background channel available - fall back to cut
        self.channels[ch_idx].note_on = false;
        self.channels[ch_idx].volume = 0.0;
        false
    }

    /// Internal note processing that accesses module by handle
    fn process_note_internal(
        &mut self,
        ch_idx: usize,
        note: &TrackerNote,
        handle: u32,
        _sounds: &[Option<Sound>],
    ) {
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

    /// Process unified TrackerEffect at tick 0 (row start)
    fn process_unified_effect_tick0(
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

    // ========================================================================
    // Channel Mixing (shared between render methods)
    // ========================================================================

    /// Mix all active channels into a stereo sample.
    ///
    /// This is the core mixing logic shared by `render_sample` and
    /// `render_sample_and_advance`. Extracts the common ~100 lines of
    /// channel processing, envelope handling, and panning.
    ///
    /// Takes `raw_handle` instead of a module reference to avoid borrow conflicts.
    ///
    /// # NNA Background Channels
    ///
    /// When IT modules use NNA settings other than Cut, notes may be moved to
    /// background channels (indices >= num_channels) to continue playing.
    /// This method mixes both regular channels and background channels.
    ///
    /// # IT-specific features
    ///
    /// - **Mix Volume**: Master output scaling (0-128, applied at the end)
    /// - **Panning Separation**: Stereo width control (0=mono, 128=full stereo)
    /// - **Surround Mode**: Phase inversion on one channel for S91 effect
    fn mix_channels(
        &mut self,
        raw_handle: u32,
        sounds: &[Option<Sound>],
        sample_rate: u32,
    ) -> (f32, f32) {
        let (num_channels, mix_volume, panning_separation) = self
            .modules
            .get(raw_handle as usize)
            .and_then(|m| m.as_ref())
            .map(|m| {
                (
                    m.module.num_channels as usize,
                    m.module.mix_volume,
                    m.module.panning_separation,
                )
            })
            .unwrap_or((0, 128, 128));

        let mut left = 0.0f32;
        let mut right = 0.0f32;

        // Mix all channels - both regular (0..num_channels) and background (num_channels..MAX)
        // Background channels are used by NNA to continue playing displaced notes
        for (ch_idx, channel) in self.channels.iter_mut().enumerate() {
            // Skip inactive channels in the regular range
            if ch_idx < num_channels {
                if !channel.note_on || channel.sample_handle == 0 {
                    continue;
                }
            } else {
                // For background channels, also check if it's actually playing
                // Background channels that have faded out are cleaned up here
                if !channel.note_on || channel.sample_handle == 0 || channel.volume_fadeout == 0 {
                    // Clean up dead background channels
                    if channel.is_background && channel.volume_fadeout == 0 {
                        channel.note_on = false;
                        channel.is_background = false;
                    }
                    continue;
                }
            }

            let sound = match sounds
                .get(channel.sample_handle as usize)
                .and_then(|s| s.as_ref())
            {
                Some(s) => s,
                None => continue,
            };

            // Get instrument reference for envelope processing (scoped to avoid borrow conflicts)
            let instr_idx = channel.instrument.saturating_sub(1) as usize;

            // Apply pitch envelope (IT only)
            if channel.pitch_envelope_enabled
                && let Some(loaded) = self
                    .modules
                    .get(raw_handle as usize)
                    .and_then(|m| m.as_ref())
                    && let Some(instr) = loaded.module.instruments.get(instr_idx)
                    && let Some(ref env) = instr.pitch_envelope
                    && env.is_enabled()
                    && !env.is_filter()
                {
                    let env_val = env.value_at(channel.pitch_envelope_pos) as f32;
                    channel.pitch_envelope_value = env_val;
                }

            // Update filter envelope (IT only)
            if channel.filter_envelope_enabled
                && let Some(loaded) = self
                    .modules
                    .get(raw_handle as usize)
                    .and_then(|m| m.as_ref())
                    && let Some(instr) = loaded.module.instruments.get(instr_idx)
                    && let Some(ref env) = instr.pitch_envelope
                    && env.is_filter()
                {
                    let env_val = env.value_at(channel.filter_envelope_pos) as f32;
                    channel.filter_cutoff = (env_val / VOLUME_ENVELOPE_MAX).clamp(0.0, 1.0);
                    channel.filter_dirty = true;
                }

            // Sample with interpolation
            let mut sample = sample_channel(channel, &sound.data, sample_rate);

            // Apply resonant low-pass filter (IT only)
            if channel.filter_cutoff < 1.0 {
                sample = channel.apply_filter(sample);
            }

            // Apply volume with envelope processing
            let mut vol = channel.volume;

            if channel.volume_envelope_enabled
                && let Some(loaded) = self
                    .modules
                    .get(raw_handle as usize)
                    .and_then(|m| m.as_ref())
                    && let Some(instr) = loaded.module.instruments.get(instr_idx)
                    && let Some(ref env) = instr.volume_envelope
                    && env.is_enabled()
                {
                    let env_val =
                        env.value_at(channel.volume_envelope_pos) as f32 / VOLUME_ENVELOPE_MAX;
                    vol *= env_val;
                }

            if channel.key_off {
                vol *= channel.volume_fadeout as f32 / VOLUME_FADEOUT_MAX;
            }

            vol *= self.global_volume;
            vol *= channel.channel_volume as f32 / CHANNEL_VOLUME_MAX;
            vol *= channel.instrument_global_volume as f32 / CHANNEL_VOLUME_MAX;

            if channel.tremor_mute {
                vol = 0.0;
            }

            // Apply panning with envelope
            let mut pan = channel.panning;

            if channel.pitch_pan_separation != 0 {
                let note_offset = channel.current_note as i16 - channel.pitch_pan_center as i16;
                let pan_offset =
                    (note_offset * channel.pitch_pan_separation as i16) as f32 / PAN_NOTE_RANGE;
                pan = (pan + pan_offset).clamp(-1.0, 1.0);
            }

            if channel.panning_envelope_enabled
                && let Some(loaded) = self
                    .modules
                    .get(raw_handle as usize)
                    .and_then(|m| m.as_ref())
                    && let Some(instr) = loaded.module.instruments.get(instr_idx)
                    && let Some(ref env) = instr.panning_envelope
                    && env.is_enabled()
                {
                    let env_val = env.value_at(channel.panning_envelope_pos) as f32;
                    pan = (env_val - PAN_ENVELOPE_CENTER) / PAN_ENVELOPE_CENTER;
                }

            if channel.panbrello_active && channel.panbrello_depth > 0 {
                let waveform_value = SINE_LUT[(channel.panbrello_pos >> 4) as usize & 0xF] as f32;
                let panbrello_offset = (waveform_value * channel.panbrello_depth as f32)
                    / (CHANNEL_VOLUME_MAX * PAN_NOTE_RANGE);
                pan = (pan + panbrello_offset).clamp(-1.0, 1.0);
            }

            // Apply panning separation (IT feature)
            // 128 = full stereo, 0 = mono
            // This reduces the stereo width by moving panning toward center
            if panning_separation < 128 {
                let sep_factor = panning_separation as f32 / 128.0;
                pan *= sep_factor;
            }

            let (l, r) = apply_channel_pan(sample * vol, pan);

            // Apply surround mode (IT S91 effect)
            // Inverts phase on right channel for "surround" psychoacoustic effect
            if channel.surround {
                left += l;
                right -= r; // Invert phase on right channel
            } else {
                left += l;
                right += r;
            }
        }

        // Apply mix volume (IT master volume, 0-128)
        let mix_scale = mix_volume as f32 / 128.0;
        (left * mix_scale, right * mix_scale)
    }

    // ========================================================================
    // Public Render Methods
    // ========================================================================

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
            let freq = super::utils::period_to_frequency(channel.period) as f64;
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

#[cfg(test)]
mod tests {
    use super::*;
    use nether_tracker::{
        FormatFlags, NewNoteAction, TrackerEffect, TrackerInstrument, TrackerModule, TrackerNote,
        TrackerPattern,
    };

    #[test]
    fn test_nna_uses_new_instrument_nna_not_channel_state() {
        // This test verifies that when a new note is triggered, the NNA action
        // comes from the NEW instrument, not the channel's previous state.
        //
        // Bug scenario: Channel has NNA=Cut from previous instrument,
        // new instrument has NNA=Continue. The old note should continue
        // playing in a background channel, not be cut.

        let mut engine = TrackerEngine::new();

        // Create instrument with NNA=Continue (should move old notes to background)
        let mut instr = TrackerInstrument::default();
        instr.nna = NewNoteAction::Continue;
        instr.fadeout = 1024; // Non-zero fadeout for audibility

        // Create pattern with a note on row 1
        let note2 = TrackerNote {
            note: 60,
            instrument: 1,
            volume: 64,
            effect: TrackerEffect::None,
        };

        let pattern = TrackerPattern {
            num_rows: 2,
            notes: vec![
                vec![TrackerNote::default()], // Row 0: empty
                vec![note2],                  // Row 1: trigger C-5
            ],
        };

        let module = TrackerModule {
            name: "NNA Test".to_string(),
            num_channels: 1,
            initial_speed: 6,
            initial_tempo: 125,
            global_volume: 128,
            mix_volume: 128,
            panning_separation: 128,
            order_table: vec![0],
            patterns: vec![pattern],
            instruments: vec![instr],
            samples: vec![],
            format: FormatFlags::IS_IT_FORMAT | FormatFlags::INSTRUMENTS,
            message: None,
            restart_position: 0,
        };

        let handle = engine.load_tracker_module(module, vec![1]); // Sample handle 1
        engine.is_it_format = true;

        // Simulate channel 0 already playing a note (as if row 0 was processed)
        engine.channels[0].note_on = true;
        engine.channels[0].sample_handle = 1;
        engine.channels[0].volume = 1.0;
        engine.channels[0].volume_fadeout = 65535;
        engine.channels[0].nna = NNA_CUT; // Channel state says NNA_CUT (stale value!)
        engine.channels[0].instrument = 1;

        // Process row 1 - new note with NNA=Continue instrument
        // This should use the instrument's NNA (Continue), not channel's (Cut)
        engine.current_row = 1;
        engine.process_row_tick0_internal(handle, &[]);

        // Verify: Background channel (index 1) should have the old note
        // because the NEW instrument has NNA=Continue
        assert!(
            engine.channels[1].note_on,
            "NNA=Continue should move old note to background channel. \
             Bug: NNA is reading from channel state (Cut) instead of new instrument (Continue)"
        );
        assert!(
            engine.channels[1].is_background,
            "Background flag should be set"
        );
    }

    #[test]
    fn test_nna_note_fade_triggers_key_off() {
        // Verify NNA=NoteFade properly triggers key_off and fadeout on displaced note

        let mut engine = TrackerEngine::new();

        let mut instr = TrackerInstrument::default();
        instr.nna = NewNoteAction::NoteFade;
        instr.fadeout = 2048;

        let note = TrackerNote {
            note: 60,
            instrument: 1,
            volume: 64,
            effect: TrackerEffect::None,
        };

        let pattern = TrackerPattern {
            num_rows: 2,
            notes: vec![vec![TrackerNote::default()], vec![note]],
        };

        let module = TrackerModule {
            name: "NNA Fade Test".to_string(),
            num_channels: 1,
            initial_speed: 6,
            initial_tempo: 125,
            global_volume: 128,
            mix_volume: 128,
            panning_separation: 128,
            order_table: vec![0],
            patterns: vec![pattern],
            instruments: vec![instr],
            samples: vec![],
            format: FormatFlags::IS_IT_FORMAT | FormatFlags::INSTRUMENTS,
            message: None,
            restart_position: 0,
        };

        let handle = engine.load_tracker_module(module, vec![1]);
        engine.is_it_format = true;

        // Set up channel with playing note (stale NNA=Cut)
        engine.channels[0].note_on = true;
        engine.channels[0].sample_handle = 1;
        engine.channels[0].volume = 1.0;
        engine.channels[0].volume_fadeout = 65535;
        engine.channels[0].nna = NNA_CUT; // Stale value
        engine.channels[0].instrument = 1;
        engine.channels[0].instrument_fadeout_rate = 0; // Will be set by NNA

        engine.current_row = 1;
        engine.process_row_tick0_internal(handle, &[]);

        // Background channel should have the old note with key_off triggered
        assert!(
            engine.channels[1].note_on,
            "NNA=NoteFade should move note to background"
        );
        assert!(
            engine.channels[1].key_off,
            "NNA=NoteFade should trigger key_off"
        );
        assert!(
            engine.channels[1].instrument_fadeout_rate > 0,
            "Fadeout rate should be set"
        );
    }
}
