//! Eurobeat XM generation - "Nether Fire"
//!
//! 155 BPM, D minor, 8 patterns, 7 instruments

use super::{write_note, write_note_vol, write_empty, write_instrument, write_instrument_with_sample};

// Eurobeat note constants (D minor: D E F G A Bb C)
const D2_E: u8 = 27;
const F2_E: u8 = 30;
const G2_E: u8 = 32;
const A2_E: u8 = 34;
const BB2_E: u8 = 35;
const C3_E: u8 = 37;
const D3_E: u8 = 39;
const F3_E: u8 = 42;
const G3_E: u8 = 44;
const A3_E: u8 = 46;
const BB3_E: u8 = 47;
const C4_E: u8 = 49;
const CS4_E: u8 = 50;
const D4_E: u8 = 51;
const E4_E: u8 = 53;
const F4_E: u8 = 54;
const G4_E: u8 = 56;
const A4_E: u8 = 58;
const BB4_E: u8 = 59;
const C5_E: u8 = 61;
const CS5_E: u8 = 62;
const D5_E: u8 = 63;
const E5_E: u8 = 65;
const F5_E: u8 = 66;
const G5_E: u8 = 68;
const A5_E: u8 = 70;
const BB5_E: u8 = 71;
const C6_E: u8 = 73;
const CS6_E: u8 = 74;
const D6_E: u8 = 75;
const E6_E: u8 = 77;
const F6_E: u8 = 78;
const G6_E: u8 = 80;
const A6_E: u8 = 82;

// Eurobeat instruments
const KICK_E: u8 = 1;
const SNARE_E: u8 = 2;
const HIHAT_E: u8 = 3;
const BASS_E: u8 = 4;
const SUPERSAW: u8 = 5;
const BRASS: u8 = 6;
const PAD: u8 = 7;

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
    xm.extend_from_slice(&3u16.to_le_bytes());  // Restart position
    xm.extend_from_slice(&8u16.to_le_bytes());  // Channels
    xm.extend_from_slice(&8u16.to_le_bytes());  // Patterns
    xm.extend_from_slice(&7u16.to_le_bytes());  // Instruments
    xm.extend_from_slice(&1u16.to_le_bytes());  // Flags
    xm.extend_from_slice(&6u16.to_le_bytes());  // Speed
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
        "kick_euro", "snare_euro", "hihat_euro", "bass_euro",
        "supersaw", "brass_euro", "pad_euro",
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
        "kick_euro", "snare_euro", "hihat_euro", "bass_euro",
        "supersaw", "brass_euro", "pad_euro",
    ];
    for (i, name) in instruments.iter().enumerate() {
        write_instrument_with_sample(&mut xm, name, &samples[i]);
    }

    xm
}

// ============================================================================
// Pattern Generators
// ============================================================================

fn generate_pattern_intro() -> Vec<u8> {
    let mut data = Vec::new();

    for row in 0..32 {
        // Ch1: Kick
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

        // Ch2: Snare
        if row >= 24 && row % 8 == 4 {
            write_note(&mut data, C4_E, SNARE_E);
        } else {
            write_empty(&mut data);
        }

        // Ch3: Hi-hat
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

        // Ch4: Bass
        if row == 0 {
            write_note(&mut data, D2_E, BASS_E);
        } else if row >= 16 && row % 2 == 0 {
            let note = if (row / 2) % 2 == 0 { D2_E } else { D3_E };
            write_note(&mut data, note, BASS_E);
        } else {
            write_empty(&mut data);
        }

        // Ch5: Supersaw - HOOK TEASER
        match row {
            0 => write_note(&mut data, D5_E, SUPERSAW),
            2 => write_note(&mut data, F5_E, SUPERSAW),
            4 => write_note(&mut data, A5_E, SUPERSAW),
            8 => write_note(&mut data, D5_E, SUPERSAW),
            10 => write_note(&mut data, F5_E, SUPERSAW),
            12 => write_note(&mut data, G5_E, SUPERSAW),
            16 => write_note(&mut data, D5_E, SUPERSAW),
            18 => write_note(&mut data, F5_E, SUPERSAW),
            20 => write_note(&mut data, A5_E, SUPERSAW),
            21 => write_note(&mut data, A5_E, SUPERSAW),
            22 => write_note(&mut data, G5_E, SUPERSAW),
            24 => write_note(&mut data, F5_E, SUPERSAW),
            26 => write_note(&mut data, E5_E, SUPERSAW),
            28 => write_note(&mut data, D5_E, SUPERSAW),
            30 => write_note(&mut data, CS5_E, SUPERSAW),
            _ => write_empty(&mut data),
        }

        // Ch6: Brass
        match row {
            6 => write_note(&mut data, A4_E, BRASS),
            14 => write_note(&mut data, G4_E, BRASS),
            30 => write_note(&mut data, A3_E, BRASS),
            _ => write_empty(&mut data),
        }

        // Ch7: Pad
        if row == 0 {
            write_note(&mut data, D3_E, PAD);
        } else if row == 8 {
            write_note(&mut data, BB3_E, PAD);
        } else if row == 16 {
            write_note(&mut data, C4_E, PAD);
        } else if row == 24 {
            write_note(&mut data, A3_E, PAD);
        } else {
            write_empty(&mut data);
        }

        // Ch8: Silent
        write_empty(&mut data);
    }

    data
}

fn generate_pattern_verse_a() -> Vec<u8> {
    let mut data = Vec::new();

    let bass_pattern: [(u8, u8); 16] = [
        (D2_E, D3_E), (D2_E, D3_E), (D2_E, D3_E), (D2_E, D3_E),
        (G2_E, G3_E), (G2_E, G3_E), (G2_E, G3_E), (G2_E, G3_E),
        (BB2_E, BB3_E), (BB2_E, BB3_E), (BB2_E, BB3_E), (BB2_E, BB3_E),
        (A2_E, A3_E), (A2_E, A3_E), (A2_E, A3_E), (A2_E, A3_E),
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

        // Ch4: Bass
        if row % 2 == 0 {
            let idx = (row / 2) as usize;
            let (low, high) = bass_pattern[idx];
            let note = if (row / 2) % 2 == 0 { low } else { high };
            write_note(&mut data, note, BASS_E);
        } else {
            write_empty(&mut data);
        }

        // Ch5: Supersaw - sparse melody
        match row {
            0 => write_note(&mut data, A4_E, SUPERSAW),
            4 => write_note(&mut data, D5_E, SUPERSAW),
            8 => write_note(&mut data, D5_E, SUPERSAW),
            12 => write_note(&mut data, A4_E, SUPERSAW),
            16 => write_note(&mut data, F4_E, SUPERSAW),
            18 => write_note(&mut data, A4_E, SUPERSAW),
            20 => write_note(&mut data, BB4_E, SUPERSAW),
            21 => write_note(&mut data, A4_E, SUPERSAW),
            22 => write_note(&mut data, F4_E, SUPERSAW),
            24 => write_note(&mut data, D5_E, SUPERSAW),
            26 => write_note(&mut data, F5_E, SUPERSAW),
            28 => write_note(&mut data, A5_E, SUPERSAW),
            29 => write_note(&mut data, A5_E, SUPERSAW),
            _ => write_empty(&mut data),
        }

        // Ch6: Brass
        match row {
            7 => write_note(&mut data, D4_E, BRASS),
            15 => write_note(&mut data, G4_E, BRASS),
            19 => write_note(&mut data, BB4_E, BRASS),
            23 => write_note(&mut data, E4_E, BRASS),
            _ => write_empty(&mut data),
        }

        // Ch7: Pad
        match row {
            0 => write_note(&mut data, D3_E, PAD),
            8 => write_note(&mut data, G3_E, PAD),
            16 => write_note(&mut data, BB3_E, PAD),
            24 => write_note(&mut data, A3_E, PAD),
            _ => write_empty(&mut data),
        }

        // Ch8: Harmony
        match row {
            6 => write_note(&mut data, F5_E, SUPERSAW),
            14 => write_note(&mut data, C5_E, SUPERSAW),
            28 => write_note(&mut data, C5_E, SUPERSAW),
            _ => write_empty(&mut data),
        }
    }

    data
}

fn generate_pattern_verse_b() -> Vec<u8> {
    let mut data = Vec::new();

    let bass_pattern: [(u8, u8); 16] = [
        (D2_E, D3_E), (D2_E, D3_E), (D2_E, D3_E), (D2_E, D3_E),
        (C3_E, C4_E), (C3_E, C4_E), (C3_E, C4_E), (C3_E, C4_E),
        (BB2_E, BB3_E), (BB2_E, BB3_E), (BB2_E, BB3_E), (BB2_E, BB3_E),
        (C3_E, C4_E), (C3_E, C4_E), (C3_E, C4_E), (C3_E, C4_E),
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

        // Ch4: Bass
        if row % 2 == 0 {
            let idx = (row / 2) as usize;
            let (low, high) = bass_pattern[idx];
            let note = if (row / 2) % 2 == 0 { low } else { high };
            write_note(&mut data, note, BASS_E);
        } else {
            write_empty(&mut data);
        }

        // Ch5: Supersaw
        match row {
            0 => write_note(&mut data, D5_E, SUPERSAW),
            4 => write_note(&mut data, C5_E, SUPERSAW),
            8 => write_note(&mut data, E5_E, SUPERSAW),
            12 => write_note(&mut data, D5_E, SUPERSAW),
            16 => write_note(&mut data, BB4_E, SUPERSAW),
            20 => write_note(&mut data, F5_E, SUPERSAW),
            21 => write_note(&mut data, F5_E, SUPERSAW),
            22 => write_note(&mut data, D5_E, SUPERSAW),
            24 => write_note(&mut data, D5_E, SUPERSAW),
            26 => write_note(&mut data, F5_E, SUPERSAW),
            28 => write_note(&mut data, A5_E, SUPERSAW),
            29 => write_note(&mut data, A5_E, SUPERSAW),
            30 => write_note(&mut data, G5_E, SUPERSAW),
            31 => write_note(&mut data, D5_E, SUPERSAW),
            _ => write_empty(&mut data),
        }

        // Ch6: Brass
        match row {
            7 => write_note(&mut data, F4_E, BRASS),
            15 => write_note(&mut data, E4_E, BRASS),
            _ => write_empty(&mut data),
        }

        // Ch7: Pad
        match row {
            0 => write_note(&mut data, D3_E, PAD),
            8 => write_note(&mut data, C4_E, PAD),
            16 => write_note(&mut data, BB3_E, PAD),
            24 => write_note(&mut data, C4_E, PAD),
            _ => write_empty(&mut data),
        }

        // Ch8: Harmony
        match row {
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

fn generate_pattern_prechorus() -> Vec<u8> {
    let mut data = Vec::new();

    for row in 0..32 {
        // Ch1: Kick
        if row < 16 {
            if row % 4 == 0 {
                write_note(&mut data, C4_E, KICK_E);
            } else {
                write_empty(&mut data);
            }
        } else if row % 2 == 0 {
            write_note(&mut data, C4_E, KICK_E);
        } else {
            write_empty(&mut data);
        }

        // Ch2: Snare
        if row < 24 {
            if row % 8 == 4 {
                write_note(&mut data, C4_E, SNARE_E);
            } else {
                write_empty(&mut data);
            }
        } else {
            write_note(&mut data, C4_E, SNARE_E);
        }

        // Ch3: Hi-hat
        if row < 16 {
            if row % 2 == 0 {
                write_note(&mut data, C4_E, HIHAT_E);
            } else {
                write_empty(&mut data);
            }
        } else {
            write_note(&mut data, C4_E, HIHAT_E);
        }

        // Ch4: Bass
        let bass_note = match row {
            0..=7 => if (row / 2) % 2 == 0 { F2_E } else { F3_E },
            8..=15 => if (row / 2) % 2 == 0 { G2_E } else { G3_E },
            16..=31 => if (row / 2) % 2 == 0 { A2_E } else { A3_E },
            _ => A2_E,
        };
        if row % 2 == 0 {
            write_note(&mut data, bass_note, BASS_E);
        } else {
            write_empty(&mut data);
        }

        // Ch5: Supersaw
        match row {
            0 => write_note(&mut data, A4_E, SUPERSAW),
            2 => write_note(&mut data, C5_E, SUPERSAW),
            4 => write_note(&mut data, C5_E, SUPERSAW),
            6 => write_note(&mut data, BB4_E, SUPERSAW),
            8 => write_note(&mut data, E5_E, SUPERSAW),
            10 => write_note(&mut data, E5_E, SUPERSAW),
            12 => write_note(&mut data, E5_E, SUPERSAW),
            13 => write_note(&mut data, F5_E, SUPERSAW),
            14 => write_note(&mut data, G5_E, SUPERSAW),
            16 => write_note(&mut data, A5_E, SUPERSAW),
            18 => write_note(&mut data, G5_E, SUPERSAW),
            20 => write_note(&mut data, F5_E, SUPERSAW),
            22 => write_note(&mut data, D5_E, SUPERSAW),
            24 => write_note(&mut data, F4_E, SUPERSAW),
            26 => write_note(&mut data, A4_E, SUPERSAW),
            28 => write_note(&mut data, C5_E, SUPERSAW),
            31 => write_note(&mut data, F5_E, SUPERSAW),
            _ => write_empty(&mut data),
        }

        // Ch6: Brass
        match row {
            0 => write_note(&mut data, F4_E, BRASS),
            8 => write_note(&mut data, G4_E, BRASS),
            16 => write_note(&mut data, A4_E, BRASS),
            28 => write_note(&mut data, C4_E, BRASS),
            _ => write_empty(&mut data),
        }

        // Ch7: Pad
        match row {
            0 => write_note(&mut data, F3_E, PAD),
            8 => write_note(&mut data, G3_E, PAD),
            16 => write_note(&mut data, A3_E, PAD),
            24 => write_note(&mut data, C4_E, PAD),
            _ => write_empty(&mut data),
        }

        // Ch8: Harmony
        match row {
            31 => write_note(&mut data, F6_E, SUPERSAW),
            _ => write_empty(&mut data),
        }
    }

    data
}

fn generate_pattern_chorus_a() -> Vec<u8> {
    let mut data = Vec::new();

    let bass_pattern: [(u8, u8); 16] = [
        (D2_E, D3_E), (D2_E, D3_E), (D2_E, D3_E), (D2_E, D3_E),
        (BB2_E, BB3_E), (BB2_E, BB3_E), (BB2_E, BB3_E), (BB2_E, BB3_E),
        (C3_E, C4_E), (C3_E, C4_E), (C3_E, C4_E), (C3_E, C4_E),
        (D2_E, D3_E), (D2_E, D3_E), (D2_E, D3_E), (D2_E, D3_E),
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

        // Ch3: Hi-hat (16ths)
        write_note(&mut data, C4_E, HIHAT_E);

        // Ch4: Bass
        if row % 2 == 0 {
            let idx = (row / 2) as usize;
            let (low, high) = bass_pattern[idx];
            let note = if (row / 2) % 2 == 0 { low } else { high };
            write_note(&mut data, note, BASS_E);
        } else {
            write_empty(&mut data);
        }

        // Ch5: THE HOOK
        match row {
            0 => write_note(&mut data, D5_E, SUPERSAW),
            2 => write_note(&mut data, F5_E, SUPERSAW),
            4 => write_note(&mut data, A5_E, SUPERSAW),
            5 => write_note(&mut data, A5_E, SUPERSAW),
            6 => write_note(&mut data, BB5_E, SUPERSAW),
            8 => write_note(&mut data, G5_E, SUPERSAW),
            14 => write_note(&mut data, D5_E, SUPERSAW),
            16 => write_note(&mut data, D5_E, SUPERSAW),
            20 => write_note(&mut data, A5_E, SUPERSAW),
            21 => write_note(&mut data, A5_E, SUPERSAW),
            24 => write_note(&mut data, G5_E, SUPERSAW),
            30 => write_note(&mut data, D5_E, SUPERSAW),
            _ => write_empty(&mut data),
        }

        // Ch6: Brass
        match row {
            7 => write_note(&mut data, D4_E, BRASS),
            15 => write_note(&mut data, BB4_E, BRASS),
            23 => write_note(&mut data, C4_E, BRASS),
            31 => write_note(&mut data, D5_E, BRASS),
            _ => write_empty(&mut data),
        }

        // Ch7: Pad
        match row {
            0 => write_note(&mut data, D3_E, PAD),
            8 => write_note(&mut data, BB3_E, PAD),
            16 => write_note(&mut data, C4_E, PAD),
            24 => write_note(&mut data, D4_E, PAD),
            _ => write_empty(&mut data),
        }

        // Ch8: Harmony
        match row {
            5 => write_note(&mut data, A4_E, SUPERSAW),
            21 => write_note(&mut data, A4_E, SUPERSAW),
            30 => write_note(&mut data, D4_E, SUPERSAW),
            _ => write_empty(&mut data),
        }
    }

    data
}

fn generate_pattern_chorus_b() -> Vec<u8> {
    let mut data = Vec::new();

    let bass_pattern: [(u8, u8); 16] = [
        (D2_E, D3_E), (D2_E, D3_E), (D2_E, D3_E), (D2_E, D3_E),
        (BB2_E, BB3_E), (BB2_E, BB3_E), (BB2_E, BB3_E), (BB2_E, BB3_E),
        (G2_E, G3_E), (G2_E, G3_E), (A2_E, A3_E), (A2_E, A3_E),
        (D2_E, D3_E), (D2_E, D3_E), (D2_E, D3_E), (D2_E, D3_E),
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

        // Ch3: Hi-hat (16ths)
        write_note(&mut data, C4_E, HIHAT_E);

        // Ch4: Bass
        if row % 2 == 0 {
            let idx = (row / 2) as usize;
            let (low, high) = bass_pattern[idx];
            let note = if (row / 2) % 2 == 0 { low } else { high };
            write_note(&mut data, note, BASS_E);
        } else {
            write_empty(&mut data);
        }

        // Ch5: Climax
        match row {
            0 => write_note(&mut data, F5_E, SUPERSAW),
            4 => write_note(&mut data, D6_E, SUPERSAW),
            5 => write_note(&mut data, D6_E, SUPERSAW),
            8 => write_note(&mut data, A5_E, SUPERSAW),
            14 => write_note(&mut data, E5_E, SUPERSAW),
            16 => write_note(&mut data, G5_E, SUPERSAW),
            19 => write_note(&mut data, C6_E, SUPERSAW),
            20 => write_note(&mut data, CS6_E, SUPERSAW),
            22 => write_note(&mut data, D6_E, SUPERSAW),
            24 => write_note(&mut data, D6_E, SUPERSAW),
            26 => write_note(&mut data, F6_E, SUPERSAW),
            28 => write_note(&mut data, A6_E, SUPERSAW),
            29 => write_note(&mut data, A6_E, SUPERSAW),
            30 => write_note(&mut data, G6_E, SUPERSAW),
            31 => write_note(&mut data, D6_E, SUPERSAW),
            _ => write_empty(&mut data),
        }

        // Ch6: Brass
        match row {
            0 => write_note(&mut data, D4_E, BRASS),
            20 => write_note(&mut data, CS5_E, BRASS),
            31 => write_note(&mut data, D5_E, BRASS),
            _ => write_empty(&mut data),
        }

        // Ch7: Pad
        match row {
            0 => write_note(&mut data, D3_E, PAD),
            8 => write_note(&mut data, BB3_E, PAD),
            16 => write_note(&mut data, G3_E, PAD),
            20 => write_note(&mut data, A3_E, PAD),
            24 => write_note(&mut data, D4_E, PAD),
            _ => write_empty(&mut data),
        }

        // Ch8: Harmony
        match row {
            29 => write_note(&mut data, A5_E, SUPERSAW),
            31 => write_note(&mut data, D5_E, SUPERSAW),
            _ => write_empty(&mut data),
        }
    }

    data
}

fn generate_pattern_breakdown() -> Vec<u8> {
    let mut data = Vec::new();

    for row in 0..32 {
        // Ch1: Kick
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

        // Ch4: Bass
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

fn generate_pattern_drop() -> Vec<u8> {
    let mut data = Vec::new();

    let bass_pattern: [(u8, u8); 16] = [
        (D2_E, D3_E), (D2_E, D3_E), (D2_E, D3_E), (D2_E, D3_E),
        (C3_E, C4_E), (C3_E, C4_E), (C3_E, C4_E), (C3_E, C4_E),
        (BB2_E, BB3_E), (BB2_E, BB3_E), (BB2_E, BB3_E), (BB2_E, BB3_E),
        (A2_E, A3_E), (A2_E, A3_E), (A2_E, A3_E), (A2_E, A3_E),
    ];

    for row in 0..32 {
        // Ch1: Kick - DOUBLE TIME
        if row % 2 == 0 {
            write_note(&mut data, C4_E, KICK_E);
        } else {
            write_empty(&mut data);
        }

        // Ch2: Snare
        if row % 8 == 4 {
            write_note(&mut data, C4_E, SNARE_E);
        } else if row % 4 == 2 {
            write_note_vol(&mut data, C4_E, SNARE_E, 0x30);
        } else {
            write_empty(&mut data);
        }

        // Ch3: Hi-hat - FULL 16ths
        write_note(&mut data, C4_E, HIHAT_E);

        // Ch4: Bass
        if row % 2 == 0 {
            let idx = (row / 2) as usize;
            let (low, high) = bass_pattern[idx];
            let note = if (row / 2) % 2 == 0 { low } else { high };
            write_note(&mut data, note, BASS_E);
        } else {
            write_empty(&mut data);
        }

        // Ch5: Hook rhythm
        match row {
            0 => write_note(&mut data, D5_E, SUPERSAW),
            2 => write_note(&mut data, F5_E, SUPERSAW),
            4 => write_note(&mut data, A5_E, SUPERSAW),
            5 => write_note(&mut data, A5_E, SUPERSAW),
            6 => write_note(&mut data, G5_E, SUPERSAW),
            8 => write_note(&mut data, C5_E, SUPERSAW),
            10 => write_note(&mut data, E5_E, SUPERSAW),
            12 => write_note(&mut data, G5_E, SUPERSAW),
            13 => write_note(&mut data, G5_E, SUPERSAW),
            14 => write_note(&mut data, E5_E, SUPERSAW),
            16 => write_note(&mut data, BB4_E, SUPERSAW),
            18 => write_note(&mut data, D5_E, SUPERSAW),
            20 => write_note(&mut data, F5_E, SUPERSAW),
            21 => write_note(&mut data, F5_E, SUPERSAW),
            22 => write_note(&mut data, D5_E, SUPERSAW),
            24 => write_note(&mut data, A4_E, SUPERSAW),
            26 => write_note(&mut data, CS5_E, SUPERSAW),
            28 => write_note(&mut data, E5_E, SUPERSAW),
            29 => write_note(&mut data, E5_E, SUPERSAW),
            30 => write_note(&mut data, A5_E, SUPERSAW),
            _ => write_empty(&mut data),
        }

        // Ch6: Brass
        match row {
            0 => write_note(&mut data, D4_E, BRASS),
            4 => write_note(&mut data, F4_E, BRASS),
            8 => write_note(&mut data, C4_E, BRASS),
            12 => write_note(&mut data, E4_E, BRASS),
            16 => write_note(&mut data, BB3_E, BRASS),
            20 => write_note(&mut data, D4_E, BRASS),
            24 => write_note(&mut data, A3_E, BRASS),
            26 => write_note(&mut data, CS4_E, BRASS),
            28 => write_note(&mut data, E4_E, BRASS),
            30 => write_note(&mut data, A4_E, BRASS),
            _ => write_empty(&mut data),
        }

        // Ch7: Pad
        match row {
            0 => write_note(&mut data, D4_E, PAD),
            8 => write_note(&mut data, C4_E, PAD),
            16 => write_note(&mut data, BB3_E, PAD),
            24 => write_note(&mut data, A3_E, PAD),
            _ => write_empty(&mut data),
        }

        // Ch8: Harmony
        match row {
            0 => write_note(&mut data, D6_E, SUPERSAW),
            5 => write_note(&mut data, A6_E, SUPERSAW),
            8 => write_note(&mut data, C6_E, SUPERSAW),
            13 => write_note(&mut data, G6_E, SUPERSAW),
            16 => write_note(&mut data, BB5_E, SUPERSAW),
            21 => write_note(&mut data, F6_E, SUPERSAW),
            24 => write_note(&mut data, A5_E, SUPERSAW),
            29 => write_note(&mut data, E6_E, SUPERSAW),
            _ => write_empty(&mut data),
        }
    }

    data
}
