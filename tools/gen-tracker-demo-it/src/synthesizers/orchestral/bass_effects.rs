//! Bass, pads, and effects: Epic bass, Orchestra pad, Epic FX

use super::super::common::{SimpleRng, SAMPLE_RATE};
use super::{blep_saw, soft_saturate, BiquadLP};
use std::f32::consts::PI;

/// Epic bass: Powerful sub with overtones
pub fn generate_bass_epic() -> Vec<i16> {
    let duration = 2.0;
    let freq = 73.42; // D2
    let samples = (SAMPLE_RATE * duration) as usize;
    let mut output = Vec::with_capacity(samples);

    let mut phase_sub = 0.0f32;
    let mut phases = [0.0f32; 3];
    let detune = [-5.0, 0.0, 5.0];

    let mut filter = BiquadLP::new(600.0, 1.2);

    let dt = freq / SAMPLE_RATE;

    for i in 0..samples {
        let t = i as f32 / SAMPLE_RATE;

        // Pure sub sine
        phase_sub += freq * 0.5 / SAMPLE_RATE;
        let sub = (phase_sub * 2.0 * PI).sin();

        // Detuned saws an octave up
        let mut saw_sum = 0.0f32;
        for (j, &cents) in detune.iter().enumerate() {
            let freq_mult = (2.0f32).powf(cents / 1200.0);
            phases[j] += freq * freq_mult / SAMPLE_RATE;
            phases[j] %= 1.0;
            saw_sum += blep_saw(phases[j], dt);
        }
        saw_sum /= 3.0;

        let mix = sub * 0.6 + saw_sum * 0.4;

        // Resonant filter adds growl
        let cutoff = 400.0 + 400.0 * (1.0 - (-t * 5.0).exp());
        filter.set_params(cutoff, 1.2);
        let filtered = filter.process(mix);

        let saturated = soft_saturate(filtered * 1.8) * 0.6;

        let env = if t < 0.05 {
            (t / 0.05).powf(1.2)
        } else if t < 1.5 {
            1.0
        } else {
            (-(t - 1.5) * 2.5).exp()
        };

        let sample = saturated * env * 31000.0;
        output.push(sample.clamp(-32767.0, 32767.0) as i16);
    }

    output
}

/// Orchestra pad: Lush background texture
pub fn generate_pad_orchestra() -> Vec<i16> {
    let duration = 3.0;
    let freq = 293.66; // D4
    let samples = (SAMPLE_RATE * duration) as usize;
    let mut output = Vec::with_capacity(samples);

    // 7 voices for rich pad
    let detune = [-15.0, -8.0, -3.0, 0.0, 3.0, 8.0, 15.0];
    let mut phases = [0.0f32; 7];

    let mut filter = BiquadLP::new(1500.0, 0.5);
    let mut lfo_phase = 0.0f32;

    let dt = freq / SAMPLE_RATE;

    for i in 0..samples {
        let t = i as f32 / SAMPLE_RATE;

        // Slow filter modulation
        let lfo = 0.5 + 0.5 * (lfo_phase * 2.0 * PI).sin();
        lfo_phase += 0.1 / SAMPLE_RATE;

        let mut sum = 0.0f32;
        for (j, &cents) in detune.iter().enumerate() {
            let freq_mult = (2.0f32).powf(cents / 1200.0);
            phases[j] += freq * freq_mult / SAMPLE_RATE;
            phases[j] %= 1.0;
            sum += blep_saw(phases[j], dt);
        }
        sum /= 7.0;

        // Slowly modulating filter
        let cutoff = 800.0 + 800.0 * lfo;
        filter.set_params(cutoff, 0.5);
        let filtered = filter.process(sum);

        let saturated = soft_saturate(filtered * 1.3) * 0.8;

        // Very slow attack/release for pad character
        let env = if t < 0.4 {
            (t / 0.4).powf(2.0)
        } else if t < 2.5 {
            1.0
        } else {
            (-(t - 2.5) * 2.0).exp()
        };

        let sample = saturated * env * 26000.0;
        output.push(sample.clamp(-32767.0, 32767.0) as i16);
    }

    output
}

/// Epic FX: Cinematic riser/impact
pub fn generate_fx_epic() -> Vec<i16> {
    let duration = 2.0;
    let samples = (SAMPLE_RATE * duration) as usize;
    let mut output = Vec::with_capacity(samples);
    let mut rng = SimpleRng::new(12121);

    let mut phase = 0.0f32;
    let mut noise_filter = BiquadLP::new(2000.0, 1.0);
    let mut tone_filter = BiquadLP::new(1000.0, 2.0);

    for i in 0..samples {
        let t = i as f32 / SAMPLE_RATE;

        // Rising pitch
        let freq = 80.0 + t * t * 300.0; // Exponential rise
        phase += freq / SAMPLE_RATE;
        let sine = (phase * 2.0 * PI).sin();

        // Add harmonics for fullness
        let h2 = (phase * 2.0 * 2.0 * PI).sin() * 0.5;
        let h3 = (phase * 3.0 * 2.0 * PI).sin() * 0.3;
        let tone = sine + h2 + h3;

        // Rising noise
        let noise = rng.next_f32() * 2.0 - 1.0;
        let noise_cutoff = 500.0 + t * 3000.0;
        noise_filter.set_params(noise_cutoff, 1.0);
        let filtered_noise = noise_filter.process(noise);

        // Rising filter on tone
        let tone_cutoff = 200.0 + t * 2000.0;
        tone_filter.set_params(tone_cutoff, 2.0);
        let filtered_tone = tone_filter.process(tone);

        let mix = filtered_tone * 0.6 + filtered_noise * 0.4;

        // Build envelope
        let env = (t / 2.0).powf(1.5);

        let saturated = soft_saturate(mix * env * 2.0) * 0.6;

        let sample = saturated * 30000.0;
        output.push(sample.clamp(-32767.0, 32767.0) as i16);
    }

    output
}
