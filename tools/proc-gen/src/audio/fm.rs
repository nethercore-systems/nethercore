//! FM (Frequency Modulation) Synthesis for retro synthwave sounds
//!
//! FM synthesis creates complex timbres by modulating the frequency of one oscillator
//! (the carrier) with another oscillator (the modulator). This was the sound of the
//! 80s (Yamaha DX7) and creates rich, metallic, bell-like tones perfect for synthwave.
//!
//! # Basic FM Equation
//! output = sin(carrier_freq * t + mod_index * sin(modulator_freq * t))
//!
//! # Key Parameters
//! - Carrier frequency: The fundamental pitch
//! - Modulator frequency: Usually a ratio of the carrier (1:1, 2:1, 3:1, etc.)
//! - Modulation index: Controls harmonic richness (higher = more complex)

use std::f32::consts::PI;

/// FM synthesis operator (either carrier or modulator)
#[derive(Debug, Clone, Copy)]
pub struct FmOperator {
    /// Frequency ratio relative to base frequency (1.0 = unison)
    pub ratio: f32,
    /// Amplitude level (0.0 to 1.0)
    pub level: f32,
    /// Detune in cents (100 cents = 1 semitone)
    pub detune: f32,
    /// Feedback amount (0.0 to 1.0, for self-modulation)
    pub feedback: f32,
}

impl Default for FmOperator {
    fn default() -> Self {
        Self {
            ratio: 1.0,
            level: 1.0,
            detune: 0.0,
            feedback: 0.0,
        }
    }
}

impl FmOperator {
    /// Create a new operator with ratio and level
    pub fn new(ratio: f32, level: f32) -> Self {
        Self {
            ratio,
            level,
            detune: 0.0,
            feedback: 0.0,
        }
    }

    /// Set detune in cents
    pub fn with_detune(mut self, cents: f32) -> Self {
        self.detune = cents;
        self
    }

    /// Set feedback (for self-modulation)
    pub fn with_feedback(mut self, feedback: f32) -> Self {
        self.feedback = feedback.clamp(0.0, 1.0);
        self
    }

    /// Calculate actual frequency given base frequency
    pub fn frequency(&self, base_freq: f32) -> f32 {
        let detune_ratio = 2.0f32.powf(self.detune / 1200.0);
        base_freq * self.ratio * detune_ratio
    }
}

/// 2-operator FM synthesizer (simple but versatile)
///
/// Carrier modulated by modulator: C(M)
pub struct Fm2Op {
    sample_rate: u32,
    carrier: FmOperator,
    modulator: FmOperator,
    mod_index: f32,
}

impl Fm2Op {
    /// Create a new 2-operator FM synth
    pub fn new(sample_rate: u32) -> Self {
        Self {
            sample_rate,
            carrier: FmOperator::default(),
            modulator: FmOperator::new(1.0, 1.0),
            mod_index: 1.0,
        }
    }

    /// Set carrier parameters
    pub fn carrier(mut self, op: FmOperator) -> Self {
        self.carrier = op;
        self
    }

    /// Set modulator parameters
    pub fn modulator(mut self, op: FmOperator) -> Self {
        self.modulator = op;
        self
    }

    /// Set modulation index (controls harmonic richness)
    pub fn mod_index(mut self, index: f32) -> Self {
        self.mod_index = index;
        self
    }

    /// Generate FM sound at given frequency and duration
    pub fn generate(&self, base_freq: f32, duration: f32) -> Vec<f32> {
        let num_samples = (duration * self.sample_rate as f32) as usize;
        let mut samples = Vec::with_capacity(num_samples);

        let carrier_freq = self.carrier.frequency(base_freq);
        let mod_freq = self.modulator.frequency(base_freq);

        let carrier_omega = 2.0 * PI * carrier_freq / self.sample_rate as f32;
        let mod_omega = 2.0 * PI * mod_freq / self.sample_rate as f32;

        let mut mod_prev = 0.0f32; // For feedback

        for i in 0..num_samples {
            let t = i as f32;

            // Modulator with optional feedback
            let feedback = mod_prev * self.modulator.feedback * PI;
            let mod_phase = mod_omega * t + feedback;
            let mod_signal = mod_phase.sin() * self.modulator.level;
            mod_prev = mod_signal;

            // Carrier modulated by modulator
            let carrier_phase = carrier_omega * t + self.mod_index * mod_signal;
            let carrier_signal = carrier_phase.sin() * self.carrier.level;

            samples.push(carrier_signal);
        }

        samples
    }
}

/// 4-operator FM synthesizer (DX7-style algorithms)
///
/// Supports multiple algorithm configurations for complex sounds.
pub struct Fm4Op {
    sample_rate: u32,
    ops: [FmOperator; 4],
    algorithm: FmAlgorithm,
    mod_indices: [f32; 4], // Modulation depth for each operator
}

/// FM algorithm (how operators are connected)
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FmAlgorithm {
    /// Stack: 4 -> 3 -> 2 -> 1 (out)
    /// Creates very bright, complex harmonics
    Stack,
    /// Parallel carriers: (4->3) + (2->1) (both out)
    /// Rich, layered sound
    ParallelPairs,
    /// 4 -> (3,2,1) all modulate carrier 1
    /// Metallic, bell-like sounds
    TripleMod,
    /// All parallel: 1 + 2 + 3 + 4 (all out)
    /// Organ-like, additive synthesis
    AllParallel,
    /// (4 -> 3) + (4 -> 2) + (4 -> 1) - One modulator, multiple carriers
    /// Good for pads with evolving harmonics
    OneToMany,
}

impl Default for FmAlgorithm {
    fn default() -> Self {
        FmAlgorithm::Stack
    }
}

impl Fm4Op {
    /// Create a new 4-operator FM synth
    pub fn new(sample_rate: u32) -> Self {
        Self {
            sample_rate,
            ops: [
                FmOperator::new(1.0, 1.0),  // Op 1 (carrier)
                FmOperator::new(2.0, 0.5),  // Op 2
                FmOperator::new(3.0, 0.3),  // Op 3
                FmOperator::new(4.0, 0.2),  // Op 4
            ],
            algorithm: FmAlgorithm::Stack,
            mod_indices: [2.0, 1.5, 1.0, 0.5],
        }
    }

    /// Set an operator
    pub fn operator(mut self, index: usize, op: FmOperator) -> Self {
        if index < 4 {
            self.ops[index] = op;
        }
        self
    }

    /// Set algorithm
    pub fn algorithm(mut self, algo: FmAlgorithm) -> Self {
        self.algorithm = algo;
        self
    }

    /// Set modulation indices
    pub fn mod_indices(mut self, indices: [f32; 4]) -> Self {
        self.mod_indices = indices;
        self
    }

    /// Generate FM sound at given frequency and duration
    pub fn generate(&self, base_freq: f32, duration: f32) -> Vec<f32> {
        let num_samples = (duration * self.sample_rate as f32) as usize;
        let mut samples = Vec::with_capacity(num_samples);

        // Calculate frequencies for each operator
        let freqs: [f32; 4] = [
            self.ops[0].frequency(base_freq),
            self.ops[1].frequency(base_freq),
            self.ops[2].frequency(base_freq),
            self.ops[3].frequency(base_freq),
        ];

        let omegas: [f32; 4] = [
            2.0 * PI * freqs[0] / self.sample_rate as f32,
            2.0 * PI * freqs[1] / self.sample_rate as f32,
            2.0 * PI * freqs[2] / self.sample_rate as f32,
            2.0 * PI * freqs[3] / self.sample_rate as f32,
        ];

        let mut feedback = [0.0f32; 4];

        for i in 0..num_samples {
            let t = i as f32;

            let output = match self.algorithm {
                FmAlgorithm::Stack => {
                    // 4 -> 3 -> 2 -> 1 (out)
                    let fb3 = feedback[3] * self.ops[3].feedback * PI;
                    let op4 = (omegas[3] * t + fb3).sin() * self.ops[3].level;
                    feedback[3] = op4;

                    let op3 = (omegas[2] * t + self.mod_indices[3] * op4).sin() * self.ops[2].level;
                    let op2 = (omegas[1] * t + self.mod_indices[2] * op3).sin() * self.ops[1].level;
                    let op1 = (omegas[0] * t + self.mod_indices[1] * op2).sin() * self.ops[0].level;

                    op1
                }
                FmAlgorithm::ParallelPairs => {
                    // (4->3) + (2->1)
                    let op4 = (omegas[3] * t).sin() * self.ops[3].level;
                    let op3 = (omegas[2] * t + self.mod_indices[3] * op4).sin() * self.ops[2].level;

                    let op2 = (omegas[1] * t).sin() * self.ops[1].level;
                    let op1 = (omegas[0] * t + self.mod_indices[1] * op2).sin() * self.ops[0].level;

                    (op1 + op3) * 0.5
                }
                FmAlgorithm::TripleMod => {
                    // 4,3,2 -> 1
                    let op4 = (omegas[3] * t).sin() * self.ops[3].level;
                    let op3 = (omegas[2] * t).sin() * self.ops[2].level;
                    let op2 = (omegas[1] * t).sin() * self.ops[1].level;

                    let mod_sum = self.mod_indices[1] * op2
                        + self.mod_indices[2] * op3
                        + self.mod_indices[3] * op4;
                    let op1 = (omegas[0] * t + mod_sum).sin() * self.ops[0].level;

                    op1
                }
                FmAlgorithm::AllParallel => {
                    // All parallel (additive)
                    let op1 = (omegas[0] * t).sin() * self.ops[0].level;
                    let op2 = (omegas[1] * t).sin() * self.ops[1].level;
                    let op3 = (omegas[2] * t).sin() * self.ops[2].level;
                    let op4 = (omegas[3] * t).sin() * self.ops[3].level;

                    (op1 + op2 + op3 + op4) * 0.25
                }
                FmAlgorithm::OneToMany => {
                    // 4 -> (3,2,1)
                    let fb3 = feedback[3] * self.ops[3].feedback * PI;
                    let op4 = (omegas[3] * t + fb3).sin() * self.ops[3].level;
                    feedback[3] = op4;

                    let op3 = (omegas[2] * t + self.mod_indices[3] * op4).sin() * self.ops[2].level;
                    let op2 = (omegas[1] * t + self.mod_indices[2] * op4).sin() * self.ops[1].level;
                    let op1 = (omegas[0] * t + self.mod_indices[1] * op4).sin() * self.ops[0].level;

                    (op1 + op2 + op3) / 3.0
                }
            };

            samples.push(output);
        }

        samples
    }
}

// ============================================================================
// Synthwave Preset FM Sounds
// ============================================================================

/// Generate classic FM electric piano sound (DX7 Rhodes-like)
pub fn fm_epiano(base_freq: f32, duration: f32, sample_rate: u32) -> Vec<f32> {
    let synth = Fm4Op::new(sample_rate)
        .algorithm(FmAlgorithm::ParallelPairs)
        .operator(0, FmOperator::new(1.0, 1.0))
        .operator(1, FmOperator::new(14.0, 0.4)) // High ratio for bell-like overtones
        .operator(2, FmOperator::new(1.0, 0.7))
        .operator(3, FmOperator::new(1.0, 0.3).with_detune(7.0))
        .mod_indices([0.0, 2.5, 0.0, 1.8]);

    synth.generate(base_freq, duration)
}

/// Generate FM bass (punchy, growling)
pub fn fm_bass(base_freq: f32, duration: f32, sample_rate: u32) -> Vec<f32> {
    let synth = Fm4Op::new(sample_rate)
        .algorithm(FmAlgorithm::Stack)
        .operator(0, FmOperator::new(1.0, 1.0))
        .operator(1, FmOperator::new(1.0, 0.8))
        .operator(2, FmOperator::new(2.0, 0.4))
        .operator(3, FmOperator::new(1.0, 0.2).with_feedback(0.5))
        .mod_indices([0.0, 3.0, 2.0, 1.5]);

    synth.generate(base_freq, duration)
}

/// Generate FM brass/synth lead
pub fn fm_brass(base_freq: f32, duration: f32, sample_rate: u32) -> Vec<f32> {
    let synth = Fm4Op::new(sample_rate)
        .algorithm(FmAlgorithm::TripleMod)
        .operator(0, FmOperator::new(1.0, 1.0))
        .operator(1, FmOperator::new(1.0, 0.6).with_detune(10.0))
        .operator(2, FmOperator::new(2.0, 0.4))
        .operator(3, FmOperator::new(3.0, 0.3))
        .mod_indices([0.0, 2.0, 1.5, 1.0]);

    synth.generate(base_freq, duration)
}

/// Generate FM bell/chime sound
pub fn fm_bell(base_freq: f32, duration: f32, sample_rate: u32) -> Vec<f32> {
    // Bells use non-integer ratios for inharmonic overtones
    let synth = Fm4Op::new(sample_rate)
        .algorithm(FmAlgorithm::Stack)
        .operator(0, FmOperator::new(1.0, 1.0))
        .operator(1, FmOperator::new(3.5, 0.6))  // Inharmonic ratio
        .operator(2, FmOperator::new(7.0, 0.3))  // High ratio for shimmer
        .operator(3, FmOperator::new(1.41, 0.2)) // sqrt(2) for metallic quality
        .mod_indices([0.0, 4.0, 2.0, 1.0]);

    synth.generate(base_freq, duration)
}

/// Generate FM pad sound (lush, evolving)
pub fn fm_pad(base_freq: f32, duration: f32, sample_rate: u32) -> Vec<f32> {
    let synth = Fm4Op::new(sample_rate)
        .algorithm(FmAlgorithm::OneToMany)
        .operator(0, FmOperator::new(1.0, 0.8))
        .operator(1, FmOperator::new(2.0, 0.6).with_detune(5.0))
        .operator(2, FmOperator::new(3.0, 0.4).with_detune(-5.0))
        .operator(3, FmOperator::new(0.5, 0.5)) // Sub modulator
        .mod_indices([0.0, 1.5, 1.5, 1.5]);

    synth.generate(base_freq, duration)
}

/// Generate FM organ sound
pub fn fm_organ(base_freq: f32, duration: f32, sample_rate: u32) -> Vec<f32> {
    let synth = Fm4Op::new(sample_rate)
        .algorithm(FmAlgorithm::AllParallel)
        .operator(0, FmOperator::new(1.0, 1.0))   // Fundamental
        .operator(1, FmOperator::new(2.0, 0.7))   // 2nd harmonic
        .operator(2, FmOperator::new(3.0, 0.5))   // 3rd harmonic
        .operator(3, FmOperator::new(4.0, 0.3))   // 4th harmonic
        .mod_indices([0.0, 0.0, 0.0, 0.0]); // No modulation = pure additive

    synth.generate(base_freq, duration)
}

/// Generate an FM sound with time-varying modulation index
/// Great for synthwave sounds that evolve over time
pub fn fm_sweep(
    base_freq: f32,
    duration: f32,
    start_mod: f32,
    end_mod: f32,
    sample_rate: u32,
) -> Vec<f32> {
    let num_samples = (duration * sample_rate as f32) as usize;
    let mut samples = Vec::with_capacity(num_samples);

    let carrier_omega = 2.0 * PI * base_freq / sample_rate as f32;
    let mod_omega = 2.0 * PI * base_freq * 2.0 / sample_rate as f32; // 2:1 ratio

    for i in 0..num_samples {
        let t = i as f32;
        let progress = i as f32 / num_samples as f32;

        // Interpolate modulation index
        let mod_index = start_mod + (end_mod - start_mod) * progress;

        let modulator = (mod_omega * t).sin();
        let carrier = (carrier_omega * t + mod_index * modulator).sin();

        samples.push(carrier);
    }

    samples
}

/// Generate a classic synthwave lead with vibrato
pub fn fm_synthwave_lead(base_freq: f32, duration: f32, sample_rate: u32) -> Vec<f32> {
    let num_samples = (duration * sample_rate as f32) as usize;
    let mut samples = Vec::with_capacity(num_samples);

    // Main carrier
    let carrier_ratio = 1.0;
    // Modulator for harmonic content
    let mod_ratio = 1.0;
    let mod_index = 2.5;

    // Vibrato parameters (delayed onset)
    let vibrato_rate = 5.5; // Hz
    let vibrato_depth = 0.015; // Pitch deviation

    let vibrato_omega = 2.0 * PI * vibrato_rate / sample_rate as f32;

    for i in 0..num_samples {
        let t = i as f32 / sample_rate as f32;
        let sample_t = i as f32;

        // Delayed vibrato onset
        let vibrato_amount = if t < 0.1 { 0.0 } else { vibrato_depth * (t - 0.1).min(1.0) };
        let vibrato = 1.0 + vibrato_amount * (vibrato_omega * sample_t).sin();

        let freq = base_freq * vibrato;
        let carrier_omega = 2.0 * PI * freq * carrier_ratio / sample_rate as f32;
        let mod_omega = 2.0 * PI * freq * mod_ratio / sample_rate as f32;

        // FM synthesis with slight mod index decay for softer tail
        let mod_decay = 1.0 - t * 0.3;
        let modulator = (mod_omega * sample_t).sin();
        let carrier = (carrier_omega * sample_t + mod_index * mod_decay.max(0.2) * modulator).sin();

        samples.push(carrier);
    }

    samples
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_SAMPLE_RATE: u32 = 22050;

    #[test]
    fn test_fm_operator_default() {
        let op = FmOperator::default();
        assert_eq!(op.ratio, 1.0);
        assert_eq!(op.level, 1.0);
        assert_eq!(op.detune, 0.0);
        assert_eq!(op.feedback, 0.0);
    }

    #[test]
    fn test_fm_operator_frequency() {
        let op = FmOperator::new(2.0, 1.0);
        assert_eq!(op.frequency(440.0), 880.0);

        let op_detune = op.with_detune(1200.0); // +1 octave
        assert!((op_detune.frequency(440.0) - 1760.0).abs() < 0.1);
    }

    #[test]
    fn test_fm_2op_generate() {
        let fm = Fm2Op::new(TEST_SAMPLE_RATE)
            .carrier(FmOperator::new(1.0, 1.0))
            .modulator(FmOperator::new(2.0, 0.5))
            .mod_index(2.0);

        let samples = fm.generate(440.0, 0.1);
        assert!(!samples.is_empty());
        assert!(samples.iter().all(|&s| s >= -1.5 && s <= 1.5));
    }

    #[test]
    fn test_fm_4op_algorithms() {
        for algo in [
            FmAlgorithm::Stack,
            FmAlgorithm::ParallelPairs,
            FmAlgorithm::TripleMod,
            FmAlgorithm::AllParallel,
            FmAlgorithm::OneToMany,
        ] {
            let fm = Fm4Op::new(TEST_SAMPLE_RATE).algorithm(algo);
            let samples = fm.generate(440.0, 0.1);
            assert!(!samples.is_empty(), "Algorithm {:?} produced no samples", algo);
        }
    }

    #[test]
    fn test_fm_presets() {
        let presets: Vec<(&str, Vec<f32>)> = vec![
            ("epiano", fm_epiano(440.0, 0.1, TEST_SAMPLE_RATE)),
            ("bass", fm_bass(110.0, 0.1, TEST_SAMPLE_RATE)),
            ("brass", fm_brass(440.0, 0.1, TEST_SAMPLE_RATE)),
            ("bell", fm_bell(880.0, 0.1, TEST_SAMPLE_RATE)),
            ("pad", fm_pad(330.0, 0.1, TEST_SAMPLE_RATE)),
            ("organ", fm_organ(440.0, 0.1, TEST_SAMPLE_RATE)),
            ("synthwave_lead", fm_synthwave_lead(440.0, 0.1, TEST_SAMPLE_RATE)),
        ];

        for (name, samples) in presets {
            assert!(!samples.is_empty(), "{} preset produced no samples", name);
        }
    }

    #[test]
    fn test_fm_sweep() {
        let samples = fm_sweep(440.0, 0.2, 0.5, 5.0, TEST_SAMPLE_RATE);
        assert!(!samples.is_empty());
    }
}
