//! Tracker channel state
//!
//! Per-channel playback state for tracker music, including volume, panning,
//! vibrato, tremolo, envelopes, and IT-specific features like NNA and filters.
//!
//! # NNA (New Note Action) System
//!
//! IT modules support polyphony-like behavior through NNA. When a new note is
//! triggered on a channel that already has a playing note:
//!
//! - **Cut**: Immediately silence the old note (default XM behavior)
//! - **Continue**: Move old note to a background channel, let it play to completion
//! - **NoteOff**: Trigger key-off on old note (release phase), move to background
//! - **NoteFade**: Start fadeout on old note, move to background
//!
//! Background channels are virtual channels beyond the module's channel count
//! that continue playing notes moved by NNA actions.

use nether_tracker::TrackerInstrument;

use super::FADE_IN_SAMPLES;
use super::utils::note_to_period;

mod filter;
mod nna;

#[cfg(test)]
mod tests;

// Re-export NNA constants
pub use nna::{
    DCA_CUT, DCA_NOTE_FADE, DCA_NOTE_OFF, DCT_INSTRUMENT, DCT_NOTE, DCT_OFF, DCT_SAMPLE,
    NNA_CONTINUE, NNA_CUT, NNA_NOTE_FADE, NNA_NOTE_OFF,
};

/// XM stereo gain transition, retained with the voice for exact rollback.
#[derive(Clone, Debug)]
pub struct XmEnvelopeRamp {
    pub from: [f32; 2],
    pub target: [f32; 2],
    pub current: [f32; 2],
    pub position: u32,
    pub stopped: bool,
}

/// Per-channel playback state
#[derive(Clone, Default, Debug)]
pub struct TrackerChannel {
    // Sample playback
    /// Sound handle from ROM (0 = none)
    pub sample_handle: u32,
    /// Exact IT sample-table entry; PCM handles may be shared.
    pub sample_index: Option<usize>,
    /// Fractional sample position for interpolation
    pub sample_pos: f64,
    /// Sample loop start
    pub sample_loop_start: u32,
    /// Sample loop end (start + length)
    pub sample_loop_end: u32,
    /// Sample loop type (0=none, 1=forward, 2=pingpong)
    pub sample_loop_type: u8,
    /// Sample data contains interleaved left/right frames.
    pub sample_is_stereo: bool,
    /// Sustain loop start while the key is held
    pub sample_sustain_loop_start: u32,
    /// Sustain loop end while the key is held
    pub sample_sustain_loop_end: u32,
    /// Sustain loop type (0=none, 1=forward, 2=pingpong)
    pub sample_sustain_loop_type: u8,
    /// Sustain loop was released by key-off and must not re-enter on porta.
    pub sample_sustain_released: bool,

    /// Playback direction for pingpong loops (1=forward, -1=backward)
    pub sample_direction: i8,

    // Volume
    /// Current volume (0.0-1.0)
    pub volume: f32,
    /// Target volume for slides
    pub target_volume: f32,
    /// Volume envelope position (ticks)
    pub volume_envelope_pos: u16,
    /// Completed active envelope ticks: bits 0/1/2/3 = volume/pan/pitch/filter.
    pub envelope_started: u8,
    pub xm_envelope_ramp: Option<XmEnvelopeRamp>,
    /// Volume fadeout value (0-65535)
    pub volume_fadeout: u16,
    /// Volume slide amount per tick
    pub volume_slide: i8,

    // Panning
    /// Current panning (-1.0=left, 0=center, 1.0=right)
    pub panning: f32,
    /// Panning envelope position (ticks)
    pub panning_envelope_pos: u16,
    /// Panning slide amount per tick
    pub panning_slide: i8,
    pub panning_left_on_ticks: bool,

    // Frequency/Pitch
    /// Current period (XM linear frequency)
    pub period: f32,
    /// Base period (without effects)
    pub base_period: f32,
    /// Target period for tone portamento
    pub target_period: f32,
    /// Portamento speed
    pub porta_speed: u16,
    /// Current instrument finetune
    pub finetune: i8,

    // Vibrato
    /// Vibrato position (0-63)
    pub vibrato_pos: u8,
    /// Vibrato speed
    pub vibrato_speed: u8,
    /// Vibrato depth
    pub vibrato_depth: u8,
    /// Vibrato waveform (0=sine, 1=ramp, 2=square, 3=random)
    pub vibrato_waveform: u8,

    // Tremolo
    /// Tremolo position (0-63)
    pub tremolo_pos: u8,
    /// Tremolo speed
    pub tremolo_speed: u8,
    /// Tremolo depth
    pub tremolo_depth: u8,
    /// Tremolo waveform
    pub tremolo_waveform: u8,
    /// XM audible modulation, separate from authored volume.
    pub xm_tremolo_delta: f32,

    // Note state
    /// Note is currently playing
    pub note_on: bool,
    /// Key-off has been triggered (release phase)
    pub key_off: bool,
    /// Explicit NoteFade, independent of sustain/key release.
    pub note_fade: bool,
    /// Current instrument index
    pub instrument: u8,
    /// IT instrument selection memory, including invalid selections.
    pub last_it_instrument: u8,
    /// XM pending selection; instrument-only cells retain the active voice.
    pub last_xm_instrument: u8,

    // Effect memory (for effects that remember last parameter)
    pub last_porta_up: u8,
    pub last_porta_down: u8,
    pub last_volume_slide: u8,
    /// Global-slide parameter memory (per-channel in IT).
    pub global_volume_slide: u8,
    pub global_volume_slide_active: bool,
    pub last_fine_porta_up: u8,
    pub last_fine_porta_down: u8,
    pub last_vibrato: u8,
    pub last_tremolo: u8,
    pub last_sample_offset: u8,
    /// Shared E/F/G portamento memory (used when LINK_G_MEMORY flag is set)
    pub shared_efg_memory: u8,
    /// Shared IT Sxy memory; S00 recalls the last full nonzero command.
    pub last_it_extended: u8,
    /// Full IT Txx parameter; T00 recalls slides and explicit tempo per channel.
    pub last_it_tempo: u8,
    pub last_it_arpeggio: u8,
    pub last_it_retrigger: u8,
    pub it_retrigger_count: u8,
    pub it_fine_vibrato: bool,
    pub it_tremolo_delta: f32,
    pub it_tremolo_from: f32,
    pub it_tremolo_ramp_pos: u32,

    // Arpeggio
    pub arpeggio_tick: u8,
    pub arpeggio_note1: u8,
    pub arpeggio_note2: u8,

    // Retrigger
    pub retrigger_tick: u8,
    pub retrigger_volume: i8,
    pub xm_retrigger_memory: u8,
    pub xm_retrigger_count: u8,
    pub xm_multi_retrigger_active: bool,

    // Pattern loop (per-channel in XM)
    pub pattern_loop_row: u16,
    pub pattern_loop_count: u8,

    // Note cut/delay (ECx/EDx)
    pub note_cut_tick: u8,
    pub note_delay_tick: u8,
    /// Full XM note retained until EDx fires; cloned with rollback snapshots.
    pub delayed_xm_note: Option<(nether_tracker::TrackerNote, u32)>,
    pub delayed_note: u8,
    pub delayed_instrument: u8,

    // Volume column effect state
    pub vol_col_effect: u8,
    pub vol_col_param: u8,

    // Glissando control (E3x)
    pub glissando: bool,
    /// XM Amiga-period slide arithmetic; retained with the channel snapshot.
    pub xm_amiga_slides: bool,
    pub xm_source_tuning: i16,
    pub xm_source_finetune: i8,
    /// Unrounded XM forward-loop start in normalized 22050 Hz frames.
    pub xm_forward_loop_start: f64,
    /// Unrounded XM forward-loop length in normalized 22050 Hz frames; zero disables.
    pub xm_forward_loop_limit: f64,
    /// Unrounded source-space playback position used only for XM stop timing.
    pub xm_source_position: f64,
    /// Source-rate stop silences PCM but retains note/effect processing.
    pub xm_loop_stopped: bool,
    pub xm_stop_tail_remaining: u16,
    pub xm_restart_attack: u32,
    /// Measured compatible-mode stopped-restart ramp, separate from effect memory.
    pub xm_short_restart_attack: bool,
    /// E5x correction relative to tuning already baked into PCM.
    pub xm_finetune_delta: i16,
    /// Latest XM pitched key, including a porta target, for E9 restart.
    pub xm_retrigger_note: u8,
    /// FT2 remembers reaching a target from lower pitch, not either direction.
    pub xm_porta_target_reached: bool,

    // Auto-vibrato (instrument) - copied from instrument on note trigger
    pub auto_vibrato_pos: u16,
    pub auto_vibrato_sweep: u16,
    pub auto_vibrato_type: u8,
    pub auto_vibrato_depth: u8,
    pub auto_vibrato_rate: u8,
    pub auto_vibrato_sweep_len: u8,

    // High sample offset (SAx extended command)
    pub sample_offset_high: u8,

    // Key off timing (Kxx)
    pub key_off_tick: u8,

    // Envelope data (cached from instrument at note trigger)
    /// Volume envelope enabled
    pub volume_envelope_enabled: bool,
    /// Silent terminal node; retained in snapshots and NNA copies.
    pub volume_envelope_zero_end: Option<u16>,
    /// Terminal volume node, including nonzero endpoints (IT automatic fade).
    pub volume_envelope_end: Option<u16>,
    /// Volume envelope inclusive sustain tick range (None if no sustain)
    pub volume_envelope_sustain_loop: Option<(u16, u16)>,
    /// Volume envelope loop range (start_tick, end_tick), None if no loop
    pub volume_envelope_loop: Option<(u16, u16)>,
    /// XM sustain and normal loop share the same authored end node.
    pub volume_envelope_escape_loop: bool,
    /// Instrument fadeout rate (subtracted from volume_fadeout per tick after key-off)
    pub instrument_fadeout_rate: u16,

    /// Panning envelope enabled
    pub panning_envelope_enabled: bool,
    /// Panning envelope inclusive sustain tick range
    pub panning_envelope_sustain_loop: Option<(u16, u16)>,
    /// Panning envelope loop range
    pub panning_envelope_loop: Option<(u16, u16)>,
    /// XM sustain and normal loop share the same authored end node.
    pub panning_envelope_escape_loop: bool,
    /// FT2 keeps an already reached panning sustain frozen even after key-off.
    pub panning_envelope_frozen: bool,

    // Retrigger mode for multiplicative volume (Rxy)
    pub retrigger_mode: u8,
    /// Supplied-module E9x/Rxy semantics, included in channel snapshots.
    pub xm_legacy_retrigger: bool,

    // Per-row effect activity flags (reset at row start, set by effects)
    // These track whether an effect is ACTIVE this row, not just remembered
    /// Volume slide is active this row
    pub volume_slide_active: bool,
    /// XM volume-column slide, independent of Axy/5xy/6xy memory.
    pub xm_volume_column_slide: i8,
    pub xm_volume_column_pan_slide: i8,
    /// Portamento up is active this row
    pub porta_up_count: u8,
    /// Portamento down is active this row
    pub porta_down_count: u8,
    /// Tone portamento is active this row
    pub tone_porta_active: bool,
    /// Vibrato is active this row
    pub vibrato_active: bool,
    /// Tremolo is active this row
    pub tremolo_active: bool,
    /// Arpeggio is active this row
    pub arpeggio_active: bool,
    /// Panning slide is active this row
    pub panning_slide_active: bool,
    /// Channel volume slide is active this row (IT only)
    pub channel_volume_slide_active: bool,

    // Fade state for smooth transitions (anti-pop)
    /// Fade-out samples remaining (0 = not fading out, >0 = fading out)
    pub fade_out_samples: u16,
    /// Fade-in samples remaining (0 = fully faded in, >0 = still fading in)
    pub fade_in_samples: u16,
    /// Previous sample value for crossfade during note transitions
    pub prev_sample: f32,
    /// Previous right sample value for stereo-source crossfades.
    pub prev_sample_right: f32,

    // ==========================================================================
    // IT-specific fields (used only when playing IT modules)
    // ==========================================================================

    // --- Pitch Envelope (IT only) ---
    /// Pitch envelope enabled
    pub pitch_envelope_enabled: bool,
    /// Pitch envelope position (ticks)
    pub pitch_envelope_pos: u16,
    /// Pitch envelope inclusive sustain tick range
    pub pitch_envelope_sustain_loop: Option<(u16, u16)>,
    /// Pitch envelope loop range
    pub pitch_envelope_loop: Option<(u16, u16)>,
    /// Current pitch envelope value (semitones offset, -32 to +32)
    pub pitch_envelope_value: f32,

    // --- Filter Envelope (IT only) ---
    /// Filter envelope enabled
    pub filter_envelope_enabled: bool,
    /// Filter envelope position (ticks)
    pub filter_envelope_pos: u16,
    /// Evaluated signed IT envelope value; None means no envelope contribution.
    pub filter_envelope_value: Option<i8>,
    /// Filter envelope inclusive sustain tick range
    pub filter_envelope_sustain_loop: Option<(u16, u16)>,
    /// Filter envelope loop range
    pub filter_envelope_loop: Option<(u16, u16)>,

    // --- Filter DSP State (IT resonant low-pass filter) ---
    /// Filter cutoff (0.0-1.0 normalized)
    pub filter_cutoff: f32,
    /// Filter resonance (0.0-1.0)
    pub filter_resonance: f32,
    /// Biquad coefficient a1
    pub filter_a1: f32,
    /// Biquad coefficient a2
    pub filter_a2: f32,
    /// Biquad coefficient b0
    pub filter_b0: f32,
    /// Biquad coefficient b1
    pub filter_b1: f32,
    /// Biquad coefficient b2
    pub filter_b2: f32,
    /// Previous filtered output y[n-1] (preserved across coefficient changes)
    pub filter_z1: f32,
    /// Previous filtered output y[n-2]
    pub filter_z2: f32,
    /// Whether filter coefficients need recalculation
    pub filter_dirty: bool,
    /// IT retains an active filter when full cutoff arrives without a note.
    pub filter_active: bool,
    /// Consumed by the first filtered sample after a real note trigger.
    pub filter_new_note: bool,
    /// Sample rate used for the cached filter coefficients.
    pub filter_sample_rate: u32,

    // --- NNA (New Note Action, IT only) ---
    /// New Note Action (0=Cut, 1=Continue, 2=NoteOff, 3=NoteFade)
    pub nna: u8,
    /// Duplicate Check Type (0=Off, 1=Note, 2=Sample, 3=Instrument)
    pub dct: u8,
    /// Duplicate Check Action (0=Cut, 1=NoteOff, 2=NoteFade)
    pub dca: u8,
    /// This channel is a "background" NNA channel (virtualized)
    pub is_background: bool,
    /// Parent channel index for background channels
    pub parent_channel: u8,

    // --- IT Channel Volume ---
    /// IT channel volume (0-64, separate from sample volume)
    pub channel_volume: u8,
    /// IT channel volume slide
    pub channel_volume_slide: i8,

    // --- IT Instrument Volume ---
    /// Instrument global volume (0-64, from TrackerInstrument.global_volume)
    pub instrument_global_volume: u8,
    /// Per-note IT swing offsets and generator state; cloned with NNA/snapshots.
    pub volume_swing: f32,
    pub pan_swing: f32,
    pub swing_rng: u32,

    // --- IT Pitch-Pan Separation ---
    /// Pitch-pan separation (-32 to +32)
    pub pitch_pan_separation: i8,
    /// Pitch-pan center note (0-119)
    pub pitch_pan_center: u8,
    /// Current note being played (for pitch-pan separation calculation)
    pub current_note: u8,

    // --- IT Tremor Effect ---
    /// Tremor on ticks (Ixy: x = on ticks)
    pub tremor_on_ticks: u8,
    /// Tremor off ticks (Ixy: y = off ticks)
    pub tremor_off_ticks: u8,
    /// Tremor tick counter
    pub tremor_counter: u8,
    /// Tremor is currently in mute phase
    pub tremor_mute: bool,
    /// Tremor is active this row
    pub tremor_active: bool,

    // --- IT Panbrello Effect ---
    /// Panbrello position (0-255)
    pub panbrello_pos: u8,
    /// Panbrello speed
    pub panbrello_speed: u8,
    /// Panbrello depth
    pub panbrello_depth: u8,
    /// Panbrello waveform (0=sine, 1=ramp, 2=square, 3=random)
    pub panbrello_waveform: u8,
    /// Panbrello is active this row
    pub panbrello_active: bool,

    // --- IT S9x Sound Control ---
    /// Surround sound mode (S91 = true, S90 = false)
    /// When enabled, the channel is mixed to both L/R with inverted phase on one side
    pub surround: bool,
}

impl TrackerChannel {
    /// IT modulation is evaluated once per tick, after both effect columns.
    pub(super) fn advance_it_modulation(&mut self, tick_zero: bool, old: bool) {
        use super::utils::it_waveform;
        if self.vibrato_active {
            if !old || !tick_zero {
                self.vibrato_pos = self.vibrato_pos.wrapping_add(self.vibrato_speed << 2);
            }
            let wave = it_waveform(self.vibrato_waveform, self.vibrato_pos);
            let scale = if self.it_fine_vibrato { 1 } else { 4 } * if old { 2 } else { 1 };
            let delta = (i32::from(wave) * i32::from(self.vibrato_depth) * scale) / 64;
            self.period =
                (self.base_period + if old { delta as f32 } else { -(delta as f32) }).max(1.0);
        }
        self.it_tremolo_from = self.it_tremolo_delta;
        self.it_tremolo_ramp_pos = 0;
        if self.tremolo_active {
            let wave = it_waveform(self.tremolo_waveform, self.tremolo_pos);
            self.it_tremolo_delta = f32::from(wave) * f32::from(self.tremolo_depth) / 2048.0;
            if !tick_zero || !old {
                self.tremolo_pos = self.tremolo_pos.wrapping_add(self.tremolo_speed << 2);
            }
        } else {
            self.it_tremolo_delta = 0.0;
            self.it_tremolo_from = 0.0;
        }
        if self.tremor_active {
            let on = self.tremor_on_ticks.max(1) + u8::from(old);
            let off = self.tremor_off_ticks.max(1) + u8::from(old);
            self.tremor_mute = self.tremor_counter >= on;
            self.tremor_counter = (self.tremor_counter + 1) % (on + off);
        }
        if self.retrigger_tick != 0 {
            if self.it_retrigger_count == 0 {
                self.retrigger_sample(false);
                self.it_retrigger_count = self.retrigger_tick;
            }
            self.it_retrigger_count -= 1;
        }
    }

    pub(super) fn advance_xm_retrigger(&mut self) {
        if self.xm_legacy_retrigger {
            self.xm_retrigger_count = self.xm_retrigger_count.saturating_add(1);
        }
        if self.xm_retrigger_count >= self.retrigger_tick {
            self.xm_retrigger_count = 0;
            self.retrigger_sample(true);
        }
        if !self.xm_legacy_retrigger {
            self.xm_retrigger_count = self.xm_retrigger_count.saturating_add(1);
        }
    }

    pub(super) fn retrigger_sample(&mut self, xm: bool) {
        self.sample_pos = 0.0;
        self.xm_source_position = 0.0;
        if self.xm_loop_stopped {
            self.xm_restart_attack = 1;
        }
        self.xm_stop_tail_remaining = 0;
        self.xm_loop_stopped = false;
        if self.retrigger_volume == 0 && !matches!(self.retrigger_mode, 6 | 7 | 14 | 15) {
            return;
        }
        self.volume = match self.retrigger_mode {
            6 => self.volume * if xm { 5.0 / 8.0 } else { 2.0 / 3.0 },
            7 => self.volume * 0.5,
            14 => self.volume * 1.5,
            15 => self.volume * 2.0,
            _ => self.volume + self.retrigger_volume as f32 / 64.0,
        }
        .clamp(0.0, 1.0);
        // Independently measured XM Rxy uses quarter-volume precision.
        if xm && matches!(self.retrigger_mode, 6 | 7 | 14 | 15) {
            self.volume = (self.volume * 256.0).floor() / 256.0;
        }
    }

    /// Reset channel to default state
    pub fn reset(&mut self) {
        *self = Self::default();
        self.sample_direction = 1;
        self.volume_fadeout = 65535;
        self.fade_out_samples = 0;
        self.fade_in_samples = 0;
        self.prev_sample = 0.0;
        self.prev_sample_right = 0.0;

        // IT-specific defaults
        self.channel_volume = 64; // Full channel volume
        self.instrument_global_volume = 64; // Full instrument volume
        self.pitch_pan_separation = 0;
        self.pitch_pan_center = 60; // C-5
        self.filter_cutoff = 1.0; // Wide open filter
        self.filter_b0 = 1.0; // Passthrough filter
        self.surround = false; // Normal stereo (no surround)
    }

    /// Original deterministic per-channel generator: stable across cold replay.
    pub(crate) fn reset_instrument_swing(
        &mut self,
        instrument: Option<&TrackerInstrument>,
        sample_global: u8,
        channel_index: usize,
    ) {
        self.volume_swing = 0.0;
        self.pan_swing = 0.0;
        let Some(instrument) = instrument else {
            return;
        };
        if instrument.random_volume == 0 && instrument.random_pan == 0 {
            return;
        }
        if self.swing_rng == 0 {
            self.swing_rng = channel_index as u32 + 1;
        }
        let mut next = || {
            self.swing_rng = self
                .swing_rng
                .wrapping_mul(1664525)
                .wrapping_add(1013904223);
            (self.swing_rng >> 8) as f32 / 8388608.0 - 1.0
        };
        self.volume_swing = next() * instrument.random_volume.min(100) as f32 / 100.0
            * instrument.global_volume.min(64) as f32
            / 64.0
            * sample_global.min(64) as f32
            / 64.0;
        self.pan_swing = next() * instrument.random_pan.min(64) as f32 / 32.0;
    }

    /// Trigger a new note (unified tracker format)
    pub fn trigger_note(&mut self, note: u8, instrument: Option<&TrackerInstrument>) {
        self.reset_instrument_swing(instrument, 64, 0);
        self.xm_source_tuning = instrument.map_or(0, |i| i.xm_source_tuning);
        self.xm_source_finetune = instrument.map_or(0, |i| i.xm_source_finetune);
        self.xm_forward_loop_start = instrument.map_or(0.0, |i| i.xm_forward_loop_start);
        self.xm_forward_loop_limit = instrument.map_or(0.0, |i| i.xm_forward_loop_limit);
        self.xm_source_position = 0.0;
        self.xm_finetune_delta = 0;
        self.xm_restart_attack = 0;
        self.xm_retrigger_note = 0;
        self.xm_porta_target_reached = false;
        self.note_on = true;
        self.key_off = false;
        self.sample_sustain_released = false;
        self.note_fade = false;
        self.xm_retrigger_count = 0;
        self.current_note = note; // Store for pitch-pan separation
        self.sample_pos = 0.0;
        self.xm_source_position = 0.0;
        self.xm_loop_stopped = false;
        self.sample_direction = 1;
        self.volume_envelope_pos = 0;
        self.envelope_started = 0;
        self.panning_envelope_pos = 0;
        self.panning_envelope_frozen = false;
        self.pitch_envelope_pos = 0; // IT pitch envelope
        self.reset_filter_envelope(instrument.and_then(|i| i.pitch_envelope.as_ref()));
        self.volume_fadeout = 65535;
        self.fade_out_samples = 0; // Cancel any fade-out
        self.fade_in_samples = FADE_IN_SAMPLES; // Start fade-in for crossfade
        // Note: prev_sample is preserved for crossfade blending

        // Reset vibrato/tremolo positions on new note
        if self.vibrato_waveform < 4 {
            self.vibrato_pos = 0;
        }
        if self.tremolo_waveform < 4 {
            self.tremolo_pos = 0;
        }

        // Reset auto-vibrato state (instrument vibrato)
        self.auto_vibrato_pos = 0;
        self.auto_vibrato_sweep = 0;

        // Set period from note with finetune
        if let Some(instr) = instrument {
            // Apply finetune from instrument (critical for XM pitch accuracy)
            self.base_period = note_to_period(note, instr.sample_finetune);
            self.finetune = instr.sample_finetune;

            // Copy sample loop data (critical for XM sample looping)
            self.sample_loop_start = instr.sample_loop_start;
            self.sample_loop_end = instr.sample_loop_end;
            self.sample_loop_type = match instr.sample_loop_type {
                nether_tracker::LoopType::None => 0,
                nether_tracker::LoopType::Forward => 1,
                nether_tracker::LoopType::PingPong => 2,
            };
            self.sample_is_stereo = instr.sample_is_stereo;
        } else {
            self.base_period = note_to_period(note, 0);
        }
        self.period = self.base_period;

        // Initialize instrument properties (both XM and IT)
        if let Some(instr) = instrument {
            // Copy NNA settings from instrument
            self.nna = match instr.nna {
                nether_tracker::NewNoteAction::Cut => 0,
                nether_tracker::NewNoteAction::Continue => 1,
                nether_tracker::NewNoteAction::NoteOff => 2,
                nether_tracker::NewNoteAction::NoteFade => 3,
            };
            self.dct = match instr.dct {
                nether_tracker::DuplicateCheckType::Off => 0,
                nether_tracker::DuplicateCheckType::Note => 1,
                nether_tracker::DuplicateCheckType::Sample => 2,
                nether_tracker::DuplicateCheckType::Instrument => 3,
            };
            self.dca = match instr.dca {
                nether_tracker::DuplicateCheckAction::Cut => 0,
                nether_tracker::DuplicateCheckAction::NoteOff => 1,
                nether_tracker::DuplicateCheckAction::NoteFade => 2,
            };

            // Copy fadeout rate
            self.instrument_fadeout_rate = instr.fadeout;

            // Copy instrument global volume (IT feature)
            self.instrument_global_volume = instr.global_volume;

            // Copy pitch-pan separation (IT feature)
            self.pitch_pan_separation = instr.pitch_pan_separation;
            self.pitch_pan_center = instr.pitch_pan_center;

            // Set up filter from instrument defaults
            self.apply_instrument_filter_defaults(instr);

            self.volume_envelope_end = instr
                .volume_envelope
                .as_ref()
                .and_then(|env| env.points.last())
                .map(|&(tick, _)| tick);
            self.volume_envelope_zero_end = instr
                .volume_envelope
                .as_ref()
                .and_then(|env| env.points.last())
                .and_then(|&(tick, value)| (value == 0).then_some(tick));
            // Enable envelopes if present
            self.volume_envelope_enabled = instr
                .volume_envelope
                .as_ref()
                .is_some_and(|e| e.is_enabled());
            self.panning_envelope_enabled = instr
                .panning_envelope
                .as_ref()
                .is_some_and(|e| e.is_enabled());
            self.pitch_envelope_enabled = instr
                .pitch_envelope
                .as_ref()
                .is_some_and(|e| e.is_enabled());
        }
    }

    /// Trigger key-off (release)
    pub fn trigger_key_off(&mut self) {
        self.key_off = true;
        self.sample_sustain_released = true;
    }

    /// Reset per-row effect activity flags (called at the start of each row)
    ///
    /// XM/IT effects only apply during the row they appear. Memory values persist
    /// for "use last param" functionality, but the effect itself doesn't continue
    /// unless explicitly present on the new row.
    pub fn reset_row_effects(&mut self) {
        self.volume_slide_active = false;
        self.xm_volume_column_slide = 0;
        self.xm_volume_column_pan_slide = 0;
        self.global_volume_slide_active = false;
        self.porta_up_count = 0;
        self.porta_down_count = 0;
        self.tone_porta_active = false;
        self.vibrato_active = false;
        self.tremolo_active = false;
        self.xm_tremolo_delta = 0.0;
        self.arpeggio_active = false;
        self.panning_slide_active = false;
        self.panning_left_on_ticks = false;
        self.channel_volume_slide_active = false;
        self.tremor_active = false;
        self.panbrello_active = false;

        // Also reset per-row timing effects
        self.note_cut_tick = 0;
        self.note_delay_tick = 0;
        self.delayed_xm_note = None;
        self.key_off_tick = 0;
        self.retrigger_tick = 0;
        self.xm_multi_retrigger_active = false;

        // Reset arpeggio notes (arpeggio only applies on the row it appears)
        self.arpeggio_note1 = 0;
        self.arpeggio_note2 = 0;
    }
}
