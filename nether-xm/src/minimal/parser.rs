//! XM parsing functions - converts minimal binary format to XmModule

use std::io::{Cursor, Read, Seek, SeekFrom};

use crate::error::XmError;
use crate::module::{XmEnvelope, XmInstrument, XmModule, XmPattern};

use super::io::{read_u8, read_u16, read_u32};
use super::{FLAG_LEGACY_MIX, FLAG_SAMPLE_PREAMP, FLAG_FT2_MIX, FLAG_LINEAR_FREQUENCY, FLAG_SAMPLE_DEFAULT_PAN, FLAG_SAMPLE_MAP, HEADER_SIZE, SUPPORTED_FLAGS};

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
    let conflicting_mix = flags & (FLAG_FT2_MIX | FLAG_LEGACY_MIX) == (FLAG_FT2_MIX | FLAG_LEGACY_MIX);
    let unsupported_flags = (flags & !SUPPORTED_FLAGS) | if conflicting_mix { FLAG_FT2_MIX | FLAG_LEGACY_MIX } else {0};
    if unsupported_flags != 0 {
        return Err(XmError::UnsupportedMinimalFlags(unsupported_flags));
    }
    let linear_frequency_table = (flags & FLAG_LINEAR_FREQUENCY) != 0;
    let has_sample_pan = (flags & FLAG_SAMPLE_DEFAULT_PAN) != 0;
    let mix_mode = if flags & FLAG_LEGACY_MIX != 0 {
        crate::XmMixMode::Legacy
    } else if flags & FLAG_FT2_MIX != 0 {
        crate::XmMixMode::Ft2
    } else {
        crate::XmMixMode::Compatible
    };

    // Skip reserved bytes
    let encoded_preamp = read_u8(&mut cursor)?;
    let sample_preamp = if flags & FLAG_SAMPLE_PREAMP != 0 { encoded_preamp } else { 48 };
    cursor.seek(SeekFrom::Current(1))?;

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
        let instrument = read_instrument(&mut cursor, has_sample_pan, flags & FLAG_SAMPLE_MAP != 0)?;
        instruments.push(instrument);
    }

    Ok(XmModule {
        name: String::new(), // Not stored in minimal format
        mix_mode,
        legacy_retrigger: flags & super::FLAG_LEGACY_RETRIGGER != 0,
        sample_preamp,
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
fn read_instrument(
    cursor: &mut Cursor<&[u8]>,
    has_sample_pan: bool,
    has_sample_map: bool,
) -> Result<XmInstrument, XmError> {
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
        sample_is_stereo,
        sample_default_volume,
        sample_default_pan,
    ) = if num_samples > 0 {
        let loop_start = read_u32(cursor)?;
        let loop_length = read_u32(cursor)?;
        let finetune = read_u8(cursor)? as i8;
        let relative_note = read_u8(cursor)? as i8;
        let encoded_loop_type = read_u8(cursor)?;
        let loop_type = encoded_loop_type & 0x03;
        let is_stereo = encoded_loop_type & super::SAMPLE_STEREO != 0;
        let encoded_volume = read_u8(cursor)?;
        if encoded_volume > 65 {
            return Err(XmError::InvalidSampleVolume(encoded_volume));
        }
        // NCXM legacy files used this byte as reserved; zero therefore means
        // the old full-volume default rather than an explicit zero volume.
        let sample_default_volume = if encoded_volume == 0 {
            Some(64)
        } else {
            Some(encoded_volume - 1)
        };
        let sample_default_pan = if has_sample_pan && num_samples == 1 {
            Some(read_u8(cursor)?)
        } else {
            None
        };
        (
            loop_start,
            loop_length,
            finetune,
            relative_note,
            loop_type,
            is_stereo,
            sample_default_volume,
            sample_default_pan,
        )
    } else {
        (0, 0, 0, 0, 0, false, None, None)
    };

    let mut sample_map = Vec::new();
    let mut samples = Vec::new();
    if has_sample_map && num_samples > 0 {
        sample_map.resize(96, 0);
        cursor.read_exact(&mut sample_map)?;
        if sample_map.iter().any(|&s| s >= num_samples) {
            return Err(XmError::InvalidInstrument(0));
        }
        for _ in 0..num_samples {
            let loop_start = read_u32(cursor)?;
            let loop_length = read_u32(cursor)?;
            let finetune = read_u8(cursor)? as i8;
            let relative_note = read_u8(cursor)? as i8;
            let encoded_loop_type = read_u8(cursor)?;
            let loop_type = encoded_loop_type & 0x03;
            let is_stereo = encoded_loop_type & super::SAMPLE_STEREO != 0;
            let volume = read_u8(cursor)?;
            let pan = read_u8(cursor)?;
            if volume > 64 { return Err(XmError::InvalidSampleVolume(volume)); }
            samples.push(crate::XmSample {source_sample_bytes: None, loop_start, loop_length, finetune, relative_note, loop_type, is_stereo, volume, pan});
        }
    }
    Ok(XmInstrument {
        sample_map,
        samples,
        source_sample_bytes: None,
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
        sample_is_stereo,
        sample_default_volume,
        sample_default_pan,
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
