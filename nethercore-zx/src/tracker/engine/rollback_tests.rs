// Authored tests build defaults incrementally to isolate each control.
#![allow(clippy::field_reassign_with_default)]
//! Original rollback regressions: compare native execution, not a reference player's state.
use super::*;
use crate::audio::{Sound, generate_audio_frame_with_tracker};
use crate::state::AudioPlaybackState;
use crate::tracker::raw_tracker_handle;
use std::sync::Arc;

#[test]
fn it_effect_gate_mid_tick_snapshot_cold_silent_replay() {
    // The same failing families and flag pairs as effect_memory_probe.rs.
    for old in [false, true] {
        for family in [
            "D-fine",
            "H",
            "H-column",
            "I",
            "J",
            "K",
            "Q",
            "R",
            "S-cut",
            "T-channel",
            "U",
        ] {
            let (template, initial, sounds) = setup_it_rollback();
            let mut song = template.modules[raw_tracker_handle(initial.handle) as usize]
                .as_ref()
                .unwrap()
                .module
                .clone();
            song.format =
                FormatFlags::IS_IT_FORMAT | FormatFlags::INSTRUMENTS | FormatFlags::LINEAR_SLIDES;
            if old {
                song.format = song.format | FormatFlags::OLD_EFFECTS | FormatFlags::LINK_G_MEMORY;
            }
            song.initial_speed = 6;
            song.initial_tempo = 125;
            song.samples[0].default_volume = 32;
            song.instruments[0].nna = nether_tracker::NewNoteAction::Cut;
            song.instruments[0].random_volume = 0;
            song.instruments[0].random_pan = 0;
            song.instruments[0].volume_envelope = None;
            let mut notes = vec![TrackerNote {
                note: 49,
                instrument: 1,
                ..Default::default()
            }];
            for index in 1..6 {
                let value = if index == 1 { 4 } else { 0 };
                let effect = match family {
                    "D-fine" if index == 1 => TrackerEffect::FineVolumeDown(2),
                    "D-fine" => TrackerEffect::VolumeSlide { up: 0, down: 0 },
                    "H" | "H-column" => TrackerEffect::Vibrato {
                        speed: if index == 1 { 3 } else { 0 },
                        depth: value,
                    },
                    "I" => TrackerEffect::Tremor {
                        ontime: if index == 1 { 2 } else { 0 },
                        offtime: if index == 1 { 1 } else { 0 },
                    },
                    "J" => TrackerEffect::Arpeggio {
                        note1: if index == 1 { 3 } else { 0 },
                        note2: if index == 1 { 7 } else { 0 },
                    },
                    "K" if index == 1 => TrackerEffect::Vibrato { speed: 3, depth: 4 },
                    "K" => TrackerEffect::VibratoVolSlide {
                        vib_speed: 0,
                        vib_depth: 0,
                        vol_up: 0,
                        vol_down: if index == 2 { 1 } else { 0 },
                    },
                    "Q" => TrackerEffect::Retrigger {
                        ticks: if index == 1 { 3 } else { 0 },
                        volume_change: if index == 1 { 9 } else { 0 },
                    },
                    "R" => TrackerEffect::Tremolo {
                        speed: if index == 1 { 3 } else { 0 },
                        depth: value,
                    },
                    "S-cut" => TrackerEffect::ItExtended(if index == 1 { 0xc2 } else { 0 }),
                    "T-channel" if index == 1 => TrackerEffect::TempoSlideUp(1),
                    "T-channel" => TrackerEffect::TempoSlideDown(0),
                    "U" => TrackerEffect::FineVibrato {
                        speed: if index == 1 { 3 } else { 0 },
                        depth: value,
                    },
                    _ => unreachable!(),
                };
                notes.push(TrackerNote {
                    effect,
                    note: if family == "S-cut" { 49 } else { 0 },
                    instrument: if family == "S-cut" { 1 } else { 0 },
                    volume_effect: if family == "H-column" {
                        TrackerEffect::VolumeSlide { up: 0, down: 1 }
                    } else {
                        TrackerEffect::None
                    },
                    ..Default::default()
                });
            }
            notes.push(TrackerNote {
                note: 61,
                instrument: 1,
                volume: 32,
                ..Default::default()
            });
            notes.push(row(TrackerEffect::None));
            song.patterns = vec![pattern(notes)];
            song.order_table = vec![0];
            if family == "T-channel" {
                song.num_channels = 2;
                for notes in &mut song.patterns[0].notes {
                    notes.push(TrackerNote::default());
                }
                song.patterns[0].notes[1][1].effect = TrackerEffect::TempoSlideDown(2);
            }
            let mut engine = TrackerEngine::new();
            let handle = engine.load_tracker_module(song.clone(), vec![1]);
            let mut state = make_state(handle, 6, 125);
            engine.sync_to_state(&state, &sounds);
            while (state.row, state.tick, state.tick_sample_pos) != (3, 2, 317) {
                assert!(state.flags & tracker_flags::PLAYING != 0);
                engine.render_sample_and_advance(&mut state, &sounds, 44100);
            }
            let channel = &engine.channels[0];
            match family {
                "D-fine" => assert_eq!(channel.volume, 26.0 / 64.0, "DF2 memory"),
                "Q" => assert_eq!(
                    channel.volume,
                    37.0 / 64.0,
                    "Q93/Q00 volume opcode and tick-zero recall"
                ),
                "R" => assert_eq!(channel.volume, 0.5, "Rxy must preserve base volume"),
                "T-channel" => assert_eq!(state.bpm, 127, "channel-ordered tempo slides"),
                "S-cut" => assert!(!channel.note_on),
                _ => {}
            }
            let saved = engine.snapshot();
            let boundary = state;
            let expected: Vec<_> = (0..20000)
                .map(|_| engine.render_sample_and_advance(&mut state, &sounds, 44100))
                .collect();
            assert!(expected.iter().any(|&(l, r)| l != 0.0 || r != 0.0));
            let final_channels = format!("{:?}", engine.channels);
            for cold in [false, true, false] {
                let mut receiver = TrackerEngine::new();
                receiver.load_tracker_module(song.clone(), vec![1]);
                let mut replay = boundary;
                if cold {
                    receiver.sync_to_state(&replay, &sounds);
                } else {
                    receiver.apply_snapshot(&saved);
                }
                if family == "R" {
                    let a = &receiver.channels[0];
                    let b = &saved.channels[0];
                    assert_eq!(
                        (
                            a.tremolo_pos,
                            a.it_tremolo_delta,
                            a.it_tremolo_from,
                            a.it_tremolo_ramp_pos
                        ),
                        (
                            b.tremolo_pos,
                            b.it_tremolo_delta,
                            b.it_tremolo_from,
                            b.it_tremolo_ramp_pos
                        ),
                        "R checkpoint cold={cold}"
                    );
                }
                let actual: Vec<_> = (0..20000)
                    .map(|_| receiver.render_sample_and_advance(&mut replay, &sounds, 44100))
                    .collect();
                assert!(
                    actual == expected,
                    "{family} old={old} cold={cold}; first mismatch {:?}",
                    actual.iter().zip(&expected).position(|(a, b)| a != b)
                );
                assert_eq!(format!("{:?}", receiver.channels), final_channels);
                assert_eq!(bytemuck::bytes_of(&replay), bytemuck::bytes_of(&state));
            }
            let mut silent = TrackerEngine::new();
            silent.apply_snapshot(&saved);
            let mut replay = boundary;
            silent.advance_positions(&mut replay, &sounds, 20000, 44100);
            assert_eq!(
                format!("{:?}", silent.channels),
                final_channels,
                "silent {family} old={old}"
            );
            assert_eq!(bytemuck::bytes_of(&replay), bytemuck::bytes_of(&state));
            for _ in 0..2048 {
                assert_eq!(
                    silent.render_sample_and_advance(&mut replay, &sounds, 44100),
                    engine.render_sample_and_advance(&mut state, &sounds, 44100)
                );
            }
        }
    }
}

#[test]
fn it_tempo_memory_mid_tick_snapshot_cold_silent_repeated_replay() {
    let (template, initial, sounds) = setup_it_rollback();
    let mut song = template.modules[raw_tracker_handle(initial.handle) as usize]
        .as_ref()
        .unwrap()
        .module
        .clone();
    song.initial_speed = 6;
    song.initial_tempo = 125;
    song.patterns = vec![pattern(vec![
        TrackerNote {
            note: 49,
            instrument: 1,
            ..Default::default()
        },
        row(TrackerEffect::TempoSlideUp(1)),
        row(TrackerEffect::None),
        row(TrackerEffect::TempoSlideDown(0)), // T00 recalls up after an empty row.
        row(TrackerEffect::TempoSlideUp(0)),   // T10 is nonzero raw parameter: stop sliding.
        row(TrackerEffect::TempoSlideDown(0)),
        row(TrackerEffect::TempoSlideDown(2)),
        row(TrackerEffect::TempoSlideDown(0)),
        row(TrackerEffect::SetTempo(150)),
        row(TrackerEffect::TempoSlideDown(0)),
        row(TrackerEffect::None),
    ])];
    song.num_channels = 2;
    for notes in &mut song.patterns[0].notes {
        notes.push(TrackerNote::default());
    }
    song.patterns[0].notes[8][1].effect = TrackerEffect::SetTempo(140);
    song.order_table = vec![0];
    let mut engine = TrackerEngine::new();
    let handle = engine.load_tracker_module(song.clone(), vec![1]);
    let mut state = make_state(handle, 6, 125);
    engine.sync_to_state(&state, &sounds);
    let mut previous_row = u16::MAX;
    let mut checkpoints = 0;
    while state.flags & tracker_flags::PLAYING != 0 {
        engine.render_sample_and_advance(&mut state, &sounds, 44100);
        if state.flags & tracker_flags::PLAYING == 0 {
            break;
        }
        if state.row != previous_row {
            assert_eq!(
                state.bpm,
                [125, 125, 130, 130, 135, 135, 135, 125, 115, 140, 150][state.row as usize],
                "row {}",
                state.row
            );
            previous_row = state.row;
        }
        if state.row == 3 && state.tick == 2 && state.tick_sample_pos == 317 {
            assert_eq!(
                state.bpm, 132,
                "T00 must recall the preceding T11 after an empty row"
            );
            let saved = engine.snapshot();
            let boundary = state;
            let expected: Vec<_> = (0..32000)
                .map(|_| engine.render_sample_and_advance(&mut state, &sounds, 44100))
                .collect();
            assert!(expected.iter().any(|&(l, r)| l != 0.0 || r != 0.0));
            let final_channels = format!("{:?}", engine.channels);
            let final_state = state;
            for cold in [false, true] {
                for _ in 0..2 {
                    let mut receiver = TrackerEngine::new();
                    receiver.load_tracker_module(song.clone(), vec![1]);
                    let mut replay = boundary;
                    if cold {
                        receiver.sync_to_state(&replay, &sounds);
                    } else {
                        receiver.apply_snapshot(&saved);
                    }
                    assert_eq!(
                        format!("{:?}", receiver.channels),
                        format!("{:?}", saved.channels)
                    );
                    let actual: Vec<_> = (0..32000)
                        .map(|_| receiver.render_sample_and_advance(&mut replay, &sounds, 44100))
                        .collect();
                    assert_eq!(actual, expected, "cold={cold}");
                    assert_eq!(format!("{:?}", receiver.channels), final_channels);
                    assert_eq!(
                        bytemuck::bytes_of(&replay),
                        bytemuck::bytes_of(&final_state)
                    );
                }
            }
            let mut silent = TrackerEngine::new();
            silent.apply_snapshot(&saved);
            let mut replay = boundary;
            silent.advance_positions(&mut replay, &sounds, 32000, 44100);
            assert_eq!(format!("{:?}", silent.channels), final_channels);
            assert_eq!(
                bytemuck::bytes_of(&replay),
                bytemuck::bytes_of(&final_state)
            );
            let audible_tail: Vec<_> = (0..2048)
                .map(|_| engine.render_sample_and_advance(&mut state, &sounds, 44100))
                .collect();
            assert!(audible_tail.iter().any(|&(l, r)| l != 0.0 || r != 0.0));
            let silent_tail: Vec<_> = (0..2048)
                .map(|_| silent.render_sample_and_advance(&mut replay, &sounds, 44100))
                .collect();
            assert_eq!(silent_tail, audible_tail);
            engine.apply_snapshot(&saved);
            state = boundary;
            checkpoints += 1;
        }
    }
    assert_eq!(checkpoints, 1);
}

#[test]
fn xm_initial_attack_stop_snapshot_silent_and_cold_replay() {
    for rate in [44100, 48000, 96000] {
        for mode in 0..3 {
            for stereo in [false, true] {
                let (template, initial, _) = setup_rollback();
                let mut song = template.modules[raw_tracker_handle(initial.handle) as usize]
                    .as_ref()
                    .unwrap()
                    .module
                    .clone();
                song.format = FormatFlags::IS_XM_FORMAT
                    | FormatFlags::INSTRUMENTS
                    | FormatFlags::LINEAR_SLIDES;
                if mode == 0 {
                    song.format = song.format | FormatFlags::XM_FT2_MIX;
                }
                if mode == 1 {
                    song.format = song.format | FormatFlags::XM_LEGACY_RETRIGGER;
                }
                song.initial_speed = 1;
                song.initial_tempo = 125;
                let instr = &mut song.instruments[0];
                instr.sample_is_stereo = stereo;
                instr.sample_loop_type = nether_tracker::LoopType::Forward;
                instr.sample_loop_start = 0;
                instr.sample_loop_end = 16;
                instr.xm_forward_loop_start = 8.0;
                instr.xm_forward_loop_limit = 0.25;
                song.patterns = vec![pattern(vec![
                    TrackerNote {
                        note: 89,
                        instrument: 1,
                        ..Default::default()
                    },
                    TrackerNote {
                        instrument: 1,
                        ..Default::default()
                    },
                    TrackerNote {
                        note: 25,
                        instrument: 1,
                        ..Default::default()
                    },
                    row(TrackerEffect::Retrigger {
                        ticks: 1,
                        volume_change: 0,
                    }),
                    row(TrackerEffect::None),
                    row(TrackerEffect::None),
                ])];
                song.order_table = vec![0];
                let sounds = vec![
                    None,
                    Some(Sound {
                        data: Arc::new(
                            (0..32)
                                .flat_map(|_| {
                                    if stereo {
                                        vec![8192, -4096]
                                    } else {
                                        vec![8192]
                                    }
                                })
                                .collect(),
                        ),
                    }),
                ];
                let mut engine = TrackerEngine::new();
                let handle = engine.load_tracker_module(song.clone(), vec![1]);
                let mut state = make_state(handle, 1, 125);
                engine.sync_to_state_at_rate(&state, &sounds, rate);
                let first = engine.render_sample_and_advance(&mut state, &sounds, rate);
                assert!(
                    first.0 > 0.0,
                    "initial attack rate={rate} mode={mode} stereo={stereo}"
                );
                let mut elapsed = 1;
                for checkpoint in [1, 10, 1000] {
                    engine.advance_positions(&mut state, &sounds, checkpoint - elapsed, rate);
                    elapsed = checkpoint;
                    if checkpoint == 10 {
                        assert!(engine.channels[0].xm_loop_stopped);
                    }
                    let saved = engine.snapshot();
                    let boundary = state;
                    let expected: Vec<_> = (0..8000)
                        .map(|_| engine.render_sample_and_advance(&mut state, &sounds, rate))
                        .collect();
                    assert!(expected[..100].iter().any(|&(l, _)| l > 0.0));
                    let final_channels = format!("{:?}", engine.channels);
                    let final_state = state;
                    for cold in [false, true] {
                        let mut receiver = TrackerEngine::new();
                        receiver.load_tracker_module(song.clone(), vec![1]);
                        let mut replay = boundary;
                        if cold {
                            receiver.sync_to_state_at_rate(&replay, &sounds, rate);
                        } else {
                            receiver.apply_snapshot(&saved);
                        }
                        assert_eq!(
                            format!("{:?}", receiver.channels),
                            format!("{:?}", saved.channels)
                        );
                        let actual: Vec<_> = (0..8000)
                            .map(|_| receiver.render_sample_and_advance(&mut replay, &sounds, rate))
                            .collect();
                        assert_eq!(
                            actual, expected,
                            "rate={rate} mode={mode} stereo={stereo} cold={cold}"
                        );
                        assert_eq!(format!("{:?}", receiver.channels), final_channels);
                        assert_eq!(
                            bytemuck::bytes_of(&replay),
                            bytemuck::bytes_of(&final_state)
                        );
                    }
                    let mut silent = TrackerEngine::new();
                    silent.apply_snapshot(&saved);
                    let mut replay = boundary;
                    silent.advance_positions(&mut replay, &sounds, 8000, rate);
                    assert_eq!(format!("{:?}", silent.channels), final_channels);
                    assert_eq!(
                        bytemuck::bytes_of(&replay),
                        bytemuck::bytes_of(&final_state)
                    );
                    engine.apply_snapshot(&saved);
                    state = boundary;
                }
            }
        }
    }
}

#[test]
fn xm_envelope_transition_snapshot_silent_and_cold_replay_at_output_rates() {
    for rate in [44100, 48000, 96000] {
        for ft2 in [false, true] {
            let (template, initial, mut sounds) = setup_rollback();
            let mut song = template.modules[raw_tracker_handle(initial.handle) as usize]
                .as_ref()
                .unwrap()
                .module
                .clone();
            if ft2 {
                song.format = song.format | FormatFlags::XM_FT2_MIX;
            } else {
                song.format =
                    song.format | FormatFlags::XM_LEGACY_MIX | FormatFlags::XM_LEGACY_RETRIGGER;
            }
            // Distinct stereo lanes catch state that only advances on audible
            // mono playback. The existing delayed note resets both envelopes.
            song.instruments[0].sample_is_stereo = true;
            song.instruments[0].panning_envelope = Some(TrackerEnvelope {
                points: vec![(0, 0), (2, -24), (5, 32)],
                flags: EnvelopeFlags::ENABLED | EnvelopeFlags::LOOP,
                loop_begin: 0,
                loop_end: 2,
                ..Default::default()
            });
            song.patterns[0].notes[3][0].effect = TrackerEffect::KeyOff;
            song.instruments[0].fadeout = 128;
            sounds[1] = Some(Sound {
                data: Arc::new(vec![
                    8192, -4096, 4096, -8192, 6144, -2048, 2048, -6144, 8192, -4096, 4096, -8192,
                    6144, -2048,
                ]),
            });
            let mut engine = TrackerEngine::new();
            let handle = engine.load_tracker_module(song.clone(), vec![1]);
            let mut state = make_state(handle, 3, 137);
            state.flags |= tracker_flags::LOOPING;
            engine.sync_to_state_at_rate(&state, &sounds, rate);
            let spt = samples_per_tick(137, rate);
            let mut elapsed = 0;
            for checkpoint in [spt + spt / 2, spt * 7 + 17, spt * 10 + 31] {
                for _ in elapsed..checkpoint {
                    engine.render_sample_and_advance(&mut state, &sounds, rate);
                }
                elapsed = checkpoint;
                if checkpoint == spt * 7 + 17 {
                    assert!(engine.channels[0].delayed_xm_note.is_some());
                }
                let saved = engine.snapshot();
                let boundary = state;
                let expected: Vec<_> = (0..8000)
                    .map(|_| engine.render_sample_and_advance(&mut state, &sounds, rate))
                    .collect();
                assert!(
                    expected
                        .iter()
                        .any(|&(l, r)| l != 0.0 && r != 0.0 && l != r)
                );
                let final_channels = format!("{:?}", engine.channels);
                let final_state = state;
                for cold in [false, true] {
                    for _ in 0..2 {
                        let mut receiver = TrackerEngine::new();
                        receiver.load_tracker_module(song.clone(), vec![1]);
                        let mut replay = boundary;
                        if cold {
                            receiver.sync_to_state_at_rate(&replay, &sounds, rate);
                        } else {
                            receiver.apply_snapshot(&saved);
                        }
                        assert_eq!(
                            format!("{:?}", receiver.channels),
                            format!("{:?}", saved.channels),
                            "mid-transition rate={rate} ft2={ft2} cold={cold}"
                        );
                        let actual: Vec<_> = (0..8000)
                            .map(|_| receiver.render_sample_and_advance(&mut replay, &sounds, rate))
                            .collect();
                        assert_eq!(actual, expected, "rate={rate} ft2={ft2} cold={cold}");
                        assert_eq!(format!("{:?}", receiver.channels), final_channels);
                        assert_eq!(
                            bytemuck::bytes_of(&replay),
                            bytemuck::bytes_of(&final_state)
                        );
                    }
                }
                let mut silent = TrackerEngine::new();
                silent.apply_snapshot(&saved);
                let mut replay = boundary;
                silent.advance_positions(&mut replay, &sounds, 8000, rate);
                assert_eq!(format!("{:?}", silent.channels), final_channels);
                assert_eq!(
                    bytemuck::bytes_of(&replay),
                    bytemuck::bytes_of(&final_state)
                );
                engine.apply_snapshot(&saved);
                state = boundary;
            }
        }
    }
}

#[test]
fn xm_rate_stopped_sample_retrigger_snapshot_and_cold_replay() {
    check_xm_rate_stopped_sample_retrigger_snapshot_and_cold_replay(false);
    check_xm_rate_stopped_sample_retrigger_snapshot_and_cold_replay(true);
}

fn check_xm_rate_stopped_sample_retrigger_snapshot_and_cold_replay(envelopes: bool) {
    for (ft2, legacy, effect) in [
        (false, false, 0),
        (false, false, 1),
        (true, false, 0),
        (true, false, 1),
        (false, true, 0),
        (false, true, 1),
        (false, true, 2),
        (true, true, 0),
        (true, true, 1),
        (true, true, 2),
    ] {
        let (template, initial, sounds) = setup_rollback();
        let mut song = template.modules[raw_tracker_handle(initial.handle) as usize]
            .as_ref()
            .unwrap()
            .module
            .clone();
        song.format =
            FormatFlags::IS_XM_FORMAT | FormatFlags::INSTRUMENTS | FormatFlags::LINEAR_SLIDES;
        if ft2 {
            song.format = song.format | FormatFlags::XM_FT2_MIX;
        }
        if legacy {
            song.format = song.format | FormatFlags::XM_LEGACY_RETRIGGER;
        }
        song.initial_speed = 6;
        song.initial_tempo = 125;
        song.instruments[0].volume_envelope = None;
        if envelopes {
            song.instruments[0].volume_envelope = Some(TrackerEnvelope {
                points: vec![(0, 64), (12, 16), (32, 48)],
                flags: EnvelopeFlags::ENABLED,
                ..Default::default()
            });
            song.instruments[0].panning_envelope = Some(TrackerEnvelope {
                points: vec![(0, 0), (12, -32), (32, 32)],
                flags: EnvelopeFlags::ENABLED,
                ..Default::default()
            });
        }
        song.instruments[0].sample_loop_type = nether_tracker::LoopType::Forward;
        song.instruments[0].xm_forward_loop_limit = 22050.0 / 8363.0;
        song.patterns = vec![pattern(vec![
            TrackerNote {
                note: 77,
                instrument: 1,
                ..Default::default()
            },
            row(TrackerEffect::PortamentoUp(4)),
            row(TrackerEffect::PortamentoDown(4)),
            row(match effect {
                1 => TrackerEffect::MultiRetrigNote {
                    ticks: 3,
                    volume: 9,
                },
                2 => TrackerEffect::NoteDelay(2),
                _ => TrackerEffect::Retrigger {
                    ticks: 2,
                    volume_change: 0,
                },
            }),
            row(TrackerEffect::None),
            row(TrackerEffect::None),
            TrackerNote {
                note: 49,
                instrument: 1,
                ..Default::default()
            },
        ])];
        song.order_table = vec![0];
        let mut engine = TrackerEngine::new();
        let handle = engine.load_tracker_module(song.clone(), vec![1]);
        let mut state = make_state(handle, 6, 125);
        engine.sync_to_state(&state, &sounds);
        let mut elapsed = 0;
        let checkpoints: &[u32] = if effect == 2 {
            &[10003, 14553, 16757]
        } else {
            &[10003, 14553]
        };
        for &checkpoint in checkpoints {
            engine.advance_positions(&mut state, &sounds, checkpoint - elapsed, 44100);
            elapsed = checkpoint;
            if checkpoint == 16757 {
                assert!(engine.channels[0].delayed_xm_note.is_some());
            }
            assert!(engine.channels[0].xm_loop_stopped);
            assert!(engine.channels[0].note_on);
            let saved = engine.snapshot();
            let boundary = state;
            let expected: Vec<_> = (0..20000)
                .map(|_| engine.render_sample_and_advance(&mut state, &sounds, 44100))
                .collect();
            if checkpoint == 10003 {
                assert!(expected[..100].iter().any(|&(l, r)| l != 0.0 || r != 0.0));
                assert!(expected[..3000].iter().any(|&(l, r)| l == 0.0 && r == 0.0));
            } else {
                assert!(expected[..100].iter().all(|&(l, r)| l == 0.0 && r == 0.0));
            }
            // This window follows E9/Rxy but precedes the explicit low-note fallback.
            if effect == 2 {
                assert!(
                    expected[12000..14000]
                        .iter()
                        .all(|&(l, r)| l == 0.0 && r == 0.0)
                );
            } else {
                assert!(
                    expected[12000..14000]
                        .iter()
                        .any(|&(l, r)| l != 0.0 || r != 0.0),
                    "ft2={ft2} legacy={legacy} effect={effect} checkpoint={checkpoint}"
                );
            }
            assert_eq!(
                engine.channels[0].xm_loop_stopped,
                effect == 2 && checkpoint == 10003
            );
            let final_snapshot = engine.snapshot();
            let final_state = state;
            for cold in [false, true] {
                for _ in 0..2 {
                    let mut receiver = TrackerEngine::new();
                    receiver.load_tracker_module(song.clone(), vec![1]);
                    let mut replay = boundary;
                    if cold {
                        receiver.sync_to_state(&replay, &sounds);
                    } else {
                        receiver.apply_snapshot(&saved);
                    }
                    assert!(receiver.channels[0].xm_loop_stopped);
                    let actual: Vec<_> = (0..20000)
                        .map(|_| receiver.render_sample_and_advance(&mut replay, &sounds, 44100))
                        .collect();
                    assert_eq!(
                        actual, expected,
                        "stop/retrigger ft2={ft2} legacy={legacy} effect={effect} cold={cold}"
                    );
                    assert_eq!(
                        format!("{:?}", receiver.channels),
                        format!("{:?}", final_snapshot.channels)
                    );
                    assert_eq!(
                        bytemuck::bytes_of(&replay),
                        bytemuck::bytes_of(&final_state)
                    );
                }
            }
            engine.apply_snapshot(&saved);
            state = boundary;
        }
    }
}

#[test]
fn xm_legacy_empty_delay_preserves_sample_position() {
    for legacy in [false, true] {
        let (template, initial, sounds) = setup_rollback();
        let mut song = template.modules[raw_tracker_handle(initial.handle) as usize]
            .as_ref()
            .unwrap()
            .module
            .clone();
        song.format =
            FormatFlags::IS_XM_FORMAT | FormatFlags::INSTRUMENTS | FormatFlags::XM_FT2_MIX;
        if legacy {
            song.format = song.format | FormatFlags::XM_LEGACY_RETRIGGER;
        }
        let mut engine = TrackerEngine::new();
        let handle = engine.load_tracker_module(song, vec![1]);
        let mut state = make_state(handle, 6, 125);
        engine.sync_to_state(&state, &sounds);
        engine.render_sample_and_advance(&mut state, &sounds, 44100);
        engine.channels[0].sample_pos = 123.0;
        engine.channels[0].xm_source_position = 123.0;
        engine.channels[0].current_note = 49;
        engine.channels[0].note_delay_tick = 2;
        engine.channels[0].delayed_xm_note = Some((TrackerNote::default(), handle));
        engine.process_tick(2, 6);
        assert_eq!(
            engine.channels[0].sample_pos,
            if legacy { 123.0 } else { 0.0 }
        );
        assert_eq!(
            engine.channels[0].xm_source_position,
            if legacy { 123.0 } else { 0.0 }
        );
        assert_eq!(engine.channels[0].xm_short_restart_attack, legacy);
    }
}

#[test]
fn xm_portamento_at_target_does_not_step_away() {
    let (mut engine, _, _) = setup_rollback();
    engine.is_it_format = false;
    engine.current_tick = 1;
    let channel = &mut engine.channels[0];
    channel.note_on = true;
    channel.period = 4544.0;
    channel.base_period = 4544.0;
    channel.target_period = 4544.0;
    channel.tone_porta_active = true;
    channel.porta_speed = 255;
    channel.xm_amiga_slides = true;
    channel.xm_source_tuning = -1537;
    engine.process_tick(1, 6);
    assert_eq!(engine.channels[0].base_period, 4544.0);
    assert_eq!(engine.channels[0].period, 4544.0);
}

#[test]
fn xm_competing_flow_state_survives_snapshot_and_cold_replay() {
    for legacy in [false, true] {
        for linear in [false, true] {
            let (template, template_state, sounds) = setup_rollback();
            let mut song = template.modules[raw_tracker_handle(template_state.handle) as usize]
                .as_ref()
                .unwrap()
                .module
                .clone();
            song.num_channels = 2;
            song.format = FormatFlags::IS_XM_FORMAT | FormatFlags::INSTRUMENTS;
            if legacy {
                song.format = song.format | FormatFlags::XM_LEGACY_RETRIGGER;
            }
            if linear {
                song.format = song.format | FormatFlags::LINEAR_SLIDES;
            }
            song.initial_speed = 3;
            song.initial_tempo = 125;
            let mut p = pattern(
                (0..8)
                    .map(|i| TrackerNote {
                        note: 49 + i,
                        instrument: 1,
                        ..Default::default()
                    })
                    .collect(),
            );
            for row in &mut p.notes {
                row.push(TrackerNote::default());
            }
            let plain = p.clone();
            p.notes[0][0].effect = TrackerEffect::PatternLoop(0);
            p.notes[1][1].effect = TrackerEffect::PatternLoop(0);
            p.notes[3][0].effect = TrackerEffect::PatternLoop(2);
            p.notes[3][1].effect = TrackerEffect::PatternLoop(1);
            song.patterns = vec![p, plain];
            song.order_table = vec![0, 1];
            let mut engine = TrackerEngine::new();
            let handle = engine.load_tracker_module(song.clone(), vec![1]);
            let mut state = make_state(handle, 3, 125);
            engine.sync_to_state(&state, &sounds);
            let mut elapsed = 0;
            for checkpoint in [10003, 30017, 60031] {
                engine.advance_positions(&mut state, &sounds, checkpoint - elapsed, 44100);
                elapsed = checkpoint;
                let saved = engine.snapshot();
                let boundary = state;
                if checkpoint == 10003 {
                    assert_eq!(saved.xm_loop_owner, if legacy { Some(0) } else { None });
                    assert_eq!(saved.xm_next_pattern_row, if legacy { 0 } else { 1 });
                }
                let expected: Vec<_> = (0..16000)
                    .map(|_| engine.render_sample_and_advance(&mut state, &sounds, 44100))
                    .collect();
                let final_snapshot = engine.snapshot();
                let final_state = state;
                for mode in ["snapshot", "cold", "warm-cache"] {
                    let cold = mode != "snapshot";
                    let mut replay = boundary;
                    let mut receiver = TrackerEngine::new();
                    receiver.load_tracker_module(song.clone(), vec![1]);
                    if mode == "warm-cache" {
                        receiver.seek_to_position(handle, 0, 7, &sounds);
                        receiver.seek_to_position(handle, 1, 2, &sounds);
                    }
                    if cold {
                        receiver.sync_to_state(&replay, &sounds);
                    } else {
                        receiver.apply_snapshot(&saved);
                    }
                    let actual: Vec<_> = (0..16000)
                        .map(|_| receiver.render_sample_and_advance(&mut replay, &sounds, 44100))
                        .collect();
                    assert!(
                        actual == expected,
                        "flow PCM legacy={legacy} linear={linear} checkpoint={checkpoint} cold={cold}"
                    );
                    assert_eq!(
                        format!("{:?}", receiver.channels),
                        format!("{:?}", final_snapshot.channels)
                    );
                    assert_eq!(receiver.xm_loop_owner, final_snapshot.xm_loop_owner);
                    assert_eq!(
                        receiver.xm_next_pattern_row,
                        final_snapshot.xm_next_pattern_row
                    );
                    assert_eq!(
                        bytemuck::bytes_of(&replay),
                        bytemuck::bytes_of(&final_state)
                    );
                }
                engine.apply_snapshot(&saved);
                state = boundary;
            }
        }
    }
}

#[test]
fn xm_cold_seek_observes_tempo_changes_before_target_row() {
    let mut song = module(
        vec![pattern(vec![
            TrackerNote {
                note: 49,
                instrument: 1,
                effect: TrackerEffect::SetTempo(250),
                ..Default::default()
            },
            row(TrackerEffect::None),
            row(TrackerEffect::None),
        ])],
        vec![0],
        FormatFlags::IS_XM_FORMAT | FormatFlags::INSTRUMENTS,
    );
    song.initial_speed = 1;
    song.initial_tempo = 125;
    song.instruments[0].sample_loop_type = nether_tracker::LoopType::PingPong;
    song.instruments[0].sample_loop_end = 7;
    let (_, _, sounds) = setup_rollback();
    let mut uninterrupted = TrackerEngine::new();
    let handle = uninterrupted.load_tracker_module(song.clone(), vec![1]);
    let mut state = make_state(handle, 1, 125);
    uninterrupted.sync_to_state(&state, &sounds);
    for _ in 0..2000 {
        if state.row == 1 {
            break;
        }
        uninterrupted.advance_positions(&mut state, &sounds, 1, 44100);
    }
    assert_eq!((state.row, state.tick_sample_pos), (1, 0));
    state.reset_sample_clock();
    let mut sought = state;
    let expected: Vec<_> = (0..200)
        .map(|_| uninterrupted.render_sample_and_advance(&mut state, &sounds, 44100))
        .collect();
    let mut cold = TrackerEngine::new();
    cold.load_tracker_module(song, vec![1]);
    cold.sync_to_state(&sought, &sounds);
    let actual: Vec<_> = (0..200)
        .map(|_| cold.render_sample_and_advance(&mut sought, &sounds, 44100))
        .collect();
    assert!(
        actual == expected,
        "cold seek skipped the tempo-shortened target-row boundary"
    );
}

#[test]
fn xm_seek_after_rate_change_uses_active_generation_rate() {
    check_seek_after_rate_change(setup_rollback);
}
#[test]
fn it_seek_after_rate_change_uses_active_generation_rate() {
    check_seek_after_rate_change(setup_it_rollback);
}
fn check_seek_after_rate_change(setup: fn() -> (TrackerEngine, TrackerState, Vec<Option<Sound>>)) {
    for rate in [44100, 48000, 96000] {
        for warmed in [false, true] {
            let (mut reference, mut expected_state, sounds) = setup();
            for _ in 0..rate * 10 {
                if (
                    expected_state.order_position,
                    expected_state.row,
                    expected_state.tick,
                    expected_state.tick_sample_pos,
                ) == (1, 2, 0, 0)
                {
                    break;
                }
                reference.advance_positions(&mut expected_state, &sounds, 1, rate);
            }
            assert_eq!(
                (
                    expected_state.order_position,
                    expected_state.row,
                    expected_state.tick,
                    expected_state.tick_sample_pos
                ),
                (1, 2, 0, 0)
            );
            expected_state.reset_sample_clock();
            let mut target = expected_state;
            target.advance_sample_clock(44100);
            target.reset_sample_clock();
            assert_eq!(target._reserved[2], 0);
            let (mut receiver, mut stale, _) = setup();
            if warmed {
                receiver.advance_positions(&mut stale, &sounds, 31001, 44100);
                receiver.sync_to_state(&stale, &sounds);
                receiver.seek_to_position(stale.handle, 0, 3, &sounds);
            }
            let mut actual = Vec::new();
            for _ in 0..5 {
                let mut expected = Vec::new();
                for _ in 0..rate / 60 {
                    let (l, r) =
                        reference.render_sample_and_advance(&mut expected_state, &sounds, rate);
                    expected.extend([l, r]);
                }
                generate_audio_frame_with_tracker(
                    &mut AudioPlaybackState::default(),
                    &mut target,
                    &mut receiver,
                    &sounds,
                    60,
                    rate,
                    &mut actual,
                );
                assert!(
                    expected.iter().any(|&s| s != 0.0),
                    "comparison must be audible"
                );
                assert!(
                    expected == actual,
                    "cold seek PCM differs: rate={rate} warmed={warmed}"
                );
                assert_eq!(
                    bytemuck::bytes_of(&target),
                    bytemuck::bytes_of(&expected_state)
                );
                assert_eq!(
                    format!("{:?}", receiver.channels),
                    format!("{:?}", reference.channels)
                );
            }
        }
    }
}

#[test]
fn it_note_swing_reaches_mixer_and_survives_nna() {
    let (mut engine, mut state, sounds) = setup_it_rollback();
    engine.advance_positions(&mut state, &sounds, 317, 44100);
    let old = engine.channels[0].clone();
    assert_ne!(
        old.volume_swing, 0.0,
        "IT authored random volume was ignored"
    );
    assert_ne!(old.pan_swing, 0.0);
    assert!(old.volume_swing.abs() <= 0.2 && old.pan_swing.abs() <= 0.25);
    let saved = engine.snapshot();
    let before = engine.mix_channels(raw_tracker_handle(state.handle), &sounds, 44100, 882);
    engine.apply_snapshot(&saved);
    engine.channels[0].volume_swing = 0.0;
    engine.channels[0].pan_swing = 0.0;
    let no_swing = engine.mix_channels(raw_tracker_handle(state.handle), &sounds, 44100, 882);
    assert_ne!(before, no_swing, "swing must reach the actual mixer");
    engine.apply_snapshot(&saved);
    engine.process_note_internal(
        0,
        &TrackerNote {
            note: 56,
            instrument: 1,
            ..Default::default()
        },
        state.handle,
        &sounds,
    );
    assert_ne!(engine.channels[0].swing_rng, old.swing_rng);
    let held = engine.channels[0].clone();
    engine.process_note_internal(
        0,
        &TrackerNote {
            note: 60,
            instrument: 1,
            effect: TrackerEffect::TonePortamento(16),
            ..Default::default()
        },
        state.handle,
        &sounds,
    );
    assert_eq!(
        engine.channels[0].swing_rng, held.swing_rng,
        "porta must not redraw swing"
    );
    assert_eq!(engine.channels[0].volume_swing, held.volume_swing);
    assert_eq!(engine.channels[0].pan_swing, held.pan_swing);
    assert!(engine.channels.iter().any(|c| c.is_background
        && c.swing_rng == old.swing_rng
        && c.volume_swing == old.volume_swing
        && c.pan_swing == old.pan_swing));
}

fn setup_rollback() -> (TrackerEngine, TrackerState, Vec<Option<Sound>>) {
    setup_format_rollback(false)
}

fn setup_stereo_rollback(short_loop: bool) -> (TrackerEngine, TrackerState, Vec<Option<Sound>>) {
    let (template, template_state, _) = setup_rollback();
    let mut song = template.modules[raw_tracker_handle(template_state.handle) as usize]
        .as_ref()
        .unwrap()
        .module
        .clone();
    song.instruments[0].sample_is_stereo = true;
    song.instruments[0].sample_loop_end = if short_loop { 2 } else { 7 };
    if short_loop {
        song.patterns[0].notes[0][0].note = 85;
    }
    let sounds = vec![
        None,
        Some(Sound {
            data: Arc::new(
                (0..7)
                    .flat_map(|i| [12000 + i * 1000, -7000 - i * 500])
                    .collect(),
            ),
        }),
    ];
    let mut engine = TrackerEngine::new();
    let handle = engine.load_tracker_module(song, vec![1]);
    let mut state = make_state(handle, 3, 137);
    state.flags |= tracker_flags::LOOPING;
    engine.sync_to_state(&state, &sounds);
    (engine, state, sounds)
}
fn setup_it_rollback() -> (TrackerEngine, TrackerState, Vec<Option<Sound>>) {
    setup_format_rollback(true)
}
fn setup_format_rollback(it: bool) -> (TrackerEngine, TrackerState, Vec<Option<Sound>>) {
    let mut song = module(
        vec![
            pattern(vec![
                TrackerNote {
                    note: 49,
                    instrument: 1,
                    effect: TrackerEffect::Vibrato { speed: 3, depth: 4 },
                    ..Default::default()
                },
                row(TrackerEffect::VolumeSlide { up: 0, down: 2 }),
                TrackerNote {
                    note: 53,
                    instrument: 1,
                    effect: TrackerEffect::NoteDelay(2),
                    ..Default::default()
                },
                row(TrackerEffect::SetPanning(192)),
            ]),
            pattern(vec![row(TrackerEffect::None); 4]),
        ],
        vec![0, 1],
        FormatFlags::IS_XM_FORMAT | FormatFlags::INSTRUMENTS,
    );
    if it {
        song.format = FormatFlags::IS_IT_FORMAT | FormatFlags::INSTRUMENTS;
        song.samples.push(TrackerSample {
            loop_type: nether_tracker::LoopType::PingPong,
            loop_end: 7,
            ..Default::default()
        });
        for entry in &mut song.instruments[0].note_sample_table {
            entry.1 = 1;
        }
        song.instruments[0].nna = nether_tracker::NewNoteAction::Continue;
        song.instruments[0].random_volume = 20;
        song.instruments[0].random_pan = 8;
    }
    song.global_volume = 64;
    song.initial_speed = 3;
    song.initial_tempo = 137;
    song.instruments[0].sample_loop_type = nether_tracker::LoopType::PingPong;
    song.instruments[0].sample_loop_end = 7;
    song.instruments[0].volume_envelope = Some(TrackerEnvelope {
        points: vec![(0, 64), (3, 32), (7, 56)],
        sustain_begin: 1,
        sustain_end: 1,
        flags: EnvelopeFlags::ENABLED | EnvelopeFlags::SUSTAIN_LOOP,
        ..Default::default()
    });
    let sounds = vec![
        None,
        Some(Sound {
            data: Arc::new(vec![1000, -3000, 5000, 2000, -4000, 700, 6000]),
        }),
    ];
    let mut engine = TrackerEngine::new();
    let handle = engine.load_tracker_module(song, vec![1]);
    let mut state = make_state(handle, 3, 137);
    state.flags |= tracker_flags::LOOPING;
    engine.sync_to_state(&state, &sounds);
    (engine, state, sounds)
}

fn frame(
    engine: &mut TrackerEngine,
    state: &mut TrackerState,
    sounds: &[Option<Sound>],
) -> Vec<f32> {
    let mut pcm = Vec::new();
    generate_audio_frame_with_tracker(
        &mut AudioPlaybackState::default(),
        state,
        engine,
        sounds,
        60,
        44100,
        &mut pcm,
    );
    pcm
}

#[test]
fn xm_stereo_snapshot_and_cold_rollback_preserve_distinct_audible_pcm() {
    for short_loop in [false, true] {
        let (mut engine, mut state, sounds) = setup_stereo_rollback(short_loop);
        engine.advance_positions(&mut state, &sounds, 317, 44100);
        assert!(engine.channels[0].sample_is_stereo);
        let saved = engine.snapshot();
        let boundary = state;
        let expected: Vec<_> = (0..2048)
            .map(|_| engine.render_sample_and_advance(&mut state, &sounds, 44100))
            .collect();
        assert!(expected.iter().any(|&(l, r)| l > 0.01 && r < -0.01));
        assert!(expected.iter().any(|&(l, r)| (l - r).abs() > 0.01));
        let final_channels = format!("{:?}", engine.channels);

        for cold in [false, true] {
            let (mut receiver, _, _) = setup_stereo_rollback(short_loop);
            let mut resumed = boundary;
            if cold {
                receiver.sync_to_state(&resumed, &sounds);
            } else {
                receiver.apply_snapshot(&saved);
            }
            let actual: Vec<_> = (0..2048)
                .map(|_| receiver.render_sample_and_advance(&mut resumed, &sounds, 44100))
                .collect();
            assert_eq!(
                actual, expected,
                "stereo subsequent PCM differs; cold={cold}"
            );
            assert_eq!(bytemuck::bytes_of(&resumed), bytemuck::bytes_of(&state));
            assert_eq!(format!("{:?}", receiver.channels), final_channels);
        }
    }
}

#[test]
fn xm_arbitrary_snapshot_audio_handoff_preserves_pcm_and_state() {
    check_arbitrary_snapshot_audio_handoff_preserves_pcm_and_state(setup_rollback);
}
#[test]
fn it_arbitrary_snapshot_audio_handoff_preserves_pcm_and_state() {
    check_arbitrary_snapshot_audio_handoff_preserves_pcm_and_state(setup_it_rollback);
}
fn check_arbitrary_snapshot_audio_handoff_preserves_pcm_and_state(
    setup: fn() -> (TrackerEngine, TrackerState, Vec<Option<Sound>>),
) {
    let (mut engine, mut state, sounds) = setup();
    for offset in [317, 4091, 22003, 31001] {
        engine.advance_positions(&mut state, &sounds, offset, 44100);
        let saved = engine.snapshot();
        let saved_state = state;
        let expected: Vec<_> = (0..7)
            .map(|_| frame(&mut engine, &mut state, &sounds))
            .collect();
        let final_channels = format!("{:?}", engine.channels);
        for _ in 0..3 {
            let mut receiver = TrackerEngine::new();
            receiver.apply_snapshot(&saved);
            let mut resumed = saved_state;
            let actual: Vec<_> = (0..7)
                .map(|_| frame(&mut receiver, &mut resumed, &sounds))
                .collect();
            assert!(
                actual == expected,
                "snapshot handoff PCM differs at offset {offset}"
            );
            assert_eq!(bytemuck::bytes_of(&resumed), bytemuck::bytes_of(&state));
            assert!(
                format!("{:?}", receiver.channels) == final_channels,
                "snapshot channel state differs"
            );
        }
    }
}

#[test]
fn xm_seek_restart_and_cold_replay_preserve_sample_state() {
    check_seek_restart_and_cold_replay_preserve_sample_state(setup_rollback);
}
#[test]
fn it_seek_restart_and_cold_replay_preserve_sample_state() {
    check_seek_restart_and_cold_replay_preserve_sample_state(setup_it_rollback);
}
fn check_seek_restart_and_cold_replay_preserve_sample_state(
    setup: fn() -> (TrackerEngine, TrackerState, Vec<Option<Sound>>),
) {
    for rate in [44100, 48000] {
        let (mut reference, mut state, sounds) = setup();
        while (
            state.order_position,
            state.row,
            state.tick,
            state.tick_sample_pos,
        ) != (1, 2, 0, 0)
        {
            reference.advance_positions(&mut state, &sounds, 1, rate);
        }
        state.reset_sample_clock();
        let origin = state;
        let expected: Vec<_> = (0..997)
            .map(|_| reference.render_sample_and_advance(&mut state, &sounds, rate))
            .collect();
        let (mut seeking, _, _) = setup();
        let mut sought = origin;
        seeking.sync_to_state_at_rate(&sought, &sounds, rate);
        let actual: Vec<_> = (0..997)
            .map(|_| seeking.render_sample_and_advance(&mut sought, &sounds, rate))
            .collect();
        assert!(expected == actual, "seek PCM differs at {rate}Hz");
        assert_eq!(bytemuck::bytes_of(&sought), bytemuck::bytes_of(&state));
        let saved = state;
        let expected: Vec<_> = (0..1597)
            .map(|_| reference.render_sample_and_advance(&mut state, &sounds, rate))
            .collect();
        let (mut cold, _, _) = setup();
        let mut resumed = saved;
        cold.sync_to_state(&resumed, &sounds);
        let actual: Vec<_> = (0..1597)
            .map(|_| cold.render_sample_and_advance(&mut resumed, &sounds, rate))
            .collect();
        assert!(expected == actual, "cold post-seek PCM differs at {rate}Hz");
        assert_eq!(bytemuck::bytes_of(&resumed), bytemuck::bytes_of(&state));
        let (mut fresh, mut beginning, _) = setup();
        let expected = frame(&mut fresh, &mut beginning, &sounds);
        let mut restart = make_state(saved.handle, 3, 137);
        restart.flags |= tracker_flags::LOOPING;
        restart.reset_sample_clock();
        let actual = frame(&mut cold, &mut restart, &sounds);
        assert!(actual == expected, "restart PCM differs");
        assert_eq!(bytemuck::bytes_of(&restart), bytemuck::bytes_of(&beginning));
    }
}

#[test]
fn xm_backward_reconstruction_after_recent_cache_eviction() {
    check_backward_reconstruction_after_recent_cache_eviction(setup_rollback);
}
#[test]
fn it_backward_reconstruction_after_recent_cache_eviction() {
    check_backward_reconstruction_after_recent_cache_eviction(setup_it_rollback);
}
fn check_backward_reconstruction_after_recent_cache_eviction(
    setup: fn() -> (TrackerEngine, TrackerState, Vec<Option<Sound>>),
) {
    let (mut engine, mut state, sounds) = setup();
    for _ in 0..135 {
        frame(&mut engine, &mut state, &sounds);
    }
    let saved_state = state;
    let expected: Vec<_> = (0..9)
        .map(|_| frame(&mut engine, &mut state, &sounds))
        .collect();
    let final_state = state;
    let final_channels = format!("{:?}", engine.channels);
    for _ in 0..80 {
        frame(&mut engine, &mut state, &sounds);
    }
    for _ in 0..3 {
        state = saved_state;
        let actual: Vec<_> = (0..9)
            .map(|_| frame(&mut engine, &mut state, &sounds))
            .collect();
        assert!(
            actual == expected,
            "cold backward reconstruction PCM differs"
        );
        assert_eq!(bytemuck::bytes_of(&state), bytemuck::bytes_of(&final_state));
        assert!(
            format!("{:?}", engine.channels) == final_channels,
            "cold backward channel state differs"
        );
    }
}

#[test]
fn xm_retrigger_zero_memory_and_tick_phase_preserve_it() {
    let mut channel = crate::tracker::TrackerChannel::default();
    channel.volume = 0.5;
    channel.retrigger_mode = 6;
    for measured in [80, 50, 31, 19] {
        channel.retrigger_sample(true);
        assert_eq!(channel.volume, measured as f32 / 256.0);
    }
    channel.volume = 0.5;
    channel.retrigger_sample(false);
    assert_eq!(
        channel.volume,
        0.5 * (2.0 / 3.0),
        "IT retains its own transform"
    );
    for it in [false, true] {
        let mut engine = TrackerEngine::new();
        engine.is_it_format = it;
        engine.channels[0].note_on = true;
        engine.channels[0].sample_pos = 12.0;
        engine.process_unified_effect_tick0(
            0,
            &TrackerEffect::Retrigger {
                ticks: 0,
                volume_change: 0,
            },
            0,
            0,
        );
        assert_eq!(engine.channels[0].sample_pos, if it { 12.0 } else { 0.0 });
        engine.channels[0].sample_pos = 12.0;
        engine.process_tick(1, 6);
        assert_eq!(
            engine.channels[0].sample_pos, 12.0,
            "E90 must not restart again on tick one"
        );
        engine.process_unified_effect_tick0(0, &TrackerEffect::SampleOffset(256), 0, 0);
        assert_eq!(
            engine.channels[0].last_sample_offset,
            if it { 1 } else { 0 }
        );
        if it {
            engine.channels[0].xm_retrigger_memory = 0x93;
            engine.channels[0].xm_retrigger_count = 2;
            engine.channels[0].xm_multi_retrigger_active = true;
            engine.process_unified_effect_tick0(
                0,
                &TrackerEffect::Retrigger {
                    ticks: 3,
                    volume_change: 1,
                },
                0,
                0,
            );
            engine.process_tick(3, 6);
            let snapshot = engine.snapshot();
            assert_eq!(snapshot.channels[0].xm_retrigger_memory, 0x93);
            assert_eq!(snapshot.channels[0].xm_retrigger_count, 2);
            assert!(snapshot.channels[0].xm_multi_retrigger_active);
            assert_eq!(snapshot.channels[0].sample_pos, 0.0);
        }
    }
    let mut engine = TrackerEngine::new();
    engine.is_it_format = false;
    engine.channels[0].note_on = true;
    engine.channels[0].volume = 0.5;
    engine.channels[0].sample_pos = 12.0;
    engine.process_unified_effect_tick0(
        0,
        &TrackerEffect::MultiRetrigNote {
            ticks: 3,
            volume: 9,
        },
        0,
        0,
    );
    engine.process_tick(1, 6);
    assert_eq!(engine.channels[0].sample_pos, 12.0);
    engine.process_tick(2, 6);
    assert_eq!(engine.channels[0].sample_pos, 12.0);
    engine.process_tick(3, 6);
    assert_eq!(engine.channels[0].sample_pos, 0.0);
    assert_eq!(engine.channels[0].volume, 33.0 / 64.0);
    engine.channels[0].reset_row_effects();
    engine.process_unified_effect_tick0(
        0,
        &TrackerEffect::MultiRetrigNote {
            ticks: 0,
            volume: 0,
        },
        0,
        0,
    );
    assert_eq!(engine.channels[0].xm_retrigger_memory, 0x93);
    assert_eq!(engine.channels[0].retrigger_tick, 3);
    assert_eq!(engine.channels[0].retrigger_volume, 1);
    engine.channels[0].reset_row_effects();
    engine.channels[0].sample_pos = 12.0;
    engine.process_tick(1, 6);
    assert_eq!(
        engine.channels[0].sample_pos, 12.0,
        "empty row must stop the effect, not erase memory"
    );
}

#[test]
fn xm_pitch_modes_glissando_and_tremor_replay_subsequent_pcm() {
    for (tuning, target, speed) in [(-64, 37, 3), (64, 61, 5), (-9, 37, 3), (127, 61, 5)] {
        for linear in [false, true] {
            for (scenario, legacy) in (0..44)
                .map(|s| (s, false))
                .chain((9..16).chain(35..44).map(|s| (s, true)))
            {
                let tremor = scenario == 1;
                let (_, _, mut sounds) = setup_rollback();
                let mut song = module(
                    vec![pattern(vec![
                        TrackerNote {
                            note: 49,
                            instrument: 1,
                            effect: if tremor {
                                TrackerEffect::Tremor {
                                    ontime: 2,
                                    offtime: 1,
                                }
                            } else {
                                TrackerEffect::SetGlissando(true)
                            },
                            ..Default::default()
                        },
                        TrackerNote {
                            note: if tremor { 0 } else { target },
                            effect: if tremor {
                                TrackerEffect::Tremor {
                                    ontime: 0,
                                    offtime: 0,
                                }
                            } else {
                                TrackerEffect::TonePortamento(speed)
                            },
                            ..Default::default()
                        },
                        row(if tremor {
                            TrackerEffect::Tremor {
                                ontime: 0,
                                offtime: 0,
                            }
                        } else {
                            TrackerEffect::TonePortamento(0)
                        }),
                    ])],
                    vec![0],
                    FormatFlags::IS_XM_FORMAT | FormatFlags::INSTRUMENTS,
                );
                if scenario >= 2 {
                    song.patterns[0] = pattern(vec![
                        TrackerNote {
                            note: 49,
                            instrument: 1,
                            ..Default::default()
                        },
                        TrackerNote {
                            note: if scenario == 2 { target } else { 0 },
                            effect: if scenario == 2 {
                                TrackerEffect::TonePortamento(speed)
                            } else {
                                TrackerEffect::FinePortaUp(3)
                            },
                            volume_effect: if scenario == 3 {
                                TrackerEffect::Vibrato { speed: 3, depth: 4 }
                            } else {
                                TrackerEffect::None
                            },
                            ..Default::default()
                        },
                        row(TrackerEffect::Vibrato { speed: 3, depth: 4 }),
                        row(TrackerEffect::None),
                    ]);
                }
                if scenario == 4 {
                    song.patterns[0] = pattern(vec![
                        TrackerNote {
                            note: 49,
                            instrument: 1,
                            effect: TrackerEffect::SetFinetune(0),
                            ..Default::default()
                        },
                        row(TrackerEffect::SetFinetune(-128)),
                        TrackerNote {
                            note: 49,
                            instrument: 1,
                            ..Default::default()
                        },
                    ]);
                }
                if scenario >= 5 {
                    song.patterns[0] = pattern(vec![
                        TrackerNote {
                            note: 49,
                            instrument: 1,
                            ..Default::default()
                        },
                        TrackerNote {
                            note: if scenario >= 6 { 61 } else { 0 },
                            effect: TrackerEffect::SampleOffset(256),
                            volume_effect: if scenario >= 6 {
                                TrackerEffect::TonePortamento(4)
                            } else {
                                TrackerEffect::None
                            },
                            ..Default::default()
                        },
                        TrackerNote {
                            note: 49,
                            effect: TrackerEffect::SampleOffset(0),
                            ..Default::default()
                        },
                    ]);
                    sounds[1] = Some(Sound {
                        data: Arc::new(
                            (0..4096).map(|i| ((i % 97) * 400 - 19200) as i16).collect(),
                        ),
                    });
                }
                if scenario >= 9 {
                    song.initial_speed = 5;
                    song.patterns[0] = pattern(vec![
                        TrackerNote {
                            note: 49,
                            instrument: 1,
                            ..Default::default()
                        },
                        row(if matches!(scenario, 9 | 11 | 13) {
                            TrackerEffect::MultiRetrigNote {
                                ticks: 3,
                                volume: if scenario == 13 { 6 } else { 9 },
                            }
                        } else {
                            TrackerEffect::Retrigger {
                                ticks: 3,
                                volume_change: 0,
                            }
                        }),
                        row(if matches!(scenario, 9 | 11 | 13) {
                            TrackerEffect::MultiRetrigNote {
                                ticks: 0,
                                volume: 0,
                            }
                        } else {
                            TrackerEffect::Retrigger {
                                ticks: 0,
                                volume_change: 0,
                            }
                        }),
                        row(TrackerEffect::None),
                    ]);
                }
                if (11..=12).contains(&scenario) {
                    song.patterns[0].notes[2][0] = TrackerNote {
                        note: 49,
                        instrument: 1,
                        effect: if scenario == 11 {
                            TrackerEffect::MultiRetrigNote {
                                ticks: 3,
                                volume: 0,
                            }
                        } else {
                            TrackerEffect::Retrigger {
                                ticks: 3,
                                volume_change: 0,
                            }
                        },
                        ..Default::default()
                    };
                }
                if (14..16).contains(&scenario) {
                    song.instruments[0].volume_envelope = Some(TrackerEnvelope {
                        points: vec![(0, 64), (20, 16), (40, 48)],
                        flags: EnvelopeFlags::ENABLED,
                        ..Default::default()
                    });
                    song.instruments[0].panning_envelope = Some(TrackerEnvelope {
                        points: vec![(0, 32), (20, 8), (40, 48)],
                        flags: EnvelopeFlags::ENABLED,
                        ..Default::default()
                    });
                    song.patterns[0] = pattern(vec![
                        TrackerNote {
                            note: 49,
                            instrument: 1,
                            ..Default::default()
                        },
                        TrackerNote {
                            note: 49,
                            ..Default::default()
                        },
                        row(if scenario == 14 {
                            TrackerEffect::Retrigger {
                                ticks: 3,
                                volume_change: 0,
                            }
                        } else {
                            TrackerEffect::MultiRetrigNote {
                                ticks: 3,
                                volume: 0,
                            }
                        }),
                        TrackerNote {
                            note: 49,
                            instrument: 1,
                            ..Default::default()
                        },
                    ]);
                }
                if scenario >= 16 {
                    let combined = if scenario == 16 {
                        TrackerEffect::TonePortaVolSlide {
                            porta: 0,
                            vol_up: 0,
                            vol_down: 1,
                        }
                    } else {
                        TrackerEffect::VibratoVolSlide {
                            vib_speed: 0,
                            vib_depth: 0,
                            vol_up: 0,
                            vol_down: 1,
                        }
                    };
                    song.patterns[0] = pattern(vec![
                        TrackerNote {
                            note: 49,
                            instrument: 1,
                            volume: 32,
                            ..Default::default()
                        },
                        TrackerNote {
                            note: if scenario == 16 { 61 } else { 0 },
                            effect: if scenario == 16 {
                                TrackerEffect::TonePortamento(5)
                            } else {
                                TrackerEffect::Vibrato { speed: 3, depth: 4 }
                            },
                            ..Default::default()
                        },
                        TrackerNote {
                            volume_effect: TrackerEffect::VolumeSlide { up: 1, down: 0 },
                            effect: combined,
                            ..Default::default()
                        },
                        row(TrackerEffect::VolumeSlide { up: 0, down: 0 }),
                    ]);
                }
                if scenario >= 18 {
                    song.patterns[0] = pattern(vec![
                        TrackerNote {
                            note: 49,
                            instrument: 1,
                            effect: TrackerEffect::SetVolume(32),
                            ..Default::default()
                        },
                        row(TrackerEffect::Tremolo { speed: 3, depth: 4 }),
                        row(TrackerEffect::Tremolo { speed: 0, depth: 0 }),
                        row(TrackerEffect::None),
                    ]);
                }
                if scenario >= 19 {
                    song.patterns[0].notes[0][0].volume_effect =
                        TrackerEffect::TremoloWaveform(if scenario == 19 { 2 } else { 1 });
                    if scenario == 21 {
                        song.patterns[0].notes[1][0].effect = TrackerEffect::Vibrato {
                            speed: 12,
                            depth: 4,
                        };
                        song.patterns[0].notes[2][0].effect =
                            TrackerEffect::Tremolo { speed: 3, depth: 4 };
                    }
                }
                if scenario == 22 {
                    song.patterns[0].notes[0][0].effect = TrackerEffect::SetVolume(0);
                }
                if scenario >= 23 {
                    song.patterns[0].notes[0][0].volume_effect =
                        TrackerEffect::VibratoWaveform(if scenario == 23 { 1 } else { 2 });
                    song.patterns[0].notes[1][0].effect =
                        TrackerEffect::Vibrato { speed: 3, depth: 4 };
                    song.patterns[0].notes[2][0].effect =
                        TrackerEffect::Vibrato { speed: 0, depth: 0 };
                }
                if scenario >= 25 {
                    song.patterns[0].notes[0][0].volume_effect = if scenario == 25 {
                        TrackerEffect::VibratoWaveform(3)
                    } else {
                        TrackerEffect::TremoloWaveform(3)
                    };
                    if scenario == 26 {
                        song.patterns[0].notes[1][0].effect =
                            TrackerEffect::Tremolo { speed: 3, depth: 4 };
                        song.patterns[0].notes[2][0].effect =
                            TrackerEffect::Tremolo { speed: 0, depth: 0 };
                    }
                }
                if scenario >= 27 {
                    song.instruments[0].auto_vibrato_type = (scenario - 27) % 4;
                    song.instruments[0].auto_vibrato_depth = 11;
                    song.instruments[0].auto_vibrato_sweep = 7;
                    song.instruments[0].auto_vibrato_rate = 13;
                }
                if scenario >= 33 {
                    song.instruments[0].volume_envelope = Some(TrackerEnvelope {
                        points: vec![(0, 64), (100, 64)],
                        flags: EnvelopeFlags::ENABLED,
                        ..Default::default()
                    });
                }
                if scenario >= 31 {
                    song.patterns[0].notes[2][0] = match scenario {
                        31 => TrackerNote {
                            instrument: 1,
                            ..Default::default()
                        },
                        32 => row(TrackerEffect::PatternDelay(1)),
                        34 => TrackerNote {
                            volume_effect: TrackerEffect::Vibrato {
                                speed: 3,
                                depth: 15,
                            },
                            effect: TrackerEffect::PatternDelay(1),
                            ..Default::default()
                        },
                        _ => TrackerNote {
                            note: TrackerNote::NOTE_OFF,
                            ..Default::default()
                        },
                    };
                }
                if scenario >= 35 {
                    song.patterns[0].notes[1][0].effect = TrackerEffect::SetGlobalVolume(64);
                    for row in [2, 3] {
                        song.patterns[0].notes[row][0].effect = if scenario == 35 {
                            TrackerEffect::PanningSlide {
                                left: 0,
                                right: if row == 2 { 1 } else { 0 },
                            }
                        } else {
                            TrackerEffect::GlobalVolumeSlide {
                                up: if row == 2 { 1 } else { 0 },
                                down: 0,
                            }
                        };
                        song.patterns[0].notes[row][0].volume_effect =
                            TrackerEffect::PanningSlide { left: 0, right: 1 };
                    }
                }
                if scenario == 37 || scenario == 39 {
                    song.patterns[0].notes[2][0].note = 49;
                    song.patterns[0].notes[2][0].instrument = 1;
                    song.patterns[0].notes[2][0].effect = TrackerEffect::NoteDelay(3);
                    song.patterns[0].notes[2][0].volume_effect = if scenario == 39 {
                        TrackerEffect::PanningLeftOnTicks
                    } else {
                        TrackerEffect::PanningSlide { left: 0, right: 15 }
                    };
                }
                if scenario == 38 {
                    song.patterns[0].notes[1][0].effect =
                        TrackerEffect::PanningSlide { left: 1, right: 0 };
                    song.patterns[0].notes[2][0].effect = TrackerEffect::None;
                    song.patterns[0].notes[2][0].volume_effect = TrackerEffect::PanningLeftOnTicks;
                    song.patterns[0].notes[3][0].effect =
                        TrackerEffect::PanningSlide { left: 0, right: 0 };
                    song.patterns[0].notes[3][0].volume_effect =
                        TrackerEffect::PanningSlide { left: 0, right: 0 };
                }
                if scenario == 40 || scenario == 41 {
                    song.patterns[0].notes[0][0].effect = TrackerEffect::SetFinetune(-128);
                    song.patterns[0].notes[0][0].volume_effect = TrackerEffect::SetVolume(32);
                    song.patterns[0].notes[1][0].effect = if scenario == 40 {
                        TrackerEffect::SetGlissando(true)
                    } else {
                        TrackerEffect::None
                    };
                    song.patterns[0].notes[2][0].volume_effect = TrackerEffect::None;
                    song.patterns[0].notes[3][0].volume_effect = TrackerEffect::None;
                    song.patterns[0].notes[3][0].effect = TrackerEffect::None;
                    if scenario == 40 {
                        song.patterns[0].notes[2][0].note = 50;
                        song.patterns[0].notes[2][0].instrument = 0;
                        song.patterns[0].notes[2][0].effect = TrackerEffect::TonePortamento(255);
                    } else {
                        song.patterns[0].notes[2][0].effect = TrackerEffect::None;
                        song.patterns[0].notes[3][0].effect = TrackerEffect::Retrigger {
                            ticks: 2,
                            volume_change: 0,
                        };
                    }
                }
                if scenario == 42 || scenario == 43 {
                    song.patterns[0].notes[0][0].effect = TrackerEffect::SetFinetune(112);
                    song.patterns[0].notes[0][0].volume_effect = TrackerEffect::SetVolume(32);
                    for (r, n) in [(1, 61), (2, 0), (3, 37)] {
                        song.patterns[0].notes[r][0] = TrackerNote {
                            note: n,
                            effect: TrackerEffect::Retrigger {
                                ticks: 2,
                                volume_change: 0,
                            },
                            volume_effect: TrackerEffect::TonePortamento(if r == 2 {
                                0
                            } else if scenario == 42 {
                                16
                            } else {
                                240
                            }),
                            ..Default::default()
                        };
                    }
                }
                if legacy {
                    song.format = song.format | FormatFlags::XM_LEGACY_RETRIGGER;
                }
                if linear {
                    song.format = song.format | FormatFlags::LINEAR_SLIDES;
                }
                song.instruments[0].xm_source_tuning = tuning;
                song.instruments[0].xm_source_finetune = tuning as i8;
                song.instruments[0].sample_loop_type = nether_tracker::LoopType::PingPong;
                song.instruments[0].sample_loop_end = if scenario >= 5 { 4096 } else { 7 };
                let handles = if scenario >= 7 {
                    song.samples = vec![
                        nether_tracker::TrackerSample::default(),
                        nether_tracker::TrackerSample {
                            default_volume: 64,
                            c5_speed: 22050,
                            xm_source_tuning: tuning,
                            xm_source_finetune: tuning as i8,
                            loop_type: nether_tracker::LoopType::PingPong,
                            loop_end: 4096,
                            ..Default::default()
                        },
                    ];
                    for entry in &mut song.instruments[0].note_sample_table {
                        entry.1 = 2;
                    }
                    if scenario == 8 {
                        song.samples[0] = song.samples[1].clone();
                        song.samples[0].xm_source_tuning = 0;
                        song.samples[0].xm_source_finetune = 0;
                        for entry in &mut song.instruments[0].note_sample_table[..48] {
                            entry.1 = 1;
                        }
                        song.patterns[0].notes[0][0].note = 37;
                        sounds.push(Some(Sound {
                            data: Arc::new(
                                (0..4096).map(|i| ((i % 53) as i16 - 26) * 180).collect(),
                            ),
                        }));
                        vec![1, 2]
                    } else {
                        vec![0, 1]
                    }
                } else {
                    vec![1]
                };
                let mut engine = TrackerEngine::new();
                let handle = engine.load_tracker_module(song.clone(), handles.clone());
                let mut state = make_state(handle, if scenario >= 9 { 5 } else { 6 }, 125);
                state.flags |= tracker_flags::LOOPING;
                engine.sync_to_state(&state, &sounds);
                for _ in 0..if scenario >= 16 { 11297 } else { 7297 } {
                    engine.render_sample_and_advance(&mut state, &sounds, 44100);
                }
                let saved = engine.snapshot();
                let saved_state = state;
                if matches!(scenario, 9 | 11 | 13) {
                    assert_eq!(
                        saved.channels[0].xm_retrigger_memory,
                        if scenario == 13 { 0x63 } else { 0x93 }
                    );
                    assert_eq!(saved.channels[0].xm_retrigger_count, 1);
                }
                if (14..16).contains(&scenario) {
                    assert_eq!(
                        saved.channels[0].volume_envelope_pos, 8,
                        "note-only row preserves envelope age"
                    );
                    assert_eq!(saved.channels[0].panning_envelope_pos, 8);
                }
                if (16..18).contains(&scenario) {
                    assert_eq!(saved.channels[0].xm_volume_column_slide, 1);
                    assert_eq!(saved.channels[0].last_volume_slide, 1);
                    assert_eq!(saved.channels[0].volume, 0.5);
                }
                if scenario >= 18 {
                    assert_eq!(
                        saved.channels[0].volume,
                        if scenario == 22 {
                            0.0
                        } else if scenario == 31 {
                            1.0
                        } else {
                            0.5
                        },
                        "modulation preserves volume; standalone selection reloads sample default"
                    );
                    if scenario < 23 {
                        assert_ne!(saved.channels[0].xm_tremolo_delta, 0.0);
                    }
                }
                assert_eq!(saved.channels[0].xm_legacy_retrigger, legacy);
                assert_eq!(saved.channels[0].xm_amiga_slides, !linear);
                assert_eq!(
                    saved.channels[0].xm_source_tuning,
                    if scenario == 8 { 0 } else { tuning }
                );
                if scenario >= 5 {
                    assert_eq!(saved.channels[0].last_sample_offset, 0);
                }
                if scenario == 4 {
                    assert_eq!(
                        saved.channels[0].base_period,
                        4608.0 + tuning as f32 / 2.0,
                        "E58 replaces baked finetune; no-note E50 must not retune playback"
                    );
                }
                if scenario == 0 {
                    assert!(saved.channels[0].glissando);
                    assert!(
                        if target > 49 {
                            saved.channels[0].period < 4608.0
                        } else {
                            saved.channels[0].period > 4608.0
                        },
                        "quantizer must not stall the accumulator"
                    );
                }
                let expected: Vec<_> = (0..16000)
                    .map(|_| engine.render_sample_and_advance(&mut state, &sounds, 44100))
                    .collect();
                if scenario == 22 {
                    assert!(expected.iter().all(|&(l, r)| l == 0.0 && r == 0.0));
                }
                if scenario >= 7 && scenario != 22 {
                    assert!(
                        expected.iter().any(|&(l, r)| l != 0.0 || r != 0.0),
                        "mapped rollback must compare audible PCM"
                    );
                }
                for cold in [false, true, false] {
                    let mut restored = TrackerEngine::new();
                    restored.load_tracker_module(song.clone(), handles.clone());
                    let mut actual_state = saved_state;
                    if !cold {
                        restored.apply_snapshot(&saved);
                    }
                    restored.sync_to_state(&actual_state, &sounds);
                    let actual: Vec<_> = (0..16000)
                        .map(|_| {
                            restored.render_sample_and_advance(&mut actual_state, &sounds, 44100)
                        })
                        .collect();
                    assert!(
                        actual == expected,
                        "mode={linear} scenario={scenario} tuning={tuning} target={target} speed={speed} cold={cold}"
                    );
                    assert_eq!(
                        bytemuck::bytes_of(&actual_state),
                        bytemuck::bytes_of(&state)
                    );
                    assert_eq!(
                        format!("{:?}", restored.snapshot().channels),
                        format!("{:?}", engine.snapshot().channels)
                    );
                }
            }
        }
    }
}
