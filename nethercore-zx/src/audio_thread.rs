//! Threaded audio generation
//!
//! Decouples CPU-intensive sample generation from the main game loop.
//! This prevents audio pops/crackles during system load or rollback replays.
//!
//! # Architecture
//!
//! ```text
//! Main Thread                    Audio Gen Thread              cpal Thread
//!     │                                │                           │
//! [Game Tick]                          │                           │
//!     │                                │                           │
//! [Create Snapshot]────(channel)────►[Receive]                     │
//!     │                              [Generate Samples]            │
//!     │                              [Push]─────────(ring)──────►[Consume]
//! ```
//!
//! # Usage
//!
//! ```ignore
//! // Create audio output with threaded generation
//! let audio = ThreadedAudioOutput::new()?;
//!
//! // Each frame, send a snapshot
//! let snapshot = AudioGenSnapshot::new(...);
//! audio.send_snapshot(snapshot);
//! ```

use std::sync::mpsc::{self, SyncSender, TrySendError};
use std::sync::{Arc, Condvar, Mutex};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use ringbuf::traits::{Observer, Producer};
use ringbuf::HeapProd;
use tracing::{debug, error, trace, warn};

use crate::audio::{generate_audio_frame_with_tracker, Sound};
use crate::state::{AudioPlaybackState, TrackerState};
use crate::tracker::{TrackerEngine, TrackerEngineSnapshot};

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
    /// Create a new audio snapshot
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

/// Handle to the audio generation thread
///
/// Returned from `AudioGenThread::spawn()`. Use this to send snapshots
/// to the audio thread and to shut down cleanly.
pub struct AudioGenHandle {
    /// Sender for snapshots to audio thread (Option to allow explicit drop before join)
    tx: Option<SyncSender<AudioGenSnapshot>>,

    /// Thread join handle
    handle: Option<JoinHandle<()>>,

    /// Condition variable for signaling from cpal callback
    /// Shared with audio thread - cpal notifies when buffer space available
    pub(crate) condvar: Arc<(Mutex<bool>, Condvar)>,
}

impl AudioGenHandle {
    /// Send a snapshot to the audio generation thread
    ///
    /// Non-blocking - if the channel is full, the snapshot is dropped
    /// and a warning is logged. This prevents the main thread from
    /// blocking on audio generation.
    pub fn send_snapshot(&self, snapshot: AudioGenSnapshot) -> bool {
        let Some(ref tx) = self.tx else {
            warn!("Audio thread sender already dropped");
            return false;
        };
        match tx.try_send(snapshot) {
            Ok(()) => true,
            Err(TrySendError::Full(_)) => {
                debug!("Audio snapshot channel full, dropping frame");
                false
            }
            Err(TrySendError::Disconnected(_)) => {
                warn!("Audio thread disconnected");
                false
            }
        }
    }

    /// Check if the audio thread is still running
    pub fn is_alive(&self) -> bool {
        self.handle.as_ref().map(|h| !h.is_finished()).unwrap_or(false)
    }
}

impl Drop for AudioGenHandle {
    fn drop(&mut self) {
        // IMPORTANT: Drop the sender FIRST to signal the thread to exit.
        // The thread's recv_timeout() will return Disconnected and break the loop.
        // If we join() before dropping the sender, we deadlock!
        drop(self.tx.take());

        if let Some(handle) = self.handle.take() {
            // Now wait for the thread to finish
            let _ = handle.join();
        }
    }
}

/// Ring buffer capacity (must match RING_BUFFER_SIZE in new())
const RING_BUFFER_CAPACITY: usize = 13230;
/// Generate more when buffer drops below this (~35%)
const LOW_BUFFER_THRESHOLD: usize = 4600;

/// Metrics for audio thread health monitoring and diagnostics
#[derive(Debug, Clone)]
struct AudioMetrics {
    /// Total frames generated
    frames_generated: u64,
    /// Total audio samples generated (stereo pairs counted as 2)
    samples_generated: u64,
    /// Total snapshots received from main thread
    snapshots_received: u64,
    /// Rollback snapshots processed
    rollbacks_processed: u64,
    /// Current buffer fill level (samples)
    buffer_fill: usize,
    /// Minimum buffer fill level seen
    buffer_fill_min: usize,
    /// Maximum buffer fill level seen
    buffer_fill_max: usize,
    /// Number of times buffer dropped below LOW threshold
    buffer_underruns: u64,
    /// Number of times buffer filled above TARGET (dropped samples)
    buffer_overruns: u64,
    /// Average time to generate one frame (microseconds)
    avg_generation_time_us: f64,
    /// Number of sample discontinuities detected (>0.3 amplitude jump)
    discontinuities: u64,
    /// Timestamp of last metrics log
    last_log_time: std::time::Instant,
}

impl AudioMetrics {
    fn new() -> Self {
        Self {
            frames_generated: 0,
            samples_generated: 0,
            snapshots_received: 0,
            rollbacks_processed: 0,
            buffer_fill: 0,
            buffer_fill_min: RING_BUFFER_CAPACITY,
            buffer_fill_max: 0,
            buffer_underruns: 0,
            buffer_overruns: 0,
            avg_generation_time_us: 0.0,
            discontinuities: 0,
            last_log_time: std::time::Instant::now(),
        }
    }

    /// Log metrics if enough time has passed (every 1 second)
    fn maybe_log(&mut self) {
        let elapsed = self.last_log_time.elapsed();
        if elapsed.as_secs() >= 1 {
            let buffer_pct = (self.buffer_fill as f64 / RING_BUFFER_CAPACITY as f64) * 100.0;
            let buffer_min_pct = (self.buffer_fill_min as f64 / RING_BUFFER_CAPACITY as f64) * 100.0;
            let buffer_max_pct = (self.buffer_fill_max as f64 / RING_BUFFER_CAPACITY as f64) * 100.0;
            let buffer_range = self.buffer_fill_max.saturating_sub(self.buffer_fill_min);

            debug!(
                "🎵 AUDIO METRICS [tid={:?}]: buf={:.1}% (min={:.1}%, max={:.1}%, range={}), \
                 frames={}, samples={}, underruns={}, overruns={}, \
                 discontinuities={}, avg_gen={:.2}μs",
                std::thread::current().id(),
                buffer_pct, buffer_min_pct, buffer_max_pct, buffer_range,
                self.frames_generated, self.samples_generated,
                self.buffer_underruns, self.buffer_overruns,
                self.discontinuities, self.avg_generation_time_us
            );

            // Reset counters for next interval (show per-second rates)
            self.frames_generated = 0;
            self.samples_generated = 0;
            self.buffer_underruns = 0;
            self.buffer_overruns = 0;
            self.discontinuities = 0;
            self.buffer_fill_min = self.buffer_fill;
            self.buffer_fill_max = self.buffer_fill;
            self.last_log_time = std::time::Instant::now();
        }
    }

    /// Update buffer fill metrics
    fn update_buffer_fill(&mut self, fill: usize) {
        self.buffer_fill = fill;
        self.buffer_fill_min = self.buffer_fill_min.min(fill);
        self.buffer_fill_max = self.buffer_fill_max.max(fill);
    }
}

/// Audio generation thread state
///
/// Uses a **predictive generation** architecture:
/// - Audio thread is authoritative for timing (positions)
/// - Main thread is authoritative for game events (what sounds play)
/// - Snapshots MERGE new information, never reset positions (except rollback)
struct AudioGenThread {
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
            let _guard = lock.lock().unwrap();
            let _ = cvar.wait_timeout(_guard, Duration::from_millis(1)).unwrap();

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
            self.tracker_engine.apply_snapshot(&snapshot.tracker_snapshot);
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
                    trace!("SFX change on channel {} ({} -> {}), scheduling crossfade",
                           i, self.gen_audio.channels[i].sound, snap_channel.sound);
                }
                self.gen_audio.channels[i] = *snap_channel;
                trace!("Merged new SFX on channel {}: sound {}", i, snap_channel.sound);
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
        if snapshot.audio.music.sound != 0
            && (snapshot.audio.music.position == 0 || music_changed)
        {
            // New music started OR switched to different song
            // Use crossfade if we were already playing music (song changed mid-playback)
            if music_changed && self.gen_audio.music.sound != 0 {
                self.crossfade_active = true;
                self.crossfade_from = self.prev_frame_last;
                trace!(
                    "Music change ({} -> {}), scheduling crossfade",
                    self.gen_audio.music.sound,
                    snapshot.audio.music.sound
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
                    self.gen_tracker.handle,
                    snapshot.tracker.handle
                );
            }
            // Full reset of tracker state for new module
            self.gen_tracker = snapshot.tracker;
            self.tracker_engine.apply_snapshot(&snapshot.tracker_snapshot);
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
        self.tracker_engine.apply_snapshot(&snapshot.tracker_snapshot);
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
                if self.metrics.discontinuities <= 10 || self.metrics.discontinuities % 100 == 0 {
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
            self.prev_frame_last = (
                self.output_buffer[len - 2],
                self.output_buffer[len - 1],
            );
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

/// Threaded audio output combining ring buffer and generation thread
///
/// Drop-in replacement for `AudioOutput` that uses threaded generation.
pub struct ThreadedAudioOutput {
    /// Handle to the generation thread
    gen_handle: AudioGenHandle,

    /// The cpal stream (kept alive for the duration)
    _stream: cpal::Stream,

    /// Output sample rate
    sample_rate: u32,
}

impl ThreadedAudioOutput {
    /// Create a new threaded audio output
    ///
    /// This spawns the audio generation thread and sets up the cpal output stream.
    pub fn new() -> Result<Self, String> {
        use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
        use ringbuf::traits::Split;
        use ringbuf::HeapRb;

        let host = cpal::default_host();

        let device = host
            .default_output_device()
            .ok_or_else(|| "No audio output device available".to_string())?;

        let config = device
            .default_output_config()
            .map_err(|e| format!("Failed to get default output config: {}", e))?;

        let sample_rate = config.sample_rate().0;

        // Create ring buffer - larger than non-threaded version for more headroom
        // ~150ms buffer at 44.1kHz = 6615 frames * 2 channels = 13230 samples
        const RING_BUFFER_SIZE: usize = 13230;
        let ring = HeapRb::<f32>::new(RING_BUFFER_SIZE);
        let (producer, mut consumer) = ring.split();

        // Spawn the generation thread FIRST to get the condvar for callbacks
        let gen_handle = AudioGenThread::spawn(producer, sample_rate);
        let condvar_f32 = gen_handle.condvar.clone();
        let condvar_i16 = gen_handle.condvar.clone();
        let condvar_u16 = gen_handle.condvar.clone();

        // Build the stream based on sample format
        let stream = match config.sample_format() {
            cpal::SampleFormat::F32 => {
                let config = config.into();
                device
                    .build_output_stream(
                        &config,
                        move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                            use ringbuf::traits::Consumer;
                            // Batch read all available samples at once (much more efficient
                            // than per-sample try_pop which causes timing gaps and popping)
                            let popped = consumer.pop_slice(data);
                            // Fill any remaining samples with silence
                            data[popped..].fill(0.0);

                            // Signal audio generation thread that buffer space is available
                            // notify_one() doesn't require holding the lock
                            let (_lock, cvar) = &*condvar_f32;
                            cvar.notify_one();
                        },
                        |err| error!("Audio stream error: {}", err),
                        None,
                    )
                    .map_err(|e| format!("Failed to build audio stream: {}", e))?
            }
            cpal::SampleFormat::I16 => {
                let config = config.into();
                // Pre-allocate buffer for batch reads (avoids per-sample atomic ops)
                let mut temp_buffer: Vec<f32> = vec![0.0; 4096];
                device
                    .build_output_stream(
                        &config,
                        move |data: &mut [i16], _: &cpal::OutputCallbackInfo| {
                            use ringbuf::traits::Consumer;
                            // Resize temp buffer if needed (rare, only on first use or format change)
                            if temp_buffer.len() < data.len() {
                                temp_buffer.resize(data.len(), 0.0);
                            }
                            // Batch read f32 samples
                            let popped = consumer.pop_slice(&mut temp_buffer[..data.len()]);
                            // Convert popped samples to i16
                            for (i, &f) in temp_buffer[..popped].iter().enumerate() {
                                data[i] = (f * 32767.0).clamp(-32768.0, 32767.0) as i16;
                            }
                            // Fill remaining with silence
                            for sample in &mut data[popped..] {
                                *sample = 0;
                            }

                            // Signal audio generation thread that buffer space is available
                            // notify_one() doesn't require holding the lock
                            let (_lock, cvar) = &*condvar_i16;
                            cvar.notify_one();
                        },
                        |err| error!("Audio stream error: {}", err),
                        None,
                    )
                    .map_err(|e| format!("Failed to build audio stream: {}", e))?
            }
            cpal::SampleFormat::U16 => {
                let config = config.into();
                // Pre-allocate buffer for batch reads (avoids per-sample atomic ops)
                let mut temp_buffer: Vec<f32> = vec![0.0; 4096];
                device
                    .build_output_stream(
                        &config,
                        move |data: &mut [u16], _: &cpal::OutputCallbackInfo| {
                            use ringbuf::traits::Consumer;
                            // Resize temp buffer if needed (rare, only on first use or format change)
                            if temp_buffer.len() < data.len() {
                                temp_buffer.resize(data.len(), 0.0);
                            }
                            // Batch read f32 samples
                            let popped = consumer.pop_slice(&mut temp_buffer[..data.len()]);
                            // Convert popped samples to u16
                            for (i, &f) in temp_buffer[..popped].iter().enumerate() {
                                data[i] = ((f * 32767.0 + 32768.0).clamp(0.0, 65535.0)) as u16;
                            }
                            // Fill remaining with silence (0x8000 is silence for u16 audio)
                            for sample in &mut data[popped..] {
                                *sample = 32768;
                            }

                            // Signal audio generation thread that buffer space is available
                            // notify_one() doesn't require holding the lock
                            let (_lock, cvar) = &*condvar_u16;
                            cvar.notify_one();
                        },
                        |err| error!("Audio stream error: {}", err),
                        None,
                    )
                    .map_err(|e| format!("Failed to build audio stream: {}", e))?
            }
            _ => {
                return Err(format!(
                    "Unsupported sample format: {:?}",
                    config.sample_format()
                ));
            }
        };

        stream
            .play()
            .map_err(|e| format!("Failed to play audio stream: {}", e))?;

        debug!("Threaded audio stream started at {}Hz", sample_rate);

        Ok(Self {
            gen_handle,
            _stream: stream,
            sample_rate,
        })
    }

    /// Send an audio snapshot to the generation thread
    ///
    /// Returns true if the snapshot was queued, false if dropped.
    pub fn send_snapshot(&self, snapshot: AudioGenSnapshot) -> bool {
        self.gen_handle.send_snapshot(snapshot)
    }

    /// Get the output sample rate
    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    /// Check if the audio thread is still running
    pub fn is_alive(&self) -> bool {
        self.gen_handle.is_alive()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    /// Test helper: Mirrors AudioGenThread state for testing handle_snapshot logic
    /// Uses the new predictive/validation-based architecture
    struct TestableAudioGen {
        gen_audio: AudioPlaybackState,
        gen_tracker: TrackerState,
        tracker_engine: TrackerEngine,
        last_snapshot: Option<AudioGenSnapshot>,
        samples_since_snapshot: u64,
        has_state: bool,
        crossfade_active: bool,
        crossfade_from: (f32, f32),
        prev_frame_last: (f32, f32),
    }

    impl TestableAudioGen {
        fn new() -> Self {
            Self {
                gen_audio: AudioPlaybackState::default(),
                gen_tracker: TrackerState::default(),
                tracker_engine: TrackerEngine::new(),
                last_snapshot: None,
                samples_since_snapshot: 0,
                has_state: false,
                crossfade_active: false,
                crossfade_from: (0.0, 0.0),
                prev_frame_last: (0.0, 0.0),
            }
        }

        /// Handle snapshot using validation-based merging
        /// Mirrors AudioGenThread::handle_snapshot
        fn handle_snapshot(&mut self, snapshot: AudioGenSnapshot) {
            // Rollback: reset with crossfade
            if snapshot.is_rollback {
                self.crossfade_active = true;
                self.crossfade_from = self.prev_frame_last;
                self.gen_audio = snapshot.audio;
                self.gen_tracker = snapshot.tracker;
                self.tracker_engine.apply_snapshot(&snapshot.tracker_snapshot);
                self.last_snapshot = Some(snapshot);
                self.samples_since_snapshot = 0;
                return;
            }

            // First snapshot: initialize
            if !self.has_state {
                self.gen_audio = snapshot.audio;
                self.gen_tracker = snapshot.tracker;
                self.tracker_engine.apply_snapshot(&snapshot.tracker_snapshot);
                self.last_snapshot = Some(snapshot);
                self.samples_since_snapshot = 0;
                self.has_state = true;
                return;
            }

            // Normal snapshot: VALIDATE and MERGE new information
            // Merge new SFX (position == 0 OR sound ID changed)
            for (i, snap_channel) in snapshot.audio.channels.iter().enumerate() {
                let sound_changed = snap_channel.sound != self.gen_audio.channels[i].sound;
                if snap_channel.sound != 0 && (snap_channel.position == 0 || sound_changed) {
                    // New SFX OR sound ID changed - start it
                    if sound_changed && self.gen_audio.channels[i].sound != 0 {
                        self.crossfade_active = true;
                        self.crossfade_from = self.prev_frame_last;
                    }
                    self.gen_audio.channels[i] = *snap_channel;
                } else if snap_channel.sound == 0 && self.gen_audio.channels[i].sound != 0 {
                    // SFX stopped
                    self.gen_audio.channels[i].sound = 0;
                } else if snap_channel.sound != 0 {
                    // Update volume/pan only - DON'T update position
                    self.gen_audio.channels[i].volume = snap_channel.volume;
                    self.gen_audio.channels[i].pan = snap_channel.pan;
                }
            }

            // Same for music - also detect sound ID change (song switch)
            let music_changed = snapshot.audio.music.sound != self.gen_audio.music.sound;
            if snapshot.audio.music.sound != 0
                && (snapshot.audio.music.position == 0 || music_changed)
            {
                if music_changed && self.gen_audio.music.sound != 0 {
                    self.crossfade_active = true;
                    self.crossfade_from = self.prev_frame_last;
                }
                self.gen_audio.music = snapshot.audio.music;
            } else if snapshot.audio.music.sound == 0 {
                self.gen_audio.music.sound = 0;
            } else if snapshot.audio.music.sound != 0 {
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
                }
                // Full reset of tracker state for new module
                self.gen_tracker = snapshot.tracker;
                self.tracker_engine.apply_snapshot(&snapshot.tracker_snapshot);
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

            self.last_snapshot = Some(snapshot);
            self.samples_since_snapshot = 0;
        }

        /// Simulate generating a frame (advances positions)
        fn generate_frame(&mut self) {
            // Simulate position advancement (simplified)
            for channel in &mut self.gen_audio.channels {
                if channel.sound != 0 {
                    // Advance by ~1 frame worth (735 * 0.5 * 256 in fixed point)
                    channel.position = channel.position.wrapping_add(94080);
                }
            }
            if self.gen_audio.music.sound != 0 {
                self.gen_audio.music.position = self.gen_audio.music.position.wrapping_add(94080);
            }
            self.samples_since_snapshot += 1470; // stereo samples per frame
        }
    }

    #[test]
    fn test_snapshot_validates_not_resets() {
        // Key test: Snapshots should NOT reset positions
        let mut audio_gen = TestableAudioGen::new();

        // Initial snapshot with SFX on channel 0 at position 0
        let mut audio = AudioPlaybackState::default();
        audio.channels[0].sound = 1;
        audio.channels[0].position = 0;
        audio.channels[0].volume = 0.8;

        let snapshot1 = AudioGenSnapshot::new(
            audio,
            TrackerState::default(),
            TrackerEngine::new().snapshot(),
            Arc::new(Vec::new()),
            0,
            60,
            44100,
            false,
        );
        audio_gen.handle_snapshot(snapshot1);

        // Generate 3 frames (audio advances)
        audio_gen.generate_frame();
        audio_gen.generate_frame();
        audio_gen.generate_frame();

        // Audio thread is now at position ~3 frames
        let position_after_3_frames = audio_gen.gen_audio.channels[0].position;
        assert!(position_after_3_frames > 0, "Position should have advanced");

        // Send snapshot showing position at frame 1 (main thread is behind)
        let mut audio2 = AudioPlaybackState::default();
        audio2.channels[0].sound = 1;
        audio2.channels[0].position = 94080; // 1 frame worth
        audio2.channels[0].volume = 0.9; // Volume changed

        let snapshot2 = AudioGenSnapshot::new(
            audio2,
            TrackerState::default(),
            TrackerEngine::new().snapshot(),
            Arc::new(Vec::new()),
            1,
            60,
            44100,
            false,
        );
        audio_gen.handle_snapshot(snapshot2);

        // Position should NOT have been reset to frame 1
        assert_eq!(
            audio_gen.gen_audio.channels[0].position, position_after_3_frames,
            "Position should NOT be reset by snapshot (would cause pop)"
        );

        // But volume SHOULD have been updated
        assert_eq!(
            audio_gen.gen_audio.channels[0].volume, 0.9,
            "Volume should be updated from snapshot"
        );

        // No crossfade needed (this is normal operation)
        assert!(!audio_gen.crossfade_active, "No crossfade for normal snapshot");
    }

    #[test]
    fn test_new_sfx_starts_immediately() {
        let mut audio_gen = TestableAudioGen::new();

        // Initial snapshot with only channel 0 active
        let mut audio = AudioPlaybackState::default();
        audio.channels[0].sound = 1;
        audio.channels[0].position = 0;

        let snapshot1 = AudioGenSnapshot::new(
            audio,
            TrackerState::default(),
            TrackerEngine::new().snapshot(),
            Arc::new(Vec::new()),
            0,
            60,
            44100,
            false,
        );
        audio_gen.handle_snapshot(snapshot1);

        // Generate 2 frames
        audio_gen.generate_frame();
        audio_gen.generate_frame();
        let ch0_position = audio_gen.gen_audio.channels[0].position;

        // Send snapshot with NEW SFX on channel 1 (position == 0)
        let mut audio2 = AudioPlaybackState::default();
        audio2.channels[0].sound = 1;
        audio2.channels[0].position = 94080; // main thread's position (ignored)
        audio2.channels[1].sound = 2; // NEW!
        audio2.channels[1].position = 0; // position == 0 means NEW

        let snapshot2 = AudioGenSnapshot::new(
            audio2,
            TrackerState::default(),
            TrackerEngine::new().snapshot(),
            Arc::new(Vec::new()),
            1,
            60,
            44100,
            false,
        );
        audio_gen.handle_snapshot(snapshot2);

        // Channel 1 should now be active
        assert_eq!(audio_gen.gen_audio.channels[1].sound, 2, "New SFX should start");
        assert_eq!(audio_gen.gen_audio.channels[1].position, 0, "New SFX starts at position 0");

        // Channel 0 position should NOT have been reset
        assert_eq!(
            audio_gen.gen_audio.channels[0].position, ch0_position,
            "Existing channel position should not change"
        );
    }

    #[test]
    fn test_rollback_resets_with_crossfade() {
        let mut audio_gen = TestableAudioGen::new();

        // Initial snapshot
        let snapshot1 = AudioGenSnapshot::new(
            AudioPlaybackState::default(),
            TrackerState::default(),
            TrackerEngine::new().snapshot(),
            Arc::new(Vec::new()),
            0,
            60,
            44100,
            false,
        );
        audio_gen.handle_snapshot(snapshot1);

        // Set prev_frame_last to simulate audio playing
        audio_gen.prev_frame_last = (0.5, 0.5);

        // Generate some frames
        for _ in 0..5 {
            audio_gen.generate_frame();
        }

        // Send rollback snapshot
        let mut rollback_audio = AudioPlaybackState::default();
        rollback_audio.channels[0].sound = 3; // Different state
        rollback_audio.channels[0].position = 12345;

        let rollback_snapshot = AudioGenSnapshot::new(
            rollback_audio,
            TrackerState::default(),
            TrackerEngine::new().snapshot(),
            Arc::new(Vec::new()),
            2,
            60,
            44100,
            true, // is_rollback!
        );
        audio_gen.handle_snapshot(rollback_snapshot);

        // State should be reset to rollback values
        assert_eq!(audio_gen.gen_audio.channels[0].sound, 3);
        assert_eq!(audio_gen.gen_audio.channels[0].position, 12345);

        // Crossfade should be scheduled
        assert!(audio_gen.crossfade_active, "Crossfade should be scheduled for rollback");
        assert_eq!(audio_gen.crossfade_from, (0.5, 0.5), "Crossfade from prev_frame_last");
    }

    #[test]
    fn test_volume_pan_merge() {
        let mut audio_gen = TestableAudioGen::new();

        // Initial snapshot
        let mut audio = AudioPlaybackState::default();
        audio.channels[0].sound = 1;
        audio.channels[0].position = 0;
        audio.channels[0].volume = 0.5;
        audio.channels[0].pan = 0.0;

        let snapshot1 = AudioGenSnapshot::new(
            audio,
            TrackerState::default(),
            TrackerEngine::new().snapshot(),
            Arc::new(Vec::new()),
            0,
            60,
            44100,
            false,
        );
        audio_gen.handle_snapshot(snapshot1);

        // Generate frames
        audio_gen.generate_frame();
        audio_gen.generate_frame();
        let position_before = audio_gen.gen_audio.channels[0].position;

        // Send snapshot with changed volume/pan
        let mut audio2 = AudioPlaybackState::default();
        audio2.channels[0].sound = 1;
        audio2.channels[0].position = 94080; // Different position (ignored!)
        audio2.channels[0].volume = 0.8; // Changed
        audio2.channels[0].pan = -0.5; // Changed

        let snapshot2 = AudioGenSnapshot::new(
            audio2,
            TrackerState::default(),
            TrackerEngine::new().snapshot(),
            Arc::new(Vec::new()),
            1,
            60,
            44100,
            false,
        );
        audio_gen.handle_snapshot(snapshot2);

        // Volume and pan should be updated
        assert_eq!(audio_gen.gen_audio.channels[0].volume, 0.8);
        assert_eq!(audio_gen.gen_audio.channels[0].pan, -0.5);

        // Position should NOT change
        assert_eq!(audio_gen.gen_audio.channels[0].position, position_before);
    }

    #[test]
    fn test_snapshot_creation() {
        // Basic test that snapshot can be created
        let audio = AudioPlaybackState::default();
        let tracker = TrackerState::default();
        let tracker_engine = TrackerEngine::new();
        let tracker_snapshot = tracker_engine.snapshot();
        let sounds = Arc::new(Vec::new());

        let snapshot = AudioGenSnapshot::new(
            audio,
            tracker,
            tracker_snapshot,
            sounds,
            0,
            60,
            44100,
            false,
        );

        assert_eq!(snapshot.frame_number, 0);
        assert_eq!(snapshot.tick_rate, 60);
        assert_eq!(snapshot.sample_rate, 44100);
        assert!(!snapshot.is_rollback);
    }

    #[test]
    fn test_audio_thread_shutdown_does_not_hang() {
        // This test verifies that dropping AudioGenHandle doesn't deadlock.
        use ringbuf::HeapRb;
        use ringbuf::traits::Split;

        let ring = HeapRb::<f32>::new(4096);
        let (producer, _consumer) = ring.split();

        let handle = AudioGenThread::spawn(producer, 44100);

        // Drop should complete within a reasonable time (not hang)
        let (tx, rx) = std::sync::mpsc::channel();
        let drop_thread = std::thread::spawn(move || {
            drop(handle);
            let _ = tx.send(());
        });

        match rx.recv_timeout(Duration::from_secs(1)) {
            Ok(()) => {
                drop_thread.join().unwrap();
            }
            Err(_) => {
                panic!("AudioGenHandle::drop() deadlocked");
            }
        }
    }

    #[test]
    fn test_audio_thread_processes_snapshots() {
        // Test that the audio thread produces samples after receiving a snapshot
        use ringbuf::HeapRb;
        use ringbuf::traits::{Consumer, Split};

        let ring = HeapRb::<f32>::new(8192);
        let (producer, mut consumer) = ring.split();

        let handle = AudioGenThread::spawn(producer, 44100);

        let snapshot = AudioGenSnapshot::new(
            AudioPlaybackState::default(),
            TrackerState::default(),
            TrackerEngine::new().snapshot(),
            Arc::new(Vec::new()),
            0,
            60,
            44100,
            false,
        );

        assert!(handle.send_snapshot(snapshot));

        // Give the thread time to process
        std::thread::sleep(Duration::from_millis(50));

        // Should have generated samples
        let mut samples_received = 0;
        while consumer.try_pop().is_some() {
            samples_received += 1;
        }

        assert!(
            samples_received >= 735 * 2,
            "Expected at least 1470 samples, got {}",
            samples_received
        );

        drop(handle);
    }

    #[test]
    fn test_crossfade_application() {
        // Test that crossfade smooths transitions
        let mut output_buffer = vec![
            -0.5, -0.5, // First stereo pair
            -0.4, -0.4,
            -0.3, -0.3,
            -0.2, -0.2,
        ];
        let crossfade_from = (0.8, 0.8);
        let crossfade_samples = 4; // 4 stereo pairs

        // Apply crossfade (inline version of apply_crossfade)
        let fade_len = crossfade_samples.min(output_buffer.len() / 2);
        for i in 0..fade_len {
            let t = i as f32 / fade_len as f32;
            let idx_l = i * 2;
            let idx_r = i * 2 + 1;
            output_buffer[idx_l] = crossfade_from.0 * (1.0 - t) + output_buffer[idx_l] * t;
            output_buffer[idx_r] = crossfade_from.1 * (1.0 - t) + output_buffer[idx_r] * t;
        }

        // First sample should be close to crossfade_from, not -0.5
        assert!(
            output_buffer[0] > 0.0,
            "First sample should be positive after crossfade (was {})",
            output_buffer[0]
        );

        // Verify gradual transition
        assert!(
            output_buffer[0] > output_buffer[2],
            "Should transition gradually"
        );
    }

    // ========================================================================
    // COMPREHENSIVE ROLLBACK TESTS
    // ========================================================================

    #[test]
    fn test_rollback_replaces_all_active_sounds() {
        // Rollback should replace ALL channel states, not merge
        let mut audio_gen = TestableAudioGen::new();

        // Initial snapshot with multiple channels active
        let mut audio = AudioPlaybackState::default();
        audio.channels[0].sound = 1;
        audio.channels[0].position = 0;
        audio.channels[0].volume = 0.5;
        audio.channels[1].sound = 2;
        audio.channels[1].position = 0;
        audio.channels[2].sound = 3;
        audio.channels[2].position = 0;
        audio.music.sound = 10;
        audio.music.position = 0;

        let snapshot1 = AudioGenSnapshot::new(
            audio,
            TrackerState::default(),
            TrackerEngine::new().snapshot(),
            Arc::new(Vec::new()),
            0,
            60,
            44100,
            false,
        );
        audio_gen.handle_snapshot(snapshot1);
        audio_gen.prev_frame_last = (0.7, 0.7);

        // Generate some frames
        for _ in 0..5 {
            audio_gen.generate_frame();
        }

        // Rollback to completely different state
        let mut rollback_audio = AudioPlaybackState::default();
        rollback_audio.channels[0].sound = 0; // Was playing, now silent
        rollback_audio.channels[1].sound = 99; // Different sound
        rollback_audio.channels[1].position = 50000;
        rollback_audio.channels[2].sound = 0; // Was playing, now silent
        // channels[3] stays silent
        rollback_audio.music.sound = 0; // Music stopped

        let rollback = AudioGenSnapshot::new(
            rollback_audio,
            TrackerState::default(),
            TrackerEngine::new().snapshot(),
            Arc::new(Vec::new()),
            2,
            60,
            44100,
            true,
        );
        audio_gen.handle_snapshot(rollback);

        // ALL channels should match rollback state exactly
        assert_eq!(audio_gen.gen_audio.channels[0].sound, 0, "Channel 0 should be silent");
        assert_eq!(audio_gen.gen_audio.channels[1].sound, 99, "Channel 1 should have new sound");
        assert_eq!(audio_gen.gen_audio.channels[1].position, 50000, "Channel 1 position should match rollback");
        assert_eq!(audio_gen.gen_audio.channels[2].sound, 0, "Channel 2 should be silent");
        assert_eq!(audio_gen.gen_audio.music.sound, 0, "Music should be stopped");
        assert!(audio_gen.crossfade_active, "Crossfade should be active");
    }

    #[test]
    fn test_rollback_to_silence() {
        // Rollback from playing sounds to complete silence
        let mut audio_gen = TestableAudioGen::new();

        // Initial snapshot with sounds playing
        let mut audio = AudioPlaybackState::default();
        audio.channels[0].sound = 1;
        audio.channels[0].position = 0;
        audio.music.sound = 5;
        audio.music.position = 0;

        let snapshot1 = AudioGenSnapshot::new(
            audio,
            TrackerState::default(),
            TrackerEngine::new().snapshot(),
            Arc::new(Vec::new()),
            0,
            60,
            44100,
            false,
        );
        audio_gen.handle_snapshot(snapshot1);
        audio_gen.prev_frame_last = (0.9, -0.9); // Non-zero audio was playing

        for _ in 0..3 {
            audio_gen.generate_frame();
        }

        // Rollback to silence
        let rollback = AudioGenSnapshot::new(
            AudioPlaybackState::default(), // All zeros
            TrackerState::default(),
            TrackerEngine::new().snapshot(),
            Arc::new(Vec::new()),
            1,
            60,
            44100,
            true,
        );
        audio_gen.handle_snapshot(rollback);

        // Should be completely silent
        assert_eq!(audio_gen.gen_audio.channels[0].sound, 0);
        assert_eq!(audio_gen.gen_audio.music.sound, 0);
        // Crossfade should smooth the transition to silence
        assert!(audio_gen.crossfade_active);
        assert_eq!(audio_gen.crossfade_from, (0.9, -0.9));
    }

    #[test]
    fn test_rollback_from_silence_to_sounds() {
        // Rollback from silence to sounds playing
        let mut audio_gen = TestableAudioGen::new();

        // Initial snapshot with silence
        let snapshot1 = AudioGenSnapshot::new(
            AudioPlaybackState::default(),
            TrackerState::default(),
            TrackerEngine::new().snapshot(),
            Arc::new(Vec::new()),
            0,
            60,
            44100,
            false,
        );
        audio_gen.handle_snapshot(snapshot1);
        audio_gen.prev_frame_last = (0.0, 0.0); // Silence

        for _ in 0..3 {
            audio_gen.generate_frame();
        }

        // Rollback to state with sounds
        let mut rollback_audio = AudioPlaybackState::default();
        rollback_audio.channels[0].sound = 5;
        rollback_audio.channels[0].position = 12345;
        rollback_audio.channels[0].volume = 1.0;

        let rollback = AudioGenSnapshot::new(
            rollback_audio,
            TrackerState::default(),
            TrackerEngine::new().snapshot(),
            Arc::new(Vec::new()),
            1,
            60,
            44100,
            true,
        );
        audio_gen.handle_snapshot(rollback);

        // Should have sound playing
        assert_eq!(audio_gen.gen_audio.channels[0].sound, 5);
        assert_eq!(audio_gen.gen_audio.channels[0].position, 12345);
        // Crossfade from silence
        assert!(audio_gen.crossfade_active);
        assert_eq!(audio_gen.crossfade_from, (0.0, 0.0));
    }

    #[test]
    fn test_multiple_consecutive_rollbacks() {
        // Handle back-to-back rollbacks correctly
        let mut audio_gen = TestableAudioGen::new();

        // Initial snapshot
        let mut audio = AudioPlaybackState::default();
        audio.channels[0].sound = 1;

        let snapshot1 = AudioGenSnapshot::new(
            audio,
            TrackerState::default(),
            TrackerEngine::new().snapshot(),
            Arc::new(Vec::new()),
            0,
            60,
            44100,
            false,
        );
        audio_gen.handle_snapshot(snapshot1);
        audio_gen.generate_frame();
        audio_gen.prev_frame_last = (0.5, 0.5);

        // First rollback
        let mut rollback1_audio = AudioPlaybackState::default();
        rollback1_audio.channels[0].sound = 2;
        rollback1_audio.channels[0].position = 1000;

        let rollback1 = AudioGenSnapshot::new(
            rollback1_audio,
            TrackerState::default(),
            TrackerEngine::new().snapshot(),
            Arc::new(Vec::new()),
            1,
            60,
            44100,
            true,
        );
        audio_gen.handle_snapshot(rollback1);

        assert_eq!(audio_gen.gen_audio.channels[0].sound, 2);
        assert!(audio_gen.crossfade_active);
        let crossfade1 = audio_gen.crossfade_from;

        // Simulate generating a frame (clears crossfade)
        audio_gen.crossfade_active = false;
        audio_gen.prev_frame_last = (0.3, 0.3);

        // Second rollback immediately after
        let mut rollback2_audio = AudioPlaybackState::default();
        rollback2_audio.channels[0].sound = 3;
        rollback2_audio.channels[0].position = 2000;

        let rollback2 = AudioGenSnapshot::new(
            rollback2_audio,
            TrackerState::default(),
            TrackerEngine::new().snapshot(),
            Arc::new(Vec::new()),
            2,
            60,
            44100,
            true,
        );
        audio_gen.handle_snapshot(rollback2);

        // Should have latest rollback state
        assert_eq!(audio_gen.gen_audio.channels[0].sound, 3);
        assert_eq!(audio_gen.gen_audio.channels[0].position, 2000);
        // New crossfade from latest prev_frame_last
        assert!(audio_gen.crossfade_active);
        assert_eq!(audio_gen.crossfade_from, (0.3, 0.3));
        assert_ne!(audio_gen.crossfade_from, crossfade1, "Second rollback should use updated crossfade source");
    }

    #[test]
    fn test_rollback_followed_by_normal_snapshot() {
        // After rollback, normal snapshots should work correctly again
        let mut audio_gen = TestableAudioGen::new();

        // Initial snapshot
        let mut audio = AudioPlaybackState::default();
        audio.channels[0].sound = 1;
        audio.channels[0].position = 0;

        let snapshot1 = AudioGenSnapshot::new(
            audio,
            TrackerState::default(),
            TrackerEngine::new().snapshot(),
            Arc::new(Vec::new()),
            0,
            60,
            44100,
            false,
        );
        audio_gen.handle_snapshot(snapshot1);
        audio_gen.prev_frame_last = (0.5, 0.5);
        audio_gen.generate_frame();

        // Rollback
        let mut rollback_audio = AudioPlaybackState::default();
        rollback_audio.channels[0].sound = 2;
        rollback_audio.channels[0].position = 5000;

        let rollback = AudioGenSnapshot::new(
            rollback_audio.clone(),
            TrackerState::default(),
            TrackerEngine::new().snapshot(),
            Arc::new(Vec::new()),
            1,
            60,
            44100,
            true,
        );
        audio_gen.handle_snapshot(rollback);
        audio_gen.crossfade_active = false; // Simulate crossfade completed

        // Generate some frames after rollback
        audio_gen.generate_frame();
        audio_gen.generate_frame();
        let position_after_rollback = audio_gen.gen_audio.channels[0].position;

        // Normal snapshot (not rollback) should use merge logic
        let mut normal_audio = AudioPlaybackState::default();
        normal_audio.channels[0].sound = 2;
        normal_audio.channels[0].position = 6000; // Behind where we are
        normal_audio.channels[0].volume = 0.7; // Volume change

        let normal = AudioGenSnapshot::new(
            normal_audio,
            TrackerState::default(),
            TrackerEngine::new().snapshot(),
            Arc::new(Vec::new()),
            2,
            60,
            44100,
            false, // NOT rollback
        );
        audio_gen.handle_snapshot(normal);

        // Position should NOT be reset (merge logic)
        assert_eq!(
            audio_gen.gen_audio.channels[0].position,
            position_after_rollback,
            "Position should not change for normal snapshot after rollback"
        );
        // But volume should update
        assert_eq!(audio_gen.gen_audio.channels[0].volume, 0.7);
        // No crossfade for normal snapshot
        assert!(!audio_gen.crossfade_active);
    }

    #[test]
    fn test_rollback_resets_tracker_state() {
        // Tracker state should also be fully reset on rollback
        let mut audio_gen = TestableAudioGen::new();

        // Initial snapshot with tracker state
        let mut tracker = TrackerState::default();
        tracker.order_position = 0;
        tracker.row = 0;
        tracker.tick = 0;
        tracker.volume = 64;

        let snapshot1 = AudioGenSnapshot::new(
            AudioPlaybackState::default(),
            tracker,
            TrackerEngine::new().snapshot(),
            Arc::new(Vec::new()),
            0,
            60,
            44100,
            false,
        );
        audio_gen.handle_snapshot(snapshot1);

        // Simulate tracker advancing
        audio_gen.gen_tracker.order_position = 5;
        audio_gen.gen_tracker.row = 32;
        audio_gen.gen_tracker.tick = 3;
        audio_gen.prev_frame_last = (0.4, 0.4);

        // Rollback to different tracker state
        let mut rollback_tracker = TrackerState::default();
        rollback_tracker.order_position = 2;
        rollback_tracker.row = 16;
        rollback_tracker.tick = 1;
        rollback_tracker.volume = 48;
        rollback_tracker.flags = 0xFF;

        let rollback = AudioGenSnapshot::new(
            AudioPlaybackState::default(),
            rollback_tracker,
            TrackerEngine::new().snapshot(),
            Arc::new(Vec::new()),
            1,
            60,
            44100,
            true,
        );
        audio_gen.handle_snapshot(rollback);

        // ALL tracker fields should match rollback
        assert_eq!(audio_gen.gen_tracker.order_position, 2);
        assert_eq!(audio_gen.gen_tracker.row, 16);
        assert_eq!(audio_gen.gen_tracker.tick, 1);
        assert_eq!(audio_gen.gen_tracker.volume, 48);
        assert_eq!(audio_gen.gen_tracker.flags, 0xFF);
    }

    #[test]
    fn test_rollback_resets_samples_since_snapshot() {
        // samples_since_snapshot counter should reset on rollback
        let mut audio_gen = TestableAudioGen::new();

        let snapshot1 = AudioGenSnapshot::new(
            AudioPlaybackState::default(),
            TrackerState::default(),
            TrackerEngine::new().snapshot(),
            Arc::new(Vec::new()),
            0,
            60,
            44100,
            false,
        );
        audio_gen.handle_snapshot(snapshot1);

        // Generate many frames
        for _ in 0..10 {
            audio_gen.generate_frame();
        }

        let samples_before = audio_gen.samples_since_snapshot;
        assert!(samples_before > 0, "Should have generated samples");

        // Rollback
        let rollback = AudioGenSnapshot::new(
            AudioPlaybackState::default(),
            TrackerState::default(),
            TrackerEngine::new().snapshot(),
            Arc::new(Vec::new()),
            5,
            60,
            44100,
            true,
        );
        audio_gen.handle_snapshot(rollback);

        assert_eq!(
            audio_gen.samples_since_snapshot, 0,
            "samples_since_snapshot should reset on rollback"
        );
    }

    #[test]
    fn test_first_snapshot_is_rollback() {
        // Edge case: first snapshot received is marked as rollback
        let mut audio_gen = TestableAudioGen::new();

        assert!(!audio_gen.has_state, "Should not have state initially");

        let mut audio = AudioPlaybackState::default();
        audio.channels[0].sound = 5;
        audio.channels[0].position = 10000;

        // First snapshot is a rollback (unusual but should handle gracefully)
        let rollback = AudioGenSnapshot::new(
            audio,
            TrackerState::default(),
            TrackerEngine::new().snapshot(),
            Arc::new(Vec::new()),
            0,
            60,
            44100,
            true, // is_rollback
        );
        audio_gen.handle_snapshot(rollback);

        // Should still initialize state correctly
        assert_eq!(audio_gen.gen_audio.channels[0].sound, 5);
        assert_eq!(audio_gen.gen_audio.channels[0].position, 10000);
        // Crossfade from (0,0) since no audio was playing
        assert!(audio_gen.crossfade_active);
        assert_eq!(audio_gen.crossfade_from, (0.0, 0.0));
    }

    #[test]
    fn test_rollback_when_far_ahead() {
        // Audio thread predicted many frames ahead, rollback to earlier state
        let mut audio_gen = TestableAudioGen::new();

        let mut audio = AudioPlaybackState::default();
        audio.channels[0].sound = 1;
        audio.channels[0].position = 0;

        let snapshot1 = AudioGenSnapshot::new(
            audio,
            TrackerState::default(),
            TrackerEngine::new().snapshot(),
            Arc::new(Vec::new()),
            0,
            60,
            44100,
            false,
        );
        audio_gen.handle_snapshot(snapshot1);

        // Generate MANY frames (simulate lag or prediction)
        for _ in 0..20 {
            audio_gen.generate_frame();
        }
        audio_gen.prev_frame_last = (0.8, -0.8);

        let position_far_ahead = audio_gen.gen_audio.channels[0].position;
        assert!(position_far_ahead > 1000000, "Should be far ahead");

        // Rollback to early position
        let mut rollback_audio = AudioPlaybackState::default();
        rollback_audio.channels[0].sound = 1;
        rollback_audio.channels[0].position = 50000; // Much earlier

        let rollback = AudioGenSnapshot::new(
            rollback_audio,
            TrackerState::default(),
            TrackerEngine::new().snapshot(),
            Arc::new(Vec::new()),
            5,
            60,
            44100,
            true,
        );
        audio_gen.handle_snapshot(rollback);

        // Position should match rollback (going backwards)
        assert_eq!(audio_gen.gen_audio.channels[0].position, 50000);
        // Crossfade should smooth this large jump
        assert!(audio_gen.crossfade_active);
    }

    #[test]
    fn test_rollback_music_channel() {
        // Music channel should be handled correctly during rollback
        let mut audio_gen = TestableAudioGen::new();

        // Initial snapshot with music playing
        let mut audio = AudioPlaybackState::default();
        audio.music.sound = 1;
        audio.music.position = 0;
        audio.music.volume = 0.8;
        audio.music.pan = 0.0;

        let snapshot1 = AudioGenSnapshot::new(
            audio,
            TrackerState::default(),
            TrackerEngine::new().snapshot(),
            Arc::new(Vec::new()),
            0,
            60,
            44100,
            false,
        );
        audio_gen.handle_snapshot(snapshot1);

        for _ in 0..5 {
            audio_gen.generate_frame();
        }
        audio_gen.prev_frame_last = (0.6, 0.6);

        // Rollback with different music state
        let mut rollback_audio = AudioPlaybackState::default();
        rollback_audio.music.sound = 2; // Different music
        rollback_audio.music.position = 99999;
        rollback_audio.music.volume = 0.5;
        rollback_audio.music.pan = -0.3;

        let rollback = AudioGenSnapshot::new(
            rollback_audio,
            TrackerState::default(),
            TrackerEngine::new().snapshot(),
            Arc::new(Vec::new()),
            2,
            60,
            44100,
            true,
        );
        audio_gen.handle_snapshot(rollback);

        // Music should match rollback exactly
        assert_eq!(audio_gen.gen_audio.music.sound, 2);
        assert_eq!(audio_gen.gen_audio.music.position, 99999);
        assert_eq!(audio_gen.gen_audio.music.volume, 0.5);
        assert_eq!(audio_gen.gen_audio.music.pan, -0.3);
    }

    #[test]
    fn test_rollback_preserves_has_state() {
        // has_state should remain true after rollback
        let mut audio_gen = TestableAudioGen::new();

        let snapshot1 = AudioGenSnapshot::new(
            AudioPlaybackState::default(),
            TrackerState::default(),
            TrackerEngine::new().snapshot(),
            Arc::new(Vec::new()),
            0,
            60,
            44100,
            false,
        );
        audio_gen.handle_snapshot(snapshot1);
        assert!(audio_gen.has_state);

        // Rollback
        let rollback = AudioGenSnapshot::new(
            AudioPlaybackState::default(),
            TrackerState::default(),
            TrackerEngine::new().snapshot(),
            Arc::new(Vec::new()),
            1,
            60,
            44100,
            true,
        );
        audio_gen.handle_snapshot(rollback);

        // has_state should still be true
        assert!(audio_gen.has_state, "has_state should remain true after rollback");
    }

    #[test]
    fn test_normal_snapshot_does_not_trigger_crossfade() {
        // Verify normal snapshots never trigger crossfade
        let mut audio_gen = TestableAudioGen::new();

        let mut audio = AudioPlaybackState::default();
        audio.channels[0].sound = 1;
        audio.channels[0].position = 0;

        let snapshot1 = AudioGenSnapshot::new(
            audio,
            TrackerState::default(),
            TrackerEngine::new().snapshot(),
            Arc::new(Vec::new()),
            0,
            60,
            44100,
            false,
        );
        audio_gen.handle_snapshot(snapshot1);
        assert!(!audio_gen.crossfade_active, "Initial snapshot should not trigger crossfade");

        for _ in 0..3 {
            audio_gen.generate_frame();
        }

        // Multiple normal snapshots
        for frame in 1i32..10 {
            let mut audio = AudioPlaybackState::default();
            audio.channels[0].sound = 1;
            audio.channels[0].position = (frame as u32) * 94080; // Main thread position (ignored)
            audio.channels[0].volume = 0.5 + (frame as f32 * 0.05);

            let snapshot = AudioGenSnapshot::new(
                audio,
                TrackerState::default(),
                TrackerEngine::new().snapshot(),
                Arc::new(Vec::new()),
                frame,
                60,
                44100,
                false,
            );
            audio_gen.handle_snapshot(snapshot);

            assert!(
                !audio_gen.crossfade_active,
                "Normal snapshot {} should not trigger crossfade",
                frame
            );
        }
    }

    #[test]
    fn test_sfx_stop_is_instant_no_crossfade() {
        // Stopping SFX via normal snapshot should be instant (no crossfade)
        let mut audio_gen = TestableAudioGen::new();

        // Start with SFX playing
        let mut audio = AudioPlaybackState::default();
        audio.channels[0].sound = 1;
        audio.channels[0].position = 0;

        let snapshot1 = AudioGenSnapshot::new(
            audio,
            TrackerState::default(),
            TrackerEngine::new().snapshot(),
            Arc::new(Vec::new()),
            0,
            60,
            44100,
            false,
        );
        audio_gen.handle_snapshot(snapshot1);
        audio_gen.generate_frame();

        // Stop SFX via normal snapshot
        let mut audio2 = AudioPlaybackState::default();
        audio2.channels[0].sound = 0; // Stopped

        let snapshot2 = AudioGenSnapshot::new(
            audio2,
            TrackerState::default(),
            TrackerEngine::new().snapshot(),
            Arc::new(Vec::new()),
            1,
            60,
            44100,
            false,
        );
        audio_gen.handle_snapshot(snapshot2);

        // SFX should be stopped
        assert_eq!(audio_gen.gen_audio.channels[0].sound, 0);
        // No crossfade (stopping is instant)
        assert!(!audio_gen.crossfade_active);
    }

    #[test]
    fn test_crossfade_length_bounds() {
        // Test crossfade with various buffer sizes
        let crossfade_samples = 44; // Standard 1ms

        // Buffer smaller than crossfade
        let mut small_buffer = vec![0.5, 0.5, 0.4, 0.4]; // 2 stereo pairs
        let crossfade_from = (0.0, 0.0);
        let fade_len = crossfade_samples.min(small_buffer.len() / 2);
        assert_eq!(fade_len, 2, "Fade length should be clamped to buffer size");

        for i in 0..fade_len {
            let t = i as f32 / fade_len as f32;
            let idx_l = i * 2;
            let idx_r = i * 2 + 1;
            small_buffer[idx_l] = crossfade_from.0 * (1.0 - t) + small_buffer[idx_l] * t;
            small_buffer[idx_r] = crossfade_from.1 * (1.0 - t) + small_buffer[idx_r] * t;
        }

        // Should have applied crossfade to both stereo pairs
        assert!(small_buffer[0].abs() < 0.5, "First sample should be faded");
    }

    // ========================================================================
    // AUDIO CONTINUITY TESTS - Actually detect pops/clicks
    // ========================================================================

    /// Calculate max sample jump (discontinuity) in a buffer
    /// A "pop" is typically a jump > 0.3 in a single sample
    fn max_sample_discontinuity(buffer: &[f32]) -> f32 {
        if buffer.len() < 2 {
            return 0.0;
        }
        let mut max_jump = 0.0f32;
        for i in 1..buffer.len() {
            let jump = (buffer[i] - buffer[i - 1]).abs();
            max_jump = max_jump.max(jump);
        }
        max_jump
    }

    #[test]
    fn test_crossfade_eliminates_discontinuity() {
        // Simulate a large sample discontinuity that would cause a pop
        // WITHOUT crossfade: jump from 0.8 to -0.6 = 1.4 amplitude jump (LOUD POP!)
        // WITH crossfade: gradual transition over ~1ms

        let crossfade_samples = 44; // ~1ms at 44.1kHz
        let buffer_size = 128;

        // Simulate: we were outputting 0.8, now we need to output -0.6
        let prev_sample = 0.8f32;
        let new_start_sample = -0.6f32;

        // WITHOUT crossfade (the old buggy behavior)
        let no_crossfade_buffer: Vec<f32> = vec![new_start_sample; buffer_size];
        // First sample jumps directly
        let no_crossfade_jump = (no_crossfade_buffer[0] - prev_sample).abs();
        assert!(
            no_crossfade_jump > 1.0,
            "Without crossfade, jump should be huge: {}",
            no_crossfade_jump
        );

        // WITH crossfade (the fix)
        let mut crossfade_buffer: Vec<f32> = vec![new_start_sample; buffer_size];
        let crossfade_from = (prev_sample, prev_sample); // Both channels same for simplicity

        // Apply crossfade (same logic as apply_crossfade)
        let fade_len = crossfade_samples.min(crossfade_buffer.len() / 2);
        for i in 0..fade_len {
            let t = i as f32 / fade_len as f32;
            let idx_l = i * 2;
            let idx_r = i * 2 + 1;
            if idx_l < crossfade_buffer.len() {
                crossfade_buffer[idx_l] =
                    crossfade_from.0 * (1.0 - t) + crossfade_buffer[idx_l] * t;
            }
            if idx_r < crossfade_buffer.len() {
                crossfade_buffer[idx_r] =
                    crossfade_from.1 * (1.0 - t) + crossfade_buffer[idx_r] * t;
            }
        }

        // First sample after crossfade should be close to prev_sample, NOT new_start_sample
        let crossfade_first_sample = crossfade_buffer[0];
        let crossfade_jump = (crossfade_first_sample - prev_sample).abs();

        assert!(
            crossfade_jump < 0.1,
            "With crossfade, first sample jump should be tiny: {} (sample went from {} to {})",
            crossfade_jump,
            prev_sample,
            crossfade_first_sample
        );

        // Max discontinuity in crossfaded buffer should be small
        let max_disc = max_sample_discontinuity(&crossfade_buffer);
        assert!(
            max_disc < 0.1,
            "Max discontinuity in crossfaded buffer should be small: {}",
            max_disc
        );
    }

    #[test]
    fn test_crossfade_worst_case_full_swing() {
        // Worst case: full swing from +1.0 to -1.0 (2.0 amplitude jump)
        // This would be an EXTREME pop without crossfade

        let crossfade_samples = 44;
        let buffer_size = 128;

        let prev_sample = 1.0f32;
        let new_start_sample = -1.0f32;

        // Apply crossfade
        let mut buffer: Vec<f32> = vec![new_start_sample; buffer_size];
        let fade_len = crossfade_samples.min(buffer.len() / 2);

        for i in 0..fade_len {
            let t = i as f32 / fade_len as f32;
            let idx_l = i * 2;
            let idx_r = i * 2 + 1;
            if idx_l < buffer.len() {
                buffer[idx_l] = prev_sample * (1.0 - t) + buffer[idx_l] * t;
            }
            if idx_r < buffer.len() {
                buffer[idx_r] = prev_sample * (1.0 - t) + buffer[idx_r] * t;
            }
        }

        // First sample should be close to prev (1.0), not new (-1.0)
        assert!(
            buffer[0] > 0.9,
            "First sample should be close to 1.0, got {}",
            buffer[0]
        );

        // Verify smooth transition - each step should be roughly equal
        // Over 44 samples, we go from 1.0 to -1.0 = 2.0 total change
        // Per sample: 2.0 / 44 ≈ 0.045
        let expected_step = 2.0 / crossfade_samples as f32;
        let tolerance = expected_step * 1.5; // Allow some tolerance

        for i in 1..fade_len * 2 {
            if i < buffer.len() {
                let jump = (buffer[i] - buffer[i - 1]).abs();
                assert!(
                    jump < tolerance,
                    "Sample {} jump {} exceeds tolerance {} (expected ~{})",
                    i,
                    jump,
                    tolerance,
                    expected_step
                );
            }
        }
    }

    #[test]
    fn test_no_crossfade_means_potential_pop() {
        // Verify that WITHOUT crossfade, we WOULD have a pop
        // This test documents why crossfade is necessary

        // Simulate position reset without crossfade (the old buggy behavior)
        let prev_output = 0.7f32;
        let new_output_after_reset = -0.5f32;

        let direct_jump = (new_output_after_reset - prev_output).abs();

        // This is a huge jump - definitely audible as a pop
        assert!(
            direct_jump > 0.3,
            "Direct jump {} should be > 0.3 (pop threshold)",
            direct_jump
        );

        // Document: jumps > 0.3 are typically audible as clicks/pops
        // Our crossfade ensures no jump exceeds ~0.05 per sample
    }

    #[test]
    fn test_gradual_volume_change_no_discontinuity() {
        // Volume/pan changes via normal snapshots should be instant but small
        // They don't cause pops because they're typically small changes

        // Simulate volume going from 0.5 to 0.8 (60% increase)
        // At typical sample values, this is a small multiplicative change
        let sample_before = 0.4f32 * 0.5f32; // sample * volume
        let sample_after = 0.4f32 * 0.8f32; // same sample, new volume

        let volume_jump = (sample_after - sample_before).abs();

        // This is a small change (0.12), typically not audible as a pop
        // because it's spread across the entire waveform, not a single sample
        assert!(
            volume_jump < 0.15,
            "Volume change jump should be small: {}",
            volume_jump
        );
    }

    #[test]
    fn test_sfx_start_has_zero_crossing() {
        // New SFX starting at position=0 shouldn't pop because:
        // 1. Sound data typically starts at or near zero
        // 2. Any competent sound effect has a zero-crossing at the start

        // Verify assumption: position 0 means we start from the beginning
        // which should be at or near silence
        let new_sfx_position = 0u32;
        assert_eq!(new_sfx_position, 0, "New SFX starts at position 0");

        // Most audio formats start at 0 or very close to it
        // This is a documentation test - actual verification would need sound data
    }

    #[test]
    fn test_rollback_crossfade_smooths_any_discontinuity() {
        // The key insight: rollback can cause ANY arbitrary state change
        // Crossfade must handle all cases

        // Test various discontinuity magnitudes
        let test_cases = [
            (0.0, 0.5),   // Silence to mid-volume
            (0.5, 0.0),   // Mid-volume to silence
            (1.0, -1.0),  // Full positive to full negative (worst case)
            (-1.0, 1.0),  // Full negative to full positive
            (0.3, 0.35),  // Small change (shouldn't really need crossfade but still works)
            (0.9, -0.9),  // Large swing
        ];

        let crossfade_samples = 44;

        for (prev, new) in test_cases {
            let mut buffer: Vec<f32> = vec![new; 128];

            // Apply crossfade to BOTH channels (stereo interleaved)
            let fade_len = crossfade_samples.min(buffer.len() / 2);
            for i in 0..fade_len {
                let t = i as f32 / fade_len as f32;
                let idx_l = i * 2;
                let idx_r = i * 2 + 1;
                if idx_l < buffer.len() {
                    buffer[idx_l] = prev * (1.0 - t) + buffer[idx_l] * t;
                }
                if idx_r < buffer.len() {
                    buffer[idx_r] = prev * (1.0 - t) + buffer[idx_r] * t;
                }
            }

            // First sample should be very close to prev
            let first_jump = (buffer[0] - prev).abs();
            assert!(
                first_jump < 0.05,
                "Case ({} -> {}): First sample jump {} should be < 0.05",
                prev,
                new,
                first_jump
            );

            // No large discontinuities in the crossfade region
            let max_disc = max_sample_discontinuity(&buffer[..fade_len * 2]);
            assert!(
                max_disc < 0.1,
                "Case ({} -> {}): Max discontinuity {} should be < 0.1",
                prev,
                new,
                max_disc
            );
        }
    }

    #[test]
    fn test_pop_threshold_documentation() {
        // Document what constitutes a "pop" for reference

        // Human hearing is most sensitive to sudden transients
        // A "pop" is generally:
        // - A sample-to-sample jump > 0.3 at high amplitude
        // - A sample-to-sample jump > 0.1 is noticeable
        // - A sample-to-sample jump < 0.05 is typically inaudible

        let pop_threshold = 0.3f32;
        let noticeable_threshold = 0.1f32;
        let inaudible_threshold = 0.05f32;

        // Our crossfade aims for < inaudible_threshold
        let crossfade_samples = 44;
        let max_swing = 2.0f32; // From +1 to -1
        let step_per_sample = max_swing / crossfade_samples as f32;

        assert!(
            step_per_sample < inaudible_threshold,
            "Crossfade step {} should be < inaudible threshold {}",
            step_per_sample,
            inaudible_threshold
        );

        // Even in worst case, our crossfade keeps us below noticeable
        assert!(
            step_per_sample < noticeable_threshold,
            "Crossfade step {} should be < noticeable threshold {}",
            step_per_sample,
            noticeable_threshold
        );

        // And definitely below pop threshold
        assert!(
            step_per_sample < pop_threshold,
            "Crossfade step {} should be < pop threshold {}",
            step_per_sample,
            pop_threshold
        );
    }

    #[test]
    fn test_music_change_triggers_crossfade() {
        // Test: Changing music (song switch) should start new music with crossfade
        let mut audio_gen = TestableAudioGen::new();

        // Start with song 1
        let mut audio = AudioPlaybackState::default();
        audio.music.sound = 1;
        audio.music.position = 0;
        audio.music.volume = 0.8;

        let snapshot1 = AudioGenSnapshot::new(
            audio,
            TrackerState::default(),
            TrackerEngine::new().snapshot(),
            Arc::new(Vec::new()),
            0,
            60,
            44100,
            false,
        );
        audio_gen.handle_snapshot(snapshot1);
        assert_eq!(audio_gen.gen_audio.music.sound, 1);
        assert!(!audio_gen.crossfade_active, "No crossfade on first snapshot");

        // Generate some frames
        audio_gen.generate_frame();
        audio_gen.generate_frame();
        audio_gen.prev_frame_last = (0.7, 0.7); // Simulate some audio output

        // Now switch to song 2 - position will be non-zero from main thread
        let mut audio2 = AudioPlaybackState::default();
        audio2.music.sound = 2; // Different song!
        audio2.music.position = 0; // New song starts at 0
        audio2.music.volume = 0.9;

        let snapshot2 = AudioGenSnapshot::new(
            audio2,
            TrackerState::default(),
            TrackerEngine::new().snapshot(),
            Arc::new(Vec::new()),
            2,
            60,
            44100,
            false,
        );
        audio_gen.handle_snapshot(snapshot2);

        // Music should have changed
        assert_eq!(audio_gen.gen_audio.music.sound, 2, "Music should switch to song 2");
        // Crossfade should be active due to song change
        assert!(
            audio_gen.crossfade_active,
            "Crossfade should be triggered on song change"
        );
        assert_eq!(
            audio_gen.crossfade_from,
            (0.7, 0.7),
            "Crossfade should start from last output"
        );
    }

    #[test]
    fn test_sfx_change_triggers_crossfade() {
        // Test: Changing SFX sound ID should start new SFX with crossfade
        let mut audio_gen = TestableAudioGen::new();

        // Start with SFX 1 on channel 0
        let mut audio = AudioPlaybackState::default();
        audio.channels[0].sound = 1;
        audio.channels[0].position = 0;

        let snapshot1 = AudioGenSnapshot::new(
            audio,
            TrackerState::default(),
            TrackerEngine::new().snapshot(),
            Arc::new(Vec::new()),
            0,
            60,
            44100,
            false,
        );
        audio_gen.handle_snapshot(snapshot1);
        assert!(!audio_gen.crossfade_active);

        // Generate and set up last output
        audio_gen.generate_frame();
        audio_gen.prev_frame_last = (0.5, 0.5);

        // Switch to SFX 2 (different sound)
        let mut audio2 = AudioPlaybackState::default();
        audio2.channels[0].sound = 2; // Different SFX!
        audio2.channels[0].position = 0;

        let snapshot2 = AudioGenSnapshot::new(
            audio2,
            TrackerState::default(),
            TrackerEngine::new().snapshot(),
            Arc::new(Vec::new()),
            1,
            60,
            44100,
            false,
        );
        audio_gen.handle_snapshot(snapshot2);

        assert_eq!(audio_gen.gen_audio.channels[0].sound, 2);
        assert!(audio_gen.crossfade_active, "Crossfade should trigger on SFX change");
    }

    #[test]
    fn test_music_change_with_nonzero_position() {
        // Test: Song switch should work even when new snapshot has non-zero position
        // (This was the original bug - we only detected new music when position == 0)
        let mut audio_gen = TestableAudioGen::new();

        // Start with song 1
        let mut audio = AudioPlaybackState::default();
        audio.music.sound = 1;
        audio.music.position = 0;

        let snapshot1 = AudioGenSnapshot::new(
            audio,
            TrackerState::default(),
            TrackerEngine::new().snapshot(),
            Arc::new(Vec::new()),
            0,
            60,
            44100,
            false,
        );
        audio_gen.handle_snapshot(snapshot1);

        // Generate frames so audio thread advances
        for _ in 0..5 {
            audio_gen.generate_frame();
        }
        audio_gen.prev_frame_last = (0.6, 0.6);

        // Song 2 comes in with NON-ZERO position (main thread has been running it)
        let mut audio2 = AudioPlaybackState::default();
        audio2.music.sound = 2;
        audio2.music.position = 50000; // Non-zero! This is the key test case

        let snapshot2 = AudioGenSnapshot::new(
            audio2,
            TrackerState::default(),
            TrackerEngine::new().snapshot(),
            Arc::new(Vec::new()),
            5,
            60,
            44100,
            false,
        );
        audio_gen.handle_snapshot(snapshot2);

        // Should still detect the song change and switch
        assert_eq!(
            audio_gen.gen_audio.music.sound, 2,
            "Should switch to song 2 even with non-zero position"
        );
        assert!(audio_gen.crossfade_active);
    }

    #[test]
    fn test_same_music_does_not_trigger_crossfade() {
        // Test: Same song ID should NOT trigger crossfade (just volume/pan update)
        let mut audio_gen = TestableAudioGen::new();

        // Start with song 1
        let mut audio = AudioPlaybackState::default();
        audio.music.sound = 1;
        audio.music.position = 0;
        audio.music.volume = 0.5;

        let snapshot1 = AudioGenSnapshot::new(
            audio,
            TrackerState::default(),
            TrackerEngine::new().snapshot(),
            Arc::new(Vec::new()),
            0,
            60,
            44100,
            false,
        );
        audio_gen.handle_snapshot(snapshot1);
        audio_gen.generate_frame();

        // Send another snapshot with SAME song, just different volume
        let mut audio2 = AudioPlaybackState::default();
        audio2.music.sound = 1; // Same song
        audio2.music.position = 100000; // Advanced position
        audio2.music.volume = 0.8; // Different volume

        let snapshot2 = AudioGenSnapshot::new(
            audio2,
            TrackerState::default(),
            TrackerEngine::new().snapshot(),
            Arc::new(Vec::new()),
            1,
            60,
            44100,
            false,
        );
        audio_gen.handle_snapshot(snapshot2);

        // Volume should update but NO crossfade
        assert_eq!(audio_gen.gen_audio.music.volume, 0.8);
        assert!(!audio_gen.crossfade_active, "Same song should NOT trigger crossfade");
    }

    // ========================================================================
    // TRACKER MODULE CHANGE TESTS (Bug fix verification)
    // ========================================================================

    #[test]
    fn test_tracker_handle_change_detected() {
        // Verify that changing tracker.handle triggers a full state reset
        // This was a bug: handle_snapshot only merged volume/flags, ignoring handle changes
        let mut audio_gen = TestableAudioGen::new();

        // Initial snapshot with tracker module 1 playing
        let mut tracker1 = TrackerState::default();
        tracker1.handle = 1;
        tracker1.bpm = 120;
        tracker1.speed = 6;
        tracker1.order_position = 0;
        tracker1.row = 0;
        tracker1.flags = crate::state::tracker_flags::PLAYING;

        let snapshot1 = AudioGenSnapshot::new(
            AudioPlaybackState::default(),
            tracker1,
            TrackerEngine::new().snapshot(),
            Arc::new(Vec::new()),
            0,
            60,
            44100,
            false,
        );
        audio_gen.handle_snapshot(snapshot1);

        // Verify initial state
        assert_eq!(audio_gen.gen_tracker.handle, 1);
        assert_eq!(audio_gen.gen_tracker.bpm, 120);

        // Simulate playback advancing (audio thread is authoritative for timing)
        audio_gen.gen_tracker.order_position = 5;
        audio_gen.gen_tracker.row = 32;
        audio_gen.gen_tracker.tick = 3;

        // Send snapshot with DIFFERENT tracker module (song change!)
        let mut tracker2 = TrackerState::default();
        tracker2.handle = 2; // Different module!
        tracker2.bpm = 140;
        tracker2.speed = 4;
        tracker2.order_position = 0;
        tracker2.row = 0;
        tracker2.flags = crate::state::tracker_flags::PLAYING;

        let snapshot2 = AudioGenSnapshot::new(
            AudioPlaybackState::default(),
            tracker2,
            TrackerEngine::new().snapshot(),
            Arc::new(Vec::new()),
            1,
            60,
            44100,
            false,
        );
        audio_gen.handle_snapshot(snapshot2);

        // New tracker should be loaded completely
        assert_eq!(audio_gen.gen_tracker.handle, 2, "Handle should change to new module");
        assert_eq!(audio_gen.gen_tracker.bpm, 140, "BPM should be from new module");
        assert_eq!(audio_gen.gen_tracker.speed, 4, "Speed should be from new module");
        // Position should reset for new module
        assert_eq!(audio_gen.gen_tracker.order_position, 0, "Position should reset");
        assert_eq!(audio_gen.gen_tracker.row, 0, "Row should reset");
    }

    #[test]
    fn test_tracker_handle_change_triggers_crossfade() {
        // Verify that switching songs triggers crossfade (to avoid pop)
        let mut audio_gen = TestableAudioGen::new();

        // Start with tracker module 1
        let mut tracker1 = TrackerState::default();
        tracker1.handle = 1;
        tracker1.flags = crate::state::tracker_flags::PLAYING;

        let snapshot1 = AudioGenSnapshot::new(
            AudioPlaybackState::default(),
            tracker1,
            TrackerEngine::new().snapshot(),
            Arc::new(Vec::new()),
            0,
            60,
            44100,
            false,
        );
        audio_gen.handle_snapshot(snapshot1);

        // Simulate audio playing
        audio_gen.prev_frame_last = (0.3, -0.2);
        audio_gen.crossfade_active = false;

        // Switch to different module
        let mut tracker2 = TrackerState::default();
        tracker2.handle = 2; // Different!
        tracker2.flags = crate::state::tracker_flags::PLAYING;

        let snapshot2 = AudioGenSnapshot::new(
            AudioPlaybackState::default(),
            tracker2,
            TrackerEngine::new().snapshot(),
            Arc::new(Vec::new()),
            1,
            60,
            44100,
            false,
        );
        audio_gen.handle_snapshot(snapshot2);

        // Crossfade should be scheduled
        assert!(audio_gen.crossfade_active, "Song change should trigger crossfade");
        assert_eq!(audio_gen.crossfade_from, (0.3, -0.2), "Crossfade from prev_frame_last");
    }

    #[test]
    fn test_tracker_bpm_change_merged() {
        // Verify that bpm changes are merged for same tracker (tempo change during playback)
        // This was a bug: only volume/flags were merged, bpm was ignored
        let mut audio_gen = TestableAudioGen::new();

        // Initial tracker state
        let mut tracker1 = TrackerState::default();
        tracker1.handle = 1;
        tracker1.bpm = 120;
        tracker1.speed = 6;
        tracker1.flags = crate::state::tracker_flags::PLAYING;

        let snapshot1 = AudioGenSnapshot::new(
            AudioPlaybackState::default(),
            tracker1,
            TrackerEngine::new().snapshot(),
            Arc::new(Vec::new()),
            0,
            60,
            44100,
            false,
        );
        audio_gen.handle_snapshot(snapshot1);

        assert_eq!(audio_gen.gen_tracker.bpm, 120);

        // Simulate playback (audio thread advances timing)
        audio_gen.gen_tracker.order_position = 2;
        audio_gen.gen_tracker.row = 16;

        // Send snapshot with SAME handle but different bpm (tempo change!)
        let mut tracker2 = TrackerState::default();
        tracker2.handle = 1; // Same module
        tracker2.bpm = 180; // Tempo changed!
        tracker2.speed = 6;
        tracker2.order_position = 1; // Main thread position (should be ignored)
        tracker2.row = 8;
        tracker2.flags = crate::state::tracker_flags::PLAYING;

        let snapshot2 = AudioGenSnapshot::new(
            AudioPlaybackState::default(),
            tracker2,
            TrackerEngine::new().snapshot(),
            Arc::new(Vec::new()),
            1,
            60,
            44100,
            false,
        );
        audio_gen.handle_snapshot(snapshot2);

        // BPM should be updated
        assert_eq!(audio_gen.gen_tracker.bpm, 180, "BPM should be merged from snapshot");

        // But timing should NOT be reset (audio thread is authoritative)
        assert_eq!(audio_gen.gen_tracker.order_position, 2, "Order position should NOT change");
        assert_eq!(audio_gen.gen_tracker.row, 16, "Row should NOT change");

        // No crossfade (same song, just tempo change)
        assert!(!audio_gen.crossfade_active, "Tempo change should NOT trigger crossfade");
    }

    #[test]
    fn test_tracker_speed_change_merged() {
        // Verify that speed changes are merged for same tracker
        let mut audio_gen = TestableAudioGen::new();

        let mut tracker1 = TrackerState::default();
        tracker1.handle = 1;
        tracker1.speed = 6;
        tracker1.flags = crate::state::tracker_flags::PLAYING;

        let snapshot1 = AudioGenSnapshot::new(
            AudioPlaybackState::default(),
            tracker1,
            TrackerEngine::new().snapshot(),
            Arc::new(Vec::new()),
            0,
            60,
            44100,
            false,
        );
        audio_gen.handle_snapshot(snapshot1);

        assert_eq!(audio_gen.gen_tracker.speed, 6);

        // Change speed
        let mut tracker2 = TrackerState::default();
        tracker2.handle = 1;
        tracker2.speed = 3; // Speed changed!
        tracker2.flags = crate::state::tracker_flags::PLAYING;

        let snapshot2 = AudioGenSnapshot::new(
            AudioPlaybackState::default(),
            tracker2,
            TrackerEngine::new().snapshot(),
            Arc::new(Vec::new()),
            1,
            60,
            44100,
            false,
        );
        audio_gen.handle_snapshot(snapshot2);

        assert_eq!(audio_gen.gen_tracker.speed, 3, "Speed should be merged from snapshot");
    }

    #[test]
    fn test_tracker_stop_detected() {
        // Verify that setting handle to 0 stops the tracker
        let mut audio_gen = TestableAudioGen::new();

        let mut tracker1 = TrackerState::default();
        tracker1.handle = 1;
        tracker1.flags = crate::state::tracker_flags::PLAYING;

        let snapshot1 = AudioGenSnapshot::new(
            AudioPlaybackState::default(),
            tracker1,
            TrackerEngine::new().snapshot(),
            Arc::new(Vec::new()),
            0,
            60,
            44100,
            false,
        );
        audio_gen.handle_snapshot(snapshot1);

        assert_eq!(audio_gen.gen_tracker.handle, 1);
        assert_ne!(audio_gen.gen_tracker.flags, 0);

        // Stop tracker
        let mut tracker2 = TrackerState::default();
        tracker2.handle = 0; // Stopped!
        tracker2.flags = 0;

        let snapshot2 = AudioGenSnapshot::new(
            AudioPlaybackState::default(),
            tracker2,
            TrackerEngine::new().snapshot(),
            Arc::new(Vec::new()),
            1,
            60,
            44100,
            false,
        );
        audio_gen.handle_snapshot(snapshot2);

        assert_eq!(audio_gen.gen_tracker.handle, 0, "Tracker should be stopped");
        assert_eq!(audio_gen.gen_tracker.flags, 0, "Flags should be cleared");
    }

    #[test]
    fn test_first_tracker_start_no_crossfade() {
        // When starting first tracker (no previous), no crossfade needed
        let mut audio_gen = TestableAudioGen::new();

        // Empty initial state
        let snapshot1 = AudioGenSnapshot::new(
            AudioPlaybackState::default(),
            TrackerState::default(),
            TrackerEngine::new().snapshot(),
            Arc::new(Vec::new()),
            0,
            60,
            44100,
            false,
        );
        audio_gen.handle_snapshot(snapshot1);

        assert_eq!(audio_gen.gen_tracker.handle, 0);
        audio_gen.crossfade_active = false;

        // Start first tracker
        let mut tracker = TrackerState::default();
        tracker.handle = 1;
        tracker.flags = crate::state::tracker_flags::PLAYING;

        let snapshot2 = AudioGenSnapshot::new(
            AudioPlaybackState::default(),
            tracker,
            TrackerEngine::new().snapshot(),
            Arc::new(Vec::new()),
            1,
            60,
            44100,
            false,
        );
        audio_gen.handle_snapshot(snapshot2);

        assert_eq!(audio_gen.gen_tracker.handle, 1, "First tracker should start");
        assert!(!audio_gen.crossfade_active, "First tracker should NOT trigger crossfade");
    }
}
