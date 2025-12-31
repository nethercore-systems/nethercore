//! Common utilities for procedural tracker music generators
//!
//! This crate provides shared functionality for generating tracker module files:
//! - WAV file writing
//! - Audio synthesis utilities (sample rate, fade functions, RNG, formant filters)

pub mod synth;
pub mod wav_writer;

// Re-export commonly used items at crate root
pub use synth::{apply_fades, formant_filter, SimpleRng, SAMPLE_RATE};
pub use wav_writer::write_wav;
