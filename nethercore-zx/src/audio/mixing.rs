//! Audio mixing utilities: channel mixing, panning, and soft clipping

use super::Sound;
use crate::state::ChannelState;
use tracing::warn;

/// Mix a single channel, returning the sample value and advancing the playhead
///
/// # Precondition
/// `channel.sound` must be non-zero (callers must check before calling)
pub fn mix_channel(
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
pub fn apply_pan(sample: f32, pan: f32, volume: f32) -> (f32, f32) {
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
pub fn soft_clip(x: f32) -> f32 {
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
