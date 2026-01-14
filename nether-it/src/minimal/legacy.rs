//! Legacy functions for backwards compatibility

use crate::error::ItError;
use crate::module::ItModule;
use crate::parser::parse_it;

/// Strip sample data from an IT file, keeping only patterns and metadata
///
/// This creates a minimal IT file that can be stored in the ROM with much smaller size.
/// Sample data is loaded separately via the ROM data pack.
///
/// **Note**: This function is deprecated in favor of `pack_ncit()` which produces
/// a more compact format.
pub fn strip_it_samples(data: &[u8]) -> Result<Vec<u8>, ItError> {
    let module = parse_it(data)?;
    Ok(pack_it_minimal(&module))
}

/// Pack an IT module into minimal IT format (legacy, maintains IT file validity)
///
/// **Note**: For new code, prefer `pack_ncit()` which produces a more compact format.
/// This function is retained for backwards compatibility and debugging (output can
/// be loaded in OpenMPT/SchismTracker).
pub fn pack_it_minimal(module: &ItModule) -> Vec<u8> {
    use crate::writer::ItWriter;

    // Use the writer but with empty sample data
    let mut writer = ItWriter::new(&module.name);
    writer.set_speed(module.initial_speed);
    writer.set_tempo(module.initial_tempo);
    writer.set_global_volume(module.global_volume);
    writer.set_mix_volume(module.mix_volume);
    writer.set_channels(module.num_channels);
    writer.set_flags(module.flags);

    // Add instruments
    for instr in &module.instruments {
        writer.add_instrument(instr.clone());
    }

    // Add samples with empty audio data
    for sample in &module.samples {
        let mut s = sample.clone();
        s.length = 0; // No audio data
        writer.add_sample(s, &[]);
    }

    // Add patterns
    for pattern in &module.patterns {
        let pat_idx = writer.add_pattern(pattern.num_rows);
        for (row, row_data) in pattern.notes.iter().enumerate() {
            for (channel, note) in row_data.iter().enumerate() {
                if note.note != 0
                    || note.instrument != 0
                    || note.volume != 0
                    || note.effect != 0
                    || note.effect_param != 0
                {
                    writer.set_note(pat_idx, row as u16, channel as u8, *note);
                }
            }
        }
    }

    // Set order table
    writer.set_orders(&module.order_table);

    // Set message if present
    if let Some(ref msg) = module.message {
        writer.set_message(msg);
    }

    writer.write()
}
