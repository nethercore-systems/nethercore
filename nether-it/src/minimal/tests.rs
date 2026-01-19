//! Tests for NCIT format

use std::io::Cursor;

use crate::IT_MAGIC;
use crate::module::*;

use super::legacy::pack_it_minimal;
use super::pack::{pack_envelope, pack_note_sample_table, pack_sample};
use super::parse::{parse_envelope, parse_note_sample_table, parse_sample};
use super::{TABLE_UNIFORM, pack_ncit, parse_it_minimal, parse_ncit};

/// Create a minimal test module
fn create_test_module() -> ItModule {
    let mut module = ItModule::default();
    module.name = "Test".to_string();
    module.num_channels = 4;
    module.initial_speed = 6;
    module.initial_tempo = 125;

    // Add a pattern
    let mut pattern = ItPattern::empty(64, 4);
    pattern.notes[0][0] = ItNote::play_note(48, 1, 64); // C-4
    module.patterns.push(pattern);
    module.num_patterns = 1;

    // Add an instrument
    let mut instr = ItInstrument::default();
    instr.fadeout = 256;
    instr.volume_envelope = Some(ItEnvelope {
        points: vec![(0, 64), (10, 32), (20, 0)],
        loop_begin: 0,
        loop_end: 2,
        sustain_begin: 1,
        sustain_end: 1,
        flags: ItEnvelopeFlags::ENABLED | ItEnvelopeFlags::SUSTAIN_LOOP,
    });
    module.instruments.push(instr);
    module.num_instruments = 1;

    // Add a sample
    let mut sample = ItSample::default();
    sample.c5_speed = 22050;
    sample.loop_begin = 100;
    sample.loop_end = 1000;
    sample.flags = ItSampleFlags::LOOP;
    module.samples.push(sample);
    module.num_samples = 1;

    // Set order table
    module.order_table = vec![0];
    module.num_orders = 1;

    module
}

#[test]
fn test_pack_and_parse_ncit() {
    let module = create_test_module();

    // Pack to NCIT
    let ncit = pack_ncit(&module);

    // Verify it doesn't start with IT magic
    assert_ne!(&ncit[0..4], IT_MAGIC);

    // Parse back
    let parsed = parse_ncit(&ncit).expect("Failed to parse NCIT");

    // Verify header fields
    assert_eq!(parsed.num_channels, module.num_channels);
    assert_eq!(parsed.num_patterns, module.num_patterns);
    assert_eq!(parsed.num_instruments, module.num_instruments);
    assert_eq!(parsed.num_samples, module.num_samples);
    assert_eq!(parsed.initial_speed, module.initial_speed);
    assert_eq!(parsed.initial_tempo, module.initial_tempo);
    assert_eq!(parsed.global_volume, module.global_volume);

    // Verify patterns
    assert_eq!(parsed.patterns.len(), 1);
    assert_eq!(parsed.patterns[0].num_rows, 64);

    // Verify instruments
    assert_eq!(parsed.instruments.len(), 1);
    assert_eq!(parsed.instruments[0].fadeout, 256);
    assert!(parsed.instruments[0].volume_envelope.is_some());

    // Verify samples
    assert_eq!(parsed.samples.len(), 1);
    assert_eq!(parsed.samples[0].c5_speed, 22050);
    assert_eq!(parsed.samples[0].loop_begin, 100);
    assert_eq!(parsed.samples[0].loop_end, 1000);
}

#[test]
fn test_ncit_size_reduction() {
    let module = create_test_module();

    // Pack to both formats
    let ncit = pack_ncit(&module);
    let legacy = pack_it_minimal(&module);

    println!("NCIT size: {} bytes", ncit.len());
    println!("Legacy IT size: {} bytes", legacy.len());
    println!(
        "Savings: {} bytes ({:.1}%)",
        legacy.len() - ncit.len(),
        (1.0 - ncit.len() as f64 / legacy.len() as f64) * 100.0
    );

    // NCIT should be significantly smaller
    assert!(
        ncit.len() < legacy.len() / 2,
        "NCIT should be at least 50% smaller than legacy"
    );
}

#[test]
fn test_auto_detection() {
    let module = create_test_module();

    // Pack to NCIT
    let ncit = pack_ncit(&module);

    // Auto-detect should recognize NCIT
    let parsed = parse_it_minimal(&ncit).expect("Failed to auto-detect NCIT");
    assert_eq!(parsed.num_channels, module.num_channels);

    // Pack to legacy IT
    let legacy = pack_it_minimal(&module);

    // Auto-detect should recognize IT
    let parsed2 = parse_it_minimal(&legacy).expect("Failed to auto-detect IT");
    assert_eq!(parsed2.num_channels, module.num_channels);
}

#[test]
fn test_note_sample_table_compression() {
    // Test uniform table
    let mut table = [(0u8, 0u8); 120];
    for (i, entry) in table.iter_mut().enumerate() {
        entry.0 = i as u8;
        entry.1 = 1; // All use sample 1
    }

    let mut output = Vec::new();
    pack_note_sample_table(&mut output, &table);

    // Uniform encoding should be tiny (2 bytes)
    assert_eq!(output.len(), 2);
    assert_eq!(output[0], TABLE_UNIFORM);
    assert_eq!(output[1], 1);

    // Parse it back
    let mut cursor = Cursor::new(output.as_slice());
    let parsed = parse_note_sample_table(&mut cursor).unwrap();

    for (i, &(note, sample)) in parsed.iter().enumerate() {
        assert_eq!(note, i as u8);
        assert_eq!(sample, 1);
    }
}

#[test]
fn test_envelope_round_trip() {
    let env = ItEnvelope {
        points: vec![(0, 64), (10, 32), (20, 0)],
        loop_begin: 0,
        loop_end: 2,
        sustain_begin: 1,
        sustain_end: 1,
        flags: ItEnvelopeFlags::ENABLED | ItEnvelopeFlags::LOOP,
    };

    let mut output = Vec::new();
    pack_envelope(&mut output, &env);

    let mut cursor = Cursor::new(output.as_slice());
    let parsed = parse_envelope(&mut cursor).unwrap();

    assert_eq!(parsed.points.len(), 3);
    assert_eq!(parsed.points[0], (0, 64));
    assert_eq!(parsed.points[1], (10, 32));
    assert_eq!(parsed.points[2], (20, 0));
    assert_eq!(parsed.loop_begin, 0);
    assert_eq!(parsed.loop_end, 2);
    assert!(parsed.flags.contains(ItEnvelopeFlags::ENABLED));
    assert!(parsed.flags.contains(ItEnvelopeFlags::LOOP));
}

#[test]
fn test_sample_round_trip() {
    let sample = ItSample {
        name: String::new(),
        filename: String::new(),
        global_volume: 48,
        flags: ItSampleFlags::LOOP | ItSampleFlags::PINGPONG_LOOP,
        default_volume: 32,
        default_pan: Some(16),
        length: 0,
        loop_begin: 100,
        loop_end: 500,
        c5_speed: 44100,
        sustain_loop_begin: 0,
        sustain_loop_end: 0,
        vibrato_speed: 10,
        vibrato_depth: 20,
        vibrato_rate: 30,
        vibrato_type: 1,
    };

    let mut output = Vec::new();
    pack_sample(&mut output, &sample);

    let mut cursor = Cursor::new(output.as_slice());
    let parsed = parse_sample(&mut cursor).unwrap();

    assert_eq!(parsed.global_volume, 48);
    assert_eq!(parsed.default_volume, 32);
    assert_eq!(parsed.default_pan, Some(16));
    assert_eq!(parsed.c5_speed, 44100);
    assert_eq!(parsed.loop_begin, 100);
    assert_eq!(parsed.loop_end, 500);
    assert!(parsed.flags.contains(ItSampleFlags::LOOP));
    assert!(parsed.flags.contains(ItSampleFlags::PINGPONG_LOOP));
    assert_eq!(parsed.vibrato_speed, 10);
    assert_eq!(parsed.vibrato_depth, 20);
}

#[test]
fn test_multiple_round_trips() {
    let module = create_test_module();

    // Do multiple round-trips
    let ncit1 = pack_ncit(&module);
    let parsed1 = parse_ncit(&ncit1).unwrap();
    let ncit2 = pack_ncit(&parsed1);
    let parsed2 = parse_ncit(&ncit2).unwrap();

    // Should be identical after multiple round-trips
    assert_eq!(parsed1.num_channels, parsed2.num_channels);
    assert_eq!(parsed1.num_patterns, parsed2.num_patterns);
    assert_eq!(parsed1.initial_speed, parsed2.initial_speed);
    assert_eq!(
        ncit1, ncit2,
        "NCIT data should be identical after round-trip"
    );
}

#[test]
fn test_channel_and_random_settings_round_trip() {
    let mut module = create_test_module();

    // Set custom channel settings
    module.channel_pan[0] = 0; // Hard left
    module.channel_pan[1] = 64; // Hard right
    module.channel_pan[2] = 32; // Center
    module.channel_pan[3] = 48; // Right-center
    module.channel_vol[0] = 48; // 75% volume
    module.channel_vol[1] = 32; // 50% volume
    module.channel_vol[2] = 64; // 100% volume
    module.channel_vol[3] = 16; // 25% volume

    // Set random variation on instrument
    module.instruments[0].random_volume = 10; // ±10% volume
    module.instruments[0].random_pan = 5; // ±5 pan units

    // Pack and parse
    let ncit = pack_ncit(&module);
    let parsed = parse_ncit(&ncit).unwrap();

    // Verify channel panning preserved
    assert_eq!(
        parsed.channel_pan[0], 0,
        "Channel 0 pan should be hard left"
    );
    assert_eq!(
        parsed.channel_pan[1], 64,
        "Channel 1 pan should be hard right"
    );
    assert_eq!(parsed.channel_pan[2], 32, "Channel 2 pan should be center");
    assert_eq!(
        parsed.channel_pan[3], 48,
        "Channel 3 pan should be right-center"
    );

    // Verify channel volume preserved
    assert_eq!(parsed.channel_vol[0], 48, "Channel 0 vol should be 75%");
    assert_eq!(parsed.channel_vol[1], 32, "Channel 1 vol should be 50%");
    assert_eq!(parsed.channel_vol[2], 64, "Channel 2 vol should be 100%");
    assert_eq!(parsed.channel_vol[3], 16, "Channel 3 vol should be 25%");

    // Verify random settings preserved
    assert_eq!(
        parsed.instruments[0].random_volume, 10,
        "Random volume should be preserved"
    );
    assert_eq!(
        parsed.instruments[0].random_pan, 5,
        "Random pan should be preserved"
    );
}
