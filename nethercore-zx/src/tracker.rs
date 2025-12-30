//! XM Tracker playback engine
//!
//! This module implements the playback engine for XM (Extended Module) tracker music.
//! It integrates with the existing audio system to provide era-authentic music playback
//! with full rollback netcode support.
//!
//! # Architecture
//!
//! - **TrackerState** (in rollback_state.rs) - Minimal 64-byte POD state for rollback
//! - **TrackerEngine** (this module) - Full playback engine with channel state
//! - **XmModule** (from nether-xm) - Parsed pattern and instrument data
//!
//! The engine is designed to reconstruct its full state from TrackerState by seeking
//! to the correct position and replaying ticks. This keeps rollback snapshots small.

use std::collections::BTreeMap;

use nether_xm::{XmInstrument, XmModule, XmNote};

use crate::audio::Sound;
use crate::state::tracker_flags;

/// Maximum number of tracker channels (XM supports up to 32)
pub const MAX_TRACKER_CHANNELS: usize = 32;

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

    /// Global volume slide memory (Hxy effect)
    last_global_vol_slide: u8,
}

/// A loaded tracker module with resolved sample handles
#[derive(Debug)]
struct LoadedModule {
    /// Parsed XM module data
    module: XmModule,
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

    // Fade state for smooth transitions (anti-pop)
    /// Fade-out samples remaining (0 = not fading out, >0 = fading out)
    pub fade_out_samples: u16,
    /// Fade-in samples remaining (0 = fully faded in, >0 = still fading in)
    pub fade_in_samples: u16,
    /// Previous sample value for crossfade during note transitions
    pub prev_sample: f32,
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
    }

    /// Trigger a new note
    pub fn trigger_note(&mut self, note: u8, instrument: Option<&XmInstrument>) {
        self.note_on = true;
        self.key_off = false;
        self.sample_pos = 0.0;
        self.sample_direction = 1;
        self.volume_envelope_pos = 0;
        self.panning_envelope_pos = 0;
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

        // Set period from note
        if let Some(instr) = instrument {
            self.base_period = note_to_period(note, instr.sample_finetune);
            self.period = self.base_period;
            self.finetune = instr.sample_finetune;
            self.sample_loop_start = instr.sample_loop_start;
            self.sample_loop_end = instr.sample_loop_start + instr.sample_loop_length;
            self.sample_loop_type = instr.sample_loop_type;
        } else {
            self.base_period = note_to_period(note, 0);
            self.period = self.base_period;
        }
    }

    /// Trigger key-off (release)
    pub fn trigger_key_off(&mut self) {
        self.key_off = true;
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
            last_global_vol_slide: 0,
        }
    }

    /// Load a module with resolved sound handles
    ///
    /// Returns a handle for later playback (1-indexed, 0 is invalid).
    /// The returned handle has TRACKER_HANDLE_FLAG set (bit 31) to distinguish
    /// it from PCM sound handles in the unified music API.
    pub fn load_module(&mut self, module: XmModule, sound_handles: Vec<u32>) -> u32 {
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
    pub fn get_module(&self, handle: u32) -> Option<&XmModule> {
        let raw = raw_tracker_handle(handle);
        self.modules
            .get(raw as usize)
            .and_then(|m| m.as_ref())
            .map(|m| &m.module)
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
        // Validate handle exists
        if self
            .modules
            .get(handle as usize)
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

            // Get current pattern length
            let (num_rows, song_length, restart_position) = {
                let loaded = match self.modules.get(handle as usize).and_then(|m| m.as_ref()) {
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
                    loaded.module.song_length,
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
        let (_num_channels, pattern_info) = {
            let loaded = match self.modules.get(handle as usize).and_then(|m| m.as_ref()) {
                Some(m) => m,
                None => return,
            };
            let pattern = match loaded.module.pattern_at_order(self.current_order) {
                Some(p) => p,
                None => return,
            };

            // Collect note data for this row
            let mut notes = Vec::new();
            for ch_idx in 0..loaded.module.num_channels as usize {
                if let Some(note) = pattern.get_note(self.current_row, ch_idx as u8) {
                    notes.push((ch_idx, *note));
                }
            }
            (loaded.module.num_channels, notes)
        };

        // Process each note
        for (ch_idx, note) in pattern_info {
            self.process_note_internal(ch_idx, &note, handle, sounds);
        }
    }

    /// Internal note processing that accesses module by handle
    fn process_note_internal(
        &mut self,
        ch_idx: usize,
        note: &XmNote,
        handle: u32,
        _sounds: &[Option<Sound>],
    ) {
        // Handle instrument change
        if note.has_instrument() {
            let instr_idx = (note.instrument - 1) as usize;
            self.channels[ch_idx].instrument = note.instrument;

            // Get sound handle and instrument data
            let (sound_handle, loop_start, loop_end, loop_type, finetune) = {
                let loaded = match self.modules.get(handle as usize).and_then(|m| m.as_ref()) {
                    Some(m) => m,
                    None => return,
                };
                let sound_handle = loaded.sound_handles.get(instr_idx).copied().unwrap_or(0);
                if let Some(instr) = loaded.module.instruments.get(instr_idx) {
                    (
                        sound_handle,
                        instr.sample_loop_start,
                        instr.sample_loop_start + instr.sample_loop_length,
                        instr.sample_loop_type,
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
            // Fetch all instrument data we need for note trigger
            let instr_data = {
                let loaded = match self.modules.get(handle as usize).and_then(|m| m.as_ref()) {
                    Some(m) => m,
                    None => return,
                };
                let instr_idx = (self.channels[ch_idx].instrument.saturating_sub(1)) as usize;
                if let Some(instr) = loaded.module.instruments.get(instr_idx) {
                    // Extract envelope data
                    let (vol_env_enabled, vol_env_sustain, vol_env_loop) =
                        if let Some(ref env) = instr.volume_envelope {
                            (env.enabled, env.sustain_tick(), env.loop_range())
                        } else {
                            (false, None, None)
                        };
                    let (pan_env_enabled, pan_env_sustain, pan_env_loop) =
                        if let Some(ref env) = instr.panning_envelope {
                            (env.enabled, env.sustain_tick(), env.loop_range())
                        } else {
                            (false, None, None)
                        };

                    Some((
                        instr.sample_finetune,
                        instr.sample_loop_start,
                        instr.sample_loop_start + instr.sample_loop_length,
                        instr.sample_loop_type,
                        instr.vibrato_type,
                        instr.vibrato_depth,
                        instr.vibrato_rate,
                        instr.vibrato_sweep,
                        instr.sample_relative_note,
                        instr.volume_fadeout,
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

        // Handle volume column
        if let Some(vol) = note.get_volume() {
            self.channels[ch_idx].volume = vol as f32 / 64.0;
        }

        // Handle volume column effects
        let channel = &mut self.channels[ch_idx];
        if let Some((effect_type, param)) = note.get_volume_effect() {
            channel.vol_col_effect = effect_type;
            channel.vol_col_param = param;

            // Tick 0 effects (fine slides, set vibrato speed, set panning)
            match effect_type {
                0x8 => {
                    // Fine volume slide down
                    channel.volume = (channel.volume - param as f32 / 64.0).max(0.0);
                }
                0x9 => {
                    // Fine volume slide up
                    channel.volume = (channel.volume + param as f32 / 64.0).min(1.0);
                }
                0xA => {
                    // Set vibrato speed
                    channel.vibrato_speed = param;
                }
                0xC => {
                    // Set panning (coarse, 0-15)
                    channel.panning = (param as f32 / 15.0) * 2.0 - 1.0;
                }
                0xF => {
                    // Tone portamento - set porta speed from volume column
                    // Volume column uses param * 16 for speed
                    channel.porta_speed = param * 16;
                }
                _ => {}
            }
        } else {
            channel.vol_col_effect = 0;
        }

        // Handle effects (tick 0 processing)
        self.process_effect_tick0(ch_idx, note.effect, note.effect_param, note.note, note.instrument);
    }

    /// Process effect at tick 0 (row start)
    /// Returns (position_jump, pattern_break) if those effects are triggered
    fn process_effect_tick0(
        &mut self,
        ch_idx: usize,
        effect: u8,
        param: u8,
        note_num: u8,
        note_instrument: u8,
    ) -> (Option<u16>, Option<u16>) {
        let channel = &mut self.channels[ch_idx];
        let mut position_jump = None;
        let mut pattern_break = None;

        match effect {
            // 0xy: Arpeggio
            0x00 if param != 0 => {
                channel.arpeggio_note1 = param >> 4;
                channel.arpeggio_note2 = param & 0x0F;
                channel.arpeggio_tick = 0;
            }
            // 1xx: Portamento up
            0x01 => {
                if param != 0 {
                    channel.last_porta_up = param;
                }
            }
            // 2xx: Portamento down
            0x02 => {
                if param != 0 {
                    channel.last_porta_down = param;
                }
            }
            // 3xx: Tone portamento
            0x03 => {
                if param != 0 {
                    channel.porta_speed = param;
                }
            }
            // 4xy: Vibrato
            0x04 => {
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
            // 5xy: Tone portamento + volume slide
            0x05 => {
                if param != 0 {
                    channel.last_volume_slide = param;
                }
            }
            // 6xy: Vibrato + volume slide
            0x06 => {
                if param != 0 {
                    channel.last_volume_slide = param;
                }
            }
            // 7xy: Tremolo
            0x07 => {
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
            // 8xx: Set panning
            0x08 => {
                channel.panning = (param as f32 / 255.0) * 2.0 - 1.0;
            }
            // 9xx: Sample offset
            0x09 => {
                if param != 0 {
                    channel.last_sample_offset = param;
                }
                // Combine high byte (from SAx) and low byte (from 9xx)
                let offset = ((channel.sample_offset_high as u32) << 16)
                    | ((channel.last_sample_offset as u32) << 8);
                channel.sample_pos = offset as f64;
            }
            // Axy: Volume slide
            0x0A => {
                if param != 0 {
                    channel.last_volume_slide = param;
                }
            }
            // Bxx: Position jump
            0x0B => {
                position_jump = Some(param as u16);
            }
            // Cxx: Set volume
            0x0C => {
                channel.volume = (param.min(64) as f32) / 64.0;
            }
            // Dxx: Pattern break
            0x0D => {
                // Parameter is BCD row: high nibble * 10 + low nibble
                let row = (param >> 4) * 10 + (param & 0x0F);
                pattern_break = Some(row as u16);
            }
            // Exy: Extended commands
            0x0E => {
                let sub_cmd = param >> 4;
                let sub_param = param & 0x0F;
                match sub_cmd {
                    // E1x: Fine portamento up
                    0x1 => {
                        if sub_param != 0 {
                            channel.last_fine_porta_up = sub_param;
                        }
                        let p = channel.last_fine_porta_up;
                        channel.period = (channel.period - p as f32 * 4.0).max(1.0);
                    }
                    // E2x: Fine portamento down
                    0x2 => {
                        if sub_param != 0 {
                            channel.last_fine_porta_down = sub_param;
                        }
                        let p = channel.last_fine_porta_down;
                        channel.period += p as f32 * 4.0;
                    }
                    // E3x: Glissando control (rounded tone portamento)
                    0x3 => {
                        channel.glissando = sub_param != 0;
                    }
                    // E4x: Set vibrato waveform
                    0x4 => {
                        channel.vibrato_waveform = sub_param & 0x07;
                    }
                    // E5x: Set finetune
                    0x5 => {
                        channel.finetune = (sub_param as i8) - 8;
                    }
                    // E6x: Pattern loop
                    0x6 => {
                        if sub_param == 0 {
                            // Set loop start
                            channel.pattern_loop_row = self.current_row;
                        } else if channel.pattern_loop_count == 0 {
                            // Start loop
                            channel.pattern_loop_count = sub_param;
                        } else {
                            channel.pattern_loop_count -= 1;
                        }
                        // Note: actual loop jump handled in caller
                    }
                    // E7x: Set tremolo waveform
                    0x7 => {
                        channel.tremolo_waveform = sub_param & 0x07;
                    }
                    // E8x: Set panning (coarse)
                    0x8 => {
                        channel.panning = (sub_param as f32 / 15.0) * 2.0 - 1.0;
                    }
                    // E9x: Retrigger note
                    0x9 => {
                        channel.retrigger_tick = sub_param;
                    }
                    // EAx: Fine volume slide up
                    0xA => {
                        channel.volume = (channel.volume + sub_param as f32 / 64.0).min(1.0);
                    }
                    // EBx: Fine volume slide down
                    0xB => {
                        channel.volume = (channel.volume - sub_param as f32 / 64.0).max(0.0);
                    }
                    // ECx: Note cut at tick x
                    0xC => {
                        channel.note_cut_tick = sub_param;
                    }
                    // EDx: Note delay - trigger note at tick x
                    0xD => {
                        channel.note_delay_tick = sub_param;
                        channel.delayed_note = note_num;
                        channel.delayed_instrument = note_instrument;
                    }
                    // EEx: Pattern delay - repeat current row x times
                    0xE => {
                        if sub_param > 0 && self.pattern_delay == 0 {
                            self.pattern_delay = sub_param;
                        }
                    }
                    _ => {}
                }
            }
            // Fxx: Set speed/tempo
            0x0F => {
                // This effect modifies TrackerState, which we don't have here
                // It will be handled in the FFI layer
            }
            // Gxx: Set global volume
            0x10 => {
                self.global_volume = (param.min(64) as f32) / 64.0;
            }
            // Hxy: Global volume slide
            0x11 => {
                if param != 0 {
                    self.last_global_vol_slide = param;
                }
            }
            // Kxx: Key off (at tick xx)
            0x14 => {
                if param == 0 {
                    channel.trigger_key_off();
                } else {
                    channel.key_off_tick = param;
                }
            }
            // Lxx: Set envelope position
            0x15 => {
                channel.volume_envelope_pos = param as u16;
            }
            // Pxy: Panning slide
            0x19 => {
                // Store for per-tick processing
            }
            // Rxy: Multi retrigger
            0x1B => {
                channel.retrigger_tick = param & 0x0F;
                channel.retrigger_mode = param >> 4;
                // Additive volume changes (modes 1-5 decrease, 9-13 increase)
                channel.retrigger_volume = match param >> 4 {
                    1 => -1,
                    2 => -2,
                    3 => -4,
                    4 => -8,
                    5 => -16,
                    // 6, 7 = multiplicative (handled in process_tick)
                    9 => 1,
                    10 => 2,
                    11 => 4,
                    12 => 8,
                    13 => 16,
                    // 14, 15 = multiplicative (handled in process_tick)
                    _ => 0,
                };
            }
            // Sxy: Extended commands (OpenMPT/FT2 extended)
            0x1C => {
                let sub_cmd = param >> 4;
                let sub_param = param & 0x0F;
                match sub_cmd {
                    // SAx: Set high sample offset byte
                    0xA => {
                        channel.sample_offset_high = sub_param;
                    }
                    _ => {}
                }
            }
            _ => {}
        }

        (position_jump, pattern_break)
    }

    /// Process per-tick effects (called every tick except tick 0)
    pub fn process_tick(&mut self, tick: u16, _speed: u16) {
        for ch_idx in 0..MAX_TRACKER_CHANNELS {
            let channel = &mut self.channels[ch_idx];
            if !channel.note_on {
                continue;
            }

            // Apply per-tick effects based on stored parameters

            // Arpeggio
            if channel.arpeggio_note1 != 0 || channel.arpeggio_note2 != 0 {
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

            // Volume slide
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

            // Portamento up
            if channel.last_porta_up != 0 {
                channel.period = (channel.period - channel.last_porta_up as f32 * 4.0).max(1.0);
            }

            // Portamento down
            if channel.last_porta_down != 0 {
                channel.period += channel.last_porta_down as f32 * 4.0;
            }

            // Tone portamento (slide toward target)
            if channel.target_period > 0.0 && channel.porta_speed > 0 {
                let diff = channel.target_period - channel.period;
                let speed = channel.porta_speed as f32 * 4.0;
                if diff.abs() < speed {
                    channel.period = channel.target_period;
                } else if diff > 0.0 {
                    channel.period += speed;
                } else {
                    channel.period -= speed;
                }
            }

            // Vibrato (FT2-compatible depth and speed)
            // Depth: 128.0/15.0 ≈ 8.533 gives ±2 semitones (±128 period units) at depth=15
            // Speed: 4x faster oscillation to match libxm/FT2
            if channel.vibrato_depth > 0 {
                let vibrato = get_waveform_value(channel.vibrato_waveform, channel.vibrato_pos);
                let delta = vibrato * channel.vibrato_depth as f32 * (128.0 / 15.0);
                channel.period = channel.base_period + delta;
                channel.vibrato_pos =
                    channel.vibrato_pos.wrapping_add(channel.vibrato_speed << 2) & 0x3F;
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
                let delta = auto_vib * channel.auto_vibrato_depth as f32 * sweep_factor * (128.0 / 15.0);
                channel.period += delta;

                // Advance auto-vibrato position (slower rate than pattern vibrato)
                channel.auto_vibrato_pos = channel.auto_vibrato_pos.wrapping_add(channel.auto_vibrato_rate as u16);

                // Advance sweep
                if channel.auto_vibrato_sweep < 65535 {
                    channel.auto_vibrato_sweep = channel.auto_vibrato_sweep.saturating_add(1);
                }
            }

            // Tremolo (FT2-compatible depth and speed)
            // Depth: * 4.0 / 128.0 matches libxm formula
            // Speed: 4x faster oscillation to match libxm/FT2
            if channel.tremolo_depth > 0 {
                let tremolo = get_waveform_value(channel.tremolo_waveform, channel.tremolo_pos);
                let delta = tremolo * channel.tremolo_depth as f32 * 4.0 / 128.0;
                channel.volume = (channel.volume + delta).clamp(0.0, 1.0);
                channel.tremolo_pos =
                    channel.tremolo_pos.wrapping_add(channel.tremolo_speed << 2) & 0x3F;
            }

            // Retrigger
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

            // Panning slide
            if channel.panning_slide != 0 {
                channel.panning =
                    (channel.panning + channel.panning_slide as f32 / 255.0).clamp(-1.0, 1.0);
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

        let module = match self
            .modules
            .get(state.handle as usize)
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

            // Sample with interpolation
            let sample = sample_channel(channel, &sound.data, sample_rate);

            // Apply volume with envelope processing
            let mut vol = channel.volume;

            // Apply volume envelope if enabled
            if channel.volume_envelope_enabled {
                let instr_idx = channel.instrument.saturating_sub(1) as usize;
                if let Some(instr) = module.module.instruments.get(instr_idx) {
                    if let Some(ref env) = instr.volume_envelope {
                        if env.enabled {
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

            // Apply panning with envelope
            let mut pan = channel.panning;
            if channel.panning_envelope_enabled {
                let instr_idx = channel.instrument.saturating_sub(1) as usize;
                if let Some(instr) = module.module.instruments.get(instr_idx) {
                    if let Some(ref env) = instr.panning_envelope {
                        if env.enabled {
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
        let module = match self
            .modules
            .get(state.handle as usize)
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

            // Sample with interpolation
            let sample = sample_channel(channel, &sound.data, sample_rate);

            // Apply volume with envelope processing
            let mut vol = channel.volume;

            // Apply volume envelope if enabled
            if channel.volume_envelope_enabled {
                let instr_idx = channel.instrument.saturating_sub(1) as usize;
                if let Some(instr) = module.module.instruments.get(instr_idx) {
                    if let Some(ref env) = instr.volume_envelope {
                        if env.enabled {
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

            // Apply panning with envelope
            let mut pan = channel.panning;
            if channel.panning_envelope_enabled {
                let instr_idx = channel.instrument.saturating_sub(1) as usize;
                if let Some(instr) = module.module.instruments.get(instr_idx) {
                    if let Some(ref env) = instr.panning_envelope {
                        if env.enabled {
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
            }

            // Check if we need to advance to next row
            if state.tick >= state.speed {
                state.tick = 0;

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
                        .get(state.handle as usize)
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
                        loaded.module.song_length,
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

    let sample2 = if pos + 1 < data.len() {
        data[pos + 1] as f32 / 32768.0
    } else if channel.sample_loop_type != 0 && channel.sample_loop_end > channel.sample_loop_start {
        // Wrap to loop start for smooth loop interpolation
        let loop_start = channel.sample_loop_start as usize;
        if loop_start < data.len() {
            data[loop_start] as f32 / 32768.0
        } else {
            sample1
        }
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

/// FT2 16-point quarter-sine lookup table for vibrato/tremolo
/// Values represent sin(i * π/32) * 127 for i = 0..15
const SINE_LUT: [i8; 16] = [0, 12, 24, 37, 48, 60, 71, 81, 90, 98, 106, 112, 118, 122, 125, 127];

/// Get waveform value for vibrato/tremolo
///
/// Uses FT2-compatible integer lookup tables for byte-exact compatibility.
///
/// Waveform types:
/// - 0: Sine (FT2 LUT with quadrant mirroring)
/// - 1: Ramp down (FT2-style sawtooth)
/// - 2: Square
/// - 3: Random (deterministic pseudo-random)
fn get_waveform_value(waveform: u8, position: u8) -> f32 {
    let pos = position & 0x3F; // 0-63

    match waveform & 0x03 {
        0 => {
            // FT2 sine LUT with quadrant mirroring
            // Quarter 0 (0-15): ascending from 0 to 127
            // Quarter 1 (16-31): descending from 127 to 0
            // Quarter 2 (32-47): ascending from 0 to -127
            // Quarter 3 (48-63): descending from -127 to 0
            let idx = (pos & 0x0F) as usize;
            let val = if (pos & 0x10) != 0 {
                // Quarters 1 and 3: mirror the LUT
                SINE_LUT[15 - idx]
            } else {
                // Quarters 0 and 2: direct LUT lookup
                SINE_LUT[idx]
            };
            let signed = if pos < 32 { val } else { -val };
            signed as f32 / 127.0
        }
        1 => {
            // Ramp down (FT2-style sawtooth)
            // FT2/libxm: "ramp down table is upside down" - starts high, goes low
            // Position 0 = +1.0, position 32 = -1.0, position 63 = ~+1.0
            let ramp = 32i8 - (pos as i8);
            (ramp as f32) / 32.0
        }
        2 => {
            // Square wave: 1.0 for first half, -1.0 for second
            if pos < 32 { 1.0 } else { -1.0 }
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
/// XM frequency formula:
/// Frequency = 8363 * 2^((4608 - Period) / 768)
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

    8363.0 * freq_frac * octave_scale
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
        // XM frequency is sample playback rate, not musical note frequency
        // C-4 (note 49) with 8363 Hz sample base produces 8363 Hz playback
        // Period = 10*12*16*4 - 48*16*4 = 7680 - 3072 = 4608
        // exp = (4608 - 4608) / 768 = 0, freq = 8363 * 2^0 = 8363
        let period = note_to_period(49, 0);
        let freq = period_to_frequency(period);
        assert!(
            (freq - 8363.0).abs() < 1.0,
            "Expected ~8363 Hz, got {}",
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
            8363.0 * 2.0_f32.powf(exp)
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
}
