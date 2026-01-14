//! Roland TB-303 acid bass synthesizer

use super::super::common::{exp_attack, exp_decay, sawtooth_blep, SAMPLE_RATE};
use super::filters::{soft_saturate, StateVariableFilter};

// ============================================================================
// TB-303 Acid Bass - THE STAR (High Quality)
// ============================================================================

pub fn generate_bass_303() -> Vec<i16> {
    let duration = 0.8; // Longer for natural release
    let samples = (SAMPLE_RATE * duration) as usize;
    let mut output = Vec::with_capacity(samples);

    // Base frequency (this will be modulated by note pitch in the tracker)
    let base_freq = 82.41; // E2 note
    let phase_inc = base_freq / SAMPLE_RATE;
    let mut phase = 0.0f32;
    let mut filter = StateVariableFilter::new();

    for i in 0..samples {
        let t = i as f32 / SAMPLE_RATE;

        // === ANTI-ALIASED SAWTOOTH WAVE (PolyBLEP) ===
        let saw = sawtooth_blep(phase, phase_inc);
        phase += phase_inc;
        if phase >= 1.0 {
            phase -= 1.0;
        }

        // === FILTER ENVELOPE (exponential for natural sound) ===
        // Creates the "squelch" when accent is triggered
        let filter_env = if t < 0.005 {
            exp_attack(t / 0.005, 12.0) * 3.0 // Very fast exponential attack
        } else {
            let decay_t = (t - 0.005) / 0.15;
            3.0 * exp_decay(decay_t, 6.0).max(0.5) // Exponential decay to sustain
        };

        // === RESONANT BANDPASS FILTER ===
        // Cutoff frequency modulated by envelope
        // Base cutoff ~200Hz, envelope can push it to ~2kHz
        let cutoff_hz = 200.0 + 1800.0 * filter_env;
        let cutoff_norm = (cutoff_hz / SAMPLE_RATE).min(0.45);

        // High resonance creates the "squelch"
        let resonance = 0.85; // Very high for acid character

        // Process through state variable filter (we want bandpass output)
        let (_low, band, _high) = filter.process(saw, cutoff_norm, resonance);

        // === AMPLITUDE ENVELOPE (exponential) ===
        let amp_env = if t < 0.003 {
            exp_attack(t / 0.003, 10.0) // Smooth attack
        } else if t < 0.7 {
            1.0 // Sustain
        } else {
            exp_decay((t - 0.7) / 0.1, 8.0) // Smooth exponential release
        };

        // === OUTPUT ===
        // Bandpass output is the signature 303 sound
        let filtered = band * amp_env;

        // Gentle soft saturation for analog warmth
        let saturated = soft_saturate(filtered * 1.5) * 0.7;
        let sample = saturated * 31000.0;

        output.push(sample.clamp(-32767.0, 32767.0) as i16);
    }

    output
}

// ============================================================================
// TB-303 Squelch Variant - Maximum resonance for climax (High Quality)
// ============================================================================

pub fn generate_bass_303_squelch() -> Vec<i16> {
    let duration = 0.8; // Longer for natural release
    let samples = (SAMPLE_RATE * duration) as usize;
    let mut output = Vec::with_capacity(samples);

    let base_freq = 82.41; // E2
    let phase_inc = base_freq / SAMPLE_RATE;
    let mut phase = 0.0f32;
    let mut filter = StateVariableFilter::new();

    for i in 0..samples {
        let t = i as f32 / SAMPLE_RATE;

        // === ANTI-ALIASED SAWTOOTH WAVE (PolyBLEP) ===
        let saw = sawtooth_blep(phase, phase_inc);
        phase += phase_inc;
        if phase >= 1.0 {
            phase -= 1.0;
        }

        // === MORE AGGRESSIVE filter envelope for maximum squelch ===
        let filter_env = if t < 0.005 {
            exp_attack(t / 0.005, 15.0) * 4.0 // Very aggressive attack
        } else {
            let decay_t = (t - 0.005) / 0.12; // Faster decay
            4.0 * exp_decay(decay_t, 8.0).max(0.3) // Lower sustain
        };

        // === WIDER filter sweep ===
        let cutoff_hz = 200.0 + 2800.0 * filter_env; // Up to 3kHz vs 2kHz
        let cutoff_norm = (cutoff_hz / SAMPLE_RATE).min(0.45);

        // === HIGHER resonance for maximum squelch ===
        let resonance = 0.92; // vs 0.85 in normal 303

        let (_low, band, _high) = filter.process(saw, cutoff_norm, resonance);

        // === AMPLITUDE ENVELOPE (exponential) ===
        let amp_env = if t < 0.003 {
            exp_attack(t / 0.003, 10.0)
        } else if t < 0.7 {
            1.0
        } else {
            exp_decay((t - 0.7) / 0.1, 8.0)
        };

        let filtered = band * amp_env;

        // More saturation for aggressive character
        let saturated = soft_saturate(filtered * 2.0) * 0.65;
        let sample = saturated * 31000.0;

        output.push(sample.clamp(-32767.0, 32767.0) as i16);
    }

    output
}
