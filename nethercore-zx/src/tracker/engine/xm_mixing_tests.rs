//! Original constant-sample checks for immutable, supplied-handle mix metadata.
use super::*;
use crate::audio::Sound;
use std::sync::Arc;

#[test]
fn xm_envelopes_transition_stereo_gains_within_the_tick() {
    // Independent original source/cart/reference controls retained in
    // xm-envelope-transition-before-worker-v1. A simultaneous volume halving
    // and center-to-left pan keeps the left gain constant in linear mode.
    for (ft2, volume, pan, curve, expected) in [
        (false, true, false, false, [378.1, 378.1]),
        (false, false, true, false, [774.9, 247.1]),
        (false, true, true, false, [512.0, 247.1]),
        (true, true, false, false, [804.6, 804.6]),
        (true, false, true, false, [1317.0, 527.2]),
        (true, true, true, false, [920.4, 527.2]),
        (false, false, true, true, [589.7, 432.3]),
    ] {
        let mut song = module(vec![pattern(vec![TrackerNote {
            note: 49, instrument: 1, ..Default::default()
        }])], vec![0], FormatFlags::IS_XM_FORMAT | FormatFlags::INSTRUMENTS);
        song.format = song.format | if ft2 { FormatFlags::XM_FT2_MIX } else {
            FormatFlags::XM_LEGACY_MIX | FormatFlags::XM_LEGACY_RETRIGGER
        };
        song.global_volume = 64;
        song.mix_volume = 48;
        let instrument = &mut song.instruments[0];
        instrument.sample_loop_type = nether_tracker::LoopType::Forward;
        instrument.sample_loop_end = 8;
        instrument.sample_default_volume = Some(32);
        if volume {
            instrument.volume_envelope = Some(TrackerEnvelope {
                points: vec![(0, 64), (1, 32), (2, 32)],
                flags: EnvelopeFlags::ENABLED, ..Default::default()
            });
        }
        if pan {
            instrument.panning_envelope = Some(TrackerEnvelope {
                points: if curve { vec![(0, 0), (12, -32), (32, 32)] }
                    else { vec![(0, 0), (1, -32), (2, -32)] },
                flags: EnvelopeFlags::ENABLED, ..Default::default()
            });
        }
        let mut engine = TrackerEngine::new();
        let handle = engine.load_tracker_module(song.clone(), vec![1]);
        let mut state = make_state(handle, 6, 125);
        let sounds = vec![None, Some(Sound { data: Arc::new(vec![8192; 8]) })];
        engine.sync_to_state(&state, &sounds);
        let start = if curve { 2535 } else { 1323 };
        for _ in 0..start { engine.render_sample_and_advance(&mut state, &sounds, 44100); }
        let mut mean = [0.0; 2];
        for _ in 0..20 {
            let (l, r) = engine.render_sample_and_advance(&mut state, &sounds, 44100);
            mean[0] += l * 32768.0 / 20.0;
            mean[1] += r * 32768.0 / 20.0;
        }
        for lane in 0..2 {
            assert!((mean[lane] - expected[lane]).abs() <= (expected[lane] * 0.02).max(2.0),
                "ft2={ft2} volume={volume} pan={pan} lane={lane}: {} vs {}", mean[lane], expected[lane]);
        }
        if !curve {
            for _ in 1343..2000 { engine.render_sample_and_advance(&mut state, &sounds, 44100); }
            let settled = engine.render_sample_and_advance(&mut state, &sounds, 44100);
            let changed_rate = engine.render_sample_and_advance(&mut state, &sounds, 48000);
            assert_eq!(changed_rate, settled, "a longer tick cannot revive a completed gain transition");
            let mut changed = TrackerEngine::new();
            let h = changed.load_tracker_module(song.clone(), vec![1]);
            let mut pos = make_state(h, 6, 125);
            changed.sync_to_state(&pos, &sounds);
            for _ in 0..2920 { changed.render_sample_and_advance(&mut pos, &sounds, 96000); }
            let shorter = changed.render_sample_and_advance(&mut pos, &sounds, 44100);
            let longer = changed.render_sample_and_advance(&mut pos, &sounds, 96000);
            assert_eq!(shorter, longer, "shortening then lengthening cannot revive an exhausted ramp");
            // Instrument-only selection resets envelopes without restarting the sample.
            let saved_gain = engine.channels[0].xm_envelope_ramp.as_ref().unwrap().current;
            engine.channels[0].xm_loop_stopped = true;
            engine.channels[0].xm_stop_tail_remaining = 100;
            engine.channels[0].envelope_started = 0;
            engine.channels[0].volume_envelope_pos = 0;
            engine.channels[0].panning_envelope_pos = 0;
            engine.mix_channels(crate::tracker::raw_tracker_handle(handle), &sounds, 44100, 882);
            assert_eq!(engine.channels[0].xm_envelope_ramp.as_ref().unwrap().current,
                saved_gain, "envelope-only reset must preserve stopped residual gain");


        }
    }
}

#[test]
fn xm_pan_and_preamp_follow_supplied_module_and_survive_snapshot() {
    let mut song = module(vec![pattern(vec![TrackerNote {
        note: 49, instrument: 1, ..Default::default()
    }])], vec![0], FormatFlags::IS_XM_FORMAT | FormatFlags::INSTRUMENTS | FormatFlags::XM_FT2_MIX);
    song.global_volume = 64;
    song.mix_volume = 48;
    song.instruments[0].sample_loop_type = nether_tracker::LoopType::Forward;
    song.instruments[0].sample_loop_end = 8;
    let mut engine = TrackerEngine::new();
    let ft2 = engine.load_tracker_module(song.clone(), vec![1]);
    song.format = FormatFlags::IS_XM_FORMAT | FormatFlags::INSTRUMENTS;
    song.mix_volume = 96;
    let compatible = engine.load_tracker_module(song.clone(), vec![1]);
    song.format = song.format | FormatFlags::XM_LEGACY_MIX | FormatFlags::XM_LEGACY_RETRIGGER;
    song.mix_volume = 97; // Preserve fractional gain, not a rounded baked preamp.
    let legacy = engine.load_tracker_module(song, vec![1]);
    let sounds = vec![None, Some(Sound { data: Arc::new(vec![8192; 8]) })];
    let mut state = make_state(ft2, 6, 125);
    engine.sync_to_state(&state, &sounds);
    // Measure steady gain after the existing click-suppression ramp settles.
    engine.advance_positions(&mut state, &sounds, 1024, 44100);
    assert!(!engine.snapshot().channels[0].xm_legacy_retrigger);
    engine.is_it_format = true; // Deliberately unrelated to either supplied XM handle.
    let equal = engine.mix_channels(crate::tracker::raw_tracker_handle(ft2), &sounds, 44100, 882);
    let linear = engine.mix_channels(crate::tracker::raw_tracker_handle(compatible), &sounds, 44100, 882);
    let legacy_pcm = engine.mix_channels(crate::tracker::raw_tracker_handle(legacy), &sounds, 44100, 882);
    for value in [legacy_pcm.0,legacy_pcm.1] { assert!((value-0.25*(97.0/128.0)*0.5*(2.0/3.0)).abs()<0.000001); }
    let expected_equal = 0.25 * (48.0 / 128.0) * 0.5f32.sqrt();
    let expected_linear = 0.25 * (96.0 / 128.0) * 0.5;
    for sample in [equal.0, equal.1] { assert!((sample - expected_equal).abs() < 0.000001, "{equal:?}"); }
    for sample in [linear.0, linear.1] { assert!((sample - expected_linear).abs() < 0.000001, "{linear:?}"); }
    let mut receiver = TrackerEngine::new();
    receiver.apply_snapshot(&engine.snapshot());
    assert_eq!(receiver.mix_channels(crate::tracker::raw_tracker_handle(ft2), &sounds, 44100, 882), equal);
    assert_eq!(receiver.mix_channels(crate::tracker::raw_tracker_handle(compatible), &sounds, 44100, 882), linear);
    assert_eq!(receiver.mix_channels(crate::tracker::raw_tracker_handle(legacy), &sounds, 44100, 882), legacy_pcm);
    let mut legacy_state = make_state(legacy,6,125);
    engine.sync_to_state(&legacy_state,&sounds);
    engine.advance_positions(&mut legacy_state,&sounds,1001,44100);
    assert!(engine.snapshot().channels[0].xm_legacy_retrigger);
    let mut cold=TrackerEngine::new();
    cold.apply_snapshot(&engine.snapshot());
    for _ in 0..4096 {
        assert_eq!(engine.mix_channels(crate::tracker::raw_tracker_handle(legacy),&sounds,44100, 882),
            cold.mix_channels(crate::tracker::raw_tracker_handle(legacy),&sounds,44100, 882));
    }
    assert_eq!(format!("{:?}",engine.snapshot().channels),format!("{:?}",cold.snapshot().channels));
}

#[test]
fn it_balance_pan_uses_supplied_handle_and_survives_snapshot() {
    let mut song = module(vec![pattern(vec![TrackerNote { note:49,instrument:1,..Default::default() }])],vec![0],FormatFlags::IS_IT_FORMAT | FormatFlags::INSTRUMENTS);
    song.global_volume=128; song.mix_volume=48;
    song.samples=vec![nether_tracker::TrackerSample { length:8,loop_end:8,loop_type:nether_tracker::LoopType::Forward,c5_speed:8363,..Default::default() }];
    let mut engine=TrackerEngine::new();
    let ordinary=engine.load_tracker_module(song.clone(),vec![1]);
    song.format=song.format | FormatFlags::IT_BALANCE_MIX | FormatFlags::XM_LEGACY_MIX | FormatFlags::XM_LEGACY_RETRIGGER;
    let balance=engine.load_tracker_module(song,vec![1]);
    let sounds=vec![None,Some(Sound {data:Arc::new(vec![8192;8])})];
    let mut state=make_state(ordinary,6,125);
    engine.sync_to_state(&state,&sounds);
    engine.advance_positions(&mut state,&sounds,1024,44100);
    let mut balance_state=make_state(balance,6,125);
    engine.sync_to_state(&balance_state,&sounds);
    engine.advance_positions(&mut balance_state,&sounds,1024,44100);
    assert!(!engine.snapshot().channels[0].xm_legacy_retrigger);
    engine.is_it_format=false;
    for pan in [-1.0,0.0,1.0] {
        engine.channels[0].panning=pan;
        let a=engine.mix_channels(crate::tracker::raw_tracker_handle(ordinary),&sounds,44100, 882);
        let b=engine.mix_channels(crate::tracker::raw_tracker_handle(balance),&sounds,44100, 882);
        assert!(a.0+a.1>0.001);
        if pan==0.0 { assert_eq!(b,(a.0*2.0,a.1*2.0)); } else { assert_eq!(a,b); }
        let mut receiver=TrackerEngine::new();receiver.apply_snapshot(&engine.snapshot());
        assert_eq!(receiver.mix_channels(crate::tracker::raw_tracker_handle(balance),&sounds,44100, 882),b);
    }
}
