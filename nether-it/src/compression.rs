//! IT215 sample compression/decompression
//!
//! Implements the IT215 compression algorithm used by Impulse Tracker
//! for sample data. This is a delta encoding scheme with variable-width
//! bit packing.

use crate::ItError;

/// Decompress IT215 8-bit sample data
///
/// # Arguments
/// * `compressed` - Compressed sample data
/// * `output_length` - Expected number of output samples
///
/// # Returns
/// Decompressed 8-bit signed samples
pub fn decompress_it215_8bit(compressed: &[u8], output_length: usize) -> Result<Vec<i8>, ItError> {
    let (samples, _bytes_consumed) = decompress_it215_8bit_with_size(compressed, output_length)?;
    Ok(samples)
}

/// Decompress IT215 8-bit sample data, returning bytes consumed
///
/// This is useful for stereo samples where we need to know where the right channel starts.
///
/// # Returns
/// (Decompressed samples, bytes consumed from input)
pub fn decompress_it215_8bit_with_size(
    compressed: &[u8],
    output_length: usize,
) -> Result<(Vec<i8>, usize), ItError> {
    let (samples, used) = decompress_it(compressed, output_length, 8, true)?;
    Ok((samples.into_iter().map(|v| v as i8).collect(), used))
}

/// Decompress IT215 16-bit sample data
///
/// # Arguments
/// * `compressed` - Compressed sample data
/// * `output_length` - Expected number of output samples (in samples, not bytes)
///
/// # Returns
/// Decompressed 16-bit signed samples
pub fn decompress_it215_16bit(
    compressed: &[u8],
    output_length: usize,
) -> Result<Vec<i16>, ItError> {
    let (samples, _bytes_consumed) = decompress_it215_16bit_with_size(compressed, output_length)?;
    Ok(samples)
}

/// Decompress IT215 16-bit sample data, returning bytes consumed
///
/// This is useful for stereo samples where we need to know where the right channel starts.
///
/// # Returns
/// (Decompressed samples, bytes consumed from input)
pub fn decompress_it215_16bit_with_size(
    compressed: &[u8],
    output_length: usize,
) -> Result<(Vec<i16>, usize), ItError> {
    let (samples, used) = decompress_it(compressed, output_length, 16, true)?;
    Ok((samples.into_iter().map(|v| v as i16).collect(), used))
}

// IT 2.14/2.15: byte-aligned bounded blocks and three width-change modes.
// Reference: OpenMPT ITCompression.cpp, ITDecompression::Uncompress,
// f83cedb0cd5446e4dfaa83ac97e3087107e26767.
pub(crate) fn decompress_it(
    data: &[u8],
    length: usize,
    bits: usize,
    it215: bool,
) -> Result<(Vec<i32>, usize), ItError> {
    let mut output = Vec::new();
    let mut offset = 0;
    let max_width = bits + 1;
    let block_size = 0x8000 / (bits / 8);
    while output.len() < length {
        let header = data.get(offset..offset + 2).ok_or(ItError::UnexpectedEof)?;
        let size = u16::from_le_bytes([header[0], header[1]]) as usize;
        offset += 2;
        let payload = data
            .get(offset..offset + size)
            .ok_or(ItError::UnexpectedEof)?;
        offset += size;
        if size == 0 {
            continue;
        }
        let mut reader = BitReader::new(payload);
        let end = output.len() + (length - output.len()).min(block_size);
        let (mut first, mut second) = (0i32, 0i32);
        let mut width = max_width;
        while output.len() < end {
            if width == 0 || width > max_width {
                return Err(ItError::DecompressionError("invalid bit width".into()));
            }
            let value = reader.read_bits(width)? as i32;
            let top = 1i32 << (width - 1);
            let change = if width <= 6 && value == top {
                Some(reader.read_bits(if bits == 8 { 3 } else { 4 })? as usize)
            } else if width > 6
                && width < max_width
                && (top - bits as i32 / 2..top + bits as i32 / 2).contains(&value)
            {
                Some((value - (top - bits as i32 / 2)) as usize)
            } else {
                None
            };
            if let Some(new_width) = change {
                let new_width = new_width + 1;
                width = new_width + usize::from(new_width >= width);
                continue;
            }
            if width == max_width && value & top != 0 {
                width = (value & !top) as usize + 1;
                continue;
            }
            let delta = if width < max_width && value & top != 0 {
                value - (top << 1)
            } else {
                value
            };
            first = first.wrapping_add(delta);
            second = second.wrapping_add(first);
            output.push(if it215 { second } else { first });
        }
    }
    Ok((output, offset))
}

/// Bit reader for compressed data
struct BitReader<'a> {
    data: &'a [u8],
    byte_pos: usize,
    bit_pos: u8,
}

impl<'a> BitReader<'a> {
    fn new(data: &'a [u8]) -> Self {
        Self {
            data,
            byte_pos: 0,
            bit_pos: 0,
        }
    }

    fn read_bits(&mut self, count: usize) -> Result<u32, ItError> {
        if count == 0 || count > 32 {
            return Ok(0);
        }

        let mut result = 0u32;
        let mut bits_read = 0;

        while bits_read < count {
            if self.byte_pos >= self.data.len() {
                return Err(ItError::UnexpectedEof);
            }

            let current_byte = self.data[self.byte_pos];
            let bits_left_in_byte = 8 - self.bit_pos as usize;
            let bits_to_read = (count - bits_read).min(bits_left_in_byte);

            // Extract bits from current byte
            // Handle the case where bits_to_read is 8 (1u8 << 8 would overflow)
            let mask = if bits_to_read >= 8 {
                0xFF
            } else {
                (1u8 << bits_to_read) - 1
            };
            let bits = (current_byte >> self.bit_pos) & mask;

            result |= (bits as u32) << bits_read;
            bits_read += bits_to_read;
            self.bit_pos += bits_to_read as u8;

            if self.bit_pos >= 8 {
                self.bit_pos = 0;
                self.byte_pos += 1;
            }
        }

        Ok(result)
    }
}

/// Compress 8-bit sample data using IT215 algorithm
///
/// Note: This is optional for the writer - we can also write uncompressed samples.
/// Currently unused but available for future optimization.
#[allow(dead_code)] // Used in tests
pub fn compress_it215_8bit(samples: &[i8]) -> Vec<u8> {
    // For simplicity, we'll implement a basic compression that works
    // but may not be optimal. The decompression is what matters most.
    let mut output = Vec::new();
    let mut writer = BitWriter::new();

    const BLOCK_SIZE: usize = 0x8000;

    for chunk in samples.chunks(BLOCK_SIZE) {
        compress_block_8bit(chunk, &mut writer);

        // Write block data
        let block_data = writer.finish();
        // Write compressed length (placeholder - we'll fix this)
        output.extend_from_slice(&(block_data.len() as u16).to_le_bytes());
        output.extend_from_slice(&block_data);

        writer = BitWriter::new();
    }

    output
}

/// Compress a single 8-bit block
fn compress_block_8bit(samples: &[i8], writer: &mut BitWriter) {
    let mut last_value: i8 = 0;
    let mut last_delta: i8 = 0;

    for &sample in samples {
        let first_delta = sample.wrapping_sub(last_value);
        let delta = first_delta.wrapping_sub(last_delta);
        last_delta = first_delta;

        // For simplicity, always use width 9 (no compression)
        // A full implementation would adaptively choose widths
        let value = if delta >= 0 {
            delta as u32
        } else {
            (delta as i32 + 256) as u32
        };

        writer.write_bits(value, 9);
        last_value = sample;
    }
}

/// Compress 16-bit sample data using IT215 algorithm
/// Currently unused but available for future optimization.
#[allow(dead_code)] // Used in tests
pub fn compress_it215_16bit(samples: &[i16]) -> Vec<u8> {
    let mut output = Vec::new();
    let mut writer = BitWriter::new();

    const BLOCK_SIZE: usize = 0x4000;

    for chunk in samples.chunks(BLOCK_SIZE) {
        compress_block_16bit(chunk, &mut writer);

        let block_data = writer.finish();
        output.extend_from_slice(&(block_data.len() as u16).to_le_bytes());
        output.extend_from_slice(&block_data);

        writer = BitWriter::new();
    }

    output
}

/// Compress a single 16-bit block
fn compress_block_16bit(samples: &[i16], writer: &mut BitWriter) {
    let mut last_value: i16 = 0;
    let mut last_delta: i16 = 0;

    for &sample in samples {
        let first_delta = sample.wrapping_sub(last_value);
        let delta = first_delta.wrapping_sub(last_delta);
        last_delta = first_delta;

        // For simplicity, always use width 17 (no compression)
        let value = if delta >= 0 {
            delta as u32
        } else {
            (delta as i32 + 0x10000) as u32
        };

        writer.write_bits(value, 17);
        last_value = sample;
    }
}

/// Bit writer for compression
#[allow(dead_code)] // Used by compress functions which are used in tests
struct BitWriter {
    data: Vec<u8>,
    current_byte: u8,
    bit_pos: u8,
}

#[allow(dead_code)] // Used by compress functions which are used in tests
impl BitWriter {
    fn new() -> Self {
        Self {
            data: Vec::new(),
            current_byte: 0,
            bit_pos: 0,
        }
    }

    fn write_bits(&mut self, value: u32, count: usize) {
        let mut value = value;
        let mut remaining = count;

        while remaining > 0 {
            let bits_left = 8 - self.bit_pos as usize;
            let bits_to_write = remaining.min(bits_left);

            let mask = (1u32 << bits_to_write) - 1;
            self.current_byte |= ((value & mask) as u8) << self.bit_pos;

            value >>= bits_to_write;
            remaining -= bits_to_write;
            self.bit_pos += bits_to_write as u8;

            if self.bit_pos >= 8 {
                self.data.push(self.current_byte);
                self.current_byte = 0;
                self.bit_pos = 0;
            }
        }
    }

    fn finish(mut self) -> Vec<u8> {
        if self.bit_pos > 0 {
            self.data.push(self.current_byte);
        }
        self.data
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Hand-packed streams, independent of our compressor. ITCompression.cpp
    // at OpenMPT f83cedb0: mode C 0x100 | (new_width-1), then mode A.
    #[test]
    fn compressed_width_markers_do_not_consume_samples() {
        let mut bits = BitWriter::new();
        bits.write_bits(0x102, 9); // width 3
        bits.write_bits(1, 3); // delta +1
        bits.write_bits(4, 3); // mode A width marker
        bits.write_bits(3, 3); // width 5 (skip current width)
        bits.write_bits(31, 5); // delta -1
        let payload = bits.finish();
        let mut stream = (payload.len() as u16).to_le_bytes().to_vec();
        stream.extend(payload);
        // IT215 integrates twice: first differences [1,0], samples [1,1].
        assert_eq!(decompress_it215_8bit(&stream, 2).unwrap(), [1, 1]);
    }

    #[test]
    fn compressed_block_length_bounds_reads_and_channel_offset() {
        let stream = [3, 0, 1, 0, 0xab, 0xde, 0xad];
        let (samples, used) = decompress_it215_8bit_with_size(&stream, 1).unwrap();
        assert_eq!(samples, [1]);
        assert_eq!(used, 5); // include unused payload padding, exclude next channel
        assert!(decompress_it215_8bit(&[1, 0, 1, 0, 0], 1).is_err());
        assert!(decompress_it215_8bit(&[], 1).is_err());
        assert!(decompress_it215_8bit(&[2, 0, 0, 0], usize::MAX).is_err());
    }

    #[test]
    fn every_width_and_delta_mode_decodes_independent_streams() {
        for bits in [8usize, 16] {
            for target in 1..=bits {
                let mut writer = BitWriter::new();
                writer.write_bits((1 << bits) | (target - 1) as u32, bits + 1);
                writer.write_bits(0, target); // one genuine sample
                // Change back to full width through mode A or B.
                if target <= 6 {
                    writer.write_bits(1 << (target - 1), target);
                    writer.write_bits((bits - 1) as u32, if bits == 8 { 3 } else { 4 });
                } else {
                    writer.write_bits(
                        (1 << (target - 1)) - bits as u32 / 2 + bits as u32 - 1,
                        target,
                    );
                }
                writer.write_bits(1, bits + 1);
                writer.write_bits(1, bits + 1);
                let payload = writer.finish();
                let mut stream = (payload.len() as u16).to_le_bytes().to_vec();
                stream.extend(payload);
                for (it215, expected) in [(false, [0, 1, 2]), (true, [0, 1, 3])] {
                    assert_eq!(
                        decompress_it(&stream, 3, bits, it215).unwrap().0,
                        expected,
                        "bits={bits} target={target} it215={it215}"
                    );
                }
            }
        }
    }

    #[test]
    fn multiple_blocks_reset_integrators_and_align_to_bytes() {
        let original8: Vec<i8> = (0..0x8003).map(|i| (i * 73) as i8).collect();
        let data = compress_it215_8bit(&original8);
        assert_eq!(
            decompress_it215_8bit(&data, original8.len()).unwrap(),
            original8
        );
        let original16: Vec<i16> = (0..0x4003).map(|i| (i * 997) as i16).collect();
        let data = compress_it215_16bit(&original16);
        assert_eq!(
            decompress_it215_16bit(&data, original16.len()).unwrap(),
            original16
        );
        assert!(decompress_it215_16bit(&[3, 0, 255, 255, 1], 1).is_err());
    }

    #[test]
    fn test_bit_reader_basic() {
        let data = [0b10101010, 0b11001100];
        let mut reader = BitReader::new(&data);

        assert_eq!(reader.read_bits(4).unwrap(), 0b1010);
        assert_eq!(reader.read_bits(4).unwrap(), 0b1010);
        assert_eq!(reader.read_bits(8).unwrap(), 0b11001100);
    }

    #[test]
    fn test_bit_writer_basic() {
        let mut writer = BitWriter::new();
        writer.write_bits(0b1010, 4);
        writer.write_bits(0b1010, 4);
        writer.write_bits(0b11001100, 8);

        let result = writer.finish();
        assert_eq!(result, vec![0b10101010, 0b11001100]);
    }

    #[test]
    fn test_roundtrip_8bit() {
        let original: Vec<i8> = vec![0, 10, -10, 50, -50, 127, -128, 0];
        let compressed = compress_it215_8bit(&original);
        let decompressed = decompress_it215_8bit(&compressed, original.len()).unwrap();

        // Note: Due to simplified compression, values should match
        assert_eq!(decompressed, original);
    }

    #[test]
    fn test_roundtrip_16bit() {
        let original: Vec<i16> = vec![0, 1000, -1000, 10000, -10000, 32767, -32768, 0];
        let compressed = compress_it215_16bit(&original);
        let decompressed = decompress_it215_16bit(&compressed, original.len()).unwrap();

        assert_eq!(decompressed, original);
    }
}
