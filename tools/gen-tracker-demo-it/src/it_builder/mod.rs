//! IT file builders for each song
//!
//! This module provides IT file generation for demo songs:
//! - **Nether Acid** - Acid Techno (130 BPM, E minor, 8 channels)
//! - **Nether Dawn** - Epic/Orchestral (90 BPM, D major, 16 channels)
//! - **Nether Storm** - DnB/Action (174 BPM, F minor, 16 channels)

pub mod acid;
pub mod dawn;
pub mod storm;

// Stripped and embedded variants
pub use acid::{generate_acid_it_embedded, generate_acid_it_stripped};
pub use dawn::{generate_dawn_it_embedded, generate_dawn_it_stripped};
pub use storm::{generate_storm_it_embedded, generate_storm_it_stripped};

use nether_it::{ItInstrument, ItSample, NewNoteAction};

/// Helper to create an instrument with a linked sample
pub fn make_instrument(name: &str, sample_num: u8) -> ItInstrument {
    let mut instr = ItInstrument {
        name: name.to_string(),
        ..Default::default()
    };

    // Map all notes to use this sample
    for entry in instr.note_sample_table.iter_mut() {
        entry.1 = sample_num;
    }

    instr
}

/// Helper to create an instrument with NNA Continue (for polyphony)
pub fn make_instrument_continue(name: &str, sample_num: u8) -> ItInstrument {
    let mut instr = make_instrument(name, sample_num);
    instr.nna = NewNoteAction::Continue;
    instr
}

/// Helper to create an instrument with NNA Fade (for smooth transitions)
pub fn make_instrument_fade(name: &str, sample_num: u8, fadeout: u16) -> ItInstrument {
    let mut instr = make_instrument(name, sample_num);
    instr.nna = NewNoteAction::NoteFade;
    instr.fadeout = fadeout;
    instr
}

/// Helper to create a sample definition
pub fn make_sample(name: &str, c5_speed: u32) -> ItSample {
    ItSample {
        name: name.to_string(),
        c5_speed,
        default_volume: 64,
        global_volume: 64,
        ..Default::default()
    }
}
