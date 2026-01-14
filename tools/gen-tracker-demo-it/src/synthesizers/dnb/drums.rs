//! Drum and Bass drum/percussion instruments

use super::super::common::{SimpleRng, SAMPLE_RATE};
use super::dsp::{soft_clip, soft_saturate, BiquadLP, StateVariableFilter};
use std::f32::consts::PI;

const TWO_PI: f32 = 2.0 * PI;

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
        if phase >= 1.0 {
            phase -= 1.0;
        }

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
        if phase >= 1.0 {
            phase -= 1.0;
        }

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

        let env = if t < 0.003 {
            t / 0.003
        } else {
            (-t * 25.0).exp()
        };
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
    let freqs = [
        2800.0, 3700.0, 4500.0, 5200.0, 6100.0, 7300.0, 8800.0, 10500.0,
    ];

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
        let env = if t < 0.002 {
            t / 0.002
        } else {
            (-t * 3.0).exp()
        };
        let sample = mix * env * 26000.0;
        output.push(sample.clamp(-32767.0, 32767.0) as i16);
    }

    output
}
