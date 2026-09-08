//! XM instrument and envelope conversion

use crate::{
    DuplicateCheckAction, DuplicateCheckType, EnvelopeFlags, LoopType, NewNoteAction,
    TrackerEnvelope, TrackerInstrument,
};

use super::TARGET_SAMPLE_RATE;

/// Convert loop points from one sample rate to TARGET_SAMPLE_RATE (22050 Hz)
///
/// This calculates the loop points at the new sample rate after resampling.
///
/// # Arguments
/// * `original_rate` - Original sample rate
/// * `loop_start` - Loop start in original samples
/// * `loop_length` - Loop length in original samples
///
/// # Returns
/// * `(new_loop_start, new_loop_length)` at 22050 Hz
pub fn convert_loop_points(original_rate: u32, loop_start: u32, loop_length: u32) -> (u32, u32) {
    if original_rate == TARGET_SAMPLE_RATE {
        return (loop_start, loop_length);
    }

    let ratio = TARGET_SAMPLE_RATE as f64 / original_rate as f64;

    let new_start = (loop_start as f64 * ratio).round() as u32;
    let new_length = (loop_length as f64 * ratio).round() as u32;

    (new_start, new_length)
}

pub(super) fn convert_xm_instrument(xm_instr: &nether_xm::XmInstrument) -> TrackerInstrument {
    // Legacy NCXM instruments use one sound each; module conversion installs
    // retained local sample maps when extended sample metadata is present.
    let mut note_sample_table = [(0u8, 1u16); 120];
    for (i, entry) in note_sample_table.iter_mut().enumerate() {
        entry.0 = i as u8; // Note plays as itself
        // The module converter replaces this legacy placeholder for mapped XM.
    }

    // Convert sample loop type
    let sample_loop_type = match xm_instr.sample_loop_type {
        1 => LoopType::Forward,
        2 => LoopType::PingPong,
        _ => LoopType::None,
    };

    // Calculate original sample rate from finetune + relative_note
    // This is the same calculation used in audio_convert::convert_xm_sample()
    // Base frequency for C-4 (Amiga standard) = 8363 Hz
    let original_sample_rate = {
        const BASE_FREQ: f64 = 8363.0;
        let total_semitones =
            xm_instr.sample_relative_note as f64 + (xm_instr.sample_finetune as f64 / 128.0);
        let freq = BASE_FREQ * 2.0_f64.powf(total_semitones / 12.0);
        (freq.round() as u32).clamp(100, 96000)
    };

    // Convert loop points to match the resampled 22050 Hz sample
    let (sample_loop_start, sample_loop_length) = convert_loop_points(
        original_sample_rate,
        xm_instr.sample_loop_start,
        xm_instr.sample_loop_length,
    );

    TrackerInstrument {
        name: xm_instr.name.clone(),
        nna: NewNoteAction::Cut, // XM doesn't have NNA
        dct: DuplicateCheckType::Off,
        dca: DuplicateCheckAction::Cut,
        // XM fade starts at 32768; runtime gain uses a 65535 accumulator.
        fadeout: xm_instr.volume_fadeout.saturating_mul(2),
        global_volume: 64, // XM doesn't have global volume per instrument
        default_pan: xm_instr.sample_default_pan,
        random_volume: 0,
        random_pan: 0,
        note_sample_table,
        volume_envelope: xm_instr
            .volume_envelope
            .as_ref()
            .map(|env| convert_xm_envelope(env, 0)),
        panning_envelope: xm_instr
            .panning_envelope
            .as_ref()
            .map(|env| convert_xm_envelope(env, 32)),
        pitch_envelope: None, // XM doesn't have pitch envelope
        filter_cutoff: None,  // XM doesn't have filters
        filter_resonance: None,
        pitch_pan_separation: 0, // XM doesn't have pitch-pan separation
        pitch_pan_center: 60,    // Default center = C-5

        // Sample loop points - CONVERTED to match resampled 22050 Hz sample
        // Original loop points are in the source sample's sample space, but the
        // sample data has been resampled during ROM packing.
        sample_loop_start,
        sample_loop_end: sample_loop_start + sample_loop_length,
        sample_loop_type,
        sample_is_stereo: xm_instr.sample_is_stereo,
        // Finetune and relative_note are ZEROED because they're already baked into the
        // resampled 22050 Hz sample during ROM packing (via audio_convert::convert_xm_sample).
        // If we passed them through here, they would be applied AGAIN during playback,
        // causing notes to play way too high (often 1+ octave off).
        sample_finetune: 0,
        xm_source_tuning: i16::from(xm_instr.sample_relative_note)*128 + i16::from(xm_instr.sample_finetune),
        xm_source_finetune: xm_instr.sample_finetune,
        xm_forward_loop_start: f64::from(xm_instr.sample_loop_start)
            * f64::from(TARGET_SAMPLE_RATE)
            / f64::from(original_sample_rate),
        xm_forward_loop_limit: if xm_instr.sample_loop_type == 1 {
            f64::from(xm_instr.sample_loop_length) * f64::from(TARGET_SAMPLE_RATE)
                / f64::from(original_sample_rate)
        } else {
            0.0
        },
        sample_relative_note: 0,
        sample_default_volume: xm_instr.sample_default_volume,

        // Auto-vibrato settings
        auto_vibrato_type: xm_instr.vibrato_type,
        auto_vibrato_sweep: xm_instr.vibrato_sweep,
        auto_vibrato_depth: xm_instr.vibrato_depth,
        auto_vibrato_rate: xm_instr.vibrato_rate,
    }
}

fn convert_xm_envelope(xm_env: &nether_xm::XmEnvelope, center: i8) -> TrackerEnvelope {
    TrackerEnvelope {
        points: xm_env
            .points
            .iter()
            .map(|&(x, y)| (x, y.min(64) as i8 - center))
            .collect(),
        loop_begin: xm_env.loop_start,
        loop_end: xm_env.loop_end,
        sustain_begin: xm_env.sustain_point,
        sustain_end: xm_env.sustain_point,
        flags: convert_xm_envelope_flags(xm_env),
    }
}

fn convert_xm_envelope_flags(xm_env: &nether_xm::XmEnvelope) -> EnvelopeFlags {
    let mut flags = EnvelopeFlags::empty();

    if xm_env.enabled {
        flags = flags | EnvelopeFlags::ENABLED;
    }
    if xm_env.sustain_enabled {
        flags = flags | EnvelopeFlags::SUSTAIN_LOOP;
    }
    if xm_env.loop_enabled {
        flags = flags | EnvelopeFlags::LOOP;
    }

    flags
}
