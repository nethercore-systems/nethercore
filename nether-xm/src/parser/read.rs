//! XM file parsing and reading functions

use std::io::{Cursor, Read, Seek, SeekFrom};

use crate::error::XmError;
use crate::module::{XmEnvelope, XmInstrument, XmModule, XmNote, XmPattern, XmSample};
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

    let mut tracker_name = [0u8; 20];
    cursor.read_exact(&mut tracker_name)?;
    let mut mix_mode = crate::XmMixMode::from_tracker_name(&tracker_name)?;

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
    let num_channels =
        u8::try_from(read_u16(&mut cursor)?).map_err(|_| XmError::InvalidHeaderSize)?;
    if num_channels == 0 {
        return Err(XmError::InvalidHeaderSize);
    }
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
    let mut non_space_sample_name = false;
    let mut compatible_empty_header = false;
    for instr_idx in 0..num_instruments {
        let start = cursor.position() as usize;
        let instrument =
            parse_instrument(&mut cursor).map_err(|_| XmError::InvalidInstrument(instr_idx))?;
        if instrument.num_samples == 0 {
            let size = u32::from_le_bytes(data[start..start + 4].try_into().unwrap());
            compatible_empty_header |= !matches!(size, 33 | 263);
        }
        if !instrument.samples.is_empty() {
            // Extents were validated by parse_instrument; retain only the measured
            // raw padding distinction, not another persisted creator field.
            let header = u32::from_le_bytes(data[start..start + 4].try_into().unwrap()) as usize;
            let stride =
                u32::from_le_bytes(data[start + 29..start + 33].try_into().unwrap()) as usize;
            for index in 0..instrument.samples.len() {
                let name = start + header + index * stride + 18;
                non_space_sample_name |= data
                    .get(name..name + 22)
                    .is_some_and(|bytes| bytes != b"                      ");
            }
        }
        instruments.push(instrument);
    }
    // Original header perturbations and mixed-slot playback independently
    // distinguish this legacy default from the space-padded FT2 sample form.
    let legacy_sample_form = tracker_name == *b"FastTracker v2.00   " && non_space_sample_name;
    if legacy_sample_form {
        mix_mode = crate::XmMixMode::Legacy;
    }
    // Original empty-header/pan controls distinguish this marker-specific default.
    // Explicit trailing mixer properties below still take precedence.
    if tracker_name == *b"FastTracker v2.00   " && compatible_empty_header {
        mix_mode = crate::XmMixMode::Compatible;
    }

    let mut legacy_retrigger = legacy_sample_form;
    let sample_preamp = read_mix_metadata(
        data.get(cursor.position() as usize..).unwrap_or_default(),
        num_instruments,
        &mut mix_mode,
        &mut legacy_retrigger,
        48,
    )?;

    Ok(XmModule {
        name,
        mix_mode,
        legacy_retrigger,
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

/// Read length-framed mixer properties; never search arbitrary payload bytes for tags.
fn read_mix_metadata(
    mut bytes: &[u8],
    instruments: u16,
    mode: &mut crate::XmMixMode,
    legacy_retrigger: &mut bool,
    mut preamp: u8,
) -> Result<u8, XmError> {
    let mut explicit_preamp = false;
    let mut field_multiplier = None;
    let mut song_fields = false;
    while !bytes.is_empty() {
        if bytes.starts_with(b"STPM") || bytes.starts_with(b"XTPM") {
            song_fields = bytes.starts_with(b"STPM");
            field_multiplier = Some(if song_fields {
                1
            } else {
                usize::from(instruments)
            });
            bytes = &bytes[4..];
            continue;
        }
        let Some(multiplier) = field_multiplier else {
            // Optional ordinary chunks have a four-byte length. Opaque trailing
            // data is not itself evidence of an unsupported mixer mode.
            if bytes.len() < 8 {
                break;
            }
            let length = u32::from_le_bytes(bytes[4..8].try_into().unwrap()) as usize;
            let Some(rest) = bytes.get(8..).and_then(|payload| payload.get(length..)) else {
                break;
            };
            bytes = rest;
            continue;
        };
        if bytes.len() < 6 {
            return Err(XmError::UnexpectedEof);
        }
        let size = usize::from(u16::from_le_bytes([bytes[4], bytes[5]]));
        let length = size.checked_mul(multiplier).ok_or(XmError::UnexpectedEof)?;
        let payload = bytes
            .get(6..)
            .and_then(|p| p.get(..length))
            .ok_or(XmError::UnexpectedEof)?;
        if song_fields && matches!(&bytes[..4], b".MMP" | b".APS") {
            let value = u32::from_le_bytes(
                payload
                    .try_into()
                    .map_err(|_| XmError::UnsupportedMixMetadata)?,
            );
            if &bytes[..4] == b".MMP" {
                if !explicit_preamp {
                    preamp = 48;
                }
                if value == 4 {
                    *legacy_retrigger = true;
                }
                *mode = match value {
                    4 => crate::XmMixMode::Compatible,
                    5 => crate::XmMixMode::Ft2,
                    _ => return Err(XmError::UnsupportedMixMetadata),
                };
            } else {
                preamp = u8::try_from(value).map_err(|_| XmError::UnsupportedMixMetadata)?;
                explicit_preamp = true;
            }
        }
        bytes = &bytes[6 + length..];
    }
    Ok(preamp)
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
    if header_length < 9 {
        return Err(XmError::InvalidHeaderSize);
    }

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
    let pattern_start = header_start + u64::from(header_length);
    let pattern_end = pattern_start + u64::from(packed_size);
    if pattern_end > cursor.get_ref().len() as u64 {
        return Err(XmError::UnexpectedEof);
    }
    cursor.seek(SeekFrom::Start(pattern_start))?;

    // Unpack pattern data
    let mut notes = Vec::with_capacity(num_rows as usize);

    if packed_size == 0 {
        // Empty pattern - fill with default notes
        for _ in 0..num_rows {
            notes.push(vec![XmNote::default(); num_channels as usize]);
        }
    } else {
        // Read and unpack pattern data
        let payload = &cursor.get_ref()[pattern_start as usize..pattern_end as usize];
        let mut packed = Cursor::new(payload);

        for _ in 0..num_rows {
            let mut row = Vec::with_capacity(num_channels as usize);

            for _ in 0..num_channels {
                let note = unpack_note(&mut packed)?;
                row.push(note);
            }

            notes.push(row);
        }

        // Seek to end of pattern data (in case we didn't read it all)
        cursor.seek(SeekFrom::Start(pattern_end))?;
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
    if header_size < 4 {
        return Err(XmError::InvalidHeaderSize);
    }
    if header_start + header_size as u64 - 4 > cursor.get_ref().len() as u64 {
        return Err(XmError::UnexpectedEof);
    }

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
    if num_samples > u8::MAX as u16 || (num_samples > 0 && header_size < 243) {
        return Err(XmError::InvalidHeaderSize);
    }

    let mut instrument = XmInstrument {
        name,
        num_samples: num_samples as u8,
        source_sample_bytes: Some(0),
        ..Default::default()
    };

    if num_samples > 0 {
        // Sample header size (4 bytes)
        let sample_header_size = read_u32(cursor)?;
        if sample_header_size < 40 {
            return Err(XmError::InvalidHeaderSize);
        }

        // Sample number for all notes (96 bytes) - retain local sample indices.
        let mut sample_map = vec![0u8; 96];
        cursor.read_exact(&mut sample_map)?;
        if sample_map
            .iter()
            .any(|&sample_index| u16::from(sample_index) >= num_samples)
        {
            // XM keymap entries are local sample ordinals; never silently alias
            // an out-of-range entry to sample zero.
            return Err(XmError::InvalidInstrument(0));
        }
        instrument.sample_map = sample_map;

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

        // XM stores every sample header first, followed by every PCM payload.
        // Seeking over PCM between headers desynchronizes multi-sample instruments.
        let mut sample_data_length = 0u64;
        let mut samples = Vec::with_capacity(num_samples as usize);
        for _ in 0..num_samples {
            // Sample length (4 bytes)
            let sample_length = read_u32(cursor)?;

            // Sample loop start/length are stored in bytes for 16-bit samples.
            let raw_loop_start = read_u32(cursor)?;
            let raw_loop_length = read_u32(cursor)?;

            // Volume (1 byte)
            let sample_volume = read_u8(cursor)?;
            if sample_volume > 64 {
                return Err(XmError::InvalidSampleVolume(sample_volume));
            }

            // Finetune (1 byte, signed)
            let sample_finetune = read_u8(cursor)? as i8;

            // Type (1 byte)
            let sample_type = read_u8(cursor)?;
            let sample_loop_type = sample_type & 0x03;
            let is_16bit = sample_type & 0x10 != 0;
            let is_stereo = sample_type & 0x20 != 0;
            let bytes_per_frame = (if is_16bit { 2 } else { 1 }) * (if is_stereo { 2 } else { 1 });
            let (sample_loop_start, sample_loop_length) = if bytes_per_frame > 1 {
                // XM stores byte offsets; metadata uses decoded frame positions.
                let end = u64::from(raw_loop_start) + u64::from(raw_loop_length);
                (
                    raw_loop_start / bytes_per_frame,
                    (end / u64::from(bytes_per_frame) - u64::from(raw_loop_start / bytes_per_frame))
                        as u32,
                )
            } else {
                (raw_loop_start, raw_loop_length)
            };

            // Panning (1 byte)
            let sample_pan = read_u8(cursor)?;

            // Relative note (1 byte, signed)
            let sample_relative_note = read_u8(cursor)? as i8;

            // Reserved (1 byte)
            cursor.seek(SeekFrom::Current(1))?;

            // Sample name (22 bytes) - skip
            cursor.seek(SeekFrom::Current(22))?;

            // Skip remaining sample header bytes
            if sample_header_size > 40 {
                cursor.seek(SeekFrom::Current((sample_header_size - 40) as i64))?;
            }

            samples.push(XmSample {
                source_sample_bytes: Some(sample_length),
                volume: sample_volume,
                pan: sample_pan,
                finetune: sample_finetune,
                relative_note: sample_relative_note,
                loop_start: sample_loop_start,
                loop_length: sample_loop_length,
                loop_type: sample_loop_type,
                is_stereo,
            });

            // Skip sample data (we don't need it - samples come from ROM)
            sample_data_length += sample_length as u64;
        }
        instrument.samples = samples;
        if num_samples == 1 {
            let sample = &instrument.samples[0];
            instrument.sample_default_volume = Some(sample.volume);
            instrument.sample_default_pan = Some(sample.pan);
            instrument.sample_finetune = sample.finetune;
            instrument.sample_relative_note = sample.relative_note;
            instrument.sample_loop_start = sample.loop_start;
            instrument.sample_loop_length = sample.loop_length;
            instrument.sample_loop_type = sample.loop_type;
            instrument.sample_is_stereo = sample.is_stereo;
        }
        if cursor.position() + sample_data_length > cursor.get_ref().len() as u64 {
            return Err(XmError::UnexpectedEof);
        }
        instrument.source_sample_bytes = Some(sample_data_length);
        cursor.seek(SeekFrom::Current(sample_data_length as i64))?;
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
