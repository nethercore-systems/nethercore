//! Drum and Bass instrument synthesis - HIGH QUALITY
//!
//! Instruments for "Nether Storm" - DnB at 174 BPM in F minor/Phrygian
//! Features: Band-limited oscillators, proper filters, punchy envelopes

use super::common::{SimpleRng, SAMPLE_RATE};
use std::f32::consts::PI;

const TWO_PI: f32 = 2.0 * PI;

// ============================================================================
// DSP Utilities
// ============================================================================

/// Soft saturation for warmth
fn soft_saturate(x: f32) -> f32 {
    x.tanh()
}

/// Hard clip with soft knee
fn soft_clip(x: f32, threshold: f32) -> f32 {
    if x.abs() < threshold {
        x
    } else {
        x.signum() * (threshold + (1.0 - threshold) * soft_saturate((x.abs() - threshold) / (1.0 - threshold)))
    }
}

/// PolyBLEP for anti-aliased discontinuities
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

/// Band-limited sawtooth
fn blep_saw(phase: f32, dt: f32) -> f32 {
    let naive = 2.0 * phase - 1.0;
    naive - poly_blep(phase, dt)
}

/// Band-limited square
fn blep_square(phase: f32, dt: f32) -> f32 {
    let naive = if phase < 0.5 { 1.0 } else { -1.0 };
    naive + poly_blep(phase, dt) - poly_blep((phase + 0.5) % 1.0, dt)
}

/// 2-pole state variable filter
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

/// Biquad low-pass filter
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

// ============================================================================
// DnB Kick - Punchy with sub and transient
// ============================================================================

pub fn generate_kick_dnb() -> Vec<i16> {
    let duration = 0.35;
    let samples = (SAMPLE_RATE * duration) as usize;
    let mut output = Vec::with_capacity(samples);
    let mut rng = SimpleRng::new(12345);
    let mut phase = 0.0f32;
    let mut click_lp = BiquadLP::new();
    click_lp.set_params(3000.0, 0.7);

    for i in 0..samples {
        let t = i as f32 / SAMPLE_RATE;

        // === CLICK/TRANSIENT (first 8ms) ===
        let click = if t < 0.008 {
            let noise = rng.next_f32() * 2.0 - 1.0;
            let env = (1.0 - t / 0.008).powf(0.5);
            click_lp.process(noise) * env * 1.2
        } else {
            0.0
        };

        // === PITCH ENVELOPE ===
        // Start at 150Hz, drop to 45Hz exponentially
        let pitch_env = (-t * 35.0).exp();
        let freq = 45.0 + 120.0 * pitch_env;

        // === SUB BODY ===
        phase += freq / SAMPLE_RATE;
        if phase >= 1.0 { phase -= 1.0; }

        // Pure sine for sub, with slight harmonic
        let body = (phase * TWO_PI).sin() + 0.15 * (phase * TWO_PI * 2.0).sin();

        // === AMPLITUDE ENVELOPE ===
        let env = if t < 0.004 {
            (t / 0.004).powf(0.3) // Fast attack
        } else {
            (-t * 8.0).exp() // Natural decay
        };

        // === MIX AND OUTPUT ===
        let sample = (click * 0.5 + body * env) * 0.9;
        let sample = soft_clip(sample, 0.85) * 32000.0;
        output.push(sample.clamp(-32767.0, 32767.0) as i16);
    }

    output
}

// ============================================================================
// DnB Snare - Punchy body + crisp noise
// ============================================================================

pub fn generate_snare_dnb() -> Vec<i16> {
    let duration = 0.28;
    let samples = (SAMPLE_RATE * duration) as usize;
    let mut output = Vec::with_capacity(samples);
    let mut rng = SimpleRng::new(54321);
    let mut phase = 0.0f32;
    let mut noise_hp = StateVariableFilter::new();
    let mut noise_lp = BiquadLP::new();
    noise_lp.set_params(8000.0, 0.5);

    for i in 0..samples {
        let t = i as f32 / SAMPLE_RATE;

        // === BODY TONE (200Hz pitched down) ===
        let body_freq = 200.0 * (-t * 15.0).exp() + 120.0;
        phase += body_freq / SAMPLE_RATE;
        if phase >= 1.0 { phase -= 1.0; }

        let body = (phase * TWO_PI).sin();
        let body_env = (-t * 25.0).exp();

        // === NOISE CRACK ===
        let noise_raw = rng.next_f32() * 2.0 - 1.0;
        let (_, band, high) = noise_hp.process(noise_raw, 0.4, 0.3);
        let noise = noise_lp.process(band * 0.6 + high * 0.4);

        let noise_env = if t < 0.003 {
            (t / 0.003).powf(0.5)
        } else {
            (-t * 20.0).exp()
        };

        // === MIX ===
        let sample = body * body_env * 0.5 + noise * noise_env * 0.7;
        let sample = soft_clip(sample, 0.8) * 32000.0;
        output.push(sample.clamp(-32767.0, 32767.0) as i16);
    }

    output
}

// ============================================================================
// DnB Closed Hi-Hat - Tight metallic
// ============================================================================

pub fn generate_hihat_closed() -> Vec<i16> {
    let duration = 0.08;
    let samples = (SAMPLE_RATE * duration) as usize;
    let mut output = Vec::with_capacity(samples);
    let mut rng = SimpleRng::new(11111);
    let mut hp = StateVariableFilter::new();
    let mut lp = BiquadLP::new();
    lp.set_params(12000.0, 0.7);

    // Metallic tones (inharmonic)
    let mut phases = [0.0f32; 6];
    let freqs = [3500.0, 4890.0, 6400.0, 7250.0, 8100.0, 9600.0];

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
        let (_, _, high) = hp.process(noise, 0.6, 0.2);
        let filtered = lp.process(high * 0.6 + metal * 0.4);

        let env = (-t * 60.0).exp();
        let sample = filtered * env * 28000.0;
        output.push(sample.clamp(-32767.0, 32767.0) as i16);
    }

    output
}

// ============================================================================
// DnB Open Hi-Hat - Longer sustain
// ============================================================================

pub fn generate_hihat_open() -> Vec<i16> {
    let duration = 0.35;
    let samples = (SAMPLE_RATE * duration) as usize;
    let mut output = Vec::with_capacity(samples);
    let mut rng = SimpleRng::new(22222);
    let mut hp = StateVariableFilter::new();
    let mut lp = BiquadLP::new();
    lp.set_params(11000.0, 0.6);

    let mut phases = [0.0f32; 6];
    let freqs = [3200.0, 4500.0, 6100.0, 7000.0, 8500.0, 10200.0];

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
        let (_, _, high) = hp.process(noise, 0.5, 0.25);
        let filtered = lp.process(high * 0.5 + metal * 0.5);

        let env = (-t * 10.0).exp();
        let sample = filtered * env * 26000.0;
        output.push(sample.clamp(-32767.0, 32767.0) as i16);
    }

    output
}

// ============================================================================
// DnB Break Slice - Crunchy transient
// ============================================================================

pub fn generate_break_slice() -> Vec<i16> {
    let duration = 0.15;
    let samples = (SAMPLE_RATE * duration) as usize;
    let mut output = Vec::with_capacity(samples);
    let mut rng = SimpleRng::new(33333);
    let mut lp = BiquadLP::new();
    lp.set_params(4000.0, 1.5); // Resonant for punch

    for i in 0..samples {
        let t = i as f32 / SAMPLE_RATE;
        let noise = rng.next_f32() * 2.0 - 1.0;

        // Resonant filter for crunch
        let freq = 4000.0 * (-t * 8.0).exp() + 800.0;
        lp.set_params(freq, 2.0);
        let filtered = lp.process(noise);

        let env = if t < 0.003 { t / 0.003 } else { (-t * 25.0).exp() };
        let sample = soft_saturate(filtered * 1.5) * env * 30000.0;
        output.push(sample.clamp(-32767.0, 32767.0) as i16);
    }

    output
}

// ============================================================================
// DnB Cymbal - Metallic shimmer
// ============================================================================

pub fn generate_cymbal() -> Vec<i16> {
    let duration = 1.2;
    let samples = (SAMPLE_RATE * duration) as usize;
    let mut output = Vec::with_capacity(samples);
    let mut rng = SimpleRng::new(44444);
    let mut hp = StateVariableFilter::new();

    let mut phases = [0.0f32; 8];
    let freqs = [2800.0, 3700.0, 4500.0, 5200.0, 6100.0, 7300.0, 8800.0, 10500.0];

    for i in 0..samples {
        let t = i as f32 / SAMPLE_RATE;

        // Rich metallic partials
        let mut metal = 0.0f32;
        for (j, freq) in freqs.iter().enumerate() {
            phases[j] += freq / SAMPLE_RATE;
            phases[j] %= 1.0;
            let decay = (-(j as f32) * 0.3 * t).exp();
            metal += (phases[j] * TWO_PI).sin() * decay;
        }
        metal /= 5.0;

        // Noise layer
        let noise = rng.next_f32() * 2.0 - 1.0;
        let (_, _, high) = hp.process(noise, 0.45, 0.2);

        let mix = metal * 0.6 + high * 0.4;
        let env = if t < 0.002 { t / 0.002 } else { (-t * 3.0).exp() };
        let sample = mix * env * 26000.0;
        output.push(sample.clamp(-32767.0, 32767.0) as i16);
    }

    output
}

// ============================================================================
// DnB Sub Bass - Clean powerful sub
// ============================================================================

pub fn generate_bass_sub_dnb() -> Vec<i16> {
    let duration = 0.6;
    let freq = 43.65; // F1 (sub)
    let samples = (SAMPLE_RATE * duration) as usize;
    let mut output = Vec::with_capacity(samples);
    let mut phase = 0.0f32;

    for i in 0..samples {
        let t = i as f32 / SAMPLE_RATE;

        phase += freq / SAMPLE_RATE;
        if phase >= 1.0 { phase -= 1.0; }

        // Pure sine with slight harmonic for presence
        let sub = (phase * TWO_PI).sin();
        let harm = (phase * TWO_PI * 2.0).sin() * 0.08;

        let env = if t < 0.015 {
            (t / 0.015).powf(0.5)
        } else if t < 0.4 {
            1.0
        } else {
            (-(t - 0.4) * 5.0).exp()
        };

        let sample = (sub + harm) * env * 31000.0;
        output.push(sample.clamp(-32767.0, 32767.0) as i16);
    }

    output
}

// ============================================================================
// DnB Reese Bass - Fat detuned growl
// ============================================================================

pub fn generate_bass_reese() -> Vec<i16> {
    let duration = 0.8;
    let freq = 87.31; // F2
    let samples = (SAMPLE_RATE * duration) as usize;
    let mut output = Vec::with_capacity(samples);

    let mut phases = [0.0f32; 4];
    let detune = [0.985, 0.995, 1.005, 1.015]; // Wide detune
    let dt = freq / SAMPLE_RATE;

    let mut svf = StateVariableFilter::new();

    for i in 0..samples {
        let t = i as f32 / SAMPLE_RATE;

        // 4 detuned saws
        let mut sum = 0.0f32;
        for (j, d) in detune.iter().enumerate() {
            phases[j] += freq * d / SAMPLE_RATE;
            if phases[j] >= 1.0 { phases[j] -= 1.0; }
            sum += blep_saw(phases[j], dt * d);
        }
        sum /= 4.0;

        // Modulating filter for movement
        let lfo = (t * 4.0 * TWO_PI).sin();
        let cutoff = 0.12 + 0.08 * (lfo * 0.5 + 0.5);
        let (low, band, _) = svf.process(sum, cutoff, 0.4);
        let filtered = low * 0.6 + band * 0.4;

        let env = if t < 0.02 {
            (t / 0.02).powf(0.7)
        } else if t < 0.5 {
            1.0
        } else {
            (-(t - 0.5) * 4.0).exp()
        };

        let sample = soft_saturate(filtered * 1.3) * env * 31000.0;
        output.push(sample.clamp(-32767.0, 32767.0) as i16);
    }

    output
}

// ============================================================================
// DnB Wobble Bass - LFO-modulated growl
// ============================================================================

pub fn generate_bass_wobble() -> Vec<i16> {
    let duration = 1.2;
    let freq = 87.31; // F2
    let samples = (SAMPLE_RATE * duration) as usize;
    let mut output = Vec::with_capacity(samples);

    let mut phases = [0.0f32; 3];
    let detune = [0.99, 1.0, 1.01];
    let dt = freq / SAMPLE_RATE;

    let mut svf = StateVariableFilter::new();

    for i in 0..samples {
        let t = i as f32 / SAMPLE_RATE;

        // Oscillators - mix of saw and square
        let mut sum = 0.0f32;
        for (j, d) in detune.iter().enumerate() {
            phases[j] += freq * d / SAMPLE_RATE;
            if phases[j] >= 1.0 { phases[j] -= 1.0; }
            let saw = blep_saw(phases[j], dt * d);
            let sq = blep_square(phases[j], dt * d);
            sum += saw * 0.6 + sq * 0.4;
        }
        sum /= 3.0;

        // LFO wobble (8Hz = half-bar at 174 BPM)
        let lfo = (t * 8.0 * TWO_PI).sin();
        let cutoff = 0.05 + 0.2 * (lfo * 0.5 + 0.5).powf(1.5);

        let (low, band, _) = svf.process(sum, cutoff, 0.6);
        let filtered = low * 0.5 + band * 0.5;

        let env = if t < 0.02 {
            (t / 0.02).powf(0.5)
        } else if t < 0.9 {
            1.0
        } else {
            (-(t - 0.9) * 4.0).exp()
        };

        let sample = soft_saturate(filtered * 1.5) * env * 30000.0;
        output.push(sample.clamp(-32767.0, 32767.0) as i16);
    }

    output
}

// ============================================================================
// DnB Dark Pad - Atmospheric layer
// ============================================================================

pub fn generate_pad_dark() -> Vec<i16> {
    let duration = 2.5;
    let freq = 174.61; // F3
    let samples = (SAMPLE_RATE * duration) as usize;
    let mut output = Vec::with_capacity(samples);

    let mut phases = [0.0f32; 5];
    let detune = [0.99, 0.995, 1.0, 1.005, 1.01];
    let dt = freq / SAMPLE_RATE;

    let mut lp1 = BiquadLP::new();
    let mut lp2 = BiquadLP::new();
    lp1.set_params(600.0, 0.7);
    lp2.set_params(400.0, 0.7);

    for i in 0..samples {
        let t = i as f32 / SAMPLE_RATE;

        // 5 detuned saws for thickness
        let mut sum = 0.0f32;
        for (j, d) in detune.iter().enumerate() {
            phases[j] += freq * d / SAMPLE_RATE;
            if phases[j] >= 1.0 { phases[j] -= 1.0; }
            sum += blep_saw(phases[j], dt * d);
        }
        sum /= 5.0;

        // Deep filtering
        let filtered = lp2.process(lp1.process(sum));

        let env = if t < 0.5 {
            (t / 0.5).powf(2.0)
        } else if t < 1.8 {
            1.0
        } else {
            (-(t - 1.8) * 2.0).exp()
        };

        let sample = filtered * env * 28000.0;
        output.push(sample.clamp(-32767.0, 32767.0) as i16);
    }

    output
}

// ============================================================================
// DnB Lead Stab - Punchy supersaw
// ============================================================================

pub fn generate_lead_stab() -> Vec<i16> {
    let duration = 0.35;
    let freq = 349.23; // F4
    let samples = (SAMPLE_RATE * duration) as usize;
    let mut output = Vec::with_capacity(samples);

    let mut phases = [0.0f32; 7];
    let detune = [0.97, 0.985, 0.995, 1.0, 1.005, 1.015, 1.03];
    let dt = freq / SAMPLE_RATE;

    let mut svf = StateVariableFilter::new();

    for i in 0..samples {
        let t = i as f32 / SAMPLE_RATE;

        // 7-voice supersaw
        let mut sum = 0.0f32;
        for (j, d) in detune.iter().enumerate() {
            phases[j] += freq * d / SAMPLE_RATE;
            if phases[j] >= 1.0 { phases[j] -= 1.0; }
            sum += blep_saw(phases[j], dt * d);
        }
        sum /= 5.0; // Slight boost

        // Filter envelope
        let cutoff = 0.08 + 0.35 * (-t * 15.0).exp();
        let (low, band, _) = svf.process(sum, cutoff, 0.3);
        let filtered = low * 0.7 + band * 0.3;

        let env = if t < 0.005 {
            (t / 0.005).powf(0.5)
        } else {
            (-t * 12.0).exp()
        };

        let sample = soft_saturate(filtered * 1.2) * env * 31000.0;
        output.push(sample.clamp(-32767.0, 32767.0) as i16);
    }

    output
}

// ============================================================================
// DnB Main Lead - Melodic saw lead
// ============================================================================

pub fn generate_lead_main() -> Vec<i16> {
    let duration = 1.2;
    let freq = 349.23; // F4
    let samples = (SAMPLE_RATE * duration) as usize;
    let mut output = Vec::with_capacity(samples);

    let mut phases = [0.0f32; 3];
    let detune = [0.995, 1.0, 1.005];
    let mut vibrato_phase = 0.0f32;

    let mut svf = StateVariableFilter::new();

    for i in 0..samples {
        let t = i as f32 / SAMPLE_RATE;

        // Delayed vibrato
        let vib_depth = if t > 0.15 { ((t - 0.15) / 0.2).min(1.0) } else { 0.0 };
        vibrato_phase += 5.5 / SAMPLE_RATE;
        let vibrato = 1.0 + 0.008 * vib_depth * (vibrato_phase * TWO_PI).sin();

        let actual_freq = freq * vibrato;
        let dt = actual_freq / SAMPLE_RATE;

        // 3 detuned saws + square
        let mut sum = 0.0f32;
        for (j, d) in detune.iter().enumerate() {
            phases[j] += actual_freq * d / SAMPLE_RATE;
            if phases[j] >= 1.0 { phases[j] -= 1.0; }
            sum += blep_saw(phases[j], dt * d);
        }
        sum = sum / 3.0 * 0.7 + blep_square(phases[1], dt) * 0.3;

        // Filter with slight movement
        let cutoff = 0.15 + 0.1 * (-t * 3.0).exp();
        let (low, band, _) = svf.process(sum, cutoff, 0.25);
        let filtered = low * 0.6 + band * 0.4;

        let env = if t < 0.02 {
            (t / 0.02).powf(0.7)
        } else if t < 0.8 {
            1.0 - (t - 0.02) * 0.1
        } else {
            0.9 * (-(t - 0.8) * 3.0).exp()
        };

        let sample = soft_saturate(filtered * 1.1) * env * 29000.0;
        output.push(sample.clamp(-32767.0, 32767.0) as i16);
    }

    output
}

// ============================================================================
// DnB Riser - Building tension
// ============================================================================

pub fn generate_fx_riser() -> Vec<i16> {
    let duration = 3.0;
    let samples = (SAMPLE_RATE * duration) as usize;
    let mut output = Vec::with_capacity(samples);
    let mut rng = SimpleRng::new(55555);
    let mut phase = 0.0f32;
    let mut svf = StateVariableFilter::new();

    for i in 0..samples {
        let t = i as f32 / SAMPLE_RATE;
        let progress = t / duration;

        // Rising sine
        let freq = 150.0 + 1500.0 * progress.powf(2.0);
        phase += freq / SAMPLE_RATE;
        if phase >= 1.0 { phase -= 1.0; }
        let sine = (phase * TWO_PI).sin();

        // Noise with rising filter
        let noise = rng.next_f32() * 2.0 - 1.0;
        let cutoff = 0.05 + 0.4 * progress;
        let (_, _, high) = svf.process(noise, cutoff, 0.3);

        let env = progress.powf(1.5);
        let sample = (sine * 0.5 + high * 0.3) * env * 28000.0;
        output.push(sample.clamp(-32767.0, 32767.0) as i16);
    }

    output
}

// ============================================================================
// DnB Impact - Massive hit
// ============================================================================

pub fn generate_fx_impact() -> Vec<i16> {
    let duration = 0.9;
    let samples = (SAMPLE_RATE * duration) as usize;
    let mut output = Vec::with_capacity(samples);
    let mut rng = SimpleRng::new(66666);
    let mut phase = 0.0f32;
    let mut lp = BiquadLP::new();

    for i in 0..samples {
        let t = i as f32 / SAMPLE_RATE;

        // Massive pitch drop
        let freq = 150.0 * (-t * 12.0).exp() + 30.0;
        phase += freq / SAMPLE_RATE;
        if phase >= 1.0 { phase -= 1.0; }
        let sub = (phase * TWO_PI).sin();

        // Noise burst
        let noise = rng.next_f32() * 2.0 - 1.0;
        lp.set_params(2000.0 * (-t * 8.0).exp() + 200.0, 0.7);
        let filtered_noise = lp.process(noise);

        let sub_env = (-t * 2.5).exp();
        let noise_env = (-t * 12.0).exp();

        let sample = (sub * sub_env * 0.8 + filtered_noise * noise_env * 0.5) * 32000.0;
        output.push(sample.clamp(-32767.0, 32767.0) as i16);
    }

    output
}

// ============================================================================
// DnB Atmosphere - Dark ambient texture
// ============================================================================

pub fn generate_atmos_storm() -> Vec<i16> {
    let duration = 3.5;
    let samples = (SAMPLE_RATE * duration) as usize;
    let mut output = Vec::with_capacity(samples);
    let mut rng = SimpleRng::new(77777);
    let mut lp1 = BiquadLP::new();
    let mut lp2 = BiquadLP::new();
    lp1.set_params(800.0, 0.7);
    lp2.set_params(400.0, 0.7);

    for i in 0..samples {
        let t = i as f32 / SAMPLE_RATE;
        let noise = rng.next_f32() * 2.0 - 1.0;

        // Modulating filter for organic movement
        let lfo1 = (t * 0.3 * TWO_PI).sin();
        let lfo2 = (t * 0.17 * TWO_PI).sin();
        let freq = 500.0 + 300.0 * lfo1 + 150.0 * lfo2;
        lp1.set_params(freq, 0.8);

        let filtered = lp2.process(lp1.process(noise));

        let env = if t < 0.5 {
            (t / 0.5).powf(1.5)
        } else if t < 3.0 {
            1.0
        } else {
            (-(t - 3.0) * 2.5).exp()
        };

        let sample = filtered * env * 18000.0;
        output.push(sample.clamp(-32767.0, 32767.0) as i16);
    }

    output
}
