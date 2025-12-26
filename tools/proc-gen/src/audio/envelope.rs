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

        // Calculate where release starts
        let release_start = if total_samples > release_samples {
            total_samples - release_samples
        } else {
            0
        };

        // Calculate where sustain phase is
        let sustain_start = attack_samples + decay_samples;

        for (i, sample) in samples.iter_mut().enumerate() {
            let amplitude = if i < attack_samples {
                // Attack phase: ramp up from 0 to 1
                i as f32 / attack_samples.max(1) as f32
            } else if i < sustain_start && i < release_start {
                // Decay phase: ramp down from 1 to sustain
                let decay_progress = (i - attack_samples) as f32 / decay_samples.max(1) as f32;
                1.0 - decay_progress * (1.0 - self.sustain)
            } else if i < release_start {
                // Sustain phase: hold at sustain level
                self.sustain
            } else {
                // Release phase: ramp down from current level to 0
                let release_progress = (i - release_start) as f32 / release_samples.max(1) as f32;
                // Start from sustain level (or wherever we are)
                let start_level = if release_start >= sustain_start {
                    self.sustain
                } else if release_start >= attack_samples {
                    // In decay phase
                    let decay_progress =
                        (release_start - attack_samples) as f32 / decay_samples.max(1) as f32;
                    1.0 - decay_progress * (1.0 - self.sustain)
                } else {
                    // Still in attack
                    release_start as f32 / attack_samples.max(1) as f32
                };
                start_level * (1.0 - release_progress)
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
}
