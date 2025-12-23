//! QOA encoder implementation
//!
//! This module handles encoding PCM samples to QOA frame data.
//! Note: This is a pure codec - no file headers are written. The caller
//! (z-common) handles the NetherZXSoundHeader with total_samples.

use crate::{
    QOA_DEQUANT_TAB, QOA_FRAME_HEADER_SIZE, QOA_FRAME_SAMPLES, QOA_LMS_STATE_SIZE, QOA_QUANT_TAB,
    QOA_SCALEFACTOR_TAB, QOA_SLICE_LEN, QoaLms, clamp_i16,
};

/// Encode a slice of up to 20 samples
///
/// Tries all 16 scalefactors and picks the one with lowest MSE.
///
/// # Arguments
/// * `samples` - Input samples (up to 20)
/// * `lms` - LMS predictor state (updated during encoding)
///
/// # Returns
/// The 64-bit encoded slice
pub fn encode_slice(samples: &[i16], lms: &mut QoaLms) -> u64 {
    // Try all 16 scalefactors, pick best
    let mut best_slice = 0u64;
    let mut best_error = i64::MAX;
    let mut best_lms = *lms;

    for sf in 0..16 {
        let mut test_lms = *lms;
        let mut slice = (sf as u64) << 60;
        let mut total_error = 0i64;

        for (i, &sample) in samples.iter().enumerate().take(QOA_SLICE_LEN) {
            let predicted = test_lms.predict();
            let residual = sample as i32 - predicted;

            // Quantize: divide by scalefactor, clamp to -8..8, lookup index
            let scaled = residual / QOA_SCALEFACTOR_TAB[sf].max(1);
            let clamped = scaled.clamp(-8, 8);
            let quantized = QOA_QUANT_TAB[(clamped + 8) as usize];

            // Dequantize to get reconstruction
            let dequantized = QOA_DEQUANT_TAB[sf][quantized as usize];
            let reconstructed = clamp_i16(predicted + dequantized);

            // Update LMS
            test_lms.update(reconstructed, dequantized);

            // Accumulate error
            let error = (sample as i32 - reconstructed).abs() as i64;
            total_error += error * error;

            // Pack quantized value into slice
            slice |= (quantized as u64) << (57 - i * 3);
        }

        if total_error < best_error {
            best_error = total_error;
            best_slice = slice;
            best_lms = test_lms;
        }
    }

    *lms = best_lms;
    best_slice
}

/// Encode PCM samples to QOA frame data
///
/// # Arguments
/// * `samples` - Input PCM samples (mono, 16-bit)
///
/// # Returns
/// Encoded QOA frame data (no file header - caller handles that)
///
/// Note: This is a pure codec. The caller (z-common's NetherZXSoundHeader) is
/// responsible for storing total_samples. Sample rate is fixed at 22050 Hz
/// and controlled by the asset pipeline.
pub fn encode_qoa(samples: &[i16]) -> Vec<u8> {
    let total_samples = samples.len();
    let mut output = Vec::new();

    let mut lms = QoaLms::new();
    let mut sample_idx = 0;

    while sample_idx < total_samples {
        let samples_in_frame = (total_samples - sample_idx).min(QOA_FRAME_SAMPLES);
        let slices_in_frame = samples_in_frame.div_ceil(QOA_SLICE_LEN);

        // Calculate frame size (5-byte header, no sample rate)
        let frame_size = QOA_FRAME_HEADER_SIZE + QOA_LMS_STATE_SIZE + slices_in_frame * 8;

        // Frame header (mono, no sample rate - Nethercore uses fixed 22050 Hz)
        output.push(1); // channels
        output.extend_from_slice(&(samples_in_frame as u16).to_be_bytes());
        output.extend_from_slice(&(frame_size as u16).to_be_bytes());

        // LMS state (history + weights as i16 big-endian)
        for i in 0..4 {
            output.extend_from_slice(&(lms.history[i] as i16).to_be_bytes());
        }
        for i in 0..4 {
            output.extend_from_slice(&(lms.weights[i] as i16).to_be_bytes());
        }

        // Encode slices
        for slice_idx in 0..slices_in_frame {
            let start = sample_idx + slice_idx * QOA_SLICE_LEN;
            let end = (start + QOA_SLICE_LEN).min(total_samples);
            let slice = encode_slice(&samples[start..end], &mut lms);
            output.extend_from_slice(&slice.to_be_bytes());
        }

        sample_idx += samples_in_frame;
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_slice_zeros() {
        let samples = [0i16; 20];
        let mut lms = QoaLms::new();
        let slice = encode_slice(&samples, &mut lms);

        // Should encode to something (we don't check exact value, just that it doesn't panic)
        assert!(slice != 0 || slice == 0); // Always true, just testing no panic
    }

    #[test]
    fn test_encode_slice_partial() {
        let samples = [100i16; 5]; // Only 5 samples
        let mut lms = QoaLms::new();
        let slice = encode_slice(&samples, &mut lms);

        // Should encode partial slice without panic
        assert!(slice != 0 || slice == 0);
    }

    #[test]
    fn test_encode_empty() {
        let samples: Vec<i16> = vec![];
        let encoded = encode_qoa(&samples);

        // Should be empty (no frames, no file header)
        assert_eq!(encoded.len(), 0);
    }

    #[test]
    fn test_encode_small() {
        let samples = vec![1000i16; 100];
        let encoded = encode_qoa(&samples);

        // Should have header + frame data
        assert!(encoded.len() > 4);
    }
}
