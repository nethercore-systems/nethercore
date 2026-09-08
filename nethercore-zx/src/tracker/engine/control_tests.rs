//! Focused regressions for tracker row flow and clock boundaries.

#[path = "rollback_tests.rs"]
mod rollback_checks;
#[path = "xm_mixing_tests.rs"]
mod xm_mixing_checks;

use super::super::TrackerEngine;
use crate::state::{TrackerState, tracker_flags};
use crate::tracker::samples_per_tick;
use nether_tracker::{
    EnvelopeFlags, FormatFlags, TrackerEffect, TrackerEnvelope, TrackerInstrument, TrackerModule,
    TrackerNote, TrackerPattern, TrackerSample,
};

fn module(patterns: Vec<TrackerPattern>, orders: Vec<u8>, format: FormatFlags) -> TrackerModule {
    TrackerModule {
        name: "control regression".into(),
        num_channels: 1,
        initial_speed: 6,
        initial_tempo: 125,
        global_volume: 128,
        mix_volume: 128,
        panning_separation: 128,
        channel_pan: [32; 64],
        channel_vol: [64; 64],
        order_table: orders,
        patterns,
        instruments: vec![TrackerInstrument::default()],
        samples: vec![],
        format,
        message: None,
        restart_position: 0,
    }
}

fn make_state(handle: u32, speed: u16, bpm: u16) -> TrackerState {
    TrackerState {
        handle,
        flags: tracker_flags::PLAYING,
        speed,
        bpm,
        volume: 256,
        ..Default::default()
    }
}

fn row(effect: TrackerEffect) -> TrackerNote {
    TrackerNote {
        effect,
        ..Default::default()
    }
}

fn pattern(rows: Vec<TrackerNote>) -> TrackerPattern {
    TrackerPattern {
        num_rows: rows.len() as u16,
        notes: rows.into_iter().map(|note| vec![note]).collect(),
    }
}

fn advance_ticks(engine: &mut TrackerEngine, state: &mut TrackerState, ticks: u32, bpm: u16) {
    let spt = samples_per_tick(bpm, 44_100);
    engine.advance_positions(state, &[], spt * ticks, 44_100);
}

#[test]
fn it_pattern_note_fade_reaches_foreground_without_key_release() {
    for instruments in [false, true] {
        let mut source = nether_it::ItModule::default();
        source.flags = nether_it::ItFlags::STEREO;
        if instruments {
            source.flags = source.flags | nether_it::ItFlags::INSTRUMENTS;
        }
        let mut p = nether_it::ItPattern::empty(1, 1);
        p.notes[0][0].note = nether_it::NOTE_FADE;
        p.notes[0][0].effect = nether_it::effects::SET_PANNING;
        p.notes[0][0].effect_param = 0;
        source.patterns.push(p);
        source.order_table = vec![0];
        let song = nether_tracker::from_it_module(&source);
        let mut engine = TrackerEngine::new();
        let handle = engine.load_tracker_module(song, vec![]);
        engine.is_it_format = true;
        engine.channels[0].note_on = true;
        engine.channels[0].panning = 1.0;
        engine.process_row_tick0_internal(handle, &[]);
        assert_eq!(engine.channels[0].note_fade, instruments);
        assert!(!engine.channels[0].key_off);
        assert!(engine.channels[0].note_on);
        assert_eq!(engine.channels[0].panning, -1.0);
    }
}

#[test]
fn duplicate_samples_do_not_confuse_deduplicated_audio_handles() {
    let mut engine = TrackerEngine::new();
    for (idx, sample) in [(1, 0), (2, 1)] {
        let ch = &mut engine.channels[idx];
        ch.note_on = true;
        ch.sample_handle = 1;
        ch.sample_index = Some(sample);
        ch.volume_fadeout = 65535;
        ch.instrument = 1;
        ch.is_background = true;
        ch.parent_channel = 0;
    }
    engine.process_duplicate_check(1, 0, 2, 0, 61, 1, Some(1), 1);
    assert!(engine.channels[1].note_on, "different source sample must survive");
    assert!(!engine.channels[2].note_on);
}

#[test]
fn duplicate_check_leaves_other_pattern_channels_voices_alone() {
    let mut engine = TrackerEngine::new();
    for (idx, parent) in [(2, 0), (3, 1)] {
        let ch = &mut engine.channels[idx];
        ch.note_on = true;
        ch.sample_handle = 1;
        ch.volume_fadeout = 65535;
        ch.current_note = 61;
        ch.instrument = 1;
        ch.is_background = true;
        ch.parent_channel = parent;
    }
    engine.process_duplicate_check(2, 0, 1, 0, 61, 1, None, 1);
    assert!(!engine.channels[2].note_on);
    assert!(engine.channels[3].note_on, "other pattern channel must survive");
}

#[test]
fn note_fade_advances_gain_without_releasing_envelope_sustain() {
    for rate in [0, 128] {
        let mut engine = TrackerEngine::new();
        let ch = &mut engine.channels[0];
        ch.note_on = true;
        ch.sample_handle = 1;
        ch.volume_fadeout = 65535;
        ch.instrument_fadeout_rate = rate;
        ch.volume_envelope_enabled = true;
        ch.volume_envelope_sustain_loop = Some((0, 0));
        ch.apply_nna_action(crate::tracker::channels::NNA_NOTE_FADE);
        engine.advance_envelopes();
        let ch = &engine.channels[0];
        assert_eq!(ch.volume_fadeout, 65535 - rate);
        assert_eq!(ch.volume_envelope_pos, 0);
        assert!(!ch.key_off);
        let snapshot = ch.copy_to_background(0);
        assert!(snapshot.note_fade);
        engine.channels[0].trigger_note(61, None);
        assert!(!engine.channels[0].note_fade);
    }
}

#[test]
fn it_empty_note_map_slot_preserves_voice_but_runs_effects() {
    let mut source = nether_it::ItModule::default();
    source.flags = nether_it::ItFlags::STEREO | nether_it::ItFlags::INSTRUMENTS;
    source.instruments.push(nether_it::ItInstrument::default());
    source.instruments[0].note_sample_table[64].1 = 0;
    let song = nether_tracker::from_it_module(&source);
    let mut engine = TrackerEngine::new();
    let handle = engine.load_tracker_module(song, vec![]);
    engine.is_it_format = true;
    engine.channels[0].instrument = 1;
    engine.channels[0].current_note = 61;
    engine.channels[0].sample_handle = 1;
    engine.channels[0].note_on = true;
    engine.channels[0].sample_pos = 12.5;
    engine.process_note_internal(0, &TrackerNote { note: 65, instrument: 1,
        effect: TrackerEffect::SetPanning(0), ..Default::default() }, handle, &[]);
    assert_eq!(engine.channels[0].current_note, 61);
    assert_eq!(engine.channels[0].sample_pos, 12.5);
    assert!(engine.channels[0].note_on);
    assert_eq!(engine.channels[0].panning, -1.0);
    engine.channels[0].target_period = 1234.0;
    engine.process_note_internal(0, &TrackerNote { note: 65, instrument: 1,
        effect: TrackerEffect::TonePortamento(16), ..Default::default() }, handle, &[]);
    assert_eq!(engine.channels[0].target_period, 1234.0);
    assert_eq!(engine.channels[0].sample_pos, 12.5);
    assert!(engine.channels[0].tone_porta_active);

}

#[test]
fn xm_delay_volume_slide_runs_before_but_not_on_instrument_trigger() {
    let mut engine = TrackerEngine::new();
    let handle = engine.load_tracker_module(module(vec![pattern(vec![row(TrackerEffect::None)])], vec![0], FormatFlags::empty()), vec![1]);
    engine.process_note_internal(0, &TrackerNote { note: 49, instrument: 1, ..Default::default() }, handle, &[]);
    engine.channels[0].volume = 0.5;
    engine.process_note_internal(0, &TrackerNote { note: 61, instrument: 1, effect: TrackerEffect::NoteDelay(3), volume_effect: TrackerEffect::VolumeSlide { up: 0, down: 1 }, ..Default::default() }, handle, &[]);
    assert_eq!(engine.channels[0].volume, 0.5);
    engine.process_tick(1, 6);
    assert_eq!(engine.channels[0].volume, 31.0 / 64.0);
    engine.process_tick(2, 6);
    engine.process_tick(3, 6);
    assert_eq!(engine.channels[0].volume, 1.0);
    engine.process_tick(4, 6);
    assert_eq!(engine.channels[0].volume, 63.0 / 64.0);
}

#[test]
fn xm_portamento_reloads_explicit_instrument_pan_and_clears_surround() {
    let mut source = module(vec![pattern(vec![row(TrackerEffect::None)])], vec![0], FormatFlags::empty());
    source.instruments[0].default_pan = Some(0);
    source.instruments.push(TrackerInstrument { default_pan: Some(64), ..Default::default() });
    let mut engine = TrackerEngine::new();
    let handle = engine.load_tracker_module(source, vec![1, 2]);
    engine.process_note_internal(0, &TrackerNote { note: 49, instrument: 1, ..Default::default() }, handle, &[]);
    engine.channels[0].surround = true;
    engine.process_note_internal(0, &TrackerNote { note: 61, instrument: 2, effect: TrackerEffect::TonePortamento(16), ..Default::default() }, handle, &[]);
    assert_eq!(engine.channels[0].sample_handle, 1, "portamento retains the playing sample");
    assert_eq!(engine.channels[0].panning, -1.0, "FT2 portamento retains the old sample pan");
    assert!(!engine.channels[0].surround);
}

#[test]
fn xm_mapped_samples_select_note_slots_and_keep_note_only_volume() {
    let mut source = module(vec![pattern(vec![row(TrackerEffect::None)])], vec![0], FormatFlags::IS_XM_FORMAT);
    source.samples = vec![
        TrackerSample { default_volume: 32, default_pan: Some(0), ..Default::default() },
        TrackerSample { default_volume: 16, default_pan: Some(255), ..Default::default() },
    ];
    source.instruments[0].note_sample_table[60] = (60, 2);
    let mut empty = TrackerInstrument::default();
    empty.note_sample_table.fill((0, 0));
    source.instruments.push(empty);
    let mut engine = TrackerEngine::new();
    let handle = engine.load_tracker_module(source, vec![7, 9]);
    engine.process_note_internal(0, &TrackerNote {note: 49, instrument: 1, ..Default::default()}, handle, &[]);
    assert_eq!(engine.channels[0].sample_handle, 7);
    assert_eq!(engine.channels[0].volume, 0.5);
    engine.process_note_internal(0, &TrackerNote {note: 61, ..Default::default()}, handle, &[]);
    assert_eq!(engine.channels[0].sample_handle, 9);
    assert_eq!(engine.channels[0].volume, 0.5);
    engine.process_note_internal(0, &TrackerNote {note: 61, instrument: 1, ..Default::default()}, handle, &[]);
    assert_eq!(engine.channels[0].volume, 0.25);
    assert_eq!(engine.channels[0].panning, 127.0 / 128.0);
    engine.process_note_internal(0, &TrackerNote {note: 49, instrument: 2, ..Default::default()}, handle, &[]);
    assert_eq!(engine.channels[0].sample_handle, 0, "unmapped note cannot alias another instrument's sample");
}

#[test]
fn xm_high_pattern_indices_are_not_it_order_markers() {
    for index in [254, 255] {
        let mut patterns = vec![pattern(vec![row(TrackerEffect::None)]); 256];
        patterns[index].notes[0][0].effect = TrackerEffect::SetPanning(0);
        let mut engine = TrackerEngine::new();
        let handle = engine.load_tracker_module(module(patterns, vec![index as u8], FormatFlags::empty()), vec![]);
        let mut state = make_state(handle, 6, 125);
        engine.advance_positions(&mut state, &[], 1, 44_100);
        assert_ne!(state.flags & tracker_flags::PLAYING, 0, "XM pattern {index} must play");
        assert_eq!(engine.channels[0].panning, -1.0);
    }
}

#[test]
fn cached_seek_does_not_apply_row_zero_twice() {
    let mut engine = TrackerEngine::new();
    let handle = engine.load_tracker_module(module(vec![pattern(vec![row(TrackerEffect::FineVolumeUp(1)), row(TrackerEffect::None)])], vec![0], FormatFlags::empty()), vec![1]);
    engine.seek_to_position(handle, 0, 1, &[]);
    let first = engine.channels[0].volume;
    engine.seek_to_position(handle, 0, 1, &[]);
    assert_eq!(engine.channels[0].volume, first, "cached and fresh seek must agree");
}

#[test]
fn cached_seek_restores_engine_control_state() {
    let mut song = module(
        vec![TrackerPattern {
            num_rows: 8,
            notes: vec![vec![TrackerNote::default(); 2]; 8],
        }],
        vec![0],
        FormatFlags::IS_IT_FORMAT | FormatFlags::OLD_EFFECTS | FormatFlags::LINK_G_MEMORY,
    );
    song.num_channels = 2;
    song.patterns[0].notes[2][0].effect = TrackerEffect::GlobalVolumeSlide { up: 2, down: 0 };
    song.patterns[0].notes[3][0].volume_effect = TrackerEffect::FinePatternDelay(4);
    song.patterns[0].notes[3][0].effect = TrackerEffect::PatternDelay(2);
    song.patterns[0].notes[3][1].effect = TrackerEffect::TempoSlideUp(3);

    let mut engine = TrackerEngine::new();
    let handle = engine.load_tracker_module(song, vec![]);
    engine.seek_to_position(handle, 0, 7, &[]);
    let later_handle = engine.load_tracker_module(
        module(
            vec![pattern(vec![row(TrackerEffect::None)])],
            vec![0],
            FormatFlags::empty(),
        ),
        vec![],
    );
    assert!(engine.row_cache.find_nearest(handle, 0, 4).is_some());

    // Poison the post-cache engine state before restoring the cached row-4 state.
    engine.channel_mutes = [true; crate::tracker::MAX_TRACKER_CHANNELS];
    engine.global_volume = 0.25;
    engine.pattern_delay = 9;
    engine.pattern_delay_count = 8;
    engine.fine_pattern_delay = 9;
    engine.last_global_vol_slide = 0xaa;
    engine.is_it_format = false;
    engine.old_effects_mode = false;
    engine.link_g_memory = true;
    engine.tempo_slide = -4;

    engine.seek_to_position(handle, 0, 4, &[]);

    assert_eq!(engine.channel_mutes, [false; crate::tracker::MAX_TRACKER_CHANNELS]);
    assert_eq!(engine.global_volume, 1.0);
    assert_eq!((engine.pattern_delay, engine.pattern_delay_count), (2, 0));
    assert_eq!(engine.fine_pattern_delay, 4);
    assert_eq!(engine.last_global_vol_slide, 0);
    assert!(engine.is_it_format);
    assert!(engine.old_effects_mode);
    assert!(!engine.link_g_memory);
    assert_eq!(engine.tempo_slide, 3);
    assert!(
        engine.modules[crate::tracker::raw_tracker_handle(later_handle) as usize].is_some()
    );
}

#[test]
fn invalid_seek_returns_without_looping_forever() {
    let (send, receive) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let mut engine = TrackerEngine::new();
        let handle = engine.load_tracker_module(module(vec![pattern(vec![row(TrackerEffect::None)])], vec![0], FormatFlags::empty()), vec![1]);
        engine.seek_to_position(handle, 1, 0, &[]);
        engine.seek_to_position(handle, 0, 1, &[]);
        send.send((engine.current_order, engine.current_row)).unwrap();
    });
    assert_eq!(receive.recv_timeout(std::time::Duration::from_secs(2)).expect("invalid seek must terminate"), (0, 0));
}

#[test]
fn xm_sample_pan_reload_and_explicit_override() {
    for mapped in [false, true] {
    for pan in 0..=255u8 {
        let mut source = module(vec![pattern(vec![row(TrackerEffect::None)])], vec![0], FormatFlags::empty());
        source.instruments[0].default_pan = Some(pan);
        if mapped {
            source.samples = vec![TrackerSample { default_pan: Some(pan), ..Default::default() }];
        }
        let mut engine = TrackerEngine::new();
        let handle = engine.load_tracker_module(source, vec![1]);
        let note = TrackerNote { note: 49, instrument: 1, ..Default::default() };
        engine.process_note_internal(0, &note, handle, &[]);
        assert_eq!(engine.channels[0].panning, pan as f32 / 128.0 - 1.0);
        engine.process_note_internal(0, &TrackerNote { effect: TrackerEffect::SetPanning(32), ..note }, handle, &[]);
        let explicit = engine.channels[0].panning;
        assert!(explicit.abs() < 0.01);
        engine.process_note_internal(0, &TrackerNote { instrument: 99, ..Default::default() }, handle, &[]);
        assert_eq!(engine.channels[0].panning, pan as f32 / 128.0 - 1.0);
        assert_eq!(engine.channels[0].instrument, 1);
        assert_eq!(engine.channels[0].last_xm_instrument, 99);
        let snapshot = engine.snapshot();
        engine.reset();
        engine.apply_snapshot(&snapshot);
        assert_eq!(engine.channels[0].last_xm_instrument, 99);
    }
    }
}

#[test]
fn xm_valid_instrument_selection_defers_voice_change_until_note() {
    for mapped in [false, true] {
        let mut source = module(vec![pattern(vec![row(TrackerEffect::None)])], vec![0], FormatFlags::empty());
        let mut first = TrackerInstrument { default_pan: Some(0), sample_default_volume: Some(32), ..Default::default() };
        let mut second = TrackerInstrument { default_pan: Some(255), sample_default_volume: Some(8), sample_relative_note: 12, ..Default::default() };
        if mapped {
            source.samples = vec![
                TrackerSample { default_volume: 32, default_pan: Some(0), ..Default::default() },
                TrackerSample { default_volume: 8, default_pan: Some(255), ..Default::default() },
            ];
            for entry in &mut first.note_sample_table { entry.1 = 1; }
            for entry in &mut second.note_sample_table {
                entry.0 = 60;
                entry.1 = 2;
            }
        }
        source.instruments = vec![first, second];

        source.instruments[0].volume_envelope = Some(TrackerEnvelope {
            points: vec![(0, 64), (32, 32)],
            flags: EnvelopeFlags::ENABLED,
            ..Default::default()
        });

        let mut engine = TrackerEngine::new();
        let handle = engine.load_tracker_module(source, vec![11, 22]);
        engine.process_note_internal(0, &TrackerNote { note: 49, instrument: 1, ..Default::default() }, handle, &[]);
        let initial_period = engine.channels[0].base_period;
        engine.channels[0].sample_pos = 12.0;
        let initial_sample = (engine.channels[0].sample_handle, engine.channels[0].sample_index);
        engine.channels[0].key_off = true;
        engine.channels[0].sample_sustain_released = true;
        engine.channels[0].note_fade = true;
        engine.channels[0].volume_envelope_pos = 7;
        engine.channels[0].panning_envelope_pos = 8;
        engine.channels[0].pitch_envelope_pos = 9;
        engine.channels[0].filter_envelope_pos = 10;
        engine.channels[0].envelope_started = 7;
        engine.channels[0].volume_fadeout = 1234;
        engine.channels[0].auto_vibrato_depth = 5;
        engine.channels[0].auto_vibrato_pos = 6;
        engine.channels[0].auto_vibrato_sweep = 9;

        engine.process_note_internal(0, &TrackerNote { instrument: 2, ..Default::default() }, handle, &[]);
        assert_eq!(engine.channels[0].instrument, 1, "selection must not switch the active instrument");
        assert_eq!(engine.channels[0].sample_handle, 11, "selection must not switch the active sample");
        assert_eq!(engine.channels[0].volume, 0.5, "selection reloads the old voice defaults");
        assert_eq!(engine.channels[0].panning, -1.0, "selection reloads the old voice pan");
        assert_eq!(engine.channels[0].sample_pos, 12.0, "selection must not retrigger the voice");
        assert_eq!((engine.channels[0].sample_handle, engine.channels[0].sample_index), initial_sample);
        assert!(!engine.channels[0].key_off);
        assert!(!engine.channels[0].sample_sustain_released);
        assert!(!engine.channels[0].note_fade);
        assert_eq!(engine.channels[0].volume_envelope_pos, 0);
        assert_eq!(engine.channels[0].panning_envelope_pos, 0);
        assert_eq!(engine.channels[0].pitch_envelope_pos, 0);
        assert_eq!(engine.channels[0].filter_envelope_pos, 0);
        assert_eq!(engine.channels[0].envelope_started, 0);
        assert_eq!(engine.channels[0].volume_fadeout, 65535);
        assert_eq!(engine.channels[0].auto_vibrato_depth, 5, "authored selection playback retains depth");
        assert_eq!(engine.channels[0].auto_vibrato_sweep, 0);
        assert_eq!(engine.channels[0].auto_vibrato_pos, 0);

        engine.process_note_internal(0, &TrackerNote { note: 49, ..Default::default() }, handle, &[]);
        assert_eq!(engine.channels[0].instrument, 2, "the remembered selection applies to the next pitched note");
        assert_eq!(engine.channels[0].sample_handle, 22, "the next pitched note must use the selected sample");
        assert!(
            engine.channels[0].base_period < initial_period,
            "the selected instrument's tuning must apply (mapped={mapped}, initial={initial_period}, actual={})",
            engine.channels[0].base_period
        );
        assert_eq!(engine.channels[0].volume, 0.5, "instrument-less note must retain prior volume");
        assert_eq!(engine.channels[0].panning, -1.0, "instrument-less note must retain prior pan");
        assert_eq!(engine.channels[0].sample_pos, 0.0, "the pitched note starts the selected sample");

        engine.process_note_internal(0, &TrackerNote { note: 49, instrument: 2, ..Default::default() }, handle, &[]);
        assert_eq!(engine.channels[0].volume, 0.125, "an explicit instrument reloads its default volume");
        assert!(engine.channels[0].panning > 0.9, "an explicit instrument reloads its default pan");
    }
}

#[test]
fn xm_k00_selection_keeps_voice_release_and_selection_memory() {
    for mapped in [false, true] {
        let mut source = module(vec![pattern(vec![row(TrackerEffect::None)])], vec![0], FormatFlags::empty());
        let mut first = TrackerInstrument { default_pan: Some(0), sample_default_volume: Some(32), ..Default::default() };
        let second = TrackerInstrument { default_pan: Some(255), sample_default_volume: Some(8), sample_relative_note: 12, ..Default::default() };
        if mapped {
            source.samples = vec![
                TrackerSample { default_volume: 32, default_pan: Some(0), ..Default::default() },
                TrackerSample { default_volume: 8, default_pan: Some(255), ..Default::default() },
            ];
            for entry in &mut first.note_sample_table { entry.1 = 1; }
            let mut mapped_second = second.clone();
            for entry in &mut mapped_second.note_sample_table {
                entry.0 = 60;
                entry.1 = 2;
            }
            source.instruments = vec![first, mapped_second];
        } else {
            source.instruments = vec![first, second];
        }
        source.instruments[0].volume_envelope = Some(TrackerEnvelope {
            points: vec![(0, 64), (32, 32)],
            flags: EnvelopeFlags::ENABLED,
            ..Default::default()
        });

        let mut engine = TrackerEngine::new();
        let handle = engine.load_tracker_module(source, vec![11, 22]);
        engine.process_note_internal(0, &TrackerNote { note: 49, instrument: 1, ..Default::default() }, handle, &[]);
        engine.channels[0].sample_pos = 12.0;
        engine.channels[0].volume_envelope_pos = 7;
        let initial_sample = (engine.channels[0].sample_handle, engine.channels[0].sample_index);

        engine.process_note_internal(0, &TrackerNote {
            instrument: 2,
            volume: 32,
            effect: TrackerEffect::KeyOff,
            ..Default::default()
        }, handle, &[]);
        let channel = &engine.channels[0];
        assert_eq!(channel.instrument, 1, "K00 selection must not switch the active instrument");
        assert_eq!((channel.sample_handle, channel.sample_index), initial_sample, "K00 selection must keep the active sample");
        assert_eq!(channel.sample_pos, 12.0, "K00 selection must not retrigger the sample");
        assert_eq!(channel.volume_envelope_pos, 7, "K00 selection must not retrigger the envelope");
        assert!(channel.key_off, "K00 must release the active envelope");
        assert!(channel.sample_sustain_released, "K00 must release sample sustain");
        assert!(!channel.note_fade, "an enabled XM envelope must not be faded by K00");
        assert_eq!(channel.volume, 0.5, "the co-located volume command must run");
        assert_eq!(channel.last_xm_instrument, 2, "K00 selection must remain remembered");

        engine.process_note_internal(0, &TrackerNote { note: 49, ..Default::default() }, handle, &[]);
        assert_eq!(engine.channels[0].instrument, 2, "the remembered K00 selection must apply on the next note");
        assert_eq!(engine.channels[0].sample_handle, 22, "the next note must use the selected sample");
    }
}

#[test]
fn xm_d0_pans_left_on_ticks_and_resets_each_row() {
    let mut engine = TrackerEngine::new();
    engine.channels[0].note_on = true;
    engine.process_unified_effect_tick0(0, &TrackerEffect::PanningLeftOnTicks, 0, 0);
    assert_eq!(engine.channels[0].panning, 0.0);
    engine.process_tick(1, 6);
    assert_eq!(engine.channels[0].panning, -1.0);
    engine.channels[0].reset_row_effects();
    engine.channels[0].panning = 0.0;
    engine.process_tick(1, 6);
    assert_eq!(engine.channels[0].panning, 0.0);
}

#[test]
fn xm_empty_delayed_cell_retriggers_previous_note() {
    let mut engine = TrackerEngine::new();
    let handle = engine.load_tracker_module(module(vec![pattern(vec![row(TrackerEffect::None)])], vec![0], FormatFlags::empty()), vec![1]);
    engine.process_note_internal(0, &TrackerNote { note: 49, instrument: 1, ..Default::default() }, handle, &[]);
    engine.channels[0].sample_pos = 12.5;
    engine.process_note_internal(0, &row(TrackerEffect::NoteDelay(3)), handle, &[]);
    assert_eq!(engine.channels[0].sample_pos, 12.5);
    engine.process_tick(3, 6);
    assert_eq!(engine.channels[0].sample_pos, 0.0);
    assert_eq!(engine.channels[0].current_note, 49);
    assert!(engine.channels[0].note_on);
}

#[test]
fn xm_delayed_note_ignores_volume_tone_portamento() {
    let mut engine = TrackerEngine::new();
    let handle = engine.load_tracker_module(module(vec![pattern(vec![row(TrackerEffect::None)])], vec![0], FormatFlags::empty()), vec![1]);
    engine.process_note_internal(0, &TrackerNote { note: 49, instrument: 1, ..Default::default() }, handle, &[]);
    engine.channels[0].sample_pos = 12.5;
    engine.process_note_internal(0, &TrackerNote { note: 61, instrument: 1, effect: TrackerEffect::NoteDelay(3), volume_effect: TrackerEffect::TonePortamento(16), ..Default::default() }, handle, &[]);
    assert_eq!(engine.channels[0].current_note, 49);
    engine.process_tick(3, 6);
    assert_eq!(engine.channels[0].current_note, 61);
    assert_eq!(engine.channels[0].sample_pos, 0.0);
    assert!(!engine.channels[0].tone_porta_active);
}

#[test]
fn xm_note_delay_defers_the_entire_note_and_survives_snapshot() {
    for delay in [1, 3, 6] {
        let mut engine = TrackerEngine::new();
        let handle = engine.load_tracker_module(module(vec![pattern(vec![row(TrackerEffect::None)])], vec![0], FormatFlags::empty()), vec![1]);
        engine.channels[0].volume = 0.25;
        let note = TrackerNote { note: 49, instrument: 1, effect: TrackerEffect::NoteDelay(delay), volume_effect: TrackerEffect::SetVolume(32), ..Default::default() };
        engine.process_note_internal(0, &note, handle, &[]);
        assert!(!engine.channels[0].note_on, "EDx must not trigger at tick zero");
        assert_eq!(engine.channels[0].volume, 0.25);
        let snapshot = engine.snapshot();
        engine.apply_snapshot(&snapshot);
        for tick in 1..=6 {
            engine.process_tick(tick, 6);
            assert_eq!(engine.channels[0].note_on, delay < 6 && tick >= u16::from(delay));
        }
        engine.channels[0].reset_row_effects();
    }
}

#[test]
fn xm_note_off_distinguishes_volume_from_other_commands() {
    for context in 0..5 {
        let mut engine = TrackerEngine::new();
        let handle = engine.load_tracker_module(module(vec![pattern(vec![row(TrackerEffect::None)])], vec![0], FormatFlags::empty()), vec![]);
        engine.channels[0].note_on = true;
        engine.channels[0].volume = 0.5;
        let note = TrackerNote {
            note: TrackerNote::NOTE_OFF,
            instrument: if context == 1 { 1 } else { 0 },
            volume_effect: match context { 2 => TrackerEffect::SetVolume(32), 3 => TrackerEffect::SetPanning(32), _ => TrackerEffect::None },
            effect: if context == 4 { TrackerEffect::SetVolume(32) } else { TrackerEffect::None },
            ..Default::default()
        };
        engine.process_note_internal(0, &note, handle, &[]);
        assert!(engine.channels[0].note_on);
        assert_eq!(engine.channels[0].volume == 0.0, context == 0 || context == 3);
        assert!(engine.channels[0].note_fade);
    }
}

#[test]
fn xm_k00_mutes_or_fades_with_row_context() {
    for envelope in [false, true] {
        for context in 0..3 {
            let mut engine = TrackerEngine::new();
            let handle = engine.load_tracker_module(module(vec![pattern(vec![row(TrackerEffect::None)])], vec![0], FormatFlags::empty()), vec![]);
            engine.channels[0].note_on = true;
            engine.channels[0].volume = 0.5;
            engine.channels[0].volume_envelope_enabled = envelope;
            let note = TrackerNote {
                effect: TrackerEffect::KeyOff,
                instrument: if context == 1 { 1 } else { 0 },
                volume_effect: if context == 2 { TrackerEffect::SetPanning(32) } else { TrackerEffect::None },
                ..Default::default()
            };
            engine.process_note_internal(0, &note, handle, &[]);
            let channel = &engine.channels[0];
            assert!(channel.key_off);
            assert!(channel.note_on);
            assert_eq!(channel.note_fade, !envelope && context != 0);
            assert_eq!(channel.volume == 0.0, !envelope && context == 0);
        }
    }
}

#[test]
fn it_sd_fine_tick_extension_uses_exclusive_extended_boundary() {
    for delay in [6,7,8] {
        let song = module(vec![pattern(vec![TrackerNote::default()])], vec![0],
            FormatFlags::IS_IT_FORMAT | FormatFlags::INSTRUMENTS);
        let mut engine = TrackerEngine::new();
        let handle = engine.load_tracker_module(song, vec![1]);
        engine.is_it_format = true;
        engine.fine_pattern_delay = 2;
        engine.process_note_internal(0, &TrackerNote { note:49, instrument:1,
            effect:TrackerEffect::NoteDelay(delay), ..Default::default() }, handle, &[]);
        for tick in 1..delay as u16 { engine.process_tick(tick,6); }
        assert!(!engine.channels[0].note_on);
        engine.process_tick(delay as u16,6);
        assert_eq!(engine.channels[0].note_on,delay < 8);
    }
}

#[test]
fn it_sd3_defers_volume_only_and_cut_without_retriggering() {
    for cut in [false, true] {
        let song = module(vec![pattern(vec![TrackerNote::default()])], vec![0],
            FormatFlags::IS_IT_FORMAT | FormatFlags::INSTRUMENTS);
        let mut engine = TrackerEngine::new();
        let handle = engine.load_tracker_module(song, vec![1]);
        engine.process_note_internal(0, &TrackerNote { note:49, instrument:1, ..Default::default() }, handle, &[]);
        engine.is_it_format = true;
        engine.channels[0].sample_pos = 17.0;
        engine.process_note_internal(0, &TrackerNote {
            note: if cut { TrackerNote::NOTE_CUT } else { 0 },
            volume_effect: if cut { TrackerEffect::None } else { TrackerEffect::SetVolume(0) },
            effect: TrackerEffect::NoteDelay(3), ..Default::default()
        }, handle, &[]);
        assert!(engine.channels[0].note_on);
        assert!(engine.channels[0].volume > 0.0);
        for tick in [1,2] { engine.process_tick(tick,6); }
        assert!(engine.channels[0].note_on);
        assert!(engine.channels[0].volume > 0.0);
        engine.process_tick(3,6);
        assert_eq!(engine.channels[0].volume,0.0);
        assert_eq!(engine.channels[0].sample_pos,17.0);
        assert_eq!(engine.channels[0].note_on,!cut);
    }
}

#[test]
fn it_sd_note_delay_defers_initialization_and_ignores_out_of_range_ticks() {
    let mut song = module(
        vec![pattern(vec![TrackerNote {
            note: 49,
            instrument: 1,
            effect: TrackerEffect::NoteDelay(0),
            ..Default::default()
        }])],
        vec![0],
        FormatFlags::IS_IT_FORMAT | FormatFlags::INSTRUMENTS,
    );
    song.instruments[0].sample_default_volume = Some(32);
    song.instruments[0].filter_cutoff = Some(64);
    song.instruments[0].filter_resonance = Some(48);
    song.instruments[0].pitch_envelope = Some(nether_tracker::TrackerEnvelope {
        points: vec![(0, -8), (1, 8)],
        flags: nether_tracker::EnvelopeFlags::ENABLED | nether_tracker::EnvelopeFlags::FILTER,
        ..Default::default()
    });
    let mut engine = TrackerEngine::new();
    let handle = engine.load_tracker_module(song, vec![1]);
    engine.channels[0].filter_cutoff = 0.25;
    let note = TrackerNote {
        note: 49,
        instrument: 1,
        effect: TrackerEffect::NoteDelay(0),
        ..Default::default()
    };
    engine.process_note_internal(0, &note, handle, &[]);
    assert!(!engine.channels[0].note_on);
    assert_eq!(engine.channels[0].note_delay_tick, 1);
    assert_eq!(engine.channels[0].filter_cutoff, 0.25);
    let snapshot = engine.snapshot();
    engine.apply_snapshot(&snapshot);

    engine.process_tick(1, 6);
    assert!(engine.channels[0].note_on);
    assert_eq!(engine.channels[0].current_note, 49);
    assert_eq!(engine.channels[0].instrument, 1);
    assert_eq!(engine.channels[0].volume, 0.5);
    assert_eq!(engine.channels[0].filter_cutoff, 64.0 / 127.0);
    assert_eq!(engine.channels[0].filter_resonance, 48.0 / 127.0);
    assert!(engine.channels[0].filter_envelope_enabled);
    assert_eq!(engine.channels[0].filter_envelope_pos, 0);
    engine.process_tick(2, 6);
    assert_eq!(engine.channels[0].filter_envelope_pos, 1);

    let mut late_song = module(
        vec![pattern(vec![TrackerNote::default()])],
        vec![0],
        FormatFlags::IS_IT_FORMAT | FormatFlags::INSTRUMENTS,
    );
    late_song.instruments[0].sample_default_volume = Some(32);
    for delay in [6, 7] {
        let mut late = TrackerEngine::new();
        let late_handle = late.load_tracker_module(late_song.clone(), vec![1]);
        let late_note = TrackerNote {
            note: 49,
            instrument: 1,
            effect: TrackerEffect::NoteDelay(delay),
            ..Default::default()
        };
        late.process_note_internal(0, &late_note, late_handle, &[]);
        for tick in 1..=delay {
            late.process_tick(tick.into(), 6);
        }
        assert!(!late.channels[0].note_on);
    }
}

#[test]
fn xm_delayed_keyoff_waits_and_row_reset_cancels_it() {
    for envelope in [false, true] {
        let mut engine = TrackerEngine::new();
        engine.channels[0].note_on = true;
        engine.channels[0].volume = 1.0;
        engine.channels[0].volume_envelope_enabled = envelope;
        engine.process_unified_effect_tick0(0, &TrackerEffect::KeyOffAt(3), 0, 0);
        for tick in 1..=3 {
            engine.process_tick(tick, 6);
            assert_eq!(engine.channels[0].key_off, tick == 3);
        }
        assert_eq!(engine.channels[0].volume, if envelope { 1.0 } else { 0.0 });
        engine.channels[0].reset_row_effects();
        assert_eq!(engine.channels[0].key_off_tick, 0);
    }
}

#[test]
fn xm_note_cut_mutes_without_stopping_and_zero_is_immediate() {
    for tick in [0, 1, 3] {
        let mut engine = TrackerEngine::new();
        engine.channels[0].note_on = true;
        engine.channels[0].volume = 1.0;
        engine.process_unified_effect_tick0(0, &TrackerEffect::NoteCut(tick), 0, 0);
        if tick > 0 {
            assert_eq!(engine.channels[0].volume, 1.0);
            engine.process_tick(tick as u16, 6);
        }
        assert_eq!(engine.channels[0].volume, 0.0);
        assert!(engine.channels[0].note_on);
        engine.process_unified_effect_tick0(0, &TrackerEffect::SetVolume(32), 0, 0);
        assert_eq!(engine.channels[0].volume, 0.5);
        assert!(engine.channels[0].note_on);
    }
}

#[test]
fn it_sc0_cuts_at_tick_one_not_zero_or_never() {
    for value in 0..=15 {
        let mut source = nether_it::ItModule::default();
        let mut p = nether_it::ItPattern::empty(1, 1);
        p.notes[0][0].effect = nether_it::effects::EXTENDED;
        p.notes[0][0].effect_param = 0xc0 | value;
        source.patterns.push(p);
        source.order_table = vec![0];
        source.num_channels = 1;
        let song = nether_tracker::from_it_module(&source);
        let effect = song.patterns[0].notes[0][0].effect;
        assert_eq!(effect, TrackerEffect::ItExtended(0xc0 | value));
        assert_eq!(TrackerEffect::from_it_extended(0xc0 | value), TrackerEffect::NoteCut(value.max(1)));
        let mut engine = TrackerEngine::new();
        engine.is_it_format = true;
        engine.channels[0].note_on = true;
        let handle = engine.load_tracker_module(song, vec![]);
        engine.process_row_tick0_internal(handle, &[]);
        assert!(engine.channels[0].note_on);
        for tick in 1..=value.max(1) {
            engine.process_tick(tick as u16, 20);
            assert_eq!(engine.channels[0].note_on, tick < value.max(1));
        }
    }
}

#[test]
fn it_s70_s72_only_affect_owned_background_voices() {
    for action in 0..=2 {
        let mut source = nether_it::ItModule::default();
        let mut p = nether_it::ItPattern::empty(1, 1);
        p.notes[0][0].effect = nether_it::effects::EXTENDED;
        p.notes[0][0].effect_param = 0x70 + action;
        source.patterns.push(p);
        source.order_table = vec![0];
        source.num_channels = 1;
        let song = nether_tracker::from_it_module(&source);
        let mut engine = TrackerEngine::new();
        for (i, parent) in [(0, 0), (1, 0), (2, 1)] {
            engine.channels[i].note_on = true;
            engine.channels[i].is_background = i != 0;
            engine.channels[i].parent_channel = parent;
        }
        let handle = engine.load_tracker_module(song, vec![]);
        engine.process_row_tick0_internal(handle, &[]);
        assert_eq!(engine.channels[1].note_on, action != 0);
        assert_eq!(engine.channels[1].key_off, action == 1);
        assert_eq!(engine.channels[1].note_fade, action == 2);
        for i in [0, 2] {
            assert!(engine.channels[i].note_on);
            assert!(!engine.channels[i].key_off && !engine.channels[i].note_fade);
        }
    }
}

#[test]
fn it_s73_s76_override_only_the_foreground_nna() {
    for value in 3..=6 {
        let mut source = nether_it::ItModule::default();
        let mut pattern = nether_it::ItPattern::empty(1, 1);
        pattern.notes[0][0] = nether_it::ItNote {
            effect: nether_it::effects::EXTENDED,
            effect_param: 0x70 | value,
            ..Default::default()
        };
        source.patterns.push(pattern);
        source.order_table = vec![0];
        source.num_channels = 1;
        let converted = nether_tracker::from_it_module(&source);
        let effect = converted.patterns[0].notes[0][0].effect;
        assert_eq!(effect, TrackerEffect::ItExtended(0x70 | value));
        assert_eq!(TrackerEffect::from_it_extended(0x70 | value), TrackerEffect::SetNewNoteAction(value - 3));
        let mut engine = TrackerEngine::new();
        engine.channels[1].nna = 2;
        engine.channels[1].is_background = true;
        let handle = engine.load_tracker_module(converted, vec![]);
        engine.process_row_tick0_internal(handle, &[]);
        assert_eq!(engine.channels[0].nna, value - 3);
        assert_eq!(engine.channels[1].nna, 2);
    }
}

#[test]
fn it_nna_uses_displaced_voice_action_not_incoming_instrument() {
    use nether_tracker::NewNoteAction;
    for (old, incoming, backgrounds) in [
        (NewNoteAction::Continue, NewNoteAction::Cut, 1),
        (NewNoteAction::Cut, NewNoteAction::Continue, 0),
    ] {
        let mut song = module(
            vec![pattern(vec![
                TrackerNote {
                    note: 61,
                    instrument: 1,
                    ..Default::default()
                },
                TrackerNote {
                    note: 65,
                    instrument: 2,
                    ..Default::default()
                },
            ])],
            vec![0],
            FormatFlags::IS_IT_FORMAT | FormatFlags::INSTRUMENTS,
        );
        song.instruments[0].nna = old;
        let mut second = song.instruments[0].clone();
        second.nna = incoming;
        song.instruments.push(second);
        song.samples.push(TrackerSample {
            loop_type: nether_tracker::LoopType::Forward,
            loop_end: 64,
            ..Default::default()
        });
        for instrument in &mut song.instruments {
            for (i, entry) in instrument.note_sample_table.iter_mut().enumerate() {
                *entry = (i as u8, 1);
            }
        }
        let mut engine = TrackerEngine::new();
        let handle = engine.load_tracker_module(song, vec![1]);
        let sounds = vec![
            None,
            Some(crate::audio::Sound {
                data: vec![1000; 64].into(),
            }),
        ];
        let mut state = make_state(handle, 6, 125);
        engine.sync_to_state(&state, &sounds);
        engine.render_sample_and_advance(&mut state, &sounds, 44100);
        engine.current_row = 1;
        engine.process_row_tick0_internal(handle, &sounds);
        assert_eq!(
            engine
                .channels
                .iter()
                .filter(|c| c.is_background && c.note_on)
                .count(),
            backgrounds
        );
        assert_eq!(engine.channels[0].instrument, 2);
        if backgrounds != 0 {
            assert_eq!(engine.channels[1].instrument, 1);
            assert_eq!(engine.channels[1].nna, 1);
            assert_eq!(engine.channels[1].current_note, 61);
        }
        assert_eq!(
            engine.channels[0].nna,
            if incoming == NewNoteAction::Cut { 0 } else { 1 }
        );
    }
}

#[test]
fn it_note_pan_defaults_override_surround_and_preserve_effect_order() {
    for instruments in [false, true] {
        for (sample_pan, instrument_pan, expected) in [
            (Some(64), Some(0), 1.0),
            (Some(0), None, -1.0),
            (None, Some(64), if instruments { 1.0 } else { 0.25 }),
            (None, None, 0.25),
        ] {
            let mut song = module(
                vec![pattern(vec![
                    TrackerNote {
                        note: 61,
                        instrument: 1,
                        ..Default::default()
                    },
                    TrackerNote {
                        note: 62,
                        ..Default::default()
                    },
                    TrackerNote {
                        instrument: 1,
                        ..Default::default()
                    },
                    TrackerNote {
                        note: 63,
                        effect: TrackerEffect::SetPanning(32),
                        ..Default::default()
                    },
                    TrackerNote {
                        note: 64,
                        effect: TrackerEffect::TonePortamento(4),
                        ..Default::default()
                    },
                ])],
                vec![0],
                FormatFlags::IS_IT_FORMAT
                    | if instruments {
                        FormatFlags::INSTRUMENTS
                    } else {
                        FormatFlags::empty()
                    },
            );
            song.samples = vec![TrackerSample {
                default_pan: sample_pan,
                ..Default::default()
            }];
            song.instruments[0].default_pan = instrument_pan;
            for (n, entry) in song.instruments[0].note_sample_table.iter_mut().enumerate() {
                *entry = (n as u8, 1);
            }
            if !instruments {
                song.instruments.clear();
            }
            let mut engine = TrackerEngine::new();
            let handle = engine.load_tracker_module(song, vec![1]);
            for row_index in 0..5 {
                engine.current_row = row_index;
                engine.channels[0].panning = 0.25;
                engine.channels[0].surround = true;
                engine.process_row_tick0_internal(handle, &[]);
                let applied = row_index != 2
                    && (sample_pan.is_some() || (instruments && instrument_pan.is_some()));
                let pan = if row_index == 3 {
                    0.0
                } else if row_index == 2 {
                    0.25
                } else {
                    expected
                };
                assert_eq!(
                    engine.channels[0].panning, pan,
                    "instruments={instruments}, row={row_index}, sample={sample_pan:?}, instrument={instrument_pan:?}"
                );
                assert_eq!(engine.channels[0].surround, !applied && row_index != 3);
            }
        }
    }
}

#[test]
fn panning_envelope_control_pauses_only_the_foreground_envelope() {
    let sounds = vec![
        None,
        Some(crate::audio::Sound {
            data: vec![1000i16; 64].into(),
        }),
    ];
    let song = module(
        vec![pattern(vec![
            row(TrackerEffect::SetPanningEnvelope(false)),
            row(TrackerEffect::SetPanningEnvelope(true)),
            TrackerNote::default(),
        ])],
        vec![0],
        FormatFlags::IS_IT_FORMAT,
    );
    let mut engine = TrackerEngine::new();
    let handle = engine.load_tracker_module(song, vec![1]);
    let mut state = make_state(handle, 2, 125);
    engine.sync_to_state(&state, &sounds);
    for index in [0, 32] {
        let ch = &mut engine.channels[index];
        ch.note_on = true;
        ch.is_background = index != 0;
        ch.sample_handle = 1;
        ch.sample_loop_type = 1;
        ch.sample_loop_end = 64;
        ch.volume_envelope_enabled = true;
        ch.panning_envelope_enabled = true;
        ch.panning_envelope_pos = 7;
        ch.volume_envelope_pos = 9;
        ch.volume = 0.5;
        ch.period = 4608.0;
        ch.base_period = 4608.0;
    }
    engine.advance_positions(&mut state, &sounds, 4, 100);
    assert!(!engine.channels[0].panning_envelope_enabled);
    assert_eq!(engine.channels[0].panning_envelope_pos, 7);
    assert_eq!(engine.channels[0].volume_envelope_pos, 11);
    assert_eq!(engine.channels[32].panning_envelope_pos, 9);
    let mut restored = TrackerEngine::new();
    restored.apply_snapshot(&engine.snapshot());
    let mut restored_state = state;
    engine.advance_positions(&mut state, &sounds, 1, 100);
    assert!(engine.channels[0].panning_envelope_enabled);
    assert_eq!(engine.channels[0].panning_envelope_pos, 7);
    engine.advance_positions(&mut state, &sounds, 1, 100);
    assert_eq!(engine.channels[0].panning_envelope_pos, 8);
    for _ in 0..2 {
        restored.render_sample_and_advance(&mut restored_state, &sounds, 100);
    }
    assert_eq!(
        format!("{:?}", engine.channels),
        format!("{:?}", restored.channels)
    );
}

#[test]
fn long_held_envelopes_saturate_instead_of_wrapping() {
    let mut engine = TrackerEngine::new();
    let ch = &mut engine.channels[0];
    ch.note_on = true;
    ch.volume_envelope_enabled = true;
    ch.panning_envelope_enabled = true;
    ch.pitch_envelope_enabled = true;
    ch.filter_envelope_enabled = true;
    ch.volume_envelope_pos = u16::MAX - 1;
    ch.panning_envelope_pos = u16::MAX - 1;
    ch.pitch_envelope_pos = u16::MAX - 1;
    ch.filter_envelope_pos = u16::MAX - 1;
    for _ in 0..3 {
        engine.advance_envelopes();
        let ch = &engine.channels[0];
        assert_eq!(
            [
                ch.volume_envelope_pos,
                ch.panning_envelope_pos,
                ch.pitch_envelope_pos,
                ch.filter_envelope_pos
            ],
            [u16::MAX; 4]
        );
        assert!(ch.note_on);
    }
}

#[test]
fn envelope_clock_counts_row_boundaries_for_foreground_and_nna() {
    let sounds = vec![
        None,
        Some(crate::audio::Sound {
            data: vec![1000i16; 64].into(),
        }),
    ];
    for (format, speed) in [
        (FormatFlags::IS_IT_FORMAT, 1),
        (FormatFlags::IS_IT_FORMAT, 2),
        (FormatFlags::IS_IT_FORMAT, 6),
        (FormatFlags::IS_XM_FORMAT, 1),
        (FormatFlags::IS_XM_FORMAT, 2),
        (FormatFlags::IS_XM_FORMAT, 6),
    ] {
        let song = module(
            vec![pattern(vec![TrackerNote::default(); 32])],
            vec![0],
            format,
        );
        let mut engine = TrackerEngine::new();
        let handle = engine.load_tracker_module(song, vec![1]);
        let mut state = make_state(handle, speed, 125);
        engine.sync_to_state(&state, &sounds);
        for index in [0, 32] {
            let ch = &mut engine.channels[index];
            ch.note_on = true;
            ch.is_background = index == 32;
            ch.sample_handle = 1;
            ch.period = 4608.0;
            ch.base_period = 4608.0;
            ch.sample_loop_type = 1;
            ch.sample_loop_end = 64;
            ch.volume_envelope_enabled = true;
            ch.panning_envelope_enabled = true;
            ch.pitch_envelope_enabled = true;
            ch.filter_envelope_enabled = true;
            ch.volume_fadeout = 65535;
            ch.instrument_fadeout_rate = 1;
            ch.key_off = true;
            ch.note_fade = true; // Explicit fade: IT key-off alone can wait for its envelope.
        }
        let mut audible = TrackerEngine::new();
        audible.apply_snapshot(&engine.snapshot());
        let mut audible_state = state;
        for tick in 1..=20 {
            engine.advance_positions(&mut state, &sounds, 2, 100);
            for _ in 0..2 {
                audible.render_sample_and_advance(&mut audible_state, &sounds, 100);
            }
            for index in [0, 32] {
                let ch = &engine.channels[index];
                assert_eq!(
                    [
                        ch.volume_envelope_pos,
                        ch.panning_envelope_pos,
                        ch.pitch_envelope_pos,
                        ch.filter_envelope_pos
                    ],
                    [tick; 4],
                    "speed={speed}, voice={index}"
                );
                assert_eq!(ch.volume_fadeout, 65535 - tick);
                assert_eq!(format!("{ch:?}"), format!("{:?}", audible.channels[index]));
            }
        }
    }
}

#[test]
fn paused_authored_it_envelopes_keep_tick_zero_values_after_playback_only() {
    let mut song = module(vec![pattern(vec![TrackerNote {note:49,instrument:1,..Default::default()}])],vec![0],FormatFlags::IS_IT_FORMAT|FormatFlags::INSTRUMENTS);
    song.instruments[0].volume_envelope=Some(nether_tracker::TrackerEnvelope {points:vec![(0,32)],flags:nether_tracker::EnvelopeFlags::ENABLED|nether_tracker::EnvelopeFlags::SUSTAIN_LOOP,..Default::default()});
    let mut engine=TrackerEngine::new();
    let handle=engine.load_tracker_module(song,vec![1]);let raw=crate::tracker::raw_tracker_handle(handle);
    engine.sync_to_state(&make_state(handle,6,125),&[]);engine.process_row_tick0_internal(raw,&[]);
    let sounds=vec![None,Some(crate::audio::Sound{data:vec![8192;64].into()})];
    engine.channels[0].fade_in_samples=0;
    engine.process_unified_effect_tick0(0,&TrackerEffect::SetVolumeEnvelope(false),0,0);
    let initial=engine.mix_channels(raw,&sounds,44100, 882).0;
    engine.process_unified_effect_tick0(0,&TrackerEffect::SetVolumeEnvelope(true),0,0);
    engine.advance_envelopes();
    assert_eq!(engine.channels[0].volume_envelope_pos,0);
    assert_eq!(engine.channels[0].envelope_started&1,1);
    engine.process_unified_effect_tick0(0,&TrackerEffect::SetVolumeEnvelope(false),0,0);
    let held=engine.mix_channels(raw,&sounds,44100, 882).0;
    assert!((held/initial-0.5).abs()<0.001);
    let mut restored=TrackerEngine::new();restored.apply_snapshot(&engine.snapshot());
    assert_eq!(restored.channels[0].envelope_started,engine.channels[0].envelope_started);
    assert_eq!(restored.mix_channels(raw,&sounds,44100, 882),engine.mix_channels(raw,&sounds,44100, 882));
    engine.process_row_tick0_internal(raw,&[]);
    assert_eq!(engine.channels[0].envelope_started,0);
}

#[test]
fn xm_lxx_pan_position_depends_on_volume_sustain_not_pan_sustain() {
    for volume_sustain in [false,true] {
        for pan_sustain in [false,true] {
            let mut engine = TrackerEngine::new();
            let ch = &mut engine.channels[0];
            ch.volume_envelope_sustain_loop = volume_sustain.then_some((2,2));
            ch.panning_envelope_sustain_loop = pan_sustain.then_some((2,2));
            ch.panning_envelope_pos = 3;
            ch.pitch_envelope_pos = 4;
            ch.panning_envelope_frozen = true;
            engine.process_unified_effect_tick0(0,&TrackerEffect::SetEnvelopePosition(24),0,0);
            assert_eq!(engine.channels[0].volume_envelope_pos,24);
            assert_eq!(engine.channels[0].panning_envelope_pos,if volume_sustain {24} else {3});
            assert_eq!(engine.channels[0].pitch_envelope_pos,if volume_sustain {24} else {4});
            assert!(engine.channels[0].panning_envelope_frozen);
        }
    }
}

#[test]
fn delayed_note_clears_previous_pan_sustain_freeze() {
    let mut engine = TrackerEngine::new();
    let channel = &mut engine.channels[0];
    channel.note_on = true;
    channel.panning_envelope_enabled = true;
    channel.panning_envelope_pos = 7;
    channel.panning_envelope_frozen = true;
    channel.note_delay_tick = 2;
    channel.delayed_note = 49;
    engine.process_tick(2,6);
    // process_tick also advances envelopes at the completed tick boundary.
    assert_eq!(engine.channels[0].panning_envelope_pos,1);
    assert!(!engine.channels[0].panning_envelope_frozen);
    engine.advance_envelopes();
    assert_eq!(engine.channels[0].panning_envelope_pos,2);
}

#[test]
fn xm_pan_sustain_freezes_only_if_reached_before_release_and_resets_on_note() {
    for it in [false, true] {
        for early_release in [false, true] {
            let format = if it {FormatFlags::IS_IT_FORMAT | FormatFlags::INSTRUMENTS} else {FormatFlags::empty()};
            let mut song = module(vec![pattern(vec![TrackerNote {note:49,instrument:1,..Default::default()}])],vec![0],format);
            song.instruments[0].panning_envelope = Some(nether_tracker::TrackerEnvelope {
                points:vec![(0,0),(2,32),(8,-32)], sustain_begin:1,sustain_end:1,
                flags:nether_tracker::EnvelopeFlags::ENABLED | nether_tracker::EnvelopeFlags::SUSTAIN_LOOP,..Default::default()
            });
            let mut engine = TrackerEngine::new();
            let handle = engine.load_tracker_module(song,vec![1]);
            let raw = crate::tracker::raw_tracker_handle(handle);
            engine.sync_to_state(&make_state(handle,6,125),&[]);
            engine.process_row_tick0_internal(raw,&[]);
            for _ in 0..if early_release {1} else {4} { engine.advance_envelopes(); }
            assert_eq!(engine.channels[0].panning_envelope_frozen,!it && !early_release);
            engine.channels[0].key_off = true;
            let mut restored = TrackerEngine::new();restored.apply_snapshot(&engine.snapshot());
            for _ in 0..3 {engine.advance_envelopes();restored.advance_envelopes();}
            assert_eq!(engine.channels[0].panning_envelope_pos, if early_release {4} else if it {5} else {2});
            assert_eq!(format!("{:?}",engine.channels),format!("{:?}",restored.channels));
            engine.process_row_tick0_internal(raw,&[]);
            assert!(!engine.channels[0].panning_envelope_frozen);
            assert_eq!(engine.channels[0].panning_envelope_pos,0);
        }
    }
}

#[test]
fn xm_loop_escape_requires_same_authored_node_and_survives_snapshot() {
    for sustain_node in [0, 1, 2] {
        let mut song = module(vec![pattern(vec![TrackerNote {note:49,instrument:1,..Default::default()}])], vec![0], FormatFlags::empty());
        let envelope = nether_tracker::TrackerEnvelope {
            points: vec![(0,64),(4,32),(4,32),(10,0)],
            sustain_begin:sustain_node, sustain_end:sustain_node, loop_begin:0, loop_end:1,
            flags:nether_tracker::EnvelopeFlags::ENABLED | nether_tracker::EnvelopeFlags::SUSTAIN_LOOP | nether_tracker::EnvelopeFlags::LOOP,
        };
        song.instruments[0].volume_envelope = Some(envelope.clone());
        song.instruments[0].panning_envelope = Some(envelope);
        let mut engine = TrackerEngine::new();
        let handle = engine.load_tracker_module(song, vec![1]);
        engine.sync_to_state(&make_state(handle,6,125), &[]);
        engine.process_row_tick0_internal(crate::tracker::raw_tracker_handle(handle), &[]);
        assert_eq!(engine.channels[0].volume_envelope_escape_loop, sustain_node == 1);
        assert_eq!(engine.channels[0].panning_envelope_escape_loop, sustain_node == 1);
        engine.channels[0].key_off = true;
        let mut restored = TrackerEngine::new();
        restored.apply_snapshot(&engine.snapshot());
        for _ in 0..5 { engine.advance_envelopes(); restored.advance_envelopes(); }
        let expected = if sustain_node == 1 {5} else {1};
        assert_eq!(engine.channels[0].volume_envelope_pos,expected);
        assert_eq!(engine.channels[0].panning_envelope_pos,expected);
        assert_eq!(format!("{:?}",engine.channels),format!("{:?}",restored.channels));
    }
}

#[test]
fn it_envelope_sustain_range_precedes_normal_loop_and_releases() {
    let mut song = module(vec![pattern(vec![TrackerNote { note:61, instrument:1, ..Default::default() }])], vec![0], FormatFlags::IS_IT_FORMAT | FormatFlags::INSTRUMENTS);
    let envelope = nether_tracker::TrackerEnvelope {
        points: vec![(0, 32), (1, 32), (3, 32), (5, 32)],
        flags: nether_tracker::EnvelopeFlags::ENABLED | nether_tracker::EnvelopeFlags::SUSTAIN_LOOP | nether_tracker::EnvelopeFlags::LOOP,
        sustain_begin: 1, sustain_end: 2, loop_begin: 0, loop_end: 3,
    };
    song.instruments[0].volume_envelope = Some(envelope.clone());
    song.instruments[0].panning_envelope = Some(envelope.clone());
    song.instruments[0].pitch_envelope = Some(envelope);
    let mut engine = TrackerEngine::new();
    let handle = engine.load_tracker_module(song, vec![1]);
    let raw = crate::tracker::raw_tracker_handle(handle);
    engine.sync_to_state(&make_state(handle, 6, 125), &[]);
    engine.process_row_tick0_internal(raw, &[]);
    assert!(engine.channels[0].note_on);
    engine.channels[32] = engine.channels[0].clone();
    engine.channels[32].is_background = true;
    for expected in [1, 2, 3, 1, 2, 3, 1] {
        engine.advance_envelopes();
        assert_eq!(engine.channels[0].volume_envelope_pos, expected);
        assert_eq!(engine.channels[0].panning_envelope_pos, expected);
        assert_eq!(engine.channels[0].pitch_envelope_pos, expected);
    }
    engine.channels[0].key_off = true;
    engine.channels[32].key_off = true;
    let mut restored = TrackerEngine::new();
    restored.apply_snapshot(&engine.snapshot());
    for expected in [2, 3, 4, 5, 0, 1] {
        restored.advance_envelopes();
        engine.advance_envelopes();
        assert_eq!(engine.channels[0].volume_envelope_pos, expected);
        assert_eq!(engine.channels[0].panning_envelope_pos, expected);
        assert_eq!(engine.channels[0].pitch_envelope_pos, expected);
        assert_eq!(engine.channels[32].pitch_envelope_pos, expected);
        assert_eq!(format!("{:?}", engine.channels), format!("{:?}", restored.channels));
    }
}

#[test]
fn pitch_envelope_control_preserves_position_and_background_voices() {
    let mut engine = TrackerEngine::new();
    for voice in [0, 32] {
        let ch = &mut engine.channels[voice];
        ch.reset();
        ch.note_on = true;
        ch.pitch_envelope_enabled = true;
        ch.filter_envelope_enabled = true;
        ch.pitch_envelope_pos = 7;
        ch.filter_envelope_pos = 7;
    }
    engine.process_unified_effect_tick0(0, &TrackerEffect::SetPitchEnvelope(false), 0, 0);
    engine.advance_envelopes();
    assert_eq!(engine.channels[0].pitch_envelope_pos, 7);
    assert_eq!(engine.channels[0].filter_envelope_pos, 7);
    assert_eq!(engine.channels[32].pitch_envelope_pos, 8);
    engine.process_unified_effect_tick0(0, &TrackerEffect::SetPitchEnvelope(true), 0, 0);
    assert_eq!(engine.channels[0].pitch_envelope_pos, 7);
    engine.advance_envelopes();
    assert_eq!(engine.channels[0].pitch_envelope_pos, 8);
    assert_eq!(engine.channels[0].filter_envelope_pos, 8);
}

#[test]
fn it_filter_envelope_preserves_base_cutoff_pause_and_snapshot_state() {
    for voice in [0,1] {
        let mut song = module(vec![pattern(vec![TrackerNote {note:49,instrument:1,..Default::default()}])],vec![0],FormatFlags::IS_IT_FORMAT|FormatFlags::INSTRUMENTS);
        song.instruments[0].filter_cutoff=Some(64);
        song.instruments[0].pitch_envelope=Some(nether_tracker::TrackerEnvelope {
            points:vec![(0,-16),(1,32)],
            flags:nether_tracker::EnvelopeFlags::ENABLED|nether_tracker::EnvelopeFlags::FILTER|nether_tracker::EnvelopeFlags::SUSTAIN_LOOP,
            ..Default::default()
        });
        song.instruments.push(TrackerInstrument::default());
        let mut engine=TrackerEngine::new();
        let handle=engine.load_tracker_module(song,vec![1,1]);let raw=crate::tracker::raw_tracker_handle(handle);
        engine.sync_to_state(&make_state(handle,6,125),&[]);engine.process_row_tick0_internal(raw,&[]);
        assert!(engine.channels[0].filter_envelope_enabled);
        assert_eq!(engine.channels[0].filter_envelope_sustain_loop,Some((0,0)));
        assert!(!engine.channels[0].pitch_envelope_enabled);
        if voice==1 {engine.channels[1]=engine.channels[0].clone();engine.channels[1].is_background=true;engine.channels[0].note_on=false;}
        let sounds=vec![None,Some(crate::audio::Sound{data:vec![8192;512].into()})];
        let base=64.0/127.0;
        engine.process_unified_effect_tick0(voice,&TrackerEffect::SetPitchEnvelope(false),0,0);
        engine.mix_channels(raw,&sounds,44100, 882);
        assert_eq!(engine.channels[voice].filter_envelope_value,Some(0));
        engine.process_unified_effect_tick0(voice,&TrackerEffect::SetPitchEnvelope(true),0,0);
        engine.advance_envelopes();
        assert_eq!(engine.channels[voice].filter_envelope_pos,0);
        assert_eq!(engine.channels[voice].envelope_started&8,8);
        engine.process_unified_effect_tick0(voice,&TrackerEffect::SetPitchEnvelope(false),0,0);
        engine.mix_channels(raw,&sounds,44100, 882);
        assert_eq!(engine.channels[voice].filter_cutoff,base);
        assert_eq!(engine.channels[voice].filter_envelope_value,Some(-16));
        let mut restored=TrackerEngine::new();restored.apply_snapshot(&engine.snapshot());
        for _ in 0..8 {engine.mix_channels(raw,&sounds,44100, 882);restored.process_channels::<false>(raw,&sounds,44100, 882);}
        assert_eq!(engine.channels[voice].filter_z1,restored.channels[voice].filter_z1);
        assert_eq!(engine.channels[voice].filter_envelope_value,restored.channels[voice].filter_envelope_value);
        engine.process_unified_effect_tick0(voice,&TrackerEffect::SetFilterCutoff(32),0,0);
        engine.mix_channels(raw,&sounds,44100, 882);
        assert_eq!(engine.channels[voice].filter_cutoff,32.0/127.0);
        assert_eq!(engine.channels[voice].filter_envelope_value,Some(-16));
        engine.process_unified_effect_tick0(voice,&TrackerEffect::SetPitchEnvelope(true),0,0);
        engine.channels[voice].key_off=true;engine.advance_envelopes();
        engine.mix_channels(raw,&sounds,44100, 882);
        assert_eq!(engine.channels[voice].filter_envelope_value,Some(32));
        engine.channels[voice].instrument=2;
        engine.mix_channels(raw,&sounds,44100, 882);
        assert_eq!(engine.channels[voice].filter_envelope_value,None);
    }
}

#[test]
fn it_live_filter_defaults_preserve_absent_values_and_row_effect_wins() {
    for cutoff in [None, Some(0), Some(64), Some(127)] {
        for effect in [TrackerEffect::None, TrackerEffect::SetFilterCutoff(32)] {
            let mut song = module(vec![pattern(vec![TrackerNote { note:61, instrument:1, effect, ..Default::default() }])], vec![0], FormatFlags::IS_IT_FORMAT | FormatFlags::INSTRUMENTS);
            song.instruments[0].filter_cutoff = cutoff;
            song.instruments[0].filter_resonance = Some(48);
            let mut engine = TrackerEngine::new();
            let handle = engine.load_tracker_module(song, vec![1]);
            engine.sync_to_state(&make_state(handle,6,125), &[]);
            engine.channels[0].filter_cutoff = 0.25;
            engine.channels[0].filter_resonance = 0.75;
            engine.process_row_tick0_internal(handle, &[]);
            assert!(engine.channels[0].note_on);
            let expected = if effect == TrackerEffect::SetFilterCutoff(32) {32.0/127.0} else {cutoff.map(|v|v as f32/127.0).unwrap_or(0.25)};
            assert_eq!(engine.channels[0].filter_cutoff, expected);
            assert_eq!(engine.channels[0].filter_resonance, 48.0/127.0);
            assert!(engine.channels[0].filter_dirty);
        }
    }
}

#[test]
fn filter_coefficients_follow_render_rate_and_silent_snapshot() {
    let song = module(vec![pattern(vec![row(TrackerEffect::None)])], vec![0], FormatFlags::IS_IT_FORMAT);
    let mut engine = TrackerEngine::new();
    let handle = engine.load_tracker_module(song, vec![1]);
    let raw = crate::tracker::raw_tracker_handle(handle);
    let sounds = vec![None, Some(crate::audio::Sound { data: vec![2000i16; 64].into() })];
    for voice in [0, 1] {
        let ch = &mut engine.channels[voice];
        ch.reset();
        ch.note_on = true;
        ch.sample_handle = 1;
        ch.period = 4608.0;
        ch.filter_cutoff = 0.5;
        ch.filter_resonance = 0.2;
        ch.filter_dirty = true;
    }
    for rate in [44100, 22050, 48000] {
        let mut silent = TrackerEngine::new();
        silent.apply_snapshot(&engine.snapshot());
        engine.mix_channels(raw, &sounds, rate, 882);
        silent.process_channels::<false>(raw, &sounds, rate, 882);
        for voice in [0, 1] {
            let got = &engine.channels[voice];
            let mut expected = got.clone();
            expected.update_filter_coefficients(rate as f32);
            assert_eq!(got.filter_b0, expected.filter_b0, "voice={voice}, rate={rate}");
            assert_eq!(got.filter_a1, expected.filter_a1);
            assert_eq!(got.filter_z1, silent.channels[voice].filter_z1);
            assert_eq!(got.filter_z2, silent.channels[voice].filter_z2);
            assert_eq!(got.sample_pos, silent.channels[voice].sample_pos);
        }
    }
}

#[test]
fn pitch_envelope_changes_rate_without_mutating_period_and_matches_silent_advance() {
    for voice in [0, 1] {
        for (value, enabled, ratio) in [(24, true, 2.0), (-24, true, 0.5), (24, false, 1.0)] {
            let mut song = module(vec![pattern(vec![row(TrackerEffect::None)])], vec![0], FormatFlags::IS_IT_FORMAT);
            song.instruments[0].pitch_envelope = Some(nether_tracker::TrackerEnvelope {
                points: vec![(0, value)],
                flags: nether_tracker::EnvelopeFlags::ENABLED,
                ..Default::default()
            });
            let mut engine = TrackerEngine::new();
            let handle = engine.load_tracker_module(song, vec![1]);
            let ch = &mut engine.channels[voice];
            ch.reset();
            ch.note_on = true;
            ch.is_background = voice != 0;
            ch.sample_handle = 1;
            ch.instrument = 1;
            ch.period = 4608.0;
            ch.pitch_envelope_enabled = enabled;
            let sounds = vec![None, Some(crate::audio::Sound { data: vec![2000i16; 64].into() })];
            let mut silent = TrackerEngine::new();
            silent.apply_snapshot(&engine.snapshot());
            let raw = crate::tracker::raw_tracker_handle(handle);
            engine.mix_channels(raw, &sounds, 44100, 882);
            silent.process_channels::<false>(raw, &sounds, 44100, 882);
            let got = engine.channels[voice].sample_pos;
            let expected = crate::tracker::utils::period_to_frequency(4608.0) as f64 / 44100.0 * ratio;
            assert!((got - expected).abs() < 0.000001, "voice={voice} value={value}: {got} != {expected}");
            assert_eq!(got, silent.channels[voice].sample_pos);
            assert_eq!(engine.channels[voice].period, 4608.0);
            engine.channels[voice].pitch_envelope_enabled = false;
            engine.mix_channels(raw, &sounds, 44100, 882);
            let unmodulated = crate::tracker::utils::period_to_frequency(4608.0) as f64 / 44100.0;
            assert!((engine.channels[voice].sample_pos - got - unmodulated).abs() < 0.000001);
        }
    }
}

#[test]
fn panning_envelope_modulates_base_pan_in_foreground_and_nna_voices() {
    use nether_tracker::TrackerEnvelope;
    let sounds = vec![
        None,
        Some(crate::audio::Sound {
            data: vec![8192i16; 64].into(),
        }),
    ];
    // OpenMPT Sndmix.cpp ProcessPanningEnvelope: extremes stay fixed; zero is neutral.
    for channel_index in [0, 1] {
        for (base, envelope, expected) in [
            (-1.0, 32, -1.0),
            (1.0, -32, 1.0),
            (-0.5, 0, -0.5),
            (0.5, 0, 0.5),
            (-0.5, 16, -0.25),
            (0.5, -16, 0.25),
            (0.0, -32, -1.0),
            (0.0, 32, 1.0),
        ] {
            let mut song = module(
                vec![pattern(vec![row(TrackerEffect::None)])],
                vec![0],
                FormatFlags::IS_IT_FORMAT,
            );
            song.instruments[0].panning_envelope = Some(TrackerEnvelope {
                points: vec![(0, envelope)],
                flags: nether_tracker::EnvelopeFlags::empty(), // S7A can enable authored-off envelopes.
                ..Default::default()
            });
            let mut engine = TrackerEngine::new();
            let handle = engine.load_tracker_module(song, vec![1]);
            let channel = &mut engine.channels[channel_index];
            channel.reset();
            channel.note_on = true;
            channel.is_background = channel_index != 0;
            channel.sample_handle = 1;
            channel.instrument = 1;
            channel.volume = 1.0;
            channel.panning = base;
            channel.panning_envelope_enabled = true;
            channel.base_period = 4608.0;
            channel.period = 4608.0;
            let raw = crate::tracker::raw_tracker_handle(handle);
            let mut reference = TrackerEngine::new();
            reference.apply_snapshot(&engine.snapshot());
            reference.channels[channel_index].panning = expected;
            reference.channels[channel_index].panning_envelope_enabled = false;
            let want = reference.mix_channels(raw, &sounds, 44100, 882);
            let got = engine.mix_channels(raw, &sounds, 44100, 882);
            assert!(
                (got.0 - want.0).abs() < 0.00001 && (got.1 - want.1).abs() < 0.00001,
                "base={base} env={envelope} voice={channel_index}: {got:?} != {want:?}"
            );
            assert_eq!(engine.channels[channel_index].panning, base);
        }
    }
}

#[test]
fn it_delayed_opposite_pitch_recall_keeps_shared_ef_memory() {
    let sounds = vec![
        None,
        Some(crate::audio::Sound {
            data: vec![1000i16; 64].into(),
        }),
    ];
    for compatible_g in [false, true] {
        for (first, recalled, memory, delta) in [
            (
                TrackerEffect::PortamentoUp(0xf2),
                TrackerEffect::PortamentoDown(0),
                0xf2,
                8.0,
            ),
            (
                TrackerEffect::PortamentoDown(0xe3),
                TrackerEffect::PortamentoUp(0),
                0xe3,
                -3.0,
            ),
        ] {
            let mut flags = FormatFlags::IS_IT_FORMAT;
            if compatible_g {
                flags = flags | FormatFlags::LINK_G_MEMORY;
            }
            let mut song = module(
                vec![TrackerPattern {
                    num_rows: 3,
                    notes: vec![
                        vec![
                            TrackerNote {
                                note: 61,
                                instrument: 1,
                                effect: first,
                                ..Default::default()
                            },
                            row(TrackerEffect::None),
                        ],
                        vec![row(recalled), row(TrackerEffect::PatternDelay(2))],
                        vec![TrackerNote::default(); 2],
                    ],
                }],
                vec![0],
                flags,
            );
            song.num_channels = 2;
            song.samples = vec![TrackerSample {
                default_volume: 32,
                loop_type: nether_tracker::LoopType::Forward,
                loop_end: 64,
                ..Default::default()
            }];
            let mut engine = TrackerEngine::new();
            let handle = engine.load_tracker_module(song, vec![1]);
            let mut state = make_state(handle, 6, 125);
            let span = samples_per_tick(125, 100) * 6;
            engine.advance_positions(&mut state, &sounds, span, 100);
            let start = engine.channels[0].base_period;
            for repeat in 1..=3 {
                engine.advance_positions(
                    &mut state,
                    &sounds,
                    if repeat == 1 { 1 } else { span },
                    100,
                );
                let channel = &engine.channels[0];
                assert_eq!(channel.last_porta_up, memory);
                assert_eq!(channel.last_porta_down, memory);
                assert_eq!(channel.base_period, start + delta * repeat as f32);
                assert_eq!(state.row, 1);
            }
        }
    }
}

#[test]
fn it_delayed_rows_repeat_main_column_fine_pitch_without_retriggering() {
    let sounds = vec![
        None,
        Some(crate::audio::Sound {
            data: vec![1000i16; 64].into(),
        }),
    ];
    for (effect, delta) in [
        (TrackerEffect::PortamentoUp(0xf2), -8.0),
        (TrackerEffect::PortamentoDown(0xe3), 3.0),
    ] {
        let note = TrackerNote {
            note: 61,
            instrument: 1,
            volume_effect: TrackerEffect::FineVolumeDown(1),
            effect,
            ..TrackerNote::default()
        };
        let mut song = module(
            vec![TrackerPattern {
                num_rows: 2,
                notes: vec![
                    vec![note, row(TrackerEffect::PatternDelay(2))],
                    vec![TrackerNote::default(); 2],
                ],
            }],
            vec![0],
            FormatFlags::IS_IT_FORMAT,
        );
        song.num_channels = 2;
        song.samples = vec![TrackerSample {
            default_volume: 32,
            loop_type: nether_tracker::LoopType::Forward,
            loop_end: 64,
            ..TrackerSample::default()
        }];
        let mut engine = TrackerEngine::new();
        let handle = engine.load_tracker_module(song, vec![1]);
        let mut state = make_state(handle, 6, 125);
        engine.advance_positions(&mut state, &sounds, 1, 100);
        let initial_pitch = engine.channels[0].base_period;
        let initial_volume = engine.channels[0].volume;
        let span = samples_per_tick(125, 100) * 6;
        engine.advance_positions(&mut state, &sounds, span * 2, 100);
        assert_eq!(engine.channels[0].base_period, initial_pitch + delta * 2.0);
        assert_eq!(
            engine.channels[0].volume, initial_volume,
            "volume column must not repeat fine slide"
        );
        assert_eq!(engine.channels[0].current_note, 61);
    }
}

#[test]
fn pattern_delay_precedence_and_tick_sum_follow_format() {
    // Pinned OpenMPT suite README: IT first SEx (including SE0), XM last EEx.
    for (format, delays, repeats) in [
        (FormatFlags::IS_IT_FORMAT, vec![0, 3], 0),
        (FormatFlags::IS_IT_FORMAT, vec![2, 3], 2),
        (FormatFlags::IS_XM_FORMAT, vec![3, 0], 0),
        (FormatFlags::IS_XM_FORMAT, vec![2, 3], 3),
    ] {
        let mut song = module(
            vec![TrackerPattern {
                num_rows: 2,
                notes: vec![
                    delays
                        .into_iter()
                        .map(|n| row(TrackerEffect::PatternDelay(n)))
                        .collect(),
                    vec![TrackerNote::default(); 2],
                ],
            }],
            vec![0],
            format,
        );
        song.num_channels = 2;
        let mut engine = TrackerEngine::new();
        let handle = engine.load_tracker_module(song, vec![]);
        let mut state = make_state(handle, 6, 125);
        let samples = samples_per_tick(125, 100) * 6 * (repeats + 1);
        engine.advance_positions(&mut state, &[], samples - 1, 100);
        assert_eq!(state.row, 0);
        engine.advance_positions(&mut state, &[], 1, 100);
        assert_eq!(state.row, 1, "wrong row delay precedence: {format:?}");
    }
    // S6x adds across all 64 channels and survives every repeated row.
    let mut notes = vec![row(TrackerEffect::FinePatternDelay(15)); 64];
    notes[0].volume_effect = TrackerEffect::PatternDelay(2);
    let mut song = module(
        vec![TrackerPattern {
            num_rows: 2,
            notes: vec![notes, vec![TrackerNote::default(); 64]],
        }],
        vec![0],
        FormatFlags::IS_IT_FORMAT,
    );
    song.num_channels = 64;
    let mut engine = TrackerEngine::new();
    let handle = engine.load_tracker_module(song, vec![]);
    let mut state = make_state(handle, 6, 125);
    let samples = samples_per_tick(125, 100) * (6 + 64 * 15) * 3;
    engine.advance_positions(&mut state, &[], samples - 1, 100);
    assert_eq!(
        state.row, 0,
        "summed S6x was truncated or discarded on a repeat"
    );
    engine.advance_positions(&mut state, &[], 1, 100);
    assert_eq!(state.row, 1);
    assert_eq!(engine.fine_pattern_delay, 0);
}

#[test]
fn restart_clears_delay_and_cached_playback_state() {
    let sounds = vec![
        None,
        Some(crate::audio::Sound {
            data: vec![2000i16; 64].into(),
        }),
    ];
    let mut song = module(
        vec![pattern(vec![
            TrackerNote {
                note: 61,
                instrument: 1,
                ..Default::default()
            };
            4
        ])],
        vec![0, 255],
        FormatFlags::IS_IT_FORMAT,
    );
    song.samples = vec![TrackerSample {
        loop_begin: 0,
        loop_end: 64,
        loop_type: nether_tracker::LoopType::Forward,
        ..Default::default()
    }];
    let mut engine = TrackerEngine::new();
    let handle = engine.load_tracker_module(song, vec![1]);
    let mut state = make_state(handle, 6, 125);
    engine.sync_to_state(&state, &sounds);
    engine.advance_positions(&mut state, &sounds, 100, 44100);
    engine.sync_to_state(&state, &sounds);
    assert!(!engine.rollback_cache.is_empty());
    engine.pattern_delay = 2;
    engine.pattern_delay_count = 1;
    engine.fine_pattern_delay = 3;
    engine.last_global_vol_slide = 0x10;
    engine.tempo_slide = 2;
    engine.old_effects_mode = true;
    engine.link_g_memory = true;
    let modules = engine.modules.clone();
    let next_handle = engine.next_handle;
    engine.reset();
    assert!(engine.rollback_cache.is_empty());
    assert_eq!(engine.sync_handle, 0);
    assert_eq!(
        (
            engine.pattern_delay,
            engine.pattern_delay_count,
            engine.fine_pattern_delay
        ),
        (0, 0, 0)
    );
    assert_eq!((engine.last_global_vol_slide, engine.tempo_slide), (0, 0));
    assert!(!engine.is_it_format && !engine.old_effects_mode && !engine.link_g_memory);
    assert_eq!(engine.next_handle, next_handle);
    assert!(std::sync::Arc::ptr_eq(
        engine.modules[1].as_ref().unwrap(),
        modules[1].as_ref().unwrap()
    ));
    state = make_state(handle, 6, 125);
    engine.sync_to_state(&state, &sounds);
    let mut peak = 0.0f32;
    for _ in 0..128 {
        let (left, right) = engine.render_sample_and_advance(&mut state, &sounds, 44100);
        peak = peak.max(left.abs()).max(right.abs());
    }
    assert!(
        peak > 0.01,
        "restart must trigger the first note, not skip it as a delayed row"
    );
}

#[test]
#[ignore = "release-mode state-parity and timing probe"]
fn silent_mixer_state_parity_and_cost() {
    use std::{hint::black_box, time::Instant};
    let sounds = vec![
        None,
        Some(crate::audio::Sound {
            data: (0..256)
                .map(|i| (i * 97 - 12000) as i16)
                .collect::<Vec<_>>()
                .into(),
        }),
    ];
    let setup = || {
        let mut engine = TrackerEngine::new();
        let mut song = module(
            vec![pattern(vec![row(TrackerEffect::None)])],
            vec![0],
            FormatFlags::IS_IT_FORMAT,
        );
        song.num_channels = 16;
        let handle = engine.load_tracker_module(song, vec![1]);
        for (i, ch) in engine.channels[..16].iter_mut().enumerate() {
            ch.note_on = true;
            ch.sample_handle = 1;
            ch.sample_direction = 1;
            ch.period = crate::tracker::utils::note_to_period(49 + i as u8, 0);
            ch.sample_loop_start = 16;
            ch.sample_loop_end = 240;
            ch.sample_loop_type = 1;
            ch.volume = 0.75;
            ch.panning = i as f32 / 8.0 - 1.0;
            ch.filter_cutoff = if i % 2 == 0 { 0.4 } else { 1.0 };
            ch.filter_dirty = true;
            ch.volume_fadeout = 65535;
        }
        (engine, super::super::raw_tracker_handle(handle))
    };
    let mut ratios = Vec::new();
    for _ in 0..5 {
        let (mut audible, handle) = setup();
        let (mut silent, _) = setup();
        let start = Instant::now();
        for _ in 0..44100 {
            black_box(audible.mix_channels(handle, &sounds, 44100, 882));
        }
        let full = start.elapsed();
        let start = Instant::now();
        for _ in 0..44100 {
            black_box(silent.process_channels::<false>(handle, &sounds, 44100, 882));
        }
        let quiet = start.elapsed();
        assert_eq!(
            format!("{:?}", audible.channels),
            format!("{:?}", silent.channels)
        );
        for _ in 0..128 {
            assert_eq!(
                audible.mix_channels(handle, &sounds, 44100, 882),
                silent.mix_channels(handle, &sounds, 44100, 882)
            );
        }
        println!(
            "full_us={} silent_us={}",
            full.as_micros(),
            quiet.as_micros()
        );
        ratios.push(full.as_secs_f64() / quiet.as_secs_f64());
    }
    ratios.sort_by(f64::total_cmp);
    println!("median_full_over_silent={:.3}", ratios[ratios.len() / 2]);
}

#[test]
fn it_header_channel_defaults_survive_conversion_and_playback_reset() {
    let mut source = nether_it::ItModule::default();
    source.channel_pan[..4].copy_from_slice(&[0, 64, 100, 160]);
    source.channel_vol[..4].copy_from_slice(&[17, 32, 64, 0]);
    source.global_volume = 96;
    source.order_table = vec![0, 255];
    source.patterns = vec![nether_it::ItPattern::empty(2, 4)];
    // Exercise the packed header, not merely an authored runtime struct.
    let packed = nether_it::parse_ncit(&nether_it::pack_ncit(&source)).unwrap();
    let mut engine = TrackerEngine::new();
    let handle = engine.load_tracker_module(nether_tracker::from_it_module(&packed), vec![]);
    engine.sync_to_state(&make_state(handle, 6, 125), &[]);
    assert_eq!(engine.channels[0].panning, -1.0);
    assert_eq!(engine.channels[1].panning, 1.0);
    assert!(engine.channels[2].surround);
    assert!(engine.channel_mutes[3]);
    assert_eq!(engine.channels[0].channel_volume, 17);
    assert_eq!(engine.global_volume, 0.75);
    let snapshot = engine.snapshot();
    engine.channels[0].panning = 0.5;
    engine.apply_snapshot(&snapshot);
    assert_eq!(engine.channels[0].panning, -1.0);
}

#[test]
fn it_volume_envelope_control_freezes_position_and_prevents_retirement() {
    let mut engine = TrackerEngine::new();
    engine.is_it_format = true;

    let channel = &mut engine.channels[0];
    channel.note_on = true;
    channel.volume_envelope_enabled = true;
    channel.volume_envelope_pos = 2;
    channel.volume_envelope_zero_end = Some(2);
    engine.process_unified_effect_tick0(0, &TrackerEffect::SetVolumeEnvelope(false), 0, 0);
    engine.process_tick(1, 6);
    assert!(engine.channels[0].note_on);
    assert_eq!(engine.channels[0].volume_envelope_pos, 2);
    engine.process_unified_effect_tick0(0, &TrackerEffect::SetVolumeEnvelope(true), 0, 0);
    engine.process_tick(2, 6);
    assert!(!engine.channels[0].note_on);
}

#[test]
fn it_terminal_zero_envelope_stops_voice_and_allows_restart() {
    for (looping, sustaining) in [(false, false), (true, false), (false, true)] {
        let mut song = module(
            vec![pattern(vec![
                TrackerNote {
                    note: 61,
                    instrument: 1,
                    ..Default::default()
                },
                TrackerNote {
                    instrument: 1,
                    ..Default::default()
                },
            ])],
            vec![0],
            FormatFlags::IS_IT_FORMAT | FormatFlags::INSTRUMENTS,
        );
        song.samples = vec![TrackerSample::default()];
        song.instruments[0].volume_envelope = Some(nether_tracker::TrackerEnvelope {
            points: vec![(0, 64), (2, 0)],
            flags: nether_tracker::EnvelopeFlags::ENABLED
                | if looping {
                    nether_tracker::EnvelopeFlags::LOOP
                } else if sustaining {
                    nether_tracker::EnvelopeFlags::SUSTAIN_LOOP
                } else {
                    nether_tracker::EnvelopeFlags::empty()
                },
            loop_begin: 0,
            loop_end: 1,
            sustain_begin: 0,
            sustain_end: 0,
        });
        let mut engine = TrackerEngine::new();
        let handle = engine.load_tracker_module(song, vec![7]);
        engine.process_row_tick0_internal(handle, &[]);
        let saved = engine.snapshot();
        engine.channels[0].volume_envelope_zero_end = None;
        engine.apply_snapshot(&saved);
        assert_eq!(engine.channels[0].volume_envelope_zero_end, Some(2));
        assert_eq!(
            engine.channels[0]
                .copy_to_background(0)
                .volume_envelope_zero_end,
            Some(2)
        );
        for tick in 1..=3 {
            engine.process_tick(tick, 6);
        }
        assert_eq!(engine.channels[0].note_on, looping || sustaining);
        if sustaining {
            engine.channels[0].key_off = true;
            for tick in 1..=3 {
                engine.process_tick(tick, 6);
            }
            assert!(!engine.channels[0].note_on);
        }
        if !looping {
            engine.current_row = 1;
            engine.process_row_tick0_internal(handle, &[]);
            assert!(engine.channels[0].note_on);
            assert_eq!(engine.channels[0].volume_envelope_pos, 0);
        }
    }
}

#[test]
fn xm_tone_portamento_preserves_wide_speed_memory_and_snapshot() {
    let mut engine = TrackerEngine::new();
    engine.channels[0].note_on = true;
    engine.channels[0].period = 8000.0;
    engine.channels[0].target_period = 1000.0;
    engine.process_unified_effect_tick0(0, &TrackerEffect::TonePortamento(480), 0, 0);
    let saved = engine.snapshot();
    engine.process_tick(1, 6);
    assert_eq!(engine.channels[0].period, 6080.0);
    engine.apply_snapshot(&saved);
    engine.process_unified_effect_tick0(0, &TrackerEffect::TonePortamento(0), 0, 0);
    assert_eq!(engine.channels[0].porta_speed, 480);
    engine.process_tick(1, 6);
    assert_eq!(engine.channels[0].period, 6080.0);
}

#[test]
fn surround_on_centers_pan_but_surround_off_preserves_it() {
    let mut engine = TrackerEngine::new();
    for pan in [-1.0, 0.25, 1.0] {
        engine.channels[0].panning = pan;
        engine.process_unified_effect_tick0(0, &TrackerEffect::SetSurround(true), 0, 0);
        assert!(engine.channels[0].surround);
        assert_eq!(engine.channels[0].panning, 0.0);
        engine.channels[0].panning = pan;
        engine.process_unified_effect_tick0(0, &TrackerEffect::SetSurround(false), 0, 0);
        assert!(!engine.channels[0].surround);
        assert_eq!(engine.channels[0].panning, pan);
    }
}

#[test]
fn explicit_panning_cancels_surround() {
    let mut engine = TrackerEngine::new();
    engine.is_it_format = true;
    for pan in [0, 32, 64] {
        engine.channels[0].surround = true;
        engine.process_unified_effect_tick0(0, &TrackerEffect::SetPanning(pan), 0, 0);
        assert!(!engine.channels[0].surround);
        assert_eq!(engine.channels[0].panning, pan as f32 / 32.0 - 1.0);
    }
}

#[test]
fn converted_xm_global_volume_preserves_authored_gain() {
    let mut source = nether_xm::XmModule {
        name: "global volume regression".into(),
        num_channels: 1,
        num_patterns: 1,
        num_instruments: 0,
        song_length: 1,
        restart_position: 0,
        default_speed: 6,
        default_bpm: 125,
        linear_frequency_table: true,
        mix_mode: nether_xm::XmMixMode::Ft2,
        legacy_retrigger: false,
        sample_preamp: 48,
        order_table: vec![0],
        instruments: vec![],
        patterns: vec![nether_xm::XmPattern::empty(1, 1)],
    };
    for parameter in 0..=255u8 {
        let mut engine = TrackerEngine::new();
        source.patterns[0].notes[0][0].effect = nether_xm::effects::SET_GLOBAL_VOLUME;
        source.patterns[0].notes[0][0].effect_param = parameter;
        let converted = nether_tracker::from_xm_module(&source);
        let effect = converted.patterns[0].notes[0][0].effect;
        engine.process_unified_effect_tick0(0, &effect, 0, 0);
        assert_eq!(
            engine.global_volume,
            parameter.min(64) as f32 / 64.0,
            "XM G{parameter:02X}"
        );
    }
}

#[test]
fn normalized_global_volume_commands_ignore_invalid_it_values() {
    for is_it in [false, true] {
        for value in [0, 1, 32, 64, 128, 129, 255] {
            let mut engine = TrackerEngine::new();
            engine.is_it_format = is_it;
            engine.global_volume = 0.375;
            engine.process_unified_effect_tick0(0, &TrackerEffect::SetGlobalVolume(value), 0, 0);
            let expected = if is_it && value > 128 {
                0.375
            } else {
                (value as f32 / 128.0).min(1.0)
            };
            assert_eq!(engine.global_volume, expected, "IT={is_it} value={value}");
        }
    }
}

#[test]
fn it_both_pitch_slide_columns_execute_and_reset_on_blank_rows() {
    for up in [false, true] {
        let effect = |v| {
            if up {
                TrackerEffect::PortamentoUp(v)
            } else {
                TrackerEffect::PortamentoDown(v)
            }
        };
        let mut song = module(
            vec![pattern(vec![
                TrackerNote {
                    note: 61,
                    instrument: 1,
                    volume_effect: effect(4),
                    effect: effect(8),
                    ..Default::default()
                },
                TrackerNote::default(),
            ])],
            vec![0],
            FormatFlags::IS_IT_FORMAT | FormatFlags::LINEAR_SLIDES,
        );
        song.samples = vec![TrackerSample::default()];
        let mut engine = TrackerEngine::new();
        let handle = engine.load_tracker_module(song, vec![7]);
        engine.process_row_tick0_internal(handle, &[]);
        let period = engine.channels[0].period;
        engine.process_tick(1, 6);
        let expected = period + if up { -64.0 } else { 64.0 };
        assert_eq!(engine.channels[0].period, expected);
        engine.current_row = 1;
        engine.process_row_tick0_internal(handle, &[]);
        engine.process_tick(1, 6);
        assert_eq!(engine.channels[0].period, expected);
    }
}

#[test]
fn it_pitch_slide_memory_preserves_fine_modes_and_tick_zero_only() {
    for raw in [0xE0, 0xE1, 0xEF, 0xF0, 0xF1, 0xFF] {
        let mut song = module(
            vec![pattern(vec![
                TrackerNote {
                    note: 61,
                    instrument: 1,
                    effect: TrackerEffect::PortamentoUp(raw),
                    ..Default::default()
                },
                TrackerNote {
                    effect: TrackerEffect::PortamentoDown(0),
                    ..Default::default()
                },
            ])],
            vec![0],
            FormatFlags::IS_IT_FORMAT,
        );
        song.samples = vec![TrackerSample::default()];
        let mut engine = TrackerEngine::new();
        let handle = engine.load_tracker_module(song, vec![7]);
        engine.process_row_tick0_internal(handle, &[]);
        let initial = crate::tracker::utils::note_to_period(49, 0);
        let delta = (raw & 15) as f32 * if raw >= 0xF0 { 4.0 } else { 1.0 };
        assert_eq!(engine.channels[0].period, initial - delta);
        engine.process_tick(1, 6);
        assert_eq!(engine.channels[0].period, initial - delta);
        engine.current_row = 1;
        engine.process_row_tick0_internal(handle, &[]);
        assert_eq!(engine.channels[0].period, initial);
        engine.process_tick(1, 6);
        assert_eq!(engine.channels[0].period, initial);
    }
}

#[test]
fn it_regular_ef_memory_is_shared_and_g_writes_it_only_when_linked() {
    for compatible in [false, true] {
        let mut flags = FormatFlags::IS_IT_FORMAT;
        if compatible {
            flags = flags | FormatFlags::LINK_G_MEMORY;
        }
        let song = module(
            vec![pattern(vec![
                TrackerNote {
                    effect: TrackerEffect::PortamentoUp(17),
                    ..Default::default()
                },
                TrackerNote {
                    effect: TrackerEffect::PortamentoDown(0),
                    ..Default::default()
                },
                TrackerNote {
                    effect: TrackerEffect::TonePortamento(9),
                    ..Default::default()
                },
                TrackerNote {
                    effect: TrackerEffect::PortamentoUp(0),
                    ..Default::default()
                },
            ])],
            vec![0],
            flags,
        );
        let mut engine = TrackerEngine::new();
        let handle = engine.load_tracker_module(song, vec![7]);
        engine.process_row_tick0_internal(handle, &[]);
        engine.current_row = 1;
        engine.process_row_tick0_internal(handle, &[]);
        assert_eq!(engine.channels[0].last_porta_down, 17);
        engine.current_row = 2;
        engine.process_row_tick0_internal(handle, &[]);
        engine.current_row = 3;
        engine.process_row_tick0_internal(handle, &[]);
        assert_eq!(
            engine.channels[0].last_porta_up,
            if compatible { 17 } else { 9 }
        );
    }
}

#[test]
fn it_compatible_gxx_disables_shared_portamento_memory() {
    for compatible in [false, true] {
        let mut flags = FormatFlags::IS_IT_FORMAT;
        if compatible {
            flags = flags | FormatFlags::LINK_G_MEMORY;
        }
        let song = module(
            vec![pattern(vec![
                TrackerNote {
                    effect: TrackerEffect::PortamentoUp(17),
                    ..Default::default()
                },
                TrackerNote {
                    effect: TrackerEffect::TonePortamento(0),
                    ..Default::default()
                },
            ])],
            vec![0],
            flags,
        );
        let mut engine = TrackerEngine::new();
        let handle = engine.load_tracker_module(song, vec![7]);
        engine.process_row_tick0_internal(handle, &[]);
        engine.current_row = 1;
        engine.process_row_tick0_internal(handle, &[]);
        assert_eq!(
            engine.channels[0].porta_speed,
            if compatible { 0 } else { 17 }
        );
    }
}

#[test]
fn it_sample_portamento_switch_respects_compatible_gxx() {
    for compatible in [false, true] {
        let mut flags = FormatFlags::IS_IT_FORMAT;
        if compatible {
            flags = flags | FormatFlags::LINK_G_MEMORY;
        }
        let mut song = module(
            vec![pattern(vec![
                TrackerNote {
                    note: 61,
                    instrument: 1,
                    ..Default::default()
                },
                TrackerNote {
                    note: 68,
                    instrument: 2,
                    effect: TrackerEffect::TonePortamento(32),
                    ..Default::default()
                },
            ])],
            vec![0],
            flags,
        );
        song.instruments.clear();
        song.samples = vec![TrackerSample::default(), TrackerSample::default()];
        let mut engine = TrackerEngine::new();
        let handle = engine.load_tracker_module(song, vec![7, 8]);
        engine.process_row_tick0_internal(handle, &[]);
        let period = engine.channels[0].period;
        engine.channels[0].sample_pos = 12.0;
        engine.current_row = 1;
        engine.process_row_tick0_internal(handle, &[]);
        assert_eq!(
            engine.channels[0].sample_handle,
            if compatible { 7 } else { 8 }
        );
        assert_eq!(
            engine.channels[0].sample_index,
            Some(if compatible { 0 } else { 1 })
        );
        // OpenMPT Snd_fx.cpp:3130-3137: actual pitched sample swaps reset position.
        assert_eq!(engine.channels[0].sample_pos, if compatible { 12.0 } else { 0.0 });
        assert_eq!(engine.channels[0].period, period);
        assert!(engine.channels[0].target_period < period);
    }
}

#[test]
fn it_tone_portamento_uses_instrument_note_map_without_retrigger() {
    let mut song = module(
        vec![pattern(vec![
            TrackerNote {
                note: 49,
                instrument: 1,
                ..Default::default()
            },
            TrackerNote {
                note: 61,
                effect: TrackerEffect::TonePortamento(4),
                ..Default::default()
            },
        ])],
        vec![0],
        FormatFlags::IS_IT_FORMAT | FormatFlags::INSTRUMENTS,
    );
    song.samples = vec![TrackerSample::default()];
    song.instruments[0].note_sample_table[48] = (60, 1);
    song.instruments[0].note_sample_table[60] = (84, 1);
    let mut engine = TrackerEngine::new();
    let handle = engine.load_tracker_module(song, vec![7]);
    engine.process_row_tick0_internal(handle, &[]);
    let period = engine.channels[0].period;
    engine.channels[0].sample_pos = 123.0;
    engine.current_row = 1;
    engine.process_row_tick0_internal(handle, &[]);
    assert_eq!(
        engine.channels[0].target_period,
        crate::tracker::utils::note_to_period(73, 0)
    );
    assert_eq!(engine.channels[0].period, period);
    assert_eq!(engine.channels[0].sample_pos, 123.0);
    assert_eq!(engine.channels[0].current_note, 61);
}

#[test]
fn row_cache_is_isolated_by_tracker_handle() {
    let mut engine = TrackerEngine::new();
    let mut handles = Vec::new();
    for (volume, pan) in [(64, 0), (128, 64)] {
        let mut song = module(
            vec![pattern(vec![TrackerNote::default(); 5])],
            vec![0],
            FormatFlags::IS_IT_FORMAT,
        );
        song.global_volume = volume;
        song.channel_pan[0] = pan;
        handles.push(engine.load_tracker_module(song, vec![]));
    }
    for index in [0, 1, 0, 1] {
        engine.seek_to_position(handles[index], 0, 1, &[]);
        assert_eq!(engine.global_volume, if index == 0 { 0.5 } else { 1.0 });
        assert_eq!(
            engine.channels[0].panning,
            if index == 0 { -1.0 } else { 1.0 }
        );
    }
}

#[test]
fn silent_advance_matches_rendered_sustain_and_release_state() {
    let sounds = vec![
        None,
        Some(crate::audio::Sound {
            data: vec![1000i16; 64].into(),
        }),
    ];
    let setup = || {
        let mut song = module(
            vec![pattern(vec![
                TrackerNote {
                    note: 61,
                    instrument: 1,
                    ..Default::default()
                },
                TrackerNote {
                    note: TrackerNote::NOTE_OFF,
                    ..Default::default()
                },
                TrackerNote::default(),
            ])],
            vec![0, 255],
            FormatFlags::IS_IT_FORMAT,
        );
        song.samples = vec![TrackerSample {
            sustain_loop_begin: 2,
            sustain_loop_end: 6,
            sustain_loop_type: nether_tracker::LoopType::Forward,
            ..Default::default()
        }];
        let mut engine = TrackerEngine::new();
        let handle = engine.load_tracker_module(song, vec![1]);
        let state = make_state(handle, 1, 125);
        (engine, state)
    };
    let (mut rendered, mut rendered_state) = setup();
    let (mut silent, mut silent_state) = setup();
    for _ in 0..48 {
        for _ in 0..64 {
            rendered.render_sample_and_advance(&mut rendered_state, &sounds, 22050);
        }
        silent.advance_positions(&mut silent_state, &sounds, 64, 22050);
        assert_eq!(
            silent.channels[0].sample_pos,
            rendered.channels[0].sample_pos
        );
        assert_eq!(silent.channels[0].note_on, rendered.channels[0].note_on);
        assert_eq!(
            silent.channels[0].fade_out_samples,
            rendered.channels[0].fade_out_samples
        );
        assert_eq!(
            bytemuck::bytes_of(&silent_state),
            bytemuck::bytes_of(&rendered_state)
        );
    }
}

#[test]
fn sustain_loop_bounds_apply_until_release() {
    use crate::tracker::channels::TrackerChannel;
    use crate::tracker::utils::{note_to_period, sample_channel};
    for regular in [false, true] {
        let mut channel = TrackerChannel::default();
        channel.note_on = true;
        channel.period = note_to_period(49, 0);
        channel.sample_pos = 5.0;
        channel.sample_direction = 1;
        channel.sample_sustain_loop_start = 2;
        channel.sample_sustain_loop_end = 6;
        channel.sample_sustain_loop_type = 1;
        if regular {
            channel.sample_loop_start = 20;
            channel.sample_loop_end = 30;
            channel.sample_loop_type = 1;
        }
        sample_channel(&mut channel, &[1000; 64], 22050);
        assert_eq!(channel.sample_pos, 2.0);
        channel.key_off = true;
        channel.sample_pos = 29.0;
        sample_channel(&mut channel, &[1000; 64], 22050);
        assert_eq!(channel.sample_pos, if regular { 20.0 } else { 30.0 });
    }
}

#[test]
fn invalid_it_instrument_only_keeps_both_effect_columns() {
    let song = module(
        vec![pattern(vec![
            TrackerNote {
                note: 49,
                instrument: 1,
                ..Default::default()
            },
            TrackerNote {
                instrument: 255,
                volume_effect: TrackerEffect::SetVolume(16),
                effect: TrackerEffect::SetVolumeEnvelope(false),
                ..Default::default()
            },
            TrackerNote {
                instrument: 255,
                volume_effect: TrackerEffect::SetVolumeEnvelope(true),
                effect: TrackerEffect::SetGlobalVolume(64),
                ..Default::default()
            },
        ])],
        vec![0],
        FormatFlags::IS_IT_FORMAT | FormatFlags::INSTRUMENTS,
    );
    let mut engine = TrackerEngine::new();
    let handle = engine.load_tracker_module(song, vec![7]);
    engine.process_row_tick0_internal(handle, &[]);
    engine.channels[0].sample_pos = 23.0;
    engine.channels[0].volume_envelope_enabled = true;
    engine.current_row = 1;
    engine.process_row_tick0_internal(handle, &[]);
    assert!(!engine.channels[0].volume_envelope_enabled);
    assert_eq!(engine.channels[0].volume, 0.25);
    assert_eq!(engine.channels[0].instrument, 1);
    assert_eq!(engine.channels[0].sample_pos, 23.0);
    engine.current_row = 2;
    engine.process_row_tick0_internal(handle, &[]);
    assert!(engine.channels[0].volume_envelope_enabled);
    assert_eq!(engine.global_volume, 0.5);
    assert_eq!(engine.channels[0].sample_handle, 7);
    assert_eq!(engine.channels[0].sample_pos, 23.0);
}

#[test]
fn it_new_note_discards_old_portamento_target() {
    let mut engine = TrackerEngine::new();
    let song = module(
        vec![pattern(vec![
            TrackerNote {
                note: 49,
                instrument: 1,
                ..Default::default()
            },
            TrackerNote {
                note: 61,
                effect: TrackerEffect::TonePortamento(4),
                ..Default::default()
            },
            TrackerNote {
                note: 49,
                ..Default::default()
            },
            row(TrackerEffect::TonePortamento(0)),
        ])],
        vec![0],
        FormatFlags::IS_IT_FORMAT | FormatFlags::INSTRUMENTS,
    );
    let handle = engine.load_tracker_module(song, vec![7]);
    for row in 0..3 {
        engine.current_row = row;
        engine.process_row_tick0_internal(handle, &[]);
    }
    let period = engine.channels[0].period;
    engine.current_row = 3;
    engine.process_row_tick0_internal(handle, &[]);
    engine.process_tick(1, 6);
    assert_eq!(engine.channels[0].period, period);
}

#[test]
fn it_instrument_note_map_selects_the_sample_handle_and_metadata() {
    let mut instrument = TrackerInstrument::default();
    instrument.note_sample_table[48] = (60, 2);
    let mut song = module(
        vec![pattern(vec![TrackerNote {
            note: 49,
            instrument: 1,
            ..Default::default()
        }])],
        vec![0],
        FormatFlags::IS_IT_FORMAT | FormatFlags::INSTRUMENTS,
    );
    song.instruments = vec![instrument];
    song.samples = vec![
        TrackerSample::default(),
        TrackerSample {
            default_volume: 32,
            loop_begin: 2,
            loop_end: 6,
            loop_type: nether_tracker::LoopType::Forward,
            ..Default::default()
        },
    ];

    let mut engine = TrackerEngine::new();
    let handle = engine.load_tracker_module(song, vec![7, 8]);
    engine.process_row_tick0_internal(handle, &[]);

    let channel = &engine.channels[0];
    assert_eq!(channel.sample_handle, 8);
    assert_eq!(channel.volume, 0.5);
    assert_eq!(channel.sample_loop_start, 2);
    assert_eq!(channel.sample_loop_end, 6);
    assert_eq!(
        channel.base_period,
        crate::tracker::utils::note_to_period(49, 0)
    );
}

#[test]
fn it_restart_preserves_sample_identity_when_pcm_handles_are_shared() {
    let mut song = module(
        vec![pattern(vec![
            TrackerNote {
                note: 49,
                instrument: 1,
                ..Default::default()
            },
            TrackerNote {
                instrument: 1,
                ..Default::default()
            },
        ])],
        vec![0],
        FormatFlags::IS_IT_FORMAT | FormatFlags::INSTRUMENTS,
    );
    song.instruments[0].note_sample_table[48] = (48, 2);
    song.samples = vec![
        TrackerSample {
            default_volume: 16,
            ..Default::default()
        },
        TrackerSample {
            default_volume: 48,
            ..Default::default()
        },
    ];
    let mut engine = TrackerEngine::new();
    let handle = engine.load_tracker_module(song, vec![7, 7]);
    engine.process_row_tick0_internal(handle, &[]);
    assert_eq!(engine.channels[0].volume, 0.75);
    let snapshot = engine.snapshot();
    engine.channels[0].sample_index = Some(0);
    engine.apply_snapshot(&snapshot);
    assert_eq!(engine.channels[0].sample_index, Some(1));
    let background = engine.channels[0].copy_to_background(0);
    assert_eq!(background.sample_index, Some(1));
    engine.channels[0].note_on = false;
    engine.current_row = 1;
    engine.process_row_tick0_internal(handle, &[]);
    assert_eq!(engine.channels[0].volume, 0.75);
}

#[test]
fn it_instrument_only_change_restarts_with_the_incoming_sample() {
    for playing in [false, true] {
        let mut song = module(
            vec![pattern(vec![
                TrackerNote {
                    note: 49,
                    instrument: 1,
                    ..Default::default()
                },
                TrackerNote {
                    instrument: 2,
                    ..Default::default()
                },
                TrackerNote {
                    instrument: 99,
                    ..Default::default()
                },
            ])],
            vec![0],
            FormatFlags::IS_IT_FORMAT | FormatFlags::INSTRUMENTS,
        );
        song.instruments.push(TrackerInstrument::default());
        song.instruments[1].note_sample_table[48] = (72, 2);
        song.samples = vec![TrackerSample::default(), TrackerSample::default()];
        let mut engine = TrackerEngine::new();
        let handle = engine.load_tracker_module(song, vec![7, 8]);
        engine.process_row_tick0_internal(handle, &[]);
        engine.channels[0].note_on = playing;
        engine.channels[0].sample_pos = 123.0;
        engine.current_row = 1;
        engine.process_row_tick0_internal(handle, &[]);
        assert_eq!(engine.channels[0].sample_handle, 8);
        assert_eq!(engine.channels[0].sample_index, Some(1));
        assert_eq!(engine.channels[0].sample_pos, 0.0);
        assert_eq!(
            engine.channels[0].period,
            crate::tracker::utils::note_to_period(61, 0)
        );
        assert_eq!(engine.channels[0].current_note, 49);
        engine.channels[0].note_on = false;
        engine.current_row = 2;
        engine.process_row_tick0_internal(handle, &[]);
        assert!(!engine.channels[0].note_on);
        assert_eq!(engine.channels[0].instrument, 2);
        assert_eq!(engine.channels[0].sample_index, Some(1));
    }
}

#[test]
fn it_lone_instrument_restarts_a_stopped_note_and_resets_portamento() {
    let mut engine = TrackerEngine::new();
    let mut song = module(
        vec![pattern(vec![
            TrackerNote {
                note: 49,
                instrument: 1,
                ..Default::default()
            },
            TrackerNote {
                instrument: 1,
                ..Default::default()
            },
        ])],
        vec![0],
        FormatFlags::IS_IT_FORMAT | FormatFlags::INSTRUMENTS,
    );
    song.samples = vec![TrackerSample {
        default_volume: 32,
        default_pan: Some(32),
        ..Default::default()
    }];
    let handle = engine.load_tracker_module(song, vec![7]);
    engine.process_row_tick0_internal(handle, &[]);
    let period = engine.channels[0].base_period;
    engine.channels[0].note_on = false;
    engine.channels[0].volume = 0.1;
    engine.channels[0].panning = 0.875;
    engine.channels[0].sample_pos = 123.0;
    engine.channels[0].target_period = period * 2.0;
    // Pitch slides must not become the pitch of an instrument-only restart.
    engine.channels[0].base_period = period * 0.75;
    engine.channels[0].period = period * 0.75;

    engine.current_row = 1;
    engine.process_row_tick0_internal(handle, &[]);

    let channel = &engine.channels[0];
    assert!(channel.note_on);
    assert_eq!(channel.volume, 0.5);
    assert_eq!(channel.panning, 0.0);
    assert_eq!(channel.sample_pos, 0.0);
    assert_eq!(channel.period, period);
    assert_eq!(channel.target_period, 0.0);
}

#[test]
fn it_cut_note_stops_voice_even_with_a_main_effect() {
    let mut engine = TrackerEngine::new();
    let song = module(
        vec![pattern(vec![
            TrackerNote {
                note: 49,
                instrument: 1,
                ..Default::default()
            },
            TrackerNote {
                note: TrackerNote::NOTE_CUT,
                effect: TrackerEffect::PatternBreak(0),
                ..Default::default()
            },
        ])],
        vec![0],
        FormatFlags::IS_IT_FORMAT | FormatFlags::INSTRUMENTS,
    );
    let handle = engine.load_tracker_module(song, vec![0]);
    engine.process_row_tick0_internal(handle, &[]);
    assert!(engine.channels[0].note_on);
    engine.current_row = 1;
    engine.process_row_tick0_internal(handle, &[]);
    assert!(!engine.channels[0].note_on);
}

#[test]
fn row_speed_and_tempo_commands_update_the_clock() {
    let mut engine = TrackerEngine::new();
    let handle = engine.load_tracker_module(
        module(
            vec![pattern(vec![
                row(TrackerEffect::SetSpeed(2)),
                row(TrackerEffect::SetTempo(200)),
            ])],
            vec![0],
            FormatFlags::IS_IT_FORMAT,
        ),
        vec![0],
    );
    let mut state = make_state(handle, 6, 125);

    advance_ticks(&mut engine, &mut state, 2, 125);
    assert_eq!((state.row, state.speed), (1, 2));

    engine.advance_positions(&mut state, &[], 1, 44_100);
    assert_eq!(state.bpm, 200);
}

#[test]
fn jump_and_break_choose_the_requested_order_and_row() {
    let mut engine = TrackerEngine::new();
    let handle = engine.load_tracker_module(
        module(
            vec![
                pattern(vec![row(TrackerEffect::PositionJump(1))]),
                pattern(vec![
                    row(TrackerEffect::None),
                    row(TrackerEffect::None),
                    row(TrackerEffect::None),
                ]),
            ],
            vec![0, 1],
            FormatFlags::IS_IT_FORMAT,
        ),
        vec![0],
    );
    let mut state = make_state(handle, 1, 125);

    advance_ticks(&mut engine, &mut state, 1, 125);
    assert_eq!((state.order_position, state.row), (1, 0));

    let mut engine = TrackerEngine::new();
    let handle = engine.load_tracker_module(
        module(
            vec![
                pattern(vec![row(TrackerEffect::PatternBreak(2))]),
                pattern(vec![
                    row(TrackerEffect::None),
                    row(TrackerEffect::None),
                    row(TrackerEffect::None),
                ]),
            ],
            vec![0, 1],
            FormatFlags::IS_IT_FORMAT,
        ),
        vec![0],
    );
    let mut state = make_state(handle, 1, 125);
    advance_ticks(&mut engine, &mut state, 1, 125);
    assert_eq!((state.order_position, state.row), (1, 2));
}

#[test]
fn it_skip_is_ignored_and_end_stops_playback() {
    let mut engine = TrackerEngine::new();
    let handle = engine.load_tracker_module(
        module(
            vec![
                pattern(vec![row(TrackerEffect::None)]),
                pattern(vec![row(TrackerEffect::None)]),
            ],
            vec![0, 254, 1, 255],
            FormatFlags::IS_IT_FORMAT,
        ),
        vec![0],
    );
    let mut state = make_state(handle, 1, 125);

    advance_ticks(&mut engine, &mut state, 1, 125);
    assert_eq!((state.order_position, state.row), (2, 0));
    advance_ticks(&mut engine, &mut state, 1, 125);
    assert_eq!(state.flags & tracker_flags::PLAYING, 0);
}

#[test]
fn pattern_loop_returns_to_marked_row_for_the_requested_count() {
    let mut engine = TrackerEngine::new();
    let handle = engine.load_tracker_module(
        module(
            vec![pattern(vec![
                row(TrackerEffect::PatternLoop(0)),
                row(TrackerEffect::PatternLoop(1)),
                row(TrackerEffect::None),
            ])],
            vec![0],
            FormatFlags::IS_IT_FORMAT,
        ),
        vec![0],
    );
    let mut state = make_state(handle, 1, 125);

    advance_ticks(&mut engine, &mut state, 1, 125);
    assert_eq!(state.row, 1);
    advance_ticks(&mut engine, &mut state, 1, 125);
    assert_eq!(state.row, 0);
    advance_ticks(&mut engine, &mut state, 1, 125);
    assert_eq!(state.row, 1);
    advance_ticks(&mut engine, &mut state, 1, 125);
    assert_eq!(state.row, 2);
}

#[test]
fn pattern_delay_holds_without_retriggering_the_row() {
    let mut engine = TrackerEngine::new();
    let handle = engine.load_tracker_module(
        module(
            vec![pattern(vec![
                TrackerNote {
                    note: 48,
                    instrument: 1,
                    effect: TrackerEffect::PatternDelay(2),
                    ..Default::default()
                },
                row(TrackerEffect::None),
            ])],
            vec![0],
            FormatFlags::IS_IT_FORMAT,
        ),
        vec![1],
    );
    let sounds = vec![
        None,
        Some(crate::audio::Sound {
            data: vec![1000i16; 4096].into(),
        }),
    ];
    let mut state = make_state(handle, 1, 125);
    let spt = samples_per_tick(125, 44_100);

    engine.advance_positions(&mut state, &sounds, spt, 44_100);
    let after_first_tick = engine.channels[0].sample_pos;
    assert_eq!(state.row, 0);
    engine.advance_positions(&mut state, &sounds, spt, 44_100);
    assert!(engine.channels[0].sample_pos > after_first_tick * 1.5);
}

#[test]
fn row_end_does_not_run_an_extra_per_tick_effect() {
    let mut engine = TrackerEngine::new();
    let handle = engine.load_tracker_module(
        module(
            vec![pattern(vec![
                TrackerNote {
                    note: 48,
                    instrument: 1,
                    volume: 64,
                    effect: TrackerEffect::VolumeSlide { up: 0, down: 1 },
                    ..Default::default()
                },
                row(TrackerEffect::None),
            ])],
            vec![0],
            FormatFlags::IS_IT_FORMAT,
        ),
        vec![0],
    );
    let mut state = make_state(handle, 3, 125);

    advance_ticks(&mut engine, &mut state, 3, 125);
    assert_eq!(state.row, 1);
    assert!((engine.channels[0].volume - 62.0 / 64.0).abs() < f32::EPSILON);
}

#[test]
fn global_slide_memory_is_per_channel_and_it_fine_slides_recall() {
    for format in [FormatFlags::empty(), FormatFlags::IS_IT_FORMAT] {
        let mut song = module(vec![], vec![0], format);
        song.num_channels = 2;
        song.patterns.push(TrackerPattern {
            num_rows: 3,
            notes: vec![
                vec![
                    row(TrackerEffect::GlobalVolumeSlide { up: 0, down: 1 }),
                    row(TrackerEffect::None),
                ],
                vec![
                    row(TrackerEffect::None),
                    row(TrackerEffect::GlobalVolumeSlide { up: 0, down: 0 }),
                ],
                vec![row(TrackerEffect::None), row(TrackerEffect::None)],
            ],
        });
        let mut engine = TrackerEngine::new();
        let handle = engine.load_tracker_module(song, vec![0]);
        let mut state = make_state(handle, 2, 125);
        advance_ticks(&mut engine, &mut state, 4, 125);
        let scale = if format.contains(FormatFlags::IS_IT_FORMAT) {
            128.0
        } else {
            64.0
        };
        assert_eq!(engine.global_volume, 1.0 - 1.0 / scale);
    }
    let mut engine = TrackerEngine::new();
    let handle = engine.load_tracker_module(
        module(
            vec![pattern(vec![
                row(TrackerEffect::FineGlobalVolumeDown(2)),
                row(TrackerEffect::None),
                row(TrackerEffect::GlobalVolumeSlide { up: 0, down: 0 }),
                row(TrackerEffect::None),
            ])],
            vec![0],
            FormatFlags::IS_IT_FORMAT,
        ),
        vec![0],
    );
    let mut state = make_state(handle, 3, 125);
    advance_ticks(&mut engine, &mut state, 9, 125);
    assert_eq!(engine.global_volume, 124.0 / 128.0);
}

#[test]
fn global_volume_slide_is_active_only_for_its_row() {
    for format in [FormatFlags::empty(), FormatFlags::IS_IT_FORMAT] {
        let mut engine = TrackerEngine::new();
        let handle = engine.load_tracker_module(
            module(
                vec![pattern(vec![
                    row(TrackerEffect::GlobalVolumeSlide { up: 0, down: 1 }),
                    row(TrackerEffect::None),
                    row(TrackerEffect::GlobalVolumeSlide { up: 0, down: 0 }),
                    row(TrackerEffect::None),
                ])],
                vec![0],
                format,
            ),
            vec![0],
        );
        let mut state = make_state(handle, 2, 125);
        let step = if format.contains(FormatFlags::IS_IT_FORMAT) {
            1.0 / 128.0
        } else {
            1.0 / 64.0
        };
        advance_ticks(&mut engine, &mut state, 4, 125);
        assert_eq!(engine.global_volume, 1.0 - step);
        let saved = engine.snapshot();
        let saved_state = state;
        advance_ticks(&mut engine, &mut state, 2, 125);
        assert_eq!(engine.global_volume, 1.0 - 2.0 * step);
        engine.apply_snapshot(&saved);
        state = saved_state;
        advance_ticks(&mut engine, &mut state, 2, 125);
        assert_eq!(engine.global_volume, 1.0 - 2.0 * step);
    }
}
