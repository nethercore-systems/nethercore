//! Plucked instruments and vocal: Harp, Piano, Choir

use super::super::common::{SimpleRng, SAMPLE_RATE};
use super::{blep_saw, soft_saturate, BiquadLP, FormantFilter};
use std::f32::consts::PI;

/// Harp: Crystalline plucked strings
pub fn generate_harp_gliss() -> Vec<i16> {
    let duration = 1.5;
    let freq = 440.0; // A4
    let samples = (SAMPLE_RATE * duration) as usize;
    let mut output = Vec::with_capacity(samples);

    let mut phase = 0.0f32;
    let mut filter = BiquadLP::new(6000.0, 0.5);

    let _dt = freq / SAMPLE_RATE;

    for i in 0..samples {
        let t = i as f32 / SAMPLE_RATE;

        phase += freq / SAMPLE_RATE;
        phase %= 1.0;

        // Triangle + sine for plucked character
        let tri = if phase < 0.5 {
            4.0 * phase - 1.0
        } else {
            3.0 - 4.0 * phase
        };
        let sine = (phase * 2.0 * PI).sin();
        let tone = tri * 0.6 + sine * 0.4;

        // Filter darkens over time (string damping)
        let cutoff = 8000.0 * (-t * 2.5).exp() + 800.0;
        filter.set_params(cutoff, 0.5);
        let filtered = filter.process(tone);

        // Fast attack, smooth decay
        let env = if t < 0.004 {
            t / 0.004
        } else {
            (-t * 3.0).exp()
        };

        let sample = filtered * env * 30000.0;
        output.push(sample.clamp(-32767.0, 32767.0) as i16);
    }

    output
}

/// Piano: Rich hammer sound with decaying harmonics
pub fn generate_piano() -> Vec<i16> {
    let duration = 2.0;
    let freq = 293.66; // D4
    let samples = (SAMPLE_RATE * duration) as usize;
    let mut output = Vec::with_capacity(samples);

    let mut phases = [0.0f32; 6];
    let harmonics = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0];
    let amplitudes = [1.0, 0.5, 0.33, 0.25, 0.15, 0.1];

    let mut filter = BiquadLP::new(4000.0, 0.7);
    let mut rng = SimpleRng::new(11111);

    for i in 0..samples {
        let t = i as f32 / SAMPLE_RATE;

        let mut sum = 0.0f32;
        for (j, (harm, amp)) in harmonics.iter().zip(amplitudes.iter()).enumerate() {
            phases[j] += freq * harm / SAMPLE_RATE;
            phases[j] %= 1.0;
            // Each harmonic decays at different rate
            let harm_decay = (-t * (1.5 + j as f32 * 0.5)).exp();
            sum += (phases[j] * 2.0 * PI).sin() * amp * harm_decay;
        }
        sum /= 2.0;

        // Hammer transient (noise + click)
        let hammer_noise = rng.next_f32() * 2.0 - 1.0;
        let hammer_env = if t < 0.002 {
            t / 0.002
        } else {
            (-t * 80.0).exp()
        };
        let hammer = hammer_noise * hammer_env * 0.2;

        // Filter darkens over time
        let cutoff = 5000.0 * (-t * 1.5).exp() + 1000.0;
        filter.set_params(cutoff, 0.7);
        let filtered = filter.process(sum + hammer);

        let env = if t < 0.003 {
            t / 0.003
        } else {
            (-t * 1.8).exp()
        };

        let sample = filtered * env * 31000.0;
        output.push(sample.clamp(-32767.0, 32767.0) as i16);
    }

    output
}

/// Choir "ah": Lush ensemble with formant filtering
pub fn generate_choir_ah() -> Vec<i16> {
    let duration = 3.0;
    let freq = 293.66; // D4
    let samples = (SAMPLE_RATE * duration) as usize;
    let mut output = Vec::with_capacity(samples);

    // 8 voices for rich choir
    let detune = [-15.0, -8.0, -4.0, -1.0, 1.0, 4.0, 8.0, 15.0];
    let mut phases = [0.0f32; 8];

    let mut formant = FormantFilter::new();
    let mut vibrato_phase = 0.0f32;

    let dt = freq / SAMPLE_RATE;

    for i in 0..samples {
        let t = i as f32 / SAMPLE_RATE;

        // Slow, subtle vibrato
        let vib_amount = (t / 0.4).min(1.0) * 0.004;
        let vibrato = 1.0 + vib_amount * (vibrato_phase * 2.0 * PI).sin();
        vibrato_phase += 5.0 / SAMPLE_RATE;

        let mut sum = 0.0f32;
        for (j, &cents) in detune.iter().enumerate() {
            // Slight random variation per voice
            let freq_mult = (2.0f32).powf(cents / 1200.0) * vibrato;
            phases[j] += freq * freq_mult / SAMPLE_RATE;
            phases[j] %= 1.0;
            sum += blep_saw(phases[j], dt);
        }
        sum /= 8.0;

        // "ah" vowel formant
        let filtered = formant.process(sum, 0.0);

        let saturated = soft_saturate(filtered * 2.0) * 0.55;

        // Very slow attack for vocal quality
        let env = if t < 0.2 {
            (t / 0.2).powf(2.0)
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

/// Choir "oh": Rounder, darker vowel
pub fn generate_choir_oh() -> Vec<i16> {
    let duration = 3.0;
    let freq = 293.66; // D4
    let samples = (SAMPLE_RATE * duration) as usize;
    let mut output = Vec::with_capacity(samples);

    let detune = [-15.0, -8.0, -4.0, -1.0, 1.0, 4.0, 8.0, 15.0];
    let mut phases = [0.0f32; 8];

    let mut formant = FormantFilter::new();
    let mut vibrato_phase = 0.0f32;

    let dt = freq / SAMPLE_RATE;

    for i in 0..samples {
        let t = i as f32 / SAMPLE_RATE;

        let vib_amount = (t / 0.4).min(1.0) * 0.004;
        let vibrato = 1.0 + vib_amount * (vibrato_phase * 2.0 * PI).sin();
        vibrato_phase += 5.2 / SAMPLE_RATE;

        let mut sum = 0.0f32;
        for (j, &cents) in detune.iter().enumerate() {
            let freq_mult = (2.0f32).powf(cents / 1200.0) * vibrato;
            phases[j] += freq * freq_mult / SAMPLE_RATE;
            phases[j] %= 1.0;
            sum += blep_saw(phases[j], dt);
        }
        sum /= 8.0;

        // "oh" vowel formant
        let filtered = formant.process(sum, 0.5);

        let saturated = soft_saturate(filtered * 2.0) * 0.55;

        let env = if t < 0.2 {
            (t / 0.2).powf(2.0)
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
