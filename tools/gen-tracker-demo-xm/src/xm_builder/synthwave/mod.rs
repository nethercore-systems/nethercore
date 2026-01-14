//! Synthwave XM pattern generation
//!
//! Patterns for "Nether Drive" - Synthwave at 105 BPM in A minor

mod constants;
mod patterns;
mod xm_generation;

// Re-export public API
pub use xm_generation::{generate_synthwave_xm, generate_synthwave_xm_embedded};
