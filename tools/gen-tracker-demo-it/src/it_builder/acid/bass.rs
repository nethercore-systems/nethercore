//! TB-303 bass pattern helper functions for Nether Acid

use super::{A2, B2, CH_303, D3, E2, E3, G2, INST_BASS_303};
use nether_it::{ItNote, ItWriter};

/// Main 303 pattern - classic acid sequence with accents and slides
pub(super) fn add_303_main_pattern(writer: &mut ItWriter, pat: u8, bar: u16) {
    let base = bar * 16;

    // Classic 16th note acid pattern
    // Notes: E-G-A-B pattern with octave jumps
    // Accents (vol 64) trigger filter envelope, no accents (vol 40) stay flat

    let notes = [
        (0, E2, 64),  // Accent - filter opens
        (4, G2, 40),  // No accent
        (8, A2, 64),  // Accent
        (10, B2, 40), // No accent - quick hit
        (12, E3, 64), // Accent - octave jump
        (16, D3, 40), // No accent
        (20, B2, 40), // No accent
        (24, A2, 64), // Accent
        (28, G2, 40), // No accent
        (32, E2, 64), // Accent
        (36, G2, 40), // No accent
        (40, B2, 64), // Accent
        (44, D3, 64), // Accent
        (48, E3, 64), // Accent
        (52, B2, 40), // No accent
    ];

    for (offset, note, vel) in notes {
        writer.set_note(
            pat,
            base + offset,
            CH_303,
            ItNote::play_note(note, INST_BASS_303, vel),
        );
    }
}

/// Simple 303 pattern for intro/outro
pub(super) fn add_303_simple(writer: &mut ItWriter, pat: u8, bar: u16) {
    let base = bar * 16;

    // Just root notes on beats
    writer.set_note(pat, base, CH_303, ItNote::play_note(E2, INST_BASS_303, 50));
    writer.set_note(
        pat,
        base + 8,
        CH_303,
        ItNote::play_note(E2, INST_BASS_303, 50),
    );
}
