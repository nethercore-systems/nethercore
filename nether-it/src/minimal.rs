//! Minimal IT packing - strip samples for ROM embedding

use std::io::{Cursor, Read, Seek, SeekFrom, Write};

use crate::error::ItError;
use crate::parser::parse_it;

/// Strip sample data from an IT file, keeping only patterns and metadata
///
/// This creates a minimal IT file that can be stored in the ROM with much smaller size.
/// Sample data is loaded separately via the ROM data pack.
///
/// The resulting IT file:
/// - Keeps all pattern data (note sequences)
/// - Keeps instrument names and envelopes (for ROM sound mapping)
/// - Keeps sample metadata (loop points, c5_speed) but NOT audio data
/// - Sets all sample lengths to 0 (no audio data embedded)
/// - Remains valid IT format that can be parsed
pub fn strip_it_samples(data: &[u8]) -> Result<Vec<u8>, ItError> {
    // Parse and validate first
    let module = parse_it(data)?;

    // Rebuild the IT file without sample data
    rebuild_it_without_samples(data, &module)
}

/// Pack an IT module into minimal format (patterns only, no sample data)
///
/// This is used after parsing to create a compact representation.
pub fn pack_it_minimal(module: &crate::ItModule) -> Vec<u8> {
    use crate::writer::ItWriter;

    // Use the writer but with empty sample data
    let mut writer = ItWriter::new(&module.name);
    writer.set_speed(module.initial_speed);
    writer.set_tempo(module.initial_tempo);
    writer.set_global_volume(module.global_volume);
    writer.set_mix_volume(module.mix_volume);
    writer.set_channels(module.num_channels);
    writer.set_flags(module.flags);

    // Add instruments
    for instr in &module.instruments {
        writer.add_instrument(instr.clone());
    }

    // Add samples with empty audio data
    for sample in &module.samples {
        let mut s = sample.clone();
        s.length = 0; // No audio data
        writer.add_sample(s, &[]);
    }

    // Add patterns
    for pattern in &module.patterns {
        let pat_idx = writer.add_pattern(pattern.num_rows);
        for (row, row_data) in pattern.notes.iter().enumerate() {
            for (channel, note) in row_data.iter().enumerate() {
                if note.note != 0 || note.instrument != 0 || note.volume != 0
                   || note.effect != 0 || note.effect_param != 0 {
                    writer.set_note(pat_idx, row as u16, channel as u8, *note);
                }
            }
        }
    }

    // Set order table
    writer.set_orders(&module.order_table);

    // Set message if present
    if let Some(ref msg) = module.message {
        writer.set_message(msg);
    }

    writer.write()
}

/// Rebuild an IT file without sample data
///
/// This modifies the original file in-place to zero out sample lengths
/// and remove sample data, preserving the exact structure otherwise.
fn rebuild_it_without_samples(
    original_data: &[u8],
    module: &crate::ItModule,
) -> Result<Vec<u8>, ItError> {
    let mut output = Vec::with_capacity(original_data.len() / 2);
    let mut cursor = Cursor::new(original_data);

    // Copy header (192 bytes)
    let header_size = 192;
    let mut header = vec![0u8; header_size];
    cursor.read_exact(&mut header)?;
    output.extend_from_slice(&header);

    // Copy order table
    let mut orders = vec![0u8; module.num_orders as usize];
    cursor.read_exact(&mut orders)?;
    output.extend_from_slice(&orders);

    // Read offset tables
    let num_instruments = module.num_instruments as usize;
    let num_samples = module.num_samples as usize;
    let num_patterns = module.num_patterns as usize;

    let mut instrument_offsets = Vec::with_capacity(num_instruments);
    for _ in 0..num_instruments {
        let mut buf = [0u8; 4];
        cursor.read_exact(&mut buf)?;
        instrument_offsets.push(u32::from_le_bytes(buf));
    }

    let mut sample_offsets = Vec::with_capacity(num_samples);
    for _ in 0..num_samples {
        let mut buf = [0u8; 4];
        cursor.read_exact(&mut buf)?;
        sample_offsets.push(u32::from_le_bytes(buf));
    }

    let mut pattern_offsets = Vec::with_capacity(num_patterns);
    for _ in 0..num_patterns {
        let mut buf = [0u8; 4];
        cursor.read_exact(&mut buf)?;
        pattern_offsets.push(u32::from_le_bytes(buf));
    }

    // Calculate new offsets
    // Instruments stay in same position relative to header
    let _offset_table_pos = output.len();

    // We'll write placeholder offsets and update them later
    let placeholder_offsets_start = output.len();
    for _ in 0..(num_instruments + num_samples + num_patterns) {
        output.extend_from_slice(&[0u8; 4]);
    }

    // Track where we're writing each section
    let mut new_instrument_offsets = Vec::with_capacity(num_instruments);
    let mut new_sample_offsets = Vec::with_capacity(num_samples);
    let mut new_pattern_offsets = Vec::with_capacity(num_patterns);

    // Copy instruments
    for &old_offset in &instrument_offsets {
        if old_offset == 0 {
            new_instrument_offsets.push(0);
            continue;
        }

        new_instrument_offsets.push(output.len() as u32);

        cursor.seek(SeekFrom::Start(old_offset as u64))?;
        // Instrument header is 547 bytes (modern format)
        let mut instr_data = vec![0u8; 547];
        cursor.read_exact(&mut instr_data)?;
        output.extend_from_slice(&instr_data);
    }

    // Copy sample headers (but zero out length and data pointer)
    for &old_offset in &sample_offsets {
        if old_offset == 0 {
            new_sample_offsets.push(0);
            continue;
        }

        new_sample_offsets.push(output.len() as u32);

        cursor.seek(SeekFrom::Start(old_offset as u64))?;
        // Sample header is 80 bytes
        let mut sample_data = vec![0u8; 80];
        cursor.read_exact(&mut sample_data)?;

        // Zero out sample length (bytes 48-51)
        sample_data[48] = 0;
        sample_data[49] = 0;
        sample_data[50] = 0;
        sample_data[51] = 0;

        // Zero out sample data pointer (bytes 72-75)
        sample_data[72] = 0;
        sample_data[73] = 0;
        sample_data[74] = 0;
        sample_data[75] = 0;

        // Clear the HAS_DATA flag (bit 0 of flags at byte 18)
        sample_data[18] &= !0x01;

        output.extend_from_slice(&sample_data);
    }

    // Copy patterns
    for &old_offset in &pattern_offsets {
        if old_offset == 0 {
            new_pattern_offsets.push(0);
            continue;
        }

        new_pattern_offsets.push(output.len() as u32);

        cursor.seek(SeekFrom::Start(old_offset as u64))?;

        // Read pattern header (8 bytes)
        let mut header = [0u8; 8];
        cursor.read_exact(&mut header)?;

        let packed_length = u16::from_le_bytes([header[0], header[1]]) as usize;

        output.extend_from_slice(&header);

        // Read and copy pattern data
        if packed_length > 0 {
            let mut pattern_data = vec![0u8; packed_length];
            cursor.read_exact(&mut pattern_data)?;
            output.extend_from_slice(&pattern_data);
        }
    }

    // Now go back and fix the offset tables
    let mut offset_cursor = Cursor::new(&mut output[placeholder_offsets_start..]);

    for &offset in &new_instrument_offsets {
        offset_cursor.write_all(&offset.to_le_bytes())?;
    }
    for &offset in &new_sample_offsets {
        offset_cursor.write_all(&offset.to_le_bytes())?;
    }
    for &offset in &new_pattern_offsets {
        offset_cursor.write_all(&offset.to_le_bytes())?;
    }

    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::IT_MAGIC;

    // Note: These tests require actual IT files to be meaningful
    // For now, we just test the pack function with a minimal module

    #[test]
    fn test_pack_minimal() {
        use crate::module::*;

        let mut module = ItModule::default();
        module.name = "Test".to_string();
        module.num_channels = 4;
        module.initial_speed = 6;
        module.initial_tempo = 125;

        // Add an empty pattern
        module.patterns.push(ItPattern::empty(64, 4));
        module.num_patterns = 1;

        // Add order table
        module.order_table = vec![0];
        module.num_orders = 1;

        let packed = pack_it_minimal(&module);

        // Verify magic
        assert_eq!(&packed[0..4], IT_MAGIC);

        // Should be parseable
        let parsed = parse_it(&packed);
        assert!(parsed.is_ok(), "Failed to parse packed IT: {:?}", parsed.err());
    }
}
