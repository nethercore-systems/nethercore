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
    if it.balance_mix {
        format = format | FormatFlags::IT_BALANCE_MIX;
    }
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
        channel_pan: it.channel_pan,
        channel_vol: it.channel_vol,
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
    // IT's note table is zero-based (C-0 = 0), while the unified engine uses
    // one-based notes (C-0 = 1). Preserve IT release commands verbatim.
    // Presence is explicit: C-0 and volume zero are not empty fields.
    let note = match it_note.note {
        nether_it::NOTE_FADE => TrackerNote::NOTE_FADE,
        nether_it::NOTE_CUT => TrackerNote::NOTE_CUT,
        nether_it::NOTE_OFF => TrackerNote::NOTE_OFF,
        nether_it::ItNote::NO_NOTE => 0,
        note if note <= nether_it::NOTE_MAX => note.saturating_add(1),
        _ => 0,
    };

    TrackerNote {
        note,
        instrument: it_note.instrument,
        volume: convert_it_volume(it_note.volume),
        effect: convert_it_effect(it_note.effect, it_note.effect_param),
        volume_effect: effects::convert_it_volume_effect(it_note.volume).unwrap_or_default(),
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_presence_survives_full_and_minimal_packing() {
        use nether_it::{ItNote, ItWriter};
        let mut writer = ItWriter::new("Presence regression");
        writer.set_channels(1);
        writer.add_pattern(3);
        writer.set_orders(&[0, 255]);
        writer.set_note(
            0,
            1,
            0,
            ItNote {
                note: 0,
                volume: 0,
                ..Default::default()
            },
        );
        writer.set_note(
            0,
            2,
            0,
            ItNote::default().with_effect(nether_it::effects::GLOBAL_VOLUME_SLIDE, 0),
        );
        let full = nether_it::parse_it(&writer.write()).unwrap();
        let packed = nether_it::parse_ncit(&nether_it::pack_ncit(&full)).unwrap();
        for source in [&full, &packed] {
            let module = super::from_it_module(source);
            let rows = &module.patterns[0].notes;
            assert_eq!(rows[0][0].note, 0);
            assert_eq!(rows[0][0].volume_effect, crate::TrackerEffect::None);
            assert_eq!(rows[1][0].note, 1); // actual C-0
            assert_eq!(rows[1][0].volume_effect, crate::TrackerEffect::SetVolume(0));
            assert_eq!(rows[2][0].note, 0); // effect only, no retrigger
            assert_eq!(rows[2][0].volume_effect, crate::TrackerEffect::None);
        }
    }
}
