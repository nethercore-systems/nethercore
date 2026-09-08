//! XM → TrackerModule conversion

/// Target sample rate for Nethercore audio
pub(crate) const TARGET_SAMPLE_RATE: u32 = 22050;

use crate::{FormatFlags, TrackerEffect, TrackerModule, TrackerNote, TrackerPattern};

mod effects;
mod instruments;

#[cfg(test)]
mod tests;

// Re-export public conversion functions
pub use effects::{convert_xm_effect, convert_xm_volume};
pub use instruments::convert_loop_points;

/// Convert an XM module to the unified TrackerModule format
pub fn from_xm_module(xm: &nether_xm::XmModule) -> TrackerModule {
    // Convert patterns
    let patterns = xm.patterns.iter().map(convert_xm_pattern).collect();

    // Convert instruments
    let instruments = xm
        .instruments
        .iter()
        .map(instruments::convert_xm_instrument)
        .collect();

    // Legacy NCXM has one sound per instrument; mapped NCXM flattens local slots.
    let mut samples = Vec::new();
    let mut instruments: Vec<crate::TrackerInstrument> = instruments;
    if xm.instruments.iter().any(|i| i.num_samples > 1 && !i.samples.is_empty()) {
        for (source, instrument) in xm.instruments.iter().zip(&mut instruments) {
            let base = samples.len();
            for (note, entry) in instrument.note_sample_table.iter_mut().enumerate() {
                entry.1 = source.sample_map.get(note)
                    .filter(|&&slot| (slot as usize) < source.samples.len())
                    .map_or(0, |&slot| (base + slot as usize + 1) as u16);
            }
            for sample in &source.samples {
                let rate = nether_xm::ExtractedSample::calculate_sample_rate(sample.finetune, sample.relative_note);
                let (start, length) = convert_loop_points(rate, sample.loop_start, sample.loop_length);
                samples.push(crate::TrackerSample {
                    default_volume: sample.volume,
                    default_pan: Some(sample.pan),
                    loop_begin: start,
                    loop_end: start.saturating_add(length),
                    loop_type: match sample.loop_type { 1 => crate::LoopType::Forward, 2 => crate::LoopType::PingPong, _ => crate::LoopType::None },
                    is_stereo: sample.is_stereo,
                    c5_speed: TARGET_SAMPLE_RATE,
                    xm_source_tuning: i16::from(sample.relative_note)*128 + i16::from(sample.finetune),
                    xm_source_finetune: sample.finetune,
                    xm_forward_loop_start: f64::from(sample.loop_start)
                        * f64::from(TARGET_SAMPLE_RATE)
                        / f64::from(rate),
                    xm_forward_loop_limit: if sample.loop_type == 1 {
                        f64::from(sample.loop_length) * f64::from(TARGET_SAMPLE_RATE)
                            / f64::from(rate)
                    } else {
                        0.0
                    },
                    vibrato_type: source.vibrato_type,
                    vibrato_depth: source.vibrato_depth,
                    vibrato_rate: source.vibrato_rate,
                    vibrato_speed: source.vibrato_sweep,
                    ..Default::default()
                });
            }
        }
    }

    // Convert format flags
    let mut format = FormatFlags::IS_XM_FORMAT | FormatFlags::INSTRUMENTS;
    if xm.linear_frequency_table {
        format = format | FormatFlags::LINEAR_SLIDES;
    }
    if xm.legacy_retrigger { format = format | FormatFlags::XM_LEGACY_RETRIGGER; }
    if matches!(xm.mix_mode, nether_xm::XmMixMode::Legacy) {
        format = format | FormatFlags::XM_LEGACY_MIX;
    }
    if matches!(xm.mix_mode, nether_xm::XmMixMode::Ft2) {
        format = format | FormatFlags::XM_FT2_MIX;
    }

    TrackerModule {
        name: xm.name.clone(),
        num_channels: xm.num_channels,
        initial_speed: xm.default_speed as u8,
        initial_tempo: xm.default_bpm as u8,
        global_volume: 64,       // XM doesn't have global volume in header
        mix_volume: xm.sample_preamp, // Source preamp in the shared 1/128 gain path
        panning_separation: 128, // XM doesn't have panning separation - default to full stereo (IT feature)
        channel_pan: [32; 64],
        channel_vol: [64; 64],
        order_table: xm.order_table.clone(),
        patterns,
        instruments,
        samples,
        format,
        message: None, // XM doesn't have song message
        restart_position: xm.restart_position,
    }
}

fn convert_xm_pattern(xm_pat: &nether_xm::XmPattern) -> TrackerPattern {
    let mut notes = Vec::with_capacity(xm_pat.num_rows as usize);

    for row in &xm_pat.notes {
        let mut tracker_row = Vec::with_capacity(row.len());
        for xm_note in row {
            tracker_row.push(convert_xm_note(xm_note));
        }
        notes.push(tracker_row);
    }

    TrackerPattern {
        num_rows: xm_pat.num_rows,
        notes,
    }
}

fn convert_xm_note(xm_note: &nether_xm::XmNote) -> TrackerNote {
    // XM note numbering: 0=none, 1-96=C-0..B-7, 97=note-off
    // TrackerNote uses XM-style 1-based numbering for compatibility with note_to_period()
    // XM C-4 (note 49) = middle C = 8363 Hz sample playback (now 22050 Hz after base freq fix)
    let note = if xm_note.note == nether_xm::NOTE_OFF {
        TrackerNote::NOTE_OFF
    } else if xm_note.note >= nether_xm::NOTE_MIN && xm_note.note <= nether_xm::NOTE_MAX {
        // Pass through unchanged - note_to_period() expects 1-based XM notes
        xm_note.note
    } else {
        0 // No note
    };

    let mut effect = convert_xm_effect(xm_note.effect, xm_note.effect_param);
    let mut volume_effect = effects::convert_xm_volume_effect(xm_note.volume).unwrap_or_default();
    // FT2 ignores 3xx when volume Fx is present and doubles Fx instead.
    if matches!(effect, TrackerEffect::TonePortamento(_))
        && let TrackerEffect::TonePortamento(speed) = volume_effect
    {
        volume_effect = TrackerEffect::TonePortamento(speed * 2);
        effect = TrackerEffect::None;
    }
    TrackerNote {
        note,
        instrument: xm_note.instrument,
        volume: convert_xm_volume(xm_note.volume),
        effect,
        volume_effect,
    }
}
