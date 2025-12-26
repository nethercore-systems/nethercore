//! Audio export and utility functions
//!
//! Provides PCM conversion and WAV export functionality.

#[cfg(feature = "wav-export")]
use std::path::Path;

/// Convert f32 samples (-1.0 to 1.0) to PCM i16
///
/// This is the format required by the ZX console (22.05kHz, mono, 16-bit).
///
/// # Arguments
/// * `samples` - Audio samples in -1.0 to 1.0 range
///
/// # Returns
/// Vector of i16 PCM samples
pub fn to_pcm_i16(samples: &[f32]) -> Vec<i16> {
    samples
        .iter()
        .map(|&s| {
            // Clamp to -1.0 to 1.0 range
            let clamped = s.clamp(-1.0, 1.0);
            // Convert to i16 range
            (clamped * i16::MAX as f32) as i16
        })
        .collect()
}

/// Convert PCM i16 samples to f32 (-1.0 to 1.0)
///
/// Useful for loading existing audio for processing.
///
/// # Arguments
/// * `samples` - PCM i16 samples
///
/// # Returns
/// Vector of f32 samples in -1.0 to 1.0 range
pub fn from_pcm_i16(samples: &[i16]) -> Vec<f32> {
    samples
        .iter()
        .map(|&s| s as f32 / i16::MAX as f32)
        .collect()
}

/// Mix multiple audio signals together
///
/// Each signal is multiplied by its volume before mixing.
/// The result is NOT normalized - use `normalize()` if needed.
///
/// # Arguments
/// * `signals` - Slice of (samples, volume) tuples
///
/// # Returns
/// Mixed audio samples
///
/// # Example
/// ```
/// use proc_gen::audio::*;
///
/// let synth = Synth::new(SAMPLE_RATE);
/// let tone = synth.tone(Waveform::Sine, 440.0, 0.5, Envelope::default());
/// let noise = synth.noise_burst(0.5, Envelope::hit());
///
/// // Mix at 70% tone, 30% noise
/// let mixed = mix(&[(&tone, 0.7), (&noise, 0.3)]);
/// ```
pub fn mix(signals: &[(&[f32], f32)]) -> Vec<f32> {
    if signals.is_empty() {
        return Vec::new();
    }

    // Find the longest signal
    let max_len = signals.iter().map(|(s, _)| s.len()).max().unwrap_or(0);

    let mut result = vec![0.0f32; max_len];

    for (samples, volume) in signals {
        for (i, &sample) in samples.iter().enumerate() {
            result[i] += sample * volume;
        }
    }

    result
}

/// Normalize audio to fit within -1.0 to 1.0 range
///
/// Scales all samples so the peak amplitude is 1.0.
/// Does nothing if the audio is silent.
///
/// # Arguments
/// * `samples` - Audio samples to normalize (modified in-place)
pub fn normalize(samples: &mut [f32]) {
    if samples.is_empty() {
        return;
    }

    let max_amplitude = samples
        .iter()
        .map(|s| s.abs())
        .fold(0.0f32, |a, b| a.max(b));

    if max_amplitude > 0.0 && max_amplitude != 1.0 {
        let scale = 1.0 / max_amplitude;
        for sample in samples.iter_mut() {
            *sample *= scale;
        }
    }
}

/// Normalize audio with target peak amplitude
///
/// # Arguments
/// * `samples` - Audio samples to normalize (modified in-place)
/// * `target` - Target peak amplitude (e.g., 0.9 for headroom)
pub fn normalize_to(samples: &mut [f32], target: f32) {
    normalize(samples);
    for sample in samples.iter_mut() {
        *sample *= target;
    }
}

/// Concatenate multiple audio signals
///
/// # Arguments
/// * `signals` - Signals to concatenate in order
///
/// # Returns
/// Concatenated audio samples
pub fn concat(signals: &[&[f32]]) -> Vec<f32> {
    let total_len: usize = signals.iter().map(|s| s.len()).sum();
    let mut result = Vec::with_capacity(total_len);
    for signal in signals {
        result.extend_from_slice(signal);
    }
    result
}

/// Add silence (zero samples) of specified duration
///
/// # Arguments
/// * `duration` - Duration in seconds
/// * `sample_rate` - Sample rate in Hz
///
/// # Returns
/// Vector of zero samples
pub fn silence(duration: f32, sample_rate: u32) -> Vec<f32> {
    let num_samples = (duration * sample_rate as f32) as usize;
    vec![0.0; num_samples]
}

/// Write audio samples to a WAV file (for debugging)
///
/// Requires the `wav-export` feature.
///
/// # Arguments
/// * `samples` - PCM i16 samples
/// * `sample_rate` - Sample rate in Hz
/// * `path` - Output file path
#[cfg(feature = "wav-export")]
pub fn write_wav(samples: &[i16], sample_rate: u32, path: &Path) -> std::io::Result<()> {
    use hound::{SampleFormat, WavSpec, WavWriter};

    let spec = WavSpec {
        channels: 1,
        sample_rate,
        bits_per_sample: 16,
        sample_format: SampleFormat::Int,
    };

    let mut writer = WavWriter::create(path, spec)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

    for &sample in samples {
        writer
            .write_sample(sample)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
    }

    writer
        .finalize()
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

    Ok(())
}

/// Write f32 samples directly to WAV (convenience function)
///
/// Converts to i16 PCM and writes to file.
///
/// Requires the `wav-export` feature.
#[cfg(feature = "wav-export")]
pub fn write_wav_f32(samples: &[f32], sample_rate: u32, path: &Path) -> std::io::Result<()> {
    let pcm = to_pcm_i16(samples);
    write_wav(&pcm, sample_rate, path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_pcm_i16() {
        let samples = vec![0.0, 0.5, 1.0, -1.0, -0.5];
        let pcm = to_pcm_i16(&samples);

        assert_eq!(pcm.len(), 5);
        assert_eq!(pcm[0], 0);
        assert!(pcm[1] > 0);
        assert_eq!(pcm[2], i16::MAX);
        assert_eq!(pcm[3], -i16::MAX); // Note: -32767, not -32768
        assert!(pcm[4] < 0);
    }

    #[test]
    fn test_to_pcm_i16_clamp() {
        let samples = vec![2.0, -2.0]; // Out of range
        let pcm = to_pcm_i16(&samples);

        assert_eq!(pcm[0], i16::MAX);
        assert_eq!(pcm[1], -i16::MAX);
    }

    #[test]
    fn test_from_pcm_i16() {
        let pcm = vec![0, i16::MAX, -i16::MAX];
        let samples = from_pcm_i16(&pcm);

        assert!((samples[0] - 0.0).abs() < 0.001);
        assert!((samples[1] - 1.0).abs() < 0.001);
        assert!((samples[2] - (-1.0)).abs() < 0.001);
    }

    #[test]
    fn test_mix() {
        let signal1 = vec![1.0, 1.0, 1.0];
        let signal2 = vec![0.5, 0.5];

        let mixed = mix(&[(&signal1, 0.5), (&signal2, 0.5)]);

        assert_eq!(mixed.len(), 3);
        assert!((mixed[0] - 0.75).abs() < 0.001); // 0.5 + 0.25
        assert!((mixed[1] - 0.75).abs() < 0.001);
        assert!((mixed[2] - 0.5).abs() < 0.001); // Only signal1
    }

    #[test]
    fn test_mix_empty() {
        let mixed = mix(&[]);
        assert!(mixed.is_empty());
    }

    #[test]
    fn test_normalize() {
        let mut samples = vec![0.5, -0.25, 0.25];
        normalize(&mut samples);

        assert!((samples[0] - 1.0).abs() < 0.001);
        assert!((samples[1] - (-0.5)).abs() < 0.001);
        assert!((samples[2] - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_normalize_silent() {
        let mut samples = vec![0.0, 0.0, 0.0];
        normalize(&mut samples);
        assert!(samples.iter().all(|&s| s == 0.0));
    }

    #[test]
    fn test_normalize_to() {
        let mut samples = vec![1.0, -0.5];
        normalize_to(&mut samples, 0.8);

        assert!((samples[0] - 0.8).abs() < 0.001);
        assert!((samples[1] - (-0.4)).abs() < 0.001);
    }

    #[test]
    fn test_concat() {
        let a = vec![1.0, 2.0];
        let b = vec![3.0, 4.0, 5.0];
        let result = concat(&[&a, &b]);

        assert_eq!(result, vec![1.0, 2.0, 3.0, 4.0, 5.0]);
    }

    #[test]
    fn test_silence() {
        let samples = silence(0.1, 22050);
        let expected = (0.1 * 22050.0) as usize;
        assert_eq!(samples.len(), expected);
        assert!(samples.iter().all(|&s| s == 0.0));
    }
}
