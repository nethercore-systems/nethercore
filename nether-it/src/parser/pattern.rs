//! Pattern parsing

use std::io::{Cursor, Seek, SeekFrom};

use crate::MAX_PATTERN_ROWS;
use crate::error::ItError;
use crate::module::{ItNote, ItPattern};

use super::helpers::{read_u8, read_u16};

/// Parse a single pattern
pub(crate) fn parse_pattern(
    cursor: &mut Cursor<&[u8]>,
    num_channels: u8,
) -> Result<ItPattern, ItError> {
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
