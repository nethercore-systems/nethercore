//! Ambient instrument synthesis - HIGH QUALITY
//!
//! Instruments for "Nether Mist" - Ambient at 70 BPM in D minor/Aeolian
//! Features: Lush pads, evolving textures, smooth envelopes, rich harmonics

use super::common::{SimpleRng, SAMPLE_RATE};
use std::f32::consts::PI;

const TWO_PI: f32 = 2.0 * PI;

// ============================================================================
// DSP Utilities
// ============================================================================

/// Soft saturation
fn _soft_saturate(x: f32) -> f32 {
    x.tanh()
}

/// PolyBLEP for anti-aliased oscillators
fn poly_blep(t: f32, dt: f32) -> f32 {
    if t < dt {
        let t = t / dt;
        2.0 * t - t * t - 1.0
    } else if t > 1.0 - dt {
        let t = (t - 1.0) / dt;
        t * t + 2.0 * t + 1.0
    } else {
        0.0
    }
}

fn blep_saw(phase: f32, dt: f32) -> f32 {
    let naive = 2.0 * phase - 1.0;
    naive - poly_blep(phase, dt)
}

/// Biquad filter (multiple modes)
struct Biquad {
    x1: f32, x2: f32,
    y1: f32, y2: f32,
    b0: f32, b1: f32, b2: f32,
    a1: f32, a2: f32,
}

impl Biquad {
    fn new() -> Self {
        Self { x1: 0.0, x2: 0.0, y1: 0.0, y2: 0.0, b0: 1.0, b1: 0.0, b2: 0.0, a1: 0.0, a2: 0.0 }
    }

    fn set_lowpass(&mut self, freq: f32, q: f32) {
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

    fn set_highpass(&mut self, freq: f32, q: f32) {
        let w0 = TWO_PI * (freq / SAMPLE_RATE).min(0.49);
        let alpha = w0.sin() / (2.0 * q);
        let cos_w0 = w0.cos();
        let a0 = 1.0 + alpha;
        self.b0 = ((1.0 + cos_w0) / 2.0) / a0;
        self.b1 = (-(1.0 + cos_w0)) / a0;
        self.b2 = self.b0;
        self.a1 = (-2.0 * cos_w0) / a0;
        self.a2 = (1.0 - alpha) / a0;
    }

    fn set_bandpass(&mut self, freq: f32, q: f32) {
        let w0 = TWO_PI * (freq / SAMPLE_RATE).min(0.49);
        let alpha = w0.sin() / (2.0 * q);
        let cos_w0 = w0.cos();
        let a0 = 1.0 + alpha;
        self.b0 = alpha / a0;
        self.b1 = 0.0;
        self.b2 = -alpha / a0;
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

/// State variable filter for smooth modulation
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

// ============================================================================
// Sub Pad - Deep foundation
// ============================================================================

pub fn generate_pad_sub() -> Vec<i16> {
    let duration = 4.0;
    let freq = 73.42; // D2
    let samples = (SAMPLE_RATE * duration) as usize;
    let mut output = Vec::with_capacity(samples);

    let mut phases = [0.0f32; 3];
    let detune = [0.998, 1.0, 1.002];

    for i in 0..samples {
        let t = i as f32 / SAMPLE_RATE;

        // Slow breathing modulation
        let breath = 1.0 + 0.004 * (t * 0.4 * TWO_PI).sin();
        let depth = 1.0 + 0.002 * (t * 0.23 * TWO_PI).sin();

        // Layered sines for warmth
        let mut sum = 0.0f32;
        for (j, d) in detune.iter().enumerate() {
            phases[j] += freq * d * breath / SAMPLE_RATE;
            if phases[j] >= 1.0 { phases[j] -= 1.0; }
            sum += (phases[j] * TWO_PI).sin();
        }
        sum /= 3.0;

        // Add subtle second harmonic
        let harm = (phases[1] * TWO_PI * 2.0).sin() * 0.1 * depth;

        // Smooth envelope
        let env = if t < 0.8 {
            (t / 0.8).powf(2.0)
        } else if t < 3.2 {
            1.0
        } else {
            (-(t - 3.2) * 1.5).exp()
        };

        let sample = (sum + harm) * env * 30000.0;
        output.push(sample.clamp(-32767.0, 32767.0) as i16);
    }

    output
}

// ============================================================================
// Air Pad - High ethereal texture
// ============================================================================

pub fn generate_pad_air() -> Vec<i16> {
    let duration = 4.0;
    let freq = 587.33; // D5
    let samples = (SAMPLE_RATE * duration) as usize;
    let mut output = Vec::with_capacity(samples);

    let mut phases = [0.0f32; 5];
    let detune = [0.995, 0.998, 1.0, 1.002, 1.005];
    let dt = freq / SAMPLE_RATE;

    let mut lp1 = Biquad::new();
    let mut lp2 = Biquad::new();
    lp1.set_lowpass(3000.0, 0.7);
    lp2.set_lowpass(2500.0, 0.7);

    for i in 0..samples {
        let t = i as f32 / SAMPLE_RATE;

        // Slow filter sweep
        let sweep = (t * 0.15 * TWO_PI).sin();
        let cutoff = 2000.0 + 800.0 * sweep;
        lp1.set_lowpass(cutoff, 1.2);

        // 5-voice detuned saws
        let mut sum = 0.0f32;
        for (j, d) in detune.iter().enumerate() {
            phases[j] += freq * d / SAMPLE_RATE;
            if phases[j] >= 1.0 { phases[j] -= 1.0; }
            sum += blep_saw(phases[j], dt * d);
        }
        sum /= 5.0;

        let filtered = lp2.process(lp1.process(sum));

        let env = if t < 1.0 {
            (t / 1.0).powf(2.5)
        } else if t < 3.0 {
            1.0
        } else {
            (-(t - 3.0) * 1.2).exp()
        };

        let sample = filtered * env * 26000.0;
        output.push(sample.clamp(-32767.0, 32767.0) as i16);
    }

    output
}

// ============================================================================
// Warm Pad - Mid-range comfort
// ============================================================================

pub fn generate_pad_warm() -> Vec<i16> {
    let duration = 4.0;
    let freq = 293.66; // D4
    let samples = (SAMPLE_RATE * duration) as usize;
    let mut output = Vec::with_capacity(samples);

    let mut phases = [0.0f32; 4];
    let detune = [0.993, 0.998, 1.002, 1.007];

    let mut lp1 = Biquad::new();
    let mut lp2 = Biquad::new();
    lp1.set_lowpass(1800.0, 0.8);
    lp2.set_lowpass(1200.0, 0.7);

    for i in 0..samples {
        let t = i as f32 / SAMPLE_RATE;

        // PWM-like modulation via phase offset
        let pw_mod = 0.5 + 0.1 * (t * 0.25 * TWO_PI).sin();

        let mut sum = 0.0f32;
        for (j, d) in detune.iter().enumerate() {
            phases[j] += freq * d / SAMPLE_RATE;
            if phases[j] >= 1.0 { phases[j] -= 1.0; }

            // Soft triangle with PWM effect
            let p = phases[j];
            let tri = if p < pw_mod {
                2.0 * p / pw_mod - 1.0
            } else {
                1.0 - 2.0 * (p - pw_mod) / (1.0 - pw_mod)
            };
            sum += tri;
        }
        sum /= 4.0;

        // Slight harmonic enrichment
        let harm = (phases[0] * TWO_PI * 2.0).sin() * 0.15;

        let filtered = lp2.process(lp1.process(sum + harm));

        let env = if t < 0.6 {
            (t / 0.6).powf(2.0)
        } else if t < 3.3 {
            1.0
        } else {
            (-(t - 3.3) * 1.5).exp()
        };

        let sample = filtered * env * 28000.0;
        output.push(sample.clamp(-32767.0, 32767.0) as i16);
    }

    output
}

// ============================================================================
// Cold Pad - Icy ethereal layer
// ============================================================================

pub fn generate_pad_cold() -> Vec<i16> {
    let duration = 4.0;
    let freq = 440.0; // A4
    let samples = (SAMPLE_RATE * duration) as usize;
    let mut output = Vec::with_capacity(samples);

    let mut phases = [0.0f32; 6];
    let detune = [0.985, 0.992, 0.998, 1.002, 1.008, 1.015];
    let dt = freq / SAMPLE_RATE;

    let mut hp = Biquad::new();
    let mut lp = Biquad::new();
    hp.set_highpass(400.0, 0.7);
    lp.set_lowpass(4000.0, 0.8);

    for i in 0..samples {
        let t = i as f32 / SAMPLE_RATE;

        // Wider detuned saws for cold shimmery texture
        let mut sum = 0.0f32;
        for (j, d) in detune.iter().enumerate() {
            phases[j] += freq * d / SAMPLE_RATE;
            if phases[j] >= 1.0 { phases[j] -= 1.0; }
            sum += blep_saw(phases[j], dt * d);
        }
        sum /= 6.0;

        // High-pass to sit above warm pad, low-pass to smooth
        let filtered = lp.process(hp.process(sum));

        let env = if t < 0.8 {
            (t / 0.8).powf(2.5)
        } else if t < 3.0 {
            1.0
        } else {
            (-(t - 3.0) * 1.3).exp()
        };

        let sample = filtered * env * 25000.0;
        output.push(sample.clamp(-32767.0, 32767.0) as i16);
    }

    output
}

// ============================================================================
// Breath Texture - Organic noise layer
// ============================================================================

pub fn generate_noise_breath() -> Vec<i16> {
    let duration = 3.0;
    let samples = (SAMPLE_RATE * duration) as usize;
    let mut output = Vec::with_capacity(samples);
    let mut rng = SimpleRng::new(11111);

    let mut bp1 = Biquad::new();
    let mut bp2 = Biquad::new();
    bp1.set_bandpass(800.0, 2.0);
    bp2.set_bandpass(1200.0, 2.0);

    for i in 0..samples {
        let t = i as f32 / SAMPLE_RATE;
        let noise = rng.next_f32() * 2.0 - 1.0;

        // Slowly shifting center frequency
        let center1 = 700.0 + 300.0 * (t * 0.2 * TWO_PI).sin();
        let center2 = 1100.0 + 400.0 * (t * 0.15 * TWO_PI).sin();
        bp1.set_bandpass(center1, 2.5);
        bp2.set_bandpass(center2, 2.5);

        let filtered = bp1.process(noise) * 0.6 + bp2.process(noise) * 0.4;

        let env = if t < 0.5 {
            (t / 0.5).powf(1.5)
        } else if t < 2.4 {
            1.0
        } else {
            (-(t - 2.4) * 2.0).exp()
        };

        let sample = filtered * env * 25000.0;
        output.push(sample.clamp(-32767.0, 32767.0) as i16);
    }

    output
}

// ============================================================================
// Glass Bell - Crystalline FM synthesis
// ============================================================================

pub fn generate_bell_glass() -> Vec<i16> {
    let duration = 3.5;
    let freq = 880.0; // A5
    let samples = (SAMPLE_RATE * duration) as usize;
    let mut output = Vec::with_capacity(samples);

    let mut carrier_phase = 0.0f32;
    let mut mod_phases = [0.0f32; 3];
    let mod_ratios = [2.76, 4.17, 7.23]; // Inharmonic for bell character
    let mod_depths = [0.8, 0.4, 0.2];

    for i in 0..samples {
        let t = i as f32 / SAMPLE_RATE;

        // Pitch bend at attack
        let pitch_env = if t < 0.015 { 1.0 + 0.03 * (1.0 - t / 0.015) } else { 1.0 };
        let actual_freq = freq * pitch_env;

        // Multiple FM modulators for complex bell timbre
        let mut mod_sum = 0.0f32;
        for j in 0..3 {
            mod_phases[j] += actual_freq * mod_ratios[j] / SAMPLE_RATE;
            if mod_phases[j] >= 1.0 { mod_phases[j] -= 1.0; }
            let mod_env = (-(j as f32 + 1.0) * t * 2.0).exp();
            mod_sum += (mod_phases[j] * TWO_PI).sin() * mod_depths[j] * mod_env;
        }

        carrier_phase += actual_freq * (1.0 + mod_sum) / SAMPLE_RATE;
        if carrier_phase >= 1.0 { carrier_phase -= 1.0; }
        let carrier = (carrier_phase * TWO_PI).sin();

        // Multi-stage decay for bell sustain
        let env = if t < 0.001 {
            t / 0.001
        } else {
            let fast = (-t * 4.0).exp() * 0.7;
            let slow = (-t * 0.8).exp() * 0.3;
            fast + slow
        };

        let sample = carrier * env * 28000.0;
        output.push(sample.clamp(-32767.0, 32767.0) as i16);
    }

    output
}

// ============================================================================
// Sub Bass - Deep foundation
// ============================================================================

pub fn generate_bass_sub() -> Vec<i16> {
    let duration = 2.5;
    let freq = 36.71; // D1
    let samples = (SAMPLE_RATE * duration) as usize;
    let mut output = Vec::with_capacity(samples);
    let mut phase = 0.0f32;

    for i in 0..samples {
        let t = i as f32 / SAMPLE_RATE;

        phase += freq / SAMPLE_RATE;
        if phase >= 1.0 { phase -= 1.0; }

        // Pure sine with subtle second harmonic
        let fundamental = (phase * TWO_PI).sin();
        let harmonic = (phase * TWO_PI * 2.0).sin() * 0.12;

        let env = if t < 0.15 {
            (t / 0.15).powf(1.2)
        } else if t < 1.8 {
            1.0
        } else {
            (-(t - 1.8) * 2.0).exp()
        };

        let sample = (fundamental + harmonic) * env * 31000.0;
        output.push(sample.clamp(-32767.0, 32767.0) as i16);
    }

    output
}

// ============================================================================
// Ghost Lead - Ethereal melodic voice
// ============================================================================

pub fn generate_lead_ghost() -> Vec<i16> {
    let duration = 2.0;
    let freq = 293.66; // D4
    let samples = (SAMPLE_RATE * duration) as usize;
    let mut output = Vec::with_capacity(samples);

    let mut phases = [0.0f32; 2];
    let detune = [0.997, 1.003];
    let mut vibrato_phase = 0.0f32;

    let mut svf = StateVariableFilter::new();

    for i in 0..samples {
        let t = i as f32 / SAMPLE_RATE;

        // Delayed vibrato
        let vib_depth = if t > 0.2 { ((t - 0.2) / 0.3).min(1.0) } else { 0.0 };
        vibrato_phase += 4.5 / SAMPLE_RATE;
        let vibrato = 1.0 + 0.01 * vib_depth * (vibrato_phase * TWO_PI).sin();

        let actual_freq = freq * vibrato;

        // Two detuned squares, heavily filtered
        let mut sum = 0.0f32;
        for (j, d) in detune.iter().enumerate() {
            phases[j] += actual_freq * d / SAMPLE_RATE;
            if phases[j] >= 1.0 { phases[j] -= 1.0; }
            let square = if phases[j] < 0.5 { 1.0 } else { -1.0 };
            sum += square;
        }
        sum /= 2.0;

        // Very soft filtering
        let (low, _, _) = svf.process(sum, 0.08, 0.2);

        let env = if t < 0.12 {
            (t / 0.12).powf(1.5)
        } else if t < 1.4 {
            1.0 - (t - 0.12) * 0.08
        } else {
            0.9 * (-(t - 1.4) * 2.5).exp()
        };

        let sample = low * env * 26000.0;
        output.push(sample.clamp(-32767.0, 32767.0) as i16);
    }

    output
}

// ============================================================================
// Reverb Simulation - Diffuse decay texture
// ============================================================================

pub fn generate_reverb_sim() -> Vec<i16> {
    let duration = 2.5;
    let samples = (SAMPLE_RATE * duration) as usize;
    let mut output = Vec::with_capacity(samples);
    let mut rng = SimpleRng::new(22222);

    // Multiple bandpass filters for diffuse character
    let mut filters = [Biquad::new(), Biquad::new(), Biquad::new(), Biquad::new()];
    filters[0].set_bandpass(400.0, 3.0);
    filters[1].set_bandpass(800.0, 3.0);
    filters[2].set_bandpass(1500.0, 3.0);
    filters[3].set_bandpass(3000.0, 3.0);

    for i in 0..samples {
        let t = i as f32 / SAMPLE_RATE;
        let noise = rng.next_f32() * 2.0 - 1.0;

        // Sum filtered bands
        let mut sum = 0.0f32;
        for (j, filter) in filters.iter_mut().enumerate() {
            let decay = (-t * (1.0 + j as f32 * 0.5)).exp();
            sum += filter.process(noise) * decay;
        }
        sum /= 3.0;

        let env = if t < 0.02 { t / 0.02 } else { (-t * 1.8).exp() };

        let sample = sum * env * 22000.0;
        output.push(sample.clamp(-32767.0, 32767.0) as i16);
    }

    output
}

// ============================================================================
// Wind Atmosphere - Organic ambient texture
// ============================================================================

pub fn generate_atmos_wind() -> Vec<i16> {
    let duration = 5.0;
    let samples = (SAMPLE_RATE * duration) as usize;
    let mut output = Vec::with_capacity(samples);
    let mut rng = SimpleRng::new(33333);

    let mut lp1 = Biquad::new();
    let mut lp2 = Biquad::new();
    let mut hp = Biquad::new();
    lp1.set_lowpass(600.0, 0.7);
    lp2.set_lowpass(400.0, 0.7);
    hp.set_highpass(80.0, 0.7);

    for i in 0..samples {
        let t = i as f32 / SAMPLE_RATE;
        let noise = rng.next_f32() * 2.0 - 1.0;

        // Complex modulation for organic movement
        let lfo1 = (t * 0.23 * TWO_PI).sin();
        let lfo2 = (t * 0.11 * TWO_PI).sin();
        let lfo3 = (t * 0.07 * TWO_PI).sin();

        let cutoff = 350.0 + 150.0 * lfo1 + 80.0 * lfo2 + 40.0 * lfo3;
        lp1.set_lowpass(cutoff, 0.8);

        let filtered = hp.process(lp2.process(lp1.process(noise)));

        let env = if t < 0.8 {
            (t / 0.8).powf(2.0)
        } else if t < 4.2 {
            1.0
        } else {
            (-(t - 4.2) * 1.5).exp()
        };

        let sample = filtered * env * 22000.0;
        output.push(sample.clamp(-32767.0, 32767.0) as i16);
    }

    output
}

// ============================================================================
// Dark Hit - Low impact accent
// ============================================================================

pub fn generate_hit_dark() -> Vec<i16> {
    let duration = 1.5;
    let freq = 55.0; // Low A1 fundamental
    let samples = (SAMPLE_RATE * duration) as usize;
    let mut output = Vec::with_capacity(samples);
    let mut rng = SimpleRng::new(44444);

    let mut phase = 0.0f32;
    let mut lp = Biquad::new();
    lp.set_lowpass(200.0, 0.8);

    for i in 0..samples {
        let t = i as f32 / SAMPLE_RATE;

        // Pitch drop on attack (impact characteristic)
        let pitch_env = if t < 0.05 { 1.5 - t * 10.0 } else { 1.0 };
        let actual_freq = freq * pitch_env;

        phase += actual_freq / SAMPLE_RATE;
        if phase >= 1.0 {
            phase -= 1.0;
        }

        // Sine with subtle noise layer
        let sine = (phase * TWO_PI).sin();
        let noise = (rng.next_f32() * 2.0 - 1.0) * 0.1;

        let filtered = lp.process(sine + noise);

        // Fast attack, exponential decay
        let env = if t < 0.01 { t / 0.01 } else { (-t * 3.0).exp() };

        let sample = filtered * env * 28000.0;
        output.push(sample.clamp(-32767.0, 32767.0) as i16);
    }

    output
}

// ============================================================================
// Echo Lead - Delayed ethereal melody voice
// ============================================================================

pub fn generate_lead_echo() -> Vec<i16> {
    let duration = 2.5;
    let freq = 293.66; // D4
    let samples = (SAMPLE_RATE * duration) as usize;
    let mut output = Vec::with_capacity(samples);

    let mut phases = [0.0f32; 2];
    let detune = [0.995, 1.005]; // Wider detune for washy echo
    let mut vibrato_phase = 0.0f32;

    let mut svf = StateVariableFilter::new();

    // Simple delay buffer for echo effect
    let delay_samples = (SAMPLE_RATE * 0.35) as usize; // 350ms delay
    let mut delay_buffer = vec![0.0f32; delay_samples];
    let mut delay_idx = 0usize;

    for i in 0..samples {
        let t = i as f32 / SAMPLE_RATE;

        // Slow vibrato
        let vib_depth = if t > 0.3 {
            ((t - 0.3) / 0.5).min(1.0)
        } else {
            0.0
        };
        vibrato_phase += 3.5 / SAMPLE_RATE;
        let vibrato = 1.0 + 0.012 * vib_depth * (vibrato_phase * TWO_PI).sin();

        let actual_freq = freq * vibrato;

        // Detuned triangles for softer character
        let mut sum = 0.0f32;
        for (j, d) in detune.iter().enumerate() {
            phases[j] += actual_freq * d / SAMPLE_RATE;
            if phases[j] >= 1.0 {
                phases[j] -= 1.0;
            }
            let tri = if phases[j] < 0.5 {
                4.0 * phases[j] - 1.0
            } else {
                3.0 - 4.0 * phases[j]
            };
            sum += tri;
        }
        sum /= 2.0;

        // Very soft filtering
        let (low, _, _) = svf.process(sum, 0.06, 0.15);

        // Add delayed signal (echo)
        let delayed = delay_buffer[delay_idx];
        delay_buffer[delay_idx] = low;
        delay_idx = (delay_idx + 1) % delay_samples;

        let mixed = low + delayed * 0.5; // 50% wet echo

        let env = if t < 0.15 {
            (t / 0.15).powf(1.5)
        } else if t < 1.8 {
            1.0 - (t - 0.15) * 0.05
        } else {
            0.9 * (-(t - 1.8) * 2.0).exp()
        };

        let sample = mixed * env * 24000.0;
        output.push(sample.clamp(-32767.0, 32767.0) as i16);
    }

    output
}
