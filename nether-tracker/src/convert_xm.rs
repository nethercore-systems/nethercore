//! XM → TrackerModule conversion

use crate::*;

/// Convert an XM module to the unified TrackerModule format
pub fn from_xm_module(xm: &nether_xm::XmModule) -> TrackerModule {
    // Convert patterns
    let patterns = xm.patterns.iter().map(convert_xm_pattern).collect();

    // Convert instruments
    let instruments = xm.instruments.iter().map(convert_xm_instrument).collect();

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
        restart_position: xm.restart_position,
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
    if (0x10..=0x50).contains(&vol) {
        vol - 0x10
    } else {
        0
    }
}

/// Convert XM effect to unified TrackerEffect
fn convert_xm_effect(effect: u8, param: u8, volume: u8) -> TrackerEffect {
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

    // Convert sample loop type
    let sample_loop_type = match xm_instr.sample_loop_type {
        1 => LoopType::Forward,
        2 => LoopType::PingPong,
        _ => LoopType::None,
    };

    // Calculate original sample rate from finetune + relative_note
    // This is the same calculation used in audio_convert::convert_xm_sample()
    // Base frequency for C-4 (Amiga standard) = 8363 Hz
    let original_sample_rate = {
        const BASE_FREQ: f64 = 8363.0;
        let total_semitones = xm_instr.sample_relative_note as f64
            + (xm_instr.sample_finetune as f64 / 128.0);
        let freq = BASE_FREQ * 2.0_f64.powf(total_semitones / 12.0);
        (freq.round() as u32).clamp(100, 96000)
    };

    // Convert loop points to match the resampled 22050 Hz sample
    // If original was 44100 Hz, loop point 1000 becomes 500 after resampling
    const TARGET_RATE: u32 = 22050;
    let (sample_loop_start, sample_loop_length) = if original_sample_rate == TARGET_RATE {
        (xm_instr.sample_loop_start, xm_instr.sample_loop_length)
    } else {
        let ratio = TARGET_RATE as f64 / original_sample_rate as f64;
        let new_start = (xm_instr.sample_loop_start as f64 * ratio).round() as u32;
        let new_length = (xm_instr.sample_loop_length as f64 * ratio).round() as u32;
        (new_start, new_length)
    };

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
        pitch_pan_separation: 0, // XM doesn't have pitch-pan separation
        pitch_pan_center: 60,    // Default center = C-5

        // Sample loop points - CONVERTED to match resampled 22050 Hz sample
        // Original loop points are in the source sample's sample space, but the
        // sample data has been resampled during ROM packing.
        sample_loop_start,
        sample_loop_end: sample_loop_start + sample_loop_length,
        sample_loop_type,
        // Finetune and relative_note are ZEROED because they're already baked into the
        // resampled 22050 Hz sample during ROM packing (via audio_convert::convert_xm_sample).
        // If we passed them through here, they would be applied AGAIN during playback,
        // causing notes to play way too high (often 1+ octave off).
        sample_finetune: 0,
        sample_relative_note: 0,

        // Auto-vibrato settings
        auto_vibrato_type: xm_instr.vibrato_type,
        auto_vibrato_sweep: xm_instr.vibrato_sweep,
        auto_vibrato_depth: xm_instr.vibrato_depth,
        auto_vibrato_rate: xm_instr.vibrato_rate,
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

    #[test]
    fn test_convert_xm_instrument_sample_loop() {
        // Create an XM instrument with sample loop info
        // Using relative_note=17, finetune=-16 which gives ~22050 Hz sample rate
        // This means the sample doesn't need resampling, so loop points stay the same
        let xm_instr = nether_xm::XmInstrument {
            name: "Test".to_string(),
            num_samples: 1,
            sample_loop_start: 1000,
            sample_loop_length: 2000,
            sample_loop_type: 1, // Forward loop
            sample_finetune: -16,
            sample_relative_note: 17, // ~22050 Hz, so ratio ≈ 1
            volume_envelope: None,
            panning_envelope: None,
            volume_fadeout: 512,
            vibrato_type: 1,
            vibrato_sweep: 128,
            vibrato_depth: 16,
            vibrato_rate: 8,
        };

        let tracker_instr = convert_xm_instrument(&xm_instr);

        // Loop points should be approximately the same since sample rate ≈ 22050 Hz
        // (small rounding differences may occur)
        assert!(tracker_instr.sample_loop_start >= 990 && tracker_instr.sample_loop_start <= 1010);
        assert_eq!(tracker_instr.sample_loop_type, LoopType::Forward);
        // Finetune and relative_note are zeroed because pitch adjustment is baked
        // into the resampled 22050 Hz sample during ROM packing
        assert_eq!(tracker_instr.sample_finetune, 0);
        assert_eq!(tracker_instr.sample_relative_note, 0);

        // Verify auto-vibrato
        assert_eq!(tracker_instr.auto_vibrato_type, 1);
        assert_eq!(tracker_instr.auto_vibrato_sweep, 128);
        assert_eq!(tracker_instr.auto_vibrato_depth, 16);
        assert_eq!(tracker_instr.auto_vibrato_rate, 8);
    }

    #[test]
    fn test_convert_xm_instrument_loop_points_scaled() {
        // Test that loop points are properly scaled when resampling
        // Using relative_note=0, finetune=0 which gives 8363 Hz base sample rate
        // Ratio = 22050 / 8363 ≈ 2.636
        let xm_instr = nether_xm::XmInstrument {
            name: "ScaledLoop".to_string(),
            num_samples: 1,
            sample_loop_start: 1000,
            sample_loop_length: 2000,
            sample_loop_type: 1, // Forward loop
            sample_finetune: 0,
            sample_relative_note: 0, // 8363 Hz base rate
            volume_envelope: None,
            panning_envelope: None,
            volume_fadeout: 0,
            vibrato_type: 0,
            vibrato_sweep: 0,
            vibrato_depth: 0,
            vibrato_rate: 0,
        };

        let tracker_instr = convert_xm_instrument(&xm_instr);

        // Expected: loop_start = 1000 * (22050/8363) ≈ 2636
        // Expected: loop_length = 2000 * (22050/8363) ≈ 5273
        // Allow small tolerance for rounding
        assert!(
            tracker_instr.sample_loop_start >= 2630 && tracker_instr.sample_loop_start <= 2640,
            "Loop start should be ~2636, got {}",
            tracker_instr.sample_loop_start
        );
        let loop_length = tracker_instr.sample_loop_end - tracker_instr.sample_loop_start;
        assert!(
            loop_length >= 5265 && loop_length <= 5280,
            "Loop length should be ~5273, got {}",
            loop_length
        );
    }

    #[test]
    fn test_convert_xm_instrument_pingpong_loop() {
        let xm_instr = nether_xm::XmInstrument {
            name: "PingPong".to_string(),
            num_samples: 1,
            sample_loop_start: 500,
            sample_loop_length: 1500,
            sample_loop_type: 2, // Ping-pong loop
            sample_finetune: 0,
            sample_relative_note: 0,
            volume_envelope: None,
            panning_envelope: None,
            volume_fadeout: 0,
            vibrato_type: 0,
            vibrato_sweep: 0,
            vibrato_depth: 0,
            vibrato_rate: 0,
        };

        let tracker_instr = convert_xm_instrument(&xm_instr);
        assert_eq!(tracker_instr.sample_loop_type, LoopType::PingPong);
    }

    #[test]
    fn test_convert_xm_instrument_no_loop() {
        let xm_instr = nether_xm::XmInstrument {
            name: "NoLoop".to_string(),
            num_samples: 1,
            sample_loop_start: 0,
            sample_loop_length: 0,
            sample_loop_type: 0, // No loop
            sample_finetune: 0,
            sample_relative_note: 0,
            volume_envelope: None,
            panning_envelope: None,
            volume_fadeout: 0,
            vibrato_type: 0,
            vibrato_sweep: 0,
            vibrato_depth: 0,
            vibrato_rate: 0,
        };

        let tracker_instr = convert_xm_instrument(&xm_instr);
        assert_eq!(tracker_instr.sample_loop_type, LoopType::None);
    }
}
