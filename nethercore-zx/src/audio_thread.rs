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

use std::sync::mpsc::{self, RecvTimeoutError, SyncSender, TrySendError};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::Duration;

use ringbuf::traits::Producer;
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
    /// Sender for snapshots to audio thread
    tx: SyncSender<AudioGenSnapshot>,

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
        match self.tx.try_send(snapshot) {
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
        // Drop the sender to signal the thread to exit
        // The thread will receive Disconnected and exit its loop
        if let Some(handle) = self.handle.take() {
            // Wait for the thread to finish (with timeout)
            let _ = handle.join();
        }
    }
}

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
                };
                audio_gen.run();
            })
            .expect("failed to spawn audio generation thread");

        AudioGenHandle {
            tx,
            handle: Some(handle),
        }
    }

    /// Main thread loop
    fn run(&mut self) {
        debug!("Audio generation thread started");

        loop {
            // Wait for a snapshot with timeout
            // Timeout allows us to generate silence if main thread stalls
            match self.rx.recv_timeout(Duration::from_millis(20)) {
                Ok(snapshot) => {
                    self.process_snapshot(snapshot);
                }
                Err(RecvTimeoutError::Timeout) => {
                    // Main thread is slow - generate silence to prevent underrun
                    trace!("Audio thread: no snapshot received, generating silence");
                    self.generate_silence();
                }
                Err(RecvTimeoutError::Disconnected) => {
                    // Main thread dropped the sender - exit
                    debug!("Audio generation thread exiting (channel disconnected)");
                    break;
                }
            }
        }

        debug!("Audio generation thread finished");
    }

    /// Process a snapshot and generate audio samples
    fn process_snapshot(&mut self, snapshot: AudioGenSnapshot) {
        // Handle rollback - drain any pending snapshots
        if snapshot.is_rollback {
            debug!("Audio thread: rollback detected, draining pending snapshots");
            while self.rx.try_recv().is_ok() {
                // Drain pending snapshots
            }
        }

        // Apply tracker snapshot to local engine
        self.tracker_engine.apply_snapshot(&snapshot.tracker_snapshot);

        // Create mutable copies for generation
        let mut audio = snapshot.audio;
        let mut tracker = snapshot.tracker;

        // Generate samples
        self.output_buffer.clear();
        generate_audio_frame_with_tracker(
            &mut audio,
            &mut tracker,
            &mut self.tracker_engine,
            &snapshot.sounds,
            snapshot.tick_rate,
            snapshot.sample_rate,
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
                            for sample in data.iter_mut() {
                                *sample = consumer.try_pop().unwrap_or(0.0);
                            }
                        },
                        |err| error!("Audio stream error: {}", err),
                        None,
                    )
                    .map_err(|e| format!("Failed to build audio stream: {}", e))?
            }
            cpal::SampleFormat::I16 => {
                let config = config.into();
                device
                    .build_output_stream(
                        &config,
                        move |data: &mut [i16], _: &cpal::OutputCallbackInfo| {
                            use ringbuf::traits::Consumer;
                            for sample in data.iter_mut() {
                                let f = consumer.try_pop().unwrap_or(0.0);
                                *sample = (f * 32767.0).clamp(-32768.0, 32767.0) as i16;
                            }
                        },
                        |err| error!("Audio stream error: {}", err),
                        None,
                    )
                    .map_err(|e| format!("Failed to build audio stream: {}", e))?
            }
            cpal::SampleFormat::U16 => {
                let config = config.into();
                device
                    .build_output_stream(
                        &config,
                        move |data: &mut [u16], _: &cpal::OutputCallbackInfo| {
                            use ringbuf::traits::Consumer;
                            for sample in data.iter_mut() {
                                let f = consumer.try_pop().unwrap_or(0.0);
                                *sample = ((f * 32767.0 + 32768.0).clamp(0.0, 65535.0)) as u16;
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
}
