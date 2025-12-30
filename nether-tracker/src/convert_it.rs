//! IT → TrackerModule conversion

use crate::*;

/// Convert an IT module to the unified TrackerModule format
pub fn from_it_module(it: &nether_it::ItModule) -> TrackerModule {
    // Convert patterns
    let patterns = it
        .patterns
        .iter()
        .map(|pat| convert_it_pattern(pat))
        .collect();

    // Convert instruments
    let instruments = it
        .instruments
        .iter()
        .map(|instr| convert_it_instrument(instr))
        .collect();

    // Convert samples
    let samples = it.samples.iter().map(|smp| convert_it_sample(smp)).collect();

    // Convert format flags
    let mut format = FormatFlags::IS_IT_FORMAT;
    if it.uses_linear_slides() {
        format = format | FormatFlags::LINEAR_SLIDES;
    }
    if it.uses_instruments() {
        format = format | FormatFlags::INSTRUMENTS;
    }

    TrackerModule {
        name: it.name.clone(),
        num_channels: it.num_channels,
        initial_speed: it.initial_speed,
        initial_tempo: it.initial_tempo,
        global_volume: it.global_volume,
        order_table: it.order_table.clone(),
        patterns,
        instruments,
        samples,
        format,
        message: it.message.clone(),
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

/// Convert IT volume column (0-64 for direct volume, or volume effects)
fn convert_it_volume(vol: u8) -> u8 {
    // Simple volume (0-64) is preserved
    // Volume effects are handled in convert_it_effect
    if vol <= 64 {
        vol
    } else {
        0
    }
}

/// Convert IT effect to unified TrackerEffect
fn convert_it_effect(effect: u8, param: u8, volume: u8) -> TrackerEffect {
    // Check volume column for volume-column effects first
    if volume > 64 {
        if let Some(vol_effect) = convert_it_volume_effect(volume) {
            return vol_effect;
        }
    }

    // Convert main effect command
    match effect {
        0 => TrackerEffect::None,

        // Axx - Set speed
        nether_it::effects::SET_SPEED => TrackerEffect::SetSpeed(param),

        // Bxx - Position jump
        nether_it::effects::POSITION_JUMP => TrackerEffect::PositionJump(param),

        // Cxx - Pattern break
        nether_it::effects::PATTERN_BREAK => TrackerEffect::PatternBreak(param),

        // Dxy - Volume slide
        nether_it::effects::VOLUME_SLIDE => {
            let up = param >> 4;
            let down = param & 0x0F;
            TrackerEffect::VolumeSlide { up, down }
        }

        // Exx - Portamento down
        nether_it::effects::PORTA_DOWN => TrackerEffect::PortamentoDown(param as u16),

        // Fxx - Portamento up
        nether_it::effects::PORTA_UP => TrackerEffect::PortamentoUp(param as u16),

        // Gxx - Tone portamento
        nether_it::effects::TONE_PORTA => TrackerEffect::TonePortamento(param as u16),

        // Hxy - Vibrato
        nether_it::effects::VIBRATO => {
            let speed = param >> 4;
            let depth = param & 0x0F;
            TrackerEffect::Vibrato { speed, depth }
        }

        // Ixy - Tremor
        nether_it::effects::TREMOR => {
            let ontime = param >> 4;
            let offtime = param & 0x0F;
            TrackerEffect::Tremor { ontime, offtime }
        }

        // Jxy - Arpeggio
        nether_it::effects::ARPEGGIO => {
            let note1 = param >> 4;
            let note2 = param & 0x0F;
            TrackerEffect::Arpeggio { note1, note2 }
        }

        // Kxy - Vibrato + volume slide
        nether_it::effects::VIBRATO_VOL_SLIDE => {
            let vol_up = param >> 4;
            let vol_down = param & 0x0F;
            TrackerEffect::VibratoVolSlide {
                vib_speed: 0, // Use memory
                vib_depth: 0,
                vol_up,
                vol_down,
            }
        }

        // Lxy - Tone portamento + volume slide
        nether_it::effects::TONE_PORTA_VOL_SLIDE => {
            let vol_up = param >> 4;
            let vol_down = param & 0x0F;
            TrackerEffect::TonePortaVolSlide {
                porta: 0, // Use memory
                vol_up,
                vol_down,
            }
        }

        // Mxx - Set channel volume
        nether_it::effects::SET_CHANNEL_VOLUME => TrackerEffect::SetChannelVolume(param),

        // Nxy - Channel volume slide
        nether_it::effects::CHANNEL_VOLUME_SLIDE => {
            let up = param >> 4;
            let down = param & 0x0F;
            TrackerEffect::ChannelVolumeSlide { up, down }
        }

        // Oxx - Sample offset
        nether_it::effects::SAMPLE_OFFSET => TrackerEffect::SampleOffset(param as u32 * 256),

        // Pxy - Panning slide
        nether_it::effects::PANNING_SLIDE => {
            let right = param >> 4;
            let left = param & 0x0F;
            TrackerEffect::PanningSlide { left, right }
        }

        // Qxy - Retrigger
        nether_it::effects::RETRIGGER => {
            let ticks = param & 0x0F;
            let vol_change = (param >> 4) as i8;
            TrackerEffect::Retrigger {
                ticks,
                volume_change: vol_change,
            }
        }

        // Rxy - Tremolo
        nether_it::effects::TREMOLO => {
            let speed = param >> 4;
            let depth = param & 0x0F;
            TrackerEffect::Tremolo { speed, depth }
        }

        // Sxy - Extended effects
        nether_it::effects::EXTENDED => convert_it_extended_effect(param),

        // Txx - Set tempo
        nether_it::effects::SET_TEMPO => TrackerEffect::SetTempo(param),

        // Uxy - Fine vibrato
        nether_it::effects::FINE_VIBRATO => {
            let speed = param >> 4;
            let depth = param & 0x0F;
            TrackerEffect::FineVibrato { speed, depth }
        }

        // Vxx - Set global volume
        nether_it::effects::SET_GLOBAL_VOLUME => TrackerEffect::SetGlobalVolume(param),

        // Wxy - Global volume slide
        nether_it::effects::GLOBAL_VOLUME_SLIDE => {
            let up = param >> 4;
            let down = param & 0x0F;
            TrackerEffect::GlobalVolumeSlide { up, down }
        }

        // Xxx - Set panning
        nether_it::effects::SET_PANNING => TrackerEffect::SetPanning(param / 2), // IT uses 0-128, we use 0-64

        // Yxy - Panbrello
        nether_it::effects::PANBRELLO => {
            let speed = param >> 4;
            let depth = param & 0x0F;
            TrackerEffect::Panbrello { speed, depth }
        }

        // Zxx - MIDI macro / filter
        nether_it::effects::MIDI_MACRO => TrackerEffect::SetFilterCutoff(param),

        _ => TrackerEffect::None,
    }
}

/// Convert IT extended effects (Sxy)
fn convert_it_extended_effect(param: u8) -> TrackerEffect {
    let sub_cmd = param >> 4;
    let value = param & 0x0F;

    match sub_cmd {
        nether_it::extended_effects::SET_FILTER => TrackerEffect::None, // Obsolete
        nether_it::extended_effects::GLISSANDO => TrackerEffect::SetGlissando(value != 0),
        nether_it::extended_effects::SET_FINETUNE => TrackerEffect::None, // Not supported
        nether_it::extended_effects::VIBRATO_WAVEFORM => TrackerEffect::VibratoWaveform(value),
        nether_it::extended_effects::TREMOLO_WAVEFORM => TrackerEffect::TremoloWaveform(value),
        nether_it::extended_effects::PANBRELLO_WAVEFORM => TrackerEffect::PanbrelloWaveform(value),
        nether_it::extended_effects::PATTERN_LOOP => TrackerEffect::PatternLoop(value),
        nether_it::extended_effects::NOTE_CUT => TrackerEffect::NoteCut(value),
        nether_it::extended_effects::NOTE_DELAY => TrackerEffect::NoteDelay(value),
        nether_it::extended_effects::PATTERN_DELAY => TrackerEffect::PatternDelay(value),
        _ => TrackerEffect::None,
    }
}

/// Convert IT volume column effects
fn convert_it_volume_effect(vol: u8) -> Option<TrackerEffect> {
    match vol {
        0..=64 => None, // Direct volume, not an effect
        65..=74 => Some(TrackerEffect::FineVolumeUp(vol - 65)),
        75..=84 => Some(TrackerEffect::FineVolumeDown(vol - 75)),
        85..=94 => Some(TrackerEffect::VolumeSlide {
            up: vol - 85,
            down: 0,
        }),
        95..=104 => Some(TrackerEffect::VolumeSlide {
            up: 0,
            down: vol - 95,
        }),
        105..=114 => Some(TrackerEffect::FinePortaDown((vol - 105) as u16 * 4)),
        115..=124 => Some(TrackerEffect::FinePortaUp((vol - 115) as u16 * 4)),
        128..=192 => None, // Panning, handled separately
        193..=202 => Some(TrackerEffect::TonePortamento((vol - 193) as u16 * 4)),
        203..=212 => Some(TrackerEffect::Vibrato {
            speed: 0,
            depth: (vol - 203),
        }),
        _ => None,
    }
}

fn convert_it_instrument(it_instr: &nether_it::ItInstrument) -> TrackerInstrument {
    TrackerInstrument {
        name: it_instr.name.clone(),
        nna: convert_it_nna(it_instr.nna),
        dct: convert_it_dct(it_instr.dct),
        dca: convert_it_dca(it_instr.dca),
        fadeout: it_instr.fadeout,
        global_volume: (it_instr.global_volume / 2).min(64), // IT uses 0-128, we use 0-64
        default_pan: it_instr.default_pan,
        note_sample_table: it_instr.note_sample_table,
        volume_envelope: it_instr
            .volume_envelope
            .as_ref()
            .map(convert_it_envelope),
        panning_envelope: it_instr
            .panning_envelope
            .as_ref()
            .map(convert_it_envelope),
        pitch_envelope: it_instr.pitch_envelope.as_ref().map(convert_it_envelope),
        filter_cutoff: it_instr.filter_cutoff,
        filter_resonance: it_instr.filter_resonance,
    }
}

fn convert_it_nna(nna: nether_it::NewNoteAction) -> NewNoteAction {
    match nna {
        nether_it::NewNoteAction::Cut => NewNoteAction::Cut,
        nether_it::NewNoteAction::Continue => NewNoteAction::Continue,
        nether_it::NewNoteAction::NoteOff => NewNoteAction::NoteOff,
        nether_it::NewNoteAction::NoteFade => NewNoteAction::NoteFade,
    }
}

fn convert_it_dct(dct: nether_it::DuplicateCheckType) -> DuplicateCheckType {
    match dct {
        nether_it::DuplicateCheckType::Off => DuplicateCheckType::Off,
        nether_it::DuplicateCheckType::Note => DuplicateCheckType::Note,
        nether_it::DuplicateCheckType::Sample => DuplicateCheckType::Sample,
        nether_it::DuplicateCheckType::Instrument => DuplicateCheckType::Instrument,
    }
}

fn convert_it_dca(dca: nether_it::DuplicateCheckAction) -> DuplicateCheckAction {
    match dca {
        nether_it::DuplicateCheckAction::Cut => DuplicateCheckAction::Cut,
        nether_it::DuplicateCheckAction::NoteOff => DuplicateCheckAction::NoteOff,
        nether_it::DuplicateCheckAction::NoteFade => DuplicateCheckAction::NoteFade,
    }
}

fn convert_it_envelope(it_env: &nether_it::ItEnvelope) -> TrackerEnvelope {
    TrackerEnvelope {
        points: it_env.points.clone(),
        loop_begin: it_env.loop_begin,
        loop_end: it_env.loop_end,
        sustain_begin: it_env.sustain_begin,
        sustain_end: it_env.sustain_end,
        flags: convert_it_envelope_flags(it_env.flags),
    }
}

fn convert_it_envelope_flags(it_flags: nether_it::ItEnvelopeFlags) -> EnvelopeFlags {
    let mut flags = EnvelopeFlags::empty();

    if it_flags.contains(nether_it::ItEnvelopeFlags::ENABLED) {
        flags = flags | EnvelopeFlags::ENABLED;
    }
    if it_flags.contains(nether_it::ItEnvelopeFlags::LOOP) {
        flags = flags | EnvelopeFlags::LOOP;
    }
    if it_flags.contains(nether_it::ItEnvelopeFlags::SUSTAIN_LOOP) {
        flags = flags | EnvelopeFlags::SUSTAIN_LOOP;
    }
    if it_flags.contains(nether_it::ItEnvelopeFlags::CARRY) {
        flags = flags | EnvelopeFlags::CARRY;
    }
    if it_flags.contains(nether_it::ItEnvelopeFlags::FILTER) {
        flags = flags | EnvelopeFlags::FILTER;
    }

    flags
}

fn convert_it_sample(it_smp: &nether_it::ItSample) -> TrackerSample {
    TrackerSample {
        name: it_smp.name.clone(),
        global_volume: it_smp.global_volume,
        default_volume: it_smp.default_volume,
        default_pan: it_smp.default_pan,
        length: it_smp.length,
        loop_begin: it_smp.loop_begin,
        loop_end: it_smp.loop_end,
        loop_type: if it_smp.has_loop() {
            if it_smp.is_pingpong_loop() {
                LoopType::PingPong
            } else {
                LoopType::Forward
            }
        } else {
            LoopType::None
        },
        c5_speed: it_smp.c5_speed,
        sustain_loop_begin: it_smp.sustain_loop_begin,
        sustain_loop_end: it_smp.sustain_loop_end,
        sustain_loop_type: if it_smp.has_sustain_loop() {
            if it_smp.is_pingpong_sustain() {
                LoopType::PingPong
            } else {
                LoopType::Forward
            }
        } else {
            LoopType::None
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_convert_it_effect_speed() {
        let effect = convert_it_effect(nether_it::effects::SET_SPEED, 6, 0);
        assert_eq!(effect, TrackerEffect::SetSpeed(6));
    }

    #[test]
    fn test_convert_it_effect_volume_slide() {
        let effect = convert_it_effect(nether_it::effects::VOLUME_SLIDE, 0x52, 0); // Up 5, down 2
        assert_eq!(
            effect,
            TrackerEffect::VolumeSlide { up: 5, down: 2 }
        );
    }

    #[test]
    fn test_convert_it_volume_effect() {
        // Fine volume up
        assert_eq!(
            convert_it_volume_effect(70),
            Some(TrackerEffect::FineVolumeUp(5))
        );

        // Fine volume down
        assert_eq!(
            convert_it_volume_effect(80),
            Some(TrackerEffect::FineVolumeDown(5))
        );

        // Direct volume
        assert_eq!(convert_it_volume_effect(32), None);
    }
}
