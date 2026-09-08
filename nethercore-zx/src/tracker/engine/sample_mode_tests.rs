use nether_tracker::{FormatFlags, TrackerModule, TrackerNote, TrackerPattern, TrackerSample};

use super::super::TrackerEngine;
use super::super::utils::note_to_period;

#[test]
fn it_sample_mode_triggers_c5_and_retains_sample_without_instrument() {
    let mut sample = TrackerSample {
        default_volume: 32,
        loop_begin: 2,
        loop_end: 6,
        loop_type: nether_tracker::LoopType::Forward,
        sustain_loop_begin: 3,
        sustain_loop_end: 5,
        sustain_loop_type: nether_tracker::LoopType::PingPong,
        ..Default::default()
    };
    sample.c5_speed = 22_050;

    let module = TrackerModule {
        name: String::new(),
        num_channels: 1,
        initial_speed: 6,
        initial_tempo: 125,
        global_volume: 128,
        mix_volume: 128,
        panning_separation: 128,
        channel_pan: [32; 64],
        channel_vol: [64; 64],
        order_table: vec![0],
        patterns: vec![TrackerPattern {
            num_rows: 4,
            notes: vec![
                vec![TrackerNote {
                    note: 61, // IT C-5 after unified one-based conversion
                    instrument: 1,
                    ..Default::default()
                }],
                vec![TrackerNote {
                    note: 62,
                    instrument: 0,
                    ..Default::default()
                }],
                vec![TrackerNote {
                    note: 63,
                    instrument: 1,
                    ..Default::default()
                }],
                vec![TrackerNote {
                    instrument: 1,
                    ..Default::default()
                }],
            ],
        }],
        instruments: Vec::new(),
        samples: vec![sample],
        format: FormatFlags::IS_IT_FORMAT,
        message: None,
        restart_position: 0,
    };

    let mut engine = TrackerEngine::new();
    let handle = engine.load_tracker_module(module, vec![7]);
    let sounds = Vec::<Option<crate::audio::Sound>>::new();

    engine.process_row_tick0_internal(handle, &sounds);
    let channel = &engine.channels[0];
    assert_eq!(channel.instrument, 1);
    assert_eq!(channel.sample_handle, 7);
    assert_eq!(channel.volume, 0.5);
    assert_eq!(channel.sample_loop_start, 2);
    assert_eq!(channel.sample_loop_end, 6);
    assert_eq!(channel.sample_loop_type, 1);
    assert_eq!(channel.sample_sustain_loop_start, 3);
    assert_eq!(channel.sample_sustain_loop_end, 5);
    assert_eq!(channel.sample_sustain_loop_type, 2);
    assert_eq!(channel.base_period, note_to_period(49, 0));

    engine.current_row = 1;
    engine.process_row_tick0_internal(handle, &sounds);
    let channel = &engine.channels[0];
    assert_eq!(channel.instrument, 1);
    assert_eq!(channel.sample_handle, 7);
    assert_eq!(channel.volume, 0.5);
    assert_eq!(channel.base_period, note_to_period(50, 0));
    engine.current_row = 2;
    engine.process_row_tick0_internal(handle, &sounds);
    assert!(engine.channels[0].note_on);
    assert_eq!(
        engine.channels[0].volume, 0.5,
        "NNA cut must not erase the new sample's volume"
    );
    engine.channels[0].note_on = false;
    engine.channels[0].sample_pos = 99.0;
    engine.channels[0].base_period = 999.0;
    engine.channels[0].target_period = 123.0;
    engine.current_row = 3;
    engine.process_row_tick0_internal(handle, &sounds);
    assert!(
        engine.channels[0].note_on,
        "instrument-only restarts stopped IT sample"
    );
    assert_eq!(engine.channels[0].sample_pos, 0.0);
    assert_eq!(engine.channels[0].base_period, note_to_period(51, 0));
}

#[test]
fn it_sample_mode_porta_swap_resets_position_only_for_changed_sample() {
    for compatible in [false, true] {
        let note = |pitch, instrument, porta| TrackerNote {
            note: pitch,
            instrument,
            effect: if porta {
                nether_tracker::TrackerEffect::TonePortamento(32)
            } else {
                nether_tracker::TrackerEffect::None
            },
            ..Default::default()
        };
        let module = TrackerModule {
            name: "sample-mode porta position".into(),
            num_channels: 1,
            initial_speed: 6,
            initial_tempo: 125,
            global_volume: 128,
            mix_volume: 128,
            panning_separation: 128,
            channel_pan: [32; 64],
            channel_vol: [64; 64],
            order_table: vec![0],
            patterns: vec![TrackerPattern {
                num_rows: 3,
                notes: vec![
                    vec![note(61, 1, false)],
                    vec![note(68, 2, true)],
                    vec![note(61, 2, true)],
                ],
            }],
            instruments: vec![],
            samples: vec![TrackerSample::default(); 2],
            format: FormatFlags::IS_IT_FORMAT
                | if compatible {
                    FormatFlags::LINK_G_MEMORY
                } else {
                    FormatFlags::empty()
                },
            message: None,
            restart_position: 0,
        };
        let mut engine = TrackerEngine::new();
        let handle = engine.load_tracker_module(module, vec![7, 8]);
        engine.process_row_tick0_internal(handle, &[]);
        let period = engine.channels[0].base_period;
        engine.channels[0].sample_pos = 5.5;
        engine.current_row = 1;
        engine.process_row_tick0_internal(handle, &[]);
        assert_eq!(
            engine.channels[0].sample_handle,
            if compatible { 7 } else { 8 }
        );
        assert_eq!(
            engine.channels[0].sample_pos,
            if compatible { 5.5 } else { 0.0 }
        );
        assert_eq!(engine.channels[0].base_period, period);
        engine.channels[0].sample_pos = 3.5;
        engine.current_row = 2;
        engine.process_row_tick0_internal(handle, &[]);
        assert_eq!(
            engine.channels[0].sample_pos, 3.5,
            "same sample does not restart"
        );
    }
}

#[test]
fn it_instrument_porta_switches_mapped_sample_without_releasing_sustain_again() {
    instrument_porta_sustain_case(false);
}

#[test]
fn it_compatible_gxx_keeps_mapped_sample_on_instrument_porta() {
    instrument_porta_sustain_case(true);
}

fn instrument_porta_sustain_case(compatible: bool) {
    use nether_tracker::TrackerInstrument;

    let mut samples = vec![TrackerSample::default(); 4];
    samples[0] = TrackerSample {
        default_volume: 64,
        loop_begin: 1,
        loop_end: 6,
        loop_type: nether_tracker::LoopType::Forward,
        sustain_loop_begin: 2,
        sustain_loop_end: 4,
        sustain_loop_type: nether_tracker::LoopType::Forward,
        ..Default::default()
    };
    samples[3] = TrackerSample {
        default_volume: 32,
        loop_begin: 3,
        loop_end: 7,
        loop_type: nether_tracker::LoopType::Forward,
        sustain_loop_begin: 1,
        sustain_loop_end: 3,
        sustain_loop_type: nether_tracker::LoopType::Forward,
        ..Default::default()
    };

    let mut first = TrackerInstrument::default();
    first.note_sample_table[47] = (48, 1);
    first.note_sample_table[71] = (72, 4);
    let mut second = TrackerInstrument::default();
    second.note_sample_table[50] = (51, 1);
    second.note_sample_table[62] = (63, 1);

    let porta = |note, instrument| TrackerNote {
        note,
        instrument,
        effect: nether_tracker::TrackerEffect::TonePortamento(16),
        ..Default::default()
    };
    let module = TrackerModule {
        name: "IT instrument porta regression".into(),
        num_channels: 1,
        initial_speed: 6,
        initial_tempo: 125,
        global_volume: 128,
        mix_volume: 128,
        panning_separation: 128,
        channel_pan: [32; 64],
        channel_vol: [64; 64],
        order_table: vec![0],
        patterns: vec![TrackerPattern {
            num_rows: 8,
            notes: vec![
                vec![TrackerNote {
                    note: 48,
                    instrument: 1,
                    ..Default::default()
                }],
                vec![TrackerNote {
                    note: TrackerNote::NOTE_OFF,
                    ..Default::default()
                }],
                vec![porta(51, 2)],
                vec![porta(52, 1)],
                vec![porta(63, 2)],
                vec![porta(72, 1)],
                vec![porta(48, 0)],
                vec![porta(48, 1)],
            ],
        }],
        instruments: vec![first, second],
        samples,
        format: FormatFlags::IS_IT_FORMAT
            | FormatFlags::INSTRUMENTS
            | if compatible {
                FormatFlags::LINK_G_MEMORY
            } else {
                FormatFlags::empty()
            },
        message: None,
        restart_position: 0,
    };

    let mut engine = TrackerEngine::new();
    let handle = engine.load_tracker_module(module, vec![7, 8, 9, 10]);
    let sounds = Vec::<Option<crate::audio::Sound>>::new();

    engine.current_row = 0;
    engine.process_row_tick0_internal(handle, &sounds);
    engine.channels[0].sample_direction = -1;
    engine.current_row = 1;
    engine.process_row_tick0_internal(handle, &sounds);
    assert!(engine.channels[0].key_off);

    engine.current_row = 2;
    engine.process_row_tick0_internal(handle, &sounds);
    let channel = &engine.channels[0];
    assert_eq!(channel.instrument, 2);
    assert_eq!(channel.sample_handle, 7);
    assert!(!channel.key_off);
    assert_eq!(channel.sample_direction, -1);
    assert_eq!(channel.sample_loop_start, 1);
    assert!(channel.sample_sustain_released);

    engine.current_row = 3;
    engine.process_row_tick0_internal(handle, &sounds);
    assert_eq!(engine.channels[0].instrument, 1);
    assert!(engine.channels[0].sample_sustain_released);

    engine.current_row = 4;
    engine.process_row_tick0_internal(handle, &sounds);
    assert_eq!(engine.channels[0].instrument, 2);
    engine.channels[0].sample_pos = 5.5;
    engine.current_row = 5;
    engine.process_row_tick0_internal(handle, &sounds);
    assert_eq!(
        engine.channels[0].sample_pos,
        if compatible { 5.5 } else { 0.0 }
    );
    assert_eq!(
        engine.channels[0].sample_handle,
        if compatible { 7 } else { 10 }
    );
    assert_eq!(
        engine.channels[0].sample_index,
        Some(if compatible { 0 } else { 3 })
    );
    engine.channels[0].sample_pos = 5.5;
    engine.channels[0].key_off = true;
    engine.channels[0].sample_sustain_released = true;
    engine.current_row = 6;
    engine.process_row_tick0_internal(handle, &sounds);
    // OpenMPT Snd_fx.cpp:3095 and 2080: instrument-less porta retains playback.
    assert_eq!(
        engine.channels[0].sample_handle,
        if compatible { 7 } else { 10 }
    );
    assert_eq!(
        engine.channels[0].sample_index,
        Some(if compatible { 0 } else { 3 })
    );
    assert_eq!(engine.channels[0].sample_pos, 5.5);
    assert!(engine.channels[0].key_off);
    engine.current_row = 7;
    engine.process_row_tick0_internal(handle, &sounds);
    assert_eq!(engine.channels[0].sample_handle, 7);
    assert_eq!(engine.channels[0].sample_index, Some(0));
    assert_eq!(
        engine.channels[0].sample_pos,
        if compatible { 5.5 } else { 0.0 }
    );
    assert!(
        engine.channels[0].key_off,
        "same instrument must not clear release"
    );
}
