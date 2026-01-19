//! Instrument and envelope parsing

use std::io::{Cursor, Read, Seek, SeekFrom};

use crate::error::ItError;
use crate::module::{
    DuplicateCheckAction, DuplicateCheckType, ItEnvelope, ItEnvelopeFlags, ItInstrument,
    NewNoteAction,
};
use crate::{INSTRUMENT_MAGIC, MAX_ENVELOPE_POINTS};

use super::helpers::{read_string, read_u8, read_u16};

/// Parse a single instrument
pub(crate) fn parse_instrument(
    cursor: &mut Cursor<&[u8]>,
    compatible_with: u16,
) -> Result<ItInstrument, ItError> {
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
pub(crate) fn parse_envelope(cursor: &mut Cursor<&[u8]>) -> Result<Option<ItEnvelope>, ItError> {
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
