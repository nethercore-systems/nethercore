//! NetherZSkeleton binary format (.ewzskel)
//!
//! ZX console skeleton format containing inverse bind matrices for skeletal animation.
//! POD format - no magic bytes.
//!
//! # Layout
//! ```text
//! 0x00: bone_count u32
//! 0x04: reserved u32 (future: bone hierarchy)
//! 0x08: inverse_bind_matrices (bone_count × 48 bytes, 3×4 column-major)
//! ```
//!
//! Each inverse bind matrix is stored as 12 floats in column-major order:
//! [col0.x, col0.y, col0.z, col1.x, col1.y, col1.z, col2.x, col2.y, col2.z, tx, ty, tz]

/// NetherZSkeleton header (8 bytes)
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct NetherZSkeletonHeader {
    /// Number of bones in the skeleton
    pub bone_count: u32,
    /// Reserved for future use (bone hierarchy, etc.)
    pub reserved: u32,
}

impl NetherZSkeletonHeader {
    pub const SIZE: usize = 8;

    pub fn new(bone_count: u32) -> Self {
        Self {
            bone_count,
            reserved: 0,
        }
    }

    /// Write header to bytes
    pub fn to_bytes(&self) -> [u8; Self::SIZE] {
        let mut bytes = [0u8; Self::SIZE];
        bytes[0..4].copy_from_slice(&self.bone_count.to_le_bytes());
        bytes[4..8].copy_from_slice(&self.reserved.to_le_bytes());
        bytes
    }

    /// Read header from bytes
    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.len() < Self::SIZE {
            return None;
        }
        Some(Self {
            bone_count: u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]),
            reserved: u32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]),
        })
    }
}

/// Size of one inverse bind matrix in bytes (12 floats × 4 bytes = 48)
pub const INVERSE_BIND_MATRIX_SIZE: usize = 48;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_skeleton_header_roundtrip() {
        let header = NetherZSkeletonHeader::new(42);
        assert_eq!(header.bone_count, 42);
        assert_eq!(header.reserved, 0);

        let bytes = header.to_bytes();
        assert_eq!(bytes.len(), NetherZSkeletonHeader::SIZE);

        let parsed = NetherZSkeletonHeader::from_bytes(&bytes).unwrap();
        assert_eq!(parsed.bone_count, header.bone_count);
        assert_eq!(parsed.reserved, header.reserved);
    }

    #[test]
    fn test_skeleton_header_size() {
        assert_eq!(NetherZSkeletonHeader::SIZE, 8);
        assert_eq!(INVERSE_BIND_MATRIX_SIZE, 48);
    }

    #[test]
    fn test_skeleton_header_from_short_bytes() {
        let short_bytes = [0u8; 4];
        assert!(NetherZSkeletonHeader::from_bytes(&short_bytes).is_none());
    }
}
