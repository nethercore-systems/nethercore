//! Audio frame generation and position advancement

use super::Sound;
use super::mixing::{apply_pan, mix_channel, soft_clip};
use super::output::SOURCE_SAMPLE_RATE;
use crate::state::{AudioPlaybackState, ChannelState, TrackerState, tracker_flags};
use crate::tracker::TrackerEngine;

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
        advance_channel_position(
            &mut playback_state.music,
            sounds,
            resample_ratio,
            samples_per_frame,
        );
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
