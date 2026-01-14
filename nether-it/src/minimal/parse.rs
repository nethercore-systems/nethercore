//! NCIT parsing functions (NCIT binary → ItModule)

use std::io::{Cursor, Read, Seek, SeekFrom};

use crate::error::ItError;
use crate::module::{
    DuplicateCheckAction, DuplicateCheckType, ItEnvelope, ItEnvelopeFlags, ItFlags, ItInstrument,
    ItModule, ItNote, ItPattern, ItSample, ItSampleFlags, NewNoteAction,
};

use super::{
    INSTR_HAS_DEFAULT_PAN, INSTR_HAS_FILTER, INSTR_HAS_PAN_ENV, INSTR_HAS_PITCH_ENV,
    INSTR_HAS_VOL_ENV, NCIT_HEADER_SIZE, SAMPLE_HAS_LOOP, SAMPLE_HAS_PAN, SAMPLE_HAS_SUSTAIN,
    SAMPLE_HAS_VIBRATO, SAMPLE_PINGPONG_LOOP, SAMPLE_PINGPONG_SUSTAIN, TABLE_FULL, TABLE_SPARSE,
    TABLE_UNIFORM,
};
use super::{read_u16, read_u32, read_u8};

/// Parse NCIT minimal format into an ItModule
///
/// This parses the compact NCIT format back into a full ItModule structure
/// that can be used for playback.
///
/// # Arguments
/// * `data` - Raw NCIT binary data
///
/// # Returns
/// * `Ok(ItModule)` - Parsed module
/// * `Err(ItError)` - Parse error
pub fn parse_ncit(data: &[u8]) -> Result<ItModule, ItError> {
    if data.len() < NCIT_HEADER_SIZE {
        return Err(ItError::TooSmall);
    }

    let mut cursor = Cursor::new(data);

    // ========== Read Header ==========
    let num_channels = read_u8(&mut cursor)?;
    let num_orders = read_u16(&mut cursor)?;
    let num_instruments = read_u16(&mut cursor)?;
    let num_samples = read_u16(&mut cursor)?;
    let num_patterns = read_u16(&mut cursor)?;
    let initial_speed = read_u8(&mut cursor)?;
    let initial_tempo = read_u8(&mut cursor)?;
    let global_volume = read_u8(&mut cursor)?;
    let mix_volume = read_u8(&mut cursor)?;
    let flags = ItFlags::from_bits(read_u16(&mut cursor)?);
    let panning_separation = read_u8(&mut cursor)?;
    cursor.seek(SeekFrom::Current(8))?; // Skip reserved

    // ========== Read Order Table ==========
    let mut order_table = vec![0u8; num_orders as usize];
    cursor.read_exact(&mut order_table)?;

    // ========== Read Channel Settings ==========
    let mut channel_pan = [128u8; 64]; // Disabled by default
    let mut channel_vol = [64u8; 64]; // Full volume by default
    cursor.read_exact(&mut channel_pan[..num_channels as usize])?;
    cursor.read_exact(&mut channel_vol[..num_channels as usize])?;

    // ========== Read Instruments ==========
    let mut instruments = Vec::with_capacity(num_instruments as usize);
    for _ in 0..num_instruments {
        instruments.push(parse_instrument(&mut cursor)?);
    }

    // ========== Read Samples ==========
    let mut samples = Vec::with_capacity(num_samples as usize);
    for _ in 0..num_samples {
        samples.push(parse_sample(&mut cursor)?);
    }

    // ========== Read Patterns ==========
    let mut patterns = Vec::with_capacity(num_patterns as usize);
    for _ in 0..num_patterns {
        patterns.push(parse_pattern(&mut cursor, num_channels)?);
    }

    Ok(ItModule {
        name: String::new(),
        num_channels,
        num_orders,
        num_instruments,
        num_samples,
        num_patterns,
        created_with: 0x0214,
        compatible_with: 0x0200,
        flags,
        special: 0,
        global_volume,
        mix_volume,
        initial_speed,
        initial_tempo,
        panning_separation,
        pitch_wheel_depth: 0,
        channel_pan,
        channel_vol,
        order_table,
        patterns,
        instruments,
        samples,
        message: None,
    })
}

/// Parse an instrument from NCIT format
pub(super) fn parse_instrument(cursor: &mut Cursor<&[u8]>) -> Result<ItInstrument, ItError> {
    let flags = read_u8(cursor)?;
    let nna_dct_dca = read_u8(cursor)?;

    let nna = NewNoteAction::from_u8(nna_dct_dca & 0x03);
    let dct = DuplicateCheckType::from_u8((nna_dct_dca >> 2) & 0x03);
    let dca = DuplicateCheckAction::from_u8((nna_dct_dca >> 4) & 0x03);

    let fadeout = read_u16(cursor)?;
    let global_volume = read_u8(cursor)?;
    let pitch_pan_separation = read_u8(cursor)? as i8;
    let pitch_pan_center = read_u8(cursor)?;

    // Random variation (for accurate playback)
    let random_volume = read_u8(cursor)?;
    let random_pan = read_u8(cursor)?;

    let default_pan = if flags & INSTR_HAS_DEFAULT_PAN != 0 {
        Some(read_u8(cursor)?)
    } else {
        None
    };

    let (filter_cutoff, filter_resonance) = if flags & INSTR_HAS_FILTER != 0 {
        (Some(read_u8(cursor)?), Some(read_u8(cursor)?))
    } else {
        (None, None)
    };

    let note_sample_table = parse_note_sample_table(cursor)?;

    let volume_envelope = if flags & INSTR_HAS_VOL_ENV != 0 {
        Some(parse_envelope(cursor)?)
    } else {
        None
    };

    let panning_envelope = if flags & INSTR_HAS_PAN_ENV != 0 {
        Some(parse_envelope(cursor)?)
    } else {
        None
    };

    let pitch_envelope = if flags & INSTR_HAS_PITCH_ENV != 0 {
        Some(parse_envelope(cursor)?)
    } else {
        None
    };

    Ok(ItInstrument {
        name: String::new(),
        filename: String::new(),
        nna,
        dct,
        dca,
        fadeout,
        pitch_pan_separation,
        pitch_pan_center,
        global_volume,
        default_pan,
        random_volume,
        random_pan,
        note_sample_table,
        volume_envelope,
        panning_envelope,
        pitch_envelope,
        filter_cutoff,
        filter_resonance,
        midi_channel: 0,
        midi_program: 0,
        midi_bank: 0,
    })
}

/// Parse a compressed note-sample table
pub(super) fn parse_note_sample_table(
    cursor: &mut Cursor<&[u8]>,
) -> Result<[(u8, u8); 120], ItError> {
    let table_type = read_u8(cursor)?;

    let mut table = [(0u8, 0u8); 120];

    match table_type {
        TABLE_UNIFORM => {
            let sample = read_u8(cursor)?;
            for (i, entry) in table.iter_mut().enumerate() {
                entry.0 = i as u8; // Identity note mapping
                entry.1 = sample;
            }
        }
        TABLE_SPARSE => {
            let default_offset = read_u8(cursor)? as i8;
            let default_sample = read_u8(cursor)?;
            let num_exceptions = read_u8(cursor)?;

            // Fill with defaults
            for (i, entry) in table.iter_mut().enumerate() {
                entry.0 = (i as i8 + default_offset) as u8;
                entry.1 = default_sample;
            }

            // Apply exceptions
            for _ in 0..num_exceptions {
                let index = read_u8(cursor)? as usize;
                let note = read_u8(cursor)?;
                let sample = read_u8(cursor)?;
                if index < 120 {
                    table[index] = (note, sample);
                }
            }
        }
        TABLE_FULL => {
            for entry in &mut table {
                entry.0 = read_u8(cursor)?;
                entry.1 = read_u8(cursor)?;
            }
        }
        _ => return Err(ItError::InvalidInstrument(0)),
    }

    Ok(table)
}

/// Parse an envelope from NCIT format
pub(super) fn parse_envelope(cursor: &mut Cursor<&[u8]>) -> Result<ItEnvelope, ItError> {
    let num_points = read_u8(cursor)?;
    let loop_begin = read_u8(cursor)?;
    let loop_end = read_u8(cursor)?;
    let sustain_begin = read_u8(cursor)?;
    let sustain_end = read_u8(cursor)?;
    let flags = ItEnvelopeFlags::from_bits(read_u8(cursor)?);

    let mut points = Vec::with_capacity(num_points as usize);
    for _ in 0..num_points {
        let tick = read_u16(cursor)?;
        let value = read_u8(cursor)? as i8;
        points.push((tick, value));
    }

    Ok(ItEnvelope {
        points,
        loop_begin,
        loop_end,
        sustain_begin,
        sustain_end,
        flags,
    })
}

/// Parse a sample from NCIT format
pub(super) fn parse_sample(cursor: &mut Cursor<&[u8]>) -> Result<ItSample, ItError> {
    let flags = read_u8(cursor)?;
    let global_volume = read_u8(cursor)?;
    let default_volume = read_u8(cursor)?;
    let c5_speed = read_u32(cursor)?;

    let (loop_begin, loop_end) = if flags & SAMPLE_HAS_LOOP != 0 {
        (read_u32(cursor)?, read_u32(cursor)?)
    } else {
        (0, 0)
    };

    let (sustain_loop_begin, sustain_loop_end) = if flags & SAMPLE_HAS_SUSTAIN != 0 {
        (read_u32(cursor)?, read_u32(cursor)?)
    } else {
        (0, 0)
    };

    let default_pan = if flags & SAMPLE_HAS_PAN != 0 {
        Some(read_u8(cursor)?)
    } else {
        None
    };

    let (vibrato_speed, vibrato_depth, vibrato_rate, vibrato_type) =
        if flags & SAMPLE_HAS_VIBRATO != 0 {
            (
                read_u8(cursor)?,
                read_u8(cursor)?,
                read_u8(cursor)?,
                read_u8(cursor)?,
            )
        } else {
            (0, 0, 0, 0)
        };

    // Reconstruct IT sample flags from NCIT flags
    let mut sample_flags = ItSampleFlags::empty();
    if flags & SAMPLE_HAS_LOOP != 0 {
        sample_flags = sample_flags | ItSampleFlags::LOOP;
    }
    if flags & SAMPLE_PINGPONG_LOOP != 0 {
        sample_flags = sample_flags | ItSampleFlags::PINGPONG_LOOP;
    }
    if flags & SAMPLE_HAS_SUSTAIN != 0 {
        sample_flags = sample_flags | ItSampleFlags::SUSTAIN_LOOP;
    }
    if flags & SAMPLE_PINGPONG_SUSTAIN != 0 {
        sample_flags = sample_flags | ItSampleFlags::PINGPONG_SUSTAIN;
    }

    Ok(ItSample {
        name: String::new(),
        filename: String::new(),
        global_volume,
        flags: sample_flags,
        default_volume,
        default_pan,
        length: 0, // Not stored in NCIT (samples come from ROM)
        loop_begin,
        loop_end,
        c5_speed,
        sustain_loop_begin,
        sustain_loop_end,
        vibrato_speed,
        vibrato_depth,
        vibrato_rate,
        vibrato_type,
    })
}

/// Parse a pattern from NCIT format
pub(super) fn parse_pattern(cursor: &mut Cursor<&[u8]>, num_channels: u8) -> Result<ItPattern, ItError> {
    let num_rows = read_u16(cursor)?;
    let packed_size = read_u16(cursor)?;

    let mut packed_data = vec![0u8; packed_size as usize];
    cursor.read_exact(&mut packed_data)?;

    let notes = unpack_pattern_data(&packed_data, num_rows, num_channels)?;

    Ok(ItPattern { num_rows, notes })
}

/// Unpack pattern data from IT packed format
pub(super) fn unpack_pattern_data(
    data: &[u8],
    num_rows: u16,
    num_channels: u8,
) -> Result<Vec<Vec<ItNote>>, ItError> {
    let mut cursor = Cursor::new(data);
    let mut notes = Vec::with_capacity(num_rows as usize);

    // Previous values for compression
    let mut prev_note = [0u8; 64];
    let mut prev_instrument = [0u8; 64];
    let mut prev_volume = [0u8; 64];
    let mut prev_effect = [0u8; 64];
    let mut prev_effect_param = [0u8; 64];

    for _ in 0..num_rows {
        let mut row = vec![ItNote::default(); num_channels as usize];

        while let Ok(channel_var) = read_u8(&mut cursor) {
            if channel_var == 0 {
                // End of row
                break;
            }

            let channel = (channel_var & 0x7F) as usize;
            if channel >= num_channels as usize {
                continue; // Skip invalid channels
            }

            // Read mask
            let mask = if channel_var & 0x80 != 0 {
                read_u8(&mut cursor)?
            } else {
                0
            };

            // Read data based on mask
            if mask & 0x01 != 0 {
                prev_note[channel] = read_u8(&mut cursor)?;
            }
            if mask & 0x02 != 0 {
                prev_instrument[channel] = read_u8(&mut cursor)?;
            }
            if mask & 0x04 != 0 {
                prev_volume[channel] = read_u8(&mut cursor)?;
            }
            if mask & 0x08 != 0 {
                prev_effect[channel] = read_u8(&mut cursor)?;
                prev_effect_param[channel] = read_u8(&mut cursor)?;
            }

            // Apply values based on mask
            if mask & 0x11 != 0 {
                row[channel].note = prev_note[channel];
            }
            if mask & 0x22 != 0 {
                row[channel].instrument = prev_instrument[channel];
            }
            if mask & 0x44 != 0 {
                row[channel].volume = prev_volume[channel];
            }
            if mask & 0x88 != 0 {
                row[channel].effect = prev_effect[channel];
                row[channel].effect_param = prev_effect_param[channel];
            }
        }

        notes.push(row);
    }

    Ok(notes)
}
