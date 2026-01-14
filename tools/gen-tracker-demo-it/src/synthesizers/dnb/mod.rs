//! Drum and Bass instrument synthesis - HIGH QUALITY
//!
//! Instruments for "Nether Storm" - DnB at 174 BPM in F minor/Phrygian
//! Features: Band-limited oscillators, proper filters, punchy envelopes

// SAMPLE_RATE is re-imported within each submodule

mod dsp;
mod drums;
mod bass;
mod synth;
mod fx;

// Re-export public API
pub use drums::{
    generate_kick_dnb,
    generate_snare_dnb,
    generate_hihat_closed,
    generate_hihat_open,
    generate_break_slice,
    generate_cymbal,
};

pub use bass::{
    generate_bass_sub_dnb,
    generate_bass_reese,
    generate_bass_wobble,
};

pub use synth::{
    generate_pad_dark,
    generate_lead_stab,
    generate_lead_main,
};

pub use fx::{
    generate_fx_riser,
    generate_fx_impact,
    generate_atmos_storm,
};
