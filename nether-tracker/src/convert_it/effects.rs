//! IT effect conversion to unified TrackerEffect

use crate::TrackerEffect;

/// Convert IT volume column (0-64 for direct volume, or volume effects)
pub(super) fn convert_it_volume(vol: u8) -> u8 {
    // Simple volume (0-64) is preserved
    // Volume effects are handled in convert_it_effect
    if vol <= 64 { vol } else { 0 }
}

/// Convert IT effect to unified TrackerEffect
pub(super) fn convert_it_effect(effect: u8, param: u8) -> TrackerEffect {
    // Volume-column commands are retained separately on TrackerNote.
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

        // Preserve the complete E/F byte: E00/F00 recalls the mode nibble too.
        nether_it::effects::PORTA_DOWN => TrackerEffect::PortamentoDown(param as u16),
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

        // Sxy - retain the raw command until runtime so S00 can recall S memory.
        nether_it::effects::EXTENDED => TrackerEffect::ItExtended(param),

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

        // Xxx - 8-bit panning: X00 = left, X80 = center, XFF = near right.
        nether_it::effects::SET_PANNING => TrackerEffect::SetPanning(param / 4),

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

#[cfg(test)]
fn convert_it_extended_effect(param: u8) -> TrackerEffect {
    TrackerEffect::from_it_extended(param)
}

/// Convert IT volume column effects
pub(super) fn convert_it_volume_effect(vol: u8) -> Option<TrackerEffect> {
    match vol {
        0..=64 => Some(TrackerEffect::SetVolume(vol)),
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
        105..=114 => Some(TrackerEffect::PortamentoDown((vol - 105) as u16 * 4)),
        115..=124 => Some(TrackerEffect::PortamentoUp((vol - 115) as u16 * 4)),
        128..=192 => Some(TrackerEffect::SetPanning(vol - 128)), // 0-64 panning
        // IT volume G0..G9 uses the original tracker's nonlinear speed table.
        193..=202 => Some(TrackerEffect::TonePortamento(
            [0, 1, 4, 8, 16, 32, 64, 96, 128, 255][(vol - 193) as usize],
        )),
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
    fn it_extended_commands_stay_raw_until_runtime() {
        assert_eq!(
            convert_it_effect(nether_it::effects::EXTENDED, 0),
            TrackerEffect::ItExtended(0)
        );
        assert_eq!(
            convert_it_effect(nether_it::effects::EXTENDED, 0x62),
            TrackerEffect::ItExtended(0x62)
        );
    }

    #[test]
    fn it_volume_pitch_slides_are_regular_tick_slides() {
        assert_eq!(
            convert_it_extended_effect(0x7b),
            TrackerEffect::SetPitchEnvelope(false)
        );
        assert_eq!(
            convert_it_extended_effect(0x7c),
            TrackerEffect::SetPitchEnvelope(true)
        );
        assert_eq!(
            convert_it_extended_effect(0x79),
            TrackerEffect::SetPanningEnvelope(false)
        );
        assert_eq!(
            convert_it_extended_effect(0x7a),
            TrackerEffect::SetPanningEnvelope(true)
        );
        assert_eq!(
            convert_it_extended_effect(0x77),
            TrackerEffect::SetVolumeEnvelope(false)
        );
        assert_eq!(
            convert_it_extended_effect(0x78),
            TrackerEffect::SetVolumeEnvelope(true)
        );
        for amount in 0..=9 {
            assert_eq!(
                convert_it_volume_effect(105 + amount),
                Some(TrackerEffect::PortamentoDown(amount as u16 * 4))
            );
            assert_eq!(
                convert_it_volume_effect(115 + amount),
                Some(TrackerEffect::PortamentoUp(amount as u16 * 4))
            );
        }
    }

    #[test]
    fn it_volume_tone_portamento_uses_tracker_speed_table() {
        for (digit, speed) in [0, 1, 4, 8, 16, 32, 64, 96, 128, 255]
            .into_iter()
            .enumerate()
        {
            assert_eq!(
                convert_it_volume_effect(193 + digit as u8),
                Some(TrackerEffect::TonePortamento(speed)),
                "volume G{digit}"
            );
        }
    }

    #[test]
    fn coarse_panning_covers_both_stereo_edges() {
        for value in 0..16u8 {
            let expected = ((u16::from(value) * 256 + 8) / 15 / 4) as u8;
            assert_eq!(
                convert_it_extended_effect(0x80 | value),
                TrackerEffect::SetPanning(expected)
            );
        }
    }

    #[test]
    fn test_convert_it_xxx_panning() {
        // IT Xxx is 8-bit (X80 = center), quantized to our 0-64 pan scale.
        // In particular, XA4 is ordinary IT panning, not S3M surround.
        for (param, expected) in [
            (0x00, 0),
            (0x01, 0),
            (0x03, 0),
            (0x04, 1),
            (0x7F, 31),
            (0x80, 32),
            (0x81, 32),
            (0x83, 32),
            (0x84, 33),
            (0xA4, 41),
            (0xFB, 62),
            (0xFC, 63),
            (0xFE, 63),
            (0xFF, 63),
        ] {
            assert_eq!(
                convert_it_effect(nether_it::effects::SET_PANNING, param),
                TrackerEffect::SetPanning(expected),
                "X{param:02X}"
            );
        }
    }

    #[test]
    fn test_convert_it_effect_speed() {
        let effect = convert_it_effect(nether_it::effects::SET_SPEED, 6);
        assert_eq!(effect, TrackerEffect::SetSpeed(6));
    }

    #[test]
    fn test_convert_it_effect_volume_slide() {
        let effect = convert_it_effect(nether_it::effects::VOLUME_SLIDE, 0x52); // Up 5, down 2
        assert_eq!(effect, TrackerEffect::VolumeSlide { up: 5, down: 2 });
    }

    #[test]
    fn test_convert_it_portamento_directions() {
        let down = convert_it_effect(nether_it::effects::PORTA_DOWN, 0x12);
        assert_eq!(down, TrackerEffect::PortamentoDown(0x12));

        let up = convert_it_effect(nether_it::effects::PORTA_UP, 0x34);
        assert_eq!(up, TrackerEffect::PortamentoUp(0x34));
    }

    #[test]
    fn test_convert_it_portamento_fine_and_extrafine() {
        let fine_down = convert_it_effect(nether_it::effects::PORTA_DOWN, 0xF3);
        assert_eq!(fine_down, TrackerEffect::PortamentoDown(0xF3));

        let xf_down = convert_it_effect(nether_it::effects::PORTA_DOWN, 0xE7);
        assert_eq!(xf_down, TrackerEffect::PortamentoDown(0xE7));

        let fine_up = convert_it_effect(nether_it::effects::PORTA_UP, 0xF2);
        assert_eq!(fine_up, TrackerEffect::PortamentoUp(0xF2));

        let xf_up = convert_it_effect(nether_it::effects::PORTA_UP, 0xE5);
        assert_eq!(xf_up, TrackerEffect::PortamentoUp(0xE5));
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
        assert_eq!(
            convert_it_volume_effect(32),
            Some(TrackerEffect::SetVolume(32))
        );
        assert_eq!(
            convert_it_volume_effect(0),
            Some(TrackerEffect::SetVolume(0))
        );
        assert_eq!(convert_it_volume_effect(255), None);
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
        let effect = convert_it_effect(nether_it::effects::MIDI_MACRO, 0x40);
        assert_eq!(effect, TrackerEffect::SetFilterCutoff(0x40));

        // Filter resonance (Zxx where xx = 80-8F)
        // 0x85 = resonance 5, scaled by 8 for 0-127 range
        let effect = convert_it_effect(nether_it::effects::MIDI_MACRO, 0x85);
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
