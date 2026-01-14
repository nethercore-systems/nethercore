//! Drum and Bass effects and atmospheres

use super::super::common::{SimpleRng, SAMPLE_RATE};
use super::dsp::{BiquadLP, StateVariableFilter};
use std::f32::consts::PI;

const TWO_PI: f32 = 2.0 * PI;

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
        if phase >= 1.0 {
            phase -= 1.0;
        }
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
        if phase >= 1.0 {
            phase -= 1.0;
        }
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
