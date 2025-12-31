//! Funk/Jazz instrument synthesis
//!
//! Instruments for "Nether Groove" - Funky Jazz at 110 BPM in F Dorian

use std::f32::consts::PI;
use super::common::{SimpleRng, SAMPLE_RATE};

/// Funk kick: warmer, less aggressive pitch sweep, good pocket feel
pub fn generate_kick_funk() -> Vec<i16> {
    let duration = 0.35; // 350ms for more body
    let samples = (SAMPLE_RATE * duration) as usize;

    let mut output = Vec::with_capacity(samples);
    let mut phase = 0.0f32;
    let mut click_phase = 0.0f32;

    // Multi-pole filter state (2-pole for smoother tone)
    let mut lp1 = 0.0f32;
    let mut lp2 = 0.0f32;

    for i in 0..samples {
        let t = i as f32 / SAMPLE_RATE;

        // Two-stage decay: fast transient + body sustain
        let decay = if t < 0.05 {
            (-t * 8.0).exp()
        } else {
            0.67 * (-((t - 0.05) * 9.0)).exp()
        };

        // Two-stage pitch sweep for more character
        // Initial click: 250Hz → 120Hz (first 30ms)
        // Body: 120Hz → 45Hz (rest of duration)
        let freq = if t < 0.03 {
            250.0 * (-t * 40.0).exp() + 120.0
        } else {
            120.0 * (-((t - 0.03) * 10.0)).exp() + 45.0
        };

        // Main body oscillator
        phase += 2.0 * PI * freq / SAMPLE_RATE;
        let body = phase.sin();

        // Click transient (high-pitched, very short)
        click_phase += 2.0 * PI * 800.0 / SAMPLE_RATE;
        let click_env = (-t * 150.0).exp();
        let click = click_phase.sin() * click_env * 0.25;

        // Add subtle 2nd harmonic for punch
        let harmonic = (phase * 2.0).sin() * 0.15 * (-t * 18.0).exp();

        // Combine layers
        let raw = body + click + harmonic;

        // 2-pole low-pass filter for warmth (cutoff ~800Hz)
        let cutoff = 0.18;
        lp1 += cutoff * (raw - lp1);
        lp2 += cutoff * (lp1 - lp2);

        // Soft saturation with musical curve
        let saturated = (lp2 * 1.3).tanh();

        let sample = saturated * decay * 31000.0;
        output.push(sample.clamp(-32767.0, 32767.0) as i16);
    }

    output
}

/// Funk snare: medium decay, good for ghost notes, less harsh
pub fn generate_snare_funk() -> Vec<i16> {
    let duration = 0.25; // 250ms
    let samples = (SAMPLE_RATE * duration) as usize;

    let mut output = Vec::with_capacity(samples);
    let mut rng = SimpleRng::new(12345);

    // Multi-pole filter states (2-pole LP + 1-pole HP for band-pass)
    let mut lp1 = 0.0f32;
    let mut lp2 = 0.0f32;
    let mut hp_prev_in = 0.0f32;
    let mut hp_prev_out = 0.0f32;

    for i in 0..samples {
        let t = i as f32 / SAMPLE_RATE;

        // Two-stage envelope: fast attack + medium release with tail
        let amp_env = if t < 0.002 {
            t / 0.002 // 2ms attack for snap
        } else if t < 0.12 {
            (-((t - 0.002) * 12.0)).exp()
        } else {
            0.30 * (-((t - 0.12) * 8.0)).exp() // Subtle tail
        };

        // Shaped noise (pink-ish for warmth)
        let white_noise = rng.next_f32() * 2.0 - 1.0;

        // High-pass filter for brightness
        let hp_alpha = 0.70;
        let hp_out = hp_alpha * (hp_prev_out + white_noise - hp_prev_in);
        hp_prev_in = white_noise;
        hp_prev_out = hp_out;

        // Body resonances (multiple modes like a real snare shell)
        let body1 = (2.0 * PI * 160.0 * t).sin() * 0.40; // Fundamental
        let body2 = (2.0 * PI * 210.0 * t).sin() * 0.25; // Second mode
        let body3 = (2.0 * PI * 295.0 * t).sin() * 0.15; // Third mode
        let body = (body1 + body2 + body3) * (-t * 18.0).exp();

        // Snare wire rattle (high-frequency burst)
        let rattle_env = (-t * 35.0).exp();
        let rattle = hp_out * rattle_env;

        // Transient snap (very short)
        let snap = (2.0 * PI * 380.0 * t).sin() * (-t * 45.0).exp() * 0.20;

        // Mix components
        let raw = body + rattle * 0.50 + snap;

        // 2-pole low-pass for smoothness
        let cutoff = 0.35;
        lp1 += cutoff * (raw - lp1);
        lp2 += cutoff * (lp1 - lp2);

        // Gentle saturation
        let saturated = (lp2 * 1.15).tanh();

        let sample = saturated * amp_env * 29000.0;
        output.push(sample.clamp(-32767.0, 32767.0) as i16);
    }

    output
}

/// Funk hi-hat: warmer, slightly longer decay for groove
pub fn generate_hihat_funk() -> Vec<i16> {
    let duration = 0.12; // 120ms
    let samples = (SAMPLE_RATE * duration) as usize;

    let mut output = Vec::with_capacity(samples);
    let mut rng = SimpleRng::new(54321);

    // Multi-stage filter states (HP + multi-pole BP for metallic character)
    let mut hp1_prev_in = 0.0f32;
    let mut hp1_prev_out = 0.0f32;
    let mut hp2_prev_in = 0.0f32;
    let mut hp2_prev_out = 0.0f32;
    let mut lp1 = 0.0f32;
    let mut lp2 = 0.0f32;

    for i in 0..samples {
        let t = i as f32 / SAMPLE_RATE;

        // Two-stage envelope with attack
        let amp_env = if t < 0.005 {
            t / 0.005 // 5ms attack for smoothness
        } else if t < 0.025 {
            1.0 - (t - 0.005) * 0.2 // Quick initial decay
        } else {
            0.80 * (-((t - 0.025) * 22.0)).exp()
        };

        // Generate noise
        let noise = rng.next_f32() * 2.0 - 1.0;

        // 2-stage high-pass for brightness (cascaded for steeper slope)
        let hp_alpha = 0.88;
        let hp1_out = hp_alpha * (hp1_prev_out + noise - hp1_prev_in);
        hp1_prev_in = noise;
        hp1_prev_out = hp1_out;

        let hp2_out = hp_alpha * (hp2_prev_out + hp1_out - hp2_prev_in);
        hp2_prev_in = hp1_out;
        hp2_prev_out = hp2_out;

        // Add metallic resonances (multiple inharmonic modes)
        let metal1 = (2.0 * PI * 7200.0 * t).sin() * 0.08;
        let metal2 = (2.0 * PI * 9300.0 * t).sin() * 0.05;
        let metal3 = (2.0 * PI * 11500.0 * t).sin() * 0.03;
        let metallic = (metal1 + metal2 + metal3) * (-t * 30.0).exp();

        // Combine noise and metallic components
        let raw = hp2_out * 0.85 + metallic;

        // 2-pole low-pass for smooth tone (not harsh)
        let cutoff = 0.55;
        lp1 += cutoff * (raw - lp1);
        lp2 += cutoff * (lp1 - lp2);

        let sample = lp2 * amp_env * 23000.0;
        output.push(sample.clamp(-32767.0, 32767.0) as i16);
    }

    output
}

/// Funk bass: sawtooth with filter envelope "pluck", chromatic-friendly
/// IMPROVED: Richer harmonics, better filter, tighter low-end
pub fn generate_bass_funk() -> Vec<i16> {
    let duration = 0.55; // 550ms total (400ms sustain + release)
    let freq = 87.31; // F2 as base
    let samples = (SAMPLE_RATE * duration) as usize;

    let mut output = Vec::with_capacity(samples);

    // Multi-pole filter (3-pole for resonant character)
    let mut lp1 = 0.0f32;
    let mut lp2 = 0.0f32;
    let mut lp3 = 0.0f32;
    let mut phase = 0.0f32;
    let mut sub_phase = 0.0f32;

    for i in 0..samples {
        let t = i as f32 / SAMPLE_RATE;

        // Improved envelope with smooth attack
        let amp_env = if t < 0.005 {
            (t / 0.005).powf(0.8) // Slightly curved for smoothness
        } else if t < 0.4 {
            1.0 - (t - 0.005) * 0.12 // Slow decay to sustain
        } else {
            0.95 * (-(t - 0.4) * 3.2).exp() // Smooth release
        };

        // Filter envelope with resonance peak for "slap"
        let filter_env = 0.06 + 0.32 * (-t * 18.0).exp();
        let resonance = 0.15 * (-t * 25.0).exp();

        // Subtle pitch bend down on attack (funky!)
        let pitch_bend = 1.0 + 0.018 * (-t * 28.0).exp();

        // Main sawtooth with proper phase accumulation
        phase += freq * pitch_bend / SAMPLE_RATE;
        phase %= 1.0;
        let saw = 2.0 * phase - 1.0;

        // Add subtle square wave for more harmonics (20% mix)
        let square = if phase < 0.5 { 1.0 } else { -1.0 };
        let osc_mix = saw * 0.80 + square * 0.20;

        // Sub oscillator with proper phase tracking
        sub_phase += (freq * 0.5 * pitch_bend) / SAMPLE_RATE;
        sub_phase %= 1.0;
        let sub = (sub_phase * 2.0 * PI).sin() * 0.38;

        // 3-pole resonant filter (adds character)
        lp1 += filter_env * (osc_mix - lp1) + resonance * (osc_mix - lp1);
        lp2 += filter_env * (lp1 - lp2);
        lp3 += filter_env * (lp2 - lp3);

        // Mix filtered oscillator with sub
        let mixed = lp3 * 0.68 + sub;

        // Gentle saturation for analog warmth
        let saturated = (mixed * 1.2).tanh();

        let sample = saturated * amp_env * 29000.0;
        output.push(sample.clamp(-32767.0, 32767.0) as i16);
    }

    output
}

/// Electric Piano: Enhanced FM synthesis for Rhodes/Wurlitzer bell-like tone
pub fn generate_epiano() -> Vec<i16> {
    let duration = 1.0; // 1 second for chord sustain
    let freq = 261.63; // C4 as base
    let samples = (SAMPLE_RATE * duration) as usize;

    let mut output = Vec::with_capacity(samples);

    // Filter state for warmth
    let mut lp1 = 0.0f32;
    let mut lp2 = 0.0f32;

    for i in 0..samples {
        let t = i as f32 / SAMPLE_RATE;

        // Improved ADSR envelope (more natural decay)
        let amp_env = if t < 0.008 {
            (t / 0.008).powf(0.7) // Curved attack for smoothness
        } else if t < 0.25 {
            1.0 - (t - 0.008) * 0.35 // Faster initial decay
        } else if t < 0.65 {
            0.92 - (t - 0.25) * 0.25 // Gradual sustain decay
        } else {
            0.82 * (-(t - 0.65) * 4.5).exp() // Release
        };

        // FM synthesis with multiple operators for richer tone
        // Operator 1: Main bell tone (2:1 ratio)
        let mod_freq1 = freq * 2.0;
        let mod_index1 = 2.8 * (-t * 9.0).exp(); // Decaying modulation
        let modulator1 = (2.0 * PI * mod_freq1 * t).sin() * mod_index1;
        let carrier1 = (2.0 * PI * freq * t + modulator1).sin();

        // Operator 2: Subtle inharmonic component (2.73:1 for character)
        let mod_freq2 = freq * 2.73;
        let mod_index2 = 1.2 * (-t * 12.0).exp();
        let modulator2 = (2.0 * PI * mod_freq2 * t).sin() * mod_index2;
        let carrier2 = (2.0 * PI * freq * 1.01 * t + modulator2).sin() * 0.25; // Slightly detuned

        // Operator 3: Upper partial (3:1 ratio for brightness)
        let partial3 = (2.0 * PI * freq * 3.0 * t).sin() * 0.12 * (-t * 15.0).exp();

        // Low-frequency body resonance (characteristic of Rhodes)
        let body = (2.0 * PI * freq * 0.5 * t).sin() * 0.08 * (-t * 6.0).exp();

        // Mix operators
        let mixed = carrier1 * 0.70 + carrier2 + partial3 + body;

        // Gentle 2-pole low-pass for warmth (simulates speaker/pickup)
        let cutoff = 0.28;
        lp1 += cutoff * (mixed - lp1);
        lp2 += cutoff * (lp1 - lp2);

        // Subtle saturation for analog character
        let saturated = (lp2 * 1.15).tanh();

        let sample = saturated * amp_env * 25000.0;
        output.push(sample.clamp(-32767.0, 32767.0) as i16);
    }

    output
}

/// Jazz lead: Enhanced filtered square with vibrato and breath
pub fn generate_lead_jazz() -> Vec<i16> {
    let duration = 0.8; // 800ms
    let freq = 261.63; // C4 as base
    let samples = (SAMPLE_RATE * duration) as usize;

    let mut output = Vec::with_capacity(samples);

    // Multi-pole filter states (3-pole for smooth rolloff)
    let mut lp1 = 0.0f32;
    let mut lp2 = 0.0f32;
    let mut lp3 = 0.0f32;

    let mut phase = 0.0f32;
    let mut vibrato_phase = 0.0f32;
    let mut breath_phase = 0.0f32;

    for i in 0..samples {
        let t = i as f32 / SAMPLE_RATE;

        // Improved ADSR envelope (smoother curves)
        let envelope = if t < 0.025 {
            (t / 0.025).powf(0.6) // Curved attack for breath-like onset
        } else if t < 0.45 {
            1.0 - (t - 0.025) * 0.18 // Slow decay to sustain
        } else {
            0.91 * (-(t - 0.45) * 3.8).exp() // Release
        };

        // Delayed vibrato (jazz style) with gradual fade-in
        let vibrato_amount = if t < 0.12 {
            0.0
        } else {
            0.0045 * ((t - 0.12) * 2.5).min(1.0).powf(0.7)
        };
        vibrato_phase += 5.2 / SAMPLE_RATE; // 5.2 Hz vibrato rate
        let vibrato = 1.0 + vibrato_amount * (vibrato_phase * 2.0 * PI).sin();

        // Subtle breath modulation (very low frequency)
        breath_phase += 0.8 / SAMPLE_RATE;
        let breath_mod = 1.0 + 0.015 * (breath_phase * 2.0 * PI).sin();

        // Accumulate phase with modulations
        phase += freq * vibrato * breath_mod / SAMPLE_RATE;
        phase %= 1.0;

        // Variable pulse width square wave (adds harmonic movement)
        let pw = 0.48 + 0.04 * (t * 1.8).sin();
        let square = if phase < pw { 1.0 } else { -1.0 };

        // Add subtle triangle wave for warmth (10% mix)
        let triangle = 4.0 * (phase - 0.5).abs() - 1.0;
        let osc_mix = square * 0.90 + triangle * 0.10;

        // 3-pole low-pass filter with envelope control
        let filter_cutoff = 0.10 + 0.05 * envelope;
        lp1 += filter_cutoff * (osc_mix - lp1);
        lp2 += filter_cutoff * (lp1 - lp2);
        lp3 += filter_cutoff * (lp2 - lp3);

        // Gentle saturation for tube-like warmth
        let saturated = (lp3 * 1.1).tanh();

        let sample = saturated * envelope * 23000.0;
        output.push(sample.clamp(-32767.0, 32767.0) as i16);
    }

    output
}
