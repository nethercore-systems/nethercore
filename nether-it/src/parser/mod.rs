//! IT file parser

use std::io::{Cursor, Read, Seek, SeekFrom};

use crate::error::ItError;
use crate::module::{ItFlags, ItInstrument, ItModule, ItPattern, ItSample};
use crate::{
    IT_MAGIC, MAX_CHANNELS, MAX_INSTRUMENTS, MAX_PATTERNS, MAX_SAMPLES, MIN_COMPATIBLE_VERSION,
};

mod helpers;
mod instrument;
mod pattern;
mod sample;
#[cfg(test)]
mod tests;

use helpers::{read_string, read_u8, read_u16, read_u32};
use instrument::parse_instrument;
use pattern::parse_pattern;

// Re-export public APIs
pub use sample::{SampleData, SampleInfo, load_sample_data, parse_sample};

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
    let num_channels = channel_pan.iter().take_while(|&&p| p < 128).count().max(1) as u8;

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
        let instrument = parse_instrument(&mut cursor, compatible_with)
            .map_err(|_| ItError::InvalidInstrument(idx as u16))?;
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
        let sample_info =
            parse_sample(&mut cursor).map_err(|_| ItError::InvalidSample(idx as u16))?;
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
