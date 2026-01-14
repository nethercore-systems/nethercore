//! Tests for the parser module

#[cfg(test)]
mod tests {
    use crate::error::ItError;
    use crate::parser::parse_it;

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
}
