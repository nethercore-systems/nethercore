//! Tests for nether-tracker types

use crate::effects::TrackerEffect;
use crate::instrument::EnvelopeFlags;
use crate::instrument::TrackerEnvelope;
use crate::pattern::{TrackerNote, TrackerPattern};
use crate::sample::TrackerSample;
use crate::{FormatFlags, TrackerModule};

#[test]
fn test_tracker_note_methods() {
    let note = TrackerNote {
        note: 48, // C-4
        instrument: 1,
        volume: 32,
        effect: TrackerEffect::None,
    };
    assert!(note.has_note());
    assert!(note.has_instrument());
    assert!(!note.is_note_off());
    assert!(!note.is_note_cut());

    let note_off = TrackerNote {
        note: TrackerNote::NOTE_OFF,
        ..Default::default()
    };
    assert!(note_off.is_note_off());
    assert!(!note_off.has_note());
}

#[test]
fn test_envelope_interpolation() {
    let env = TrackerEnvelope {
        points: vec![(0, 64), (10, 32), (20, 0)],
        flags: EnvelopeFlags::ENABLED,
        ..Default::default()
    };

    assert_eq!(env.value_at(0), 64);
    assert_eq!(env.value_at(5), 48); // Midpoint between 64 and 32
    assert_eq!(env.value_at(10), 32);
    assert_eq!(env.value_at(15), 16); // Midpoint between 32 and 0
    assert_eq!(env.value_at(20), 0);
    assert_eq!(env.value_at(30), 0); // Past end
}

#[test]
fn test_pattern_empty() {
    let pattern = TrackerPattern::empty(64, 8);
    assert_eq!(pattern.num_rows, 64);
    assert_eq!(pattern.notes.len(), 64);
    assert_eq!(pattern.notes[0].len(), 8);
}

#[test]
fn test_tracker_sample_auto_vibrato_defaults() {
    let sample = TrackerSample::default();

    // Auto-vibrato should default to off (all zeros)
    assert_eq!(sample.vibrato_speed, 0);
    assert_eq!(sample.vibrato_depth, 0);
    assert_eq!(sample.vibrato_rate, 0);
    assert_eq!(sample.vibrato_type, 0);
}

#[test]
fn test_tracker_sample_auto_vibrato_fields() {
    let sample = TrackerSample {
        vibrato_speed: 15,
        vibrato_depth: 32,
        vibrato_rate: 64,
        vibrato_type: 1, // ramp down
        ..Default::default()
    };

    assert_eq!(sample.vibrato_speed, 15);
    assert_eq!(sample.vibrato_depth, 32);
    assert_eq!(sample.vibrato_rate, 64);
    assert_eq!(sample.vibrato_type, 1);
}

#[test]
fn test_tracker_module_mix_volume_default() {
    // IT modules should have mix_volume from header
    // XM modules default to 128 (full volume)
    // This tests the field exists and can hold IT range (0-128)
    let module = TrackerModule {
        name: "Test".to_string(),
        num_channels: 4,
        initial_speed: 6,
        initial_tempo: 125,
        global_volume: 128,
        mix_volume: 80, // IT allows 0-128
        panning_separation: 128,
        order_table: vec![0],
        patterns: vec![],
        instruments: vec![],
        samples: vec![],
        format: FormatFlags::IS_IT_FORMAT,
        message: None,
        restart_position: 0,
    };

    assert_eq!(module.mix_volume, 80);
}

#[test]
fn test_tracker_module_panning_separation() {
    // Panning separation: 0 = mono, 128 = full stereo
    let mono_module = TrackerModule {
        name: "Mono".to_string(),
        num_channels: 4,
        initial_speed: 6,
        initial_tempo: 125,
        global_volume: 128,
        mix_volume: 128,
        panning_separation: 0, // Mono
        order_table: vec![0],
        patterns: vec![],
        instruments: vec![],
        samples: vec![],
        format: FormatFlags::IS_IT_FORMAT,
        message: None,
        restart_position: 0,
    };

    let stereo_module = TrackerModule {
        panning_separation: 128, // Full stereo
        ..mono_module.clone()
    };

    assert_eq!(mono_module.panning_separation, 0);
    assert_eq!(stereo_module.panning_separation, 128);
}
