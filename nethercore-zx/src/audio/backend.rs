//! ZXAudio backend and AudioGenerator trait implementation

use std::sync::Arc;
use tracing::warn;

use super::Sound;
use super::generation::{advance_audio_positions, generate_audio_frame_with_tracker};
use super::output::{AudioOutput, OUTPUT_SAMPLE_RATE};

/// Nethercore ZX audio backend
///
/// Wraps AudioOutput and provides the Console::Audio interface.
/// Supports both synchronous (push_samples) and threaded (send_snapshot) modes.
pub struct ZXAudio {
    /// Audio output (cpal stream + ring buffer) - for synchronous mode
    output: Option<AudioOutput>,
    /// Threaded audio output - for threaded mode
    threaded_output: Option<crate::audio_thread::ThreadedAudioOutput>,
    /// Master volume (0.0 - 1.0)
    master_volume: f32,
    /// Pre-allocated buffer for volume scaling (avoids allocation per push)
    scale_buffer: Vec<f32>,
    /// Pre-allocated buffer for audio frame generation (avoids allocation per frame)
    frame_buffer: Vec<f32>,
    /// Whether to use threaded audio generation
    use_threaded: bool,
}

impl ZXAudio {
    /// Create new audio backend (synchronous mode)
    pub fn new() -> Result<Self, String> {
        match AudioOutput::new() {
            Ok(output) => Ok(Self {
                output: Some(output),
                threaded_output: None,
                master_volume: 1.0,
                scale_buffer: Vec::with_capacity(2048), // Pre-allocate for typical frame size
                frame_buffer: Vec::with_capacity(2048), // ~735*2 stereo samples at 60fps
                use_threaded: false,
            }),
            Err(e) => {
                warn!("Failed to create audio output: {}. Audio disabled.", e);
                Ok(Self {
                    output: None,
                    threaded_output: None,
                    master_volume: 1.0,
                    scale_buffer: Vec::new(),
                    frame_buffer: Vec::new(),
                    use_threaded: false,
                })
            }
        }
    }

    /// Create new audio backend with threaded generation
    ///
    /// This offloads audio sample generation to a separate thread,
    /// preventing audio pops during system load or rollback replays.
    pub fn new_threaded() -> Result<Self, String> {
        match crate::audio_thread::ThreadedAudioOutput::new() {
            Ok(output) => Ok(Self {
                output: None,
                threaded_output: Some(output),
                master_volume: 1.0,
                scale_buffer: Vec::new(), // Not needed for threaded mode
                frame_buffer: Vec::new(), // Not needed - uses lightweight advance
                use_threaded: true,
            }),
            Err(e) => {
                warn!(
                    "Failed to create threaded audio output: {}. Audio disabled.",
                    e
                );
                Ok(Self {
                    output: None,
                    threaded_output: None,
                    master_volume: 1.0,
                    scale_buffer: Vec::new(),
                    frame_buffer: Vec::new(),
                    use_threaded: true,
                })
            }
        }
    }

    /// Create a stub audio backend (no actual audio output)
    ///
    /// Use this when audio is needed for trait compliance but not for actual playback,
    /// such as during resource loading where the Audio trait is required but unused.
    pub fn new_stub() -> Self {
        Self {
            output: None,
            threaded_output: None,
            master_volume: 1.0,
            scale_buffer: Vec::new(),
            frame_buffer: Vec::new(),
            use_threaded: false,
        }
    }

    /// Check if using threaded audio mode
    pub fn is_threaded(&self) -> bool {
        self.use_threaded
    }

    /// Send an audio snapshot to the generation thread (threaded mode only)
    ///
    /// Returns true if the snapshot was queued, false if dropped or not in threaded mode.
    pub fn send_snapshot(&self, snapshot: crate::audio_thread::AudioGenSnapshot) -> bool {
        if let Some(ref output) = self.threaded_output {
            output.send_snapshot(snapshot)
        } else {
            false
        }
    }

    /// Set the master volume (0.0 - 1.0)
    pub fn set_master_volume(&mut self, volume: f32) {
        self.master_volume = volume.clamp(0.0, 1.0);
    }

    /// Get the current master volume
    pub fn master_volume(&self) -> f32 {
        self.master_volume
    }

    /// Get the sample rate (or default if audio is disabled)
    pub fn sample_rate(&self) -> u32 {
        if let Some(ref output) = self.threaded_output {
            output.sample_rate()
        } else if let Some(ref output) = self.output {
            output.sample_rate()
        } else {
            OUTPUT_SAMPLE_RATE
        }
    }

    /// Push generated audio samples to the output
    ///
    /// Samples are scaled by the master volume before being pushed to the output.
    pub fn push_samples(&mut self, samples: &[f32]) {
        if let Some(output) = &mut self.output {
            // Skip scaling if volume is at 100%
            if (self.master_volume - 1.0).abs() < f32::EPSILON {
                output.push_samples(samples);
            } else {
                // Scale samples by master volume using pre-allocated buffer
                self.scale_buffer.clear();
                self.scale_buffer
                    .extend(samples.iter().map(|s| s * self.master_volume));
                output.push_samples(&self.scale_buffer);
            }
        }
    }

    /// Get a reference to the sounds storage
    ///
    /// This is used to access loaded sounds for audio generation.
    /// Sounds are stored in ZXFFIState.sounds, not here.
    pub fn sounds<'a>(&self, _state: &'a crate::state::ZXFFIState) -> &'a [Option<Sound>] {
        // Sounds are stored in ZXFFIState, this method exists for API consistency
        // but the actual sounds slice comes from the state
        &[]
    }
}

impl Default for ZXAudio {
    fn default() -> Self {
        Self::new().unwrap_or(Self {
            output: None,
            threaded_output: None,
            master_volume: 1.0,
            scale_buffer: Vec::new(),
            frame_buffer: Vec::new(),
            use_threaded: false,
        })
    }
}

/// Audio generator for Nethercore ZX
///
/// Implements the AudioGenerator trait to enable console-agnostic audio generation
/// in the generic StandaloneApp.
pub struct ZXAudioGenerator;

impl nethercore_core::AudioGenerator for ZXAudioGenerator {
    type RollbackState = crate::state::ZRollbackState;
    type State = crate::state::ZXFFIState;
    type Audio = ZXAudio;

    fn default_sample_rate() -> u32 {
        OUTPUT_SAMPLE_RATE
    }

    fn generate_frame(
        rollback_state: &mut Self::RollbackState,
        state: &mut Self::State,
        tick_rate: u32,
        sample_rate: u32,
        output: &mut Vec<f32>,
    ) {
        generate_audio_frame_with_tracker(
            &mut rollback_state.audio,
            &mut rollback_state.tracker,
            &mut state.tracker_engine,
            &state.sounds,
            tick_rate,
            sample_rate,
            output,
        );
    }

    fn process_audio(
        rollback_state: &mut Self::RollbackState,
        state: &mut Self::State,
        audio: &mut Self::Audio,
        tick_rate: u32,
        sample_rate: u32,
    ) {
        if audio.is_threaded() {
            // Threaded mode: create snapshot and send to audio thread
            //
            // IMPORTANT: We must also advance the main thread's audio state!
            // The audio thread will generate samples from the snapshot, but the
            // main thread's rollback state must stay in sync (positions advance,
            // finished sounds get cleared, etc.) for deterministic rollback.
            //
            // Flow:
            // 1. Create snapshot with CURRENT positions (start of frame)
            // 2. Send snapshot to audio thread (it will generate samples)
            // 3. Advance main thread state using lightweight position-only advance
            //
            // The audio thread and main thread both advance positions by the same
            // amount, staying in sync.
            let snapshot = crate::audio_thread::AudioGenSnapshot {
                audio: rollback_state.audio,
                tracker: rollback_state.tracker,
                tracker_snapshot: state.tracker_engine.snapshot(),
                sounds: Arc::new(state.sounds.clone()),
                frame_number: 0, // frame_number not used currently
                tick_rate,
                sample_rate,
                is_rollback: false, // is_rollback - main loop only calls this for confirmed frames
            };
            audio.send_snapshot(snapshot);

            // Advance main thread state (lightweight - no sample generation)
            // This is ~10-20x faster than generate_frame as it skips mixing
            advance_audio_positions(
                &mut rollback_state.audio,
                &mut rollback_state.tracker,
                &mut state.tracker_engine,
                &state.sounds,
                tick_rate,
                sample_rate,
            );
        } else {
            // Synchronous mode: generate samples and push using reusable buffer
            // Note: We need to take the buffer out temporarily to avoid borrow conflicts
            let mut buffer = std::mem::take(&mut audio.frame_buffer);
            buffer.clear();
            Self::generate_frame(rollback_state, state, tick_rate, sample_rate, &mut buffer);
            audio.push_samples(&buffer);
            audio.frame_buffer = buffer;
        }
    }
}
