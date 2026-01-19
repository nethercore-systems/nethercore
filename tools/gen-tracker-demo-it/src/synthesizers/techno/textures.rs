//! Acid techno texture instruments: pads, stabs, risers, atmosphere, and crash

use super::super::common::{exp_attack, exp_decay, sawtooth_blep, SimpleRng, SAMPLE_RATE};
use super::filters::{soft_saturate, BiquadHP, StateVariableFilter};
use std::f32::consts::PI;

const TWO_PI: f32 = 2.0 * PI;

// ============================================================================
// Acid Pad - Rich background texture (High Quality)
// ============================================================================

pub fn generate_pad_acid() -> Vec<i16> {
    let duration = 2.5; // Longer for fuller sustain
    let samples = (SAMPLE_RATE * duration) as usize;
    let mut output = Vec::with_capacity(samples);

    // Base frequency (E3)
    let base_freq = 164.81;

    // Local filter state
    let mut lp_state = 0.0f32;

    for i in 0..samples {
        let t = i as f32 / SAMPLE_RATE;

        // Five detuned oscillators for richer chorus effect
        let osc1 = (t * base_freq * TWO_PI).sin();
        let osc2 = (t * base_freq * 1.005 * TWO_PI).sin(); // +5 cents
        let osc3 = (t * base_freq * 0.995 * TWO_PI).sin(); // -5 cents
        let osc4 = (t * base_freq * 1.010 * TWO_PI).sin(); // +10 cents
        let osc5 = (t * base_freq * 0.990 * TWO_PI).sin(); // -10 cents

        let mut oscillator = (osc1 + osc2 + osc3 + osc4 + osc5) / 5.0;

        // Low-pass filter for warmth (1-pole)
        lp_state = lp_state * 0.93 + oscillator * 0.07;
        oscillator = lp_state;

        // Exponential ADSR envelope
        let amp_env = if t < 0.3 {
            exp_attack(t / 0.3, 5.0) // Slow exponential attack
        } else if t < 2.0 {
            1.0 // Sustain
        } else {
            exp_decay((t - 2.0) / 0.5, 6.0) // Smooth exponential release
        };

        let saturated = soft_saturate(oscillator * amp_env * 1.2) * 0.7;
        let sample = saturated * 29000.0;

        output.push(sample.clamp(-32767.0, 32767.0) as i16);
    }

    output
}

// ============================================================================
// Acid Stab - Chord hits (High Quality)
// ============================================================================

pub fn generate_stab_acid() -> Vec<i16> {
    let duration = 0.4; // Longer tail for natural release
    let samples = (SAMPLE_RATE * duration) as usize;
    let mut output = Vec::with_capacity(samples);

    // Base frequency (E4)
    let base_freq = 329.63;

    // Phase accumulators for each oscillator
    let mut phase1 = 0.0f32;
    let mut phase2 = 0.0f32;
    let mut phase3 = 0.0f32;
    let mut phase4 = 0.0f32;
    let mut phase5 = 0.0f32;

    let freq1 = base_freq;
    let freq2 = base_freq * 1.005; // +5 cents
    let freq3 = base_freq * 0.995; // -5 cents
    let freq4 = base_freq * 1.010; // +10 cents
    let freq5 = base_freq * 0.990; // -10 cents

    let phase_inc1 = freq1 / SAMPLE_RATE;
    let phase_inc2 = freq2 / SAMPLE_RATE;
    let phase_inc3 = freq3 / SAMPLE_RATE;
    let phase_inc4 = freq4 / SAMPLE_RATE;
    let phase_inc5 = freq5 / SAMPLE_RATE;

    for i in 0..samples {
        let t = i as f32 / SAMPLE_RATE;

        // Five detuned anti-aliased sawtooth oscillators (supersaw)
        let saw1 = sawtooth_blep(phase1, phase_inc1);
        let saw2 = sawtooth_blep(phase2, phase_inc2);
        let saw3 = sawtooth_blep(phase3, phase_inc3);
        let saw4 = sawtooth_blep(phase4, phase_inc4);
        let saw5 = sawtooth_blep(phase5, phase_inc5);

        phase1 += phase_inc1;
        if phase1 >= 1.0 {
            phase1 -= 1.0;
        }
        phase2 += phase_inc2;
        if phase2 >= 1.0 {
            phase2 -= 1.0;
        }
        phase3 += phase_inc3;
        if phase3 >= 1.0 {
            phase3 -= 1.0;
        }
        phase4 += phase_inc4;
        if phase4 >= 1.0 {
            phase4 -= 1.0;
        }
        phase5 += phase_inc5;
        if phase5 >= 1.0 {
            phase5 -= 1.0;
        }

        let supersaw = (saw1 + saw2 + saw3 + saw4 + saw5) / 5.0;

        // Exponential amplitude envelope - punchy but smooth
        let amp_env = if t < 0.005 {
            exp_attack(t / 0.005, 12.0) // Very fast attack
        } else {
            exp_decay((t - 0.005) / 0.35, 10.0) // Fast exponential decay
        };

        let saturated = soft_saturate(supersaw * amp_env * 1.5) * 0.7;
        let sample = saturated * 30000.0;

        output.push(sample.clamp(-32767.0, 32767.0) as i16);
    }

    output
}

// ============================================================================
// Acid Riser - Smooth sweep for builds (High Quality)
// ============================================================================

pub fn generate_riser_acid() -> Vec<i16> {
    let duration = 2.5;
    let samples = (SAMPLE_RATE * duration) as usize;
    let mut output = Vec::with_capacity(samples);
    let mut rng = SimpleRng::new(99999);
    let mut phase = 0.0f32;
    let mut svf = StateVariableFilter::new();

    for i in 0..samples {
        let t = i as f32 / SAMPLE_RATE;
        let progress = t / duration;

        // Rising sine
        let freq = 120.0 + 1400.0 * progress.powf(2.0);
        phase += freq / SAMPLE_RATE;
        if phase >= 1.0 {
            phase -= 1.0;
        }
        let sine = (phase * TWO_PI).sin();

        // Noise with rising filter
        let noise = rng.next_f32() * 2.0 - 1.0;
        let cutoff = 0.04 + 0.38 * progress;
        let (_, _, high) = svf.process(noise, cutoff, 0.3);

        let env = progress.powf(1.5);
        let mixed = (sine * 0.5 + high * 0.35) * env;
        let saturated = soft_saturate(mixed * 1.4) * 0.7;
        let sample = saturated * 29000.0;
        output.push(sample.clamp(-32767.0, 32767.0) as i16);
    }

    output
}

// ============================================================================
// Acid Atmosphere - Subtle texture layer (High Quality)
// ============================================================================

pub fn generate_atmosphere_acid() -> Vec<i16> {
    let duration = 4.0; // Very long sustain for subtle texture
    let samples = (SAMPLE_RATE * duration) as usize;
    let mut output = Vec::with_capacity(samples);

    // Very low frequency for atmosphere
    let base_freq = 55.0; // A1

    for i in 0..samples {
        let t = i as f32 / SAMPLE_RATE;

        // Multiple slow LFOs for movement
        let lfo1 = (t * 0.4 * TWO_PI).sin();
        let lfo2 = (t * 0.6 * TWO_PI).sin();
        let lfo3 = (t * 0.9 * TWO_PI).sin();

        // Five detuned sine oscillators (very subtle chorus)
        let osc1 = (t * base_freq * TWO_PI).sin();
        let osc2 = (t * base_freq * 1.002 * TWO_PI).sin();
        let osc3 = (t * base_freq * 0.998 * TWO_PI).sin();
        let osc4 = (t * base_freq * 1.004 * TWO_PI).sin();
        let osc5 = (t * base_freq * 0.996 * TWO_PI).sin();

        let mut oscillator = (osc1 + osc2 + osc3 + osc4 + osc5) / 5.0;

        // Modulate amplitude with LFOs for subtle movement
        oscillator *= 0.4 + 0.2 * lfo1 + 0.15 * lfo2 + 0.1 * lfo3;

        // Exponential ADSR envelope - very slow
        let amp_env = if t < 0.8 {
            exp_attack(t / 0.8, 3.0) // Very slow attack
        } else if t < 3.0 {
            1.0 // Long sustain
        } else {
            exp_decay((t - 3.0) / 1.0, 4.0) // Slow release
        };

        let saturated = soft_saturate(oscillator * amp_env * 1.1) * 0.7;

        // VERY quiet - just subtle texture
        let sample = saturated * 26000.0;
        output.push(sample.clamp(-32767.0, 32767.0) as i16);
    }

    output
}

// ============================================================================
// 909 Crash Cymbal - For transitions (High Quality)
// ============================================================================

pub fn generate_crash_909() -> Vec<i16> {
    let duration = 1.5;
    let samples = (SAMPLE_RATE * duration) as usize;
    let mut output = Vec::with_capacity(samples);
    let mut rng = SimpleRng::new(77777);

    // Multiple bandpass filters for metallic complexity
    let mut bp1 = BiquadHP::new();
    bp1.set_params(3800.0, 1.0);
    let mut bp2 = BiquadHP::new();
    bp2.set_params(5800.0, 1.2);
    let mut bp3 = BiquadHP::new();
    bp3.set_params(7800.0, 0.8);

    for i in 0..samples {
        let t = i as f32 / SAMPLE_RATE;
        let noise = rng.next_f32() * 2.0 - 1.0;

        // Multiple bands for shimmer
        let band1 = bp1.process(noise);
        let band2 = bp2.process(noise);
        let band3 = bp3.process(noise);

        let mix = band1 * 0.5 + band2 * 0.3 + band3 * 0.2;

        let env = if t < 0.008 {
            t / 0.008
        } else {
            (-t * 2.2).exp()
        };

        let saturated = soft_saturate(mix * env * 1.4) * 0.75;
        let sample = saturated * 27000.0;
        output.push(sample.clamp(-32767.0, 32767.0) as i16);
    }

    output
}
