//! Eurobeat instrument synthesis
//!
//! Instruments for "Nether Fire" - Eurobeat at 155 BPM in D minor

use super::common::{SimpleRng, SAMPLE_RATE};
use std::f32::consts::PI;

/// Eurobeat kick: Improved 909-style with aggressive punch and harmonics
pub fn generate_kick_euro() -> Vec<i16> {
    let duration = 0.3; // 300ms
    let samples = (SAMPLE_RATE * duration) as usize;

    let mut output = Vec::with_capacity(samples);
    let mut phase = 0.0f32;
    let mut click_phase = 0.0f32;

    for i in 0..samples {
        let t = i as f32 / SAMPLE_RATE;

        // Two-stage decay for maximum punch
        let decay = if t < 0.04 {
            (-t * 16.0).exp()
        } else {
            0.52 * (-((t - 0.04) * 20.0)).exp()
        };

        // Aggressive three-stage pitch sweep for 909 character
        let freq = if t < 0.015 {
            // Initial click/snap: 300Hz → 200Hz
            300.0 * (-t * 50.0).exp() + 200.0
        } else if t < 0.06 {
            // Main body: 200Hz → 60Hz
            200.0 * (-((t - 0.015) * 22.0)).exp() + 60.0
        } else {
            // Tail: 60Hz → 40Hz
            60.0 * (-((t - 0.06) * 10.0)).exp() + 40.0
        };

        // Main body oscillator
        phase += 2.0 * PI * freq / SAMPLE_RATE;
        let body = phase.sin();

        // High-frequency click transient (beater impact)
        click_phase += 2.0 * PI * 1200.0 / SAMPLE_RATE;
        let click_env = (-t * 180.0).exp();
        let click = click_phase.sin() * click_env * 0.35;

        // Add 2nd harmonic for punch
        let harmonic = (phase * 2.0).sin() * 0.18 * (-t * 22.0).exp();

        // Combine layers
        let raw = body + click + harmonic;

        // Hard saturation for 909-style punch
        let saturated = (raw * 1.4).clamp(-1.0, 1.0);

        // Add subtle bit-crush effect for digital punch
        let crushed = (saturated * 28.0).round() / 28.0;

        let sample = crushed * decay * 32000.0;
        output.push(sample.clamp(-32767.0, 32767.0) as i16);
    }

    output
}

/// Eurobeat snare: Improved crisp attack with gated reverb tail
pub fn generate_snare_euro() -> Vec<i16> {
    let duration = 0.22; // 220ms
    let samples = (SAMPLE_RATE * duration) as usize;

    let mut output = Vec::with_capacity(samples);
    let mut rng = SimpleRng::new(99999);

    // Band-pass filter states
    let mut bp1 = 0.0f32;
    let mut bp2 = 0.0f32;
    let mut hp_prev_in = 0.0f32;
    let mut hp_prev_out = 0.0f32;

    for i in 0..samples {
        let t = i as f32 / SAMPLE_RATE;

        // Three-stage envelope for gated reverb character
        let amp_env = if t < 0.003 {
            t / 0.003 // Instant attack
        } else if t < 0.045 {
            (-((t - 0.003) * 32.0)).exp() // Fast initial decay
        } else if t < 0.12 {
            // Gated sustain plateau
            0.20 * (1.0 - (t - 0.045) * 0.8)
        } else {
            // Gate close
            0.14 * (-(t - 0.12) * 15.0).exp()
        };

        // White noise
        let white_noise = rng.next_f32() * 2.0 - 1.0;

        // High-pass for brightness
        let hp_alpha = 0.75;
        let hp_out = hp_alpha * (hp_prev_out + white_noise - hp_prev_in);
        hp_prev_in = white_noise;
        hp_prev_out = hp_out;

        // Multiple body resonances for realistic shell
        let body1 = (2.0 * PI * 180.0 * t).sin() * 0.42;
        let body2 = (2.0 * PI * 240.0 * t).sin() * 0.28;
        let body3 = (2.0 * PI * 315.0 * t).sin() * 0.18;
        let body = (body1 + body2 + body3) * (-t * 38.0).exp();

        // High crack transient
        let crack1 = (2.0 * PI * 420.0 * t).sin() * 0.25 * (-t * 55.0).exp();
        let crack2 = (2.0 * PI * 680.0 * t).sin() * 0.15 * (-t * 70.0).exp();

        // Mix components
        let raw = hp_out * 0.50 + body + crack1 + crack2;

        // Band-pass for tightness (remove extreme lows/highs)
        let cutoff = 0.40;
        bp1 += cutoff * (raw - bp1);
        bp2 += cutoff * (bp1 - bp2);

        // Hard saturation for snap
        let saturated = (bp2 * 1.25).tanh();

        let sample = saturated * amp_env * 31000.0;
        output.push(sample.clamp(-32767.0, 32767.0) as i16);
    }

    output
}

/// Eurobeat hi-hat: Improved bright, cutting sound with metallic character
pub fn generate_hihat_euro() -> Vec<i16> {
    let duration = 0.08; // 80ms - very short for rapid patterns
    let samples = (SAMPLE_RATE * duration) as usize;

    let mut output = Vec::with_capacity(samples);
    let mut rng = SimpleRng::new(77777);

    // Multi-stage high-pass filtering
    let mut hp1_prev_in = 0.0f32;
    let mut hp1_prev_out = 0.0f32;
    let mut hp2_prev_in = 0.0f32;
    let mut hp2_prev_out = 0.0f32;

    for i in 0..samples {
        let t = i as f32 / SAMPLE_RATE;

        // Very fast exponential decay
        let decay = (-t * 55.0).exp();

        // Generate noise
        let noise = rng.next_f32() * 2.0 - 1.0;

        // 2-stage high-pass for extreme brightness
        let hp_alpha = 0.96;
        let hp1_out = hp_alpha * (hp1_prev_out + noise - hp1_prev_in);
        hp1_prev_in = noise;
        hp1_prev_out = hp1_out;

        let hp2_out = hp_alpha * (hp2_prev_out + hp1_out - hp2_prev_in);
        hp2_prev_in = hp1_out;
        hp2_prev_out = hp2_out;

        // Add metallic resonances for character
        let metal1 = (2.0 * PI * 8200.0 * t).sin() * 0.10 * (-t * 45.0).exp();
        let metal2 = (2.0 * PI * 10500.0 * t).sin() * 0.08 * (-t * 50.0).exp();
        let metal3 = (2.0 * PI * 12800.0 * t).sin() * 0.05 * (-t * 60.0).exp();

        // Mix noise and metallics
        let raw = hp2_out * 0.88 + metal1 + metal2 + metal3;

        // Subtle saturation for digital edge
        let saturated = (raw * 1.15).tanh();

        let sample = saturated * decay * 25000.0;
        output.push(sample.clamp(-32767.0, 32767.0) as i16);
    }

    output
}

/// Eurobeat bass: Enhanced bouncy bass with richer harmonics
pub fn generate_bass_euro() -> Vec<i16> {
    let duration = 0.25; // 250ms - short for bounce
    let freq = 73.42; // D2 as base
    let samples = (SAMPLE_RATE * duration) as usize;

    let mut output = Vec::with_capacity(samples);

    // Filter state for punch
    let mut lp1 = 0.0f32;
    let mut lp2 = 0.0f32;
    let mut phase = 0.0f32;
    let mut sub_phase = 0.0f32;

    for i in 0..samples {
        let t = i as f32 / SAMPLE_RATE;

        // Snappy envelope with slight curve for smoothness
        let envelope = if t < 0.002 {
            (t / 0.002).powf(0.9) // Nearly instant attack
        } else if t < 0.048 {
            1.0 // Short sustain
        } else {
            (-(t - 0.048) * 13.5).exp() // Fast decay
        };

        // Subtle pitch envelope for punch (very short)
        let pitch_env = 1.0 + 0.012 * (-t * 80.0).exp();

        // Proper phase accumulation
        phase += freq * pitch_env / SAMPLE_RATE;
        phase %= 1.0;

        // Pulse wave (45% duty) with slight detuning for width
        let pulse1 = if phase < 0.45 { 1.0 } else { -1.0 };
        let phase2 = (phase + 0.003) % 1.0; // Slight phase offset
        let pulse2 = if phase2 < 0.45 { 1.0 } else { -1.0 };

        // Saw wave for brightness
        let saw = 2.0 * phase - 1.0;

        // Mix oscillators
        let osc_mix = pulse1 * 0.55 + pulse2 * 0.15 + saw * 0.30;

        // Sub oscillator with proper phase tracking
        sub_phase += (freq * 0.5) / SAMPLE_RATE;
        sub_phase %= 1.0;
        let sub = (sub_phase * 2.0 * PI).sin() * 0.28;

        // 2-pole low-pass filter for punch
        let cutoff = 0.35;
        lp1 += cutoff * (osc_mix - lp1);
        lp2 += cutoff * (lp1 - lp2);

        // Mix filtered oscillator with sub
        let mixed = lp2 * 0.72 + sub;

        // Saturation for energy
        let saturated = (mixed * 1.2).tanh();

        let sample = saturated * envelope * 27000.0;
        output.push(sample.clamp(-32767.0, 32767.0) as i16);
    }

    output
}

/// Supersaw: Enhanced 7-oscillator supersaw with maximum width
pub fn generate_supersaw() -> Vec<i16> {
    let duration = 0.8; // 800ms
    let freq = 261.63; // C4 as base
    let samples = (SAMPLE_RATE * duration) as usize;

    let mut output = Vec::with_capacity(samples);

    // 7 oscillator detune for wider, richer sound
    let detune_cents: [f32; 7] = [-18.0, -10.0, -4.0, 0.0, 4.0, 10.0, 18.0];
    let detune_ratios: Vec<f32> = detune_cents
        .iter()
        .map(|c| 2.0f32.powf(c / 1200.0))
        .collect();

    // 2-pole filter for smoother tone
    let mut lp1 = 0.0f32;
    let mut lp2 = 0.0f32;
    let mut phases = [0.0f32; 7];
    let mut vibrato_phase = 0.0f32;

    for i in 0..samples {
        let t = i as f32 / SAMPLE_RATE;

        // Improved ADSR envelope
        let envelope = if t < 0.008 {
            (t / 0.008).powf(0.7) // Curved attack
        } else if t < 0.48 {
            1.0 - (t - 0.008) * 0.14
        } else {
            0.86 * (-(t - 0.48) * 3.2).exp()
        };

        // Subtle vibrato
        vibrato_phase += 5.8 / SAMPLE_RATE;
        let vibrato = 1.0 + 0.0035 * (vibrato_phase * 2.0 * PI).sin();

        // Sum 7 detuned saws with proper phase accumulation
        let mut saw_sum = 0.0f32;
        for (idx, ratio) in detune_ratios.iter().enumerate() {
            let osc_freq = freq * ratio * vibrato;
            phases[idx] += osc_freq / SAMPLE_RATE;
            phases[idx] %= 1.0;

            // Saw wave
            let saw = 2.0 * phases[idx] - 1.0;
            saw_sum += saw;
        }
        saw_sum /= 7.0; // Normalize

        // Add subtle pulse wave layer for character (8% mix)
        let pulse = if phases[3] < 0.5 { 1.0 } else { -1.0 };
        let mixed = saw_sum * 0.92 + pulse * 0.08;

        // 2-pole low-pass with high cutoff for brightness
        let cutoff = 0.30 + 0.08 * envelope; // Filter opens with envelope
        lp1 += cutoff * (mixed - lp1);
        lp2 += cutoff * (lp1 - lp2);

        // Subtle saturation for energy
        let saturated = (lp2 * 1.1).tanh();

        let sample = saturated * envelope * 27000.0;
        output.push(sample.clamp(-32767.0, 32767.0) as i16);
    }

    output
}

/// Eurobeat brass: Short punchy stabs with tight envelope
pub fn generate_brass_euro() -> Vec<i16> {
    let duration = 0.2; // 200ms - SHORT for punchy stabs!
    let freq = 261.63; // C4 as base
    let samples = (SAMPLE_RATE * duration) as usize;

    let mut output = Vec::with_capacity(samples);

    // 3-pole resonant filter
    let mut lp1 = 0.0f32;
    let mut lp2 = 0.0f32;
    let mut lp3 = 0.0f32;
    let mut phase1 = 0.0f32;
    let mut phase2 = 0.0f32;
    let mut phase3 = 0.0f32;

    for i in 0..samples {
        let t = i as f32 / SAMPLE_RATE;

        // PUNCHY ADSR envelope - fast attack, quick decay, clean release
        let envelope = if t < 0.005 {
            t / 0.005 // 5ms attack (super fast!)
        } else if t < 0.08 {
            1.0 - (t - 0.005) * 6.0 // Fast decay to ~55%
        } else if t < 0.15 {
            0.55 // Short sustain
        } else {
            0.55 * (-(t - 0.15) * 20.0).exp() // Quick release
        };

        // Subtle pitch bend: start slightly sharp, settle to correct pitch (Eurobeat brass is TIGHT)
        // Only 0.3% max deviation (5 cents), settles quickly
        let pitch_bend = 1.0 + 0.003 * (-t * 25.0).exp();

        // Three detuned pulse waves for thickness
        phase1 += freq * pitch_bend / SAMPLE_RATE;
        phase1 %= 1.0;
        phase2 += freq * pitch_bend * 1.005 / SAMPLE_RATE;
        phase2 %= 1.0;
        phase3 += freq * pitch_bend * 0.995 / SAMPLE_RATE;
        phase3 %= 1.0;

        let pw = 0.38; // Narrow pulse for brass character
        let pulse1 = if phase1 < pw { 1.0 } else { -1.0 };
        let pulse2 = if phase2 < pw { 1.0 } else { -1.0 };
        let pulse3 = if phase3 < pw { 1.0 } else { -1.0 };

        // Add saw for brightness
        let saw = 2.0 * phase1 - 1.0;

        // Mix oscillators
        let osc_mix = pulse1 * 0.35 + pulse2 * 0.30 + pulse3 * 0.15 + saw * 0.20;

        // 3-pole resonant filter with envelope
        let filter_cutoff = 0.08 + 0.18 * envelope;
        let resonance = 0.12 * envelope; // Adds brass "buzz"
        lp1 += filter_cutoff * (osc_mix - lp1) + resonance * (osc_mix - lp1);
        lp2 += filter_cutoff * (lp1 - lp2);
        lp3 += filter_cutoff * (lp2 - lp3);

        // Saturation for punch
        let saturated = (lp3 * 1.2).tanh();

        let sample = saturated * envelope * 26000.0;
        output.push(sample.clamp(-32767.0, 32767.0) as i16);
    }

    output
}

/// Eurobeat pad: Enhanced lush pad with 5 oscillators and movement
pub fn generate_pad_euro() -> Vec<i16> {
    let duration = 1.5; // 1.5 seconds for long sustain
    let freq = 261.63; // C4 as base
    let samples = (SAMPLE_RATE * duration) as usize;

    let mut output = Vec::with_capacity(samples);

    // 3-pole filter for smooth character
    let mut lp1 = 0.0f32;
    let mut lp2 = 0.0f32;
    let mut lp3 = 0.0f32;

    let mut phases = [0.0f32; 5];
    let mut lfo_phase = 0.0f32;

    // 5 oscillator detuning for width
    let detune_amounts = [0.992, 0.997, 1.0, 1.003, 1.008];

    for i in 0..samples {
        let t = i as f32 / SAMPLE_RATE;

        // Slow, smooth attack
        let envelope = if t < 0.12 {
            (t / 0.12).powf(1.5) // Very smooth attack curve
        } else if t < 0.95 {
            1.0
        } else {
            (-(t - 0.95) * 2.2).exp()
        };

        // Subtle LFO for movement
        lfo_phase += 0.3 / SAMPLE_RATE;
        let lfo = (lfo_phase * 2.0 * PI).sin() * 0.002;

        // 5 detuned saws with proper phase accumulation
        let mut saw_sum = 0.0f32;
        for (idx, detune) in detune_amounts.iter().enumerate() {
            phases[idx] += freq * detune * (1.0 + lfo) / SAMPLE_RATE;
            phases[idx] %= 1.0;

            // Mix saw and triangle for warmth
            let saw = 2.0 * phases[idx] - 1.0;
            let tri = 4.0 * (phases[idx] - 0.5).abs() - 1.0;
            saw_sum += saw * 0.75 + tri * 0.25;
        }
        saw_sum /= 5.0;

        // 3-pole low-pass for ultra-smooth pad character
        let cutoff = 0.18;
        lp1 += cutoff * (saw_sum - lp1);
        lp2 += cutoff * (lp1 - lp2);
        lp3 += cutoff * (lp2 - lp3);

        let sample = lp3 * envelope * 32000.0;
        output.push(sample.clamp(-32767.0, 32767.0) as i16);
    }

    output
}
