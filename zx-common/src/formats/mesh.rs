//! NetherZXMesh binary format (.nczmesh)
//!
//! ZX console GPU-ready mesh format with packed vertices.
//! POD format - no magic bytes.
//!
//! # Layout
//! ```text
//! 0x00: vertex_count u32
//! 0x04: index_count u32
//! 0x08: format u8 (vertex format flags)
//! 0x09: padding (3 bytes)
//! 0x0C: vertex_data (vertex_count * stride)
//! var:  index_data (index_count * 2 bytes), if indexed
//! ```
//!
//! For vertex format constants and stride calculation, see `z_common::packing`.

/// NetherZXMesh header (12 bytes)
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct NetherZXMeshHeader {
    pub vertex_count: u32,
    pub index_count: u32,
    pub format: u8,
    pub _padding: [u8; 3],
}

impl NetherZXMeshHeader {
    pub const SIZE: usize = 12;

    pub fn new(vertex_count: u32, index_count: u32, format: u8) -> Self {
        Self {
            vertex_count,
            index_count,
            format,
            _padding: [0; 3],
        }
    }

    /// Write header to bytes
    pub fn to_bytes(&self) -> [u8; Self::SIZE] {
        let mut bytes = [0u8; Self::SIZE];
        bytes[0..4].copy_from_slice(&self.vertex_count.to_le_bytes());
        bytes[4..8].copy_from_slice(&self.index_count.to_le_bytes());
        bytes[8] = self.format;
        // padding bytes stay 0
        bytes
    }

    /// Read header from bytes
    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.len() < Self::SIZE {
            return None;
        }
        Some(Self {
            vertex_count: u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]),
            index_count: u32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]),
            format: bytes[8],
            _padding: [0; 3],
        })
    }
}
