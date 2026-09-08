// Authored tests build defaults incrementally to isolate each control.
#![allow(clippy::field_reassign_with_default)]
//! Tests for IT file writer

use super::*;
use crate::{IT_MAGIC, ItInstrument, ItSample, parse_it};

#[test]
fn instrument_pan_disable_bit_is_opposite_to_sample_pan_enable() {
    for pan in [None, Some(0), Some(32), Some(64)] {
        let mut writer = ItWriter::new("pan flags");
        writer.add_instrument(ItInstrument {
            default_pan: pan,
            ..Default::default()
        });
        writer.add_sample(
            ItSample {
                default_pan: pan,
                ..Default::default()
            },
            &[1000],
        );
        let mut data = writer.write();
        let instrument = data.windows(4).position(|x| x == b"IMPI").unwrap();
        let sample = data.windows(4).position(|x| x == b"IMPS").unwrap();
        assert_eq!(data[instrument + 25], pan.unwrap_or(32 | 0x80));
        assert_eq!(data[sample + 47], pan.map(|p| p | 0x80).unwrap_or(0));
        for raw in 0..=255u8 {
            data[instrument + 25] = raw;
            let parsed = parse_it(&data).unwrap();
            let value = raw & 127;
            let expected = if raw & 128 != 0 {
                None
            } else {
                Some(if value <= 64 { value } else { 32 })
            };
            assert_eq!(parsed.instruments[0].default_pan, expected);
            assert_eq!(parsed.samples[0].default_pan, pan);
        }
    }
}

#[test]
fn disabled_envelopes_keep_nodes_through_source_and_ncit() {
    use crate::{ItEnvelope, ItEnvelopeFlags, pack_ncit, parse_ncit};
    let envelope = ItEnvelope {
        points: vec![(0, 32), (10, 16)],
        flags: ItEnvelopeFlags::LOOP,
        loop_end: 1,
        ..Default::default()
    };
    let mut writer = ItWriter::new("disabled envelopes");
    writer.add_instrument(ItInstrument {
        volume_envelope: Some(envelope.clone()),
        panning_envelope: Some(envelope.clone()),
        pitch_envelope: Some(envelope.clone()),
        ..Default::default()
    });
    let parsed = parse_it(&writer.write()).unwrap();
    let packed = parse_ncit(&pack_ncit(&parsed)).unwrap();
    for module in [parsed, packed] {
        let instrument = &module.instruments[0];
        for env in [
            &instrument.volume_envelope,
            &instrument.panning_envelope,
            &instrument.pitch_envelope,
        ] {
            let env = env
                .as_ref()
                .expect("disabled envelope must retain authored nodes");
            assert_eq!(env.points, envelope.points);
            assert_eq!(env.flags, envelope.flags);
            assert_eq!(env.loop_end, 1);
        }
    }
}

#[test]
fn writer_sample_presence_matches_payload() {
    for preexisting in [ItSampleFlags::empty(), ItSampleFlags::HAS_DATA] {
        let mut writer = ItWriter::new("sample presence");
        writer.add_sample(
            ItSample {
                flags: preexisting,
                ..Default::default()
            },
            &[],
        );
        writer.add_sample(
            ItSample {
                flags: preexisting,
                ..Default::default()
            },
            &[0],
        );
        let parsed = parse_it(&writer.write()).unwrap();
        assert!(!parsed.samples[0].flags.contains(ItSampleFlags::HAS_DATA));
        assert!(parsed.samples[1].flags.contains(ItSampleFlags::HAS_DATA));
    }
}

#[test]
fn test_write_empty_module() {
    let mut writer = ItWriter::new("Test Song");
    writer.set_channels(4);
    writer.set_speed(6);
    writer.set_tempo(125);

    // Add a simple pattern
    let pat = writer.add_pattern(64);
    writer.set_orders(&[pat]);

    let data = writer.write();

    // Verify magic
    assert_eq!(&data[0..4], IT_MAGIC);

    // Try to parse it back
    let result = parse_it(&data);
    assert!(
        result.is_ok(),
        "Failed to parse written IT: {:?}",
        result.err()
    );

    let module = result.unwrap();
    assert_eq!(module.name, "Test Song");
    assert_eq!(module.initial_speed, 6);
    assert_eq!(module.initial_tempo, 125);
}

#[test]
fn test_write_with_instrument() {
    let mut writer = ItWriter::new("Instr Test");
    writer.set_channels(4);
    writer.set_speed(6);
    writer.set_tempo(125);

    // Add an instrument
    let mut instr = ItInstrument::default();
    instr.name = "Kick".to_string();
    writer.add_instrument(instr);

    // Add a sample
    let mut sample = ItSample::default();
    sample.name = "Kick Sample".to_string();
    sample.c5_speed = 22050;
    let audio = vec![0i16; 1000]; // 1000 samples of silence
    writer.add_sample(sample, &audio);

    // Add a pattern and order table
    let pat = writer.add_pattern(64);
    writer.set_orders(&[pat]);

    let data = writer.write();
    let module = parse_it(&data).expect("Failed to parse written IT file");

    assert_eq!(module.num_instruments, 1);
    assert_eq!(module.num_samples, 1);
    assert_eq!(module.instruments[0].name, "Kick");
    assert_eq!(module.samples[0].name, "Kick Sample");
}

#[test]
fn test_pattern_writer_uses_one_based_channel_markers() {
    let mut pattern = ItPattern::empty(1, 64);
    pattern.notes[0][0] = ItNote {
        note: 60,
        ..ItNote::default()
    };
    pattern.notes[0][63] = ItNote {
        note: 61,
        ..ItNote::default()
    };

    let packed = pack_pattern(&pattern, 64);

    assert_eq!(packed, vec![0x81, 0x01, 60, 0xc0, 0x01, 61, 0]);
}
