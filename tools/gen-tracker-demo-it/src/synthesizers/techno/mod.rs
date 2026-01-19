//! Acid Techno instrument synthesis
//!
//! Instruments for "Nether Acid" - Acid Techno at 130 BPM in E minor
//! Features: TB-303 acid bassline with resonant filter, 909 drums

mod bass_303;
mod drums_909;
mod filters;
mod textures;

// Re-export all public instrument generators
pub use drums_909::{
    generate_clap_909, generate_hat_909_closed, generate_hat_909_open, generate_kick_909,
};

pub use bass_303::{generate_bass_303, generate_bass_303_squelch};

pub use textures::{
    generate_atmosphere_acid, generate_crash_909, generate_pad_acid, generate_riser_acid,
    generate_stab_acid,
};
