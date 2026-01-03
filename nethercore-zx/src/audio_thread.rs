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
use std::sync::Arc;
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
/// Target buffer fill level - keep ~60% full (~90ms)
const TARGET_BUFFER_SAMPLES: usize = 7938;
/// Generate more when buffer drops below this (~40%)
const LOW_BUFFER_THRESHOLD: usize = 5292;

/// Audio generation thread state
struct AudioGenThread {
    /// Receive snapshots from main thread
    rx: mpsc::Receiver<AudioGenSnapshot>,

    /// Ring buffer producer for output samples
    producer: HeapProd<f32>,

    /// Pre-allocated output buffer (reused each frame)
    output_buffer: Vec<f32>,

    /// Local tracker engine for sample generation
    tracker_engine: TrackerEngine,

    /// Output sample rate
    sample_rate: u32,

    // === Persistent playback state (continues between snapshots) ===
    /// Current audio playback state (SFX channels)
    current_audio: AudioPlaybackState,

    /// Current tracker state (position, tempo, etc.)
    current_tracker: TrackerState,

    /// Current sound data reference
    current_sounds: Option<Arc<Vec<Option<Sound>>>>,

    /// Current tick rate (fps)
    current_tick_rate: u32,

    /// Whether we have valid state to generate audio from
    has_state: bool,
}

impl AudioGenThread {
    /// Spawn the audio generation thread
    ///
    /// Returns a handle for sending snapshots to the thread.
    pub fn spawn(producer: HeapProd<f32>, sample_rate: u32) -> AudioGenHandle {
        // Bounded channel with capacity for ~8 frames
        // This provides enough buffer for jitter while limiting memory
        let (tx, rx) = mpsc::sync_channel::<AudioGenSnapshot>(8);

        let handle = thread::Builder::new()
            .name("audio-gen".into())
            .spawn(move || {
                let mut audio_gen = Self {
                    rx,
                    producer,
                    output_buffer: Vec::with_capacity(2048),
                    tracker_engine: TrackerEngine::new(),
                    sample_rate,
                    // Persistent state - starts empty, filled by first snapshot
                    current_audio: AudioPlaybackState::default(),
                    current_tracker: TrackerState::default(),
                    current_sounds: None,
                    current_tick_rate: 60,
                    has_state: false,
                };
                audio_gen.run();
            })
            .expect("failed to spawn audio generation thread");

        AudioGenHandle {
            tx: Some(tx),
            handle: Some(handle),
        }
    }

    /// Main thread loop - BUFFER-DRIVEN, not snapshot-driven
    ///
    /// Key insight: Generate audio based on buffer fill level, not snapshot arrival.
    /// This prevents timing mismatches between main thread jitter and cpal consumption.
    fn run(&mut self) {
        debug!("Audio generation thread started (buffer-driven mode)");

        loop {
            // 1. Check for new snapshots (non-blocking)
            //    New snapshots UPDATE our state but don't trigger immediate generation
            match self.rx.try_recv() {
                Ok(snapshot) => {
                    self.apply_snapshot(snapshot);
                }
                Err(mpsc::TryRecvError::Empty) => {
                    // No new snapshot - that's fine, we continue with current state
                }
                Err(mpsc::TryRecvError::Disconnected) => {
                    debug!("Audio generation thread exiting (channel disconnected)");
                    break;
                }
            }

            // 2. Check buffer fill level and generate if needed
            let buffer_filled = RING_BUFFER_CAPACITY - self.producer.vacant_len();

            if buffer_filled < LOW_BUFFER_THRESHOLD {
                // Buffer is getting low - generate more samples
                if self.has_state {
                    // Generate a frame's worth of audio using persistent state
                    self.generate_frame();
                } else {
                    // No state yet - generate silence
                    self.generate_silence();
                }
            } else if buffer_filled >= TARGET_BUFFER_SAMPLES {
                // Buffer is healthy - sleep a bit to avoid busy-waiting
                thread::sleep(Duration::from_micros(500));
            }
            // If between thresholds, continue loop without sleeping (responsive to snapshots)
        }

        debug!("Audio generation thread finished");
    }

    /// Apply a snapshot to update persistent state (does NOT generate samples)
    ///
    /// This is called when a new snapshot arrives from the main thread.
    /// Key insight: If we already have state and are "in sync", we just continue
    /// generating without resetting positions. Only rollbacks force a full reset.
    fn apply_snapshot(&mut self, snapshot: AudioGenSnapshot) {
        // Handle rollback - drain pending and FORCE reset all state
        if snapshot.is_rollback {
            debug!("Audio thread: rollback detected, forcing state reset");
            while self.rx.try_recv().is_ok() {
                // Drain pending snapshots
            }
            // Full reset on rollback
            self.current_audio = snapshot.audio;
            self.current_tracker = snapshot.tracker;
            self.current_sounds = Some(snapshot.sounds);
            self.current_tick_rate = snapshot.tick_rate;
            self.sample_rate = snapshot.sample_rate;
            self.tracker_engine.apply_snapshot(&snapshot.tracker_snapshot);
            self.has_state = true;
            debug!("Rollback: reset to frame {}", snapshot.frame_number);
            return;
        }

        // First snapshot - initialize everything
        if !self.has_state {
            self.current_audio = snapshot.audio;
            self.current_tracker = snapshot.tracker;
            self.current_sounds = Some(snapshot.sounds);
            self.current_tick_rate = snapshot.tick_rate;
            self.sample_rate = snapshot.sample_rate;
            self.tracker_engine.apply_snapshot(&snapshot.tracker_snapshot);
            self.has_state = true;
            trace!("Initialized from snapshot frame {}", snapshot.frame_number);
            return;
        }

        // Already have state - only update what's necessary:
        // 1. Sound data (in case new sounds were loaded)
        self.current_sounds = Some(snapshot.sounds);
        self.current_tick_rate = snapshot.tick_rate;
        self.sample_rate = snapshot.sample_rate;

        // 2. Tracker engine state (modules, channel configs) - always apply
        //    This ensures we have the latest module data and channel configurations
        self.tracker_engine.apply_snapshot(&snapshot.tracker_snapshot);

        // 3. DON'T reset audio/tracker positions - we're already ahead!
        //    The main thread is just "confirming" where we were.
        //    Our continuous generation keeps advancing from where we are.

        // 4. However, check for new SFX that started (sound != 0 with position 0)
        //    These need to be copied over as they're new events from main thread
        for (i, channel) in snapshot.audio.channels.iter().enumerate() {
            if channel.sound != 0 && channel.position == 0 {
                // New sound started on main thread - apply it
                self.current_audio.channels[i] = *channel;
                trace!("New SFX on channel {}: sound {}", i, channel.sound);
            }
        }

        trace!("Updated snapshot frame {} (continuing)", snapshot.frame_number);
    }

    /// Generate one frame of audio using persistent state
    ///
    /// This advances the audio/tracker state and pushes samples to the ring buffer.
    /// Called continuously based on buffer fill level, NOT on snapshot arrival.
    fn generate_frame(&mut self) {
        let Some(ref sounds) = self.current_sounds else {
            // No sounds loaded yet
            self.generate_silence();
            return;
        };

        // Generate samples using persistent state
        self.output_buffer.clear();
        generate_audio_frame_with_tracker(
            &mut self.current_audio,
            &mut self.current_tracker,
            &mut self.tracker_engine,
            sounds,
            self.current_tick_rate,
            self.sample_rate,
            &mut self.output_buffer,
        );

        // Push to ring buffer
        let pushed = self.producer.push_slice(&self.output_buffer);
        if pushed < self.output_buffer.len() {
            trace!(
                "Audio buffer full: dropped {} samples",
                self.output_buffer.len() - pushed
            );
        }
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

        // Spawn the generation thread
        let gen_handle = AudioGenThread::spawn(producer, sample_rate);

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
        // The bug was: Drop tried to join() the thread before dropping the sender,
        // but the thread was waiting for Disconnected which only happens when sender drops.
        use ringbuf::HeapRb;
        use ringbuf::traits::Split;

        let ring = HeapRb::<f32>::new(4096);
        let (producer, _consumer) = ring.split();

        let handle = AudioGenThread::spawn(producer, 44100);

        // Drop should complete within a reasonable time (not hang)
        // We use a thread with timeout to detect hangs
        let (tx, rx) = std::sync::mpsc::channel();
        let drop_thread = std::thread::spawn(move || {
            drop(handle);
            let _ = tx.send(());
        });

        // Wait up to 1 second for drop to complete
        match rx.recv_timeout(Duration::from_secs(1)) {
            Ok(()) => {
                // Success - drop completed
                drop_thread.join().unwrap();
            }
            Err(_) => {
                panic!("AudioGenHandle::drop() deadlocked - sender must be dropped before join()");
            }
        }
    }

    #[test]
    fn test_audio_thread_processes_snapshots() {
        // Test that the audio thread actually processes snapshots and produces samples
        use ringbuf::HeapRb;
        use ringbuf::traits::{Consumer, Split};

        let ring = HeapRb::<f32>::new(8192);
        let (producer, mut consumer) = ring.split();

        let handle = AudioGenThread::spawn(producer, 44100);

        // Create a snapshot with empty audio (should generate silence)
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

        // Send snapshot
        assert!(handle.send_snapshot(snapshot));

        // Give the thread time to process
        std::thread::sleep(Duration::from_millis(50));

        // Should have generated samples (735 stereo samples = 1470 floats per frame at 60fps/44.1kHz)
        let mut samples_received = 0;
        while consumer.try_pop().is_some() {
            samples_received += 1;
        }

        // Should have generated at least one frame's worth of samples
        assert!(
            samples_received >= 735 * 2,
            "Expected at least 1470 samples, got {}",
            samples_received
        );

        drop(handle);
    }
}
