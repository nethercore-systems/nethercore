//! IT file parser

use std::io::{Cursor, Read, Seek, SeekFrom};

// Compression functions for loading sample data from IT files
use crate::compression::{
    decompress_it215_16bit, decompress_it215_16bit_with_size, decompress_it215_8bit,
    decompress_it215_8bit_with_size,
};
use crate::error::ItError;
use crate::module::{
    DuplicateCheckAction, DuplicateCheckType, ItEnvelope, ItEnvelopeFlags, ItFlags, ItInstrument,
    ItModule, ItNote, ItPattern, ItSample, ItSampleFlags, NewNoteAction,
};
use crate::{
    INSTRUMENT_MAGIC, IT_MAGIC, MAX_CHANNELS, MAX_ENVELOPE_POINTS, MAX_INSTRUMENTS,
    MAX_PATTERN_ROWS, MAX_PATTERNS, MAX_SAMPLES, MIN_COMPATIBLE_VERSION, SAMPLE_MAGIC,
};

/// Parse an IT file into an ItModule
///
/// This extracts pattern data and instrument/sample metadata. Sample audio data
/// is ignored as it will be loaded from the ROM data pack.
///
/// # Arguments
/// * `data` - Raw IT file bytes
///
/// # Returns
/// * `Ok(ItModule)` - Parsed module
/// * `Err(ItError)` - Parse error
pub fn parse_it(data: &[u8]) -> Result<ItModule, ItError> {
    // Minimum header size: magic (4) + name (26) + ... = 192 bytes
    if data.len() < 192 {
        return Err(ItError::TooSmall);
    }

    // Validate magic "IMPM"
    if &data[0..4] != IT_MAGIC {
        return Err(ItError::InvalidMagic);
    }

    let mut cursor = Cursor::new(data);

    // Skip magic (4 bytes)
    cursor.seek(SeekFrom::Start(4))?;

    // Read song name (26 bytes, null-terminated)
    let mut name_bytes = [0u8; 26];
    cursor.read_exact(&mut name_bytes)?;
    let name = read_string(&name_bytes);

    // Pattern row highlight (2 bytes) - skip
    cursor.seek(SeekFrom::Current(2))?;

    // OrdNum (2 bytes)
    let num_orders = read_u16(&mut cursor)?;

    // InsNum (2 bytes)
    let num_instruments = read_u16(&mut cursor)?;
    if num_instruments > MAX_INSTRUMENTS {
        return Err(ItError::TooManyInstruments(num_instruments));
    }

    // SmpNum (2 bytes)
    let num_samples = read_u16(&mut cursor)?;
    if num_samples > MAX_SAMPLES {
        return Err(ItError::TooManySamples(num_samples));
    }

    // PatNum (2 bytes)
    let num_patterns = read_u16(&mut cursor)?;
    if num_patterns > MAX_PATTERNS {
        return Err(ItError::TooManyPatterns(num_patterns));
    }

    // Cwt/v (2 bytes) - Created with tracker version
    let created_with = read_u16(&mut cursor)?;

    // Cmwt (2 bytes) - Compatible with version
    let compatible_with = read_u16(&mut cursor)?;
    if compatible_with < MIN_COMPATIBLE_VERSION {
        return Err(ItError::UnsupportedVersion(compatible_with));
    }

    // Flags (2 bytes)
    let flags = ItFlags::from_bits(read_u16(&mut cursor)?);

    // Special (2 bytes)
    let special = read_u16(&mut cursor)?;

    // GV - Global volume (1 byte)
    let global_volume = read_u8(&mut cursor)?;

    // MV - Mix volume (1 byte)
    let mix_volume = read_u8(&mut cursor)?;

    // IS - Initial speed (1 byte)
    let initial_speed = read_u8(&mut cursor)?;

    // IT - Initial tempo (1 byte)
    let initial_tempo = read_u8(&mut cursor)?;

    // Sep - Panning separation (1 byte)
    let panning_separation = read_u8(&mut cursor)?;

    // PWD - Pitch wheel depth (1 byte)
    let pitch_wheel_depth = read_u8(&mut cursor)?;

    // MsgLgth (2 bytes)
    let message_length = read_u16(&mut cursor)?;

    // MsgOff (4 bytes)
    let message_offset = read_u32(&mut cursor)?;

    // Reserved (4 bytes) - skip
    cursor.seek(SeekFrom::Current(4))?;

    // Channel pan (64 bytes)
    let mut channel_pan = [0u8; 64];
    cursor.read_exact(&mut channel_pan)?;

    // Channel volume (64 bytes)
    let mut channel_vol = [0u8; 64];
    cursor.read_exact(&mut channel_vol)?;

    // Calculate number of used channels from channel_pan
    // Channels with pan >= 128 are disabled
    let num_channels = channel_pan
        .iter()
        .take_while(|&&p| p < 128)
        .count()
        .max(1) as u8;

    if num_channels > MAX_CHANNELS {
        return Err(ItError::TooManyChannels(num_channels));
    }

    // Read order table
    let mut order_table = vec![0u8; num_orders as usize];
    cursor.read_exact(&mut order_table)?;

    // Read offset tables
    let mut instrument_offsets = Vec::with_capacity(num_instruments as usize);
    for _ in 0..num_instruments {
        instrument_offsets.push(read_u32(&mut cursor)?);
    }

    let mut sample_offsets = Vec::with_capacity(num_samples as usize);
    for _ in 0..num_samples {
        sample_offsets.push(read_u32(&mut cursor)?);
    }

    let mut pattern_offsets = Vec::with_capacity(num_patterns as usize);
    for _ in 0..num_patterns {
        pattern_offsets.push(read_u32(&mut cursor)?);
    }

    // Parse instruments
    let mut instruments = Vec::with_capacity(num_instruments as usize);
    for (idx, &offset) in instrument_offsets.iter().enumerate() {
        if offset == 0 {
            instruments.push(ItInstrument::default());
            continue;
        }

        cursor.seek(SeekFrom::Start(offset as u64))?;
        let instrument =
            parse_instrument(&mut cursor, compatible_with).map_err(|_| ItError::InvalidInstrument(idx as u16))?;
        instruments.push(instrument);
    }

    // Parse samples
    let mut samples = Vec::with_capacity(num_samples as usize);
    for (idx, &offset) in sample_offsets.iter().enumerate() {
        if offset == 0 {
            samples.push(ItSample::default());
            continue;
        }

        cursor.seek(SeekFrom::Start(offset as u64))?;
        let sample_info = parse_sample(&mut cursor).map_err(|_| ItError::InvalidSample(idx as u16))?;
        samples.push(sample_info.sample);
    }

    // Parse patterns
    let mut patterns = Vec::with_capacity(num_patterns as usize);
    for (idx, &offset) in pattern_offsets.iter().enumerate() {
        if offset == 0 {
            // Empty pattern - create default 64-row pattern
            patterns.push(ItPattern::empty(64, num_channels));
            continue;
        }

        cursor.seek(SeekFrom::Start(offset as u64))?;
        let pattern = parse_pattern(&mut cursor, num_channels)
            .map_err(|_| ItError::InvalidPattern(idx as u16))?;
        patterns.push(pattern);
    }

    // Parse message (if present)
    let message = if (special & 1) != 0 && message_length > 0 && message_offset > 0 {
        cursor.seek(SeekFrom::Start(message_offset as u64))?;
        let mut msg_bytes = vec![0u8; message_length as usize];
        if cursor.read_exact(&mut msg_bytes).is_ok() {
            Some(read_string(&msg_bytes))
        } else {
            None
        }
    } else {
        None
    };

    Ok(ItModule {
        name,
        num_channels,
        num_orders,
        num_instruments,
        num_samples,
        num_patterns,
        created_with,
        compatible_with,
        flags,
        special,
        global_volume,
        mix_volume,
        initial_speed,
        initial_tempo,
        panning_separation,
        pitch_wheel_depth,
        channel_pan,
        channel_vol,
        order_table,
        patterns,
        instruments,
        samples,
        message,
    })
}

/// Parse a single instrument
fn parse_instrument(cursor: &mut Cursor<&[u8]>, compatible_with: u16) -> Result<ItInstrument, ItError> {
    // Read magic "IMPI"
    let mut magic = [0u8; 4];
    cursor.read_exact(&mut magic)?;
    if &magic != INSTRUMENT_MAGIC {
        return Err(ItError::InvalidInstrument(0));
    }

    // DOS filename (12 bytes)
    let mut filename_bytes = [0u8; 12];
    cursor.read_exact(&mut filename_bytes)?;
    let filename = read_string(&filename_bytes);

    // Reserved (1 byte)
    cursor.seek(SeekFrom::Current(1))?;

    // NNA, DCT, DCA
    let nna = NewNoteAction::from_u8(read_u8(cursor)?);
    let dct = DuplicateCheckType::from_u8(read_u8(cursor)?);
    let dca = DuplicateCheckAction::from_u8(read_u8(cursor)?);

    // Fadeout (2 bytes)
    let fadeout = read_u16(cursor)?;

    // PPS, PPC
    let pitch_pan_separation = read_u8(cursor)? as i8;
    let pitch_pan_center = read_u8(cursor)?;

    // GbV, DfP
    let global_volume = read_u8(cursor)?;
    let dfp = read_u8(cursor)?;
    let default_pan = if dfp & 0x80 != 0 {
        Some(dfp & 0x7F)
    } else {
        None
    };

    // RV, RP (random variation)
    let random_volume = read_u8(cursor)?;
    let random_pan = read_u8(cursor)?;

    // TrkVers, NoS (for instrument files only) - skip 4 bytes
    cursor.seek(SeekFrom::Current(4))?;

    // Instrument name (26 bytes)
    let mut name_bytes = [0u8; 26];
    cursor.read_exact(&mut name_bytes)?;
    let name = read_string(&name_bytes);

    // IFC, IFR (initial filter cutoff/resonance)
    let ifc = read_u8(cursor)?;
    let ifr = read_u8(cursor)?;
    let filter_cutoff = if ifc & 0x80 != 0 {
        Some(ifc & 0x7F)
    } else {
        None
    };
    let filter_resonance = if ifr & 0x80 != 0 {
        Some(ifr & 0x7F)
    } else {
        None
    };

    // MCh, MPr, MIDIBnk
    let midi_channel = read_u8(cursor)?;
    let midi_program = read_u8(cursor)?;
    let midi_bank = read_u16(cursor)?;

    // Note-Sample-Keyboard table (240 bytes = 120 × 2)
    let mut note_sample_table = [(0u8, 0u8); 120];
    for entry in note_sample_table.iter_mut() {
        let note = read_u8(cursor)?;
        let sample = read_u8(cursor)?;
        *entry = (note, sample);
    }

    // Envelopes (only for compatible_with >= 0x0200)
    let (volume_envelope, panning_envelope, pitch_envelope) = if compatible_with >= 0x0200 {
        let vol_env = parse_envelope(cursor)?;
        let pan_env = parse_envelope(cursor)?;
        let pitch_env = parse_envelope(cursor)?;
        (vol_env, pan_env, pitch_env)
    } else {
        (None, None, None)
    };

    Ok(ItInstrument {
        name,
        filename,
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
        midi_channel,
        midi_program,
        midi_bank,
    })
}

/// Parse an envelope
fn parse_envelope(cursor: &mut Cursor<&[u8]>) -> Result<Option<ItEnvelope>, ItError> {
    // Flags (1 byte)
    let flags = ItEnvelopeFlags::from_bits(read_u8(cursor)?);

    // Num (1 byte) - number of node points
    let num_points = read_u8(cursor)? as usize;

    // LpB, LpE (loop begin/end)
    let loop_begin = read_u8(cursor)?;
    let loop_end = read_u8(cursor)?;

    // SLB, SLE (sustain loop begin/end)
    let sustain_begin = read_u8(cursor)?;
    let sustain_end = read_u8(cursor)?;

    // Node data (75 bytes = 25 × 3: 1 byte y-value + 2 bytes tick)
    let mut points = Vec::with_capacity(num_points.min(MAX_ENVELOPE_POINTS));
    for _ in 0..MAX_ENVELOPE_POINTS {
        let y_value = read_u8(cursor)? as i8;
        let tick = read_u16(cursor)?;
        if points.len() < num_points {
            points.push((tick, y_value));
        }
    }

    // Reserved (1 byte)
    cursor.seek(SeekFrom::Current(1))?;

    if num_points == 0 || !flags.contains(ItEnvelopeFlags::ENABLED) {
        return Ok(None);
    }

    Ok(Some(ItEnvelope {
        points,
        loop_begin,
        loop_end,
        sustain_begin,
        sustain_end,
        flags,
    }))
}

/// Parse a single sample header
pub fn parse_sample(cursor: &mut Cursor<&[u8]>) -> Result<SampleInfo, ItError> {
    // Read magic "IMPS"
    let mut magic = [0u8; 4];
    cursor.read_exact(&mut magic)?;
    if &magic != SAMPLE_MAGIC {
        return Err(ItError::InvalidSample(0));
    }

    // DOS filename (12 bytes)
    let mut filename_bytes = [0u8; 12];
    cursor.read_exact(&mut filename_bytes)?;
    let filename = read_string(&filename_bytes);

    // Reserved (1 byte)
    cursor.seek(SeekFrom::Current(1))?;

    // GvL (global volume)
    let global_volume = read_u8(cursor)?;

    // Flg (flags)
    let flags = ItSampleFlags::from_bits(read_u8(cursor)?);

    // Vol (default volume)
    let default_volume = read_u8(cursor)?;

    // Sample name (26 bytes)
    let mut name_bytes = [0u8; 26];
    cursor.read_exact(&mut name_bytes)?;
    let name = read_string(&name_bytes);

    // Cvt (convert flags) - skip
    let _cvt = read_u8(cursor)?;

    // DfP (default pan)
    let dfp = read_u8(cursor)?;
    let default_pan = if dfp & 0x80 != 0 {
        Some(dfp & 0x7F)
    } else {
        None
    };

    // Length (4 bytes)
    let length = read_u32(cursor)?;

    // LoopBeg (4 bytes)
    let loop_begin = read_u32(cursor)?;

    // LoopEnd (4 bytes)
    let loop_end = read_u32(cursor)?;

    // C5Speed (4 bytes)
    let c5_speed = read_u32(cursor)?;

    // SusLBeg (4 bytes)
    let sustain_loop_begin = read_u32(cursor)?;

    // SusLEnd (4 bytes)
    let sustain_loop_end = read_u32(cursor)?;

    // SmpPoint (4 bytes) - offset to sample data
    let data_offset = read_u32(cursor)?;

    // ViS, ViD, ViR, ViT (vibrato)
    let vibrato_speed = read_u8(cursor)?;
    let vibrato_depth = read_u8(cursor)?;
    let vibrato_rate = read_u8(cursor)?;
    let vibrato_type = read_u8(cursor)?;

    Ok(SampleInfo {
        sample: ItSample {
            name,
            filename,
            global_volume,
            flags,
            default_volume,
            default_pan,
            length,
            loop_begin,
            loop_end,
            c5_speed,
            sustain_loop_begin,
            sustain_loop_end,
            vibrato_speed,
            vibrato_depth,
            vibrato_rate,
            vibrato_type,
        },
        data_offset,
    })
}

/// Parse a single pattern
fn parse_pattern(cursor: &mut Cursor<&[u8]>, num_channels: u8) -> Result<ItPattern, ItError> {
    // Pattern header
    // Length (2 bytes) - packed data size (excluding 8-byte header)
    let packed_length = read_u16(cursor)?;

    // Rows (2 bytes)
    let num_rows = read_u16(cursor)?;
    if num_rows == 0 || num_rows > MAX_PATTERN_ROWS {
        return Err(ItError::InvalidPattern(0));
    }

    // Reserved (4 bytes)
    cursor.seek(SeekFrom::Current(4))?;

    // Allocate pattern
    let mut notes = Vec::with_capacity(num_rows as usize);
    for _ in 0..num_rows {
        notes.push(vec![ItNote::default(); num_channels as usize]);
    }

    if packed_length == 0 {
        // Empty pattern
        return Ok(ItPattern { num_rows, notes });
    }

    // Read packed data
    let pattern_start = cursor.position();

    // Per-channel previous values for pattern compression
    let mut prev_mask = [0u8; 64];
    let mut prev_note = [0u8; 64];
    let mut prev_instrument = [0u8; 64];
    let mut prev_volume = [0u8; 64];
    let mut prev_effect = [0u8; 64];
    let mut prev_effect_param = [0u8; 64];

    let mut row = 0;
    while row < num_rows && cursor.position() < pattern_start + packed_length as u64 {
        // Read channel marker
        let channel_marker = read_u8(cursor)?;

        if channel_marker == 0 {
            // End of row
            row += 1;
            continue;
        }

        // Extract channel number (bits 0-5)
        let channel = (channel_marker & 0x3F) as usize;
        if channel >= num_channels as usize {
            // Skip this note - channel out of range
            // Still need to read the data though
            let mask = if channel_marker & 0x80 != 0 {
                read_u8(cursor)?
            } else {
                prev_mask.get(channel).copied().unwrap_or(0)
            };

            // Skip data based on mask
            if mask & 0x01 != 0 {
                let _ = read_u8(cursor)?;
            }
            if mask & 0x02 != 0 {
                let _ = read_u8(cursor)?;
            }
            if mask & 0x04 != 0 {
                let _ = read_u8(cursor)?;
            }
            if mask & 0x08 != 0 {
                let _ = read_u8(cursor)?;
                let _ = read_u8(cursor)?;
            }
            continue;
        }

        // Get mask
        let mask = if channel_marker & 0x80 != 0 {
            let m = read_u8(cursor)?;
            prev_mask[channel] = m;
            m
        } else {
            prev_mask[channel]
        };

        let note = &mut notes[row as usize][channel];

        // Read/use note
        if mask & 0x01 != 0 {
            let n = read_u8(cursor)?;
            prev_note[channel] = n;
            note.note = n;
        } else if mask & 0x10 != 0 {
            note.note = prev_note[channel];
        }

        // Read/use instrument
        if mask & 0x02 != 0 {
            let i = read_u8(cursor)?;
            prev_instrument[channel] = i;
            note.instrument = i;
        } else if mask & 0x20 != 0 {
            note.instrument = prev_instrument[channel];
        }

        // Read/use volume
        if mask & 0x04 != 0 {
            let v = read_u8(cursor)?;
            prev_volume[channel] = v;
            note.volume = v;
        } else if mask & 0x40 != 0 {
            note.volume = prev_volume[channel];
        }

        // Read/use effect
        if mask & 0x08 != 0 {
            let e = read_u8(cursor)?;
            let p = read_u8(cursor)?;
            prev_effect[channel] = e;
            prev_effect_param[channel] = p;
            note.effect = e;
            note.effect_param = p;
        } else if mask & 0x80 != 0 {
            note.effect = prev_effect[channel];
            note.effect_param = prev_effect_param[channel];
        }
    }

    Ok(ItPattern { num_rows, notes })
}

/// Get list of instrument names from an IT file (for sample ID mapping)
pub fn get_instrument_names(data: &[u8]) -> Result<Vec<String>, ItError> {
    let module = parse_it(data)?;
    Ok(module.instruments.iter().map(|i| i.name.clone()).collect())
}

/// Get list of sample names from an IT file
pub fn get_sample_names(data: &[u8]) -> Result<Vec<String>, ItError> {
    let module = parse_it(data)?;
    Ok(module.samples.iter().map(|s| s.name.clone()).collect())
}

// =============================================================================
// Helper functions for reading data
// =============================================================================

/// Load sample data from an IT file, automatically decompressing if needed
///
/// This function is useful for extracting sample data from IT files,
/// particularly when samples use IT215 compression. The nether-pack tool
/// strips samples during ROM packing, but this function is available for
/// tools that need to access the original sample data.
///
/// # Arguments
/// * `data` - Complete IT file bytes
/// * `sample_offset` - Offset to sample data (from sample header)
/// * `sample` - Sample header information (contains flags, length)
///
/// # Returns
/// * 8-bit samples: `Ok(SampleData::I8(Vec<i8>))`
/// * 16-bit samples: `Ok(SampleData::I16(Vec<i16>))`
pub fn load_sample_data(
    data: &[u8],
    sample_offset: u32,
    sample: &ItSample,
) -> Result<SampleData, ItError> {
    if sample_offset == 0 || sample.length == 0 {
        // No sample data
        return if sample.flags.contains(ItSampleFlags::SAMPLE_16BIT) {
            Ok(SampleData::I16(Vec::new()))
        } else {
            Ok(SampleData::I8(Vec::new()))
        };
    }

    let offset = sample_offset as usize;
    if offset >= data.len() {
        return Err(ItError::InvalidSample(0));
    }

    let is_16bit = sample.flags.contains(ItSampleFlags::SAMPLE_16BIT);
    let is_compressed = sample.flags.contains(ItSampleFlags::COMPRESSED);
    let is_stereo = sample.flags.contains(ItSampleFlags::STEREO);

    // For stereo, IT stores Left channel first, then Right channel (sequential, not interleaved)
    // We read both channels and interleave them for compatibility with standard audio processing
    let channel_count = if is_stereo { 2 } else { 1 };
    let samples_per_channel = sample.length as usize;
    let total_samples = samples_per_channel * channel_count;

    if is_compressed {
        // IT215 compression - for stereo, each channel is compressed separately
        let compressed_data = &data[offset..];

        if is_16bit {
            if is_stereo {
                // Decompress left channel and get bytes consumed to find right channel offset
                let (left, left_bytes_consumed) =
                    decompress_it215_16bit_with_size(compressed_data, samples_per_channel)?;

                // Right channel starts after left channel's compressed data
                let right_offset = left_bytes_consumed;
                let right = if right_offset < compressed_data.len() {
                    decompress_it215_16bit(&compressed_data[right_offset..], samples_per_channel)?
                } else {
                    // No right channel data available, fill with zeros
                    vec![0i16; samples_per_channel]
                };

                // Interleave left and right channels
                let mut interleaved = Vec::with_capacity(total_samples);
                for i in 0..samples_per_channel {
                    interleaved.push(left.get(i).copied().unwrap_or(0));
                    interleaved.push(right.get(i).copied().unwrap_or(0));
                }
                Ok(SampleData::I16(interleaved))
            } else {
                let samples = decompress_it215_16bit(compressed_data, samples_per_channel)?;
                Ok(SampleData::I16(samples))
            }
        } else if is_stereo {
            // Decompress left channel and get bytes consumed to find right channel offset
            let (left, left_bytes_consumed) =
                decompress_it215_8bit_with_size(compressed_data, samples_per_channel)?;

            // Right channel starts after left channel's compressed data
            let right_offset = left_bytes_consumed;
            let right = if right_offset < compressed_data.len() {
                decompress_it215_8bit(&compressed_data[right_offset..], samples_per_channel)?
            } else {
                // No right channel data available, fill with zeros
                vec![0i8; samples_per_channel]
            };

            // Interleave left and right channels
            let mut interleaved = Vec::with_capacity(total_samples);
            for i in 0..samples_per_channel {
                interleaved.push(left.get(i).copied().unwrap_or(0));
                interleaved.push(right.get(i).copied().unwrap_or(0));
            }
            Ok(SampleData::I8(interleaved))
        } else {
            let samples = decompress_it215_8bit(compressed_data, samples_per_channel)?;
            Ok(SampleData::I8(samples))
        }
    } else {
        // Uncompressed sample data
        let bytes_per_sample = if is_16bit { 2 } else { 1 };
        let total_size = total_samples * bytes_per_sample;

        if offset + total_size > data.len() {
            return Err(ItError::InvalidSample(0));
        }

        if is_16bit {
            if is_stereo {
                // Read left channel, then right channel, then interleave
                let mut interleaved = Vec::with_capacity(total_samples);
                for i in 0..samples_per_channel {
                    // Left sample
                    let left_idx = offset + i * 2;
                    let left = i16::from_le_bytes([data[left_idx], data[left_idx + 1]]);
                    // Right sample (offset by samples_per_channel * 2 bytes)
                    let right_idx = offset + (samples_per_channel + i) * 2;
                    let right = i16::from_le_bytes([data[right_idx], data[right_idx + 1]]);
                    interleaved.push(left);
                    interleaved.push(right);
                }
                Ok(SampleData::I16(interleaved))
            } else {
                let mut samples = Vec::with_capacity(samples_per_channel);
                for i in 0..samples_per_channel {
                    let idx = offset + i * 2;
                    let sample_val = i16::from_le_bytes([data[idx], data[idx + 1]]);
                    samples.push(sample_val);
                }
                Ok(SampleData::I16(samples))
            }
        } else if is_stereo {
            // Read left channel, then right channel, then interleave
            let mut interleaved = Vec::with_capacity(total_samples);
            for i in 0..samples_per_channel {
                let left = data[offset + i] as i8;
                let right = data[offset + samples_per_channel + i] as i8;
                interleaved.push(left);
                interleaved.push(right);
            }
            Ok(SampleData::I8(interleaved))
        } else {
            let samples: Vec<i8> = data[offset..offset + samples_per_channel]
                .iter()
                .map(|&b| b as i8)
                .collect();
            Ok(SampleData::I8(samples))
        }
    }
}

/// Represents sample data loaded from an IT file
#[derive(Debug, Clone)]
pub enum SampleData {
    /// 8-bit signed samples
    I8(Vec<i8>),
    /// 16-bit signed samples
    I16(Vec<i16>),
}

/// Sample metadata including offset for loading sample data
#[derive(Debug, Clone)]
pub struct SampleInfo {
    /// Sample header information
    pub sample: ItSample,
    /// Offset to sample data in the IT file
    pub data_offset: u32,
}

// =============================================================================
// Helper Functions
// =============================================================================

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

fn read_string(bytes: &[u8]) -> String {
    // Find null terminator or end of slice
    let len = bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len());
    // Trim trailing spaces and convert
    String::from_utf8_lossy(&bytes[..len])
        .trim_end()
        .to_string()
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_string() {
        assert_eq!(read_string(b"Hello\0World"), "Hello");
        assert_eq!(read_string(b"No null"), "No null");
        assert_eq!(read_string(b"Trailing   "), "Trailing");
        assert_eq!(read_string(b""), "");
    }

    #[test]
    fn test_parse_invalid_magic() {
        // Need at least 192 bytes for the header check to pass size validation
        let mut data = vec![0u8; 192];
        data[..4].copy_from_slice(b"XXXX"); // Invalid magic
        let result = parse_it(&data);
        assert!(matches!(result, Err(ItError::InvalidMagic)));
    }

    #[test]
    fn test_parse_too_small() {
        let data = b"IMPM test";
        let result = parse_it(data);
        assert!(matches!(result, Err(ItError::TooSmall)));
    }

    #[test]
    fn test_load_sample_data_uncompressed_8bit() {
        // Create a simple uncompressed 8-bit sample
        let original_samples: Vec<i8> = vec![0, 10, -10, 50, -50, 127, -128, 0];

        // Build a minimal IT file with this sample
        let mut it_data = Vec::new();

        // Create sample at offset 1000 (arbitrary)
        it_data.resize(1000, 0);
        for &sample in &original_samples {
            it_data.push(sample as u8);
        }

        // Create sample header
        let sample = ItSample {
            name: "Test".into(),
            filename: "test.raw".into(),
            length: original_samples.len() as u32,
            flags: ItSampleFlags::empty(), // Uncompressed, 8-bit
            ..Default::default()
        };

        // Load the sample data
        let result = load_sample_data(&it_data, 1000, &sample).unwrap();

        match result {
            SampleData::I8(loaded) => {
                assert_eq!(loaded.len(), original_samples.len());
                assert_eq!(loaded, original_samples);
            }
            _ => panic!("Expected I8 sample data"),
        }
    }

    #[test]
    fn test_load_sample_data_uncompressed_16bit() {
        // Create a simple uncompressed 16-bit sample
        let original_samples: Vec<i16> = vec![0, 1000, -1000, 10000, -10000, 32767, -32768, 0];

        // Build sample data
        let mut it_data = Vec::new();
        it_data.resize(1000, 0);
        for &sample in &original_samples {
            it_data.extend_from_slice(&sample.to_le_bytes());
        }

        // Create sample header
        let sample = ItSample {
            name: "Test".into(),
            filename: "test.raw".into(),
            length: original_samples.len() as u32,
            flags: ItSampleFlags::SAMPLE_16BIT, // Uncompressed, 16-bit
            ..Default::default()
        };

        // Load the sample data
        let result = load_sample_data(&it_data, 1000, &sample).unwrap();

        match result {
            SampleData::I16(loaded) => {
                assert_eq!(loaded.len(), original_samples.len());
                assert_eq!(loaded, original_samples);
            }
            _ => panic!("Expected I16 sample data"),
        }
    }

    #[test]
    fn test_load_sample_data_compressed_8bit() {
        use crate::compression::compress_it215_8bit;

        // Create a sample
        let original_samples: Vec<i8> = vec![0, 10, -10, 50, -50, 127, -128, 0];

        // Compress it
        let compressed = compress_it215_8bit(&original_samples);

        // Build IT file data
        let mut it_data = Vec::new();
        it_data.resize(1000, 0);
        it_data.extend_from_slice(&compressed);

        // Create sample header with compression flag
        let sample = ItSample {
            name: "Test".into(),
            filename: "test.it".into(),
            length: original_samples.len() as u32,
            flags: ItSampleFlags::COMPRESSED, // Compressed, 8-bit
            ..Default::default()
        };

        // Load and decompress
        let result = load_sample_data(&it_data, 1000, &sample).unwrap();

        match result {
            SampleData::I8(loaded) => {
                assert_eq!(loaded.len(), original_samples.len());
                // Note: Due to delta encoding, values should match exactly
                for (i, (&loaded_val, &orig_val)) in loaded.iter().zip(&original_samples).enumerate() {
                    assert_eq!(loaded_val, orig_val, "Mismatch at index {}", i);
                }
            }
            _ => panic!("Expected I8 sample data"),
        }
    }

    #[test]
    fn test_load_sample_data_compressed_16bit() {
        use crate::compression::compress_it215_16bit;

        // Create a sample
        let original_samples: Vec<i16> = vec![0, 1000, -1000, 10000, -10000, 32767, -32768, 0];

        // Compress it
        let compressed = compress_it215_16bit(&original_samples);

        // Build IT file data
        let mut it_data = Vec::new();
        it_data.resize(1000, 0);
        it_data.extend_from_slice(&compressed);

        // Create sample header with compression flag
        let sample = ItSample {
            name: "Test".into(),
            filename: "test.it".into(),
            length: original_samples.len() as u32,
            flags: ItSampleFlags::SAMPLE_16BIT | ItSampleFlags::COMPRESSED, // Compressed, 16-bit
            ..Default::default()
        };

        // Load and decompress
        let result = load_sample_data(&it_data, 1000, &sample).unwrap();

        match result {
            SampleData::I16(loaded) => {
                assert_eq!(loaded.len(), original_samples.len());
                // Values should match exactly
                for (i, (&loaded_val, &orig_val)) in loaded.iter().zip(&original_samples).enumerate() {
                    assert_eq!(loaded_val, orig_val, "Mismatch at index {}", i);
                }
            }
            _ => panic!("Expected I16 sample data"),
        }
    }

    #[test]
    fn test_load_sample_data_empty() {
        let it_data = vec![0u8; 1000];

        let sample = ItSample {
            name: "Empty".into(),
            length: 0, // No samples
            ..Default::default()
        };

        let result = load_sample_data(&it_data, 0, &sample).unwrap();

        match result {
            SampleData::I8(loaded) => assert_eq!(loaded.len(), 0),
            _ => panic!("Expected I8 sample data"),
        }
    }
}
