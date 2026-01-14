//! IT resonant low-pass filter implementation

use super::TrackerChannel;

impl TrackerChannel {
    /// Apply resonant low-pass filter to sample (IT only)
    ///
    /// Uses Direct Form II transposed biquad filter.
    pub fn apply_filter(&mut self, input: f32) -> f32 {
        // If filter is wide open (cutoff = 1.0) or disabled, bypass
        if self.filter_cutoff >= 1.0 {
            return input;
        }

        // Update filter coefficients if dirty
        if self.filter_dirty {
            self.update_filter_coefficients(22050.0); // ZX sample rate
            self.filter_dirty = false;
        }

        // Direct Form II transposed biquad
        let output = self.filter_b0 * input + self.filter_z1;
        self.filter_z1 = self.filter_b1 * input - self.filter_a1 * output + self.filter_z2;
        self.filter_z2 = self.filter_b2 * input - self.filter_a2 * output;
        output
    }

    /// Recalculate filter coefficients from cutoff and resonance (IT only)
    ///
    /// IT formula: freq = 110 * 2^(cutoff/24 + 0.25)
    /// where cutoff is normalized 0.0-1.0 (from IT's 0-127 range)
    pub fn update_filter_coefficients(&mut self, sample_rate: f32) {
        // Convert normalized cutoff (0.0-1.0) to frequency
        // IT uses: freq = 110 * 2^((cutoff * 127)/24 + 0.25)
        let cutoff_it = self.filter_cutoff * 127.0;
        let freq = 110.0 * 2.0_f32.powf(cutoff_it / 24.0 + 0.25);

        // Clamp frequency to Nyquist
        let freq = freq.min(sample_rate / 2.0 - 1.0);

        let omega = 2.0 * std::f32::consts::PI * freq / sample_rate;
        let sin_omega = omega.sin();
        let cos_omega = omega.cos();

        // Q factor from resonance (higher resonance = lower Q denominator)
        // IT resonance 0-127 mapped to 0.0-1.0
        let q_denom = 1.0 + self.filter_resonance * 10.0;
        let alpha = sin_omega / (2.0 * q_denom);

        // Low-pass filter coefficients
        let b0 = (1.0 - cos_omega) / 2.0;
        let b1 = 1.0 - cos_omega;
        let b2 = (1.0 - cos_omega) / 2.0;
        let a0 = 1.0 + alpha;
        let a1 = -2.0 * cos_omega;
        let a2 = 1.0 - alpha;

        // Normalize by a0
        self.filter_b0 = b0 / a0;
        self.filter_b1 = b1 / a0;
        self.filter_b2 = b2 / a0;
        self.filter_a1 = a1 / a0;
        self.filter_a2 = a2 / a0;
    }
}
