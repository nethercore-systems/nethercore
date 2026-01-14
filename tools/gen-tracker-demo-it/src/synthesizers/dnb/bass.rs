//! Drum and Bass bass instruments

use super::super::common::SAMPLE_RATE;
use super::dsp::{blep_saw, blep_square, soft_saturate, StateVariableFilter};
use std::f32::consts::PI;

const TWO_PI: f32 = 2.0 * PI;

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
        if phase >= 1.0 {
            phase -= 1.0;
        }

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
            if phases[j] >= 1.0 {
                phases[j] -= 1.0;
            }
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
            if phases[j] >= 1.0 {
                phases[j] -= 1.0;
            }
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
