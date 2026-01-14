//! Low-level encoding functions for IT file components

use crate::module::{ItEnvelope, ItInstrument, ItSample, ItSampleFlags};
use crate::{INSTRUMENT_MAGIC, MAX_ENVELOPE_POINTS, SAMPLE_MAGIC};

/// Write a fixed-length string, padded with zeros
pub fn write_string(output: &mut Vec<u8>, s: &str, len: usize) {
    let bytes = s.as_bytes();
    let copy_len = bytes.len().min(len);
    output.extend_from_slice(&bytes[..copy_len]);
    for _ in copy_len..len {
        output.push(0);
    }
}

/// Write an instrument to the output
pub fn write_instrument(output: &mut Vec<u8>, instr: &ItInstrument) {
    // Magic "IMPI"
    output.extend_from_slice(INSTRUMENT_MAGIC);

    // DOS filename (12 bytes)
    write_string(output, &instr.filename, 12);

    // Reserved (1 byte)
    output.push(0);

    // NNA, DCT, DCA
    output.push(instr.nna as u8);
    output.push(instr.dct as u8);
    output.push(instr.dca as u8);

    // Fadeout (2 bytes)
    output.extend_from_slice(&instr.fadeout.to_le_bytes());

    // PPS, PPC
    output.push(instr.pitch_pan_separation as u8);
    output.push(instr.pitch_pan_center);

    // GbV, DfP
    output.push(instr.global_volume);
    let dfp = instr.default_pan.map(|p| p | 0x80).unwrap_or(32);
    output.push(dfp);

    // RV, RP
    output.push(instr.random_volume);
    output.push(instr.random_pan);

    // TrkVers, NoS (4 bytes) - for instrument files only
    output.extend_from_slice(&[0u8; 4]);

    // Instrument name (26 bytes)
    write_string(output, &instr.name, 26);

    // IFC, IFR
    let ifc = instr.filter_cutoff.map(|c| c | 0x80).unwrap_or(0);
    let ifr = instr.filter_resonance.map(|r| r | 0x80).unwrap_or(0);
    output.push(ifc);
    output.push(ifr);

    // MCh, MPr, MIDIBnk
    output.push(instr.midi_channel);
    output.push(instr.midi_program);
    output.extend_from_slice(&instr.midi_bank.to_le_bytes());

    // Note-Sample-Keyboard table (240 bytes)
    for &(note, sample) in &instr.note_sample_table {
        output.push(note);
        output.push(sample);
    }

    // Volume envelope
    write_envelope(output, instr.volume_envelope.as_ref());

    // Panning envelope
    write_envelope(output, instr.panning_envelope.as_ref());

    // Pitch envelope
    write_envelope(output, instr.pitch_envelope.as_ref());
}

/// Write an envelope
pub fn write_envelope(output: &mut Vec<u8>, env: Option<&ItEnvelope>) {
    let env = env.cloned().unwrap_or_default();

    // Flags (1 byte)
    output.push(env.flags.bits());

    // Num points (1 byte)
    output.push(env.points.len().min(MAX_ENVELOPE_POINTS) as u8);

    // LpB, LpE, SLB, SLE
    output.push(env.loop_begin);
    output.push(env.loop_end);
    output.push(env.sustain_begin);
    output.push(env.sustain_end);

    // Node data (75 bytes = 25 × 3)
    for i in 0..MAX_ENVELOPE_POINTS {
        if let Some(&(tick, value)) = env.points.get(i) {
            output.push(value as u8);
            output.extend_from_slice(&tick.to_le_bytes());
        } else {
            output.extend_from_slice(&[0u8; 3]);
        }
    }

    // Reserved (1 byte)
    output.push(0);
}

/// Write a sample header
pub fn write_sample_header(output: &mut Vec<u8>, sample: &ItSample, data_offset: u32) {
    // Magic "IMPS"
    output.extend_from_slice(SAMPLE_MAGIC);

    // DOS filename (12 bytes)
    write_string(output, &sample.filename, 12);

    // Reserved (1 byte)
    output.push(0);

    // GvL
    output.push(sample.global_volume);

    // Flg - add SAMPLE_16BIT flag since we always write 16-bit
    let flags = sample.flags | ItSampleFlags::SAMPLE_16BIT;
    output.push(flags.bits());

    // Vol
    output.push(sample.default_volume);

    // Sample name (26 bytes)
    write_string(output, &sample.name, 26);

    // Cvt (convert flags) - 0x01 = signed samples
    output.push(0x01);

    // DfP
    let dfp = sample.default_pan.map(|p| p | 0x80).unwrap_or(0);
    output.push(dfp);

    // Length (4 bytes)
    output.extend_from_slice(&sample.length.to_le_bytes());

    // LoopBeg (4 bytes)
    output.extend_from_slice(&sample.loop_begin.to_le_bytes());

    // LoopEnd (4 bytes)
    output.extend_from_slice(&sample.loop_end.to_le_bytes());

    // C5Speed (4 bytes)
    output.extend_from_slice(&sample.c5_speed.to_le_bytes());

    // SusLBeg (4 bytes)
    output.extend_from_slice(&sample.sustain_loop_begin.to_le_bytes());

    // SusLEnd (4 bytes)
    output.extend_from_slice(&sample.sustain_loop_end.to_le_bytes());

    // SmpPoint (4 bytes) - offset to sample data
    output.extend_from_slice(&data_offset.to_le_bytes());

    // ViS, ViD, ViR, ViT
    output.push(sample.vibrato_speed);
    output.push(sample.vibrato_depth);
    output.push(sample.vibrato_rate);
    output.push(sample.vibrato_type);
}
