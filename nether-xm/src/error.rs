//! XM parsing error types

use thiserror::Error;

/// XM parsing error types
#[derive(Debug, Clone, PartialEq, Error)]
pub enum XmError {
    #[error("Explicit XM mixing property is unsupported; refusing to discard it")]
    UnsupportedMixMetadata,
    /// File too small to contain header
    #[error("File too small to contain XM header")]
    TooSmall,

    /// Invalid magic string (not "Extended Module: ")
    #[error("Invalid XM magic string")]
    InvalidMagic,

    /// Unsupported XM version
    #[error("Unsupported XM version: 0x{0:04X}")]
    UnsupportedVersion(u16),

    /// Invalid header size
    #[error("Invalid XM header size")]
    InvalidHeaderSize,

    /// Too many channels (> 32)
    #[error("Too many channels: {0} (max 32)")]
    TooManyChannels(u8),

    /// Pattern count exceeds maximum
    #[error("Too many patterns: {0} (max 256)")]
    TooManyPatterns(u16),

    /// Invalid pattern data
    #[error("Invalid pattern data at index {0}")]
    InvalidPattern(u16),

    /// Instrument parsing error
    #[error("Invalid instrument at index {0}")]
    InvalidInstrument(u16),

    /// XM sample volume is outside the format's 0-64 range
    #[error("Invalid sample volume: {0} (max 64)")]
    InvalidSampleVolume(u8),

    /// A packed module cannot mix single-sample instruments with and without pan metadata.
    #[error("Missing sample default pan in a module with pan metadata")]
    MissingSampleDefaultPan,

    /// A minimal NCXM header flag changes the layout but is not supported.
    #[error("Unsupported NCXM flags: 0x{0:02X}")]
    UnsupportedMinimalFlags(u8),

    /// Unexpected end of file
    #[error("Unexpected end of file")]
    UnexpectedEof,

    /// IO error during parsing
    #[error("IO error: {0}")]
    IoError(String),
}

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
