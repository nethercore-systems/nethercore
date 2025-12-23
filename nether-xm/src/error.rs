//! XM parsing error types

use core::fmt;

/// XM parsing error types
#[derive(Debug, Clone, PartialEq)]
pub enum XmError {
    /// File too small to contain header
    TooSmall,
    /// Invalid magic string (not "Extended Module: ")
    InvalidMagic,
    /// Unsupported XM version
    UnsupportedVersion(u16),
    /// Invalid header size
    InvalidHeaderSize,
    /// Too many channels (> 32)
    TooManyChannels(u8),
    /// Pattern count exceeds maximum
    TooManyPatterns(u16),
    /// Invalid pattern data
    InvalidPattern(u16),
    /// Instrument parsing error
    InvalidInstrument(u16),
    /// Unexpected end of file
    UnexpectedEof,
    /// IO error during parsing
    IoError(String),
}

impl fmt::Display for XmError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            XmError::TooSmall => write!(f, "File too small to contain XM header"),
            XmError::InvalidMagic => write!(f, "Invalid XM magic string"),
            XmError::UnsupportedVersion(v) => write!(f, "Unsupported XM version: 0x{:04X}", v),
            XmError::InvalidHeaderSize => write!(f, "Invalid XM header size"),
            XmError::TooManyChannels(n) => {
                write!(f, "Too many channels: {} (max {})", n, crate::MAX_CHANNELS)
            }
            XmError::TooManyPatterns(n) => {
                write!(f, "Too many patterns: {} (max {})", n, crate::MAX_PATTERNS)
            }
            XmError::InvalidPattern(n) => write!(f, "Invalid pattern data at index {}", n),
            XmError::InvalidInstrument(n) => write!(f, "Invalid instrument at index {}", n),
            XmError::UnexpectedEof => write!(f, "Unexpected end of file"),
            XmError::IoError(e) => write!(f, "IO error: {}", e),
        }
    }
}

impl std::error::Error for XmError {}

impl From<std::io::Error> for XmError {
    fn from(e: std::io::Error) -> Self {
        XmError::IoError(e.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        assert_eq!(
            XmError::TooSmall.to_string(),
            "File too small to contain XM header"
        );
        assert_eq!(XmError::InvalidMagic.to_string(), "Invalid XM magic string");
        assert_eq!(
            XmError::UnsupportedVersion(0x0103).to_string(),
            "Unsupported XM version: 0x0103"
        );
        assert_eq!(
            XmError::TooManyChannels(64).to_string(),
            "Too many channels: 64 (max 32)"
        );
    }
}
