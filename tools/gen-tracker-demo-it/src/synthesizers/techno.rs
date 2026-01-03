//! Acid Techno instrument synthesis
//!
//! Instruments for "Nether Acid" - Acid Techno at 130 BPM in E minor
//! Features: TB-303 acid bassline with resonant filter, 909 drums

use super::common::{SimpleRng, SAMPLE_RATE};
use std::f32::consts::PI;

const TWO_PI: f32 = 2.0 * PI;

// ============================================================================
// DSP Utilities
// ============================================================================

/// Soft saturation using tanh for analog warmth
#[inline]
fn soft_saturate(x: f32) -> f32 {
    x.tanh()
}

/// 2-pole state variable filter (for TB-303 bandpass)
struct StateVariableFilter {
    low: f32,
    band: f32,
}

impl StateVariableFilter {
    fn new() -> Self {
        Self { low: 0.0, band: 0.0 }
    }

    fn process(&mut self, input: f32, cutoff: f32, resonance: f32) -> (f32, f32, f32) {
        let f = (cutoff * PI).min(0.99);
        let q = 1.0 - resonance.min(0.95);

        self.low += f * self.band;
        let high = input - self.low - q * self.band;
        self.band += f * high;

        (self.low, self.band, high)
    }
}

/// Biquad low-pass filter (professional quality)
struct BiquadLP {
    x1: f32, x2: f32,
    y1: f32, y2: f32,
    b0: f32, b1: f32, b2: f32,
    a1: f32, a2: f32,
}

impl BiquadLP {
    fn new() -> Self {
        Self { x1: 0.0, x2: 0.0, y1: 0.0, y2: 0.0, b0: 1.0, b1: 0.0, b2: 0.0, a1: 0.0, a2: 0.0 }
    }

    fn set_params(&mut self, freq: f32, q: f32) {
        let w0 = TWO_PI * (freq / SAMPLE_RATE).min(0.49);
        let alpha = w0.sin() / (2.0 * q);
        let cos_w0 = w0.cos();

        let a0 = 1.0 + alpha;
        self.b0 = ((1.0 - cos_w0) / 2.0) / a0;
        self.b1 = (1.0 - cos_w0) / a0;
        self.b2 = self.b0;
        self.a1 = (-2.0 * cos_w0) / a0;
        self.a2 = (1.0 - alpha) / a0;
    }

    fn process(&mut self, input: f32) -> f32 {
        let output = self.b0 * input + self.b1 * self.x1 + self.b2 * self.x2 - self.a1 * self.y1 - self.a2 * self.y2;
        self.x2 = self.x1;
        self.x1 = input;
        self.y2 = self.y1;
        self.y1 = output;
        output
    }
}

/// Biquad high-pass filter (professional quality)
struct BiquadHP {
    x1: f32, x2: f32,
    y1: f32, y2: f32,
    b0: f32, b1: f32, b2: f32,
    a1: f32, a2: f32,
}

impl BiquadHP {
    fn new() -> Self {
        Self { x1: 0.0, x2: 0.0, y1: 0.0, y2: 0.0, b0: 1.0, b1: 0.0, b2: 0.0, a1: 0.0, a2: 0.0 }
    }

    fn set_params(&mut self, freq: f32, q: f32) {
        let w0 = TWO_PI * (freq / SAMPLE_RATE).min(0.49);
        let alpha = w0.sin() / (2.0 * q);
        let cos_w0 = w0.cos();

        let a0 = 1.0 + alpha;
        self.b0 = ((1.0 + cos_w0) / 2.0) / a0;
        self.b1 = -(1.0 + cos_w0) / a0;
        self.b2 = self.b0;
        self.a1 = (-2.0 * cos_w0) / a0;
        self.a2 = (1.0 - alpha) / a0;
    }

    fn process(&mut self, input: f32) -> f32 {
        let output = self.b0 * input + self.b1 * self.x1 + self.b2 * self.x2 - self.a1 * self.y1 - self.a2 * self.y2;
        self.x2 = self.x1;
        self.x1 = input;
        self.y2 = self.y1;
        self.y1 = output;
        output
    }
}

// ============================================================================
// 909 Kick - Clean and punchy (Professional Quality)
// ============================================================================

pub fn generate_kick_909() -> Vec<i16> {
    let duration = 0.4;
    let samples = (SAMPLE_RATE * duration) as usize;
    let mut output = Vec::with_capacity(samples);
    let mut rng = SimpleRng::new(12345);
    let mut phase = 0.0f32;
    let mut click_lp = BiquadLP::new();
    click_lp.set_params(3500.0, 0.7);

    for i in 0..samples {
        let t = i as f32 / SAMPLE_RATE;

        // === CLICK/TRANSIENT (first 8ms) ===
        let click = if t < 0.008 {
            let noise = rng.next_f32() * 2.0 - 1.0;
            let env = (1.0 - t / 0.008).powf(0.5);
            click_lp.process(noise) * env
        } else {
            0.0
        };

        // === PITCH ENVELOPE ===
        // Start at 160Hz, drop to 40Hz exponentially
        let pitch_env = (-t * 30.0).exp();
        let freq = 40.0 + 120.0 * pitch_env;

        // === BODY ===
        phase += freq / SAMPLE_RATE;
        if phase >= 1.0 { phase -= 1.0; }

        // Pure sine + slight 2nd harmonic for punch
        let body = (phase * TWO_PI).sin() + 0.2 * (phase * TWO_PI * 2.0).sin();

        // === AMPLITUDE ENVELOPE ===
        let env = if t < 0.004 {
            (t / 0.004).powf(0.3) // Fast attack
        } else {
            (-t * 9.0).exp() // Natural decay
        };

        // === MIX, SATURATE, AND OUTPUT ===
        let mixed = click * 0.4 + body * env;
        let saturated = soft_saturate(mixed * 1.5) * 0.7;
        let sample = saturated * 31000.0;
        output.push(sample.clamp(-32767.0, 32767.0) as i16);
    }

    output
}

// ============================================================================
// 909 Clap - Multiple layers for classic sound (High Quality)
// ============================================================================

pub fn generate_clap_909() -> Vec<i16> {
    let duration = 0.3;
    let samples = (SAMPLE_RATE * duration) as usize;
    let mut output = Vec::with_capacity(samples);
    let mut rng = SimpleRng::new(67890);
    let mut phase = 0.0f32;
    let mut noise_hp = StateVariableFilter::new();
    let mut noise_lp = BiquadLP::new();
    noise_lp.set_params(7000.0, 0.6);

    for i in 0..samples {
        let t = i as f32 / SAMPLE_RATE;

        // === BODY TONE (180Hz pitched down) ===
        let body_freq = 180.0 * (-t * 18.0).exp() + 100.0;
        phase += body_freq / SAMPLE_RATE;
        if phase >= 1.0 { phase -= 1.0; }

        let body = (phase * TWO_PI).sin();
        let body_env = (-t * 28.0).exp();

        // === NOISE CRACK (multi-layered) ===
        let noise_raw = rng.next_f32() * 2.0 - 1.0;
        let (_, band, high) = noise_hp.process(noise_raw, 0.42, 0.25);
        let noise = noise_lp.process(band * 0.6 + high * 0.4);

        // Multi-tap envelope for clap character
        let layer1 = if t < 0.002 { (t / 0.002).powf(0.5) } else { (-t * 35.0).exp() };
        let layer2 = if t >= 0.003 && t < 0.006 {
            ((t - 0.003) / 0.003).powf(0.5) * 0.7
        } else if t >= 0.006 {
            (-((t - 0.006) * 30.0)).exp() * 0.7
        } else {
            0.0
        };
        let layer3 = if t >= 0.008 && t < 0.012 {
            ((t - 0.008) / 0.004).powf(0.5) * 0.5
        } else if t >= 0.012 {
            (-((t - 0.012) * 25.0)).exp() * 0.5
        } else {
            0.0
        };

        let noise_env = layer1 + layer2 + layer3;

        // === MIX, SATURATE, AND OUTPUT ===
        let mixed = body * body_env * 0.3 + noise * noise_env;
        let saturated = soft_saturate(mixed * 1.4) * 0.7;
        let sample = saturated * 30000.0;
        output.push(sample.clamp(-32767.0, 32767.0) as i16);
    }

    output
}

// ============================================================================
// 909 Hi-hat Closed - Bright and tight (High Quality)
// ============================================================================

pub fn generate_hat_909_closed() -> Vec<i16> {
    let duration = 0.08;
    let samples = (SAMPLE_RATE * duration) as usize;
    let mut output = Vec::with_capacity(samples);
    let mut rng = SimpleRng::new(11111);
    let mut hp = StateVariableFilter::new();
    let mut lp = BiquadLP::new();
    lp.set_params(11000.0, 0.7);

    // Metallic tones (inharmonic)
    let mut phases = [0.0f32; 6];
    let freqs = [3200.0, 4600.0, 6100.0, 7300.0, 8200.0, 9800.0];

    for i in 0..samples {
        let t = i as f32 / SAMPLE_RATE;

        // Metallic component
        let mut metal = 0.0f32;
        for (j, freq) in freqs.iter().enumerate() {
            phases[j] += freq / SAMPLE_RATE;
            phases[j] %= 1.0;
            metal += (phases[j] * TWO_PI).sin() * (1.0 - j as f32 * 0.12);
        }
        metal /= 4.0;

        // Noise component
        let noise = rng.next_f32() * 2.0 - 1.0;
        let (_, _, high) = hp.process(noise, 0.55, 0.2);
        let filtered = lp.process(high * 0.5 + metal * 0.5);

        let env = (-t * 55.0).exp();
        let saturated = soft_saturate(filtered * env * 1.3) * 0.75;
        let sample = saturated * 28000.0;
        output.push(sample.clamp(-32767.0, 32767.0) as i16);
    }

    output
}

// ============================================================================
// 909 Hi-hat Open - Longer decay (High Quality)
// ============================================================================

pub fn generate_hat_909_open() -> Vec<i16> {
    let duration = 0.32;
    let samples = (SAMPLE_RATE * duration) as usize;
    let mut output = Vec::with_capacity(samples);
    let mut rng = SimpleRng::new(22222);
    let mut hp = StateVariableFilter::new();
    let mut lp = BiquadLP::new();
    lp.set_params(10500.0, 0.6);

    let mut phases = [0.0f32; 6];
    let freqs = [3000.0, 4300.0, 5900.0, 6900.0, 8300.0, 10000.0];

    for i in 0..samples {
        let t = i as f32 / SAMPLE_RATE;

        let mut metal = 0.0f32;
        for (j, freq) in freqs.iter().enumerate() {
            phases[j] += freq / SAMPLE_RATE;
            phases[j] %= 1.0;
            metal += (phases[j] * TWO_PI).sin() * (1.0 - j as f32 * 0.1);
        }
        metal /= 4.0;

        let noise = rng.next_f32() * 2.0 - 1.0;
        let (_, _, high) = hp.process(noise, 0.48, 0.25);
        let filtered = lp.process(high * 0.45 + metal * 0.55);

        let env = (-t * 11.0).exp();
        let saturated = soft_saturate(filtered * env * 1.3) * 0.75;
        let sample = saturated * 27000.0;
        output.push(sample.clamp(-32767.0, 32767.0) as i16);
    }

    output
}

// ============================================================================
// TB-303 Acid Bass - THE STAR (High Quality)
// ============================================================================

pub fn generate_bass_303() -> Vec<i16> {
    use super::common::{exp_decay, exp_attack, sawtooth_blep};

    let duration = 0.8;  // Longer for natural release
    let samples = (SAMPLE_RATE * duration) as usize;
    let mut output = Vec::with_capacity(samples);

    // Base frequency (this will be modulated by note pitch in the tracker)
    let base_freq = 82.41;  // E2 note
    let phase_inc = base_freq / SAMPLE_RATE;
    let mut phase = 0.0f32;
    let mut filter = StateVariableFilter::new();

    for i in 0..samples {
        let t = i as f32 / SAMPLE_RATE;

        // === ANTI-ALIASED SAWTOOTH WAVE (PolyBLEP) ===
        let saw = sawtooth_blep(phase, phase_inc);
        phase += phase_inc;
        if phase >= 1.0 { phase -= 1.0; }

        // === FILTER ENVELOPE (exponential for natural sound) ===
        // Creates the "squelch" when accent is triggered
        let filter_env = if t < 0.005 {
            exp_attack(t / 0.005, 12.0) * 3.0  // Very fast exponential attack
        } else {
            let decay_t = (t - 0.005) / 0.15;
            3.0 * exp_decay(decay_t, 6.0).max(0.5)  // Exponential decay to sustain
        };

        // === RESONANT BANDPASS FILTER ===
        // Cutoff frequency modulated by envelope
        // Base cutoff ~200Hz, envelope can push it to ~2kHz
        let cutoff_hz = 200.0 + 1800.0 * filter_env;
        let cutoff_norm = (cutoff_hz / SAMPLE_RATE).min(0.45);

        // High resonance creates the "squelch"
        let resonance = 0.85;  // Very high for acid character

        // Process through state variable filter (we want bandpass output)
        let (_low, band, _high) = filter.process(saw, cutoff_norm, resonance);

        // === AMPLITUDE ENVELOPE (exponential) ===
        let amp_env = if t < 0.003 {
            exp_attack(t / 0.003, 10.0)  // Smooth attack
        } else if t < 0.7 {
            1.0  // Sustain
        } else {
            exp_decay((t - 0.7) / 0.1, 8.0)  // Smooth exponential release
        };

        // === OUTPUT ===
        // Bandpass output is the signature 303 sound
        let filtered = band * amp_env;

        // Gentle soft saturation for analog warmth
        let saturated = soft_saturate(filtered * 1.5) * 0.7;
        let sample = saturated * 31000.0;

        output.push(sample.clamp(-32767.0, 32767.0) as i16);
    }

    output
}

// ============================================================================
// Acid Pad - Rich background texture (High Quality)
// ============================================================================

pub fn generate_pad_acid() -> Vec<i16> {
    use super::common::{exp_attack, exp_decay};

    let duration = 2.5;  // Longer for fuller sustain
    let samples = (SAMPLE_RATE * duration) as usize;
    let mut output = Vec::with_capacity(samples);

    // Base frequency (E3)
    let base_freq = 164.81;

    // Local filter state
    let mut lp_state = 0.0f32;

    for i in 0..samples {
        let t = i as f32 / SAMPLE_RATE;

        // Five detuned oscillators for richer chorus effect
        let osc1 = (t * base_freq * TWO_PI).sin();
        let osc2 = (t * base_freq * 1.005 * TWO_PI).sin();  // +5 cents
        let osc3 = (t * base_freq * 0.995 * TWO_PI).sin();  // -5 cents
        let osc4 = (t * base_freq * 1.010 * TWO_PI).sin();  // +10 cents
        let osc5 = (t * base_freq * 0.990 * TWO_PI).sin();  // -10 cents

        let mut oscillator = (osc1 + osc2 + osc3 + osc4 + osc5) / 5.0;

        // Low-pass filter for warmth (1-pole)
        lp_state = lp_state * 0.93 + oscillator * 0.07;
        oscillator = lp_state;

        // Exponential ADSR envelope
        let amp_env = if t < 0.3 {
            exp_attack(t / 0.3, 5.0)  // Slow exponential attack
        } else if t < 2.0 {
            1.0  // Sustain
        } else {
            exp_decay((t - 2.0) / 0.5, 6.0)  // Smooth exponential release
        };

        let saturated = soft_saturate(oscillator * amp_env * 1.2) * 0.7;
        let sample = saturated * 29000.0;

        output.push(sample.clamp(-32767.0, 32767.0) as i16);
    }

    output
}

// ============================================================================
// Acid Stab - Chord hits (High Quality)
// ============================================================================

pub fn generate_stab_acid() -> Vec<i16> {
    use super::common::{exp_attack, exp_decay, sawtooth_blep};

    let duration = 0.4;  // Longer tail for natural release
    let samples = (SAMPLE_RATE * duration) as usize;
    let mut output = Vec::with_capacity(samples);

    // Base frequency (E4)
    let base_freq = 329.63;

    // Phase accumulators for each oscillator
    let mut phase1 = 0.0f32;
    let mut phase2 = 0.0f32;
    let mut phase3 = 0.0f32;
    let mut phase4 = 0.0f32;
    let mut phase5 = 0.0f32;

    let freq1 = base_freq;
    let freq2 = base_freq * 1.005;  // +5 cents
    let freq3 = base_freq * 0.995;  // -5 cents
    let freq4 = base_freq * 1.010;  // +10 cents
    let freq5 = base_freq * 0.990;  // -10 cents

    let phase_inc1 = freq1 / SAMPLE_RATE;
    let phase_inc2 = freq2 / SAMPLE_RATE;
    let phase_inc3 = freq3 / SAMPLE_RATE;
    let phase_inc4 = freq4 / SAMPLE_RATE;
    let phase_inc5 = freq5 / SAMPLE_RATE;

    for i in 0..samples {
        let t = i as f32 / SAMPLE_RATE;

        // Five detuned anti-aliased sawtooth oscillators (supersaw)
        let saw1 = sawtooth_blep(phase1, phase_inc1);
        let saw2 = sawtooth_blep(phase2, phase_inc2);
        let saw3 = sawtooth_blep(phase3, phase_inc3);
        let saw4 = sawtooth_blep(phase4, phase_inc4);
        let saw5 = sawtooth_blep(phase5, phase_inc5);

        phase1 += phase_inc1; if phase1 >= 1.0 { phase1 -= 1.0; }
        phase2 += phase_inc2; if phase2 >= 1.0 { phase2 -= 1.0; }
        phase3 += phase_inc3; if phase3 >= 1.0 { phase3 -= 1.0; }
        phase4 += phase_inc4; if phase4 >= 1.0 { phase4 -= 1.0; }
        phase5 += phase_inc5; if phase5 >= 1.0 { phase5 -= 1.0; }

        let supersaw = (saw1 + saw2 + saw3 + saw4 + saw5) / 5.0;

        // Exponential amplitude envelope - punchy but smooth
        let amp_env = if t < 0.005 {
            exp_attack(t / 0.005, 12.0)  // Very fast attack
        } else {
            exp_decay((t - 0.005) / 0.35, 10.0)  // Fast exponential decay
        };

        let saturated = soft_saturate(supersaw * amp_env * 1.5) * 0.7;
        let sample = saturated * 30000.0;

        output.push(sample.clamp(-32767.0, 32767.0) as i16);
    }

    output
}

// ============================================================================
// TB-303 Squelch Variant - Maximum resonance for climax (High Quality)
// ============================================================================

pub fn generate_bass_303_squelch() -> Vec<i16> {
    use super::common::{exp_decay, exp_attack, sawtooth_blep};

    let duration = 0.8;  // Longer for natural release
    let samples = (SAMPLE_RATE * duration) as usize;
    let mut output = Vec::with_capacity(samples);

    let base_freq = 82.41;  // E2
    let phase_inc = base_freq / SAMPLE_RATE;
    let mut phase = 0.0f32;
    let mut filter = StateVariableFilter::new();

    for i in 0..samples {
        let t = i as f32 / SAMPLE_RATE;

        // === ANTI-ALIASED SAWTOOTH WAVE (PolyBLEP) ===
        let saw = sawtooth_blep(phase, phase_inc);
        phase += phase_inc;
        if phase >= 1.0 { phase -= 1.0; }

        // === MORE AGGRESSIVE filter envelope for maximum squelch ===
        let filter_env = if t < 0.005 {
            exp_attack(t / 0.005, 15.0) * 4.0  // Very aggressive attack
        } else {
            let decay_t = (t - 0.005) / 0.12;  // Faster decay
            4.0 * exp_decay(decay_t, 8.0).max(0.3)  // Lower sustain
        };

        // === WIDER filter sweep ===
        let cutoff_hz = 200.0 + 2800.0 * filter_env;  // Up to 3kHz vs 2kHz
        let cutoff_norm = (cutoff_hz / SAMPLE_RATE).min(0.45);

        // === HIGHER resonance for maximum squelch ===
        let resonance = 0.92;  // vs 0.85 in normal 303

        let (_low, band, _high) = filter.process(saw, cutoff_norm, resonance);

        // === AMPLITUDE ENVELOPE (exponential) ===
        let amp_env = if t < 0.003 {
            exp_attack(t / 0.003, 10.0)
        } else if t < 0.7 {
            1.0
        } else {
            exp_decay((t - 0.7) / 0.1, 8.0)
        };

        let filtered = band * amp_env;

        // More saturation for aggressive character
        let saturated = soft_saturate(filtered * 2.0) * 0.65;
        let sample = saturated * 31000.0;

        output.push(sample.clamp(-32767.0, 32767.0) as i16);
    }

    output
}

// ============================================================================
// Acid Riser - Smooth sweep for builds (High Quality)
// ============================================================================

pub fn generate_riser_acid() -> Vec<i16> {
    let duration = 2.5;
    let samples = (SAMPLE_RATE * duration) as usize;
    let mut output = Vec::with_capacity(samples);
    let mut rng = SimpleRng::new(99999);
    let mut phase = 0.0f32;
    let mut svf = StateVariableFilter::new();

    for i in 0..samples {
        let t = i as f32 / SAMPLE_RATE;
        let progress = t / duration;

        // Rising sine
        let freq = 120.0 + 1400.0 * progress.powf(2.0);
        phase += freq / SAMPLE_RATE;
        if phase >= 1.0 { phase -= 1.0; }
        let sine = (phase * TWO_PI).sin();

        // Noise with rising filter
        let noise = rng.next_f32() * 2.0 - 1.0;
        let cutoff = 0.04 + 0.38 * progress;
        let (_, _, high) = svf.process(noise, cutoff, 0.3);

        let env = progress.powf(1.5);
        let mixed = (sine * 0.5 + high * 0.35) * env;
        let saturated = soft_saturate(mixed * 1.4) * 0.7;
        let sample = saturated * 29000.0;
        output.push(sample.clamp(-32767.0, 32767.0) as i16);
    }

    output
}

// ============================================================================
// Acid Atmosphere - Subtle texture layer (High Quality)
// ============================================================================

pub fn generate_atmosphere_acid() -> Vec<i16> {
    use super::common::{exp_attack, exp_decay};

    let duration = 4.0;  // Very long sustain for subtle texture
    let samples = (SAMPLE_RATE * duration) as usize;
    let mut output = Vec::with_capacity(samples);

    // Very low frequency for atmosphere
    let base_freq = 55.0;  // A1

    for i in 0..samples {
        let t = i as f32 / SAMPLE_RATE;

        // Multiple slow LFOs for movement
        let lfo1 = (t * 0.4 * TWO_PI).sin();
        let lfo2 = (t * 0.6 * TWO_PI).sin();
        let lfo3 = (t * 0.9 * TWO_PI).sin();

        // Five detuned sine oscillators (very subtle chorus)
        let osc1 = (t * base_freq * TWO_PI).sin();
        let osc2 = (t * base_freq * 1.002 * TWO_PI).sin();
        let osc3 = (t * base_freq * 0.998 * TWO_PI).sin();
        let osc4 = (t * base_freq * 1.004 * TWO_PI).sin();
        let osc5 = (t * base_freq * 0.996 * TWO_PI).sin();

        let mut oscillator = (osc1 + osc2 + osc3 + osc4 + osc5) / 5.0;

        // Modulate amplitude with LFOs for subtle movement
        oscillator *= 0.4 + 0.2 * lfo1 + 0.15 * lfo2 + 0.1 * lfo3;

        // Exponential ADSR envelope - very slow
        let amp_env = if t < 0.8 {
            exp_attack(t / 0.8, 3.0)  // Very slow attack
        } else if t < 3.0 {
            1.0  // Long sustain
        } else {
            exp_decay((t - 3.0) / 1.0, 4.0)  // Slow release
        };

        let saturated = soft_saturate(oscillator * amp_env * 1.1) * 0.7;

        // VERY quiet - just subtle texture
        let sample = saturated * 26000.0;
        output.push(sample.clamp(-32767.0, 32767.0) as i16);
    }

    output
}

// ============================================================================
// 909 Crash Cymbal - For transitions (High Quality)
// ============================================================================

pub fn generate_crash_909() -> Vec<i16> {
    let duration = 1.5;
    let samples = (SAMPLE_RATE * duration) as usize;
    let mut output = Vec::with_capacity(samples);
    let mut rng = SimpleRng::new(77777);

    // Multiple bandpass filters for metallic complexity
    let mut bp1 = BiquadHP::new();
    bp1.set_params(3800.0, 1.0);
    let mut bp2 = BiquadHP::new();
    bp2.set_params(5800.0, 1.2);
    let mut bp3 = BiquadHP::new();
    bp3.set_params(7800.0, 0.8);

    for i in 0..samples {
        let t = i as f32 / SAMPLE_RATE;
        let noise = rng.next_f32() * 2.0 - 1.0;

        // Multiple bands for shimmer
        let band1 = bp1.process(noise);
        let band2 = bp2.process(noise);
        let band3 = bp3.process(noise);

        let mix = band1 * 0.5 + band2 * 0.3 + band3 * 0.2;

        let env = if t < 0.008 { t / 0.008 } else { (-t * 2.2).exp() };

        let saturated = soft_saturate(mix * env * 1.4) * 0.75;
        let sample = saturated * 27000.0;
        output.push(sample.clamp(-32767.0, 32767.0) as i16);
    }

    output
}
