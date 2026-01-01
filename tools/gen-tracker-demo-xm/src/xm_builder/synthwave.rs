//! Synthwave XM pattern generation
//!
//! Patterns for "Nether Drive" - Synthwave at 105 BPM in A minor

use super::{
    write_empty, write_instrument, write_instrument_with_sample, write_note, write_note_vol,
};

// ============================================================================
// Synthwave Note Constants (A minor: A B C D E F G, plus G# for E major chord)
// ============================================================================

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
const _GS4_S: u8 = 57; // G#4 for E major chord
const A4_S: u8 = 58;
const B4_S: u8 = 60;
const C5_S: u8 = 61;
const D5_S: u8 = 63;
const E5_S: u8 = 65;

// ============================================================================
// Synthwave Instrument Constants
// ============================================================================

const KICK_S: u8 = 1;
const SNARE_S: u8 = 2;
const HIHAT_S: u8 = 3;
const BASS_S: u8 = 4;
const LEAD_S: u8 = 5;
const ARP_S: u8 = 6;
const PAD_S: u8 = 7;

// ============================================================================
// XM File Generation
// ============================================================================

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

// ============================================================================
// Pattern Generators
// ============================================================================

/// Synthwave Pattern 0: Intro - Synths warming up, atmospheric
fn generate_pattern_intro() -> Vec<u8> {
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
fn generate_pattern_verse_a() -> Vec<u8> {
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
        0, 0, E4_S, 0, D4_S, 0, C4_S, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, C4_S, 0, D4_S, 0, E4_S, 0,
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
fn generate_pattern_verse_b() -> Vec<u8> {
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
        E4_S, 0, D4_S, C4_S, 0, 0, B3_S, 0, A3_S, 0, 0, 0, C4_S, 0, D4_S, 0, E4_S, 0, G4_S, 0,
        E4_S, 0, D4_S, 0, C4_S, 0, B3_S, 0, A3_S, 0, 0, 0,
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
fn generate_pattern_chorus_a() -> Vec<u8> {
    let mut data = Vec::new();

    // Bass: F - G - Am - Am
    let bass_roots = [
        F3_S, F3_S, F3_S, F3_S, F3_S, F3_S, F3_S, F3_S, G3_S, G3_S, G3_S, G3_S, G3_S, G3_S, G3_S,
        G3_S, A2_S, A2_S, A2_S, A2_S, A2_S, A2_S, A2_S, A2_S, A2_S, A2_S, A2_S, A2_S, A2_S, A2_S,
        A2_S, A2_S,
    ];

    // Soaring chorus melody
    let melody = [
        A4_S, 0, C5_S, 0, 0, 0, B4_S, A4_S, G4_S, 0, 0, 0, A4_S, 0, B4_S, 0, C5_S, 0, 0, 0, B4_S,
        0, A4_S, 0, G4_S, 0, E4_S, 0, A4_S, 0, 0, 0,
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
fn generate_pattern_chorus_b() -> Vec<u8> {
    let mut data = Vec::new();

    // Bass: F - G - C - E (major chord for drama)
    let bass_roots = [
        F3_S, F3_S, F3_S, F3_S, F3_S, F3_S, F3_S, F3_S, G3_S, G3_S, G3_S, G3_S, G3_S, G3_S, G3_S,
        G3_S, C3_S, C3_S, C3_S, C3_S, C3_S, C3_S, C3_S, C3_S, E3_S, E3_S, E3_S, E3_S, E3_S, E3_S,
        E3_S, E3_S,
    ];

    // Triumphant melody with higher reach
    let melody = [
        C5_S, 0, E5_S, 0, D5_S, 0, C5_S, 0, B4_S, 0, D5_S, 0, C5_S, 0, B4_S, 0, C5_S, 0, 0, 0,
        E5_S, 0, D5_S, 0, C5_S, 0, B4_S, 0, A4_S, 0, 0, 0,
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
        write_note(
            &mut data,
            arp_patterns[chord_idx][(row % 4) as usize],
            ARP_S,
        );

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
fn generate_pattern_bridge() -> Vec<u8> {
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
            E4_S, 0, 0, 0, D4_S, 0, 0, 0, C4_S, 0, 0, 0, 0, 0, 0, 0, A3_S, 0, 0, 0, B3_S, 0, 0, 0,
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
fn generate_pattern_build() -> Vec<u8> {
    let mut data = Vec::new();

    for row in 0..32 {
        // Ch1: Kick - increasing density
        if row < 16 {
            if row % 8 == 0 {
                write_note(&mut data, C4_S, KICK_S);
            } else {
                write_empty(&mut data);
            }
        } else if row % 4 == 0 {
            write_note(&mut data, C4_S, KICK_S);
        } else {
            write_empty(&mut data);
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
        } else if row % 2 == 0 {
            write_note(&mut data, C4_S, HIHAT_S);
        } else {
            write_empty(&mut data);
        }

        // Ch4: Bass - A pedal building
        if row % 4 == 0 {
            write_note(&mut data, A2_S, BASS_S);
        } else {
            write_empty(&mut data);
        }

        // Ch5: Lead - rising
        let melody = [
            A3_S, 0, 0, 0, B3_S, 0, 0, 0, C4_S, 0, 0, 0, D4_S, 0, 0, 0, E4_S, 0, 0, 0, F4_S, 0, 0,
            0, G4_S, 0, A4_S, 0, B4_S, 0, C5_S, 0,
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
fn generate_pattern_outro() -> Vec<u8> {
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
            A3_S, 0, 0, 0, G3_S, 0, 0, 0, F3_S, 0, 0, 0, E3_S, 0, 0, 0, D3_S, 0, 0, 0, C3_S, 0, 0,
            0, A2_S, 0, 0, 0, 0, 0, 0, 0,
        ];
        let bass = bass_notes[row as usize];
        if bass != 0 {
            write_note(&mut data, bass, BASS_S);
        } else {
            write_empty(&mut data);
        }

        // Ch5: Lead - final phrase
        let melody = [
            E4_S, 0, D4_S, 0, C4_S, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, A3_S, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0,
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
