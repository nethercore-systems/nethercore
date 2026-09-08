//! XM packing functions - converts XmModule to minimal binary format

use std::io::Write;

use crate::error::XmError;
use crate::module::{XmEnvelope, XmModule};
use crate::parser::pack_pattern_data;

use super::MAX_ENVELOPE_POINTS;
use super::io::{write_u16, write_u32};
use super::{
    FLAG_FT2_MIX, FLAG_LEGACY_MIX, FLAG_LINEAR_FREQUENCY, FLAG_SAMPLE_DEFAULT_PAN, FLAG_SAMPLE_MAP,
    FLAG_SAMPLE_PREAMP,
};

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
    let has_sample_map = module
        .instruments
        .iter()
        .any(|i| i.num_samples > 1 && !i.samples.is_empty());
    let has_sample_pan = module
        .instruments
        .iter()
        .any(|instrument| instrument.num_samples == 1 && instrument.sample_default_pan.is_some());

    if has_sample_pan
        && module.instruments.iter().any(|instrument| {
            instrument.num_samples == 1 && instrument.sample_default_pan.is_none()
        })
    {
        return Err(XmError::MissingSampleDefaultPan);
    }

    // ========== Write Header ==========
    output.write_all(&[module.num_channels]).unwrap();
    write_u16(&mut output, module.num_patterns);
    write_u16(&mut output, module.num_instruments);
    write_u16(&mut output, module.song_length);
    write_u16(&mut output, module.restart_position);
    write_u16(&mut output, module.default_speed);
    write_u16(&mut output, module.default_bpm);

    // Pack flags. The pan flag changes the instrument tail layout, so it is
    // explicit rather than inferred from trailing bytes.
    let mut flags = if module.linear_frequency_table {
        FLAG_LINEAR_FREQUENCY
    } else {
        0
    };
    if has_sample_pan {
        flags |= FLAG_SAMPLE_DEFAULT_PAN;
    }
    if has_sample_map {
        flags |= FLAG_SAMPLE_MAP;
    }
    if matches!(module.mix_mode, crate::XmMixMode::Ft2) {
        flags |= FLAG_FT2_MIX;
    }
    if matches!(module.mix_mode, crate::XmMixMode::Legacy) {
        flags |= FLAG_LEGACY_MIX;
    }
    if module.legacy_retrigger {
        flags |= super::FLAG_LEGACY_RETRIGGER;
    }
    if module.sample_preamp != 48 {
        flags |= FLAG_SAMPLE_PREAMP;
    }
    output.write_all(&[flags]).unwrap();

    // Tagged preamp and reserved byte
    output
        .write_all(&[
            if flags & FLAG_SAMPLE_PREAMP != 0 {
                module.sample_preamp
            } else {
                0
            },
            0,
        ])
        .unwrap();

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
            let encoded_volume = match instrument.sample_default_volume {
                Some(volume) if volume <= 64 => {
                    if instrument.num_samples == 1 {
                        volume + 1
                    } else {
                        0
                    }
                }
                Some(volume) => return Err(XmError::InvalidSampleVolume(volume)),
                None => 0,
            };
            write_u32(&mut output, instrument.sample_loop_start);
            write_u32(&mut output, instrument.sample_loop_length);
            output
                .write_all(&[instrument.sample_finetune as u8])
                .unwrap();
            output
                .write_all(&[instrument.sample_relative_note as u8])
                .unwrap();
            output
                .write_all(&[instrument.sample_loop_type
                    | if instrument.sample_is_stereo {
                        super::SAMPLE_STEREO
                    } else {
                        0
                    }])
                .unwrap();
            output.write_all(&[encoded_volume]).unwrap();
            if has_sample_pan && instrument.num_samples == 1 {
                output
                    .write_all(&[instrument.sample_default_pan.unwrap()])
                    .unwrap();
            }
            if has_sample_map {
                if instrument.num_samples > 63
                    || instrument.samples.len() != instrument.num_samples as usize
                    || instrument.sample_map.len() != 96
                    || instrument
                        .sample_map
                        .iter()
                        .any(|&s| s >= instrument.num_samples)
                {
                    return Err(XmError::InvalidInstrument(0));
                }
                output.write_all(&instrument.sample_map).unwrap();
                for sample in &instrument.samples {
                    if sample.volume > 64 {
                        return Err(XmError::InvalidSampleVolume(sample.volume));
                    }
                    write_u32(&mut output, sample.loop_start);
                    write_u32(&mut output, sample.loop_length);
                    output
                        .write_all(&[
                            sample.finetune as u8,
                            sample.relative_note as u8,
                            sample.loop_type
                                | if sample.is_stereo {
                                    super::SAMPLE_STEREO
                                } else {
                                    0
                                },
                            sample.volume,
                            sample.pan,
                        ])
                        .unwrap();
                }
            }
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
