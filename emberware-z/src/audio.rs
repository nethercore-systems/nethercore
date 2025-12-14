//! Per-frame audio generation for Emberware Z
//!
//! This module implements deterministic audio generation that integrates with
//! GGRS rollback netcode. Audio state (playhead positions, volumes) is part of
//! the rollback state, ensuring perfect audio synchronization during rollback.
//!
//! # Architecture
//!
//! - `AudioOutput`: cpal stream + lock-free ring buffer (runs in audio thread)
//! - `AudioPlaybackState`: POD struct saved/restored during rollback
//! - `generate_audio_frame()`: Called once per frame to generate samples
//!
//! # Sample Format
//!
//! - Sample rate: 22,050 Hz (PS1/N64 authentic)
//! - Format: 16-bit signed mono PCM (stored in Sound)
//! - Output: Stereo interleaved (L, R, L, R, ...)
//! - Equal-power panning for constant loudness

use std::sync::Arc;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Device, SampleFormat, Stream, StreamConfig};
use ringbuf::traits::{Consumer, Producer, Split};
use ringbuf::HeapRb;

use bytemuck::{Pod, Zeroable};

// ============================================================================
// Constants
// ============================================================================

/// Target sample rate (22.05 kHz - PS1/N64 authentic)
pub const SAMPLE_RATE: u32 = 22050;

/// Samples per frame at 60 FPS (22050 / 60 = 367.5, round up to 368)
pub const SAMPLES_PER_FRAME: usize = 368;

/// Ring buffer size (frames of audio to buffer for smooth playback)
/// 4 frames = ~67ms latency at 60 FPS
const RING_BUFFER_FRAMES: usize = 4;

/// Ring buffer capacity in stereo samples
const RING_BUFFER_SIZE: usize = SAMPLES_PER_FRAME * 2 * RING_BUFFER_FRAMES;

/// Maximum number of SFX channels
pub const MAX_SFX_CHANNELS: usize = 16;

/// Maximum number of sounds that can be loaded
pub const MAX_SOUNDS: usize = 256;

// ============================================================================
// Sound Data
// ============================================================================

/// Loaded sound data (22.05 kHz 16-bit mono PCM)
#[derive(Debug, Clone)]
pub struct Sound {
    /// Raw PCM samples
    pub data: Arc<Vec<i16>>,
}

impl Sound {
    /// Create a new sound from raw PCM data
    pub fn new(data: Vec<i16>) -> Self {
        Self {
            data: Arc::new(data),
        }
    }

    /// Get the number of samples
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Check if the sound is empty
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
}

// ============================================================================
// Audio Playback State (POD - saved/restored during rollback)
// ============================================================================

/// State for a single SFX channel (20 bytes)
#[repr(C)]
#[derive(Debug, Clone, Copy, Default, Pod, Zeroable)]
pub struct ChannelState {
    /// Sound handle (0 = no sound playing)
    pub sound_handle: u32,
    /// Playhead position in samples
    pub playhead: u32,
    /// Volume (0.0 - 1.0, stored as fixed-point)
    pub volume_fixed: u16,
    /// Pan (-1.0 to 1.0, stored as signed fixed-point)
    pub pan_fixed: i16,
    /// Flags: bit 0 = looping, bit 1 = playing
    pub flags: u8,
    /// Padding for alignment
    pub _pad: [u8; 3],
}

impl ChannelState {
    /// Flag: channel is looping
    const FLAG_LOOPING: u8 = 1 << 0;
    /// Flag: channel is playing
    const FLAG_PLAYING: u8 = 1 << 1;

    /// Check if the channel is playing
    #[inline]
    pub fn is_playing(&self) -> bool {
        self.flags & Self::FLAG_PLAYING != 0
    }

    /// Check if the channel is looping
    #[inline]
    pub fn is_looping(&self) -> bool {
        self.flags & Self::FLAG_LOOPING != 0
    }

    /// Set the playing flag
    #[inline]
    pub fn set_playing(&mut self, playing: bool) {
        if playing {
            self.flags |= Self::FLAG_PLAYING;
        } else {
            self.flags &= !Self::FLAG_PLAYING;
        }
    }

    /// Set the looping flag
    #[inline]
    pub fn set_looping(&mut self, looping: bool) {
        if looping {
            self.flags |= Self::FLAG_LOOPING;
        } else {
            self.flags &= !Self::FLAG_LOOPING;
        }
    }

    /// Get volume as f32 (0.0 - 1.0)
    #[inline]
    pub fn volume(&self) -> f32 {
        self.volume_fixed as f32 / 65535.0
    }

    /// Set volume from f32 (clamped to 0.0 - 1.0)
    #[inline]
    pub fn set_volume(&mut self, volume: f32) {
        self.volume_fixed = (volume.clamp(0.0, 1.0) * 65535.0) as u16;
    }

    /// Get pan as f32 (-1.0 to 1.0)
    #[inline]
    pub fn pan(&self) -> f32 {
        self.pan_fixed as f32 / 32767.0
    }

    /// Set pan from f32 (clamped to -1.0 to 1.0)
    #[inline]
    pub fn set_pan(&mut self, pan: f32) {
        self.pan_fixed = (pan.clamp(-1.0, 1.0) * 32767.0) as i16;
    }

    /// Start playing a sound
    pub fn play(&mut self, sound_handle: u32, volume: f32, pan: f32, looping: bool) {
        self.sound_handle = sound_handle;
        self.playhead = 0;
        self.set_volume(volume);
        self.set_pan(pan);
        self.set_playing(true);
        self.set_looping(looping);
    }

    /// Stop playing
    pub fn stop(&mut self) {
        self.set_playing(false);
        self.sound_handle = 0;
        self.playhead = 0;
    }
}

/// Music channel state (separate from SFX, 20 bytes)
#[repr(C)]
#[derive(Debug, Clone, Copy, Default, Pod, Zeroable)]
pub struct MusicState {
    /// Sound handle for music (0 = no music)
    pub sound_handle: u32,
    /// Playhead position in samples
    pub playhead: u32,
    /// Volume (0.0 - 1.0, stored as fixed-point)
    pub volume_fixed: u16,
    /// Flags: bit 0 = playing (music always loops)
    pub flags: u8,
    /// Padding for alignment
    pub _pad: [u8; 5],
}

impl MusicState {
    /// Flag: music is playing
    const FLAG_PLAYING: u8 = 1 << 0;

    /// Check if music is playing
    #[inline]
    pub fn is_playing(&self) -> bool {
        self.flags & Self::FLAG_PLAYING != 0
    }

    /// Set the playing flag
    #[inline]
    pub fn set_playing(&mut self, playing: bool) {
        if playing {
            self.flags |= Self::FLAG_PLAYING;
        } else {
            self.flags &= !Self::FLAG_PLAYING;
        }
    }

    /// Get volume as f32 (0.0 - 1.0)
    #[inline]
    pub fn volume(&self) -> f32 {
        self.volume_fixed as f32 / 65535.0
    }

    /// Set volume from f32 (clamped to 0.0 - 1.0)
    #[inline]
    pub fn set_volume(&mut self, volume: f32) {
        self.volume_fixed = (volume.clamp(0.0, 1.0) * 65535.0) as u16;
    }

    /// Start playing music
    pub fn play(&mut self, sound_handle: u32, volume: f32) {
        self.sound_handle = sound_handle;
        self.playhead = 0;
        self.set_volume(volume);
        self.set_playing(true);
    }

    /// Stop music
    pub fn stop(&mut self) {
        self.set_playing(false);
        self.sound_handle = 0;
        self.playhead = 0;
    }
}

/// Complete audio playback state (340 bytes total)
///
/// This is the POD struct that gets saved/restored during GGRS rollback.
/// All fields are fixed-size and use bytemuck for zero-copy serialization.
#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct AudioPlaybackState {
    /// SFX channel states (16 channels × 20 bytes = 320 bytes)
    pub channels: [ChannelState; MAX_SFX_CHANNELS],
    /// Music channel state (20 bytes)
    pub music: MusicState,
}

impl Default for AudioPlaybackState {
    fn default() -> Self {
        Self {
            channels: [ChannelState::default(); MAX_SFX_CHANNELS],
            music: MusicState::default(),
        }
    }
}

impl AudioPlaybackState {
    /// Create a new audio playback state
    pub fn new() -> Self {
        Self::default()
    }

    /// Find an available channel for fire-and-forget sounds
    ///
    /// Returns the index of an available channel, or None if all are in use.
    pub fn find_free_channel(&self) -> Option<usize> {
        self.channels.iter().position(|c| !c.is_playing())
    }
}

// ============================================================================
// Audio Output (cpal stream + ring buffer)
// ============================================================================

/// Audio output using cpal with lock-free ring buffer
///
/// The audio callback runs in a separate thread and consumes samples from
/// the ring buffer. The game thread produces samples via `write_samples()`.
pub struct AudioOutput {
    /// The cpal output stream (must be kept alive)
    _stream: Stream,
    /// Producer side of the ring buffer (boxed to avoid self-referential struct)
    producer: Box<ringbuf::HeapProd<'static, f32>>,
    /// Detected sample rate (may differ from target)
    sample_rate: u32,
}

impl AudioOutput {
    /// Create a new audio output
    ///
    /// Attempts to find a suitable output device and configure it for
    /// stereo output at our target sample rate. Falls back to the device's
    /// default sample rate if 22050 Hz is not supported.
    pub fn new() -> anyhow::Result<Self> {
        let host = cpal::default_host();
        let device = host
            .default_output_device()
            .ok_or_else(|| anyhow::anyhow!("No audio output device found"))?;

        // Try to get a config at our target sample rate, fall back to device default
        let sample_rate = Self::find_sample_rate(&device)?;

        let config = StreamConfig {
            channels: 2,
            sample_rate: cpal::SampleRate(sample_rate),
            buffer_size: cpal::BufferSize::Default,
        };

        // Create the ring buffer
        let ring = HeapRb::<f32>::new(RING_BUFFER_SIZE);
        let (producer, mut consumer) = ring.split();

        // Create the output stream
        let stream = device.build_output_stream(
            &config,
            move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                // Fill the output buffer from the ring buffer
                let samples_read = consumer.pop_slice(data);
                // Fill any remaining samples with silence
                for sample in &mut data[samples_read..] {
                    *sample = 0.0;
                }
            },
            |err| {
                tracing::error!("Audio stream error: {}", err);
            },
            None,
        )?;

        stream.play()?;

        tracing::info!(
            "Audio output initialized: {} Hz, {} sample ring buffer",
            sample_rate,
            RING_BUFFER_SIZE
        );

        // Box the producer to avoid self-referential issues
        let producer = Box::new(producer);

        Ok(Self {
            _stream: stream,
            producer,
            sample_rate,
        })
    }

    /// Find a suitable sample rate for the device
    fn find_sample_rate(device: &Device) -> anyhow::Result<u32> {
        let supported_configs = device.supported_output_configs()?;

        // First, try to find a config that supports exactly our target sample rate
        for config in supported_configs.clone() {
            if config.channels() == 2
                && config.min_sample_rate().0 <= SAMPLE_RATE
                && config.max_sample_rate().0 >= SAMPLE_RATE
                && config.sample_format() == SampleFormat::F32
            {
                return Ok(SAMPLE_RATE);
            }
        }

        // Fall back to the device's default config
        let default_config = device.default_output_config()?;
        Ok(default_config.sample_rate().0)
    }

    /// Get the actual sample rate being used
    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    /// Write stereo samples to the ring buffer
    ///
    /// Returns the number of samples written. If the ring buffer is full,
    /// samples may be dropped (this shouldn't happen under normal operation).
    pub fn write_samples(&mut self, samples: &[f32]) -> usize {
        self.producer.push_slice(samples)
    }

    /// Get the number of samples available in the ring buffer
    pub fn available(&self) -> usize {
        self.producer.vacant_len()
    }
}

// ============================================================================
// Audio Generation
// ============================================================================

/// Generate audio samples for one frame
///
/// This function is called once per game frame and generates `SAMPLES_PER_FRAME`
/// stereo samples based on the current audio playback state.
///
/// # Arguments
/// * `state` - The current audio playback state (updated in place)
/// * `sounds` - Slice of loaded sounds (indexed by sound_handle - 1)
/// * `output` - Output buffer for stereo samples (must be at least SAMPLES_PER_FRAME * 2)
///
/// # Panning
/// Uses equal-power panning (constant loudness across the stereo field):
/// - `left_gain = cos((pan + 1) * π/4)`
/// - `right_gain = sin((pan + 1) * π/4)`
pub fn generate_audio_frame(
    state: &mut AudioPlaybackState,
    sounds: &[Option<Sound>],
    output: &mut [f32],
) {
    debug_assert!(output.len() >= SAMPLES_PER_FRAME * 2);

    // Clear the output buffer
    for sample in output.iter_mut().take(SAMPLES_PER_FRAME * 2) {
        *sample = 0.0;
    }

    // Mix all playing channels
    for channel in state.channels.iter_mut() {
        if !channel.is_playing() {
            continue;
        }

        let sound_idx = channel.sound_handle.saturating_sub(1) as usize;
        let Some(Some(sound)) = sounds.get(sound_idx) else {
            channel.stop();
            continue;
        };

        mix_channel_samples(channel, &sound.data, output, SAMPLES_PER_FRAME);
    }

    // Mix music
    if state.music.is_playing() {
        let sound_idx = state.music.sound_handle.saturating_sub(1) as usize;
        if let Some(Some(sound)) = sounds.get(sound_idx) {
            mix_music_samples(&mut state.music, &sound.data, output, SAMPLES_PER_FRAME);
        } else {
            state.music.stop();
        }
    }

    // Clamp output to [-1.0, 1.0]
    for sample in output.iter_mut().take(SAMPLES_PER_FRAME * 2) {
        *sample = sample.clamp(-1.0, 1.0);
    }
}

/// Mix samples from a single channel into the output buffer
fn mix_channel_samples(
    channel: &mut ChannelState,
    sound_data: &[i16],
    output: &mut [f32],
    num_samples: usize,
) {
    let volume = channel.volume();
    let pan = channel.pan();

    // Equal-power panning
    let pan_angle = (pan + 1.0) * std::f32::consts::FRAC_PI_4;
    let left_gain = pan_angle.cos() * volume;
    let right_gain = pan_angle.sin() * volume;

    let sound_len = sound_data.len() as u32;

    for i in 0..num_samples {
        // Check for end of sound
        if channel.playhead >= sound_len {
            if channel.is_looping() {
                channel.playhead = 0;
            } else {
                channel.stop();
                break;
            }
        }

        let sample_idx = channel.playhead as usize;
        if sample_idx >= sound_data.len() {
            break;
        }

        // Convert i16 to f32 (-1.0 to 1.0)
        let sample = sound_data[sample_idx] as f32 / 32768.0;

        // Mix into stereo output
        let out_idx = i * 2;
        output[out_idx] += sample * left_gain;
        output[out_idx + 1] += sample * right_gain;

        channel.playhead += 1;
    }
}

/// Mix music samples into the output buffer (always loops, center-panned)
fn mix_music_samples(
    music: &mut MusicState,
    sound_data: &[i16],
    output: &mut [f32],
    num_samples: usize,
) {
    let volume = music.volume();
    let sound_len = sound_data.len() as u32;

    for i in 0..num_samples {
        if music.playhead >= sound_len {
            music.playhead = 0; // Music always loops
        }

        let sample_idx = music.playhead as usize;
        let sample = sound_data[sample_idx] as f32 / 32768.0 * volume;

        // Center-panned (equal to both channels)
        let out_idx = i * 2;
        output[out_idx] += sample;
        output[out_idx + 1] += sample;

        music.playhead += 1;
    }
}

// ============================================================================
// Legacy Audio Commands (for backwards compatibility during migration)
// ============================================================================

/// Audio command (legacy - kept for migration)
#[derive(Debug, Clone)]
pub enum AudioCommand {
    PlaySound {
        sound: u32,
        volume: f32,
        pan: f32,
    },
    ChannelPlay {
        channel: u32,
        sound: u32,
        volume: f32,
        pan: f32,
        looping: bool,
    },
    ChannelSet {
        channel: u32,
        volume: f32,
        pan: f32,
    },
    ChannelStop {
        channel: u32,
    },
    MusicPlay {
        sound: u32,
        volume: f32,
    },
    MusicStop,
    MusicSetVolume {
        volume: f32,
    },
}

// ============================================================================
// ZAudio - Main Audio Backend
// ============================================================================

/// Audio backend for Emberware Z
///
/// This implements the per-frame audio generation approach:
/// - Audio playback state is part of rollback state
/// - `generate_and_submit()` is called once per frame after update
/// - Audio output uses cpal + lock-free ring buffer
pub struct ZAudio {
    /// The audio output (cpal stream)
    output: Option<AudioOutput>,
    /// Frame sample buffer (reused each frame)
    frame_buffer: Vec<f32>,
    /// Whether we're in rollback mode (mutes new sounds)
    rollback_mode: bool,
}

impl ZAudio {
    /// Create a new audio backend
    pub fn new() -> anyhow::Result<Self> {
        let output = match AudioOutput::new() {
            Ok(output) => Some(output),
            Err(e) => {
                tracing::warn!("Failed to initialize audio output: {}. Audio disabled.", e);
                None
            }
        };

        Ok(Self {
            output,
            frame_buffer: vec![0.0; SAMPLES_PER_FRAME * 2],
            rollback_mode: false,
        })
    }

    /// Generate audio for the current frame and submit to output
    ///
    /// This should be called once per frame after update() completes.
    /// Skips audio generation during rollback replay.
    pub fn generate_and_submit(
        &mut self,
        state: &mut AudioPlaybackState,
        sounds: &[Option<Sound>],
    ) {
        // Skip audio generation during rollback - state will be restored
        // and we'll generate audio once we're at the confirmed frame
        if self.rollback_mode {
            return;
        }

        // Generate samples for this frame
        generate_audio_frame(state, sounds, &mut self.frame_buffer);

        // Submit to audio output
        if let Some(output) = &mut self.output {
            output.write_samples(&self.frame_buffer);
        }
    }

    /// Check if audio is available
    pub fn is_available(&self) -> bool {
        self.output.is_some()
    }

    /// Get the sample rate
    pub fn sample_rate(&self) -> u32 {
        self.output
            .as_ref()
            .map_or(SAMPLE_RATE, |o| o.sample_rate())
    }

    /// Set rollback mode
    pub fn set_rollback_mode(&mut self, rolling_back: bool) {
        self.rollback_mode = rolling_back;
    }

    /// Check if in rollback mode
    pub fn is_rolling_back(&self) -> bool {
        self.rollback_mode
    }

    /// Process buffered audio commands (legacy - for backwards compatibility)
    ///
    /// In the new architecture, audio is generated per-frame via `generate_and_submit()`.
    /// This method exists for compatibility during migration.
    pub fn process_commands(&mut self, _commands: &[AudioCommand], _sounds: &[Option<Sound>]) {
        // Legacy - no-op in new architecture
        // Audio is now generated per-frame via AudioPlaybackState
    }
}

// Implement Audio trait
impl emberware_core::console::Audio for ZAudio {
    fn set_rollback_mode(&mut self, rolling_back: bool) {
        self.rollback_mode = rolling_back;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_channel_state_volume() {
        let mut channel = ChannelState::default();
        channel.set_volume(0.5);
        assert!((channel.volume() - 0.5).abs() < 0.001);

        channel.set_volume(0.0);
        assert!((channel.volume()).abs() < 0.001);

        channel.set_volume(1.0);
        assert!((channel.volume() - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_channel_state_pan() {
        let mut channel = ChannelState::default();
        channel.set_pan(0.0);
        assert!((channel.pan()).abs() < 0.001);

        channel.set_pan(-1.0);
        assert!((channel.pan() + 1.0).abs() < 0.001);

        channel.set_pan(1.0);
        assert!((channel.pan() - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_channel_state_flags() {
        let mut channel = ChannelState::default();
        assert!(!channel.is_playing());
        assert!(!channel.is_looping());

        channel.set_playing(true);
        assert!(channel.is_playing());
        assert!(!channel.is_looping());

        channel.set_looping(true);
        assert!(channel.is_playing());
        assert!(channel.is_looping());

        channel.stop();
        assert!(!channel.is_playing());
    }

    #[test]
    fn test_audio_playback_state_find_free_channel() {
        let mut state = AudioPlaybackState::new();

        // All channels should be free initially
        assert_eq!(state.find_free_channel(), Some(0));

        // Mark first channel as playing
        state.channels[0].set_playing(true);
        assert_eq!(state.find_free_channel(), Some(1));

        // Mark all channels as playing
        for channel in &mut state.channels {
            channel.set_playing(true);
        }
        assert_eq!(state.find_free_channel(), None);
    }

    #[test]
    fn test_audio_playback_state_is_pod() {
        // Verify the state is exactly the expected size
        assert_eq!(
            std::mem::size_of::<AudioPlaybackState>(),
            std::mem::size_of::<ChannelState>() * MAX_SFX_CHANNELS
                + std::mem::size_of::<MusicState>()
        );

        // Verify we can serialize/deserialize via bytemuck
        let state = AudioPlaybackState::new();
        let bytes: &[u8] = bytemuck::bytes_of(&state);
        let _restored: AudioPlaybackState = *bytemuck::from_bytes(bytes);
    }

    #[test]
    fn test_generate_audio_frame_silence() {
        let mut state = AudioPlaybackState::new();
        let sounds: Vec<Option<Sound>> = Vec::new();
        let mut output = vec![0.0f32; SAMPLES_PER_FRAME * 2];

        generate_audio_frame(&mut state, &sounds, &mut output);

        // Should all be silence
        for sample in &output {
            assert_eq!(*sample, 0.0);
        }
    }

    #[test]
    fn test_generate_audio_frame_with_sound() {
        let mut state = AudioPlaybackState::new();

        // Create a simple test sound (sine wave)
        let sound_data: Vec<i16> = (0..1000)
            .map(|i| ((i as f32 * 0.1).sin() * 16000.0) as i16)
            .collect();
        let sounds = vec![Some(Sound::new(sound_data))];

        // Play on channel 0
        state.channels[0].play(1, 1.0, 0.0, false);

        let mut output = vec![0.0f32; SAMPLES_PER_FRAME * 2];
        generate_audio_frame(&mut state, &sounds, &mut output);

        // Should have non-zero output
        let has_audio = output.iter().any(|s| *s != 0.0);
        assert!(has_audio);
    }

    #[test]
    fn test_channel_state_size() {
        assert_eq!(std::mem::size_of::<ChannelState>(), 16);
    }

    #[test]
    fn test_music_state_size() {
        assert_eq!(std::mem::size_of::<MusicState>(), 16);
    }
}
