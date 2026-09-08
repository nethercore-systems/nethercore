//! Unit tests for tracker engine

#![allow(clippy::field_reassign_with_default)]

use super::super::TrackerEngine;
use super::super::channels::NNA_CUT;
use nether_tracker::{
    FormatFlags, NewNoteAction, TrackerEffect, TrackerInstrument, TrackerModule, TrackerNote,
    TrackerPattern,
};

fn make_it_module_with_second_row_effect(effect: TrackerEffect) -> TrackerModule {
    let instr = TrackerInstrument {
        fadeout: 1024,
        ..Default::default()
    };

    let row0 = TrackerNote {
        note: 48,
        instrument: 1,
        volume: 64,
        effect: TrackerEffect::None,
        ..Default::default()
    };

    let row1 = TrackerNote {
        note: 0,
        instrument: 0,
        volume: 0,
        effect,
        ..Default::default()
    };

    let pattern = TrackerPattern {
        num_rows: 2,
        notes: vec![vec![row0], vec![row1]],
    };

    TrackerModule {
        name: "IT Slide Test".to_string(),
        num_channels: 1,
        initial_speed: 6,
        initial_tempo: 125,
        global_volume: 128,
        mix_volume: 128,
        panning_separation: 128,
        channel_pan: [32; 64],
        channel_vol: [64; 64],
        order_table: vec![0],
        patterns: vec![pattern],
        instruments: vec![instr],
        samples: vec![],
        format: FormatFlags::IS_IT_FORMAT | FormatFlags::INSTRUMENTS,
        message: None,
        restart_position: 0,
    }
}

fn make_it_s_memory_module(first: u8, second: u8, second_note: bool) -> TrackerModule {
    let mut row0 = TrackerNote::default();
    row0.effect = TrackerEffect::ItExtended(first);
    let mut row1 = TrackerNote::default();
    row1.effect = TrackerEffect::ItExtended(second);
    if second_note {
        row1.note = 60;
        row1.instrument = 1;
    }
    let mut module = make_it_module_with_second_row_effect(TrackerEffect::None);
    module.patterns[0] = TrackerPattern {
        num_rows: 2,
        notes: vec![vec![row0], vec![row1]],
    };
    module
}

#[test]
fn it_seek_does_not_replay_past_end_marker() {
    let mut module = make_it_s_memory_module(0xD3, 0x81, false);
    module.patterns = module.patterns[0]
        .notes
        .iter()
        .map(|row| TrackerPattern {
            num_rows: 1,
            notes: vec![row.clone()],
        })
        .collect();
    module.order_table = vec![0, 255, 1];
    let mut engine = TrackerEngine::new();
    let handle = engine.load_tracker_module(module, vec![1]);
    engine.seek_to_position(handle, 2, 0, &[]);
    assert_eq!(engine.current_order, 1);
    assert_eq!(engine.channels[0].last_it_extended, 0xD3);
}

#[test]
fn it_seek_skips_order_marker_preserving_s_memory() {
    let mut module = make_it_s_memory_module(0xD3, 0, true);
    let first = module.patterns[0].notes[0].clone();
    let second = module.patterns[0].notes[1].clone();
    module.patterns = vec![
        TrackerPattern {
            num_rows: 1,
            notes: vec![first],
        },
        TrackerPattern {
            num_rows: 1,
            notes: vec![second],
        },
    ];
    module.order_table = vec![0, 254, 1, 255];
    let mut engine = TrackerEngine::new();
    let handle = engine.load_tracker_module(module, vec![1]);
    engine.seek_to_position(handle, 2, 0, &[]);
    assert_eq!((engine.current_order, engine.current_row), (2, 0));
    assert_eq!(engine.channels[0].last_it_extended, 0xD3);
    engine.process_row_tick0_internal(handle, &[]);
    assert_eq!(engine.channels[0].note_delay_tick, 3);
}

#[test]
fn it_sample_global_volume_scales_foreground_and_background() {
    let render = |global_volume, background| {
        let mut module = make_it_module_with_second_row_effect(TrackerEffect::None);
        module.format = FormatFlags::IS_IT_FORMAT;
        module.samples = vec![nether_tracker::TrackerSample {
            global_volume,
            length: 64,
            ..Default::default()
        }];
        let mut engine = TrackerEngine::new();
        let handle = engine.load_tracker_module(module, vec![1]);
        let sounds = vec![
            None,
            Some(crate::audio::Sound {
                data: vec![12000i16; 64].into(),
            }),
        ];
        engine.process_row_tick0_internal(handle, &sounds);
        engine.channels[0].panning = 0.0;
        engine.channels[0].channel_volume = 64;
        if background {
            engine.channels[1] = engine.channels[0].clone();
            engine.channels[1].is_background = true;
            engine.channels[0].note_on = false;
        }
        (0..16)
            .map(|_| {
                engine
                    .mix_channels(
                        super::super::raw_tracker_handle(handle),
                        &sounds,
                        44100,
                        882,
                    )
                    .0
            })
            .sum::<f32>()
    };
    for background in [false, true] {
        let full = render(64, background);
        assert!(full.abs() > 0.001);
        assert!((render(20, background) / full - 20.0 / 64.0).abs() < 0.0001);
        assert_eq!(render(0, background), 0.0);
    }
}

#[test]
fn it_empty_map_selection_survives_snapshot_without_replacing_voice() {
    let mut module = make_it_module_with_second_row_effect(TrackerEffect::None);
    module.samples = vec![nether_tracker::TrackerSample::default()];
    for (n, entry) in module.instruments[0]
        .note_sample_table
        .iter_mut()
        .enumerate()
    {
        *entry = (n as u8, 1);
    }
    let mut second = module.instruments[0].clone();
    second.note_sample_table[64].1 = 0;
    second.note_sample_table[72].0 = 60;
    module.instruments.push(second);
    module.patterns[0].num_rows = 3;
    module.patterns[0].notes = vec![
        vec![TrackerNote {
            note: 61,
            instrument: 1,
            ..Default::default()
        }],
        vec![TrackerNote {
            note: 65,
            instrument: 2,
            volume: 16,
            effect: TrackerEffect::ItExtended(0x63),
            ..Default::default()
        }],
        vec![TrackerNote {
            note: 73,
            ..Default::default()
        }],
    ];
    let mut engine = TrackerEngine::new();
    let handle = engine.load_tracker_module(module, vec![1]);
    engine.process_row_tick0_internal(handle, &[]);
    let original_period = engine.channels[0].period;
    engine.current_row = 1;
    engine.process_row_tick0_internal(handle, &[]);
    assert_eq!(engine.channels[0].instrument, 1);
    assert_eq!(engine.channels[0].period, original_period);
    assert_eq!(engine.channels[0].last_it_instrument, 2);
    assert_eq!(engine.channels[0].last_it_extended, 0);
    assert_eq!(engine.fine_pattern_delay, 0);
    assert_eq!(engine.resolved_row_notes[0].1, TrackerNote::default());
    let snapshot = engine.snapshot();
    let mut restored = TrackerEngine::new();
    restored.apply_snapshot(&snapshot);
    restored.current_row = 2;
    restored.process_row_tick0_internal(handle, &[]);
    assert_eq!(restored.channels[0].instrument, 2);
    assert_eq!(restored.channels[0].period, original_period);
}

#[test]
fn it_special_notes_remember_selection_without_replacing_active_instrument() {
    for special in [
        TrackerNote::NOTE_OFF,
        TrackerNote::NOTE_CUT,
        TrackerNote::NOTE_FADE,
    ] {
        for selection in [2, 99] {
            let mut module = make_it_module_with_second_row_effect(TrackerEffect::None);
            module.samples = vec![
                nether_tracker::TrackerSample::default(),
                nether_tracker::TrackerSample {
                    default_volume: 16,
                    ..Default::default()
                },
            ];
            for (n, entry) in module.instruments[0]
                .note_sample_table
                .iter_mut()
                .enumerate()
            {
                *entry = (n as u8, 1);
            }
            module.instruments.push(module.instruments[0].clone());
            for entry in &mut module.instruments[1].note_sample_table {
                entry.1 = 2;
            }
            module.patterns[0].notes[1][0] = TrackerNote {
                note: special,
                instrument: selection,
                ..Default::default()
            };
            let mut engine = TrackerEngine::new();
            let handle = engine.load_tracker_module(module, vec![1, 2]);
            engine.process_row_tick0_internal(handle, &[]);
            engine.current_row = 1;
            engine.process_row_tick0_internal(handle, &[]);
            assert_eq!(
                engine.channels[0].instrument, 1,
                "special {special}, selection {selection}"
            );
            assert_eq!(engine.channels[0].last_it_instrument, selection);
            assert_eq!(engine.channels[0].sample_index, Some(0));
            if special != TrackerNote::NOTE_CUT {
                assert_eq!(
                    engine.channels[0].volume,
                    if selection == 2 { 0.25 } else { 1.0 }
                );
            }
        }
    }
}

#[test]
fn it_pan_law_preserves_sum_and_doubles_center_gain_at_edge() {
    let mut engine = TrackerEngine::new();
    engine.reset();
    let mut module = make_it_module_with_second_row_effect(TrackerEffect::None);
    module.samples = vec![nether_tracker::TrackerSample::default()];
    for entry in &mut module.instruments[0].note_sample_table {
        entry.1 = 1;
    }
    let handle = engine.load_tracker_module(module, vec![1]);
    engine.process_row_tick0_internal(handle, &[]);
    let sounds = vec![
        None,
        Some(crate::audio::Sound {
            data: vec![4096; 64].into(),
        }),
    ];
    engine.channels[0].fade_in_samples = 0;
    engine.channels[0].channel_volume = 64;
    engine.channels[0].panning = 0.0;
    let center = engine.mix_channels(
        super::super::raw_tracker_handle(handle),
        &sounds,
        44100,
        882,
    );
    assert!(center.0 > 0.0, "channel={:?}", engine.channels[0]);
    let snapshot = engine.snapshot();
    for background in [false, true] {
        engine.apply_snapshot(&snapshot);
        let voice = if background { 32 } else { 0 };
        if background {
            engine.channels[voice] = engine.channels[0].copy_to_background(0);
            engine.channels[0].note_on = false;
        }
        // The explicit mixer handle owns the pan law, even with stale row state.
        engine.is_it_format = false;
        for pan in [-1.0, -0.5, 0.0, 0.5, 1.0] {
            engine.channels[voice].panning = pan;
            let (left, right) = engine.mix_channels(
                super::super::raw_tracker_handle(handle),
                &sounds,
                44100,
                882,
            );
            assert!((left + right - center.0 - center.1).abs() < 0.000001);
            assert!((right / (left + right) - (pan + 1.0) * 0.5).abs() < 0.000001);
        }
    }
}

#[test]
fn it_sample_special_recalls_volume_without_replacing_voice() {
    for special in [
        TrackerNote::NOTE_OFF,
        TrackerNote::NOTE_FADE,
        TrackerNote::NOTE_CUT,
    ] {
        let mut module = make_it_module_with_second_row_effect(TrackerEffect::None);
        module.format = FormatFlags::IS_IT_FORMAT;
        module.samples = vec![
            nether_tracker::TrackerSample::default(),
            nether_tracker::TrackerSample {
                default_volume: 16,
                loop_begin: 10,
                loop_end: 30,
                ..Default::default()
            },
        ];
        module.patterns[0].notes[1][0] = TrackerNote {
            note: special,
            instrument: 2,
            ..Default::default()
        };
        let mut engine = TrackerEngine::new();
        let handle = engine.load_tracker_module(module, vec![1, 2]);
        engine.process_row_tick0_internal(handle, &[]);
        engine.channels[0].sample_pos = 7.0;
        engine.current_row = 1;
        engine.process_row_tick0_internal(handle, &[]);
        let channel = &engine.channels[0];
        assert_eq!(channel.sample_handle, 1);
        assert_eq!(channel.sample_index, Some(0));
        assert_eq!(channel.sample_pos, 7.0);
        if special != TrackerNote::NOTE_CUT {
            assert_eq!(channel.volume, 0.25);
        }
    }
}

#[test]
fn it_old_effects_noteoff_changes_envelope_not_sample() {
    for old in [false, true] {
        let mut module = make_it_module_with_second_row_effect(TrackerEffect::None);
        if old {
            module.format = module.format | FormatFlags::OLD_EFFECTS;
        }
        module.instruments[0].volume_envelope = Some(nether_tracker::TrackerEnvelope {
            points: vec![(0, 64), (12, 64)],
            flags: nether_tracker::EnvelopeFlags::ENABLED,
            ..Default::default()
        });
        let mut next = module.instruments[0].clone();
        next.volume_envelope.as_mut().unwrap().points = vec![(0, 16), (8, 16)];
        next.fadeout = 32;
        next.nna = nether_tracker::NewNoteAction::Continue;
        module.samples = vec![
            nether_tracker::TrackerSample::default(),
            nether_tracker::TrackerSample {
                default_pan: Some(64),
                ..Default::default()
            },
        ];
        for entry in &mut next.note_sample_table {
            entry.1 = 2;
        }
        module.instruments.push(next);
        module.patterns[0].notes[1][0] = TrackerNote {
            note: TrackerNote::NOTE_OFF,
            instrument: 2,
            ..Default::default()
        };
        let mut engine = TrackerEngine::new();
        let handle = engine.load_tracker_module(module, vec![1]);
        engine.process_row_tick0_internal(handle, &[]);
        engine.channels[0].sample_pos = 7.0;
        engine.channels[0].volume_envelope_pos = 5;
        engine.channels[0].volume_fadeout = 30000;
        engine.current_row = 1;
        engine.process_row_tick0_internal(handle, &[]);
        let channel = &engine.channels[0];
        assert_eq!(channel.instrument, if old { 2 } else { 1 });
        assert_eq!(channel.nna, if old { 1 } else { 0 });
        assert_eq!(channel.panning, if old { 1.0 } else { 0.0 });
        assert_eq!(channel.volume_envelope_end, Some(if old { 8 } else { 12 }));
        assert_eq!(channel.volume_envelope_pos, if old { 0 } else { 5 });
        assert_eq!(channel.volume_fadeout, if old { 65535 } else { 30000 });
        assert_eq!(channel.key_off, !old);
        assert!(channel.sample_sustain_released);
        assert_eq!(channel.sample_pos, 7.0);
    }
}

#[test]
fn it_note_dispatch_carries_terminal_envelope_into_nna_and_snapshot() {
    let mut module = make_it_module_with_second_row_effect(TrackerEffect::None);
    module.instruments[0].volume_envelope = Some(nether_tracker::TrackerEnvelope {
        points: vec![(0, 64), (3, 64)],
        flags: nether_tracker::EnvelopeFlags::ENABLED,
        ..Default::default()
    });
    module.instruments[0].fadeout = 16;
    let mut engine = TrackerEngine::new();
    let handle = engine.load_tracker_module(module, vec![1]);
    engine.process_row_tick0_internal(handle, &[]);
    assert_eq!(engine.channels[0].volume_envelope_end, Some(3));
    engine.channels[32] = engine.channels[0].copy_to_background(0);
    let snapshot = engine.snapshot();
    engine.reset();
    engine.apply_snapshot(&snapshot);
    for _ in 0..3 {
        engine.advance_envelopes();
    }
    for index in [0, 32] {
        assert!(!engine.channels[index].note_fade);
    }
    engine.advance_envelopes();
    for index in [0, 32] {
        assert!(engine.channels[index].note_fade);
        assert!(engine.channels[index].volume_fadeout < u16::MAX);
    }
}

#[test]
fn it_keyoff_waits_for_nonlooping_envelope_but_direct_fade_does_not() {
    for (enabled, looped, direct_fade, fades_now) in [
        (true, false, false, false),
        (true, true, false, true),
        (false, false, false, true),
        (true, false, true, true),
    ] {
        let mut engine = TrackerEngine::new();
        engine.is_it_format = true;
        for index in [0, 32] {
            let channel = &mut engine.channels[index];
            channel.note_on = true;
            channel.is_background = index == 32;
            channel.key_off = true;
            channel.note_fade = direct_fade;
            channel.volume_envelope_enabled = enabled;
            channel.volume_envelope_end = Some(3);
            channel.volume_envelope_loop = looped.then_some((0, 3));
            channel.instrument_fadeout_rate = 16;
            channel.volume_fadeout = 1024;
        }
        engine.advance_envelopes();
        for index in [0, 32] {
            assert_eq!(
                engine.channels[index].volume_fadeout,
                if fades_now { 1008 } else { 1024 }
            );
        }
        let snapshot = engine.snapshot();
        engine.reset();
        engine.apply_snapshot(&snapshot);
        if enabled && !looped && !direct_fade {
            for _ in 0..3 {
                engine.advance_envelopes();
            }
            for index in [0, 32] {
                assert!(engine.channels[index].note_fade);
                assert_eq!(engine.channels[index].volume_fadeout, 1008);
            }
        }
    }
}

#[test]
fn it_invalid_instrument_selection_survives_snapshot_and_reset() {
    let mut engine = TrackerEngine::new();
    engine.channels[0].instrument = 1;
    engine.channels[0].last_it_instrument = 99;
    let snapshot = engine.snapshot();
    let mut restored = TrackerEngine::new();
    restored.apply_snapshot(&snapshot);
    assert_eq!(restored.channels[0].instrument, 1);
    assert_eq!(restored.channels[0].last_it_instrument, 99);
    restored.reset();
    assert_eq!(restored.channels[0].last_it_instrument, 0);
}

#[test]
fn it_s00_recalls_note_delay_and_snapshot_memory() {
    let mut engine = TrackerEngine::new();
    let handle = engine.load_tracker_module(make_it_s_memory_module(0xD3, 0x00, true), vec![1]);

    engine.current_row = 0;
    engine.process_row_tick0_internal(handle, &[]);
    let snapshot = engine.snapshot();

    let mut restored = TrackerEngine::new();
    restored.apply_snapshot(&snapshot);
    restored.current_row = 1;
    restored.process_row_tick0_internal(handle, &[]);

    assert_eq!(restored.channels[0].last_it_extended, 0xD3);
    assert_eq!(restored.channels[0].note_delay_tick, 3);
    assert!(restored.channels[0].delayed_xm_note.is_some());
}

#[test]
fn it_s00_recall_reaches_fine_and_row_delay_collectors() {
    for (memory, expected_fine, expected_rows) in [(0x63, 3, 0), (0xE1, 0, 1)] {
        let mut engine = TrackerEngine::new();
        let handle =
            engine.load_tracker_module(make_it_s_memory_module(memory, 0x00, false), vec![1]);
        engine.current_row = 0;
        engine.process_row_tick0_internal(handle, &[]);
        engine.current_row = 1;
        engine.process_row_tick0_internal(handle, &[]);

        assert_eq!(engine.fine_pattern_delay, expected_fine);
        assert_eq!(engine.pattern_delay, expected_rows);
    }
}

#[test]
fn both_effect_columns_and_explicit_silence_reach_runtime() {
    let mut engine = TrackerEngine::new();
    let handle = engine.load_tracker_module(
        make_it_module_with_second_row_effect(TrackerEffect::None),
        vec![1],
    );
    let note = TrackerNote {
        note: 48,
        instrument: 1,
        volume: 32,
        volume_effect: TrackerEffect::VolumeSlide { up: 0, down: 1 },
        effect: TrackerEffect::SetPanning(0),
    };
    engine.process_note_internal(0, &note, handle, &[]);
    assert_eq!(engine.channels[0].panning, -1.0);
    engine.process_tick(1, 6);
    assert_eq!(engine.channels[0].volume, 31.0 / 64.0);
    engine.process_note_internal(
        0,
        &TrackerNote {
            volume_effect: TrackerEffect::SetVolume(0),
            ..Default::default()
        },
        handle,
        &[],
    );
    assert_eq!(engine.channels[0].volume, 0.0);
}

#[test]
fn xm_instrument_default_volume_is_reloaded_before_row_volume() {
    let mut module = make_it_module_with_second_row_effect(TrackerEffect::None);
    module.format = FormatFlags::empty();
    module.instruments[0].sample_default_volume = Some(32);

    let mut engine = TrackerEngine::new();
    let handle = engine.load_tracker_module(module, vec![1]);
    engine.process_note_internal(
        0,
        &TrackerNote {
            note: 48,
            instrument: 1,
            ..Default::default()
        },
        handle,
        &[],
    );
    assert_eq!(engine.channels[0].volume, 32.0 / 64.0);

    engine.process_note_internal(
        0,
        &TrackerNote {
            note: 49,
            instrument: 1,
            volume: 16,
            ..Default::default()
        },
        handle,
        &[],
    );
    assert_eq!(engine.channels[0].volume, 16.0 / 64.0);
}
#[test]
fn uninterrupted_tracker_matches_frame_chunks_and_fast_clock() {
    use crate::audio::Sound;
    use crate::state::{TrackerState, tracker_flags};

    let sounds = vec![
        None,
        Some(Sound {
            data: (0..22050)
                .map(|i| ((i as f32 * 0.017).sin() * 12000.0) as i16)
                .collect::<Vec<_>>()
                .into(),
        }),
    ];
    let setup = || {
        let mut engine = TrackerEngine::new();
        let module = make_it_module_with_second_row_effect(TrackerEffect::None);
        let handle = engine.load_tracker_module(module, vec![1]);
        let state = TrackerState {
            handle,
            flags: tracker_flags::PLAYING | tracker_flags::LOOPING,
            speed: 6,
            bpm: 125,
            volume: 256,
            ..Default::default()
        };
        (engine, state)
    };
    let (mut continuous, mut continuous_state) = setup();
    let (mut chunked, mut chunked_state) = setup();
    let (mut fast, mut fast_state) = setup();
    continuous.sync_to_state(&continuous_state, &sounds);
    let mut peak = 0.0f32;
    for frame in 0..30 {
        chunked.sync_to_state(&chunked_state, &sounds);
        fast.sync_to_state(&fast_state, &sounds);
        for sample in 0..735 {
            let expected =
                continuous.render_sample_and_advance(&mut continuous_state, &sounds, 44100);
            let actual = chunked.render_sample_and_advance(&mut chunked_state, &sounds, 44100);
            peak = peak.max(expected.0.abs()).max(expected.1.abs());
            assert_eq!(
                actual, expected,
                "PCM reset at frame {frame}, sample {sample}"
            );
        }
        fast.advance_positions(&mut fast_state, &sounds, 735, 44100);
        assert_eq!(
            (fast.current_tick, fast.tick_samples_rendered),
            (fast_state.tick, fast_state.tick_sample_pos),
            "fast clock at frame {frame}"
        );
        assert_eq!(
            (chunked.current_tick, chunked.tick_samples_rendered),
            (chunked_state.tick, chunked_state.tick_sample_pos),
            "render clock at frame {frame}"
        );
    }
    assert!(peak > 0.01, "fixture must produce real non-silent PCM");
}

#[test]
fn test_nna_uses_current_voice_action_before_new_instrument() {
    // Pinned OpenMPT CheckNNA uses srcChn.nNNA, not incoming instrument NNA.

    let mut engine = TrackerEngine::new();

    // Create instrument with NNA=Continue (should move old notes to background)
    let mut instr = TrackerInstrument::default();
    instr.nna = NewNoteAction::Continue;
    instr.fadeout = 1024; // Non-zero fadeout for audibility

    // Create pattern with a note on row 1
    let note2 = TrackerNote {
        note: 60,
        instrument: 1,
        volume: 64,
        effect: TrackerEffect::None,
        ..Default::default()
    };

    let pattern = TrackerPattern {
        num_rows: 2,
        notes: vec![
            vec![TrackerNote::default()], // Row 0: empty
            vec![note2],                  // Row 1: trigger C-5
        ],
    };

    let module = TrackerModule {
        name: "NNA Test".to_string(),
        num_channels: 1,
        initial_speed: 6,
        initial_tempo: 125,
        global_volume: 128,
        mix_volume: 128,
        panning_separation: 128,
        channel_pan: [32; 64],
        channel_vol: [64; 64],
        order_table: vec![0],
        patterns: vec![pattern],
        instruments: vec![instr],
        samples: vec![],
        format: FormatFlags::IS_IT_FORMAT | FormatFlags::INSTRUMENTS,
        message: None,
        restart_position: 0,
    };

    let handle = engine.load_tracker_module(module, vec![1]); // Sample handle 1
    engine.is_it_format = true;

    // Simulate channel 0 already playing a note (as if row 0 was processed)
    engine.channels[0].note_on = true;
    engine.channels[0].sample_handle = 1;
    engine.channels[0].volume = 1.0;
    engine.channels[0].volume_fadeout = 65535;
    engine.channels[0].nna = NNA_CUT; // Authoritative action for the displaced voice.
    engine.channels[0].instrument = 1;

    // Process row 1 - new note with NNA=Continue instrument
    // The old voice uses Cut; Continue is installed for the incoming note.
    engine.current_row = 1;
    engine.process_row_tick0_internal(handle, &[]);

    assert!(
        !engine.channels[1].note_on,
        "old Cut voice must not be virtualized"
    );
    assert_eq!(
        engine.channels[0].nna, 1,
        "incoming Continue applies to the next displacement"
    );
}

#[test]
fn test_nna_note_fade_preserves_key() {
    // Verify NoteFade starts fading without releasing the displaced key.

    let mut engine = TrackerEngine::new();

    let mut instr = TrackerInstrument::default();
    instr.nna = NewNoteAction::NoteFade;
    instr.fadeout = 2048;

    let note = TrackerNote {
        note: 60,
        instrument: 1,
        volume: 64,
        effect: TrackerEffect::None,
        ..Default::default()
    };

    let pattern = TrackerPattern {
        num_rows: 2,
        notes: vec![vec![TrackerNote::default()], vec![note]],
    };

    let module = TrackerModule {
        name: "NNA Fade Test".to_string(),
        num_channels: 1,
        initial_speed: 6,
        initial_tempo: 125,
        global_volume: 128,
        mix_volume: 128,
        panning_separation: 128,
        channel_pan: [32; 64],
        channel_vol: [64; 64],
        order_table: vec![0],
        patterns: vec![pattern],
        instruments: vec![instr],
        samples: vec![],
        format: FormatFlags::IS_IT_FORMAT | FormatFlags::INSTRUMENTS,
        message: None,
        restart_position: 0,
    };

    let handle = engine.load_tracker_module(module, vec![1]);
    engine.is_it_format = true;

    // Set up a playing voice with NoteFade.
    engine.channels[0].note_on = true;
    engine.channels[0].sample_handle = 1;
    engine.channels[0].volume = 1.0;
    engine.channels[0].volume_fadeout = 65535;
    engine.channels[0].nna = 3; // The displaced voice uses NoteFade.
    engine.channels[0].instrument = 1;
    engine.channels[0].instrument_fadeout_rate = 0; // Authored zero is preserved

    engine.current_row = 1;
    engine.process_row_tick0_internal(handle, &[]);

    // Background channel keeps sustain while explicit fading is enabled.
    assert!(
        engine.channels[1].note_on,
        "NNA=NoteFade should move note to background"
    );
    assert!(
        !engine.channels[1].key_off,
        "NNA=NoteFade must preserve key"
    );
    assert!(engine.channels[1].note_fade);
    assert_eq!(engine.channels[1].instrument_fadeout_rate, 0);
}

#[test]
fn test_it_portamento_up_decreases_period() {
    let mut engine = TrackerEngine::new();
    let module = make_it_module_with_second_row_effect(TrackerEffect::PortamentoUp(4));
    let handle = engine.load_tracker_module(module, vec![1]);

    engine.current_order = 0;
    engine.current_row = 0;
    engine.process_row_tick0_internal(handle, &[]);
    let start_period = engine.channels[0].period;

    engine.current_row = 1;
    engine.process_row_tick0_internal(handle, &[]);
    engine.process_tick(1, 6);

    assert!(
        engine.channels[0].period < start_period,
        "IT portamento up should decrease period (raise pitch)"
    );
}

#[test]
fn test_it_portamento_down_increases_period() {
    let mut engine = TrackerEngine::new();
    let module = make_it_module_with_second_row_effect(TrackerEffect::PortamentoDown(4));
    let handle = engine.load_tracker_module(module, vec![1]);

    engine.current_order = 0;
    engine.current_row = 0;
    engine.process_row_tick0_internal(handle, &[]);
    let start_period = engine.channels[0].period;

    engine.current_row = 1;
    engine.process_row_tick0_internal(handle, &[]);
    engine.process_tick(1, 6);

    assert!(
        engine.channels[0].period > start_period,
        "IT portamento down should increase period (lower pitch)"
    );
}

#[test]
fn test_it_fine_porta_directions_on_tick0() {
    let mut engine = TrackerEngine::new();
    let up_module = make_it_module_with_second_row_effect(TrackerEffect::FinePortaUp(4));
    let handle = engine.load_tracker_module(up_module, vec![1]);

    engine.current_order = 0;
    engine.current_row = 0;
    engine.process_row_tick0_internal(handle, &[]);
    let start_period = engine.channels[0].period;

    engine.current_row = 1;
    engine.process_row_tick0_internal(handle, &[]);
    let up_period = engine.channels[0].period;

    assert!(
        up_period < start_period,
        "IT fine portamento up should decrease period on tick 0"
    );

    let mut engine = TrackerEngine::new();
    let down_module = make_it_module_with_second_row_effect(TrackerEffect::FinePortaDown(4));
    let handle = engine.load_tracker_module(down_module, vec![1]);

    engine.current_order = 0;
    engine.current_row = 0;
    engine.process_row_tick0_internal(handle, &[]);
    let start_period = engine.channels[0].period;

    engine.current_row = 1;
    engine.process_row_tick0_internal(handle, &[]);
    let down_period = engine.channels[0].period;

    assert!(
        down_period > start_period,
        "IT fine portamento down should increase period on tick 0"
    );
}

#[test]
fn test_tone_porta_note_does_not_retrigger_active_note() {
    for format in [FormatFlags::empty(), FormatFlags::IS_IT_FORMAT] {
        for (volume_effect, effect) in [
            (TrackerEffect::None, TrackerEffect::TonePortamento(4)),
            (TrackerEffect::TonePortamento(4), TrackerEffect::None),
            (
                TrackerEffect::None,
                TrackerEffect::TonePortaVolSlide {
                    porta: 0,
                    vol_up: 0,
                    vol_down: 1,
                },
            ),
        ] {
            check_tone_porta_continues(format, volume_effect, effect);
        }
    }
}

fn check_tone_porta_continues(
    format: FormatFlags,
    volume_effect: TrackerEffect,
    effect: TrackerEffect,
) {
    let mut engine = TrackerEngine::new();
    let instr = TrackerInstrument {
        sample_relative_note: -12,
        fadeout: 1024,
        ..Default::default()
    };

    let row0 = TrackerNote {
        note: 48,
        instrument: 1,
        volume: 64,
        effect: TrackerEffect::None,
        ..Default::default()
    };
    let row1 = TrackerNote {
        note: 60,
        instrument: 1,
        volume: 64,
        effect,
        volume_effect,
    };
    let pattern = TrackerPattern {
        num_rows: 2,
        notes: vec![vec![row0], vec![row1]],
    };
    let module = TrackerModule {
        name: "Tone Porta Trigger Test".to_string(),
        num_channels: 1,
        initial_speed: 6,
        initial_tempo: 125,
        global_volume: 128,
        mix_volume: 128,
        panning_separation: 128,
        channel_pan: [32; 64],
        channel_vol: [64; 64],
        order_table: vec![0],
        patterns: vec![pattern],
        instruments: vec![instr],
        samples: vec![],
        format: format | FormatFlags::INSTRUMENTS,
        message: None,
        restart_position: 0,
    };

    let handle = engine.load_tracker_module(module, vec![1]);

    engine.current_order = 0;
    engine.current_row = 0;
    engine.process_row_tick0_internal(handle, &[]);

    // The combined command recalls an earlier portamento speed.
    engine.channels[0].porta_speed = 4;
    // Simulate in-flight sample playback before the porta row.
    engine.channels[0].sample_pos = 12.5;
    let before_period = engine.channels[0].period;

    engine.current_row = 1;
    engine.process_row_tick0_internal(handle, &[]);

    // Gxx with a new note should continue current note instead of retriggering.
    assert!(
        (engine.channels[0].sample_pos - 12.5).abs() < f64::EPSILON,
        "Tone portamento note should not reset sample position"
    );
    assert!(
        (engine.channels[0].period - before_period).abs() < f32::EPSILON,
        "Tone portamento note should not jump period on tick 0"
    );

    let target = engine.channels[0].target_period;
    assert_eq!(target, super::super::utils::note_to_period(48, 0));
    engine.process_tick(1, 6);
    let after_tick = engine.channels[0].period;
    assert!(
        after_tick < before_period,
        "Period should slide toward target"
    );
    assert!(
        after_tick > target,
        "Slide should not jump directly to target"
    );
}
