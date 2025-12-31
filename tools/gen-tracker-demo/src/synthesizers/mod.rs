//! Audio synthesizers for tracker-demo
//!
//! This module provides procedural audio synthesis for three genres:
//! - **Funk**: "Nether Groove" - Funky Jazz at 110 BPM in F Dorian
//! - **Eurobeat**: "Nether Fire" - Eurobeat at 155 BPM in D minor
//! - **Synthwave**: "Nether Drive" - Synthwave at 105 BPM in A minor

pub mod common;
pub mod funk;
pub mod eurobeat;
pub mod synthwave;

pub use common::{apply_fades, SAMPLE_RATE};

// Re-export all instrument generators
pub use funk::{
    generate_kick_funk, generate_snare_funk, generate_hihat_funk,
    generate_bass_funk, generate_epiano, generate_lead_jazz,
};

pub use eurobeat::{
    generate_kick_euro, generate_snare_euro, generate_hihat_euro,
    generate_bass_euro, generate_supersaw, generate_brass_euro, generate_pad_euro,
};

pub use synthwave::{
    generate_kick_synth, generate_snare_synth, generate_hihat_synth,
    generate_bass_synth, generate_lead_synth, generate_arp_synth, generate_pad_synth,
};
