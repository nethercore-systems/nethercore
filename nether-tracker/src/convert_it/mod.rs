//! IT → TrackerModule conversion

mod effects;
mod instrument;

use effects::{convert_it_effect, convert_it_volume};
use instrument::{convert_it_instrument, convert_it_sample};

use crate::{FormatFlags, TrackerModule, TrackerNote, TrackerPattern};

/// Convert an IT module to the unified TrackerModule format
pub fn from_it_module(it: &nether_it::ItModule) -> TrackerModule {
    // Convert patterns
    let patterns = it.patterns.iter().map(convert_it_pattern).collect();

    // Convert instruments
    let instruments = it.instruments.iter().map(convert_it_instrument).collect();

    // Convert samples
    let samples = it.samples.iter().map(convert_it_sample).collect();

    // Convert format flags
    let mut format = FormatFlags::IS_IT_FORMAT;
    if it.uses_linear_slides() {
        format = format | FormatFlags::LINEAR_SLIDES;
    }
    if it.uses_instruments() {
        format = format | FormatFlags::INSTRUMENTS;
    }
    if it.uses_old_effects() {
        format = format | FormatFlags::OLD_EFFECTS;
    }
    if it.uses_link_g_memory() {
        format = format | FormatFlags::LINK_G_MEMORY;
    }

    TrackerModule {
        name: it.name.clone(),
        num_channels: it.num_channels,
        initial_speed: it.initial_speed,
        initial_tempo: it.initial_tempo,
        global_volume: it.global_volume,
        mix_volume: it.mix_volume,
        panning_separation: it.panning_separation,
        order_table: it.order_table.clone(),
        patterns,
        instruments,
        samples,
        format,
        message: it.message.clone(),
        restart_position: 0, // IT doesn't have restart position feature
    }
}

fn convert_it_pattern(it_pat: &nether_it::ItPattern) -> TrackerPattern {
    let mut notes = Vec::with_capacity(it_pat.num_rows as usize);

    for row in &it_pat.notes {
        let mut tracker_row = Vec::with_capacity(row.len());
        for it_note in row {
            tracker_row.push(convert_it_note(it_note));
        }
        notes.push(tracker_row);
    }

    TrackerPattern {
        num_rows: it_pat.num_rows,
        notes,
    }
}

fn convert_it_note(it_note: &nether_it::ItNote) -> TrackerNote {
    TrackerNote {
        note: it_note.note,
        instrument: it_note.instrument,
        volume: convert_it_volume(it_note.volume),
        effect: convert_it_effect(it_note.effect, it_note.effect_param, it_note.volume),
    }
}
