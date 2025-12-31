//! Synthwave instrument synthesis
//!
//! Instruments for "Nether Drive" - Synthwave at 105 BPM in A minor

use std::f32::consts::PI;
use super::common::{SimpleRng, SAMPLE_RATE};

/// Synthwave kick: Enhanced 808-style with rich harmonics and warmth
pub fn generate_kick_synth() -> Vec<i16> {
    let duration = 0.4; // 400ms for that 80s thump
    let samples = (SAMPLE_RATE * duration) as usize;

    let mut output = Vec::with_capacity(samples);
    let mut phase = 0.0f32;

    // 2-pole filter for warm character
    let mut lp1 = 0.0f32;
    let mut lp2 = 0.0f32;

    for i in 0..samples {
        let t = i as f32 / SAMPLE_RATE;

        // Long, smooth decay for 808 character
        let decay = if t < 0.08 {
            (-t * 6.5).exp()
        } else {
            0.52 * (-((t - 0.08) * 7.5)).exp()
        };

        // Gentle pitch sweep for warm thump
        let freq = if t < 0.04 {
            170.0 * (-t * 22.0).exp() + 65.0
        } else {
            65.0 * (-((t - 0.04) * 12.0)).exp() + 45.0
        };

        // Main sine oscillator
        phase += 2.0 * PI * freq / SAMPLE_RATE;
        let sine = phase.sin();

        // Add subtle 2nd harmonic for depth
        let harmonic = (phase * 2.0).sin() * 0.12 * (-t * 15.0).exp();

        // Sub harmonic (808 characteristic)
        let sub = (phase * 0.5).sin() * 0.18 * (-t * 10.0).exp();

        // Combine oscillators
        let raw = sine + harmonic + sub;

        // 2-pole low-pass for vintage warmth
        let cutoff = 0.22;
        lp1 += cutoff * (raw - lp1);
        lp2 += cutoff * (lp1 - lp2);

        // Soft saturation for analog character
        let saturated = (lp2 * 1.15).tanh();

        let sample = saturated * decay * 31000.0;
        output.push(sample.clamp(-32767.0, 32767.0) as i16);
    }

    output
}

/// Synthwave snare: Enhanced gated reverb with rich body
pub fn generate_snare_synth() -> Vec<i16> {
    let duration = 0.18; // 180ms - gated feel
    let samples = (SAMPLE_RATE * duration) as usize;

    let mut output = Vec::with_capacity(samples);
    let mut rng = SimpleRng::new(88888);

    // Filter states
    let mut lp1 = 0.0f32;
    let mut lp2 = 0.0f32;
    let mut hp_prev_in = 0.0f32;
    let mut hp_prev_out = 0.0f32;

    for i in 0..samples {
        let t = i as f32 / SAMPLE_RATE;

        // Classic gated reverb envelope
        let envelope = if t < 0.015 {
            (t / 0.015).powf(0.8) // Smooth attack
        } else if t < 0.11 {
            1.0 - (t - 0.015) * 0.25 // Sustain plateau
        } else {
            // Abrupt gate close (iconic 80s sound)
            0.75 * (1.0 - ((t - 0.11) / 0.07).powf(1.5)).max(0.0)
        };

        // Noise burst
        let white_noise = rng.next_f32() * 2.0 - 1.0;

        // High-pass for clarity
        let hp_alpha = 0.72;
        let hp_out = hp_alpha * (hp_prev_out + white_noise - hp_prev_in);
        hp_prev_in = white_noise;
        hp_prev_out = hp_out;

        // Multiple body modes
        let body1 = (2.0 * PI * 195.0 * t).sin() * 0.48;
        let body2 = (2.0 * PI * 260.0 * t).sin() * 0.30;
        let body_env = (-t * 18.0).exp();
        let body = (body1 + body2) * body_env;

        // Mix components
        let raw = hp_out * 0.50 + body;

        // 2-pole low-pass for smoothness
        let cutoff = 0.38;
        lp1 += cutoff * (raw - lp1);
        lp2 += cutoff * (lp1 - lp2);

        // Soft saturation
        let saturated = (lp2 * 1.18).tanh();

        let sample = saturated * envelope * 29000.0;
        output.push(sample.clamp(-32767.0, 32767.0) as i16);
    }

    output
}

/// Synthwave hi-hat: Enhanced with metallic resonances
pub fn generate_hihat_synth() -> Vec<i16> {
    let duration = 0.1; // 100ms
    let samples = (SAMPLE_RATE * duration) as usize;

    let mut output = Vec::with_capacity(samples);
    let mut rng = SimpleRng::new(66666);

    // Filter states
    let mut hp_prev_in = 0.0f32;
    let mut hp_prev_out = 0.0f32;
    let mut lp1 = 0.0f32;
    let mut lp2 = 0.0f32;

    for i in 0..samples {
        let t = i as f32 / SAMPLE_RATE;

        // Smooth envelope
        let decay = if t < 0.008 {
            t / 0.008 // Short attack
        } else {
            (-((t - 0.008) * 32.0)).exp()
        };

        // White noise
        let noise = rng.next_f32() * 2.0 - 1.0;

        // High-pass for brightness
        let hp_alpha = 0.92;
        let hp_out = hp_alpha * (hp_prev_out + noise - hp_prev_in);
        hp_prev_in = noise;
        hp_prev_out = hp_out;

        // Add metallic character
        let metal1 = (2.0 * PI * 7500.0 * t).sin() * 0.09 * (-t * 38.0).exp();
        let metal2 = (2.0 * PI * 9800.0 * t).sin() * 0.06 * (-t * 42.0).exp();

        // Mix
        let raw = hp_out * 0.88 + metal1 + metal2;

        // 2-pole low-pass for smoothness
        let cutoff = 0.48;
        lp1 += cutoff * (raw - lp1);
        lp2 += cutoff * (lp1 - lp2);

        let sample = lp2 * decay * 23000.0;
        output.push(sample.clamp(-32767.0, 32767.0) as i16);
    }

    output
}

/// Synthwave bass: Enhanced pulsing bass with rich sub harmonics
pub fn generate_bass_synth() -> Vec<i16> {
    let duration = 0.35; // 350ms
    let freq = 55.0; // A1 as base
    let samples = (SAMPLE_RATE * duration) as usize;

    let mut output = Vec::with_capacity(samples);

    // 3-pole filter for warm character
    let mut lp1 = 0.0f32;
    let mut lp2 = 0.0f32;
    let mut lp3 = 0.0f32;
    let mut phase = 0.0f32;
    let mut sub_phase = 0.0f32;

    for i in 0..samples {
        let t = i as f32 / SAMPLE_RATE;

        // Smooth envelope
        let envelope = if t < 0.010 {
            (t / 0.010).powf(0.85) // Slightly curved attack
        } else if t < 0.24 {
            1.0 - (t - 0.010) * 0.048
        } else {
            0.96 * (-(t - 0.24) * 6.5).exp()
        };

        // Filter sweep with resonance
        let filter_cutoff = if t < 0.19 {
            0.45 - (t / 0.19) * 0.35 // 800Hz → 200Hz sweep
        } else {
            0.10 // Low sustain
        };
        let resonance = 0.08 * (-t * 12.0).exp(); // Resonance peak at attack

        // Main sawtooth with phase accumulation
        phase += freq / SAMPLE_RATE;
        phase = phase % 1.0;
        let saw = 2.0 * phase - 1.0;

        // Add subtle pulse for warmth (15% mix)
        let pulse = if phase < 0.5 { 1.0 } else { -1.0 };
        let osc_mix = saw * 0.85 + pulse * 0.15;

        // Sub oscillator with proper phase tracking
        sub_phase += (freq * 0.5) / SAMPLE_RATE;
        sub_phase = sub_phase % 1.0;
        let sub = (sub_phase * 2.0 * PI).sin() * 0.32;

        // 3-pole resonant filter
        lp1 += filter_cutoff * (osc_mix - lp1) + resonance * (osc_mix - lp1);
        lp2 += filter_cutoff * (lp1 - lp2);
        lp3 += filter_cutoff * (lp2 - lp3);

        // Mix filtered oscillator with sub
        let mixed = lp3 * 0.70 + sub;

        // Warm saturation
        let saturated = (mixed * 1.15).tanh();

        let sample = saturated * envelope * 29000.0;
        output.push(sample.clamp(-32767.0, 32767.0) as i16);
    }

    output
}

/// Synthwave lead: Enhanced soaring lead with 3 oscillators and warmth
pub fn generate_lead_synth() -> Vec<i16> {
    let duration = 0.9; // 900ms
    let freq = 220.0; // A3 as base
    let samples = (SAMPLE_RATE * duration) as usize;

    let mut output = Vec::with_capacity(samples);

    // Multi-pole filter
    let mut lp1 = 0.0f32;
    let mut lp2 = 0.0f32;
    let mut lp3 = 0.0f32;

    let mut phases = [0.0f32; 3];
    let mut vibrato_phase = 0.0f32;

    // 3 oscillators for width
    let detune_ratios = [0.9965, 1.0, 1.0072]; // ~12 cents spread

    for i in 0..samples {
        let t = i as f32 / SAMPLE_RATE;

        // Smooth ADSR envelope
        let envelope = if t < 0.018 {
            (t / 0.018).powf(0.65) // Smooth curved attack
        } else if t < 0.58 {
            1.0 - (t - 0.018) * 0.092
        } else {
            0.91 * (-(t - 0.58) * 3.2).exp()
        };

        // Delayed vibrato
        let vibrato_amount = if t < 0.08 {
            0.0
        } else {
            0.0055 * ((t - 0.08) * 2.2).min(1.0)
        };
        vibrato_phase += 5.2 / SAMPLE_RATE;
        let vibrato = 1.0 + vibrato_amount * (vibrato_phase * 2.0 * PI).sin();

        // 3 detuned saw oscillators
        let mut saw_sum = 0.0f32;
        for (idx, ratio) in detune_ratios.iter().enumerate() {
            phases[idx] += freq * ratio * vibrato / SAMPLE_RATE;
            phases[idx] = phases[idx] % 1.0;

            // Mix saw and triangle for warmth
            let saw = 2.0 * phases[idx] - 1.0;
            let tri = 4.0 * (phases[idx] - 0.5).abs() - 1.0;
            saw_sum += saw * 0.82 + tri * 0.18;
        }
        saw_sum /= 3.0;

        // 3-pole low-pass with envelope control
        let filter_cutoff = 0.12 + 0.06 * envelope;
        lp1 += filter_cutoff * (saw_sum - lp1);
        lp2 += filter_cutoff * (lp1 - lp2);
        lp3 += filter_cutoff * (lp2 - lp3);

        // Gentle saturation
        let saturated = (lp3 * 1.08).tanh();

        let sample = saturated * envelope * 25000.0;
        output.push(sample.clamp(-32767.0, 32767.0) as i16);
    }

    output
}

/// Synthwave arpeggiator: Enhanced plucky sound with sparkle
pub fn generate_arp_synth() -> Vec<i16> {
    let duration = 0.3; // 300ms
    let freq = 440.0; // A4 as base
    let samples = (SAMPLE_RATE * duration) as usize;

    let mut output = Vec::with_capacity(samples);

    // 2-pole filter
    let mut lp1 = 0.0f32;
    let mut lp2 = 0.0f32;
    let mut phase = 0.0f32;

    for i in 0..samples {
        let t = i as f32 / SAMPLE_RATE;

        // Plucky envelope - fast attack, smooth decay
        let envelope = if t < 0.002 {
            (t / 0.002).powf(0.7)
        } else {
            (-(t - 0.002) * 8.5).exp()
        };

        // Phase accumulation
        phase += freq / SAMPLE_RATE;
        phase = phase % 1.0;

        // Square wave with variable pulse width for character
        let pw = 0.48 + 0.04 * (t * 8.0).sin();
        let square = if phase < pw { 1.0 } else { -1.0 };

        // Add subtle saw for brightness (10% mix)
        let saw = 2.0 * phase - 1.0;
        let osc_mix = square * 0.90 + saw * 0.10;

        // Filter envelope creates pluck character
        let filter_cutoff = 0.12 + 0.42 * (-t * 16.0).exp();
        lp1 += filter_cutoff * (osc_mix - lp1);
        lp2 += filter_cutoff * (lp1 - lp2);

        // Gentle saturation for digital sparkle
        let saturated = (lp2 * 1.05).tanh();

        let sample = saturated * envelope * 21000.0;
        output.push(sample.clamp(-32767.0, 32767.0) as i16);
    }

    output
}

/// Synthwave pad: Ultra-lush 5-oscillator pad with movement
pub fn generate_pad_synth() -> Vec<i16> {
    let duration = 2.0; // 2 seconds for long sustain
    let freq = 220.0; // A3 as base
    let samples = (SAMPLE_RATE * duration) as usize;

    let mut output = Vec::with_capacity(samples);

    // 4-pole filter for ultra-smooth character
    let mut lp1 = 0.0f32;
    let mut lp2 = 0.0f32;
    let mut lp3 = 0.0f32;
    let mut lp4 = 0.0f32;

    let mut phases = [0.0f32; 5];
    let mut lfo_phase = 0.0f32;

    // 5 oscillators for maximum width
    let detune_amounts = [0.990, 0.996, 1.0, 1.004, 1.010];

    for i in 0..samples {
        let t = i as f32 / SAMPLE_RATE;

        // Very slow, smooth attack curve
        let envelope = if t < 0.35 {
            (t / 0.35).powf(1.8) // Ultra-smooth attack
        } else if t < 1.45 {
            1.0
        } else {
            (-(t - 1.45) * 2.3).exp()
        };

        // Subtle LFO for movement (very slow)
        lfo_phase += 0.25 / SAMPLE_RATE;
        let lfo = (lfo_phase * 2.0 * PI).sin() * 0.0018;

        // 5 detuned oscillators
        let mut osc_sum = 0.0f32;
        for (idx, detune) in detune_amounts.iter().enumerate() {
            phases[idx] += freq * detune * (1.0 + lfo) / SAMPLE_RATE;
            phases[idx] = phases[idx] % 1.0;

            // Mix saw, triangle, and sine for ultra-smooth character
            let saw = 2.0 * phases[idx] - 1.0;
            let tri = 4.0 * (phases[idx] - 0.5).abs() - 1.0;
            let sine = (phases[idx] * 2.0 * PI).sin();
            osc_sum += saw * 0.50 + tri * 0.30 + sine * 0.20;
        }
        osc_sum /= 5.0;

        // 4-pole low-pass for vintage warmth
        let cutoff = 0.16;
        lp1 += cutoff * (osc_sum - lp1);
        lp2 += cutoff * (lp1 - lp2);
        lp3 += cutoff * (lp2 - lp3);
        lp4 += cutoff * (lp3 - lp4);

        let sample = lp4 * envelope * 30000.0;
        output.push(sample.clamp(-32767.0, 32767.0) as i16);
    }

    output
}
