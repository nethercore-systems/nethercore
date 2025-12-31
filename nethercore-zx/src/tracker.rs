//! Tracker playback engine (XM and IT formats)
//!
//! This module implements the playback engine for tracker music, supporting both:
//! - XM (Extended Module) - FastTracker II format, up to 32 channels
//! - IT (Impulse Tracker) - Impulse Tracker format, up to 64 channels
//!
//! It integrates with the existing audio system to provide era-authentic music playback
//! with full rollback netcode support.
//!
//! # Architecture
//!
//! - **TrackerState** (in rollback_state.rs) - Minimal 64-byte POD state for rollback
//! - **TrackerEngine** (this module) - Full playback engine with channel state
//! - **XmModule** (from nether-xm) - Parsed XM pattern and instrument data
//! - **ItModule** (from nether-it) - Parsed IT pattern and instrument data
//!
//! The engine is designed to reconstruct its full state from TrackerState by seeking
//! to the correct position and replaying ticks. This keeps rollback snapshots small.
//!
//! # IT-Specific Features
//!
//! - **NNA (New Note Action)** - Cut, Continue, NoteOff, or NoteFade when new note arrives
//! - **Pitch Envelope** - Modulate pitch over time with envelope points
//! - **Filter Envelope** - Resonant low-pass filter with cutoff envelope
//! - **64 Channels** - Twice the channel count of XM

use std::collections::BTreeMap;

use nether_tracker::{TrackerModule, TrackerInstrument, TrackerNote, TrackerEffect};
use nether_xm::XmModule;
use nether_it::ItModule;

use crate::audio::Sound;
use crate::state::tracker_flags;

/// Maximum number of tracker channels (XM: 32, IT: 64)
pub const MAX_TRACKER_CHANNELS: usize = 64;

/// Default XM speed (ticks per row)
pub const DEFAULT_SPEED: u16 = 6;

/// Default XM tempo (BPM)
pub const DEFAULT_BPM: u16 = 125;

/// Number of samples for fade-out (anti-pop) at 44.1kHz
/// ~3ms fade-out = 132 samples, enough to avoid pops while being inaudible
pub const FADE_OUT_SAMPLES: u16 = 132;

/// Number of samples for fade-in (anti-pop) at 44.1kHz
/// ~2ms fade-in = 88 samples, short enough to not affect attack
pub const FADE_IN_SAMPLES: u16 = 88;

/// Flag bit for tracker handles (bit 31)
///
/// Tracker handles have this bit set to distinguish them from PCM sound handles.
/// This enables the unified music API to detect which type of music to play.
pub const TRACKER_HANDLE_FLAG: u32 = 0x80000000;

/// Check if a handle is a tracker handle
#[inline]
pub fn is_tracker_handle(handle: u32) -> bool {
    (handle & TRACKER_HANDLE_FLAG) != 0
}

/// Get the raw handle value (strip the tracker flag)
#[inline]
pub fn raw_tracker_handle(handle: u32) -> u32 {
    handle & !TRACKER_HANDLE_FLAG
}

/// Main tracker playback engine
///
/// This contains the "heavy" state that doesn't need to be rolled back.
/// It can be reconstructed from TrackerState by seeking to the position.
#[derive(Debug)]
pub struct TrackerEngine {
    /// Loaded tracker modules (by handle, 1-indexed)
    modules: Vec<Option<LoadedModule>>,

    /// Per-channel playback state
    channels: [TrackerChannel; MAX_TRACKER_CHANNELS],

    /// Global volume (0.0-1.0)
    global_volume: f32,

    /// Next handle to allocate
    next_handle: u32,

    /// Current playback position (for sync detection)
    current_order: u16,
    current_row: u16,
    current_tick: u16,

    /// Samples rendered within current tick
    tick_samples_rendered: u32,

    /// Row state cache for fast rollback seeks
    row_cache: RowStateCache,

    /// Pattern delay (EEx) - number of times to repeat current row
    pattern_delay: u8,
    /// Pattern delay counter - tracks how many times row has been repeated
    pattern_delay_count: u8,

    /// Fine pattern delay (S6x) - extra ticks to add to current row
    fine_pattern_delay: u8,

    /// Global volume slide memory (Hxy effect)
    last_global_vol_slide: u8,

    /// Whether current module is IT format (affects vibrato depth, etc.)
    is_it_format: bool,

    /// Old effects mode (S3M compatibility - affects vibrato/tremolo depth)
    old_effects_mode: bool,

    /// Link G memory with E/F for portamento
    link_g_memory: bool,

    /// Tempo slide amount per tick (positive = up, negative = down, 0 = none)
    tempo_slide: i8,
}

/// A loaded tracker module with resolved sample handles
#[derive(Debug)]
struct LoadedModule {
    /// Parsed tracker module data (unified format)
    module: TrackerModule,
    /// Sound handles for each instrument (instrument index -> sound handle)
    sound_handles: Vec<u32>,
}

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
            self.volume_envelope_enabled = instr.volume_envelope.as_ref().map_or(false, |e| e.is_enabled());
            self.panning_envelope_enabled = instr.panning_envelope.as_ref().map_or(false, |e| e.is_enabled());
            self.pitch_envelope_enabled = instr.pitch_envelope.as_ref().map_or(false, |e| e.is_enabled());
            self.filter_envelope_enabled = instr.pitch_envelope.as_ref().map_or(false, |e| e.is_filter());
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

/// Row state cache for fast rollback reconstruction
///
/// Uses BTreeMap for O(log n) range queries instead of O(n) linear search.
#[derive(Debug)]
struct RowStateCache {
    /// Cached channel states: (order, row) -> channels (sorted by key)
    cache: BTreeMap<(u16, u16), CachedRowState>,
    /// Maximum cache entries
    max_entries: usize,
}

#[derive(Debug)]
struct CachedRowState {
    channels: Box<[TrackerChannel; MAX_TRACKER_CHANNELS]>,
    global_volume: f32,
}

impl Default for RowStateCache {
    fn default() -> Self {
        Self {
            cache: BTreeMap::new(),
            max_entries: 256, // ~256 * 32 channels * ~200 bytes = ~1.6MB max
        }
    }
}

impl RowStateCache {
    /// Check if we should cache this row (every 4 rows or pattern boundary)
    fn should_cache(row: u16) -> bool {
        row % 4 == 0
    }

    /// Find nearest cached state before or at target position (O(log n) with BTreeMap)
    fn find_nearest(
        &self,
        target_order: u16,
        target_row: u16,
    ) -> Option<((u16, u16), &CachedRowState)> {
        // Use range query to find the greatest key <= (target_order, target_row)
        self.cache
            .range(..=(target_order, target_row))
            .next_back()
            .map(|(pos, state)| (*pos, state))
    }

    /// Store state at row
    fn store(
        &mut self,
        order: u16,
        row: u16,
        channels: &[TrackerChannel; MAX_TRACKER_CHANNELS],
        global_volume: f32,
    ) {
        // Evict oldest entry if at capacity (BTreeMap keeps entries sorted, so first is oldest by position)
        if self.cache.len() >= self.max_entries {
            if let Some(&key) = self.cache.keys().next() {
                self.cache.remove(&key);
            }
        }

        self.cache.insert(
            (order, row),
            CachedRowState {
                channels: Box::new(channels.clone()),
                global_volume,
            },
        );
    }

    /// Clear the cache
    fn clear(&mut self) {
        self.cache.clear();
    }
}

impl Default for TrackerEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl TrackerEngine {
    /// Create a new tracker engine
    pub fn new() -> Self {
        Self {
            modules: Vec::new(),
            channels: std::array::from_fn(|_| {
                let mut ch = TrackerChannel::default();
                ch.sample_direction = 1;
                ch.volume_fadeout = 65535;
                ch
            }),
            global_volume: 1.0,
            next_handle: 1,
            current_order: 0,
            current_row: 0,
            current_tick: 0,
            tick_samples_rendered: 0,
            row_cache: RowStateCache::default(),
            pattern_delay: 0,
            pattern_delay_count: 0,
            fine_pattern_delay: 0,
            last_global_vol_slide: 0,
            is_it_format: false,
            old_effects_mode: false,
            link_g_memory: false,
            tempo_slide: 0,
        }
    }

    /// Load a module with resolved sound handles
    ///
    /// Returns a handle for later playback (1-indexed, 0 is invalid).
    /// The returned handle has TRACKER_HANDLE_FLAG set (bit 31) to distinguish
    /// it from PCM sound handles in the unified music API.
    /// Load an XM module and convert to unified TrackerModule
    pub fn load_xm_module(&mut self, xm_module: XmModule, sound_handles: Vec<u32>) -> u32 {
        let tracker_module = nether_tracker::from_xm_module(&xm_module);
        self.load_tracker_module(tracker_module, sound_handles)
    }

    /// Load an IT module and convert to unified TrackerModule
    pub fn load_it_module(&mut self, it_module: ItModule, sound_handles: Vec<u32>) -> u32 {
        let tracker_module = nether_tracker::from_it_module(&it_module);
        self.load_tracker_module(tracker_module, sound_handles)
    }

    /// Load a unified TrackerModule (internal)
    fn load_tracker_module(&mut self, module: TrackerModule, sound_handles: Vec<u32>) -> u32 {
        let raw_handle = self.next_handle;
        self.next_handle += 1;

        // Extend modules vector if needed
        let idx = raw_handle as usize;
        if idx >= self.modules.len() {
            self.modules.resize_with(idx + 1, || None);
        }

        self.modules[idx] = Some(LoadedModule {
            module,
            sound_handles,
        });

        // Return flagged handle so unified music API can detect tracker vs PCM
        raw_handle | TRACKER_HANDLE_FLAG
    }

    /// Get a loaded module by handle
    ///
    /// Accepts both flagged (from load_module) and raw handles.
    pub fn get_module(&self, handle: u32) -> Option<&TrackerModule> {
        let raw = raw_tracker_handle(handle);
        self.modules
            .get(raw as usize)
            .and_then(|m| m.as_ref())
            .map(|m| &m.module)
    }

    /// Get the tempo slide amount for this row
    /// Returns BPM adjustment per tick (positive = faster, negative = slower)
    /// IT effect: T0x = slide down by x, T1x = slide up by x
    pub fn get_tempo_slide(&self) -> i8 {
        self.tempo_slide
    }

    /// Reset playback to the beginning
    pub fn reset(&mut self) {
        for ch in &mut self.channels {
            ch.reset();
        }
        self.global_volume = 1.0;
        self.current_order = 0;
        self.current_row = 0;
        self.current_tick = 0;
        self.tick_samples_rendered = 0;
        self.row_cache.clear();
    }

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
                let loaded = match self.modules.get(raw_handle as usize).and_then(|m| m.as_ref()) {
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
    fn process_row_tick0_internal(&mut self, handle: u32, sounds: &[Option<Sound>]) {
        // Get module data - need to access by index to work around borrow checker
        let raw_handle = raw_tracker_handle(handle);
        let (num_channels, pattern_info, is_it, old_effects, link_g) = {
            let loaded = match self.modules.get(raw_handle as usize).and_then(|m| m.as_ref()) {
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
            (loaded.module.num_channels, notes, is_it, old_effects, link_g)
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
                let loaded = match self.modules.get(raw_handle as usize).and_then(|m| m.as_ref()) {
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
                    (sound_handle, instr.sample_loop_start, instr.sample_loop_end, loop_type, instr.sample_finetune)
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
            // Fetch all instrument data we need for note trigger
            let instr_data = {
                let loaded = match self.modules.get(raw_handle as usize).and_then(|m| m.as_ref()) {
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
                                env.points.get(env.sustain_begin as usize).map(|(tick, _)| *tick)
                            } else {
                                None
                            };
                            let loop_range = if env.has_loop() {
                                let start = env.points.get(env.loop_begin as usize).map(|(tick, _)| *tick).unwrap_or(0);
                                let end = env.points.get(env.loop_end as usize).map(|(tick, _)| *tick).unwrap_or(0);
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
                                env.points.get(env.sustain_begin as usize).map(|(tick, _)| *tick)
                            } else {
                                None
                            };
                            let loop_range = if env.has_loop() {
                                let start = env.points.get(env.loop_begin as usize).map(|(tick, _)| *tick).unwrap_or(0);
                                let end = env.points.get(env.loop_end as usize).map(|(tick, _)| *tick).unwrap_or(0);
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
            ) = instr_data.unwrap_or((0, 0, 0, 0, 0, 0, 0, 0, 0, 0, false, None, None, false, None, None));

            let channel = &mut self.channels[ch_idx];
            channel.note_on = true;
            channel.key_off = false;
            channel.sample_pos = 0.0;
            channel.sample_direction = 1;
            channel.volume_envelope_pos = 0;
            channel.panning_envelope_pos = 0;
            channel.volume_fadeout = 65535;
            channel.fade_out_samples = 0; // Cancel any fade-out
            channel.fade_in_samples = FADE_IN_SAMPLES; // Start fade-in for crossfade
            // Note: prev_sample is preserved for crossfade blending

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
            self.channels[ch_idx].volume = note.volume as f32 / 64.0;
        }

        // Volume column effects are already converted to TrackerEffect during parsing
        // and are in the effect field along with other effects

        // Handle effects (tick 0 processing)
        // TrackerEffect is an enum, not u8 + param
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
            TrackerEffect::None => {},

            // =====================================================================
            // Speed and Tempo (handled by caller via return value in legacy code)
            // =====================================================================
            TrackerEffect::SetSpeed(_) | TrackerEffect::SetTempo(_) => {
                // These modify TrackerState, handled in FFI layer
            }

            TrackerEffect::TempoSlideUp(amount) => {
                // T1x = slide tempo up by x BPM per tick
                self.tempo_slide = *amount as i8;
            }

            TrackerEffect::TempoSlideDown(amount) => {
                // T0x = slide tempo down by x BPM per tick
                self.tempo_slide = -(*amount as i8);
            }

            // =====================================================================
            // Pattern Flow (handled by caller)
            // =====================================================================
            TrackerEffect::PositionJump(_) | TrackerEffect::PatternBreak(_) => {
                // Handled by caller after row processing
            }
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
                // S6x - adds x extra ticks to the current row
                // Unlike SEx (pattern delay) which repeats the row, this just extends the tick count
                self.fine_pattern_delay = *ticks;
            }

            TrackerEffect::HighSampleOffset(value) => {
                // SAx - sets high byte for next Oxx command
                channel.sample_offset_high = *value;
            }

            // =====================================================================
            // Volume Effects
            // =====================================================================
            TrackerEffect::SetVolume(vol) => {
                channel.volume = ((*vol).min(64) as f32) / 64.0;
            }
            TrackerEffect::VolumeSlide { up, down } => {
                channel.volume_slide_active = true;
                let param = (*up << 4) | *down;
                if param != 0 {
                    channel.last_volume_slide = param;
                }
            }
            TrackerEffect::FineVolumeUp(val) => {
                channel.volume = (channel.volume + *val as f32 / 64.0).min(1.0);
            }
            TrackerEffect::FineVolumeDown(val) => {
                channel.volume = (channel.volume - *val as f32 / 64.0).max(0.0);
            }
            TrackerEffect::SetGlobalVolume(vol) => {
                // Global volume is 0-128 in unified format (XM: 0-64 * 2, IT: native 0-128)
                self.global_volume = ((*vol).min(128) as f32) / 128.0;
            }
            TrackerEffect::GlobalVolumeSlide { up, down } => {
                let param = (*up << 4) | *down;
                if param != 0 {
                    self.last_global_vol_slide = param;
                }
            }
            TrackerEffect::FineGlobalVolumeUp(val) => {
                // Fine global volume slide up - applies on tick 0 only
                self.global_volume = (self.global_volume + *val as f32 / 64.0).min(1.0);
            }
            TrackerEffect::FineGlobalVolumeDown(val) => {
                // Fine global volume slide down - applies on tick 0 only
                self.global_volume = (self.global_volume - *val as f32 / 64.0).max(0.0);
            }
            TrackerEffect::SetChannelVolume(vol) => {
                channel.channel_volume = (*vol).min(64);
            }
            TrackerEffect::ChannelVolumeSlide { up, down } => {
                channel.channel_volume_slide_active = true;
                let param = (*up << 4) | *down;
                if param != 0 {
                    channel.channel_volume_slide = if *up > 0 {
                        *up as i8
                    } else {
                        -(*down as i8)
                    };
                }
            }
            TrackerEffect::FineChannelVolumeUp(val) => {
                // Fine channel volume slide up - applies on tick 0 only
                channel.channel_volume = channel.channel_volume.saturating_add(*val).min(64);
            }
            TrackerEffect::FineChannelVolumeDown(val) => {
                // Fine channel volume slide down - applies on tick 0 only
                channel.channel_volume = channel.channel_volume.saturating_sub(*val);
            }

            // =====================================================================
            // Pitch Effects
            // =====================================================================
            TrackerEffect::PortamentoUp(val) => {
                channel.porta_up_active = true;
                let v = *val as u8;
                if v != 0 {
                    channel.last_porta_up = v;
                    // Update shared E/F/G memory when LINK_G_MEMORY is set
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
                    // Update shared E/F/G memory when LINK_G_MEMORY is set
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
                channel.period = (channel.period - channel.last_fine_porta_up as f32 * 4.0).max(1.0);
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
                    // Update shared E/F/G memory when LINK_G_MEMORY is set
                    if self.link_g_memory {
                        channel.shared_efg_memory = v;
                    }
                } else if self.link_g_memory && channel.shared_efg_memory != 0 {
                    // When G00 and LINK_G_MEMORY, use shared E/F/G memory
                    channel.porta_speed = channel.shared_efg_memory;
                }
                // Set target period from note if a note was triggered
                if note_num > 0 && note_num <= 96 {
                    channel.target_period = note_to_period(note_num, channel.finetune);
                }
            }
            TrackerEffect::TonePortaVolSlide { porta: _, vol_up, vol_down } => {
                channel.tone_porta_active = true;
                channel.volume_slide_active = true;
                let param = (*vol_up << 4) | *vol_down;
                if param != 0 {
                    channel.last_volume_slide = param;
                }
                // Portamento uses memory, don't update porta_speed
            }

            // =====================================================================
            // Modulation Effects
            // =====================================================================
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
            TrackerEffect::VibratoVolSlide { vib_speed: _, vib_depth: _, vol_up, vol_down } => {
                channel.vibrato_active = true;
                channel.volume_slide_active = true;
                let param = (*vol_up << 4) | *vol_down;
                if param != 0 {
                    channel.last_volume_slide = param;
                }
                // Vibrato uses memory
            }
            TrackerEffect::FineVibrato { speed, depth } => {
                channel.vibrato_active = true;
                if *speed != 0 {
                    channel.vibrato_speed = *speed;
                }
                if *depth != 0 {
                    // Fine vibrato has 4x smaller depth
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

            // =====================================================================
            // Panning Effects
            // =====================================================================
            TrackerEffect::SetPanning(pan) => {
                // TrackerEffect uses 0-64 panning, convert to -1.0 to 1.0
                channel.panning = (*pan as f32 / 64.0) * 2.0 - 1.0;
            }
            TrackerEffect::PanningSlide { left, right } => {
                channel.panning_slide_active = true;
                // Store for per-tick processing
                // Positive = right, negative = left
                channel.panning_slide = (*right as i8) - (*left as i8);
            }
            TrackerEffect::FinePanningRight(amount) => {
                // Fine panning slide right - apply immediately on tick 0 only
                // Panning is -1.0 to 1.0, amount is 0-15
                channel.panning = (channel.panning + *amount as f32 / 64.0).clamp(-1.0, 1.0);
            }
            TrackerEffect::FinePanningLeft(amount) => {
                // Fine panning slide left - apply immediately on tick 0 only
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

            // =====================================================================
            // Sample Effects
            // =====================================================================
            TrackerEffect::SampleOffset(offset) => {
                // TrackerEffect stores full offset, but we need to extract for memory
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
            TrackerEffect::Retrigger { ticks, volume_change } => {
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

            // =====================================================================
            // Filter Effects (IT only)
            // =====================================================================
            TrackerEffect::SetFilterCutoff(cutoff) => {
                channel.filter_cutoff = *cutoff as f32 / 127.0;
                channel.filter_dirty = true;
            }
            TrackerEffect::SetFilterResonance(res) => {
                channel.filter_resonance = *res as f32 / 127.0;
                channel.filter_dirty = true;
            }

            // =====================================================================
            // Waveform Control
            // =====================================================================
            TrackerEffect::VibratoWaveform(wf) => {
                channel.vibrato_waveform = *wf & 0x07;
            }
            TrackerEffect::TremoloWaveform(wf) => {
                channel.tremolo_waveform = *wf & 0x07;
            }
            TrackerEffect::PanbrelloWaveform(wf) => {
                channel.panbrello_waveform = *wf & 0x07;
            }

            // =====================================================================
            // Other Effects
            // =====================================================================
            TrackerEffect::SetEnvelopePosition(pos) => {
                channel.volume_envelope_pos = *pos as u16;
            }
            TrackerEffect::KeyOff => {
                channel.key_off = true;
            }
            TrackerEffect::SetGlissando(enabled) => {
                channel.glissando = *enabled;
            }
            TrackerEffect::MultiRetrigNote { ticks, volume } => {
                channel.retrigger_tick = *ticks;
                channel.retrigger_mode = *volume;
                // Convert volume mode to additive delta
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

            // Apply per-tick effects based on stored parameters
            // Effects only apply if they were present on this row (active flag set)

            // Arpeggio - only apply if arpeggio effect is active this row
            if channel.arpeggio_active && (channel.arpeggio_note1 != 0 || channel.arpeggio_note2 != 0) {
                channel.arpeggio_tick = ((channel.arpeggio_tick as u16 + 1) % 3) as u8;
                let note_offset = match channel.arpeggio_tick {
                    0 => 0,
                    1 => channel.arpeggio_note1,
                    _ => channel.arpeggio_note2,
                };
                // Adjust period for arpeggio (each semitone is 16*4 = 64 period units)
                let arp_period = channel.base_period - note_offset as f32 * 64.0;
                channel.period = arp_period.max(1.0);
            }

            // Volume slide - only apply if volume slide effect is active this row
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

            // Channel volume slide (IT only) - apply if active this row
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

            // Portamento up - only apply if portamento up effect is active this row
            if channel.porta_up_active && channel.last_porta_up != 0 {
                if self.is_it_format {
                    // IT linear slide: freq = freq * 2^(slide/768)
                    channel.period = apply_it_linear_slide(channel.period, channel.last_porta_up as i16);
                } else {
                    // XM linear period slide
                    channel.period = (channel.period - channel.last_porta_up as f32 * 4.0).max(1.0);
                }
            }

            // Portamento down - only apply if portamento down effect is active this row
            if channel.porta_down_active && channel.last_porta_down != 0 {
                if self.is_it_format {
                    // IT linear slide: freq = freq / 2^(slide/768)
                    channel.period = apply_it_linear_slide(channel.period, -(channel.last_porta_down as i16));
                } else {
                    // XM linear period slide
                    channel.period += channel.last_porta_down as f32 * 4.0;
                }
            }

            // Tone portamento (slide toward target) - only apply if tone porta is active
            if channel.tone_porta_active && channel.target_period > 0.0 && channel.porta_speed > 0 {
                let diff = channel.target_period - channel.period;
                if self.is_it_format {
                    // IT linear tone portamento
                    let slide = channel.porta_speed as i16;
                    if diff > 0.0 {
                        // Slide down (toward lower frequency, higher period)
                        let new_period = apply_it_linear_slide(channel.period, -slide);
                        if new_period >= channel.target_period {
                            channel.period = channel.target_period;
                        } else {
                            channel.period = new_period;
                        }
                    } else if diff < 0.0 {
                        // Slide up (toward higher frequency, lower period)
                        let new_period = apply_it_linear_slide(channel.period, slide);
                        if new_period <= channel.target_period {
                            channel.period = channel.target_period;
                        } else {
                            channel.period = new_period;
                        }
                    }
                } else {
                    // XM linear period slide
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

            // Vibrato - only apply if vibrato is active
            // XM depth: 128.0/15.0 ≈ 8.533 gives ±2 semitones (±128 period units) at depth=15
            // IT depth: 4x finer than XM (per ITTECH.TXT spec)
            // OLD_EFFECTS: Use S3M-compatible (coarser) depth like XM
            // Speed: 4x faster oscillation to match libxm/FT2
            if channel.vibrato_active && channel.vibrato_depth > 0 {
                let vibrato = get_waveform_value(channel.vibrato_waveform, channel.vibrato_pos);
                // IT vibrato is 4x finer than XM, unless OLD_EFFECTS mode
                let depth_scale = if self.is_it_format && !self.old_effects_mode {
                    32.0 / 15.0
                } else {
                    128.0 / 15.0
                };
                let delta = vibrato * channel.vibrato_depth as f32 * depth_scale;
                channel.period = channel.base_period + delta;
                // Update position for 256-point table (speed * 4 gives good wrap behavior)
                channel.vibrato_pos =
                    channel.vibrato_pos.wrapping_add(channel.vibrato_speed << 2);
            }

            // Auto-vibrato (instrument vibrato, applied in addition to pattern vibrato)
            // Rate is 4x slower than pattern vibrato, uses sweep envelope to ramp in
            if channel.auto_vibrato_depth > 0 {
                // Get waveform value using stored instrument waveform type
                let auto_vib = get_waveform_value(channel.auto_vibrato_type, (channel.auto_vibrato_pos >> 2) as u8);

                // Apply sweep envelope (ramps in over vibrato_sweep ticks)
                let sweep_factor = if channel.auto_vibrato_sweep_len > 0 {
                    let sweep_progress = channel.auto_vibrato_sweep as f32
                        / (channel.auto_vibrato_sweep_len as f32 * 256.0);
                    sweep_progress.min(1.0)
                } else {
                    1.0
                };

                // Apply auto-vibrato to period (adds to current period, not base)
                // IT vibrato is 4x finer than XM, unless OLD_EFFECTS mode
                let depth_scale = if self.is_it_format && !self.old_effects_mode {
                    32.0 / 15.0
                } else {
                    128.0 / 15.0
                };
                let delta = auto_vib * channel.auto_vibrato_depth as f32 * sweep_factor * depth_scale;
                channel.period += delta;

                // Advance auto-vibrato position (slower rate than pattern vibrato)
                channel.auto_vibrato_pos = channel.auto_vibrato_pos.wrapping_add(channel.auto_vibrato_rate as u16);

                // Advance sweep
                if channel.auto_vibrato_sweep < 65535 {
                    channel.auto_vibrato_sweep = channel.auto_vibrato_sweep.saturating_add(1);
                }
            }

            // Tremolo (FT2-compatible depth and speed) - only apply if tremolo is active
            // Depth: * 4.0 / 128.0 matches libxm formula
            // Speed: 4x faster oscillation to match libxm/FT2
            if channel.tremolo_active && channel.tremolo_depth > 0 {
                let tremolo = get_waveform_value(channel.tremolo_waveform, channel.tremolo_pos);
                let delta = tremolo * channel.tremolo_depth as f32 * 4.0 / 128.0;
                channel.volume = (channel.volume + delta).clamp(0.0, 1.0);
                // Update position for 256-point table
                channel.tremolo_pos =
                    channel.tremolo_pos.wrapping_add(channel.tremolo_speed << 2);
            }

            // Retrigger - note: retrigger_tick is reset per-row in reset_row_effects
            if channel.retrigger_tick > 0 && tick % channel.retrigger_tick as u16 == 0 {
                channel.sample_pos = 0.0;
                // Apply volume change based on retrigger mode
                match channel.retrigger_mode {
                    // Multiplicative modes
                    6 => channel.volume = (channel.volume * (2.0 / 3.0)).clamp(0.0, 1.0),
                    7 => channel.volume = (channel.volume * 0.5).clamp(0.0, 1.0),
                    14 => channel.volume = (channel.volume * 1.5).clamp(0.0, 1.0),
                    15 => channel.volume = (channel.volume * 2.0).clamp(0.0, 1.0),
                    // Additive modes (use stored delta)
                    _ => {
                        if channel.retrigger_volume != 0 {
                            channel.volume =
                                (channel.volume + channel.retrigger_volume as f32 / 64.0)
                                    .clamp(0.0, 1.0);
                        }
                    }
                }
            }

            // Panning slide - only apply if panning slide is active this row
            if channel.panning_slide_active && channel.panning_slide != 0 {
                channel.panning =
                    (channel.panning + channel.panning_slide as f32 / 255.0).clamp(-1.0, 1.0);
            }

            // Tremor (IT Ixy) - rapidly switch volume on/off
            if channel.tremor_active && (channel.tremor_on_ticks > 0 || channel.tremor_off_ticks > 0) {
                channel.tremor_counter = channel.tremor_counter.saturating_add(1);
                if channel.tremor_mute {
                    // Currently in off phase
                    if channel.tremor_counter >= channel.tremor_off_ticks {
                        channel.tremor_mute = false;
                        channel.tremor_counter = 0;
                    }
                } else {
                    // Currently in on phase
                    if channel.tremor_counter >= channel.tremor_on_ticks {
                        channel.tremor_mute = true;
                        channel.tremor_counter = 0;
                    }
                }
            }

            // Panbrello (IT Yxy) - oscillate panning
            if channel.panbrello_active && channel.panbrello_depth > 0 {
                let panbrello = get_waveform_value(channel.panbrello_waveform, channel.panbrello_pos);
                let delta = panbrello * channel.panbrello_depth as f32 / 64.0;
                channel.panning = (channel.panning + delta).clamp(-1.0, 1.0);
                channel.panbrello_pos = channel.panbrello_pos.wrapping_add(channel.panbrello_speed);
            }

            // Note cut (ECx) - cut note at tick x
            if channel.note_cut_tick > 0 && tick == channel.note_cut_tick as u16 {
                channel.volume = 0.0;
                channel.note_on = false;
            }

            // Note delay (EDx) - trigger note at tick x
            // Note: The actual note triggering with instrument data happens in process_note
            // Here we just set the period/volume if delayed note data is stored
            if channel.note_delay_tick > 0 && tick == channel.note_delay_tick as u16 {
                if channel.delayed_note > 0 && channel.delayed_note <= 96 {
                    // Reset sample position and trigger note-like behavior
                    channel.sample_pos = 0.0;
                    channel.note_on = true;
                    channel.key_off = false;
                    channel.volume_envelope_pos = 0;
                    channel.panning_envelope_pos = 0;
                    channel.volume_fadeout = 65535;
                    // Reset vibrato/tremolo positions
                    if channel.vibrato_waveform < 4 {
                        channel.vibrato_pos = 0;
                    }
                    if channel.tremolo_waveform < 4 {
                        channel.tremolo_pos = 0;
                    }
                    // Set period from delayed note
                    channel.base_period = note_to_period(channel.delayed_note, channel.finetune);
                    channel.period = channel.base_period;
                }
                // Clear the delay tick so it doesn't trigger again
                channel.note_delay_tick = 0;
            }

            // Key off timing (Kxx) - key off at tick x
            if channel.key_off_tick > 0 && tick == channel.key_off_tick as u16 {
                channel.key_off = true;
            }

            // Volume column effects (per-tick)
            match channel.vol_col_effect {
                0x6 => {
                    // Volume slide down
                    channel.volume =
                        (channel.volume - channel.vol_col_param as f32 / 64.0).max(0.0);
                }
                0x7 => {
                    // Volume slide up
                    channel.volume =
                        (channel.volume + channel.vol_col_param as f32 / 64.0).min(1.0);
                }
                0xB => {
                    // Vibrato with set depth (vibrato already applied above if depth > 0)
                    channel.vibrato_depth = channel.vol_col_param;
                }
                0xD => {
                    // Panning slide left
                    channel.panning =
                        (channel.panning - channel.vol_col_param as f32 / 16.0).clamp(-1.0, 1.0);
                }
                0xE => {
                    // Panning slide right
                    channel.panning =
                        (channel.panning + channel.vol_col_param as f32 / 16.0).clamp(-1.0, 1.0);
                }
                0xF => {
                    // Tone portamento from volume column - porta_speed already set on tick 0
                    // The actual tone portamento is handled above in the tone portamento section
                }
                _ => {}
            }

            // Glissando - round period to semitone if enabled during tone portamento
            if channel.glissando && channel.target_period > 0.0 {
                // Round to nearest 64 period units (one semitone)
                channel.period = (channel.period / 64.0).round() * 64.0;
            }

            // Volume envelope advancement
            if channel.volume_envelope_enabled {
                // Check sustain - don't advance past sustain point unless key-off
                let at_sustain = if let Some(sus_tick) = channel.volume_envelope_sustain_tick {
                    channel.volume_envelope_pos >= sus_tick && !channel.key_off
                } else {
                    false
                };

                if !at_sustain {
                    channel.volume_envelope_pos += 1;
                }

                // Handle envelope loop
                if let Some((loop_start, loop_end)) = channel.volume_envelope_loop {
                    if channel.volume_envelope_pos >= loop_end {
                        channel.volume_envelope_pos = loop_start;
                    }
                }
            }

            // Panning envelope advancement
            if channel.panning_envelope_enabled {
                // Check sustain
                let at_sustain = if let Some(sus_tick) = channel.panning_envelope_sustain_tick {
                    channel.panning_envelope_pos >= sus_tick && !channel.key_off
                } else {
                    false
                };

                if !at_sustain {
                    channel.panning_envelope_pos += 1;
                }

                // Handle envelope loop
                if let Some((loop_start, loop_end)) = channel.panning_envelope_loop {
                    if channel.panning_envelope_pos >= loop_end {
                        channel.panning_envelope_pos = loop_start;
                    }
                }
            }

            // Pitch envelope advancement (IT only)
            if channel.pitch_envelope_enabled {
                // Pitch envelope has same sustain logic
                let at_sustain = if let Some(sus_tick) = channel.pitch_envelope_sustain_tick {
                    channel.pitch_envelope_pos >= sus_tick && !channel.key_off
                } else {
                    false
                };

                if !at_sustain {
                    channel.pitch_envelope_pos += 1;
                }

                // Handle envelope loop
                if let Some((loop_start, loop_end)) = channel.pitch_envelope_loop {
                    if channel.pitch_envelope_pos >= loop_end {
                        channel.pitch_envelope_pos = loop_start;
                    }
                }
            }

            // Filter envelope advancement (IT only)
            if channel.filter_envelope_enabled {
                // Filter envelope has same sustain logic
                let at_sustain = if let Some(sus_tick) = channel.filter_envelope_sustain_tick {
                    channel.filter_envelope_pos >= sus_tick && !channel.key_off
                } else {
                    false
                };

                if !at_sustain {
                    channel.filter_envelope_pos += 1;
                }

                // Handle envelope loop
                if let Some((loop_start, loop_end)) = channel.filter_envelope_loop {
                    if channel.filter_envelope_pos >= loop_end {
                        channel.filter_envelope_pos = loop_start;
                    }
                }
            }

            // Volume fadeout after key-off
            if channel.key_off && channel.instrument_fadeout_rate > 0 {
                channel.volume_fadeout =
                    channel.volume_fadeout.saturating_sub(channel.instrument_fadeout_rate);

                // When fadeout reaches 0, stop the note
                if channel.volume_fadeout == 0 {
                    channel.note_on = false;
                }
            }
        }

        // Global volume slide (Hxy) - applied outside channel loop
        if self.last_global_vol_slide != 0 {
            let up = (self.last_global_vol_slide >> 4) as f32 / 64.0;
            let down = (self.last_global_vol_slide & 0x0F) as f32 / 64.0;
            if up > 0.0 {
                self.global_volume = (self.global_volume + up).min(1.0);
            } else if down > 0.0 {
                self.global_volume = (self.global_volume - down).max(0.0);
            }
        }
    }

    /// Render one stereo sample from the tracker
    ///
    /// Returns (left, right) sample values.
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
        let module = match self
            .modules
            .get(raw_handle as usize)
            .and_then(|m| m.as_ref())
        {
            Some(m) => m,
            None => return (0.0, 0.0),
        };

        let mut left = 0.0f32;
        let mut right = 0.0f32;

        // Mix all active channels
        for (ch_idx, channel) in self.channels.iter_mut().enumerate() {
            if ch_idx >= module.module.num_channels as usize {
                break;
            }

            if !channel.note_on || channel.sample_handle == 0 {
                continue;
            }

            // Get sound data
            let sound = match sounds
                .get(channel.sample_handle as usize)
                .and_then(|s| s.as_ref())
            {
                Some(s) => s,
                None => continue,
            };

            // Apply pitch envelope (IT only) - modifies period before sampling
            if channel.pitch_envelope_enabled {
                let instr_idx = channel.instrument.saturating_sub(1) as usize;
                if let Some(instr) = module.module.instruments.get(instr_idx) {
                    if let Some(ref env) = instr.pitch_envelope {
                        if env.is_enabled() && !env.is_filter() {
                            // Pitch envelope value is in half-semitones (-32 to +32)
                            let env_val = env.value_at(channel.pitch_envelope_pos) as f32;
                            channel.pitch_envelope_value = env_val;
                        }
                    }
                }
            }

            // Update filter envelope (IT only) - modifies filter cutoff
            if channel.filter_envelope_enabled {
                let instr_idx = channel.instrument.saturating_sub(1) as usize;
                if let Some(instr) = module.module.instruments.get(instr_idx) {
                    if let Some(ref env) = instr.pitch_envelope {
                        if env.is_filter() {
                            // Filter envelope controls cutoff (0-64 maps to 0.0-1.0)
                            let env_val = env.value_at(channel.filter_envelope_pos) as f32;
                            channel.filter_cutoff = (env_val / 64.0).clamp(0.0, 1.0);
                            channel.filter_dirty = true;
                        }
                    }
                }
            }

            // Sample with interpolation
            let mut sample = sample_channel(channel, &sound.data, sample_rate);

            // Apply resonant low-pass filter (IT only)
            if channel.filter_cutoff < 1.0 {
                sample = channel.apply_filter(sample);
            }

            // Apply volume with envelope processing
            let mut vol = channel.volume;

            // Apply volume envelope if enabled
            if channel.volume_envelope_enabled {
                let instr_idx = channel.instrument.saturating_sub(1) as usize;
                if let Some(instr) = module.module.instruments.get(instr_idx) {
                    if let Some(ref env) = instr.volume_envelope {
                        if env.is_enabled() {
                            let env_val = env.value_at(channel.volume_envelope_pos) as f32 / 64.0;
                            vol *= env_val;
                        }
                    }
                }
            }

            // Apply volume fadeout after key-off
            if channel.key_off {
                vol *= channel.volume_fadeout as f32 / 65535.0;
            }

            vol *= self.global_volume;

            // Apply channel volume (IT feature)
            vol *= channel.channel_volume as f32 / 64.0;

            // Apply instrument global volume (IT feature)
            vol *= channel.instrument_global_volume as f32 / 64.0;

            // Apply tremor mute (IT feature)
            if channel.tremor_mute {
                vol = 0.0;
            }

            // Apply panning with envelope
            let mut pan = channel.panning;

            // Apply pitch-pan separation (IT feature)
            // Formula: NotePan = NotePan + (Note - PPCenter) × PPSeparation / 8
            // IT uses 0-64 panning, we use -1.0 to 1.0, so divide by additional 32
            if channel.pitch_pan_separation != 0 {
                let note_offset = channel.current_note as i16 - channel.pitch_pan_center as i16;
                let pan_offset = (note_offset * channel.pitch_pan_separation as i16) as f32 / 256.0;
                pan = (pan + pan_offset).clamp(-1.0, 1.0);
            }

            if channel.panning_envelope_enabled {
                let instr_idx = channel.instrument.saturating_sub(1) as usize;
                if let Some(instr) = module.module.instruments.get(instr_idx) {
                    if let Some(ref env) = instr.panning_envelope {
                        if env.is_enabled() {
                            // Panning envelope: 0-64 maps to -1.0 to 1.0 (32 = center)
                            let env_val = env.value_at(channel.panning_envelope_pos) as f32;
                            pan = (env_val - 32.0) / 32.0;
                        }
                    }
                }
            }

            // Apply panbrello offset (IT feature)
            // Calculate panbrello offset from depth and waveform position
            if channel.panbrello_active && channel.panbrello_depth > 0 {
                let waveform_value = SINE_LUT[(channel.panbrello_pos >> 4) as usize & 0xF] as f32;
                let panbrello_offset = (waveform_value * channel.panbrello_depth as f32) / (64.0 * 256.0);
                pan = (pan + panbrello_offset).clamp(-1.0, 1.0);
            }

            let (l, r) = apply_channel_pan(sample * vol, pan);
            left += l;
            right += r;
        }

        // Scale by tracker volume
        let vol = state.volume as f32 / 256.0;
        (left * vol, right * vol)
    }

    /// Render one stereo sample and advance the tracker state
    ///
    /// This handles the complete playback loop:
    /// - Renders audio for the current position
    /// - Advances tick_sample_pos
    /// - When tick completes, advances tick and processes effects
    /// - When row completes, advances row and processes notes
    /// - When pattern completes, advances to next order
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

        // Process tick 0 at the start of a row (trigger notes, process effects)
        if state.tick == 0 && state.tick_sample_pos == 0 {
            self.process_row_tick0_internal(state.handle, sounds);
        }

        // Render the audio sample
        let raw_handle = raw_tracker_handle(state.handle);
        let module = match self
            .modules
            .get(raw_handle as usize)
            .and_then(|m| m.as_ref())
        {
            Some(m) => m,
            None => return (0.0, 0.0),
        };

        let mut left = 0.0f32;
        let mut right = 0.0f32;

        // Mix all active channels
        for (ch_idx, channel) in self.channels.iter_mut().enumerate() {
            if ch_idx >= module.module.num_channels as usize {
                break;
            }

            if !channel.note_on || channel.sample_handle == 0 {
                continue;
            }

            // Get sound data
            let sound = match sounds
                .get(channel.sample_handle as usize)
                .and_then(|s| s.as_ref())
            {
                Some(s) => s,
                None => continue,
            };

            // Apply pitch envelope (IT only) - modifies period before sampling
            if channel.pitch_envelope_enabled {
                let instr_idx = channel.instrument.saturating_sub(1) as usize;
                if let Some(instr) = module.module.instruments.get(instr_idx) {
                    if let Some(ref env) = instr.pitch_envelope {
                        if env.is_enabled() && !env.is_filter() {
                            // Pitch envelope value is in half-semitones (-32 to +32)
                            let env_val = env.value_at(channel.pitch_envelope_pos) as f32;
                            channel.pitch_envelope_value = env_val;
                        }
                    }
                }
            }

            // Update filter envelope (IT only) - modifies filter cutoff
            if channel.filter_envelope_enabled {
                let instr_idx = channel.instrument.saturating_sub(1) as usize;
                if let Some(instr) = module.module.instruments.get(instr_idx) {
                    if let Some(ref env) = instr.pitch_envelope {
                        if env.is_filter() {
                            // Filter envelope controls cutoff (0-64 maps to 0.0-1.0)
                            let env_val = env.value_at(channel.filter_envelope_pos) as f32;
                            channel.filter_cutoff = (env_val / 64.0).clamp(0.0, 1.0);
                            channel.filter_dirty = true;
                        }
                    }
                }
            }

            // Sample with interpolation
            let mut sample = sample_channel(channel, &sound.data, sample_rate);

            // Apply resonant low-pass filter (IT only)
            if channel.filter_cutoff < 1.0 {
                sample = channel.apply_filter(sample);
            }

            // Apply volume with envelope processing
            let mut vol = channel.volume;

            // Apply volume envelope if enabled
            if channel.volume_envelope_enabled {
                let instr_idx = channel.instrument.saturating_sub(1) as usize;
                if let Some(instr) = module.module.instruments.get(instr_idx) {
                    if let Some(ref env) = instr.volume_envelope {
                        if env.is_enabled() {
                            let env_val = env.value_at(channel.volume_envelope_pos) as f32 / 64.0;
                            vol *= env_val;
                        }
                    }
                }
            }

            // Apply volume fadeout after key-off
            if channel.key_off {
                vol *= channel.volume_fadeout as f32 / 65535.0;
            }

            vol *= self.global_volume;

            // Apply channel volume (IT feature)
            vol *= channel.channel_volume as f32 / 64.0;

            // Apply instrument global volume (IT feature)
            vol *= channel.instrument_global_volume as f32 / 64.0;

            // Apply tremor mute (IT feature)
            if channel.tremor_mute {
                vol = 0.0;
            }

            // Apply panning with envelope
            let mut pan = channel.panning;

            // Apply pitch-pan separation (IT feature)
            // Formula: NotePan = NotePan + (Note - PPCenter) × PPSeparation / 8
            // IT uses 0-64 panning, we use -1.0 to 1.0, so divide by additional 32
            if channel.pitch_pan_separation != 0 {
                let note_offset = channel.current_note as i16 - channel.pitch_pan_center as i16;
                let pan_offset = (note_offset * channel.pitch_pan_separation as i16) as f32 / 256.0;
                pan = (pan + pan_offset).clamp(-1.0, 1.0);
            }

            if channel.panning_envelope_enabled {
                let instr_idx = channel.instrument.saturating_sub(1) as usize;
                if let Some(instr) = module.module.instruments.get(instr_idx) {
                    if let Some(ref env) = instr.panning_envelope {
                        if env.is_enabled() {
                            // Panning envelope: 0-64 maps to -1.0 to 1.0 (32 = center)
                            let env_val = env.value_at(channel.panning_envelope_pos) as f32;
                            pan = (env_val - 32.0) / 32.0;
                        }
                    }
                }
            }

            let (l, r) = apply_channel_pan(sample * vol, pan);
            left += l;
            right += r;
        }

        // Advance tick position
        state.tick_sample_pos += 1;
        let spt = samples_per_tick(state.bpm, sample_rate);

        if state.tick_sample_pos >= spt {
            state.tick_sample_pos = 0;
            state.tick += 1;

            // Process per-tick effects (not on tick 0)
            if state.tick > 0 {
                self.process_tick(state.tick, state.speed);

                // Apply tempo slide (IT Txy where x<2)
                // Slides BPM by the slide amount each tick
                if self.tempo_slide != 0 {
                    let new_bpm = (state.bpm as i16 + self.tempo_slide as i16).clamp(32, 255) as u16;
                    state.bpm = new_bpm;
                }
            }

            // Check if we need to advance to next row
            // S6x (fine pattern delay) extends the row by extra ticks
            let effective_speed = state.speed + self.fine_pattern_delay as u16;
            if state.tick >= effective_speed {
                state.tick = 0;
                self.fine_pattern_delay = 0; // Reset fine pattern delay for next row

                // Pattern delay (EEx): repeat current row for pattern_delay additional times
                if self.pattern_delay > 0 {
                    if self.pattern_delay_count < self.pattern_delay {
                        self.pattern_delay_count += 1;
                        // Don't advance row, just repeat it
                        // Next tick will re-process the same row
                        let vol = state.volume as f32 / 256.0;
                        return (left * vol, right * vol);
                    } else {
                        // Delay complete, reset and advance normally
                        self.pattern_delay = 0;
                        self.pattern_delay_count = 0;
                    }
                }

                state.row += 1;

                // Sync engine's current position
                self.current_row = state.row;

                // Check if we need to advance to next pattern
                let (num_rows, song_length, restart_position) = {
                    let loaded = match self
                        .modules
                        .get(raw_handle as usize)
                        .and_then(|m| m.as_ref())
                    {
                        Some(m) => m,
                        None => {
                            return (
                                left * state.volume as f32 / 256.0,
                                right * state.volume as f32 / 256.0,
                            );
                        }
                    };
                    let num_rows = loaded
                        .module
                        .pattern_at_order(state.order_position)
                        .map(|p| p.num_rows)
                        .unwrap_or(64);
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

                    // Check for end of song
                    if state.order_position >= song_length {
                        if (state.flags & tracker_flags::LOOPING) != 0 {
                            state.order_position = restart_position;
                            self.current_order = restart_position;
                        } else {
                            // Stop playback
                            state.flags &= !tracker_flags::PLAYING;
                        }
                    }
                }
            }
        }

        // Scale by tracker volume
        let vol = state.volume as f32 / 256.0;
        (left * vol, right * vol)
    }
}

/// Sample a channel with linear interpolation and anti-pop fade-in/out
fn sample_channel(channel: &mut TrackerChannel, data: &[i16], sample_rate: u32) -> f32 {
    if data.is_empty() {
        return 0.0;
    }

    // Handle fade-out phase (anti-pop when sample ends)
    if channel.fade_out_samples > 0 {
        let fade_ratio = channel.fade_out_samples as f32 / FADE_OUT_SAMPLES as f32;
        channel.fade_out_samples -= 1;

        // Fade from previous sample value to zero
        let sample = channel.prev_sample * fade_ratio;

        // When fade-out completes, stop the channel
        if channel.fade_out_samples == 0 {
            channel.note_on = false;
            channel.prev_sample = 0.0;
        }

        return sample;
    }

    let pos = channel.sample_pos as usize;
    let frac = (channel.sample_pos - pos as f64) as f32;

    // Get samples for interpolation
    let sample1 = if pos < data.len() {
        data[pos] as f32 / 32768.0
    } else {
        0.0
    };

    // For interpolation sample2, we need to handle the loop boundary correctly.
    // If we're at or past (loop_end - 1), we should wrap to loop_start.
    let sample2 = if channel.sample_loop_type != 0 && channel.sample_loop_end > channel.sample_loop_start {
        // Check if we're at the loop boundary (pos is the last sample before loop_end)
        let loop_end = channel.sample_loop_end as usize;
        if pos + 1 >= loop_end {
            // Wrap to loop start for smooth loop interpolation
            let loop_start = channel.sample_loop_start as usize;
            if loop_start < data.len() {
                data[loop_start] as f32 / 32768.0
            } else {
                sample1
            }
        } else if pos + 1 < data.len() {
            data[pos + 1] as f32 / 32768.0
        } else {
            sample1
        }
    } else if pos + 1 < data.len() {
        data[pos + 1] as f32 / 32768.0
    } else {
        sample1
    };

    let mut sample = sample1 + (sample2 - sample1) * frac;

    // Handle fade-in phase (crossfade from previous sample when new note triggers)
    if channel.fade_in_samples > 0 {
        let fade_ratio = 1.0 - (channel.fade_in_samples as f32 / FADE_IN_SAMPLES as f32);
        channel.fade_in_samples -= 1;

        // Crossfade: blend from previous sample value to new sample
        sample = channel.prev_sample * (1.0 - fade_ratio) + sample * fade_ratio;
    }

    // Store current sample for future crossfade (only update after fade-in complete)
    if channel.fade_in_samples == 0 {
        channel.prev_sample = sample;
    }

    // Calculate playback rate from period
    // XM frequency tells us the target playback frequency
    // Divide by output sample rate to get sample increment per output sample
    let freq = period_to_frequency(channel.period);
    let rate = freq / sample_rate as f32;

    // Advance sample position
    channel.sample_pos += rate as f64 * channel.sample_direction as f64;

    // Handle loop
    if channel.sample_loop_type != 0 && channel.sample_loop_end > channel.sample_loop_start {
        if channel.sample_direction > 0 && channel.sample_pos >= channel.sample_loop_end as f64 {
            if channel.sample_loop_type == 2 {
                // Ping-pong
                channel.sample_direction = -1;
                channel.sample_pos = channel.sample_loop_end as f64
                    - (channel.sample_pos - channel.sample_loop_end as f64);
            } else {
                // Forward loop
                channel.sample_pos = channel.sample_loop_start as f64
                    + (channel.sample_pos - channel.sample_loop_end as f64);
            }
        } else if channel.sample_direction < 0
            && channel.sample_pos < channel.sample_loop_start as f64
        {
            // Ping-pong reverse hit
            channel.sample_direction = 1;
            channel.sample_pos = channel.sample_loop_start as f64
                + (channel.sample_loop_start as f64 - channel.sample_pos);
        }
    } else if channel.sample_pos >= data.len() as f64 {
        // No loop - start fade-out instead of abrupt stop (anti-pop)
        channel.fade_out_samples = FADE_OUT_SAMPLES;
    }

    sample
}

/// Fast panning gains using the existing SINE_LUT with interpolation
///
/// Uses the 16-point sine LUT already defined for vibrato/tremolo.
/// cos(x) = sin(π/2 - x), so we read the LUT in reverse for left channel.
#[inline]
fn fast_pan_gains(pan: f32) -> (f32, f32) {
    // Map pan [-1, 1] to [0, 15] range for LUT indexing
    let pos = (pan + 1.0) * 7.5;
    let idx = (pos as usize).min(14);
    let frac = pos - idx as f32;

    // Linear interpolation between LUT points
    // Right channel uses sin (direct LUT), left uses cos (reversed LUT)
    let sin_val = SINE_LUT[idx] as f32 * (1.0 - frac) + SINE_LUT[idx + 1] as f32 * frac;
    let cos_val =
        SINE_LUT[15 - idx] as f32 * (1.0 - frac) + SINE_LUT[14 - idx.min(14)] as f32 * frac;

    // Scale from [0, 127] to [0, 1]
    (cos_val / 127.0, sin_val / 127.0)
}

/// Apply panning to a sample using fast LUT lookup
#[inline]
fn apply_channel_pan(sample: f32, pan: f32) -> (f32, f32) {
    let (left_gain, right_gain) = fast_pan_gains(pan);
    (sample * left_gain, sample * right_gain)
}

/// Calculate samples per tick from BPM
///
/// XM timing: samples_per_tick = sample_rate * 2.5 / bpm
fn samples_per_tick(bpm: u16, sample_rate: u32) -> u32 {
    if bpm == 0 {
        return sample_rate; // Fallback to 1 tick per second
    }
    (sample_rate * 5 / 2) / bpm as u32
}

/// 64-point quarter-sine lookup table for vibrato/tremolo (IT-compatible resolution)
/// Values represent sin(i * π/128) * 127 for i = 0..63
/// This gives 256 effective positions when mirrored across 4 quadrants
const SINE_LUT_64: [i8; 64] = [
    0, 2, 4, 6, 8, 10, 12, 14, 16, 18, 20, 22, 24, 26, 28, 30,
    32, 34, 36, 38, 40, 42, 44, 46, 48, 50, 52, 54, 56, 58, 60, 62,
    64, 66, 68, 70, 72, 74, 76, 78, 80, 82, 84, 86, 88, 89, 91, 93,
    95, 96, 98, 100, 101, 103, 104, 106, 107, 108, 110, 111, 112, 113, 114, 115,
];

/// Legacy 16-point quarter-sine for XM/FT2 compatibility (used for panning calculations)
const SINE_LUT: [i8; 16] = [0, 12, 24, 37, 48, 60, 71, 81, 90, 98, 106, 112, 118, 122, 125, 127];

/// Get waveform value for vibrato/tremolo
///
/// Uses IT-compatible 64-point quarter-sine table (256 effective positions)
///
/// Waveform types:
/// - 0: Sine (IT LUT with quadrant mirroring)
/// - 1: Ramp down (sawtooth)
/// - 2: Square
/// - 3: Random (deterministic pseudo-random)
fn get_waveform_value(waveform: u8, position: u8) -> f32 {
    let pos = position & 0xFF; // Full 256 positions for IT compatibility

    match waveform & 0x03 {
        0 => {
            // IT 256-point sine using 64-point quarter table with mirroring
            // Quarter 0 (0-63): ascending from 0 to peak
            // Quarter 1 (64-127): descending from peak to 0
            // Quarter 2 (128-191): ascending from 0 to -peak
            // Quarter 3 (192-255): descending from -peak to 0
            let quarter = pos >> 6; // 0-3
            let idx = (pos & 0x3F) as usize; // 0-63
            let val = match quarter {
                0 => SINE_LUT_64[idx],           // 0-63: ascending
                1 => SINE_LUT_64[63 - idx],     // 64-127: descending
                2 => -SINE_LUT_64[idx],          // 128-191: negative ascending
                _ => -SINE_LUT_64[63 - idx],    // 192-255: negative descending
            };
            val as f32 / 115.0 // Normalize to roughly -1.0 to 1.0
        }
        1 => {
            // Ramp down (sawtooth)
            // Position 0 = +1.0, position 128 = -1.0, position 255 = ~+1.0
            let ramp = 128i16 - (pos as i16);
            (ramp as f32) / 128.0
        }
        2 => {
            // Square wave: 1.0 for first half, -1.0 for second
            if pos < 128 { 1.0 } else { -1.0 }
        }
        _ => {
            // "Random" - deterministic pseudo-random using position as seed
            let x = position.wrapping_mul(0x9E) ^ 0x5C;
            (x as f32 / 127.5) - 1.0
        }
    }
}

/// Convert note number to period (linear frequency table)
///
/// XM linear period formula:
/// Period = 10*12*16*4 - Note*16*4 - FineTune/2
fn note_to_period(note: u8, finetune: i8) -> f32 {
    if note == 0 || note > 96 {
        return 0.0;
    }
    let n = (note - 1) as i32;
    let ft = finetune as i32;
    let period = 10 * 12 * 16 * 4 - n * 16 * 4 - ft / 2;
    period.max(1) as f32
}

/// Apply IT linear slide to period (ITTECH.TXT formula)
///
/// IT linear slides modify frequency by 2^(slide_amount/768) per tick.
/// Since period is inversely proportional to frequency:
/// - Pitch up: new_period = old_period / 2^(slide/768)
/// - Pitch down: new_period = old_period * 2^(slide/768)
///
/// Uses the LINEAR_FREQ_TABLE for accurate 2^(x/768) lookup.
#[inline]
fn apply_it_linear_slide(period: f32, slide: i16) -> f32 {
    if slide == 0 || period <= 0.0 {
        return period;
    }

    // Get 2^(|slide|/768) from the lookup table
    let abs_slide = slide.unsigned_abs() as usize;

    // For large slides, compute in octave chunks
    let octaves = abs_slide / 768;
    let frac = abs_slide % 768;

    // Table lookup for fractional part
    let frac_mult = if frac < 768 {
        LINEAR_FREQ_TABLE[frac]
    } else {
        LINEAR_FREQ_TABLE[767]
    };

    // Combine with octave scaling
    let full_mult = frac_mult * (1u32 << octaves.min(8)) as f32;

    if slide > 0 {
        // Pitch up: divide period (increase frequency)
        (period / full_mult).max(1.0)
    } else {
        // Pitch down: multiply period (decrease frequency)
        period * full_mult
    }
}

/// Lookup table for 2^(i/768) where i = 0..768
///
/// This is the canonical XM optimization used by MilkyTracker, ModPlug, etc.
/// The XM spec itself recommends: "To avoid floating point operations, you can
/// use a 768 doubleword array."
///
/// 768 = 12 * 16 * 4 (12 notes × 16 finetune levels × 4 for portamento precision)
/// Entry 768 is included for interpolation at the boundary.
const LINEAR_FREQ_TABLE: [f32; 769] = {
    let mut table = [0.0f32; 769];
    let mut i = 0;
    while i < 769 {
        // 2^(i/768) using const-compatible computation
        // We use the identity: 2^x = e^(x * ln(2))
        // For const eval, we compute this at compile time
        let x = i as f64 / 768.0;
        // 2^x where x is in [0, 1]
        // Using a high-precision polynomial approximation for const context
        // P(x) ≈ 2^x, accurate to ~10 decimal places for x in [0,1]
        let ln2 = 0.693147180559945309417232121458176568;
        let t = x * ln2;
        // e^t Taylor series (enough terms for f32 precision)
        let e_t = 1.0
            + t * (1.0
                + t * (0.5
                    + t * (0.16666666666666666
                        + t * (0.041666666666666664
                            + t * (0.008333333333333333
                                + t * (0.001388888888888889 + t * 0.0001984126984126984))))));
        table[i] = e_t as f32;
        i += 1;
    }
    table
};

/// Convert period to frequency (Hz) using lookup table
///
/// Modified XM frequency formula for 22050 Hz samples:
/// Frequency = 22050 * 2^((4608 - Period) / 768)
///
/// The original XM formula used 8363 Hz (Amiga C-4 rate), but since all our
/// samples are resampled to 22050 Hz, we use that as the base frequency.
/// This ensures samples play at their natural pitch at C-4.
///
/// This uses a 768-entry lookup table for the fractional part of the exponent,
/// making it O(1) and fast even in debug builds (no powf() calls).
#[inline]
fn period_to_frequency(period: f32) -> f32 {
    if period <= 0.0 {
        return 0.0;
    }

    // 4608 = 6 * 12 * 16 * 4 (middle C-4 reference point)
    let diff = 4608.0 - period;

    // Split into octave (integer) and fractional parts
    // diff / 768 = number of octaves from C-4
    let octaves = (diff / 768.0).floor();
    let frac = diff - (octaves * 768.0);

    // Table lookup with linear interpolation for fractional indices
    let idx = frac as usize;
    let t = frac - idx as f32;

    // Clamp index to valid range (handles edge cases)
    let idx = idx.min(767);
    let freq_frac = LINEAR_FREQ_TABLE[idx] * (1.0 - t) + LINEAR_FREQ_TABLE[idx + 1] * t;

    // Apply octave scaling: multiply by 2^octaves
    // For positive octaves: multiply by 2^n
    // For negative octaves: divide by 2^|n|
    let octave_scale = if octaves >= 0.0 {
        (1u32 << (octaves as u32).min(31)) as f32
    } else {
        1.0 / (1u32 << ((-octaves) as u32).min(31)) as f32
    };

    // Base frequency is 22050 Hz (our standardized sample rate)
    // This replaces the original 8363 Hz (Amiga C-4 rate)
    22050.0 * freq_frac * octave_scale
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_note_to_period() {
        // C-4 is note 49
        let period = note_to_period(49, 0);
        assert!(period > 0.0);

        // Higher notes have lower periods
        let period_c5 = note_to_period(61, 0);
        assert!(period_c5 < period);
    }

    #[test]
    fn test_period_to_frequency() {
        // Frequency is sample playback rate for 22050 Hz samples
        // C-4 (note 49) produces 22050 Hz playback (sample plays at natural speed)
        // Period = 10*12*16*4 - 48*16*4 = 7680 - 3072 = 4608
        // exp = (4608 - 4608) / 768 = 0, freq = 22050 * 2^0 = 22050
        let period = note_to_period(49, 0);
        let freq = period_to_frequency(period);
        assert!(
            (freq - 22050.0).abs() < 1.0,
            "Expected ~22050 Hz, got {}",
            freq
        );

        // C-5 (one octave up) should be double the frequency
        let period_c5 = note_to_period(61, 0);
        let freq_c5 = period_to_frequency(period_c5);
        assert!(
            (freq_c5 / freq - 2.0).abs() < 0.01,
            "C-5 should be ~2x C-4 frequency"
        );
    }

    #[test]
    fn test_samples_per_tick() {
        // At 125 BPM and 44100 Hz: 44100 * 2.5 / 125 = 882
        let spt = samples_per_tick(125, 44100);
        assert_eq!(spt, 882);
    }

    #[test]
    fn test_tracker_engine_new() {
        let engine = TrackerEngine::new();
        assert_eq!(engine.next_handle, 1);
        assert_eq!(engine.global_volume, 1.0);
    }

    #[test]
    fn test_channel_reset() {
        let mut ch = TrackerChannel::default();
        ch.volume = 0.5;
        ch.note_on = true;

        ch.reset();

        assert_eq!(ch.volume, 0.0);
        assert!(!ch.note_on);
        assert_eq!(ch.sample_direction, 1);
    }

    #[test]
    fn test_lut_accuracy() {
        // Verify the LUT matches the original formula within acceptable tolerance
        // Human pitch perception threshold is ~0.3%, we should be well under that
        fn reference_period_to_frequency(period: f32) -> f32 {
            if period <= 0.0 {
                return 0.0;
            }
            let exp = (4608.0 - period) / 768.0;
            22050.0 * 2.0_f32.powf(exp)
        }

        // Test across the full XM period range (roughly 50-7680)
        for period_int in (50..7680).step_by(10) {
            let period = period_int as f32;
            let lut_freq = period_to_frequency(period);
            let ref_freq = reference_period_to_frequency(period);

            let error_pct = ((lut_freq - ref_freq) / ref_freq).abs() * 100.0;
            assert!(
                error_pct < 0.01, // Less than 0.01% error
                "Period {} LUT={} ref={} error={}%",
                period,
                lut_freq,
                ref_freq,
                error_pct
            );
        }

        // Also test fractional periods (for vibrato/portamento)
        for i in 0..100 {
            let period = 4608.0 + (i as f32 * 0.37); // Arbitrary fractional steps
            let lut_freq = period_to_frequency(period);
            let ref_freq = reference_period_to_frequency(period);

            let error_pct = ((lut_freq - ref_freq) / ref_freq).abs() * 100.0;
            assert!(
                error_pct < 0.01,
                "Fractional period {} error={}%",
                period,
                error_pct
            );
        }
    }

    #[test]
    fn test_instrument_change_preserves_loop_points() {
        // Regression test for unified tracker bug where loop points were
        // reset to 0 when an instrument was set without a note.
        // This broke XM playback where patterns often set instruments
        // without triggering new notes (to change the active instrument).

        use nether_tracker::{LoopType, TrackerInstrument, TrackerModule, TrackerNote, TrackerPattern, FormatFlags, TrackerEffect};

        let mut engine = TrackerEngine::new();

        // Create an instrument with loop points
        let mut instrument = TrackerInstrument::default();
        instrument.name = "TestInstr".to_string();
        instrument.sample_loop_start = 1000;
        instrument.sample_loop_end = 3000;
        instrument.sample_loop_type = LoopType::Forward;
        instrument.sample_finetune = 10;

        // Create a minimal tracker module
        let module = TrackerModule {
            name: "Test".to_string(),
            num_channels: 4,
            initial_speed: 6,
            initial_tempo: 125,
            global_volume: 64,
            order_table: vec![0],
            patterns: vec![TrackerPattern::empty(64, 4)],
            instruments: vec![instrument],
            samples: vec![],
            format: FormatFlags::IS_XM_FORMAT | FormatFlags::INSTRUMENTS,
            message: None,
            restart_position: 0,
        };

        // Load the module (with dummy sound handle)
        let handle = engine.load_tracker_module(module, vec![42]); // sound_handle=42

        // Create a note with ONLY an instrument number (no note)
        // This is common in tracker music to change the active instrument
        let note = TrackerNote {
            note: 0,           // No note
            instrument: 1,     // Set instrument 1
            volume: 0,         // No volume
            effect: TrackerEffect::None,
        };

        // Process the note (this simulates what happens during playback)
        engine.process_note_internal(0, &note, handle, &[]);

        // Verify that the loop points are set correctly
        let channel = &engine.channels[0];
        assert_eq!(channel.sample_handle, 42, "Sample handle should be set");
        assert_eq!(channel.sample_loop_start, 1000, "Loop start should be preserved from instrument");
        assert_eq!(channel.sample_loop_end, 3000, "Loop end should be preserved from instrument");
        assert_eq!(channel.sample_loop_type, 1, "Loop type should be Forward (1)");
        assert_eq!(channel.finetune, 10, "Finetune should be preserved from instrument");
    }

    #[test]
    fn test_note_without_instrument_preserves_loop_points() {
        // Test that when a note is triggered without an instrument number,
        // it uses the loop points from the currently active instrument.
        // This is critical for XM playback where notes often don't repeat the instrument number.

        use nether_tracker::{LoopType, TrackerInstrument, TrackerModule, TrackerNote, TrackerPattern, FormatFlags, TrackerEffect};

        let mut engine = TrackerEngine::new();

        // Create an instrument with loop points
        let mut instrument = TrackerInstrument::default();
        instrument.name = "TestInstr".to_string();
        instrument.sample_loop_start = 1000;
        instrument.sample_loop_end = 3000;
        instrument.sample_loop_type = LoopType::Forward;
        instrument.sample_finetune = 10;
        instrument.sample_relative_note = 2; // Transpose up 2 semitones

        // Create a minimal tracker module
        let module = TrackerModule {
            name: "Test".to_string(),
            num_channels: 4,
            initial_speed: 6,
            initial_tempo: 125,
            global_volume: 64,
            order_table: vec![0],
            patterns: vec![TrackerPattern::empty(64, 4)],
            instruments: vec![instrument],
            samples: vec![],
            format: FormatFlags::IS_XM_FORMAT | FormatFlags::INSTRUMENTS,
            message: None,
            restart_position: 0,
        };

        // Load the module (with dummy sound handle)
        let handle = engine.load_tracker_module(module, vec![42]); // sound_handle=42

        // First, set the instrument
        let note1 = TrackerNote {
            note: 0,           // No note yet
            instrument: 1,     // Set instrument 1
            volume: 0,
            effect: TrackerEffect::None,
        };
        engine.process_note_internal(0, &note1, handle, &[]);

        // Now trigger a note WITHOUT specifying the instrument
        // This should use the loop points from instrument 1 (currently active)
        let note2 = TrackerNote {
            note: 49,          // C-4
            instrument: 0,     // No instrument specified
            volume: 0,
            effect: TrackerEffect::None,
        };
        engine.process_note_internal(0, &note2, handle, &[]);

        // Verify that the loop points are STILL set from instrument 1
        let channel = &engine.channels[0];
        assert_eq!(channel.sample_handle, 42, "Sample handle should still be 42");
        assert_eq!(channel.sample_loop_start, 1000, "Loop start should be preserved from instrument 1");
        assert_eq!(channel.sample_loop_end, 3000, "Loop end should be preserved from instrument 1");
        assert_eq!(channel.sample_loop_type, 1, "Loop type should be Forward (1)");
    }

    #[test]
    fn test_sample_loop_wraps_correctly() {
        // Test that sample playback wraps at the loop end point
        let mut channel = TrackerChannel::default();
        channel.sample_loop_start = 100;
        channel.sample_loop_end = 200;
        channel.sample_loop_type = 1; // Forward loop
        channel.sample_direction = 1;
        channel.note_on = true;
        channel.period = 4608.0; // C-4, plays at 1x speed

        // Create sample data (larger than loop end)
        let mut data = vec![0i16; 500];
        // Put some markers at key positions
        for i in 0..500 {
            data[i] = i as i16; // Ascending values
        }

        // Position sample just before loop end
        channel.sample_pos = 198.0;

        // Sample a few times to trigger the loop wrap
        let sample_rate = 22050;
        let _ = sample_channel(&mut channel, &data, sample_rate);

        // Check if sample wrapped (should still be moving forward, but position should have wrapped)
        // After several samples, we should have moved past loop_end=200 and wrapped back
        for _ in 0..10 {
            let _ = sample_channel(&mut channel, &data, sample_rate);
        }

        // Sample position should be in the loop range, not past loop_end
        assert!(
            channel.sample_pos >= channel.sample_loop_start as f64
                && channel.sample_pos < channel.sample_loop_end as f64,
            "Sample position {} should be within loop range [{}, {})",
            channel.sample_pos,
            channel.sample_loop_start,
            channel.sample_loop_end
        );
    }

    #[test]
    fn test_sample_interpolation_at_loop_boundary() {
        // Test that interpolation at loop boundary reads from loop_start, not past loop_end
        let mut channel = TrackerChannel::default();
        channel.sample_loop_start = 100;
        channel.sample_loop_end = 200;
        channel.sample_loop_type = 1; // Forward loop
        channel.sample_direction = 1;
        channel.note_on = true;
        channel.period = 4608.0;

        // Create sample data with distinct values
        let mut data = vec![0i16; 500];
        data[199] = 1000;   // Last sample in loop (loop_end - 1)
        data[200] = -9999;  // First sample AFTER loop (should NOT be read)
        data[100] = 2000;   // First sample of loop (should be used for interpolation)

        // Position at 199.5 (between last loop sample and what would be next)
        channel.sample_pos = 199.5;

        // If interpolation is correct, it should interpolate between
        // data[199] (1000) and data[loop_start=100] (2000), not data[200] (-9999)
        let sample = sample_channel(&mut channel, &data, 22050);

        // Normalized: sample1 = 1000/32768 ≈ 0.0305, sample2 should be 2000/32768 ≈ 0.061
        // With frac=0.5, result ≈ 0.0458
        // If buggy (using data[200]=-9999), result would be negative

        // The sample should be positive (interpolating between 1000 and 2000)
        // If it's negative or very different, interpolation is reading past loop_end
        assert!(
            sample > 0.0,
            "Sample at loop boundary should be positive (interpolating within loop), got {}",
            sample
        );
    }

    #[test]
    fn test_sample_keeps_looping_indefinitely() {
        // Test that a looping sample continues playing indefinitely
        // without note_on being set to false
        let mut channel = TrackerChannel::default();
        channel.sample_loop_start = 50;
        channel.sample_loop_end = 150;
        channel.sample_loop_type = 1; // Forward loop
        channel.sample_direction = 1;
        channel.note_on = true;
        channel.period = 4608.0; // C-4, plays at 1x speed
        channel.volume = 1.0;

        // Create sample data - loop is within the data
        let data = vec![1000i16; 200];

        // Simulate playing through many loop iterations
        let sample_rate = 22050;
        for iteration in 0..1000 {
            let _ = sample_channel(&mut channel, &data, sample_rate);

            // Check that note is still on
            assert!(
                channel.note_on,
                "Note should still be on after {} samples, but was turned off. sample_pos={}, fade_out={}",
                iteration, channel.sample_pos, channel.fade_out_samples
            );

            // Check sample position is within valid range
            assert!(
                channel.sample_pos >= 0.0 && channel.sample_pos < data.len() as f64,
                "Sample position {} is out of bounds [0, {})",
                channel.sample_pos, data.len()
            );
        }

        // After 1000 samples at 1x speed, we should have looped many times
        // but the note should still be playing
        assert!(channel.note_on, "Note should still be playing after extended playback");
    }

    #[test]
    fn test_non_looping_sample_stops() {
        // Test that a non-looping sample stops when it reaches the end
        let mut channel = TrackerChannel::default();
        channel.sample_loop_start = 0;
        channel.sample_loop_end = 0;
        channel.sample_loop_type = 0; // No loop
        channel.sample_direction = 1;
        channel.note_on = true;
        channel.period = 4608.0;
        channel.volume = 1.0;

        // Create short sample data
        let data = vec![1000i16; 100];

        // Play through the sample until it ends
        let sample_rate = 22050;
        let mut stopped = false;
        for _ in 0..500 {
            let _ = sample_channel(&mut channel, &data, sample_rate);
            if !channel.note_on {
                stopped = true;
                break;
            }
        }

        assert!(stopped, "Non-looping sample should have stopped when reaching end, final pos={}, fade_out={}", channel.sample_pos, channel.fade_out_samples);
    }

    #[test]
    fn test_xm_module_loop_points_flow_through() {
        // Test that loop points from XM instruments correctly flow through
        // to the tracker channel during playback
        use nether_xm::XmModule;

        // Create a minimal XM module with a looping instrument
        let xm_module = XmModule {
            name: "Test".to_string(),
            num_channels: 4,
            song_length: 1,
            restart_position: 0,
            num_patterns: 1,
            num_instruments: 1,
            linear_frequency_table: true,
            default_speed: 6,
            default_bpm: 125,
            order_table: vec![0],
            patterns: vec![nether_xm::XmPattern {
                num_rows: 64,
                notes: vec![vec![nether_xm::XmNote::default(); 4]; 64],
            }],
            instruments: vec![nether_xm::XmInstrument {
                name: "LoopingInstr".to_string(),
                num_samples: 1,
                sample_loop_start: 100,
                sample_loop_length: 200,
                sample_loop_type: 1, // Forward loop
                sample_finetune: 0,
                sample_relative_note: 17, // ~22050 Hz (so no scaling needed)
                volume_envelope: None,
                panning_envelope: None,
                volume_fadeout: 0,
                vibrato_type: 0,
                vibrato_sweep: 0,
                vibrato_depth: 0,
                vibrato_rate: 0,
            }],
        };

        // Load into tracker engine
        let mut engine = TrackerEngine::new();
        let handle = engine.load_xm_module(xm_module, vec![42]); // dummy sound handle

        // Get the loaded module and check the TrackerInstrument
        let module = engine.get_module(handle).expect("Module should be loaded");
        let instr = &module.instruments[0];

        // Loop points should be approximately 100 and 300 (100+200) since sample rate ≈ 22050 Hz
        assert!(
            instr.sample_loop_start >= 95 && instr.sample_loop_start <= 105,
            "Loop start should be ~100, got {}",
            instr.sample_loop_start
        );
        assert!(
            instr.sample_loop_end >= 295 && instr.sample_loop_end <= 305,
            "Loop end should be ~300, got {}",
            instr.sample_loop_end
        );
        assert_eq!(
            instr.sample_loop_type,
            nether_tracker::LoopType::Forward,
            "Loop type should be Forward"
        );

        // Now simulate triggering a note and check the channel
        let note = nether_tracker::TrackerNote {
            note: 49,      // C-4
            instrument: 1, // Instrument 1
            volume: 64,
            effect: nether_tracker::TrackerEffect::None,
        };
        engine.process_note_internal(0, &note, handle, &[]);

        // Check channel has correct loop settings
        let channel = &engine.channels[0];

        assert!(
            channel.sample_loop_start >= 95 && channel.sample_loop_start <= 105,
            "Channel loop start should be ~100, got {}",
            channel.sample_loop_start
        );
        assert!(
            channel.sample_loop_end >= 295 && channel.sample_loop_end <= 305,
            "Channel loop end should be ~300, got {}",
            channel.sample_loop_end
        );
        assert_eq!(
            channel.sample_loop_type, 1,
            "Channel loop type should be 1 (Forward)"
        );
    }

    #[test]
    fn test_volume_envelope_with_sustain_holds_note() {
        // Test that a note with volume envelope sustain holds indefinitely
        // until key_off is triggered
        use nether_tracker::{TrackerEnvelope, EnvelopeFlags};

        // Create a volume envelope with sustain at point 1 (tick 20, value 64)
        // The envelope goes: (0, 64) -> (20, 64) -> (50, 0)
        // Sustain at point 1 (tick 20) should hold at volume 64
        let envelope = TrackerEnvelope {
            points: vec![(0, 64), (20, 64), (50, 0)],
            sustain_begin: 1,
            sustain_end: 1,
            loop_begin: 0,
            loop_end: 0,
            flags: EnvelopeFlags::from_bits(0x01 | 0x04), // ENABLED | SUSTAIN_LOOP
        };

        assert!(envelope.is_enabled(), "Envelope should be enabled");
        assert!(envelope.has_sustain(), "Envelope should have sustain");

        // Test envelope values
        assert_eq!(envelope.value_at(0), 64, "Value at tick 0 should be 64");
        assert_eq!(envelope.value_at(20), 64, "Value at tick 20 (sustain) should be 64");
        assert_eq!(envelope.value_at(50), 0, "Value at tick 50 (after sustain) should be 0");

        // Now test with a channel
        let mut channel = TrackerChannel::default();
        channel.note_on = true;
        channel.key_off = false;
        channel.volume = 1.0;
        channel.volume_envelope_enabled = true;
        channel.volume_envelope_pos = 0;
        // Sustain is at tick 20 (the x-coordinate of point 1)
        channel.volume_envelope_sustain_tick = Some(20);

        // Advance the envelope 50 times - it should stop at sustain (tick 20)
        for i in 0..50 {
            // Check if at sustain point
            let at_sustain = if let Some(sus_tick) = channel.volume_envelope_sustain_tick {
                channel.volume_envelope_pos >= sus_tick && !channel.key_off
            } else {
                false
            };

            if !at_sustain {
                channel.volume_envelope_pos += 1;
            }

            // After 20 iterations, envelope should be held at sustain
            if i >= 20 {
                assert_eq!(
                    channel.volume_envelope_pos, 20,
                    "Envelope should be held at sustain point 20, not advancing to {}",
                    channel.volume_envelope_pos
                );
            }
        }

        // Now trigger key_off and verify envelope continues past sustain
        channel.key_off = true;

        for _ in 0..35 {
            let at_sustain = if let Some(sus_tick) = channel.volume_envelope_sustain_tick {
                channel.volume_envelope_pos >= sus_tick && !channel.key_off
            } else {
                false
            };

            if !at_sustain {
                channel.volume_envelope_pos += 1;
            }
        }

        // After key_off and 35 more iterations, envelope should have advanced to ~55
        assert!(
            channel.volume_envelope_pos > 50,
            "After key_off, envelope should continue past sustain, got {}",
            channel.volume_envelope_pos
        );
    }

    #[test]
    fn test_held_note_with_looping_sample_and_envelope() {
        // This test simulates what happens during real playback:
        // 1. A note with a looping sample
        // 2. A volume envelope with sustain
        // 3. Extended playback without key_off
        // The note should continue playing indefinitely
        use crate::audio::Sound;
        use std::sync::Arc;

        // Create sample data (400 samples, loops from 100-300)
        let sample_data: Vec<i16> = (0..400).map(|i| ((i as f32 * 0.1).sin() * 16000.0) as i16).collect();
        let sound = Sound {
            data: Arc::new(sample_data.clone()),
        };
        let sounds: Vec<Option<Sound>> = vec![None, Some(sound)];

        // Create an XM module with a looping instrument and volume envelope
        let xm_module = nether_xm::XmModule {
            name: "HeldNoteTest".to_string(),
            num_channels: 4,
            song_length: 1,
            restart_position: 0,
            num_patterns: 1,
            num_instruments: 1,
            linear_frequency_table: true,
            default_speed: 6,
            default_bpm: 125,
            order_table: vec![0],
            patterns: vec![nether_xm::XmPattern {
                num_rows: 64,
                notes: vec![
                    // Row 0: Note C-4 with instrument 1
                    vec![
                        nether_xm::XmNote {
                            note: 49, // C-4
                            instrument: 1,
                            volume: 0x40, // Full volume
                            effect: 0,
                            effect_param: 0,
                        },
                        nether_xm::XmNote::default(),
                        nether_xm::XmNote::default(),
                        nether_xm::XmNote::default(),
                    ],
                    // Rows 1-63: Empty
                ].into_iter().chain(std::iter::repeat_with(|| vec![
                    nether_xm::XmNote::default(),
                    nether_xm::XmNote::default(),
                    nether_xm::XmNote::default(),
                    nether_xm::XmNote::default(),
                ]).take(63)).collect(),
            }],
            instruments: vec![nether_xm::XmInstrument {
                name: "LoopingWithEnvelope".to_string(),
                num_samples: 1,
                sample_loop_start: 100,
                sample_loop_length: 200,
                sample_loop_type: 1, // Forward loop
                sample_finetune: -16,
                sample_relative_note: 17, // ~22050 Hz
                volume_envelope: Some(nether_xm::XmEnvelope {
                    points: vec![(0, 64), (20, 64), (100, 0)],
                    sustain_point: 1, // Sustain at point 1 (tick 20)
                    loop_start: 0,
                    loop_end: 0,
                    enabled: true,
                    sustain_enabled: true,
                    loop_enabled: false,
                }),
                panning_envelope: None,
                volume_fadeout: 0,
                vibrato_type: 0,
                vibrato_sweep: 0,
                vibrato_depth: 0,
                vibrato_rate: 0,
            }],
        };

        // Load the module
        let mut engine = TrackerEngine::new();
        let handle = engine.load_xm_module(xm_module, vec![1]); // sound_handle 1 points to sounds[1]

        // Initialize tracker state
        let mut state = crate::state::TrackerState::default();
        state.handle = handle;
        state.flags = crate::state::tracker_flags::PLAYING;
        state.volume = 256;
        state.bpm = 125; // Default XM tempo
        state.speed = 6; // Default XM speed
        // state.tick and state.tick_sample_pos are 0 by default
        // This means the first call to render_sample_and_advance will process tick 0

        // First sync to state (this doesn't process notes, just syncs position)
        engine.sync_to_state(&state, &sounds);

        // Render the first sample - this triggers tick 0 processing on the first row
        let sample_rate = 22050u32;
        let _ = engine.render_sample_and_advance(&mut state, &sounds, sample_rate);

        // Verify the channel is set up correctly after the first render
        let channel = &engine.channels[0];
        assert!(channel.note_on, "Note should be on after triggering");
        assert_eq!(channel.sample_loop_type, 1, "Loop type should be Forward");

        // Render many samples - the note should keep playing
        let samples_to_render = 50000; // About 2+ seconds worth

        for i in 0..samples_to_render {
            let _ = engine.render_sample_and_advance(&mut state, &sounds, sample_rate);

            // The note should still be on (not cut off)
            if !engine.channels[0].note_on {
                panic!("Note stopped at sample {} - note_on became false", i);
            }
        }

        // Verify envelope advanced (should be at sustain point 20 or higher)
        // With BPM=125, speed=6, sample_rate=22050:
        // samples_per_tick = 22050 * 2.5 / 125 = 441
        // 50000 samples = ~113 ticks = ~18 rows
        // Envelope should have reached sustain point at tick 20
        assert!(
            engine.channels[0].volume_envelope_pos >= 20,
            "Envelope should have advanced to sustain point (20), but is at {}",
            engine.channels[0].volume_envelope_pos
        );

        // After extended playback, note should still be on
        assert!(engine.channels[0].note_on, "Note should still be playing after extended playback");
    }
}
