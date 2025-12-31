//! NCIT: Nethercore IT minimal format for ROM packing
//!
//! This module implements a highly optimized binary format that strips all
//! unnecessary IT overhead while preserving playback data. This is designed
//! exclusively for ROM storage where samples come from a separate data pack.
//!
//! # Format Overview (NCIT - Nethercore IT)
//!
//! ## Header (24 bytes)
//! ```text
//! 0x00  u8      num_channels (1-64)
//! 0x01  u16 LE  num_orders
//! 0x03  u16 LE  num_instruments
//! 0x05  u16 LE  num_samples
//! 0x07  u16 LE  num_patterns
//! 0x09  u8      initial_speed
//! 0x0A  u8      initial_tempo
//! 0x0B  u8      global_volume (0-128)
//! 0x0C  u8      mix_volume (0-128)
//! 0x0D  u16 LE  flags (IT flags)
//! 0x0F  u8      panning_separation (0-128)
//! 0x10  8 bytes reserved
//! ```
//!
//! ## Savings vs Standard IT
//!
//! - Removes 192-byte IT header (replaced with 24-byte NCIT header)
//! - Removes 547-byte instrument headers (replaced with ~15-50 bytes each)
//! - Removes 80-byte sample headers (replaced with ~7-27 bytes each)
//! - Note-sample tables compressed (240 bytes → 2-50 bytes typically)
//! - Pattern data unchanged (IT's packing is already efficient)
//!
//! **Total savings: ~75-80% reduction in metadata overhead**

use std::io::{Cursor, Read, Seek, SeekFrom, Write};

use crate::IT_MAGIC;
use crate::error::ItError;
use crate::module::{
    DuplicateCheckAction, DuplicateCheckType, ItEnvelope, ItEnvelopeFlags, ItFlags, ItInstrument,
    ItModule, ItNote, ItPattern, ItSample, ItSampleFlags, NewNoteAction,
};
use crate::parser::parse_it;

/// NCIT header size in bytes
const NCIT_HEADER_SIZE: usize = 24;

/// Maximum envelope points we support
const MAX_ENVELOPE_POINTS: usize = 25;

// =============================================================================
// Instrument Flags (for NCIT format)
// =============================================================================

const INSTR_HAS_VOL_ENV: u8 = 0x01;
const INSTR_HAS_PAN_ENV: u8 = 0x02;
const INSTR_HAS_PITCH_ENV: u8 = 0x04;
const INSTR_HAS_FILTER: u8 = 0x08;
const INSTR_HAS_DEFAULT_PAN: u8 = 0x10;

// =============================================================================
// Sample Flags (for NCIT format)
// =============================================================================

const SAMPLE_HAS_LOOP: u8 = 0x01;
const SAMPLE_PINGPONG_LOOP: u8 = 0x02;
const SAMPLE_HAS_SUSTAIN: u8 = 0x04;
const SAMPLE_PINGPONG_SUSTAIN: u8 = 0x08;
const SAMPLE_HAS_PAN: u8 = 0x10;
const SAMPLE_HAS_VIBRATO: u8 = 0x20;

// =============================================================================
// Note-Sample Table Types
// =============================================================================

const TABLE_UNIFORM: u8 = 0;
const TABLE_SPARSE: u8 = 1;
const TABLE_FULL: u8 = 2;

// =============================================================================
// Pack Functions (ItModule → NCIT binary)
// =============================================================================

/// Pack an ItModule into NCIT minimal binary format
///
/// This creates a highly optimized binary representation that strips all
/// unnecessary IT overhead. The result is typically 75-80% smaller than
/// the standard IT format (metadata only, pattern data unchanged).
///
/// # Arguments
/// * `module` - Parsed IT module to pack
///
/// # Returns
/// * Packed binary data
///
/// # Example
/// ```ignore
/// let it_data = std::fs::read("song.it")?;
/// let module = parse_it(&it_data)?;
/// let ncit = pack_ncit(&module);
/// println!("Reduced from {} to {} bytes", it_data.len(), ncit.len());
/// ```
pub fn pack_ncit(module: &ItModule) -> Vec<u8> {
    let mut output = Vec::with_capacity(4096);

    // ========== Write Header (24 bytes) ==========
    output.push(module.num_channels);
    write_u16(&mut output, module.num_orders);
    write_u16(&mut output, module.num_instruments);
    write_u16(&mut output, module.num_samples);
    write_u16(&mut output, module.num_patterns);
    output.push(module.initial_speed);
    output.push(module.initial_tempo);
    output.push(module.global_volume);
    output.push(module.mix_volume);
    write_u16(&mut output, module.flags.bits());
    output.push(module.panning_separation);
    output.extend_from_slice(&[0u8; 8]); // Reserved

    // ========== Write Order Table ==========
    output.extend_from_slice(&module.order_table[..module.num_orders as usize]);

    // ========== Write Channel Settings (for accurate playback) ==========
    // Only write num_channels worth (not full 64)
    output.extend_from_slice(&module.channel_pan[..module.num_channels as usize]);
    output.extend_from_slice(&module.channel_vol[..module.num_channels as usize]);

    // ========== Write Instruments ==========
    for instrument in &module.instruments {
        pack_instrument(&mut output, instrument);
    }

    // ========== Write Samples ==========
    for sample in &module.samples {
        pack_sample(&mut output, sample);
    }

    // ========== Write Patterns ==========
    for pattern in &module.patterns {
        pack_pattern(&mut output, pattern, module.num_channels);
    }

    output
}

/// Pack an instrument to NCIT format
fn pack_instrument(output: &mut Vec<u8>, instr: &ItInstrument) {
    // Build flags byte
    let mut flags = 0u8;
    if instr.volume_envelope.is_some() {
        flags |= INSTR_HAS_VOL_ENV;
    }
    if instr.panning_envelope.is_some() {
        flags |= INSTR_HAS_PAN_ENV;
    }
    if instr.pitch_envelope.is_some() {
        flags |= INSTR_HAS_PITCH_ENV;
    }
    if instr.filter_cutoff.is_some() || instr.filter_resonance.is_some() {
        flags |= INSTR_HAS_FILTER;
    }
    if instr.default_pan.is_some() {
        flags |= INSTR_HAS_DEFAULT_PAN;
    }
    output.push(flags);

    // Pack NNA/DCT/DCA into single byte
    let nna_dct_dca = (instr.nna as u8 & 0x03)
        | ((instr.dct as u8 & 0x03) << 2)
        | ((instr.dca as u8 & 0x03) << 4);
    output.push(nna_dct_dca);

    // Core metadata
    write_u16(output, instr.fadeout);
    output.push(instr.global_volume);
    output.push(instr.pitch_pan_separation as u8);
    output.push(instr.pitch_pan_center);

    // Random variation (for accurate playback)
    output.push(instr.random_volume);
    output.push(instr.random_pan);

    // Optional default panning
    if instr.default_pan.is_some() {
        output.push(instr.default_pan.unwrap_or(32));
    }

    // Optional filter settings
    if flags & INSTR_HAS_FILTER != 0 {
        output.push(instr.filter_cutoff.unwrap_or(127));
        output.push(instr.filter_resonance.unwrap_or(0));
    }

    // Note-sample table (compressed)
    pack_note_sample_table(output, &instr.note_sample_table);

    // Envelopes (conditional)
    if let Some(ref env) = instr.volume_envelope {
        pack_envelope(output, env);
    }
    if let Some(ref env) = instr.panning_envelope {
        pack_envelope(output, env);
    }
    if let Some(ref env) = instr.pitch_envelope {
        pack_envelope(output, env);
    }
}

/// Analyze and pack a note-sample table with compression
fn pack_note_sample_table(output: &mut Vec<u8>, table: &[(u8, u8); 120]) {
    // Check if uniform (all entries map to same sample with identity note mapping)
    let first_sample = table[0].1;
    let is_uniform = table
        .iter()
        .enumerate()
        .all(|(i, &(note, sample))| note == i as u8 && sample == first_sample);

    if is_uniform {
        // Type 0: Uniform - all notes use same sample
        output.push(TABLE_UNIFORM);
        output.push(first_sample);
        return;
    }

    // Check if sparse (most entries are uniform with few exceptions)
    // Find the most common (note_offset, sample) pair
    let mut counts: std::collections::HashMap<(i8, u8), u32> = std::collections::HashMap::new();
    for (i, (note, sample)) in table.iter().enumerate() {
        let offset = *note as i8 - i as i8;
        *counts.entry((offset, *sample)).or_insert(0) += 1;
    }

    let (most_common, count) = counts.iter().max_by_key(|(_, c)| *c).unwrap();
    let exceptions: Vec<(u8, u8, u8)> = table
        .iter()
        .enumerate()
        .filter(|(i, (note, sample))| {
            let offset = *note as i8 - *i as i8;
            (offset, *sample) != *most_common
        })
        .map(|(i, (note, sample))| (i as u8, *note, *sample))
        .collect();

    // Use sparse if:
    // 1. Most entries are uniform (>75% same pattern)
    // 2. Sparse encoding would be smaller than full
    let sparse_size = 3 + exceptions.len() * 3; // type + default_offset + default_sample + exception_count + exceptions
    let full_size = 1 + 240; // type + raw table

    if *count > 90 && sparse_size < full_size {
        // Type 1: Sparse encoding
        output.push(TABLE_SPARSE);
        output.push(most_common.0 as u8); // default note offset
        output.push(most_common.1); // default sample
        output.push(exceptions.len() as u8);

        for (index, note, sample) in exceptions {
            output.push(index);
            output.push(note);
            output.push(sample);
        }
    } else {
        // Type 2: Full table
        output.push(TABLE_FULL);
        for &(note, sample) in table {
            output.push(note);
            output.push(sample);
        }
    }
}

/// Pack an envelope to NCIT format
fn pack_envelope(output: &mut Vec<u8>, env: &ItEnvelope) {
    let num_points = env.points.len().min(MAX_ENVELOPE_POINTS) as u8;
    output.push(num_points);
    output.push(env.loop_begin);
    output.push(env.loop_end);
    output.push(env.sustain_begin);
    output.push(env.sustain_end);
    output.push(env.flags.bits());

    // Write envelope points
    for i in 0..num_points as usize {
        let (tick, value) = env.points[i];
        write_u16(output, tick);
        output.push(value as u8);
    }
}

/// Pack a sample to NCIT format
fn pack_sample(output: &mut Vec<u8>, sample: &ItSample) {
    // Build flags byte
    let mut flags = 0u8;
    if sample.flags.contains(ItSampleFlags::LOOP) {
        flags |= SAMPLE_HAS_LOOP;
    }
    if sample.flags.contains(ItSampleFlags::PINGPONG_LOOP) {
        flags |= SAMPLE_PINGPONG_LOOP;
    }
    if sample.flags.contains(ItSampleFlags::SUSTAIN_LOOP) {
        flags |= SAMPLE_HAS_SUSTAIN;
    }
    if sample.flags.contains(ItSampleFlags::PINGPONG_SUSTAIN) {
        flags |= SAMPLE_PINGPONG_SUSTAIN;
    }
    if sample.default_pan.is_some() {
        flags |= SAMPLE_HAS_PAN;
    }
    if sample.vibrato_speed > 0 || sample.vibrato_depth > 0 {
        flags |= SAMPLE_HAS_VIBRATO;
    }
    output.push(flags);

    // Core metadata
    output.push(sample.global_volume);
    output.push(sample.default_volume);
    write_u32(output, sample.c5_speed);

    // Optional loop points
    if flags & SAMPLE_HAS_LOOP != 0 {
        write_u32(output, sample.loop_begin);
        write_u32(output, sample.loop_end);
    }

    // Optional sustain loop points
    if flags & SAMPLE_HAS_SUSTAIN != 0 {
        write_u32(output, sample.sustain_loop_begin);
        write_u32(output, sample.sustain_loop_end);
    }

    // Optional default panning
    if flags & SAMPLE_HAS_PAN != 0 {
        output.push(sample.default_pan.unwrap_or(32));
    }

    // Optional vibrato settings
    if flags & SAMPLE_HAS_VIBRATO != 0 {
        output.push(sample.vibrato_speed);
        output.push(sample.vibrato_depth);
        output.push(sample.vibrato_rate);
        output.push(sample.vibrato_type);
    }
}

/// Pack a pattern to NCIT format (using IT's pattern packing)
fn pack_pattern(output: &mut Vec<u8>, pattern: &ItPattern, num_channels: u8) {
    let packed = pack_pattern_data(pattern, num_channels);
    write_u16(output, pattern.num_rows);
    write_u16(output, packed.len() as u16);
    output.extend_from_slice(&packed);
}

/// Pack pattern data using IT compression
fn pack_pattern_data(pattern: &ItPattern, num_channels: u8) -> Vec<u8> {
    let mut packed = Vec::new();

    // Previous values for compression
    let mut prev_note = [0u8; 64];
    let mut prev_instrument = [0u8; 64];
    let mut prev_volume = [0u8; 64];
    let mut prev_effect = [0u8; 64];
    let mut prev_effect_param = [0u8; 64];

    for row in &pattern.notes {
        for (channel, note) in row.iter().enumerate().take(num_channels as usize) {
            // Skip empty notes
            if note.note == 0
                && note.instrument == 0
                && note.volume == 0
                && note.effect == 0
                && note.effect_param == 0
            {
                continue;
            }

            // Build mask
            let mut mask = 0u8;

            if note.note != 0 && note.note != prev_note[channel] {
                mask |= 0x01;
                prev_note[channel] = note.note;
            } else if note.note != 0 {
                mask |= 0x10;
            }

            if note.instrument != 0 && note.instrument != prev_instrument[channel] {
                mask |= 0x02;
                prev_instrument[channel] = note.instrument;
            } else if note.instrument != 0 {
                mask |= 0x20;
            }

            if note.volume != 0 && note.volume != prev_volume[channel] {
                mask |= 0x04;
                prev_volume[channel] = note.volume;
            } else if note.volume != 0 {
                mask |= 0x40;
            }

            if (note.effect != 0 || note.effect_param != 0)
                && (note.effect != prev_effect[channel]
                    || note.effect_param != prev_effect_param[channel])
            {
                mask |= 0x08;
                prev_effect[channel] = note.effect;
                prev_effect_param[channel] = note.effect_param;
            } else if note.effect != 0 || note.effect_param != 0 {
                mask |= 0x80;
            }

            if mask == 0 {
                continue;
            }

            // Write channel marker with mask flag
            packed.push((channel as u8) | 0x80);
            packed.push(mask);

            // Write data
            if mask & 0x01 != 0 {
                packed.push(note.note);
            }
            if mask & 0x02 != 0 {
                packed.push(note.instrument);
            }
            if mask & 0x04 != 0 {
                packed.push(note.volume);
            }
            if mask & 0x08 != 0 {
                packed.push(note.effect);
                packed.push(note.effect_param);
            }
        }

        // End of row marker
        packed.push(0);
    }

    packed
}

// =============================================================================
// Parse Functions (NCIT binary → ItModule)
// =============================================================================

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
fn parse_instrument(cursor: &mut Cursor<&[u8]>) -> Result<ItInstrument, ItError> {
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
fn parse_note_sample_table(cursor: &mut Cursor<&[u8]>) -> Result<[(u8, u8); 120], ItError> {
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
fn parse_envelope(cursor: &mut Cursor<&[u8]>) -> Result<ItEnvelope, ItError> {
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
fn parse_sample(cursor: &mut Cursor<&[u8]>) -> Result<ItSample, ItError> {
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
fn parse_pattern(cursor: &mut Cursor<&[u8]>, num_channels: u8) -> Result<ItPattern, ItError> {
    let num_rows = read_u16(cursor)?;
    let packed_size = read_u16(cursor)?;

    let mut packed_data = vec![0u8; packed_size as usize];
    cursor.read_exact(&mut packed_data)?;

    let notes = unpack_pattern_data(&packed_data, num_rows, num_channels)?;

    Ok(ItPattern { num_rows, notes })
}

/// Unpack pattern data from IT packed format
fn unpack_pattern_data(
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

// =============================================================================
// Auto-detection Wrapper
// =============================================================================

/// Parse IT file with auto-detection (NCIT or standard IT)
///
/// This function auto-detects the format by checking for the IT magic bytes.
/// If present, it parses as standard IT format; otherwise, it parses as NCIT.
///
/// # Arguments
/// * `data` - Raw binary data (either IT or NCIT format)
///
/// # Returns
/// * `Ok(ItModule)` - Parsed module
/// * `Err(ItError)` - Parse error
pub fn parse_it_minimal(data: &[u8]) -> Result<ItModule, ItError> {
    if data.len() >= 4 && &data[0..4] == IT_MAGIC {
        // Standard IT format - use full parser
        parse_it(data)
    } else {
        // Assume NCIT minimal format
        parse_ncit(data)
    }
}

// =============================================================================
// Legacy Functions (for backwards compatibility)
// =============================================================================

/// Strip sample data from an IT file, keeping only patterns and metadata
///
/// This creates a minimal IT file that can be stored in the ROM with much smaller size.
/// Sample data is loaded separately via the ROM data pack.
///
/// **Note**: This function is deprecated in favor of `pack_ncit()` which produces
/// a more compact format.
pub fn strip_it_samples(data: &[u8]) -> Result<Vec<u8>, ItError> {
    let module = parse_it(data)?;
    Ok(pack_it_minimal(&module))
}

/// Pack an IT module into minimal IT format (legacy, maintains IT file validity)
///
/// **Note**: For new code, prefer `pack_ncit()` which produces a more compact format.
/// This function is retained for backwards compatibility and debugging (output can
/// be loaded in OpenMPT/SchismTracker).
pub fn pack_it_minimal(module: &ItModule) -> Vec<u8> {
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
                if note.note != 0
                    || note.instrument != 0
                    || note.volume != 0
                    || note.effect != 0
                    || note.effect_param != 0
                {
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

// =============================================================================
// Helper Functions
// =============================================================================

fn write_u16<W: Write>(output: &mut W, val: u16) {
    output.write_all(&val.to_le_bytes()).unwrap();
}

fn write_u32<W: Write>(output: &mut W, val: u32) {
    output.write_all(&val.to_le_bytes()).unwrap();
}

fn read_u8(cursor: &mut Cursor<&[u8]>) -> Result<u8, ItError> {
    let mut buf = [0u8; 1];
    cursor
        .read_exact(&mut buf)
        .map_err(|_| ItError::UnexpectedEof)?;
    Ok(buf[0])
}

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

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::module::*;

    /// Create a minimal test module
    fn create_test_module() -> ItModule {
        let mut module = ItModule::default();
        module.name = "Test".to_string();
        module.num_channels = 4;
        module.initial_speed = 6;
        module.initial_tempo = 125;

        // Add a pattern
        let mut pattern = ItPattern::empty(64, 4);
        pattern.notes[0][0] = ItNote::play_note(48, 1, 64); // C-4
        module.patterns.push(pattern);
        module.num_patterns = 1;

        // Add an instrument
        let mut instr = ItInstrument::default();
        instr.fadeout = 256;
        instr.volume_envelope = Some(ItEnvelope {
            points: vec![(0, 64), (10, 32), (20, 0)],
            loop_begin: 0,
            loop_end: 2,
            sustain_begin: 1,
            sustain_end: 1,
            flags: ItEnvelopeFlags::ENABLED | ItEnvelopeFlags::SUSTAIN_LOOP,
        });
        module.instruments.push(instr);
        module.num_instruments = 1;

        // Add a sample
        let mut sample = ItSample::default();
        sample.c5_speed = 22050;
        sample.loop_begin = 100;
        sample.loop_end = 1000;
        sample.flags = ItSampleFlags::LOOP;
        module.samples.push(sample);
        module.num_samples = 1;

        // Set order table
        module.order_table = vec![0];
        module.num_orders = 1;

        module
    }

    #[test]
    fn test_pack_and_parse_ncit() {
        let module = create_test_module();

        // Pack to NCIT
        let ncit = pack_ncit(&module);

        // Verify it doesn't start with IT magic
        assert_ne!(&ncit[0..4], IT_MAGIC);

        // Parse back
        let parsed = parse_ncit(&ncit).expect("Failed to parse NCIT");

        // Verify header fields
        assert_eq!(parsed.num_channels, module.num_channels);
        assert_eq!(parsed.num_patterns, module.num_patterns);
        assert_eq!(parsed.num_instruments, module.num_instruments);
        assert_eq!(parsed.num_samples, module.num_samples);
        assert_eq!(parsed.initial_speed, module.initial_speed);
        assert_eq!(parsed.initial_tempo, module.initial_tempo);
        assert_eq!(parsed.global_volume, module.global_volume);

        // Verify patterns
        assert_eq!(parsed.patterns.len(), 1);
        assert_eq!(parsed.patterns[0].num_rows, 64);

        // Verify instruments
        assert_eq!(parsed.instruments.len(), 1);
        assert_eq!(parsed.instruments[0].fadeout, 256);
        assert!(parsed.instruments[0].volume_envelope.is_some());

        // Verify samples
        assert_eq!(parsed.samples.len(), 1);
        assert_eq!(parsed.samples[0].c5_speed, 22050);
        assert_eq!(parsed.samples[0].loop_begin, 100);
        assert_eq!(parsed.samples[0].loop_end, 1000);
    }

    #[test]
    fn test_ncit_size_reduction() {
        let module = create_test_module();

        // Pack to both formats
        let ncit = pack_ncit(&module);
        let legacy = pack_it_minimal(&module);

        println!("NCIT size: {} bytes", ncit.len());
        println!("Legacy IT size: {} bytes", legacy.len());
        println!(
            "Savings: {} bytes ({:.1}%)",
            legacy.len() - ncit.len(),
            (1.0 - ncit.len() as f64 / legacy.len() as f64) * 100.0
        );

        // NCIT should be significantly smaller
        assert!(
            ncit.len() < legacy.len() / 2,
            "NCIT should be at least 50% smaller than legacy"
        );
    }

    #[test]
    fn test_auto_detection() {
        let module = create_test_module();

        // Pack to NCIT
        let ncit = pack_ncit(&module);

        // Auto-detect should recognize NCIT
        let parsed = parse_it_minimal(&ncit).expect("Failed to auto-detect NCIT");
        assert_eq!(parsed.num_channels, module.num_channels);

        // Pack to legacy IT
        let legacy = pack_it_minimal(&module);

        // Auto-detect should recognize IT
        let parsed2 = parse_it_minimal(&legacy).expect("Failed to auto-detect IT");
        assert_eq!(parsed2.num_channels, module.num_channels);
    }

    #[test]
    fn test_note_sample_table_compression() {
        // Test uniform table
        let mut table = [(0u8, 0u8); 120];
        for (i, entry) in table.iter_mut().enumerate() {
            entry.0 = i as u8;
            entry.1 = 1; // All use sample 1
        }

        let mut output = Vec::new();
        pack_note_sample_table(&mut output, &table);

        // Uniform encoding should be tiny (2 bytes)
        assert_eq!(output.len(), 2);
        assert_eq!(output[0], TABLE_UNIFORM);
        assert_eq!(output[1], 1);

        // Parse it back
        let mut cursor = Cursor::new(output.as_slice());
        let parsed = parse_note_sample_table(&mut cursor).unwrap();

        for (i, &(note, sample)) in parsed.iter().enumerate() {
            assert_eq!(note, i as u8);
            assert_eq!(sample, 1);
        }
    }

    #[test]
    fn test_envelope_round_trip() {
        let env = ItEnvelope {
            points: vec![(0, 64), (10, 32), (20, 0)],
            loop_begin: 0,
            loop_end: 2,
            sustain_begin: 1,
            sustain_end: 1,
            flags: ItEnvelopeFlags::ENABLED | ItEnvelopeFlags::LOOP,
        };

        let mut output = Vec::new();
        pack_envelope(&mut output, &env);

        let mut cursor = Cursor::new(output.as_slice());
        let parsed = parse_envelope(&mut cursor).unwrap();

        assert_eq!(parsed.points.len(), 3);
        assert_eq!(parsed.points[0], (0, 64));
        assert_eq!(parsed.points[1], (10, 32));
        assert_eq!(parsed.points[2], (20, 0));
        assert_eq!(parsed.loop_begin, 0);
        assert_eq!(parsed.loop_end, 2);
        assert!(parsed.flags.contains(ItEnvelopeFlags::ENABLED));
        assert!(parsed.flags.contains(ItEnvelopeFlags::LOOP));
    }

    #[test]
    fn test_sample_round_trip() {
        let sample = ItSample {
            name: String::new(),
            filename: String::new(),
            global_volume: 48,
            flags: ItSampleFlags::LOOP | ItSampleFlags::PINGPONG_LOOP,
            default_volume: 32,
            default_pan: Some(16),
            length: 0,
            loop_begin: 100,
            loop_end: 500,
            c5_speed: 44100,
            sustain_loop_begin: 0,
            sustain_loop_end: 0,
            vibrato_speed: 10,
            vibrato_depth: 20,
            vibrato_rate: 30,
            vibrato_type: 1,
        };

        let mut output = Vec::new();
        pack_sample(&mut output, &sample);

        let mut cursor = Cursor::new(output.as_slice());
        let parsed = parse_sample(&mut cursor).unwrap();

        assert_eq!(parsed.global_volume, 48);
        assert_eq!(parsed.default_volume, 32);
        assert_eq!(parsed.default_pan, Some(16));
        assert_eq!(parsed.c5_speed, 44100);
        assert_eq!(parsed.loop_begin, 100);
        assert_eq!(parsed.loop_end, 500);
        assert!(parsed.flags.contains(ItSampleFlags::LOOP));
        assert!(parsed.flags.contains(ItSampleFlags::PINGPONG_LOOP));
        assert_eq!(parsed.vibrato_speed, 10);
        assert_eq!(parsed.vibrato_depth, 20);
    }

    #[test]
    fn test_multiple_round_trips() {
        let module = create_test_module();

        // Do multiple round-trips
        let ncit1 = pack_ncit(&module);
        let parsed1 = parse_ncit(&ncit1).unwrap();
        let ncit2 = pack_ncit(&parsed1);
        let parsed2 = parse_ncit(&ncit2).unwrap();

        // Should be identical after multiple round-trips
        assert_eq!(parsed1.num_channels, parsed2.num_channels);
        assert_eq!(parsed1.num_patterns, parsed2.num_patterns);
        assert_eq!(parsed1.initial_speed, parsed2.initial_speed);
        assert_eq!(
            ncit1, ncit2,
            "NCIT data should be identical after round-trip"
        );
    }

    #[test]
    fn test_channel_and_random_settings_round_trip() {
        let mut module = create_test_module();

        // Set custom channel settings
        module.channel_pan[0] = 0; // Hard left
        module.channel_pan[1] = 64; // Hard right
        module.channel_pan[2] = 32; // Center
        module.channel_pan[3] = 48; // Right-center
        module.channel_vol[0] = 48; // 75% volume
        module.channel_vol[1] = 32; // 50% volume
        module.channel_vol[2] = 64; // 100% volume
        module.channel_vol[3] = 16; // 25% volume

        // Set random variation on instrument
        module.instruments[0].random_volume = 10; // ±10% volume
        module.instruments[0].random_pan = 5; // ±5 pan units

        // Pack and parse
        let ncit = pack_ncit(&module);
        let parsed = parse_ncit(&ncit).unwrap();

        // Verify channel panning preserved
        assert_eq!(
            parsed.channel_pan[0], 0,
            "Channel 0 pan should be hard left"
        );
        assert_eq!(
            parsed.channel_pan[1], 64,
            "Channel 1 pan should be hard right"
        );
        assert_eq!(parsed.channel_pan[2], 32, "Channel 2 pan should be center");
        assert_eq!(
            parsed.channel_pan[3], 48,
            "Channel 3 pan should be right-center"
        );

        // Verify channel volume preserved
        assert_eq!(parsed.channel_vol[0], 48, "Channel 0 vol should be 75%");
        assert_eq!(parsed.channel_vol[1], 32, "Channel 1 vol should be 50%");
        assert_eq!(parsed.channel_vol[2], 64, "Channel 2 vol should be 100%");
        assert_eq!(parsed.channel_vol[3], 16, "Channel 3 vol should be 25%");

        // Verify random settings preserved
        assert_eq!(
            parsed.instruments[0].random_volume, 10,
            "Random volume should be preserved"
        );
        assert_eq!(
            parsed.instruments[0].random_pan, 5,
            "Random pan should be preserved"
        );
    }
}
