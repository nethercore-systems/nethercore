//! Audio generation snapshot
//!
//! Captures all state needed to generate audio samples for one frame.

use std::sync::Arc;

use crate::audio::Sound;
use crate::state::{AudioPlaybackState, TrackerState};
use crate::tracker::TrackerEngineSnapshot;

/// Snapshot of audio state sent from main thread to audio generation thread
///
/// This captures all state needed to generate audio samples for one frame.
/// Created on the main thread after each confirmed game tick.
#[derive(Clone)]
pub struct AudioGenSnapshot {
    /// SFX channel states (positions, volumes, pans)
    pub audio: AudioPlaybackState,

    /// Tracker position state (order, row, tick, etc.)
    pub tracker: TrackerState,

    /// Tracker engine snapshot (channel states, modules)
    pub tracker_snapshot: TrackerEngineSnapshot,

    /// Sound data - Arc for sharing without copying
    pub sounds: Arc<Vec<Option<Sound>>>,

    /// Frame identifier for ordering and debugging
    pub frame_number: i32,

    /// Game tick rate (e.g., 60 for 60fps)
    pub tick_rate: u32,

    /// Output sample rate (e.g., 44100)
    pub sample_rate: u32,

    /// If true, this is a rollback - discard pending work
    pub is_rollback: bool,
}

impl AudioGenSnapshot {
    pub fn new(
        audio: AudioPlaybackState,
        tracker: TrackerState,
        tracker_snapshot: TrackerEngineSnapshot,
        sounds: Arc<Vec<Option<Sound>>>,
        frame_number: i32,
        tick_rate: u32,
        sample_rate: u32,
        is_rollback: bool,
    ) -> Self {
        Self {
            audio,
            tracker,
            tracker_snapshot,
            sounds,
            frame_number,
            tick_rate,
            sample_rate,
            is_rollback,
        }
    }
}
