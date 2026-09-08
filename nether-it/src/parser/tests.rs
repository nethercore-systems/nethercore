//! Tests for the parser module

#[cfg(test)]
mod tests {
    use crate::error::ItError;
    use crate::parser::parse_it;
    use crate::{ItNote, ItWriter};

    #[test]
    fn test_parse_invalid_magic() {
        // Need at least 192 bytes for the header check to pass size validation
        let mut data = vec![0u8; 192];
        data[..4].copy_from_slice(b"XXXX"); // Invalid magic
        let result = parse_it(&data);
        assert!(matches!(result, Err(ItError::InvalidMagic)));
    }

    #[test]
    fn test_parse_too_small() {
        let data = b"IMPM test";
        let result = parse_it(data);
        assert!(matches!(result, Err(ItError::TooSmall)));
    }

    #[test]
    fn sparse_channel_settings_keep_muted_channel_effects() {
        let mut writer = ItWriter::new("Sparse");
        writer.set_channels(3);
        writer.module.channel_pan[1] = 128; // muted, but still an active IT channel
        let pattern = writer.add_pattern(1);
        writer.set_note(pattern, 0, 1, ItNote::default().with_effect(2, 0x34));
        writer.set_orders(&[pattern]);

        let parsed = parse_it(&writer.write()).unwrap();

        assert_eq!(parsed.num_channels, 3);
        assert_eq!(parsed.patterns[0].notes[0][1].effect, 2);
        assert_eq!(parsed.patterns[0].notes[0][1].effect_param, 0x34);
    }
}
