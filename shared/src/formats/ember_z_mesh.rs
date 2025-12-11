//! EmberZMesh binary format (.embermesh)
//!
//! Z console GPU-ready mesh format with packed vertices.
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

// =============================================================================
// Vertex Format Flags
// =============================================================================

/// Vertex format flag: Has UV coordinates
pub const FORMAT_UV: u8 = 1;
/// Vertex format flag: Has per-vertex color
pub const FORMAT_COLOR: u8 = 2;
/// Vertex format flag: Has normals
pub const FORMAT_NORMAL: u8 = 4;
/// Vertex format flag: Has bone indices/weights for skinning
pub const FORMAT_SKINNED: u8 = 8;

/// All format flags combined
pub const FORMAT_ALL: u8 = FORMAT_UV | FORMAT_COLOR | FORMAT_NORMAL | FORMAT_SKINNED;

/// Number of vertex format permutations (16: 0-15)
pub const VERTEX_FORMAT_COUNT: usize = 16;

/// Calculate vertex stride in bytes for packed format (GPU-ready)
///
/// This is the stride used in `.ewzmesh` files and GPU vertex buffers.
/// Format values are 0-15 (combination of FORMAT_* flags).
#[inline]
pub const fn vertex_stride_packed(format: u8) -> usize {
    // Position: Float16x4 (8 bytes)
    let mut stride = 8;

    if format & FORMAT_UV != 0 {
        stride += 4; // Unorm16x2
    }
    if format & FORMAT_COLOR != 0 {
        stride += 4; // Unorm8x4
    }
    if format & FORMAT_NORMAL != 0 {
        stride += 4; // Octahedral u32
    }
    if format & FORMAT_SKINNED != 0 {
        stride += 8; // Bone indices (u8x4) + weights (unorm8x4)
    }

    stride
}

// =============================================================================
// Mesh Header
// =============================================================================

/// EmberZMesh header (12 bytes)
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct EmberZMeshHeader {
    pub vertex_count: u32,
    pub index_count: u32,
    pub format: u8,
    pub _padding: [u8; 3],
}

impl EmberZMeshHeader {
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
