//! Audio synthesizers for IT tracker demo
//!
//! This module provides procedural audio synthesis for multiple genres:
//! - **Acid Techno**: "Nether Acid" - Hypnotic at 130 BPM in E minor
//! - **Orchestral**: "Nether Dawn" - Epic at 90 BPM in D major
//! - **DnB**: "Nether Storm" - Action at 174 BPM in F minor

pub mod common;
pub mod dnb;
pub mod orchestral;
pub mod techno;

// Note: Common utilities are imported directly where needed

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

// Re-export techno instrument generators
pub use techno::{
    generate_atmosphere_acid, generate_bass_303, generate_bass_303_squelch, generate_clap_909,
    generate_crash_909, generate_hat_909_closed, generate_hat_909_open, generate_kick_909,
    generate_pad_acid, generate_riser_acid, generate_stab_acid,
};
