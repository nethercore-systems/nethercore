//! Funk pattern generators for "Nether Groove"

use super::super::{write_empty, write_note, write_note_vol};
use super::*;

/// Funk Pattern 0: Intro - Ghost notes establish groove
pub(super) fn generate_pattern_intro() -> Vec<u8> {
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
pub(super) fn generate_pattern_groove_a() -> Vec<u8> {
    let mut data = Vec::new();

    // Syncopated bass line for Fm7 -> Bb7
    let bass_notes = [
        F2, 0, 0, AB2, 0, C3, 0, EB3, F2, 0, F3, 0, EB3, 0, C3, 0, BB2, 0, 0, D3, 0, F3, 0, AB3,
        BB2, 0, BB3, 0, AB3, 0, F3, 0,
    ];

    // Call melody
    let melody = [
        0, 0, 0, 0, C5, 0, EB5, 0, F5, 0, EB5, 0, C5, 0, 0, 0, 0, 0, 0, 0, D4, 0, F4, 0, AB4, 0,
        F4, 0, D4, 0, 0, 0,
    ];

    // Response melody
    let response = [
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, AB4, 0, C5, 0, 0, 0, 0, 0, 0, 0, 0, 0, BB4, 0,
        AB4, 0, F4, 0,
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
pub(super) fn generate_pattern_groove_b() -> Vec<u8> {
    let mut data = Vec::new();

    // Bass line Eb7 -> Fm7
    let bass_notes = [
        EB3, 0, 0, G3, 0, BB3, 0, 0, EB3, 0, 0, 0, D3, 0, EB3, 0, F2, 0, 0, AB2, 0, C3, 0, EB3, F2,
        0, F3, 0, C3, 0, AB2, 0,
    ];

    // Counter melody
    let melody = [
        BB4, 0, G4, 0, EB4, 0, 0, 0, 0, 0, D4, 0, EB4, 0, G4, 0, AB4, 0, 0, 0, C5, 0, AB4, 0, F4,
        0, 0, 0, 0, 0, 0, 0,
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
pub(super) fn generate_pattern_bridge() -> Vec<u8> {
    let mut data = Vec::new();

    // Chromatic walking bass
    let bass_notes = [
        F2, 0, 0, 0, G2, 0, 0, 0, AB2, 0, 0, 0, BB2, 0, 0, 0, C3, 0, 0, 0, D3, 0, 0, 0, EB3, 0, D3,
        0, C3, 0, BB2, 0,
    ];

    // Jazz runs
    let melody = [
        C5, EB5, F5, 0, EB5, C5, BB4, 0, AB4, 0, G4, 0, F4, 0, 0, 0, C5, 0, D4, 0, EB4, 0, F4, 0,
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
pub(super) fn generate_pattern_solo() -> Vec<u8> {
    let mut data = Vec::new();

    // Vamp bass on Fm7
    let bass_notes = [
        F2, 0, 0, AB2, 0, C3, 0, 0, F2, 0, 0, 0, C3, 0, AB2, 0, F2, 0, 0, AB2, 0, C3, 0, EB3, F3,
        0, EB3, 0, C3, 0, AB2, 0,
    ];

    // EP "solo" - improvisatory feel
    let ep_solo = [
        C5, 0, AB4, 0, F4, 0, 0, 0, AB4, C5, EB5, 0, C5, 0, 0, 0, F5, 0, EB5, 0, C5, 0, AB4, 0,
        BB4, 0, AB4, 0, F4, 0, 0, 0,
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
pub(super) fn generate_pattern_outro() -> Vec<u8> {
    let mut data = Vec::new();

    // Descending bass
    let bass_notes = [
        F3, 0, 0, 0, EB3, 0, 0, 0, C3, 0, 0, 0, BB2, 0, 0, 0, AB2, 0, 0, 0, G2, 0, 0, 0, F2, 0, 0,
        0, 0, 0, 0, 0,
    ];

    // Final melody phrase
    let melody = [
        C5, 0, AB4, 0, F4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, AB4, 0, F4, 0, C4, 0, 0, 0, F4, 0, 0,
        0, 0, 0, 0, 0,
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
