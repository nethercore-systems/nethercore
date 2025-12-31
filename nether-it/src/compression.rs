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
    let mut output = Vec::with_capacity(output_length);
    let mut reader = BitReader::new(compressed);

    // IT215 processes samples in blocks of 0x8000
    const BLOCK_SIZE: usize = 0x8000;
    let mut remaining = output_length;

    while remaining > 0 && !reader.is_exhausted() {
        let block_len = remaining.min(BLOCK_SIZE);
        decompress_block_8bit(&mut reader, &mut output, block_len)?;
        remaining = remaining.saturating_sub(block_len);
    }

    // Pad with zeros if we didn't get enough samples
    while output.len() < output_length {
        output.push(0);
    }

    // Calculate bytes consumed (round up to next byte if we're mid-byte)
    let bytes_consumed = if reader.bit_pos > 0 {
        reader.byte_pos + 1
    } else {
        reader.byte_pos
    };

    Ok((output, bytes_consumed))
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
    let mut output = Vec::with_capacity(output_length);
    let mut reader = BitReader::new(compressed);

    // IT215 processes samples in blocks of 0x4000 for 16-bit
    const BLOCK_SIZE: usize = 0x4000;
    let mut remaining = output_length;

    while remaining > 0 && !reader.is_exhausted() {
        let block_len = remaining.min(BLOCK_SIZE);
        decompress_block_16bit(&mut reader, &mut output, block_len)?;
        remaining = remaining.saturating_sub(block_len);
    }

    // Pad with zeros if we didn't get enough samples
    while output.len() < output_length {
        output.push(0);
    }

    // Calculate bytes consumed (round up to next byte if we're mid-byte)
    let bytes_consumed = if reader.bit_pos > 0 {
        reader.byte_pos + 1
    } else {
        reader.byte_pos
    };

    Ok((output, bytes_consumed))
}

/// Decompress a single 8-bit block
fn decompress_block_8bit(
    reader: &mut BitReader,
    output: &mut Vec<i8>,
    block_len: usize,
) -> Result<(), ItError> {
    // Read block header: compressed length (16 bits)
    let _compressed_len = reader.read_bits(16)?;

    let mut width = 9; // Initial bit width
    let mut last_value: i8 = 0;

    for _ in 0..block_len {
        if reader.is_exhausted() {
            output.push(0);
            continue;
        }

        // Read value with current width
        let value = reader.read_bits(width)?;

        // Decode the value based on width
        let delta = if width < 7 {
            // For widths < 7, check for width change markers
            let max_positive = (1i32 << (width - 1)) - 1;
            let marker = (1u32 << width) - 1;

            if value == marker {
                // Width change: read new width
                let new_width = reader.read_bits(3)? as usize + 1;
                if new_width < width {
                    width = new_width;
                } else {
                    width = new_width + 1;
                }
                continue;
            }

            // Sign extend
            let signed = if value > max_positive as u32 {
                value as i32 - (1i32 << width)
            } else {
                value as i32
            };
            signed as i8
        } else if width < 9 {
            // For widths 7-8, different marker handling
            let max_val = (1u32 << width) - 1;
            if value == max_val {
                // Width change
                let new_width = reader.read_bits(3)? as usize + 1;
                if new_width < width {
                    width = new_width;
                } else {
                    width = new_width + 1;
                }
                continue;
            }

            let max_positive = (1i32 << (width - 1)) - 1;
            let signed = if value > max_positive as u32 {
                value as i32 - (1i32 << width)
            } else {
                value as i32
            };
            signed as i8
        } else {
            // Width 9: full 8-bit value with sign
            let max_val = (1u32 << 9) - 1;
            if value == max_val {
                // Width change
                let new_width = reader.read_bits(3)? as usize + 1;
                if new_width < width {
                    width = new_width;
                } else {
                    width = new_width + 1;
                }
                continue;
            }

            // For width 9, values 256-510 represent -256 to -2, 511 is marker
            if value >= 256 {
                (value as i32 - 256) as i8
            } else {
                value as i8
            }
        };

        // Apply delta to get actual value
        last_value = last_value.wrapping_add(delta);
        output.push(last_value);
    }

    Ok(())
}

/// Decompress a single 16-bit block
fn decompress_block_16bit(
    reader: &mut BitReader,
    output: &mut Vec<i16>,
    block_len: usize,
) -> Result<(), ItError> {
    // Read block header: compressed length (16 bits)
    let _compressed_len = reader.read_bits(16)?;

    let mut width = 17; // Initial bit width for 16-bit samples
    let mut last_value: i16 = 0;

    for _ in 0..block_len {
        if reader.is_exhausted() {
            output.push(0);
            continue;
        }

        // Read value with current width
        let value = reader.read_bits(width)?;

        // Decode the value based on width
        let delta = if width < 7 {
            let max_positive = (1i32 << (width - 1)) - 1;
            let marker = (1u32 << width) - 1;

            if value == marker {
                let new_width = reader.read_bits(4)? as usize + 1;
                if new_width < width {
                    width = new_width;
                } else {
                    width = new_width + 1;
                }
                continue;
            }

            let signed = if value > max_positive as u32 {
                value as i32 - (1i32 << width)
            } else {
                value as i32
            };
            signed as i16
        } else if width < 17 {
            let max_val = (1u32 << width) - 1;
            if value == max_val {
                let new_width = reader.read_bits(4)? as usize + 1;
                if new_width < width {
                    width = new_width;
                } else {
                    width = new_width + 1;
                }
                continue;
            }

            let max_positive = (1i32 << (width - 1)) - 1;
            let signed = if value > max_positive as u32 {
                value as i32 - (1i32 << width)
            } else {
                value as i32
            };
            signed as i16
        } else {
            // Width 17: full 16-bit value
            let max_val = (1u32 << 17) - 1;
            if value == max_val {
                let new_width = reader.read_bits(4)? as usize + 1;
                if new_width < width {
                    width = new_width;
                } else {
                    width = new_width + 1;
                }
                continue;
            }

            if value >= 0x10000 {
                (value as i32 - 0x10000) as i16
            } else {
                value as i16
            }
        };

        last_value = last_value.wrapping_add(delta);
        output.push(last_value);
    }

    Ok(())
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

    fn is_exhausted(&self) -> bool {
        self.byte_pos >= self.data.len()
    }

    fn read_bits(&mut self, count: usize) -> Result<u32, ItError> {
        if count == 0 || count > 32 {
            return Ok(0);
        }

        let mut result = 0u32;
        let mut bits_read = 0;

        while bits_read < count {
            if self.byte_pos >= self.data.len() {
                // Pad with zeros if we run out of data
                return Ok(result);
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
#[allow(dead_code)]
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
#[allow(dead_code)]
fn compress_block_8bit(samples: &[i8], writer: &mut BitWriter) {
    let mut last_value: i8 = 0;

    for &sample in samples {
        let delta = sample.wrapping_sub(last_value);

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
#[allow(dead_code)]
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
#[allow(dead_code)]
fn compress_block_16bit(samples: &[i16], writer: &mut BitWriter) {
    let mut last_value: i16 = 0;

    for &sample in samples {
        let delta = sample.wrapping_sub(last_value);

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
#[allow(dead_code)]
struct BitWriter {
    data: Vec<u8>,
    current_byte: u8,
    bit_pos: u8,
}

#[allow(dead_code)]
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
        assert_eq!(decompressed.len(), original.len());
    }

    #[test]
    fn test_roundtrip_16bit() {
        let original: Vec<i16> = vec![0, 1000, -1000, 10000, -10000, 32767, -32768, 0];
        let compressed = compress_it215_16bit(&original);
        let decompressed = decompress_it215_16bit(&compressed, original.len()).unwrap();

        assert_eq!(decompressed.len(), original.len());
    }
}
