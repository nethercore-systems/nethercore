//! Animation header structure and operations

use super::PLATFORM_BONE_KEYFRAME_SIZE;

/// NetherZXMesh header (4 bytes)
///
/// Note: Not packed - we use explicit byte serialization.
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct NetherZXAnimationHeader {
    /// Number of bones per frame (max 255)
    pub bone_count: u8,
    /// Reserved flags (must be 0)
    pub flags: u8,
    /// Total number of frames in the animation
    pub frame_count: u16,
}

impl NetherZXAnimationHeader {
    pub const SIZE: usize = 4;

    pub fn new(bone_count: u8, frame_count: u16) -> Self {
        Self {
            bone_count,
            flags: 0,
            frame_count,
        }
    }

    /// Write header to bytes
    pub fn to_bytes(&self) -> [u8; Self::SIZE] {
        let mut bytes = [0u8; Self::SIZE];
        bytes[0] = self.bone_count;
        bytes[1] = self.flags;
        bytes[2..4].copy_from_slice(&self.frame_count.to_le_bytes());
        bytes
    }

    /// Read header from bytes
    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.len() < Self::SIZE {
            return None;
        }
        Some(Self {
            bone_count: bytes[0],
            flags: bytes[1],
            frame_count: u16::from_le_bytes([bytes[2], bytes[3]]),
        })
    }

    /// Validate header
    pub fn validate(&self) -> bool {
        self.bone_count > 0 && self.frame_count > 0 && self.flags == 0
    }

    /// Calculate expected data size (excluding header)
    pub fn data_size(&self) -> usize {
        self.frame_count as usize * self.bone_count as usize * PLATFORM_BONE_KEYFRAME_SIZE
    }

    /// Calculate total file size (header + data)
    pub fn file_size(&self) -> usize {
        Self::SIZE + self.data_size()
    }
}
