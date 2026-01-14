//! Audio generation thread implementation
//!
//! Runs audio generation on a separate thread using predictive generation.

use std::sync::mpsc;
use std::sync::{Arc, Condvar, Mutex};
use std::thread;
use std::time::Duration;

use ringbuf::HeapProd;
use ringbuf::traits::{Observer, Producer};
use tracing::{debug, trace, warn};

use crate::audio::generate_audio_frame_with_tracker;
use crate::state::{AudioPlaybackState, TrackerState};
use crate::tracker::TrackerEngine;

use super::handle::AudioGenHandle;
use super::metrics::{AudioMetrics, LOW_BUFFER_THRESHOLD, RING_BUFFER_CAPACITY};
use super::snapshot::AudioGenSnapshot;

/// Audio generation thread state
///
/// Uses a **predictive generation** architecture:
/// - Audio thread is authoritative for timing (positions)
/// - Main thread is authoritative for game events (what sounds play)
/// - Snapshots MERGE new information, never reset positions (except rollback)
#[cfg_attr(test, allow(dead_code))]
pub(super) struct AudioGenThread {
    /// Receive snapshots from main thread
    rx: mpsc::Receiver<AudioGenSnapshot>,

    /// Ring buffer producer for output samples
    producer: HeapProd<f32>,

    /// Condition variable for signaling from cpal callback
    /// When cpal consumes samples, it notifies this condvar to wake the audio thread
    condvar: Arc<(Mutex<bool>, Condvar)>,

    /// Pre-allocated output buffer (reused each frame)
    output_buffer: Vec<f32>,

    /// Local tracker engine for sample generation
    tracker_engine: TrackerEngine,

    /// Output sample rate
    sample_rate: u32,

    // === Predictive state (audio thread is authoritative for timing) ===
    /// Current generation state - continuously advanced by audio thread
    gen_audio: AudioPlaybackState,

    /// Current tracker state - continuously advanced by audio thread
    gen_tracker: TrackerState,

    /// Last confirmed snapshot from main thread (authoritative for game state)
    /// Used for sound data reference and tick rate
    last_snapshot: Option<AudioGenSnapshot>,

    /// Samples generated since last snapshot was applied
    /// Used to track how far ahead we've predicted
    samples_since_snapshot: u64,

    /// Whether we have received at least one snapshot
    has_state: bool,

    // === Crossfade state (for rollback transitions) ===
    /// Last stereo sample pair for continuity tracking
    prev_frame_last: (f32, f32),

    /// Crossfade length in stereo sample pairs (~1ms at 44.1kHz)
    crossfade_samples: usize,

    /// Whether crossfade is currently active
    crossfade_active: bool,

    /// Sample values to crossfade FROM (captured before state reset)
    crossfade_from: (f32, f32),

    /// Performance and health metrics for diagnostics
    metrics: AudioMetrics,
}

impl AudioGenThread {
    /// Spawn the audio generation thread
    ///
    /// Returns a handle for sending snapshots to the thread.
    pub fn spawn(producer: HeapProd<f32>, sample_rate: u32) -> AudioGenHandle {
        // Bounded channel with capacity for ~8 frames
        // This provides enough buffer for jitter while limiting memory
        let (tx, rx) = mpsc::sync_channel::<AudioGenSnapshot>(8);

        // Condition variable for efficient signaling between cpal callback and audio thread
        // cpal notifies when it consumes samples, waking the audio thread to generate more
        let condvar = Arc::new((Mutex::new(false), Condvar::new()));
        let condvar_clone = condvar.clone();

        let handle = thread::Builder::new()
            .name("audio-gen".into())
            .spawn(move || {
                let mut audio_gen = Self {
                    rx,
                    producer,
                    condvar: condvar_clone,
                    output_buffer: Vec::with_capacity(2048),
                    tracker_engine: TrackerEngine::new(),
                    sample_rate,
                    // Predictive state - starts empty, filled by first snapshot
                    gen_audio: AudioPlaybackState::default(),
                    gen_tracker: TrackerState::default(),
                    last_snapshot: None,
                    samples_since_snapshot: 0,
                    has_state: false,
                    // Crossfade state
                    prev_frame_last: (0.0, 0.0),
                    crossfade_samples: 44, // ~1ms at 44.1kHz stereo
                    crossfade_active: false,
                    crossfade_from: (0.0, 0.0),
                    metrics: AudioMetrics::new(),
                };
                audio_gen.run();
            })
            .expect("failed to spawn audio generation thread");

        AudioGenHandle {
            tx: Some(tx),
            handle: Some(handle),
            condvar,
        }
    }

    /// Main thread loop - PREDICTIVE generation with validation
    ///
    /// Key insight: Audio thread is authoritative for timing (positions).
    /// Snapshots MERGE new information (new SFX, volume changes), they don't reset positions.
    /// Only rollbacks cause position resets (with crossfade to hide the discontinuity).
    fn run(&mut self) {
        debug!("Audio generation thread started (predictive mode)");

        loop {
            // 1. Check for new snapshots (non-blocking)
            //    Snapshots validate our prediction and merge new game events
            match self.rx.try_recv() {
                Ok(snapshot) => {
                    self.handle_snapshot(snapshot);
                }
                Err(mpsc::TryRecvError::Empty) => {
                    // No new snapshot - that's fine, we continue predicting ahead
                }
                Err(mpsc::TryRecvError::Disconnected) => {
                    debug!("Audio generation thread exiting (channel disconnected)");
                    break;
                }
            }

            // 2. Track buffer fill metrics
            let buffer_filled = RING_BUFFER_CAPACITY - self.producer.vacant_len();
            self.metrics.update_buffer_fill(buffer_filled);

            // 3. Generate if buffer has space (continuous predictive generation)
            let frame_size = 1470; // ~735 stereo samples per frame at 60fps
            if self.producer.vacant_len() >= frame_size {
                if buffer_filled < LOW_BUFFER_THRESHOLD {
                    self.metrics.buffer_underruns += 1;
                }

                if self.has_state {
                    self.generate_frame();
                } else {
                    self.generate_silence();
                }
            }

            // 4. Wait with timeout (prevents busy loop)
            // cpal signals wake us up immediately when buffer space is available
            let (lock, cvar) = &*self.condvar;
            let guard = lock.lock().unwrap_or_else(|e| {
                tracing::warn!("Audio thread condvar mutex poisoned; continuing");
                e.into_inner()
            });
            let _ = cvar
                .wait_timeout(guard, Duration::from_millis(1))
                .unwrap_or_else(|e| {
                    tracing::warn!("Audio thread condvar wait mutex poisoned; continuing");
                    e.into_inner()
                });

            // Log metrics periodically
            self.metrics.maybe_log();
        }

        debug!("Audio generation thread finished");
    }

    /// Handle a snapshot using validation-based merging
    ///
    /// Key insight: Snapshots validate our prediction, they don't correct our position.
    /// We never go backwards (that would cause pops). We only need to:
    /// 1. Merge NEW information (new SFX, volume changes)
    /// 2. Correct on ROLLBACK (with crossfade)
    fn handle_snapshot(&mut self, snapshot: AudioGenSnapshot) {
        self.metrics.snapshots_received += 1;

        // Rollback: our prediction was wrong, reset with crossfade
        if snapshot.is_rollback {
            self.handle_rollback(snapshot);
            return;
        }

        // First snapshot: initialize everything
        if !self.has_state {
            self.gen_audio = snapshot.audio;
            self.gen_tracker = snapshot.tracker;
            self.tracker_engine
                .apply_snapshot(&snapshot.tracker_snapshot);
            self.last_snapshot = Some(snapshot);
            self.samples_since_snapshot = 0;
            self.has_state = true;
            trace!("Initialized from first snapshot");
            return;
        }

        // Normal snapshot: VALIDATE our prediction and MERGE new information
        //
        // Example scenario:
        //   - We started from snapshot frame 0
        //   - We predicted ahead and generated frames 0, 1, 2 (now at position ~2.x)
        //   - Main thread sends snapshot for frame 1
        //   - We DON'T go back to frame 1 - that would cause a pop!
        //   - We merge: any NEW sounds that started, volume/pan changes
        //   - We continue from our current position

        // Merge new SFX that started (sound != 0 with position == 0 OR sound ID changed)
        for (i, snap_channel) in snapshot.audio.channels.iter().enumerate() {
            let sound_changed = snap_channel.sound != self.gen_audio.channels[i].sound;
            if snap_channel.sound != 0 && (snap_channel.position == 0 || sound_changed) {
                // New SFX started OR switched to different sound - start it fresh
                // Use crossfade if we were already playing something (sound changed mid-playback)
                if sound_changed && self.gen_audio.channels[i].sound != 0 {
                    self.crossfade_active = true;
                    self.crossfade_from = self.prev_frame_last;
                    trace!(
                        "SFX change on channel {} ({} -> {}), scheduling crossfade",
                        i, self.gen_audio.channels[i].sound, snap_channel.sound
                    );
                }
                self.gen_audio.channels[i] = *snap_channel;
                trace!(
                    "Merged new SFX on channel {}: sound {}",
                    i, snap_channel.sound
                );
            } else if snap_channel.sound == 0 && self.gen_audio.channels[i].sound != 0 {
                // SFX was stopped by game - stop it (instant, no pop needed for stop)
                self.gen_audio.channels[i].sound = 0;
                trace!("Stopped SFX on channel {}", i);
            } else if snap_channel.sound != 0 {
                // Existing SFX - update volume/pan (cosmetic, no position change)
                self.gen_audio.channels[i].volume = snap_channel.volume;
                self.gen_audio.channels[i].pan = snap_channel.pan;
                // DON'T update position - we're authoritative for timing
            }
        }

        // Same for music channel - also detect sound ID change (song switch)
        let music_changed = snapshot.audio.music.sound != self.gen_audio.music.sound;
        if snapshot.audio.music.sound != 0 && (snapshot.audio.music.position == 0 || music_changed)
        {
            // New music started OR switched to different song
            // Use crossfade if we were already playing music (song changed mid-playback)
            if music_changed && self.gen_audio.music.sound != 0 {
                self.crossfade_active = true;
                self.crossfade_from = self.prev_frame_last;
                trace!(
                    "Music change ({} -> {}), scheduling crossfade",
                    self.gen_audio.music.sound, snapshot.audio.music.sound
                );
            }
            self.gen_audio.music = snapshot.audio.music;
            trace!("Merged new music: sound {}", snapshot.audio.music.sound);
        } else if snapshot.audio.music.sound == 0 && self.gen_audio.music.sound != 0 {
            // Music was stopped
            self.gen_audio.music.sound = 0;
        } else if snapshot.audio.music.sound != 0 {
            // Update volume/pan only (same song continuing)
            self.gen_audio.music.volume = snapshot.audio.music.volume;
            self.gen_audio.music.pan = snapshot.audio.music.pan;
        }

        // Tracker: detect module change (new song) and merge controllable values
        let tracker_changed = snapshot.tracker.handle != self.gen_tracker.handle;
        if tracker_changed && snapshot.tracker.handle != 0 {
            // New tracker module started
            if self.gen_tracker.handle != 0 {
                // Schedule crossfade from old song to new
                self.crossfade_active = true;
                self.crossfade_from = self.prev_frame_last;
                trace!(
                    "Tracker change ({} -> {}), scheduling crossfade",
                    self.gen_tracker.handle, snapshot.tracker.handle
                );
            }
            // Full reset of tracker state for new module
            self.gen_tracker = snapshot.tracker;
            self.tracker_engine
                .apply_snapshot(&snapshot.tracker_snapshot);
            trace!("Merged new tracker: handle {}", snapshot.tracker.handle);
        } else if snapshot.tracker.handle == 0 && self.gen_tracker.handle != 0 {
            // Tracker was stopped
            self.gen_tracker.handle = 0;
            self.gen_tracker.flags = 0;
        } else if snapshot.tracker.handle != 0 {
            // Same tracker continuing - merge controllable values (volume, flags, tempo, speed)
            // DON'T update order_position, row, tick - we're authoritative for timing
            self.gen_tracker.volume = snapshot.tracker.volume;
            self.gen_tracker.flags = snapshot.tracker.flags;
            self.gen_tracker.bpm = snapshot.tracker.bpm;
            self.gen_tracker.speed = snapshot.tracker.speed;
        }

        // Update reference snapshot (for sound data access) and reset counter
        self.last_snapshot = Some(snapshot);
        self.samples_since_snapshot = 0;
    }

    /// Handle a rollback snapshot - full reset with crossfade
    fn handle_rollback(&mut self, snapshot: AudioGenSnapshot) {
        self.metrics.rollbacks_processed += 1;
        debug!("Audio thread: rollback detected, resetting with crossfade");

        // Drain pending snapshots (they're all invalid now)
        while self.rx.try_recv().is_ok() {}

        // Schedule crossfade from current audio to rollback state
        self.crossfade_active = true;
        self.crossfade_from = self.prev_frame_last;

        // Full reset to rollback state
        self.gen_audio = snapshot.audio;
        self.gen_tracker = snapshot.tracker;
        self.tracker_engine
            .apply_snapshot(&snapshot.tracker_snapshot);
        self.last_snapshot = Some(snapshot);
        self.samples_since_snapshot = 0;

        debug!("Rollback applied, crossfade scheduled");
    }

    /// Generate one frame of audio using predictive state
    ///
    /// This uses our current gen_audio/gen_tracker state (which we're authoritative for)
    /// and advances positions. Called continuously based on buffer fill level.
    fn generate_frame(&mut self) {
        let Some(ref snapshot) = self.last_snapshot else {
            // No snapshot yet - can't generate
            self.generate_silence();
            return;
        };

        // Time the generation for performance monitoring
        let start = std::time::Instant::now();

        // Generate samples using our predictive state
        self.output_buffer.clear();
        generate_audio_frame_with_tracker(
            &mut self.gen_audio,
            &mut self.gen_tracker,
            &mut self.tracker_engine,
            &snapshot.sounds,
            snapshot.tick_rate,
            snapshot.sample_rate,
            &mut self.output_buffer,
        );

        // Track samples generated for prediction tracking
        self.samples_since_snapshot += self.output_buffer.len() as u64;

        // Apply crossfade if scheduled (rollback recovery)
        if self.crossfade_active {
            self.apply_crossfade();
        }

        // Update generation timing metrics
        let elapsed_us = start.elapsed().as_micros() as f64;
        self.metrics.avg_generation_time_us =
            0.1 * elapsed_us + 0.9 * self.metrics.avg_generation_time_us;

        // Check for sample discontinuities at frame boundary
        // This is the critical point where pops occur - the first sample of this frame
        // should be continuous with the last sample of the previous frame
        if self.output_buffer.len() >= 2 {
            let (prev_l, prev_r) = self.prev_frame_last;
            let curr_l = self.output_buffer[0];
            let curr_r = self.output_buffer[1];
            // Check both channels for discontinuities
            let jump_l = (curr_l - prev_l).abs();
            let jump_r = (curr_r - prev_r).abs();
            let max_jump = jump_l.max(jump_r);
            if max_jump > 0.3 {
                self.metrics.discontinuities += 1;
                if self.metrics.discontinuities <= 10
                    || self.metrics.discontinuities.is_multiple_of(100)
                {
                    warn!(
                        "Audio discontinuity at frame boundary: L={:.3}->{:.3} (d{:.3}), R={:.3}->{:.3} (d{:.3})",
                        prev_l, curr_l, jump_l, prev_r, curr_r, jump_r
                    );
                }
            }
        }

        // Update prev_frame_last for continuity tracking (AFTER discontinuity check)
        if self.output_buffer.len() >= 2 {
            let len = self.output_buffer.len();
            self.prev_frame_last = (self.output_buffer[len - 2], self.output_buffer[len - 1]);
        }

        // Push to ring buffer
        let pushed = self.producer.push_slice(&self.output_buffer);
        if pushed < self.output_buffer.len() {
            self.metrics.buffer_overruns += 1;
            trace!(
                "Audio buffer full: dropped {} samples",
                self.output_buffer.len() - pushed
            );
        }

        // Update metrics
        self.metrics.frames_generated += 1;
        self.metrics.samples_generated += pushed as u64;
    }

    /// Apply crossfade to smooth transitions after rollback
    ///
    /// This linearly interpolates from the last sample values before the rollback
    /// to the new sample values, preventing audible clicks.
    fn apply_crossfade(&mut self) {
        if self.output_buffer.len() < 2 {
            self.crossfade_active = false;
            return;
        }

        let (old_l, old_r) = self.crossfade_from;
        let fade_len = self.crossfade_samples.min(self.output_buffer.len() / 2);

        // Linear crossfade from old position to new
        for i in 0..fade_len {
            let t = i as f32 / fade_len as f32;
            let idx_l = i * 2;
            let idx_r = i * 2 + 1;
            self.output_buffer[idx_l] = old_l * (1.0 - t) + self.output_buffer[idx_l] * t;
            self.output_buffer[idx_r] = old_r * (1.0 - t) + self.output_buffer[idx_r] * t;
        }

        self.crossfade_active = false;
        trace!("Applied crossfade over {} stereo samples", fade_len);
    }

    /// Generate silence to prevent ring buffer underrun
    fn generate_silence(&mut self) {
        // Generate ~16ms of silence at current sample rate
        let silence_samples = (self.sample_rate as usize / 60) * 2; // stereo
        self.output_buffer.clear();
        self.output_buffer.resize(silence_samples, 0.0);

        let pushed = self.producer.push_slice(&self.output_buffer);
        if pushed > 0 {
            trace!("Generated {} silence samples", pushed);
        }
    }
}
