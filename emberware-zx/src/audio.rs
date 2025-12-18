//! Emberware ZX audio backend
//!
//! Per-frame audio generation with rollback support.
//!
//! Architecture:
//! - Audio state (playhead positions, volumes) is part of ZRollbackState
//! - Each frame, generate_audio_frame() generates samples from the current state
//! - Samples are pushed to a ring buffer consumed by the cpal audio thread
//! - During rollback, state is restored and no samples are generated
//!
//! Audio specs:
//! - 44,100 Hz sample rate (native for most audio hardware)
//! - Stereo output
//! - 16-bit signed PCM mono source sounds (22,050 Hz, upsampled)

use std::sync::Arc;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use ringbuf::{
    HeapRb,
    traits::{Consumer, Producer, Split},
};
use tracing::{debug, error, info, warn};

use crate::state::{AudioPlaybackState, ChannelState};

/// Audio sample rate for output (44.1 kHz - native for most hardware)
pub const OUTPUT_SAMPLE_RATE: u32 = 44_100;

/// Audio sample rate for source sounds (22.05 kHz - PS1/N64 authentic)
pub const SOURCE_SAMPLE_RATE: u32 = 22_050;

/// Ring buffer size in samples (stereo frames * 2 channels)
/// ~150ms buffer at 44.1kHz = 6615 frames * 2 channels = 13230 samples
/// Larger buffer provides more headroom for frame timing jitter
const RING_BUFFER_SIZE: usize = 13230;

/// Sound data (raw PCM)
#[derive(Clone, Debug)]
pub struct Sound {
    /// Raw PCM data (16-bit signed, mono, 22.05kHz)
    pub data: Arc<Vec<i16>>,
}

/// Audio output using cpal and ring buffer
pub struct AudioOutput {
    /// Producer side of the ring buffer (main thread writes here)
    producer: ringbuf::HeapProd<f32>,
    /// The cpal stream (kept alive for the duration)
    _stream: cpal::Stream,
    /// Output sample rate
    sample_rate: u32,
}

impl AudioOutput {
    /// Create a new audio output
    pub fn new() -> Result<Self, String> {
        let host = cpal::default_host();

        let device = host
            .default_output_device()
            .ok_or_else(|| "No audio output device available".to_string())?;

        let config = device
            .default_output_config()
            .map_err(|e| format!("Failed to get default output config: {}", e))?;

        let sample_rate = config.sample_rate().0;

        info!(
            "Audio output: {} channels, {} Hz",
            config.channels(),
            sample_rate
        );

        // Create ring buffer
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
                            // Read samples from ring buffer, fill with silence if not enough
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

        debug!("Audio stream started");

        Ok(Self {
            producer,
            _stream: stream,
            sample_rate,
        })
    }

    /// Push audio samples to the ring buffer
    ///
    /// Samples should be interleaved stereo (left, right, left, right, ...)
    pub fn push_samples(&mut self, samples: &[f32]) {
        // Push as many samples as we can fit
        let pushed = self.producer.push_slice(samples);
        if pushed < samples.len() {
            // Ring buffer overflow - this can happen if game is running slow
            // Just drop the extra samples (audio will slightly desync but recover)
            debug!(
                "Audio buffer overflow: dropped {} samples",
                samples.len() - pushed
            );
        }
    }

    /// Get the output sample rate
    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }
}

/// Generate one frame of audio samples
///
/// This is called once per confirmed game frame (not during rollback).
/// It reads the current audio state, mixes all active channels, and
/// outputs interleaved stereo samples.
///
/// # Arguments
/// * `playback_state` - Current audio playback state (will be mutated to advance playheads)
/// * `sounds` - Loaded sound data (indexed by sound handle)
/// * `tick_rate` - Game tick rate (e.g., 60 for 60fps)
/// * `sample_rate` - Output sample rate (e.g., 44100)
/// * `output` - Output buffer for interleaved stereo samples
pub fn generate_audio_frame(
    playback_state: &mut AudioPlaybackState,
    sounds: &[Option<Sound>],
    tick_rate: u32,
    sample_rate: u32,
    output: &mut Vec<f32>,
) {
    // Calculate how many output samples per frame
    // At 60fps with 44100Hz: 44100/60 = 735 samples per frame
    let samples_per_frame = sample_rate / tick_rate;

    // Clear output buffer and reserve space for stereo samples
    output.clear();
    output.reserve(samples_per_frame as usize * 2);

    // Calculate resampling ratio (source is 22050Hz, output is usually 44100Hz)
    let resample_ratio = SOURCE_SAMPLE_RATE as f32 / sample_rate as f32;

    // Generate each output sample
    for _ in 0..samples_per_frame {
        let mut left = 0.0f32;
        let mut right = 0.0f32;

        // Mix all active channels
        for channel in playback_state.channels.iter_mut() {
            if channel.sound == 0 {
                continue; // Channel is silent
            }

            if let Some(sample) = mix_channel(channel, sounds, resample_ratio) {
                let (l, r) = apply_pan(sample, channel.pan, channel.volume);
                left += l;
                right += r;
            }
        }

        // Mix music channel
        if playback_state.music.sound != 0
            && let Some(sample) = mix_channel(&mut playback_state.music, sounds, resample_ratio)
        {
            // Music is centered (no pan)
            let vol = playback_state.music.volume;
            left += sample * vol;
            right += sample * vol;
        }

        // Soft clamp to prevent harsh clipping
        left = soft_clip(left);
        right = soft_clip(right);

        output.push(left);
        output.push(right);
    }
}

/// Mix a single channel, returning the sample value and advancing the playhead
///
/// # Precondition
/// `channel.sound` must be non-zero (callers must check before calling)
fn mix_channel(
    channel: &mut ChannelState,
    sounds: &[Option<Sound>],
    resample_ratio: f32,
) -> Option<f32> {
    let sound_idx = channel.sound as usize;
    debug_assert!(sound_idx != 0, "mix_channel called with silent channel");

    // Validate sound handle (handles start at 1, stored at their index)
    if sound_idx >= sounds.len() {
        warn!(
            "mix_channel: sound handle {} out of bounds (max {})",
            sound_idx,
            sounds.len()
        );
        channel.sound = 0; // Stop the invalid channel
        return None;
    }

    let Some(sound) = sounds.get(sound_idx).and_then(|s| s.as_ref()) else {
        warn!("mix_channel: sound handle {} has no data", sound_idx);
        channel.sound = 0; // Stop the invalid channel
        return None;
    };
    let data = &sound.data;

    if data.is_empty() {
        return None;
    }

    // Get current position (in source samples)
    let source_pos = channel.position as f32 * resample_ratio;
    let source_idx = source_pos as usize;

    // Check if we've reached the end
    if source_idx >= data.len() {
        if channel.looping != 0 {
            // Loop back to start
            channel.position = 0;
            return mix_channel(channel, sounds, resample_ratio);
        } else {
            // Sound finished
            channel.sound = 0;
            channel.position = 0;
            return None;
        }
    }

    // Linear interpolation for smoother resampling
    let frac = source_pos.fract();
    let sample1 = data[source_idx] as f32 / 32768.0;
    let sample2 = if source_idx + 1 < data.len() {
        data[source_idx + 1] as f32 / 32768.0
    } else if channel.looping != 0 {
        data[0] as f32 / 32768.0
    } else {
        sample1
    };
    let sample = sample1 + (sample2 - sample1) * frac;

    // Advance playhead (in output sample rate)
    channel.position += 1;

    Some(sample)
}

/// Apply equal-power panning and volume
///
/// Equal-power panning formula ensures constant perceived loudness across the stereo field:
/// - pan = -1: left = 1.0, right = 0.0 (full left)
/// - pan = 0: left = 0.707, right = 0.707 (center, -3dB each)
/// - pan = +1: left = 0.0, right = 1.0 (full right)
fn apply_pan(sample: f32, pan: f32, volume: f32) -> (f32, f32) {
    // pan and volume are already clamped when stored in ChannelState (via clamp_safe)
    let angle = (pan + 1.0) * 0.25 * std::f32::consts::PI; // Map -1..1 to 0..PI/2
    let left_gain = angle.cos();
    let right_gain = angle.sin();

    let scaled = sample * volume;
    (scaled * left_gain, scaled * right_gain)
}

/// Soft clipping to prevent harsh digital clipping
///
/// Uses hyperbolic tangent for smooth compression:
/// - Values in [-1, 1] pass through unchanged
/// - Values outside are smoothly compressed toward ±2.0 asymptotically
fn soft_clip(x: f32) -> f32 {
    if x.abs() <= 1.0 {
        x
    } else {
        // For |x| > 1, smoothly compress using tanh
        // tanh(1) ≈ 0.76, so soft_clip(2) ≈ 1.76
        x.signum() * (1.0 + (x.abs() - 1.0).tanh())
    }
}

/// Emberware ZX audio backend
///
/// Wraps AudioOutput and provides the Console::Audio interface.
pub struct ZAudio {
    /// Audio output (cpal stream + ring buffer)
    output: Option<AudioOutput>,
    /// Master volume (0.0 - 1.0)
    master_volume: f32,
}

impl ZAudio {
    /// Create new audio backend
    pub fn new() -> Result<Self, String> {
        match AudioOutput::new() {
            Ok(output) => Ok(Self {
                output: Some(output),
                master_volume: 1.0,
            }),
            Err(e) => {
                warn!("Failed to create audio output: {}. Audio disabled.", e);
                Ok(Self {
                    output: None,
                    master_volume: 1.0,
                })
            }
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
        self.output
            .as_ref()
            .map(|o| o.sample_rate())
            .unwrap_or(OUTPUT_SAMPLE_RATE)
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
                // Scale samples by master volume
                let scaled: Vec<f32> = samples.iter().map(|s| s * self.master_volume).collect();
                output.push_samples(&scaled);
            }
        }
    }

    /// Get a reference to the sounds storage
    ///
    /// This is used to access loaded sounds for audio generation.
    /// Sounds are stored in ZFFIState.sounds, not here.
    pub fn sounds<'a>(&self, _state: &'a crate::state::ZFFIState) -> &'a [Option<Sound>] {
        // Sounds are stored in ZFFIState, this method exists for API consistency
        // but the actual sounds slice comes from the state
        &[]
    }
}

impl Default for ZAudio {
    fn default() -> Self {
        Self::new().unwrap_or(Self {
            output: None,
            master_volume: 1.0,
        })
    }
}

/// Audio generator for Emberware ZX
///
/// Implements the AudioGenerator trait to enable console-agnostic audio generation
/// in the generic StandaloneApp.
pub struct ZXAudioGenerator;

impl emberware_core::AudioGenerator for ZXAudioGenerator {
    type RollbackState = crate::state::ZRollbackState;
    type State = crate::state::ZFFIState;

    fn default_sample_rate() -> u32 {
        OUTPUT_SAMPLE_RATE
    }

    fn generate_frame(
        rollback_state: &mut Self::RollbackState,
        state: &Self::State,
        tick_rate: u32,
        sample_rate: u32,
        output: &mut Vec<f32>,
    ) {
        generate_audio_frame(
            &mut rollback_state.audio,
            &state.sounds,
            tick_rate,
            sample_rate,
            output,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_apply_pan_center() {
        let (l, r) = apply_pan(1.0, 0.0, 1.0);
        // Center should be roughly equal power (-3dB each)
        assert!((l - 0.707).abs() < 0.01);
        assert!((r - 0.707).abs() < 0.01);
    }

    #[test]
    fn test_apply_pan_left() {
        let (l, r) = apply_pan(1.0, -1.0, 1.0);
        assert!((l - 1.0).abs() < 0.01);
        assert!(r.abs() < 0.01);
    }

    #[test]
    fn test_apply_pan_right() {
        let (l, r) = apply_pan(1.0, 1.0, 1.0);
        assert!(l.abs() < 0.01);
        assert!((r - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_soft_clip_passthrough() {
        assert_eq!(soft_clip(0.5), 0.5);
        assert_eq!(soft_clip(-0.5), -0.5);
        assert_eq!(soft_clip(1.0), 1.0);
        assert_eq!(soft_clip(-1.0), -1.0);
    }

    #[test]
    fn test_soft_clip_limits() {
        // Values > 1 should be soft clipped but approach 2.0 asymptotically
        let clipped = soft_clip(2.0);
        assert!(clipped > 1.0 && clipped < 2.0);

        let clipped_neg = soft_clip(-2.0);
        assert!(clipped_neg < -1.0 && clipped_neg > -2.0);
    }

    #[test]
    fn test_generate_empty_state() {
        let mut state = AudioPlaybackState::default();
        let sounds: Vec<Option<Sound>> = vec![];
        let mut output = Vec::new();

        generate_audio_frame(&mut state, &sounds, 60, 44100, &mut output);

        // Should generate silence (735 stereo samples at 60fps/44100Hz)
        assert_eq!(output.len(), 735 * 2);
        assert!(output.iter().all(|&s| s == 0.0));
    }
}
