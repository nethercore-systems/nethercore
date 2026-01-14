//! Brass and woodwind instruments: Horn, Trumpet, Flute

use super::super::common::{SimpleRng, SAMPLE_RATE};
use super::{blep_saw, blep_square, soft_saturate, BiquadLP};
use std::f32::consts::PI;

/// French horn: Warm, mellow brass with resonant filter
pub fn generate_brass_horn() -> Vec<i16> {
    let duration = 2.0;
    let freq = 146.83; // D3
    let samples = (SAMPLE_RATE * duration) as usize;
    let mut output = Vec::with_capacity(samples);

    let detune = [-6.0, -2.0, 2.0, 6.0];
    let mut phases = [0.0f32; 4];

    let mut filter = BiquadLP::new(1000.0, 1.5); // Resonant for horn character
    let mut vibrato_phase = 0.0f32;

    let dt = freq / SAMPLE_RATE;

    for i in 0..samples {
        let t = i as f32 / SAMPLE_RATE;

        let vib_amount = (t / 0.3).min(1.0) * 0.003;
        let vibrato = 1.0 + vib_amount * (vibrato_phase * 2.0 * PI).sin();
        vibrato_phase += 5.0 / SAMPLE_RATE;

        // Mix of square and saw for horn character
        let mut sum = 0.0f32;
        for (j, &cents) in detune.iter().enumerate() {
            let freq_mult = (2.0f32).powf(cents / 1200.0) * vibrato;
            phases[j] += freq * freq_mult / SAMPLE_RATE;
            phases[j] %= 1.0;
            let sq = blep_square(phases[j], dt);
            let saw = blep_saw(phases[j], dt);
            sum += sq * 0.6 + saw * 0.4;
        }
        sum /= 4.0;

        // Filter opens during attack for "blat"
        let cutoff = 600.0 + 800.0 * (1.0 - (-t * 5.0).exp());
        filter.set_params(cutoff, 1.5);
        let filtered = filter.process(sum);

        let saturated = soft_saturate(filtered * 1.6) * 0.65;

        let env = if t < 0.08 {
            (t / 0.08).powf(1.3)
        } else if t < 1.5 {
            1.0
        } else {
            (-(t - 1.5) * 2.0).exp()
        };

        let sample = saturated * env * 30000.0;
        output.push(sample.clamp(-32767.0, 32767.0) as i16);
    }

    output
}

/// Trumpet: Bright, punchy brass for fanfares
pub fn generate_brass_trumpet() -> Vec<i16> {
    let duration = 1.5;
    let freq = 587.33; // D5
    let samples = (SAMPLE_RATE * duration) as usize;
    let mut output = Vec::with_capacity(samples);

    let detune = [-4.0, 0.0, 4.0];
    let mut phases = [0.0f32; 3];

    let mut filter = BiquadLP::new(3000.0, 1.2);

    let dt = freq / SAMPLE_RATE;

    for i in 0..samples {
        let t = i as f32 / SAMPLE_RATE;

        let mut sum = 0.0f32;
        for (j, &cents) in detune.iter().enumerate() {
            let freq_mult = (2.0f32).powf(cents / 1200.0);
            phases[j] += freq * freq_mult / SAMPLE_RATE;
            phases[j] %= 1.0;
            sum += blep_saw(phases[j], dt);
        }
        sum /= 3.0;

        // Bright attack that mellows
        let cutoff = 4000.0 + 2000.0 * (-t * 4.0).exp();
        filter.set_params(cutoff, 1.2);
        let filtered = filter.process(sum);

        let saturated = soft_saturate(filtered * 1.5) * 0.7;

        let env = if t < 0.025 {
            (t / 0.025).powf(0.8)
        } else if t < 1.0 {
            1.0 - (t - 0.025) * 0.05
        } else {
            0.95 * (-(t - 1.0) * 3.0).exp()
        };

        let sample = saturated * env * 31000.0;
        output.push(sample.clamp(-32767.0, 32767.0) as i16);
    }

    output
}

/// Flute: Pure, airy tone with subtle breath noise
pub fn generate_flute() -> Vec<i16> {
    let duration = 2.0;
    let freq = 587.33; // D5
    let samples = (SAMPLE_RATE * duration) as usize;
    let mut output = Vec::with_capacity(samples);

    let mut phase = 0.0f32;
    let mut vibrato_phase = 0.0f32;
    let mut rng = SimpleRng::new(54321);
    let mut noise_filter = BiquadLP::new(8000.0, 0.5);

    for i in 0..samples {
        let t = i as f32 / SAMPLE_RATE;

        // Light vibrato
        let vibrato = 1.0 + 0.004 * (vibrato_phase * 2.0 * PI).sin();
        vibrato_phase += 6.0 / SAMPLE_RATE;

        phase += freq * vibrato / SAMPLE_RATE;
        phase %= 1.0;

        // Mix of sine and triangle for flute tone
        let sine = (phase * 2.0 * PI).sin();
        let tri = if phase < 0.5 {
            4.0 * phase - 1.0
        } else {
            3.0 - 4.0 * phase
        };
        let tone = sine * 0.7 + tri * 0.3;

        // Subtle breath noise
        let noise = rng.next_f32() * 2.0 - 1.0;
        let filtered_noise = noise_filter.process(noise);

        // Breath more prominent during attack
        let breath_amount = 0.05 + 0.1 * (-t * 8.0).exp();
        let mix = tone + filtered_noise * breath_amount;

        let env = if t < 0.06 {
            (t / 0.06).powf(1.2)
        } else if t < 1.7 {
            1.0
        } else {
            (-(t - 1.7) * 3.5).exp()
        };

        let sample = mix * env * 28000.0;
        output.push(sample.clamp(-32767.0, 32767.0) as i16);
    }

    output
}
