//! Generates synthwave FM music for NEON DRIFT racing game
//!
//! Creates 5 unique tracker files with retro FM synthesis sounds:
//! - sunset_strip.xm - Chill, cruising vibes (beginner track)
//! - neon_city.xm - Energetic urban synthwave (intermediate)
//! - void_tunnel.xm - Dark, pulsing electronic (advanced)
//! - crystal_cavern.xm - Mystical, shimmering sounds (hard)
//! - solar_highway.xm - Triumphant, driving anthem (expert)
//!
//! All sounds use FM synthesis for authentic retro synthwave aesthetic.

use proc_gen::audio::SAMPLE_RATE;
use std::f32::consts::PI;
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;

/// Fade-in duration in seconds (prevents clicks)
const FADE_IN_SECS: f32 = 0.002;

/// Fade-out duration in seconds (prevents clicks)
const FADE_OUT_SECS: f32 = 0.005;

/// Apply fade-in and fade-out to prevent clicks
fn apply_fades(samples: &mut [i16]) {
    let fade_in_samples = (SAMPLE_RATE as f32 * FADE_IN_SECS) as usize;
    let fade_out_samples = (SAMPLE_RATE as f32 * FADE_OUT_SECS) as usize;

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
    let output_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("examples")
        .join("7-games")
        .join("neon-drift")
        .join("assets")
        .join("audio");

    // Create output directories
    let samples_dir = output_dir.clone();
    let music_dir = output_dir.join("music");
    fs::create_dir_all(&samples_dir).expect("Failed to create samples directory");
    fs::create_dir_all(&music_dir).expect("Failed to create music directory");

    println!("Generating NEON DRIFT synthwave music to {:?}", output_dir);

    // ========================================================================
    // Generate FM Synthwave Samples
    // ========================================================================

    println!("\n=== Generating FM Synthwave Samples ===");

    // Kick drum - punchy FM with pitch sweep
    let mut kick = generate_fm_kick();
    apply_fades(&mut kick);
    write_wav_file(&samples_dir.join("synth_kick.wav"), &kick);
    println!("  Generated synth_kick.wav ({} samples)", kick.len());

    // Snare - FM with noise layer
    let mut snare = generate_fm_snare();
    apply_fades(&mut snare);
    write_wav_file(&samples_dir.join("synth_snare.wav"), &snare);
    println!("  Generated synth_snare.wav ({} samples)", snare.len());

    // Hi-hat - metallic FM
    let mut hihat = generate_fm_hihat();
    apply_fades(&mut hihat);
    write_wav_file(&samples_dir.join("synth_hihat.wav"), &hihat);
    println!("  Generated synth_hihat.wav ({} samples)", hihat.len());

    // Open hi-hat
    let mut openhat = generate_fm_openhat();
    apply_fades(&mut openhat);
    write_wav_file(&samples_dir.join("synth_openhat.wav"), &openhat);
    println!("  Generated synth_openhat.wav ({} samples)", openhat.len());

    // Bass - deep FM bass
    let mut bass = generate_fm_synth_bass();
    apply_fades(&mut bass);
    write_wav_file(&samples_dir.join("synth_bass.wav"), &bass);
    println!("  Generated synth_bass.wav ({} samples)", bass.len());

    // Lead - synthwave lead
    let mut lead = generate_fm_synth_lead();
    apply_fades(&mut lead);
    write_wav_file(&samples_dir.join("synth_lead.wav"), &lead);
    println!("  Generated synth_lead.wav ({} samples)", lead.len());

    // Pad - lush FM pad
    let mut pad = generate_fm_synth_pad();
    apply_fades(&mut pad);
    write_wav_file(&samples_dir.join("synth_pad.wav"), &pad);
    println!("  Generated synth_pad.wav ({} samples)", pad.len());

    // Arp - arpeggiated synth
    let mut arp = generate_fm_arp();
    apply_fades(&mut arp);
    write_wav_file(&samples_dir.join("synth_arp.wav"), &arp);
    println!("  Generated synth_arp.wav ({} samples)", arp.len());

    // ========================================================================
    // Generate XM Tracker Files (5 unique tracks)
    // ========================================================================

    println!("\n=== Generating XM Tracker Files ===");

    // Track 1: Sunset Strip - chill, cruising
    let xm = generate_sunset_strip();
    fs::write(music_dir.join("sunset_strip.xm"), &xm).expect("Failed to write sunset_strip.xm");
    println!("  Generated sunset_strip.xm ({} bytes)", xm.len());

    // Track 2: Neon City - energetic urban
    let xm = generate_neon_city();
    fs::write(music_dir.join("neon_city.xm"), &xm).expect("Failed to write neon_city.xm");
    println!("  Generated neon_city.xm ({} bytes)", xm.len());

    // Track 3: Void Tunnel - dark, pulsing
    let xm = generate_void_tunnel();
    fs::write(music_dir.join("void_tunnel.xm"), &xm).expect("Failed to write void_tunnel.xm");
    println!("  Generated void_tunnel.xm ({} bytes)", xm.len());

    // Track 4: Crystal Cavern - mystical, shimmering
    let xm = generate_crystal_cavern();
    fs::write(music_dir.join("crystal_cavern.xm"), &xm).expect("Failed to write crystal_cavern.xm");
    println!("  Generated crystal_cavern.xm ({} bytes)", xm.len());

    // Track 5: Solar Highway - triumphant, driving
    let xm = generate_solar_highway();
    fs::write(music_dir.join("solar_highway.xm"), &xm).expect("Failed to write solar_highway.xm");
    println!("  Generated solar_highway.xm ({} bytes)", xm.len());

    println!("\nDone! All NEON DRIFT music generated.");
}

// ============================================================================
// FM SYNTHWAVE SAMPLE GENERATORS
// ============================================================================

/// FM kick drum - sine wave with pitch sweep and click transient
fn generate_fm_kick() -> Vec<i16> {
    let duration = 0.35;
    let num_samples = (SAMPLE_RATE as f32 * duration) as usize;
    let mut samples = Vec::with_capacity(num_samples);

    let mut phase = 0.0f32;

    for i in 0..num_samples {
        let t = i as f32 / SAMPLE_RATE as f32;

        // Exponential decay
        let decay = (-t * 12.0).exp();

        // Pitch sweep from 180Hz to 45Hz
        let freq = 180.0 * (-t * 25.0).exp() + 45.0;

        // Phase accumulation for smooth sweep
        phase += 2.0 * PI * freq / SAMPLE_RATE as f32;

        // Main sine body
        let body = phase.sin();

        // FM modulator for punch (quick attack click)
        let click = if t < 0.015 {
            (phase * 4.0).sin() * (1.0 - t / 0.015)
        } else {
            0.0
        };

        let sample = (body + click * 0.3) * decay * 30000.0;
        samples.push(sample.clamp(-32767.0, 32767.0) as i16);
    }

    samples
}

/// FM snare - noise + FM body for punchy synthwave snare
fn generate_fm_snare() -> Vec<i16> {
    let duration = 0.25;
    let num_samples = (SAMPLE_RATE as f32 * duration) as usize;
    let mut samples = Vec::with_capacity(num_samples);

    let mut rng = SimpleRng::new(12345);
    let mut phase = 0.0f32;

    for i in 0..num_samples {
        let t = i as f32 / SAMPLE_RATE as f32;

        // Fast decay
        let decay = (-t * 18.0).exp();

        // Noise component (filtered)
        let noise = rng.next_f32() * 2.0 - 1.0;

        // FM body for tonal punch
        let freq = 200.0 * (-t * 30.0).exp() + 150.0;
        phase += 2.0 * PI * freq / SAMPLE_RATE as f32;

        // Carrier modulated by higher ratio
        let mod_signal = (phase * 2.3).sin();
        let body = (phase + 1.5 * mod_signal).sin();

        // Mix noise and FM body
        let sample = (noise * 0.55 + body * 0.45) * decay * 28000.0;
        samples.push(sample.clamp(-32767.0, 32767.0) as i16);
    }

    samples
}

/// FM hi-hat - metallic FM synthesis
fn generate_fm_hihat() -> Vec<i16> {
    let duration = 0.08;
    let num_samples = (SAMPLE_RATE as f32 * duration) as usize;
    let mut samples = Vec::with_capacity(num_samples);

    for i in 0..num_samples {
        let t = i as f32 / SAMPLE_RATE as f32;
        let sample_t = i as f32;

        // Very fast decay
        let decay = (-t * 50.0).exp();

        // Metallic FM using non-integer ratios (inharmonic)
        let freq = 6000.0;
        let omega = 2.0 * PI * freq / SAMPLE_RATE as f32;

        let mod1 = (omega * 1.41 * sample_t).sin(); // sqrt(2) ratio
        let mod2 = (omega * 2.73 * sample_t).sin(); // e ratio
        let carrier = (omega * sample_t + 2.0 * mod1 + 1.5 * mod2).sin();

        let sample = carrier * decay * 18000.0;
        samples.push(sample.clamp(-32767.0, 32767.0) as i16);
    }

    samples
}

/// FM open hi-hat - longer decay metallic
fn generate_fm_openhat() -> Vec<i16> {
    let duration = 0.2;
    let num_samples = (SAMPLE_RATE as f32 * duration) as usize;
    let mut samples = Vec::with_capacity(num_samples);

    for i in 0..num_samples {
        let t = i as f32 / SAMPLE_RATE as f32;
        let sample_t = i as f32;

        // Slower decay
        let decay = (-t * 15.0).exp();

        // Metallic FM
        let freq = 5500.0;
        let omega = 2.0 * PI * freq / SAMPLE_RATE as f32;

        let mod1 = (omega * 1.41 * sample_t).sin();
        let mod2 = (omega * 2.73 * sample_t).sin();
        let mod3 = (omega * 3.14 * sample_t).sin();
        let carrier = (omega * sample_t + 1.8 * mod1 + 1.2 * mod2 + 0.8 * mod3).sin();

        let sample = carrier * decay * 16000.0;
        samples.push(sample.clamp(-32767.0, 32767.0) as i16);
    }

    samples
}

/// FM synth bass - deep, growling synthwave bass
fn generate_fm_synth_bass() -> Vec<i16> {
    let duration = 0.6;
    let base_freq = 55.0; // A1
    let num_samples = (SAMPLE_RATE as f32 * duration) as usize;
    let mut samples = Vec::with_capacity(num_samples);

    let mut phase = 0.0f32;
    let mut mod_phase = 0.0f32;
    let mut prev_mod = 0.0f32;

    let carrier_omega = 2.0 * PI * base_freq / SAMPLE_RATE as f32;
    let mod_omega = 2.0 * PI * base_freq / SAMPLE_RATE as f32;

    for i in 0..num_samples {
        let t = i as f32 / SAMPLE_RATE as f32;

        // ADSR envelope
        let env = if t < 0.01 {
            t / 0.01
        } else if t < 0.4 {
            1.0 - (t - 0.01) * 0.15
        } else {
            0.85 * (-(t - 0.4) * 3.0).exp()
        };

        // Modulation index decay for evolving sound
        let mod_index = 3.0 - t * 2.0;

        // Modulator with feedback for growl
        mod_phase += mod_omega;
        let feedback = prev_mod * 0.4;
        let modulator = (mod_phase + feedback).sin();
        prev_mod = modulator;

        // Carrier
        phase += carrier_omega;
        let carrier = (phase + mod_index * modulator).sin();

        // Add sub-octave for weight
        let sub = (phase * 0.5).sin() * 0.3;

        let sample = (carrier + sub) * env * 26000.0;
        samples.push(sample.clamp(-32767.0, 32767.0) as i16);
    }

    samples
}

/// FM synth lead - bright, cutting lead sound
fn generate_fm_synth_lead() -> Vec<i16> {
    let duration = 0.7;
    let base_freq = 440.0; // A4
    let num_samples = (SAMPLE_RATE as f32 * duration) as usize;
    let mut samples = Vec::with_capacity(num_samples);

    let vibrato_rate = 5.5;
    let vibrato_omega = 2.0 * PI * vibrato_rate / SAMPLE_RATE as f32;

    for i in 0..num_samples {
        let t = i as f32 / SAMPLE_RATE as f32;
        let sample_t = i as f32;

        // ADSR envelope
        let env = if t < 0.02 {
            t / 0.02
        } else if t < 0.45 {
            1.0 - (t - 0.02) * 0.2
        } else {
            0.8 * (-(t - 0.45) * 2.5).exp()
        };

        // Delayed vibrato
        let vibrato_depth = if t < 0.1 { 0.0 } else { 0.012 * (t - 0.1).min(1.0) };
        let vibrato = 1.0 + vibrato_depth * (vibrato_omega * sample_t).sin();

        let freq = base_freq * vibrato;
        let omega = 2.0 * PI * freq / SAMPLE_RATE as f32;

        // 2-op FM with slight detune for thickness
        let mod1 = (omega * sample_t).sin();
        let mod2 = (omega * 1.003 * sample_t).sin(); // Slight detune
        let carrier1 = (omega * sample_t + 2.5 * mod1).sin();
        let carrier2 = (omega * 0.997 * sample_t + 2.3 * mod2).sin();

        let sample = (carrier1 * 0.6 + carrier2 * 0.4) * env * 24000.0;
        samples.push(sample.clamp(-32767.0, 32767.0) as i16);
    }

    samples
}

/// FM synth pad - lush, evolving pad
fn generate_fm_synth_pad() -> Vec<i16> {
    let duration = 1.2;
    let base_freq = 220.0; // A3
    let num_samples = (SAMPLE_RATE as f32 * duration) as usize;
    let mut samples = Vec::with_capacity(num_samples);

    for i in 0..num_samples {
        let t = i as f32 / SAMPLE_RATE as f32;
        let sample_t = i as f32;

        // Slow ADSR for pad
        let env = if t < 0.2 {
            t / 0.2
        } else if t < 0.8 {
            1.0
        } else {
            (-(t - 0.8) * 2.5).exp()
        };

        // Multiple detuned oscillators
        let omega = 2.0 * PI * base_freq / SAMPLE_RATE as f32;

        // Slow LFO for modulation depth variation
        let lfo = (0.3 * 2.0 * PI * t).sin() * 0.3 + 1.0;

        let osc1 = (omega * sample_t).sin();
        let osc2 = (omega * 1.005 * sample_t).sin();
        let osc3 = (omega * 0.995 * sample_t).sin();
        let osc4 = (omega * 2.0 * sample_t + lfo * osc1).sin(); // FM modulated

        let mix = (osc1 + osc2 + osc3) * 0.2 + osc4 * 0.4;

        let sample = mix * env * 22000.0;
        samples.push(sample.clamp(-32767.0, 32767.0) as i16);
    }

    samples
}

/// FM arp - short, punchy arp sound
fn generate_fm_arp() -> Vec<i16> {
    let duration = 0.15;
    let base_freq = 523.25; // C5
    let num_samples = (SAMPLE_RATE as f32 * duration) as usize;
    let mut samples = Vec::with_capacity(num_samples);

    for i in 0..num_samples {
        let t = i as f32 / SAMPLE_RATE as f32;
        let sample_t = i as f32;

        // Fast decay
        let env = if t < 0.005 {
            t / 0.005
        } else {
            (-(t - 0.005) * 15.0).exp()
        };

        let omega = 2.0 * PI * base_freq / SAMPLE_RATE as f32;

        // Sharp FM for plucky sound
        let mod_index = 3.0 * (-(t) * 20.0).exp();
        let modulator = (omega * 2.0 * sample_t).sin();
        let carrier = (omega * sample_t + mod_index * modulator).sin();

        let sample = carrier * env * 24000.0;
        samples.push(sample.clamp(-32767.0, 32767.0) as i16);
    }

    samples
}

// ============================================================================
// XM TRACKER GENERATION
// ============================================================================

// XM note values
#[allow(dead_code)]
const REST: u8 = 0;
const C2: u8 = 25;
const CS2: u8 = 26;
const D2: u8 = 27;
const DS2: u8 = 28;
const E2: u8 = 29;
const F2: u8 = 30;
const FS2: u8 = 31;
const G2: u8 = 32;
const GS2: u8 = 33;
const A2: u8 = 34;
const AS2: u8 = 35;
const B2: u8 = 36;

const C3: u8 = 37;
const CS3: u8 = 38;
const D3: u8 = 39;
const DS3: u8 = 40;
const E3: u8 = 41;
const F3: u8 = 42;
const FS3: u8 = 43;
const G3: u8 = 44;
const GS3: u8 = 45;
const A3: u8 = 46;
const AS3: u8 = 47;
const B3: u8 = 48;

const C4: u8 = 49;
const CS4: u8 = 50;
const D4: u8 = 51;
const DS4: u8 = 52;
const E4: u8 = 53;
const F4: u8 = 54;
const FS4: u8 = 55;
const G4: u8 = 56;
const GS4: u8 = 57;
const A4: u8 = 58;
const AS4: u8 = 59;
const B4: u8 = 60;

const C5: u8 = 61;
const CS5: u8 = 62;
const D5: u8 = 63;
const DS5: u8 = 64;
const E5: u8 = 65;
const F5: u8 = 66;
const FS5: u8 = 67;
const G5: u8 = 68;
const GS5: u8 = 69;
const A5: u8 = 70;

// Instruments
const KICK: u8 = 1;
const SNARE: u8 = 2;
const HIHAT: u8 = 3;
const OPENHAT: u8 = 4;
const BASS: u8 = 5;
const LEAD: u8 = 6;
const PAD: u8 = 7;
const ARP: u8 = 8;

/// Write XM header
fn write_xm_header(xm: &mut Vec<u8>, name: &str, num_patterns: u16, num_orders: u16, bpm: u16, speed: u16, order_table: &[u8]) {
    // ID text (17 bytes)
    xm.extend_from_slice(b"Extended Module: ");

    // Module name (20 bytes, null-padded)
    let name_bytes = name.as_bytes();
    xm.extend_from_slice(&name_bytes[..name_bytes.len().min(20)]);
    xm.extend(std::iter::repeat(0u8).take(20 - name_bytes.len().min(20)));

    // 0x1A marker
    xm.push(0x1A);

    // Tracker name (20 bytes)
    let tracker = b"NEON DRIFT SYNTH";
    xm.extend_from_slice(tracker);
    xm.extend(std::iter::repeat(0u8).take(20 - tracker.len()));

    // Version (0x0104)
    xm.extend_from_slice(&0x0104u16.to_le_bytes());

    // Header size (16 bytes header fields + 256 bytes order table = 272)
    xm.extend_from_slice(&272u32.to_le_bytes());

    // Song length (number of orders)
    xm.extend_from_slice(&num_orders.to_le_bytes());

    // Restart position
    xm.extend_from_slice(&1u16.to_le_bytes());

    // Number of channels (8)
    xm.extend_from_slice(&8u16.to_le_bytes());

    // Number of patterns
    xm.extend_from_slice(&num_patterns.to_le_bytes());

    // Number of instruments (8)
    xm.extend_from_slice(&8u16.to_le_bytes());

    // Flags (linear frequency table)
    xm.extend_from_slice(&1u16.to_le_bytes());

    // Default speed
    xm.extend_from_slice(&speed.to_le_bytes());

    // Default BPM
    xm.extend_from_slice(&bpm.to_le_bytes());

    // Pattern order table (256 bytes)
    xm.extend_from_slice(&order_table[..order_table.len().min(256)]);
    xm.extend(std::iter::repeat(0u8).take(256 - order_table.len().min(256)));
}

/// Write pattern to XM
fn write_pattern(xm: &mut Vec<u8>, pattern_data: &[u8], num_rows: u16) {
    let data_size = pattern_data.len() as u16;

    // XM pattern header: 5 bytes (4 for header length field + 1 for packing type)
    // Note: header length value doesn't include itself
    xm.extend_from_slice(&5u32.to_le_bytes()); // header length (not including this field)
    xm.push(0); // packing type
    xm.extend_from_slice(&num_rows.to_le_bytes());
    xm.extend_from_slice(&data_size.to_le_bytes());
    xm.extend_from_slice(pattern_data);
}

/// Write minimal instrument header
fn write_instrument(xm: &mut Vec<u8>, name: &str) {
    let header_size: u32 = 29;
    xm.extend_from_slice(&header_size.to_le_bytes());

    let name_bytes = name.as_bytes();
    xm.extend_from_slice(&name_bytes[..name_bytes.len().min(22)]);
    xm.extend(std::iter::repeat(0u8).take(22 - name_bytes.len().min(22)));

    xm.push(0); // Instrument type
    xm.extend_from_slice(&0u16.to_le_bytes()); // Number of samples
}

/// Write all instruments for neon drift
fn write_neon_instruments(xm: &mut Vec<u8>) {
    let instruments = [
        "synth_kick",
        "synth_snare",
        "synth_hihat",
        "synth_openhat",
        "synth_bass",
        "synth_lead",
        "synth_pad",
        "synth_arp",
    ];

    for name in &instruments {
        write_instrument(xm, name);
    }
}

/// Helper to write a note
fn write_note(data: &mut Vec<u8>, note: u8, instrument: u8) {
    if note == 0 {
        data.push(0x80); // Empty
    } else {
        data.push(0x80 | 0x01 | 0x02); // Has note + instrument
        data.push(note);
        data.push(instrument);
    }
}

/// Helper to write empty channel
fn write_empty(data: &mut Vec<u8>) {
    data.push(0x80);
}

// ============================================================================
// TRACK 1: SUNSET STRIP - Chill, cruising (Am -> F -> C -> G)
// ============================================================================

fn generate_sunset_strip() -> Vec<u8> {
    let mut xm = Vec::new();

    let order_table: Vec<u8> = vec![0, 1, 1, 2, 1, 2, 1, 3];

    write_xm_header(&mut xm, "Sunset Strip", 4, 8, 110, 6, &order_table);

    // Pattern 0: Intro - sparse drums, bass comes in
    write_pattern(&mut xm, &sunset_pattern_intro(), 32);

    // Pattern 1: Main groove - four on floor with bass
    write_pattern(&mut xm, &sunset_pattern_main(), 32);

    // Pattern 2: Melody section
    write_pattern(&mut xm, &sunset_pattern_melody(), 32);

    // Pattern 3: Breakdown
    write_pattern(&mut xm, &sunset_pattern_breakdown(), 32);

    write_neon_instruments(&mut xm);

    xm
}

fn sunset_pattern_intro() -> Vec<u8> {
    let mut data = Vec::new();

    for row in 0..32u8 {
        // Kick - builds up
        if row >= 16 && (row % 8 == 0 || row % 8 == 4) {
            write_note(&mut data, C4, KICK);
        } else {
            write_empty(&mut data);
        }

        // Snare
        if row >= 24 && row % 8 == 4 {
            write_note(&mut data, C4, SNARE);
        } else {
            write_empty(&mut data);
        }

        // HiHat
        if row >= 16 && row % 4 == 0 {
            write_note(&mut data, C4, HIHAT);
        } else {
            write_empty(&mut data);
        }

        // Open hat
        write_empty(&mut data);

        // Bass - enters at row 24
        if row >= 24 {
            let bass_pattern = [A2, 0, A2, 0, A2, 0, G2, 0];
            let note = bass_pattern[(row - 24) as usize % 8];
            if note != 0 {
                write_note(&mut data, note, BASS);
            } else {
                write_empty(&mut data);
            }
        } else {
            write_empty(&mut data);
        }

        // Lead - empty
        write_empty(&mut data);

        // Pad - ambient swell
        if row == 0 {
            write_note(&mut data, A3, PAD);
        } else if row == 16 {
            write_note(&mut data, E3, PAD);
        } else {
            write_empty(&mut data);
        }

        // Arp - empty
        write_empty(&mut data);
    }

    data
}

fn sunset_pattern_main() -> Vec<u8> {
    let mut data = Vec::new();

    let bass_line = [
        A2, A2, 0, 0, A2, 0, 0, 0,
        F2, F2, 0, 0, F2, 0, 0, 0,
        C3, C3, 0, 0, C3, 0, 0, 0,
        G2, G2, 0, 0, G2, 0, 0, 0,
    ];

    for row in 0..32u8 {
        // Kick
        if row % 8 == 0 || row % 8 == 4 {
            write_note(&mut data, C4, KICK);
        } else {
            write_empty(&mut data);
        }

        // Snare
        if row % 8 == 4 {
            write_note(&mut data, C4, SNARE);
        } else {
            write_empty(&mut data);
        }

        // HiHat
        if row % 2 == 0 {
            write_note(&mut data, C4, HIHAT);
        } else {
            write_empty(&mut data);
        }

        // Open hat on offbeats
        if row % 8 == 6 {
            write_note(&mut data, C4, OPENHAT);
        } else {
            write_empty(&mut data);
        }

        // Bass
        let bass = bass_line[row as usize];
        if bass != 0 {
            write_note(&mut data, bass, BASS);
        } else {
            write_empty(&mut data);
        }

        // Lead - empty in main
        write_empty(&mut data);

        // Pad
        if row == 0 {
            write_note(&mut data, A3, PAD);
        } else if row == 8 {
            write_note(&mut data, F3, PAD);
        } else if row == 16 {
            write_note(&mut data, C3, PAD);
        } else if row == 24 {
            write_note(&mut data, G3, PAD);
        } else {
            write_empty(&mut data);
        }

        // Arp - empty
        write_empty(&mut data);
    }

    data
}

fn sunset_pattern_melody() -> Vec<u8> {
    let mut data = Vec::new();

    let bass_line = [
        A2, A2, 0, 0, A2, 0, 0, 0,
        F2, F2, 0, 0, F2, 0, 0, 0,
        C3, C3, 0, 0, C3, 0, 0, 0,
        G2, G2, 0, 0, G2, 0, 0, 0,
    ];

    let melody = [
        C4, 0, E4, 0, G4, 0, A4, 0,
        A4, 0, G4, 0, F4, 0, E4, 0,
        E4, 0, G4, 0, C5, 0, G4, 0,
        G4, 0, E4, 0, D4, 0, C4, 0,
    ];

    for row in 0..32u8 {
        // Kick
        if row % 8 == 0 || row % 8 == 4 {
            write_note(&mut data, C4, KICK);
        } else {
            write_empty(&mut data);
        }

        // Snare
        if row % 8 == 4 {
            write_note(&mut data, C4, SNARE);
        } else {
            write_empty(&mut data);
        }

        // HiHat
        if row % 4 == 0 {
            write_note(&mut data, C4, HIHAT);
        } else {
            write_empty(&mut data);
        }

        // Open hat
        write_empty(&mut data);

        // Bass
        let bass = bass_line[row as usize];
        if bass != 0 {
            write_note(&mut data, bass, BASS);
        } else {
            write_empty(&mut data);
        }

        // Lead melody
        let mel = melody[row as usize];
        if mel != 0 {
            write_note(&mut data, mel, LEAD);
        } else {
            write_empty(&mut data);
        }

        // Pad
        if row == 0 {
            write_note(&mut data, A3, PAD);
        } else if row == 16 {
            write_note(&mut data, C3, PAD);
        } else {
            write_empty(&mut data);
        }

        // Arp
        write_empty(&mut data);
    }

    data
}

fn sunset_pattern_breakdown() -> Vec<u8> {
    let mut data = Vec::new();

    for row in 0..32u8 {
        // Sparse kick
        if row % 8 == 0 {
            write_note(&mut data, C4, KICK);
        } else {
            write_empty(&mut data);
        }

        // Sparse snare
        if row == 12 || row == 28 {
            write_note(&mut data, C4, SNARE);
        } else {
            write_empty(&mut data);
        }

        // Minimal hihat
        if row >= 24 && row % 2 == 0 {
            write_note(&mut data, C4, HIHAT);
        } else {
            write_empty(&mut data);
        }

        write_empty(&mut data); // Open hat
        write_empty(&mut data); // Bass

        // Lead accent
        if row == 8 {
            write_note(&mut data, E4, LEAD);
        } else if row == 24 {
            write_note(&mut data, A4, LEAD);
        } else {
            write_empty(&mut data);
        }

        // Pad swell
        if row == 0 {
            write_note(&mut data, A3, PAD);
        } else {
            write_empty(&mut data);
        }

        write_empty(&mut data); // Arp
    }

    data
}

// ============================================================================
// TRACK 2: NEON CITY - Energetic, urban (Em progression)
// ============================================================================

fn generate_neon_city() -> Vec<u8> {
    let mut xm = Vec::new();

    let order_table: Vec<u8> = vec![0, 1, 1, 2, 1, 2, 3, 1];

    write_xm_header(&mut xm, "Neon City", 4, 8, 128, 6, &order_table);

    write_pattern(&mut xm, &neon_pattern_intro(), 32);
    write_pattern(&mut xm, &neon_pattern_main(), 32);
    write_pattern(&mut xm, &neon_pattern_melody(), 32);
    write_pattern(&mut xm, &neon_pattern_breakdown(), 32);

    write_neon_instruments(&mut xm);

    xm
}

fn neon_pattern_intro() -> Vec<u8> {
    let mut data = Vec::new();

    for row in 0..32u8 {
        // Kick builds
        if row >= 8 && (row % 4 == 0) {
            write_note(&mut data, C4, KICK);
        } else {
            write_empty(&mut data);
        }

        // Snare
        if row >= 16 && row % 8 == 4 {
            write_note(&mut data, C4, SNARE);
        } else {
            write_empty(&mut data);
        }

        // Hihat builds to 16ths
        if row < 16 {
            if row % 4 == 0 { write_note(&mut data, C4, HIHAT); }
            else { write_empty(&mut data); }
        } else {
            if row % 2 == 0 { write_note(&mut data, C4, HIHAT); }
            else { write_empty(&mut data); }
        }

        write_empty(&mut data); // Open hat

        // Bass pulse
        if row >= 16 {
            let bass_notes = [E2, 0, E2, 0, E2, 0, D2, 0, E2, 0, E2, 0, G2, 0, A2, 0];
            let bass = bass_notes[(row - 16) as usize];
            if bass != 0 { write_note(&mut data, bass, BASS); }
            else { write_empty(&mut data); }
        } else {
            write_empty(&mut data);
        }

        write_empty(&mut data); // Lead

        // Pad
        if row == 0 { write_note(&mut data, E3, PAD); }
        else { write_empty(&mut data); }

        // Arp starts
        if row >= 24 && row % 2 == 0 {
            let arp_notes = [E4, G4, B4, E5];
            write_note(&mut data, arp_notes[((row - 24) / 2) as usize % 4], ARP);
        } else {
            write_empty(&mut data);
        }
    }

    data
}

fn neon_pattern_main() -> Vec<u8> {
    let mut data = Vec::new();

    let bass_line = [
        E2, 0, E2, E2, 0, E2, 0, 0,
        D2, 0, D2, D2, 0, D2, 0, 0,
        C3, 0, C3, C3, 0, C3, 0, 0,
        B2, 0, B2, B2, 0, B2, A2, 0,
    ];

    for row in 0..32u8 {
        // Four on floor with offbeat
        if row % 4 == 0 || row % 4 == 2 {
            write_note(&mut data, C4, KICK);
        } else {
            write_empty(&mut data);
        }

        // Snare
        if row % 8 == 4 {
            write_note(&mut data, C4, SNARE);
        } else {
            write_empty(&mut data);
        }

        // Fast hihat
        write_note(&mut data, C4, HIHAT);

        // Open hat accent
        if row % 8 == 6 {
            write_note(&mut data, C4, OPENHAT);
        } else {
            write_empty(&mut data);
        }

        // Driving bass
        let bass = bass_line[row as usize];
        if bass != 0 {
            write_note(&mut data, bass, BASS);
        } else {
            write_empty(&mut data);
        }

        write_empty(&mut data); // Lead

        // Pad chords
        if row == 0 { write_note(&mut data, E3, PAD); }
        else if row == 8 { write_note(&mut data, D3, PAD); }
        else if row == 16 { write_note(&mut data, C3, PAD); }
        else if row == 24 { write_note(&mut data, B2, PAD); }
        else { write_empty(&mut data); }

        // Arp pattern
        if row % 2 == 0 {
            let arp_notes = [E4, G4, B4, E5, B4, G4, E4, D4,
                            E4, G4, B4, E5, B4, G4, E4, G4];
            write_note(&mut data, arp_notes[(row / 2) as usize], ARP);
        } else {
            write_empty(&mut data);
        }
    }

    data
}

fn neon_pattern_melody() -> Vec<u8> {
    let mut data = Vec::new();

    let bass_line = [
        E2, 0, E2, E2, 0, E2, 0, 0,
        D2, 0, D2, D2, 0, D2, 0, 0,
        C3, 0, C3, C3, 0, C3, 0, 0,
        B2, 0, B2, B2, 0, B2, A2, 0,
    ];

    let melody = [
        E4, 0, G4, 0, B4, 0, E5, 0,
        D5, 0, B4, 0, G4, 0, 0, 0,
        C5, 0, E5, 0, G5, 0, E5, 0,
        D5, 0, B4, 0, A4, 0, G4, 0,
    ];

    for row in 0..32u8 {
        if row % 4 == 0 {
            write_note(&mut data, C4, KICK);
        } else {
            write_empty(&mut data);
        }

        if row % 8 == 4 {
            write_note(&mut data, C4, SNARE);
        } else {
            write_empty(&mut data);
        }

        if row % 2 == 0 {
            write_note(&mut data, C4, HIHAT);
        } else {
            write_empty(&mut data);
        }

        write_empty(&mut data);

        let bass = bass_line[row as usize];
        if bass != 0 {
            write_note(&mut data, bass, BASS);
        } else {
            write_empty(&mut data);
        }

        let mel = melody[row as usize];
        if mel != 0 {
            write_note(&mut data, mel, LEAD);
        } else {
            write_empty(&mut data);
        }

        if row == 0 { write_note(&mut data, E3, PAD); }
        else if row == 16 { write_note(&mut data, C3, PAD); }
        else { write_empty(&mut data); }

        write_empty(&mut data);
    }

    data
}

fn neon_pattern_breakdown() -> Vec<u8> {
    let mut data = Vec::new();

    for row in 0..32u8 {
        if row >= 24 && row % 2 == 0 {
            write_note(&mut data, C4, KICK);
        } else {
            write_empty(&mut data);
        }

        if row == 28 || row == 30 {
            write_note(&mut data, C4, SNARE);
        } else {
            write_empty(&mut data);
        }

        if row >= 16 && row % 4 == 0 {
            write_note(&mut data, C4, HIHAT);
        } else {
            write_empty(&mut data);
        }

        write_empty(&mut data);
        write_empty(&mut data);

        if row == 24 {
            write_note(&mut data, E5, LEAD);
        } else {
            write_empty(&mut data);
        }

        if row == 0 {
            write_note(&mut data, E3, PAD);
        } else {
            write_empty(&mut data);
        }

        write_empty(&mut data);
    }

    data
}

// ============================================================================
// TRACK 3: VOID TUNNEL - Dark, pulsing (Dm progression)
// ============================================================================

fn generate_void_tunnel() -> Vec<u8> {
    let mut xm = Vec::new();

    let order_table: Vec<u8> = vec![0, 1, 2, 1, 2, 1, 3, 1];

    write_xm_header(&mut xm, "Void Tunnel", 4, 8, 135, 6, &order_table);

    write_pattern(&mut xm, &void_pattern_intro(), 32);
    write_pattern(&mut xm, &void_pattern_main(), 32);
    write_pattern(&mut xm, &void_pattern_intensity(), 32);
    write_pattern(&mut xm, &void_pattern_breakdown(), 32);

    write_neon_instruments(&mut xm);

    xm
}

fn void_pattern_intro() -> Vec<u8> {
    let mut data = Vec::new();

    for row in 0..32u8 {
        // Dark kick builds
        if row >= 16 && row % 4 == 0 {
            write_note(&mut data, C4, KICK);
        } else {
            write_empty(&mut data);
        }

        write_empty(&mut data); // Snare

        // Sparse hihat
        if row >= 24 && row % 4 == 0 {
            write_note(&mut data, C4, HIHAT);
        } else {
            write_empty(&mut data);
        }

        write_empty(&mut data);

        // Dark bass pulse
        if row >= 16 && row % 8 == 0 {
            write_note(&mut data, D2, BASS);
        } else {
            write_empty(&mut data);
        }

        write_empty(&mut data);

        // Ominous pad
        if row == 0 {
            write_note(&mut data, D3, PAD);
        } else if row == 16 {
            write_note(&mut data, A2, PAD);
        } else {
            write_empty(&mut data);
        }

        write_empty(&mut data);
    }

    data
}

fn void_pattern_main() -> Vec<u8> {
    let mut data = Vec::new();

    let bass_line = [
        D2, 0, D2, 0, D2, D2, 0, 0,
        A2, 0, A2, 0, AS2, 0, 0, 0,
        C3, 0, C3, 0, C3, C3, 0, 0,
        A2, 0, A2, 0, G2, 0, 0, 0,
    ];

    for row in 0..32u8 {
        // Heavy kick
        if row % 4 == 0 {
            write_note(&mut data, C4, KICK);
        } else {
            write_empty(&mut data);
        }

        // Clap/snare
        if row % 8 == 4 {
            write_note(&mut data, C4, SNARE);
        } else {
            write_empty(&mut data);
        }

        // Driving hihat
        write_note(&mut data, C4, HIHAT);

        // Open hat accent
        if row % 4 == 2 {
            write_note(&mut data, C4, OPENHAT);
        } else {
            write_empty(&mut data);
        }

        // Dark bass
        let bass = bass_line[row as usize];
        if bass != 0 {
            write_note(&mut data, bass, BASS);
        } else {
            write_empty(&mut data);
        }

        write_empty(&mut data);

        // Minor pad
        if row == 0 { write_note(&mut data, D3, PAD); }
        else if row == 16 { write_note(&mut data, A2, PAD); }
        else { write_empty(&mut data); }

        // Dark arp
        if row % 4 == 0 {
            let arp = [D4, F4, A4, D5, A4, F4, D4, A3];
            write_note(&mut data, arp[(row / 4) as usize], ARP);
        } else {
            write_empty(&mut data);
        }
    }

    data
}

fn void_pattern_intensity() -> Vec<u8> {
    let mut data = Vec::new();

    let bass_line = [
        D2, D2, 0, D2, 0, D2, D2, 0,
        AS2, AS2, 0, AS2, 0, AS2, AS2, 0,
        C3, C3, 0, C3, 0, C3, C3, 0,
        A2, A2, 0, A2, G2, 0, A2, 0,
    ];

    let melody = [
        D4, 0, F4, 0, A4, 0, D5, D5,
        0, C5, 0, AS4, 0, A4, 0, 0,
        C5, 0, E5, 0, G5, 0, C5, 0,
        A4, 0, 0, G4, 0, F4, D4, 0,
    ];

    for row in 0..32u8 {
        if row % 2 == 0 {
            write_note(&mut data, C4, KICK);
        } else {
            write_empty(&mut data);
        }

        if row % 8 == 4 || row % 8 == 6 {
            write_note(&mut data, C4, SNARE);
        } else {
            write_empty(&mut data);
        }

        write_note(&mut data, C4, HIHAT);

        if row % 8 == 2 {
            write_note(&mut data, C4, OPENHAT);
        } else {
            write_empty(&mut data);
        }

        let bass = bass_line[row as usize];
        if bass != 0 {
            write_note(&mut data, bass, BASS);
        } else {
            write_empty(&mut data);
        }

        let mel = melody[row as usize];
        if mel != 0 {
            write_note(&mut data, mel, LEAD);
        } else {
            write_empty(&mut data);
        }

        if row == 0 { write_note(&mut data, D3, PAD); }
        else { write_empty(&mut data); }

        write_empty(&mut data);
    }

    data
}

fn void_pattern_breakdown() -> Vec<u8> {
    let mut data = Vec::new();

    for row in 0..32u8 {
        if row >= 24 && row % 4 == 0 {
            write_note(&mut data, C4, KICK);
        } else {
            write_empty(&mut data);
        }

        if row == 28 {
            write_note(&mut data, C4, SNARE);
        } else {
            write_empty(&mut data);
        }

        write_empty(&mut data);
        write_empty(&mut data);

        // Rising bass
        if row >= 16 {
            let bass = [D2, 0, 0, 0, E2, 0, 0, 0, G2, 0, 0, 0, A2, 0, A2, A2];
            let b = bass[(row - 16) as usize];
            if b != 0 { write_note(&mut data, b, BASS); }
            else { write_empty(&mut data); }
        } else {
            write_empty(&mut data);
        }

        write_empty(&mut data);

        if row == 0 {
            write_note(&mut data, D3, PAD);
        } else {
            write_empty(&mut data);
        }

        write_empty(&mut data);
    }

    data
}

// ============================================================================
// TRACK 4: CRYSTAL CAVERN - Mystical, shimmering (F# minor)
// ============================================================================

fn generate_crystal_cavern() -> Vec<u8> {
    let mut xm = Vec::new();

    let order_table: Vec<u8> = vec![0, 1, 2, 1, 2, 3, 1, 2];

    write_xm_header(&mut xm, "Crystal Cavern", 4, 8, 118, 6, &order_table);

    write_pattern(&mut xm, &crystal_pattern_intro(), 32);
    write_pattern(&mut xm, &crystal_pattern_main(), 32);
    write_pattern(&mut xm, &crystal_pattern_melody(), 32);
    write_pattern(&mut xm, &crystal_pattern_breakdown(), 32);

    write_neon_instruments(&mut xm);

    xm
}

fn crystal_pattern_intro() -> Vec<u8> {
    let mut data = Vec::new();

    for row in 0..32u8 {
        // Sparse kick
        if row >= 24 && row % 8 == 0 {
            write_note(&mut data, C4, KICK);
        } else {
            write_empty(&mut data);
        }

        write_empty(&mut data);

        // Shimmery hihat
        if row >= 16 && row % 4 == 0 {
            write_note(&mut data, C4, HIHAT);
        } else {
            write_empty(&mut data);
        }

        write_empty(&mut data);
        write_empty(&mut data);
        write_empty(&mut data);

        // Mystical pad
        if row == 0 { write_note(&mut data, FS3, PAD); }
        else if row == 16 { write_note(&mut data, D3, PAD); }
        else { write_empty(&mut data); }

        // Crystal arp
        if row >= 8 && row % 2 == 0 {
            let arp = [FS4, A4, CS5, FS5, CS5, A4, FS4, D4,
                      FS4, A4, CS5, FS5];
            write_note(&mut data, arp[((row - 8) / 2) as usize % 12], ARP);
        } else {
            write_empty(&mut data);
        }
    }

    data
}

fn crystal_pattern_main() -> Vec<u8> {
    let mut data = Vec::new();

    let bass_line = [
        FS2, 0, FS2, 0, 0, FS2, 0, 0,
        D2, 0, D2, 0, 0, D2, 0, 0,
        A2, 0, A2, 0, 0, A2, 0, 0,
        E2, 0, E2, 0, 0, E2, FS2, 0,
    ];

    for row in 0..32u8 {
        if row % 8 == 0 || row % 8 == 4 {
            write_note(&mut data, C4, KICK);
        } else {
            write_empty(&mut data);
        }

        if row % 8 == 4 {
            write_note(&mut data, C4, SNARE);
        } else {
            write_empty(&mut data);
        }

        if row % 2 == 0 {
            write_note(&mut data, C4, HIHAT);
        } else {
            write_empty(&mut data);
        }

        if row % 8 == 6 {
            write_note(&mut data, C4, OPENHAT);
        } else {
            write_empty(&mut data);
        }

        let bass = bass_line[row as usize];
        if bass != 0 {
            write_note(&mut data, bass, BASS);
        } else {
            write_empty(&mut data);
        }

        write_empty(&mut data);

        if row == 0 { write_note(&mut data, FS3, PAD); }
        else if row == 8 { write_note(&mut data, D3, PAD); }
        else if row == 16 { write_note(&mut data, A3, PAD); }
        else if row == 24 { write_note(&mut data, E3, PAD); }
        else { write_empty(&mut data); }

        // Shimmering arp
        if row % 2 == 0 {
            let arp = [FS4, A4, CS5, FS5, A4, FS4, D4, FS4,
                      A4, D5, A4, FS4, E4, GS4, B4, E5];
            write_note(&mut data, arp[(row / 2) as usize], ARP);
        } else {
            write_empty(&mut data);
        }
    }

    data
}

fn crystal_pattern_melody() -> Vec<u8> {
    let mut data = Vec::new();

    let bass_line = [
        FS2, 0, FS2, 0, 0, FS2, 0, 0,
        D2, 0, D2, 0, 0, D2, 0, 0,
        A2, 0, A2, 0, 0, A2, 0, 0,
        E2, 0, E2, 0, 0, E2, FS2, 0,
    ];

    let melody = [
        FS4, 0, A4, 0, CS5, 0, FS5, 0,
        E5, 0, D5, 0, CS5, 0, A4, 0,
        D5, 0, FS5, 0, A5, 0, FS5, 0,
        E5, 0, CS5, 0, B4, 0, A4, 0,
    ];

    for row in 0..32u8 {
        if row % 8 == 0 || row % 8 == 4 {
            write_note(&mut data, C4, KICK);
        } else {
            write_empty(&mut data);
        }

        if row % 8 == 4 {
            write_note(&mut data, C4, SNARE);
        } else {
            write_empty(&mut data);
        }

        if row % 4 == 0 {
            write_note(&mut data, C4, HIHAT);
        } else {
            write_empty(&mut data);
        }

        write_empty(&mut data);

        let bass = bass_line[row as usize];
        if bass != 0 {
            write_note(&mut data, bass, BASS);
        } else {
            write_empty(&mut data);
        }

        let mel = melody[row as usize];
        if mel != 0 {
            write_note(&mut data, mel, LEAD);
        } else {
            write_empty(&mut data);
        }

        if row == 0 { write_note(&mut data, FS3, PAD); }
        else if row == 16 { write_note(&mut data, A3, PAD); }
        else { write_empty(&mut data); }

        write_empty(&mut data);
    }

    data
}

fn crystal_pattern_breakdown() -> Vec<u8> {
    let mut data = Vec::new();

    for row in 0..32u8 {
        if row >= 24 && row % 4 == 0 {
            write_note(&mut data, C4, KICK);
        } else {
            write_empty(&mut data);
        }

        write_empty(&mut data);

        if row >= 20 && row % 4 == 0 {
            write_note(&mut data, C4, HIHAT);
        } else {
            write_empty(&mut data);
        }

        write_empty(&mut data);
        write_empty(&mut data);
        write_empty(&mut data);

        if row == 0 { write_note(&mut data, FS3, PAD); }
        else { write_empty(&mut data); }

        // Sparse arp
        if row % 4 == 0 && row < 24 {
            let arp = [FS4, A4, CS5, FS5, A4, FS4];
            write_note(&mut data, arp[(row / 4) as usize % 6], ARP);
        } else {
            write_empty(&mut data);
        }
    }

    data
}

// ============================================================================
// TRACK 5: SOLAR HIGHWAY - Triumphant, driving (C major)
// ============================================================================

fn generate_solar_highway() -> Vec<u8> {
    let mut xm = Vec::new();

    let order_table: Vec<u8> = vec![0, 1, 1, 2, 1, 2, 3, 1];

    write_xm_header(&mut xm, "Solar Highway", 4, 8, 140, 6, &order_table);

    write_pattern(&mut xm, &solar_pattern_intro(), 32);
    write_pattern(&mut xm, &solar_pattern_main(), 32);
    write_pattern(&mut xm, &solar_pattern_melody(), 32);
    write_pattern(&mut xm, &solar_pattern_breakdown(), 32);

    write_neon_instruments(&mut xm);

    xm
}

fn solar_pattern_intro() -> Vec<u8> {
    let mut data = Vec::new();

    for row in 0..32u8 {
        // Building kick
        if row >= 8 && (row % 8 == 0 || (row >= 24 && row % 4 == 0)) {
            write_note(&mut data, C4, KICK);
        } else {
            write_empty(&mut data);
        }

        // Snare builds
        if row >= 24 && row % 8 == 4 {
            write_note(&mut data, C4, SNARE);
        } else {
            write_empty(&mut data);
        }

        // Hihat builds
        if row >= 16 && row % 2 == 0 {
            write_note(&mut data, C4, HIHAT);
        } else {
            write_empty(&mut data);
        }

        write_empty(&mut data);

        // Bass enters
        if row >= 16 {
            let bass = [C3, 0, C3, 0, C3, 0, G2, 0, C3, 0, C3, 0, E3, 0, G3, 0];
            let b = bass[(row - 16) as usize];
            if b != 0 { write_note(&mut data, b, BASS); }
            else { write_empty(&mut data); }
        } else {
            write_empty(&mut data);
        }

        write_empty(&mut data);

        // Triumphant pad
        if row == 0 { write_note(&mut data, C3, PAD); }
        else if row == 16 { write_note(&mut data, G3, PAD); }
        else { write_empty(&mut data); }

        // Rising arp
        if row >= 24 && row % 2 == 0 {
            let arp = [C4, E4, G4, C5];
            write_note(&mut data, arp[((row - 24) / 2) as usize], ARP);
        } else {
            write_empty(&mut data);
        }
    }

    data
}

fn solar_pattern_main() -> Vec<u8> {
    let mut data = Vec::new();

    let bass_line = [
        C3, 0, C3, C3, 0, C3, 0, 0,
        G2, 0, G2, G2, 0, G2, 0, 0,
        A2, 0, A2, A2, 0, A2, 0, 0,
        F2, 0, F2, F2, 0, G2, 0, 0,
    ];

    for row in 0..32u8 {
        // Driving kick
        if row % 4 == 0 || row % 4 == 2 {
            write_note(&mut data, C4, KICK);
        } else {
            write_empty(&mut data);
        }

        if row % 8 == 4 {
            write_note(&mut data, C4, SNARE);
        } else {
            write_empty(&mut data);
        }

        write_note(&mut data, C4, HIHAT);

        if row % 8 == 6 {
            write_note(&mut data, C4, OPENHAT);
        } else {
            write_empty(&mut data);
        }

        let bass = bass_line[row as usize];
        if bass != 0 {
            write_note(&mut data, bass, BASS);
        } else {
            write_empty(&mut data);
        }

        write_empty(&mut data);

        if row == 0 { write_note(&mut data, C3, PAD); }
        else if row == 8 { write_note(&mut data, G3, PAD); }
        else if row == 16 { write_note(&mut data, A3, PAD); }
        else if row == 24 { write_note(&mut data, F3, PAD); }
        else { write_empty(&mut data); }

        // Triumphant arp
        if row % 2 == 0 {
            let arp = [C4, E4, G4, C5, G4, E4, C4, G4,
                      A4, C5, E5, A5, E5, C5, A4, E4];
            write_note(&mut data, arp[(row / 2) as usize], ARP);
        } else {
            write_empty(&mut data);
        }
    }

    data
}

fn solar_pattern_melody() -> Vec<u8> {
    let mut data = Vec::new();

    let bass_line = [
        C3, 0, C3, C3, 0, C3, 0, 0,
        G2, 0, G2, G2, 0, G2, 0, 0,
        A2, 0, A2, A2, 0, A2, 0, 0,
        F2, 0, F2, F2, 0, G2, 0, 0,
    ];

    let melody = [
        C5, 0, E5, 0, G5, 0, C5 + 12, 0,
        G5, 0, E5, 0, D5, 0, C5, 0,
        A4, 0, C5, 0, E5, 0, A5, 0,
        G5, 0, F5, 0, E5, 0, D5, 0,
    ];

    for row in 0..32u8 {
        if row % 4 == 0 {
            write_note(&mut data, C4, KICK);
        } else {
            write_empty(&mut data);
        }

        if row % 8 == 4 {
            write_note(&mut data, C4, SNARE);
        } else {
            write_empty(&mut data);
        }

        if row % 2 == 0 {
            write_note(&mut data, C4, HIHAT);
        } else {
            write_empty(&mut data);
        }

        write_empty(&mut data);

        let bass = bass_line[row as usize];
        if bass != 0 {
            write_note(&mut data, bass, BASS);
        } else {
            write_empty(&mut data);
        }

        let mel = melody[row as usize];
        if mel != 0 {
            write_note(&mut data, mel, LEAD);
        } else {
            write_empty(&mut data);
        }

        if row == 0 { write_note(&mut data, C3, PAD); }
        else if row == 16 { write_note(&mut data, A3, PAD); }
        else { write_empty(&mut data); }

        write_empty(&mut data);
    }

    data
}

fn solar_pattern_breakdown() -> Vec<u8> {
    let mut data = Vec::new();

    for row in 0..32u8 {
        // Builds up
        if row >= 16 && row % 4 == 0 {
            write_note(&mut data, C4, KICK);
        } else if row >= 24 && row % 2 == 0 {
            write_note(&mut data, C4, KICK);
        } else {
            write_empty(&mut data);
        }

        if row == 28 || row == 30 {
            write_note(&mut data, C4, SNARE);
        } else {
            write_empty(&mut data);
        }

        if row >= 20 && row % 2 == 0 {
            write_note(&mut data, C4, HIHAT);
        } else {
            write_empty(&mut data);
        }

        write_empty(&mut data);

        // Rising bass
        if row >= 24 {
            let bass = [C3, 0, D3, 0, E3, 0, G3, C3];
            let b = bass[(row - 24) as usize];
            if b != 0 { write_note(&mut data, b, BASS); }
            else { write_empty(&mut data); }
        } else {
            write_empty(&mut data);
        }

        write_empty(&mut data);

        if row == 0 { write_note(&mut data, C3, PAD); }
        else { write_empty(&mut data); }

        // Building arp
        if row >= 16 && row % 4 == 0 {
            let arp = [C4, E4, G4, C5];
            write_note(&mut data, arp[((row - 16) / 4) as usize % 4], ARP);
        } else {
            write_empty(&mut data);
        }
    }

    data
}

// ============================================================================
// UTILITIES
// ============================================================================

/// Simple PRNG
struct SimpleRng {
    state: u32,
}

impl SimpleRng {
    fn new(seed: u32) -> Self {
        Self { state: if seed == 0 { 1 } else { seed } }
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

/// Write WAV file
fn write_wav_file(path: &Path, samples: &[i16]) {
    let mut file = File::create(path).expect("Failed to create WAV file");
    let data_size = (samples.len() * 2) as u32;

    // RIFF header
    file.write_all(b"RIFF").unwrap();
    file.write_all(&(36 + data_size).to_le_bytes()).unwrap();
    file.write_all(b"WAVE").unwrap();

    // fmt chunk
    file.write_all(b"fmt ").unwrap();
    file.write_all(&16u32.to_le_bytes()).unwrap();
    file.write_all(&1u16.to_le_bytes()).unwrap(); // PCM
    file.write_all(&1u16.to_le_bytes()).unwrap(); // Mono
    file.write_all(&22050u32.to_le_bytes()).unwrap(); // Sample rate
    file.write_all(&44100u32.to_le_bytes()).unwrap(); // Byte rate
    file.write_all(&2u16.to_le_bytes()).unwrap(); // Block align
    file.write_all(&16u16.to_le_bytes()).unwrap(); // Bits per sample

    // data chunk
    file.write_all(b"data").unwrap();
    file.write_all(&data_size.to_le_bytes()).unwrap();
    for sample in samples {
        file.write_all(&sample.to_le_bytes()).unwrap();
    }
}
