//! Procedural audio synthesis
//!
//! This module provides tools for generating audio procedurally,
//! including oscillators, envelopes, filters, and a high-level Synth API.
//!
//! # Example
//! ```no_run
//! use proc_gen::audio::*;
//!
//! // Generate a simple tone
//! let mut synth = Synth::new(SAMPLE_RATE);
//! let tone = synth.tone(Waveform::Sine, 440.0, 0.5, Envelope::pluck());
//!
//! // Generate a sweep effect
//! let sweep = synth.sweep(Waveform::Saw, 200.0, 800.0, 0.3, Envelope::default());
//!
//! // Mix sounds together
//! let mixed = mix(&[(&tone, 0.7), (&sweep, 0.3)]);
//!
//! // Convert to PCM i16 for ZX console
//! let pcm = to_pcm_i16(&mixed);
//!
//! // Export to WAV for debugging (requires wav-export feature)
//! #[cfg(feature = "wav-export")]
//! write_wav(&pcm, SAMPLE_RATE, std::path::Path::new("output.wav")).unwrap();
//! ```

mod oscillators;
mod envelope;
mod filters;
mod synth;
mod export;
pub mod showcase;
pub mod fm;

/// ZX console sample rate (22.05kHz)
pub const SAMPLE_RATE: u32 = 22050;

// Oscillators
pub use oscillators::{Waveform, oscillator, noise, pink_noise};

// Envelope
pub use envelope::Envelope;

// Filters
pub use filters::{high_pass, high_pass_resonant, low_pass, low_pass_resonant, low_pass_simple, FilterConfig};

// Synth API
pub use synth::Synth;

// Utilities and export
pub use export::{concat, from_pcm_i16, mix, normalize, normalize_to, silence, to_pcm_i16};

#[cfg(feature = "wav-export")]
pub use export::{write_wav, write_wav_f32};

/// Audio sample buffer (f32 samples, -1.0 to 1.0 range)
#[derive(Clone)]
pub struct AudioBuffer {
    /// Sample rate in Hz
    pub sample_rate: u32,
    /// Audio samples in -1.0 to 1.0 range
    pub samples: Vec<f32>,
}

impl AudioBuffer {
    /// Create a new empty audio buffer
    pub fn new(sample_rate: u32) -> Self {
        Self {
            sample_rate,
            samples: Vec::new(),
        }
    }

    /// Create a buffer with pre-allocated capacity
    pub fn with_capacity(sample_rate: u32, capacity: usize) -> Self {
        Self {
            sample_rate,
            samples: Vec::with_capacity(capacity),
        }
    }

    /// Create a buffer from samples
    pub fn from_samples(sample_rate: u32, samples: Vec<f32>) -> Self {
        Self { sample_rate, samples }
    }

    /// Duration in seconds
    pub fn duration(&self) -> f32 {
        self.samples.len() as f32 / self.sample_rate as f32
    }

    /// Number of samples
    pub fn len(&self) -> usize {
        self.samples.len()
    }

    /// Check if buffer is empty
    pub fn is_empty(&self) -> bool {
        self.samples.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audio_buffer_new() {
        let buf = AudioBuffer::new(SAMPLE_RATE);
        assert_eq!(buf.sample_rate, SAMPLE_RATE);
        assert!(buf.is_empty());
    }

    #[test]
    fn test_audio_buffer_from_samples() {
        let samples = vec![0.0, 0.5, 1.0, -1.0];
        let buf = AudioBuffer::from_samples(SAMPLE_RATE, samples.clone());
        assert_eq!(buf.len(), 4);
        assert_eq!(buf.samples, samples);
    }

    #[test]
    fn test_audio_buffer_duration() {
        let samples = vec![0.0; SAMPLE_RATE as usize]; // 1 second of audio
        let buf = AudioBuffer::from_samples(SAMPLE_RATE, samples);
        assert!((buf.duration() - 1.0).abs() < 0.001);
    }
}
