//! Tracker channel state
//!
//! Per-channel playback state for tracker music, including volume, panning,
//! vibrato, tremolo, envelopes, and IT-specific features like NNA and filters.

use nether_tracker::TrackerInstrument;

use super::FADE_IN_SAMPLES;
use super::utils::note_to_period;

/// Per-channel playback state
#[derive(Clone, Default, Debug)]
pub struct TrackerChannel {
    // Sample playback
    /// Sound handle from ROM (0 = none)
    pub sample_handle: u32,
    /// Fractional sample position for interpolation
    pub sample_pos: f64,
    /// Sample loop start
    pub sample_loop_start: u32,
    /// Sample loop end (start + length)
    pub sample_loop_end: u32,
    /// Sample loop type (0=none, 1=forward, 2=pingpong)
    pub sample_loop_type: u8,
    /// Playback direction for pingpong loops (1=forward, -1=backward)
    pub sample_direction: i8,

    // Volume
    /// Current volume (0.0-1.0)
    pub volume: f32,
    /// Target volume for slides
    pub target_volume: f32,
    /// Volume envelope position (ticks)
    pub volume_envelope_pos: u16,
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

    // Frequency/Pitch
    /// Current period (XM linear frequency)
    pub period: f32,
    /// Base period (without effects)
    pub base_period: f32,
    /// Target period for tone portamento
    pub target_period: f32,
    /// Portamento speed
    pub porta_speed: u8,
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

    // Note state
    /// Note is currently playing
    pub note_on: bool,
    /// Key-off has been triggered (release phase)
    pub key_off: bool,
    /// Current instrument index
    pub instrument: u8,

    // Effect memory (for effects that remember last parameter)
    pub last_porta_up: u8,
    pub last_porta_down: u8,
    pub last_volume_slide: u8,
    pub last_fine_porta_up: u8,
    pub last_fine_porta_down: u8,
    pub last_vibrato: u8,
    pub last_tremolo: u8,
    pub last_sample_offset: u8,
    /// Shared E/F/G portamento memory (used when LINK_G_MEMORY flag is set)
    pub shared_efg_memory: u8,

    // Arpeggio
    pub arpeggio_tick: u8,
    pub arpeggio_note1: u8,
    pub arpeggio_note2: u8,

    // Retrigger
    pub retrigger_tick: u8,
    pub retrigger_volume: i8,

    // Pattern loop (per-channel in XM)
    pub pattern_loop_row: u16,
    pub pattern_loop_count: u8,

    // Note cut/delay (ECx/EDx)
    pub note_cut_tick: u8,
    pub note_delay_tick: u8,
    pub delayed_note: u8,
    pub delayed_instrument: u8,

    // Volume column effect state
    pub vol_col_effect: u8,
    pub vol_col_param: u8,

    // Glissando control (E3x)
    pub glissando: bool,

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
    /// Volume envelope sustain tick (None if no sustain)
    pub volume_envelope_sustain_tick: Option<u16>,
    /// Volume envelope loop range (start_tick, end_tick), None if no loop
    pub volume_envelope_loop: Option<(u16, u16)>,
    /// Instrument fadeout rate (subtracted from volume_fadeout per tick after key-off)
    pub instrument_fadeout_rate: u16,

    /// Panning envelope enabled
    pub panning_envelope_enabled: bool,
    /// Panning envelope sustain tick
    pub panning_envelope_sustain_tick: Option<u16>,
    /// Panning envelope loop range
    pub panning_envelope_loop: Option<(u16, u16)>,

    // Retrigger mode for multiplicative volume (Rxy)
    pub retrigger_mode: u8,

    // Per-row effect activity flags (reset at row start, set by effects)
    // These track whether an effect is ACTIVE this row, not just remembered
    /// Volume slide is active this row
    pub volume_slide_active: bool,
    /// Portamento up is active this row
    pub porta_up_active: bool,
    /// Portamento down is active this row
    pub porta_down_active: bool,
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

    // ==========================================================================
    // IT-specific fields (used only when playing IT modules)
    // ==========================================================================

    // --- Pitch Envelope (IT only) ---
    /// Pitch envelope enabled
    pub pitch_envelope_enabled: bool,
    /// Pitch envelope position (ticks)
    pub pitch_envelope_pos: u16,
    /// Pitch envelope sustain tick
    pub pitch_envelope_sustain_tick: Option<u16>,
    /// Pitch envelope loop range
    pub pitch_envelope_loop: Option<(u16, u16)>,
    /// Current pitch envelope value (semitones offset, -32 to +32)
    pub pitch_envelope_value: f32,

    // --- Filter Envelope (IT only) ---
    /// Filter envelope enabled
    pub filter_envelope_enabled: bool,
    /// Filter envelope position (ticks)
    pub filter_envelope_pos: u16,
    /// Filter envelope sustain tick
    pub filter_envelope_sustain_tick: Option<u16>,
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
    /// Filter state z^-1
    pub filter_z1: f32,
    /// Filter state z^-2
    pub filter_z2: f32,
    /// Whether filter coefficients need recalculation
    pub filter_dirty: bool,

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
}

impl TrackerChannel {
    /// Reset channel to default state
    pub fn reset(&mut self) {
        *self = Self::default();
        self.sample_direction = 1;
        self.volume_fadeout = 65535;
        self.fade_out_samples = 0;
        self.fade_in_samples = 0;
        self.prev_sample = 0.0;

        // IT-specific defaults
        self.channel_volume = 64; // Full channel volume
        self.instrument_global_volume = 64; // Full instrument volume
        self.pitch_pan_separation = 0;
        self.pitch_pan_center = 60; // C-5
        self.filter_cutoff = 1.0; // Wide open filter
        self.filter_b0 = 1.0; // Passthrough filter
    }

    /// Trigger a new note (unified tracker format)
    pub fn trigger_note(&mut self, note: u8, instrument: Option<&TrackerInstrument>) {
        self.note_on = true;
        self.key_off = false;
        self.current_note = note; // Store for pitch-pan separation
        self.sample_pos = 0.0;
        self.sample_direction = 1;
        self.volume_envelope_pos = 0;
        self.panning_envelope_pos = 0;
        self.pitch_envelope_pos = 0; // IT pitch envelope
        self.filter_envelope_pos = 0; // IT filter envelope
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
            if let Some(cutoff) = instr.filter_cutoff {
                self.filter_cutoff = cutoff as f32 / 127.0;
                self.filter_dirty = true;
            }
            if let Some(resonance) = instr.filter_resonance {
                self.filter_resonance = resonance as f32 / 127.0;
                self.filter_dirty = true;
            }

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
            self.filter_envelope_enabled =
                instr.pitch_envelope.as_ref().is_some_and(|e| e.is_filter());
        }
    }

    /// Trigger key-off (release)
    pub fn trigger_key_off(&mut self) {
        self.key_off = true;
    }

    /// Reset per-row effect activity flags (called at the start of each row)
    ///
    /// XM/IT effects only apply during the row they appear. Memory values persist
    /// for "use last param" functionality, but the effect itself doesn't continue
    /// unless explicitly present on the new row.
    pub fn reset_row_effects(&mut self) {
        self.volume_slide_active = false;
        self.porta_up_active = false;
        self.porta_down_active = false;
        self.tone_porta_active = false;
        self.vibrato_active = false;
        self.tremolo_active = false;
        self.arpeggio_active = false;
        self.panning_slide_active = false;
        self.channel_volume_slide_active = false;
        self.tremor_active = false;
        self.panbrello_active = false;

        // Also reset per-row timing effects
        self.note_cut_tick = 0;
        self.note_delay_tick = 0;
        self.key_off_tick = 0;
        self.retrigger_tick = 0;

        // Reset arpeggio notes (arpeggio only applies on the row it appears)
        self.arpeggio_note1 = 0;
        self.arpeggio_note2 = 0;
    }

    /// Apply resonant low-pass filter to sample (IT only)
    ///
    /// Uses Direct Form II transposed biquad filter.
    pub fn apply_filter(&mut self, input: f32) -> f32 {
        // If filter is wide open (cutoff = 1.0) or disabled, bypass
        if self.filter_cutoff >= 1.0 {
            return input;
        }

        // Update filter coefficients if dirty
        if self.filter_dirty {
            self.update_filter_coefficients(22050.0); // ZX sample rate
            self.filter_dirty = false;
        }

        // Direct Form II transposed biquad
        let output = self.filter_b0 * input + self.filter_z1;
        self.filter_z1 = self.filter_b1 * input - self.filter_a1 * output + self.filter_z2;
        self.filter_z2 = self.filter_b2 * input - self.filter_a2 * output;
        output
    }

    /// Recalculate filter coefficients from cutoff and resonance (IT only)
    ///
    /// IT formula: freq = 110 * 2^(cutoff/24 + 0.25)
    /// where cutoff is normalized 0.0-1.0 (from IT's 0-127 range)
    pub fn update_filter_coefficients(&mut self, sample_rate: f32) {
        // Convert normalized cutoff (0.0-1.0) to frequency
        // IT uses: freq = 110 * 2^((cutoff * 127)/24 + 0.25)
        let cutoff_it = self.filter_cutoff * 127.0;
        let freq = 110.0 * 2.0_f32.powf(cutoff_it / 24.0 + 0.25);

        // Clamp frequency to Nyquist
        let freq = freq.min(sample_rate / 2.0 - 1.0);

        let omega = 2.0 * std::f32::consts::PI * freq / sample_rate;
        let sin_omega = omega.sin();
        let cos_omega = omega.cos();

        // Q factor from resonance (higher resonance = lower Q denominator)
        // IT resonance 0-127 mapped to 0.0-1.0
        let q_denom = 1.0 + self.filter_resonance * 10.0;
        let alpha = sin_omega / (2.0 * q_denom);

        // Low-pass filter coefficients
        let b0 = (1.0 - cos_omega) / 2.0;
        let b1 = 1.0 - cos_omega;
        let b2 = (1.0 - cos_omega) / 2.0;
        let a0 = 1.0 + alpha;
        let a1 = -2.0 * cos_omega;
        let a2 = 1.0 - alpha;

        // Normalize by a0
        self.filter_b0 = b0 / a0;
        self.filter_b1 = b1 / a0;
        self.filter_b2 = b2 / a0;
        self.filter_a1 = a1 / a0;
        self.filter_a2 = a2 / a0;
    }
}
