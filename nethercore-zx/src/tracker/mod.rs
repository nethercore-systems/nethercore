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

mod channels;
mod engine;
mod state;
mod utils;

pub use channels::TrackerChannel;
pub use state::{CachedRowState, RowStateCache};
pub use utils::{
    LINEAR_FREQ_TABLE, SINE_LUT, SINE_LUT_64, apply_channel_pan, apply_it_linear_slide,
    fast_pan_gains, get_waveform_value, note_to_period, period_to_frequency, sample_channel,
    samples_per_tick,
};

use nether_it::ItModule;
use nether_tracker::TrackerModule;
use nether_xm::XmModule;

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
    pub(crate) modules: Vec<Option<LoadedModule>>,

    /// Per-channel playback state
    pub(crate) channels: [TrackerChannel; MAX_TRACKER_CHANNELS],

    /// Global volume (0.0-1.0)
    pub(crate) global_volume: f32,

    /// Next handle to allocate
    pub(crate) next_handle: u32,

    /// Current playback position (for sync detection)
    pub(crate) current_order: u16,
    pub(crate) current_row: u16,
    pub(crate) current_tick: u16,

    /// Samples rendered within current tick
    pub(crate) tick_samples_rendered: u32,

    /// Row state cache for fast rollback seeks
    pub(crate) row_cache: RowStateCache,

    /// Pattern delay (EEx) - number of times to repeat current row
    pub(crate) pattern_delay: u8,
    /// Pattern delay counter - tracks how many times row has been repeated
    pub(crate) pattern_delay_count: u8,

    /// Fine pattern delay (S6x) - extra ticks to add to current row
    pub(crate) fine_pattern_delay: u8,

    /// Global volume slide memory (Hxy effect)
    pub(crate) last_global_vol_slide: u8,

    /// Whether current module is IT format (affects vibrato depth, etc.)
    pub(crate) is_it_format: bool,

    /// Old effects mode (S3M compatibility - affects vibrato/tremolo depth)
    pub(crate) old_effects_mode: bool,

    /// Link G memory with E/F for portamento
    pub(crate) link_g_memory: bool,

    /// Tempo slide amount per tick (positive = up, negative = down, 0 = none)
    pub(crate) tempo_slide: i8,
}

/// A loaded tracker module with resolved sample handles
#[derive(Debug)]
pub(crate) struct LoadedModule {
    /// Parsed tracker module data (unified format)
    pub module: TrackerModule,
    /// Sound handles for each instrument (instrument index -> sound handle)
    pub sound_handles: Vec<u32>,
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tracker_handle_flag() {
        let raw_handle = 5u32;
        let flagged = raw_handle | TRACKER_HANDLE_FLAG;

        assert!(is_tracker_handle(flagged));
        assert!(!is_tracker_handle(raw_handle));
        assert_eq!(raw_tracker_handle(flagged), raw_handle);
    }

    #[test]
    fn test_engine_creation() {
        let engine = TrackerEngine::new();
        assert_eq!(engine.global_volume, 1.0);
        assert_eq!(engine.next_handle, 1);
        assert_eq!(engine.current_order, 0);
        assert_eq!(engine.current_row, 0);
    }
}
