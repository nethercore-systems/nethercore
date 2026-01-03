//! Nethercore ZX audio backend
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
use tracing::{debug, error, warn};

use crate::state::{AudioPlaybackState, ChannelState, TrackerState, tracker_flags};
use crate::tracker::TrackerEngine;

/// Audio sample rate for output (44.1 kHz - native for most hardware)
pub const OUTPUT_SAMPLE_RATE: u32 = 44_100;

/// Audio sample rate for source sounds (22.05 kHz - PS1/N64 authentic)
pub const SOURCE_SAMPLE_RATE: u32 = 22_050;

/// Ring buffer size in samples (stereo frames * 2 channels)
/// ~100ms buffer at 44.1kHz = 4410 frames * 2 channels = 8820 samples
/// This provides ~6 frames of headroom at 60fps - enough for minor jitter.
const RING_BUFFER_SIZE: usize = 8820; // ~100ms buffer

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

        // Create ring buffer
        let ring = HeapRb::<f32>::new(RING_BUFFER_SIZE);
        let (producer, mut consumer) = ring.split();

        // Build the stream based on sample format
        // NOTE: Using batch pop_slice() instead of per-sample try_pop() to prevent timing gaps
        let stream = match config.sample_format() {
            cpal::SampleFormat::F32 => {
                let config = config.into();
                device
                    .build_output_stream(
                        &config,
                        move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                            // Batch read all available samples (much more efficient than per-sample)
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
                // Pre-allocate buffer for batch reads
                let mut temp_buffer: Vec<f32> = vec![0.0; 4096];
                device
                    .build_output_stream(
                        &config,
                        move |data: &mut [i16], _: &cpal::OutputCallbackInfo| {
                            // Resize temp buffer if needed
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
                // Pre-allocate buffer for batch reads
                let mut temp_buffer: Vec<f32> = vec![0.0; 4096];
                device
                    .build_output_stream(
                        &config,
                        move |data: &mut [u16], _: &cpal::OutputCallbackInfo| {
                            // Resize temp buffer if needed
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

/// Generate one frame of audio samples with tracker support
///
/// This is called once per confirmed game frame (not during rollback).
/// It reads the current audio state, mixes all active channels including
/// tracker output, and outputs interleaved stereo samples.
///
/// # Arguments
/// * `playback_state` - Current audio playback state (will be mutated to advance playheads)
/// * `tracker_state` - Current tracker state (will be mutated to advance position)
/// * `tracker_engine` - Tracker engine instance (for channel state and module data)
/// * `sounds` - Loaded sound data (indexed by sound handle)
/// * `tick_rate` - Game tick rate (e.g., 60 for 60fps)
/// * `sample_rate` - Output sample rate (e.g., 44100)
/// * `output` - Output buffer for interleaved stereo samples
pub fn generate_audio_frame_with_tracker(
    playback_state: &mut AudioPlaybackState,
    tracker_state: &mut TrackerState,
    tracker_engine: &mut TrackerEngine,
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

    // Check if tracker is active (mutually exclusive with PCM music)
    let tracker_active = tracker_state.handle != 0
        && (tracker_state.flags & tracker_flags::PLAYING) != 0
        && (tracker_state.flags & tracker_flags::PAUSED) == 0;

    // Sync tracker engine to state at start of frame
    if tracker_active {
        tracker_engine.sync_to_state(tracker_state, sounds);
    }

    // Generate each output sample
    for _ in 0..samples_per_frame {
        let mut left = 0.0f32;
        let mut right = 0.0f32;

        // Mix all active SFX channels
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

        // Mix tracker OR PCM music (mutually exclusive)
        if tracker_active {
            // Mix tracker output and advance tracker state
            let (tracker_l, tracker_r) =
                tracker_engine.render_sample_and_advance(tracker_state, sounds, sample_rate);
            left += tracker_l;
            right += tracker_r;
        } else if playback_state.music.sound != 0
            && let Some(sample) = mix_channel(&mut playback_state.music, sounds, resample_ratio)
        {
            // Mix PCM music (centered, no pan)
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

/// Advance audio playback positions without generating samples
///
/// This is used in threaded audio mode where the audio thread generates
/// the actual samples from a snapshot. The main thread still needs to
/// advance positions to maintain rollback state consistency.
///
/// This is ~10-20x faster than `generate_audio_frame_with_tracker` since
/// it skips interpolation, panning, mixing, and soft clipping.
pub fn advance_audio_positions(
    playback_state: &mut AudioPlaybackState,
    tracker_state: &mut TrackerState,
    tracker_engine: &mut TrackerEngine,
    sounds: &[Option<Sound>],
    tick_rate: u32,
    sample_rate: u32,
) {
    let samples_per_frame = sample_rate / tick_rate;
    let resample_ratio = SOURCE_SAMPLE_RATE as f32 / sample_rate as f32;

    // Check if tracker is active
    let tracker_active = tracker_state.handle != 0
        && (tracker_state.flags & tracker_flags::PLAYING) != 0
        && (tracker_state.flags & tracker_flags::PAUSED) == 0;

    // Sync tracker engine to state at start of frame
    if tracker_active {
        tracker_engine.sync_to_state(tracker_state, sounds);
    }

    // Advance SFX channel positions
    for channel in playback_state.channels.iter_mut() {
        if channel.sound == 0 {
            continue;
        }
        advance_channel_position(channel, sounds, resample_ratio, samples_per_frame);
    }

    // Advance music channel position (if not using tracker)
    if !tracker_active && playback_state.music.sound != 0 {
        advance_channel_position(&mut playback_state.music, sounds, resample_ratio, samples_per_frame);
    }

    // Advance tracker position (if using tracker)
    if tracker_active {
        tracker_engine.advance_positions(tracker_state, sounds, samples_per_frame, sample_rate);
    }
}

/// Advance a single channel's position by one frame's worth of samples
///
/// This is a lightweight version of `mix_channel` that only advances the playhead
/// position without performing interpolation or returning a sample value.
fn advance_channel_position(
    channel: &mut ChannelState,
    sounds: &[Option<Sound>],
    resample_ratio: f32,
    samples_per_frame: u32,
) {
    let sound_idx = channel.sound as usize;

    // Validate sound handle (handles start at 1, stored at their index)
    let sound_len = match sounds.get(sound_idx).and_then(|s| s.as_ref()) {
        Some(s) => s.data.len(),
        None => {
            channel.sound = 0;
            return;
        }
    };

    if sound_len == 0 {
        return;
    }

    // Calculate total position advancement for this frame
    // Each output sample advances by resample_ratio source samples
    let total_advance = samples_per_frame as f32 * resample_ratio;

    // Get current position
    let (current_idx, current_frac) = channel.get_position();
    let current_pos = current_idx as f32 + current_frac;
    let new_pos = current_pos + total_advance;

    if new_pos >= sound_len as f32 {
        if channel.looping != 0 {
            // Wrap position for looping sounds
            let wrapped = new_pos % sound_len as f32;
            channel.set_position(wrapped);
        } else {
            // Sound finished
            channel.sound = 0;
            channel.reset_position();
        }
    } else {
        // Normal advancement - add to fixed-point position directly
        let delta_fixed = (total_advance * ChannelState::FRAC_ONE as f32) as u32;
        channel.position = channel.position.wrapping_add(delta_fixed);
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

    // Get current position (24.8 fixed-point) as (integer, fraction)
    let (source_idx, frac) = channel.get_position();

    // Check if we've reached the end
    if source_idx >= data.len() {
        if channel.looping != 0 {
            // Loop back to start
            channel.reset_position();
            return mix_channel(channel, sounds, resample_ratio);
        } else {
            // Sound finished
            channel.sound = 0;
            channel.reset_position();
            return None;
        }
    }

    // Linear interpolation for smoother resampling
    let sample1 = data[source_idx] as f32 / 32768.0;
    let sample2 = if source_idx + 1 < data.len() {
        data[source_idx + 1] as f32 / 32768.0
    } else if channel.looping != 0 {
        data[0] as f32 / 32768.0
    } else {
        sample1
    };
    let sample = sample1 + (sample2 - sample1) * frac;

    // Advance playhead by fractional resample ratio for smooth sub-sample tracking
    channel.advance_position(resample_ratio);

    Some(sample)
}

/// 17-point quarter-sine lookup table (cos values for left channel).
/// Values are cos(i * PI/32) for i = 0..16, scaled to 0-255.
const PAN_COS_LUT: [u8; 17] = [
    255, 254, 251, 245, 237, 226, 213, 198, 181, 162, 142, 121, 98, 75, 51, 26, 0,
];

/// Fast panning gains using 17-point LUT with interpolation.
#[inline]
fn fast_pan_gains(pan: f32) -> (f32, f32) {
    // Map pan [-1, 1] to [0, 16] range
    let pos = (pan + 1.0) * 8.0;
    let idx = (pos as usize).min(15);
    let frac = pos - idx as f32;

    // Linear interpolation between LUT points
    let cos_val = PAN_COS_LUT[idx] as f32 * (1.0 - frac) + PAN_COS_LUT[idx + 1] as f32 * frac;
    let sin_val = PAN_COS_LUT[16 - idx] as f32 * (1.0 - frac) + PAN_COS_LUT[15 - idx] as f32 * frac;

    (cos_val / 255.0, sin_val / 255.0)
}

/// Apply equal-power panning and volume to a sample.
///
/// Uses LUT-based panning for constant perceived loudness across the stereo field:
///   - pan = -1: full left
///   - pan = 0: center (-3dB each channel)
///   - pan = +1: full right
#[inline]
fn apply_pan(sample: f32, pan: f32, volume: f32) -> (f32, f32) {
    let (left_gain, right_gain) = fast_pan_gains(pan);
    let scaled = sample * volume;
    (scaled * left_gain, scaled * right_gain)
}

/// Tanh lookup table for soft clipping (29 points, t = 0.0 to 7.0 in steps of 0.25).
/// Values are tanh(t) for t = 0.00, 0.25, 0.50, ..., 7.00.
/// Used for fast soft clipping without expensive tanh() calls.
const TANH_LUT: [f32; 29] = [
    0.0,      // t=0.00
    0.244919, // t=0.25
    0.462117, // t=0.50
    0.635149, // t=0.75
    0.761594, // t=1.00
    0.848284, // t=1.25
    0.905148, // t=1.50
    0.941389, // t=1.75
    0.964028, // t=2.00
    0.978034, // t=2.25
    0.986614, // t=2.50
    0.991815, // t=2.75
    0.995055, // t=3.00
    0.997109, // t=3.25
    0.998396, // t=3.50
    0.999198, // t=3.75
    0.999665, // t=4.00
    0.999892, // t=4.25
    0.999988, // t=4.50
    0.999998, // t=4.75
    1.0,      // t=5.00+
    1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, // t=5.25-7.00
];

/// Soft clipping to prevent harsh digital clipping
///
/// Uses lookup table approximation of hyperbolic tangent for smooth compression:
/// - Values in [-1, 1] pass through unchanged
/// - Values outside are smoothly compressed toward ±2.0 asymptotically
///
/// Performance: ~20x faster than tanh() for the clipping path.
#[inline]
fn soft_clip(x: f32) -> f32 {
    if x.abs() <= 1.0 {
        return x;
    }

    // For |x| > 1, compute: sign(x) * (1 + tanh(|x| - 1))
    // Using LUT with linear interpolation
    let t = x.abs() - 1.0; // Range: 0.0 to ~7.0 (for inputs up to ±8)
    let t = t.min(7.0); // Clamp to LUT range

    // Map t to LUT index (step size = 0.25, so multiply by 4)
    let pos = t * 4.0;
    let idx = pos as usize;
    let frac = pos - idx as f32;

    // Linear interpolation between LUT points
    let idx = idx.min(27); // Ensure we don't read past end
    let tanh_val = TANH_LUT[idx] * (1.0 - frac) + TANH_LUT[idx + 1] * frac;

    x.signum() * (1.0 + tanh_val)
}

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
                warn!("Failed to create threaded audio output: {}. Audio disabled.", e);
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
            let snapshot = crate::audio_thread::AudioGenSnapshot::new(
                rollback_state.audio,
                rollback_state.tracker,
                state.tracker_engine.snapshot(),
                Arc::new(state.sounds.clone()),
                0, // frame_number not used currently
                tick_rate,
                sample_rate,
                false, // is_rollback - main loop only calls this for confirmed frames
            );
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
        let mut tracker_state = TrackerState::default();
        let mut tracker_engine = TrackerEngine::new();
        let sounds: Vec<Option<Sound>> = vec![];
        let mut output = Vec::new();

        generate_audio_frame_with_tracker(
            &mut state,
            &mut tracker_state,
            &mut tracker_engine,
            &sounds,
            60,
            44100,
            &mut output,
        );

        // Should generate silence (735 stereo samples at 60fps/44100Hz)
        assert_eq!(output.len(), 735 * 2);
        assert!(output.iter().all(|&s| s == 0.0));
    }

    #[test]
    fn test_generate_frame_advances_position() {
        // Test that generate_audio_frame_with_tracker advances channel positions
        // This is critical for threaded audio: if positions don't advance,
        // sounds would restart from the beginning every frame!

        // Create a simple sound (1 second of samples at 22050 Hz)
        let sound_data: Vec<i16> = (0..22050).map(|i| (i % 1000) as i16).collect();
        let sound = Sound {
            data: Arc::new(sound_data),
        };

        // Set up state with a playing sound on channel 0
        let mut state = AudioPlaybackState::default();
        state.channels[0].sound = 1; // Sound handle 1
        state.channels[0].volume = 1.0;
        state.channels[0].pan = 0.0;
        state.channels[0].position = 0; // Start at position 0
        state.channels[0].looping = 0;

        let mut tracker_state = TrackerState::default();
        let mut tracker_engine = TrackerEngine::new();

        // Sound at index 1 (handles start at 1)
        let sounds: Vec<Option<Sound>> = vec![None, Some(sound)];
        let mut output = Vec::new();

        // Get initial position
        let initial_position = state.channels[0].position;

        // Generate one frame
        generate_audio_frame_with_tracker(
            &mut state,
            &mut tracker_state,
            &mut tracker_engine,
            &sounds,
            60,
            44100,
            &mut output,
        );

        // Position should have advanced
        // At 60fps with 44100Hz output and 22050Hz source, we generate 735 output samples
        // Each output sample advances by 0.5 source samples (22050/44100)
        // So position should advance by ~367.5 (in fixed point: 367.5 * 256 = ~94080)
        let new_position = state.channels[0].position;
        assert!(
            new_position > initial_position,
            "Position should advance: initial={}, new={}",
            initial_position,
            new_position
        );

        // Position should advance by approximately 735 * 0.5 = 367.5 source samples
        // In 24.8 fixed point, that's about 367.5 * 256 = ~94080
        let position_delta = new_position - initial_position;
        let expected_delta = (735.0 * 0.5 * 256.0) as u32;
        assert!(
            (position_delta as i64 - expected_delta as i64).abs() < 512,
            "Position delta {} should be close to expected {}",
            position_delta,
            expected_delta
        );
    }

    #[test]
    fn test_channel_sound_cleared_when_finished() {
        // Test that channel.sound is set to 0 when sound finishes playing
        // This is important for the game to know when to start new sounds

        // Create a very short sound (100 samples)
        let sound_data: Vec<i16> = (0..100).map(|i| (i * 100) as i16).collect();
        let sound = Sound {
            data: Arc::new(sound_data),
        };

        let mut state = AudioPlaybackState::default();
        state.channels[0].sound = 1;
        state.channels[0].volume = 1.0;
        state.channels[0].pan = 0.0;
        state.channels[0].position = 0;
        state.channels[0].looping = 0; // Non-looping

        let mut tracker_state = TrackerState::default();
        let mut tracker_engine = TrackerEngine::new();
        let sounds: Vec<Option<Sound>> = vec![None, Some(sound)];
        let mut output = Vec::new();

        // The sound is 100 samples at 22050Hz
        // At 44100Hz output with 0.5 resample ratio, we'll exhaust it in ~200 output samples
        // One frame at 60fps is 735 samples, so sound should finish

        generate_audio_frame_with_tracker(
            &mut state,
            &mut tracker_state,
            &mut tracker_engine,
            &sounds,
            60,
            44100,
            &mut output,
        );

        // Channel should be cleared (sound = 0) because the short sound finished
        assert_eq!(
            state.channels[0].sound, 0,
            "Channel sound should be cleared to 0 when sound finishes"
        );
    }

    #[test]
    fn test_advance_positions_matches_generate_frame() {
        // Test that advance_audio_positions produces the same final positions
        // as generate_audio_frame_with_tracker (which is critical for rollback determinism)

        // Create a sound with enough samples for multiple frames
        let sound_data: Vec<i16> = (0..22050).map(|i| (i % 1000) as i16).collect();
        let sound = Sound {
            data: Arc::new(sound_data),
        };

        // Set up identical states for both paths
        let mut state1 = AudioPlaybackState::default();
        let mut state2 = AudioPlaybackState::default();

        // Play same sound on channel 0
        state1.channels[0].sound = 1;
        state1.channels[0].volume = 0.8;
        state1.channels[0].pan = 0.3;
        state1.channels[0].position = 0;
        state1.channels[0].looping = 1; // Looping to test wrap-around

        state2.channels[0].sound = 1;
        state2.channels[0].volume = 0.8;
        state2.channels[0].pan = 0.3;
        state2.channels[0].position = 0;
        state2.channels[0].looping = 1;

        // Also test music channel
        state1.music.sound = 1;
        state1.music.volume = 0.5;
        state1.music.position = 5000 << ChannelState::FRAC_BITS; // Start mid-sound

        state2.music.sound = 1;
        state2.music.volume = 0.5;
        state2.music.position = 5000 << ChannelState::FRAC_BITS;

        let mut tracker_state1 = TrackerState::default();
        let mut tracker_state2 = TrackerState::default();
        let mut tracker_engine1 = TrackerEngine::new();
        let mut tracker_engine2 = TrackerEngine::new();

        let sounds: Vec<Option<Sound>> = vec![None, Some(sound)];

        // Advance state1 using full generation (the original method)
        let mut output = Vec::new();
        generate_audio_frame_with_tracker(
            &mut state1,
            &mut tracker_state1,
            &mut tracker_engine1,
            &sounds,
            60,
            44100,
            &mut output,
        );

        // Advance state2 using lightweight position-only method
        advance_audio_positions(
            &mut state2,
            &mut tracker_state2,
            &mut tracker_engine2,
            &sounds,
            60,
            44100,
        );

        // Verify positions match exactly
        assert_eq!(
            state1.channels[0].position, state2.channels[0].position,
            "Channel 0 position mismatch: generate_frame={}, advance_positions={}",
            state1.channels[0].position, state2.channels[0].position
        );

        assert_eq!(
            state1.channels[0].sound, state2.channels[0].sound,
            "Channel 0 sound handle mismatch"
        );

        assert_eq!(
            state1.music.position, state2.music.position,
            "Music channel position mismatch: generate_frame={}, advance_positions={}",
            state1.music.position, state2.music.position
        );

        assert_eq!(
            state1.music.sound, state2.music.sound,
            "Music channel sound handle mismatch"
        );
    }

    #[test]
    fn test_advance_positions_sound_finishes_correctly() {
        // Test that advance_audio_positions correctly clears sound when finished
        // (matches behavior of generate_audio_frame_with_tracker)

        // Create a very short sound that will finish within one frame
        let sound_data: Vec<i16> = (0..100).map(|i| (i * 100) as i16).collect();
        let sound = Sound {
            data: Arc::new(sound_data),
        };

        let mut state = AudioPlaybackState::default();
        state.channels[0].sound = 1;
        state.channels[0].volume = 1.0;
        state.channels[0].position = 0;
        state.channels[0].looping = 0; // Non-looping

        let mut tracker_state = TrackerState::default();
        let mut tracker_engine = TrackerEngine::new();
        let sounds: Vec<Option<Sound>> = vec![None, Some(sound)];

        // Advance using lightweight method
        advance_audio_positions(
            &mut state,
            &mut tracker_state,
            &mut tracker_engine,
            &sounds,
            60,
            44100,
        );

        // Channel should be cleared because the short sound finished
        assert_eq!(
            state.channels[0].sound, 0,
            "Channel sound should be cleared when sound finishes (advance_positions)"
        );
    }
}
