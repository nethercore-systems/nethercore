//! Tests for minimal XM format packing and parsing

use crate::module::{XmEnvelope, XmInstrument, XmModule, XmPattern};
use crate::XmNote;

use super::{pack_xm_minimal, parse_xm_minimal, HEADER_SIZE, MAX_ENVELOPE_POINTS};

/// Create a minimal test module
fn create_test_module() -> XmModule {
    // Create a pattern with actual 64 rows
    let mut notes = Vec::with_capacity(64);
    for row in 0..64 {
        if row == 0 {
            // First row has a note
            notes.push(vec![
                XmNote {
                    note: 49, // C-4
                    instrument: 1,
                    volume: 0x40,
                    effect: 0,
                    effect_param: 0,
                },
                XmNote::default(),
            ]);
        } else {
            // Other rows are empty
            notes.push(vec![XmNote::default(), XmNote::default()]);
        }
    }

    let pattern = XmPattern {
        num_rows: 64,
        notes,
    };

    let instrument = XmInstrument {
        name: "Test".to_string(),
        num_samples: 1,
        volume_envelope: Some(XmEnvelope {
            points: vec![(0, 64), (10, 32), (20, 0)],
            sustain_point: 1,
            loop_start: 0,
            loop_end: 2,
            enabled: true,
            sustain_enabled: true,
            loop_enabled: false,
        }),
        panning_envelope: None,
        vibrato_type: 0,
        vibrato_sweep: 0,
        vibrato_depth: 0,
        vibrato_rate: 0,
        volume_fadeout: 256,
        sample_finetune: 0,
        sample_relative_note: 0,
        sample_loop_start: 0,
        sample_loop_length: 1000,
        sample_loop_type: 1,
    };

    XmModule {
        name: "Test Module".to_string(),
        num_channels: 2,
        num_patterns: 1,
        num_instruments: 1,
        song_length: 1,
        restart_position: 0,
        default_speed: 6,
        default_bpm: 125,
        linear_frequency_table: true,
        order_table: vec![0],
        patterns: vec![pattern],
        instruments: vec![instrument],
    }
}

#[test]
fn test_pack_and_parse_minimal() {
    let module = create_test_module();

    // Pack to minimal format
    let packed = pack_xm_minimal(&module).expect("Packing should succeed");

    // Verify header starts with num_channels (2 in our test)
    assert_eq!(packed[0], 2);

    // Parse back
    let parsed = parse_xm_minimal(&packed).expect("Parsing should succeed");

    // Verify header fields
    assert_eq!(parsed.num_channels, module.num_channels);
    assert_eq!(parsed.num_patterns, module.num_patterns);
    assert_eq!(parsed.num_instruments, module.num_instruments);
    assert_eq!(parsed.song_length, module.song_length);
    assert_eq!(parsed.restart_position, module.restart_position);
    assert_eq!(parsed.default_speed, module.default_speed);
    assert_eq!(parsed.default_bpm, module.default_bpm);
    assert_eq!(parsed.linear_frequency_table, module.linear_frequency_table);

    // Verify pattern order
    assert_eq!(parsed.order_table, module.order_table);

    // Verify patterns
    assert_eq!(parsed.patterns.len(), 1);
    assert_eq!(parsed.patterns[0].num_rows, 64);

    // Verify instruments
    assert_eq!(parsed.instruments.len(), 1);
    let instr = &parsed.instruments[0];
    assert_eq!(instr.num_samples, 1);
    assert_eq!(instr.vibrato_type, 0);
    assert_eq!(instr.volume_fadeout, 256);
    assert_eq!(instr.sample_loop_length, 1000);
    assert_eq!(instr.sample_loop_type, 1);

    // Verify volume envelope
    assert!(instr.volume_envelope.is_some());
    let env = instr.volume_envelope.as_ref().unwrap();
    assert_eq!(env.points.len(), 3);
    assert_eq!(env.points[0], (0, 64));
    assert_eq!(env.points[1], (10, 32));
    assert_eq!(env.points[2], (20, 0));
    assert_eq!(env.sustain_point, 1);
    assert!(env.enabled);
    assert!(env.sustain_enabled);
    assert!(!env.loop_enabled);

    // Verify no panning envelope
    assert!(instr.panning_envelope.is_none());
}

#[test]
fn test_minimal_format_size() {
    let module = create_test_module();
    let packed = pack_xm_minimal(&module).unwrap();

    // Verify size is minimal
    // Header: 16 bytes
    // Pattern order: 1 byte
    // Pattern: 2 (rows) + 2 (size) + data
    // Instrument: 1 (flags) + envelope + vibrato (4) + fadeout (2) + sample (15)

    println!("Packed size: {} bytes", packed.len());
    assert!(packed.len() < 200, "Minimal format should be compact");
}

#[test]
fn test_multiple_patterns() {
    let mut module = create_test_module();

    // Add more patterns
    module.num_patterns = 3;
    module.patterns.push(module.patterns[0].clone());
    module.patterns.push(module.patterns[0].clone());
    module.song_length = 3;
    module.order_table = vec![0, 1, 2];

    let packed = pack_xm_minimal(&module).unwrap();
    let parsed = parse_xm_minimal(&packed).unwrap();

    assert_eq!(parsed.num_patterns, 3);
    assert_eq!(parsed.patterns.len(), 3);
    assert_eq!(parsed.order_table, vec![0, 1, 2]);
}

#[test]
fn test_multiple_instruments() {
    let mut module = create_test_module();

    // Add another instrument with panning envelope
    let mut instr2 = module.instruments[0].clone();
    instr2.volume_envelope = None;
    instr2.panning_envelope = Some(XmEnvelope {
        points: vec![(0, 32), (5, 64)],
        sustain_point: 0,
        loop_start: 0,
        loop_end: 1,
        enabled: true,
        sustain_enabled: false,
        loop_enabled: true,
    });

    module.num_instruments = 2;
    module.instruments.push(instr2);

    let packed = pack_xm_minimal(&module).unwrap();
    let parsed = parse_xm_minimal(&packed).unwrap();

    assert_eq!(parsed.num_instruments, 2);
    assert_eq!(parsed.instruments.len(), 2);

    // First instrument has volume envelope
    assert!(parsed.instruments[0].volume_envelope.is_some());
    assert!(parsed.instruments[0].panning_envelope.is_none());

    // Second instrument has panning envelope
    assert!(parsed.instruments[1].volume_envelope.is_none());
    assert!(parsed.instruments[1].panning_envelope.is_some());

    let pan_env = parsed.instruments[1].panning_envelope.as_ref().unwrap();
    assert_eq!(pan_env.points.len(), 2);
    assert_eq!(pan_env.points[0], (0, 32));
    assert!(pan_env.loop_enabled);
}

#[test]
fn test_instrument_without_samples() {
    let mut module = create_test_module();
    module.instruments[0].num_samples = 0;

    let packed = pack_xm_minimal(&module).unwrap();
    let parsed = parse_xm_minimal(&packed).unwrap();

    assert_eq!(parsed.instruments[0].num_samples, 0);
}

#[test]
fn test_empty_pattern() {
    let mut module = create_test_module();

    // Create empty pattern
    let empty_pattern = XmPattern {
        num_rows: 64,
        notes: vec![vec![XmNote::default(), XmNote::default()]; 64],
    };
    module.patterns[0] = empty_pattern;

    let packed = pack_xm_minimal(&module).unwrap();
    let parsed = parse_xm_minimal(&packed).unwrap();

    assert_eq!(parsed.patterns[0].num_rows, 64);
    assert_eq!(parsed.patterns[0].notes.len(), 64);
}

#[test]
fn test_linear_vs_amiga_frequency() {
    let mut module = create_test_module();

    // Test linear frequency table
    module.linear_frequency_table = true;
    let packed_linear = pack_xm_minimal(&module).unwrap();
    let parsed_linear = parse_xm_minimal(&packed_linear).unwrap();
    assert!(parsed_linear.linear_frequency_table);

    // Test Amiga frequency table
    module.linear_frequency_table = false;
    let packed_amiga = pack_xm_minimal(&module).unwrap();
    let parsed_amiga = parse_xm_minimal(&packed_amiga).unwrap();
    assert!(!parsed_amiga.linear_frequency_table);
}

#[test]
fn test_invalid_data() {
    // Test with data that's too small
    let bad_data = b"BADMAGIC";
    let result = parse_xm_minimal(bad_data);
    assert!(result.is_err());

    // Test with corrupted header (invalid pattern count would cause issues)
    let mut bad_header = vec![0u8; HEADER_SIZE];
    bad_header[0] = 2; // num_channels
    bad_header[1] = 255; // num_patterns low byte (way too many)
    bad_header[2] = 255; // num_patterns high byte
    let result2 = parse_xm_minimal(&bad_header);
    assert!(result2.is_err());
}

#[test]
fn test_truncated_data() {
    let module = create_test_module();
    let packed = pack_xm_minimal(&module).unwrap();

    // Truncate the data
    let truncated = &packed[..10];
    let result = parse_xm_minimal(truncated);
    assert!(result.is_err());
}

#[test]
fn test_pattern_order_not_padded() {
    let mut module = create_test_module();
    module.song_length = 5;
    module.order_table = vec![0, 0, 0, 0, 0];

    let packed = pack_xm_minimal(&module).unwrap();
    let parsed = parse_xm_minimal(&packed).unwrap();

    // Should only store 5 bytes, not 256
    assert_eq!(parsed.order_table.len(), 5);
    assert_eq!(parsed.song_length, 5);
}

#[test]
fn test_round_trip_preserves_data() {
    let module = create_test_module();

    // Do multiple round-trips
    let packed1 = pack_xm_minimal(&module).unwrap();
    let parsed1 = parse_xm_minimal(&packed1).unwrap();
    let packed2 = pack_xm_minimal(&parsed1).unwrap();
    let parsed2 = parse_xm_minimal(&packed2).unwrap();

    // Should be identical after multiple round-trips
    assert_eq!(parsed1.num_channels, parsed2.num_channels);
    assert_eq!(parsed1.num_patterns, parsed2.num_patterns);
    assert_eq!(parsed1.default_speed, parsed2.default_speed);
    assert_eq!(packed1, packed2);
}

#[test]
fn test_max_envelope_points() {
    let mut module = create_test_module();

    // Create envelope with max points
    let mut points = Vec::new();
    for i in 0..MAX_ENVELOPE_POINTS {
        points.push((i as u16 * 10, (64 - i * 5) as u16));
    }

    module.instruments[0].volume_envelope = Some(XmEnvelope {
        points,
        sustain_point: 5,
        loop_start: 2,
        loop_end: 10,
        enabled: true,
        sustain_enabled: true,
        loop_enabled: true,
    });

    let packed = pack_xm_minimal(&module).unwrap();
    let parsed = parse_xm_minimal(&packed).unwrap();

    let env = parsed.instruments[0].volume_envelope.as_ref().unwrap();
    assert_eq!(env.points.len(), MAX_ENVELOPE_POINTS);
    assert_eq!(env.sustain_point, 5);
    assert_eq!(env.loop_start, 2);
    assert_eq!(env.loop_end, 10);
}

#[test]
fn test_vibrato_parameters() {
    let mut module = create_test_module();
    module.instruments[0].vibrato_type = 2;
    module.instruments[0].vibrato_sweep = 16;
    module.instruments[0].vibrato_depth = 32;
    module.instruments[0].vibrato_rate = 8;

    let packed = pack_xm_minimal(&module).unwrap();
    let parsed = parse_xm_minimal(&packed).unwrap();

    let instr = &parsed.instruments[0];
    assert_eq!(instr.vibrato_type, 2);
    assert_eq!(instr.vibrato_sweep, 16);
    assert_eq!(instr.vibrato_depth, 32);
    assert_eq!(instr.vibrato_rate, 8);
}

#[test]
fn test_sample_metadata() {
    let mut module = create_test_module();
    module.instruments[0].sample_loop_start = 1000;
    module.instruments[0].sample_loop_length = 5000;
    module.instruments[0].sample_finetune = -16;
    module.instruments[0].sample_relative_note = 12; // +1 octave
    module.instruments[0].sample_loop_type = 2; // Ping-pong

    let packed = pack_xm_minimal(&module).unwrap();
    let parsed = parse_xm_minimal(&packed).unwrap();

    let instr = &parsed.instruments[0];
    assert_eq!(instr.sample_loop_start, 1000);
    assert_eq!(instr.sample_loop_length, 5000);
    assert_eq!(instr.sample_finetune, -16);
    assert_eq!(instr.sample_relative_note, 12);
    assert_eq!(instr.sample_loop_type, 2);
}

#[test]
fn test_minimal_format_very_compact() {
    // Create a realistic module for testing
    let mut module = create_test_module();

    // Add more complexity to make it realistic
    module.num_patterns = 4;
    for _ in 1..4 {
        module.patterns.push(module.patterns[0].clone());
    }
    module.order_table = vec![0, 1, 2, 3];
    module.song_length = 4;

    // Pack to minimal format
    let minimal = pack_xm_minimal(&module).unwrap();

    println!("Minimal NCXM size: {} bytes", minimal.len());
    println!("  Header: {} bytes", HEADER_SIZE);
    println!("  Pattern order: {} bytes", module.song_length);
    println!(
        "  Patterns + instruments: {} bytes",
        minimal.len() - HEADER_SIZE - module.song_length as usize
    );

    // Verify it's compact (no magic, no padding, no names)
    // Header (16) + order (4) + 4 patterns (64 rows each) + 1 instrument
    // Should be under 1000 bytes for this module
    assert!(
        minimal.len() < 1000,
        "Minimal format should be very compact"
    );

    // Verify it parses correctly
    let parsed = parse_xm_minimal(&minimal).unwrap();
    assert_eq!(parsed.num_channels, module.num_channels);
    assert_eq!(parsed.num_patterns, module.num_patterns);
    assert_eq!(parsed.song_length, module.song_length);
}
