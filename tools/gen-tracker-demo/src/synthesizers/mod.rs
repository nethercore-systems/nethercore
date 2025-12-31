//! Audio synthesizers for tracker-demo
//!
//! This module provides procedural audio synthesis for three genres:
//! - **Funk**: "Nether Groove" - Funky Jazz at 110 BPM in F Dorian
//! - **Eurobeat**: "Nether Fire" - Eurobeat at 155 BPM in D minor
//! - **Synthwave**: "Nether Drive" - Synthwave at 105 BPM in A minor

pub mod common;
pub mod eurobeat;
pub mod funk;
pub mod synthwave;

pub use common::{apply_fades, SAMPLE_RATE};

// Re-export all instrument generators
pub use funk::{
    generate_bass_funk, generate_epiano, generate_hihat_funk, generate_kick_funk,
    generate_lead_jazz, generate_snare_funk,
};

pub use eurobeat::{
    generate_bass_euro, generate_brass_euro, generate_hihat_euro, generate_kick_euro,
    generate_pad_euro, generate_snare_euro, generate_supersaw,
};

pub use synthwave::{
    generate_arp_synth, generate_bass_synth, generate_hihat_synth, generate_kick_synth,
    generate_lead_synth, generate_pad_synth, generate_snare_synth,
};
