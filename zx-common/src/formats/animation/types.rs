//! Animation data types

/// Size of one bone transform in the platform format (16 bytes)
pub const PLATFORM_BONE_KEYFRAME_SIZE: usize = 16;

/// Size of decoded BoneTransform struct (40 bytes)
pub const BONE_TRANSFORM_SIZE: usize = 40;

/// Platform bone keyframe format (16 bytes per bone)
///
/// Compressed format stored in ROM:
/// - rotation: u32 (smallest-three packed quaternion)
/// - position: [u16; 3] (f16 × 3)
/// - scale: [u16; 3] (f16 × 3)
///
/// Note: Not packed - we use explicit byte serialization.
#[derive(Debug, Clone, Copy, Default)]
#[repr(C)]
pub struct PlatformBoneKeyframe {
    /// Smallest-three packed quaternion
    pub rotation: u32,
    /// Position as f16 × 3
    pub position: [u16; 3],
    /// Scale as f16 × 3
    pub scale: [u16; 3],
}

impl PlatformBoneKeyframe {
    /// Parse from raw bytes (16 bytes)
    pub fn from_bytes(bytes: &[u8]) -> Self {
        debug_assert!(bytes.len() >= 16);
        Self {
            rotation: u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]),
            position: [
                u16::from_le_bytes([bytes[4], bytes[5]]),
                u16::from_le_bytes([bytes[6], bytes[7]]),
                u16::from_le_bytes([bytes[8], bytes[9]]),
            ],
            scale: [
                u16::from_le_bytes([bytes[10], bytes[11]]),
                u16::from_le_bytes([bytes[12], bytes[13]]),
                u16::from_le_bytes([bytes[14], bytes[15]]),
            ],
        }
    }

    /// Write to raw bytes (16 bytes)
    pub fn to_bytes(&self) -> [u8; 16] {
        let mut bytes = [0u8; 16];
        bytes[0..4].copy_from_slice(&self.rotation.to_le_bytes());
        bytes[4..6].copy_from_slice(&self.position[0].to_le_bytes());
        bytes[6..8].copy_from_slice(&self.position[1].to_le_bytes());
        bytes[8..10].copy_from_slice(&self.position[2].to_le_bytes());
        bytes[10..12].copy_from_slice(&self.scale[0].to_le_bytes());
        bytes[12..14].copy_from_slice(&self.scale[1].to_le_bytes());
        bytes[14..16].copy_from_slice(&self.scale[2].to_le_bytes());
        bytes
    }
}

/// Decoded bone transform (40 bytes)
///
/// Ready-to-use format for WASM memory after decoding from platform format.
#[derive(Debug, Clone, Copy, Default)]
#[repr(C)]
pub struct BoneTransform {
    /// Quaternion rotation [x, y, z, w]
    pub rotation: [f32; 4],
    /// Translation position
    pub position: [f32; 3],
    /// Non-uniform scale [x, y, z]
    pub scale: [f32; 3],
}

impl BoneTransform {
    /// Identity transform (no rotation, no translation, unit scale)
    pub const IDENTITY: Self = Self {
        rotation: [0.0, 0.0, 0.0, 1.0],
        position: [0.0, 0.0, 0.0],
        scale: [1.0, 1.0, 1.0],
    };

    /// Size in bytes (40)
    pub const SIZE: usize = 40;

    /// Write to raw bytes (40 bytes)
    pub fn to_bytes(&self) -> [u8; Self::SIZE] {
        let mut bytes = [0u8; Self::SIZE];
        // Rotation (16 bytes)
        for (i, &f) in self.rotation.iter().enumerate() {
            bytes[i * 4..(i + 1) * 4].copy_from_slice(&f.to_le_bytes());
        }
        // Position (12 bytes)
        for (i, &f) in self.position.iter().enumerate() {
            bytes[16 + i * 4..16 + (i + 1) * 4].copy_from_slice(&f.to_le_bytes());
        }
        // Scale (12 bytes)
        for (i, &f) in self.scale.iter().enumerate() {
            bytes[28 + i * 4..28 + (i + 1) * 4].copy_from_slice(&f.to_le_bytes());
        }
        bytes
    }
}
