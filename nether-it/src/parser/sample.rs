//! Sample header parsing and sample data loading

use std::io::{Cursor, Read, Seek, SeekFrom};

use crate::compression::{
    decompress_it215_16bit, decompress_it215_16bit_with_size, decompress_it215_8bit,
    decompress_it215_8bit_with_size,
};
use crate::error::ItError;
use crate::module::{ItSample, ItSampleFlags};
use crate::SAMPLE_MAGIC;

use super::helpers::{read_string, read_u32, read_u8};

/// Sample metadata including offset for loading sample data
#[derive(Debug, Clone)]
pub struct SampleInfo {
    /// Sample header information
    pub sample: ItSample,
    /// Offset to sample data in the IT file
    pub data_offset: u32,
}

/// Represents sample data loaded from an IT file
#[derive(Debug, Clone)]
pub enum SampleData {
    /// 8-bit signed samples
    I8(Vec<i8>),
    /// 16-bit signed samples
    I16(Vec<i16>),
}

/// Parse a single sample header
pub fn parse_sample(cursor: &mut Cursor<&[u8]>) -> Result<SampleInfo, ItError> {
    // Read magic "IMPS"
    let mut magic = [0u8; 4];
    cursor.read_exact(&mut magic)?;
    if &magic != SAMPLE_MAGIC {
        return Err(ItError::InvalidSample(0));
    }

    // DOS filename (12 bytes)
    let mut filename_bytes = [0u8; 12];
    cursor.read_exact(&mut filename_bytes)?;
    let filename = read_string(&filename_bytes);

    // Reserved (1 byte)
    cursor.seek(SeekFrom::Current(1))?;

    // GvL (global volume)
    let global_volume = read_u8(cursor)?;

    // Flg (flags)
    let flags = ItSampleFlags::from_bits(read_u8(cursor)?);

    // Vol (default volume)
    let default_volume = read_u8(cursor)?;

    // Sample name (26 bytes)
    let mut name_bytes = [0u8; 26];
    cursor.read_exact(&mut name_bytes)?;
    let name = read_string(&name_bytes);

    // Cvt (convert flags) - skip
    let _cvt = read_u8(cursor)?;

    // DfP (default pan)
    let dfp = read_u8(cursor)?;
    let default_pan = if dfp & 0x80 != 0 {
        Some(dfp & 0x7F)
    } else {
        None
    };

    // Length (4 bytes)
    let length = read_u32(cursor)?;

    // LoopBeg (4 bytes)
    let loop_begin = read_u32(cursor)?;

    // LoopEnd (4 bytes)
    let loop_end = read_u32(cursor)?;

    // C5Speed (4 bytes)
    let c5_speed = read_u32(cursor)?;

    // SusLBeg (4 bytes)
    let sustain_loop_begin = read_u32(cursor)?;

    // SusLEnd (4 bytes)
    let sustain_loop_end = read_u32(cursor)?;

    // SmpPoint (4 bytes) - offset to sample data
    let data_offset = read_u32(cursor)?;

    // ViS, ViD, ViR, ViT (vibrato)
    let vibrato_speed = read_u8(cursor)?;
    let vibrato_depth = read_u8(cursor)?;
    let vibrato_rate = read_u8(cursor)?;
    let vibrato_type = read_u8(cursor)?;

    Ok(SampleInfo {
        sample: ItSample {
            name,
            filename,
            global_volume,
            flags,
            default_volume,
            default_pan,
            length,
            loop_begin,
            loop_end,
            c5_speed,
            sustain_loop_begin,
            sustain_loop_end,
            vibrato_speed,
            vibrato_depth,
            vibrato_rate,
            vibrato_type,
        },
        data_offset,
    })
}

/// Load sample data from an IT file, automatically decompressing if needed
///
/// This function is useful for extracting sample data from IT files,
/// particularly when samples use IT215 compression. The nether-pack tool
/// strips samples during ROM packing, but this function is available for
/// tools that need to access the original sample data.
///
/// # Arguments
/// * `data` - Complete IT file bytes
/// * `sample_offset` - Offset to sample data (from sample header)
/// * `sample` - Sample header information (contains flags, length)
///
/// # Returns
/// * 8-bit samples: `Ok(SampleData::I8(Vec<i8>))`
/// * 16-bit samples: `Ok(SampleData::I16(Vec<i16>))`
pub fn load_sample_data(
    data: &[u8],
    sample_offset: u32,
    sample: &ItSample,
) -> Result<SampleData, ItError> {
    if sample_offset == 0 || sample.length == 0 {
        // No sample data
        return if sample.flags.contains(ItSampleFlags::SAMPLE_16BIT) {
            Ok(SampleData::I16(Vec::new()))
        } else {
            Ok(SampleData::I8(Vec::new()))
        };
    }

    let offset = sample_offset as usize;
    if offset >= data.len() {
        return Err(ItError::InvalidSample(0));
    }

    let is_16bit = sample.flags.contains(ItSampleFlags::SAMPLE_16BIT);
    let is_compressed = sample.flags.contains(ItSampleFlags::COMPRESSED);
    let is_stereo = sample.flags.contains(ItSampleFlags::STEREO);

    // For stereo, IT stores Left channel first, then Right channel (sequential, not interleaved)
    // We read both channels and interleave them for compatibility with standard audio processing
    let channel_count = if is_stereo { 2 } else { 1 };
    let samples_per_channel = sample.length as usize;
    let total_samples = samples_per_channel * channel_count;

    if is_compressed {
        // IT215 compression - for stereo, each channel is compressed separately
        let compressed_data = &data[offset..];

        if is_16bit {
            if is_stereo {
                // Decompress left channel and get bytes consumed to find right channel offset
                let (left, left_bytes_consumed) =
                    decompress_it215_16bit_with_size(compressed_data, samples_per_channel)?;

                // Right channel starts after left channel's compressed data
                let right_offset = left_bytes_consumed;
                let right = if right_offset < compressed_data.len() {
                    decompress_it215_16bit(&compressed_data[right_offset..], samples_per_channel)?
                } else {
                    // No right channel data available, fill with zeros
                    vec![0i16; samples_per_channel]
                };

                // Interleave left and right channels
                let mut interleaved = Vec::with_capacity(total_samples);
                for i in 0..samples_per_channel {
                    interleaved.push(left.get(i).copied().unwrap_or(0));
                    interleaved.push(right.get(i).copied().unwrap_or(0));
                }
                Ok(SampleData::I16(interleaved))
            } else {
                let samples = decompress_it215_16bit(compressed_data, samples_per_channel)?;
                Ok(SampleData::I16(samples))
            }
        } else if is_stereo {
            // Decompress left channel and get bytes consumed to find right channel offset
            let (left, left_bytes_consumed) =
                decompress_it215_8bit_with_size(compressed_data, samples_per_channel)?;

            // Right channel starts after left channel's compressed data
            let right_offset = left_bytes_consumed;
            let right = if right_offset < compressed_data.len() {
                decompress_it215_8bit(&compressed_data[right_offset..], samples_per_channel)?
            } else {
                // No right channel data available, fill with zeros
                vec![0i8; samples_per_channel]
            };

            // Interleave left and right channels
            let mut interleaved = Vec::with_capacity(total_samples);
            for i in 0..samples_per_channel {
                interleaved.push(left.get(i).copied().unwrap_or(0));
                interleaved.push(right.get(i).copied().unwrap_or(0));
            }
            Ok(SampleData::I8(interleaved))
        } else {
            let samples = decompress_it215_8bit(compressed_data, samples_per_channel)?;
            Ok(SampleData::I8(samples))
        }
    } else {
        // Uncompressed sample data
        let bytes_per_sample = if is_16bit { 2 } else { 1 };
        let total_size = total_samples * bytes_per_sample;

        if offset + total_size > data.len() {
            return Err(ItError::InvalidSample(0));
        }

        if is_16bit {
            if is_stereo {
                // Read left channel, then right channel, then interleave
                let mut interleaved = Vec::with_capacity(total_samples);
                for i in 0..samples_per_channel {
                    // Left sample
                    let left_idx = offset + i * 2;
                    let left = i16::from_le_bytes([data[left_idx], data[left_idx + 1]]);
                    // Right sample (offset by samples_per_channel * 2 bytes)
                    let right_idx = offset + (samples_per_channel + i) * 2;
                    let right = i16::from_le_bytes([data[right_idx], data[right_idx + 1]]);
                    interleaved.push(left);
                    interleaved.push(right);
                }
                Ok(SampleData::I16(interleaved))
            } else {
                let mut samples = Vec::with_capacity(samples_per_channel);
                for i in 0..samples_per_channel {
                    let idx = offset + i * 2;
                    let sample_val = i16::from_le_bytes([data[idx], data[idx + 1]]);
                    samples.push(sample_val);
                }
                Ok(SampleData::I16(samples))
            }
        } else if is_stereo {
            // Read left channel, then right channel, then interleave
            let mut interleaved = Vec::with_capacity(total_samples);
            for i in 0..samples_per_channel {
                let left = data[offset + i] as i8;
                let right = data[offset + samples_per_channel + i] as i8;
                interleaved.push(left);
                interleaved.push(right);
            }
            Ok(SampleData::I8(interleaved))
        } else {
            let samples: Vec<i8> = data[offset..offset + samples_per_channel]
                .iter()
                .map(|&b| b as i8)
                .collect();
            Ok(SampleData::I8(samples))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_sample_data_uncompressed_8bit() {
        // Create a simple uncompressed 8-bit sample
        let original_samples: Vec<i8> = vec![0, 10, -10, 50, -50, 127, -128, 0];

        // Build a minimal IT file with this sample
        let mut it_data = Vec::new();

        // Create sample at offset 1000 (arbitrary)
        it_data.resize(1000, 0);
        for &sample in &original_samples {
            it_data.push(sample as u8);
        }

        // Create sample header
        let sample = ItSample {
            name: "Test".into(),
            filename: "test.raw".into(),
            length: original_samples.len() as u32,
            flags: ItSampleFlags::empty(), // Uncompressed, 8-bit
            ..Default::default()
        };

        // Load the sample data
        let result = load_sample_data(&it_data, 1000, &sample).unwrap();

        match result {
            SampleData::I8(loaded) => {
                assert_eq!(loaded.len(), original_samples.len());
                assert_eq!(loaded, original_samples);
            }
            _ => panic!("Expected I8 sample data"),
        }
    }

    #[test]
    fn test_load_sample_data_uncompressed_16bit() {
        // Create a simple uncompressed 16-bit sample
        let original_samples: Vec<i16> = vec![0, 1000, -1000, 10000, -10000, 32767, -32768, 0];

        // Build sample data
        let mut it_data = vec![0; 1000];
        for &sample in &original_samples {
            it_data.extend_from_slice(&sample.to_le_bytes());
        }

        // Create sample header
        let sample = ItSample {
            name: "Test".into(),
            filename: "test.raw".into(),
            length: original_samples.len() as u32,
            flags: ItSampleFlags::SAMPLE_16BIT, // Uncompressed, 16-bit
            ..Default::default()
        };

        // Load the sample data
        let result = load_sample_data(&it_data, 1000, &sample).unwrap();

        match result {
            SampleData::I16(loaded) => {
                assert_eq!(loaded.len(), original_samples.len());
                assert_eq!(loaded, original_samples);
            }
            _ => panic!("Expected I16 sample data"),
        }
    }

    #[test]
    fn test_load_sample_data_compressed_8bit() {
        use crate::compression::compress_it215_8bit;

        // Create a sample
        let original_samples: Vec<i8> = vec![0, 10, -10, 50, -50, 127, -128, 0];

        // Compress it
        let compressed = compress_it215_8bit(&original_samples);

        // Build IT file data
        let mut it_data = vec![0; 1000];
        it_data.extend_from_slice(&compressed);

        // Create sample header with compression flag
        let sample = ItSample {
            name: "Test".into(),
            filename: "test.it".into(),
            length: original_samples.len() as u32,
            flags: ItSampleFlags::COMPRESSED, // Compressed, 8-bit
            ..Default::default()
        };

        // Load and decompress
        let result = load_sample_data(&it_data, 1000, &sample).unwrap();

        match result {
            SampleData::I8(loaded) => {
                assert_eq!(loaded.len(), original_samples.len());
                // Note: Due to delta encoding, values should match exactly
                for (i, (&loaded_val, &orig_val)) in
                    loaded.iter().zip(&original_samples).enumerate()
                {
                    assert_eq!(loaded_val, orig_val, "Mismatch at index {}", i);
                }
            }
            _ => panic!("Expected I8 sample data"),
        }
    }

    #[test]
    fn test_load_sample_data_compressed_16bit() {
        use crate::compression::compress_it215_16bit;

        // Create a sample
        let original_samples: Vec<i16> = vec![0, 1000, -1000, 10000, -10000, 32767, -32768, 0];

        // Compress it
        let compressed = compress_it215_16bit(&original_samples);

        // Build IT file data
        let mut it_data = vec![0; 1000];
        it_data.extend_from_slice(&compressed);

        // Create sample header with compression flag
        let sample = ItSample {
            name: "Test".into(),
            filename: "test.it".into(),
            length: original_samples.len() as u32,
            flags: ItSampleFlags::SAMPLE_16BIT | ItSampleFlags::COMPRESSED, // Compressed, 16-bit
            ..Default::default()
        };

        // Load and decompress
        let result = load_sample_data(&it_data, 1000, &sample).unwrap();

        match result {
            SampleData::I16(loaded) => {
                assert_eq!(loaded.len(), original_samples.len());
                // Values should match exactly
                for (i, (&loaded_val, &orig_val)) in
                    loaded.iter().zip(&original_samples).enumerate()
                {
                    assert_eq!(loaded_val, orig_val, "Mismatch at index {}", i);
                }
            }
            _ => panic!("Expected I16 sample data"),
        }
    }

    #[test]
    fn test_load_sample_data_empty() {
        let it_data = vec![0u8; 1000];

        let sample = ItSample {
            name: "Empty".into(),
            length: 0, // No samples
            ..Default::default()
        };

        let result = load_sample_data(&it_data, 0, &sample).unwrap();

        match result {
            SampleData::I8(loaded) => assert_eq!(loaded.len(), 0),
            _ => panic!("Expected I8 sample data"),
        }
    }
}
