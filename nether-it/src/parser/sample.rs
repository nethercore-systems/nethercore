//! Sample header parsing and sample data loading

use std::io::{Cursor, Read, Seek, SeekFrom};

use crate::SAMPLE_MAGIC;
use crate::compression::decompress_it;
use crate::error::ItError;
use crate::module::{ItSample, ItSampleFlags};

use super::helpers::{read_string, read_u8, read_u32};

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
    let convert_flags = read_u8(cursor)?;

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
            convert_flags,
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
    if sample.length == 0 {
        // No sample data
        return if sample.flags.contains(ItSampleFlags::SAMPLE_16BIT) {
            Ok(SampleData::I16(Vec::new()))
        } else {
            Ok(SampleData::I8(Vec::new()))
        };
    }

    let offset = sample_offset as usize;
    if offset == 0 || offset >= data.len() {
        return Err(ItError::InvalidSampleOffset(sample_offset));
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
        let mut channels = Vec::with_capacity(channel_count);
        let mut position = offset;
        for _ in 0..channel_count {
            let (decoded, used) = decompress_it(
                &data[position..],
                samples_per_channel,
                if is_16bit { 16 } else { 8 },
                sample.convert_flags & 4 != 0,
            )?;
            position += used;
            channels.push(decoded);
        }
        if is_16bit {
            let mut pcm = Vec::with_capacity(total_samples);
            for i in 0..samples_per_channel {
                for channel in &channels {
                    pcm.push(channel[i] as i16);
                }
            }
            Ok(SampleData::I16(pcm))
        } else {
            let mut pcm = Vec::with_capacity(total_samples);
            for i in 0..samples_per_channel {
                for channel in &channels {
                    pcm.push(channel[i] as i8);
                }
            }
            Ok(SampleData::I8(pcm))
        }
    } else {
        // IT stores stereo PCM as separate channel planes. Conversion state
        // starts at zero independently for each plane, not for each frame.
        let width = if sample.is_16bit() { 2 } else { 1 };
        let channels = if sample.is_stereo() { 2 } else { 1 };
        let frames = sample.length as usize;
        let bytes = frames
            .checked_mul(channels)
            .and_then(|n| n.checked_mul(width))
            .ok_or(ItError::UnexpectedEof)?;
        let end = offset.checked_add(bytes).ok_or(ItError::UnexpectedEof)?;
        let input = data.get(offset..end).ok_or(ItError::UnexpectedEof)?;
        if sample.convert_flags == 0xff && width == 1 {
            return Err(ItError::DecompressionError(
                "ModPlugin ADPCM is not PCM".into(),
            ));
        }
        let mut output = vec![0i16; frames * channels];
        for channel in 0..channels {
            let mut delta = 0u16;
            for frame in 0..frames {
                let pos = (channel * frames + frame) * width;
                let value = if width == 2 && sample.convert_flags & 8 != 0 {
                    // PTM byte-delta conversion takes precedence over endian/delta flags.
                    delta = delta.wrapping_add(input[pos] as u16);
                    let low = delta & 255;
                    delta = delta.wrapping_add(input[pos + 1] as u16);
                    low | (delta << 8)
                } else {
                    let raw = if width == 1 {
                        input[pos] as u16
                    } else if sample.convert_flags & 2 != 0 {
                        u16::from_be_bytes([input[pos], input[pos + 1]])
                    } else {
                        u16::from_le_bytes([input[pos], input[pos + 1]])
                    };
                    if sample.convert_flags & 4 != 0 {
                        delta = delta.wrapping_add(raw);
                        delta
                    } else if sample.convert_flags & 1 == 0 {
                        raw.wrapping_sub(if width == 1 { 128 } else { 32768 })
                    } else {
                        raw
                    }
                };
                output[frame * channels + channel] = value as i16;
            }
        }
        if width == 2 {
            Ok(SampleData::I16(output))
        } else {
            Ok(SampleData::I8(
                output.into_iter().map(|v| v as i8).collect(),
            ))
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
        // Create sample at offset 1000 (arbitrary)
        let mut it_data = vec![0; 1000];
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
            convert_flags: 5,                 // Signed, double delta (IT215)
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
            convert_flags: 5, // Signed, double delta (IT215)
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
    fn compressed_stereo_keeps_channel_boundaries_and_cvt_mode() {
        for bits in [8, 16] {
            let mut sample = ItSample {
                length: 3,
                flags: ItSampleFlags::COMPRESSED | ItSampleFlags::STEREO,
                convert_flags: 5,
                ..Default::default()
            };
            let mut data = vec![0];
            if bits == 16 {
                sample.flags = sample.flags | ItSampleFlags::SAMPLE_16BIT;
                data.extend(crate::compression::compress_it215_16bit(&[1, 3, 6]));
                data.extend(crate::compression::compress_it215_16bit(&[-1, -3, -6]));
            } else {
                data.extend(crate::compression::compress_it215_8bit(&[1, 3, 6]));
                data.extend(crate::compression::compress_it215_8bit(&[-1, -3, -6]));
            }
            for (cvt, expected) in [
                (5, vec![1, -1, 3, -3, 6, -6]),
                (1, vec![1, -1, 2, -2, 3, -3]),
            ] {
                sample.convert_flags = cvt;
                let decoded: Vec<i16> = match load_sample_data(&data, 1, &sample).unwrap() {
                    SampleData::I8(v) => v.into_iter().map(i16::from).collect(),
                    SampleData::I16(v) => v,
                };
                assert_eq!(decoded, expected);
            }
            data.pop();
            assert!(load_sample_data(&data, 1, &sample).is_err());
        }
    }

    #[test]
    fn pcm_conversion_flags_preserve_values_and_channel_state() {
        // ITTools.cpp GetSampleFormat / SampleDecode.hpp, pinned in compatibility doc.
        for (bits, cvt, bytes, expected) in [
            (8, 0, vec![0, 128, 255], vec![-128, 0, 127]),
            (8, 4, vec![1, 2, 252], vec![1, 3, -1]),
            (16, 0, vec![0, 0, 0, 128, 255, 255], vec![-32768, 0, 32767]),
            (16, 3, vec![0, 1, 127, 255, 128, 0], vec![1, 32767, -32768]),
            (16, 6, vec![0, 1, 0, 2, 255, 252], vec![1, 3, -1]),
            (16, 8, vec![1, 2, 4, 8], vec![0x0301, 0x0f07]),
        ] {
            let sample = ItSample {
                flags: if bits == 16 {
                    ItSampleFlags::SAMPLE_16BIT
                } else {
                    ItSampleFlags::empty()
                },
                convert_flags: cvt,
                length: expected.len() as u32,
                ..Default::default()
            };
            let mut data = vec![0];
            data.extend(bytes);
            let actual: Vec<i16> = match load_sample_data(&data, 1, &sample).unwrap() {
                SampleData::I8(v) => v.into_iter().map(i16::from).collect(),
                SampleData::I16(v) => v,
            };
            assert_eq!(actual, expected, "bits={bits} cvt={cvt}");
        }
        let stereo = ItSample {
            flags: ItSampleFlags::STEREO,
            convert_flags: 4,
            length: 2,
            ..Default::default()
        };
        match load_sample_data(&[0, 1, 2, 255, 254], 1, &stereo).unwrap() {
            SampleData::I8(v) => assert_eq!(v, [1, -1, 3, -3]),
            _ => panic!("wrong sample width"),
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

/// Find the framed end without decoding or interpreting sample payload as tags.
pub(super) fn sample_data_end(data: &[u8], info: &SampleInfo) -> Result<usize, ItError> {
    if info.sample.length == 0 || !info.sample.flags.contains(ItSampleFlags::HAS_DATA) {
        return Ok(0);
    }
    if info.data_offset == 0 {
        return Err(ItError::InvalidSampleOffset(0));
    }
    let flags = info.sample.flags;
    let channels = if flags.contains(ItSampleFlags::STEREO) {
        2
    } else {
        1
    };
    let width = if flags.contains(ItSampleFlags::SAMPLE_16BIT) {
        2
    } else {
        1
    };
    let mut end = info.data_offset as usize;
    if flags.contains(ItSampleFlags::COMPRESSED) {
        for _ in 0..channels {
            let blocks = (info.sample.length as usize).div_ceil(32768 / width);
            for _ in 0..blocks {
                let header = data
                    .get(end..end.checked_add(2).ok_or(ItError::UnexpectedEof)?)
                    .ok_or(ItError::UnexpectedEof)?;
                let length = u16::from_le_bytes([header[0], header[1]]) as usize;
                end = end.checked_add(2 + length).ok_or(ItError::UnexpectedEof)?;
                if end > data.len() {
                    return Err(ItError::UnexpectedEof);
                }
            }
        }
    } else {
        end = end
            .checked_add(
                (info.sample.length as usize)
                    .checked_mul(width * channels)
                    .ok_or(ItError::UnexpectedEof)?,
            )
            .ok_or(ItError::UnexpectedEof)?;
    }
    if end > data.len() {
        return Err(ItError::UnexpectedEof);
    }
    Ok(end)
}

#[test]
fn metadata_boundary_counts_pcm_width_and_compressed_stereo_blocks() {
    let mut info = SampleInfo {
        data_offset: 3,
        sample: ItSample {
            length: 10,
            flags: ItSampleFlags::HAS_DATA | ItSampleFlags::COMPRESSED | ItSampleFlags::STEREO,
            ..Default::default()
        },
    };
    // Framing only: two channels, each with its own declared compressed block.
    let data = [0, 0, 0, 3, 0, 11, 22, 33, 2, 0, 44, 55];
    assert_eq!(sample_data_end(&data, &info).unwrap(), 12);
    for length in 0..data.len() {
        assert!(sample_data_end(&data[..length], &info).is_err());
    }
    info.sample.length = 32769;
    assert!(sample_data_end(&data, &info).is_err());
    info.sample.length = 1;
    info.sample.flags =
        ItSampleFlags::HAS_DATA | ItSampleFlags::SAMPLE_16BIT | ItSampleFlags::STEREO;
    assert_eq!(sample_data_end(&data, &info).unwrap(), 7);
    info.data_offset = 0;
    assert!(matches!(
        sample_data_end(&data, &info),
        Err(ItError::InvalidSampleOffset(0))
    ));
    info.sample.flags = ItSampleFlags::default();
    assert_eq!(sample_data_end(&[], &info).unwrap(), 0);
}
