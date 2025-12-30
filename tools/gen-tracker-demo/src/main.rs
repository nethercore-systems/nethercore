//! Generates procedural audio samples and XM tracker files for tracker-demo example
//!
//! Creates three distinct songs:
//! - nether_groove.xm - Funky Jazz (default, 110 BPM, F Dorian) - Purple theme
//! - nether_fire.xm - Eurobeat (155 BPM, D minor) - Orange theme
//! - nether_drive.xm - Synthwave (105 BPM, A minor) - Green theme
//!
//! Each song has its own instrument set optimized for the genre.

use std::f32::consts::PI;
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;

const SAMPLE_RATE: f32 = 22050.0;

/// Fade-in duration in seconds (prevents clicks from abrupt sample starts)
const FADE_IN_SECS: f32 = 0.002; // 2ms

/// Fade-out duration in seconds (prevents clicks from sample cutoffs)
const FADE_OUT_SECS: f32 = 0.005; // 5ms

/// Apply fade-in and fade-out to a sample buffer to prevent clicks
fn apply_fades(samples: &mut [i16]) {
    let fade_in_samples = (SAMPLE_RATE * FADE_IN_SECS) as usize;
    let fade_out_samples = (SAMPLE_RATE * FADE_OUT_SECS) as usize;

    // Fade in
    for i in 0..fade_in_samples.min(samples.len()) {
        let factor = i as f32 / fade_in_samples as f32;
        samples[i] = (samples[i] as f32 * factor) as i16;
    }

    // Fade out
    let start = samples.len().saturating_sub(fade_out_samples);
    for i in start..samples.len() {
        let factor = (samples.len() - i) as f32 / fade_out_samples as f32;
        samples[i] = (samples[i] as f32 * factor) as i16;
    }
}

fn main() {
    // Output to shared examples/assets folder with tracker- prefix
    let output_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("examples")
        .join("assets");

    // Create output directory if it doesn't exist
    fs::create_dir_all(&output_dir).expect("Failed to create assets directory");

    println!("Generating tracker-demo assets to shared examples/assets...");

    // Generate Funky Jazz song (default)
    println!("\n=== Generating 'Nether Groove' (Funky Jazz) ===");
    generate_funk_assets(&output_dir);

    // Generate Eurobeat song
    println!("\n=== Generating 'Nether Fire' (Eurobeat) ===");
    generate_eurobeat_assets(&output_dir);

    // Generate Synthwave song
    println!("\n=== Generating 'Nether Drive' (Synthwave) ===");
    generate_synthwave_assets(&output_dir);

    println!("\nDone!");
}

// ============================================================================
// FUNKY JAZZ SONG - "Nether Groove"
// ============================================================================

fn generate_funk_assets(output_dir: &Path) {
    // Generate funk instruments (with tracker- prefix for shared assets folder)
    let mut kick = generate_kick_funk();
    apply_fades(&mut kick);
    write_wav(&output_dir.join("tracker-kick_funk.wav"), &kick);
    println!("  Generated tracker-kick_funk.wav ({} samples)", kick.len());

    let mut snare = generate_snare_funk();
    apply_fades(&mut snare);
    write_wav(&output_dir.join("tracker-snare_funk.wav"), &snare);
    println!("  Generated tracker-snare_funk.wav ({} samples)", snare.len());

    let mut hihat = generate_hihat_funk();
    apply_fades(&mut hihat);
    write_wav(&output_dir.join("tracker-hihat_funk.wav"), &hihat);
    println!("  Generated tracker-hihat_funk.wav ({} samples)", hihat.len());

    let mut bass = generate_bass_funk();
    apply_fades(&mut bass);
    write_wav(&output_dir.join("tracker-bass_funk.wav"), &bass);
    println!("  Generated tracker-bass_funk.wav ({} samples)", bass.len());

    let mut epiano = generate_epiano();
    apply_fades(&mut epiano);
    write_wav(&output_dir.join("tracker-epiano.wav"), &epiano);
    println!("  Generated tracker-epiano.wav ({} samples)", epiano.len());

    let mut lead = generate_lead_jazz();
    apply_fades(&mut lead);
    write_wav(&output_dir.join("tracker-lead_jazz.wav"), &lead);
    println!("  Generated tracker-lead_jazz.wav ({} samples)", lead.len());

    // Generate sample-less XM file
    let xm = generate_funk_xm();
    fs::write(output_dir.join("tracker-nether_groove.xm"), &xm).expect("Failed to write tracker-nether_groove.xm");
    println!("  Generated tracker-nether_groove.xm ({} bytes)", xm.len());

    // Generate embedded XM file
    let samples = vec![kick, snare, hihat, bass, epiano, lead];
    let xm_embedded = generate_funk_xm_embedded(&samples);
    fs::write(output_dir.join("tracker-nether_groove-embedded.xm"), &xm_embedded).expect("Failed to write tracker-nether_groove-embedded.xm");
    println!("  Generated tracker-nether_groove-embedded.xm ({} bytes)", xm_embedded.len());
}

/// Funk kick: warmer, less aggressive pitch sweep, good pocket feel
fn generate_kick_funk() -> Vec<i16> {
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
fn generate_snare_funk() -> Vec<i16> {
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
fn generate_hihat_funk() -> Vec<i16> {
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
fn generate_bass_funk() -> Vec<i16> {
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
        phase = phase % 1.0;
        let saw = 2.0 * phase - 1.0;

        // Add subtle square wave for more harmonics (20% mix)
        let square = if phase < 0.5 { 1.0 } else { -1.0 };
        let osc_mix = saw * 0.80 + square * 0.20;

        // Sub oscillator with proper phase tracking
        sub_phase += (freq * 0.5 * pitch_bend) / SAMPLE_RATE;
        sub_phase = sub_phase % 1.0;
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
fn generate_epiano() -> Vec<i16> {
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
fn generate_lead_jazz() -> Vec<i16> {
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
        phase = phase % 1.0;

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

// ============================================================================
// EUROBEAT SONG - "Nether Fire"
// ============================================================================

fn generate_eurobeat_assets(output_dir: &Path) {
    // Generate eurobeat instruments (with tracker- prefix for shared assets folder)
    let mut kick = generate_kick_euro();
    apply_fades(&mut kick);
    write_wav(&output_dir.join("tracker-kick_euro.wav"), &kick);
    println!("  Generated tracker-kick_euro.wav ({} samples)", kick.len());

    let mut snare = generate_snare_euro();
    apply_fades(&mut snare);
    write_wav(&output_dir.join("tracker-snare_euro.wav"), &snare);
    println!("  Generated tracker-snare_euro.wav ({} samples)", snare.len());

    let mut hihat = generate_hihat_euro();
    apply_fades(&mut hihat);
    write_wav(&output_dir.join("tracker-hihat_euro.wav"), &hihat);
    println!("  Generated tracker-hihat_euro.wav ({} samples)", hihat.len());

    let mut bass = generate_bass_euro();
    apply_fades(&mut bass);
    write_wav(&output_dir.join("tracker-bass_euro.wav"), &bass);
    println!("  Generated tracker-bass_euro.wav ({} samples)", bass.len());

    let mut supersaw = generate_supersaw();
    apply_fades(&mut supersaw);
    write_wav(&output_dir.join("tracker-supersaw.wav"), &supersaw);
    println!("  Generated tracker-supersaw.wav ({} samples)", supersaw.len());

    let mut brass = generate_brass_euro();
    apply_fades(&mut brass);
    write_wav(&output_dir.join("tracker-brass_euro.wav"), &brass);
    println!("  Generated tracker-brass_euro.wav ({} samples)", brass.len());

    let mut pad = generate_pad_euro();
    apply_fades(&mut pad);
    write_wav(&output_dir.join("tracker-pad_euro.wav"), &pad);
    println!("  Generated tracker-pad_euro.wav ({} samples)", pad.len());

    // Generate sample-less XM file
    let xm = generate_eurobeat_xm();
    fs::write(output_dir.join("tracker-nether_fire.xm"), &xm).expect("Failed to write tracker-nether_fire.xm");
    println!("  Generated tracker-nether_fire.xm ({} bytes)", xm.len());

    // Generate embedded XM file
    let samples = vec![kick, snare, hihat, bass, supersaw, brass, pad];
    let xm_embedded = generate_eurobeat_xm_embedded(&samples);
    fs::write(output_dir.join("tracker-nether_fire-embedded.xm"), &xm_embedded).expect("Failed to write tracker-nether_fire-embedded.xm");
    println!("  Generated tracker-nether_fire-embedded.xm ({} bytes)", xm_embedded.len());
}

/// Eurobeat kick: 909-style, punchy with aggressive pitch sweep
/// Eurobeat kick: Improved 909-style with aggressive punch and harmonics
fn generate_kick_euro() -> Vec<i16> {
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
fn generate_snare_euro() -> Vec<i16> {
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
fn generate_hihat_euro() -> Vec<i16> {
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
fn generate_bass_euro() -> Vec<i16> {
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
        phase = phase % 1.0;

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
        sub_phase = sub_phase % 1.0;
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
fn generate_supersaw() -> Vec<i16> {
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
            phases[idx] = phases[idx] % 1.0;

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

/// Eurobeat brass: Enhanced with richer harmonics and filter resonance
fn generate_brass_euro() -> Vec<i16> {
    let duration = 0.7; // 700ms
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

        // Improved ADSR envelope
        let envelope = if t < 0.012 {
            (t / 0.012).powf(0.6) // Curved attack
        } else if t < 0.38 {
            1.0 - (t - 0.012) * 0.18
        } else {
            0.82 * (-(t - 0.38) * 3.8).exp()
        };

        // Pitch bend with overshoot for realism
        let pitch_bend = 1.0 + 0.018 * (1.0 - (-t * 18.0).exp()) - 0.003 * (-t * 12.0).exp();

        // Three detuned pulse waves for thickness
        phase1 += freq * pitch_bend / SAMPLE_RATE;
        phase1 = phase1 % 1.0;
        phase2 += freq * pitch_bend * 1.005 / SAMPLE_RATE;
        phase2 = phase2 % 1.0;
        phase3 += freq * pitch_bend * 0.995 / SAMPLE_RATE;
        phase3 = phase3 % 1.0;

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
fn generate_pad_euro() -> Vec<i16> {
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
            phases[idx] = phases[idx] % 1.0;

            // Mix saw and triangle for warmth
            let saw = 2.0 * phases[idx] - 1.0;
            let tri = 4.0 * (phases[idx] - 0.5).abs() - 1.0;
            saw_sum += saw * 0.75 + tri * 0.25;
        }
        saw_sum /= 5.0;

        // 3-pole low-pass for ultra-smooth pad character
        let cutoff = 0.18; // FIXED: Was 0.05 (way too aggressive!)
        lp1 += cutoff * (saw_sum - lp1);
        lp2 += cutoff * (lp1 - lp2);
        lp3 += cutoff * (lp2 - lp3);

        let sample = lp3 * envelope * 32000.0; // Increased gain to compensate for filtering
        output.push(sample.clamp(-32767.0, 32767.0) as i16);
    }

    output
}

// ============================================================================
// Simple PRNG (xorshift32)
// ============================================================================

struct SimpleRng {
    state: u32,
}

impl SimpleRng {
    fn new(seed: u32) -> Self {
        Self {
            state: if seed == 0 { 1 } else { seed },
        }
    }

    fn next(&mut self) -> u32 {
        let mut x = self.state;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.state = x;
        x
    }

    fn next_f32(&mut self) -> f32 {
        self.next() as f32 / u32::MAX as f32
    }
}

// ============================================================================
// WAV File Writer
// ============================================================================

fn write_wav(path: &Path, samples: &[i16]) {
    let mut file = File::create(path).expect("Failed to create WAV file");
    let data_size = (samples.len() * 2) as u32;

    // RIFF header
    file.write_all(b"RIFF").unwrap();
    file.write_all(&(36 + data_size).to_le_bytes()).unwrap();
    file.write_all(b"WAVE").unwrap();

    // fmt chunk (16 bytes)
    file.write_all(b"fmt ").unwrap();
    file.write_all(&16u32.to_le_bytes()).unwrap(); // chunk size
    file.write_all(&1u16.to_le_bytes()).unwrap(); // audio format (1 = PCM)
    file.write_all(&1u16.to_le_bytes()).unwrap(); // num channels (mono)
    file.write_all(&22050u32.to_le_bytes()).unwrap(); // sample rate
    file.write_all(&44100u32.to_le_bytes()).unwrap(); // byte rate (22050 * 2)
    file.write_all(&2u16.to_le_bytes()).unwrap(); // block align (2 bytes)
    file.write_all(&16u16.to_le_bytes()).unwrap(); // bits per sample

    // data chunk
    file.write_all(b"data").unwrap();
    file.write_all(&data_size.to_le_bytes()).unwrap();
    for sample in samples {
        file.write_all(&sample.to_le_bytes()).unwrap();
    }
}

// ============================================================================
// XM File Generation - Funky Jazz "Nether Groove"
// ============================================================================

fn generate_funk_xm() -> Vec<u8> {
    let mut xm = Vec::new();

    // XM Header
    xm.extend_from_slice(b"Extended Module: ");

    // Module name (20 bytes)
    let name = b"Nether Groove";
    xm.extend_from_slice(name);
    xm.extend(std::iter::repeat(0u8).take(20 - name.len()));

    xm.push(0x1A);

    // Tracker name
    let tracker = b"gen-tracker-demo";
    xm.extend_from_slice(tracker);
    xm.extend(std::iter::repeat(0u8).take(20 - tracker.len()));

    // Version
    xm.extend_from_slice(&0x0104u16.to_le_bytes());

    // Header size (276 = 4 bytes header_size + 16 bytes of header fields + 256 byte order table)
    // Per XM spec, header_size is measured from the position of this field itself
    xm.extend_from_slice(&276u32.to_le_bytes());

    // Song length (10 orders)
    xm.extend_from_slice(&10u16.to_le_bytes());

    // Restart position
    xm.extend_from_slice(&1u16.to_le_bytes());

    // Number of channels (8)
    xm.extend_from_slice(&8u16.to_le_bytes());

    // Number of patterns (6)
    xm.extend_from_slice(&6u16.to_le_bytes());

    // Number of instruments (6)
    xm.extend_from_slice(&6u16.to_le_bytes());

    // Flags (linear frequency table)
    xm.extend_from_slice(&1u16.to_le_bytes());

    // Default speed (6 ticks per row)
    xm.extend_from_slice(&6u16.to_le_bytes());

    // Default BPM (110 for funk)
    xm.extend_from_slice(&110u16.to_le_bytes());

    // Pattern order table: Intro -> Groove A -> Groove B -> (repeat) -> Bridge -> Solo -> Outro
    // [0, 1, 2, 1, 2, 3, 4, 1, 2, 5]
    let order = [0u8, 1, 2, 1, 2, 3, 4, 1, 2, 5];
    xm.extend_from_slice(&order);
    xm.extend(std::iter::repeat(0u8).take(256 - order.len()));

    // Generate patterns
    for i in 0..6 {
        let pattern_data = match i {
            0 => generate_funk_pattern_intro(),
            1 => generate_funk_pattern_groove_a(),
            2 => generate_funk_pattern_groove_b(),
            3 => generate_funk_pattern_bridge(),
            4 => generate_funk_pattern_solo(),
            5 => generate_funk_pattern_outro(),
            _ => unreachable!(),
        };
        let pattern_size = pattern_data.len() as u16;

        // Debug validation
        eprintln!("Funk Pattern {}: size={} bytes", i, pattern_size);
        if pattern_size < 256 {
            eprintln!("WARNING: Funk Pattern {} too small (expected min 256)", i);
        }

        xm.extend_from_slice(&9u32.to_le_bytes()); // header length (including length field: 4+1+2+2=9)
        xm.push(0); // packing type
        xm.extend_from_slice(&32u16.to_le_bytes()); // 32 rows
        xm.extend_from_slice(&pattern_size.to_le_bytes());
        xm.extend_from_slice(&pattern_data);
    }

    // Instruments
    let instruments = [
        "kick_funk",
        "snare_funk",
        "hihat_funk",
        "bass_funk",
        "epiano",
        "lead_jazz",
    ];
    for name in &instruments {
        write_instrument(&mut xm, name);
    }

    xm
}

fn generate_funk_xm_embedded(samples: &[Vec<i16>]) -> Vec<u8> {
    let mut xm = Vec::new();

    // XM Header
    xm.extend_from_slice(b"Extended Module: ");

    // Module name (20 bytes)
    let name = b"Nether Groove";
    xm.extend_from_slice(name);
    xm.extend(std::iter::repeat(0u8).take(20 - name.len()));

    xm.push(0x1A);

    // Tracker name
    let tracker = b"gen-tracker-demo";
    xm.extend_from_slice(tracker);
    xm.extend(std::iter::repeat(0u8).take(20 - tracker.len()));

    // Version
    xm.extend_from_slice(&0x0104u16.to_le_bytes());

    // Header size (276 = 4 bytes header_size + 16 bytes of header fields + 256 byte order table)
    xm.extend_from_slice(&276u32.to_le_bytes());

    // Song length (10 orders)
    xm.extend_from_slice(&10u16.to_le_bytes());

    // Restart position
    xm.extend_from_slice(&1u16.to_le_bytes());

    // Number of channels (8)
    xm.extend_from_slice(&8u16.to_le_bytes());

    // Number of patterns (6)
    xm.extend_from_slice(&6u16.to_le_bytes());

    // Number of instruments (6)
    xm.extend_from_slice(&6u16.to_le_bytes());

    // Flags (linear frequency table)
    xm.extend_from_slice(&1u16.to_le_bytes());

    // Default speed (6 ticks per row)
    xm.extend_from_slice(&6u16.to_le_bytes());

    // Default BPM (110 for funk)
    xm.extend_from_slice(&110u16.to_le_bytes());

    // Pattern order table
    let order = [0u8, 1, 2, 1, 2, 3, 4, 1, 2, 5];
    xm.extend_from_slice(&order);
    xm.extend(std::iter::repeat(0u8).take(256 - order.len()));

    // Generate patterns (same as sample-less version)
    for i in 0..6 {
        let pattern_data = match i {
            0 => generate_funk_pattern_intro(),
            1 => generate_funk_pattern_groove_a(),
            2 => generate_funk_pattern_groove_b(),
            3 => generate_funk_pattern_bridge(),
            4 => generate_funk_pattern_solo(),
            5 => generate_funk_pattern_outro(),
            _ => unreachable!(),
        };
        let pattern_size = pattern_data.len() as u16;

        xm.extend_from_slice(&9u32.to_le_bytes());
        xm.push(0);
        xm.extend_from_slice(&32u16.to_le_bytes());
        xm.extend_from_slice(&pattern_size.to_le_bytes());
        xm.extend_from_slice(&pattern_data);
    }

    // Instruments WITH embedded samples
    let instruments = [
        "kick_funk",
        "snare_funk",
        "hihat_funk",
        "bass_funk",
        "epiano",
        "lead_jazz",
    ];
    for (i, name) in instruments.iter().enumerate() {
        write_instrument_with_sample(&mut xm, name, &samples[i]);
    }

    xm
}

// Funk note constants (F Dorian: F G Ab Bb C D Eb)
const F2: u8 = 30;
const G2: u8 = 32;
const AB2: u8 = 33;
const BB2: u8 = 35;
const C3: u8 = 37;
const D3: u8 = 39;
const EB3: u8 = 40;
const F3: u8 = 42;
const G3: u8 = 44;
const AB3: u8 = 45;
const BB3: u8 = 47;
const C4: u8 = 49;
const D4: u8 = 51;
const EB4: u8 = 52;
const F4: u8 = 54;
const G4: u8 = 56;
const AB4: u8 = 57;
const BB4: u8 = 59;
const C5: u8 = 61;
const EB5: u8 = 64;
const F5: u8 = 66;

// Funk instruments
const KICK_F: u8 = 1;
const SNARE_F: u8 = 2;
const HIHAT_F: u8 = 3;
const BASS_F: u8 = 4;
const EPIANO: u8 = 5;
const LEAD_J: u8 = 6;

/// Helper to write a note with volume
fn write_note_vol(data: &mut Vec<u8>, note: u8, instrument: u8, volume: u8) {
    data.push(0x80 | 0x01 | 0x02 | 0x04); // packed byte: has note + instrument + volume
    data.push(note);
    data.push(instrument);
    data.push(volume);
}

/// Helper to write a note
fn write_note(data: &mut Vec<u8>, note: u8, instrument: u8) {
    data.push(0x80 | 0x01 | 0x02); // packed byte: has note + instrument
    data.push(note);
    data.push(instrument);
}

/// Helper to write an empty channel
fn write_empty(data: &mut Vec<u8>) {
    data.push(0x80); // packed byte: nothing
}

/// Funk Pattern 0: Intro - Ghost notes establish groove
fn generate_funk_pattern_intro() -> Vec<u8> {
    let mut data = Vec::new();

    for row in 0..32 {
        // Ch1: Kick - sparse
        if row == 0 || row == 16 {
            write_note(&mut data, C4, KICK_F);
        } else {
            write_empty(&mut data);
        }

        // Ch2: Snare - ghost notes only
        if row == 12 || row == 14 || row == 28 || row == 30 {
            write_note_vol(&mut data, C4, SNARE_F, 0x18); // Low volume ghost
        } else {
            write_empty(&mut data);
        }

        // Ch3: Hi-hat - sparse
        if row % 8 == 0 {
            write_note(&mut data, C4, HIHAT_F);
        } else {
            write_empty(&mut data);
        }

        // Ch4: Bass - just root notes
        if row == 0 {
            write_note(&mut data, F2, BASS_F);
        } else if row == 16 {
            write_note(&mut data, F2, BASS_F);
        } else {
            write_empty(&mut data);
        }

        // Ch5: Lead - silent in intro
        write_empty(&mut data);

        // Ch6: EPiano - chord stabs
        if row == 0 {
            write_note_vol(&mut data, F3, EPIANO, 0x30);
        } else if row == 16 {
            write_note_vol(&mut data, AB3, EPIANO, 0x30);
        } else {
            write_empty(&mut data);
        }

        // Ch7: EPiano chords - silent
        write_empty(&mut data);

        // Ch8: Lead response - silent
        write_empty(&mut data);
    }

    data
}

/// Funk Pattern 1: Groove A - Full pocket, Fm7 to Bb7
fn generate_funk_pattern_groove_a() -> Vec<u8> {
    let mut data = Vec::new();

    // Syncopated bass line for Fm7 -> Bb7
    let bass_notes = [
        F2, 0, 0, AB2, 0, C3, 0, EB3, // Fm7 arpeggio, syncopated
        F2, 0, F3, 0, EB3, 0, C3, 0, // Fm7 octave bounce
        BB2, 0, 0, D3, 0, F3, 0, AB3, // Bb7 arpeggio
        BB2, 0, BB3, 0, AB3, 0, F3, 0, // Bb7 octave bounce
    ];

    // Call melody
    let melody = [
        0, 0, 0, 0, C5, 0, EB5, 0, // Rest, then call
        F5, 0, EB5, 0, C5, 0, 0, 0, // Descending answer
        0, 0, 0, 0, D4, 0, F4, 0, // Bb7 phrase
        AB4, 0, F4, 0, D4, 0, 0, 0, // Resolution
    ];

    // Response melody
    let response = [
        0, 0, 0, 0, 0, 0, 0, 0, // Wait
        0, 0, 0, 0, 0, 0, AB4, 0, // Answer starts
        C5, 0, 0, 0, 0, 0, 0, 0, // Peak
        0, 0, BB4, 0, AB4, 0, F4, 0, // Resolve
    ];

    for row in 0..32 {
        // Ch1: Kick - funk pattern
        if row == 0 || row == 6 || row == 10 || row == 16 || row == 22 || row == 26 {
            write_note(&mut data, C4, KICK_F);
        } else {
            write_empty(&mut data);
        }

        // Ch2: Snare - backbeat + ghosts
        if row == 8 || row == 24 {
            write_note(&mut data, C4, SNARE_F); // Backbeat
        } else if row == 4 || row == 12 || row == 20 || row == 28 {
            write_note_vol(&mut data, C4, SNARE_F, 0x15); // Ghost notes
        } else {
            write_empty(&mut data);
        }

        // Ch3: Hi-hat - 8th notes with accents
        if row % 4 == 0 {
            write_note(&mut data, C4, HIHAT_F);
        } else if row % 2 == 0 {
            write_note_vol(&mut data, C4, HIHAT_F, 0x20); // Softer off-beats
        } else {
            write_empty(&mut data);
        }

        // Ch4: Bass
        let bass = bass_notes[row as usize];
        if bass != 0 {
            write_note(&mut data, bass, BASS_F);
        } else {
            write_empty(&mut data);
        }

        // Ch5: Lead melody (call)
        let mel = melody[row as usize];
        if mel != 0 {
            write_note(&mut data, mel, LEAD_J);
        } else {
            write_empty(&mut data);
        }

        // Ch6: EPiano - chord comping
        if row == 0 {
            write_note(&mut data, C4, EPIANO); // Fm7 - C
        } else if row == 2 {
            write_note(&mut data, EB4, EPIANO); // Fm7 - Eb
        } else if row == 16 {
            write_note(&mut data, D4, EPIANO); // Bb7 - D
        } else if row == 18 {
            write_note(&mut data, F4, EPIANO); // Bb7 - F
        } else {
            write_empty(&mut data);
        }

        // Ch7: EPiano - bass notes of chords
        if row == 0 || row == 8 {
            write_note(&mut data, F3, EPIANO);
        } else if row == 16 || row == 24 {
            write_note(&mut data, BB3, EPIANO);
        } else {
            write_empty(&mut data);
        }

        // Ch8: Response melody
        let resp = response[row as usize];
        if resp != 0 {
            write_note(&mut data, resp, LEAD_J);
        } else {
            write_empty(&mut data);
        }
    }

    data
}

/// Funk Pattern 2: Groove B - Eb7 to Fm7 with fills
fn generate_funk_pattern_groove_b() -> Vec<u8> {
    let mut data = Vec::new();

    // Bass line Eb7 -> Fm7
    let bass_notes = [
        EB3, 0, 0, G3, 0, BB3, 0, 0, // Eb7
        EB3, 0, 0, 0, D3, 0, EB3, 0, // Eb7 walk
        F2, 0, 0, AB2, 0, C3, 0, EB3, // Fm7
        F2, 0, F3, 0, C3, 0, AB2, 0, // Fm7 resolve
    ];

    // Counter melody
    let melody = [
        BB4, 0, G4, 0, EB4, 0, 0, 0, // Eb7 descending
        0, 0, D4, 0, EB4, 0, G4, 0, // Rising
        AB4, 0, 0, 0, C5, 0, AB4, 0, // Fm7 phrase
        F4, 0, 0, 0, 0, 0, 0, 0, // Resolve
    ];

    for row in 0..32 {
        // Ch1: Kick - similar pocket
        if row == 0 || row == 6 || row == 10 || row == 16 || row == 22 || row == 26 {
            write_note(&mut data, C4, KICK_F);
        } else {
            write_empty(&mut data);
        }

        // Ch2: Snare with fill at end
        if row == 8 || row == 24 {
            write_note(&mut data, C4, SNARE_F);
        } else if row == 28 || row == 29 || row == 30 || row == 31 {
            write_note_vol(&mut data, C4, SNARE_F, 0x30); // Fill
        } else if row == 4 || row == 12 || row == 20 {
            write_note_vol(&mut data, C4, SNARE_F, 0x15);
        } else {
            write_empty(&mut data);
        }

        // Ch3: Hi-hat
        if row % 4 == 0 {
            write_note(&mut data, C4, HIHAT_F);
        } else if row % 2 == 0 {
            write_note_vol(&mut data, C4, HIHAT_F, 0x20);
        } else {
            write_empty(&mut data);
        }

        // Ch4: Bass
        let bass = bass_notes[row as usize];
        if bass != 0 {
            write_note(&mut data, bass, BASS_F);
        } else {
            write_empty(&mut data);
        }

        // Ch5: Lead melody
        let mel = melody[row as usize];
        if mel != 0 {
            write_note(&mut data, mel, LEAD_J);
        } else {
            write_empty(&mut data);
        }

        // Ch6: EPiano chords
        if row == 0 {
            write_note(&mut data, G4, EPIANO); // Eb7
        } else if row == 16 {
            write_note(&mut data, AB4, EPIANO); // Fm7
        } else {
            write_empty(&mut data);
        }

        // Ch7: EP bass
        if row == 0 || row == 8 {
            write_note(&mut data, EB3, EPIANO);
        } else if row == 16 || row == 24 {
            write_note(&mut data, F3, EPIANO);
        } else {
            write_empty(&mut data);
        }

        // Ch8: Empty for variety
        write_empty(&mut data);
    }

    data
}

/// Funk Pattern 3: Bridge - Building intensity, chromatic bass
fn generate_funk_pattern_bridge() -> Vec<u8> {
    let mut data = Vec::new();

    // Chromatic walking bass
    let bass_notes = [
        F2, 0, 0, 0, G2, 0, 0, 0, // F -> G
        AB2, 0, 0, 0, BB2, 0, 0, 0, // Ab -> Bb
        C3, 0, 0, 0, D3, 0, 0, 0, // C -> D
        EB3, 0, D3, 0, C3, 0, BB2, 0, // Descending run
    ];

    // Jazz runs
    let melody = [
        C5, EB5, F5, 0, EB5, C5, BB4, 0, // Fast run
        AB4, 0, G4, 0, F4, 0, 0, 0, // Descending
        C5, 0, D4, 0, EB4, 0, F4, 0, // Building
        G4, AB4, BB4, C5, EB5, 0, F5, 0, // Climax run
    ];

    for row in 0..32 {
        // Ch1: Kick - driving
        if row % 4 == 0 {
            write_note(&mut data, C4, KICK_F);
        } else {
            write_empty(&mut data);
        }

        // Ch2: Snare - building intensity
        if row == 8 || row == 24 {
            write_note(&mut data, C4, SNARE_F);
        } else if row >= 28 {
            write_note_vol(&mut data, C4, SNARE_F, 0x35); // Build
        } else {
            write_empty(&mut data);
        }

        // Ch3: Hi-hat - 16ths
        if row % 2 == 0 {
            write_note(&mut data, C4, HIHAT_F);
        } else {
            write_note_vol(&mut data, C4, HIHAT_F, 0x18);
        }

        // Ch4: Bass
        let bass = bass_notes[row as usize];
        if bass != 0 {
            write_note(&mut data, bass, BASS_F);
        } else {
            write_empty(&mut data);
        }

        // Ch5: Lead - jazz runs
        let mel = melody[row as usize];
        if mel != 0 {
            write_note(&mut data, mel, LEAD_J);
        } else {
            write_empty(&mut data);
        }

        // Ch6: EPiano arpeggios
        if row % 4 == 0 {
            let notes = [C4, EB4, G4, BB4, C5, EB5, G4, C5];
            write_note(&mut data, notes[(row / 4) as usize], EPIANO);
        } else {
            write_empty(&mut data);
        }

        // Ch7-8: Building harmony
        if row >= 24 {
            if row % 2 == 0 {
                write_note(&mut data, F4, EPIANO);
            } else {
                write_empty(&mut data);
            }
            write_empty(&mut data);
        } else {
            write_empty(&mut data);
            write_empty(&mut data);
        }
    }

    data
}

/// Funk Pattern 4: Solo - EP takes the lead
fn generate_funk_pattern_solo() -> Vec<u8> {
    let mut data = Vec::new();

    // Vamp bass on Fm7
    let bass_notes = [
        F2, 0, 0, AB2, 0, C3, 0, 0, F2, 0, 0, 0, C3, 0, AB2, 0, F2, 0, 0, AB2, 0, C3, 0, EB3, F3, 0,
        EB3, 0, C3, 0, AB2, 0,
    ];

    // EP "solo" - improvisatory feel
    let ep_solo = [
        C5, 0, AB4, 0, F4, 0, 0, 0, AB4, C5, EB5, 0, C5, 0, 0, 0, F5, 0, EB5, 0, C5, 0, AB4, 0, BB4,
        0, AB4, 0, F4, 0, 0, 0,
    ];

    for row in 0..32 {
        // Ch1: Kick
        if row == 0 || row == 6 || row == 10 || row == 16 || row == 22 || row == 26 {
            write_note(&mut data, C4, KICK_F);
        } else {
            write_empty(&mut data);
        }

        // Ch2: Snare
        if row == 8 || row == 24 {
            write_note(&mut data, C4, SNARE_F);
        } else if row == 4 || row == 12 || row == 20 || row == 28 {
            write_note_vol(&mut data, C4, SNARE_F, 0x15);
        } else {
            write_empty(&mut data);
        }

        // Ch3: Hi-hat
        if row % 4 == 0 {
            write_note(&mut data, C4, HIHAT_F);
        } else if row % 2 == 0 {
            write_note_vol(&mut data, C4, HIHAT_F, 0x20);
        } else {
            write_empty(&mut data);
        }

        // Ch4: Bass
        let bass = bass_notes[row as usize];
        if bass != 0 {
            write_note(&mut data, bass, BASS_F);
        } else {
            write_empty(&mut data);
        }

        // Ch5: Lead - quiet, supportive
        write_empty(&mut data);

        // Ch6: EPiano solo!
        let solo = ep_solo[row as usize];
        if solo != 0 {
            write_note(&mut data, solo, EPIANO);
        } else {
            write_empty(&mut data);
        }

        // Ch7: Chord hits
        if row == 0 || row == 16 {
            write_note(&mut data, AB3, EPIANO);
        } else {
            write_empty(&mut data);
        }

        // Ch8: Empty
        write_empty(&mut data);
    }

    data
}

/// Funk Pattern 5: Outro - Fading groove
fn generate_funk_pattern_outro() -> Vec<u8> {
    let mut data = Vec::new();

    // Descending bass
    let bass_notes = [
        F3, 0, 0, 0, EB3, 0, 0, 0, C3, 0, 0, 0, BB2, 0, 0, 0, AB2, 0, 0, 0, G2, 0, 0, 0, F2, 0, 0,
        0, 0, 0, 0, 0,
    ];

    // Final melody phrase
    let melody = [
        C5, 0, AB4, 0, F4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, AB4, 0, F4, 0, C4, 0, 0, 0, F4, 0, 0, 0,
        0, 0, 0, 0,
    ];

    for row in 0..32 {
        // Ch1: Kick - sparse
        if row == 0 || row == 16 {
            write_note(&mut data, C4, KICK_F);
        } else {
            write_empty(&mut data);
        }

        // Ch2: Snare - just ghosts
        if row == 8 || row == 24 {
            write_note_vol(&mut data, C4, SNARE_F, 0x25);
        } else {
            write_empty(&mut data);
        }

        // Ch3: Hi-hat - quarter notes fading
        if row % 8 == 0 && row < 24 {
            write_note_vol(&mut data, C4, HIHAT_F, (0x30 - row) as u8);
        } else {
            write_empty(&mut data);
        }

        // Ch4: Bass
        let bass = bass_notes[row as usize];
        if bass != 0 {
            write_note(&mut data, bass, BASS_F);
        } else {
            write_empty(&mut data);
        }

        // Ch5: Final melody
        let mel = melody[row as usize];
        if mel != 0 {
            write_note(&mut data, mel, LEAD_J);
        } else {
            write_empty(&mut data);
        }

        // Ch6: Outro chords
        if row == 0 {
            write_note(&mut data, C4, EPIANO);
        } else if row == 24 {
            write_note(&mut data, F3, EPIANO); // Final chord
        } else {
            write_empty(&mut data);
        }

        // Ch7-8: Empty
        write_empty(&mut data);
        write_empty(&mut data);
    }

    data
}

// ============================================================================
// XM File Generation - Eurobeat "Nether Fire"
// ============================================================================

fn generate_eurobeat_xm() -> Vec<u8> {
    let mut xm = Vec::new();

    // XM Header
    xm.extend_from_slice(b"Extended Module: ");

    // Module name
    let name = b"Nether Fire";
    xm.extend_from_slice(name);
    xm.extend(std::iter::repeat(0u8).take(20 - name.len()));

    xm.push(0x1A);

    // Tracker name
    let tracker = b"gen-tracker-demo";
    xm.extend_from_slice(tracker);
    xm.extend(std::iter::repeat(0u8).take(20 - tracker.len()));

    // Version
    xm.extend_from_slice(&0x0104u16.to_le_bytes());

    // Header size (276 = 4 bytes header_size + 16 bytes of header fields + 256 byte order table)
    // Per XM spec, header_size is measured from the position of this field itself
    xm.extend_from_slice(&276u32.to_le_bytes());

    // Song length (15 orders)
    xm.extend_from_slice(&15u16.to_le_bytes());

    // Restart position
    xm.extend_from_slice(&3u16.to_le_bytes());

    // Number of channels (8)
    xm.extend_from_slice(&8u16.to_le_bytes());

    // Number of patterns (8)
    xm.extend_from_slice(&8u16.to_le_bytes());

    // Number of instruments (7)
    xm.extend_from_slice(&7u16.to_le_bytes());

    // Flags
    xm.extend_from_slice(&1u16.to_le_bytes());

    // Default speed
    xm.extend_from_slice(&6u16.to_le_bytes());

    // Default BPM (155 for Eurobeat!)
    xm.extend_from_slice(&155u16.to_le_bytes());

    // Pattern order: Intro -> Verse -> Verse -> Pre-Chorus -> Chorus A -> Chorus B ->
    //                Verse -> Verse -> Pre-Chorus -> Chorus A -> Chorus B -> Breakdown -> Drop -> Chorus A -> Chorus B
    let order = [0u8, 1, 2, 3, 4, 5, 1, 2, 3, 4, 5, 6, 7, 4, 5];
    xm.extend_from_slice(&order);
    xm.extend(std::iter::repeat(0u8).take(256 - order.len()));

    // Generate patterns
    for i in 0..8 {
        let pattern_data = match i {
            0 => generate_euro_pattern_intro(),
            1 => generate_euro_pattern_verse_a(),
            2 => generate_euro_pattern_verse_b(),
            3 => generate_euro_pattern_prechorus(),
            4 => generate_euro_pattern_chorus_a(),
            5 => generate_euro_pattern_chorus_b(),
            6 => generate_euro_pattern_breakdown(),
            7 => generate_euro_pattern_drop(),
            _ => unreachable!(),
        };
        let pattern_size = pattern_data.len() as u16;

        // Debug validation
        eprintln!("Eurobeat Pattern {}: size={} bytes", i, pattern_size);
        if pattern_size < 256 {
            eprintln!("WARNING: Eurobeat Pattern {} too small (expected min 256)", i);
        }

        xm.extend_from_slice(&9u32.to_le_bytes()); // header length (including length field: 4+1+2+2=9)
        xm.push(0);
        xm.extend_from_slice(&32u16.to_le_bytes());
        xm.extend_from_slice(&pattern_size.to_le_bytes());
        xm.extend_from_slice(&pattern_data);
    }

    // Instruments
    let instruments = [
        "kick_euro",
        "snare_euro",
        "hihat_euro",
        "bass_euro",
        "supersaw",
        "brass_euro",
        "pad_euro",
    ];
    for name in &instruments {
        write_instrument(&mut xm, name);
    }

    xm
}

fn generate_eurobeat_xm_embedded(samples: &[Vec<i16>]) -> Vec<u8> {
    let mut xm = Vec::new();

    // XM Header
    xm.extend_from_slice(b"Extended Module: ");

    let name = b"Nether Fire";
    xm.extend_from_slice(name);
    xm.extend(std::iter::repeat(0u8).take(20 - name.len()));

    xm.push(0x1A);

    let tracker = b"gen-tracker-demo";
    xm.extend_from_slice(tracker);
    xm.extend(std::iter::repeat(0u8).take(20 - tracker.len()));

    xm.extend_from_slice(&0x0104u16.to_le_bytes());
    xm.extend_from_slice(&276u32.to_le_bytes());
    xm.extend_from_slice(&15u16.to_le_bytes());
    xm.extend_from_slice(&3u16.to_le_bytes());
    xm.extend_from_slice(&8u16.to_le_bytes());
    xm.extend_from_slice(&8u16.to_le_bytes());
    xm.extend_from_slice(&7u16.to_le_bytes());
    xm.extend_from_slice(&1u16.to_le_bytes());
    xm.extend_from_slice(&6u16.to_le_bytes());
    xm.extend_from_slice(&155u16.to_le_bytes());

    let order = [0u8, 1, 2, 3, 4, 5, 1, 2, 3, 4, 5, 6, 7, 4, 5];
    xm.extend_from_slice(&order);
    xm.extend(std::iter::repeat(0u8).take(256 - order.len()));

    for i in 0..8 {
        let pattern_data = match i {
            0 => generate_euro_pattern_intro(),
            1 => generate_euro_pattern_verse_a(),
            2 => generate_euro_pattern_verse_b(),
            3 => generate_euro_pattern_prechorus(),
            4 => generate_euro_pattern_chorus_a(),
            5 => generate_euro_pattern_chorus_b(),
            6 => generate_euro_pattern_breakdown(),
            7 => generate_euro_pattern_drop(),
            _ => unreachable!(),
        };
        let pattern_size = pattern_data.len() as u16;

        xm.extend_from_slice(&9u32.to_le_bytes());
        xm.push(0);
        xm.extend_from_slice(&32u16.to_le_bytes());
        xm.extend_from_slice(&pattern_size.to_le_bytes());
        xm.extend_from_slice(&pattern_data);
    }

    let instruments = [
        "kick_euro",
        "snare_euro",
        "hihat_euro",
        "bass_euro",
        "supersaw",
        "brass_euro",
        "pad_euro",
    ];
    for (i, name) in instruments.iter().enumerate() {
        write_instrument_with_sample(&mut xm, name, &samples[i]);
    }

    xm
}

// Eurobeat note constants (D minor: D E F G A Bb C)
const D2_E: u8 = 27;
const F2_E: u8 = 30;
const G2_E: u8 = 32; // G for Gm bass
const A2_E: u8 = 34;
const BB2_E: u8 = 35;
const C3_E: u8 = 37;
const D3_E: u8 = 39;
const F3_E: u8 = 42;
const G3_E: u8 = 44; // G for Gm bass
const A3_E: u8 = 46;
const BB3_E: u8 = 47;
const C4_E: u8 = 49;
const CS4_E: u8 = 50; // C# for A major (harmonic minor)
const D4_E: u8 = 51;
const E4_E: u8 = 53;
const F4_E: u8 = 54;
const G4_E: u8 = 56;
const A4_E: u8 = 58;
const BB4_E: u8 = 59;
const C5_E: u8 = 61;
const CS5_E: u8 = 62; // C# for A major (harmonic minor)
const D5_E: u8 = 63;
const E5_E: u8 = 65;
const F5_E: u8 = 66;
const G5_E: u8 = 68;
const A5_E: u8 = 70;
const BB5_E: u8 = 71; // Bb for Gm high register
// Octave 6 for climax hook
const C6_E: u8 = 73;
const CS6_E: u8 = 74; // C# for harmonic minor climax
const D6_E: u8 = 75;
const E6_E: u8 = 77;
const F6_E: u8 = 78;
const G6_E: u8 = 80;
const A6_E: u8 = 82;
// D Major for Picardy third
const FS4_E: u8 = 55; // F# for D major
const FS5_E: u8 = 67; // F# octave 5 for D major variations

// Eurobeat instruments
const KICK_E: u8 = 1;
const SNARE_E: u8 = 2;
const HIHAT_E: u8 = 3;
const BASS_E: u8 = 4;
const SUPERSAW: u8 = 5;
const BRASS: u8 = 6;
const PAD: u8 = 7;

/// Eurobeat Pattern 0: Intro - Hook teaser, building energy
fn generate_euro_pattern_intro() -> Vec<u8> {
    let mut data = Vec::new();

    // Hook teaser melody: tease the hook, then state it fully
    // Rows 0-7: Partial hook (D5-F5-A5)
    // Rows 8-15: Hint (D5-F5-G5)
    // Rows 16-23: FULL HOOK (D5-F5-A5-A5-G5-F5-E5-D5)
    // Rows 24-31: Resolution with harmonic minor (F5-E5-D5-C#5-D5)

    for row in 0..32 {
        // Ch1: Kick - sparse at first, builds to four-on-floor
        if row < 16 {
            if row == 0 {
                write_note(&mut data, C4_E, KICK_E);
            } else {
                write_empty(&mut data);
            }
        } else if row % 4 == 0 {
            write_note(&mut data, C4_E, KICK_E);
        } else {
            write_empty(&mut data);
        }

        // Ch2: Snare - enters at row 24 on backbeat
        if row >= 24 && row % 8 == 4 {
            write_note(&mut data, C4_E, SNARE_E);
        } else {
            write_empty(&mut data);
        }

        // Ch3: Hi-hat - builds from sparse to 8ths
        if row < 8 {
            if row == 0 {
                write_note(&mut data, C4_E, HIHAT_E);
            } else {
                write_empty(&mut data);
            }
        } else if row < 16 {
            if row % 4 == 0 {
                write_note(&mut data, C4_E, HIHAT_E);
            } else {
                write_empty(&mut data);
            }
        } else if row % 2 == 0 {
            write_note(&mut data, C4_E, HIHAT_E);
        } else {
            write_empty(&mut data);
        }

        // Ch4: Bass - Dm pedal with octave bounce starting row 16
        if row == 0 {
            write_note(&mut data, D2_E, BASS_E);
        } else if row >= 16 && row % 2 == 0 {
            let note = if (row / 2) % 2 == 0 { D2_E } else { D3_E };
            write_note(&mut data, note, BASS_E);
        } else {
            write_empty(&mut data);
        }

        // Ch5: Supersaw - HOOK TEASER
        // Rows 0-1: D5-F5-A5 (partial hook)
        // Rows 8-9: D5-F5-G5 (hint/variation)
        // Rows 16-23: FULL HOOK D5-F5-A5-A5-G5-F5-E5-D5
        // Rows 24-27: F5-E5-D5-C#5-D5 (harmonic minor resolution)
        match row {
            // Partial hook (bars 1-2)
            0 => write_note(&mut data, D5_E, SUPERSAW),
            2 => write_note(&mut data, F5_E, SUPERSAW),
            4 => write_note(&mut data, A5_E, SUPERSAW),
            // Hint variation (bars 3-4)
            8 => write_note(&mut data, D5_E, SUPERSAW),
            10 => write_note(&mut data, F5_E, SUPERSAW),
            12 => write_note(&mut data, G5_E, SUPERSAW),
            // FULL HOOK STATEMENT (bars 5-6) - the money!
            16 => write_note(&mut data, D5_E, SUPERSAW),
            18 => write_note(&mut data, F5_E, SUPERSAW),
            20 => write_note(&mut data, A5_E, SUPERSAW),
            21 => write_note(&mut data, A5_E, SUPERSAW), // 16th note repeat
            22 => write_note(&mut data, G5_E, SUPERSAW),
            24 => write_note(&mut data, F5_E, SUPERSAW),
            26 => write_note(&mut data, E5_E, SUPERSAW),
            28 => write_note(&mut data, D5_E, SUPERSAW),
            // Harmonic minor tag (bars 7-8)
            30 => write_note(&mut data, CS5_E, SUPERSAW), // C# leading tone
            _ => write_empty(&mut data),
        }

        // Ch6: Brass - answer stabs
        match row {
            6 => write_note(&mut data, A4_E, BRASS),  // Answer to partial hook
            14 => write_note(&mut data, G4_E, BRASS), // Answer to hint
            // Silent during full hook to let it breathe
            30 => write_note(&mut data, A3_E, BRASS), // Dominant stab at end
            _ => write_empty(&mut data),
        }

        // Ch7: Pad - Dm -> Bb -> C progression building to verse
        if row == 0 {
            write_note(&mut data, D3_E, PAD); // Dm
        } else if row == 8 {
            write_note(&mut data, BB3_E, PAD); // Bb (VI)
        } else if row == 16 {
            write_note(&mut data, C4_E, PAD); // C (VII)
        } else if row == 24 {
            write_note(&mut data, A3_E, PAD); // A (V) - dominant
        } else {
            write_empty(&mut data);
        }

        // Ch8: Silent - hook is solo
        write_empty(&mut data);
    }

    data
}

/// Eurobeat Pattern 1: Verse A - Melodic intro with stepwise contour
fn generate_euro_pattern_verse_a() -> Vec<u8> {
    let mut data = Vec::new();

    // NEW progression: Dm -> Gm -> Bb -> A (8 rows each)
    // Adds iv (Gm) for minor color, ends on dominant A for tension
    let bass_pattern: [(u8, u8); 16] = [
        // Dm (rows 0-7)
        (D2_E, D3_E), (D2_E, D3_E), (D2_E, D3_E), (D2_E, D3_E),
        // Gm (rows 8-15) - adds minor color
        (G2_E, G3_E), (G2_E, G3_E), (G2_E, G3_E), (G2_E, G3_E),
        // Bb (rows 16-23) - VI chord
        (BB2_E, BB3_E), (BB2_E, BB3_E), (BB2_E, BB3_E), (BB2_E, BB3_E),
        // A (rows 24-31) - dominant V, creates tension
        (A2_E, A3_E), (A2_E, A3_E), (A2_E, A3_E), (A2_E, A3_E),
    ];

    for row in 0..32 {
        // Ch1: Kick - four on the floor
        if row % 4 == 0 {
            write_note(&mut data, C4_E, KICK_E);
        } else {
            write_empty(&mut data);
        }

        // Ch2: Snare - beats 2 and 4
        if row % 8 == 4 {
            write_note(&mut data, C4_E, SNARE_E);
        } else {
            write_empty(&mut data);
        }

        // Ch3: Hi-hat - 8th notes
        if row % 2 == 0 {
            write_note(&mut data, C4_E, HIHAT_E);
        } else {
            write_empty(&mut data);
        }

        // Ch4: Bass - octave bouncing on 8th notes
        if row % 2 == 0 {
            let idx = (row / 2) as usize;
            let (low, high) = bass_pattern[idx];
            let note = if (row / 2) % 2 == 0 { low } else { high };
            write_note(&mut data, note, BASS_E);
        } else {
            write_empty(&mut data);
        }

        // Ch5: Supersaw - STEPWISE MELODY with real contour
        // Bars 1-2 (Dm): Rising phrase A4-Bb4-C5-D5
        // Bars 3-4 (Gm): Answer phrase D5-C5-Bb4-A4
        // Bars 5-6 (Bb): 16th run building F4-G4-A4-Bb4-A4-G4-F4-E4
        // Bars 7-8 (A): HOOK VARIANT D5-F5-A5-A5-G5-F5-E5-E5 (tension ending)
        match row {
            // Bars 1-2 (Dm): Rising stepwise melody
            0 => write_note(&mut data, A4_E, SUPERSAW),
            2 => write_note(&mut data, BB4_E, SUPERSAW),
            4 => write_note(&mut data, C5_E, SUPERSAW),
            6 => write_note(&mut data, D5_E, SUPERSAW),
            // Bars 3-4 (Gm): Descending answer
            8 => write_note(&mut data, D5_E, SUPERSAW),
            10 => write_note(&mut data, C5_E, SUPERSAW),
            12 => write_note(&mut data, BB4_E, SUPERSAW),
            14 => write_note(&mut data, A4_E, SUPERSAW),
            // Bars 5-6 (Bb): 16th note run - building energy
            16 => write_note(&mut data, F4_E, SUPERSAW),
            17 => write_note(&mut data, G4_E, SUPERSAW),
            18 => write_note(&mut data, A4_E, SUPERSAW),
            19 => write_note(&mut data, BB4_E, SUPERSAW),
            20 => write_note(&mut data, A4_E, SUPERSAW),
            21 => write_note(&mut data, G4_E, SUPERSAW),
            22 => write_note(&mut data, F4_E, SUPERSAW),
            23 => write_note(&mut data, E4_E, SUPERSAW),
            // Bars 7-8 (A): HOOK VARIANT - ends on E5 (2nd degree) for tension
            24 => write_note(&mut data, D5_E, SUPERSAW),
            26 => write_note(&mut data, F5_E, SUPERSAW),
            28 => write_note(&mut data, A5_E, SUPERSAW),
            29 => write_note(&mut data, A5_E, SUPERSAW), // 16th repeat
            30 => write_note(&mut data, G5_E, SUPERSAW),
            31 => write_note(&mut data, E5_E, SUPERSAW), // Tension! Not resolved to D
            _ => write_empty(&mut data),
        }

        // Ch6: Brass - CALL AND RESPONSE with melody
        // Only plays AFTER melody phrases, never simultaneously with hook
        match row {
            // Answer to rising phrase (bar 2)
            7 => write_note(&mut data, D4_E, BRASS), // Stab after phrase peak
            // Answer to descending phrase (bar 4)
            15 => write_note(&mut data, G4_E, BRASS), // Gm chord stab
            // During 16th run - sparse accents
            19 => write_note(&mut data, BB4_E, BRASS), // Peak accent
            23 => write_note(&mut data, E4_E, BRASS),  // Low accent
            // SILENT during hook variant (let it breathe!)
            _ => write_empty(&mut data),
        }

        // Ch7: Pad - follows new chord progression
        match row {
            0 => write_note(&mut data, D3_E, PAD),   // Dm
            8 => write_note(&mut data, G3_E, PAD),   // Gm
            16 => write_note(&mut data, BB3_E, PAD), // Bb
            24 => write_note(&mut data, A3_E, PAD),  // A (dominant)
            _ => write_empty(&mut data),
        }

        // Ch8: Harmony - thirds above melody in key moments
        match row {
            // Harmonize the peak of each phrase
            6 => write_note(&mut data, F5_E, SUPERSAW),  // Third above D5
            14 => write_note(&mut data, C5_E, SUPERSAW), // Third above A4
            // Harmony during hook variant
            28 => write_note(&mut data, C5_E, SUPERSAW), // Third below A5
            _ => write_empty(&mut data),
        }
    }

    data
}

/// Eurobeat Pattern 2: Verse B - Full melody with hook return
fn generate_euro_pattern_verse_b() -> Vec<u8> {
    let mut data = Vec::new();

    // Progression: Dm -> C -> Bb -> C (classic VI-VII movement at end)
    let bass_pattern: [(u8, u8); 16] = [
        // Dm (rows 0-7)
        (D2_E, D3_E), (D2_E, D3_E), (D2_E, D3_E), (D2_E, D3_E),
        // C (rows 8-15) - VII chord
        (C3_E, C4_E), (C3_E, C4_E), (C3_E, C4_E), (C3_E, C4_E),
        // Bb (rows 16-23) - VI chord
        (BB2_E, BB3_E), (BB2_E, BB3_E), (BB2_E, BB3_E), (BB2_E, BB3_E),
        // C (rows 24-31) - VII leading to pre-chorus
        (C3_E, C4_E), (C3_E, C4_E), (C3_E, C4_E), (C3_E, C4_E),
    ];

    for row in 0..32 {
        // Ch1: Kick - four on the floor
        if row % 4 == 0 {
            write_note(&mut data, C4_E, KICK_E);
        } else {
            write_empty(&mut data);
        }

        // Ch2: Snare - beats 2 and 4
        if row % 8 == 4 {
            write_note(&mut data, C4_E, SNARE_E);
        } else {
            write_empty(&mut data);
        }

        // Ch3: Hi-hat - 8th notes
        if row % 2 == 0 {
            write_note(&mut data, C4_E, HIHAT_E);
        } else {
            write_empty(&mut data);
        }

        // Ch4: Bass - octave bouncing on 8th notes
        if row % 2 == 0 {
            let idx = (row / 2) as usize;
            let (low, high) = bass_pattern[idx];
            let note = if (row / 2) % 2 == 0 { low } else { high };
            write_note(&mut data, note, BASS_E);
        } else {
            write_empty(&mut data);
        }

        // Ch5: Supersaw - STEPWISE MELODY (not arpeggios!)
        // Bars 1-2 (Dm): Inverted contour - starts high, dips, rises
        //   D5-C5-Bb4-A4 | Bb4-C5-D5 (arch shape)
        // Bars 3-4 (C): Descending phrase
        //   E5-D5-C5-Bb4-A4 (long descent)
        // Bars 5-6 (Bb): Ascending 16th run building energy
        //   Bb4-C5-D5-E5-F5-E5-D5-C5
        // Bars 7-8 (C): FULL HOOK RETURN - D5-F5-A5-A5-G5-F5-E5-D5
        match row {
            // Bars 1-2 (Dm): Arch shape - high start, dip, rise
            0 => write_note(&mut data, D5_E, SUPERSAW),
            2 => write_note(&mut data, C5_E, SUPERSAW),
            4 => write_note(&mut data, BB4_E, SUPERSAW),
            5 => write_note(&mut data, A4_E, SUPERSAW),  // Quick dip
            6 => write_note(&mut data, BB4_E, SUPERSAW),
            7 => write_note(&mut data, C5_E, SUPERSAW),  // Rising back
            // Bars 3-4 (C): Long descending line
            8 => write_note(&mut data, E5_E, SUPERSAW),
            10 => write_note(&mut data, D5_E, SUPERSAW),
            12 => write_note(&mut data, C5_E, SUPERSAW),
            13 => write_note(&mut data, BB4_E, SUPERSAW),
            14 => write_note(&mut data, A4_E, SUPERSAW),
            // Bars 5-6 (Bb): 16th note ascending run - maximum energy buildup
            16 => write_note(&mut data, BB4_E, SUPERSAW),
            17 => write_note(&mut data, C5_E, SUPERSAW),
            18 => write_note(&mut data, D5_E, SUPERSAW),
            19 => write_note(&mut data, E5_E, SUPERSAW),
            20 => write_note(&mut data, F5_E, SUPERSAW), // Peak!
            21 => write_note(&mut data, E5_E, SUPERSAW),
            22 => write_note(&mut data, D5_E, SUPERSAW),
            23 => write_note(&mut data, C5_E, SUPERSAW),
            // Bars 7-8 (C): FULL HOOK - resolves to D5 (tonic)
            24 => write_note(&mut data, D5_E, SUPERSAW),
            26 => write_note(&mut data, F5_E, SUPERSAW),
            28 => write_note(&mut data, A5_E, SUPERSAW),
            29 => write_note(&mut data, A5_E, SUPERSAW), // 16th repeat (signature!)
            30 => write_note(&mut data, G5_E, SUPERSAW),
            31 => write_note(&mut data, D5_E, SUPERSAW), // RESOLVED to tonic!
            _ => write_empty(&mut data),
        }

        // Ch6: Brass - CALL AND RESPONSE
        // Answers melody phrases, silent during hook
        match row {
            // Answer to arch shape (bars 1-2)
            3 => write_note(&mut data, F4_E, BRASS),  // Dm chord tone
            // Answer to descent (bars 3-4)
            11 => write_note(&mut data, G4_E, BRASS), // C chord tone
            15 => write_note(&mut data, E4_E, BRASS), // Resolution answer
            // Accent the 16th run peak (bars 5-6)
            20 => write_note(&mut data, F4_E, BRASS), // Peak accent
            // SILENT during full hook (bars 7-8) - let it shine!
            _ => write_empty(&mut data),
        }

        // Ch7: Pad - follows chord changes
        match row {
            0 => write_note(&mut data, D3_E, PAD),   // Dm
            8 => write_note(&mut data, C4_E, PAD),   // C
            16 => write_note(&mut data, BB3_E, PAD), // Bb
            24 => write_note(&mut data, C4_E, PAD),  // C
            _ => write_empty(&mut data),
        }

        // Ch8: Harmony - octave below during hook
        match row {
            // Harmony only during the hook for emphasis
            24 => write_note(&mut data, D4_E, SUPERSAW),
            26 => write_note(&mut data, F4_E, SUPERSAW),
            28 => write_note(&mut data, A4_E, SUPERSAW),
            30 => write_note(&mut data, G4_E, SUPERSAW),
            31 => write_note(&mut data, D4_E, SUPERSAW),
            _ => write_empty(&mut data),
        }
    }

    data
}

/// Eurobeat Pattern 3: Pre-Chorus - Build maximum tension into chorus
fn generate_euro_pattern_prechorus() -> Vec<u8> {
    let mut data = Vec::new();

    // Progression: F -> Gm -> A (dominant pedal) -> Bb-C (VI-VII ramp)
    // Bass stays on A for bars 3-4 to create dominant tension

    for row in 0..32 {
        // Ch1: Kick - builds from normal to double-time
        if row < 16 {
            if row % 4 == 0 {
                write_note(&mut data, C4_E, KICK_E);
            } else {
                write_empty(&mut data);
            }
        } else if row % 2 == 0 {
            // Double-time kicks in second half
            write_note(&mut data, C4_E, KICK_E);
        } else {
            write_empty(&mut data);
        }

        // Ch2: Snare - normal backbeat then rolls
        if row < 24 {
            if row % 8 == 4 {
                write_note(&mut data, C4_E, SNARE_E);
            } else {
                write_empty(&mut data);
            }
        } else {
            // Snare roll for final 8 rows
            write_note(&mut data, C4_E, SNARE_E);
        }

        // Ch3: Hi-hat - 8ths then 16ths
        if row < 16 {
            if row % 2 == 0 {
                write_note(&mut data, C4_E, HIHAT_E);
            } else {
                write_empty(&mut data);
            }
        } else {
            // 16th notes for tension
            write_note(&mut data, C4_E, HIHAT_E);
        }

        // Ch4: Bass - F -> Gm -> A pedal -> A pedal (dominant tension!)
        let bass_note = match row {
            0..=7 => if (row / 2) % 2 == 0 { F2_E } else { F3_E },   // F
            8..=15 => if (row / 2) % 2 == 0 { G2_E } else { G3_E },  // Gm
            16..=31 => if (row / 2) % 2 == 0 { A2_E } else { A3_E }, // A PEDAL (tension!)
            _ => A2_E,
        };
        if row % 2 == 0 {
            write_note(&mut data, bass_note, BASS_E);
        } else {
            write_empty(&mut data);
        }

        // Ch5: Supersaw - shaped melody then 16th run explosion
        // Bars 1-2 (F): Plateau on C5, stepwise descent
        // Bars 3-4 (Gm -> A): E5 PEDAL POINT - dominant tension!
        // Bars 5-6 (A): Peak descent from A5
        // Bars 7-8 (Bb -> C): 16th ASCENDING RUN into chorus
        match row {
            // Bars 1-2 (F): Plateau then descent
            0 => write_note(&mut data, A4_E, SUPERSAW),
            2 => write_note(&mut data, C5_E, SUPERSAW),
            4 => write_note(&mut data, C5_E, SUPERSAW), // Plateau
            6 => write_note(&mut data, BB4_E, SUPERSAW),
            // Bars 3-4 (Gm -> A): E5 pedal point - TENSION!
            8 => write_note(&mut data, E5_E, SUPERSAW),
            10 => write_note(&mut data, E5_E, SUPERSAW),
            12 => write_note(&mut data, E5_E, SUPERSAW),
            13 => write_note(&mut data, F5_E, SUPERSAW), // Brief rise
            14 => write_note(&mut data, G5_E, SUPERSAW), // Building...
            // Bars 5-6 (A): Peak and descent
            16 => write_note(&mut data, A5_E, SUPERSAW), // PEAK!
            18 => write_note(&mut data, G5_E, SUPERSAW),
            20 => write_note(&mut data, F5_E, SUPERSAW),
            22 => write_note(&mut data, D5_E, SUPERSAW),
            // Bars 7-8: 16th ASCENDING RUN - maximum energy into chorus!
            24 => write_note(&mut data, F4_E, SUPERSAW),
            25 => write_note(&mut data, G4_E, SUPERSAW),
            26 => write_note(&mut data, A4_E, SUPERSAW),
            27 => write_note(&mut data, BB4_E, SUPERSAW),
            28 => write_note(&mut data, C5_E, SUPERSAW),
            29 => write_note(&mut data, D5_E, SUPERSAW),
            30 => write_note(&mut data, E5_E, SUPERSAW),
            31 => write_note(&mut data, F5_E, SUPERSAW), // Launches into chorus!
            _ => write_empty(&mut data),
        }

        // Ch6: Brass - building sustained pads then stabs
        match row {
            0 => write_note(&mut data, F4_E, BRASS),   // F chord
            8 => write_note(&mut data, G4_E, BRASS),   // Gm
            12 => write_note(&mut data, A4_E, BRASS),  // A major stab
            14 => write_note(&mut data, CS5_E, BRASS), // C# for A major (harmonic minor!)
            16 => write_note(&mut data, A4_E, BRASS),  // A sustained
            // Brass doubles the ascending run octave lower
            24 => write_note(&mut data, F3_E, BRASS),
            26 => write_note(&mut data, A3_E, BRASS),
            28 => write_note(&mut data, C4_E, BRASS),
            30 => write_note(&mut data, E4_E, BRASS),
            _ => write_empty(&mut data),
        }

        // Ch7: Pad - builds F -> Gm -> A -> C (dominant to chorus)
        match row {
            0 => write_note(&mut data, F3_E, PAD),
            8 => write_note(&mut data, G3_E, PAD),
            16 => write_note(&mut data, A3_E, PAD),
            24 => write_note(&mut data, C4_E, PAD), // VII chord into chorus
            _ => write_empty(&mut data),
        }

        // Ch8: Harmony - octave up during ascending run
        match row {
            28 => write_note(&mut data, C6_E, SUPERSAW),
            29 => write_note(&mut data, D6_E, SUPERSAW),
            30 => write_note(&mut data, E6_E, SUPERSAW),
            31 => write_note(&mut data, F6_E, SUPERSAW),
            _ => write_empty(&mut data),
        }
    }

    data
}

/// Eurobeat Pattern 4: Chorus A - THE HOOK with Bb5 extension!
fn generate_euro_pattern_chorus_a() -> Vec<u8> {
    let mut data = Vec::new();

    // EUROBEAT CADENCE: Dm -> Bb -> C -> Dm (i -> VI -> VII -> i)
    // This is THE signature eurobeat progression
    let bass_pattern: [(u8, u8); 16] = [
        // Dm (rows 0-7)
        (D2_E, D3_E), (D2_E, D3_E), (D2_E, D3_E), (D2_E, D3_E),
        // Bb (rows 8-15) - VI chord
        (BB2_E, BB3_E), (BB2_E, BB3_E), (BB2_E, BB3_E), (BB2_E, BB3_E),
        // C (rows 16-23) - VII chord
        (C3_E, C4_E), (C3_E, C4_E), (C3_E, C4_E), (C3_E, C4_E),
        // Dm (rows 24-31) - resolution to i
        (D2_E, D3_E), (D2_E, D3_E), (D2_E, D3_E), (D2_E, D3_E),
    ];

    for row in 0..32 {
        // Ch1: Kick - four on the floor
        if row % 4 == 0 {
            write_note(&mut data, C4_E, KICK_E);
        } else {
            write_empty(&mut data);
        }

        // Ch2: Snare - beats 2 and 4
        if row % 8 == 4 {
            write_note(&mut data, C4_E, SNARE_E);
        } else {
            write_empty(&mut data);
        }

        // Ch3: Hi-hat - 16ths for chorus energy
        write_note(&mut data, C4_E, HIHAT_E);

        // Ch4: Bass - octave bouncing on 8th notes
        if row % 2 == 0 {
            let idx = (row / 2) as usize;
            let (low, high) = bass_pattern[idx];
            let note = if (row / 2) % 2 == 0 { low } else { high };
            write_note(&mut data, note, BASS_E);
        } else {
            write_empty(&mut data);
        }

        // Ch5: THE HOOK with Bb5 EXTENSION!
        // Bars 1-2 (Dm): D5-F5-A5-A5-Bb5-A5-G5-F5 (hook with extension to Bb5!)
        // Bars 3-4 (Bb): G5-F5-E5-D5 (resolution) then pause
        // Bars 5-6 (C): REPEAT hook D5-F5-A5-A5-Bb5-A5-G5-F5
        // Bars 7-8 (Dm): G5-F5-E5-D5 (final resolution to tonic)
        match row {
            // Bars 1-2 (Dm): HOOK with Bb5 extension!
            0 => write_note(&mut data, D5_E, SUPERSAW),
            2 => write_note(&mut data, F5_E, SUPERSAW),
            4 => write_note(&mut data, A5_E, SUPERSAW),
            5 => write_note(&mut data, A5_E, SUPERSAW),   // 16th repeat (signature!)
            6 => write_note(&mut data, BB5_E, SUPERSAW),  // Bb5 EXTENSION - goes UP!
            7 => write_note(&mut data, A5_E, SUPERSAW),
            // Bars 3-4 (Bb): Descending resolution
            8 => write_note(&mut data, G5_E, SUPERSAW),
            10 => write_note(&mut data, F5_E, SUPERSAW),
            12 => write_note(&mut data, E5_E, SUPERSAW),
            14 => write_note(&mut data, D5_E, SUPERSAW),
            // Bars 5-6 (C): HOOK REPEAT
            16 => write_note(&mut data, D5_E, SUPERSAW),
            18 => write_note(&mut data, F5_E, SUPERSAW),
            20 => write_note(&mut data, A5_E, SUPERSAW),
            21 => write_note(&mut data, A5_E, SUPERSAW),  // 16th repeat
            22 => write_note(&mut data, BB5_E, SUPERSAW), // Bb5 extension
            23 => write_note(&mut data, A5_E, SUPERSAW),
            // Bars 7-8 (Dm): Final resolution
            24 => write_note(&mut data, G5_E, SUPERSAW),
            26 => write_note(&mut data, F5_E, SUPERSAW),
            28 => write_note(&mut data, E5_E, SUPERSAW),
            30 => write_note(&mut data, D5_E, SUPERSAW),  // RESOLVED to tonic!
            _ => write_empty(&mut data),
        }

        // Ch6: Brass - stabs on beats 2 and 4, silent during hook peaks
        match row {
            // Stabs during resolution phrases
            9 => write_note(&mut data, BB4_E, BRASS),  // Bb chord tone
            11 => write_note(&mut data, D5_E, BRASS),  // High answer
            13 => write_note(&mut data, F4_E, BRASS),  // Low stab
            // During second hook - counter melody
            17 => write_note(&mut data, C5_E, BRASS),  // C chord tone
            19 => write_note(&mut data, E5_E, BRASS),
            // Resolution emphasis
            25 => write_note(&mut data, D4_E, BRASS),  // Dm chord
            27 => write_note(&mut data, F4_E, BRASS),
            29 => write_note(&mut data, A4_E, BRASS),
            31 => write_note(&mut data, D5_E, BRASS),  // Final hit
            _ => write_empty(&mut data),
        }

        // Ch7: Pad - VI-VII-i progression
        match row {
            0 => write_note(&mut data, D3_E, PAD),   // Dm
            8 => write_note(&mut data, BB3_E, PAD),  // Bb (VI)
            16 => write_note(&mut data, C4_E, PAD),  // C (VII)
            24 => write_note(&mut data, D4_E, PAD),  // Dm (i) resolution
            _ => write_empty(&mut data),
        }

        // Ch8: Harmony - octave below during hook statements
        match row {
            // Harmony during hook bars 1-2
            0 => write_note(&mut data, D4_E, SUPERSAW),
            2 => write_note(&mut data, F4_E, SUPERSAW),
            4 => write_note(&mut data, A4_E, SUPERSAW),
            6 => write_note(&mut data, BB4_E, SUPERSAW),
            // Harmony during hook bars 5-6
            16 => write_note(&mut data, D4_E, SUPERSAW),
            18 => write_note(&mut data, F4_E, SUPERSAW),
            20 => write_note(&mut data, A4_E, SUPERSAW),
            22 => write_note(&mut data, BB4_E, SUPERSAW),
            // Final resolution harmony
            30 => write_note(&mut data, D4_E, SUPERSAW),
            _ => write_empty(&mut data),
        }
    }

    data
}

/// Eurobeat Pattern 5: Chorus B - Octave-leap climax and triumphant resolution!
fn generate_euro_pattern_chorus_b() -> Vec<u8> {
    let mut data = Vec::new();

    // Progression: Dm -> Bb -> Gm -> A -> Dm with CLIMAX
    // Ends on Dm (or D major for Picardy third variation)
    let bass_pattern: [(u8, u8); 16] = [
        // Dm (rows 0-7)
        (D2_E, D3_E), (D2_E, D3_E), (D2_E, D3_E), (D2_E, D3_E),
        // Bb (rows 8-15) - VI
        (BB2_E, BB3_E), (BB2_E, BB3_E), (BB2_E, BB3_E), (BB2_E, BB3_E),
        // Gm -> A (rows 16-23) - building to climax
        (G2_E, G3_E), (G2_E, G3_E), (A2_E, A3_E), (A2_E, A3_E),
        // Dm (rows 24-31) - triumphant resolution
        (D2_E, D3_E), (D2_E, D3_E), (D2_E, D3_E), (D2_E, D3_E),
    ];

    for row in 0..32 {
        // Ch1: Kick - four on the floor
        if row % 4 == 0 {
            write_note(&mut data, C4_E, KICK_E);
        } else {
            write_empty(&mut data);
        }

        // Ch2: Snare - beats 2 and 4
        if row % 8 == 4 {
            write_note(&mut data, C4_E, SNARE_E);
        } else {
            write_empty(&mut data);
        }

        // Ch3: Hi-hat - 16ths
        write_note(&mut data, C4_E, HIHAT_E);

        // Ch4: Bass - octave bouncing on 8th notes
        if row % 2 == 0 {
            let idx = (row / 2) as usize;
            let (low, high) = bass_pattern[idx];
            let note = if (row / 2) % 2 == 0 { low } else { high };
            write_note(&mut data, note, BASS_E);
        } else {
            write_empty(&mut data);
        }

        // Ch5: OCTAVE-LEAP HOOK VARIANT with CLIMAX at D6!
        // Bars 1-2 (Dm): F5-A5-D6-D6-C6-Bb5-A5-G5 (JUMPED to D6!)
        // Bars 3-4 (Bb): Descending resolution A5-G5-F5-E5
        // Bars 5-6 (Gm->A): Building G5-A5-Bb5-C6-C#6-D6 (harmonic minor!)
        // Bars 7-8 (Dm): D6-F6-A6-A6-G6-F6-E6-D6 (HOOK AT HIGHEST OCTAVE!)
        match row {
            // Bars 1-2 (Dm): Octave-leap variant - JUMPED UP to D6!
            0 => write_note(&mut data, F5_E, SUPERSAW),
            2 => write_note(&mut data, A5_E, SUPERSAW),
            4 => write_note(&mut data, D6_E, SUPERSAW),  // OCTAVE LEAP!
            5 => write_note(&mut data, D6_E, SUPERSAW),  // 16th repeat
            6 => write_note(&mut data, C6_E, SUPERSAW),
            7 => write_note(&mut data, BB5_E, SUPERSAW),
            // Bars 3-4 (Bb): Descending
            8 => write_note(&mut data, A5_E, SUPERSAW),
            10 => write_note(&mut data, G5_E, SUPERSAW),
            12 => write_note(&mut data, F5_E, SUPERSAW),
            14 => write_note(&mut data, E5_E, SUPERSAW),
            // Bars 5-6 (Gm -> A): Ascending to CLIMAX with C#6 (harmonic minor!)
            16 => write_note(&mut data, G5_E, SUPERSAW),
            17 => write_note(&mut data, A5_E, SUPERSAW),
            18 => write_note(&mut data, BB5_E, SUPERSAW),
            19 => write_note(&mut data, C6_E, SUPERSAW),
            20 => write_note(&mut data, CS6_E, SUPERSAW), // C# harmonic minor!
            22 => write_note(&mut data, D6_E, SUPERSAW),  // Peak before final hook
            // Bars 7-8 (Dm): THE ULTIMATE HOOK - HIGHEST OCTAVE!
            24 => write_note(&mut data, D6_E, SUPERSAW),
            26 => write_note(&mut data, F6_E, SUPERSAW),
            28 => write_note(&mut data, A6_E, SUPERSAW),  // HIGHEST NOTE!
            29 => write_note(&mut data, A6_E, SUPERSAW),  // 16th repeat
            30 => write_note(&mut data, G6_E, SUPERSAW),
            31 => write_note(&mut data, D6_E, SUPERSAW),  // Triumphant resolution!
            _ => write_empty(&mut data),
        }

        // Ch6: Brass - sustained pads building to climax, then big hit
        match row {
            0 => write_note(&mut data, D4_E, BRASS),   // Dm foundation
            8 => write_note(&mut data, BB3_E, BRASS),  // Bb
            16 => write_note(&mut data, G4_E, BRASS),  // Gm
            20 => write_note(&mut data, A4_E, BRASS),  // A - building
            21 => write_note(&mut data, CS5_E, BRASS), // C# - harmonic minor!
            // Big brass hit during final hook
            24 => write_note(&mut data, D4_E, BRASS),
            26 => write_note(&mut data, F4_E, BRASS),
            28 => write_note(&mut data, A4_E, BRASS),
            30 => write_note(&mut data, D5_E, BRASS),  // Final triumphant hit
            _ => write_empty(&mut data),
        }

        // Ch7: Pad - building progression
        match row {
            0 => write_note(&mut data, D3_E, PAD),   // Dm
            8 => write_note(&mut data, BB3_E, PAD),  // Bb
            16 => write_note(&mut data, G3_E, PAD),  // Gm
            20 => write_note(&mut data, A3_E, PAD),  // A
            24 => write_note(&mut data, D4_E, PAD),  // Dm triumphant
            _ => write_empty(&mut data),
        }

        // Ch8: Harmony - octave below during climax hook
        match row {
            // Harmony during final hook (bars 7-8)
            24 => write_note(&mut data, D5_E, SUPERSAW),
            26 => write_note(&mut data, F5_E, SUPERSAW),
            28 => write_note(&mut data, A5_E, SUPERSAW),
            30 => write_note(&mut data, G5_E, SUPERSAW),
            31 => write_note(&mut data, D5_E, SUPERSAW),
            _ => write_empty(&mut data),
        }
    }

    data
}

/// Eurobeat Pattern 6: Breakdown - Atmospheric tension
fn generate_euro_pattern_breakdown() -> Vec<u8> {
    let mut data = Vec::new();

    for row in 0..32 {
        // Ch1: Kick - very sparse
        if row == 0 || row == 16 {
            write_note(&mut data, C4_E, KICK_E);
        } else {
            write_empty(&mut data);
        }

        // Ch2: Snare - silent
        write_empty(&mut data);

        // Ch3: Hi-hat - sparse
        if row % 8 == 0 {
            write_note_vol(&mut data, C4_E, HIHAT_E, 0x20);
        } else {
            write_empty(&mut data);
        }

        // Ch4: Bass - Dm sustained
        if row == 0 {
            write_note(&mut data, D2_E, BASS_E);
        } else {
            write_empty(&mut data);
        }

        // Ch5: Atmospheric lead
        if row == 0 {
            write_note_vol(&mut data, D5_E, SUPERSAW, 0x25);
        } else if row == 16 {
            write_note_vol(&mut data, F5_E, SUPERSAW, 0x25);
        } else {
            write_empty(&mut data);
        }

        // Ch6: Silent
        write_empty(&mut data);

        // Ch7: Ambient pad
        if row == 0 {
            write_note(&mut data, D3_E, PAD);
        } else if row == 8 {
            write_note(&mut data, F3_E, PAD);
        } else if row == 16 {
            write_note(&mut data, A3_E, PAD);
        } else if row == 24 {
            write_note(&mut data, D4_E, PAD);
        } else {
            write_empty(&mut data);
        }

        // Ch8: Silent
        write_empty(&mut data);
    }

    data
}

/// Eurobeat Pattern 7: Drop - MAXIMUM ENERGY with 16th hook fragments!
fn generate_euro_pattern_drop() -> Vec<u8> {
    let mut data = Vec::new();

    // i-VII-VI-V descent: Dm -> C -> Bb -> A (classic eurobeat energy!)
    let bass_pattern: [(u8, u8); 16] = [
        // Dm (rows 0-7)
        (D2_E, D3_E), (D2_E, D3_E), (D2_E, D3_E), (D2_E, D3_E),
        // C (rows 8-15) - VII
        (C3_E, C4_E), (C3_E, C4_E), (C3_E, C4_E), (C3_E, C4_E),
        // Bb (rows 16-23) - VI
        (BB2_E, BB3_E), (BB2_E, BB3_E), (BB2_E, BB3_E), (BB2_E, BB3_E),
        // A (rows 24-31) - V dominant
        (A2_E, A3_E), (A2_E, A3_E), (A2_E, A3_E), (A2_E, A3_E),
    ];

    // 16TH NOTE HOOK FRAGMENTS - rapid fire hook variations!
    // Each 8-bar section uses hook rhythm (D-F-A-A-G-F-E-D) transposed
    let melody: [u8; 32] = [
        // Dm (rows 0-7): Hook at D5
        D5_E, D5_E, F5_E, F5_E, A5_E, A5_E, A5_E, G5_E,
        // C (rows 8-15): Hook rhythm on C chord
        C5_E, C5_E, E5_E, E5_E, G5_E, G5_E, G5_E, E5_E,
        // Bb (rows 16-23): Hook rhythm on Bb chord
        BB4_E, BB4_E, D5_E, D5_E, F5_E, F5_E, F5_E, D5_E,
        // A (rows 24-31): Hook rhythm with C# harmonic minor!
        A4_E, A4_E, CS5_E, CS5_E, E5_E, E5_E, E5_E, A5_E,
    ];

    for row in 0..32 {
        // Ch1: Kick - DOUBLE TIME (every other 16th)
        if row % 2 == 0 {
            write_note(&mut data, C4_E, KICK_E);
        } else {
            write_empty(&mut data);
        }

        // Ch2: Snare - on 2 and 4 with ghost notes
        if row % 8 == 4 {
            write_note(&mut data, C4_E, SNARE_E);
        } else if row % 4 == 2 {
            write_note_vol(&mut data, C4_E, SNARE_E, 0x30); // Ghost hit
        } else {
            write_empty(&mut data);
        }

        // Ch3: Hi-hat - FULL 16ths for maximum energy
        write_note(&mut data, C4_E, HIHAT_E);

        // Ch4: Bass - octave bouncing on 8th notes
        if row % 2 == 0 {
            let idx = (row / 2) as usize;
            let (low, high) = bass_pattern[idx];
            let note = if (row / 2) % 2 == 0 { low } else { high };
            write_note(&mut data, note, BASS_E);
        } else {
            write_empty(&mut data);
        }

        // Ch5: SUPERSAW - 16th note hook fragments, every single row!
        write_note(&mut data, melody[row as usize], SUPERSAW);

        // Ch6: Brass - sustained power chords, big stabs on downbeats
        match row {
            0 => write_note(&mut data, D4_E, BRASS),   // Dm power chord
            4 => write_note(&mut data, F4_E, BRASS),
            8 => write_note(&mut data, C4_E, BRASS),   // C power chord
            12 => write_note(&mut data, E4_E, BRASS),
            16 => write_note(&mut data, BB3_E, BRASS), // Bb power chord
            20 => write_note(&mut data, D4_E, BRASS),
            24 => write_note(&mut data, A3_E, BRASS),  // A power chord
            26 => write_note(&mut data, CS4_E, BRASS), // C# for harmonic minor!
            28 => write_note(&mut data, E4_E, BRASS),
            30 => write_note(&mut data, A4_E, BRASS),  // Launch note
            _ => write_empty(&mut data),
        }

        // Ch7: Pad - full chords following progression
        match row {
            0 => write_note(&mut data, D4_E, PAD),   // Dm
            8 => write_note(&mut data, C4_E, PAD),   // C
            16 => write_note(&mut data, BB3_E, PAD), // Bb
            24 => write_note(&mut data, A3_E, PAD),  // A
            _ => write_empty(&mut data),
        }

        // Ch8: Harmony - octave up for maximum thickness, every 16th!
        write_note(&mut data, melody[row as usize].saturating_add(12).min(A6_E), SUPERSAW);
    }

    data
}

/// Calculate finetune and relative_note for a sample at given sample rate.
///
/// XM expects samples tuned for 8363 Hz at C-4. This function calculates
/// the pitch correction needed to play a sample at the correct pitch.
///
/// # Arguments
/// * `sample_rate` - The sample rate of the audio (e.g., 22050)
///
/// # Returns
/// (finetune, relative_note) where:
/// - finetune: -128 to 127 (1/128th semitone precision)
/// - relative_note: semitone offset from C-4
///
/// # Formula
/// ```text
/// semitones = 12 × log₂(sample_rate / 8363)
/// relative_note = floor(semitones)
/// finetune = round((semitones - relative_note) × 128)
/// ```
///
/// # Examples
/// - 22050 Hz → (101, 16) — plays 22050Hz sample at correct pitch
/// - 44100 Hz → (101, 28) — one octave higher than 22050
/// - 8363 Hz → (0, 0) — native XM rate, no correction needed
fn calculate_pitch_correction(sample_rate: u32) -> (i8, i8) {
    const BASE_FREQ: f64 = 8363.0; // C-4 reference frequency

    // Calculate semitone offset: 12 × log₂(sample_rate / 8363)
    let semitones = 12.0 * (sample_rate as f64 / BASE_FREQ).log2();

    // Split into integer and fractional parts
    let relative_note = semitones.floor() as i32;
    let finetune = ((semitones - relative_note as f64) * 128.0).round() as i32;

    // Handle edge case where finetune rounds to 128
    let (finetune, relative_note) = if finetune >= 128 {
        (finetune - 128, relative_note + 1)
    } else {
        (finetune, relative_note)
    };

    (finetune as i8, relative_note as i8)
}

/// Write an instrument header with pitch correction for ROM samples at 22050 Hz.
///
/// This writes a full instrument header with num_samples=1 but sample_length=0.
/// The pitch correction (finetune/relative_note) tells the tracker how to play
/// ROM samples that are stored at 22050 Hz (ZX standard) when XM expects 8363 Hz.
///
/// For 22050 Hz samples: relative_note=16, finetune=101
/// Formula verification: 8363 × 2^((16 + 101/128) / 12) ≈ 22050 Hz
fn write_instrument(xm: &mut Vec<u8>, name: &str) {
    write_instrument_with_pitch(xm, name, SAMPLE_RATE as u32);
}

/// Write an instrument header with pitch correction for a specific sample rate.
///
/// Use this when samples are generated at different rates (e.g., bass at 11025 Hz).
fn write_instrument_with_pitch(xm: &mut Vec<u8>, name: &str, sample_rate: u32) {
    // Extended instrument header size INCLUDES the 4-byte field itself
    // Same structure as write_instrument_with_sample, but sample_length = 0
    // Content: 22 name + 1 type + 2 num_samples + 4 sample_header_size + 96 mapping +
    // 48 vol_env + 48 pan_env + 2 num_points + 6 sustain/loop + 2 env_flags +
    // 4 vibrato + 2 fadeout + 2 reserved = 239
    // Total with field: 4 + 239 = 243
    let header_size: u32 = 243;
    xm.extend_from_slice(&header_size.to_le_bytes());

    // Instrument name (22 bytes)
    let name_bytes = name.as_bytes();
    xm.extend_from_slice(&name_bytes[..name_bytes.len().min(22)]);
    xm.extend(std::iter::repeat(0u8).take(22 - name_bytes.len().min(22)));

    // Instrument type (0)
    xm.push(0);

    // Number of samples (1) - we need a sample header for pitch info
    xm.extend_from_slice(&1u16.to_le_bytes());

    // Sample header size (40 bytes)
    xm.extend_from_slice(&40u32.to_le_bytes());

    // Sample mapping (96 bytes - all notes map to sample 0)
    xm.extend(std::iter::repeat(0u8).take(96));

    // Volume envelope points (48 bytes) - simple sustain envelope
    xm.extend_from_slice(&0u16.to_le_bytes()); // Point 0: x=0
    xm.extend_from_slice(&64u16.to_le_bytes()); // Point 0: y=64
    xm.extend(std::iter::repeat(0u8).take(44)); // Remaining 11 points

    // Panning envelope points (48 bytes) - disabled
    xm.extend(std::iter::repeat(0u8).take(48));

    // Number of volume/panning envelope points
    xm.push(1); // num_vol_points
    xm.push(1); // num_pan_points

    // Volume envelope sustain/loop points (3 bytes)
    xm.push(0); // vol_sustain
    xm.push(0); // vol_loop_start
    xm.push(0); // vol_loop_end

    // Panning envelope sustain/loop points (3 bytes)
    xm.push(0); // pan_sustain
    xm.push(0); // pan_loop_start
    xm.push(0); // pan_loop_end

    // Envelope type flags (2 bytes)
    xm.push(0x01); // vol_type (volume envelope enabled)
    xm.push(0x00); // pan_type (panning envelope disabled)

    // Vibrato (4 bytes)
    xm.push(0); // vibrato_type
    xm.push(0); // vibrato_sweep
    xm.push(0); // vibrato_depth
    xm.push(0); // vibrato_rate

    // Volume fadeout (2 bytes)
    xm.extend_from_slice(&328u16.to_le_bytes());

    // Reserved (2 bytes to reach header_size - 4)
    xm.extend_from_slice(&[0u8; 2]);

    // ========== Sample Header (40 bytes) ==========

    // Calculate pitch correction for the sample rate
    let (finetune, relative_note) = calculate_pitch_correction(sample_rate);

    // Sample length (4 bytes) - 0 because sample comes from ROM
    xm.extend_from_slice(&0u32.to_le_bytes());

    // Loop start/length (no loop - ROM samples handle their own loops)
    xm.extend_from_slice(&0u32.to_le_bytes());
    xm.extend_from_slice(&0u32.to_le_bytes());

    // Volume (64 = max)
    xm.push(64);

    // Finetune (signed byte)
    xm.push(finetune as u8);

    // Type (0x10 = 16-bit, no loop)
    xm.push(0x10);

    // Panning (128 = center)
    xm.push(128);

    // Relative note (signed byte)
    xm.push(relative_note as u8);

    // Reserved
    xm.push(0);

    // Sample name (22 bytes) - same as instrument name
    xm.extend_from_slice(&name_bytes[..name_bytes.len().min(22)]);
    xm.extend(std::iter::repeat(0u8).take(22 - name_bytes.len().min(22)));

    // NO SAMPLE DATA - sample comes from ROM at runtime
}

/// Write an instrument header with embedded sample data
fn write_instrument_with_sample(xm: &mut Vec<u8>, name: &str, sample_data: &[i16]) {
    // Extended instrument header size INCLUDES the 4-byte field itself
    // Parser does: cursor.seek(header_start + header_size - 4)
    // Content: 22 name + 1 type + 2 num_samples + 4 sample_header_size + 96 mapping +
    // 48 vol_env + 48 pan_env + 2 num_points + 6 sustain/loop + 2 env_flags +
    // 4 vibrato + 2 fadeout + 2 reserved = 239
    // Total with field: 4 + 239 = 243
    let header_size: u32 = 243;
    xm.extend_from_slice(&header_size.to_le_bytes());

    // Instrument name (22 bytes)
    let name_bytes = name.as_bytes();
    xm.extend_from_slice(&name_bytes[..name_bytes.len().min(22)]);
    xm.extend(std::iter::repeat(0u8).take(22 - name_bytes.len().min(22)));

    // Instrument type (0)
    xm.push(0);

    // Number of samples (1)
    xm.extend_from_slice(&1u16.to_le_bytes());

    // Sample header size (40 bytes)
    xm.extend_from_slice(&40u32.to_le_bytes());

    // Sample mapping (96 bytes - all notes map to sample 0)
    xm.extend(std::iter::repeat(0u8).take(96));

    // Volume envelope points (48 bytes) - simple sustain envelope
    xm.extend_from_slice(&0u16.to_le_bytes()); // Point 0: x=0
    xm.extend_from_slice(&64u16.to_le_bytes()); // Point 0: y=64
    xm.extend(std::iter::repeat(0u8).take(44)); // Remaining 11 points

    // Panning envelope points (48 bytes) - disabled
    xm.extend(std::iter::repeat(0u8).take(48));

    // Number of volume/panning envelope points
    xm.push(1); // num_vol_points
    xm.push(1); // num_pan_points

    // Volume envelope sustain/loop points (3 bytes)
    xm.push(0); // vol_sustain
    xm.push(0); // vol_loop_start
    xm.push(0); // vol_loop_end

    // Panning envelope sustain/loop points (3 bytes)
    xm.push(0); // pan_sustain
    xm.push(0); // pan_loop_start
    xm.push(0); // pan_loop_end

    // Envelope type flags (2 bytes)
    xm.push(0x01); // vol_type (volume envelope enabled)
    xm.push(0x00); // pan_type (panning envelope disabled)

    // Vibrato (4 bytes)
    xm.push(0); // vibrato_type
    xm.push(0); // vibrato_sweep
    xm.push(0); // vibrato_depth
    xm.push(0); // vibrato_rate

    // Volume fadeout (2 bytes)
    xm.extend_from_slice(&328u16.to_le_bytes());

    // Reserved (2 bytes to reach header_size - 4)
    xm.extend_from_slice(&[0u8; 2]);

    // Sample header
    let sample_len = (sample_data.len() * 2) as u32; // 16-bit samples
    xm.extend_from_slice(&sample_len.to_le_bytes());

    // Loop start/length (no loop)
    xm.extend_from_slice(&0u32.to_le_bytes());
    xm.extend_from_slice(&0u32.to_le_bytes());

    // Volume (64 = max)
    xm.push(64);

    // Calculate pitch correction for 22050 Hz samples
    // Formula: semitones = 12 × log₂(22050/8363) = 16.784
    // finetune = 100, relative_note = 16
    let (finetune, relative_note) = calculate_pitch_correction(SAMPLE_RATE as u32);
    xm.push(finetune as u8);

    // Type (0x10 = 16-bit)
    xm.push(0x10);

    // Panning (128 = center)
    xm.push(128);

    // Relative note (calculated from sample rate)
    xm.push(relative_note as u8);

    // Reserved
    xm.push(0);

    // Sample name (22 bytes) - same as instrument name
    xm.extend_from_slice(&name_bytes[..name_bytes.len().min(22)]);
    xm.extend(std::iter::repeat(0u8).take(22 - name_bytes.len().min(22)));

    // Sample data (delta-encoded 16-bit)
    let mut old = 0i16;
    for &sample in sample_data {
        let delta = sample.wrapping_sub(old);
        xm.extend_from_slice(&delta.to_le_bytes());
        old = sample;
    }
}

// ============================================================================
// SYNTHWAVE SONG - "Nether Drive"
// ============================================================================

fn generate_synthwave_assets(output_dir: &Path) {
    // Generate synthwave instruments (with tracker- prefix for shared assets folder)
    let mut kick = generate_kick_synth();
    apply_fades(&mut kick);
    write_wav(&output_dir.join("tracker-kick_synth.wav"), &kick);
    println!("  Generated tracker-kick_synth.wav ({} samples)", kick.len());

    let mut snare = generate_snare_synth();
    apply_fades(&mut snare);
    write_wav(&output_dir.join("tracker-snare_synth.wav"), &snare);
    println!("  Generated tracker-snare_synth.wav ({} samples)", snare.len());

    let mut hihat = generate_hihat_synth();
    apply_fades(&mut hihat);
    write_wav(&output_dir.join("tracker-hihat_synth.wav"), &hihat);
    println!("  Generated tracker-hihat_synth.wav ({} samples)", hihat.len());

    let mut bass = generate_bass_synth();
    apply_fades(&mut bass);
    write_wav(&output_dir.join("tracker-bass_synth.wav"), &bass);
    println!("  Generated tracker-bass_synth.wav ({} samples)", bass.len());

    let mut lead = generate_lead_synth();
    apply_fades(&mut lead);
    write_wav(&output_dir.join("tracker-lead_synth.wav"), &lead);
    println!("  Generated tracker-lead_synth.wav ({} samples)", lead.len());

    let mut arp = generate_arp_synth();
    apply_fades(&mut arp);
    write_wav(&output_dir.join("tracker-arp_synth.wav"), &arp);
    println!("  Generated tracker-arp_synth.wav ({} samples)", arp.len());

    let mut pad = generate_pad_synth();
    apply_fades(&mut pad);
    write_wav(&output_dir.join("tracker-pad_synth.wav"), &pad);
    println!("  Generated tracker-pad_synth.wav ({} samples)", pad.len());

    // Generate sample-less XM file
    let xm = generate_synthwave_xm();
    fs::write(output_dir.join("tracker-nether_drive.xm"), &xm).expect("Failed to write tracker-nether_drive.xm");
    println!("  Generated tracker-nether_drive.xm ({} bytes)", xm.len());

    // Generate embedded XM file
    let samples = vec![kick, snare, hihat, bass, lead, arp, pad];
    let xm_embedded = generate_synthwave_xm_embedded(&samples);
    fs::write(output_dir.join("tracker-nether_drive-embedded.xm"), &xm_embedded).expect("Failed to write tracker-nether_drive-embedded.xm");
    println!("  Generated tracker-nether_drive-embedded.xm ({} bytes)", xm_embedded.len());
}

/// Synthwave kick: Enhanced 808-style with rich harmonics and warmth
fn generate_kick_synth() -> Vec<i16> {
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
fn generate_snare_synth() -> Vec<i16> {
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
fn generate_hihat_synth() -> Vec<i16> {
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
fn generate_bass_synth() -> Vec<i16> {
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
fn generate_lead_synth() -> Vec<i16> {
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
fn generate_arp_synth() -> Vec<i16> {
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
fn generate_pad_synth() -> Vec<i16> {
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
        let cutoff = 0.16; // FIXED: Was 0.042 (way too aggressive for 4 poles!)
        lp1 += cutoff * (osc_sum - lp1);
        lp2 += cutoff * (lp1 - lp2);
        lp3 += cutoff * (lp2 - lp3);
        lp4 += cutoff * (lp3 - lp4);

        let sample = lp4 * envelope * 30000.0; // Increased gain to compensate
        output.push(sample.clamp(-32767.0, 32767.0) as i16);
    }

    output
}

// ============================================================================
// XM File Generation - Synthwave "Nether Drive"
// ============================================================================

fn generate_synthwave_xm() -> Vec<u8> {
    let mut xm = Vec::new();

    // XM Header
    xm.extend_from_slice(b"Extended Module: ");

    // Module name
    let name = b"Nether Drive";
    xm.extend_from_slice(name);
    xm.extend(std::iter::repeat(0u8).take(20 - name.len()));

    xm.push(0x1A);

    // Tracker name
    let tracker = b"gen-tracker-demo";
    xm.extend_from_slice(tracker);
    xm.extend(std::iter::repeat(0u8).take(20 - tracker.len()));

    // Version
    xm.extend_from_slice(&0x0104u16.to_le_bytes());

    // Header size (276 = 4 bytes header_size + 16 bytes of header fields + 256 byte order table)
    // Per XM spec, header_size is measured from the position of this field itself
    xm.extend_from_slice(&276u32.to_le_bytes());

    // Song length (12 orders)
    xm.extend_from_slice(&12u16.to_le_bytes());

    // Restart position
    xm.extend_from_slice(&1u16.to_le_bytes());

    // Number of channels (8)
    xm.extend_from_slice(&8u16.to_le_bytes());

    // Number of patterns (8)
    xm.extend_from_slice(&8u16.to_le_bytes());

    // Number of instruments (7)
    xm.extend_from_slice(&7u16.to_le_bytes());

    // Flags
    xm.extend_from_slice(&1u16.to_le_bytes());

    // Default speed
    xm.extend_from_slice(&6u16.to_le_bytes());

    // Default BPM (105 for Synthwave)
    xm.extend_from_slice(&105u16.to_le_bytes());

    // Pattern order: Intro -> Verse A -> Verse B -> Chorus -> Verse A -> Verse B -> Bridge -> Chorus -> Outro
    let order = [0u8, 1, 2, 3, 4, 1, 2, 5, 6, 3, 4, 7];
    xm.extend_from_slice(&order);
    xm.extend(std::iter::repeat(0u8).take(256 - order.len()));

    // Generate patterns
    for i in 0..8 {
        let pattern_data = match i {
            0 => generate_synth_pattern_intro(),
            1 => generate_synth_pattern_verse_a(),
            2 => generate_synth_pattern_verse_b(),
            3 => generate_synth_pattern_chorus_a(),
            4 => generate_synth_pattern_chorus_b(),
            5 => generate_synth_pattern_bridge(),
            6 => generate_synth_pattern_build(),
            7 => generate_synth_pattern_outro(),
            _ => unreachable!(),
        };
        let pattern_size = pattern_data.len() as u16;

        // Debug validation
        eprintln!("Synthwave Pattern {}: size={} bytes", i, pattern_size);
        if pattern_size < 256 {
            eprintln!("WARNING: Synthwave Pattern {} too small (expected min 256)", i);
        }

        xm.extend_from_slice(&9u32.to_le_bytes()); // header length (including length field: 4+1+2+2=9)
        xm.push(0);
        xm.extend_from_slice(&32u16.to_le_bytes());
        xm.extend_from_slice(&pattern_size.to_le_bytes());
        xm.extend_from_slice(&pattern_data);
    }

    // Instruments
    let instruments = [
        "kick_synth",
        "snare_synth",
        "hihat_synth",
        "bass_synth",
        "lead_synth",
        "arp_synth",
        "pad_synth",
    ];
    for name in &instruments {
        write_instrument(&mut xm, name);
    }

    xm
}

fn generate_synthwave_xm_embedded(samples: &[Vec<i16>]) -> Vec<u8> {
    let mut xm = Vec::new();

    // XM Header
    xm.extend_from_slice(b"Extended Module: ");

    let name = b"Nether Drive";
    xm.extend_from_slice(name);
    xm.extend(std::iter::repeat(0u8).take(20 - name.len()));

    xm.push(0x1A);

    let tracker = b"gen-tracker-demo";
    xm.extend_from_slice(tracker);
    xm.extend(std::iter::repeat(0u8).take(20 - tracker.len()));

    xm.extend_from_slice(&0x0104u16.to_le_bytes());
    xm.extend_from_slice(&276u32.to_le_bytes());
    xm.extend_from_slice(&12u16.to_le_bytes());
    xm.extend_from_slice(&1u16.to_le_bytes());
    xm.extend_from_slice(&8u16.to_le_bytes());
    xm.extend_from_slice(&8u16.to_le_bytes());
    xm.extend_from_slice(&7u16.to_le_bytes());
    xm.extend_from_slice(&1u16.to_le_bytes());
    xm.extend_from_slice(&6u16.to_le_bytes());
    xm.extend_from_slice(&105u16.to_le_bytes());

    let order = [0u8, 1, 2, 3, 4, 1, 2, 5, 6, 3, 4, 7];
    xm.extend_from_slice(&order);
    xm.extend(std::iter::repeat(0u8).take(256 - order.len()));

    for i in 0..8 {
        let pattern_data = match i {
            0 => generate_synth_pattern_intro(),
            1 => generate_synth_pattern_verse_a(),
            2 => generate_synth_pattern_verse_b(),
            3 => generate_synth_pattern_chorus_a(),
            4 => generate_synth_pattern_chorus_b(),
            5 => generate_synth_pattern_bridge(),
            6 => generate_synth_pattern_build(),
            7 => generate_synth_pattern_outro(),
            _ => unreachable!(),
        };
        let pattern_size = pattern_data.len() as u16;

        xm.extend_from_slice(&9u32.to_le_bytes());
        xm.push(0);
        xm.extend_from_slice(&32u16.to_le_bytes());
        xm.extend_from_slice(&pattern_size.to_le_bytes());
        xm.extend_from_slice(&pattern_data);
    }

    let instruments = [
        "kick_synth",
        "snare_synth",
        "hihat_synth",
        "bass_synth",
        "lead_synth",
        "arp_synth",
        "pad_synth",
    ];
    for (i, name) in instruments.iter().enumerate() {
        write_instrument_with_sample(&mut xm, name, &samples[i]);
    }

    xm
}

// Synthwave note constants (A minor: A B C D E F G, plus G# for E major chord)
const A2_S: u8 = 34;
const B2_S: u8 = 36;
const C3_S: u8 = 37;
const D3_S: u8 = 39;
const E3_S: u8 = 41;
const F3_S: u8 = 42;
const G3_S: u8 = 44;
const GS3_S: u8 = 45; // G#3 for E major chord
const A3_S: u8 = 46;
const B3_S: u8 = 48;
const C4_S: u8 = 49;
const D4_S: u8 = 51;
const E4_S: u8 = 53;
const F4_S: u8 = 54;
const G4_S: u8 = 56;
#[ignore]
const GS4_S: u8 = 57; // G#4 for E major chord
const A4_S: u8 = 58;
const B4_S: u8 = 60;
const C5_S: u8 = 61;
const D5_S: u8 = 63;
const E5_S: u8 = 65;

// Synthwave instruments
const KICK_S: u8 = 1;
const SNARE_S: u8 = 2;
const HIHAT_S: u8 = 3;
const BASS_S: u8 = 4;
const LEAD_S: u8 = 5;
const ARP_S: u8 = 6;
const PAD_S: u8 = 7;

/// Synthwave Pattern 0: Intro - Synths warming up, atmospheric
fn generate_synth_pattern_intro() -> Vec<u8> {
    let mut data = Vec::new();

    for row in 0..32 {
        // Ch1: Kick - sparse, beat 1 only
        if row == 0 || row == 16 {
            write_note(&mut data, C4_S, KICK_S);
        } else {
            write_empty(&mut data);
        }

        // Ch2: Snare - silent in intro
        write_empty(&mut data);

        // Ch3: Hi-hat - enters mid-pattern
        if row >= 16 && row % 4 == 0 {
            write_note(&mut data, C4_S, HIHAT_S);
        } else {
            write_empty(&mut data);
        }

        // Ch4: Bass - Am pedal, smooth pulsing
        if row == 0 || row == 16 {
            write_note(&mut data, A2_S, BASS_S);
        } else {
            write_empty(&mut data);
        }

        // Ch5: Lead - silent
        write_empty(&mut data);

        // Ch6: Arp - starts at row 8, simple Am pattern
        if row >= 8 && row % 4 == 0 {
            let arp_notes = [A3_S, C4_S, E4_S, C4_S, A3_S, C4_S, E4_S, C4_S];
            let idx = ((row - 8) / 4) as usize % 8;
            write_note(&mut data, arp_notes[idx], ARP_S);
        } else {
            write_empty(&mut data);
        }

        // Ch7: Pad - Am chord swell
        if row == 0 {
            write_note(&mut data, A3_S, PAD_S);
        } else {
            write_empty(&mut data);
        }

        // Ch8: Silent
        write_empty(&mut data);
    }

    data
}

/// Synthwave Pattern 1: Verse A - Main groove establishes
fn generate_synth_pattern_verse_a() -> Vec<u8> {
    let mut data = Vec::new();

    // Bass line: Am - F - C - G (smooth quarter notes)
    let bass_pattern = [
        A2_S, A2_S, A2_S, A2_S, A2_S, A2_S, A2_S, A2_S, // Am
        F3_S, F3_S, F3_S, F3_S, F3_S, F3_S, F3_S, F3_S, // F
        C3_S, C3_S, C3_S, C3_S, C3_S, C3_S, C3_S, C3_S, // C
        G3_S, G3_S, G3_S, G3_S, G3_S, G3_S, G3_S, G3_S, // G
    ];

    // Simple melodic line
    let melody = [
        0, 0, E4_S, 0, D4_S, 0, C4_S, 0,
        0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, C4_S, 0, D4_S, 0, E4_S, 0,
        0, 0, D4_S, 0, 0, 0, 0, 0,
    ];

    for row in 0..32 {
        // Ch1: Kick - beats 1 and 3
        if row % 8 == 0 || row % 8 == 4 {
            write_note(&mut data, C4_S, KICK_S);
        } else {
            write_empty(&mut data);
        }

        // Ch2: Snare - beats 2 and 4
        if row % 8 == 2 || row % 8 == 6 {
            write_note(&mut data, C4_S, SNARE_S);
        } else {
            write_empty(&mut data);
        }

        // Ch3: Hi-hat - 8th notes
        if row % 2 == 0 {
            write_note(&mut data, C4_S, HIHAT_S);
        } else {
            write_empty(&mut data);
        }

        // Ch4: Bass - smooth quarter notes
        if row % 4 == 0 {
            write_note(&mut data, bass_pattern[row as usize], BASS_S);
        } else {
            write_empty(&mut data);
        }

        // Ch5: Lead melody
        let mel = melody[row as usize];
        if mel != 0 {
            write_note(&mut data, mel, LEAD_S);
        } else {
            write_empty(&mut data);
        }

        // Ch6: Arp - 16th note pattern
        let arp_notes = [A3_S, C4_S, E4_S, C4_S];
        write_note(&mut data, arp_notes[(row % 4) as usize], ARP_S);

        // Ch7: Pad - chord on downbeats
        if row == 0 {
            write_note(&mut data, A3_S, PAD_S); // Am
        } else if row == 8 {
            write_note(&mut data, F3_S, PAD_S); // F
        } else if row == 16 {
            write_note(&mut data, C4_S, PAD_S); // C
        } else if row == 24 {
            write_note(&mut data, G3_S, PAD_S); // G
        } else {
            write_empty(&mut data);
        }

        // Ch8: Silent in Verse A - simple melody doesn't need harmony
        // Harmony comes in later patterns for build effect
        write_empty(&mut data);
    }

    data
}

/// Synthwave Pattern 2: Verse B - More movement
fn generate_synth_pattern_verse_b() -> Vec<u8> {
    let mut data = Vec::new();

    // Bass with more movement: Am - F - C - Em
    let bass_pattern = [
        A2_S, A2_S, A3_S, A2_S, A2_S, A2_S, A3_S, A2_S, // Am with octave
        F3_S, F3_S, A3_S, F3_S, F3_S, F3_S, C3_S, F3_S, // F
        C3_S, C3_S, E3_S, C3_S, C3_S, C3_S, G3_S, C3_S, // C
        E3_S, E3_S, G3_S, E3_S, E3_S, E3_S, B2_S, E3_S, // Em
    ];

    // More active melody
    let melody = [
        E4_S, 0, D4_S, C4_S, 0, 0, B3_S, 0,
        A3_S, 0, 0, 0, C4_S, 0, D4_S, 0,
        E4_S, 0, G4_S, 0, E4_S, 0, D4_S, 0,
        C4_S, 0, B3_S, 0, A3_S, 0, 0, 0,
    ];

    for row in 0..32 {
        // Ch1: Kick with off-beat at end
        if row % 8 == 0 || row % 8 == 4 || (row >= 28 && row % 2 == 0) {
            write_note(&mut data, C4_S, KICK_S);
        } else {
            write_empty(&mut data);
        }

        // Ch2: Snare - beats 2 and 4 with ghost notes
        if row % 8 == 2 || row % 8 == 6 {
            write_note(&mut data, C4_S, SNARE_S);
        } else if row == 12 || row == 28 {
            write_note_vol(&mut data, C4_S, SNARE_S, 0x20); // Ghost
        } else {
            write_empty(&mut data);
        }

        // Ch3: Hi-hat - 8th notes with accents
        if row % 2 == 0 {
            if row % 4 == 0 {
                write_note(&mut data, C4_S, HIHAT_S);
            } else {
                write_note_vol(&mut data, C4_S, HIHAT_S, 0x28);
            }
        } else {
            write_empty(&mut data);
        }

        // Ch4: Bass with movement
        if row % 2 == 0 {
            write_note(&mut data, bass_pattern[row as usize], BASS_S);
        } else {
            write_empty(&mut data);
        }

        // Ch5: Lead melody
        let mel = melody[row as usize];
        if mel != 0 {
            write_note(&mut data, mel, LEAD_S);
        } else {
            write_empty(&mut data);
        }

        // Ch6: Arp follows chords
        let arp_patterns: [[u8; 4]; 4] = [
            [A3_S, C4_S, E4_S, C4_S], // Am
            [F3_S, A3_S, C4_S, A3_S], // F
            [C4_S, E4_S, G4_S, E4_S], // C
            [E3_S, G3_S, B3_S, G3_S], // Em
        ];
        let chord_idx = (row / 8) as usize;
        let arp_idx = (row % 4) as usize;
        write_note(&mut data, arp_patterns[chord_idx][arp_idx], ARP_S);

        // Ch7: Pad
        if row == 0 {
            write_note(&mut data, A3_S, PAD_S);
        } else if row == 8 {
            write_note(&mut data, F3_S, PAD_S);
        } else if row == 16 {
            write_note(&mut data, C4_S, PAD_S);
        } else if row == 24 {
            write_note(&mut data, E3_S, PAD_S);
        } else {
            write_empty(&mut data);
        }

        // Ch8: Empty for variation
        write_empty(&mut data);
    }

    data
}

/// Synthwave Pattern 3: Chorus A - Energy peak, soaring lead
fn generate_synth_pattern_chorus_a() -> Vec<u8> {
    let mut data = Vec::new();

    // Bass: F - G - Am - Am
    let bass_roots = [F3_S, F3_S, F3_S, F3_S, F3_S, F3_S, F3_S, F3_S,
                      G3_S, G3_S, G3_S, G3_S, G3_S, G3_S, G3_S, G3_S,
                      A2_S, A2_S, A2_S, A2_S, A2_S, A2_S, A2_S, A2_S,
                      A2_S, A2_S, A2_S, A2_S, A2_S, A2_S, A2_S, A2_S];

    // Soaring chorus melody
    let melody = [
        A4_S, 0, C5_S, 0, 0, 0, B4_S, A4_S,
        G4_S, 0, 0, 0, A4_S, 0, B4_S, 0,
        C5_S, 0, 0, 0, B4_S, 0, A4_S, 0,
        G4_S, 0, E4_S, 0, A4_S, 0, 0, 0,
    ];

    for row in 0..32 {
        // Ch1: Kick - full four-on-floor with off-beats
        if row % 4 == 0 || row % 8 == 6 {
            write_note(&mut data, C4_S, KICK_S);
        } else {
            write_empty(&mut data);
        }

        // Ch2: Snare with fills
        if row % 8 == 2 || row % 8 == 6 {
            write_note(&mut data, C4_S, SNARE_S);
        } else if row >= 28 {
            write_note_vol(&mut data, C4_S, SNARE_S, 0x30); // Fill
        } else {
            write_empty(&mut data);
        }

        // Ch3: Hi-hat - 16ths for energy
        write_note(&mut data, C4_S, HIHAT_S);

        // Ch4: Bass - octave movement
        if row % 2 == 0 {
            let root = bass_roots[row as usize];
            let note = if (row / 2) % 2 == 0 { root } else { root + 12 };
            write_note(&mut data, note, BASS_S);
        } else {
            write_empty(&mut data);
        }

        // Ch5: Lead - soaring melody
        let mel = melody[row as usize];
        if mel != 0 {
            write_note(&mut data, mel, LEAD_S);
        } else {
            write_empty(&mut data);
        }

        // Ch6: Arp - faster for energy
        let arp_notes = [A3_S, C4_S, E4_S, A4_S, E4_S, C4_S, A3_S, C4_S];
        write_note(&mut data, arp_notes[(row % 8) as usize], ARP_S);

        // Ch7: Pad - full chords
        if row == 0 {
            write_note(&mut data, F4_S, PAD_S);
        } else if row == 8 {
            write_note(&mut data, G4_S, PAD_S);
        } else if row == 16 || row == 24 {
            write_note(&mut data, A3_S, PAD_S);
        } else {
            write_empty(&mut data);
        }

        // Ch8: Lead harmony - octave up
        if mel != 0 {
            write_note(&mut data, (mel + 12).min(96), LEAD_S);
        } else {
            write_empty(&mut data);
        }
    }

    data
}

/// Synthwave Pattern 4: Chorus B - Triumphant variation
fn generate_synth_pattern_chorus_b() -> Vec<u8> {
    let mut data = Vec::new();

    // Bass: F - G - C - E (major chord for drama)
    let bass_roots = [F3_S, F3_S, F3_S, F3_S, F3_S, F3_S, F3_S, F3_S,
                      G3_S, G3_S, G3_S, G3_S, G3_S, G3_S, G3_S, G3_S,
                      C3_S, C3_S, C3_S, C3_S, C3_S, C3_S, C3_S, C3_S,
                      E3_S, E3_S, E3_S, E3_S, E3_S, E3_S, E3_S, E3_S];

    // Triumphant melody with higher reach
    let melody = [
        C5_S, 0, E5_S, 0, D5_S, 0, C5_S, 0,
        B4_S, 0, D5_S, 0, C5_S, 0, B4_S, 0,
        C5_S, 0, 0, 0, E5_S, 0, D5_S, 0,
        C5_S, 0, B4_S, 0, A4_S, 0, 0, 0,
    ];

    for row in 0..32 {
        // Ch1: Kick - full energy
        if row % 4 == 0 || row % 8 == 6 {
            write_note(&mut data, C4_S, KICK_S);
        } else {
            write_empty(&mut data);
        }

        // Ch2: Snare
        if row % 8 == 2 || row % 8 == 6 {
            write_note(&mut data, C4_S, SNARE_S);
        } else {
            write_empty(&mut data);
        }

        // Ch3: Hi-hat
        write_note(&mut data, C4_S, HIHAT_S);

        // Ch4: Bass
        if row % 2 == 0 {
            let root = bass_roots[row as usize];
            let note = if (row / 2) % 2 == 0 { root } else { root + 12 };
            write_note(&mut data, note, BASS_S);
        } else {
            write_empty(&mut data);
        }

        // Ch5: Lead
        let mel = melody[row as usize];
        if mel != 0 {
            write_note(&mut data, mel, LEAD_S);
        } else {
            write_empty(&mut data);
        }

        // Ch6: Arp
        let arp_patterns: [[u8; 4]; 4] = [
            [F3_S, A3_S, C4_S, A3_S],
            [G3_S, B3_S, D4_S, B3_S],
            [C4_S, E4_S, G4_S, E4_S],
            [E3_S, GS3_S, B3_S, GS3_S], // E major (E-G#-B)
        ];
        let chord_idx = (row / 8) as usize;
        write_note(&mut data, arp_patterns[chord_idx][(row % 4) as usize], ARP_S);

        // Ch7: Pad
        if row == 0 {
            write_note(&mut data, F4_S, PAD_S);
        } else if row == 8 {
            write_note(&mut data, G4_S, PAD_S);
        } else if row == 16 {
            write_note(&mut data, C4_S, PAD_S);
        } else if row == 24 {
            write_note(&mut data, E4_S, PAD_S); // E major!
        } else {
            write_empty(&mut data);
        }

        // Ch8: Fifth harmony
        if mel != 0 {
            write_note(&mut data, mel + 7, LEAD_S); // Perfect fifth
        } else {
            write_empty(&mut data);
        }
    }

    data
}

/// Synthwave Pattern 5: Bridge - Atmospheric breakdown
fn generate_synth_pattern_bridge() -> Vec<u8> {
    let mut data = Vec::new();

    for row in 0..32 {
        // Ch1: Kick - sparse
        if row == 0 || row == 16 {
            write_note(&mut data, C4_S, KICK_S);
        } else {
            write_empty(&mut data);
        }

        // Ch2: Snare - removed
        write_empty(&mut data);

        // Ch3: Hi-hat - open feel
        if row % 8 == 0 {
            write_note(&mut data, C4_S, HIHAT_S);
        } else {
            write_empty(&mut data);
        }

        // Ch4: Bass - Am sustained
        if row == 0 || row == 16 {
            write_note(&mut data, A2_S, BASS_S);
        } else {
            write_empty(&mut data);
        }

        // Ch5: Lead - introspective phrase
        let melody = [
            E4_S, 0, 0, 0, D4_S, 0, 0, 0,
            C4_S, 0, 0, 0, 0, 0, 0, 0,
            A3_S, 0, 0, 0, B3_S, 0, 0, 0,
            C4_S, 0, 0, 0, 0, 0, 0, 0,
        ];
        let mel = melody[row as usize];
        if mel != 0 {
            write_note(&mut data, mel, LEAD_S);
        } else {
            write_empty(&mut data);
        }

        // Ch6: Arp - half speed
        if row % 8 == 0 {
            let notes = [A3_S, C4_S, E4_S, A3_S];
            write_note(&mut data, notes[(row / 8) as usize], ARP_S);
        } else {
            write_empty(&mut data);
        }

        // Ch7: Pad - Am to Dm
        if row == 0 {
            write_note(&mut data, A3_S, PAD_S);
        } else if row == 16 {
            write_note(&mut data, D3_S, PAD_S);
        } else {
            write_empty(&mut data);
        }

        // Ch8: Ambient swells
        write_empty(&mut data);
    }

    data
}

/// Synthwave Pattern 6: Build - Building back to chorus
fn generate_synth_pattern_build() -> Vec<u8> {
    let mut data = Vec::new();

    for row in 0..32 {
        // Ch1: Kick - increasing density
        if row < 16 {
            if row % 8 == 0 {
                write_note(&mut data, C4_S, KICK_S);
            } else {
                write_empty(&mut data);
            }
        } else {
            if row % 4 == 0 {
                write_note(&mut data, C4_S, KICK_S);
            } else {
                write_empty(&mut data);
            }
        }

        // Ch2: Snare - builds with rolls
        if row < 24 {
            if row % 8 == 4 {
                write_note(&mut data, C4_S, SNARE_S);
            } else {
                write_empty(&mut data);
            }
        } else {
            // Roll at end
            if row % 2 == 0 {
                write_note(&mut data, C4_S, SNARE_S);
            } else {
                write_note_vol(&mut data, C4_S, SNARE_S, 0x25);
            }
        }

        // Ch3: Hi-hat - increasing
        if row < 16 {
            if row % 4 == 0 {
                write_note(&mut data, C4_S, HIHAT_S);
            } else {
                write_empty(&mut data);
            }
        } else {
            if row % 2 == 0 {
                write_note(&mut data, C4_S, HIHAT_S);
            } else {
                write_empty(&mut data);
            }
        }

        // Ch4: Bass - A pedal building
        if row % 4 == 0 {
            write_note(&mut data, A2_S, BASS_S);
        } else {
            write_empty(&mut data);
        }

        // Ch5: Lead - rising
        let melody = [
            A3_S, 0, 0, 0, B3_S, 0, 0, 0,
            C4_S, 0, 0, 0, D4_S, 0, 0, 0,
            E4_S, 0, 0, 0, F4_S, 0, 0, 0,
            G4_S, 0, A4_S, 0, B4_S, 0, C5_S, 0,
        ];
        let mel = melody[row as usize];
        if mel != 0 {
            write_note(&mut data, mel, LEAD_S);
        } else {
            write_empty(&mut data);
        }

        // Ch6: Arp - builds
        if row >= 16 {
            let arp_notes = [A3_S, C4_S, E4_S, A4_S];
            write_note(&mut data, arp_notes[(row % 4) as usize], ARP_S);
        } else {
            write_empty(&mut data);
        }

        // Ch7: Pad - swelling
        if row == 0 {
            write_note(&mut data, A3_S, PAD_S);
        } else if row == 16 {
            write_note(&mut data, E4_S, PAD_S);
        } else {
            write_empty(&mut data);
        }

        // Ch8: Empty
        write_empty(&mut data);
    }

    data
}

/// Synthwave Pattern 7: Outro - Fading to loop point
fn generate_synth_pattern_outro() -> Vec<u8> {
    let mut data = Vec::new();

    for row in 0..32 {
        // Ch1: Kick - sparse
        if row == 0 || row == 16 {
            write_note(&mut data, C4_S, KICK_S);
        } else {
            write_empty(&mut data);
        }

        // Ch2: Snare - soft
        if row == 8 || row == 24 {
            write_note_vol(&mut data, C4_S, SNARE_S, 0x28);
        } else {
            write_empty(&mut data);
        }

        // Ch3: Hi-hat - quarters fading
        if row % 8 == 0 && row < 24 {
            write_note_vol(&mut data, C4_S, HIHAT_S, (0x30 - row) as u8);
        } else {
            write_empty(&mut data);
        }

        // Ch4: Bass - descending
        let bass_notes = [
            A3_S, 0, 0, 0, G3_S, 0, 0, 0,
            F3_S, 0, 0, 0, E3_S, 0, 0, 0,
            D3_S, 0, 0, 0, C3_S, 0, 0, 0,
            A2_S, 0, 0, 0, 0, 0, 0, 0,
        ];
        let bass = bass_notes[row as usize];
        if bass != 0 {
            write_note(&mut data, bass, BASS_S);
        } else {
            write_empty(&mut data);
        }

        // Ch5: Lead - final phrase
        let melody = [
            E4_S, 0, D4_S, 0, C4_S, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0,
            A3_S, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0,
        ];
        let mel = melody[row as usize];
        if mel != 0 {
            write_note(&mut data, mel, LEAD_S);
        } else {
            write_empty(&mut data);
        }

        // Ch6: Arp - slowing
        if row < 16 && row % 4 == 0 {
            write_note(&mut data, A3_S, ARP_S);
        } else {
            write_empty(&mut data);
        }

        // Ch7: Pad - Am sustained, fading
        if row == 0 {
            write_note(&mut data, A3_S, PAD_S);
        } else {
            write_empty(&mut data);
        }

        // Ch8: Empty
        write_empty(&mut data);
    }

    data
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_funk_xm_parses() {
        let xm_data = generate_funk_xm();
        let module =
            nether_xm::parse_xm(&xm_data).expect("Funk XM should parse");

        assert_eq!(module.name, "Nether Groove");
        assert_eq!(module.num_channels, 8);
        assert_eq!(module.num_patterns, 6);
        assert_eq!(module.num_instruments, 6);
        assert_eq!(module.default_bpm, 110);
    }

    #[test]
    fn test_eurobeat_xm_parses() {
        let xm_data = generate_eurobeat_xm();
        let module =
            nether_xm::parse_xm(&xm_data).expect("Eurobeat XM should parse");

        assert_eq!(module.name, "Nether Fire");
        assert_eq!(module.num_channels, 8);
        assert_eq!(module.num_patterns, 8);
        assert_eq!(module.num_instruments, 7);
        assert_eq!(module.default_bpm, 155);
    }

    #[test]
    fn test_synthwave_xm_parses() {
        let xm_data = generate_synthwave_xm();
        let module =
            nether_xm::parse_xm(&xm_data).expect("Synthwave XM should parse");

        assert_eq!(module.name, "Nether Drive");
        assert_eq!(module.num_channels, 8);
        assert_eq!(module.num_patterns, 8);
        assert_eq!(module.num_instruments, 7);
        assert_eq!(module.default_bpm, 105);
    }

    #[test]
    fn test_funk_instrument_names() {
        let xm_data = generate_funk_xm();
        let names = nether_xm::get_instrument_names(&xm_data)
            .expect("Should get funk instrument names");

        assert_eq!(names.len(), 6);
        assert_eq!(names[0], "kick_funk");
        assert_eq!(names[1], "snare_funk");
        assert_eq!(names[2], "hihat_funk");
        assert_eq!(names[3], "bass_funk");
        assert_eq!(names[4], "epiano");
        assert_eq!(names[5], "lead_jazz");
    }

    #[test]
    fn test_eurobeat_instrument_names() {
        let xm_data = generate_eurobeat_xm();
        let names = nether_xm::get_instrument_names(&xm_data)
            .expect("Should get eurobeat instrument names");

        assert_eq!(names.len(), 7);
        assert_eq!(names[0], "kick_euro");
        assert_eq!(names[1], "snare_euro");
        assert_eq!(names[2], "hihat_euro");
        assert_eq!(names[3], "bass_euro");
        assert_eq!(names[4], "supersaw");
        assert_eq!(names[5], "brass_euro");
        assert_eq!(names[6], "pad_euro");
    }

    #[test]
    fn test_synthwave_instrument_names() {
        let xm_data = generate_synthwave_xm();
        let names = nether_xm::get_instrument_names(&xm_data)
            .expect("Should get synthwave instrument names");

        assert_eq!(names.len(), 7);
        assert_eq!(names[0], "kick_synth");
        assert_eq!(names[1], "snare_synth");
        assert_eq!(names[2], "hihat_synth");
        assert_eq!(names[3], "bass_synth");
        assert_eq!(names[4], "lead_synth");
        assert_eq!(names[5], "arp_synth");
        assert_eq!(names[6], "pad_synth");
    }

    #[test]
    fn test_supersaw_synthesis() {
        let supersaw = generate_supersaw();
        assert!(!supersaw.is_empty());
        // Should have 5 detuned oscillators creating rich harmonics
        // Check that we have non-zero samples
        assert!(supersaw.iter().any(|&s| s != 0));
    }

    #[test]
    fn test_epiano_synthesis() {
        let epiano = generate_epiano();
        assert!(!epiano.is_empty());
        // FM synthesis should create bell-like tones
        assert!(epiano.iter().any(|&s| s != 0));
    }
}
