//! IT pattern compression

use crate::module::ItPattern;

/// Pack pattern data using IT compression
pub fn pack_pattern(pattern: &ItPattern, num_channels: u8) -> Vec<u8> {
    let mut output = Vec::new();

    // Previous values for compression
    let mut prev_note = [crate::ItNote::NO_NOTE; 64];
    let mut prev_instrument = [0u8; 64];
    let mut prev_volume = [crate::ItNote::NO_VOLUME; 64];
    let mut prev_effect = [0u8; 64];
    let mut prev_effect_param = [0u8; 64];

    for row in &pattern.notes {
        for (channel, note) in row.iter().enumerate().take(num_channels as usize) {
            // Skip empty notes
            if note.note == crate::ItNote::NO_NOTE
                && note.instrument == 0
                && note.volume == crate::ItNote::NO_VOLUME
                && note.effect == 0
                && note.effect_param == 0
            {
                continue;
            }

            // Build mask
            let mut mask = 0u8;

            if note.note != crate::ItNote::NO_NOTE && note.note != prev_note[channel] {
                mask |= 0x01;
                prev_note[channel] = note.note;
            } else if note.note != crate::ItNote::NO_NOTE {
                mask |= 0x10;
            }

            if note.instrument != 0 && note.instrument != prev_instrument[channel] {
                mask |= 0x02;
                prev_instrument[channel] = note.instrument;
            } else if note.instrument != 0 {
                mask |= 0x20;
            }

            if note.volume != crate::ItNote::NO_VOLUME && note.volume != prev_volume[channel] {
                mask |= 0x04;
                prev_volume[channel] = note.volume;
            } else if note.volume != crate::ItNote::NO_VOLUME {
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
            // IT channel markers are one-based: 1..=64 map to channels 0..=63.
            output.push((channel as u8 + 1) | 0x80);
            output.push(mask);

            // Write data
            if mask & 0x01 != 0 {
                output.push(note.note);
            }
            if mask & 0x02 != 0 {
                output.push(note.instrument);
            }
            if mask & 0x04 != 0 {
                output.push(note.volume);
            }
            if mask & 0x08 != 0 {
                output.push(note.effect);
                output.push(note.effect_param);
            }
        }

        // End of row marker
        output.push(0);
    }

    output
}
