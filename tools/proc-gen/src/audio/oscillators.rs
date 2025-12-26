//! Audio oscillators and waveform generators
//!
//! Provides basic waveform generation for audio synthesis.

use std::f32::consts::PI;

/// Waveform types for audio synthesis
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Waveform {
    /// Pure sine wave - smooth, fundamental tone
    Sine,
    /// Square wave - hollow, retro sound (odd harmonics)
    Square,
    /// Sawtooth wave - bright, buzzy sound (all harmonics)
    Saw,
    /// Triangle wave - softer than square (odd harmonics, quieter)
    Triangle,
}

/// Generate oscillator samples for a given waveform
///
/// # Arguments
/// * `waveform` - The type of waveform to generate
/// * `frequency` - Frequency in Hz
/// * `duration` - Duration in seconds
/// * `sample_rate` - Sample rate in Hz
///
/// # Returns
/// Vector of samples in -1.0 to 1.0 range
pub fn oscillator(waveform: Waveform, frequency: f32, duration: f32, sample_rate: u32) -> Vec<f32> {
    let num_samples = (duration * sample_rate as f32) as usize;
    let mut samples = Vec::with_capacity(num_samples);

    let omega = 2.0 * PI * frequency / sample_rate as f32;

    for i in 0..num_samples {
        let phase = omega * i as f32;
        let sample = match waveform {
            Waveform::Sine => phase.sin(),
            Waveform::Square => {
                if phase.sin() >= 0.0 {
                    1.0
                } else {
                    -1.0
                }
            }
            Waveform::Saw => {
                // Normalized sawtooth: goes from -1 to 1 over one period
                let t = (phase / (2.0 * PI)).fract();
                2.0 * t - 1.0
            }
            Waveform::Triangle => {
                // Triangle wave from sawtooth
                let t = (phase / (2.0 * PI)).fract();
                4.0 * (t - 0.5).abs() - 1.0
            }
        };
        samples.push(sample);
    }

    samples
}

/// Generate white noise samples
///
/// White noise has equal energy at all frequencies.
///
/// # Arguments
/// * `duration` - Duration in seconds
/// * `sample_rate` - Sample rate in Hz
/// * `seed` - Random seed for reproducibility
///
/// # Returns
/// Vector of samples in -1.0 to 1.0 range
pub fn noise(duration: f32, sample_rate: u32, seed: u64) -> Vec<f32> {
    let num_samples = (duration * sample_rate as f32) as usize;
    let mut samples = Vec::with_capacity(num_samples);

    // Simple LCG random number generator for reproducibility
    let mut state = seed;
    for _ in 0..num_samples {
        // LCG parameters (from Numerical Recipes)
        state = state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        // Convert to -1.0 to 1.0 range
        let sample = (state as f32 / u64::MAX as f32) * 2.0 - 1.0;
        samples.push(sample);
    }

    samples
}

/// Generate pink noise samples
///
/// Pink noise has equal energy per octave (1/f spectrum).
/// Uses the Voss-McCartney algorithm.
///
/// # Arguments
/// * `duration` - Duration in seconds
/// * `sample_rate` - Sample rate in Hz
/// * `seed` - Random seed for reproducibility
///
/// # Returns
/// Vector of samples in approximately -1.0 to 1.0 range
pub fn pink_noise(duration: f32, sample_rate: u32, seed: u64) -> Vec<f32> {
    let num_samples = (duration * sample_rate as f32) as usize;
    let mut samples = Vec::with_capacity(num_samples);

    // Voss-McCartney algorithm with 8 octaves
    const NUM_OCTAVES: usize = 8;
    let mut octave_values = [0.0f32; NUM_OCTAVES];
    let mut state = seed;

    // Helper to get next random value
    let mut next_random = || {
        state = state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        (state as f32 / u64::MAX as f32) * 2.0 - 1.0
    };

    // Initialize octave values
    for value in &mut octave_values {
        *value = next_random();
    }

    for i in 0..num_samples {
        // Update octaves based on counter bits
        for (oct, value) in octave_values.iter_mut().enumerate() {
            if i & (1 << oct) == 0 {
                *value = next_random();
            }
        }

        // Sum all octaves
        let sum: f32 = octave_values.iter().sum();
        // Normalize by number of octaves
        samples.push(sum / NUM_OCTAVES as f32);
    }

    samples
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_SAMPLE_RATE: u32 = 22050;

    #[test]
    fn test_sine_wave() {
        let samples = oscillator(Waveform::Sine, 440.0, 0.01, TEST_SAMPLE_RATE);
        assert!(!samples.is_empty());
        // All samples should be in -1 to 1 range
        assert!(samples.iter().all(|&s| s >= -1.0 && s <= 1.0));
    }

    #[test]
    fn test_square_wave() {
        let samples = oscillator(Waveform::Square, 440.0, 0.01, TEST_SAMPLE_RATE);
        assert!(!samples.is_empty());
        // Square wave should only have values of -1 or 1
        assert!(samples.iter().all(|&s| s == -1.0 || s == 1.0));
    }

    #[test]
    fn test_saw_wave() {
        let samples = oscillator(Waveform::Saw, 440.0, 0.01, TEST_SAMPLE_RATE);
        assert!(!samples.is_empty());
        assert!(samples.iter().all(|&s| s >= -1.0 && s <= 1.0));
    }

    #[test]
    fn test_triangle_wave() {
        let samples = oscillator(Waveform::Triangle, 440.0, 0.01, TEST_SAMPLE_RATE);
        assert!(!samples.is_empty());
        assert!(samples.iter().all(|&s| s >= -1.0 && s <= 1.0));
    }

    #[test]
    fn test_white_noise() {
        let samples = noise(0.1, TEST_SAMPLE_RATE, 12345);
        assert!(!samples.is_empty());
        assert!(samples.iter().all(|&s| s >= -1.0 && s <= 1.0));
    }

    #[test]
    fn test_pink_noise() {
        let samples = pink_noise(0.1, TEST_SAMPLE_RATE, 12345);
        assert!(!samples.is_empty());
        // Pink noise should be roughly normalized but may exceed slightly
        assert!(samples.iter().all(|&s| s >= -2.0 && s <= 2.0));
    }

    #[test]
    fn test_noise_reproducibility() {
        let samples1 = noise(0.01, TEST_SAMPLE_RATE, 42);
        let samples2 = noise(0.01, TEST_SAMPLE_RATE, 42);
        assert_eq!(samples1, samples2);
    }

    #[test]
    fn test_sample_count() {
        let duration = 0.5;
        let samples = oscillator(Waveform::Sine, 440.0, duration, TEST_SAMPLE_RATE);
        let expected = (duration * TEST_SAMPLE_RATE as f32) as usize;
        assert_eq!(samples.len(), expected);
    }
}
