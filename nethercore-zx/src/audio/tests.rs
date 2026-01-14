//! Audio module tests

use super::*;
use super::mixing::{apply_pan, soft_clip};
use crate::state::{AudioPlaybackState, ChannelState, TrackerState};
use crate::tracker::TrackerEngine;

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
