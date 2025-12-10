//! EmberZTexture binary format (.embertex)
//!
//! Z console texture format. Always RGBA8.
//! POD format - no magic bytes.
//!
//! # Layout
//! ```text
//! 0x00: width u32
//! 0x04: height u32
//! 0x08: pixel_data (width * height * 4 bytes, RGBA8)
//! ```

/// EmberZTexture header (8 bytes)
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct EmberZTextureHeader {
    pub width: u32,
    pub height: u32,
}

impl EmberZTextureHeader {
    pub const SIZE: usize = 8;

    pub fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }

    /// Calculate pixel data size (always RGBA8 = 4 bytes per pixel)
    pub fn pixel_size(&self) -> usize {
        self.width as usize * self.height as usize * 4
    }

    /// Write header to bytes
    pub fn to_bytes(&self) -> [u8; Self::SIZE] {
        let mut bytes = [0u8; Self::SIZE];
        bytes[0..4].copy_from_slice(&self.width.to_le_bytes());
        bytes[4..8].copy_from_slice(&self.height.to_le_bytes());
        bytes
    }

    /// Read header from bytes
    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.len() < Self::SIZE {
            return None;
        }
        Some(Self {
            width: u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]),
            height: u32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]),
        })
    }
}
