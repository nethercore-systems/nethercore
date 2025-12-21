//! NetherZTexture binary format (.embertex)
//!
//! ZX console texture format. Supports RGBA8 (Mode 0) or BC7 (Modes 1-3).
//! POD format - no magic bytes.
//!
//! # Layout
//! ```text
//! 0x00: width u16 (max 65535)
//! 0x02: height u16 (max 65535)
//! 0x04: pixel_data (RGBA8: width × height × 4 bytes)
//!       or block_data (BC7: (width/4) × (height/4) × 16 bytes)
//! ```

/// NetherZTexture header (4 bytes)
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct NetherZTextureHeader {
    pub width: u16,
    pub height: u16,
}

impl NetherZTextureHeader {
    pub const SIZE: usize = 4;

    pub fn new(width: u16, height: u16) -> Self {
        Self { width, height }
    }

    /// Create header from u32 dimensions (truncates to u16)
    pub fn from_u32(width: u32, height: u32) -> Self {
        Self {
            width: width as u16,
            height: height as u16,
        }
    }

    /// Calculate RGBA8 pixel data size (4 bytes per pixel)
    pub fn rgba8_size(&self) -> usize {
        self.width as usize * self.height as usize * 4
    }

    /// Calculate BC7 compressed data size (16 bytes per 4×4 block)
    pub fn bc7_size(&self) -> usize {
        let blocks_x = (self.width as usize).div_ceil(4);
        let blocks_y = (self.height as usize).div_ceil(4);
        blocks_x * blocks_y * 16
    }

    /// Write header to bytes
    pub fn to_bytes(&self) -> [u8; Self::SIZE] {
        let mut bytes = [0u8; Self::SIZE];
        bytes[0..2].copy_from_slice(&self.width.to_le_bytes());
        bytes[2..4].copy_from_slice(&self.height.to_le_bytes());
        bytes
    }

    /// Read header from bytes
    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.len() < Self::SIZE {
            return None;
        }
        Some(Self {
            width: u16::from_le_bytes([bytes[0], bytes[1]]),
            height: u16::from_le_bytes([bytes[2], bytes[3]]),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_header_size() {
        assert_eq!(NetherZTextureHeader::SIZE, 4);
    }

    #[test]
    fn test_header_parsing() {
        // 64×32 texture header
        let data = [
            0x40, 0x00, // width = 64 (little-endian u16)
            0x20, 0x00, // height = 32 (little-endian u16)
        ];

        let header = NetherZTextureHeader::from_bytes(&data).unwrap();
        assert_eq!(header.width, 64);
        assert_eq!(header.height, 32);
    }

    #[test]
    fn test_header_max_dimensions() {
        // Max supported: 65535×65535
        let data = [0xFF, 0xFF, 0xFF, 0xFF];
        let header = NetherZTextureHeader::from_bytes(&data).unwrap();

        assert_eq!(header.width, 65535);
        assert_eq!(header.height, 65535);
    }

    #[test]
    fn test_header_roundtrip() {
        let header = NetherZTextureHeader::new(128, 256);
        let bytes = header.to_bytes();
        let parsed = NetherZTextureHeader::from_bytes(&bytes).unwrap();

        assert_eq!(parsed.width, 128);
        assert_eq!(parsed.height, 256);
    }

    #[test]
    fn test_rgba8_size() {
        let header = NetherZTextureHeader::new(64, 64);
        assert_eq!(header.rgba8_size(), 64 * 64 * 4); // 16384 bytes
    }

    #[test]
    fn test_bc7_size() {
        let header = NetherZTextureHeader::new(64, 64);
        // 64/4 = 16 blocks per row, 16 blocks per column
        // 16 × 16 × 16 bytes = 4096 bytes
        assert_eq!(header.bc7_size(), 4096);
    }

    #[test]
    fn test_bc7_size_non_aligned() {
        // 30×30 → rounds up to 8×8 blocks = 64 blocks × 16 bytes = 1024 bytes
        let header = NetherZTextureHeader::new(30, 30);
        assert_eq!(header.bc7_size(), 8 * 8 * 16); // 1024 bytes
    }

    #[test]
    fn test_bc7_compression_ratio() {
        let header = NetherZTextureHeader::new(64, 64);
        let rgba8 = header.rgba8_size();
        let bc7 = header.bc7_size();
        assert_eq!(rgba8 / bc7, 4); // 4× compression ratio
    }
}
