//! Nether-QOA: Modified QOA codec for Nethercore
//!
//! This is a **modified** implementation of QOA (Quite OK Audio) tailored for
//! Nethercore's fantasy consoles. It is NOT compatible with standard QOA files.
//!
//! **This is a pure codec** - it handles only the compression/decompression of
//! audio data. File format headers (like total_samples) are handled by the caller
//! (z-common's NetherZXSoundHeader).
//!
//! # Differences from Standard QOA
//!
//! | Feature | Standard QOA | Nether-QOA |
//! |---------|--------------|-----------|
//! | File magic | "qoaf" (4 bytes) | None (pure codec) |
//! | File header | 8 bytes (magic + total_samples) | None (caller handles) |
//! | Frame header | 8 bytes (channels + sample_rate + samples + size) | 5 bytes (no sample_rate) |
//! | Sample rate | 24-bit per frame | Not stored (fixed 22050 Hz) |
//!
//! # Why the modifications?
//!
//! Nethercore uses a fixed sample rate (22050 Hz) controlled by the asset pipeline,
//! not the audio files themselves. This codec is a drop-in replacement for ADPCM -
//! just different encoding, no extra features.
//!
//! # Frame Format
//!
//! ```text
//! Frame header (5 bytes, repeats):
//!   0x00: channels (u8)
//!   0x01: samples_in_frame (u16 BE)
//!   0x03: frame_size (u16 BE)
//!
//! Per-channel LMS state (16 bytes each):
//!   history[4] as i16 BE + weights[4] as i16 BE
//!
//! Slices (8 bytes each, up to 256 per channel):
//!   20 samples encoded as scalefactor (4 bits) + residuals (60 bits)
//! ```
//!
//! # Compression
//!
//! Achieves approximately 5:1 compression (3.2 bits per sample) using LMS
//! prediction with adaptive quantization - identical to standard QOA.
//!
//! # Usage
//!
//! ```
//! use nether_qoa::{encode_qoa, decode_qoa};
//!
//! // Encode PCM samples to QOA frame data
//! let samples: Vec<i16> = vec![0; 1000];
//! let qoa_data = encode_qoa(&samples);
//!
//! // Decode QOA frame data back to PCM (caller provides total_samples)
//! let decoded = decode_qoa(&qoa_data, samples.len()).unwrap();
//! assert_eq!(decoded.len(), samples.len());
//! ```

mod decode;
mod encode;
mod lms;

pub use decode::{decode_qoa, decode_slice};
pub use encode::{encode_qoa, encode_slice};
pub use lms::QoaLms;

// =============================================================================
// Constants
// =============================================================================

/// Samples per slice (each slice is 64 bits)
pub const QOA_SLICE_LEN: usize = 20;

/// Maximum slices per frame per channel
pub const QOA_MAX_SLICES: usize = 256;

/// Maximum samples per frame (256 slices x 20 samples)
pub const QOA_FRAME_SAMPLES: usize = 5120;

/// LMS filter history/weight length
pub const QOA_LMS_LEN: usize = 4;

/// Frame header size (channels + samples_in_frame + frame_size, no sample_rate)
pub const QOA_FRAME_HEADER_SIZE: usize = 5;

/// LMS state size per channel (4 history + 4 weights as i16)
pub const QOA_LMS_STATE_SIZE: usize = 16;

/// Scalefactor table (16 entries)
/// Used to scale residuals during quantization
pub const QOA_SCALEFACTOR_TAB: [i32; 16] = [
    1, 7, 21, 45, 84, 138, 211, 304, 421, 562, 731, 928, 1157, 1419, 1715, 2048,
];

/// Quantization table (17 entries)
/// Maps residual / scalefactor result (-8..8) to 3-bit index
pub const QOA_QUANT_TAB: [u8; 17] = [
    7, 7, 7, 5, 5, 3, 3, 1, // -8..-1
    0, // 0
    0, 2, 2, 4, 4, 6, 6, 6, // 1..8
];

/// Dequantization table (16 scalefactors x 8 quantized values)
/// Pre-computed: dequant_tab[sf][qval] = round(scalefactor * dequant_mul[qval])
/// where dequant_mul = [0.75, -0.75, 2.5, -2.5, 4.5, -4.5, 7.0, -7.0]
pub const QOA_DEQUANT_TAB: [[i32; 8]; 16] = [
    [1, -1, 3, -3, 5, -5, 7, -7],
    [5, -5, 18, -18, 32, -32, 49, -49],
    [16, -16, 53, -53, 95, -95, 147, -147],
    [34, -34, 113, -113, 203, -203, 315, -315],
    [63, -63, 210, -210, 378, -378, 588, -588],
    [104, -104, 345, -345, 621, -621, 966, -966],
    [158, -158, 528, -528, 950, -950, 1477, -1477],
    [228, -228, 760, -760, 1368, -1368, 2128, -2128],
    [316, -316, 1053, -1053, 1895, -1895, 2947, -2947],
    [422, -422, 1405, -1405, 2529, -2529, 3934, -3934],
    [548, -548, 1828, -1828, 3290, -3290, 5117, -5117],
    [696, -696, 2320, -2320, 4176, -4176, 6496, -6496],
    [868, -868, 2893, -2893, 5207, -5207, 8099, -8099],
    [1064, -1064, 3548, -3548, 6386, -6386, 9933, -9933],
    [1286, -1286, 4288, -4288, 7718, -7718, 12005, -12005],
    [1536, -1536, 5120, -5120, 9216, -9216, 14336, -14336],
];

// =============================================================================
// Error Type
// =============================================================================

/// Errors that can occur during QOA decoding
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QoaError {
    /// File is too small to contain valid QOA data
    FileTooSmall,
    /// Invalid channel count (must be 1-8)
    InvalidChannelCount,
    /// Data was truncated before end of file
    TruncatedData,
}

impl core::fmt::Display for QoaError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            QoaError::FileTooSmall => write!(f, "file too small for QOA header"),
            QoaError::InvalidChannelCount => write!(f, "invalid channel count (must be 1-8)"),
            QoaError::TruncatedData => write!(f, "truncated QOA data"),
        }
    }
}

impl std::error::Error for QoaError {}

// =============================================================================
// Helper Functions
// =============================================================================

/// Clamp value to 16-bit signed range
#[inline]
pub(crate) fn clamp_i16(v: i32) -> i32 {
    v.clamp(-32768, 32767)
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn generate_sine(freq: f32, sample_rate: u32, duration_sec: f32) -> Vec<i16> {
        let num_samples = (sample_rate as f32 * duration_sec) as usize;
        (0..num_samples)
            .map(|i| {
                let t = i as f32 / sample_rate as f32;
                (f32::sin(t * freq * std::f32::consts::TAU) * 16000.0) as i16
            })
            .collect()
    }

    #[test]
    fn test_roundtrip_sine() {
        let original = generate_sine(440.0, 22050, 1.0);
        let encoded = encode_qoa(&original);
        let decoded = decode_qoa(&encoded, original.len()).unwrap();

        assert_eq!(decoded.len(), original.len());
    }

    #[test]
    fn test_roundtrip_silence() {
        let original = vec![0i16; 22050]; // 1 second of silence
        let encoded = encode_qoa(&original);
        let decoded = decode_qoa(&encoded, original.len()).unwrap();

        assert_eq!(decoded.len(), original.len());

        // Silence should have very low error
        let max_error: i16 = original
            .iter()
            .zip(&decoded)
            .map(|(a, b)| (a - b).abs())
            .max()
            .unwrap_or(0);
        assert!(max_error < 100, "Silence max error too high: {}", max_error);
    }

    #[test]
    fn test_roundtrip_preserves_length() {
        // Test various lengths including frame boundaries
        for len in [
            1,
            20,
            100,
            QOA_FRAME_SAMPLES - 1,
            QOA_FRAME_SAMPLES,
            QOA_FRAME_SAMPLES + 1,
            QOA_FRAME_SAMPLES * 2,
            22050,
        ] {
            let original: Vec<i16> = (0..len).map(|i| (i as i16).wrapping_mul(7)).collect();
            let encoded = encode_qoa(&original);
            let decoded = decode_qoa(&encoded, original.len()).unwrap();

            assert_eq!(
                decoded.len(),
                original.len(),
                "Length mismatch for {} samples",
                len
            );
        }
    }

    #[test]
    fn test_compression_ratio() {
        let original = generate_sine(440.0, 22050, 10.0); // 10 seconds
        let encoded = encode_qoa(&original);

        let pcm_size = original.len() * 2; // 16-bit samples
        let qoa_size = encoded.len();
        let ratio = pcm_size as f64 / qoa_size as f64;

        // Nether-QOA should achieve slightly better than 5:1 due to smaller headers
        assert!(
            ratio > 4.5,
            "Compression ratio too low: {:.2}:1 (expected > 4.5:1)",
            ratio
        );
        assert!(
            ratio < 6.0,
            "Compression ratio too high: {:.2}:1 (expected < 6.0:1)",
            ratio
        );
    }
}
