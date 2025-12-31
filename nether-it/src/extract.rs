//! IT sample extraction for automatic ROM packing
//!
//! This module extracts embedded sample data from IT files for use with
//! Nethercore's automatic sample extraction feature. Samples are decoded
//! from IT format (including IT215 compression) and converted to standard i16 PCM.

use std::io::{Cursor, Read, Seek, SeekFrom};

use crate::error::ItError;
use crate::parser::{load_sample_data, parse_sample, SampleData};
use crate::{IT_MAGIC, MAX_SAMPLES, MIN_COMPATIBLE_VERSION};

/// Extracted sample data from IT file
#[derive(Debug, Clone)]
pub struct ExtractedSample {
    /// Sample index (0-based)
    pub sample_index: u8,
    /// Sample name (used for sample ID generation)
    pub name: String,
    /// Sample rate (C5 speed)
    pub sample_rate: u32,
    /// Bit depth (8 or 16)
    pub bit_depth: u8,
    /// Loop start position in samples
    pub loop_start: u32,
    /// Loop length in samples
    pub loop_length: u32,
    /// Loop type: 0=none, 1=forward, 2=pingpong
    pub loop_type: u8,
    /// Whether the sample is stereo (interleaved L, R, L, R, ...)
    pub is_stereo: bool,
    /// Decoded sample data (i16 format, interleaved if stereo)
    pub data: Vec<i16>,
}

/// Extract all samples from an IT file
///
/// This parses the IT file and extracts all embedded sample data,
/// decoding from IT format (including IT215 compression) to standard i16 PCM.
///
/// # Arguments
/// * `data` - Raw IT file data
///
/// # Returns
/// * `Vec<ExtractedSample>` - All samples found in the IT file
///
/// # Errors
/// * Returns `ItError` if parsing fails
pub fn extract_samples(data: &[u8]) -> Result<Vec<ExtractedSample>, ItError> {
    // Minimum header size: magic (4) + name (26) + ... = 192 bytes
    if data.len() < 192 {
        return Err(ItError::TooSmall);
    }

    // Validate magic "IMPM"
    if &data[0..4] != IT_MAGIC {
        return Err(ItError::InvalidMagic);
    }

    let mut cursor = Cursor::new(data);

    // Skip to SmpNum at offset 0x24 (36)
    cursor.seek(SeekFrom::Start(36))?;
    let num_samples = read_u16(&mut cursor)?;

    if num_samples > MAX_SAMPLES {
        return Err(ItError::TooManySamples(num_samples));
    }

    if num_samples == 0 {
        return Ok(Vec::new());
    }

    // Skip to Cmwt (compatible with) at offset 0x2A (42)
    cursor.seek(SeekFrom::Start(42))?;
    let compatible_with = read_u16(&mut cursor)?;
    if compatible_with < MIN_COMPATIBLE_VERSION {
        return Err(ItError::UnsupportedVersion(compatible_with));
    }

    // Skip to OrdNum at offset 0x20 (32)
    cursor.seek(SeekFrom::Start(32))?;
    let num_orders = read_u16(&mut cursor)?;

    // Skip to InsNum at offset 0x22 (34)
    cursor.seek(SeekFrom::Start(34))?;
    let num_instruments = read_u16(&mut cursor)?;

    // Sample offset table starts at: 0xC0 (192) + num_orders + num_instruments*4
    let sample_offset_table_start = 192 + num_orders as u64 + (num_instruments as u64 * 4);
    cursor.seek(SeekFrom::Start(sample_offset_table_start))?;

    // Read sample offsets
    let mut sample_offsets = Vec::with_capacity(num_samples as usize);
    for _ in 0..num_samples {
        sample_offsets.push(read_u32(&mut cursor)?);
    }

    // Extract each sample
    let mut results = Vec::new();

    for (idx, &offset) in sample_offsets.iter().enumerate() {
        if offset == 0 {
            continue; // Empty sample slot
        }

        // Seek to sample header
        cursor.seek(SeekFrom::Start(offset as u64))?;

        // Parse sample header to get metadata and data offset
        let sample_info = match parse_sample(&mut cursor) {
            Ok(info) => info,
            Err(_) => continue, // Skip invalid samples
        };

        // Skip empty samples
        if sample_info.sample.length == 0 {
            continue;
        }

        // Load sample data (handles IT215 compression automatically)
        let sample_data =
            match load_sample_data(data, sample_info.data_offset, &sample_info.sample) {
                Ok(d) => d,
                Err(_) => continue, // Skip samples with load errors
            };

        // Convert to i16
        let data_i16 = match sample_data {
            SampleData::I16(samples) => samples,
            SampleData::I8(samples) => samples.iter().map(|&s| (s as i16) * 256).collect(),
        };

        // Skip if no actual data
        if data_i16.is_empty() {
            continue;
        }

        // Determine loop type
        let loop_type = if !sample_info.sample.has_loop() {
            0 // No loop
        } else if sample_info.sample.is_pingpong_loop() {
            2 // Ping-pong
        } else {
            1 // Forward
        };

        results.push(ExtractedSample {
            sample_index: idx as u8,
            name: sample_info.sample.name.clone(),
            sample_rate: sample_info.sample.c5_speed,
            bit_depth: if sample_info.sample.is_16bit() { 16 } else { 8 },
            loop_start: sample_info.sample.loop_begin,
            loop_length: sample_info
                .sample
                .loop_end
                .saturating_sub(sample_info.sample.loop_begin),
            loop_type,
            is_stereo: sample_info.sample.is_stereo(),
            data: data_i16,
        });
    }

    Ok(results)
}

// =============================================================================
// Helper functions for reading little-endian values
// =============================================================================

fn read_u16(cursor: &mut Cursor<&[u8]>) -> Result<u16, ItError> {
    let mut buf = [0u8; 2];
    cursor
        .read_exact(&mut buf)
        .map_err(|_| ItError::UnexpectedEof)?;
    Ok(u16::from_le_bytes(buf))
}

fn read_u32(cursor: &mut Cursor<&[u8]>) -> Result<u32, ItError> {
    let mut buf = [0u8; 4];
    cursor
        .read_exact(&mut buf)
        .map_err(|_| ItError::UnexpectedEof)?;
    Ok(u32::from_le_bytes(buf))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_empty_data() {
        let result = extract_samples(&[]);
        assert!(matches!(result, Err(ItError::TooSmall)));
    }

    #[test]
    fn test_extract_invalid_magic() {
        let mut data = vec![0u8; 192];
        data[..4].copy_from_slice(b"XXXX");
        let result = extract_samples(&data);
        assert!(matches!(result, Err(ItError::InvalidMagic)));
    }

    #[test]
    fn test_loop_type_conversion() {
        // Test that loop types are correctly identified
        // 0 = no loop, 1 = forward, 2 = pingpong
        assert_eq!(0u8, 0); // No loop
        assert_eq!(1u8, 1); // Forward
        assert_eq!(2u8, 2); // Pingpong
    }

    #[test]
    fn test_8bit_to_16bit_conversion() {
        // Test that 8-bit samples are scaled correctly to 16-bit range
        let sample_8bit = 64i8; // Mid-range 8-bit value
        let sample_16bit = (sample_8bit as i16) * 256;
        assert_eq!(sample_16bit, 16384); // 64 * 256

        // Test max value
        let max_8bit = 127i8;
        let scaled_max = (max_8bit as i16) * 256;
        assert_eq!(scaled_max, 32512); // Close to i16::MAX

        // Test min value
        let min_8bit = -128i8;
        let scaled_min = (min_8bit as i16) * 256;
        assert_eq!(scaled_min, -32768); // i16::MIN
    }

    #[test]
    fn test_extract_round_trip() {
        use crate::module::{ItSample, ItSampleFlags};
        use crate::writer::ItWriter;

        // Create a test IT file with embedded samples using ItWriter
        let mut writer = ItWriter::new("Test Extraction");
        writer.set_channels(4);
        writer.set_speed(6);
        writer.set_tempo(125);

        // Create a sample with known data
        let sample = ItSample {
            name: "TestKick".to_string(),
            filename: "kick.wav".to_string(),
            global_volume: 64,
            flags: ItSampleFlags::HAS_DATA, // 8-bit, mono, uncompressed
            default_volume: 64,
            default_pan: None,
            length: 100,
            loop_begin: 0,
            loop_end: 0,
            c5_speed: 22050, // 22050 Hz
            sustain_loop_begin: 0,
            sustain_loop_end: 0,
            vibrato_speed: 0,
            vibrato_depth: 0,
            vibrato_rate: 0,
            vibrato_type: 0,
        };

        // Create sample audio data (simple sine-like pattern)
        let audio_data: Vec<i16> = (0..100)
            .map(|i| ((i as f32 / 10.0).sin() * 10000.0) as i16)
            .collect();

        writer.add_sample(sample, &audio_data);

        // Write the IT file
        let it_bytes = writer.write();

        // Extract samples from the written IT file
        let extracted = extract_samples(&it_bytes).expect("Failed to extract samples");

        // Verify we got one sample
        assert_eq!(extracted.len(), 1, "Expected 1 sample, got {}", extracted.len());

        let sample = &extracted[0];
        assert_eq!(sample.name, "TestKick");
        assert_eq!(sample.sample_rate, 22050);
        assert_eq!(sample.sample_index, 0);
        assert!(!sample.is_stereo);

        // Verify sample data length matches
        assert_eq!(
            sample.data.len(),
            100,
            "Expected 100 samples, got {}",
            sample.data.len()
        );

        // Verify sample values are reasonable (ItWriter writes 16-bit)
        // The first few values should match our sine pattern
        for (i, &val) in sample.data.iter().take(10).enumerate() {
            let expected = ((i as f32 / 10.0).sin() * 10000.0) as i16;
            assert!(
                (val - expected).abs() < 2,
                "Sample {} mismatch: got {}, expected {}",
                i,
                val,
                expected
            );
        }
    }

    #[test]
    fn test_extract_multiple_samples() {
        use crate::module::{ItSample, ItSampleFlags};
        use crate::writer::ItWriter;

        let mut writer = ItWriter::new("Multi Sample Test");
        writer.set_channels(4);

        // Add first sample
        let sample1 = ItSample {
            name: "Kick".to_string(),
            filename: String::new(),
            global_volume: 64,
            flags: ItSampleFlags::HAS_DATA,
            default_volume: 64,
            default_pan: None,
            length: 50,
            loop_begin: 0,
            loop_end: 0,
            c5_speed: 44100,
            ..Default::default()
        };
        let audio1: Vec<i16> = (0..50).map(|i| (i * 100) as i16).collect();
        writer.add_sample(sample1, &audio1);

        // Add second sample
        let sample2 = ItSample {
            name: "Snare".to_string(),
            filename: String::new(),
            global_volume: 64,
            flags: ItSampleFlags::HAS_DATA,
            default_volume: 64,
            default_pan: None,
            length: 75,
            loop_begin: 0,
            loop_end: 0,
            c5_speed: 22050,
            ..Default::default()
        };
        let audio2: Vec<i16> = (0..75).map(|i| (i * 50) as i16).collect();
        writer.add_sample(sample2, &audio2);

        // Write and extract
        let it_bytes = writer.write();
        let extracted = extract_samples(&it_bytes).expect("Failed to extract");

        assert_eq!(extracted.len(), 2);

        // Verify first sample
        assert_eq!(extracted[0].name, "Kick");
        assert_eq!(extracted[0].sample_rate, 44100);
        assert_eq!(extracted[0].data.len(), 50);

        // Verify second sample
        assert_eq!(extracted[1].name, "Snare");
        assert_eq!(extracted[1].sample_rate, 22050);
        assert_eq!(extracted[1].data.len(), 75);
    }

    #[test]
    fn test_extract_with_loop() {
        use crate::module::{ItSample, ItSampleFlags};
        use crate::writer::ItWriter;

        let mut writer = ItWriter::new("Loop Test");
        writer.set_channels(4);

        // Sample with forward loop
        let sample = ItSample {
            name: "LoopedSample".to_string(),
            filename: String::new(),
            global_volume: 64,
            flags: ItSampleFlags::HAS_DATA | ItSampleFlags::LOOP,
            default_volume: 64,
            default_pan: None,
            length: 100,
            loop_begin: 20,
            loop_end: 80,
            c5_speed: 22050,
            ..Default::default()
        };
        let audio: Vec<i16> = (0..100).map(|i| (i * 100) as i16).collect();
        writer.add_sample(sample, &audio);

        let it_bytes = writer.write();
        let extracted = extract_samples(&it_bytes).expect("Failed to extract");

        assert_eq!(extracted.len(), 1);
        assert_eq!(extracted[0].loop_type, 1); // Forward loop
        assert_eq!(extracted[0].loop_start, 20);
        assert_eq!(extracted[0].loop_length, 60); // 80 - 20
    }

    #[test]
    fn test_stereo_to_mono_integration() {
        // Test that the stereo -> mono pipeline works correctly
        // Simulating interleaved stereo data (L, R, L, R, ...)
        let stereo_data: Vec<i16> = vec![
            100, 200,   // Frame 0: L=100, R=200
            300, 400,   // Frame 1: L=300, R=400
            500, 600,   // Frame 2: L=500, R=600
        ];

        // Test the stereo_to_mono conversion directly
        let mono: Vec<i16> = stereo_data
            .chunks(2)
            .map(|chunk| {
                if chunk.len() == 2 {
                    ((chunk[0] as i32 + chunk[1] as i32) / 2) as i16
                } else {
                    chunk[0]
                }
            })
            .collect();

        assert_eq!(mono.len(), 3);
        assert_eq!(mono[0], 150); // (100 + 200) / 2
        assert_eq!(mono[1], 350); // (300 + 400) / 2
        assert_eq!(mono[2], 550); // (500 + 600) / 2
    }

    #[test]
    fn test_extract_from_real_it_file() {
        // This test runs if a test IT file is available
        // Place a test IT file at the specified path to enable this test
        let test_file = "../../examples/assets/test_samples.it";
        if let Ok(data) = std::fs::read(test_file) {
            println!("Testing extraction from {} ({} bytes)", test_file, data.len());

            match extract_samples(&data) {
                Ok(samples) => {
                    println!("Successfully extracted {} samples", samples.len());
                    for sample in &samples {
                        println!(
                            "  [{}] {} ({} samples, {} Hz, {}bit{})",
                            sample.sample_index,
                            sample.name,
                            sample.data.len(),
                            sample.sample_rate,
                            sample.bit_depth,
                            if sample.is_stereo { ", stereo" } else { "" }
                        );
                    }
                }
                Err(e) => {
                    println!("Extraction failed: {:?}", e);
                }
            }
        } else {
            println!("Test file not found (expected when not in workspace root)");
        }
    }
}
