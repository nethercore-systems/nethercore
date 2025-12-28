//! ADSR envelope generator
//!
//! Provides Attack-Decay-Sustain-Release envelope shaping for audio synthesis.

/// ADSR Envelope parameters
///
/// Controls the amplitude shape of a sound over time:
/// - Attack: Time to reach peak amplitude (0 to 1)
/// - Decay: Time to fall from peak to sustain level
/// - Sustain: Amplitude level held during the sustain phase (0.0 to 1.0)
/// - Release: Time to fade from sustain to silence
///
/// All times are in seconds.
#[derive(Debug, Clone, Copy)]
pub struct Envelope {
    /// Attack time in seconds
    pub attack: f32,
    /// Decay time in seconds
    pub decay: f32,
    /// Sustain level (0.0 to 1.0)
    pub sustain: f32,
    /// Release time in seconds
    pub release: f32,
}

impl Default for Envelope {
    fn default() -> Self {
        Self {
            attack: 0.01,
            decay: 0.1,
            sustain: 0.7,
            release: 0.2,
        }
    }
}

impl Envelope {
    /// Create a new envelope with custom parameters
    pub fn new(attack: f32, decay: f32, sustain: f32, release: f32) -> Self {
        Self {
            attack,
            decay,
            sustain: sustain.clamp(0.0, 1.0),
            release,
        }
    }

    /// Quick pluck/stab envelope - fast attack, no sustain
    pub fn pluck() -> Self {
        Self {
            attack: 0.001,
            decay: 0.15,
            sustain: 0.0,
            release: 0.1,
        }
    }

    /// Pad/ambient envelope - slow attack and release
    pub fn pad() -> Self {
        Self {
            attack: 0.3,
            decay: 0.2,
            sustain: 0.8,
            release: 0.5,
        }
    }

    /// Percussive hit - instant attack, quick decay
    pub fn hit() -> Self {
        Self {
            attack: 0.001,
            decay: 0.05,
            sustain: 0.0,
            release: 0.05,
        }
    }

    /// Laser/zap sound - instant attack, medium decay
    pub fn zap() -> Self {
        Self {
            attack: 0.001,
            decay: 0.2,
            sustain: 0.0,
            release: 0.05,
        }
    }

    /// Explosion/boom - fast attack, long decay
    pub fn explosion() -> Self {
        Self {
            attack: 0.005,
            decay: 0.5,
            sustain: 0.0,
            release: 0.3,
        }
    }

    /// UI click/blip - very short
    pub fn click() -> Self {
        Self {
            attack: 0.001,
            decay: 0.02,
            sustain: 0.0,
            release: 0.01,
        }
    }

    /// Apply envelope to samples
    ///
    /// The envelope is applied based on the total sound duration.
    /// The sustain phase fills the time between attack+decay and the release phase.
    /// The release phase overlays on top of the base envelope, fading from the
    /// current level to zero.
    ///
    /// # Arguments
    /// * `samples` - Audio samples to shape
    /// * `sample_rate` - Sample rate in Hz
    pub fn apply(&self, samples: &mut [f32], sample_rate: u32) {
        if samples.is_empty() {
            return;
        }

        let total_samples = samples.len();

        // Convert times to sample counts
        let attack_samples = (self.attack * sample_rate as f32) as usize;
        let decay_samples = (self.decay * sample_rate as f32) as usize;
        let release_samples = (self.release * sample_rate as f32) as usize;

        // Calculate where release starts (release overlays on top of base envelope)
        let release_start = total_samples.saturating_sub(release_samples);

        // Calculate where sustain phase begins
        let sustain_start = attack_samples + decay_samples;

        for (i, sample) in samples.iter_mut().enumerate() {
            // First calculate base envelope level (attack/decay/sustain)
            let base_level = if i < attack_samples {
                // Attack phase: ramp up from 0 to 1
                i as f32 / attack_samples.max(1) as f32
            } else if i < sustain_start {
                // Decay phase: ramp down from 1 to sustain
                let decay_progress = (i - attack_samples) as f32 / decay_samples.max(1) as f32;
                1.0 - decay_progress * (1.0 - self.sustain)
            } else {
                // Sustain phase: hold at sustain level
                self.sustain
            };

            // Then apply release fade if we're in the release region
            // Release fades from current base_level to 0
            let amplitude = if i >= release_start && release_samples > 0 {
                let release_progress = (i - release_start) as f32 / release_samples as f32;
                base_level * (1.0 - release_progress)
            } else {
                base_level
            };

            *sample *= amplitude;
        }
    }

    /// Generate envelope curve as samples
    ///
    /// Useful for visualization or debugging.
    ///
    /// # Arguments
    /// * `duration` - Total duration in seconds
    /// * `sample_rate` - Sample rate in Hz
    pub fn generate(&self, duration: f32, sample_rate: u32) -> Vec<f32> {
        let num_samples = (duration * sample_rate as f32) as usize;
        let mut samples = vec![1.0; num_samples];
        self.apply(&mut samples, sample_rate);
        samples
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_SAMPLE_RATE: u32 = 22050;

    #[test]
    fn test_envelope_default() {
        let env = Envelope::default();
        assert!(env.attack > 0.0);
        assert!(env.decay > 0.0);
        assert!(env.sustain >= 0.0 && env.sustain <= 1.0);
        assert!(env.release > 0.0);
    }

    #[test]
    fn test_envelope_presets() {
        // Just ensure presets don't panic
        let _ = Envelope::pluck();
        let _ = Envelope::pad();
        let _ = Envelope::hit();
        let _ = Envelope::zap();
        let _ = Envelope::explosion();
        let _ = Envelope::click();
    }

    #[test]
    fn test_envelope_apply() {
        let env = Envelope::new(0.01, 0.05, 0.5, 0.1);
        let mut samples = vec![1.0; (0.2 * TEST_SAMPLE_RATE as f32) as usize];
        env.apply(&mut samples, TEST_SAMPLE_RATE);

        // First sample should be near 0 (attack phase start)
        assert!(samples[0] < 0.1);

        // Last sample should be near 0 (release phase end)
        assert!(samples.last().unwrap().abs() < 0.1);

        // All samples should be in valid range
        assert!(samples.iter().all(|&s| s >= 0.0 && s <= 1.0));
    }

    #[test]
    fn test_envelope_generate() {
        let env = Envelope::pluck();
        let curve = env.generate(0.1, TEST_SAMPLE_RATE);

        assert!(!curve.is_empty());
        // All values should be in 0-1 range
        assert!(curve.iter().all(|&v| v >= 0.0 && v <= 1.0));
    }

    #[test]
    fn test_envelope_sustain_clamp() {
        let env = Envelope::new(0.01, 0.01, 1.5, 0.01);
        assert_eq!(env.sustain, 1.0);

        let env = Envelope::new(0.01, 0.01, -0.5, 0.01);
        assert_eq!(env.sustain, 0.0);
    }

    #[test]
    fn test_envelope_empty_samples() {
        let env = Envelope::default();
        let mut samples: Vec<f32> = vec![];
        env.apply(&mut samples, TEST_SAMPLE_RATE);
        assert!(samples.is_empty());
    }

    #[test]
    fn test_envelope_short_sound_with_long_release() {
        // Regression test: when sound duration <= release time, envelope should
        // still produce audible output (not near-zero amplitude throughout)
        let env = Envelope::pluck(); // release = 0.1s
        let duration = 0.1; // Same as release time
        let mut samples = vec![1.0; (duration * TEST_SAMPLE_RATE as f32) as usize];
        env.apply(&mut samples, TEST_SAMPLE_RATE);

        // The peak amplitude should be significant (not just clicks)
        let max_amplitude = samples.iter().cloned().fold(0.0f32, f32::max);
        assert!(
            max_amplitude > 0.5,
            "Short sound should have significant amplitude, got {}",
            max_amplitude
        );

        // Attack should still ramp up at the beginning
        assert!(samples[0] < 0.1, "First sample should be near 0 (attack start)");

        // Samples after attack should have meaningful amplitude
        let attack_samples = (0.001 * TEST_SAMPLE_RATE as f32) as usize;
        if samples.len() > attack_samples + 10 {
            assert!(
                samples[attack_samples + 10] > 0.3,
                "Post-attack samples should have amplitude"
            );
        }
    }
}
