//! Generates procedural audio samples and XM tracker file for tracker-demo example
//!
//! Creates:
//! - kick.wav (sine sweep drum)
//! - snare.wav (noise + body)
//! - hihat.wav (high-frequency noise)
//! - bass.wav (filtered square wave)
//! - demo.xm (4-channel beat pattern)

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

/// Generate bass: filtered square wave at C2 (65.41 Hz)
fn generate_bass() -> Vec<i16> {
    let duration = 0.4; // 400ms
    let freq = 65.41; // C2
    let samples = (SAMPLE_RATE * duration) as usize;

    let mut output = Vec::with_capacity(samples);

    // Simple low-pass filter state
    let mut filtered = 0.0f32;
    let cutoff = 0.15; // Low-pass coefficient

    for i in 0..samples {
        let t = i as f32 / SAMPLE_RATE;

        // Slow decay for sustained bass
        let decay = (-t * 5.0).exp();

        // Square wave
        let phase = (2.0 * PI * freq * t) % (2.0 * PI);
        let square = if phase < PI { 1.0 } else { -1.0 };

        // Simple low-pass filter to smooth harsh edges
        filtered = filtered + cutoff * (square - filtered);

        let sample = filtered * decay * 24000.0;
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

    // Song length (number of orders)
    xm.extend_from_slice(&1u16.to_le_bytes());

    // Restart position
    xm.extend_from_slice(&0u16.to_le_bytes());

    // Number of channels
    xm.extend_from_slice(&4u16.to_le_bytes());

    // Number of patterns
    xm.extend_from_slice(&1u16.to_le_bytes());

    // Number of instruments
    xm.extend_from_slice(&4u16.to_le_bytes());

    // Flags (bit 0 = linear frequency table)
    xm.extend_from_slice(&1u16.to_le_bytes());

    // Default speed (ticks per row)
    xm.extend_from_slice(&6u16.to_le_bytes());

    // Default BPM
    xm.extend_from_slice(&125u16.to_le_bytes());

    // Pattern order table (256 bytes)
    xm.push(0); // Pattern 0 at order 0
    xm.extend(std::iter::repeat(0u8).take(255));

    // ========================================================================
    // Pattern 0: 16 rows, 4 channels
    // ========================================================================
    // Beat pattern:
    // Row 0:  Kick C-4, HiHat C-4, Bass C-2
    // Row 4:  HiHat C-4
    // Row 8:  Snare C-4, HiHat C-4
    // Row 12: HiHat C-4

    let pattern_data = generate_pattern();
    let pattern_data_size = pattern_data.len() as u16;

    // Pattern header
    // Note: header_length is the size AFTER this field (packing_type + num_rows + packed_size = 5)
    xm.extend_from_slice(&5u32.to_le_bytes()); // header length (5 bytes after this field)
    xm.push(0); // packing type (always 0)
    xm.extend_from_slice(&16u16.to_le_bytes()); // number of rows
    xm.extend_from_slice(&pattern_data_size.to_le_bytes()); // packed data size

    // Pattern note data
    xm.extend_from_slice(&pattern_data);

    // ========================================================================
    // Instruments (4 instruments, no samples - samples from ROM)
    // ========================================================================

    // Instrument names must match sound IDs in nether.toml
    let instruments = ["kick", "snare", "hihat", "bass"];

    for name in &instruments {
        write_instrument(&mut xm, name);
    }

    xm
}

/// Generate packed pattern data for 16 rows, 4 channels
fn generate_pattern() -> Vec<u8> {
    let mut data = Vec::new();

    // Note value for C-4 (middle C) = 49, C-2 = 25
    const C4: u8 = 49;
    const C2: u8 = 25;

    for row in 0..16 {
        // Channel 0: Kick (instrument 1)
        if row == 0 {
            // Full note: note + instrument
            data.push(0x80 | 0x01 | 0x02); // packed byte: has note + instrument
            data.push(C4); // note
            data.push(1); // instrument
        } else {
            // Empty
            data.push(0x80); // packed byte: nothing
        }

        // Channel 1: Snare (instrument 2)
        if row == 8 {
            data.push(0x80 | 0x01 | 0x02);
            data.push(C4);
            data.push(2);
        } else {
            data.push(0x80);
        }

        // Channel 2: HiHat (instrument 3)
        if row == 0 || row == 4 || row == 8 || row == 12 {
            data.push(0x80 | 0x01 | 0x02);
            data.push(C4);
            data.push(3);
        } else {
            data.push(0x80);
        }

        // Channel 3: Bass (instrument 4)
        if row == 0 {
            data.push(0x80 | 0x01 | 0x02);
            data.push(C2);
            data.push(4);
        } else {
            data.push(0x80);
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
        assert_eq!(module.num_channels, 4);
        assert_eq!(module.num_patterns, 1);
        assert_eq!(module.num_instruments, 4);
        assert_eq!(module.song_length, 1);
        assert_eq!(module.default_speed, 6);
        assert_eq!(module.default_bpm, 125);

        // Verify instrument names
        assert_eq!(module.instruments[0].name, "kick");
        assert_eq!(module.instruments[1].name, "snare");
        assert_eq!(module.instruments[2].name, "hihat");
        assert_eq!(module.instruments[3].name, "bass");

        // Verify pattern has 16 rows
        assert_eq!(module.patterns[0].num_rows, 16);
    }

    #[test]
    fn test_generate_xm_instrument_names() {
        let xm_data = generate_xm();

        // Verify get_instrument_names works (used by pack command)
        let names =
            nether_xm::get_instrument_names(&xm_data).expect("Should extract instrument names");

        assert_eq!(names.len(), 4);
        assert_eq!(names[0], "kick");
        assert_eq!(names[1], "snare");
        assert_eq!(names[2], "hihat");
        assert_eq!(names[3], "bass");
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
    fn test_generate_pattern_size() {
        let pattern = generate_pattern();

        // 16 rows * 4 channels = 64 notes
        // Each note is at least 1 byte (packed empty = 0x80)
        // Notes with data: row 0 has 3 notes with data, row 4/8/12 have 1 each
        // Empty notes: 1 byte each
        // Notes with data: 3 bytes each (0x83, note, instrument)

        // Row 0: kick(3) + empty(1) + hihat(3) + bass(3) = 10
        // Row 4: empty(1) + empty(1) + hihat(3) + empty(1) = 6
        // Row 8: empty(1) + snare(3) + hihat(3) + empty(1) = 8
        // Row 12: empty(1) + empty(1) + hihat(3) + empty(1) = 6
        // Other 12 rows: 4 empty = 4 bytes each = 48
        // Total = 10 + 6 + 8 + 6 + 48 = 78 bytes
        assert_eq!(pattern.len(), 78);
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
        // 400ms at 22050Hz = 8820 samples
        assert_eq!(bass.len(), (SAMPLE_RATE * 0.4) as usize);
    }
}
