//! QOA decoder implementation
//!
//! This module handles decoding QOA frame data to PCM samples.
//! Note: This is a pure codec - no file headers are parsed. The caller
//! (z-common) provides total_samples from NetherZSoundHeader.

use crate::{QOA_DEQUANT_TAB, QOA_FRAME_HEADER_SIZE, QOA_SLICE_LEN, QoaError, QoaLms, clamp_i16};

/// Decode a single slice (8 bytes = 20 samples)
///
/// # Slice format (64 bits, big-endian)
/// - Bits 60-63: Scalefactor index (4 bits)
/// - Bits 0-59:  20 quantized residuals (3 bits each)
///
/// # Arguments
/// * `slice` - The 64-bit encoded slice
/// * `lms` - LMS predictor state (updated during decoding)
/// * `output` - Output buffer for decoded samples
///
/// # Returns
/// Number of samples decoded (up to 20, or `output.len()` if smaller)
pub fn decode_slice(slice: u64, lms: &mut QoaLms, output: &mut [i16]) -> usize {
    let scalefactor = ((slice >> 60) & 0xF) as usize;
    let mut sample_count = 0;

    for i in 0..QOA_SLICE_LEN {
        if sample_count >= output.len() {
            break;
        }

        // Extract 3-bit quantized value (from high bits down)
        let quantized = ((slice >> (57 - i * 3)) & 0x7) as usize;

        // Predict and dequantize
        let predicted = lms.predict();
        let dequantized = QOA_DEQUANT_TAB[scalefactor][quantized];
        let sample = clamp_i16(predicted + dequantized);

        // Update LMS state
        lms.update(sample, dequantized);

        output[sample_count] = sample as i16;
        sample_count += 1;
    }

    sample_count
}

/// Decode QOA frame data to PCM
///
/// # Arguments
/// * `qoa_data` - Raw QOA frame data (no file header - caller provides total_samples)
/// * `total_samples` - Total number of samples to decode (from NetherZSoundHeader)
///
/// # Returns
/// Decoded PCM samples (mono, 16-bit) or error
///
/// Note: This is a pure codec. The caller (z-common) provides total_samples from
/// NetherZSoundHeader. Sample rate is fixed at 22050 Hz and controlled by the
/// asset pipeline.
///
/// # Errors
/// Returns `QoaError` if the data is invalid or truncated
pub fn decode_qoa(qoa_data: &[u8], total_samples: usize) -> Result<Vec<i16>, QoaError> {
    if total_samples == 0 {
        return Ok(Vec::new());
    }

    if qoa_data.is_empty() {
        return Err(QoaError::FileTooSmall);
    }

    let mut output = Vec::with_capacity(total_samples);
    let mut data_idx = 0;
    let mut lms_states = [QoaLms::new(); 8]; // Max 8 channels

    while data_idx + QOA_FRAME_HEADER_SIZE <= qoa_data.len() && output.len() < total_samples {
        // Read frame header (5 bytes: channels + samples_in_frame + frame_size)
        let channels = qoa_data[data_idx] as usize;
        let samples_in_frame =
            u16::from_be_bytes([qoa_data[data_idx + 1], qoa_data[data_idx + 2]]) as usize;
        let _frame_size =
            u16::from_be_bytes([qoa_data[data_idx + 3], qoa_data[data_idx + 4]]) as usize;

        if channels == 0 || channels > 8 {
            return Err(QoaError::InvalidChannelCount);
        }

        data_idx += QOA_FRAME_HEADER_SIZE;

        // Read LMS state for each channel
        for lms_state in lms_states.iter_mut().take(channels) {
            if data_idx + 16 > qoa_data.len() {
                return Err(QoaError::TruncatedData);
            }

            // History (4 x i16, big-endian)
            for i in 0..4 {
                lms_state.history[i] = i16::from_be_bytes([
                    qoa_data[data_idx + i * 2],
                    qoa_data[data_idx + i * 2 + 1],
                ]) as i32;
            }
            data_idx += 8;

            // Weights (4 x i16, big-endian)
            for i in 0..4 {
                lms_state.weights[i] = i16::from_be_bytes([
                    qoa_data[data_idx + i * 2],
                    qoa_data[data_idx + i * 2 + 1],
                ]) as i32;
            }
            data_idx += 8;
        }

        // Decode slices
        let slices_per_channel = samples_in_frame.div_ceil(QOA_SLICE_LEN);

        // For multi-channel, we need to interleave and mix
        if channels == 1 {
            // Mono: decode directly to output
            for slice_idx in 0..slices_per_channel {
                if data_idx + 8 > qoa_data.len() {
                    return Err(QoaError::TruncatedData);
                }

                let slice = u64::from_be_bytes([
                    qoa_data[data_idx],
                    qoa_data[data_idx + 1],
                    qoa_data[data_idx + 2],
                    qoa_data[data_idx + 3],
                    qoa_data[data_idx + 4],
                    qoa_data[data_idx + 5],
                    qoa_data[data_idx + 6],
                    qoa_data[data_idx + 7],
                ]);
                data_idx += 8;

                let samples_remaining = samples_in_frame.saturating_sub(slice_idx * QOA_SLICE_LEN);
                let samples_to_decode = samples_remaining.min(QOA_SLICE_LEN);

                let mut temp = [0i16; QOA_SLICE_LEN];
                decode_slice(slice, &mut lms_states[0], &mut temp[..samples_to_decode]);
                output.extend_from_slice(&temp[..samples_to_decode]);
            }
        } else {
            // Multi-channel: decode each channel, then mix to mono
            let mut channel_buffers: Vec<Vec<i16>> = vec![Vec::new(); channels];

            for slice_idx in 0..slices_per_channel {
                for ch in 0..channels {
                    if data_idx + 8 > qoa_data.len() {
                        return Err(QoaError::TruncatedData);
                    }

                    let slice = u64::from_be_bytes([
                        qoa_data[data_idx],
                        qoa_data[data_idx + 1],
                        qoa_data[data_idx + 2],
                        qoa_data[data_idx + 3],
                        qoa_data[data_idx + 4],
                        qoa_data[data_idx + 5],
                        qoa_data[data_idx + 6],
                        qoa_data[data_idx + 7],
                    ]);
                    data_idx += 8;

                    let samples_remaining =
                        samples_in_frame.saturating_sub(slice_idx * QOA_SLICE_LEN);
                    let samples_to_decode = samples_remaining.min(QOA_SLICE_LEN);

                    let mut temp = [0i16; QOA_SLICE_LEN];
                    decode_slice(slice, &mut lms_states[ch], &mut temp[..samples_to_decode]);
                    channel_buffers[ch].extend_from_slice(&temp[..samples_to_decode]);
                }
            }

            // Mix channels to mono (average)
            let frame_samples = channel_buffers[0].len();
            for i in 0..frame_samples {
                let mut sum: i32 = 0;
                for ch_buf in &channel_buffers {
                    sum += ch_buf[i] as i32;
                }
                output.push((sum / channels as i32) as i16);
            }
        }
    }

    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_slice_zeros() {
        // Slice with sf=0 and all quantized values = 0
        // sf=0, all q=0 -> slice = 0x0000_0000_0000_0000
        let slice: u64 = 0x0000_0000_0000_0000;
        let mut lms = QoaLms::new();
        let mut output = [0i16; 20];

        let decoded = decode_slice(slice, &mut lms, &mut output);

        assert_eq!(decoded, 20);
        // With default weights and zero history, prediction starts at 0
        // dequant[0][0] = 1, so first sample = clamp(0 + 1) = 1
        assert_eq!(output[0], 1);
    }

    #[test]
    fn test_decode_slice_max_scalefactor() {
        // sf=15 (max), all q=0
        let slice: u64 = 0xF000_0000_0000_0000;
        let mut lms = QoaLms::new();
        let mut output = [0i16; 20];

        decode_slice(slice, &mut lms, &mut output);

        // dequant[15][0] = 1536
        assert_eq!(output[0], 1536);
    }

    #[test]
    fn test_decode_slice_partial() {
        let slice: u64 = 0x0000_0000_0000_0000;
        let mut lms = QoaLms::new();
        let mut output = [0i16; 5]; // Only 5 samples

        let decoded = decode_slice(slice, &mut lms, &mut output);

        assert_eq!(decoded, 5);
    }

    #[test]
    fn test_decode_empty_samples() {
        // Zero samples requested = empty output
        let result = decode_qoa(&[], 0);
        assert_eq!(result, Ok(vec![]));
    }

    #[test]
    fn test_decode_empty_data() {
        // Non-zero samples but no data = error
        let result = decode_qoa(&[], 100);
        assert_eq!(result, Err(QoaError::FileTooSmall));
    }
}
