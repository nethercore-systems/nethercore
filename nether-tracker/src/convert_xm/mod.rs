//! XM → TrackerModule conversion

/// Target sample rate for Nethercore audio
pub(crate) const TARGET_SAMPLE_RATE: u32 = 22050;

use crate::{FormatFlags, TrackerModule, TrackerNote, TrackerPattern};

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

    // XM doesn't have separate samples at the module level (they're in instruments)
    // Create placeholder samples from instrument metadata
    let samples = vec![];

    // Convert format flags
    let mut format = FormatFlags::IS_XM_FORMAT | FormatFlags::INSTRUMENTS;
    if xm.linear_frequency_table {
        format = format | FormatFlags::LINEAR_SLIDES;
    }

    TrackerModule {
        name: xm.name.clone(),
        num_channels: xm.num_channels,
        initial_speed: xm.default_speed as u8,
        initial_tempo: xm.default_bpm as u8,
        global_volume: 64,       // XM doesn't have global volume in header
        mix_volume: 128,         // XM doesn't have mix volume - default to full (IT feature)
        panning_separation: 128, // XM doesn't have panning separation - default to full stereo (IT feature)
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

    TrackerNote {
        note,
        instrument: xm_note.instrument,
        volume: convert_xm_volume(xm_note.volume),
        effect: convert_xm_effect(xm_note.effect, xm_note.effect_param, xm_note.volume),
    }
}
