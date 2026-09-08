//! Tests for XM conversion

use super::*;
use crate::{LoopType, TrackerEffect};

#[test]
fn xm_sample_maps_preserve_slots_beyond_255() {
    let instrument = nether_xm::XmInstrument {
        num_samples: 2,
        sample_map: vec![1; 96],
        samples: vec![
            nether_xm::XmSample {
                volume: 16,
                pan: 255,
                ..Default::default()
            };
            2
        ],
        ..Default::default()
    };
    let xm = nether_xm::XmModule {
        mix_mode: nether_xm::XmMixMode::Compatible,
        legacy_retrigger: false,
        sample_preamp: 48,
        name: String::new(),
        num_channels: 1,
        num_patterns: 0,
        num_instruments: 129,
        song_length: 0,
        restart_position: 0,
        default_speed: 6,
        default_bpm: 125,
        linear_frequency_table: true,
        order_table: vec![],
        patterns: vec![],
        instruments: vec![instrument; 129],
    };
    let converted = from_xm_module(&xm);
    assert_eq!(converted.samples.len(), 258);
    assert_eq!(converted.instruments[128].sample_for_note(60), Some(258));
    assert_eq!(converted.samples[257].default_volume, 16);
    assert_eq!(converted.samples[257].default_pan, Some(255));
}

#[test]
fn xm_zero_pan_slide_left_is_not_a_noop() {
    assert_eq!(
        effects::convert_xm_volume_effect(0xd0),
        Some(TrackerEffect::PanningLeftOnTicks)
    );
    // Keep E00 identity for legacy recall; standard XM leaves it inactive.
    assert_eq!(
        effects::convert_xm_volume_effect(0xe0),
        Some(TrackerEffect::PanningSlide { left: 0, right: 0 })
    );
}

#[test]
fn xm_zero_volume_slides_do_not_recall_memory() {
    for volume in [0x60, 0x70] {
        assert_eq!(effects::convert_xm_volume_effect(volume), None);
    }
    assert_eq!(
        effects::convert_xm_effect(0x0a, 0),
        TrackerEffect::VolumeSlide { up: 0, down: 0 }
    );
}

#[test]
fn xm_keyoff_parameter_is_not_discarded() {
    for tick in 1..=255 {
        assert_eq!(
            convert_xm_effect(nether_xm::effects::KEY_OFF, tick),
            TrackerEffect::KeyOffAt(tick)
        );
    }
}

#[test]
fn xm_global_volume_clamps_before_scaling() {
    for parameter in 0..=255u8 {
        assert_eq!(
            convert_xm_effect(nether_xm::effects::SET_GLOBAL_VOLUME, parameter),
            TrackerEffect::SetGlobalVolume(parameter.min(64) * 2)
        );
    }
}

#[test]
fn xm_dual_tone_portamento_uses_doubled_volume_speed() {
    for digit in 0..16u8 {
        for parameter in [0, 1, 127, 255] {
            let note = convert_xm_note(&nether_xm::XmNote {
                volume: 0xf0 + digit,
                effect: 3,
                effect_param: parameter,
                ..Default::default()
            });
            assert_eq!(note.effect, TrackerEffect::None);
            assert_eq!(
                note.volume_effect,
                TrackerEffect::TonePortamento(u16::from(digit) * 32)
            );
        }
    }
}

#[test]
fn xm_coarse_panning_covers_both_stereo_edges() {
    for value in 0..16u8 {
        let expected = ((u16::from(value) * 256 + 8) / 15 / 4) as u8;
        assert_eq!(
            convert_xm_effect(0x0e, 0x80 | value),
            TrackerEffect::SetPanning(expected)
        );
    }
}

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
fn simultaneous_columns_preserve_main_effect() {
    let note = super::convert_xm_note(&nether_xm::XmNote {
        volume: 0x61,
        effect: nether_xm::effects::SET_SPEED_TEMPO,
        effect_param: 3,
        ..Default::default()
    });
    assert_eq!(note.effect, TrackerEffect::SetSpeed(3));
}

#[test]
fn volume_column_command_layout() {
    assert_eq!(
        effects::convert_xm_volume_effect(0xC8),
        Some(TrackerEffect::SetPanning(32))
    );
    assert_eq!(
        effects::convert_xm_volume_effect(0xD2),
        Some(TrackerEffect::PanningSlide { left: 2, right: 0 })
    );
    assert_eq!(
        effects::convert_xm_volume_effect(0xE2),
        Some(TrackerEffect::PanningSlide { left: 0, right: 2 })
    );
    assert_eq!(
        effects::convert_xm_volume_effect(0xF2),
        Some(TrackerEffect::TonePortamento(32))
    );
}

#[test]
fn test_convert_xm_effect_speed() {
    let effect = effects::convert_xm_effect(nether_xm::effects::SET_SPEED_TEMPO, 6);
    assert_eq!(effect, TrackerEffect::SetSpeed(6));
}

#[test]
fn test_convert_xm_effect_tempo() {
    let effect = effects::convert_xm_effect(nether_xm::effects::SET_SPEED_TEMPO, 125);
    assert_eq!(effect, TrackerEffect::SetTempo(125));
}

#[test]
fn test_convert_xm_volume_slide() {
    let effect = effects::convert_xm_effect(nether_xm::effects::VOLUME_SLIDE, 0x52);
    assert_eq!(effect, TrackerEffect::VolumeSlide { up: 5, down: 2 });
}

#[test]
fn xm_single_sample_default_pan_preserves_all_source_positions() {
    for source in 0..=255u8 {
        let expected = source;
        let source = nether_xm::XmInstrument {
            num_samples: 1,
            sample_default_pan: Some(source),
            ..Default::default()
        };
        assert_eq!(
            instruments::convert_xm_instrument(&source).default_pan,
            Some(expected)
        );
        for mapped in [false, true] {
            let mut instrument = source.clone();
            if mapped {
                instrument.num_samples = 2;
                instrument.sample_map = vec![1; 96];
                instrument.samples = vec![
                    nether_xm::XmSample {
                        pan: expected,
                        ..Default::default()
                    };
                    2
                ];
            }
            let xm = nether_xm::XmModule {
                mix_mode: nether_xm::XmMixMode::Compatible,
                legacy_retrigger: false,
                sample_preamp: 48,
                name: String::new(),
                num_channels: 1,
                num_patterns: 0,
                num_instruments: 1,
                song_length: 0,
                restart_position: 0,
                default_speed: 6,
                default_bpm: 125,
                linear_frequency_table: true,
                order_table: vec![],
                patterns: vec![],
                instruments: vec![instrument],
            };
            let packed = nether_xm::pack_xm_minimal(&xm).unwrap();
            let loaded = nether_xm::parse_xm_minimal(&packed).unwrap();
            let converted = from_xm_module(&loaded);
            let pan = if mapped {
                converted.samples[1].default_pan
            } else {
                converted.instruments[0].default_pan
            };
            assert_eq!(pan, Some(expected));
        }
    }
}

#[test]
fn test_convert_xm_instrument_sample_loop() {
    // Create an XM instrument with sample loop info
    // Using relative_note=17, finetune=-16 which gives ~22050 Hz sample rate
    // This means the sample doesn't need resampling, so loop points stay the same
    let xm_instr = nether_xm::XmInstrument {
        source_sample_bytes: None,
        sample_map: Vec::new(),
        samples: Vec::new(),
        name: "Test".to_string(),
        num_samples: 1,
        sample_loop_start: 1000,
        sample_loop_length: 2000,
        sample_loop_type: 1, // Forward loop
        sample_is_stereo: false,
        sample_default_volume: None,
        sample_default_pan: None,
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

    let tracker_instr = instruments::convert_xm_instrument(&xm_instr);

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
    for (rate, expected) in [
        (0, 0),
        (512, 1024),
        (4096, 8192),
        (32767, 65534),
        (65535, 65535),
    ] {
        let source = nether_xm::XmInstrument {
            volume_fadeout: rate,
            ..xm_instr.clone()
        };
        assert_eq!(
            instruments::convert_xm_instrument(&source).fadeout,
            expected
        );
    }
}

#[test]
fn test_convert_xm_instrument_loop_points_scaled() {
    // Test that loop points are properly scaled when resampling
    // Using relative_note=0, finetune=0 which gives 8363 Hz base sample rate
    // Ratio = 22050 / 8363 ≈ 2.636
    let xm_instr = nether_xm::XmInstrument {
        source_sample_bytes: None,
        sample_map: Vec::new(),
        samples: Vec::new(),
        name: "ScaledLoop".to_string(),
        num_samples: 1,
        sample_loop_start: 1000,
        sample_loop_length: 2000,
        sample_loop_type: 1, // Forward loop
        sample_is_stereo: false,
        sample_default_volume: None,
        sample_default_pan: None,
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

    let tracker_instr = instruments::convert_xm_instrument(&xm_instr);

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
        (5265..=5280).contains(&loop_length),
        "Loop length should be ~5273, got {}",
        loop_length
    );
}

#[test]
fn test_convert_xm_instrument_pingpong_loop() {
    let xm_instr = nether_xm::XmInstrument {
        source_sample_bytes: None,
        sample_map: Vec::new(),
        samples: Vec::new(),
        name: "PingPong".to_string(),
        num_samples: 1,
        sample_loop_start: 500,
        sample_loop_length: 1500,
        sample_loop_type: 2, // Ping-pong loop
        sample_is_stereo: false,
        sample_default_volume: None,
        sample_default_pan: None,
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

    let tracker_instr = instruments::convert_xm_instrument(&xm_instr);
    assert_eq!(tracker_instr.sample_loop_type, LoopType::PingPong);
}

#[test]
fn test_convert_xm_instrument_no_loop() {
    let xm_instr = nether_xm::XmInstrument {
        source_sample_bytes: None,
        sample_map: Vec::new(),
        samples: Vec::new(),
        name: "NoLoop".to_string(),
        num_samples: 1,
        sample_loop_start: 0,
        sample_loop_length: 0,
        sample_loop_type: 0, // No loop
        sample_is_stereo: false,
        sample_default_volume: None,
        sample_default_pan: None,
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

    let tracker_instr = instruments::convert_xm_instrument(&xm_instr);
    assert_eq!(tracker_instr.sample_loop_type, LoopType::None);
}

#[test]
fn test_convert_xm_extended_e8x_coarse_panning() {
    // E80/E8F reach the hard edges; E88 lies just right of center.
    let left = effects::convert_xm_extended_effect(0x80);
    assert_eq!(left, TrackerEffect::SetPanning(0));

    let center = effects::convert_xm_extended_effect(0x88);
    assert_eq!(center, TrackerEffect::SetPanning(34));

    let right = effects::convert_xm_extended_effect(0x8F);
    assert_eq!(right, TrackerEffect::SetPanning(64));
}

#[test]
fn test_convert_xm_extended_e9x_retrigger() {
    // E93 = retrigger every 3 ticks
    let effect = effects::convert_xm_extended_effect(0x93);
    assert_eq!(
        effect,
        TrackerEffect::Retrigger {
            ticks: 3,
            volume_change: 0
        }
    );

    // E90 = no retrigger (0 ticks)
    let no_retrigger = effects::convert_xm_extended_effect(0x90);
    assert_eq!(
        no_retrigger,
        TrackerEffect::Retrigger {
            ticks: 0,
            volume_change: 0
        }
    );
}

#[test]
fn test_convert_xm_volume_column_vibrato() {
    // 0xB0 = vibrato depth 0 (speed from memory)
    let vib0 = effects::convert_xm_volume_effect(0xB0);
    assert_eq!(vib0, Some(TrackerEffect::Vibrato { speed: 0, depth: 0 }));

    // 0xB8 = vibrato depth 8
    let vib8 = effects::convert_xm_volume_effect(0xB8);
    assert_eq!(vib8, Some(TrackerEffect::Vibrato { speed: 0, depth: 8 }));

    // 0xBF = vibrato depth 15
    let vib15 = effects::convert_xm_volume_effect(0xBF);
    assert_eq!(
        vib15,
        Some(TrackerEffect::Vibrato {
            speed: 0,
            depth: 15
        })
    );
}

#[test]
fn xm_pan_envelope_converts_to_signed_offsets_without_changing_volume() {
    let env = nether_xm::XmEnvelope {
        points: vec![(0, 0), (4, 32), (8, 64)],
        enabled: true,
        ..Default::default()
    };
    let source = nether_xm::XmInstrument {
        volume_envelope: Some(env.clone()),
        panning_envelope: Some(env),
        ..Default::default()
    };
    let converted = super::instruments::convert_xm_instrument(&source);
    assert_eq!(
        converted.panning_envelope.unwrap().points,
        vec![(0, -32), (4, 0), (8, 32)]
    );
    assert_eq!(
        converted.volume_envelope.unwrap().points,
        vec![(0, 0), (4, 32), (8, 64)]
    );
}

#[test]
fn xm_glissando_and_tremor_are_not_discarded() {
    assert_eq!(
        convert_xm_effect(0x0e, 0x31),
        TrackerEffect::SetGlissando(true)
    );
    assert_eq!(
        convert_xm_effect(0x0e, 0x30),
        TrackerEffect::SetGlissando(false)
    );
    for parameter in 0..=255 {
        assert_eq!(
            convert_xm_effect(0x1d, parameter),
            TrackerEffect::Tremor {
                ontime: parameter >> 4,
                offtime: parameter & 15
            }
        );
    }
}
