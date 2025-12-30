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

    for i in 0..samples {
        let t = i as f32 / SAMPLE_RATE;

        // Slower decay for warmer tone
        let decay = (-t * 10.0).exp();

        // Gentler pitch sweep: 120Hz down to 45Hz
        let freq = 120.0 * (-t * 12.0).exp() + 45.0;

        phase += 2.0 * PI * freq / SAMPLE_RATE;

        // Add slight saturation for warmth
        let mut sample = phase.sin() * decay;
        sample = (sample * 1.2).tanh(); // Soft clip

        output.push((sample * 30000.0).clamp(-32767.0, 32767.0) as i16);
    }

    output
}

/// Funk snare: medium decay, good for ghost notes, less harsh
fn generate_snare_funk() -> Vec<i16> {
    let duration = 0.25; // 250ms
    let samples = (SAMPLE_RATE * duration) as usize;

    let mut output = Vec::with_capacity(samples);
    let mut rng = SimpleRng::new(12345);

    // Filter state
    let mut lp_state = 0.0f32;

    for i in 0..samples {
        let t = i as f32 / SAMPLE_RATE;

        // Medium decay envelope
        let decay = (-t * 15.0).exp();

        // Noise component (slightly filtered for warmth)
        let noise = rng.next_f32() * 2.0 - 1.0;

        // Body component at 160Hz (warmer)
        let body = (2.0 * PI * 160.0 * t).sin();

        // Second harmonic for snap
        let snap = (2.0 * PI * 320.0 * t).sin() * (-t * 30.0).exp();

        // Mix: more body for funk feel
        let raw = noise * 0.5 + body * 0.35 + snap * 0.15;

        // Gentle low-pass
        lp_state += 0.4 * (raw - lp_state);

        let sample = lp_state * decay * 28000.0;
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

    // Band-pass filter states
    let mut hp_prev_in = 0.0f32;
    let mut hp_prev_out = 0.0f32;
    let mut lp_state = 0.0f32;

    for i in 0..samples {
        let t = i as f32 / SAMPLE_RATE;

        // Decay with slight sustain
        let decay = if t < 0.02 {
            t / 0.02 // Quick attack
        } else {
            (-((t - 0.02) * 25.0)).exp()
        };

        // Noise
        let noise = rng.next_f32() * 2.0 - 1.0;

        // High-pass (less aggressive for warmer tone)
        let hp_alpha = 0.85;
        let hp_out = hp_alpha * (hp_prev_out + noise - hp_prev_in);
        hp_prev_in = noise;
        hp_prev_out = hp_out;

        // Gentle low-pass to tame harshness
        lp_state += 0.6 * (hp_out - lp_state);

        let sample = lp_state * decay * 22000.0;
        output.push(sample.clamp(-32767.0, 32767.0) as i16);
    }

    output
}

/// Funk bass: sawtooth with filter envelope "pluck", chromatic-friendly
/// IMPROVED: Smoother attack and sustain, less choppy
fn generate_bass_funk() -> Vec<i16> {
    // From nether-groove.spec.md:
    // - Sawtooth with filter envelope "pluck"
    // - Attack: 5ms (fast but not instant)
    // - Filter sweep creates "slap" without click
    // - Slight pitch bend down on attack (funky!)
    // - Sustain: 400ms for smooth groove
    // - Sub sine at -1 octave for weight
    let duration = 0.55; // 550ms total (400ms sustain + release)
    let freq = 87.31; // F2 as base
    let samples = (SAMPLE_RATE * duration) as usize;

    let mut output = Vec::with_capacity(samples);
    let mut filtered = 0.0f32;

    for i in 0..samples {
        let t = i as f32 / SAMPLE_RATE;

        // EXACT SPEC: 5ms attack, linear ramp (not quadratic)
        let amp_env = if t < 0.005 {
            t / 0.005 // Linear ramp - 5ms attack
        } else if t < 0.4 {
            1.0 - (t - 0.005) * 0.15 // Slow decay to sustain
        } else {
            0.94 * (-(t - 0.4) * 3.0).exp() // Smooth release
        };

        // EXACT SPEC: Filter envelope creates "pluck" without click
        let filter_env = 0.08 + 0.25 * (-t * 20.0).exp();

        // Subtle pitch bend down on attack (funky but smooth)
        let pitch_bend = 1.0 + 0.02 * (-t * 25.0).exp();

        // Sawtooth wave
        let phase = (freq * pitch_bend * t) % 1.0;
        let saw = 2.0 * phase - 1.0;

        // Sub sine at -1 octave for weight
        let sub = (2.0 * PI * freq * 0.5 * t).sin() * 0.35;

        // Dynamic low-pass filter
        filtered += filter_env * (saw - filtered);

        let sample = (filtered * 0.65 + sub) * amp_env * 28000.0;
        output.push(sample.clamp(-32767.0, 32767.0) as i16);
    }

    output
}

/// Electric Piano: FM synthesis for Rhodes/Wurlitzer bell-like tone
fn generate_epiano() -> Vec<i16> {
    let duration = 1.0; // 1 second for chord sustain
    let freq = 261.63; // C4 as base
    let samples = (SAMPLE_RATE * duration) as usize;

    let mut output = Vec::with_capacity(samples);

    for i in 0..samples {
        let t = i as f32 / SAMPLE_RATE;

        // ADSR envelope
        let amp_env = if t < 0.01 {
            t / 0.01 // Fast attack
        } else if t < 0.3 {
            1.0 - (t - 0.01) * 0.3 // Decay to 91%
        } else if t < 0.7 {
            0.91 - (t - 0.3) * 0.2 // Slow decay to sustain
        } else {
            0.83 * (-(t - 0.7) * 4.0).exp() // Release
        };

        // FM synthesis: carrier + modulator
        // Modulator frequency = 2x carrier (creates bell-like harmonics)
        let mod_freq = freq * 2.0;
        let mod_index = 2.5 * (-t * 8.0).exp(); // Decaying modulation for bell attack

        let modulator = (2.0 * PI * mod_freq * t).sin() * mod_index;
        let carrier = (2.0 * PI * freq * t + modulator).sin();

        // Add subtle second partial for warmth
        let partial2 = (2.0 * PI * freq * 2.0 * t).sin() * 0.15 * (-t * 12.0).exp();

        let sample = (carrier + partial2) * amp_env * 24000.0;
        output.push(sample.clamp(-32767.0, 32767.0) as i16);
    }

    output
}

/// Jazz lead: filtered square with vibrato, smooth attack
fn generate_lead_jazz() -> Vec<i16> {
    let duration = 0.8; // 800ms
    let freq = 261.63; // C4 as base
    let samples = (SAMPLE_RATE * duration) as usize;

    let mut output = Vec::with_capacity(samples);
    let mut filtered = 0.0f32;
    let mut phase = 0.0f32; // Proper phase accumulator
    let mut vibrato_phase = 0.0f32;

    for i in 0..samples {
        let t = i as f32 / SAMPLE_RATE;

        // Smooth ADSR envelope
        let envelope = if t < 0.03 {
            t / 0.03 // Smooth attack (30ms)
        } else if t < 0.5 {
            1.0 - (t - 0.03) * 0.2 // Slow decay to 90%
        } else {
            0.9 * (-(t - 0.5) * 3.5).exp() // Release
        };

        // Delayed vibrato (jazz style) - using phase accumulator
        let vibrato_amount = if t < 0.15 { 0.0 } else { 0.004 * ((t - 0.15) * 2.0).min(1.0) };
        vibrato_phase += 5.0 / SAMPLE_RATE; // 5 Hz vibrato rate
        let vibrato = 1.0 + vibrato_amount * (vibrato_phase * 2.0 * PI).sin();

        // Accumulate phase properly (prevents discontinuities)
        phase += freq * vibrato / SAMPLE_RATE;
        phase = phase % 1.0;

        // Square wave with variable pulse width
        let pw = 0.5 + 0.03 * (t * 2.0).sin();
        let square = if phase < pw { 1.0 } else { -1.0 };

        // Warm low-pass filter
        filtered += 0.12 * (square - filtered);

        let sample = filtered * envelope * 22000.0;
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
fn generate_kick_euro() -> Vec<i16> {
    let duration = 0.3; // 300ms
    let samples = (SAMPLE_RATE * duration) as usize;

    let mut output = Vec::with_capacity(samples);
    let mut phase = 0.0f32;

    for i in 0..samples {
        let t = i as f32 / SAMPLE_RATE;

        // Fast decay for punch
        let decay = (-t * 18.0).exp();

        // Aggressive pitch sweep: 200Hz down to 40Hz
        let freq = 200.0 * (-t * 25.0).exp() + 40.0;

        phase += 2.0 * PI * freq / SAMPLE_RATE;

        // Add click transient
        let click = if t < 0.003 {
            (t / 0.003) * (1.0 - t / 0.003)
        } else {
            0.0
        };

        let sample = (phase.sin() + click * 0.3) * decay;

        // Hard clip for punch
        let clipped = (sample * 1.3).clamp(-1.0, 1.0);

        output.push((clipped * 32000.0).clamp(-32767.0, 32767.0) as i16);
    }

    output
}

/// Eurobeat snare: tight, crisp, with reverb tail feel
fn generate_snare_euro() -> Vec<i16> {
    let duration = 0.22; // 220ms
    let samples = (SAMPLE_RATE * duration) as usize;

    let mut output = Vec::with_capacity(samples);
    let mut rng = SimpleRng::new(99999);

    for i in 0..samples {
        let t = i as f32 / SAMPLE_RATE;

        // Two-stage decay: fast initial, slower tail
        let decay = if t < 0.05 {
            (-t * 35.0).exp()
        } else {
            0.17 * (-(t - 0.05) * 12.0).exp() // "Reverb" tail
        };

        // Noise burst
        let noise = rng.next_f32() * 2.0 - 1.0;

        // Body at 180Hz
        let body = (2.0 * PI * 180.0 * t).sin() * (-t * 40.0).exp();

        // High harmonic for crack
        let crack = (2.0 * PI * 400.0 * t).sin() * (-t * 50.0).exp();

        let sample = (noise * 0.55 + body * 0.30 + crack * 0.15) * decay * 30000.0;
        output.push(sample.clamp(-32767.0, 32767.0) as i16);
    }

    output
}

/// Eurobeat hi-hat: bright, cutting, fast decay for 16th note patterns
fn generate_hihat_euro() -> Vec<i16> {
    let duration = 0.08; // 80ms - very short for rapid patterns
    let samples = (SAMPLE_RATE * duration) as usize;

    let mut output = Vec::with_capacity(samples);
    let mut rng = SimpleRng::new(77777);

    let mut hp_prev_in = 0.0f32;
    let mut hp_prev_out = 0.0f32;

    for i in 0..samples {
        let t = i as f32 / SAMPLE_RATE;

        // Very fast decay
        let decay = (-t * 50.0).exp();

        let noise = rng.next_f32() * 2.0 - 1.0;

        // Aggressive high-pass for brightness
        let hp_alpha = 0.95;
        let hp_out = hp_alpha * (hp_prev_out + noise - hp_prev_in);
        hp_prev_in = noise;
        hp_prev_out = hp_out;

        let sample = hp_out * decay * 24000.0;
        output.push(sample.clamp(-32767.0, 32767.0) as i16);
    }

    output
}

/// Eurobeat bass: bouncy square wave, octave-jump friendly
/// IMPROVED: Smoother attack/release for better bounce feel without harshness
fn generate_bass_euro() -> Vec<i16> {
    // From nether-fire.spec.md:
    // - THE HAPPY BOUNCING BASS (critical for Eurobeat)
    // - Square wave (45% pulse width) + saw blend
    // - Snappy envelope: 2ms attack, short sustain, fast decay
    // - Sub sine for weight
    // - Duration: 250ms (short for bounce, not legato)
    let duration = 0.25; // EXACT SPEC: 250ms - short for bounce
    let freq = 73.42; // D2 as base
    let samples = (SAMPLE_RATE * duration) as usize;

    let mut output = Vec::with_capacity(samples);

    for i in 0..samples {
        let t = i as f32 / SAMPLE_RATE;

        // EXACT SPEC: 2ms attack, 50ms sustain, fast decay
        let envelope = if t < 0.002 {
            t / 0.002 // 2ms attack - INSTANT
        } else if t < 0.05 {
            1.0 // Short sustain (50ms)
        } else {
            (-(t - 0.05) * 12.0).exp() // Fast decay
        };

        // EXACT SPEC: pulse (45% duty) + saw blend
        let phase = (freq * t) % 1.0;
        let pulse = if phase < 0.45 { 1.0 } else { -1.0 };
        let saw = 2.0 * phase - 1.0;

        // EXACT SPEC: 70% pulse + 30% saw
        let mix = pulse * 0.7 + saw * 0.3;

        // Sub sine for weight
        let sub = (2.0 * PI * freq * 0.5 * t).sin() * 0.25;

        let sample = (mix * 0.75 + sub) * envelope * 26000.0;
        output.push(sample.clamp(-32767.0, 32767.0) as i16);
    }

    output
}

/// Supersaw: the classic Eurobeat lead - 5 detuned saw oscillators
fn generate_supersaw() -> Vec<i16> {
    let duration = 0.8; // 800ms
    let freq = 261.63; // C4 as base
    let samples = (SAMPLE_RATE * duration) as usize;

    let mut output = Vec::with_capacity(samples);

    // 5 oscillator detune amounts in cents (converted to frequency ratios)
    let detune_cents: [f32; 5] = [-15.0, -7.0, 0.0, 7.0, 15.0];
    let detune_ratios: Vec<f32> = detune_cents
        .iter()
        .map(|c| 2.0f32.powf(c / 1200.0))
        .collect();

    let mut filtered = 0.0f32;
    let mut phases = [0.0f32; 5]; // Proper phase accumulators for each oscillator
    let mut vibrato_phase = 0.0f32;

    for i in 0..samples {
        let t = i as f32 / SAMPLE_RATE;

        // ADSR envelope
        let envelope = if t < 0.01 {
            t / 0.01 // Fast attack
        } else if t < 0.5 {
            1.0 - (t - 0.01) * 0.15
        } else {
            0.85 * (-(t - 0.5) * 3.0).exp()
        };

        // Subtle vibrato with proper phase accumulation
        vibrato_phase += 5.5 / SAMPLE_RATE;
        let vibrato = 1.0 + 0.003 * (vibrato_phase * 2.0 * PI).sin();

        // Sum 5 detuned saw waves with proper phase accumulation
        let mut saw_sum = 0.0f32;
        for (idx, ratio) in detune_ratios.iter().enumerate() {
            let osc_freq = freq * ratio * vibrato;
            phases[idx] += osc_freq / SAMPLE_RATE;
            phases[idx] = phases[idx] % 1.0;
            saw_sum += 2.0 * phases[idx] - 1.0;
        }
        saw_sum /= 5.0; // Normalize

        // Bright low-pass filter (higher cutoff for Eurobeat brightness)
        filtered += 0.25 * (saw_sum - filtered);

        let sample = filtered * envelope * 26000.0;
        output.push(sample.clamp(-32767.0, 32767.0) as i16);
    }

    output
}

/// Eurobeat brass: detuned pulse waves with pitch bend envelope
fn generate_brass_euro() -> Vec<i16> {
    let duration = 0.7; // 700ms
    let freq = 261.63; // C4 as base
    let samples = (SAMPLE_RATE * duration) as usize;

    let mut output = Vec::with_capacity(samples);
    let mut filtered = 0.0f32;

    for i in 0..samples {
        let t = i as f32 / SAMPLE_RATE;

        // ADSR envelope
        let envelope = if t < 0.015 {
            t / 0.015 // Fast attack
        } else if t < 0.4 {
            1.0 - (t - 0.015) * 0.2
        } else {
            0.8 * (-(t - 0.4) * 3.5).exp()
        };

        // Pitch bend: slight rise then settle (Eurobeat characteristic)
        let pitch_bend = 1.0 + 0.015 * (1.0 - (-t * 15.0).exp());

        // Two detuned pulse waves (40% duty cycle)
        let detune = 1.005;
        let pw = 0.4;

        let phase1 = (freq * pitch_bend * t) % 1.0;
        let phase2 = (freq * pitch_bend * detune * t) % 1.0;

        let pulse1 = if phase1 < pw { 1.0 } else { -1.0 };
        let pulse2 = if phase2 < pw { 1.0 } else { -1.0 };

        // Add saw wave for brightness
        let phase_saw = (freq * pitch_bend * t) % 1.0;
        let saw = 2.0 * phase_saw - 1.0;

        let mix = pulse1 * 0.4 + pulse2 * 0.4 + saw * 0.2;

        // Resonant filter with envelope
        let cutoff = 0.1 + 0.15 * envelope;
        filtered += cutoff * (mix - filtered);

        let sample = filtered * envelope * 25000.0;
        output.push(sample.clamp(-32767.0, 32767.0) as i16);
    }

    output
}

/// Eurobeat pad: sustained, detuned for chord swells
fn generate_pad_euro() -> Vec<i16> {
    let duration = 1.5; // 1.5 seconds for long sustain
    let freq = 261.63; // C4 as base
    let samples = (SAMPLE_RATE * duration) as usize;

    let mut output = Vec::with_capacity(samples);
    let mut filtered = 0.0f32;

    for i in 0..samples {
        let t = i as f32 / SAMPLE_RATE;

        // Slow attack, long sustain
        let envelope = if t < 0.1 {
            t / 0.1 // 100ms attack
        } else if t < 1.0 {
            1.0
        } else {
            (-(t - 1.0) * 2.0).exp()
        };

        // Three slightly detuned saws for width
        let detune_amounts = [0.995, 1.0, 1.005];
        let mut sum = 0.0f32;
        for d in detune_amounts {
            let phase = (freq * d * t) % 1.0;
            sum += 2.0 * phase - 1.0;
        }
        sum /= 3.0;

        // Gentle low-pass for pad character
        filtered += 0.06 * (sum - filtered);

        let sample = filtered * envelope * 20000.0;
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
const A2_E: u8 = 34;
const BB2_E: u8 = 35;
const C3_E: u8 = 37;
const D3_E: u8 = 39;
const F3_E: u8 = 42;
const A3_E: u8 = 46;
const BB3_E: u8 = 47;
const C4_E: u8 = 49;
const D4_E: u8 = 51;
const E4_E: u8 = 53;
const F4_E: u8 = 54;
const G4_E: u8 = 56;
const A4_E: u8 = 58;
const BB4_E: u8 = 59;
const C5_E: u8 = 61;
const D5_E: u8 = 63;
const E5_E: u8 = 65;
const F5_E: u8 = 66;
const G5_E: u8 = 68;
const A5_E: u8 = 70;
// D Major for Picardy third
const FS4_E: u8 = 55; // F# for D major

// Eurobeat instruments
const KICK_E: u8 = 1;
const SNARE_E: u8 = 2;
const HIHAT_E: u8 = 3;
const BASS_E: u8 = 4;
const SUPERSAW: u8 = 5;
const BRASS: u8 = 6;
const PAD: u8 = 7;

/// Eurobeat Pattern 0: Intro - Filter sweep, building energy
fn generate_euro_pattern_intro() -> Vec<u8> {
    let mut data = Vec::new();

    for row in 0..32 {
        // Ch1: Kick - sparse at first, builds
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

        // Ch2: Snare - enters at row 24
        if row >= 24 && row % 4 == 0 {
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

        // Ch4: Bass - Dm pedal
        if row == 0 || row == 16 {
            write_note(&mut data, D2_E, BASS_E);
        } else {
            write_empty(&mut data);
        }

        // Ch5: Supersaw - silent in intro
        write_empty(&mut data);

        // Ch6: Brass - silent in intro
        write_empty(&mut data);

        // Ch7: Pad - Dm chord swell
        if row == 0 {
            write_note(&mut data, D3_E, PAD);
        } else if row == 8 {
            write_note(&mut data, F3_E, PAD);
        } else if row == 16 {
            write_note(&mut data, A3_E, PAD);
        } else {
            write_empty(&mut data);
        }

        // Ch8: Lead harmony - silent
        write_empty(&mut data);
    }

    data
}

/// Eurobeat Pattern 1: Verse A - Four-on-the-floor, octave bass
fn generate_euro_pattern_verse_a() -> Vec<u8> {
    let mut data = Vec::new();

    // Octave-bouncing bass on 8th notes: Dm -> C -> Bb -> C
    // Each chord gets 8 rows, bass plays on even rows (0,2,4,6)
    let bass_pattern: [(u8, u8); 16] = [
        // Dm (rows 0-7)
        (D2_E, D3_E), (D2_E, D3_E), (D2_E, D3_E), (D2_E, D3_E),
        // C (rows 8-15)
        (C3_E, C4_E), (C3_E, C4_E), (C3_E, C4_E), (C3_E, C4_E),
        // Bb (rows 16-23)
        (BB2_E, BB3_E), (BB2_E, BB3_E), (BB2_E, BB3_E), (BB2_E, BB3_E),
        // C (rows 24-31)
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

        // Ch4: Bass - octave bouncing on 8th notes (every 2 rows)
        if row % 2 == 0 {
            let idx = (row / 2) as usize;
            let (low, high) = bass_pattern[idx];
            // Alternate low-high within each beat pair
            let note = if (row / 2) % 2 == 0 { low } else { high };
            write_note(&mut data, note, BASS_E);
        } else {
            write_empty(&mut data);
        }

        // Ch5: Supersaw - silent in verse A
        write_empty(&mut data);

        // Ch6: Brass - chord stabs
        if row == 0 {
            write_note(&mut data, D4_E, BRASS);
        } else if row == 8 {
            write_note(&mut data, C4_E, BRASS);
        } else if row == 16 {
            write_note(&mut data, BB3_E, BRASS);
        } else if row == 24 {
            write_note(&mut data, C4_E, BRASS);
        } else {
            write_empty(&mut data);
        }

        // Ch7: Pad - sustained chords
        if row == 0 {
            write_note(&mut data, F3_E, PAD);
        } else if row == 16 {
            write_note(&mut data, D3_E, PAD);
        } else {
            write_empty(&mut data);
        }

        // Ch8: Empty
        write_empty(&mut data);
    }

    data
}

/// Eurobeat Pattern 2: Verse B - Add simple melody
fn generate_euro_pattern_verse_b() -> Vec<u8> {
    let mut data = Vec::new();

    // Bass pattern on 8th notes: Dm -> C -> Bb -> A (harmonic minor resolution!)
    let bass_pattern: [(u8, u8); 16] = [
        // Dm (rows 0-7)
        (D2_E, D3_E), (D2_E, D3_E), (D2_E, D3_E), (D2_E, D3_E),
        // C (rows 8-15)
        (C3_E, C4_E), (C3_E, C4_E), (C3_E, C4_E), (C3_E, C4_E),
        // Bb (rows 16-23)
        (BB2_E, BB3_E), (BB2_E, BB3_E), (BB2_E, BB3_E), (BB2_E, BB3_E),
        // A (rows 24-31) - harmonic minor!
        (A2_E, A3_E), (A2_E, A3_E), (A2_E, A3_E), (A2_E, A3_E),
    ];

    // Simple verse melody on 8th notes
    let melody: [u8; 16] = [
        D4_E, F4_E, A4_E, F4_E, // Dm arpeggio
        C4_E, E4_E, G4_E, E4_E, // C arpeggio
        BB3_E, D4_E, F4_E, D4_E, // Bb arpeggio
        A3_E, C4_E, E4_E, A4_E, // A (harmonic minor!)
    ];

    for row in 0..32 {
        // Ch1: Kick
        if row % 4 == 0 {
            write_note(&mut data, C4_E, KICK_E);
        } else {
            write_empty(&mut data);
        }

        // Ch2: Snare
        if row % 8 == 4 {
            write_note(&mut data, C4_E, SNARE_E);
        } else {
            write_empty(&mut data);
        }

        // Ch3: Hi-hat
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

        // Ch5: Supersaw melody on 8th notes
        if row % 2 == 0 {
            let idx = (row / 2) as usize;
            write_note(&mut data, melody[idx], SUPERSAW);
        } else {
            write_empty(&mut data);
        }

        // Ch6: Brass counter
        if row == 4 {
            write_note(&mut data, F4_E, BRASS);
        } else if row == 12 {
            write_note(&mut data, E4_E, BRASS);
        } else if row == 20 {
            write_note(&mut data, D4_E, BRASS);
        } else if row == 28 {
            write_note(&mut data, C4_E, BRASS);
        } else {
            write_empty(&mut data);
        }

        // Ch7: Pad
        if row == 0 {
            write_note(&mut data, A3_E, PAD);
        } else {
            write_empty(&mut data);
        }

        // Ch8: Empty
        write_empty(&mut data);
    }

    data
}

/// Eurobeat Pattern 3: Pre-Chorus - Build tension
fn generate_euro_pattern_prechorus() -> Vec<u8> {
    let mut data = Vec::new();

    // Rising run on 8th notes
    let melody: [u8; 16] = [
        D4_E, E4_E, F4_E, G4_E, A4_E, BB4_E, C5_E, D5_E,
        D5_E, E5_E, F5_E, G5_E, A5_E, A5_E, A5_E, A5_E,
    ];

    for row in 0..32 {
        // Ch1: Kick - double kicks building
        if row % 4 == 0 || (row >= 16 && row % 4 == 2) {
            write_note(&mut data, C4_E, KICK_E);
        } else {
            write_empty(&mut data);
        }

        // Ch2: Snare - rolls at end
        if row < 24 {
            if row % 8 == 4 {
                write_note(&mut data, C4_E, SNARE_E);
            } else {
                write_empty(&mut data);
            }
        } else if row % 2 == 0 {
            write_note(&mut data, C4_E, SNARE_E); // Snare roll
        } else {
            write_empty(&mut data);
        }

        // Ch3: Hi-hat - 16ths in second half
        if row < 16 {
            if row % 2 == 0 {
                write_note(&mut data, C4_E, HIHAT_E);
            } else {
                write_empty(&mut data);
            }
        } else {
            write_note(&mut data, C4_E, HIHAT_E); // 16th notes
        }

        // Ch4: Bass - A pedal, octave bounce on 8th notes
        if row % 2 == 0 {
            let note = if (row / 2) % 2 == 0 { A2_E } else { A3_E };
            write_note(&mut data, note, BASS_E);
        } else {
            write_empty(&mut data);
        }

        // Ch5: Rising melody on 8th notes
        if row % 2 == 0 {
            let idx = (row / 2) as usize;
            write_note(&mut data, melody[idx], SUPERSAW);
        } else {
            write_empty(&mut data);
        }

        // Ch6: Brass - building
        if row == 0 {
            write_note(&mut data, E4_E, BRASS);
        } else if row == 8 {
            write_note(&mut data, A4_E, BRASS);
        } else if row == 16 {
            write_note(&mut data, C5_E, BRASS);
        } else if row == 24 {
            write_note(&mut data, E5_E, BRASS);
        } else {
            write_empty(&mut data);
        }

        // Ch7: Pad swell
        if row == 0 {
            write_note(&mut data, A3_E, PAD);
        } else if row == 16 {
            write_note(&mut data, E4_E, PAD);
        } else {
            write_empty(&mut data);
        }

        // Ch8: Empty
        write_empty(&mut data);
    }

    data
}

/// Eurobeat Pattern 4: Chorus A - The hook! Full supersaw energy
fn generate_euro_pattern_chorus_a() -> Vec<u8> {
    let mut data = Vec::new();

    // Bass on 8th notes: Dm -> C -> Bb -> C
    let bass_pattern: [(u8, u8); 16] = [
        (D2_E, D3_E), (D2_E, D3_E), (D2_E, D3_E), (D2_E, D3_E),
        (C3_E, C4_E), (C3_E, C4_E), (C3_E, C4_E), (C3_E, C4_E),
        (BB2_E, BB3_E), (BB2_E, BB3_E), (BB2_E, BB3_E), (BB2_E, BB3_E),
        (C3_E, C4_E), (C3_E, C4_E), (C3_E, C4_E), (C3_E, C4_E),
    ];

    // THE HOOK - fast arpeggio runs (16ths for energy!)
    let melody = [
        D5_E, F5_E, A5_E, D5_E, A4_E, F5_E, D5_E, A4_E, // Dm fast arp
        C5_E, E5_E, G5_E, C5_E, G4_E, E5_E, C5_E, G4_E, // C fast arp
        BB4_E, D5_E, F5_E, BB4_E, F4_E, D5_E, BB4_E, F4_E, // Bb fast arp
        C5_E, E5_E, G5_E, C5_E, G4_E, E5_E, C5_E, G4_E, // C fast arp
    ];

    // Counter riff on 8th notes
    let brass_counter: [u8; 16] = [
        F4_E, A4_E, D5_E, A4_E, E4_E, G4_E, C5_E, G4_E,
        D4_E, F4_E, BB4_E, F4_E, E4_E, G4_E, C5_E, E5_E,
    ];

    for row in 0..32 {
        // Ch1: Kick - full power
        if row % 4 == 0 {
            write_note(&mut data, C4_E, KICK_E);
        } else {
            write_empty(&mut data);
        }

        // Ch2: Snare
        if row % 8 == 4 {
            write_note(&mut data, C4_E, SNARE_E);
        } else {
            write_empty(&mut data);
        }

        // Ch3: Hi-hat - 16ths for energy
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

        // Ch5: SUPERSAW HOOK - 16ths for that fast arp energy
        write_note(&mut data, melody[row as usize], SUPERSAW);

        // Ch6: Brass counter on 8th notes
        if row % 2 == 0 {
            let idx = (row / 2) as usize;
            write_note(&mut data, brass_counter[idx], BRASS);
        } else {
            write_empty(&mut data);
        }

        // Ch7: Pad - full chords
        if row == 0 {
            write_note(&mut data, D4_E, PAD);
        } else if row == 8 {
            write_note(&mut data, C4_E, PAD);
        } else if row == 16 {
            write_note(&mut data, BB3_E, PAD);
        } else if row == 24 {
            write_note(&mut data, C4_E, PAD);
        } else {
            write_empty(&mut data);
        }

        // Ch8: Harmony - octave below hook (also 16ths)
        write_note(&mut data, melody[row as usize].saturating_sub(12), SUPERSAW);
    }

    data
}

/// Eurobeat Pattern 5: Chorus B - Picardy third! Triumphant resolution
fn generate_euro_pattern_chorus_b() -> Vec<u8> {
    let mut data = Vec::new();

    // Bass on 8th notes: Dm -> F -> C -> D MAJOR (Picardy!)
    let bass_pattern: [(u8, u8); 16] = [
        (D2_E, D3_E), (D2_E, D3_E), (D2_E, D3_E), (D2_E, D3_E),
        (F2_E, F3_E), (F2_E, F3_E), (F2_E, F3_E), (F2_E, F3_E),
        (C3_E, C4_E), (C3_E, C4_E), (C3_E, C4_E), (C3_E, C4_E),
        (D3_E, D4_E), (D3_E, D4_E), (D3_E, D4_E), (D3_E, D4_E),
    ];

    // Triumphant melody on 8th notes
    let melody: [u8; 16] = [
        D5_E, F5_E, A5_E, D5_E, // Dm
        F5_E, A5_E, C5_E, F5_E, // F major
        C5_E, E5_E, G5_E, C5_E, // C major
        D5_E, FS4_E, A4_E, D5_E, // D MAJOR! (F# = Picardy)
    ];

    // Main brass riff on 8th notes
    let brass_riff: [u8; 16] = [
        A4_E, D5_E, F5_E, A5_E, A4_E, C5_E, F5_E, A5_E,
        G4_E, C5_E, E5_E, G5_E, FS4_E, A4_E, D5_E, FS4_E + 12,
    ];

    for row in 0..32 {
        // Ch1: Kick
        if row % 4 == 0 {
            write_note(&mut data, C4_E, KICK_E);
        } else {
            write_empty(&mut data);
        }

        // Ch2: Snare
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

        // Ch5: Lead melody on 8th notes
        if row % 2 == 0 {
            let idx = (row / 2) as usize;
            write_note(&mut data, melody[idx], SUPERSAW);
        } else {
            write_empty(&mut data);
        }

        // Ch6: Brass riff on 8th notes
        if row % 2 == 0 {
            let idx = (row / 2) as usize;
            write_note(&mut data, brass_riff[idx], BRASS);
        } else {
            write_empty(&mut data);
        }

        // Ch7: Triumphant pad
        if row == 0 {
            write_note(&mut data, F4_E, PAD);
        } else if row == 8 {
            write_note(&mut data, A4_E, PAD);
        } else if row == 16 {
            write_note(&mut data, G4_E, PAD);
        } else if row == 24 {
            write_note(&mut data, FS4_E, PAD); // F# for D major!
        } else {
            write_empty(&mut data);
        }

        // Ch8: Harmony on 8th notes
        if row % 2 == 0 {
            let idx = (row / 2) as usize;
            let mel = melody[idx];
            write_note(&mut data, mel.saturating_sub(12), SUPERSAW);
        } else {
            write_empty(&mut data);
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

/// Eurobeat Pattern 7: Drop - MAXIMUM ENERGY
fn generate_euro_pattern_drop() -> Vec<u8> {
    let mut data = Vec::new();

    // Bass on 8th notes: Dm -> C -> Bb -> A (harmonic minor climax!)
    let bass_pattern: [(u8, u8); 16] = [
        (D2_E, D3_E), (D2_E, D3_E), (D2_E, D3_E), (D2_E, D3_E),
        (C3_E, C4_E), (C3_E, C4_E), (C3_E, C4_E), (C3_E, C4_E),
        (BB2_E, BB3_E), (BB2_E, BB3_E), (BB2_E, BB3_E), (BB2_E, BB3_E),
        (A2_E, A3_E), (A2_E, A3_E), (A2_E, A3_E), (A2_E, A3_E),
    ];

    // Fast arpeggios - every row for maximum energy!
    let melody = [
        D5_E, F5_E, A5_E, D5_E, F5_E, A5_E, D5_E, F5_E, // Dm ultra fast
        C5_E, E5_E, G5_E, C5_E, E5_E, G5_E, C5_E, E5_E, // C ultra fast
        BB4_E, D5_E, F5_E, BB4_E, D5_E, F5_E, BB4_E, D5_E, // Bb ultra fast
        A4_E, C5_E, E5_E, A5_E, E5_E, C5_E, A4_E, A5_E, // A (harmonic minor) climax!
    ];

    // Brass riff on 8th notes
    let brass_notes: [u8; 16] = [
        F4_E, A4_E, D5_E, F5_E, E4_E, G4_E, C5_E, E5_E,
        D4_E, F4_E, BB4_E, D5_E, C4_E, E4_E, A4_E, C5_E,
    ];

    for row in 0..32 {
        // Ch1: Kick - double time feel
        if row % 2 == 0 {
            write_note(&mut data, C4_E, KICK_E);
        } else {
            write_empty(&mut data);
        }

        // Ch2: Snare - on 2 and 4 plus extra hits
        if row % 8 == 4 {
            write_note(&mut data, C4_E, SNARE_E);
        } else if row % 4 == 2 {
            write_note_vol(&mut data, C4_E, SNARE_E, 0x30);
        } else {
            write_empty(&mut data);
        }

        // Ch3: Hi-hat - full 16ths for energy
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

        // Ch5: SUPERSAW - every single row for maximum energy!
        write_note(&mut data, melody[row as usize], SUPERSAW);

        // Ch6: Brass riff on 8th notes
        if row % 2 == 0 {
            write_note(&mut data, brass_notes[(row / 2) as usize], BRASS);
        } else {
            write_empty(&mut data);
        }

        // Ch7: Pad - full chords
        if row == 0 {
            write_note(&mut data, D4_E, PAD);
        } else if row == 8 {
            write_note(&mut data, C4_E, PAD);
        } else if row == 16 {
            write_note(&mut data, BB3_E, PAD);
        } else if row == 24 {
            write_note(&mut data, A3_E, PAD);
        } else {
            write_empty(&mut data);
        }

        // Ch8: Harmony - 5th above (16ths for energy)
        let harm_note = melody[row as usize] + 7; // Perfect 5th up
        write_note(&mut data, harm_note.min(96), SUPERSAW);
    }

    data
}

/// Write a minimal instrument header (sample-less)
fn write_instrument(xm: &mut Vec<u8>, name: &str) {
    // Header size includes the 4-byte field itself: 4 + 22 name + 1 type + 2 num_samples = 29
    // Parser seeks to: header_start + header_size - 4
    let header_size: u32 = 29;
    xm.extend_from_slice(&header_size.to_le_bytes());

    let name_bytes = name.as_bytes();
    xm.extend_from_slice(&name_bytes[..name_bytes.len().min(22)]);
    xm.extend(std::iter::repeat(0u8).take(22 - name_bytes.len().min(22)));

    xm.push(0); // instrument type
    xm.extend_from_slice(&0u16.to_le_bytes()); // num samples = 0
}

/// Downsample from 22050 Hz to 8363 Hz (Amiga base rate)
fn downsample_to_8363(samples: &[i16]) -> Vec<i16> {
    let ratio = 22050.0 / 8363.0; // ~2.636
    let new_len = (samples.len() as f32 / ratio) as usize;
    let mut resampled = Vec::with_capacity(new_len);

    for i in 0..new_len {
        let src_pos = i as f32 * ratio;
        let src_idx = src_pos as usize;

        // Linear interpolation
        if src_idx + 1 < samples.len() {
            let frac = src_pos - src_idx as f32;
            let s0 = samples[src_idx] as f32;
            let s1 = samples[src_idx + 1] as f32;
            let interpolated = s0 + (s1 - s0) * frac;
            resampled.push(interpolated.round() as i16);
        } else if src_idx < samples.len() {
            resampled.push(samples[src_idx]);
        }
    }

    resampled
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

    // Finetune: +101 for precise 22050 Hz
    // 22050/8363 = 2.637 → log2(2.637) = 1.399 → 1.399*12 = 16.787 semitones
    // Fractional: 0.787 * 128 = 100.74 ≈ 101
    xm.push(101);

    // Type (0x10 = 16-bit)
    xm.push(0x10);

    // Panning (128 = center)
    xm.push(128);

    // Relative note: +16 (integer part of 16.787 semitones)
    // Formula: 8363 * 2^((16 + 101/128) / 12) ≈ 22050 Hz
    xm.push(16);

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

/// Synthwave kick: 808-style with longer decay, warm and round
fn generate_kick_synth() -> Vec<i16> {
    let duration = 0.4; // 400ms for that 80s thump
    let samples = (SAMPLE_RATE * duration) as usize;

    let mut output = Vec::with_capacity(samples);
    let mut phase = 0.0f32;

    for i in 0..samples {
        let t = i as f32 / SAMPLE_RATE;

        // Longer decay for warmth
        let decay = (-t * 8.0).exp();

        // Pitch sweep: 150Hz down to 45Hz - slower than eurobeat
        let freq = 150.0 * (-t * 15.0).exp() + 45.0;

        phase += 2.0 * PI * freq / SAMPLE_RATE;

        // Sine with soft saturation for analog warmth
        let mut sample = phase.sin() * decay;
        sample = (sample * 1.1).tanh(); // Gentle soft clip

        output.push((sample * 30000.0).clamp(-32767.0, 32767.0) as i16);
    }

    output
}

/// Synthwave snare: gated reverb style - short burst with abrupt cutoff
fn generate_snare_synth() -> Vec<i16> {
    let duration = 0.18; // 180ms - gated feel
    let samples = (SAMPLE_RATE * duration) as usize;

    let mut output = Vec::with_capacity(samples);
    let mut rng = SimpleRng::new(88888);

    for i in 0..samples {
        let t = i as f32 / SAMPLE_RATE;

        // Gated reverb envelope: sustains then cuts abruptly
        let envelope = if t < 0.02 {
            t / 0.02 // Fast attack
        } else if t < 0.12 {
            1.0 - (t - 0.02) * 0.3 // Slight decay during sustain
        } else {
            // Abrupt gate cutoff (characteristic of 80s gated reverb)
            0.7 * (1.0 - ((t - 0.12) / 0.06)).max(0.0)
        };

        // Noise component
        let noise = rng.next_f32() * 2.0 - 1.0;

        // Body at 200Hz
        let body = (2.0 * PI * 200.0 * t).sin() * (-t * 20.0).exp();

        // Mix - more body for 80s feel
        let sample = (noise * 0.55 + body * 0.45) * envelope * 28000.0;
        output.push(sample.clamp(-32767.0, 32767.0) as i16);
    }

    output
}

/// Synthwave hi-hat: crisp but not harsh, medium decay
fn generate_hihat_synth() -> Vec<i16> {
    let duration = 0.1; // 100ms
    let samples = (SAMPLE_RATE * duration) as usize;

    let mut output = Vec::with_capacity(samples);
    let mut rng = SimpleRng::new(66666);

    let mut hp_prev_in = 0.0f32;
    let mut hp_prev_out = 0.0f32;
    let mut lp_state = 0.0f32;

    for i in 0..samples {
        let t = i as f32 / SAMPLE_RATE;

        // Medium decay
        let decay = (-t * 35.0).exp();

        let noise = rng.next_f32() * 2.0 - 1.0;

        // High-pass
        let hp_alpha = 0.9;
        let hp_out = hp_alpha * (hp_prev_out + noise - hp_prev_in);
        hp_prev_in = noise;
        hp_prev_out = hp_out;

        // Gentle low-pass to take edge off
        lp_state += 0.5 * (hp_out - lp_state);

        let sample = lp_state * decay * 22000.0;
        output.push(sample.clamp(-32767.0, 32767.0) as i16);
    }

    output
}

/// Synthwave bass: pulsing sawtooth with filter envelope
/// Smooth, warm, drives the groove without harshness
fn generate_bass_synth() -> Vec<i16> {
    // From nether-drive.spec.md:
    // - Pulsing sawtooth with filter envelope
    // - Attack: 10ms (not instant - removes "pop")
    // - Filter sweep: 800Hz → 200Hz over 200ms
    // - Sustain longer for smooth groove
    // - Duration: 350ms (overlap slightly for legato feel)
    // - Sub oscillator: sine at -1 octave, 30% mix
    let duration = 0.35; // EXACT SPEC: 350ms
    let freq = 55.0; // A1 as base
    let samples = (SAMPLE_RATE * duration) as usize;

    let mut output = Vec::with_capacity(samples);
    let mut filtered = 0.0f32;

    for i in 0..samples {
        let t = i as f32 / SAMPLE_RATE;

        // EXACT SPEC: 10ms attack (linear for smoothness)
        let envelope = if t < 0.010 {
            t / 0.010 // 10ms linear attack
        } else if t < 0.25 {
            1.0 - (t - 0.01) * 0.05 // Gentle sustain decay
        } else {
            0.95 * (-(t - 0.25) * 6.0).exp() // Smooth release
        };

        // EXACT SPEC: Filter sweep 800Hz → 200Hz over 200ms
        // Convert to one-pole coefficient: coeff = 2 * pi * fc / sr (simplified)
        let filter_fc = if t < 0.2 {
            800.0 - (t / 0.2) * 600.0 // 800Hz → 200Hz over 200ms
        } else {
            200.0 // Sustain at 200Hz
        };
        let filter_coeff = (2.0 * PI * filter_fc / SAMPLE_RATE).min(0.99);

        // Sawtooth oscillator
        let phase = (freq * t) % 1.0;
        let saw = 2.0 * phase - 1.0;

        // EXACT SPEC: Sub oscillator sine at -1 octave, 30% mix
        let sub = (2.0 * PI * freq * t).sin() * 0.30;

        // Dynamic low-pass filter
        filtered += filter_coeff * (saw - filtered);

        let sample = (filtered * 0.70 + sub) * envelope * 28000.0;
        output.push(sample.clamp(-32767.0, 32767.0) as i16);
    }

    output
}

/// Synthwave lead: two detuned saws with vibrato, warm and soaring
fn generate_lead_synth() -> Vec<i16> {
    let duration = 0.9; // 900ms
    let freq = 220.0; // A3 as base
    let samples = (SAMPLE_RATE * duration) as usize;

    let mut output = Vec::with_capacity(samples);
    let mut filtered = 0.0f32;
    let mut phase1 = 0.0f32; // Proper phase accumulators
    let mut phase2 = 0.0f32;
    let mut vibrato_phase = 0.0f32;

    // Detune ratio for second oscillator (~12 cents)
    let detune = 1.007f32;

    for i in 0..samples {
        let t = i as f32 / SAMPLE_RATE;

        // Smooth ADSR envelope
        let envelope = if t < 0.02 {
            t / 0.02 // 20ms attack
        } else if t < 0.6 {
            1.0 - (t - 0.02) * 0.1
        } else {
            0.9 * (-(t - 0.6) * 3.0).exp()
        };

        // Delayed vibrato with proper phase accumulation
        let vibrato_amount = if t < 0.1 { 0.0 } else { 0.005 * ((t - 0.1) * 2.0).min(1.0) };
        vibrato_phase += 5.0 / SAMPLE_RATE;
        let vibrato = 1.0 + vibrato_amount * (vibrato_phase * 2.0 * PI).sin();

        // Two detuned saw oscillators with proper phase accumulation
        phase1 += freq * vibrato / SAMPLE_RATE;
        phase1 = phase1 % 1.0;
        phase2 += freq * vibrato * detune / SAMPLE_RATE;
        phase2 = phase2 % 1.0;

        let saw1 = 2.0 * phase1 - 1.0;
        let saw2 = 2.0 * phase2 - 1.0;

        // Mix the two saws
        let mix = (saw1 + saw2) * 0.5;

        // Warm low-pass filter
        filtered += 0.15 * (mix - filtered);

        let sample = filtered * envelope * 24000.0;
        output.push(sample.clamp(-32767.0, 32767.0) as i16);
    }

    output
}

/// Synthwave arpeggiator: plucky square wave for rhythmic sparkle
fn generate_arp_synth() -> Vec<i16> {
    let duration = 0.3; // 300ms
    let freq = 440.0; // A4 as base
    let samples = (SAMPLE_RATE * duration) as usize;

    let mut output = Vec::with_capacity(samples);
    let mut filtered = 0.0f32;

    for i in 0..samples {
        let t = i as f32 / SAMPLE_RATE;

        // Plucky envelope - fast attack, medium decay
        let envelope = if t < 0.003 {
            t / 0.003
        } else {
            (-(t - 0.003) * 8.0).exp()
        };

        // Square wave
        let phase = (freq * t) % 1.0;
        let square = if phase < 0.5 { 1.0 } else { -1.0 };

        // Filter envelope for pluck character
        let filter_env = 0.15 + 0.35 * (-t * 15.0).exp();
        filtered += filter_env * (square - filtered);

        let sample = filtered * envelope * 20000.0;
        output.push(sample.clamp(-32767.0, 32767.0) as i16);
    }

    output
}

/// Synthwave pad: lush, 3 detuned saws with slow attack
fn generate_pad_synth() -> Vec<i16> {
    let duration = 2.0; // 2 seconds for long sustain
    let freq = 220.0; // A3 as base
    let samples = (SAMPLE_RATE * duration) as usize;

    let mut output = Vec::with_capacity(samples);
    let mut filtered = 0.0f32;

    for i in 0..samples {
        let t = i as f32 / SAMPLE_RATE;

        // Very slow attack, long sustain for lush pad
        let envelope = if t < 0.3 {
            t / 0.3 // 300ms attack
        } else if t < 1.5 {
            1.0
        } else {
            (-(t - 1.5) * 2.0).exp()
        };

        // Three detuned saw oscillators for width
        let detune_amounts = [0.993, 1.0, 1.007];
        let mut sum = 0.0f32;
        for d in detune_amounts {
            let phase = (freq * d * t) % 1.0;
            sum += 2.0 * phase - 1.0;
        }
        sum /= 3.0;

        // Warm low-pass filter
        filtered += 0.05 * (sum - filtered);

        let sample = filtered * envelope * 18000.0;
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
