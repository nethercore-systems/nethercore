//! Audio conversion utilities for XM sample extraction
//!
//! This module provides audio conversion functions for processing extracted
//! XM samples into Nethercore's standard audio format (22050 Hz, mono, i16).

/// Target sample rate for Nethercore audio
pub const TARGET_SAMPLE_RATE: u32 = 22050;

/// Resample audio to 22050 Hz using linear interpolation
///
/// This function uses simple linear interpolation to resample audio data.
/// While not as high quality as sinc resampling, it's fast and produces
/// acceptable results for game audio.
///
/// # Arguments
/// * `samples` - Input sample data (i16 PCM)
/// * `source_rate` - Original sample rate in Hz
///
/// # Returns
/// * Resampled audio at 22050 Hz
pub fn resample_to_22050(samples: &[i16], source_rate: u32) -> Vec<i16> {
    if samples.is_empty() {
        return Vec::new();
    }

    // If already at target rate, return clone
    if source_rate == TARGET_SAMPLE_RATE {
        return samples.to_vec();
    }

    // Calculate resampling ratio
    let ratio = source_rate as f64 / TARGET_SAMPLE_RATE as f64;

    // Calculate output length
    let output_len = (samples.len() as f64 / ratio).ceil() as usize;
    let mut output = Vec::with_capacity(output_len);

    // Linear interpolation resampling
    for i in 0..output_len {
        let src_pos = i as f64 * ratio;
        let src_idx = src_pos.floor() as usize;
        let frac = src_pos - src_idx as f64;

        let sample = if src_idx + 1 < samples.len() {
            // Interpolate between two samples
            let s1 = samples[src_idx] as f64;
            let s2 = samples[src_idx + 1] as f64;
            (s1 + (s2 - s1) * frac).round() as i16
        } else {
            // Last sample, no interpolation
            samples[src_idx]
        };

        output.push(sample);
    }

    output
}

/// Convert stereo audio to mono by averaging channels
///
/// Assumes interleaved stereo data (L, R, L, R, ...)
///
/// # Arguments
/// * `samples` - Interleaved stereo sample data
///
/// # Returns
/// * Mono audio (averaged from both channels)
pub fn stereo_to_mono(samples: &[i16]) -> Vec<i16> {
    if samples.is_empty() {
        return Vec::new();
    }

    let mut mono = Vec::with_capacity(samples.len() / 2);

    for chunk in samples.chunks(2) {
        if chunk.len() == 2 {
            // Average left and right channels
            let left = chunk[0] as i32;
            let right = chunk[1] as i32;
            let avg = ((left + right) / 2) as i16;
            mono.push(avg);
        } else {
            // Odd number of samples, just use the last one
            mono.push(chunk[0]);
        }
    }

    mono
}

/// Full conversion pipeline for XM samples
///
/// Converts an XM sample to Nethercore's standard format:
/// - Resample to 22050 Hz
/// - Ensure mono (XM samples are already mono, so this is a no-op)
///
/// # Arguments
/// * `sample` - Extracted XM sample
///
/// # Returns
/// * Converted audio data ready for ROM packing
pub fn convert_xm_sample(sample: &nether_xm::ExtractedSample) -> Vec<i16> {
    // XM samples are already mono, so we only need to resample
    resample_to_22050(&sample.data, sample.sample_rate)
}

/// Apply loop to sample if specified
///
/// This doesn't modify the sample data, but calculates the loop points
/// at the new sample rate after resampling.
///
/// # Arguments
/// * `original_rate` - Original sample rate
/// * `loop_start` - Loop start in original samples
/// * `loop_length` - Loop length in original samples
///
/// # Returns
/// * `(new_loop_start, new_loop_length)` at 22050 Hz
pub fn convert_loop_points(
    original_rate: u32,
    loop_start: u32,
    loop_length: u32,
) -> (u32, u32) {
    if original_rate == TARGET_SAMPLE_RATE {
        return (loop_start, loop_length);
    }

    let ratio = TARGET_SAMPLE_RATE as f64 / original_rate as f64;

    let new_start = (loop_start as f64 * ratio).round() as u32;
    let new_length = (loop_length as f64 * ratio).round() as u32;

    (new_start, new_length)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resample_identity() {
        // Resampling at same rate should return identical data
        let samples = vec![100i16, 200, 300, 400, 500];
        let resampled = resample_to_22050(&samples, TARGET_SAMPLE_RATE);
        assert_eq!(samples, resampled);
    }

    #[test]
    fn test_resample_downsample() {
        // Downsampling should reduce number of samples
        let samples = vec![0i16, 100, 200, 300, 400, 500];
        let resampled = resample_to_22050(&samples, 44100);

        // Should be roughly half the samples
        assert!(resampled.len() < samples.len());
        assert!(resampled.len() >= samples.len() / 2 - 1);
    }

    #[test]
    fn test_resample_upsample() {
        // Upsampling should increase number of samples
        let samples = vec![0i16, 100, 200, 300];
        let resampled = resample_to_22050(&samples, 11025);

        // Should be roughly double the samples
        assert!(resampled.len() > samples.len());
        assert!(resampled.len() <= samples.len() * 2 + 1);
    }

    #[test]
    fn test_resample_empty() {
        let samples: Vec<i16> = Vec::new();
        let resampled = resample_to_22050(&samples, 44100);
        assert!(resampled.is_empty());
    }

    #[test]
    fn test_stereo_to_mono() {
        // Interleaved stereo [L, R, L, R, ...]
        let stereo = vec![100i16, 200, 300, 400, 500, 600];
        let mono = stereo_to_mono(&stereo);

        assert_eq!(mono.len(), 3);
        assert_eq!(mono[0], 150); // (100 + 200) / 2
        assert_eq!(mono[1], 350); // (300 + 400) / 2
        assert_eq!(mono[2], 550); // (500 + 600) / 2
    }

    #[test]
    fn test_stereo_to_mono_odd() {
        // Odd number of samples (incomplete last pair)
        let stereo = vec![100i16, 200, 300];
        let mono = stereo_to_mono(&stereo);

        assert_eq!(mono.len(), 2);
        assert_eq!(mono[0], 150); // (100 + 200) / 2
        assert_eq!(mono[1], 300); // Last sample as-is
    }

    #[test]
    fn test_stereo_to_mono_empty() {
        let stereo: Vec<i16> = Vec::new();
        let mono = stereo_to_mono(&stereo);
        assert!(mono.is_empty());
    }

    #[test]
    fn test_convert_loop_points_identity() {
        let (start, length) = convert_loop_points(TARGET_SAMPLE_RATE, 100, 500);
        assert_eq!(start, 100);
        assert_eq!(length, 500);
    }

    #[test]
    fn test_convert_loop_points_downsample() {
        // 44100 Hz -> 22050 Hz (half the rate)
        let (start, length) = convert_loop_points(44100, 1000, 2000);

        // Should be roughly half
        assert!((start as i32 - 500).abs() <= 1);
        assert!((length as i32 - 1000).abs() <= 1);
    }

    #[test]
    fn test_convert_loop_points_upsample() {
        // 11025 Hz -> 22050 Hz (double the rate)
        let (start, length) = convert_loop_points(11025, 500, 1000);

        // Should be roughly double
        assert!((start as i32 - 1000).abs() <= 2);
        assert!((length as i32 - 2000).abs() <= 2);
    }

    #[test]
    fn test_linear_interpolation() {
        // Test that interpolation is working correctly
        let samples = vec![0i16, 1000, 2000];

        // Upsample by 2x
        let resampled = resample_to_22050(&samples, 11025);

        // Should have interpolated values between the originals
        // First sample should be 0
        assert_eq!(resampled[0], 0);

        // Should have approximately 6 samples total (3 * 2)
        assert!(resampled.len() >= 5 && resampled.len() <= 7);

        // Values should be increasing
        for i in 1..resampled.len() {
            assert!(resampled[i] >= resampled[i - 1]);
        }
    }
}
