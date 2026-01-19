//! IT file serialization orchestration

use super::{ItWriter, pack_pattern, write_instrument, write_sample_header, write_string};
use crate::IT_MAGIC;

/// Serialize a complete IT module to bytes
pub fn serialize_module(writer: &ItWriter) -> Vec<u8> {
    let mut output = Vec::new();

    // Calculate offsets
    let header_size = 192;
    let orders_size = writer.module.num_orders as usize;

    // Offset table starts after header + orders
    let offset_table_start = header_size + orders_size;
    let num_instruments = writer.module.num_instruments as usize;
    let num_samples = writer.module.num_samples as usize;
    let num_patterns = writer.module.num_patterns as usize;

    let offset_table_size = (num_instruments + num_samples + num_patterns) * 4;

    // Message (if any)
    let message_offset = if writer.module.message.is_some() {
        offset_table_start + offset_table_size
    } else {
        0
    };
    let message_size = writer
        .module
        .message
        .as_ref()
        .map(|m| m.len() + 1)
        .unwrap_or(0);

    // Instruments start after message
    let instruments_start = offset_table_start + offset_table_size + message_size;

    // Pre-calculate instrument offsets
    // Instrument size: 4 (magic) + 12 (filename) + 1 (reserved) + 3 (NNA/DCT/DCA) +
    //                  2 (fadeout) + 2 (PPS/PPC) + 2 (GbV/DfP) + 2 (RV/RP) +
    //                  4 (TrkVers) + 26 (name) + 2 (IFC/IFR) + 4 (MIDI) +
    //                  240 (note-sample table) + 3*82 (envelopes) = 550 bytes
    let instrument_size = 550;
    let mut instrument_offsets = Vec::new();
    for i in 0..num_instruments {
        instrument_offsets.push((instruments_start + i * instrument_size) as u32);
    }

    // Samples start after instruments
    let samples_start = instruments_start + num_instruments * instrument_size;
    let sample_header_size = 80;
    let mut sample_offsets = Vec::new();
    for i in 0..num_samples {
        sample_offsets.push((samples_start + i * sample_header_size) as u32);
    }

    // Patterns start after sample headers
    let patterns_start = samples_start + num_samples * sample_header_size;

    // Pack patterns and calculate their sizes/offsets
    let mut packed_patterns = Vec::new();
    let mut pattern_offsets = Vec::new();
    let mut current_pattern_offset = patterns_start;

    for pattern in &writer.module.patterns {
        let packed = pack_pattern(pattern, writer.module.num_channels);
        pattern_offsets.push(current_pattern_offset as u32);
        current_pattern_offset += 8 + packed.len(); // 8 byte header + data
        packed_patterns.push(packed);
    }

    // Sample data starts after patterns
    let sample_data_start = current_pattern_offset;
    let mut sample_data_offsets = Vec::new();
    let mut current_data_offset = sample_data_start;

    for data in &writer.sample_data {
        sample_data_offsets.push(current_data_offset as u32);
        current_data_offset += data.len() * 2; // 16-bit samples
    }

    // ========== Write Header ==========

    write_header(&mut output, writer, message_offset);

    // ========== Write Order Table ==========
    for &order in &writer.module.order_table {
        output.push(order);
    }

    // ========== Write Offset Tables ==========
    for &offset in &instrument_offsets {
        output.extend_from_slice(&offset.to_le_bytes());
    }
    for &offset in &sample_offsets {
        output.extend_from_slice(&offset.to_le_bytes());
    }
    for &offset in &pattern_offsets {
        output.extend_from_slice(&offset.to_le_bytes());
    }

    // ========== Write Message ==========
    if let Some(ref msg) = writer.module.message {
        output.extend_from_slice(msg.as_bytes());
        output.push(0); // Null terminator
    }

    // ========== Write Instruments ==========
    for instrument in &writer.module.instruments {
        write_instrument(&mut output, instrument);
    }

    // ========== Write Sample Headers ==========
    for (i, sample) in writer.module.samples.iter().enumerate() {
        let data_offset = sample_data_offsets.get(i).copied().unwrap_or(0);
        write_sample_header(&mut output, sample, data_offset);
    }

    // ========== Write Patterns ==========
    for (i, packed) in packed_patterns.iter().enumerate() {
        let pattern = &writer.module.patterns[i];
        // Pattern header (8 bytes)
        output.extend_from_slice(&(packed.len() as u16).to_le_bytes()); // Length
        output.extend_from_slice(&pattern.num_rows.to_le_bytes()); // Rows
        output.extend_from_slice(&[0u8; 4]); // Reserved
        // Pattern data
        output.extend_from_slice(packed);
    }

    // ========== Write Sample Data ==========
    for data in &writer.sample_data {
        for &sample in data {
            output.extend_from_slice(&sample.to_le_bytes());
        }
    }

    output
}

/// Write the IT file header
fn write_header(output: &mut Vec<u8>, writer: &ItWriter, message_offset: usize) {
    // Magic "IMPM"
    output.extend_from_slice(IT_MAGIC);

    // Song name (26 bytes)
    write_string(output, &writer.module.name, 26);

    // PHilight (2 bytes) - row highlight info
    output.extend_from_slice(&[0x04, 0x10]); // Default: 4/16 highlight

    // OrdNum (2 bytes)
    output.extend_from_slice(&(writer.module.num_orders).to_le_bytes());

    // InsNum (2 bytes)
    output.extend_from_slice(&(writer.module.num_instruments).to_le_bytes());

    // SmpNum (2 bytes)
    output.extend_from_slice(&(writer.module.num_samples).to_le_bytes());

    // PatNum (2 bytes)
    output.extend_from_slice(&(writer.module.num_patterns).to_le_bytes());

    // Cwt/v (2 bytes) - created with version
    output.extend_from_slice(&writer.module.created_with.to_le_bytes());

    // Cmwt (2 bytes) - compatible with version
    output.extend_from_slice(&writer.module.compatible_with.to_le_bytes());

    // Flags (2 bytes)
    output.extend_from_slice(&writer.module.flags.bits().to_le_bytes());

    // Special (2 bytes)
    output.extend_from_slice(&writer.module.special.to_le_bytes());

    // GV (1 byte)
    output.push(writer.module.global_volume);

    // MV (1 byte)
    output.push(writer.module.mix_volume);

    // IS (1 byte)
    output.push(writer.module.initial_speed);

    // IT (1 byte)
    output.push(writer.module.initial_tempo);

    // Sep (1 byte)
    output.push(writer.module.panning_separation);

    // PWD (1 byte)
    output.push(writer.module.pitch_wheel_depth);

    // MsgLgth (2 bytes)
    let msg_len = writer.module.message.as_ref().map(|m| m.len()).unwrap_or(0) as u16;
    output.extend_from_slice(&msg_len.to_le_bytes());

    // MsgOff (4 bytes)
    output.extend_from_slice(&(message_offset as u32).to_le_bytes());

    // Reserved (4 bytes)
    output.extend_from_slice(&[0u8; 4]);

    // Channel pan (64 bytes)
    output.extend_from_slice(&writer.module.channel_pan);

    // Channel vol (64 bytes)
    output.extend_from_slice(&writer.module.channel_vol);
}
