//! Generates procedural audio samples and XM tracker file for tracker-demo example
//!
//! Creates:
//! - kick.wav (sine sweep drum)
//! - snare.wav (noise + body)
//! - hihat.wav (high-frequency noise)
//! - bass.wav (filtered square wave)
//! - lead.wav (detuned saw synth)
//! - demo.xm (6-channel beat pattern with bass line and melody)

use std::f32::consts::PI;
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;

const SAMPLE_RATE: f32 = 22050.0;

fn main() {
    let output_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("examples")
        .join("tracker-demo")
        .join("assets");

    // Create output directory if it doesn't exist
    fs::create_dir_all(&output_dir).expect("Failed to create assets directory");

    println!("Generating tracker-demo assets to {:?}", output_dir);

    // Generate procedural sounds
    let kick = generate_kick();
    write_wav(&output_dir.join("kick.wav"), &kick);
    println!("  Generated kick.wav ({} samples)", kick.len());

    let snare = generate_snare();
    write_wav(&output_dir.join("snare.wav"), &snare);
    println!("  Generated snare.wav ({} samples)", snare.len());

    let hihat = generate_hihat();
    write_wav(&output_dir.join("hihat.wav"), &hihat);
    println!("  Generated hihat.wav ({} samples)", hihat.len());

    let bass = generate_bass();
    write_wav(&output_dir.join("bass.wav"), &bass);
    println!("  Generated bass.wav ({} samples)", bass.len());

    let lead = generate_lead();
    write_wav(&output_dir.join("lead.wav"), &lead);
    println!("  Generated lead.wav ({} samples)", lead.len());

    // Generate XM file
    let xm = generate_xm();
    fs::write(output_dir.join("demo.xm"), &xm).expect("Failed to write demo.xm");
    println!("  Generated demo.xm ({} bytes)", xm.len());

    println!("Done!");
}

// ============================================================================
// Procedural Sound Generators
// ============================================================================

/// Generate kick drum: sine wave with pitch sweep (150Hz→50Hz) + exponential decay
fn generate_kick() -> Vec<i16> {
    let duration = 0.3; // 300ms
    let samples = (SAMPLE_RATE * duration) as usize;

    let mut output = Vec::with_capacity(samples);
    let mut phase = 0.0f32;

    for i in 0..samples {
        let t = i as f32 / SAMPLE_RATE;

        // Exponential decay envelope
        let decay = (-t * 15.0).exp();

        // Pitch sweep from 150Hz down to 50Hz
        let freq = 150.0 * (-t * 20.0).exp() + 50.0;

        // Phase accumulation for smooth frequency sweep
        phase += 2.0 * PI * freq / SAMPLE_RATE;

        let sample = phase.sin() * decay * 32000.0;
        output.push(sample.clamp(-32767.0, 32767.0) as i16);
    }

    output
}

/// Generate snare: noise burst + sine body (200Hz), fast decay
fn generate_snare() -> Vec<i16> {
    let duration = 0.2; // 200ms
    let samples = (SAMPLE_RATE * duration) as usize;

    let mut output = Vec::with_capacity(samples);
    let mut rng = SimpleRng::new(12345);

    for i in 0..samples {
        let t = i as f32 / SAMPLE_RATE;

        // Fast exponential decay
        let decay = (-t * 20.0).exp();

        // Noise component (filtered white noise)
        let noise = rng.next_f32() * 2.0 - 1.0;

        // Body component (low sine)
        let body = (2.0 * PI * 200.0 * t).sin();

        // Mix noise and body
        let sample = (noise * 0.6 + body * 0.4) * decay * 28000.0;
        output.push(sample.clamp(-32767.0, 32767.0) as i16);
    }

    output
}

/// Generate hi-hat: high-frequency filtered noise with very short decay
fn generate_hihat() -> Vec<i16> {
    let duration = 0.1; // 100ms
    let samples = (SAMPLE_RATE * duration) as usize;

    let mut output = Vec::with_capacity(samples);
    let mut rng = SimpleRng::new(54321);

    // Simple high-pass filter state
    let mut prev_sample = 0.0f32;
    let mut prev_output = 0.0f32;
    let alpha = 0.95; // High-pass coefficient

    for i in 0..samples {
        let t = i as f32 / SAMPLE_RATE;

        // Very fast decay
        let decay = (-t * 40.0).exp();

        // Raw noise
        let noise = rng.next_f32() * 2.0 - 1.0;

        // Simple high-pass filter: y[n] = alpha * (y[n-1] + x[n] - x[n-1])
        let filtered = alpha * (prev_output + noise - prev_sample);
        prev_sample = noise;
        prev_output = filtered;

        let sample = filtered * decay * 20000.0;
        output.push(sample.clamp(-32767.0, 32767.0) as i16);
    }

    output
}

/// Generate bass: filtered square wave at base frequency (will be pitch-shifted by tracker)
fn generate_bass() -> Vec<i16> {
    let duration = 0.5; // 500ms for more sustain
    let freq = 130.81; // C3 as base (tracker will pitch shift)
    let samples = (SAMPLE_RATE * duration) as usize;

    let mut output = Vec::with_capacity(samples);

    // Simple low-pass filter state
    let mut filtered = 0.0f32;
    let cutoff = 0.12; // Low-pass coefficient

    for i in 0..samples {
        let t = i as f32 / SAMPLE_RATE;

        // ADSR-like envelope: quick attack, sustain, slow release
        let envelope = if t < 0.01 {
            t / 0.01 // Attack
        } else if t < 0.3 {
            1.0 // Sustain
        } else {
            (-(t - 0.3) * 4.0).exp() // Release
        };

        // Square wave with slight pulse width modulation
        let pw = 0.5 + 0.1 * (t * 3.0).sin(); // Pulse width varies slightly
        let phase = (2.0 * PI * freq * t) % (2.0 * PI);
        let square = if phase < PI * pw * 2.0 { 1.0 } else { -1.0 };

        // Simple low-pass filter to smooth harsh edges
        filtered = filtered + cutoff * (square - filtered);

        let sample = filtered * envelope * 26000.0;
        output.push(sample.clamp(-32767.0, 32767.0) as i16);
    }

    output
}

/// Generate lead: detuned saw waves with vibrato for a rich synth sound
fn generate_lead() -> Vec<i16> {
    let duration = 0.6; // 600ms
    let freq = 261.63; // C4 as base (tracker will pitch shift)
    let samples = (SAMPLE_RATE * duration) as usize;

    let mut output = Vec::with_capacity(samples);

    for i in 0..samples {
        let t = i as f32 / SAMPLE_RATE;

        // ADSR envelope
        let envelope = if t < 0.02 {
            t / 0.02 // Fast attack
        } else if t < 0.4 {
            1.0 - (t - 0.02) * 0.3 // Slow decay to sustain
        } else {
            0.7 * (-(t - 0.4) * 3.0).exp() // Release
        };

        // Vibrato (pitch wobble)
        let vibrato = 1.0 + 0.015 * (t * 5.0 * 2.0 * PI).sin();

        // Two detuned saw waves for richness
        let detune = 1.003; // Slight detune
        let phase1 = (freq * vibrato * t) % 1.0;
        let phase2 = (freq * vibrato * detune * t) % 1.0;

        // Saw wave: 2 * phase - 1 gives range [-1, 1]
        let saw1 = 2.0 * phase1 - 1.0;
        let saw2 = 2.0 * phase2 - 1.0;

        // Mix the two saws
        let sample = (saw1 * 0.5 + saw2 * 0.5) * envelope * 22000.0;
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
// XM File Generator
// ============================================================================

fn generate_xm() -> Vec<u8> {
    let mut xm = Vec::new();

    // ========================================================================
    // XM Header (60 bytes fixed + 256 order table)
    // ========================================================================

    // ID text (17 bytes)
    xm.extend_from_slice(b"Extended Module: ");

    // Module name (20 bytes, null-padded)
    let name = b"Tracker Demo";
    xm.extend_from_slice(name);
    xm.extend(std::iter::repeat(0u8).take(20 - name.len()));

    // 0x1A marker
    xm.push(0x1A);

    // Tracker name (20 bytes)
    let tracker = b"gen-tracker-demo";
    xm.extend_from_slice(tracker);
    xm.extend(std::iter::repeat(0u8).take(20 - tracker.len()));

    // Version (must be 0x0104)
    xm.extend_from_slice(&0x0104u16.to_le_bytes());

    // Header size (from this point, includes order table)
    // = 16 bytes (remaining header: 8 x u16) + 256 bytes (order table) = 272
    xm.extend_from_slice(&272u32.to_le_bytes());

    // Song length (number of orders) - 2 patterns for variety
    xm.extend_from_slice(&2u16.to_le_bytes());

    // Restart position
    xm.extend_from_slice(&0u16.to_le_bytes());

    // Number of channels (6: kick, snare, hihat, bass, lead1, lead2)
    xm.extend_from_slice(&6u16.to_le_bytes());

    // Number of patterns
    xm.extend_from_slice(&2u16.to_le_bytes());

    // Number of instruments
    xm.extend_from_slice(&5u16.to_le_bytes());

    // Flags (bit 0 = linear frequency table)
    xm.extend_from_slice(&1u16.to_le_bytes());

    // Default speed (ticks per row)
    xm.extend_from_slice(&6u16.to_le_bytes());

    // Default BPM
    xm.extend_from_slice(&120u16.to_le_bytes());

    // Pattern order table (256 bytes)
    xm.push(0); // Pattern 0 at order 0
    xm.push(1); // Pattern 1 at order 1
    xm.extend(std::iter::repeat(0u8).take(254));

    // ========================================================================
    // Pattern 0: 32 rows, 6 channels - Main groove with bass line
    // ========================================================================

    let pattern0_data = generate_pattern_main();
    let pattern0_data_size = pattern0_data.len() as u16;

    xm.extend_from_slice(&5u32.to_le_bytes()); // header length
    xm.push(0); // packing type
    xm.extend_from_slice(&32u16.to_le_bytes()); // 32 rows
    xm.extend_from_slice(&pattern0_data_size.to_le_bytes());
    xm.extend_from_slice(&pattern0_data);

    // ========================================================================
    // Pattern 1: 32 rows, 6 channels - Variation with melody
    // ========================================================================

    let pattern1_data = generate_pattern_melody();
    let pattern1_data_size = pattern1_data.len() as u16;

    xm.extend_from_slice(&5u32.to_le_bytes()); // header length
    xm.push(0); // packing type
    xm.extend_from_slice(&32u16.to_le_bytes()); // 32 rows
    xm.extend_from_slice(&pattern1_data_size.to_le_bytes());
    xm.extend_from_slice(&pattern1_data);

    // ========================================================================
    // Instruments (5 instruments, no samples - samples from ROM)
    // ========================================================================

    // Instrument names must match sound IDs in nether.toml
    let instruments = ["kick", "snare", "hihat", "bass", "lead"];

    for name in &instruments {
        write_instrument(&mut xm, name);
    }

    xm
}

// XM note values: C-0 = 1, C-1 = 13, C-2 = 25, C-3 = 37, C-4 = 49, C-5 = 61
// Each octave is 12 semitones
const C2: u8 = 25;
const D2: u8 = 27;
const E2: u8 = 29;
const F2: u8 = 30;
const G2: u8 = 32;
const A2: u8 = 34;

const C3: u8 = 37;
const D3: u8 = 39;
const E3: u8 = 41;
const G3: u8 = 44;
const A3: u8 = 46;

const C4: u8 = 49;
const D4: u8 = 51;
const E4: u8 = 53;
const G4: u8 = 56;
const A4: u8 = 58;

const C5: u8 = 61;

// Instruments
const KICK: u8 = 1;
const SNARE: u8 = 2;
const HIHAT: u8 = 3;
const BASS: u8 = 4;
const LEAD: u8 = 5;

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

/// Generate main pattern: 32 rows, 6 channels
/// Drums + bass line (Am - F - C - G progression feel)
fn generate_pattern_main() -> Vec<u8> {
    let mut data = Vec::new();

    // Bass line: simple root notes following chord progression
    // Rows 0-7: A, Rows 8-15: F, Rows 16-23: C, Rows 24-31: G
    let bass_notes = [
        A2, A2, 0, 0, A2, 0, 0, 0,  // A minor feel
        F2, F2, 0, 0, F2, 0, 0, 0,  // F major feel
        C3, C3, 0, 0, C3, 0, 0, 0,  // C major feel
        G2, G2, 0, 0, G2, 0, 0, 0,  // G major feel
    ];

    for row in 0..32 {
        // Channel 0: Kick - four on the floor
        if row % 8 == 0 || row % 8 == 4 {
            write_note(&mut data, C4, KICK);
        } else {
            write_empty(&mut data);
        }

        // Channel 1: Snare - beats 2 and 4
        if row % 8 == 4 {
            write_note(&mut data, C4, SNARE);
        } else {
            write_empty(&mut data);
        }

        // Channel 2: HiHat - off-beats
        if row % 2 == 0 {
            write_note(&mut data, C4, HIHAT);
        } else {
            write_empty(&mut data);
        }

        // Channel 3: Bass line
        let bass = bass_notes[row];
        if bass != 0 {
            write_note(&mut data, bass, BASS);
        } else {
            write_empty(&mut data);
        }

        // Channel 4: Lead (empty in main pattern)
        write_empty(&mut data);

        // Channel 5: Lead harmony (empty in main pattern)
        write_empty(&mut data);
    }

    data
}

/// Generate melody pattern: 32 rows, 6 channels
/// Same drums + bass, but with melody on top
fn generate_pattern_melody() -> Vec<u8> {
    let mut data = Vec::new();

    // Bass line (same as main)
    let bass_notes = [
        A2, A2, 0, 0, A2, 0, 0, 0,
        F2, F2, 0, 0, F2, 0, 0, 0,
        C3, C3, 0, 0, C3, 0, 0, 0,
        G2, G2, 0, 0, G2, 0, 0, 0,
    ];

    // Melody line - simple ascending/descending phrase
    let melody = [
        C4, 0, E4, 0, G4, 0, A4, 0,     // Rising on Am
        A4, 0, G4, 0, F2+24, 0, E4, 0,  // Falling on F (F4 = F2+24)
        E4, 0, G4, 0, C5, 0, G4, 0,     // Rising on C
        G4, 0, E4, 0, D4, 0, C4, 0,     // Falling on G
    ];

    // Harmony (thirds below melody)
    let harmony = [
        A3, 0, C4, 0, E4, 0, C4, 0,     // Thirds below
        C4, 0, E4, 0, D4, 0, C4, 0,
        C4, 0, E4, 0, G4, 0, E4, 0,
        E4, 0, C4, 0, 0, 0, A3, 0,      // sparse ending
    ];

    for row in 0..32 {
        // Channel 0: Kick - four on the floor
        if row % 8 == 0 || row % 8 == 4 {
            write_note(&mut data, C4, KICK);
        } else {
            write_empty(&mut data);
        }

        // Channel 1: Snare - beats 2 and 4
        if row % 8 == 4 {
            write_note(&mut data, C4, SNARE);
        } else {
            write_empty(&mut data);
        }

        // Channel 2: HiHat - off-beats (slightly less frequent in melody section)
        if row % 4 == 0 || row % 4 == 2 {
            write_note(&mut data, C4, HIHAT);
        } else {
            write_empty(&mut data);
        }

        // Channel 3: Bass line
        let bass = bass_notes[row];
        if bass != 0 {
            write_note(&mut data, bass, BASS);
        } else {
            write_empty(&mut data);
        }

        // Channel 4: Lead melody
        let mel = melody[row];
        if mel != 0 {
            write_note(&mut data, mel, LEAD);
        } else {
            write_empty(&mut data);
        }

        // Channel 5: Lead harmony
        let harm = harmony[row];
        if harm != 0 {
            write_note(&mut data, harm, LEAD);
        } else {
            write_empty(&mut data);
        }
    }

    data
}

/// Write a minimal instrument header (no samples - we use ROM sounds)
fn write_instrument(xm: &mut Vec<u8>, name: &str) {
    // Instrument header size (263 bytes standard, but we use minimal)
    // For no-sample instruments: just 29 bytes header
    let header_size: u32 = 29;
    xm.extend_from_slice(&header_size.to_le_bytes());

    // Instrument name (22 bytes)
    let name_bytes = name.as_bytes();
    xm.extend_from_slice(name_bytes);
    xm.extend(std::iter::repeat(0u8).take(22 - name_bytes.len().min(22)));

    // Instrument type (always 0)
    xm.push(0);

    // Number of samples (0 = no embedded samples)
    xm.extend_from_slice(&0u16.to_le_bytes());

    // No additional data since num_samples = 0
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_xm_parses_correctly() {
        let xm_data = generate_xm();

        // Verify the XM can be parsed by nether-xm
        let module = nether_xm::parse_xm(&xm_data).expect("Generated XM should parse correctly");

        assert_eq!(module.name, "Tracker Demo");
        assert_eq!(module.num_channels, 6);
        assert_eq!(module.num_patterns, 2);
        assert_eq!(module.num_instruments, 5);
        assert_eq!(module.song_length, 2);
        assert_eq!(module.default_speed, 6);
        assert_eq!(module.default_bpm, 120);

        // Verify instrument names
        assert_eq!(module.instruments[0].name, "kick");
        assert_eq!(module.instruments[1].name, "snare");
        assert_eq!(module.instruments[2].name, "hihat");
        assert_eq!(module.instruments[3].name, "bass");
        assert_eq!(module.instruments[4].name, "lead");

        // Verify patterns have 32 rows
        assert_eq!(module.patterns[0].num_rows, 32);
        assert_eq!(module.patterns[1].num_rows, 32);
    }

    #[test]
    fn test_generate_xm_instrument_names() {
        let xm_data = generate_xm();

        // Verify get_instrument_names works (used by pack command)
        let names =
            nether_xm::get_instrument_names(&xm_data).expect("Should extract instrument names");

        assert_eq!(names.len(), 5);
        assert_eq!(names[0], "kick");
        assert_eq!(names[1], "snare");
        assert_eq!(names[2], "hihat");
        assert_eq!(names[3], "bass");
        assert_eq!(names[4], "lead");
    }

    #[test]
    fn test_generate_xm_strip_samples() {
        let xm_data = generate_xm();

        // Verify strip_xm_samples works (used by pack command)
        let stripped = nether_xm::strip_xm_samples(&xm_data).expect("Should strip samples from XM");

        // Since our XM has no samples, stripped should equal original
        assert_eq!(stripped.len(), xm_data.len());
    }

    #[test]
    fn test_generate_pattern_main_size() {
        let pattern = generate_pattern_main();

        // 32 rows * 6 channels
        // Each channel entry is either 1 byte (empty) or 3 bytes (note+instrument)
        assert!(pattern.len() > 0);
        assert!(pattern.len() <= 32 * 6 * 3); // Max possible size
    }

    #[test]
    fn test_generate_pattern_melody_size() {
        let pattern = generate_pattern_melody();

        // 32 rows * 6 channels
        assert!(pattern.len() > 0);
        assert!(pattern.len() <= 32 * 6 * 3); // Max possible size
    }

    #[test]
    fn test_kick_sample_length() {
        let kick = generate_kick();
        // 300ms at 22050Hz = 6615 samples
        assert_eq!(kick.len(), (SAMPLE_RATE * 0.3) as usize);
    }

    #[test]
    fn test_snare_sample_length() {
        let snare = generate_snare();
        // 200ms at 22050Hz = 4410 samples
        assert_eq!(snare.len(), (SAMPLE_RATE * 0.2) as usize);
    }

    #[test]
    fn test_hihat_sample_length() {
        let hihat = generate_hihat();
        // 100ms at 22050Hz = 2205 samples
        assert_eq!(hihat.len(), (SAMPLE_RATE * 0.1) as usize);
    }

    #[test]
    fn test_bass_sample_length() {
        let bass = generate_bass();
        // 500ms at 22050Hz = 11025 samples
        assert_eq!(bass.len(), (SAMPLE_RATE * 0.5) as usize);
    }

    #[test]
    fn test_lead_sample_length() {
        let lead = generate_lead();
        // 600ms at 22050Hz = 13230 samples
        assert_eq!(lead.len(), (SAMPLE_RATE * 0.6) as usize);
    }
}
