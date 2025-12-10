//! EmberZAnimation binary format (.ewzanim)
//!
//! Z console animation clip format containing sampled bone transforms.
//! POD format - no magic bytes.
//!
//! # Layout
//! ```text
//! Header (16 bytes):
//! 0x00: bone_count u32        - Number of bones per frame
//! 0x04: frame_count u32       - Total number of frames
//! 0x08: frame_rate f32        - Frames per second (e.g., 30.0)
//! 0x0C: flags u32             - Reserved for future use
//!
//! Frame Data (bone_count × 48 bytes per frame):
//! Each frame contains bone_count 3×4 matrices in column-major order.
//! Matrices are stored sequentially: [bone0_frame0, bone1_frame0, ..., bone0_frame1, ...]
//! ```
//!
//! Each bone transform is stored as 12 floats (3×4 matrix, column-major):
//! [col0.x, col0.y, col0.z, col1.x, col1.y, col1.z, col2.x, col2.y, col2.z, tx, ty, tz]

/// EmberZAnimation header (16 bytes)
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct EmberZAnimationHeader {
    /// Number of bones per frame
    pub bone_count: u32,
    /// Total number of frames in the animation
    pub frame_count: u32,
    /// Frames per second (playback rate)
    pub frame_rate: f32,
    /// Reserved flags for future use
    pub flags: u32,
}

impl EmberZAnimationHeader {
    pub const SIZE: usize = 16;

    pub fn new(bone_count: u32, frame_count: u32, frame_rate: f32) -> Self {
        Self {
            bone_count,
            frame_count,
            frame_rate,
            flags: 0,
        }
    }

    /// Write header to bytes
    pub fn to_bytes(&self) -> [u8; Self::SIZE] {
        let mut bytes = [0u8; Self::SIZE];
        bytes[0..4].copy_from_slice(&self.bone_count.to_le_bytes());
        bytes[4..8].copy_from_slice(&self.frame_count.to_le_bytes());
        bytes[8..12].copy_from_slice(&self.frame_rate.to_le_bytes());
        bytes[12..16].copy_from_slice(&self.flags.to_le_bytes());
        bytes
    }

    /// Read header from bytes
    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.len() < Self::SIZE {
            return None;
        }
        Some(Self {
            bone_count: u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]),
            frame_count: u32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]),
            frame_rate: f32::from_le_bytes([bytes[8], bytes[9], bytes[10], bytes[11]]),
            flags: u32::from_le_bytes([bytes[12], bytes[13], bytes[14], bytes[15]]),
        })
    }

    /// Calculate total animation duration in seconds
    pub fn duration(&self) -> f32 {
        if self.frame_rate > 0.0 {
            self.frame_count as f32 / self.frame_rate
        } else {
            0.0
        }
    }
}

/// Size of one bone transform matrix in bytes (12 floats × 4 bytes = 48)
pub const BONE_TRANSFORM_SIZE: usize = 48;
