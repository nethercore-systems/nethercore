//! Tests for XM parser

use super::read::*;
use super::write::*;
use crate::module::{XmNote, XmPattern};
use crate::{XM_MAGIC, XM_VERSION};
use std::io::Cursor;

#[test]
fn test_read_string() {
    assert_eq!(read_string(b"Hello\0World"), "Hello");
    assert_eq!(read_string(b"No null"), "No null");
    assert_eq!(read_string(b"Trailing   "), "Trailing");
    assert_eq!(read_string(b""), "");
}

#[test]
fn test_parse_invalid_magic() {
    let data = b"Not an XM file at all!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!";
    let result = parse_xm(data);
    assert!(matches!(result, Err(crate::error::XmError::InvalidMagic)));
}

#[test]
fn test_parse_too_small() {
    let data = b"Extended Module: test";
    let result = parse_xm(data);
    assert!(matches!(result, Err(crate::error::XmError::TooSmall)));
}

#[test]
fn test_unpack_note_packed() {
    // Test packed note with all fields present
    let data = [
        0b10011111u8, // All fields present
        0x31,         // Note C-4
        0x01,         // Instrument 1
        0x40,         // Volume 64
        0x0F,         // Effect F (set speed)
        0x06,         // Param 6
    ];
    let mut cursor = Cursor::new(&data[..]);
    let note = unpack_note(&mut cursor).unwrap();

    assert_eq!(note.note, 0x31);
    assert_eq!(note.instrument, 0x01);
    assert_eq!(note.volume, 0x40);
    assert_eq!(note.effect, 0x0F);
    assert_eq!(note.effect_param, 0x06);
}

#[test]
fn test_unpack_note_packed_partial() {
    // Test packed note with only note and effect
    let data = [
        0b10001001u8, // Note and effect present
        0x31,         // Note C-4
        0x0F,         // Effect F
    ];
    let mut cursor = Cursor::new(&data[..]);
    let note = unpack_note(&mut cursor).unwrap();

    assert_eq!(note.note, 0x31);
    assert_eq!(note.instrument, 0);
    assert_eq!(note.volume, 0);
    assert_eq!(note.effect, 0x0F);
    assert_eq!(note.effect_param, 0);
}

#[test]
fn test_unpack_note_unpacked() {
    // Test unpacked note (first byte < 0x80)
    let data = [
        0x31, // Note C-4 (not packed because < 0x80)
        0x01, // Instrument 1
        0x40, // Volume
        0x00, // Effect
        0x00, // Param
    ];
    let mut cursor = Cursor::new(&data[..]);
    let note = unpack_note(&mut cursor).unwrap();

    assert_eq!(note.note, 0x31);
    assert_eq!(note.instrument, 0x01);
    assert_eq!(note.volume, 0x40);
    assert_eq!(note.effect, 0x00);
    assert_eq!(note.effect_param, 0x00);
}

/// Load demo.xm for testing
fn load_demo_xm() -> Option<Vec<u8>> {
    fn write_fixed_str<const N: usize>(out: &mut Vec<u8>, s: &str) {
        let mut buf = [0u8; N];
        let bytes = s.as_bytes();
        let copy_len = bytes.len().min(N);
        buf[..copy_len].copy_from_slice(&bytes[..copy_len]);
        out.extend_from_slice(&buf);
    }

    // Keep tests self-contained: synthesize a small, valid XM with one instrument
    // and a small amount of embedded sample data.
    let num_channels: u8 = 2;
    let num_patterns: u16 = 2;
    let num_instruments: u16 = 1;
    let song_length: u16 = 2;

    // Pattern 0: one note, rest empty
    let pattern0 = XmPattern {
        num_rows: 4,
        notes: vec![
            vec![
                XmNote {
                    note: 0x31, // C-4
                    instrument: 1,
                    volume: 64,
                    effect: 0,
                    effect_param: 0,
                },
                XmNote::default(),
            ],
            vec![XmNote::default(), XmNote::default()],
            vec![XmNote::default(), XmNote::default()],
            vec![XmNote::default(), XmNote::default()],
        ],
    };

    // Pattern 1: all empty
    let pattern1 = XmPattern {
        num_rows: 4,
        notes: vec![
            vec![XmNote::default(), XmNote::default()],
            vec![XmNote::default(), XmNote::default()],
            vec![XmNote::default(), XmNote::default()],
            vec![XmNote::default(), XmNote::default()],
        ],
    };

    let patterns = [pattern0, pattern1];
    let packed_patterns: Vec<Vec<u8>> = patterns
        .iter()
        .map(|p| pack_pattern_data(p, num_channels))
        .collect();

    let mut out = Vec::new();

    // ========== XM Header ==========
    out.extend_from_slice(XM_MAGIC); // Magic (17)
    write_fixed_str::<20>(&mut out, "nether-xm demo"); // Module name (20)
    out.push(0x1A); // 0x1A marker (1)
    write_fixed_str::<20>(&mut out, "nether-xm tests"); // Tracker name (20)
    out.extend_from_slice(&XM_VERSION.to_le_bytes()); // Version (2)
    out.extend_from_slice(&276u32.to_le_bytes()); // Header size (4)
    out.extend_from_slice(&song_length.to_le_bytes()); // Song length (2)
    out.extend_from_slice(&0u16.to_le_bytes()); // Restart position (2)
    out.extend_from_slice(&(num_channels as u16).to_le_bytes()); // Channels (2)
    out.extend_from_slice(&num_patterns.to_le_bytes()); // Patterns (2)
    out.extend_from_slice(&num_instruments.to_le_bytes()); // Instruments (2)
    out.extend_from_slice(&1u16.to_le_bytes()); // Flags (2) - linear frequency table
    out.extend_from_slice(&6u16.to_le_bytes()); // Default speed (2)
    out.extend_from_slice(&125u16.to_le_bytes()); // Default BPM (2)

    // Pattern order table (256)
    for i in 0..256 {
        out.push(if i < song_length as usize { i as u8 } else { 0 });
    }

    // ========== Patterns ==========
    for (pattern, packed) in patterns.iter().zip(packed_patterns.iter()) {
        out.extend_from_slice(&9u32.to_le_bytes()); // pattern header length
        out.push(0); // packing type
        out.extend_from_slice(&pattern.num_rows.to_le_bytes());
        out.extend_from_slice(&(packed.len() as u16).to_le_bytes());
        out.extend_from_slice(packed);
    }

    // ========== Instrument (1 sample) ==========
    // Standard instrument header with samples is 243 bytes.
    out.extend_from_slice(&243u32.to_le_bytes());
    write_fixed_str::<22>(&mut out, "DemoInstr");
    out.push(0); // instrument type
    out.extend_from_slice(&1u16.to_le_bytes()); // num_samples

    // Sample header size (always 40)
    out.extend_from_slice(&40u32.to_le_bytes());

    // Note -> sample map (96 bytes)
    out.extend_from_slice(&[0u8; 96]);

    // Volume envelope points (48 bytes) + panning envelope points (48 bytes)
    out.extend_from_slice(&[0u8; 48]);
    out.extend_from_slice(&[0u8; 48]);

    // Envelope point counts
    out.push(0); // num vol points
    out.push(0); // num pan points

    // Envelope sustain/loop points (6 bytes)
    out.extend_from_slice(&[0u8; 6]);

    // Envelope types
    out.push(0); // vol_type
    out.push(0); // pan_type

    // Vibrato params (4 bytes)
    out.extend_from_slice(&[0u8; 4]);

    // Volume fadeout (2 bytes)
    out.extend_from_slice(&256u16.to_le_bytes());

    // Reserved (2 bytes)
    out.extend_from_slice(&0u16.to_le_bytes());

    // Sample header (40 bytes)
    let sample_length = 16u32;
    out.extend_from_slice(&sample_length.to_le_bytes()); // length
    out.extend_from_slice(&0u32.to_le_bytes()); // loop start
    out.extend_from_slice(&0u32.to_le_bytes()); // loop length
    out.push(64); // volume
    out.push(0); // finetune
    out.push(0); // type
    out.push(128); // panning
    out.push(0); // relative note
    out.push(0); // reserved
    write_fixed_str::<22>(&mut out, "DemoInstr"); // sample name

    // Sample data (dummy)
    for i in 0..sample_length {
        out.push(i as u8);
    }

    Some(out)
}

#[test]
fn test_load_demo_xm() {
    let xm = load_demo_xm().expect("demo.xm should be available");
    let module = parse_xm(&xm).expect("demo.xm should parse");
    println!(
        "Demo XM: {} instruments, {} patterns",
        module.num_instruments, module.num_patterns
    );
    for (i, pattern) in module.patterns.iter().enumerate().take(2) {
        println!("Pattern {}: {} rows", i, pattern.num_rows);
    }
}

#[test]
fn test_rebuild_demo_xm() {
    let xm = load_demo_xm().expect("demo.xm should be available");
    let before = parse_xm(&xm).expect("demo.xm should parse");

    // Try to rebuild it
    let rebuilt = rebuild_xm_without_samples(&xm, &before).expect("Rebuild should work");

    // Try to parse the rebuilt XM
    let after = parse_xm(&rebuilt).expect("Rebuilt XM should parse");

    // Verify basic metadata preserved
    assert_eq!(after.name, before.name);
    assert_eq!(after.num_channels, before.num_channels);
    assert_eq!(after.num_patterns, before.num_patterns);
    assert_eq!(after.num_instruments, before.num_instruments);
    assert_eq!(after.song_length, before.song_length);
}

#[test]
fn test_strip_xm_samples_removes_data() {
    // Load demo XM file
    let xm_with_samples = load_demo_xm().expect("demo.xm should be available for testing");
    let original_size = xm_with_samples.len();

    // Verify it parses before stripping
    let before = parse_xm(&xm_with_samples).expect("demo.xm should be valid");

    // Strip samples
    let stripped = strip_xm_samples(&xm_with_samples).unwrap();
    let stripped_size = stripped.len();

    // Verify:
    // 1. Stripped file still parses
    let module = parse_xm(&stripped).expect("Stripped XM should parse correctly");

    // 2. Pattern count is preserved
    assert_eq!(module.num_patterns, before.num_patterns);
    assert_eq!(module.patterns.len(), before.patterns.len());

    // 3. Pattern data is preserved (verify row counts match)
    for (i, (orig_pattern, stripped_pattern)) in before
        .patterns
        .iter()
        .zip(module.patterns.iter())
        .enumerate()
    {
        assert_eq!(
            orig_pattern.num_rows, stripped_pattern.num_rows,
            "Pattern {} row count should be preserved",
            i
        );
    }

    // 4. Instrument names preserved (critical for ROM mapping!)
    assert_eq!(module.num_instruments, before.num_instruments);
    for (i, (orig, stripped)) in before
        .instruments
        .iter()
        .zip(module.instruments.iter())
        .enumerate()
    {
        assert_eq!(
            orig.name, stripped.name,
            "Instrument {} name should be preserved",
            i
        );
    }

    // 5. File size should be similar or smaller (packed format keeps it compact)
    // For files with large embedded samples, stripped will be much smaller
    // For minimal files like demo.xm (already small), size should be comparable
    println!(
        "Original: {} bytes, Stripped: {} bytes",
        original_size, stripped_size
    );

    // Stripped file shouldn't be massively larger (allow up to 20% increase for overhead)
    assert!(
        stripped_size <= original_size * 12 / 10,
        "Stripped file ({} bytes) should not be much larger than original ({} bytes)",
        stripped_size,
        original_size
    );
}

#[test]
fn test_strip_xm_maintains_format_compliance() {
    let xm_data = load_demo_xm().expect("demo.xm should be available for testing");
    let stripped = strip_xm_samples(&xm_data).unwrap();

    // Verify XM magic
    assert_eq!(
        &stripped[0..17],
        XM_MAGIC,
        "Stripped XM should maintain magic header"
    );

    // Verify version
    let version = u16::from_le_bytes([stripped[58], stripped[59]]);
    assert_eq!(
        version, XM_VERSION,
        "Stripped XM should maintain version 0x0104"
    );

    // Verify it can be parsed by standard XM parser
    let result = parse_xm(&stripped);
    assert!(
        result.is_ok(),
        "Stripped XM should parse without errors: {:?}",
        result.err()
    );
}

#[test]
fn test_stripped_xm_preserves_metadata() {
    let xm_data = load_demo_xm().expect("demo.xm should be available for testing");
    let before = parse_xm(&xm_data).unwrap();
    let stripped = strip_xm_samples(&xm_data).unwrap();
    let after = parse_xm(&stripped).unwrap();

    // Verify metadata is preserved
    assert_eq!(after.name, before.name);
    assert_eq!(after.num_channels, before.num_channels);
    assert_eq!(after.default_speed, before.default_speed);
    assert_eq!(after.default_bpm, before.default_bpm);
    assert_eq!(after.linear_frequency_table, before.linear_frequency_table);
    assert_eq!(after.song_length, before.song_length);
    assert_eq!(after.restart_position, before.restart_position);
}

#[test]
fn test_rebuild_from_packed_input() {
    // Verify we can read a packed XM (like demo.xm) and rebuild it
    let xm = load_demo_xm().expect("demo.xm should be available");
    let before = parse_xm(&xm).expect("demo.xm should parse");

    // demo.xm uses packed format (verified by small file size)
    let original_size = xm.len();

    // Rebuild it
    let rebuilt = rebuild_xm_without_samples(&xm, &before).expect("Rebuild should work");
    let rebuilt_size = rebuilt.len();

    // Verify it parses
    let after = parse_xm(&rebuilt).expect("Rebuilt XM should parse");

    // Verify data preserved
    assert_eq!(after.num_patterns, before.num_patterns);
    assert_eq!(after.num_instruments, before.num_instruments);

    // Rebuilt should be similar size (both use packed format)
    println!(
        "Packed input: {} bytes → Rebuilt: {} bytes",
        original_size, rebuilt_size
    );
    assert!(
        rebuilt_size <= original_size * 12 / 10,
        "Rebuilt packed format should be compact"
    );
}

#[test]
fn test_rebuild_from_unpacked_input() {
    // Create an XM file with unpacked pattern data to verify we can read it
    let xm = load_demo_xm().expect("demo.xm should be available");
    let module = parse_xm(&xm).expect("demo.xm should parse");

    // Create a manually-built XM with unpacked patterns
    // (This simulates what would happen if someone created an XM with unpacked format)
    let mut unpacked_xm = Vec::new();

    // Write header (copy from original)
    unpacked_xm.extend_from_slice(&xm[0..336]); // Header up to pattern data

    // Write patterns in UNPACKED format (5 bytes per note)
    for pattern in &module.patterns {
        // Pattern header
        unpacked_xm.extend_from_slice(&9u32.to_le_bytes()); // header_length
        unpacked_xm.push(0); // packing_type
        unpacked_xm.extend_from_slice(&pattern.num_rows.to_le_bytes()); // num_rows

        // Calculate unpacked size: rows × channels × 5 bytes
        let unpacked_size = (pattern.num_rows as usize) * (module.num_channels as usize) * 5;
        unpacked_xm.extend_from_slice(&(unpacked_size as u16).to_le_bytes());

        // Write unpacked note data
        for row in &pattern.notes {
            for (ch_idx, note) in row.iter().enumerate() {
                if ch_idx >= module.num_channels as usize {
                    break;
                }
                unpacked_xm.push(note.note);
                unpacked_xm.push(note.instrument);
                unpacked_xm.push(note.volume);
                unpacked_xm.push(note.effect);
                unpacked_xm.push(note.effect_param);
            }
        }
    }

    // Append instrument headers + sample data from the original XM so the file is complete.
    // This keeps the test focused on verifying UNPACKED pattern parsing.
    fn find_instrument_offset(xm: &[u8], num_patterns: u16) -> usize {
        // Header size is at offset 60 (after magic + name + marker + tracker + version).
        let header_size = u32::from_le_bytes([xm[60], xm[61], xm[62], xm[63]]) as usize;
        let mut pos = 60 + header_size;

        for _ in 0..num_patterns {
            let header_start = pos;
            let header_len = u32::from_le_bytes([
                xm[header_start],
                xm[header_start + 1],
                xm[header_start + 2],
                xm[header_start + 3],
            ]) as usize;
            let packed_size = u16::from_le_bytes([xm[header_start + 7], xm[header_start + 8]])
                as usize;
            pos = header_start + header_len + packed_size;
        }

        pos
    }

    let instrument_offset = find_instrument_offset(&xm, module.num_patterns);
    unpacked_xm.extend_from_slice(&xm[instrument_offset..]);

    // Parse the unpacked XM
    let unpacked_module = parse_xm(&unpacked_xm).expect("Unpacked XM should parse");
    let unpacked_size = unpacked_xm.len();

    // Rebuild it (should output packed format)
    let rebuilt =
        rebuild_xm_without_samples(&unpacked_xm, &unpacked_module).expect("Rebuild should work");
    let rebuilt_size = rebuilt.len();

    println!(
        "Unpacked input: {} bytes → Rebuilt (packed): {} bytes",
        unpacked_size, rebuilt_size
    );

    // Rebuilt should be SMALLER (packed format compression)
    assert!(
        rebuilt_size < unpacked_size,
        "Rebuilt should be smaller than unpacked input ({} < {})",
        rebuilt_size,
        unpacked_size
    );
}

#[test]
fn test_pack_pattern_data() {
    // Create a pattern with mixed notes (some with data, some empty)
    let pattern = XmPattern {
        num_rows: 2,
        notes: vec![
            vec![
                XmNote {
                    note: 0x31,
                    instrument: 1,
                    volume: 64,
                    effect: 0,
                    effect_param: 0,
                },
                XmNote::default(), // Empty note
            ],
            vec![XmNote::default(), XmNote::default()], // Two empty notes
        ],
    };

    let packed = pack_pattern_data(&pattern, 2);

    // Verify packed format compression:
    // Row 0, Ch 0: flag (0x87 = note+inst+vol) + note + inst + vol = 4 bytes
    // Row 0, Ch 1: 0x80 (empty) = 1 byte
    // Row 1, Ch 0: 0x80 (empty) = 1 byte
    // Row 1, Ch 1: 0x80 (empty) = 1 byte
    // Total: 7 bytes (vs 20 bytes unpacked!)

    assert_eq!(packed.len(), 7, "Packed format should compress empty notes");

    // First note: flag byte with note+instrument+volume
    assert_eq!(
        packed[0], 0x87,
        "Flag should indicate note(0x01) + instrument(0x02) + volume(0x04) present"
    );
    assert_eq!(packed[1], 0x31, "Note should be C#-1");
    assert_eq!(packed[2], 1, "Instrument should be 1");
    assert_eq!(packed[3], 64, "Volume should be 64");

    // Remaining notes are empty (just 0x80 marker)
    assert_eq!(packed[4], 0x80, "Second note (ch 1) should be empty marker");
    assert_eq!(
        packed[5], 0x80,
        "Third note (row 1, ch 0) should be empty marker"
    );
    assert_eq!(
        packed[6], 0x80,
        "Fourth note (row 1, ch 1) should be empty marker"
    );
}
