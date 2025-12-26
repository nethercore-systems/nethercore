//! High-level synthesizer API
//!
//! Provides a convenient interface for generating common sound effects.

use super::envelope::Envelope;
use super::filters::{high_pass, low_pass};
use super::oscillators::{noise, oscillator, Waveform};

/// High-level synthesizer for generating sound effects
///
/// # Example
/// ```
/// use proc_gen::audio::*;
///
/// let mut synth = Synth::new(SAMPLE_RATE);
///
/// // Generate a laser sound
/// let laser = synth.sweep(Waveform::Saw, 1000.0, 200.0, 0.2, Envelope::zap());
///
/// // Generate an explosion
/// let explosion = synth.noise_burst(0.5, Envelope::explosion());
/// ```
pub struct Synth {
    sample_rate: u32,
}

impl Synth {
    /// Create a new synthesizer with the given sample rate
    pub fn new(sample_rate: u32) -> Self {
        Self { sample_rate }
    }

    /// Generate a simple tone with envelope
    ///
    /// # Arguments
    /// * `waveform` - Type of oscillator
    /// * `frequency` - Frequency in Hz
    /// * `duration` - Duration in seconds
    /// * `envelope` - ADSR envelope to apply
    pub fn tone(
        &self,
        waveform: Waveform,
        frequency: f32,
        duration: f32,
        envelope: Envelope,
    ) -> Vec<f32> {
        let mut samples = oscillator(waveform, frequency, duration, self.sample_rate);
        envelope.apply(&mut samples, self.sample_rate);
        samples
    }

    /// Generate a frequency sweep
    ///
    /// Creates a sound that glides from one frequency to another.
    /// Great for laser sounds, power-ups, and sci-fi effects.
    ///
    /// # Arguments
    /// * `waveform` - Type of oscillator
    /// * `start_freq` - Starting frequency in Hz
    /// * `end_freq` - Ending frequency in Hz
    /// * `duration` - Duration in seconds
    /// * `envelope` - ADSR envelope to apply
    pub fn sweep(
        &self,
        waveform: Waveform,
        start_freq: f32,
        end_freq: f32,
        duration: f32,
        envelope: Envelope,
    ) -> Vec<f32> {
        let num_samples = (duration * self.sample_rate as f32) as usize;
        let mut samples = Vec::with_capacity(num_samples);

        let mut phase = 0.0f32;

        for i in 0..num_samples {
            let t = i as f32 / num_samples as f32;
            // Exponential interpolation for more natural pitch sweep
            let freq = start_freq * (end_freq / start_freq).powf(t);
            let omega = 2.0 * std::f32::consts::PI * freq / self.sample_rate as f32;

            phase += omega;
            if phase > 2.0 * std::f32::consts::PI {
                phase -= 2.0 * std::f32::consts::PI;
            }

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
                    let t = (phase / (2.0 * std::f32::consts::PI)).fract();
                    2.0 * t - 1.0
                }
                Waveform::Triangle => {
                    let t = (phase / (2.0 * std::f32::consts::PI)).fract();
                    4.0 * (t - 0.5).abs() - 1.0
                }
            };
            samples.push(sample);
        }

        envelope.apply(&mut samples, self.sample_rate);
        samples
    }

    /// Generate a noise burst with envelope
    ///
    /// Great for explosions, impacts, and percussive effects.
    ///
    /// # Arguments
    /// * `duration` - Duration in seconds
    /// * `envelope` - ADSR envelope to apply
    pub fn noise_burst(&self, duration: f32, envelope: Envelope) -> Vec<f32> {
        self.noise_burst_with_seed(duration, envelope, 0)
    }

    /// Generate a noise burst with specific seed
    pub fn noise_burst_with_seed(&self, duration: f32, envelope: Envelope, seed: u64) -> Vec<f32> {
        let mut samples = noise(duration, self.sample_rate, seed);
        envelope.apply(&mut samples, self.sample_rate);
        samples
    }

    /// Generate filtered noise (for wind, fire, etc.)
    ///
    /// # Arguments
    /// * `duration` - Duration in seconds
    /// * `low_cutoff` - Low frequency cutoff (high-pass) in Hz, or None
    /// * `high_cutoff` - High frequency cutoff (low-pass) in Hz, or None
    /// * `envelope` - ADSR envelope to apply
    /// * `seed` - Random seed
    pub fn filtered_noise(
        &self,
        duration: f32,
        low_cutoff: Option<f32>,
        high_cutoff: Option<f32>,
        envelope: Envelope,
        seed: u64,
    ) -> Vec<f32> {
        let mut samples = noise(duration, self.sample_rate, seed);

        if let Some(cutoff) = high_cutoff {
            low_pass(&mut samples, cutoff, self.sample_rate);
        }
        if let Some(cutoff) = low_cutoff {
            high_pass(&mut samples, cutoff, self.sample_rate);
        }

        envelope.apply(&mut samples, self.sample_rate);
        samples
    }

    /// Generate a coin/pickup sound
    ///
    /// Quick upward sweep with a pleasant tone.
    pub fn coin(&self) -> Vec<f32> {
        self.sweep(Waveform::Square, 600.0, 1200.0, 0.1, Envelope::pluck())
    }

    /// Generate a jump sound
    ///
    /// Quick upward sweep.
    pub fn jump(&self) -> Vec<f32> {
        self.sweep(Waveform::Square, 200.0, 500.0, 0.15, Envelope::pluck())
    }

    /// Generate a laser/shoot sound
    ///
    /// High to low frequency sweep.
    pub fn laser(&self) -> Vec<f32> {
        self.sweep(Waveform::Saw, 800.0, 200.0, 0.15, Envelope::zap())
    }

    /// Generate an explosion sound
    ///
    /// Low-passed noise with long decay.
    pub fn explosion(&self) -> Vec<f32> {
        self.filtered_noise(0.6, None, Some(800.0), Envelope::explosion(), 42)
    }

    /// Generate a hit/damage sound
    ///
    /// Short noise burst.
    pub fn hit(&self) -> Vec<f32> {
        self.noise_burst(0.1, Envelope::hit())
    }

    /// Generate a UI click sound
    ///
    /// Very short blip.
    pub fn click(&self) -> Vec<f32> {
        self.tone(Waveform::Sine, 800.0, 0.03, Envelope::click())
    }

    /// Generate a power-up sound
    ///
    /// Rising arpeggio effect.
    pub fn powerup(&self) -> Vec<f32> {
        let note1 = self.tone(Waveform::Square, 440.0, 0.1, Envelope::pluck());
        let note2 = self.tone(Waveform::Square, 554.0, 0.1, Envelope::pluck());
        let note3 = self.tone(Waveform::Square, 659.0, 0.15, Envelope::pluck());

        // Concatenate notes with small gaps
        let gap_samples = (0.02 * self.sample_rate as f32) as usize;
        let mut result = Vec::with_capacity(note1.len() + note2.len() + note3.len() + gap_samples * 2);

        result.extend_from_slice(&note1);
        result.extend(std::iter::repeat(0.0).take(gap_samples));
        result.extend_from_slice(&note2);
        result.extend(std::iter::repeat(0.0).take(gap_samples));
        result.extend_from_slice(&note3);

        result
    }

    /// Generate a death/game-over sound
    ///
    /// Descending arpeggio with longer notes.
    pub fn death(&self) -> Vec<f32> {
        let note1 = self.tone(Waveform::Square, 440.0, 0.15, Envelope::default());
        let note2 = self.tone(Waveform::Square, 349.0, 0.15, Envelope::default());
        let note3 = self.tone(Waveform::Square, 294.0, 0.3, Envelope::default());

        let gap_samples = (0.05 * self.sample_rate as f32) as usize;
        let mut result = Vec::with_capacity(note1.len() + note2.len() + note3.len() + gap_samples * 2);

        result.extend_from_slice(&note1);
        result.extend(std::iter::repeat(0.0).take(gap_samples));
        result.extend_from_slice(&note2);
        result.extend(std::iter::repeat(0.0).take(gap_samples));
        result.extend_from_slice(&note3);

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audio::SAMPLE_RATE;

    #[test]
    fn test_synth_tone() {
        let synth = Synth::new(SAMPLE_RATE);
        let tone = synth.tone(Waveform::Sine, 440.0, 0.1, Envelope::default());
        assert!(!tone.is_empty());
        let expected_samples = (0.1 * SAMPLE_RATE as f32) as usize;
        assert_eq!(tone.len(), expected_samples);
    }

    #[test]
    fn test_synth_sweep() {
        let synth = Synth::new(SAMPLE_RATE);
        let sweep = synth.sweep(Waveform::Saw, 200.0, 800.0, 0.2, Envelope::zap());
        assert!(!sweep.is_empty());
    }

    #[test]
    fn test_synth_noise_burst() {
        let synth = Synth::new(SAMPLE_RATE);
        let noise = synth.noise_burst(0.1, Envelope::hit());
        assert!(!noise.is_empty());
    }

    #[test]
    fn test_synth_filtered_noise() {
        let synth = Synth::new(SAMPLE_RATE);
        let noise = synth.filtered_noise(0.2, Some(200.0), Some(2000.0), Envelope::default(), 123);
        assert!(!noise.is_empty());
    }

    #[test]
    fn test_synth_presets() {
        let synth = Synth::new(SAMPLE_RATE);

        // All presets should generate non-empty audio
        assert!(!synth.coin().is_empty());
        assert!(!synth.jump().is_empty());
        assert!(!synth.laser().is_empty());
        assert!(!synth.explosion().is_empty());
        assert!(!synth.hit().is_empty());
        assert!(!synth.click().is_empty());
        assert!(!synth.powerup().is_empty());
        assert!(!synth.death().is_empty());
    }
}
