//! IT file builders for each song
//!
//! This module provides IT file generation for demo songs:
//! - **Nether Acid** - Acid Techno (130 BPM, E minor, 8 channels)
//! - **Nether Dawn** - Epic/Orchestral (90 BPM, D major, 16 channels)
//! - **Nether Storm** - DnB/Action (174 BPM, F minor, 16 channels)

pub mod acid;
pub mod dawn;
pub mod storm;

pub use acid::generate_acid_it;
pub use dawn::generate_dawn_it;
pub use storm::generate_storm_it;

use nether_it::{ItInstrument, ItSample, NewNoteAction};

/// Helper to create an instrument with a linked sample
pub fn make_instrument(name: &str, sample_num: u8) -> ItInstrument {
    let mut instr = ItInstrument::default();
    instr.name = name.to_string();

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
    let mut sample = ItSample::default();
    sample.name = name.to_string();
    sample.c5_speed = c5_speed;
    sample.default_volume = 64;
    sample.global_volume = 64;
    sample
}
