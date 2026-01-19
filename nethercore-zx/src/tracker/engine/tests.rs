//! Unit tests for tracker engine

use super::super::TrackerEngine;
use super::super::channels::NNA_CUT;
use nether_tracker::{
    FormatFlags, NewNoteAction, TrackerEffect, TrackerInstrument, TrackerModule, TrackerNote,
    TrackerPattern,
};

#[test]
fn test_nna_uses_new_instrument_nna_not_channel_state() {
    // This test verifies that when a new note is triggered, the NNA action
    // comes from the NEW instrument, not the channel's previous state.
    //
    // Bug scenario: Channel has NNA=Cut from previous instrument,
    // new instrument has NNA=Continue. The old note should continue
    // playing in a background channel, not be cut.

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
    engine.channels[0].nna = NNA_CUT; // Channel state says NNA_CUT (stale value!)
    engine.channels[0].instrument = 1;

    // Process row 1 - new note with NNA=Continue instrument
    // This should use the instrument's NNA (Continue), not channel's (Cut)
    engine.current_row = 1;
    engine.process_row_tick0_internal(handle, &[]);

    // Verify: Background channel (index 1) should have the old note
    // because the NEW instrument has NNA=Continue
    assert!(
        engine.channels[1].note_on,
        "NNA=Continue should move old note to background channel. \
         Bug: NNA is reading from channel state (Cut) instead of new instrument (Continue)"
    );
    assert!(
        engine.channels[1].is_background,
        "Background flag should be set"
    );
}

#[test]
fn test_nna_note_fade_triggers_key_off() {
    // Verify NNA=NoteFade properly triggers key_off and fadeout on displaced note

    let mut engine = TrackerEngine::new();

    let mut instr = TrackerInstrument::default();
    instr.nna = NewNoteAction::NoteFade;
    instr.fadeout = 2048;

    let note = TrackerNote {
        note: 60,
        instrument: 1,
        volume: 64,
        effect: TrackerEffect::None,
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

    // Set up channel with playing note (stale NNA=Cut)
    engine.channels[0].note_on = true;
    engine.channels[0].sample_handle = 1;
    engine.channels[0].volume = 1.0;
    engine.channels[0].volume_fadeout = 65535;
    engine.channels[0].nna = NNA_CUT; // Stale value
    engine.channels[0].instrument = 1;
    engine.channels[0].instrument_fadeout_rate = 0; // Will be set by NNA

    engine.current_row = 1;
    engine.process_row_tick0_internal(handle, &[]);

    // Background channel should have the old note with key_off triggered
    assert!(
        engine.channels[1].note_on,
        "NNA=NoteFade should move note to background"
    );
    assert!(
        engine.channels[1].key_off,
        "NNA=NoteFade should trigger key_off"
    );
    assert!(
        engine.channels[1].instrument_fadeout_rate > 0,
        "Fadeout rate should be set"
    );
}
