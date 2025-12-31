//! Minimal XM format for Nethercore ROM packing
//!
//! This module implements a highly optimized binary format that strips all
//! unnecessary XM overhead while preserving playback data. This is designed
//! exclusively for ROM storage where samples come from a separate data pack.
//!
//! # Format Overview (NCXM - Nethercore XM)
//!
//! ```text
//! [Header: 16 bytes]
//! - num_channels: u8
//! - num_patterns: u16 (LE)
//! - num_instruments: u16 (LE)
//! - song_length: u16 (LE)
//! - restart_position: u16 (LE)
//! - default_speed: u16 (LE)
//! - default_bpm: u16 (LE)
//! - flags: u8 (bit 0 = linear_frequency_table)
//! - reserved: [u8; 2]
//!
//! [Pattern Order Table: song_length bytes]
//! - order_table[0..song_length]
//!
//! [Patterns: variable]
//! For each pattern:
//!   - num_rows: u16 (LE)
//!   - packed_size: u16 (LE)
//!   - packed_data: [u8; packed_size]
//!
//! [Instruments: variable]
//! For each instrument:
//!   - flags: u8 (bits 0-1: envelope flags, bits 2-7: num_samples)
//!   - [if has_vol_env] volume envelope data
//!   - [if has_pan_env] panning envelope data
//!   - vibrato_type: u8
//!   - vibrato_sweep: u8
//!   - vibrato_depth: u8
//!   - vibrato_rate: u8
//!   - volume_fadeout: u16 (LE)
//!   - [if num_samples > 0] sample metadata (15 bytes)
//! ```
//!
//! # Savings
//!
//! Compared to standard XM format:
//! - Removes magic header (17 bytes)
//! - Removes module name (20 bytes)
//! - Removes tracker name (20 bytes)
//! - Removes version (2 bytes)
//! - Removes 0x1A marker (1 byte)
//! - Removes pattern order padding (~200 bytes)
//! - Removes instrument names (22 bytes × N)
//! - Removes sample names (22 bytes × N × M)
//! - Removes sample headers (40 bytes × total samples)
//! - Removes all sample data (handled separately)
//!
//! **Total savings: ~1,500-3,000 bytes per typical XM file**

use std::io::{Cursor, Read, Seek, SeekFrom, Write};

use crate::error::XmError;
use crate::module::{XmEnvelope, XmInstrument, XmModule, XmPattern};
use crate::parser::pack_pattern_data;

/// Header size in bytes
const HEADER_SIZE: usize = 16;

/// Maximum envelope points we support (XM spec allows 12)
const MAX_ENVELOPE_POINTS: usize = 12;

// =============================================================================
// Packing (XmModule → minimal binary)
// =============================================================================

/// Pack an XmModule into minimal binary format
///
/// This creates a highly optimized binary representation that strips all
/// unnecessary XM overhead. The result is typically 60-80% smaller than
/// the standard XM format.
///
/// # Arguments
/// * `module` - Parsed XM module to pack
///
/// # Returns
/// * `Ok(Vec<u8>)` - Packed binary data
/// * `Err(XmError)` - Packing error
///
/// # Example
/// ```ignore
/// let xm_data = std::fs::read("song.xm")?;
/// let module = parse_xm(&xm_data)?;
/// let minimal = pack_xm_minimal(&module)?;
/// println!("Reduced from {} to {} bytes", xm_data.len(), minimal.len());
/// ```
pub fn pack_xm_minimal(module: &XmModule) -> Result<Vec<u8>, XmError> {
    let mut output = Vec::with_capacity(4096);

    // ========== Write Header ==========
    output.write_all(&[module.num_channels]).unwrap();
    write_u16(&mut output, module.num_patterns);
    write_u16(&mut output, module.num_instruments);
    write_u16(&mut output, module.song_length);
    write_u16(&mut output, module.restart_position);
    write_u16(&mut output, module.default_speed);
    write_u16(&mut output, module.default_bpm);

    // Pack flags
    let flags = if module.linear_frequency_table { 1 } else { 0 };
    output.write_all(&[flags]).unwrap();

    // Reserved bytes
    output.write_all(&[0, 0]).unwrap();

    // ========== Write Pattern Order Table (only song_length entries) ==========
    output
        .write_all(&module.order_table[..module.song_length as usize])
        .unwrap();

    // ========== Write Patterns ==========
    for pattern in &module.patterns {
        // Write pattern header
        write_u16(&mut output, pattern.num_rows);

        // Pack pattern data
        let packed_data = pack_pattern_data(pattern, module.num_channels);
        write_u16(&mut output, packed_data.len() as u16);
        output.write_all(&packed_data).unwrap();
    }

    // ========== Write Instruments ==========
    for instrument in &module.instruments {
        // Pack flags: bits 0-1 for envelopes, bits 2-7 for num_samples
        let mut instr_flags = 0u8;
        if instrument.volume_envelope.is_some() {
            instr_flags |= 0x01;
        }
        if instrument.panning_envelope.is_some() {
            instr_flags |= 0x02;
        }
        instr_flags |= (instrument.num_samples & 0x3F) << 2;
        output.write_all(&[instr_flags]).unwrap();

        // Write volume envelope if present
        if let Some(ref env) = instrument.volume_envelope {
            write_envelope(&mut output, env)?;
        }

        // Write panning envelope if present
        if let Some(ref env) = instrument.panning_envelope {
            write_envelope(&mut output, env)?;
        }

        // Write vibrato parameters
        output
            .write_all(&[
                instrument.vibrato_type,
                instrument.vibrato_sweep,
                instrument.vibrato_depth,
                instrument.vibrato_rate,
            ])
            .unwrap();

        // Write volume fadeout
        write_u16(&mut output, instrument.volume_fadeout);

        // Write sample metadata if instrument has samples
        if instrument.num_samples > 0 {
            write_u32(&mut output, instrument.sample_loop_start);
            write_u32(&mut output, instrument.sample_loop_length);
            output
                .write_all(&[instrument.sample_finetune as u8])
                .unwrap();
            output
                .write_all(&[instrument.sample_relative_note as u8])
                .unwrap();
            output.write_all(&[instrument.sample_loop_type]).unwrap();
            output.write_all(&[0]).unwrap(); // reserved
        }
    }

    Ok(output)
}

/// Write an envelope to the output stream
fn write_envelope<W: Write>(output: &mut W, env: &XmEnvelope) -> Result<(), XmError> {
    // Write number of points
    let num_points = env.points.len().min(MAX_ENVELOPE_POINTS) as u8;
    output.write_all(&[num_points]).unwrap();

    // Write sustain/loop points
    output
        .write_all(&[env.sustain_point, env.loop_start, env.loop_end])
        .unwrap();

    // Pack envelope flags
    let mut flags = 0u8;
    if env.enabled {
        flags |= 0x01;
    }
    if env.sustain_enabled {
        flags |= 0x02;
    }
    if env.loop_enabled {
        flags |= 0x04;
    }
    output.write_all(&[flags]).unwrap();

    // Write envelope points
    for i in 0..num_points as usize {
        let (x, y) = env.points[i];
        write_u16(output, x);
        write_u16(output, y);
    }

    Ok(())
}

// =============================================================================
// Parsing (minimal binary → XmModule)
// =============================================================================

/// Parse a minimal XM format into an XmModule
///
/// This can parse both the minimal NCXM format and standard XM format.
/// It auto-detects the format by checking for XM magic bytes.
///
/// # Arguments
/// * `data` - Raw binary data (NCXM or XM format)
///
/// # Returns
/// * `Ok(XmModule)` - Parsed module
/// * `Err(XmError)` - Parse error
///
/// # Example
/// ```ignore
/// let minimal_data = std::fs::read("song.ncxm")?;
/// let module = parse_xm_minimal(&minimal_data)?;
/// println!("Loaded: {} channels, {} patterns", module.num_channels, module.num_patterns);
/// ```
pub fn parse_xm_minimal(data: &[u8]) -> Result<XmModule, XmError> {
    if data.len() < HEADER_SIZE {
        return Err(XmError::TooSmall);
    }

    // Check if it's standard XM format (has magic header)
    if data.len() >= 17 && &data[0..17] == crate::XM_MAGIC {
        // Standard XM format
        crate::parse_xm(data)
    } else {
        // Assume minimal NCXM format (no magic bytes)
        parse_ncxm(data)
    }
}

/// Parse NCXM minimal format
fn parse_ncxm(data: &[u8]) -> Result<XmModule, XmError> {
    if data.len() < HEADER_SIZE {
        return Err(XmError::TooSmall);
    }

    let mut cursor = Cursor::new(data);

    // Read header (no magic bytes in minimal format)
    let num_channels = read_u8(&mut cursor)?;
    let num_patterns = read_u16(&mut cursor)?;
    let num_instruments = read_u16(&mut cursor)?;
    let song_length = read_u16(&mut cursor)?;
    let restart_position = read_u16(&mut cursor)?;
    let default_speed = read_u16(&mut cursor)?;
    let default_bpm = read_u16(&mut cursor)?;
    let flags = read_u8(&mut cursor)?;
    let linear_frequency_table = (flags & 0x01) != 0;

    // Skip reserved bytes
    cursor.seek(SeekFrom::Current(2))?;

    // Read pattern order table
    let mut order_table = vec![0u8; song_length as usize];
    cursor.read_exact(&mut order_table)?;

    // Read patterns
    let mut patterns = Vec::with_capacity(num_patterns as usize);
    for _i in 0..num_patterns {
        let pattern = read_pattern(&mut cursor, num_channels)?;
        patterns.push(pattern);
    }

    // Read instruments
    let mut instruments = Vec::with_capacity(num_instruments as usize);
    for _i in 0..num_instruments {
        let instrument = read_instrument(&mut cursor)?;
        instruments.push(instrument);
    }

    Ok(XmModule {
        name: String::new(), // Not stored in minimal format
        num_channels,
        num_patterns,
        num_instruments,
        song_length,
        restart_position,
        default_speed,
        default_bpm,
        linear_frequency_table,
        order_table,
        patterns,
        instruments,
    })
}

/// Read a pattern from minimal format
fn read_pattern(cursor: &mut Cursor<&[u8]>, num_channels: u8) -> Result<XmPattern, XmError> {
    let num_rows = read_u16(cursor)?;
    let packed_size = read_u16(cursor)?;

    // Read packed pattern data
    let mut packed_data = vec![0u8; packed_size as usize];
    cursor.read_exact(&mut packed_data)?;

    // Unpack pattern data
    let notes = unpack_pattern_data(&packed_data, num_rows, num_channels)?;

    Ok(XmPattern { num_rows, notes })
}

/// Unpack pattern data from XM packed format
fn unpack_pattern_data(
    data: &[u8],
    num_rows: u16,
    num_channels: u8,
) -> Result<Vec<Vec<crate::XmNote>>, XmError> {
    let mut cursor = Cursor::new(data);
    let mut notes = Vec::with_capacity(num_rows as usize);

    for _row in 0..num_rows {
        let mut row = Vec::with_capacity(num_channels as usize);

        for _ch in 0..num_channels {
            let note = unpack_note(&mut cursor)?;
            row.push(note);
        }

        notes.push(row);
    }

    Ok(notes)
}

/// Unpack a single note from XM packed format
fn unpack_note(cursor: &mut Cursor<&[u8]>) -> Result<crate::XmNote, XmError> {
    let first_byte = read_u8(cursor)?;

    if first_byte & 0x80 != 0 {
        // Packed format
        let mut note = crate::XmNote::default();

        if first_byte & 0x01 != 0 {
            note.note = read_u8(cursor)?;
        }
        if first_byte & 0x02 != 0 {
            note.instrument = read_u8(cursor)?;
        }
        if first_byte & 0x04 != 0 {
            note.volume = read_u8(cursor)?;
        }
        if first_byte & 0x08 != 0 {
            note.effect = read_u8(cursor)?;
        }
        if first_byte & 0x10 != 0 {
            note.effect_param = read_u8(cursor)?;
        }

        Ok(note)
    } else {
        // Unpacked format
        let note = first_byte;
        let instrument = read_u8(cursor)?;
        let volume = read_u8(cursor)?;
        let effect = read_u8(cursor)?;
        let effect_param = read_u8(cursor)?;

        Ok(crate::XmNote {
            note,
            instrument,
            volume,
            effect,
            effect_param,
        })
    }
}

/// Read an instrument from minimal format
fn read_instrument(cursor: &mut Cursor<&[u8]>) -> Result<XmInstrument, XmError> {
    let flags = read_u8(cursor)?;
    let has_vol_env = (flags & 0x01) != 0;
    let has_pan_env = (flags & 0x02) != 0;
    let num_samples = (flags >> 2) & 0x3F;

    // Read volume envelope
    let volume_envelope = if has_vol_env {
        Some(read_envelope(cursor)?)
    } else {
        None
    };

    // Read panning envelope
    let panning_envelope = if has_pan_env {
        Some(read_envelope(cursor)?)
    } else {
        None
    };

    // Read vibrato parameters
    let vibrato_type = read_u8(cursor)?;
    let vibrato_sweep = read_u8(cursor)?;
    let vibrato_depth = read_u8(cursor)?;
    let vibrato_rate = read_u8(cursor)?;

    // Read volume fadeout
    let volume_fadeout = read_u16(cursor)?;

    // Read sample metadata
    let (
        sample_loop_start,
        sample_loop_length,
        sample_finetune,
        sample_relative_note,
        sample_loop_type,
    ) = if num_samples > 0 {
        let loop_start = read_u32(cursor)?;
        let loop_length = read_u32(cursor)?;
        let finetune = read_u8(cursor)? as i8;
        let relative_note = read_u8(cursor)? as i8;
        let loop_type = read_u8(cursor)?;
        cursor.seek(SeekFrom::Current(1))?; // skip reserved byte
        (loop_start, loop_length, finetune, relative_note, loop_type)
    } else {
        (0, 0, 0, 0, 0)
    };

    Ok(XmInstrument {
        name: String::new(), // Not stored in minimal format
        num_samples,
        volume_envelope,
        panning_envelope,
        vibrato_type,
        vibrato_sweep,
        vibrato_depth,
        vibrato_rate,
        volume_fadeout,
        sample_finetune,
        sample_relative_note,
        sample_loop_start,
        sample_loop_length,
        sample_loop_type,
    })
}

/// Read an envelope from minimal format
fn read_envelope(cursor: &mut Cursor<&[u8]>) -> Result<XmEnvelope, XmError> {
    let num_points = read_u8(cursor)?;
    let sustain_point = read_u8(cursor)?;
    let loop_start = read_u8(cursor)?;
    let loop_end = read_u8(cursor)?;
    let flags = read_u8(cursor)?;

    let enabled = (flags & 0x01) != 0;
    let sustain_enabled = (flags & 0x02) != 0;
    let loop_enabled = (flags & 0x04) != 0;

    let mut points = Vec::with_capacity(num_points as usize);
    for _i in 0..num_points {
        let x = read_u16(cursor)?;
        let y = read_u16(cursor)?;
        points.push((x, y));
    }

    Ok(XmEnvelope {
        points,
        sustain_point,
        loop_start,
        loop_end,
        enabled,
        sustain_enabled,
        loop_enabled,
    })
}

// =============================================================================
// Helper functions
// =============================================================================

fn write_u16<W: Write>(output: &mut W, val: u16) {
    output.write_all(&val.to_le_bytes()).unwrap();
}

fn write_u32<W: Write>(output: &mut W, val: u32) {
    output.write_all(&val.to_le_bytes()).unwrap();
}

fn read_u8(cursor: &mut Cursor<&[u8]>) -> Result<u8, XmError> {
    let mut buf = [0u8; 1];
    cursor
        .read_exact(&mut buf)
        .map_err(|_| XmError::UnexpectedEof)?;
    Ok(buf[0])
}

fn read_u16(cursor: &mut Cursor<&[u8]>) -> Result<u16, XmError> {
    let mut buf = [0u8; 2];
    cursor
        .read_exact(&mut buf)
        .map_err(|_| XmError::UnexpectedEof)?;
    Ok(u16::from_le_bytes(buf))
}

fn read_u32(cursor: &mut Cursor<&[u8]>) -> Result<u32, XmError> {
    let mut buf = [0u8; 4];
    cursor
        .read_exact(&mut buf)
        .map_err(|_| XmError::UnexpectedEof)?;
    Ok(u32::from_le_bytes(buf))
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{XmNote, XmPattern};

    /// Create a minimal test module
    fn create_test_module() -> XmModule {
        // Create a pattern with actual 64 rows
        let mut notes = Vec::with_capacity(64);
        for row in 0..64 {
            if row == 0 {
                // First row has a note
                notes.push(vec![
                    XmNote {
                        note: 49, // C-4
                        instrument: 1,
                        volume: 0x40,
                        effect: 0,
                        effect_param: 0,
                    },
                    XmNote::default(),
                ]);
            } else {
                // Other rows are empty
                notes.push(vec![XmNote::default(), XmNote::default()]);
            }
        }

        let pattern = XmPattern {
            num_rows: 64,
            notes,
        };

        let instrument = XmInstrument {
            name: "Test".to_string(),
            num_samples: 1,
            volume_envelope: Some(XmEnvelope {
                points: vec![(0, 64), (10, 32), (20, 0)],
                sustain_point: 1,
                loop_start: 0,
                loop_end: 2,
                enabled: true,
                sustain_enabled: true,
                loop_enabled: false,
            }),
            panning_envelope: None,
            vibrato_type: 0,
            vibrato_sweep: 0,
            vibrato_depth: 0,
            vibrato_rate: 0,
            volume_fadeout: 256,
            sample_finetune: 0,
            sample_relative_note: 0,
            sample_loop_start: 0,
            sample_loop_length: 1000,
            sample_loop_type: 1,
        };

        XmModule {
            name: "Test Module".to_string(),
            num_channels: 2,
            num_patterns: 1,
            num_instruments: 1,
            song_length: 1,
            restart_position: 0,
            default_speed: 6,
            default_bpm: 125,
            linear_frequency_table: true,
            order_table: vec![0],
            patterns: vec![pattern],
            instruments: vec![instrument],
        }
    }

    #[test]
    fn test_pack_and_parse_minimal() {
        let module = create_test_module();

        // Pack to minimal format
        let packed = pack_xm_minimal(&module).expect("Packing should succeed");

        // Verify header starts with num_channels (2 in our test)
        assert_eq!(packed[0], 2);

        // Parse back
        let parsed = parse_xm_minimal(&packed).expect("Parsing should succeed");

        // Verify header fields
        assert_eq!(parsed.num_channels, module.num_channels);
        assert_eq!(parsed.num_patterns, module.num_patterns);
        assert_eq!(parsed.num_instruments, module.num_instruments);
        assert_eq!(parsed.song_length, module.song_length);
        assert_eq!(parsed.restart_position, module.restart_position);
        assert_eq!(parsed.default_speed, module.default_speed);
        assert_eq!(parsed.default_bpm, module.default_bpm);
        assert_eq!(parsed.linear_frequency_table, module.linear_frequency_table);

        // Verify pattern order
        assert_eq!(parsed.order_table, module.order_table);

        // Verify patterns
        assert_eq!(parsed.patterns.len(), 1);
        assert_eq!(parsed.patterns[0].num_rows, 64);

        // Verify instruments
        assert_eq!(parsed.instruments.len(), 1);
        let instr = &parsed.instruments[0];
        assert_eq!(instr.num_samples, 1);
        assert_eq!(instr.vibrato_type, 0);
        assert_eq!(instr.volume_fadeout, 256);
        assert_eq!(instr.sample_loop_length, 1000);
        assert_eq!(instr.sample_loop_type, 1);

        // Verify volume envelope
        assert!(instr.volume_envelope.is_some());
        let env = instr.volume_envelope.as_ref().unwrap();
        assert_eq!(env.points.len(), 3);
        assert_eq!(env.points[0], (0, 64));
        assert_eq!(env.points[1], (10, 32));
        assert_eq!(env.points[2], (20, 0));
        assert_eq!(env.sustain_point, 1);
        assert!(env.enabled);
        assert!(env.sustain_enabled);
        assert!(!env.loop_enabled);

        // Verify no panning envelope
        assert!(instr.panning_envelope.is_none());
    }

    #[test]
    fn test_minimal_format_size() {
        let module = create_test_module();
        let packed = pack_xm_minimal(&module).unwrap();

        // Verify size is minimal
        // Header: 16 bytes
        // Pattern order: 1 byte
        // Pattern: 2 (rows) + 2 (size) + data
        // Instrument: 1 (flags) + envelope + vibrato (4) + fadeout (2) + sample (15)

        println!("Packed size: {} bytes", packed.len());
        assert!(packed.len() < 200, "Minimal format should be compact");
    }

    #[test]
    fn test_multiple_patterns() {
        let mut module = create_test_module();

        // Add more patterns
        module.num_patterns = 3;
        module.patterns.push(module.patterns[0].clone());
        module.patterns.push(module.patterns[0].clone());
        module.song_length = 3;
        module.order_table = vec![0, 1, 2];

        let packed = pack_xm_minimal(&module).unwrap();
        let parsed = parse_xm_minimal(&packed).unwrap();

        assert_eq!(parsed.num_patterns, 3);
        assert_eq!(parsed.patterns.len(), 3);
        assert_eq!(parsed.order_table, vec![0, 1, 2]);
    }

    #[test]
    fn test_multiple_instruments() {
        let mut module = create_test_module();

        // Add another instrument with panning envelope
        let mut instr2 = module.instruments[0].clone();
        instr2.volume_envelope = None;
        instr2.panning_envelope = Some(XmEnvelope {
            points: vec![(0, 32), (5, 64)],
            sustain_point: 0,
            loop_start: 0,
            loop_end: 1,
            enabled: true,
            sustain_enabled: false,
            loop_enabled: true,
        });

        module.num_instruments = 2;
        module.instruments.push(instr2);

        let packed = pack_xm_minimal(&module).unwrap();
        let parsed = parse_xm_minimal(&packed).unwrap();

        assert_eq!(parsed.num_instruments, 2);
        assert_eq!(parsed.instruments.len(), 2);

        // First instrument has volume envelope
        assert!(parsed.instruments[0].volume_envelope.is_some());
        assert!(parsed.instruments[0].panning_envelope.is_none());

        // Second instrument has panning envelope
        assert!(parsed.instruments[1].volume_envelope.is_none());
        assert!(parsed.instruments[1].panning_envelope.is_some());

        let pan_env = parsed.instruments[1].panning_envelope.as_ref().unwrap();
        assert_eq!(pan_env.points.len(), 2);
        assert_eq!(pan_env.points[0], (0, 32));
        assert!(pan_env.loop_enabled);
    }

    #[test]
    fn test_instrument_without_samples() {
        let mut module = create_test_module();
        module.instruments[0].num_samples = 0;

        let packed = pack_xm_minimal(&module).unwrap();
        let parsed = parse_xm_minimal(&packed).unwrap();

        assert_eq!(parsed.instruments[0].num_samples, 0);
    }

    #[test]
    fn test_empty_pattern() {
        let mut module = create_test_module();

        // Create empty pattern
        let empty_pattern = XmPattern {
            num_rows: 64,
            notes: vec![vec![XmNote::default(), XmNote::default()]; 64],
        };
        module.patterns[0] = empty_pattern;

        let packed = pack_xm_minimal(&module).unwrap();
        let parsed = parse_xm_minimal(&packed).unwrap();

        assert_eq!(parsed.patterns[0].num_rows, 64);
        assert_eq!(parsed.patterns[0].notes.len(), 64);
    }

    #[test]
    fn test_linear_vs_amiga_frequency() {
        let mut module = create_test_module();

        // Test linear frequency table
        module.linear_frequency_table = true;
        let packed_linear = pack_xm_minimal(&module).unwrap();
        let parsed_linear = parse_xm_minimal(&packed_linear).unwrap();
        assert!(parsed_linear.linear_frequency_table);

        // Test Amiga frequency table
        module.linear_frequency_table = false;
        let packed_amiga = pack_xm_minimal(&module).unwrap();
        let parsed_amiga = parse_xm_minimal(&packed_amiga).unwrap();
        assert!(!parsed_amiga.linear_frequency_table);
    }

    #[test]
    fn test_invalid_data() {
        // Test with data that's too small
        let bad_data = b"BADMAGIC";
        let result = parse_xm_minimal(bad_data);
        assert!(result.is_err());

        // Test with corrupted header (invalid pattern count would cause issues)
        let mut bad_header = vec![0u8; HEADER_SIZE];
        bad_header[0] = 2; // num_channels
        bad_header[1] = 255; // num_patterns low byte (way too many)
        bad_header[2] = 255; // num_patterns high byte
        let result2 = parse_xm_minimal(&bad_header);
        assert!(result2.is_err());
    }

    #[test]
    fn test_truncated_data() {
        let module = create_test_module();
        let packed = pack_xm_minimal(&module).unwrap();

        // Truncate the data
        let truncated = &packed[..10];
        let result = parse_xm_minimal(truncated);
        assert!(result.is_err());
    }

    #[test]
    fn test_pattern_order_not_padded() {
        let mut module = create_test_module();
        module.song_length = 5;
        module.order_table = vec![0, 0, 0, 0, 0];

        let packed = pack_xm_minimal(&module).unwrap();
        let parsed = parse_xm_minimal(&packed).unwrap();

        // Should only store 5 bytes, not 256
        assert_eq!(parsed.order_table.len(), 5);
        assert_eq!(parsed.song_length, 5);
    }

    #[test]
    fn test_round_trip_preserves_data() {
        let module = create_test_module();

        // Do multiple round-trips
        let packed1 = pack_xm_minimal(&module).unwrap();
        let parsed1 = parse_xm_minimal(&packed1).unwrap();
        let packed2 = pack_xm_minimal(&parsed1).unwrap();
        let parsed2 = parse_xm_minimal(&packed2).unwrap();

        // Should be identical after multiple round-trips
        assert_eq!(parsed1.num_channels, parsed2.num_channels);
        assert_eq!(parsed1.num_patterns, parsed2.num_patterns);
        assert_eq!(parsed1.default_speed, parsed2.default_speed);
        assert_eq!(packed1, packed2);
    }

    #[test]
    fn test_max_envelope_points() {
        let mut module = create_test_module();

        // Create envelope with max points
        let mut points = Vec::new();
        for i in 0..MAX_ENVELOPE_POINTS {
            points.push((i as u16 * 10, (64 - i * 5) as u16));
        }

        module.instruments[0].volume_envelope = Some(XmEnvelope {
            points,
            sustain_point: 5,
            loop_start: 2,
            loop_end: 10,
            enabled: true,
            sustain_enabled: true,
            loop_enabled: true,
        });

        let packed = pack_xm_minimal(&module).unwrap();
        let parsed = parse_xm_minimal(&packed).unwrap();

        let env = parsed.instruments[0].volume_envelope.as_ref().unwrap();
        assert_eq!(env.points.len(), MAX_ENVELOPE_POINTS);
        assert_eq!(env.sustain_point, 5);
        assert_eq!(env.loop_start, 2);
        assert_eq!(env.loop_end, 10);
    }

    #[test]
    fn test_vibrato_parameters() {
        let mut module = create_test_module();
        module.instruments[0].vibrato_type = 2;
        module.instruments[0].vibrato_sweep = 16;
        module.instruments[0].vibrato_depth = 32;
        module.instruments[0].vibrato_rate = 8;

        let packed = pack_xm_minimal(&module).unwrap();
        let parsed = parse_xm_minimal(&packed).unwrap();

        let instr = &parsed.instruments[0];
        assert_eq!(instr.vibrato_type, 2);
        assert_eq!(instr.vibrato_sweep, 16);
        assert_eq!(instr.vibrato_depth, 32);
        assert_eq!(instr.vibrato_rate, 8);
    }

    #[test]
    fn test_sample_metadata() {
        let mut module = create_test_module();
        module.instruments[0].sample_loop_start = 1000;
        module.instruments[0].sample_loop_length = 5000;
        module.instruments[0].sample_finetune = -16;
        module.instruments[0].sample_relative_note = 12; // +1 octave
        module.instruments[0].sample_loop_type = 2; // Ping-pong

        let packed = pack_xm_minimal(&module).unwrap();
        let parsed = parse_xm_minimal(&packed).unwrap();

        let instr = &parsed.instruments[0];
        assert_eq!(instr.sample_loop_start, 1000);
        assert_eq!(instr.sample_loop_length, 5000);
        assert_eq!(instr.sample_finetune, -16);
        assert_eq!(instr.sample_relative_note, 12);
        assert_eq!(instr.sample_loop_type, 2);
    }

    #[test]
    fn test_minimal_format_very_compact() {
        // Create a realistic module for testing
        let mut module = create_test_module();

        // Add more complexity to make it realistic
        module.num_patterns = 4;
        for _ in 1..4 {
            module.patterns.push(module.patterns[0].clone());
        }
        module.order_table = vec![0, 1, 2, 3];
        module.song_length = 4;

        // Pack to minimal format
        let minimal = pack_xm_minimal(&module).unwrap();

        println!("Minimal NCXM size: {} bytes", minimal.len());
        println!("  Header: {} bytes", HEADER_SIZE);
        println!("  Pattern order: {} bytes", module.song_length);
        println!(
            "  Patterns + instruments: {} bytes",
            minimal.len() - HEADER_SIZE - module.song_length as usize
        );

        // Verify it's compact (no magic, no padding, no names)
        // Header (16) + order (4) + 4 patterns (64 rows each) + 1 instrument
        // Should be under 1000 bytes for this module
        assert!(
            minimal.len() < 1000,
            "Minimal format should be very compact"
        );

        // Verify it parses correctly
        let parsed = parse_xm_minimal(&minimal).unwrap();
        assert_eq!(parsed.num_channels, module.num_channels);
        assert_eq!(parsed.num_patterns, module.num_patterns);
        assert_eq!(parsed.song_length, module.song_length);
    }
}
