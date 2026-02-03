//! Error types for IT module parsing and writing

use std::io;
use thiserror::Error;

/// Errors that can occur when parsing or writing IT modules
#[derive(Debug, Error)]
pub enum ItError {
    /// File is too small to be a valid IT module
    #[error("File too small to be valid IT module")]
    TooSmall,

    /// Invalid magic bytes (expected "IMPM")
    #[error("Invalid magic bytes (expected 'IMPM')")]
    InvalidMagic,

    /// Unsupported IT format version
    #[error("Unsupported IT version: 0x{0:04X}")]
    UnsupportedVersion(u16),

    /// Too many channels (max 64)
    #[error("Too many channels: {0} (max 64)")]
    TooManyChannels(u8),

    /// Too many patterns (max 256)
    #[error("Too many patterns: {0} (max 256)")]
    TooManyPatterns(u16),

    /// Too many instruments (max 99)
    #[error("Too many instruments: {0} (max 99)")]
    TooManyInstruments(u16),

    /// Too many samples (max 99)
    #[error("Too many samples: {0} (max 99)")]
    TooManySamples(u16),

    /// Invalid pattern data
    #[error("Invalid pattern data at index {0}")]
    InvalidPattern(u16),

    /// Invalid instrument data
    #[error("Invalid instrument data at index {0}")]
    InvalidInstrument(u16),

    /// Invalid sample data
    #[error("Invalid sample data at index {0}")]
    InvalidSample(u16),

    /// Invalid envelope data
    #[error("Invalid envelope data")]
    InvalidEnvelope,

    /// Unexpected end of file
    #[error("Unexpected end of file")]
    UnexpectedEof,

    /// IO error during parsing
    #[error("IO error: {0}")]
    IoError(#[source] io::Error),

    /// Invalid compressed sample data
    #[error("Decompression error: {0}")]
    DecompressionError(String),

    /// Sample data offset is out of bounds
    #[error("Invalid sample offset: 0x{0:08X}")]
    InvalidSampleOffset(u32),

    /// Pattern offset is out of bounds
    #[error("Invalid pattern offset: 0x{0:08X}")]
    InvalidPatternOffset(u32),

    /// Instrument offset is out of bounds
    #[error("Invalid instrument offset: 0x{0:08X}")]
    InvalidInstrumentOffset(u32),
}

impl From<io::Error> for ItError {
    fn from(err: io::Error) -> Self {
        Self::IoError(err)
    }
}
