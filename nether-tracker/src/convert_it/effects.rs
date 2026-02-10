//! IT effect conversion to unified TrackerEffect

use crate::TrackerEffect;

/// Convert IT volume column (0-64 for direct volume, or volume effects)
pub(super) fn convert_it_volume(vol: u8) -> u8 {
    // Simple volume (0-64) is preserved
    // Volume effects are handled in convert_it_effect
    if vol <= 64 { vol } else { 0 }
}

/// Convert IT effect to unified TrackerEffect
pub(super) fn convert_it_effect(effect: u8, param: u8, volume: u8) -> TrackerEffect {
    // Check volume column for volume-column effects first
    if volume > 64
        && let Some(vol_effect) = convert_it_volume_effect(volume)
    {
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
        // EEx = Extra-fine portamento down (tick 0 only)
        // EFx = Fine portamento down (tick 0 only)
        nether_it::effects::PORTA_DOWN => {
            let hi = param >> 4;
            let lo = param & 0x0F;
            match hi {
                0xE => TrackerEffect::ExtraFinePortaDown(lo as u16),
                0xF => TrackerEffect::FinePortaDown(lo as u16),
                _ => TrackerEffect::PortamentoDown(param as u16),
            }
        }

        // Fxx - Portamento up
        // FEx = Extra-fine portamento up (tick 0 only)
        // FFx = Fine portamento up (tick 0 only)
        nether_it::effects::PORTA_UP => {
            let hi = param >> 4;
            let lo = param & 0x0F;
            match hi {
                0xE => TrackerEffect::ExtraFinePortaUp(lo as u16),
                0xF => TrackerEffect::FinePortaUp(lo as u16),
                _ => TrackerEffect::PortamentoUp(param as u16),
            }
        }

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
        // S2x - Set finetune: value 0-15 maps to -8..+7 semitones (centered at 8)
        nether_it::extended_effects::SET_FINETUNE => TrackerEffect::SetFinetune((value as i8) - 8),
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
        // S9x - Sound control (surround/reverse)
        // S90 = surround off, S91 = surround on
        // S9E = play forwards, S9F = play backwards (reverse)
        nether_it::extended_effects::SOUND_CONTROL => match value {
            0 => TrackerEffect::SetSurround(false),
            1 => TrackerEffect::SetSurround(true),
            0xE => TrackerEffect::SetSampleReverse(false),
            0xF => TrackerEffect::SetSampleReverse(true),
            _ => TrackerEffect::None, // S92-S9D are reserved/unused
        },
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
        assert_eq!(effect, TrackerEffect::VolumeSlide { up: 5, down: 2 });
    }

    #[test]
    fn test_convert_it_portamento_directions() {
        let down = convert_it_effect(nether_it::effects::PORTA_DOWN, 0x12, 0);
        assert_eq!(down, TrackerEffect::PortamentoDown(0x12));

        let up = convert_it_effect(nether_it::effects::PORTA_UP, 0x34, 0);
        assert_eq!(up, TrackerEffect::PortamentoUp(0x34));
    }

    #[test]
    fn test_convert_it_portamento_fine_and_extrafine() {
        let fine_down = convert_it_effect(nether_it::effects::PORTA_DOWN, 0xF3, 0);
        assert_eq!(fine_down, TrackerEffect::FinePortaDown(3));

        let xf_down = convert_it_effect(nether_it::effects::PORTA_DOWN, 0xE7, 0);
        assert_eq!(xf_down, TrackerEffect::ExtraFinePortaDown(7));

        let fine_up = convert_it_effect(nether_it::effects::PORTA_UP, 0xF2, 0);
        assert_eq!(fine_up, TrackerEffect::FinePortaUp(2));

        let xf_up = convert_it_effect(nether_it::effects::PORTA_UP, 0xE5, 0);
        assert_eq!(xf_up, TrackerEffect::ExtraFinePortaUp(5));
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

    #[test]
    fn test_convert_it_extended_s2x_finetune() {
        // S20 = finetune -8 (0 - 8)
        let ft_low = convert_it_extended_effect(0x20);
        assert_eq!(ft_low, TrackerEffect::SetFinetune(-8));

        // S28 = finetune 0 (8 - 8)
        let ft_center = convert_it_extended_effect(0x28);
        assert_eq!(ft_center, TrackerEffect::SetFinetune(0));

        // S2F = finetune +7 (15 - 8)
        let ft_high = convert_it_extended_effect(0x2F);
        assert_eq!(ft_high, TrackerEffect::SetFinetune(7));
    }

    #[test]
    fn test_convert_it_extended_s9x_sound_control() {
        // S90 = surround off
        let surround_off = convert_it_extended_effect(0x90);
        assert_eq!(surround_off, TrackerEffect::SetSurround(false));

        // S91 = surround on
        let surround_on = convert_it_extended_effect(0x91);
        assert_eq!(surround_on, TrackerEffect::SetSurround(true));

        // S9E = play forwards
        let forward = convert_it_extended_effect(0x9E);
        assert_eq!(forward, TrackerEffect::SetSampleReverse(false));

        // S9F = play backwards (reverse)
        let reverse = convert_it_extended_effect(0x9F);
        assert_eq!(reverse, TrackerEffect::SetSampleReverse(true));

        // S92-S9D are reserved/unused
        let reserved = convert_it_extended_effect(0x95);
        assert_eq!(reserved, TrackerEffect::None);
    }
}
