//! IT → TrackerModule conversion

use crate::*;

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
    if volume > 64 && let Some(vol_effect) = convert_it_volume_effect(volume) {
        return vol_effect;
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

        // Dxy - Volume slide (IT spec order: Dx0, D0x, DxF fine up, DFx fine down)
        nether_it::effects::VOLUME_SLIDE => {
            let up = param >> 4;
            let down = param & 0x0F;
            // DxF (x != 0, x != F) = Fine volume slide up (tick 0 only)
            if down == 0x0F && up != 0 && up != 0x0F {
                TrackerEffect::FineVolumeUp(up)
            }
            // DFx (x != 0, x != F) = Fine volume slide down (tick 0 only)
            else if up == 0x0F && down != 0 && down != 0x0F {
                TrackerEffect::FineVolumeDown(down)
            }
            // Regular volume slide (every tick except tick 0)
            else {
                TrackerEffect::VolumeSlide { up, down }
            }
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

        // Nxy - Channel volume slide (same fine slide rules as Dxy)
        nether_it::effects::CHANNEL_VOLUME_SLIDE => {
            let up = param >> 4;
            let down = param & 0x0F;
            // NxF (x != 0, x != F) = Fine channel volume slide up (tick 0 only)
            if down == 0x0F && up != 0 && up != 0x0F {
                TrackerEffect::FineChannelVolumeUp(up)
            }
            // NFx (x != 0, x != F) = Fine channel volume slide down (tick 0 only)
            else if up == 0x0F && down != 0 && down != 0x0F {
                TrackerEffect::FineChannelVolumeDown(down)
            }
            // Regular channel volume slide
            else {
                TrackerEffect::ChannelVolumeSlide { up, down }
            }
        }

        // Oxx - Sample offset
        nether_it::effects::SAMPLE_OFFSET => TrackerEffect::SampleOffset(param as u32 * 256),

        // Pxy - Panning slide (IT spec order: Px0, P0x, PxF fine right, PFx fine left)
        nether_it::effects::PANNING_SLIDE => {
            let right = param >> 4;
            let left = param & 0x0F;
            // PxF (x != 0, x != F) = Fine panning slide right (tick 0 only)
            if left == 0x0F && right != 0 && right != 0x0F {
                TrackerEffect::FinePanningRight(right)
            }
            // PFx (x != 0, x != F) = Fine panning slide left (tick 0 only)
            else if right == 0x0F && left != 0 && left != 0x0F {
                TrackerEffect::FinePanningLeft(left)
            }
            // Regular panning slide (every tick except tick 0)
            else {
                TrackerEffect::PanningSlide { left, right }
            }
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

        // Txx - Set tempo / tempo slide
        // T0x = tempo slide down by x BPM per tick
        // T1x = tempo slide up by x BPM per tick
        // Txx (xx >= 0x20) = set tempo directly
        nether_it::effects::SET_TEMPO => {
            if param < 0x10 {
                // T0x = tempo slide down
                TrackerEffect::TempoSlideDown(param)
            } else if param < 0x20 {
                // T1x = tempo slide up
                TrackerEffect::TempoSlideUp(param & 0x0F)
            } else {
                // Txx = set tempo directly
                TrackerEffect::SetTempo(param)
            }
        }

        // Uxy - Fine vibrato
        nether_it::effects::FINE_VIBRATO => {
            let speed = param >> 4;
            let depth = param & 0x0F;
            TrackerEffect::FineVibrato { speed, depth }
        }

        // Vxx - Set global volume
        nether_it::effects::SET_GLOBAL_VOLUME => TrackerEffect::SetGlobalVolume(param),

        // Wxy - Global volume slide (same fine slide rules as Dxy)
        nether_it::effects::GLOBAL_VOLUME_SLIDE => {
            let up = param >> 4;
            let down = param & 0x0F;
            // WxF (x != 0, x != F) = Fine global volume slide up (tick 0 only)
            if down == 0x0F && up != 0 && up != 0x0F {
                TrackerEffect::FineGlobalVolumeUp(up)
            }
            // WFx (x != 0, x != F) = Fine global volume slide down (tick 0 only)
            else if up == 0x0F && down != 0 && down != 0x0F {
                TrackerEffect::FineGlobalVolumeDown(down)
            }
            // Regular global volume slide
            else {
                TrackerEffect::GlobalVolumeSlide { up, down }
            }
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
        // Z00-Z7F: Filter cutoff (0-127)
        // Z80-Z8F: Filter resonance (0-15 mapped to 0-127)
        nether_it::effects::MIDI_MACRO => {
            if param <= 0x7F {
                TrackerEffect::SetFilterCutoff(param)
            } else if param <= 0x8F {
                // Resonance 0-15, scale to 0-127 for consistency
                TrackerEffect::SetFilterResonance((param & 0x0F) * 8)
            } else {
                // Z90-ZFF: Other MIDI macros, not commonly used for filters
                TrackerEffect::None
            }
        }

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
        nether_it::extended_effects::FINE_PATTERN_DELAY => TrackerEffect::FinePatternDelay(value),
        // S7x - Instrument control (NNA/envelope) - complex, rarely used in practice
        nether_it::extended_effects::INSTRUMENT_CONTROL => TrackerEffect::None,
        // S8x - coarse panning: value * 4 + 2 (centers 0-15 to 2-62)
        nether_it::extended_effects::SET_PANNING_COARSE => {
            TrackerEffect::SetPanning((value * 4).saturating_add(2).min(64))
        }
        // S9x - Sound control (surround/reverse) - rarely used
        nether_it::extended_effects::SOUND_CONTROL => TrackerEffect::None,
        nether_it::extended_effects::HIGH_SAMPLE_OFFSET => TrackerEffect::HighSampleOffset(value),
        nether_it::extended_effects::PATTERN_LOOP => TrackerEffect::PatternLoop(value),
        nether_it::extended_effects::NOTE_CUT => TrackerEffect::NoteCut(value),
        nether_it::extended_effects::NOTE_DELAY => TrackerEffect::NoteDelay(value),
        nether_it::extended_effects::PATTERN_DELAY => TrackerEffect::PatternDelay(value),
        nether_it::extended_effects::SET_ACTIVE_MACRO => TrackerEffect::None, // MIDI macro
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
        128..=192 => Some(TrackerEffect::SetPanning(vol - 128)), // 0-64 panning
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
        // IT spec: NFC starts at 1024, fadeout subtracted each tick
        // We use 65535 for more precision, so scale by 64 (65535/1024)
        fadeout: it_instr.fadeout.saturating_mul(64),
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
        pitch_pan_separation: it_instr.pitch_pan_separation,
        pitch_pan_center: it_instr.pitch_pan_center,

        // IT stores sample metadata per-sample, not per-instrument.
        // The playback engine should look up from TrackerModule.samples
        // using note_sample_table. For now, use defaults.
        sample_loop_start: 0,
        sample_loop_end: 0,
        sample_loop_type: LoopType::None,
        sample_finetune: 0,
        sample_relative_note: 0,

        // IT doesn't have XM-style auto-vibrato per instrument
        // (IT uses sample vibrato instead)
        auto_vibrato_type: 0,
        auto_vibrato_sweep: 0,
        auto_vibrato_depth: 0,
        auto_vibrato_rate: 0,
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

    #[test]
    fn test_convert_it_volume_effect_panning() {
        // IT volume column panning: 128-192 maps to panning 0-64
        assert_eq!(
            convert_it_volume_effect(128),
            Some(TrackerEffect::SetPanning(0)) // Full left
        );
        assert_eq!(
            convert_it_volume_effect(160),
            Some(TrackerEffect::SetPanning(32)) // Center
        );
        assert_eq!(
            convert_it_volume_effect(192),
            Some(TrackerEffect::SetPanning(64)) // Full right
        );
    }

    #[test]
    fn test_convert_it_filter_effects() {
        // Filter cutoff (Zxx where xx = 00-7F)
        let effect = convert_it_effect(nether_it::effects::MIDI_MACRO, 0x40, 0);
        assert_eq!(effect, TrackerEffect::SetFilterCutoff(0x40));

        // Filter resonance (Zxx where xx = 80-8F)
        // 0x85 = resonance 5, scaled by 8 for 0-127 range
        let effect = convert_it_effect(nether_it::effects::MIDI_MACRO, 0x85, 0);
        assert_eq!(effect, TrackerEffect::SetFilterResonance(5 * 8));
    }
}
