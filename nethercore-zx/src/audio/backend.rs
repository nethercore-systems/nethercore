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
    /// Cached sound table for audio snapshots (avoids per-frame cloning in threaded mode)
    cached_sounds: Option<Arc<Vec<Option<Sound>>>>,
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
                cached_sounds: None,
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
                    cached_sounds: None,
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
                cached_sounds: None,
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
                    cached_sounds: None,
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
            cached_sounds: None,
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
            cached_sounds: None,
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

    fn advance_state(
        rollback_state: &mut Self::RollbackState,
        state: &mut Self::State,
        tick_rate: u32,
        sample_rate: u32,
    ) {
        advance_audio_positions(
            &mut rollback_state.audio,
            &mut rollback_state.tracker,
            &mut state.tracker_engine,
            &state.sounds,
            tick_rate,
            sample_rate,
        );
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
            // Threaded mode sends canonical state to the output-only audio thread.
            let sounds = match &audio.cached_sounds {
                Some(cached) if cached.len() == state.sounds.len() => Arc::clone(cached),
                _ => {
                    let cached = Arc::new(state.sounds.clone());
                    audio.cached_sounds = Some(Arc::clone(&cached));
                    cached
                }
            };

            let snapshot = crate::audio_thread::AudioGenSnapshot {
                audio: rollback_state.audio,
                tracker: rollback_state.tracker,
                tracker_snapshot: state.tracker_engine.snapshot(),
                sounds,
                frame_number: 0, // frame_number not used currently
                tick_rate,
                sample_rate,
                is_rollback: false, // is_rollback - main loop only calls this for confirmed frames
            };
            audio.send_snapshot(snapshot);
        } else {
            // Generate output from copies so audible processing never advances
            // checksummed state or the canonical tracker engine a second time.
            let mut buffer = std::mem::take(&mut audio.frame_buffer);
            buffer.clear();
            let mut output_state = *rollback_state;
            let tracker_snapshot = state.tracker_engine.snapshot();
            Self::generate_frame(
                &mut output_state,
                state,
                tick_rate,
                sample_rate,
                &mut buffer,
            );
            state.tracker_engine.apply_snapshot(&tracker_snapshot);
            audio.push_samples(&buffer);
            audio.frame_buffer = buffer;
        }
    }
}

#[cfg(test)]
mod onset_regression {
    use super::*;
    use nethercore_core::AudioGenerator;
    #[test]
    fn sub_tick_sfx_outputs_its_onset_before_canonical_completion() {
        let mut rollback = crate::state::ZRollbackState::default();
        rollback.audio.channels[0].sound = 1;
        rollback.audio.channels[0].volume = 1.0;
        let mut state = crate::state::ZXFFIState::default();
        state.sounds = vec![
            None,
            Some(Sound {
                data: Arc::new(vec![16000; 100]),
            }),
        ];
        let mut audio = ZXAudio::new_stub();
        ZXAudioGenerator::process_audio(
            &mut rollback,
            &mut state,
            &mut audio,
            60,
            OUTPUT_SAMPLE_RATE,
        );
        assert!(
            audio.frame_buffer[0] > 0.1,
            "the leading sample must be audible"
        );
        assert_eq!(rollback.audio.channels[0].position, 0);
        ZXAudioGenerator::advance_state(&mut rollback, &mut state, 60, OUTPUT_SAMPLE_RATE);
        assert_eq!(rollback.audio.channels[0].sound, 0);
    }
}
