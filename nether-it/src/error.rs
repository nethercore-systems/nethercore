//! Error types for IT module parsing and writing

use std::fmt;
use std::io;

/// Errors that can occur when parsing or writing IT modules
#[derive(Debug)]
pub enum ItError {
    /// File is too small to be a valid IT module
    TooSmall,
    /// Invalid magic bytes (expected "IMPM")
    InvalidMagic,
    /// Unsupported IT format version
    UnsupportedVersion(u16),
    /// Too many channels (max 64)
    TooManyChannels(u8),
    /// Too many patterns (max 256)
    TooManyPatterns(u16),
    /// Too many instruments (max 99)
    TooManyInstruments(u16),
    /// Too many samples (max 99)
    TooManySamples(u16),
    /// Invalid pattern data
    InvalidPattern(u16),
    /// Invalid instrument data
    InvalidInstrument(u16),
    /// Invalid sample data
    InvalidSample(u16),
    /// Invalid envelope data
    InvalidEnvelope,
    /// Unexpected end of file
    UnexpectedEof,
    /// IO error during parsing
    IoError(io::Error),
    /// Invalid compressed sample data
    DecompressionError(String),
    /// Sample data offset is out of bounds
    InvalidSampleOffset(u32),
    /// Pattern offset is out of bounds
    InvalidPatternOffset(u32),
    /// Instrument offset is out of bounds
    InvalidInstrumentOffset(u32),
}

impl fmt::Display for ItError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TooSmall => write!(f, "File too small to be valid IT module"),
            Self::InvalidMagic => write!(f, "Invalid magic bytes (expected 'IMPM')"),
            Self::UnsupportedVersion(v) => write!(f, "Unsupported IT version: 0x{:04X}", v),
            Self::TooManyChannels(n) => write!(f, "Too many channels: {} (max 64)", n),
            Self::TooManyPatterns(n) => write!(f, "Too many patterns: {} (max 256)", n),
            Self::TooManyInstruments(n) => write!(f, "Too many instruments: {} (max 99)", n),
            Self::TooManySamples(n) => write!(f, "Too many samples: {} (max 99)", n),
            Self::InvalidPattern(n) => write!(f, "Invalid pattern data at index {}", n),
            Self::InvalidInstrument(n) => write!(f, "Invalid instrument data at index {}", n),
            Self::InvalidSample(n) => write!(f, "Invalid sample data at index {}", n),
            Self::InvalidEnvelope => write!(f, "Invalid envelope data"),
            Self::UnexpectedEof => write!(f, "Unexpected end of file"),
            Self::IoError(e) => write!(f, "IO error: {}", e),
            Self::DecompressionError(msg) => write!(f, "Decompression error: {}", msg),
            Self::InvalidSampleOffset(off) => write!(f, "Invalid sample offset: 0x{:08X}", off),
            Self::InvalidPatternOffset(off) => write!(f, "Invalid pattern offset: 0x{:08X}", off),
            Self::InvalidInstrumentOffset(off) => {
                write!(f, "Invalid instrument offset: 0x{:08X}", off)
            }
        }
    }
}

impl std::error::Error for ItError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::IoError(e) => Some(e),
            _ => None,
        }
    }
}

impl From<io::Error> for ItError {
    fn from(err: io::Error) -> Self {
        Self::IoError(err)
    }
}
