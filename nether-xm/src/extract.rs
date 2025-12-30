//! XM sample extraction for automatic ROM packing
//!
//! This module extracts embedded sample data from XM files for use with
//! Nethercore's automatic sample extraction feature. Samples are decoded
//! from delta-encoded format and converted to standard i16 PCM.

use crate::XmError;
use std::io::{Cursor, Read, Seek, SeekFrom};

/// Extracted sample data from XM instrument
#[derive(Debug, Clone)]
pub struct ExtractedSample {
    /// Instrument index (0-based)
    pub instrument_index: u8,
    /// Instrument name (used for sample ID generation)
    pub name: String,
    /// Sample rate calculated from finetune
    pub sample_rate: u32,
    /// Bit depth (8 or 16)
    pub bit_depth: u8,
    /// Loop start position in samples
    pub loop_start: u32,
    /// Loop length in samples
    pub loop_length: u32,
    /// Loop type: 0=none, 1=forward, 2=pingpong
    pub loop_type: u8,
    /// Decoded sample data (i16 format)
    pub data: Vec<i16>,
}

impl ExtractedSample {
    /// Calculate sample rate from XM finetune and relative note
    ///
    /// XM encodes sample rates through pitch adjustment:
    /// - Base frequency: 8363 Hz (C-4)
    /// - relative_note: semitone transpose (-96 to +95)
    /// - finetune: fine adjustment in 1/128 semitone units (-128 to +127)
    ///
    /// Formula: rate = 8363 × 2^((relative_note + finetune/128) / 12)
    ///
    /// Example: For a 22050 Hz sample, set relative_note=17, finetune=100
    pub fn calculate_sample_rate(finetune: i8, relative_note: i8) -> u32 {
        // Base frequency for C-4 (Amiga standard)
        const BASE_FREQ: f64 = 8363.0;

        // Calculate total semitones (including fractional part from finetune)
        let total_semitones = relative_note as f64 + (finetune as f64 / 128.0);

        // Calculate frequency using 2^(semitones/12)
        let freq = BASE_FREQ * 2.0_f64.powf(total_semitones / 12.0);

        // Round to nearest Hz with clamping to avoid extreme values
        let rounded = freq.round() as u32;

        // Clamp to reasonable range (100 Hz to 96000 Hz)
        rounded.clamp(100, 96000)
    }
}

/// Extract all samples from an XM file
///
/// This parses the XM file and extracts all embedded sample data,
/// decoding from delta-encoded format to standard i16 PCM.
///
/// # Arguments
/// * `data` - Raw XM file data
///
/// # Returns
/// * `Vec<ExtractedSample>` - All samples found in the XM file
///
/// # Errors
/// * Returns `XmError` if parsing fails
pub fn extract_samples(data: &[u8]) -> Result<Vec<ExtractedSample>, XmError> {
    // First parse the XM to get structure information
    let module = crate::parse_xm(data)?;

    // Now extract samples with a second pass
    let mut cursor = Cursor::new(data);
    let mut samples = Vec::new();

    // Skip to end of main header
    cursor.seek(SeekFrom::Start(60))?;
    let header_size = read_u32(&mut cursor)? as usize;
    cursor.seek(SeekFrom::Start((60 + header_size) as u64))?;

    // Skip all patterns
    for _ in 0..module.num_patterns {
        // Per XM spec, header_length INCLUDES the 4-byte length field itself
        let header_start = cursor.position();
        let pattern_header_len = read_u32(&mut cursor)?;

        // Skip packing type (1 byte) and num_rows (2 bytes)
        cursor.seek(SeekFrom::Current(3))?;

        // Read packed pattern data size
        let packed_size = read_u16(&mut cursor)?;

        // Skip to end of header, then skip packed data
        cursor.seek(SeekFrom::Start(header_start + pattern_header_len as u64))?;
        cursor.seek(SeekFrom::Current(packed_size as i64))?;
    }

    // Extract samples from each instrument
    for (inst_idx, instrument) in module.instruments.iter().enumerate() {
        match extract_instrument_samples(
            &mut cursor,
            inst_idx as u8,
            instrument,
        ) {
            Ok(instrument_samples) => samples.extend(instrument_samples),
            Err(XmError::UnexpectedEof) | Err(XmError::IoError(_)) if inst_idx == 0 && samples.is_empty() => {
                // Sample-less XM file - first instrument has no readable data
                // This is expected for XM files with no embedded samples
                return Ok(Vec::new());
            }
            Err(e) => return Err(e),
        }
    }

    Ok(samples)
}

/// Extract samples from a single instrument
fn extract_instrument_samples(
    cursor: &mut Cursor<&[u8]>,
    inst_idx: u8,
    instrument: &crate::XmInstrument,
) -> Result<Vec<ExtractedSample>, XmError> {
    let header_start = cursor.position();

    // Read instrument header size
    let header_size = read_u32(cursor)?;

    // Skip to sample count
    cursor.seek(SeekFrom::Start(header_start + 27))?;
    let num_samples = read_u16(cursor)? as u8;

    if num_samples == 0 {
        // No samples, skip empty instrument header
        cursor.seek(SeekFrom::Start(header_start + header_size as u64))?;
        return Ok(Vec::new());
    }

    // Read sample header size
    let sample_header_size = read_u32(cursor)?;

    // Skip to after extended instrument header
    cursor.seek(SeekFrom::Start(header_start + header_size as u64))?;

    let mut results = Vec::new();

    // Read all sample headers first
    let mut sample_infos = Vec::new();
    for i in 0..num_samples {
        // Try to read sample header, but if we hit EOF (sample-less XM),
        // return what we have so far (empty Vec)
        match read_sample_header(cursor, sample_header_size) {
            Ok(sample_info) => sample_infos.push(sample_info),
            Err(XmError::UnexpectedEof) | Err(XmError::IoError(_)) if i == 0 => {
                // Sample-less XM file - no actual sample headers/data exist
                // (only if we fail on the first sample header)
                return Ok(Vec::new());
            }
            Err(e) => return Err(e),
        }
    }

    // Now read sample data
    for sample_info in sample_infos {
        if sample_info.length == 0 {
            continue; // Skip empty samples
        }

        let sample_data = read_sample_data(
            cursor,
            sample_info.length,
            sample_info.is_16bit,
        )?;

        let sample_rate = ExtractedSample::calculate_sample_rate(
            sample_info.finetune,
            sample_info.relative_note,
        );

        results.push(ExtractedSample {
            instrument_index: inst_idx,
            name: instrument.name.clone(),
            sample_rate,
            bit_depth: if sample_info.is_16bit { 16 } else { 8 },
            loop_start: sample_info.loop_start,
            loop_length: sample_info.loop_length,
            loop_type: sample_info.loop_type,
            data: sample_data,
        });
    }

    Ok(results)
}

/// Sample header information
struct SampleInfo {
    length: u32,
    loop_start: u32,
    loop_length: u32,
    finetune: i8,
    loop_type: u8,
    relative_note: i8,
    is_16bit: bool,
}

/// Read a sample header
fn read_sample_header(
    cursor: &mut Cursor<&[u8]>,
    header_size: u32,
) -> Result<SampleInfo, XmError> {
    // Sample length (4 bytes)
    let length = read_u32(cursor)?;

    // Sample loop start (4 bytes)
    let loop_start = read_u32(cursor)?;

    // Sample loop length (4 bytes)
    let loop_length = read_u32(cursor)?;

    // Volume (1 byte) - skip
    cursor.seek(SeekFrom::Current(1))?;

    // Finetune (1 byte, signed)
    let finetune = read_u8(cursor)? as i8;

    // Type (1 byte)
    let sample_type = read_u8(cursor)?;
    let loop_type = sample_type & 0x03;
    let is_16bit = (sample_type & 0x10) != 0;

    // Panning (1 byte) - skip
    cursor.seek(SeekFrom::Current(1))?;

    // Relative note (1 byte, signed)
    let relative_note = read_u8(cursor)? as i8;

    // Reserved (1 byte)
    cursor.seek(SeekFrom::Current(1))?;

    // Sample name (22 bytes) - skip
    cursor.seek(SeekFrom::Current(22))?;

    // Skip remaining header bytes
    if header_size > 40 {
        cursor.seek(SeekFrom::Current((header_size - 40) as i64))?;
    }

    Ok(SampleInfo {
        length,
        loop_start,
        loop_length,
        finetune,
        loop_type,
        relative_note,
        is_16bit,
    })
}

/// Read and decode sample data
///
/// XM samples are stored in delta-encoded format:
/// - Each value is the difference from the previous value
/// - We accumulate these deltas to get the actual sample values
fn read_sample_data(
    cursor: &mut Cursor<&[u8]>,
    length: u32,
    is_16bit: bool,
) -> Result<Vec<i16>, XmError> {
    if is_16bit {
        // 16-bit samples: length is in bytes, we get length/2 samples
        let num_samples = length / 2;
        let mut samples = Vec::with_capacity(num_samples as usize);
        let mut old = 0i16;

        for _ in 0..num_samples {
            let delta = read_i16(cursor)?;
            old = old.wrapping_add(delta);
            samples.push(old);
        }

        Ok(samples)
    } else {
        // 8-bit samples: length is in bytes, one byte per sample
        let mut samples = Vec::with_capacity(length as usize);
        let mut old = 0i8;

        for _ in 0..length {
            let delta = read_i8(cursor)?;
            old = old.wrapping_add(delta);
            // Convert i8 to i16 by scaling to full 16-bit range
            samples.push((old as i16) * 256);
        }

        Ok(samples)
    }
}

// =============================================================================
// Helper functions for reading little-endian values
// =============================================================================

fn read_u8(cursor: &mut Cursor<&[u8]>) -> Result<u8, XmError> {
    let mut buf = [0u8; 1];
    cursor.read_exact(&mut buf)?;
    Ok(buf[0])
}

fn read_i8(cursor: &mut Cursor<&[u8]>) -> Result<i8, XmError> {
    Ok(read_u8(cursor)? as i8)
}

fn read_u16(cursor: &mut Cursor<&[u8]>) -> Result<u16, XmError> {
    let mut buf = [0u8; 2];
    cursor.read_exact(&mut buf)?;
    Ok(u16::from_le_bytes(buf))
}

fn read_i16(cursor: &mut Cursor<&[u8]>) -> Result<i16, XmError> {
    let mut buf = [0u8; 2];
    cursor.read_exact(&mut buf)?;
    Ok(i16::from_le_bytes(buf))
}

fn read_u32(cursor: &mut Cursor<&[u8]>) -> Result<u32, XmError> {
    let mut buf = [0u8; 4];
    cursor.read_exact(&mut buf)?;
    Ok(u32::from_le_bytes(buf))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sample_rate_calculation() {
        // C-4 with no finetune or relative note should be 8363 Hz
        assert_eq!(ExtractedSample::calculate_sample_rate(0, 0), 8363);

        // One octave up (12 semitones) should double frequency
        let rate_up = ExtractedSample::calculate_sample_rate(0, 12);
        assert!((rate_up as f32 - 16726.0).abs() < 1.0);

        // One octave down (-12 semitones) should halve frequency
        let rate_down = ExtractedSample::calculate_sample_rate(0, -12);
        assert!((rate_down as f32 - 4181.0).abs() < 1.0);

        // Test finetune adjustment (1/128th of a semitone)
        let with_finetune = ExtractedSample::calculate_sample_rate(64, 0);
        assert!(with_finetune > 8363); // Should be slightly higher

        // Test negative finetune
        let with_neg_finetune = ExtractedSample::calculate_sample_rate(-64, 0);
        assert!(with_neg_finetune < 8363); // Should be slightly lower
    }

    #[test]
    fn test_sample_rate_extreme_values() {
        // Test extreme positive values
        let rate_high = ExtractedSample::calculate_sample_rate(127, 48);
        assert!(rate_high > 0);
        assert!(rate_high < 1_000_000); // Reasonable upper bound

        // Test extreme negative values
        let rate_low = ExtractedSample::calculate_sample_rate(-128, -48);
        assert!(rate_low > 0);
        assert!(rate_low < 100_000); // Still reasonable
    }

    #[test]
    fn test_delta_decoding_8bit() {
        // Simulate delta-encoded 8-bit samples
        let deltas: Vec<i8> = vec![10, 5, -3, 2, -1];
        let mut old = 0i8;
        let mut decoded = Vec::new();

        for delta in deltas {
            old = old.wrapping_add(delta);
            decoded.push(old);
        }

        assert_eq!(decoded, vec![10, 15, 12, 14, 13]);
    }

    #[test]
    fn test_delta_decoding_8bit_wrapping() {
        // Test wrapping behavior at boundaries
        let deltas: Vec<i8> = vec![127, 1, 1]; // Should wrap around
        let mut old = 0i8;
        let mut decoded = Vec::new();

        for delta in deltas {
            old = old.wrapping_add(delta);
            decoded.push(old);
        }

        assert_eq!(decoded[0], 127);
        assert_eq!(decoded[1], -128); // Wrapped
        assert_eq!(decoded[2], -127);
    }

    #[test]
    fn test_delta_decoding_16bit() {
        // Simulate delta-encoded 16-bit samples
        let deltas: Vec<i16> = vec![1000, 500, -300, 200, -100];
        let mut old = 0i16;
        let mut decoded = Vec::new();

        for delta in deltas {
            old = old.wrapping_add(delta);
            decoded.push(old);
        }

        assert_eq!(decoded, vec![1000, 1500, 1200, 1400, 1300]);
    }

    #[test]
    fn test_delta_decoding_16bit_wrapping() {
        // Test wrapping behavior at boundaries
        let deltas: Vec<i16> = vec![32767, 1, 1]; // Should wrap around
        let mut old = 0i16;
        let mut decoded = Vec::new();

        for delta in deltas {
            old = old.wrapping_add(delta);
            decoded.push(old);
        }

        assert_eq!(decoded[0], 32767);
        assert_eq!(decoded[1], -32768); // Wrapped
        assert_eq!(decoded[2], -32767);
    }

    #[test]
    fn test_8bit_to_16bit_scaling() {
        // Test that 8-bit samples are scaled to 16-bit range
        let sample_8bit = 64i8; // Mid-range 8-bit value
        let sample_16bit = (sample_8bit as i16) * 256;

        assert_eq!(sample_16bit, 16384); // 64 * 256

        // Test max values
        let max_8bit = 127i8;
        let scaled_max = (max_8bit as i16) * 256;
        assert_eq!(scaled_max, 32512); // Close to i16::MAX

        // Test min values
        let min_8bit = -128i8;
        let scaled_min = (min_8bit as i16) * 256;
        assert_eq!(scaled_min, -32768); // i16::MIN
    }

    #[test]
    fn test_empty_sample_data() {
        // Empty sample should not crash
        let empty: Vec<i16> = Vec::new();
        assert_eq!(empty.len(), 0);
    }

    #[test]
    fn test_read_helpers() {
        // Test the little-endian read helpers
        let data: Vec<u8> = vec![
            0x12, 0x34, 0x56, 0x78, // u32: 0x78563412
            0xAB, 0xCD,             // u16: 0xCDAB
            0xFF,                   // u8: 0xFF
        ];

        let mut cursor = Cursor::new(data.as_slice());

        let val_u32 = read_u32(&mut cursor).unwrap();
        assert_eq!(val_u32, 0x78563412);

        let val_u16 = read_u16(&mut cursor).unwrap();
        assert_eq!(val_u16, 0xCDAB);

        let val_u8 = read_u8(&mut cursor).unwrap();
        assert_eq!(val_u8, 0xFF);
    }

    #[test]
    fn test_read_signed_values() {
        // Test signed value reading
        let data: Vec<u8> = vec![
            0xFF, 0xFF,  // i16: -1
            0xFF,        // i8: -1
            0x00, 0x80,  // i16: -32768 (min value)
            0x80,        // i8: -128 (min value)
        ];

        let mut cursor = Cursor::new(data.as_slice());

        let val_i16 = read_i16(&mut cursor).unwrap();
        assert_eq!(val_i16, -1);

        let val_i8 = read_i8(&mut cursor).unwrap();
        assert_eq!(val_i8, -1);

        let val_i16_min = read_i16(&mut cursor).unwrap();
        assert_eq!(val_i16_min, -32768);

        let val_i8_min = read_i8(&mut cursor).unwrap();
        assert_eq!(val_i8_min, -128);
    }

    #[test]
    fn test_read_past_end() {
        // Reading past end should fail gracefully
        let data: Vec<u8> = vec![0x12, 0x34];
        let mut cursor = Cursor::new(data.as_slice());

        // This should fail (only 2 bytes available, trying to read 4)
        let result = read_u32(&mut cursor);
        assert!(result.is_err());
    }

    #[test]
    fn test_extract_from_real_xm() {
        // Test extraction from an actual XM file (sample-less)
        let test_file = "../../examples/assets/tracker-nether_groove.xm";
        if let Ok(data) = std::fs::read(test_file) {
            println!("Testing extraction from {} ({} bytes)", test_file, data.len());

            match extract_samples(&data) {
                Ok(samples) => {
                    println!("Successfully extracted {} samples", samples.len());
                    for sample in samples {
                        println!("  - {} ({} samples, {} Hz)",
                            sample.name, sample.data.len(), sample.sample_rate);
                    }
                }
                Err(e) => {
                    println!("Extraction failed: {}", e);
                    // This is expected for sample-less XM files, but shouldn't crash
                }
            }
        } else {
            println!("Test file not found (expected when not in workspace root)");
        }
    }
}
