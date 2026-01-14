//! XM file generation for synthwave tracks

use super::patterns::{
    generate_pattern_bridge, generate_pattern_build, generate_pattern_chorus_a,
    generate_pattern_chorus_b, generate_pattern_intro, generate_pattern_outro,
    generate_pattern_verse_a, generate_pattern_verse_b,
};
use crate::xm_builder::{write_instrument, write_instrument_with_sample};

/// Generate synthwave XM with sample-less instruments (for ROM samples)
pub fn generate_synthwave_xm() -> Vec<u8> {
    let mut xm = Vec::new();

    // XM Header
    xm.extend_from_slice(b"Extended Module: ");

    // Module name
    let name = b"Nether Drive";
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
    xm.extend(std::iter::repeat_n(0u8, 256 - order.len()));

    // Generate patterns
    for i in 0..8 {
        let pattern_data = match i {
            0 => generate_pattern_intro(),
            1 => generate_pattern_verse_a(),
            2 => generate_pattern_verse_b(),
            3 => generate_pattern_chorus_a(),
            4 => generate_pattern_chorus_b(),
            5 => generate_pattern_bridge(),
            6 => generate_pattern_build(),
            7 => generate_pattern_outro(),
            _ => unreachable!(),
        };
        let pattern_size = pattern_data.len() as u16;

        // Debug validation
        eprintln!("Synthwave Pattern {}: size={} bytes", i, pattern_size);
        if pattern_size < 256 {
            eprintln!(
                "WARNING: Synthwave Pattern {} too small (expected min 256)",
                i
            );
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

/// Generate synthwave XM with embedded samples
pub fn generate_synthwave_xm_embedded(samples: &[Vec<i16>]) -> Vec<u8> {
    let mut xm = Vec::new();

    // XM Header
    xm.extend_from_slice(b"Extended Module: ");

    let name = b"Nether Drive";
    xm.extend_from_slice(name);
    xm.extend(std::iter::repeat_n(0u8, 20 - name.len()));

    xm.push(0x1A);

    let tracker = b"gen-tracker-demo";
    xm.extend_from_slice(tracker);
    xm.extend(std::iter::repeat_n(0u8, 20 - tracker.len()));

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
    xm.extend(std::iter::repeat_n(0u8, 256 - order.len()));

    for i in 0..8 {
        let pattern_data = match i {
            0 => generate_pattern_intro(),
            1 => generate_pattern_verse_a(),
            2 => generate_pattern_verse_b(),
            3 => generate_pattern_chorus_a(),
            4 => generate_pattern_chorus_b(),
            5 => generate_pattern_bridge(),
            6 => generate_pattern_build(),
            7 => generate_pattern_outro(),
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
