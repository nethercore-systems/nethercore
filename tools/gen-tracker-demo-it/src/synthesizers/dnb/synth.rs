//! Drum and Bass synth instruments (pads and leads)

use super::super::common::SAMPLE_RATE;
use super::dsp::{blep_saw, blep_square, soft_saturate, BiquadLP, StateVariableFilter};
use std::f32::consts::PI;

const TWO_PI: f32 = 2.0 * PI;

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
            if phases[j] >= 1.0 {
                phases[j] -= 1.0;
            }
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
            if phases[j] >= 1.0 {
                phases[j] -= 1.0;
            }
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
        let vib_depth = if t > 0.15 {
            ((t - 0.15) / 0.2).min(1.0)
        } else {
            0.0
        };
        vibrato_phase += 5.5 / SAMPLE_RATE;
        let vibrato = 1.0 + 0.008 * vib_depth * (vibrato_phase * TWO_PI).sin();

        let actual_freq = freq * vibrato;
        let dt = actual_freq / SAMPLE_RATE;

        // 3 detuned saws + square
        let mut sum = 0.0f32;
        for (j, d) in detune.iter().enumerate() {
            phases[j] += actual_freq * d / SAMPLE_RATE;
            if phases[j] >= 1.0 {
                phases[j] -= 1.0;
            }
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
