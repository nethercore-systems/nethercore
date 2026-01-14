//! I/O helper functions for reading and writing binary data

use std::io::{Cursor, Read, Write};

use crate::error::XmError;

/// Write a u16 in little-endian format
pub(crate) fn write_u16<W: Write>(output: &mut W, val: u16) {
    output.write_all(&val.to_le_bytes()).unwrap();
}

/// Write a u32 in little-endian format
pub(crate) fn write_u32<W: Write>(output: &mut W, val: u32) {
    output.write_all(&val.to_le_bytes()).unwrap();
}

/// Read a single byte
pub(crate) fn read_u8(cursor: &mut Cursor<&[u8]>) -> Result<u8, XmError> {
    let mut buf = [0u8; 1];
    cursor
        .read_exact(&mut buf)
        .map_err(|_| XmError::UnexpectedEof)?;
    Ok(buf[0])
}

/// Read a u16 in little-endian format
pub(crate) fn read_u16(cursor: &mut Cursor<&[u8]>) -> Result<u16, XmError> {
    let mut buf = [0u8; 2];
    cursor
        .read_exact(&mut buf)
        .map_err(|_| XmError::UnexpectedEof)?;
    Ok(u16::from_le_bytes(buf))
}

/// Read a u32 in little-endian format
pub(crate) fn read_u32(cursor: &mut Cursor<&[u8]>) -> Result<u32, XmError> {
    let mut buf = [0u8; 4];
    cursor
        .read_exact(&mut buf)
        .map_err(|_| XmError::UnexpectedEof)?;
    Ok(u32::from_le_bytes(buf))
}
