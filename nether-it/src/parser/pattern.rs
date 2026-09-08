//! Pattern parsing

use std::io::{Cursor, Read, Seek, SeekFrom};

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

    // Read exactly the packed data. Pattern lengths bound the pattern block;
    // never let a malformed pattern consume the following file section.
    let mut packed_data = vec![0u8; packed_length as usize];
    cursor
        .read_exact(&mut packed_data)
        .map_err(|_| ItError::UnexpectedEof)?;
    let mut packed_cursor = Cursor::new(packed_data.as_slice());

    // Per-channel previous values for pattern compression
    let mut prev_mask = [0u8; 64];
    let mut prev_note = [0u8; 64];
    let mut prev_instrument = [0u8; 64];
    let mut prev_volume = [0u8; 64];
    let mut prev_effect = [0u8; 64];
    let mut prev_effect_param = [0u8; 64];

    let mut row = 0;
    while row < num_rows {
        // Read channel marker
        let channel_marker = read_u8(&mut packed_cursor)?;

        if channel_marker == 0 {
            // End of row
            row += 1;
            continue;
        }

        // IT stores channel + 1 in the marker, so marker 64 denotes channel 63.
        let channel = channel_marker.wrapping_sub(1) as usize & 63;

        // Get mask
        let mask = if channel_marker & 0x80 != 0 {
            let m = read_u8(&mut packed_cursor)?;
            prev_mask[channel] = m;
            m
        } else {
            prev_mask[channel]
        };

        if channel >= num_channels as usize {
            // Consume data for out-of-range channels so later rows stay aligned.
            if mask & 0x01 != 0 {
                let _ = read_u8(&mut packed_cursor)?;
            }
            if mask & 0x02 != 0 {
                let _ = read_u8(&mut packed_cursor)?;
            }
            if mask & 0x04 != 0 {
                let _ = read_u8(&mut packed_cursor)?;
            }
            if mask & 0x08 != 0 {
                let _ = read_u8(&mut packed_cursor)?;
                let _ = read_u8(&mut packed_cursor)?;
            }
            continue;
        }

        let note = &mut notes[row as usize][channel];

        // Read/use note
        if mask & 0x01 != 0 {
            let n = read_u8(&mut packed_cursor)?;
            prev_note[channel] = n;
            note.note = n;
        } else if mask & 0x10 != 0 {
            note.note = prev_note[channel];
        }

        // Read/use instrument
        if mask & 0x02 != 0 {
            let i = read_u8(&mut packed_cursor)?;
            prev_instrument[channel] = i;
            note.instrument = i;
        } else if mask & 0x20 != 0 {
            note.instrument = prev_instrument[channel];
        }

        // Read/use volume
        if mask & 0x04 != 0 {
            let v = read_u8(&mut packed_cursor)?;
            prev_volume[channel] = v;
            note.volume = v;
        } else if mask & 0x40 != 0 {
            note.volume = prev_volume[channel];
        }

        // Read/use effect
        if mask & 0x08 != 0 {
            let e = read_u8(&mut packed_cursor)?;
            let p = read_u8(&mut packed_cursor)?;
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

#[cfg(test)]
mod tests {
    use super::*;

    fn pattern_bytes(packed: &[u8]) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&(packed.len() as u16).to_le_bytes());
        bytes.extend_from_slice(&1u16.to_le_bytes());
        bytes.extend_from_slice(&[0; 4]);
        bytes.extend_from_slice(packed);
        bytes
    }

    #[test]
    fn it_markers_are_one_based_for_channels_zero_and_sixty_three() {
        let data = pattern_bytes(&[0x81, 0x01, 60, 0xc0, 0x01, 61, 0]);
        let mut cursor = Cursor::new(data.as_slice());
        let pattern = parse_pattern(&mut cursor, 64).unwrap();

        assert_eq!(pattern.notes[0][0].note, 60);
        assert_eq!(pattern.notes[0][63].note, 61);
    }

    #[test]
    fn repeated_mask_uses_the_channel_previous_mask() {
        let data = {
            let packed = [0x81, 0x09, 60, 2, 0x12, 0, 0x01, 61, 3, 0x34, 0];
            let mut bytes = Vec::new();
            bytes.extend_from_slice(&(packed.len() as u16).to_le_bytes());
            bytes.extend_from_slice(&2u16.to_le_bytes());
            bytes.extend_from_slice(&[0; 4]);
            bytes.extend_from_slice(&packed);
            bytes
        };
        let mut cursor = Cursor::new(data.as_slice());
        let pattern = parse_pattern(&mut cursor, 1).unwrap();

        assert_eq!(pattern.notes[0][0].note, 60);
        assert_eq!(pattern.notes[0][0].effect, 2);
        assert_eq!(pattern.notes[1][0].note, 61);
        assert_eq!(pattern.notes[1][0].effect, 3);
        assert_eq!(pattern.notes[1][0].effect_param, 0x34);
    }

    #[test]
    fn packed_length_bounds_pattern_reads() {
        let mut data = pattern_bytes(&[0x81]);
        data.extend_from_slice(&[0x01, 60, 0]);
        let mut cursor = Cursor::new(data.as_slice());

        assert!(matches!(
            parse_pattern(&mut cursor, 1),
            Err(ItError::UnexpectedEof)
        ));
    }
}
