//! Nethercore ZX audio backend
//!
//! Per-frame audio generation with rollback support.
//!
//! Architecture:
//! - Audio state (playhead positions, volumes) is part of ZRollbackState
//! - Each frame, generate_audio_frame() generates samples from the current state
//! - Samples are pushed to a ring buffer consumed by the cpal audio thread
//! - During rollback, state is restored and no samples are generated
//!
//! Audio specs:
//! - 44,100 Hz sample rate (native for most audio hardware)
//! - Stereo output
//! - 16-bit signed PCM mono source sounds (22,050 Hz, upsampled)

use std::sync::Arc;

mod backend;
mod generation;
mod mixing;
mod output;

#[cfg(test)]
mod tests;

// Re-export public API
pub use backend::{ZXAudio, ZXAudioGenerator};
pub use generation::{advance_audio_positions, generate_audio_frame_with_tracker};
pub use output::{AudioOutput, OUTPUT_SAMPLE_RATE, SOURCE_SAMPLE_RATE};

/// Sound data (raw PCM)
#[derive(Clone, Debug)]
pub struct Sound {
    /// Raw PCM data (16-bit signed, mono, 22.05kHz)
    pub data: Arc<Vec<i16>>,
}
