//! Helper functions for reading binary data

use std::io::{Cursor, Read};

use crate::error::ItError;

/// Read a single byte
pub(crate) fn read_u8(cursor: &mut Cursor<&[u8]>) -> Result<u8, ItError> {
    let mut buf = [0u8; 1];
    cursor
        .read_exact(&mut buf)
        .map_err(|_| ItError::UnexpectedEof)?;
    Ok(buf[0])
}

/// Read a 16-bit little-endian integer
pub(crate) fn read_u16(cursor: &mut Cursor<&[u8]>) -> Result<u16, ItError> {
    let mut buf = [0u8; 2];
    cursor
        .read_exact(&mut buf)
        .map_err(|_| ItError::UnexpectedEof)?;
    Ok(u16::from_le_bytes(buf))
}

/// Read a 32-bit little-endian integer
pub(crate) fn read_u32(cursor: &mut Cursor<&[u8]>) -> Result<u32, ItError> {
    let mut buf = [0u8; 4];
    cursor
        .read_exact(&mut buf)
        .map_err(|_| ItError::UnexpectedEof)?;
    Ok(u32::from_le_bytes(buf))
}

/// Read a null-terminated or fixed-length string
pub(crate) fn read_string(bytes: &[u8]) -> String {
    // Find null terminator or end of slice
    let len = bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len());
    // Trim trailing spaces and convert
    String::from_utf8_lossy(&bytes[..len])
        .trim_end()
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_string() {
        assert_eq!(read_string(b"Hello\0World"), "Hello");
        assert_eq!(read_string(b"No null"), "No null");
        assert_eq!(read_string(b"Trailing   "), "Trailing");
        assert_eq!(read_string(b""), "");
    }
}
