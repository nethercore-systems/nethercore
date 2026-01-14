//! Synthwave pattern generation
//!
//! Patterns for "Nether Drive" - Synthwave at 105 BPM in A minor

use super::constants::*;
use crate::xm_builder::{write_empty, write_note, write_note_vol};

/// Synthwave Pattern 0: Intro - Synths warming up, atmospheric
pub fn generate_pattern_intro() -> Vec<u8> {
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
pub fn generate_pattern_verse_a() -> Vec<u8> {
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
pub fn generate_pattern_verse_b() -> Vec<u8> {
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
pub fn generate_pattern_chorus_a() -> Vec<u8> {
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
pub fn generate_pattern_chorus_b() -> Vec<u8> {
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
pub fn generate_pattern_bridge() -> Vec<u8> {
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
pub fn generate_pattern_build() -> Vec<u8> {
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
pub fn generate_pattern_outro() -> Vec<u8> {
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
