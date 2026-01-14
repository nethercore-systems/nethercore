//! Main XM file generators for Eurobeat

use super::patterns::{
    generate_pattern_breakdown, generate_pattern_chorus_a, generate_pattern_chorus_b,
    generate_pattern_drop, generate_pattern_intro, generate_pattern_prechorus,
    generate_pattern_verse_a, generate_pattern_verse_b,
};
use crate::xm_builder::{write_instrument, write_instrument_with_sample};

/// Generate eurobeat XM file (sample-less, for ROM samples)
pub fn generate_eurobeat_xm() -> Vec<u8> {
    let mut xm = Vec::new();

    // XM Header
    xm.extend_from_slice(b"Extended Module: ");

    let name = b"Nether Fire";
    xm.extend_from_slice(name);
    xm.extend(std::iter::repeat_n(0u8, 20 - name.len()));

    xm.push(0x1A);

    let tracker = b"gen-tracker-demo";
    xm.extend_from_slice(tracker);
    xm.extend(std::iter::repeat_n(0u8, 20 - tracker.len()));

    xm.extend_from_slice(&0x0104u16.to_le_bytes());
    xm.extend_from_slice(&276u32.to_le_bytes());
    xm.extend_from_slice(&15u16.to_le_bytes()); // Song length
    xm.extend_from_slice(&3u16.to_le_bytes()); // Restart position
    xm.extend_from_slice(&8u16.to_le_bytes()); // Channels
    xm.extend_from_slice(&8u16.to_le_bytes()); // Patterns
    xm.extend_from_slice(&7u16.to_le_bytes()); // Instruments
    xm.extend_from_slice(&1u16.to_le_bytes()); // Flags
    xm.extend_from_slice(&6u16.to_le_bytes()); // Speed
    xm.extend_from_slice(&155u16.to_le_bytes()); // BPM

    let order = [0u8, 1, 2, 3, 4, 5, 1, 2, 3, 4, 5, 6, 7, 4, 5];
    xm.extend_from_slice(&order);
    xm.extend(std::iter::repeat_n(0u8, 256 - order.len()));

    // Generate patterns
    for i in 0..8 {
        let pattern_data = match i {
            0 => generate_pattern_intro(),
            1 => generate_pattern_verse_a(),
            2 => generate_pattern_verse_b(),
            3 => generate_pattern_prechorus(),
            4 => generate_pattern_chorus_a(),
            5 => generate_pattern_chorus_b(),
            6 => generate_pattern_breakdown(),
            7 => generate_pattern_drop(),
            _ => unreachable!(),
        };
        let pattern_size = pattern_data.len() as u16;

        eprintln!("Eurobeat Pattern {}: size={} bytes", i, pattern_size);

        xm.extend_from_slice(&9u32.to_le_bytes());
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

/// Generate eurobeat XM file with embedded samples
pub fn generate_eurobeat_xm_embedded(samples: &[Vec<i16>]) -> Vec<u8> {
    let mut xm = Vec::new();

    xm.extend_from_slice(b"Extended Module: ");

    let name = b"Nether Fire";
    xm.extend_from_slice(name);
    xm.extend(std::iter::repeat_n(0u8, 20 - name.len()));

    xm.push(0x1A);

    let tracker = b"gen-tracker-demo";
    xm.extend_from_slice(tracker);
    xm.extend(std::iter::repeat_n(0u8, 20 - tracker.len()));

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
    xm.extend(std::iter::repeat_n(0u8, 256 - order.len()));

    for i in 0..8 {
        let pattern_data = match i {
            0 => generate_pattern_intro(),
            1 => generate_pattern_verse_a(),
            2 => generate_pattern_verse_b(),
            3 => generate_pattern_prechorus(),
            4 => generate_pattern_chorus_a(),
            5 => generate_pattern_chorus_b(),
            6 => generate_pattern_breakdown(),
            7 => generate_pattern_drop(),
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
