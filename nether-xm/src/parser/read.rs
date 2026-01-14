//! XM file parsing and reading functions

use std::io::{Cursor, Read, Seek, SeekFrom};

use crate::error::XmError;
use crate::module::{XmEnvelope, XmInstrument, XmModule, XmNote, XmPattern};
use crate::{MAX_CHANNELS, MAX_PATTERN_ROWS, MAX_PATTERNS, XM_MAGIC, XM_VERSION};

/// Parse an XM file into an XmModule
///
/// This extracts pattern data and instrument metadata. Sample data is ignored
/// as it will be loaded from the ROM data pack.
///
/// # Arguments
/// * `data` - Raw XM file bytes
///
/// # Returns
/// * `Ok(XmModule)` - Parsed module
/// * `Err(XmError)` - Parse error
///
/// # Example
/// ```ignore
/// let xm_data = std::fs::read("song.xm")?;
/// let module = parse_xm(&xm_data)?;
/// println!("Loaded: {}", module.name);
/// ```
pub fn parse_xm(data: &[u8]) -> Result<XmModule, XmError> {
    if data.len() < 60 {
        return Err(XmError::TooSmall);
    }

    // Validate magic
    if &data[0..17] != XM_MAGIC {
        return Err(XmError::InvalidMagic);
    }

    let mut cursor = Cursor::new(data);

    // Skip magic (17 bytes)
    cursor.seek(SeekFrom::Start(17))?;

    // Read module name (20 bytes, null-terminated)
    let mut name_bytes = [0u8; 20];
    cursor.read_exact(&mut name_bytes)?;
    let name = read_string(&name_bytes);

    // Skip 0x1A marker (1 byte)
    cursor.seek(SeekFrom::Current(1))?;

    // Read tracker name (20 bytes) - skip it
    cursor.seek(SeekFrom::Current(20))?;

    // Read version (2 bytes)
    let version = read_u16(&mut cursor)?;
    if version != XM_VERSION {
        return Err(XmError::UnsupportedVersion(version));
    }

    // Header size (4 bytes)
    // Per XM spec, header_size is measured from the position of this field itself
    let header_start = cursor.position(); // Position BEFORE reading header_size (offset 60)
    let header_size = read_u32(&mut cursor)?;

    // Song length (2 bytes)
    let song_length = read_u16(&mut cursor)?;

    // Restart position (2 bytes)
    let restart_position = read_u16(&mut cursor)?;

    // Number of channels (2 bytes)
    let num_channels = read_u16(&mut cursor)? as u8;
    if num_channels > MAX_CHANNELS {
        return Err(XmError::TooManyChannels(num_channels));
    }

    // Number of patterns (2 bytes)
    let num_patterns = read_u16(&mut cursor)?;
    if num_patterns > MAX_PATTERNS {
        return Err(XmError::TooManyPatterns(num_patterns));
    }

    // Number of instruments (2 bytes)
    let num_instruments = read_u16(&mut cursor)?;

    // Flags (2 bytes)
    let flags = read_u16(&mut cursor)?;
    let linear_frequency_table = (flags & 1) != 0;

    // Default speed (2 bytes)
    let default_speed = read_u16(&mut cursor)?;

    // Default BPM (2 bytes)
    let default_bpm = read_u16(&mut cursor)?;

    // Pattern order table (256 bytes)
    let mut order_table = vec![0u8; 256];
    cursor.read_exact(&mut order_table)?;
    order_table.truncate(song_length as usize);

    // Seek to end of header
    cursor.seek(SeekFrom::Start(header_start + header_size as u64))?;

    // Parse patterns
    let mut patterns = Vec::with_capacity(num_patterns as usize);
    for pattern_idx in 0..num_patterns {
        let pattern = parse_pattern(&mut cursor, num_channels)
            .map_err(|_| XmError::InvalidPattern(pattern_idx))?;
        patterns.push(pattern);
    }

    // Parse instruments
    let mut instruments = Vec::with_capacity(num_instruments as usize);
    for instr_idx in 0..num_instruments {
        let instrument =
            parse_instrument(&mut cursor).map_err(|_| XmError::InvalidInstrument(instr_idx))?;
        instruments.push(instrument);
    }

    Ok(XmModule {
        name,
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

/// Parse a single pattern from the cursor
pub(crate) fn parse_pattern(
    cursor: &mut Cursor<&[u8]>,
    num_channels: u8,
) -> Result<XmPattern, XmError> {
    // Pattern header length (4 bytes)
    // Per XM spec, this value INCLUDES the 4-byte length field itself
    let header_start = cursor.position(); // Position BEFORE reading header_length
    let header_length = read_u32(cursor)?;

    // Packing type (1 byte) - always 0
    let _packing_type = read_u8(cursor)?;

    // Number of rows (2 bytes)
    let num_rows = read_u16(cursor)?;
    if num_rows == 0 || num_rows > MAX_PATTERN_ROWS {
        return Err(XmError::InvalidPattern(0));
    }

    // Packed pattern data size (2 bytes)
    let packed_size = read_u16(cursor)?;

    // Seek to end of pattern header (header_length includes the 4-byte length field)
    cursor.seek(SeekFrom::Start(header_start + header_length as u64))?;

    // Unpack pattern data
    let mut notes = Vec::with_capacity(num_rows as usize);

    if packed_size == 0 {
        // Empty pattern - fill with default notes
        for _ in 0..num_rows {
            notes.push(vec![XmNote::default(); num_channels as usize]);
        }
    } else {
        // Read and unpack pattern data
        let pattern_start = cursor.position();

        for _ in 0..num_rows {
            let mut row = Vec::with_capacity(num_channels as usize);

            for _ in 0..num_channels {
                let note = unpack_note(cursor)?;
                row.push(note);
            }

            notes.push(row);
        }

        // Seek to end of pattern data (in case we didn't read it all)
        cursor.seek(SeekFrom::Start(pattern_start + packed_size as u64))?;
    }

    Ok(XmPattern { num_rows, notes })
}

/// Unpack a single note from the pattern data
pub(crate) fn unpack_note(cursor: &mut Cursor<&[u8]>) -> Result<XmNote, XmError> {
    let first_byte = read_u8(cursor)?;

    // Check if this is a packed note (high bit set)
    if first_byte & 0x80 != 0 {
        // Packed format - first byte indicates which fields are present
        let mut note = XmNote::default();

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
        // Unpacked format - 5 bytes in sequence
        let note = first_byte;
        let instrument = read_u8(cursor)?;
        let volume = read_u8(cursor)?;
        let effect = read_u8(cursor)?;
        let effect_param = read_u8(cursor)?;

        Ok(XmNote {
            note,
            instrument,
            volume,
            effect,
            effect_param,
        })
    }
}

/// Parse a single instrument from the cursor
pub(crate) fn parse_instrument(cursor: &mut Cursor<&[u8]>) -> Result<XmInstrument, XmError> {
    // Instrument header size (4 bytes)
    let header_size = read_u32(cursor)?;
    let header_start = cursor.position();

    if header_size < 29 {
        // Minimal header - seek past and return empty instrument
        cursor.seek(SeekFrom::Start(header_start + header_size as u64 - 4))?;
        return Ok(XmInstrument::default());
    }

    // Instrument name (22 bytes)
    let mut name_bytes = [0u8; 22];
    cursor.read_exact(&mut name_bytes)?;
    let name = read_string(&name_bytes);

    // Instrument type (1 byte) - always 0
    let _instrument_type = read_u8(cursor)?;

    // Number of samples (2 bytes)
    let num_samples = read_u16(cursor)?;

    let mut instrument = XmInstrument {
        name,
        num_samples: num_samples as u8,
        ..Default::default()
    };

    if num_samples > 0 {
        // Sample header size (4 bytes)
        let sample_header_size = read_u32(cursor)?;

        // Sample number for all notes (96 bytes) - skip
        cursor.seek(SeekFrom::Current(96))?;

        // Volume envelope points (48 bytes = 12 points * 4 bytes)
        let mut vol_points = Vec::with_capacity(12);
        for _ in 0..12 {
            let x = read_u16(cursor)?;
            let y = read_u16(cursor)?;
            vol_points.push((x, y));
        }

        // Panning envelope points (48 bytes = 12 points * 4 bytes)
        let mut pan_points = Vec::with_capacity(12);
        for _ in 0..12 {
            let x = read_u16(cursor)?;
            let y = read_u16(cursor)?;
            pan_points.push((x, y));
        }

        // Number of volume envelope points (1 byte)
        let num_vol_points = read_u8(cursor)?;
        // Number of panning envelope points (1 byte)
        let num_pan_points = read_u8(cursor)?;

        // Volume sustain point (1 byte)
        let vol_sustain = read_u8(cursor)?;
        // Volume loop start (1 byte)
        let vol_loop_start = read_u8(cursor)?;
        // Volume loop end (1 byte)
        let vol_loop_end = read_u8(cursor)?;

        // Panning sustain point (1 byte)
        let pan_sustain = read_u8(cursor)?;
        // Panning loop start (1 byte)
        let pan_loop_start = read_u8(cursor)?;
        // Panning loop end (1 byte)
        let pan_loop_end = read_u8(cursor)?;

        // Volume type (1 byte)
        let vol_type = read_u8(cursor)?;
        // Panning type (1 byte)
        let pan_type = read_u8(cursor)?;

        // Vibrato type (1 byte)
        instrument.vibrato_type = read_u8(cursor)?;
        // Vibrato sweep (1 byte)
        instrument.vibrato_sweep = read_u8(cursor)?;
        // Vibrato depth (1 byte)
        instrument.vibrato_depth = read_u8(cursor)?;
        // Vibrato rate (1 byte)
        instrument.vibrato_rate = read_u8(cursor)?;

        // Volume fadeout (2 bytes)
        instrument.volume_fadeout = read_u16(cursor)?;

        // Reserved (2 bytes) - skip to end of header
        cursor.seek(SeekFrom::Start(header_start + header_size as u64 - 4))?;

        // Build volume envelope
        if num_vol_points > 0 && (vol_type & 1) != 0 {
            vol_points.truncate(num_vol_points as usize);
            instrument.volume_envelope = Some(XmEnvelope {
                points: vol_points,
                sustain_point: vol_sustain,
                loop_start: vol_loop_start,
                loop_end: vol_loop_end,
                enabled: true,
                sustain_enabled: (vol_type & 2) != 0,
                loop_enabled: (vol_type & 4) != 0,
            });
        }

        // Build panning envelope
        if num_pan_points > 0 && (pan_type & 1) != 0 {
            pan_points.truncate(num_pan_points as usize);
            instrument.panning_envelope = Some(XmEnvelope {
                points: pan_points,
                sustain_point: pan_sustain,
                loop_start: pan_loop_start,
                loop_end: pan_loop_end,
                enabled: true,
                sustain_enabled: (pan_type & 2) != 0,
                loop_enabled: (pan_type & 4) != 0,
            });
        }

        // Parse sample headers and skip sample data
        for _ in 0..num_samples {
            // Sample length (4 bytes)
            let sample_length = read_u32(cursor)?;

            // Sample loop start (4 bytes)
            instrument.sample_loop_start = read_u32(cursor)?;

            // Sample loop length (4 bytes)
            instrument.sample_loop_length = read_u32(cursor)?;

            // Volume (1 byte) - skip
            cursor.seek(SeekFrom::Current(1))?;

            // Finetune (1 byte, signed)
            instrument.sample_finetune = read_u8(cursor)? as i8;

            // Type (1 byte)
            let sample_type = read_u8(cursor)?;
            instrument.sample_loop_type = sample_type & 0x03;

            // Panning (1 byte) - skip
            cursor.seek(SeekFrom::Current(1))?;

            // Relative note (1 byte, signed)
            instrument.sample_relative_note = read_u8(cursor)? as i8;

            // Reserved (1 byte)
            cursor.seek(SeekFrom::Current(1))?;

            // Sample name (22 bytes) - skip
            cursor.seek(SeekFrom::Current(22))?;

            // Skip remaining sample header bytes
            if sample_header_size > 40 {
                cursor.seek(SeekFrom::Current((sample_header_size - 40) as i64))?;
            }

            // Skip sample data (we don't need it - samples come from ROM)
            cursor.seek(SeekFrom::Current(sample_length as i64))?;
        }
    } else {
        // Seek to end of header
        cursor.seek(SeekFrom::Start(header_start + header_size as u64 - 4))?;
    }

    Ok(instrument)
}

/// Get list of instrument names from an XM file (for sample ID mapping)
///
/// This is useful during asset packing to validate that all required
/// samples are present in the ROM.
pub fn get_instrument_names(data: &[u8]) -> Result<Vec<String>, XmError> {
    let module = parse_xm(data)?;
    Ok(module.instruments.iter().map(|i| i.name.clone()).collect())
}

// =============================================================================
// Helper functions for reading data
// =============================================================================

pub(crate) fn read_u8(cursor: &mut Cursor<&[u8]>) -> Result<u8, XmError> {
    let mut buf = [0u8; 1];
    cursor
        .read_exact(&mut buf)
        .map_err(|_| XmError::UnexpectedEof)?;
    Ok(buf[0])
}

pub(crate) fn read_u16(cursor: &mut Cursor<&[u8]>) -> Result<u16, XmError> {
    let mut buf = [0u8; 2];
    cursor
        .read_exact(&mut buf)
        .map_err(|_| XmError::UnexpectedEof)?;
    Ok(u16::from_le_bytes(buf))
}

pub(crate) fn read_u32(cursor: &mut Cursor<&[u8]>) -> Result<u32, XmError> {
    let mut buf = [0u8; 4];
    cursor
        .read_exact(&mut buf)
        .map_err(|_| XmError::UnexpectedEof)?;
    Ok(u32::from_le_bytes(buf))
}

pub(crate) fn read_string(bytes: &[u8]) -> String {
    // Find null terminator or end of slice
    let len = bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len());
    // Trim trailing spaces and convert
    String::from_utf8_lossy(&bytes[..len])
        .trim_end()
        .to_string()
}
