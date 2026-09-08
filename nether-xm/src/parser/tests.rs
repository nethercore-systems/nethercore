//! Tests for XM parser

use super::read::*;
use super::write::*;
use crate::module::{XmNote, XmPattern};
use crate::{XM_MAGIC, XM_VERSION};
use std::io::Cursor;

#[test]
fn extension_containers_distinguish_empty_from_truncated_fields() {
    let base = multisample_xm();
    let original = crate::parse_xm(&base).unwrap();
    for marker in [b"STPM", b"XTPM"] {
        let count = if marker == b"STPM" {
            1
        } else {
            usize::from(original.num_instruments)
        };
        let mut field = b"demo\x04\x00".to_vec();
        // Marker-looking bytes inside a framed payload are opaque, not nested tags.
        for _ in 0..count {
            field.extend_from_slice(b"STPM");
        }
        for prefix in 0..=field.len() {
            let mut data = base.clone();
            data.extend_from_slice(marker);
            data.extend_from_slice(&field[..prefix]);
            if prefix == 0 || prefix == field.len() {
                let parsed = crate::parse_xm(&data).unwrap();
                assert_eq!(parsed.mix_mode, original.mix_mode);
                assert_eq!(parsed.sample_preamp, original.sample_preamp);
                let packed = crate::pack_xm_minimal(&parsed).unwrap();
                let restored = crate::parse_xm_minimal(&packed).unwrap();
                assert_eq!(restored.mix_mode, original.mix_mode);
            } else {
                assert_eq!(
                    crate::parse_xm(&data).unwrap_err(),
                    crate::XmError::UnexpectedEof,
                    "marker={marker:?} prefix={prefix}"
                );
            }
        }
    }
    for suffix in [
        b"STPMXTPMSTPM".as_slice(),
        b"opaque trailing data",
        b"XTPMdemo\x00\x00STPM",
    ] {
        let mut data = base.clone();
        data.extend_from_slice(suffix);
        assert!(crate::parse_xm(&data).is_ok());
    }
}

#[test]
fn source_channel_count_is_validated_before_narrowing() {
    for channels in [0u16, 256, 257, u16::MAX] {
        let mut bytes = multisample_xm();
        bytes[68..70].copy_from_slice(&channels.to_le_bytes());
        assert!(
            crate::parse_xm(&bytes).is_err(),
            "accepted {channels} channels"
        );
    }
}

#[test]
fn pattern_reads_stay_inside_declared_header_and_payload() {
    for (header, size, payload, channels, expected) in [
        (8u32, 0u16, vec![], 1, crate::XmError::InvalidHeaderSize),
        (40, 0, vec![], 1, crate::XmError::UnexpectedEof),
        (9, 4, vec![0x80], 1, crate::XmError::UnexpectedEof),
        (9, 2, vec![0x81, 49, 0x80], 2, crate::XmError::UnexpectedEof),
    ] {
        let mut bytes = header.to_le_bytes().to_vec();
        bytes.extend_from_slice(&[0, 1, 0]);
        bytes.extend_from_slice(&size.to_le_bytes());
        bytes.extend(payload);
        assert_eq!(
            parse_pattern(&mut Cursor::new(bytes.as_slice()), channels).unwrap_err(),
            expected
        );
    }
    // Bytes after a complete payload are not a reason to reject the pattern.
    for header in [9u32, 11] {
        let mut bytes = header.to_le_bytes().to_vec();
        bytes.extend_from_slice(&[0, 1, 0, 1, 0]);
        bytes.resize(header as usize, 0x7f);
        bytes.extend_from_slice(&[0x80, 0x7f]);
        let mut cursor = Cursor::new(bytes.as_slice());
        assert_eq!(parse_pattern(&mut cursor, 1).unwrap().num_rows, 1);
        assert_eq!(cursor.position(), u64::from(header) + 1);
    }
}

#[test]
fn measured_modern_openmpt_mix_survives_pack() {
    let mut data = multisample_xm();
    data[38..58].copy_from_slice(b"OpenMPT 1.32.00.00  ");
    data.extend_from_slice(b"opaque trailing bytes");
    let source = crate::parse_xm(&data).unwrap();
    assert_eq!(source.mix_mode, crate::XmMixMode::Ft2);
    assert!(!source.legacy_retrigger);
    let packed = crate::pack_xm_minimal(&source).unwrap();
    assert_eq!(
        crate::parse_xm_minimal(&packed).unwrap().mix_mode,
        crate::XmMixMode::Ft2
    );
}

#[test]
fn measured_retrigger_semantics_are_independent_of_mixer() {
    for legacy in [false, true] {
        for mode in [None, Some(4u32), Some(5u32)] {
            let mut data = multisample_xm();
            data[38..58].copy_from_slice(b"FastTracker v2.00   ");
            for slot in 0..2 {
                data[336 + 243 + slot * 40 + 18..336 + 243 + slot * 40 + 40].fill(if legacy {
                    0
                } else {
                    b' '
                });
            }
            if let Some(mode) = mode {
                data.extend_from_slice(b"STPM.MMP\x04\x00");
                data.extend_from_slice(&mode.to_le_bytes());
            }
            let expected = legacy || mode == Some(4);
            let parsed = crate::parse_xm(&data).unwrap();
            assert_eq!(parsed.legacy_retrigger, expected);
            let packed = crate::pack_xm_minimal(&parsed).unwrap();
            assert_eq!(packed[13] & 0x40 != 0, expected);
            assert_eq!(
                crate::parse_xm_minimal(&packed).unwrap().legacy_retrigger,
                expected
            );
            let stripped = rebuild_xm_without_samples(&data, &parsed).unwrap();
            assert_eq!(
                crate::parse_xm(&stripped).unwrap().legacy_retrigger,
                expected
            );
            let mut old = packed;
            old[13] &= !0x40;
            assert!(!crate::parse_xm_minimal(&old).unwrap().legacy_retrigger);
        }
    }
}

#[test]
fn measured_empty_instrument_header_mix_survives_pack() {
    for size in [29u32, 30, 33, 260, 262, 263, 264] {
        for tracker in [b"FastTracker v2.00   ", b"MilkyTracker 1.03.00"] {
            let mut data = multisample_xm();
            data[38..58].copy_from_slice(tracker);
            for slot in 0..2 {
                data[336 + 243 + slot * 40 + 18..336 + 243 + slot * 40 + 40].fill(b' ');
            }
            data.truncate(data.len() - 29); // Replace the helper's existing empty instrument.
            let mut empty = vec![0; size as usize];
            empty[..4].copy_from_slice(&size.to_le_bytes());
            data.extend(empty);
            let expected = if tracker == b"FastTracker v2.00   " && !matches!(size, 33 | 263) {
                crate::XmMixMode::Compatible
            } else {
                crate::XmMixMode::Ft2
            };
            let parsed = crate::parse_xm(&data).unwrap();
            assert_eq!(
                parsed.mix_mode, expected,
                "size={size}, tracker={tracker:?}"
            );
            let packed = crate::pack_xm_minimal(&parsed).unwrap();
            assert_eq!(crate::parse_xm_minimal(&packed).unwrap().mix_mode, expected);
        }
    }
}

#[test]
fn measured_tracker_padding_survives_parse_pack_and_strip() {
    for padding in [b' ', 0] {
        for sample_padding in [b' ', 0] {
            let mut data = multisample_xm();
            // Isolate sample padding from the independently measured empty-header mode.
            let empty_start = data.len() - 29;
            data[empty_start..empty_start + 4].copy_from_slice(&33u32.to_le_bytes());
            data.extend_from_slice(&[0; 4]);
            data[38..58].fill(padding);
            data[38..55].copy_from_slice(b"FastTracker v2.00");
            for slot in 0..2 {
                data[336 + 243 + slot * 40 + 18..336 + 243 + slot * 40 + 40].fill(sample_padding);
            }
            data.extend_from_slice(b"opaque trailing bytes");
            let expected = if padding == b' ' {
                if sample_padding == b' ' {
                    crate::XmMixMode::Ft2
                } else {
                    crate::XmMixMode::Legacy
                }
            } else {
                crate::XmMixMode::Compatible
            };
            let preamp = 48;
            let parsed = crate::parse_xm(&data).unwrap();
            assert_eq!((parsed.mix_mode, parsed.sample_preamp), (expected, preamp));
            let packed = crate::pack_xm_minimal(&parsed).unwrap();
            let mut conflicting = packed.clone();
            conflicting[13] |= 0x28;
            assert!(crate::parse_xm_minimal(&conflicting).is_err());
            let mut old = packed.clone();
            old[13] &= !0x28;
            assert_eq!(
                crate::parse_xm_minimal(&old).unwrap().mix_mode,
                crate::XmMixMode::Compatible
            );
            let restored = crate::parse_xm_minimal(&packed).unwrap();
            assert_eq!(
                (restored.mix_mode, restored.sample_preamp),
                (expected, preamp)
            );
            let stripped = rebuild_xm_without_samples(&data, &parsed).unwrap();
            let restored = crate::parse_xm(&stripped).unwrap();
            assert_eq!(
                (restored.mix_mode, restored.sample_preamp),
                (expected, preamp)
            );
        }
    }
}

#[test]
fn explicit_xm_mix_properties_survive_parse_pack_and_strip() {
    for mode in [4u32, 5] {
        for preamp in [0u8, 48, 96, 255] {
            let mut data = multisample_xm();
            let creator = b"FastTracker v2.00";
            data[38..58].fill(b' ');
            data[38..38 + creator.len()].copy_from_slice(creator);
            data.extend_from_slice(b"XTPMdemo\x01\x00\x12\x34STPM.MMP\x04\x00");
            data.extend_from_slice(&mode.to_le_bytes());
            data.extend_from_slice(b".APS\x04\x00");
            data.extend_from_slice(&u32::from(preamp).to_le_bytes());
            let parsed = crate::parse_xm(&data).unwrap();
            let expected = if mode == 4 {
                crate::XmMixMode::Compatible
            } else {
                crate::XmMixMode::Ft2
            };
            assert_eq!((parsed.mix_mode, parsed.sample_preamp), (expected, preamp));
            let packed = crate::pack_xm_minimal(&parsed).unwrap();
            let restored = crate::parse_xm_minimal(&packed).unwrap();
            assert_eq!(
                (restored.mix_mode, restored.sample_preamp),
                (expected, preamp)
            );
            let stripped = rebuild_xm_without_samples(&data, &parsed).unwrap();
            let restored = crate::parse_xm(&stripped).unwrap();
            assert_eq!(
                (restored.mix_mode, restored.sample_preamp),
                (expected, preamp)
            );
            let mut legacy = packed;
            legacy[13] &= !(0x08 | 0x10);
            legacy[14] = 199; // Previously reserved bytes must not invent a gain override.
            let old = crate::parse_xm_minimal(&legacy).unwrap();
            assert_eq!(
                (old.mix_mode, old.sample_preamp),
                (crate::XmMixMode::Compatible, 48)
            );
        }
    }
}

#[test]
fn xm_mix_extensions_reject_explicit_unsupported_values_not_creator_names() {
    for creator in [
        b"OpenMPT 1.32.00.00  ".as_slice(),
        b"ModPlug Tracker     ".as_slice(),
    ] {
        let mut data = multisample_xm();
        data[38..58].fill(b' ');
        data[38..38 + creator.len()].copy_from_slice(creator);
        data.extend_from_slice(b"opaque trailing information");
        assert!(crate::parse_xm(&data).is_ok());
    }
    for (tag, value) in [(b".MMP", 0u32), (b".APS", 300)] {
        let mut data = multisample_xm();
        data.extend_from_slice(b"STPM");
        data.extend_from_slice(tag);
        data.extend_from_slice(&4u16.to_le_bytes());
        data.extend_from_slice(&value.to_le_bytes());
        assert_eq!(
            crate::parse_xm(&data).unwrap_err(),
            crate::XmError::UnsupportedMixMetadata
        );
        data.pop();
        assert_eq!(
            crate::parse_xm(&data).unwrap_err(),
            crate::XmError::UnexpectedEof
        );
    }
}

fn multisample_xm() -> Vec<u8> {
    fn fixed<const N: usize>(out: &mut Vec<u8>, value: &str) {
        let mut bytes = [0u8; N];
        let value = value.as_bytes();
        bytes[..value.len().min(N)].copy_from_slice(&value[..value.len().min(N)]);
        out.extend_from_slice(&bytes);
    }

    let mut xm = Vec::new();
    xm.extend_from_slice(XM_MAGIC);
    fixed::<20>(&mut xm, "multi-source");
    xm.push(0x1a);
    fixed::<20>(&mut xm, "nether-xm tests");
    xm.extend_from_slice(&XM_VERSION.to_le_bytes());
    xm.extend_from_slice(&276u32.to_le_bytes());
    xm.extend_from_slice(&1u16.to_le_bytes());
    xm.extend_from_slice(&0u16.to_le_bytes());
    xm.extend_from_slice(&1u16.to_le_bytes());
    xm.extend_from_slice(&0u16.to_le_bytes());
    xm.extend_from_slice(&2u16.to_le_bytes());
    xm.extend_from_slice(&0u16.to_le_bytes());
    xm.extend_from_slice(&6u16.to_le_bytes());
    xm.extend_from_slice(&125u16.to_le_bytes());
    xm.extend_from_slice(&[0u8; 256]);

    xm.extend_from_slice(&243u32.to_le_bytes());
    fixed::<22>(&mut xm, "multi");
    xm.push(0);
    xm.extend_from_slice(&2u16.to_le_bytes());
    xm.extend_from_slice(&40u32.to_le_bytes());
    let mut map = [0u8; 96];
    map[0] = 1;
    map[1] = 0;
    xm.extend_from_slice(&map);
    xm.extend_from_slice(&[0u8; 48 + 48 + 2 + 6 + 2 + 4 + 2 + 2]);

    for (length, loop_start, loop_length, volume, finetune, kind, pan, relative_note) in [
        (0u32, 0u32, 0u32, 11u8, -3i8, 0u8, 22u8, 4i8),
        (3u32, 1u32, 2u32, 33u8, -5i8, 1u8, 44u8, 7i8),
    ] {
        xm.extend_from_slice(&length.to_le_bytes());
        xm.extend_from_slice(&loop_start.to_le_bytes());
        xm.extend_from_slice(&loop_length.to_le_bytes());
        xm.extend_from_slice(&[volume, finetune as u8, kind, pan, relative_note as u8, 0]);
        fixed::<22>(&mut xm, "sample");
    }
    xm.extend_from_slice(&[1, 2, 3]);

    xm.extend_from_slice(&29u32.to_le_bytes());
    fixed::<22>(&mut xm, "following");
    xm.extend_from_slice(&[0, 0, 0]);
    xm
}

fn multisample_xm_with_16bit_sample() -> Vec<u8> {
    let mut xm = multisample_xm();
    let second_sample_header = 336 + 243 + 40;
    xm[second_sample_header..second_sample_header + 4].copy_from_slice(&6u32.to_le_bytes());
    xm[second_sample_header + 4..second_sample_header + 8].copy_from_slice(&2u32.to_le_bytes());
    xm[second_sample_header + 8..second_sample_header + 12].copy_from_slice(&4u32.to_le_bytes());
    xm[second_sample_header + 14] = 0x11;
    xm.splice(336 + 243 + 80 + 3..336 + 243 + 80 + 3, [4, 5, 6]);
    xm
}

#[test]
fn multisample_parse_and_extract_preserve_local_ordinals_and_metadata() {
    let xm = multisample_xm();
    let module = crate::parse_xm(&xm).unwrap();
    let instrument = &module.instruments[0];
    assert_eq!(instrument.sample_map.len(), 96);
    assert_eq!(&instrument.sample_map[..2], &[1, 0]);
    assert_eq!(instrument.samples.len(), 2);
    assert_eq!(instrument.samples[0].volume, 11);
    assert_eq!(instrument.samples[0].pan, 22);
    assert_eq!(instrument.samples[0].finetune, -3);
    assert_eq!(instrument.samples[0].relative_note, 4);
    assert_eq!(instrument.samples[1].volume, 33);
    assert_eq!(instrument.samples[1].pan, 44);
    assert_eq!(instrument.samples[1].loop_start, 1);
    assert_eq!(instrument.samples[1].loop_length, 2);
    assert_eq!(module.instruments[1].name, "following");

    let extracted = crate::extract_samples(&xm).unwrap();
    assert_eq!(extracted.len(), 1);
    assert_eq!(extracted[0].instrument_index, 0);
    assert_eq!(extracted[0].sample_index, 1);
    assert_eq!(extracted[0].volume, 33);
    assert_eq!(extracted[0].pan, 44);
    assert_eq!(extracted[0].finetune, -5);
    assert_eq!(extracted[0].relative_note, 7);
    assert_eq!(extracted[0].data, vec![256, 768, 1536]);
}

#[test]
fn stereo_samples_decode_planar_delta_streams_into_frames() {
    let payload_offset = 336 + 243 + 80;
    for (kind, payload, expected) in [
        (0x20, vec![10, 1, 246, 255], vec![2560, -2560, 2816, -2816]),
        (
            0x30,
            [1000i16, 100, -1000, -100]
                .into_iter()
                .flat_map(i16::to_le_bytes)
                .collect(),
            vec![1000, -1000, 1100, -1100],
        ),
    ] {
        let mut xm = multisample_xm();
        let header = 336 + 243 + 40;
        xm[header..header + 4].copy_from_slice(&(payload.len() as u32).to_le_bytes());
        xm[header + 4..header + 8].copy_from_slice(&0u32.to_le_bytes());
        xm[header + 8..header + 12].copy_from_slice(&(payload.len() as u32).to_le_bytes());
        xm[header + 14] = kind;
        xm.splice(payload_offset..payload_offset + 3, payload);

        let module = crate::parse_xm(&xm).unwrap();
        let sample = &module.instruments[0].samples[1];
        assert!(sample.is_stereo);
        assert_eq!(sample.loop_length, 2);
        let extracted = crate::extract_samples(&xm).unwrap();
        assert!(extracted[0].is_stereo);
        assert_eq!(extracted[0].data, expected);
    }
}

#[test]
fn multisample_strip_preserves_keymap_sample_metadata_and_16bit_loops() {
    let xm = multisample_xm_with_16bit_sample();
    let before = crate::parse_xm(&xm).unwrap();
    let stripped = crate::strip_xm_samples(&xm).unwrap();
    let after = crate::parse_xm(&stripped).unwrap();

    assert_eq!(
        after.instruments[0].sample_map,
        before.instruments[0].sample_map
    );
    for (before, after) in before.instruments[0]
        .samples
        .iter()
        .zip(after.instruments[0].samples.iter())
    {
        assert_eq!(after.source_sample_bytes, Some(0));
        assert_eq!(after.volume, before.volume);
        assert_eq!(after.pan, before.pan);
        assert_eq!(after.finetune, before.finetune);
        assert_eq!(after.relative_note, before.relative_note);
        assert_eq!(after.loop_start, before.loop_start);
        assert_eq!(after.loop_length, before.loop_length);
        assert_eq!(after.loop_type, before.loop_type);
    }
    assert_eq!(after.instruments[1].name, before.instruments[1].name);
}

#[test]
fn multisample_keymap_rejects_out_of_range_local_slot() {
    let mut xm = multisample_xm();
    xm[336 + 33] = 2;
    assert!(crate::parse_xm(&xm).is_err());
}

#[test]
fn sample_loop_bytes_become_sample_positions() {
    for (kind, start, length) in [(1, 3, 7), (0x11, 1, 4)] {
        let mut bytes = vec![0u8; 243];
        bytes[..4].copy_from_slice(&243u32.to_le_bytes());
        bytes[27..29].copy_from_slice(&1u16.to_le_bytes());
        bytes[29..33].copy_from_slice(&40u32.to_le_bytes());
        let mut sample = [0u8; 40];
        sample[..4].copy_from_slice(&12u32.to_le_bytes());
        sample[4..8].copy_from_slice(&3u32.to_le_bytes());
        sample[8..12].copy_from_slice(&7u32.to_le_bytes());
        sample[14] = kind;
        bytes.extend_from_slice(&sample);
        bytes.extend_from_slice(&[0; 12]);
        let mut cursor = Cursor::new(bytes.as_slice());
        let instrument = parse_instrument(&mut cursor).unwrap();
        assert_eq!(
            (instrument.sample_loop_start, instrument.sample_loop_length),
            (start, length)
        );
        assert_eq!(cursor.position(), bytes.len() as u64);
    }
}

#[test]
fn short_instrument_headers_are_errors_not_underflow() {
    for size in 0u32..4 {
        let bytes = size.to_le_bytes();
        assert!(parse_instrument(&mut Cursor::new(bytes.as_slice())).is_err());
    }
}

#[test]
fn multisample_headers_precede_all_pcm_and_preserve_next_instrument() {
    let mut bytes = vec![0u8; 243];
    bytes[..4].copy_from_slice(&243u32.to_le_bytes());
    bytes[27..29].copy_from_slice(&2u16.to_le_bytes());
    bytes[29..33].copy_from_slice(&40u32.to_le_bytes());
    for len in [2u32, 3] {
        let mut header = [0u8; 40];
        header[..4].copy_from_slice(&len.to_le_bytes());
        bytes.extend_from_slice(&header);
    }
    bytes.extend_from_slice(&[1, 2, 3, 4, 5]);
    let next = bytes.len();
    bytes.extend_from_slice(&29u32.to_le_bytes());
    bytes.extend_from_slice(b"following instrument  ");
    bytes.extend_from_slice(&[0, 0, 0]);
    let mut cursor = Cursor::new(bytes.as_slice());
    let instrument = parse_instrument(&mut cursor).unwrap();
    assert_eq!(instrument.num_samples, 2);
    assert_eq!(cursor.position(), next as u64);
    assert_eq!(
        parse_instrument(&mut cursor).unwrap().name,
        "following instrument"
    );
}

#[test]
fn extended_headers_preserve_pan_and_next_instrument() {
    let mut bytes = vec![0u8; 263];
    bytes[..4].copy_from_slice(&263u32.to_le_bytes());
    bytes[27..29].copy_from_slice(&1u16.to_le_bytes());
    bytes[29..33].copy_from_slice(&48u32.to_le_bytes());
    bytes[243..].fill(0xaa);
    let mut sample = [0u8; 48];
    sample[12] = 32;
    sample[15] = 255;
    sample[40..].fill(0xbb);
    bytes.extend_from_slice(&sample);
    let end = bytes.len();
    bytes.extend_from_slice(&29u32.to_le_bytes());
    bytes.extend_from_slice(&[0; 25]);
    let mut cursor = Cursor::new(bytes.as_slice());
    let instrument = parse_instrument(&mut cursor).unwrap();
    assert_eq!(instrument.sample_default_pan, Some(255));
    assert_eq!(instrument.sample_default_volume, Some(32));
    assert_eq!(cursor.position(), end as u64);
    assert_eq!(parse_instrument(&mut cursor).unwrap().num_samples, 0);
}

#[test]
fn single_sample_headers_preserve_bounded_volume() {
    for volume in [0, 32, 64] {
        let mut bytes = vec![0u8; 243];
        bytes[..4].copy_from_slice(&243u32.to_le_bytes());
        bytes[27..29].copy_from_slice(&1u16.to_le_bytes());
        bytes[29..33].copy_from_slice(&40u32.to_le_bytes());
        let mut sample_header = [0u8; 40];
        sample_header[12] = volume;
        bytes.extend_from_slice(&sample_header);

        let instrument = parse_instrument(&mut Cursor::new(bytes.as_slice())).unwrap();
        assert_eq!(instrument.sample_default_volume, Some(volume));
    }
}

#[test]
fn single_sample_headers_preserve_full_default_pan() {
    for pan in [0, 128, 255] {
        let mut bytes = vec![0u8; 243];
        bytes[..4].copy_from_slice(&243u32.to_le_bytes());
        bytes[27..29].copy_from_slice(&1u16.to_le_bytes());
        bytes[29..33].copy_from_slice(&40u32.to_le_bytes());
        let mut sample_header = [0u8; 40];
        sample_header[12] = 64;
        sample_header[15] = pan;
        bytes.extend_from_slice(&sample_header);

        let instrument = parse_instrument(&mut Cursor::new(bytes.as_slice())).unwrap();
        assert_eq!(instrument.sample_default_pan, Some(pan));
    }
}

#[test]
fn single_sample_volume_above_xm_bound_is_rejected() {
    let mut bytes = vec![0u8; 243];
    bytes[..4].copy_from_slice(&243u32.to_le_bytes());
    bytes[27..29].copy_from_slice(&1u16.to_le_bytes());
    bytes[29..33].copy_from_slice(&40u32.to_le_bytes());
    let mut sample_header = [0u8; 40];
    sample_header[12] = 65;
    bytes.extend_from_slice(&sample_header);

    assert!(matches!(
        parse_instrument(&mut Cursor::new(bytes.as_slice())),
        Err(crate::error::XmError::InvalidSampleVolume(65))
    ));
}
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
            let packed_size =
                u16::from_le_bytes([xm[header_start + 7], xm[header_start + 8]]) as usize;
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
