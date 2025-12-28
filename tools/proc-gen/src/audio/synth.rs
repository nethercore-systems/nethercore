//! High-level synthesizer API
//!
//! Provides a convenient interface for generating high-quality sound effects
//! using professional sound design techniques: layering, filtering, harmonics,
//! and sophisticated envelopes.

use super::envelope::Envelope;
use super::export::{mix, normalize_to};
use super::filters::{high_pass, low_pass, low_pass_resonant, FilterConfig};
use super::oscillators::{noise, oscillator, pink_noise, Waveform};
use std::f32::consts::PI;

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

    /// Generate a detuned unison tone (multiple oscillators slightly detuned)
    ///
    /// Creates a richer, fuller sound than a single oscillator.
    ///
    /// # Arguments
    /// * `waveform` - Type of oscillator
    /// * `frequency` - Center frequency in Hz
    /// * `detune_cents` - Detune amount in cents (100 cents = 1 semitone)
    /// * `voices` - Number of voices (2-7 recommended)
    /// * `duration` - Duration in seconds
    /// * `envelope` - ADSR envelope to apply
    pub fn unison_tone(
        &self,
        waveform: Waveform,
        frequency: f32,
        detune_cents: f32,
        voices: usize,
        duration: f32,
        envelope: Envelope,
    ) -> Vec<f32> {
        let voices = voices.max(1);
        let detune_ratio = 2.0f32.powf(detune_cents / 1200.0);

        let mut layers: Vec<(Vec<f32>, f32)> = Vec::new();
        let volume_per_voice = 1.0 / (voices as f32).sqrt();

        for i in 0..voices {
            // Spread voices across the detune range
            let t = if voices > 1 {
                (i as f32 / (voices - 1) as f32) * 2.0 - 1.0 // -1 to 1
            } else {
                0.0
            };
            let freq = frequency * detune_ratio.powf(t);
            let osc = oscillator(waveform, freq, duration, self.sample_rate);
            layers.push((osc, volume_per_voice));
        }

        let refs: Vec<(&[f32], f32)> = layers.iter().map(|(s, v)| (s.as_slice(), *v)).collect();
        let mut mixed = mix(&refs);
        envelope.apply(&mut mixed, self.sample_rate);
        mixed
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
            let omega = 2.0 * PI * freq / self.sample_rate as f32;

            phase += omega;
            if phase > 2.0 * PI {
                phase -= 2.0 * PI;
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
                    let t = (phase / (2.0 * PI)).fract();
                    2.0 * t - 1.0
                }
                Waveform::Triangle => {
                    let t = (phase / (2.0 * PI)).fract();
                    4.0 * (t - 0.5).abs() - 1.0
                }
            };
            samples.push(sample);
        }

        envelope.apply(&mut samples, self.sample_rate);
        samples
    }

    /// Generate a sweep with filter modulation
    ///
    /// Applies a sweeping low-pass filter for more dynamic sound.
    fn sweep_filtered(
        &self,
        waveform: Waveform,
        start_freq: f32,
        end_freq: f32,
        duration: f32,
        envelope: Envelope,
        filter_start: f32,
        filter_end: f32,
        resonance: f32,
    ) -> Vec<f32> {
        let num_samples = (duration * self.sample_rate as f32) as usize;
        let mut samples = Vec::with_capacity(num_samples);
        let mut phase = 0.0f32;

        // Generate the sweep
        for i in 0..num_samples {
            let t = i as f32 / num_samples as f32;
            let freq = start_freq * (end_freq / start_freq).powf(t);
            let omega = 2.0 * PI * freq / self.sample_rate as f32;

            phase += omega;
            if phase > 2.0 * PI {
                phase -= 2.0 * PI;
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
                Waveform::Saw => 2.0 * (phase / (2.0 * PI)).fract() - 1.0,
                Waveform::Triangle => 4.0 * ((phase / (2.0 * PI)).fract() - 0.5).abs() - 1.0,
            };
            samples.push(sample);
        }

        // Apply time-varying filter by processing in chunks
        let chunk_size = (self.sample_rate as usize / 100).max(1); // 10ms chunks
        for (chunk_idx, chunk) in samples.chunks_mut(chunk_size).enumerate() {
            let t = (chunk_idx * chunk_size) as f32 / num_samples as f32;
            let cutoff = filter_start * (filter_end / filter_start).powf(t);
            low_pass_resonant(chunk, FilterConfig::new(cutoff, resonance), self.sample_rate);
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

    // ========================================================================
    // HIGH-QUALITY PRESET SOUNDS
    // ========================================================================

    /// Generate a coin/pickup sound
    ///
    /// Rich, shimmering sound with harmonics and a pleasant tail.
    /// Layers multiple detuned oscillators with filtered harmonics.
    pub fn coin(&self) -> Vec<f32> {
        let duration = 0.25;

        // Main body: detuned triangle waves for warmth
        let body = self.unison_tone(
            Waveform::Triangle,
            880.0, // A5
            8.0,   // slight detune for shimmer
            3,
            duration,
            Envelope::new(0.001, 0.08, 0.3, 0.15),
        );

        // Harmonic layer: higher octave sine for sparkle
        let mut sparkle = oscillator(Waveform::Sine, 1760.0, duration, self.sample_rate);
        Envelope::new(0.001, 0.05, 0.0, 0.1).apply(&mut sparkle, self.sample_rate);

        // Attack transient: short noise burst for definition
        let mut transient = noise(0.02, self.sample_rate, 42);
        high_pass(&mut transient, 4000.0, self.sample_rate);
        Envelope::new(0.0, 0.015, 0.0, 0.005).apply(&mut transient, self.sample_rate);
        // Pad to full duration
        transient.resize((duration * self.sample_rate as f32) as usize, 0.0);

        // Rising pitch sweep for "bling" effect
        let bling = self.sweep(
            Waveform::Sine,
            660.0,
            1320.0,
            0.08,
            Envelope::new(0.001, 0.06, 0.0, 0.02),
        );
        let mut bling_padded = bling;
        bling_padded.resize((duration * self.sample_rate as f32) as usize, 0.0);

        // Mix layers
        let mut result = mix(&[
            (&body, 0.5),
            (&sparkle, 0.25),
            (&transient, 0.15),
            (&bling_padded, 0.3),
        ]);

        // Gentle high-shelf boost for clarity
        low_pass(&mut result, 8000.0, self.sample_rate);
        normalize_to(&mut result, 0.85);

        result
    }

    /// Generate a jump sound
    ///
    /// Punchy, satisfying jump with sub-bass thump and bright attack.
    pub fn jump(&self) -> Vec<f32> {
        let duration = 0.18;

        // Main pitch sweep: quick upward glide
        let mut main_sweep = self.sweep(
            Waveform::Triangle,
            180.0,
            420.0,
            0.12,
            Envelope::new(0.001, 0.08, 0.0, 0.04),
        );
        main_sweep.resize((duration * self.sample_rate as f32) as usize, 0.0);

        // Sub-bass thump for weight
        let mut sub = oscillator(Waveform::Sine, 80.0, 0.1, self.sample_rate);
        Envelope::new(0.001, 0.06, 0.0, 0.04).apply(&mut sub, self.sample_rate);
        sub.resize((duration * self.sample_rate as f32) as usize, 0.0);

        // Bright attack layer
        let mut attack = self.sweep(
            Waveform::Saw,
            400.0,
            800.0,
            0.05,
            Envelope::new(0.0, 0.03, 0.0, 0.02),
        );
        low_pass(&mut attack, 3000.0, self.sample_rate);
        attack.resize((duration * self.sample_rate as f32) as usize, 0.0);

        // Air/breath layer
        let mut air = noise(0.08, self.sample_rate, 123);
        high_pass(&mut air, 2000.0, self.sample_rate);
        low_pass(&mut air, 6000.0, self.sample_rate);
        Envelope::new(0.001, 0.05, 0.0, 0.03).apply(&mut air, self.sample_rate);
        air.resize((duration * self.sample_rate as f32) as usize, 0.0);

        let mut result = mix(&[
            (&main_sweep, 0.5),
            (&sub, 0.35),
            (&attack, 0.25),
            (&air, 0.15),
        ]);

        normalize_to(&mut result, 0.85);
        result
    }

    /// Generate a laser/shoot sound
    ///
    /// Aggressive, punchy laser with rich harmonics and filter sweep.
    pub fn laser(&self) -> Vec<f32> {
        let duration = 0.2;

        // Main laser sweep with filter
        let main = self.sweep_filtered(
            Waveform::Saw,
            1200.0,
            150.0,
            duration,
            Envelope::new(0.0, 0.15, 0.0, 0.05),
            8000.0,
            800.0,
            2.5, // resonance for that "pew" quality
        );

        // Second detuned layer for thickness
        let mut layer2 = self.sweep(
            Waveform::Square,
            1180.0,
            145.0,
            duration,
            Envelope::new(0.0, 0.14, 0.0, 0.05),
        );
        low_pass(&mut layer2, 4000.0, self.sample_rate);

        // High frequency sizzle
        let mut sizzle = noise(0.08, self.sample_rate, 777);
        high_pass(&mut sizzle, 3000.0, self.sample_rate);
        Envelope::new(0.0, 0.05, 0.0, 0.03).apply(&mut sizzle, self.sample_rate);
        sizzle.resize((duration * self.sample_rate as f32) as usize, 0.0);

        // Sub impact for punch
        let mut sub = oscillator(Waveform::Sine, 60.0, 0.08, self.sample_rate);
        Envelope::new(0.0, 0.05, 0.0, 0.03).apply(&mut sub, self.sample_rate);
        sub.resize((duration * self.sample_rate as f32) as usize, 0.0);

        let mut result = mix(&[
            (&main, 0.45),
            (&layer2, 0.3),
            (&sizzle, 0.15),
            (&sub, 0.25),
        ]);

        normalize_to(&mut result, 0.9);
        result
    }

    /// Generate an explosion sound
    ///
    /// Deep, impactful explosion with layered noise bands and sub-rumble.
    pub fn explosion(&self) -> Vec<f32> {
        let duration = 0.8;

        // Sub-bass rumble
        let mut sub_rumble = pink_noise(duration, self.sample_rate, 42);
        low_pass(&mut sub_rumble, 80.0, self.sample_rate);
        Envelope::new(0.005, 0.6, 0.0, 0.2).apply(&mut sub_rumble, self.sample_rate);

        // Low-mid body
        let mut body = noise(duration, self.sample_rate, 123);
        low_pass(&mut body, 400.0, self.sample_rate);
        high_pass(&mut body, 60.0, self.sample_rate);
        Envelope::new(0.003, 0.4, 0.1, 0.3).apply(&mut body, self.sample_rate);

        // Mid crackle
        let mut crackle = noise(0.5, self.sample_rate, 456);
        low_pass(&mut crackle, 2000.0, self.sample_rate);
        high_pass(&mut crackle, 300.0, self.sample_rate);
        Envelope::new(0.001, 0.2, 0.0, 0.3).apply(&mut crackle, self.sample_rate);
        crackle.resize((duration * self.sample_rate as f32) as usize, 0.0);

        // High sizzle/debris
        let mut debris = noise(0.4, self.sample_rate, 789);
        high_pass(&mut debris, 2000.0, self.sample_rate);
        low_pass(&mut debris, 8000.0, self.sample_rate);
        Envelope::new(0.0, 0.15, 0.0, 0.25).apply(&mut debris, self.sample_rate);
        debris.resize((duration * self.sample_rate as f32) as usize, 0.0);

        // Initial impact transient
        let mut impact = noise(0.03, self.sample_rate, 999);
        Envelope::new(0.0, 0.02, 0.0, 0.01).apply(&mut impact, self.sample_rate);
        impact.resize((duration * self.sample_rate as f32) as usize, 0.0);

        // Tonal thump
        let mut thump = oscillator(Waveform::Sine, 50.0, 0.15, self.sample_rate);
        Envelope::new(0.001, 0.1, 0.0, 0.05).apply(&mut thump, self.sample_rate);
        thump.resize((duration * self.sample_rate as f32) as usize, 0.0);

        let mut result = mix(&[
            (&sub_rumble, 0.5),
            (&body, 0.4),
            (&crackle, 0.3),
            (&debris, 0.2),
            (&impact, 0.6),
            (&thump, 0.4),
        ]);

        normalize_to(&mut result, 0.9);
        result
    }

    /// Generate a hit/damage sound
    ///
    /// Punchy impact with body and high-frequency crack.
    pub fn hit(&self) -> Vec<f32> {
        let duration = 0.15;

        // Tonal thump
        let mut thump = self.sweep(
            Waveform::Sine,
            200.0,
            80.0,
            0.08,
            Envelope::new(0.0, 0.05, 0.0, 0.03),
        );
        thump.resize((duration * self.sample_rate as f32) as usize, 0.0);

        // Mid-range body
        let mut body = noise(0.1, self.sample_rate, 111);
        low_pass(&mut body, 1500.0, self.sample_rate);
        high_pass(&mut body, 150.0, self.sample_rate);
        Envelope::new(0.0, 0.06, 0.0, 0.04).apply(&mut body, self.sample_rate);
        body.resize((duration * self.sample_rate as f32) as usize, 0.0);

        // High crack
        let mut crack = noise(0.04, self.sample_rate, 222);
        high_pass(&mut crack, 2500.0, self.sample_rate);
        Envelope::new(0.0, 0.025, 0.0, 0.015).apply(&mut crack, self.sample_rate);
        crack.resize((duration * self.sample_rate as f32) as usize, 0.0);

        // Distortion-like harmonic
        let mut distort = oscillator(Waveform::Square, 150.0, 0.05, self.sample_rate);
        low_pass(&mut distort, 2000.0, self.sample_rate);
        Envelope::new(0.0, 0.03, 0.0, 0.02).apply(&mut distort, self.sample_rate);
        distort.resize((duration * self.sample_rate as f32) as usize, 0.0);

        let mut result = mix(&[
            (&thump, 0.5),
            (&body, 0.35),
            (&crack, 0.3),
            (&distort, 0.2),
        ]);

        normalize_to(&mut result, 0.85);
        result
    }

    /// Generate a UI click sound
    ///
    /// Clean, professional click with subtle body.
    pub fn click(&self) -> Vec<f32> {
        let duration = 0.05;

        // Tonal click
        let mut tone = oscillator(Waveform::Sine, 1200.0, 0.03, self.sample_rate);
        Envelope::new(0.0, 0.015, 0.0, 0.015).apply(&mut tone, self.sample_rate);
        tone.resize((duration * self.sample_rate as f32) as usize, 0.0);

        // Subtle low thump
        let mut thump = oscillator(Waveform::Sine, 400.0, 0.025, self.sample_rate);
        Envelope::new(0.0, 0.01, 0.0, 0.015).apply(&mut thump, self.sample_rate);
        thump.resize((duration * self.sample_rate as f32) as usize, 0.0);

        // Tiny noise transient
        let mut transient = noise(0.008, self.sample_rate, 333);
        high_pass(&mut transient, 3000.0, self.sample_rate);
        Envelope::new(0.0, 0.005, 0.0, 0.003).apply(&mut transient, self.sample_rate);
        transient.resize((duration * self.sample_rate as f32) as usize, 0.0);

        let mut result = mix(&[(&tone, 0.5), (&thump, 0.3), (&transient, 0.2)]);

        normalize_to(&mut result, 0.75);
        result
    }

    /// Generate a power-up sound
    ///
    /// Ascending, triumphant sound with rich harmonics and shimmer.
    pub fn powerup(&self) -> Vec<f32> {
        let duration = 0.6;

        // Main ascending sweep with harmonics
        let mut sweep1 = self.sweep_filtered(
            Waveform::Saw,
            220.0,
            880.0,
            0.4,
            Envelope::new(0.01, 0.25, 0.4, 0.15),
            2000.0,
            6000.0,
            1.5,
        );
        sweep1.resize((duration * self.sample_rate as f32) as usize, 0.0);

        // Octave layer
        let mut sweep2 = self.sweep(
            Waveform::Triangle,
            440.0,
            1760.0,
            0.35,
            Envelope::new(0.02, 0.2, 0.3, 0.15),
        );
        sweep2.resize((duration * self.sample_rate as f32) as usize, 0.0);

        // Shimmer: high frequency arpeggiated feel
        let shimmer = self.arpeggio_shimmer(duration);

        // Sub reinforcement
        let mut sub = self.sweep(
            Waveform::Sine,
            55.0,
            110.0,
            0.3,
            Envelope::new(0.01, 0.2, 0.0, 0.1),
        );
        sub.resize((duration * self.sample_rate as f32) as usize, 0.0);

        // Sparkle noise
        let mut sparkle = noise(0.4, self.sample_rate, 555);
        high_pass(&mut sparkle, 5000.0, self.sample_rate);
        Envelope::new(0.05, 0.2, 0.1, 0.15).apply(&mut sparkle, self.sample_rate);
        sparkle.resize((duration * self.sample_rate as f32) as usize, 0.0);

        let mut result = mix(&[
            (&sweep1, 0.35),
            (&sweep2, 0.25),
            (&shimmer, 0.2),
            (&sub, 0.25),
            (&sparkle, 0.1),
        ]);

        normalize_to(&mut result, 0.85);
        result
    }

    /// Helper: Create shimmer effect for power-up
    fn arpeggio_shimmer(&self, duration: f32) -> Vec<f32> {
        let num_samples = (duration * self.sample_rate as f32) as usize;
        let mut result = vec![0.0; num_samples];

        // Quick ascending notes
        let notes = [523.25, 659.25, 783.99, 1046.5, 1318.5]; // C5, E5, G5, C6, E6
        let note_duration = 0.08;
        let note_gap = 0.04;

        for (i, &freq) in notes.iter().enumerate() {
            let start_sample = ((i as f32 * (note_duration + note_gap)) * self.sample_rate as f32) as usize;
            let mut note = oscillator(Waveform::Sine, freq, note_duration, self.sample_rate);
            Envelope::new(0.001, 0.04, 0.2, 0.04).apply(&mut note, self.sample_rate);

            for (j, &sample) in note.iter().enumerate() {
                let idx = start_sample + j;
                if idx < num_samples {
                    result[idx] += sample * 0.4;
                }
            }
        }

        result
    }

    /// Generate a death/game-over sound
    ///
    /// Somber, descending sound with minor tonality and weight.
    pub fn death(&self) -> Vec<f32> {
        let duration = 1.0;

        // Main descending sweep
        let mut main = self.sweep_filtered(
            Waveform::Saw,
            440.0,
            110.0,
            0.7,
            Envelope::new(0.01, 0.4, 0.3, 0.3),
            3000.0,
            500.0,
            1.2,
        );
        main.resize((duration * self.sample_rate as f32) as usize, 0.0);

        // Minor third layer (sad quality)
        let mut minor = self.sweep(
            Waveform::Triangle,
            523.0, // C5
            131.0, // C3
            0.65,
            Envelope::new(0.02, 0.35, 0.2, 0.3),
        );
        minor.resize((duration * self.sample_rate as f32) as usize, 0.0);

        // Low drone
        let mut drone = oscillator(Waveform::Sine, 82.0, 0.8, self.sample_rate);
        Envelope::new(0.1, 0.4, 0.3, 0.3).apply(&mut drone, self.sample_rate);
        drone.resize((duration * self.sample_rate as f32) as usize, 0.0);

        // Descending minor arpeggio
        let arp = self.death_arpeggio(duration);

        // Subtle noise for texture
        let mut texture = pink_noise(0.6, self.sample_rate, 666);
        low_pass(&mut texture, 800.0, self.sample_rate);
        Envelope::new(0.1, 0.3, 0.1, 0.2).apply(&mut texture, self.sample_rate);
        texture.resize((duration * self.sample_rate as f32) as usize, 0.0);

        let mut result = mix(&[
            (&main, 0.35),
            (&minor, 0.25),
            (&drone, 0.2),
            (&arp, 0.25),
            (&texture, 0.1),
        ]);

        normalize_to(&mut result, 0.85);
        result
    }

    /// Helper: Create descending minor arpeggio for death sound
    fn death_arpeggio(&self, duration: f32) -> Vec<f32> {
        let num_samples = (duration * self.sample_rate as f32) as usize;
        let mut result = vec![0.0; num_samples];

        // Descending minor notes: Am chord descending
        let notes = [440.0, 392.0, 329.6, 293.7, 220.0]; // A4, G4, E4, D4, A3
        let note_duration = 0.15;
        let note_gap = 0.05;

        for (i, &freq) in notes.iter().enumerate() {
            let start_sample =
                ((i as f32 * (note_duration + note_gap)) * self.sample_rate as f32) as usize;

            // Detuned for sadness
            let note = self.unison_tone(
                Waveform::Triangle,
                freq,
                12.0, // more detune for melancholy
                3,
                note_duration,
                Envelope::new(0.005, 0.08, 0.3, 0.07),
            );

            for (j, &sample) in note.iter().enumerate() {
                let idx = start_sample + j;
                if idx < num_samples {
                    result[idx] += sample * 0.5;
                }
            }
        }

        result
    }

    // ========================================================================
    // ADDITIONAL HIGH-QUALITY PRESETS
    // ========================================================================

    /// Generate a menu select/confirm sound
    ///
    /// Satisfying confirmation beep with harmonic richness.
    pub fn confirm(&self) -> Vec<f32> {
        let duration = 0.2;

        // Two-note confirmation (ascending perfect fourth)
        let note1 = self.unison_tone(
            Waveform::Triangle,
            440.0,
            6.0,
            3,
            0.1,
            Envelope::new(0.001, 0.05, 0.3, 0.05),
        );

        let offset = (0.08 * self.sample_rate as f32) as usize;
        let note2 = self.unison_tone(
            Waveform::Triangle,
            587.0, // D5 - perfect fourth above A4
            6.0,
            3,
            0.12,
            Envelope::new(0.001, 0.06, 0.2, 0.06),
        );

        // Build result with offset
        let total_samples = (duration * self.sample_rate as f32) as usize;
        let mut result = vec![0.0; total_samples];

        for (i, &s) in note1.iter().enumerate() {
            if i < total_samples {
                result[i] += s * 0.6;
            }
        }
        for (i, &s) in note2.iter().enumerate() {
            let idx = offset + i;
            if idx < total_samples {
                result[idx] += s * 0.6;
            }
        }

        normalize_to(&mut result, 0.8);
        result
    }

    /// Generate an error/cancel sound
    ///
    /// Short, distinct sound indicating something went wrong.
    pub fn error(&self) -> Vec<f32> {
        let duration = 0.15;

        // Dissonant buzz
        let mut buzz = self.unison_tone(
            Waveform::Square,
            180.0,
            50.0, // heavy detune for dissonance
            4,
            0.12,
            Envelope::new(0.0, 0.08, 0.0, 0.04),
        );
        low_pass(&mut buzz, 2000.0, self.sample_rate);
        buzz.resize((duration * self.sample_rate as f32) as usize, 0.0);

        // Descending pitch
        let mut desc = self.sweep(
            Waveform::Saw,
            400.0,
            200.0,
            0.1,
            Envelope::new(0.0, 0.06, 0.0, 0.04),
        );
        low_pass(&mut desc, 3000.0, self.sample_rate);
        desc.resize((duration * self.sample_rate as f32) as usize, 0.0);

        let mut result = mix(&[(&buzz, 0.5), (&desc, 0.4)]);

        normalize_to(&mut result, 0.75);
        result
    }

    /// Generate a whoosh/swoosh sound
    ///
    /// Movement sound for transitions, attacks, etc.
    pub fn whoosh(&self) -> Vec<f32> {
        let duration = 0.25;

        // Filtered noise sweep
        let mut noise_main = noise(duration, self.sample_rate, 888);

        // Apply time-varying bandpass by processing in chunks
        let chunk_size = (self.sample_rate as usize / 100).max(1);
        let num_samples = noise_main.len();

        for (chunk_idx, chunk) in noise_main.chunks_mut(chunk_size).enumerate() {
            let t = (chunk_idx * chunk_size) as f32 / num_samples as f32;
            // Sweep from low to high
            let center = 200.0 + t * 4000.0;
            let bandwidth = 500.0 + t * 1500.0;
            low_pass(chunk, center + bandwidth, self.sample_rate);
            high_pass(chunk, (center - bandwidth).max(50.0), self.sample_rate);
        }

        Envelope::new(0.01, 0.15, 0.0, 0.1).apply(&mut noise_main, self.sample_rate);

        // Add subtle pitch element
        let mut tone = self.sweep(
            Waveform::Sine,
            150.0,
            600.0,
            0.2,
            Envelope::new(0.02, 0.12, 0.0, 0.06),
        );
        tone.resize((duration * self.sample_rate as f32) as usize, 0.0);

        let mut result = mix(&[(&noise_main, 0.7), (&tone, 0.2)]);

        normalize_to(&mut result, 0.75);
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
    fn test_synth_unison_tone() {
        let synth = Synth::new(SAMPLE_RATE);
        let tone = synth.unison_tone(Waveform::Saw, 440.0, 10.0, 3, 0.1, Envelope::default());
        assert!(!tone.is_empty());
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

    #[test]
    fn test_synth_new_presets() {
        let synth = Synth::new(SAMPLE_RATE);

        // New presets
        assert!(!synth.confirm().is_empty());
        assert!(!synth.error().is_empty());
        assert!(!synth.whoosh().is_empty());
    }

    #[test]
    fn test_preset_amplitudes() {
        let synth = Synth::new(SAMPLE_RATE);

        // All presets should be properly normalized (no clipping)
        let presets: Vec<Vec<f32>> = vec![
            synth.coin(),
            synth.jump(),
            synth.laser(),
            synth.explosion(),
            synth.hit(),
            synth.click(),
            synth.powerup(),
            synth.death(),
            synth.confirm(),
            synth.error(),
            synth.whoosh(),
        ];

        for (i, preset) in presets.iter().enumerate() {
            let max_amp = preset.iter().map(|s| s.abs()).fold(0.0f32, f32::max);
            assert!(
                max_amp <= 1.0,
                "Preset {} has amplitude {} > 1.0 (clipping)",
                i,
                max_amp
            );
            assert!(
                max_amp > 0.5,
                "Preset {} has low amplitude {} (too quiet)",
                i,
                max_amp
            );
        }
    }
}
