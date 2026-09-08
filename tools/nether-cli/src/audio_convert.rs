//! Audio conversion utilities for XM sample extraction
//!
//! This module provides audio conversion functions for processing extracted
//! XM samples into Nethercore's standard audio format (22050 Hz, i16).

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
/// - Preserve mono/stereo channels and authored loop boundaries
///
/// # Arguments
/// * `sample` - Extracted XM sample
///
/// # Returns
/// * Converted audio data ready for ROM packing
pub fn convert_xm_sample(sample: &nether_xm::ExtractedSample) -> Vec<i16> {
    let channels = if sample.is_stereo { 2 } else { 1 };
    let mut output = if sample.is_stereo {
        let left: Vec<_> = sample.data.iter().step_by(2).copied().collect();
        let right: Vec<_> = sample.data.iter().skip(1).step_by(2).copied().collect();
        resample_to_22050(&left, sample.sample_rate)
            .into_iter()
            .zip(resample_to_22050(&right, sample.sample_rate))
            .flat_map(|(l, r)| [l, r])
            .collect::<Vec<_>>()
    } else {
        resample_to_22050(&sample.data, sample.sample_rate)
    };
    let start = sample.loop_start as usize;
    let length = sample.loop_length as usize;
    if matches!(sample.loop_type, 1 | 2)
        && length > 0
        && start + length <= sample.data.len() / channels
    {
        // Match convert_loop_points: independently round the start and length.
        let ratio = TARGET_SAMPLE_RATE as f64 / sample.sample_rate as f64;
        let target_start = (start as f64 * ratio).round() as usize;
        let target_length = (length as f64 * ratio).round() as usize;
        if target_length > 0 {
            // Independent rounding can put a valid loop one frame beyond the PCM.
            output.resize(
                output.len().max((target_start + target_length) * channels),
                0,
            );
            // Resample the authored loop region, wrapping or holding its endpoint.
            // Absolute source positions can read the post-loop tail and bias short loops.
            for frame in 0..target_length {
                let position = frame as f64 * length as f64 / target_length as f64;
                let index = position.floor() as usize;
                let fraction = position - index as f64;
                for lane in 0..channels {
                    let a = sample.data[(start + index) * channels + lane] as f64;
                    let next = if sample.loop_type == 1 {
                        (index + 1) % length
                    } else {
                        (index + 1).min(length - 1)
                    };
                    let b = sample.data[(start + next) * channels + lane] as f64;
                    output[(target_start + frame) * channels + lane] =
                        (a + (b - a) * fraction).round() as i16;
                }
            }
        }
    }
    output
}

/// Full conversion pipeline for IT samples
///
/// Converts an IT sample to Nethercore's standard format:
/// - Convert stereo to mono if needed
/// - Resample to 22050 Hz
///
/// # Arguments
/// * `sample` - Extracted IT sample
///
/// # Returns
/// * Converted audio data ready for ROM packing
pub fn convert_it_sample(sample: &nether_it::ExtractedSample) -> Vec<i16> {
    // Convert stereo to mono if needed
    let mono_data = if sample.is_stereo {
        stereo_to_mono(&sample.data)
    } else {
        sample.data.clone()
    };

    // Resample to 22050 Hz
    resample_to_22050(&mono_data, sample.sample_rate)
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

    #[test]
    fn xm_stereo_resampling_keeps_channels_independent() {
        let mut sample = nether_xm::ExtractedSample {
            instrument_index: 0,
            sample_index: 0,
            name: String::new(),
            volume: 64,
            pan: 128,
            finetune: 0,
            relative_note: 0,
            sample_rate: 11025,
            bit_depth: 16,
            is_stereo: true,
            loop_start: 0,
            loop_length: 0,
            loop_type: 0,
            data: vec![1000, -2000, 2000, -3000],
        };
        let converted = convert_xm_sample(&sample);
        assert_eq!(converted.len() % 2, 0);
        assert!(converted
            .chunks_exact(2)
            .all(|frame| frame[0] > 0 && frame[1] < 0));
        sample.sample_rate = 8363;
        sample.loop_start = 4;
        sample.loop_length = 8;
        sample.loop_type = 1;
        sample.data = (0..64).flat_map(|i| [i * 256, -i * 256]).collect();
        let stereo = convert_xm_sample(&sample);
        // Metadata rounds the start to 11 frames and length to 21 frames.
        assert_eq!(stereo[22], 4 * 256, "loop start lost its phase");
        assert_eq!(
            stereo[62], 1707,
            "last interpolation must wrap from source 11 to 4"
        );
        sample.is_stereo = false;
        sample.data = sample.data.into_iter().step_by(2).collect();
        assert_eq!(
            convert_xm_sample(&sample),
            stereo.into_iter().step_by(2).collect::<Vec<_>>()
        );
        sample.sample_rate = 44100;
        sample.loop_start = 1;
        sample.loop_length = 1;
        sample.data = vec![1000, 2000];
        assert_eq!(convert_xm_sample(&sample), vec![1000, 2000]);
        sample.is_stereo = true;
        sample.data = vec![1000, -1000, 2000, -2000];
        assert_eq!(convert_xm_sample(&sample), sample.data);
    }
}
