//! Audio filters
//!
//! Provides low-pass and high-pass filters for audio processing.

use std::f32::consts::PI;

/// Filter configuration
#[derive(Debug, Clone, Copy)]
pub struct FilterConfig {
    /// Cutoff frequency in Hz
    pub cutoff: f32,
    /// Resonance (Q factor) - higher values create a peak at cutoff
    pub resonance: f32,
}

impl Default for FilterConfig {
    fn default() -> Self {
        Self {
            cutoff: 1000.0,
            resonance: 0.707, // Butterworth (no resonance peak)
        }
    }
}

impl FilterConfig {
    /// Create a new filter config
    pub fn new(cutoff: f32, resonance: f32) -> Self {
        Self {
            cutoff: cutoff.max(20.0),
            resonance: resonance.clamp(0.1, 10.0),
        }
    }

    /// Create with just cutoff frequency (default resonance)
    pub fn cutoff(cutoff: f32) -> Self {
        Self::new(cutoff, 0.707)
    }
}

/// Simple one-pole low-pass filter state
struct OnePoleFilter {
    z1: f32,
}

impl OnePoleFilter {
    fn new() -> Self {
        Self { z1: 0.0 }
    }

    fn process(&mut self, input: f32, alpha: f32) -> f32 {
        self.z1 = input * alpha + self.z1 * (1.0 - alpha);
        self.z1
    }
}

/// Apply low-pass filter to samples (in-place)
///
/// Attenuates frequencies above the cutoff.
/// Uses a biquad filter implementation for quality.
///
/// # Arguments
/// * `samples` - Audio samples to filter (modified in-place)
/// * `cutoff` - Cutoff frequency in Hz
/// * `sample_rate` - Sample rate in Hz
pub fn low_pass(samples: &mut [f32], cutoff: f32, sample_rate: u32) {
    low_pass_resonant(samples, FilterConfig::cutoff(cutoff), sample_rate);
}

/// Apply low-pass filter with resonance control
///
/// # Arguments
/// * `samples` - Audio samples to filter (modified in-place)
/// * `config` - Filter configuration (cutoff and resonance)
/// * `sample_rate` - Sample rate in Hz
pub fn low_pass_resonant(samples: &mut [f32], config: FilterConfig, sample_rate: u32) {
    if samples.is_empty() {
        return;
    }

    // Clamp cutoff to valid range
    let cutoff = config.cutoff.clamp(20.0, sample_rate as f32 * 0.49);

    // Biquad coefficients for low-pass
    let omega = 2.0 * PI * cutoff / sample_rate as f32;
    let sin_omega = omega.sin();
    let cos_omega = omega.cos();
    let alpha = sin_omega / (2.0 * config.resonance);

    let b0 = (1.0 - cos_omega) / 2.0;
    let b1 = 1.0 - cos_omega;
    let b2 = (1.0 - cos_omega) / 2.0;
    let a0 = 1.0 + alpha;
    let a1 = -2.0 * cos_omega;
    let a2 = 1.0 - alpha;

    // Normalize coefficients
    let b0 = b0 / a0;
    let b1 = b1 / a0;
    let b2 = b2 / a0;
    let a1 = a1 / a0;
    let a2 = a2 / a0;

    // Filter state
    let mut x1 = 0.0f32;
    let mut x2 = 0.0f32;
    let mut y1 = 0.0f32;
    let mut y2 = 0.0f32;

    for sample in samples.iter_mut() {
        let x0 = *sample;
        let y0 = b0 * x0 + b1 * x1 + b2 * x2 - a1 * y1 - a2 * y2;

        // Shift delay line
        x2 = x1;
        x1 = x0;
        y2 = y1;
        y1 = y0;

        *sample = y0;
    }
}

/// Apply high-pass filter to samples (in-place)
///
/// Attenuates frequencies below the cutoff.
/// Uses a biquad filter implementation.
///
/// # Arguments
/// * `samples` - Audio samples to filter (modified in-place)
/// * `cutoff` - Cutoff frequency in Hz
/// * `sample_rate` - Sample rate in Hz
pub fn high_pass(samples: &mut [f32], cutoff: f32, sample_rate: u32) {
    high_pass_resonant(samples, FilterConfig::cutoff(cutoff), sample_rate);
}

/// Apply high-pass filter with resonance control
///
/// # Arguments
/// * `samples` - Audio samples to filter (modified in-place)
/// * `config` - Filter configuration (cutoff and resonance)
/// * `sample_rate` - Sample rate in Hz
pub fn high_pass_resonant(samples: &mut [f32], config: FilterConfig, sample_rate: u32) {
    if samples.is_empty() {
        return;
    }

    // Clamp cutoff to valid range
    let cutoff = config.cutoff.clamp(20.0, sample_rate as f32 * 0.49);

    // Biquad coefficients for high-pass
    let omega = 2.0 * PI * cutoff / sample_rate as f32;
    let sin_omega = omega.sin();
    let cos_omega = omega.cos();
    let alpha = sin_omega / (2.0 * config.resonance);

    let b0 = (1.0 + cos_omega) / 2.0;
    let b1 = -(1.0 + cos_omega);
    let b2 = (1.0 + cos_omega) / 2.0;
    let a0 = 1.0 + alpha;
    let a1 = -2.0 * cos_omega;
    let a2 = 1.0 - alpha;

    // Normalize coefficients
    let b0 = b0 / a0;
    let b1 = b1 / a0;
    let b2 = b2 / a0;
    let a1 = a1 / a0;
    let a2 = a2 / a0;

    // Filter state
    let mut x1 = 0.0f32;
    let mut x2 = 0.0f32;
    let mut y1 = 0.0f32;
    let mut y2 = 0.0f32;

    for sample in samples.iter_mut() {
        let x0 = *sample;
        let y0 = b0 * x0 + b1 * x1 + b2 * x2 - a1 * y1 - a2 * y2;

        // Shift delay line
        x2 = x1;
        x1 = x0;
        y2 = y1;
        y1 = y0;

        *sample = y0;
    }
}

/// Simple one-pole low-pass filter (less CPU, lower quality)
///
/// Good for simple smoothing or when performance is critical.
///
/// # Arguments
/// * `samples` - Audio samples to filter (modified in-place)
/// * `cutoff` - Cutoff frequency in Hz
/// * `sample_rate` - Sample rate in Hz
pub fn low_pass_simple(samples: &mut [f32], cutoff: f32, sample_rate: u32) {
    if samples.is_empty() {
        return;
    }

    let cutoff = cutoff.clamp(20.0, sample_rate as f32 * 0.49);
    let rc = 1.0 / (2.0 * PI * cutoff);
    let dt = 1.0 / sample_rate as f32;
    let alpha = dt / (rc + dt);

    let mut filter = OnePoleFilter::new();
    for sample in samples.iter_mut() {
        *sample = filter.process(*sample, alpha);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_SAMPLE_RATE: u32 = 22050;

    #[test]
    fn test_low_pass_basic() {
        // Create a signal with low and high frequency content
        let mut samples: Vec<f32> = (0..1000)
            .map(|i| {
                let t = i as f32 / TEST_SAMPLE_RATE as f32;
                // 100 Hz + 5000 Hz
                (2.0 * PI * 100.0 * t).sin() + (2.0 * PI * 5000.0 * t).sin()
            })
            .collect();

        let original_energy: f32 = samples.iter().map(|s| s * s).sum();

        // Apply low-pass at 500 Hz
        low_pass(&mut samples, 500.0, TEST_SAMPLE_RATE);

        let filtered_energy: f32 = samples.iter().map(|s| s * s).sum();

        // Energy should be reduced (high frequencies removed)
        assert!(filtered_energy < original_energy);
    }

    #[test]
    fn test_high_pass_basic() {
        // Create a signal with low and high frequency content
        let mut samples: Vec<f32> = (0..1000)
            .map(|i| {
                let t = i as f32 / TEST_SAMPLE_RATE as f32;
                // 100 Hz + 5000 Hz
                (2.0 * PI * 100.0 * t).sin() + (2.0 * PI * 5000.0 * t).sin()
            })
            .collect();

        let original_energy: f32 = samples.iter().map(|s| s * s).sum();

        // Apply high-pass at 2000 Hz
        high_pass(&mut samples, 2000.0, TEST_SAMPLE_RATE);

        let filtered_energy: f32 = samples.iter().map(|s| s * s).sum();

        // Energy should be reduced (low frequencies removed)
        assert!(filtered_energy < original_energy);
    }

    #[test]
    fn test_filter_empty() {
        let mut samples: Vec<f32> = vec![];
        low_pass(&mut samples, 1000.0, TEST_SAMPLE_RATE);
        high_pass(&mut samples, 1000.0, TEST_SAMPLE_RATE);
        assert!(samples.is_empty());
    }

    #[test]
    fn test_filter_config() {
        let config = FilterConfig::new(2000.0, 2.0);
        assert_eq!(config.cutoff, 2000.0);
        assert_eq!(config.resonance, 2.0);

        let config = FilterConfig::cutoff(1500.0);
        assert_eq!(config.cutoff, 1500.0);
        assert!((config.resonance - 0.707).abs() < 0.001);
    }

    #[test]
    fn test_filter_config_clamp() {
        let config = FilterConfig::new(10.0, 0.05);
        assert_eq!(config.cutoff, 20.0);
        assert_eq!(config.resonance, 0.1);

        let config = FilterConfig::new(1000.0, 15.0);
        assert_eq!(config.resonance, 10.0);
    }

    #[test]
    fn test_low_pass_simple() {
        let mut samples: Vec<f32> = (0..500)
            .map(|i| {
                let t = i as f32 / TEST_SAMPLE_RATE as f32;
                (2.0 * PI * 5000.0 * t).sin()
            })
            .collect();

        let original_energy: f32 = samples.iter().map(|s| s * s).sum();

        low_pass_simple(&mut samples, 500.0, TEST_SAMPLE_RATE);

        let filtered_energy: f32 = samples.iter().map(|s| s * s).sum();

        // High frequency signal should be significantly attenuated
        assert!(filtered_energy < original_energy * 0.5);
    }
}
