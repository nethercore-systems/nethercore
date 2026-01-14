//! NCIT packing functions (ItModule → NCIT binary)

use crate::module::{
    ItEnvelope, ItInstrument, ItModule, ItPattern, ItSample,
    ItSampleFlags,
};

use super::{
    INSTR_HAS_DEFAULT_PAN, INSTR_HAS_FILTER, INSTR_HAS_PAN_ENV, INSTR_HAS_PITCH_ENV,
    INSTR_HAS_VOL_ENV, MAX_ENVELOPE_POINTS, SAMPLE_HAS_LOOP, SAMPLE_HAS_PAN,
    SAMPLE_HAS_SUSTAIN, SAMPLE_HAS_VIBRATO, SAMPLE_PINGPONG_LOOP, SAMPLE_PINGPONG_SUSTAIN,
    TABLE_FULL, TABLE_SPARSE, TABLE_UNIFORM,
};
use super::{write_u16, write_u32};

/// Pack an ItModule into NCIT minimal binary format
///
/// This creates a highly optimized binary representation that strips all
/// unnecessary IT overhead. The result is typically 75-80% smaller than
/// the standard IT format (metadata only, pattern data unchanged).
///
/// # Arguments
/// * `module` - Parsed IT module to pack
///
/// # Returns
/// * Packed binary data
///
/// # Example
/// ```ignore
/// let it_data = std::fs::read("song.it")?;
/// let module = parse_it(&it_data)?;
/// let ncit = pack_ncit(&module);
/// println!("Reduced from {} to {} bytes", it_data.len(), ncit.len());
/// ```
pub fn pack_ncit(module: &ItModule) -> Vec<u8> {
    let mut output = Vec::with_capacity(4096);

    // ========== Write Header (24 bytes) ==========
    output.push(module.num_channels);
    write_u16(&mut output, module.num_orders);
    write_u16(&mut output, module.num_instruments);
    write_u16(&mut output, module.num_samples);
    write_u16(&mut output, module.num_patterns);
    output.push(module.initial_speed);
    output.push(module.initial_tempo);
    output.push(module.global_volume);
    output.push(module.mix_volume);
    write_u16(&mut output, module.flags.bits());
    output.push(module.panning_separation);
    output.extend_from_slice(&[0u8; 8]); // Reserved

    // ========== Write Order Table ==========
    output.extend_from_slice(&module.order_table[..module.num_orders as usize]);

    // ========== Write Channel Settings (for accurate playback) ==========
    // Only write num_channels worth (not full 64)
    output.extend_from_slice(&module.channel_pan[..module.num_channels as usize]);
    output.extend_from_slice(&module.channel_vol[..module.num_channels as usize]);

    // ========== Write Instruments ==========
    for instrument in &module.instruments {
        pack_instrument(&mut output, instrument);
    }

    // ========== Write Samples ==========
    for sample in &module.samples {
        pack_sample(&mut output, sample);
    }

    // ========== Write Patterns ==========
    for pattern in &module.patterns {
        pack_pattern(&mut output, pattern, module.num_channels);
    }

    output
}

/// Pack an instrument to NCIT format
pub(super) fn pack_instrument(output: &mut Vec<u8>, instr: &ItInstrument) {
    // Build flags byte
    let mut flags = 0u8;
    if instr.volume_envelope.is_some() {
        flags |= INSTR_HAS_VOL_ENV;
    }
    if instr.panning_envelope.is_some() {
        flags |= INSTR_HAS_PAN_ENV;
    }
    if instr.pitch_envelope.is_some() {
        flags |= INSTR_HAS_PITCH_ENV;
    }
    if instr.filter_cutoff.is_some() || instr.filter_resonance.is_some() {
        flags |= INSTR_HAS_FILTER;
    }
    if instr.default_pan.is_some() {
        flags |= INSTR_HAS_DEFAULT_PAN;
    }
    output.push(flags);

    // Pack NNA/DCT/DCA into single byte
    let nna_dct_dca = (instr.nna as u8 & 0x03)
        | ((instr.dct as u8 & 0x03) << 2)
        | ((instr.dca as u8 & 0x03) << 4);
    output.push(nna_dct_dca);

    // Core metadata
    write_u16(output, instr.fadeout);
    output.push(instr.global_volume);
    output.push(instr.pitch_pan_separation as u8);
    output.push(instr.pitch_pan_center);

    // Random variation (for accurate playback)
    output.push(instr.random_volume);
    output.push(instr.random_pan);

    // Optional default panning
    if instr.default_pan.is_some() {
        output.push(instr.default_pan.unwrap_or(32));
    }

    // Optional filter settings
    if flags & INSTR_HAS_FILTER != 0 {
        output.push(instr.filter_cutoff.unwrap_or(127));
        output.push(instr.filter_resonance.unwrap_or(0));
    }

    // Note-sample table (compressed)
    pack_note_sample_table(output, &instr.note_sample_table);

    // Envelopes (conditional)
    if let Some(ref env) = instr.volume_envelope {
        pack_envelope(output, env);
    }
    if let Some(ref env) = instr.panning_envelope {
        pack_envelope(output, env);
    }
    if let Some(ref env) = instr.pitch_envelope {
        pack_envelope(output, env);
    }
}

/// Analyze and pack a note-sample table with compression
pub(super) fn pack_note_sample_table(output: &mut Vec<u8>, table: &[(u8, u8); 120]) {
    // Check if uniform (all entries map to same sample with identity note mapping)
    let first_sample = table[0].1;
    let is_uniform = table
        .iter()
        .enumerate()
        .all(|(i, &(note, sample))| note == i as u8 && sample == first_sample);

    if is_uniform {
        // Type 0: Uniform - all notes use same sample
        output.push(TABLE_UNIFORM);
        output.push(first_sample);
        return;
    }

    // Check if sparse (most entries are uniform with few exceptions)
    // Find the most common (note_offset, sample) pair
    let mut counts: hashbrown::HashMap<(i8, u8), u32> = hashbrown::HashMap::new();
    for (i, (note, sample)) in table.iter().enumerate() {
        let offset = *note as i8 - i as i8;
        *counts.entry((offset, *sample)).or_insert(0) += 1;
    }

    let (most_common, count) = counts.iter().max_by_key(|(_, c)| *c).unwrap();
    let exceptions: Vec<(u8, u8, u8)> = table
        .iter()
        .enumerate()
        .filter(|(i, (note, sample))| {
            let offset = *note as i8 - *i as i8;
            (offset, *sample) != *most_common
        })
        .map(|(i, (note, sample))| (i as u8, *note, *sample))
        .collect();

    // Use sparse if:
    // 1. Most entries are uniform (>75% same pattern)
    // 2. Sparse encoding would be smaller than full
    let sparse_size = 3 + exceptions.len() * 3; // type + default_offset + default_sample + exception_count + exceptions
    let full_size = 1 + 240; // type + raw table

    if *count > 90 && sparse_size < full_size {
        // Type 1: Sparse encoding
        output.push(TABLE_SPARSE);
        output.push(most_common.0 as u8); // default note offset
        output.push(most_common.1); // default sample
        output.push(exceptions.len() as u8);

        for (index, note, sample) in exceptions {
            output.push(index);
            output.push(note);
            output.push(sample);
        }
    } else {
        // Type 2: Full table
        output.push(TABLE_FULL);
        for &(note, sample) in table {
            output.push(note);
            output.push(sample);
        }
    }
}

/// Pack an envelope to NCIT format
pub(super) fn pack_envelope(output: &mut Vec<u8>, env: &ItEnvelope) {
    let num_points = env.points.len().min(MAX_ENVELOPE_POINTS) as u8;
    output.push(num_points);
    output.push(env.loop_begin);
    output.push(env.loop_end);
    output.push(env.sustain_begin);
    output.push(env.sustain_end);
    output.push(env.flags.bits());

    // Write envelope points
    for i in 0..num_points as usize {
        let (tick, value) = env.points[i];
        write_u16(output, tick);
        output.push(value as u8);
    }
}

/// Pack a sample to NCIT format
pub(super) fn pack_sample(output: &mut Vec<u8>, sample: &ItSample) {
    // Build flags byte
    let mut flags = 0u8;
    if sample.flags.contains(ItSampleFlags::LOOP) {
        flags |= SAMPLE_HAS_LOOP;
    }
    if sample.flags.contains(ItSampleFlags::PINGPONG_LOOP) {
        flags |= SAMPLE_PINGPONG_LOOP;
    }
    if sample.flags.contains(ItSampleFlags::SUSTAIN_LOOP) {
        flags |= SAMPLE_HAS_SUSTAIN;
    }
    if sample.flags.contains(ItSampleFlags::PINGPONG_SUSTAIN) {
        flags |= SAMPLE_PINGPONG_SUSTAIN;
    }
    if sample.default_pan.is_some() {
        flags |= SAMPLE_HAS_PAN;
    }
    if sample.vibrato_speed > 0 || sample.vibrato_depth > 0 {
        flags |= SAMPLE_HAS_VIBRATO;
    }
    output.push(flags);

    // Core metadata
    output.push(sample.global_volume);
    output.push(sample.default_volume);
    write_u32(output, sample.c5_speed);

    // Optional loop points
    if flags & SAMPLE_HAS_LOOP != 0 {
        write_u32(output, sample.loop_begin);
        write_u32(output, sample.loop_end);
    }

    // Optional sustain loop points
    if flags & SAMPLE_HAS_SUSTAIN != 0 {
        write_u32(output, sample.sustain_loop_begin);
        write_u32(output, sample.sustain_loop_end);
    }

    // Optional default panning
    if flags & SAMPLE_HAS_PAN != 0 {
        output.push(sample.default_pan.unwrap_or(32));
    }

    // Optional vibrato settings
    if flags & SAMPLE_HAS_VIBRATO != 0 {
        output.push(sample.vibrato_speed);
        output.push(sample.vibrato_depth);
        output.push(sample.vibrato_rate);
        output.push(sample.vibrato_type);
    }
}

/// Pack a pattern to NCIT format (using IT's pattern packing)
pub(super) fn pack_pattern(output: &mut Vec<u8>, pattern: &ItPattern, num_channels: u8) {
    let packed = pack_pattern_data(pattern, num_channels);
    write_u16(output, pattern.num_rows);
    write_u16(output, packed.len() as u16);
    output.extend_from_slice(&packed);
}

/// Pack pattern data using IT compression
pub(super) fn pack_pattern_data(pattern: &ItPattern, num_channels: u8) -> Vec<u8> {
    let mut packed = Vec::new();

    // Previous values for compression
    let mut prev_note = [0u8; 64];
    let mut prev_instrument = [0u8; 64];
    let mut prev_volume = [0u8; 64];
    let mut prev_effect = [0u8; 64];
    let mut prev_effect_param = [0u8; 64];

    for row in &pattern.notes {
        for (channel, note) in row.iter().enumerate().take(num_channels as usize) {
            // Skip empty notes
            if note.note == 0
                && note.instrument == 0
                && note.volume == 0
                && note.effect == 0
                && note.effect_param == 0
            {
                continue;
            }

            // Build mask
            let mut mask = 0u8;

            if note.note != 0 && note.note != prev_note[channel] {
                mask |= 0x01;
                prev_note[channel] = note.note;
            } else if note.note != 0 {
                mask |= 0x10;
            }

            if note.instrument != 0 && note.instrument != prev_instrument[channel] {
                mask |= 0x02;
                prev_instrument[channel] = note.instrument;
            } else if note.instrument != 0 {
                mask |= 0x20;
            }

            if note.volume != 0 && note.volume != prev_volume[channel] {
                mask |= 0x04;
                prev_volume[channel] = note.volume;
            } else if note.volume != 0 {
                mask |= 0x40;
            }

            if (note.effect != 0 || note.effect_param != 0)
                && (note.effect != prev_effect[channel]
                    || note.effect_param != prev_effect_param[channel])
            {
                mask |= 0x08;
                prev_effect[channel] = note.effect;
                prev_effect_param[channel] = note.effect_param;
            } else if note.effect != 0 || note.effect_param != 0 {
                mask |= 0x80;
            }

            if mask == 0 {
                continue;
            }

            // Write channel marker with mask flag
            packed.push((channel as u8) | 0x80);
            packed.push(mask);

            // Write data
            if mask & 0x01 != 0 {
                packed.push(note.note);
            }
            if mask & 0x02 != 0 {
                packed.push(note.instrument);
            }
            if mask & 0x04 != 0 {
                packed.push(note.volume);
            }
            if mask & 0x08 != 0 {
                packed.push(note.effect);
                packed.push(note.effect_param);
            }
        }

        // End of row marker
        packed.push(0);
    }

    packed
}
