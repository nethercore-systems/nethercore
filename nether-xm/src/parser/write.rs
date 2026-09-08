//! XM file writing and rebuilding functions

use crate::error::XmError;
use crate::module::{XmModule, XmPattern};
use crate::{XM_MAGIC, XM_VERSION};

fn read_u32_at(data: &[u8], offset: usize) -> Result<u32, XmError> {
    let bytes = data
        .get(offset..offset.checked_add(4).ok_or(XmError::UnexpectedEof)?)
        .ok_or(XmError::UnexpectedEof)?;
    Ok(u32::from_le_bytes(bytes.try_into().unwrap()))
}

fn read_u16_at(data: &[u8], offset: usize) -> Result<u16, XmError> {
    let bytes = data
        .get(offset..offset.checked_add(2).ok_or(XmError::UnexpectedEof)?)
        .ok_or(XmError::UnexpectedEof)?;
    Ok(u16::from_le_bytes(bytes.try_into().unwrap()))
}

/// Locate original sample headers before rebuilding them without their PCM.
/// Keeping these bytes retains XM flags such as the 16-bit sample bit, which
/// is not represented in the decoded `XmSample` metadata.
fn original_sample_headers(
    original_data: &[u8],
    module: &XmModule,
) -> Result<(Vec<Vec<[u8; 40]>>, usize), XmError> {
    let header_size = usize::try_from(read_u32_at(original_data, 60)?)
        .map_err(|_| XmError::UnexpectedEof)?;
    let mut offset = 60usize
        .checked_add(header_size)
        .ok_or(XmError::UnexpectedEof)?;

    for _ in 0..module.num_patterns {
        let pattern_header_size = usize::try_from(read_u32_at(original_data, offset)?)
            .map_err(|_| XmError::UnexpectedEof)?;
        let packed_size = usize::from(read_u16_at(
            original_data,
            offset.checked_add(7).ok_or(XmError::UnexpectedEof)?,
        )?);
        offset = offset
            .checked_add(pattern_header_size)
            .and_then(|value| value.checked_add(packed_size))
            .ok_or(XmError::UnexpectedEof)?;
        if offset > original_data.len() {
            return Err(XmError::UnexpectedEof);
        }
    }

    let mut headers = Vec::with_capacity(module.instruments.len());
    for (instrument_index, instrument) in module.instruments.iter().enumerate() {
        let instrument_start = offset;
        let instrument_header_size = usize::try_from(read_u32_at(original_data, offset)?)
            .map_err(|_| XmError::UnexpectedEof)?;
        let source_num_samples = usize::from(read_u16_at(
            original_data,
            offset.checked_add(27).ok_or(XmError::UnexpectedEof)?,
        )?);
        if source_num_samples != usize::from(instrument.num_samples) {
            return Err(XmError::InvalidInstrument(instrument_index as u16));
        }

        let instrument_end = instrument_start
            .checked_add(instrument_header_size)
            .ok_or(XmError::UnexpectedEof)?;
        if source_num_samples == 0 {
            if instrument_end > original_data.len() {
                return Err(XmError::UnexpectedEof);
            }
            offset = instrument_end;
            headers.push(Vec::new());
            continue;
        }

        let sample_header_size = usize::try_from(read_u32_at(
            original_data,
            instrument_start
                .checked_add(29)
                .ok_or(XmError::UnexpectedEof)?,
        )?)
        .map_err(|_| XmError::UnexpectedEof)?;
        if sample_header_size < 40 {
            return Err(XmError::InvalidHeaderSize);
        }

        let sample_headers_end = instrument_end
            .checked_add(
                sample_header_size
                    .checked_mul(source_num_samples)
                    .ok_or(XmError::UnexpectedEof)?,
            )
            .ok_or(XmError::UnexpectedEof)?;
        let mut instrument_headers = Vec::with_capacity(source_num_samples);
        let mut sample_data_bytes = 0usize;
        for sample_index in 0..source_num_samples {
            let sample_start = instrument_end
                .checked_add(
                    sample_header_size
                        .checked_mul(sample_index)
                        .ok_or(XmError::UnexpectedEof)?,
                )
                .ok_or(XmError::UnexpectedEof)?;
            let sample_header: [u8; 40] = original_data
                .get(sample_start..sample_start.checked_add(40).ok_or(XmError::UnexpectedEof)?)
                .ok_or(XmError::UnexpectedEof)?
                .try_into()
                .unwrap();
            instrument_headers.push(sample_header);
            sample_data_bytes = sample_data_bytes
                .checked_add(usize::try_from(u32::from_le_bytes(sample_header[..4].try_into().unwrap())).map_err(|_| XmError::UnexpectedEof)?)
                .ok_or(XmError::UnexpectedEof)?;
        }

        offset = sample_headers_end
            .checked_add(sample_data_bytes)
            .ok_or(XmError::UnexpectedEof)?;
        if offset > original_data.len() {
            return Err(XmError::UnexpectedEof);
        }
        headers.push(instrument_headers);
    }

    Ok((headers, offset))
}

/// Rebuild an XM file without sample data
///
/// This creates a new XM file with the same structure but with all sample data removed.
/// Sample lengths are set to 0, making the file much smaller while remaining valid XM format.
pub(crate) fn rebuild_xm_without_samples(
    original_data: &[u8],
    module: &XmModule,
) -> Result<Vec<u8>, XmError> {
    let (source_sample_headers, source_tail) = original_sample_headers(original_data, module)?;
    let mut output = Vec::with_capacity(original_data.len() / 4); // Estimate smaller size

    // Helper to write little-endian values
    let write_u8 = |out: &mut Vec<u8>, val: u8| out.push(val);
    let write_u16 = |out: &mut Vec<u8>, val: u16| out.extend_from_slice(&val.to_le_bytes());
    let write_u32 = |out: &mut Vec<u8>, val: u32| out.extend_from_slice(&val.to_le_bytes());
    let write_bytes = |out: &mut Vec<u8>, bytes: &[u8]| out.extend_from_slice(bytes);

    // ========== Write XM Header ==========

    // Magic (17 bytes)
    write_bytes(&mut output, XM_MAGIC);

    // Module name (20 bytes) - copy from original
    write_bytes(&mut output, &original_data[17..37]);

    // 0x1A marker (1 byte)
    write_u8(&mut output, 0x1A);

    // Tracker name (20 bytes) - copy from original
    write_bytes(&mut output, &original_data[38..58]);

    // Version (2 bytes)
    write_u16(&mut output, XM_VERSION);

    // Header size (4 bytes) - per XM spec, includes this 4-byte field itself
    // 276 = 4 (header_size) + 2 (song_length) + 2 (restart) + 2 (channels) + 2 (patterns)
    //     + 2 (instruments) + 2 (flags) + 2 (speed) + 2 (bpm) + 256 (order_table)
    write_u32(&mut output, 276);

    // Song length (2 bytes)
    write_u16(&mut output, module.song_length);

    // Restart position (2 bytes)
    write_u16(&mut output, module.restart_position);

    // Number of channels (2 bytes)
    write_u16(&mut output, module.num_channels as u16);

    // Number of patterns (2 bytes)
    write_u16(&mut output, module.num_patterns);

    // Number of instruments (2 bytes)
    write_u16(&mut output, module.num_instruments);

    // Flags (2 bytes)
    let flags = if module.linear_frequency_table { 1 } else { 0 };
    write_u16(&mut output, flags);

    // Default speed (2 bytes)
    write_u16(&mut output, module.default_speed);

    // Default BPM (2 bytes)
    write_u16(&mut output, module.default_bpm);

    // Pattern order table (256 bytes)
    for i in 0..256 {
        if i < module.order_table.len() {
            write_u8(&mut output, module.order_table[i]);
        } else {
            write_u8(&mut output, 0);
        }
    }

    // ========== Write Pattern Data ==========

    for pattern in &module.patterns {
        // Pattern header length (4 bytes) - per XM spec, includes the 4-byte length field itself
        // Standard value: 9 = 4 (length) + 1 (packing) + 2 (rows) + 2 (packed_size)
        write_u32(&mut output, 9);

        // Packing type (1 byte) - always 0
        write_u8(&mut output, 0);

        // Number of rows (2 bytes)
        write_u16(&mut output, pattern.num_rows);

        // We need to pack the pattern data
        let packed_pattern = pack_pattern_data(pattern, module.num_channels);

        // Packed pattern data size (2 bytes)
        write_u16(&mut output, packed_pattern.len() as u16);

        // Pattern data
        write_bytes(&mut output, &packed_pattern);
    }

    // ========== Write Instruments (WITHOUT sample data) ==========

    for (instrument_index, instrument) in module.instruments.iter().enumerate() {
        // Calculate instrument header size
        let header_size = if instrument.num_samples > 0 { 243 }
            else if module.mix_mode == crate::XmMixMode::Compatible { 29 } else { 33 };

        // Instrument header size (4 bytes)
        write_u32(&mut output, header_size);

        // Instrument name (22 bytes)
        let mut name_bytes = [0u8; 22];
        let name_str = instrument.name.as_bytes();
        let copy_len = name_str.len().min(22);
        name_bytes[..copy_len].copy_from_slice(&name_str[..copy_len]);
        write_bytes(&mut output, &name_bytes);

        // Instrument type (1 byte) - always 0
        write_u8(&mut output, 0);

        // Number of samples (2 bytes)
        write_u16(&mut output, instrument.num_samples as u16);
        // Preserve the measured empty-header mixer distinction when stripping PCM.
        if instrument.num_samples == 0 && header_size == 33 {
            write_u32(&mut output, 0);
        }

        if instrument.num_samples > 0 {
            if instrument.sample_map.len() != 96
                || instrument.sample_map.iter().any(|&sample| sample >= instrument.num_samples)
                || instrument.samples.len() != instrument.num_samples as usize
            {
                return Err(XmError::InvalidInstrument(0));
            }

            // Sample header size (4 bytes) - always 40
            write_u32(&mut output, 40);

            // Sample number for all notes (96 bytes) - retain local slot mapping.
            write_bytes(&mut output, &instrument.sample_map);

            // Volume envelope points (48 bytes)
            if let Some(ref env) = instrument.volume_envelope {
                for i in 0..12 {
                    if i < env.points.len() {
                        write_u16(&mut output, env.points[i].0);
                        write_u16(&mut output, env.points[i].1);
                    } else {
                        write_u32(&mut output, 0);
                    }
                }
            } else {
                for _ in 0..48 {
                    write_u8(&mut output, 0);
                }
            }

            // Panning envelope points (48 bytes)
            if let Some(ref env) = instrument.panning_envelope {
                for i in 0..12 {
                    if i < env.points.len() {
                        write_u16(&mut output, env.points[i].0);
                        write_u16(&mut output, env.points[i].1);
                    } else {
                        write_u32(&mut output, 0);
                    }
                }
            } else {
                for _ in 0..48 {
                    write_u8(&mut output, 0);
                }
            }

            // Number of volume envelope points (1 byte)
            let num_vol_points = instrument
                .volume_envelope
                .as_ref()
                .map_or(0, |e| e.points.len() as u8);
            write_u8(&mut output, num_vol_points);

            // Number of panning envelope points (1 byte)
            let num_pan_points = instrument
                .panning_envelope
                .as_ref()
                .map_or(0, |e| e.points.len() as u8);
            write_u8(&mut output, num_pan_points);

            // Volume sustain point (1 byte)
            let vol_sustain = instrument
                .volume_envelope
                .as_ref()
                .map_or(0, |e| e.sustain_point);
            write_u8(&mut output, vol_sustain);

            // Volume loop start (1 byte)
            let vol_loop_start = instrument
                .volume_envelope
                .as_ref()
                .map_or(0, |e| e.loop_start);
            write_u8(&mut output, vol_loop_start);

            // Volume loop end (1 byte)
            let vol_loop_end = instrument
                .volume_envelope
                .as_ref()
                .map_or(0, |e| e.loop_end);
            write_u8(&mut output, vol_loop_end);

            // Panning sustain point (1 byte)
            let pan_sustain = instrument
                .panning_envelope
                .as_ref()
                .map_or(0, |e| e.sustain_point);
            write_u8(&mut output, pan_sustain);

            // Panning loop start (1 byte)
            let pan_loop_start = instrument
                .panning_envelope
                .as_ref()
                .map_or(0, |e| e.loop_start);
            write_u8(&mut output, pan_loop_start);

            // Panning loop end (1 byte)
            let pan_loop_end = instrument
                .panning_envelope
                .as_ref()
                .map_or(0, |e| e.loop_end);
            write_u8(&mut output, pan_loop_end);

            // Volume type (1 byte)
            let vol_type = if let Some(ref env) = instrument.volume_envelope {
                let mut flags = if env.enabled { 1 } else { 0 };
                if env.sustain_enabled {
                    flags |= 2;
                }
                if env.loop_enabled {
                    flags |= 4;
                }
                flags
            } else {
                0
            };
            write_u8(&mut output, vol_type);

            // Panning type (1 byte)
            let pan_type = if let Some(ref env) = instrument.panning_envelope {
                let mut flags = if env.enabled { 1 } else { 0 };
                if env.sustain_enabled {
                    flags |= 2;
                }
                if env.loop_enabled {
                    flags |= 4;
                }
                flags
            } else {
                0
            };
            write_u8(&mut output, pan_type);

            // Vibrato type (1 byte)
            write_u8(&mut output, instrument.vibrato_type);

            // Vibrato sweep (1 byte)
            write_u8(&mut output, instrument.vibrato_sweep);

            // Vibrato depth (1 byte)
            write_u8(&mut output, instrument.vibrato_depth);

            // Vibrato rate (1 byte)
            write_u8(&mut output, instrument.vibrato_rate);

            // Volume fadeout (2 bytes)
            write_u16(&mut output, instrument.volume_fadeout);

            // Reserved (2 bytes)
            write_u16(&mut output, 0);

            // ========== Sample Headers (WITH sample_length = 0) ==========

            for sample_header in &source_sample_headers[instrument_index] {
                // Preserve all original sample metadata, but strip the PCM length.
                let mut stripped_header = *sample_header;
                stripped_header[..4].fill(0);
                write_bytes(&mut output, &stripped_header);
            }
        }
    }

    // Keep any existing mixer extensions and other trailing XM chunks intact.
    write_bytes(&mut output, &original_data[source_tail..]);
    Ok(output)
}

/// Pack pattern data into XM format using compressed packed format
///
/// Uses the XM packed format where empty notes are compressed to 1 byte (0x80).
/// This typically achieves 60-70% compression compared to unpacked format.
///
/// Packed format:
/// - If note is all zeros: single byte 0x80
/// - Otherwise: flag byte (0x80 | field_flags) followed by present fields
///   - Bit 0 (0x01): Note present
///   - Bit 1 (0x02): Instrument present
///   - Bit 2 (0x04): Volume present
///   - Bit 3 (0x08): Effect present
///   - Bit 4 (0x10): Effect param present
pub(crate) fn pack_pattern_data(pattern: &XmPattern, num_channels: u8) -> Vec<u8> {
    let mut output = Vec::new();

    for row in &pattern.notes {
        for (ch_idx, note) in row.iter().enumerate() {
            if ch_idx >= num_channels as usize {
                break;
            }

            // Check if note is completely empty
            if note.note == 0
                && note.instrument == 0
                && note.volume == 0
                && note.effect == 0
                && note.effect_param == 0
            {
                // Empty note: just the packed marker
                output.push(0x80);
                continue;
            }

            // Build flag byte indicating which fields are present
            let mut flags = 0x80u8; // Packed format marker

            if note.note != 0 {
                flags |= 0x01;
            }
            if note.instrument != 0 {
                flags |= 0x02;
            }
            if note.volume != 0 {
                flags |= 0x04;
            }
            if note.effect != 0 {
                flags |= 0x08;
            }
            if note.effect_param != 0 {
                flags |= 0x10;
            }

            // Write flag byte
            output.push(flags);

            // Write only the present fields
            if note.note != 0 {
                output.push(note.note);
            }
            if note.instrument != 0 {
                output.push(note.instrument);
            }
            if note.volume != 0 {
                output.push(note.volume);
            }
            if note.effect != 0 {
                output.push(note.effect);
            }
            if note.effect_param != 0 {
                output.push(note.effect_param);
            }
        }
    }

    output
}

/// Strip sample data from an XM file, keeping only patterns and instrument metadata
///
/// This creates a minimal XM file that can be stored in the ROM with much smaller size.
/// Sample data is loaded separately via the ROM data pack.
///
/// The resulting XM file:
/// - Keeps all pattern data (note sequences)
/// - Keeps instrument names and envelopes (for ROM sound mapping)
/// - Sets all sample lengths to 0 (no audio data embedded)
/// - Remains valid XM format that can be parsed
///
/// This typically reduces file size by 60-80%.
pub fn strip_xm_samples(data: &[u8]) -> Result<Vec<u8>, XmError> {
    // Parse and validate the XM first
    let module = super::read::parse_xm(data)?;

    // Rebuild XM without sample data
    rebuild_xm_without_samples(data, &module)
}
