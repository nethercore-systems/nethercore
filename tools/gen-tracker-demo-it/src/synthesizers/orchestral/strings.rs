//! String instruments: Cello, Viola, Violin

use super::super::common::SAMPLE_RATE;
use super::{blep_saw, soft_saturate, BiquadLP};
use std::f32::consts::PI;

/// Cello: Rich ensemble of detuned saws with warm filtering
pub fn generate_strings_cello() -> Vec<i16> {
    let duration = 3.0;
    let freq = 146.83; // D3
    let samples = (SAMPLE_RATE * duration) as usize;
    let mut output = Vec::with_capacity(samples);

    // 6 detuned oscillators for richness
    let detune = [-12.0, -5.0, -2.0, 2.0, 5.0, 12.0]; // cents
    let mut phases = [0.0f32; 6];

    let mut filter = BiquadLP::new(1200.0, 0.7);
    let mut vibrato_phase = 0.0f32;

    let dt = freq / SAMPLE_RATE;

    for i in 0..samples {
        let t = i as f32 / SAMPLE_RATE;

        // Subtle vibrato that develops over time
        let vib_amount = (t / 0.5).min(1.0) * 0.004;
        let vibrato = 1.0 + vib_amount * (vibrato_phase * 2.0 * PI).sin();
        vibrato_phase += 5.5 / SAMPLE_RATE;

        let mut sum = 0.0f32;
        for (j, &cents) in detune.iter().enumerate() {
            let freq_mult = (2.0f32).powf(cents / 1200.0) * vibrato;
            phases[j] += freq * freq_mult / SAMPLE_RATE;
            phases[j] %= 1.0;
            sum += blep_saw(phases[j], dt);
        }
        sum /= 6.0;

        // Warm lowpass that opens slightly over attack
        let cutoff = 800.0 + 600.0 * (1.0 - (-t * 3.0).exp());
        filter.set_params(cutoff, 0.7);
        let filtered = filter.process(sum);

        // Apply soft saturation for warmth
        let saturated = soft_saturate(filtered * 1.5) * 0.7;

        // Smooth envelope: slow attack, sustain, gentle release
        let env = if t < 0.12 {
            (t / 0.12).powf(2.0)
        } else if t < 2.5 {
            1.0
        } else {
            (-(t - 2.5) * 2.0).exp()
        };

        let sample = saturated * env * 31000.0;
        output.push(sample.clamp(-32767.0, 32767.0) as i16);
    }

    output
}

/// Viola: Mid-range strings, slightly brighter than cello
pub fn generate_strings_viola() -> Vec<i16> {
    let duration = 3.0;
    let freq = 293.66; // D4
    let samples = (SAMPLE_RATE * duration) as usize;
    let mut output = Vec::with_capacity(samples);

    let detune = [-10.0, -4.0, -1.0, 1.0, 4.0, 10.0];
    let mut phases = [0.0f32; 6];

    let mut filter = BiquadLP::new(2000.0, 0.6);
    let mut vibrato_phase = 0.0f32;

    let dt = freq / SAMPLE_RATE;

    for i in 0..samples {
        let t = i as f32 / SAMPLE_RATE;

        let vib_amount = (t / 0.4).min(1.0) * 0.005;
        let vibrato = 1.0 + vib_amount * (vibrato_phase * 2.0 * PI).sin();
        vibrato_phase += 5.8 / SAMPLE_RATE;

        let mut sum = 0.0f32;
        for (j, &cents) in detune.iter().enumerate() {
            let freq_mult = (2.0f32).powf(cents / 1200.0) * vibrato;
            phases[j] += freq * freq_mult / SAMPLE_RATE;
            phases[j] %= 1.0;
            sum += blep_saw(phases[j], dt);
        }
        sum /= 6.0;

        let cutoff = 1200.0 + 1000.0 * (1.0 - (-t * 3.0).exp());
        filter.set_params(cutoff, 0.6);
        let filtered = filter.process(sum);

        let saturated = soft_saturate(filtered * 1.4) * 0.75;

        let env = if t < 0.10 {
            (t / 0.10).powf(2.0)
        } else if t < 2.5 {
            1.0
        } else {
            (-(t - 2.5) * 2.0).exp()
        };

        let sample = saturated * env * 30000.0;
        output.push(sample.clamp(-32767.0, 32767.0) as i16);
    }

    output
}

/// Violin: Brightest strings with expressive vibrato
pub fn generate_strings_violin() -> Vec<i16> {
    let duration = 3.0;
    let freq = 587.33; // D5
    let samples = (SAMPLE_RATE * duration) as usize;
    let mut output = Vec::with_capacity(samples);

    let detune = [-8.0, -3.0, 0.0, 3.0, 8.0];
    let mut phases = [0.0f32; 5];

    let mut filter = BiquadLP::new(3500.0, 0.5);
    let mut vibrato_phase = 0.0f32;

    let dt = freq / SAMPLE_RATE;

    for i in 0..samples {
        let t = i as f32 / SAMPLE_RATE;

        // Delayed vibrato that intensifies
        let vib_depth = if t < 0.15 {
            0.0
        } else {
            ((t - 0.15) / 0.3).min(1.0)
        };
        let vibrato = 1.0 + 0.008 * vib_depth * (vibrato_phase * 2.0 * PI).sin();
        vibrato_phase += 5.5 / SAMPLE_RATE;

        let mut sum = 0.0f32;
        for (j, &cents) in detune.iter().enumerate() {
            let freq_mult = (2.0f32).powf(cents / 1200.0) * vibrato;
            phases[j] += freq * freq_mult / SAMPLE_RATE;
            phases[j] %= 1.0;
            sum += blep_saw(phases[j], dt);
        }
        sum /= 5.0;

        let cutoff = 2000.0 + 2000.0 * (1.0 - (-t * 4.0).exp());
        filter.set_params(cutoff, 0.5);
        let filtered = filter.process(sum);

        let saturated = soft_saturate(filtered * 1.3) * 0.8;

        let env = if t < 0.08 {
            (t / 0.08).powf(1.8)
        } else if t < 2.5 {
            1.0
        } else {
            (-(t - 2.5) * 2.0).exp()
        };

        let sample = saturated * env * 29000.0;
        output.push(sample.clamp(-32767.0, 32767.0) as i16);
    }

    output
}
