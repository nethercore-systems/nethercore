//! XM packing functions - converts XmModule to minimal binary format

use std::io::Write;

use crate::error::XmError;
use crate::module::{XmEnvelope, XmModule};
use crate::parser::pack_pattern_data;

use super::MAX_ENVELOPE_POINTS;
use super::io::{write_u16, write_u32};

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
