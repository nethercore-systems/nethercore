//! Drum and Bass instrument synthesis - HIGH QUALITY
//!
//! Instruments for "Nether Storm" - DnB at 174 BPM in F minor/Phrygian
//! Features: Band-limited oscillators, proper filters, punchy envelopes

// SAMPLE_RATE is re-imported within each submodule

mod bass;
mod drums;
mod dsp;
mod fx;
mod synth;

// Re-export public API
pub use drums::{
    generate_break_slice, generate_cymbal, generate_hihat_closed, generate_hihat_open,
    generate_kick_dnb, generate_snare_dnb,
};

pub use bass::{generate_bass_reese, generate_bass_sub_dnb, generate_bass_wobble};

pub use synth::{generate_lead_main, generate_lead_stab, generate_pad_dark};

pub use fx::{generate_atmos_storm, generate_fx_impact, generate_fx_riser};
