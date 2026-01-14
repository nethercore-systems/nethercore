//! XM effect conversion functions

use crate::TrackerEffect;

/// Convert XM volume column (0x10-0x50 for volume, others for effects)
pub fn convert_xm_volume(vol: u8) -> u8 {
    // XM volume column: 0x10-0x50 = volume 0-64
    if (0x10..=0x50).contains(&vol) {
        vol - 0x10
    } else {
        0
    }
}

/// Convert XM effect to unified TrackerEffect
pub fn convert_xm_effect(effect: u8, param: u8, volume: u8) -> TrackerEffect {
    // Check volume column for volume-column effects first
    if volume != 0
        && !(0x10..=0x50).contains(&volume)
        && let Some(vol_effect) = convert_xm_volume_effect(volume)
    {
        return vol_effect;
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
pub(super) fn convert_xm_extended_effect(param: u8) -> TrackerEffect {
    let sub_cmd = param >> 4;
    let value = param & 0x0F;

    match sub_cmd {
        0x1 => TrackerEffect::FinePortaUp(value as u16),
        0x2 => TrackerEffect::FinePortaDown(value as u16),
        0x4 => TrackerEffect::VibratoWaveform(value),
        0x5 => TrackerEffect::SetFinetune(value as i8),
        0x6 => TrackerEffect::PatternLoop(value),
        0x7 => TrackerEffect::TremoloWaveform(value),
        // E8x - Set coarse panning (0-F maps to panning 0-60, centered)
        0x8 => TrackerEffect::SetPanning((value * 4).saturating_add(2).min(64)),
        // E9x - Retrigger note every x ticks (0 = no retrigger)
        0x9 => TrackerEffect::Retrigger {
            ticks: value,
            volume_change: 0,
        },
        0xA => TrackerEffect::FineVolumeUp(value),
        0xB => TrackerEffect::FineVolumeDown(value),
        0xC => TrackerEffect::NoteCut(value),
        0xD => TrackerEffect::NoteDelay(value),
        0xE => TrackerEffect::PatternDelay(value),
        _ => TrackerEffect::None,
    }
}

/// Convert XM volume column effects
pub(super) fn convert_xm_volume_effect(vol: u8) -> Option<TrackerEffect> {
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
        // 0xE0-0xEF: Vibrato depth (speed uses memory)
        0xE0..=0xEF => Some(TrackerEffect::Vibrato {
            speed: 0, // Use memory for speed
            depth: vol - 0xE0,
        }),
        _ => None,
    }
}
