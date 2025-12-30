//! XM → TrackerModule conversion

use crate::*;

/// Convert an XM module to the unified TrackerModule format
pub fn from_xm_module(xm: &nether_xm::XmModule) -> TrackerModule {
    // Convert patterns
    let patterns = xm
        .patterns
        .iter()
        .map(|pat| convert_xm_pattern(pat))
        .collect();

    // Convert instruments
    let instruments = xm
        .instruments
        .iter()
        .map(|instr| convert_xm_instrument(instr))
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
        global_volume: 64, // XM doesn't have global volume in header
        order_table: xm.order_table.clone(),
        patterns,
        instruments,
        samples,
        format,
        message: None, // XM doesn't have song message
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

/// Convert XM volume column (0x10-0x50 for volume, others for effects)
fn convert_xm_volume(vol: u8) -> u8 {
    // XM volume column: 0x10-0x50 = volume 0-64
    if vol >= 0x10 && vol <= 0x50 {
        vol - 0x10
    } else {
        0
    }
}

/// Convert XM effect to unified TrackerEffect
fn convert_xm_effect(effect: u8, param: u8, volume: u8) -> TrackerEffect {
    // Check volume column for volume-column effects first
    if volume != 0 && (volume < 0x10 || volume > 0x50) {
        if let Some(vol_effect) = convert_xm_volume_effect(volume) {
            return vol_effect;
        }
    }

    // Convert main effect command
    match effect {
        0 => {
            // Arpeggio (special case: 0x00 with no param is "no effect")
            if param == 0 {
                TrackerEffect::None
            } else {
                let note1 = param >> 4;
                let note2 = param & 0x0F;
                TrackerEffect::Arpeggio { note1, note2 }
            }
        }

        // 1xx - Portamento up
        nether_xm::effects::PORTA_UP => TrackerEffect::PortamentoUp(param as u16),

        // 2xx - Portamento down
        nether_xm::effects::PORTA_DOWN => TrackerEffect::PortamentoDown(param as u16),

        // 3xx - Tone portamento
        nether_xm::effects::TONE_PORTA => TrackerEffect::TonePortamento(param as u16),

        // 4xy - Vibrato
        nether_xm::effects::VIBRATO => {
            let speed = param >> 4;
            let depth = param & 0x0F;
            TrackerEffect::Vibrato { speed, depth }
        }

        // 5xy - Tone portamento + volume slide
        nether_xm::effects::TONE_PORTA_VOL_SLIDE => {
            let vol_up = param >> 4;
            let vol_down = param & 0x0F;
            TrackerEffect::TonePortaVolSlide {
                porta: 0, // Use memory
                vol_up,
                vol_down,
            }
        }

        // 6xy - Vibrato + volume slide
        nether_xm::effects::VIBRATO_VOL_SLIDE => {
            let vol_up = param >> 4;
            let vol_down = param & 0x0F;
            TrackerEffect::VibratoVolSlide {
                vib_speed: 0, // Use memory
                vib_depth: 0,
                vol_up,
                vol_down,
            }
        }

        // 7xy - Tremolo
        nether_xm::effects::TREMOLO => {
            let speed = param >> 4;
            let depth = param & 0x0F;
            TrackerEffect::Tremolo { speed, depth }
        }

        // 8xx - Set panning
        nether_xm::effects::SET_PANNING => TrackerEffect::SetPanning(param / 4), // XM uses 0-255, we use 0-64

        // 9xx - Sample offset
        nether_xm::effects::SAMPLE_OFFSET => TrackerEffect::SampleOffset(param as u32 * 256),

        // Axy - Volume slide
        nether_xm::effects::VOLUME_SLIDE => {
            let up = param >> 4;
            let down = param & 0x0F;
            TrackerEffect::VolumeSlide { up, down }
        }

        // Bxx - Position jump
        nether_xm::effects::POSITION_JUMP => TrackerEffect::PositionJump(param),

        // Cxx - Set volume
        nether_xm::effects::SET_VOLUME => TrackerEffect::SetVolume(param.min(64)),

        // Dxx - Pattern break
        nether_xm::effects::PATTERN_BREAK => TrackerEffect::PatternBreak(param),

        // Exy - Extended effects
        nether_xm::effects::EXTENDED => convert_xm_extended_effect(param),

        // Fxx - Set speed/tempo
        nether_xm::effects::SET_SPEED_TEMPO => {
            if param < 0x20 {
                TrackerEffect::SetSpeed(param)
            } else {
                TrackerEffect::SetTempo(param)
            }
        }

        // Gxx - Set global volume
        nether_xm::effects::SET_GLOBAL_VOLUME => TrackerEffect::SetGlobalVolume(param * 2), // XM uses 0-64, we use 0-128

        // Hxy - Global volume slide
        nether_xm::effects::GLOBAL_VOLUME_SLIDE => {
            let up = param >> 4;
            let down = param & 0x0F;
            TrackerEffect::GlobalVolumeSlide { up, down }
        }

        // Kxx - Key off
        nether_xm::effects::KEY_OFF => TrackerEffect::KeyOff,

        // Lxx - Set envelope position
        nether_xm::effects::SET_ENVELOPE_POS => TrackerEffect::SetEnvelopePosition(param),

        // Pxy - Panning slide
        nether_xm::effects::PANNING_SLIDE => {
            let right = param >> 4;
            let left = param & 0x0F;
            TrackerEffect::PanningSlide { left, right }
        }

        // Rxy - Multi retrig note
        nether_xm::effects::MULTI_RETRIG => {
            let ticks = param & 0x0F;
            let volume = param >> 4;
            TrackerEffect::MultiRetrigNote { ticks, volume }
        }

        // Xxx - Extra fine portamento (XM specific)
        nether_xm::effects::EXTRA_FINE_PORTA => {
            if param & 0xF0 == 0x10 {
                TrackerEffect::ExtraFinePortaUp((param & 0x0F) as u16)
            } else if param & 0xF0 == 0x20 {
                TrackerEffect::ExtraFinePortaDown((param & 0x0F) as u16)
            } else {
                TrackerEffect::None
            }
        }

        _ => TrackerEffect::None,
    }
}

/// Convert XM extended effects (Exy)
fn convert_xm_extended_effect(param: u8) -> TrackerEffect {
    let sub_cmd = param >> 4;
    let value = param & 0x0F;

    match sub_cmd {
        0x1 => TrackerEffect::FinePortaUp(value as u16),
        0x2 => TrackerEffect::FinePortaDown(value as u16),
        0x4 => TrackerEffect::VibratoWaveform(value),
        0x5 => TrackerEffect::SetFinetune(value as i8),
        0x6 => TrackerEffect::PatternLoop(value),
        0x7 => TrackerEffect::TremoloWaveform(value),
        0x9 => TrackerEffect::None, // Retrigger note (not used in modern XM)
        0xA => TrackerEffect::FineVolumeUp(value),
        0xB => TrackerEffect::FineVolumeDown(value),
        0xC => TrackerEffect::NoteCut(value),
        0xD => TrackerEffect::NoteDelay(value),
        0xE => TrackerEffect::PatternDelay(value),
        _ => TrackerEffect::None,
    }
}

/// Convert XM volume column effects
fn convert_xm_volume_effect(vol: u8) -> Option<TrackerEffect> {
    match vol {
        0x10..=0x50 => None, // Direct volume, not an effect
        0x60..=0x6F => Some(TrackerEffect::VolumeSlide {
            up: 0,
            down: vol - 0x60,
        }),
        0x70..=0x7F => Some(TrackerEffect::VolumeSlide {
            up: vol - 0x70,
            down: 0,
        }),
        0x80..=0x8F => Some(TrackerEffect::FineVolumeDown(vol - 0x80)),
        0x90..=0x9F => Some(TrackerEffect::FineVolumeUp(vol - 0x90)),
        0xA0..=0xAF => Some(TrackerEffect::SetPanning((vol - 0xA0) * 4)),
        0xB0..=0xBF => Some(TrackerEffect::TonePortamento((vol - 0xB0) as u16)),
        0xC0..=0xCF => Some(TrackerEffect::PortamentoDown((vol - 0xC0) as u16)),
        0xD0..=0xDF => Some(TrackerEffect::PortamentoUp((vol - 0xD0) as u16)),
        _ => None,
    }
}

fn convert_xm_instrument(xm_instr: &nether_xm::XmInstrument) -> TrackerInstrument {
    // XM instruments don't have per-note sample mapping like IT
    // All notes use the same sample (sample 1)
    let mut note_sample_table = [(0u8, 1u8); 120];
    for (i, entry) in note_sample_table.iter_mut().enumerate() {
        entry.0 = i as u8; // Note plays as itself
        // All notes map to sample 1 (XM has simpler mapping)
    }

    TrackerInstrument {
        name: xm_instr.name.clone(),
        nna: NewNoteAction::Cut, // XM doesn't have NNA
        dct: DuplicateCheckType::Off,
        dca: DuplicateCheckAction::Cut,
        fadeout: xm_instr.volume_fadeout,
        global_volume: 64, // XM doesn't have global volume per instrument
        default_pan: None, // XM doesn't have default pan per instrument
        note_sample_table,
        volume_envelope: xm_instr
            .volume_envelope
            .as_ref()
            .map(convert_xm_envelope),
        panning_envelope: xm_instr
            .panning_envelope
            .as_ref()
            .map(convert_xm_envelope),
        pitch_envelope: None, // XM doesn't have pitch envelope
        filter_cutoff: None,  // XM doesn't have filters
        filter_resonance: None,
    }
}

fn convert_xm_envelope(xm_env: &nether_xm::XmEnvelope) -> TrackerEnvelope {
    TrackerEnvelope {
        points: xm_env
            .points
            .iter()
            .map(|&(x, y)| (x, y as i8))
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_convert_xm_note() {
        // XM note 49 (C-4, middle C) passes through unchanged
        // note_to_period() expects 1-based XM notes where 49 = C-4
        let xm_note = nether_xm::XmNote {
            note: 49,
            instrument: 1,
            volume: 0x30, // Volume 32
            effect: 0,
            effect_param: 0,
        };

        let tracker_note = convert_xm_note(&xm_note);
        assert_eq!(tracker_note.note, 49); // C-4, unchanged from XM
        assert_eq!(tracker_note.instrument, 1);
        assert_eq!(tracker_note.volume, 32);
    }

    #[test]
    fn test_convert_xm_effect_speed() {
        let effect = convert_xm_effect(nether_xm::effects::SET_SPEED_TEMPO, 6, 0);
        assert_eq!(effect, TrackerEffect::SetSpeed(6));
    }

    #[test]
    fn test_convert_xm_effect_tempo() {
        let effect = convert_xm_effect(nether_xm::effects::SET_SPEED_TEMPO, 125, 0);
        assert_eq!(effect, TrackerEffect::SetTempo(125));
    }

    #[test]
    fn test_convert_xm_volume_slide() {
        let effect = convert_xm_effect(nether_xm::effects::VOLUME_SLIDE, 0x52, 0);
        assert_eq!(
            effect,
            TrackerEffect::VolumeSlide { up: 5, down: 2 }
        );
    }
}
