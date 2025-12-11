//! EmberZSound binary format (.embersnd)
//!
//! Z console audio format. Always 22050Hz mono PCM16.
//! POD format - no magic bytes.
//!
//! # Layout
//! ```text
//! 0x00: sample_count u32
//! 0x04: samples (sample_count * 2 bytes, PCM16)
//! ```

/// Z console sample rate (fixed)
pub const SAMPLE_RATE: u32 = 22050;

/// EmberZSound header (4 bytes)
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct EmberZSoundHeader {
    pub sample_count: u32,
}

impl EmberZSoundHeader {
    pub const SIZE: usize = 4;

    pub fn new(sample_count: u32) -> Self {
        Self { sample_count }
    }

    /// Calculate sample data size (always PCM16 = 2 bytes per sample)
    pub fn data_size(&self) -> usize {
        self.sample_count as usize * 2
    }

    /// Write header to bytes
    pub fn to_bytes(&self) -> [u8; Self::SIZE] {
        self.sample_count.to_le_bytes()
    }

    /// Read header from bytes
    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.len() < Self::SIZE {
            return None;
        }
        Some(Self {
            sample_count: u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]),
        })
    }
}
