//! Eurobeat XM generation - "Nether Fire"
//!
//! 155 BPM, D minor, 8 patterns, 7 instruments

use super::{
    write_empty, write_instrument, write_instrument_with_sample, write_note, write_note_fx,
    write_note_vol,
};

// XM effect constants
const FX_NOTE_CUT: u8 = 0x0C; // ECx - cut note at tick x

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
const DS4_E: u8 = 52;
const E4_E: u8 = 53;
const FS4_E: u8 = 55;
const F4_E: u8 = 54;
const G4_E: u8 = 56;
const A4_E: u8 = 58;
const BB4_E: u8 = 59;
const C5_E: u8 = 61;
const CS5_E: u8 = 62;
const D5_E: u8 = 63;
const E5_E: u8 = 65;
const F5_E: u8 = 66;
const FS5_E: u8 = 67;
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

// ============================================================================
// Pattern Generators
// ============================================================================

fn generate_pattern_intro() -> Vec<u8> {
    let mut data = Vec::new();

    for row in 0..32 {
        // Ch1: Kick - sparse, building
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

        // Ch2: Snare - enters late
        if row >= 24 && row % 8 == 4 {
            write_note(&mut data, C4_E, SNARE_E);
        } else {
            write_empty(&mut data);
        }

        // Ch3: Hi-hat - gradual build
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

        // Ch4: Bass - Dm pedal, then octave bounce
        if row == 0 {
            write_note(&mut data, D2_E, BASS_E);
        } else if row >= 16 && row % 2 == 0 {
            let note = if (row / 2) % 2 == 0 { D2_E } else { D3_E };
            write_note(&mut data, note, BASS_E);
        } else {
            write_empty(&mut data);
        }

        // Ch5: Supersaw - Hook teaser (D-F-A arpeggio motif)
        // Simple ascending: D→F→A, then D→F→G→A
        match row {
            0 => write_note(&mut data, D5_E, SUPERSAW),
            4 => write_note(&mut data, F5_E, SUPERSAW),
            8 => write_note(&mut data, A5_E, SUPERSAW),
            16 => write_note(&mut data, D5_E, SUPERSAW),
            20 => write_note(&mut data, F5_E, SUPERSAW),
            24 => write_note(&mut data, G5_E, SUPERSAW),
            28 => write_note(&mut data, A5_E, SUPERSAW),
            _ => write_empty(&mut data),
        }

        // Ch6: Brass - Single stab answers final hook teaser note
        // Lead ends at row 28 (A5), brass answers at row 30
        match row {
            30 => write_note_fx(&mut data, A3_E, BRASS, FX_NOTE_CUT, 0x04), // Answer hook teaser
            _ => write_empty(&mut data),
        }

        // Ch7: Pad - Dm swell
        match row {
            0 => write_note(&mut data, D3_E, PAD),
            16 => write_note(&mut data, F3_E, PAD),
            _ => write_empty(&mut data),
        }

        // Ch8: Harmony - Silent in intro
        write_empty(&mut data);
    }

    data
}

fn generate_pattern_verse_a() -> Vec<u8> {
    let mut data = Vec::new();

    // Chord: Dm → G → Bb → A
    let bass_pattern: [(u8, u8); 16] = [
        (D2_E, D3_E),
        (D2_E, D3_E),
        (D2_E, D3_E),
        (D2_E, D3_E),
        (G2_E, G3_E),
        (G2_E, G3_E),
        (G2_E, G3_E),
        (G2_E, G3_E),
        (BB2_E, BB3_E),
        (BB2_E, BB3_E),
        (BB2_E, BB3_E),
        (BB2_E, BB3_E),
        (A2_E, A3_E),
        (A2_E, A3_E),
        (A2_E, A3_E),
        (A2_E, A3_E),
    ];

    for row in 0..32 {
        // Ch1: Kick - 4-on-floor
        if row % 4 == 0 {
            write_note(&mut data, C4_E, KICK_E);
        } else {
            write_empty(&mut data);
        }

        // Ch2: Snare - 2 and 4
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

        // Ch4: Bass - octave bounce
        if row % 2 == 0 {
            let idx = (row / 2) as usize;
            let (low, high) = bass_pattern[idx];
            let note = if (row / 2) % 2 == 0 { low } else { high };
            write_note(&mut data, note, BASS_E);
        } else {
            write_empty(&mut data);
        }

        // Ch5: Supersaw - Simple call-and-response melody
        // Phrase 1 (0-14): A→D→D (establishing tonic)
        // Phrase 2 (16-30): Bb→F→A (answering, leading back)
        match row {
            0 => write_note(&mut data, A4_E, SUPERSAW),
            6 => write_note(&mut data, D5_E, SUPERSAW),
            12 => write_note(&mut data, D5_E, SUPERSAW),
            16 => write_note(&mut data, BB4_E, SUPERSAW),
            22 => write_note(&mut data, F4_E, SUPERSAW),
            28 => write_note(&mut data, A4_E, SUPERSAW),
            _ => write_empty(&mut data),
        }

        // Ch6: Brass - CALL AND RESPONSE (sparse in verse, building energy)
        // Lead phrases end at rows 12 and 28
        // Single brass answer at row 30 only (building toward chorus)
        match row {
            30 => write_note_fx(&mut data, A3_E, BRASS, FX_NOTE_CUT, 0x04), // Answer phrase 2
            _ => write_empty(&mut data),
        }

        // Ch7: Pad - Chord changes
        match row {
            0 => write_note(&mut data, D3_E, PAD),
            16 => write_note(&mut data, G3_E, PAD),
            _ => write_empty(&mut data),
        }

        // Ch8: Harmony - Light support
        match row {
            12 => write_note(&mut data, A4_E, SUPERSAW),
            _ => write_empty(&mut data),
        }
    }

    data
}

fn generate_pattern_verse_b() -> Vec<u8> {
    let mut data = Vec::new();

    // Chord: Dm → C → Bb → C
    let bass_pattern: [(u8, u8); 16] = [
        (D2_E, D3_E),
        (D2_E, D3_E),
        (D2_E, D3_E),
        (D2_E, D3_E),
        (C3_E, C4_E),
        (C3_E, C4_E),
        (C3_E, C4_E),
        (C3_E, C4_E),
        (BB2_E, BB3_E),
        (BB2_E, BB3_E),
        (BB2_E, BB3_E),
        (BB2_E, BB3_E),
        (C3_E, C4_E),
        (C3_E, C4_E),
        (C3_E, C4_E),
        (C3_E, C4_E),
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

        // Ch4: Bass - octave bounce
        if row % 2 == 0 {
            let idx = (row / 2) as usize;
            let (low, high) = bass_pattern[idx];
            let note = if (row / 2) % 2 == 0 { low } else { high };
            write_note(&mut data, note, BASS_E);
        } else {
            write_empty(&mut data);
        }

        // Ch5: Supersaw - More active, building energy
        // Phrase 1: D→E→F then E→D (rise and fall)
        // Phrase 2: Bb→C→D→E→F (ascending, setting up pre-chorus)
        match row {
            0 => write_note(&mut data, D5_E, SUPERSAW),
            4 => write_note(&mut data, E5_E, SUPERSAW),
            6 => write_note(&mut data, F5_E, SUPERSAW),
            12 => write_note(&mut data, E5_E, SUPERSAW),
            14 => write_note(&mut data, D5_E, SUPERSAW),
            16 => write_note(&mut data, BB4_E, SUPERSAW),
            20 => write_note(&mut data, C5_E, SUPERSAW),
            22 => write_note(&mut data, D5_E, SUPERSAW),
            28 => write_note(&mut data, E5_E, SUPERSAW),
            30 => write_note(&mut data, F5_E, SUPERSAW),
            _ => write_empty(&mut data),
        }

        // Ch6: Brass - CALL AND RESPONSE (more energy building to pre-chorus)
        // Lead phrases: ends ~14, ends ~30
        // Two stabs: answer phrase 1, answer phrase 2
        match row {
            14 => write_note_fx(&mut data, D4_E, BRASS, FX_NOTE_CUT, 0x04), // Answer phrase 1
            26 => write_note_fx(&mut data, C4_E, BRASS, FX_NOTE_CUT, 0x04), // Build to phrase 2 end
            _ => write_empty(&mut data),
        }

        // Ch7: Pad
        match row {
            0 => write_note(&mut data, D3_E, PAD),
            16 => write_note(&mut data, C4_E, PAD),
            _ => write_empty(&mut data),
        }

        // Ch8: Harmony - Third below on key notes
        match row {
            6 => write_note(&mut data, D5_E, SUPERSAW),
            22 => write_note(&mut data, BB4_E, SUPERSAW),
            _ => write_empty(&mut data),
        }
    }

    data
}

fn generate_pattern_prechorus() -> Vec<u8> {
    let mut data = Vec::new();

    // Chord: F → G → A pedal (building tension)

    for row in 0..32 {
        // Ch1: Kick - builds to double-time
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

        // Ch2: Snare - builds with rolls
        if row < 24 {
            if row % 8 == 4 {
                write_note(&mut data, C4_E, SNARE_E);
            } else {
                write_empty(&mut data);
            }
        } else {
            // Snare roll in last 8 rows
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
            write_note(&mut data, C4_E, HIHAT_E);
        }

        // Ch4: Bass - F → G → A pedal
        let bass_note = match row {
            0..=7 => {
                if (row / 2) % 2 == 0 {
                    F2_E
                } else {
                    F3_E
                }
            }
            8..=15 => {
                if (row / 2) % 2 == 0 {
                    G2_E
                } else {
                    G3_E
                }
            }
            16..=31 => {
                if (row / 2) % 2 == 0 {
                    A2_E
                } else {
                    A3_E
                }
            }
            _ => A2_E,
        };
        if row % 2 == 0 {
            write_note(&mut data, bass_note, BASS_E);
        } else {
            write_empty(&mut data);
        }

        // Ch5: Supersaw - Rising intensity, accelerating rhythm
        // Phrase 1 (0-14): A→C→E→F (rising through F major)
        // Phrase 2 (16-30): G→G→A→A→Bb→A (climbs to Bb, pulls back to A = leading tone!)
        match row {
            0 => write_note(&mut data, A4_E, SUPERSAW),
            4 => write_note(&mut data, C5_E, SUPERSAW),
            8 => write_note(&mut data, E5_E, SUPERSAW),
            12 => write_note(&mut data, E5_E, SUPERSAW),
            14 => write_note(&mut data, F5_E, SUPERSAW),
            16 => write_note(&mut data, G5_E, SUPERSAW),
            20 => write_note(&mut data, G5_E, SUPERSAW),
            22 => write_note(&mut data, A5_E, SUPERSAW),
            26 => write_note(&mut data, A5_E, SUPERSAW),
            28 => write_note(&mut data, BB5_E, SUPERSAW),
            30 => write_note(&mut data, A5_E, SUPERSAW), // Leading tone!
            _ => write_empty(&mut data),
        }

        // Ch6: Brass - MAX ENERGY call-and-response before chorus
        // Lead rises: 0→4→8, 12→14, 16→20→22, 26→28→30
        // Brass answers each mini-phrase with punchy stabs
        match row {
            6 => write_note_fx(&mut data, F3_E, BRASS, FX_NOTE_CUT, 0x04),  // Answer first rise
            14 => write_note_fx(&mut data, F3_E, BRASS, FX_NOTE_CUT, 0x04), // Answer E→F
            22 => write_note_fx(&mut data, G3_E, BRASS, FX_NOTE_CUT, 0x04), // Answer G rise
            30 => write_note_fx(&mut data, A3_E, BRASS, FX_NOTE_CUT, 0x04), // Final answer (leading tone tension!)
            _ => write_empty(&mut data),
        }

        // Ch7: Pad - Swelling
        match row {
            0 => write_note(&mut data, F3_E, PAD),
            16 => write_note(&mut data, G3_E, PAD),
            _ => write_empty(&mut data),
        }

        // Ch8: Harmony - Building to unison at climax
        match row {
            28 => write_note(&mut data, A5_E, SUPERSAW),
            30 => write_note(&mut data, A5_E, SUPERSAW),
            _ => write_empty(&mut data),
        }
    }

    data
}

fn generate_pattern_chorus_a() -> Vec<u8> {
    let mut data = Vec::new();

    // Chord: Dm → Bb → C → Dm
    // THE MAIN HOOK - Dave Rodgers style: soaring rise to peak, then descending resolution
    let bass_pattern: [(u8, u8); 16] = [
        (D2_E, D3_E),
        (D2_E, D3_E),
        (D2_E, D3_E),
        (D2_E, D3_E),
        (BB2_E, BB3_E),
        (BB2_E, BB3_E),
        (BB2_E, BB3_E),
        (BB2_E, BB3_E),
        (C3_E, C4_E),
        (C3_E, C4_E),
        (C3_E, C4_E),
        (C3_E, C4_E),
        (D2_E, D3_E),
        (D2_E, D3_E),
        (D2_E, D3_E),
        (D2_E, D3_E),
    ];

    for row in 0..32 {
        // Ch1: Kick - 4-on-floor
        if row % 4 == 0 {
            write_note(&mut data, C4_E, KICK_E);
        } else {
            write_empty(&mut data);
        }

        // Ch2: Snare - 2 and 4
        if row % 8 == 4 {
            write_note(&mut data, C4_E, SNARE_E);
        } else {
            write_empty(&mut data);
        }

        // Ch3: Hi-hat - driving 16ths
        write_note(&mut data, C4_E, HIHAT_E);

        // Ch4: Bass - octave bounce
        if row % 2 == 0 {
            let idx = (row / 2) as usize;
            let (low, high) = bass_pattern[idx];
            let note = if (row / 2) % 2 == 0 { low } else { high };
            write_note(&mut data, note, BASS_E);
        } else {
            write_empty(&mut data);
        }

        // Ch5: THE HOOK - Soaring phrase
        // Phrase 1 (0-14): D→D→E→F→A (RISE to peak on A5, HOLD)
        // Phrase 2 (16-30): A→G→F→E→D (DESCEND, resolve to tonic)
        match row {
            0 => write_note(&mut data, D5_E, SUPERSAW),
            4 => write_note(&mut data, D5_E, SUPERSAW),
            6 => write_note(&mut data, E5_E, SUPERSAW),
            8 => write_note(&mut data, F5_E, SUPERSAW),
            12 => write_note(&mut data, A5_E, SUPERSAW), // PEAK!
            16 => write_note(&mut data, A5_E, SUPERSAW), // HOLD peak
            20 => write_note(&mut data, G5_E, SUPERSAW),
            22 => write_note(&mut data, F5_E, SUPERSAW),
            24 => write_note(&mut data, E5_E, SUPERSAW),
            28 => write_note(&mut data, D5_E, SUPERSAW), // RESOLVE
            _ => write_empty(&mut data),
        }

        // Ch6: Brass - CALL AND RESPONSE with lead melody
        // Lead phrases: Phrase 1 peaks at row 12, Phrase 2 resolves at row 28
        // Brass ANSWERS at: row 14 (after peak), row 30 (after resolve)
        // Note-cut at tick 4 for punchy stabs (don't ring)
        match row {
            14 => write_note_fx(&mut data, A3_E, BRASS, FX_NOTE_CUT, 0x04), // Answer phrase 1 (A = 5th of Dm)
            30 => write_note_fx(&mut data, D4_E, BRASS, FX_NOTE_CUT, 0x04), // Answer phrase 2 (D = root, resolution)
            _ => write_empty(&mut data),
        }

        // Ch7: Pad - Full chords
        match row {
            0 => write_note(&mut data, D3_E, PAD),
            16 => write_note(&mut data, BB3_E, PAD),
            _ => write_empty(&mut data),
        }

        // Ch8: Harmony - Octave doubling on PEAK and RESOLUTION only
        match row {
            12 => write_note(&mut data, A4_E, SUPERSAW), // Double peak
            16 => write_note(&mut data, A4_E, SUPERSAW), // Hold peak double
            28 => write_note(&mut data, D4_E, SUPERSAW), // Double resolution
            _ => write_empty(&mut data),
        }
    }

    data
}

fn generate_pattern_chorus_b() -> Vec<u8> {
    let mut data = Vec::new();

    // Chord: Dm → Bb → G → A → D MAJOR (Picardy third!)
    // CLIMAX - Same shape as Chorus A but HIGHER, ends on D MAJOR
    let bass_pattern: [(u8, u8); 16] = [
        (D2_E, D3_E),
        (D2_E, D3_E),
        (D2_E, D3_E),
        (D2_E, D3_E),
        (BB2_E, BB3_E),
        (BB2_E, BB3_E),
        (BB2_E, BB3_E),
        (BB2_E, BB3_E),
        (G2_E, G3_E),
        (G2_E, G3_E),
        (A2_E, A3_E),
        (A2_E, A3_E),
        (D2_E, D3_E),
        (D2_E, D3_E),
        (D2_E, D3_E),
        (D2_E, D3_E),
    ];

    for row in 0..32 {
        // Ch1: Kick - 4-on-floor
        if row % 4 == 0 {
            write_note(&mut data, C4_E, KICK_E);
        } else {
            write_empty(&mut data);
        }

        // Ch2: Snare - 2 and 4
        if row % 8 == 4 {
            write_note(&mut data, C4_E, SNARE_E);
        } else {
            write_empty(&mut data);
        }

        // Ch3: Hi-hat - driving 16ths
        write_note(&mut data, C4_E, HIHAT_E);

        // Ch4: Bass - octave bounce
        if row % 2 == 0 {
            let idx = (row / 2) as usize;
            let (low, high) = bass_pattern[idx];
            let note = if (row / 2) % 2 == 0 { low } else { high };
            write_note(&mut data, note, BASS_E);
        } else {
            write_empty(&mut data);
        }

        // Ch5: Climax hook - HIGHER, ends on F# (Picardy!)
        // Phrase 1 (0-14): D→F→G→A→D6 (FASTER rise to D6 - OCTAVE ABOVE TONIC!)
        // Phrase 2 (16-30): D6→C→Bb→A→F# (descend, end on MAJOR THIRD = TRIUMPHANT!)
        match row {
            0 => write_note(&mut data, D5_E, SUPERSAW),
            4 => write_note(&mut data, F5_E, SUPERSAW),
            6 => write_note(&mut data, G5_E, SUPERSAW),
            8 => write_note(&mut data, A5_E, SUPERSAW),
            12 => write_note(&mut data, D6_E, SUPERSAW), // PEAK on D6!
            16 => write_note(&mut data, D6_E, SUPERSAW), // HOLD peak
            20 => write_note(&mut data, C6_E, SUPERSAW),
            22 => write_note(&mut data, BB5_E, SUPERSAW),
            24 => write_note(&mut data, A5_E, SUPERSAW),
            28 => write_note(&mut data, FS5_E, SUPERSAW), // PICARDY THIRD!
            _ => write_empty(&mut data),
        }

        // Ch6: Brass - CALL AND RESPONSE (same pattern as Chorus A)
        // Lead phrases: Phrase 1 peaks at row 12 (D6!), Phrase 2 resolves at row 28 (F#5 Picardy)
        // Brass ANSWERS at: row 14 (after peak), row 30 (after Picardy)
        match row {
            14 => write_note_fx(&mut data, D4_E, BRASS, FX_NOTE_CUT, 0x04), // Answer phrase 1 (D = octave below peak)
            30 => write_note_fx(&mut data, FS4_E, BRASS, FX_NOTE_CUT, 0x04), // Answer phrase 2 (F# = Picardy harmony!)
            _ => write_empty(&mut data),
        }

        // Ch7: Pad - Ends on D major chord
        match row {
            0 => write_note(&mut data, D3_E, PAD),
            16 => write_note(&mut data, BB3_E, PAD),
            28 => write_note(&mut data, D4_E, PAD), // D major
            _ => write_empty(&mut data),
        }

        // Ch8: Harmony - Power doubling on peak and Picardy resolution
        match row {
            12 => write_note(&mut data, D5_E, SUPERSAW), // Double peak
            16 => write_note(&mut data, D5_E, SUPERSAW), // Hold peak double
            28 => write_note(&mut data, FS4_E, SUPERSAW), // F# harmony = D MAJOR!
            _ => write_empty(&mut data),
        }
    }

    data
}

fn generate_pattern_breakdown() -> Vec<u8> {
    let mut data = Vec::new();

    // Atmospheric stripped hook - Dm pedal → Bb → A
    // Space to breathe before the drop

    for row in 0..32 {
        // Ch1: Kick - very sparse
        if row == 0 || row == 16 {
            write_note(&mut data, C4_E, KICK_E);
        } else {
            write_empty(&mut data);
        }

        // Ch2: Snare - silent
        write_empty(&mut data);

        // Ch3: Hi-hat - sparse, quiet
        if row % 8 == 0 {
            write_note_vol(&mut data, C4_E, HIHAT_E, 0x20);
        } else {
            write_empty(&mut data);
        }

        // Ch4: Bass - sustained Dm
        if row == 0 {
            write_note(&mut data, D2_E, BASS_E);
        } else {
            write_empty(&mut data);
        }

        // Ch5: Atmospheric - just D and F (echo of hook's DNA)
        match row {
            0 => write_note_vol(&mut data, D5_E, SUPERSAW, 0x25),
            16 => write_note_vol(&mut data, F5_E, SUPERSAW, 0x25),
            _ => write_empty(&mut data),
        }

        // Ch6: Brass - Silent in breakdown
        write_empty(&mut data);

        // Ch7: Pad - Ambient swell: Dm → F → A (building to drop)
        match row {
            0 => write_note(&mut data, D3_E, PAD),
            16 => write_note(&mut data, F3_E, PAD),
            28 => write_note(&mut data, A3_E, PAD), // Building tension
            _ => write_empty(&mut data),
        }

        // Ch8: Harmony - D6 anticipating the drop at very end
        match row {
            30 => write_note(&mut data, D6_E, SUPERSAW),
            _ => write_empty(&mut data),
        }
    }

    data
}

fn generate_pattern_drop() -> Vec<u8> {
    let mut data = Vec::new();

    // MAXIMUM ENERGY - Fast arpeggios with clear patterns
    // Chord: Dm → C → Bb → A (A major for tension at end!)
    let bass_pattern: [(u8, u8); 16] = [
        (D2_E, D3_E),
        (D2_E, D3_E),
        (D2_E, D3_E),
        (D2_E, D3_E),
        (C3_E, C4_E),
        (C3_E, C4_E),
        (C3_E, C4_E),
        (C3_E, C4_E),
        (BB2_E, BB3_E),
        (BB2_E, BB3_E),
        (BB2_E, BB3_E),
        (BB2_E, BB3_E),
        (A2_E, A3_E),
        (A2_E, A3_E),
        (A2_E, A3_E),
        (A2_E, A3_E),
    ];

    // Fast arpeggio patterns (16th notes = every row)
    // Dm arp: D-F-A-D, C arp: C-E-G-C, Bb arp: Bb-D-F-Bb, A arp: A-C#-E-A
    let lead_arp: [u8; 32] = [
        // Rows 0-7: Dm arpeggio x2
        D5_E, F5_E, A5_E, D6_E, D5_E, F5_E, A5_E, D6_E,
        // Rows 8-15: C major arpeggio x2
        C5_E, E5_E, G5_E, C6_E, C5_E, E5_E, G5_E, C6_E,
        // Rows 16-23: Bb major arpeggio x2
        BB4_E, D5_E, F5_E, BB5_E, BB4_E, D5_E, F5_E, BB5_E,
        // Rows 24-31: A major arpeggio x2 (tension!)
        A4_E, CS5_E, E5_E, A5_E, A4_E, CS5_E, E5_E, A5_E,
    ];

    for row in 0..32 {
        // Ch1: Kick - DOUBLE TIME (every 2 rows)
        if row % 2 == 0 {
            write_note(&mut data, C4_E, KICK_E);
        } else {
            write_empty(&mut data);
        }

        // Ch2: Snare - with ghost notes
        if row % 8 == 4 {
            write_note(&mut data, C4_E, SNARE_E);
        } else if row % 4 == 2 {
            write_note_vol(&mut data, C4_E, SNARE_E, 0x30);
        } else {
            write_empty(&mut data);
        }

        // Ch3: Hi-hat - FULL 16ths
        write_note(&mut data, C4_E, HIHAT_E);

        // Ch4: Bass - octave bounce
        if row % 2 == 0 {
            let idx = (row / 2) as usize;
            let (low, high) = bass_pattern[idx];
            let note = if (row / 2) % 2 == 0 { low } else { high };
            write_note(&mut data, note, BASS_E);
        } else {
            write_empty(&mut data);
        }

        // Ch5: Lead - Fast ascending arpeggios (every row = 16th notes)
        write_note(&mut data, lead_arp[row as usize], SUPERSAW);

        // Ch6: Brass - DOWNBEAT stabs marking chord changes (different from verse/chorus)
        // In drop, arpeggios are busy so brass marks structure on DOWNBEATS
        // Note-cut for punchy stabs that don't interfere with arpeggios
        match row {
            0 => write_note_fx(&mut data, D4_E, BRASS, FX_NOTE_CUT, 0x03),   // Dm start (shorter cut)
            8 => write_note_fx(&mut data, C4_E, BRASS, FX_NOTE_CUT, 0x03),   // C chord
            16 => write_note_fx(&mut data, BB3_E, BRASS, FX_NOTE_CUT, 0x03), // Bb chord
            24 => write_note_fx(&mut data, A3_E, BRASS, FX_NOTE_CUT, 0x03),  // A chord
            _ => write_empty(&mut data),
        }

        // Ch7: Pad - Full power chords
        match row {
            0 => write_note(&mut data, D4_E, PAD),
            16 => write_note(&mut data, C4_E, PAD),
            _ => write_empty(&mut data),
        }

        // Ch8: Harmony - Octave doubling on peaks of each arpeggio
        match row {
            3 => write_note(&mut data, D5_E, SUPERSAW),  // Dm peak
            7 => write_note(&mut data, D5_E, SUPERSAW),
            11 => write_note(&mut data, C5_E, SUPERSAW), // C peak
            15 => write_note(&mut data, C5_E, SUPERSAW),
            19 => write_note(&mut data, BB4_E, SUPERSAW), // Bb peak
            23 => write_note(&mut data, BB4_E, SUPERSAW),
            27 => write_note(&mut data, A4_E, SUPERSAW),  // A peak
            31 => write_note(&mut data, A4_E, SUPERSAW),
            _ => write_empty(&mut data),
        }
    }

    data
}
