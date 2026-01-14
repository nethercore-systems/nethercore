//! Roland TR-909 drum machine sounds

use super::super::common::{SimpleRng, SAMPLE_RATE};
use super::filters::{soft_saturate, StateVariableFilter, BiquadLP};
use std::f32::consts::PI;

const TWO_PI: f32 = 2.0 * PI;

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
        if phase >= 1.0 {
            phase -= 1.0;
        }

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
        if phase >= 1.0 {
            phase -= 1.0;
        }

        let body = (phase * TWO_PI).sin();
        let body_env = (-t * 28.0).exp();

        // === NOISE CRACK (multi-layered) ===
        let noise_raw = rng.next_f32() * 2.0 - 1.0;
        let (_, band, high) = noise_hp.process(noise_raw, 0.42, 0.25);
        let noise = noise_lp.process(band * 0.6 + high * 0.4);

        // Multi-tap envelope for clap character
        let layer1 = if t < 0.002 {
            (t / 0.002).powf(0.5)
        } else {
            (-t * 35.0).exp()
        };
        let layer2 = if (0.003..0.006).contains(&t) {
            ((t - 0.003) / 0.003).powf(0.5) * 0.7
        } else if t >= 0.006 {
            (-((t - 0.006) * 30.0)).exp() * 0.7
        } else {
            0.0
        };
        let layer3 = if (0.008..0.012).contains(&t) {
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
