//! Audio synthesizers for IT tracker demo
//!
//! This module provides procedural audio synthesis for three genres:
//! - **Orchestral**: "Nether Dawn" - Epic at 90 BPM in D major
//! - **Ambient**: "Nether Mist" - Atmospheric at 70 BPM in D minor
//! - **DnB**: "Nether Storm" - Action at 174 BPM in F minor

pub mod ambient;
pub mod common;
pub mod dnb;
pub mod orchestral;

pub use common::{apply_fades, SAMPLE_RATE};

// Re-export ambient instrument generators
pub use ambient::{
    generate_atmos_wind, generate_bass_sub, generate_bell_glass, generate_hit_dark,
    generate_lead_echo, generate_lead_ghost, generate_noise_breath, generate_pad_air,
    generate_pad_cold, generate_pad_sub, generate_pad_warm, generate_reverb_sim,
};

// Re-export DnB instrument generators
pub use dnb::{
    generate_atmos_storm, generate_bass_reese, generate_bass_sub_dnb, generate_bass_wobble,
    generate_break_slice, generate_cymbal, generate_fx_impact, generate_fx_riser,
    generate_hihat_closed, generate_hihat_open, generate_kick_dnb, generate_lead_main,
    generate_lead_stab, generate_pad_dark, generate_snare_dnb,
};

// Re-export orchestral instrument generators
pub use orchestral::{
    generate_bass_epic, generate_brass_horn, generate_brass_trumpet, generate_choir_ah,
    generate_choir_oh, generate_cymbal_crash, generate_flute, generate_fx_epic,
    generate_harp_gliss, generate_pad_orchestra, generate_piano, generate_snare_orch,
    generate_strings_cello, generate_strings_viola, generate_strings_violin, generate_timpani,
};
