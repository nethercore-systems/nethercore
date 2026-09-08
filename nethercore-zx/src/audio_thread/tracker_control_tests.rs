//! Exercise the real predictive-thread merge, not the mirrored test helper.
use super::*;
use crate::audio::Sound;
use crate::state::tracker_flags;
use nether_tracker::{
    FormatFlags, LoopType, TrackerInstrument, TrackerModule, TrackerNote, TrackerPattern,
};
use ringbuf::{HeapRb, traits::Split};

#[test]
fn same_handle_controls_replace_snapshot_but_ordinary_updates_keep_prediction() {
    same_handle_controls(false);
}

#[test]
fn it_same_handle_controls_preserve_swing_and_subsequent_pcm() {
    same_handle_controls(true);
}

fn same_handle_controls(it: bool) {
    let (_, rx) = mpsc::sync_channel(8);
    let (producer, _consumer) = HeapRb::<f32>::new(RING_BUFFER_CAPACITY).split();
    let mut audio = AudioGenThread {
        rx,
        producer,
        condvar: Arc::new((Mutex::new(false), Condvar::new())),
        output_buffer: Vec::new(),
        tracker_engine: TrackerEngine::new(),
        sample_rate: 44100,
        gen_audio: AudioPlaybackState::default(),
        gen_tracker: TrackerState::default(),
        last_snapshot: None,
        samples_since_snapshot: 0,
        has_state: false,
        prev_frame_last: (0.0, 0.0),
        crossfade_samples: 44,
        crossfade_active: false,
        crossfade_from: (0.0, 0.0),
        metrics: AudioMetrics::new(),
    };
    let mut pattern = TrackerPattern::empty(8, 1);
    pattern.notes[0][0] = TrackerNote {
        note: 49,
        instrument: 1,
        ..Default::default()
    };
    let mut song = TrackerModule {
        name: "control regression".into(),
        num_channels: 1,
        initial_speed: 3,
        initial_tempo: 125,
        global_volume: 64,
        mix_volume: 48,
        panning_separation: 128,
        channel_pan: [32; 64],
        channel_vol: [64; 64],
        order_table: vec![0],
        patterns: vec![pattern],
        instruments: vec![TrackerInstrument {
            sample_loop_type: LoopType::Forward,
            sample_loop_end: 7,
            ..Default::default()
        }],
        samples: vec![],
        format: FormatFlags::IS_XM_FORMAT | FormatFlags::INSTRUMENTS,
        message: None,
        restart_position: 0,
    };
    if it {
        song.format = FormatFlags::IS_IT_FORMAT | FormatFlags::INSTRUMENTS;
        song.samples = vec![nether_tracker::TrackerSample {
            loop_type: LoopType::Forward,
            loop_end: 7,
            ..Default::default()
        }];
        song.instruments[0].random_volume = 20;
        song.instruments[0].random_pan = 8;
    }
    let mut main = TrackerEngine::new();
    let handle = 1;
    main.modules = vec![
        None,
        Some(Arc::new(crate::tracker::LoadedModule {
            module: song,
            sound_handles: vec![1],
        })),
    ];
    let mut state = TrackerState {
        handle,
        speed: 3,
        bpm: 125,
        volume: 256,
        flags: tracker_flags::PLAYING | tracker_flags::LOOPING,
        ..Default::default()
    };
    let sounds = Arc::new(vec![
        None,
        Some(Sound {
            data: Arc::new(vec![1000, -2000, 5000, -3000, 2000, 4000, -700]),
        }),
    ]);
    main.sync_to_state(&state, &sounds);
    let snapshot = |engine: &TrackerEngine, state| {
        AudioGenSnapshot::new(
            AudioPlaybackState::default(),
            state,
            engine.snapshot(),
            sounds.clone(),
            0,
            60,
            44100,
            false,
        )
    };
    audio.handle_snapshot(snapshot(&main, state));
    audio
        .tracker_engine
        .advance_positions(&mut audio.gen_tracker, &sounds, 21000, 44100);
    let predicted = audio.gen_tracker;
    audio.handle_snapshot(snapshot(&main, state));
    assert_eq!(
        bytemuck::bytes_of(&predicted),
        bytemuck::bytes_of(&audio.gen_tracker)
    );
    for row in [0, 3, 0] {
        state.row = row;
        state.order_position = 0;
        state.tick = 0;
        state.tick_sample_pos = 0;
        state.request_position_change();
        main.sync_to_state(&state, &sounds);
        audio.handle_snapshot(snapshot(&main, state));
        assert_eq!(
            bytemuck::bytes_of(&state),
            bytemuck::bytes_of(&audio.gen_tracker)
        );
        // Crossfade is intentional output decoration; compare the underlying subsequent PCM.
        let mut audible = false;
        for _ in 0..4000 {
            let expected = main.render_sample_and_advance(&mut state, &sounds, 44100);
            let actual = audio.tracker_engine.render_sample_and_advance(
                &mut audio.gen_tracker,
                &sounds,
                44100,
            );
            audible |= expected != (0.0, 0.0);
            assert_eq!(expected, actual);
        }
        assert!(audible);
        assert_eq!(
            bytemuck::bytes_of(&state),
            bytemuck::bytes_of(&audio.gen_tracker)
        );
    }
}
