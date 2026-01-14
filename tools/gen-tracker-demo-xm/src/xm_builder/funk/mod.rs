//! Funk XM generation - "Nether Groove"
//!
//! 110 BPM, F Dorian mode, 6 patterns, 6 instruments

mod patterns;

use super::{write_instrument, write_instrument_with_sample};
use patterns::*;

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

/// Generate funk XM file (sample-less, for ROM samples)
pub fn generate_funk_xm() -> Vec<u8> {
    let mut xm = Vec::new();

    // XM Header
    xm.extend_from_slice(b"Extended Module: ");

    // Module name (20 bytes)
    let name = b"Nether Groove";
    xm.extend_from_slice(name);
    xm.extend(std::iter::repeat_n(0u8, 20 - name.len()));

    xm.push(0x1A);

    // Tracker name
    let tracker = b"gen-tracker-demo";
    xm.extend_from_slice(tracker);
    xm.extend(std::iter::repeat_n(0u8, 20 - tracker.len()));

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

    // Pattern order table: Intro -> Groove A -> Groove B -> (repeat) -> Bridge -> Solo -> Outro
    let order = [0u8, 1, 2, 1, 2, 3, 4, 1, 2, 5];
    xm.extend_from_slice(&order);
    xm.extend(std::iter::repeat_n(0u8, 256 - order.len()));

    // Generate patterns
    for i in 0..6 {
        let pattern_data = match i {
            0 => generate_pattern_intro(),
            1 => generate_pattern_groove_a(),
            2 => generate_pattern_groove_b(),
            3 => generate_pattern_bridge(),
            4 => generate_pattern_solo(),
            5 => generate_pattern_outro(),
            _ => unreachable!(),
        };
        let pattern_size = pattern_data.len() as u16;

        // Debug validation
        eprintln!("Funk Pattern {}: size={} bytes", i, pattern_size);
        if pattern_size < 256 {
            eprintln!("WARNING: Funk Pattern {} too small (expected min 256)", i);
        }

        xm.extend_from_slice(&9u32.to_le_bytes()); // header length
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

/// Generate funk XM file with embedded samples
pub fn generate_funk_xm_embedded(samples: &[Vec<i16>]) -> Vec<u8> {
    let mut xm = Vec::new();

    // XM Header
    xm.extend_from_slice(b"Extended Module: ");

    let name = b"Nether Groove";
    xm.extend_from_slice(name);
    xm.extend(std::iter::repeat_n(0u8, 20 - name.len()));

    xm.push(0x1A);

    let tracker = b"gen-tracker-demo";
    xm.extend_from_slice(tracker);
    xm.extend(std::iter::repeat_n(0u8, 20 - tracker.len()));

    xm.extend_from_slice(&0x0104u16.to_le_bytes());
    xm.extend_from_slice(&276u32.to_le_bytes());
    xm.extend_from_slice(&10u16.to_le_bytes());
    xm.extend_from_slice(&1u16.to_le_bytes());
    xm.extend_from_slice(&8u16.to_le_bytes());
    xm.extend_from_slice(&6u16.to_le_bytes());
    xm.extend_from_slice(&6u16.to_le_bytes());
    xm.extend_from_slice(&1u16.to_le_bytes());
    xm.extend_from_slice(&6u16.to_le_bytes());
    xm.extend_from_slice(&110u16.to_le_bytes());

    let order = [0u8, 1, 2, 1, 2, 3, 4, 1, 2, 5];
    xm.extend_from_slice(&order);
    xm.extend(std::iter::repeat_n(0u8, 256 - order.len()));

    // Generate patterns (same as sample-less version)
    for i in 0..6 {
        let pattern_data = match i {
            0 => generate_pattern_intro(),
            1 => generate_pattern_groove_a(),
            2 => generate_pattern_groove_b(),
            3 => generate_pattern_bridge(),
            4 => generate_pattern_solo(),
            5 => generate_pattern_outro(),
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
