//! Funk XM generation - "Nether Groove"
//!
//! 110 BPM, F Dorian mode, 6 patterns, 6 instruments

use super::{write_note, write_note_vol, write_empty, write_instrument, write_instrument_with_sample};

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

// ============================================================================
// Pattern Generators
// ============================================================================

/// Funk Pattern 0: Intro - Ghost notes establish groove
fn generate_pattern_intro() -> Vec<u8> {
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
            write_note_vol(&mut data, C4, SNARE_F, 0x18);
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
        if row == 0 || row == 16 {
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
fn generate_pattern_groove_a() -> Vec<u8> {
    let mut data = Vec::new();

    // Syncopated bass line for Fm7 -> Bb7
    let bass_notes = [
        F2, 0, 0, AB2, 0, C3, 0, EB3,
        F2, 0, F3, 0, EB3, 0, C3, 0,
        BB2, 0, 0, D3, 0, F3, 0, AB3,
        BB2, 0, BB3, 0, AB3, 0, F3, 0,
    ];

    // Call melody
    let melody = [
        0, 0, 0, 0, C5, 0, EB5, 0,
        F5, 0, EB5, 0, C5, 0, 0, 0,
        0, 0, 0, 0, D4, 0, F4, 0,
        AB4, 0, F4, 0, D4, 0, 0, 0,
    ];

    // Response melody
    let response = [
        0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, AB4, 0,
        C5, 0, 0, 0, 0, 0, 0, 0,
        0, 0, BB4, 0, AB4, 0, F4, 0,
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
            write_note(&mut data, C4, SNARE_F);
        } else if row == 4 || row == 12 || row == 20 || row == 28 {
            write_note_vol(&mut data, C4, SNARE_F, 0x15);
        } else {
            write_empty(&mut data);
        }

        // Ch3: Hi-hat - 8th notes with accents
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

        // Ch5: Lead melody (call)
        let mel = melody[row as usize];
        if mel != 0 {
            write_note(&mut data, mel, LEAD_J);
        } else {
            write_empty(&mut data);
        }

        // Ch6: EPiano - chord comping
        if row == 0 {
            write_note(&mut data, C4, EPIANO);
        } else if row == 2 {
            write_note(&mut data, EB4, EPIANO);
        } else if row == 16 {
            write_note(&mut data, D4, EPIANO);
        } else if row == 18 {
            write_note(&mut data, F4, EPIANO);
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
fn generate_pattern_groove_b() -> Vec<u8> {
    let mut data = Vec::new();

    // Bass line Eb7 -> Fm7
    let bass_notes = [
        EB3, 0, 0, G3, 0, BB3, 0, 0,
        EB3, 0, 0, 0, D3, 0, EB3, 0,
        F2, 0, 0, AB2, 0, C3, 0, EB3,
        F2, 0, F3, 0, C3, 0, AB2, 0,
    ];

    // Counter melody
    let melody = [
        BB4, 0, G4, 0, EB4, 0, 0, 0,
        0, 0, D4, 0, EB4, 0, G4, 0,
        AB4, 0, 0, 0, C5, 0, AB4, 0,
        F4, 0, 0, 0, 0, 0, 0, 0,
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
            write_note_vol(&mut data, C4, SNARE_F, 0x30);
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
            write_note(&mut data, G4, EPIANO);
        } else if row == 16 {
            write_note(&mut data, AB4, EPIANO);
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
fn generate_pattern_bridge() -> Vec<u8> {
    let mut data = Vec::new();

    // Chromatic walking bass
    let bass_notes = [
        F2, 0, 0, 0, G2, 0, 0, 0,
        AB2, 0, 0, 0, BB2, 0, 0, 0,
        C3, 0, 0, 0, D3, 0, 0, 0,
        EB3, 0, D3, 0, C3, 0, BB2, 0,
    ];

    // Jazz runs
    let melody = [
        C5, EB5, F5, 0, EB5, C5, BB4, 0,
        AB4, 0, G4, 0, F4, 0, 0, 0,
        C5, 0, D4, 0, EB4, 0, F4, 0,
        G4, AB4, BB4, C5, EB5, 0, F5, 0,
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
            write_note_vol(&mut data, C4, SNARE_F, 0x35);
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
fn generate_pattern_solo() -> Vec<u8> {
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
fn generate_pattern_outro() -> Vec<u8> {
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
            write_note(&mut data, F3, EPIANO);
        } else {
            write_empty(&mut data);
        }

        // Ch7-8: Empty
        write_empty(&mut data);
        write_empty(&mut data);
    }

    data
}
