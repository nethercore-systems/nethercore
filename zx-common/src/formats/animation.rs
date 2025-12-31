//! NetherZXMesh binary format (.nczxanim)
//!
//! ZX console animation clip format containing sampled bone transforms.
//! POD format with minimal header - no magic bytes.
//!
//! # Layout
//! ```text
//! Header (4 bytes):
//! 0x00: bone_count u8        - Number of bones per frame (max 255)
//! 0x01: flags u8             - Reserved, must be 0
//! 0x02: frame_count u16 LE   - Total number of frames (max 65535)
//!
//! Frame Data (frame_count × bone_count × 16 bytes):
//! Each bone transform is stored in 16 bytes:
//! - rotation: u32 (smallest-three packed quaternion)
//! - position: [u16; 3] (f16 × 3)
//! - scale: [u16; 3] (f16 × 3)
//! ```
//!
//! Frame data is stored sequentially: [frame0_bone0, frame0_bone1, ..., frame1_bone0, ...]

use half::f16;

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

// ============================================================================
// Quaternion Encoding: Smallest-Three
// ============================================================================

/// Encode a quaternion using smallest-three encoding (32 bits)
///
/// Industry-standard encoding used by Unreal, Unity, and ACL.
/// Drops the largest component and reconstructs it from the other three.
///
/// Bit layout: `[a:10][b:10][c:10][idx:2]`
/// - idx identifies which component was dropped (largest)
/// - a, b, c are the three smallest components quantized to 10 bits
///
/// Uses the quantization formula from the spec's test vectors:
/// `round((v * √2 + 1) * 511.5)` for [0, 1023] range
pub fn encode_quat_smallest_three(q: [f32; 4]) -> u32 {
    let [x, y, z, w] = q;

    // 1. Find index of largest absolute component
    let abs_q = [x.abs(), y.abs(), z.abs(), w.abs()];
    let idx = if abs_q[0] > abs_q[1] && abs_q[0] > abs_q[2] && abs_q[0] > abs_q[3] {
        0
    } else if abs_q[1] > abs_q[2] && abs_q[1] > abs_q[3] {
        1
    } else if abs_q[2] > abs_q[3] {
        2
    } else {
        3
    };

    // 2. Ensure largest component is positive (q == -q for rotations)
    let sign = if q[idx] < 0.0 { -1.0 } else { 1.0 };
    let q = [q[0] * sign, q[1] * sign, q[2] * sign, q[3] * sign];

    // 3. Select the 3 smallest components (skip idx)
    let (a, b, c) = match idx {
        0 => (q[1], q[2], q[3]),
        1 => (q[0], q[2], q[3]),
        2 => (q[0], q[1], q[3]),
        _ => (q[0], q[1], q[2]),
    };

    // 4. Quantize: [-1/√2, 1/√2] → [0, 1023] (10 bits)
    //    Formula: round((v * √2 + 1) * 511.5)
    let scale = 511.5;
    let sqrt2 = std::f32::consts::SQRT_2;
    let qa = ((a * sqrt2 + 1.0) * scale).round() as u32;
    let qb = ((b * sqrt2 + 1.0) * scale).round() as u32;
    let qc = ((c * sqrt2 + 1.0) * scale).round() as u32;

    // Clamp to valid range
    let qa = qa.min(1023);
    let qb = qb.min(1023);
    let qc = qc.min(1023);

    // 5. Pack: [a:10][b:10][c:10][idx:2]
    (qa << 22) | (qb << 12) | (qc << 2) | (idx as u32)
}

/// Decode a smallest-three encoded quaternion (32 bits)
///
/// Returns [x, y, z, w] quaternion.
pub fn decode_quat_smallest_three(packed: u32) -> [f32; 4] {
    let idx = (packed & 0x3) as usize;
    let qc = ((packed >> 2) & 0x3FF) as f32;
    let qb = ((packed >> 12) & 0x3FF) as f32;
    let qa = ((packed >> 22) & 0x3FF) as f32;

    // Dequantize: [0, 1023] → [-1/√2, 1/√2]
    let scale = 1.0 / 511.5;
    let sqrt2_inv = 1.0 / std::f32::consts::SQRT_2;
    let a = (qa * scale - 1.0) * sqrt2_inv;
    let b = (qb * scale - 1.0) * sqrt2_inv;
    let c = (qc * scale - 1.0) * sqrt2_inv;

    // Reconstruct largest component: sqrt(1 - a² - b² - c²)
    let largest = (1.0 - a * a - b * b - c * c).max(0.0).sqrt();

    // Rebuild quaternion
    match idx {
        0 => [largest, a, b, c],
        1 => [a, largest, b, c],
        2 => [a, b, largest, c],
        _ => [a, b, c, largest],
    }
}

// ============================================================================
// Half-Float (f16) Conversion
// ============================================================================

/// Convert f32 to f16 bits
#[inline]
pub fn f32_to_f16(value: f32) -> u16 {
    f16::from_f32(value).to_bits()
}

/// Convert f16 bits to f32
#[inline]
pub fn f16_to_f32(bits: u16) -> f32 {
    f16::from_bits(bits).to_f32()
}

// ============================================================================
// Full Encode/Decode Pipeline
// ============================================================================

/// Encode a bone transform to platform format (16 bytes)
pub fn encode_bone_transform(
    rotation: [f32; 4],
    position: [f32; 3],
    scale: [f32; 3],
) -> PlatformBoneKeyframe {
    PlatformBoneKeyframe {
        rotation: encode_quat_smallest_three(rotation),
        position: [
            f32_to_f16(position[0]),
            f32_to_f16(position[1]),
            f32_to_f16(position[2]),
        ],
        scale: [
            f32_to_f16(scale[0]),
            f32_to_f16(scale[1]),
            f32_to_f16(scale[2]),
        ],
    }
}

/// Decode a platform bone keyframe to BoneTransform (40 bytes)
pub fn decode_bone_transform(kf: &PlatformBoneKeyframe) -> BoneTransform {
    BoneTransform {
        rotation: decode_quat_smallest_three(kf.rotation),
        position: [
            f16_to_f32(kf.position[0]),
            f16_to_f32(kf.position[1]),
            f16_to_f32(kf.position[2]),
        ],
        scale: [
            f16_to_f32(kf.scale[0]),
            f16_to_f32(kf.scale[1]),
            f16_to_f32(kf.scale[2]),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ========================================================================
    // Header Tests
    // ========================================================================

    #[test]
    fn test_animation_header_roundtrip() {
        let header = NetherZXAnimationHeader::new(25, 90);
        assert_eq!(header.bone_count, 25);
        assert_eq!(header.frame_count, 90);
        assert_eq!(header.flags, 0);

        let bytes = header.to_bytes();
        assert_eq!(bytes.len(), NetherZXAnimationHeader::SIZE);

        let parsed = NetherZXAnimationHeader::from_bytes(&bytes).unwrap();
        assert_eq!(parsed.bone_count, header.bone_count);
        assert_eq!(parsed.frame_count, header.frame_count);
        assert_eq!(parsed.flags, header.flags);
    }

    #[test]
    fn test_animation_header_size() {
        assert_eq!(NetherZXAnimationHeader::SIZE, 4);
        assert_eq!(PLATFORM_BONE_KEYFRAME_SIZE, 16);
        assert_eq!(BONE_TRANSFORM_SIZE, 40);
    }

    #[test]
    fn test_animation_file_size() {
        // Example from spec: 40 bones, 60 frames
        let header = NetherZXAnimationHeader::new(40, 60);
        // file_size = 4 + (frame_count x bone_count x 16) = 4 + 38400 = 38404
        assert_eq!(header.file_size(), 38404);
    }

    #[test]
    fn test_animation_header_from_short_bytes() {
        let short_bytes = [0u8; 2];
        assert!(NetherZXAnimationHeader::from_bytes(&short_bytes).is_none());
    }

    #[test]
    fn test_header_validation() {
        let valid = NetherZXAnimationHeader::new(10, 100);
        assert!(valid.validate());

        let invalid_bones = NetherZXAnimationHeader::new(0, 100);
        assert!(!invalid_bones.validate());

        let invalid_frames = NetherZXAnimationHeader::new(10, 0);
        assert!(!invalid_frames.validate());
    }

    // ========================================================================
    // Smallest-Three Quaternion Encoding Tests
    // ========================================================================

    #[test]
    fn test_quat_identity_roundtrip() {
        // Identity quaternion: [0, 0, 0, 1]
        let q = [0.0, 0.0, 0.0, 1.0];
        let encoded = encode_quat_smallest_three(q);
        let decoded = decode_quat_smallest_three(encoded);

        // Verify idx is 3 (w is largest)
        assert_eq!(
            encoded & 0x3,
            3,
            "Identity quaternion should drop w (idx=3)"
        );

        // Verify roundtrip (dot product should be close to 1)
        let dot = q[0] * decoded[0] + q[1] * decoded[1] + q[2] * decoded[2] + q[3] * decoded[3];
        assert!(
            dot.abs() > 0.999,
            "Identity roundtrip failed: dot = {}",
            dot
        );
    }

    #[test]
    fn test_quat_90_x_roundtrip() {
        // 90° X rotation (axis-angle): [1, 0, 0, 0]
        let q = [1.0, 0.0, 0.0, 0.0];
        let encoded = encode_quat_smallest_three(q);
        let decoded = decode_quat_smallest_three(encoded);

        // Verify idx is 0 (x is largest)
        assert_eq!(encoded & 0x3, 0, "90° X rotation should drop x (idx=0)");

        // Verify roundtrip
        let dot = q[0] * decoded[0] + q[1] * decoded[1] + q[2] * decoded[2] + q[3] * decoded[3];
        assert!(dot.abs() > 0.999, "90° X roundtrip failed: dot = {}", dot);
    }

    #[test]
    fn test_quat_90_y_roundtrip() {
        // 90° Y rotation: [0, 1, 0, 0]
        let q = [0.0, 1.0, 0.0, 0.0];
        let encoded = encode_quat_smallest_three(q);
        let decoded = decode_quat_smallest_three(encoded);

        // Verify idx is 1 (y is largest)
        assert_eq!(encoded & 0x3, 1, "90° Y rotation should drop y (idx=1)");

        // Verify roundtrip
        let dot = q[0] * decoded[0] + q[1] * decoded[1] + q[2] * decoded[2] + q[3] * decoded[3];
        assert!(dot.abs() > 0.999, "90° Y roundtrip failed: dot = {}", dot);
    }

    #[test]
    fn test_quat_90_z_roundtrip() {
        // 90° Z rotation: [0, 0, 1, 0]
        let q = [0.0, 0.0, 1.0, 0.0];
        let encoded = encode_quat_smallest_three(q);
        let decoded = decode_quat_smallest_three(encoded);

        // Verify idx is 2 (z is largest)
        assert_eq!(encoded & 0x3, 2, "90° Z rotation should drop z (idx=2)");

        // Verify roundtrip
        let dot = q[0] * decoded[0] + q[1] * decoded[1] + q[2] * decoded[2] + q[3] * decoded[3];
        assert!(dot.abs() > 0.999, "90° Z roundtrip failed: dot = {}", dot);
    }

    #[test]
    fn test_quat_120_rotation_roundtrip() {
        // 120° rotation around [1,1,1]: [0.5, 0.5, 0.5, 0.5]
        let q = [0.5, 0.5, 0.5, 0.5];
        let encoded = encode_quat_smallest_three(q);
        let decoded = decode_quat_smallest_three(encoded);

        // Verify roundtrip
        let dot = q[0] * decoded[0] + q[1] * decoded[1] + q[2] * decoded[2] + q[3] * decoded[3];
        assert!(
            dot.abs() > 0.999,
            "120° [1,1,1] roundtrip failed: dot = {}",
            dot
        );
    }

    #[test]
    fn test_quat_half_angle_90_x_roundtrip() {
        // 90° X (half-angle form): [0.707107, 0, 0, 0.707107]
        let sqrt2_inv = std::f32::consts::FRAC_1_SQRT_2;
        let q = [sqrt2_inv, 0.0, 0.0, sqrt2_inv];
        let encoded = encode_quat_smallest_three(q);
        let decoded = decode_quat_smallest_three(encoded);

        // Verify roundtrip
        let dot = q[0] * decoded[0] + q[1] * decoded[1] + q[2] * decoded[2] + q[3] * decoded[3];
        assert!(
            dot.abs() > 0.999,
            "Half-angle 90° X roundtrip failed: dot = {}",
            dot
        );
    }

    #[test]
    fn test_quat_sign_flip_roundtrip() {
        // Sign flip: [-0.5, -0.5, -0.5, 0.5]
        // Should produce same rotation as [0.5, 0.5, 0.5, -0.5]
        let q = [-0.5, -0.5, -0.5, 0.5];
        let encoded = encode_quat_smallest_three(q);
        let decoded = decode_quat_smallest_three(encoded);

        // For q and -q representing same rotation, dot can be positive or negative
        let dot = q[0] * decoded[0] + q[1] * decoded[1] + q[2] * decoded[2] + q[3] * decoded[3];
        assert!(
            dot.abs() > 0.999,
            "Sign flip roundtrip failed: dot = {}",
            dot
        );
    }

    #[test]
    fn test_quat_roundtrip_precision() {
        // Test roundtrip precision for arbitrary quaternion
        let q = [0.270598, 0.0, 0.0, 0.962728]; // ~31.4° X rotation
        let encoded = encode_quat_smallest_three(q);
        let decoded = decode_quat_smallest_three(encoded);

        // Compute dot product (should be > 0.9999 for < 0.1° error)
        let dot = q[0] * decoded[0] + q[1] * decoded[1] + q[2] * decoded[2] + q[3] * decoded[3];
        assert!(
            dot.abs() > 0.9999,
            "Quaternion roundtrip precision failed: dot = {}",
            dot
        );
    }

    // ========================================================================
    // Half-Float (f16) Tests (from spec)
    // ========================================================================

    #[test]
    fn test_f16_zero() {
        assert_eq!(f32_to_f16(0.0), 0x0000);
        assert_eq!(f16_to_f32(0x0000), 0.0);
    }

    #[test]
    fn test_f16_negative_zero() {
        assert_eq!(f32_to_f16(-0.0), 0x8000);
        // Note: -0.0 == 0.0 in Rust
    }

    #[test]
    fn test_f16_one() {
        assert_eq!(f32_to_f16(1.0), 0x3C00);
        assert_eq!(f16_to_f32(0x3C00), 1.0);
    }

    #[test]
    fn test_f16_negative_one() {
        assert_eq!(f32_to_f16(-1.0), 0xBC00);
        assert_eq!(f16_to_f32(0xBC00), -1.0);
    }

    #[test]
    fn test_f16_half() {
        assert_eq!(f32_to_f16(0.5), 0x3800);
        assert_eq!(f16_to_f32(0x3800), 0.5);
    }

    #[test]
    fn test_f16_two() {
        assert_eq!(f32_to_f16(2.0), 0x4000);
        assert_eq!(f16_to_f32(0x4000), 2.0);
    }

    #[test]
    fn test_f16_max_normal() {
        assert_eq!(f32_to_f16(65504.0), 0x7BFF);
        assert_eq!(f16_to_f32(0x7BFF), 65504.0);
    }

    #[test]
    fn test_f16_min_normal() {
        assert_eq!(f32_to_f16(-65504.0), 0xFBFF);
        assert_eq!(f16_to_f32(0xFBFF), -65504.0);
    }

    // ========================================================================
    // Full Roundtrip Tests (from spec)
    // ========================================================================

    #[test]
    fn test_identity_transform_roundtrip() {
        let input = BoneTransform {
            rotation: [0.0, 0.0, 0.0, 1.0],
            position: [0.0, 0.0, 0.0],
            scale: [1.0, 1.0, 1.0],
        };

        let encoded = encode_bone_transform(input.rotation, input.position, input.scale);

        // Verify idx=3 (w is dropped as largest component)
        assert_eq!(
            encoded.rotation & 0x3,
            3,
            "Identity rotation should drop w (idx=3)"
        );
        assert_eq!(
            encoded.position,
            [0x0000, 0x0000, 0x0000],
            "Zero position encoding"
        );
        assert_eq!(
            encoded.scale,
            [0x3C00, 0x3C00, 0x3C00],
            "Unit scale encoding"
        );

        let decoded = decode_bone_transform(&encoded);

        assert!((decoded.rotation[3] - 1.0).abs() < 0.002, "w ≈ 1");
        assert!(
            decoded.position.iter().all(|&v| v.abs() < 0.001),
            "position ≈ 0"
        );
        assert!(
            decoded.scale.iter().all(|&v| (v - 1.0).abs() < 0.001),
            "scale ≈ 1"
        );
    }

    #[test]
    fn test_typical_animation_pose_roundtrip() {
        let input = BoneTransform {
            rotation: [0.270598, 0.0, 0.0, 0.962728], // 31.4° X rotation
            position: [1.5, 2.25, -0.75],
            scale: [1.0, 1.0, 1.0],
        };

        let encoded = encode_bone_transform(input.rotation, input.position, input.scale);
        let decoded = decode_bone_transform(&encoded);

        // Verify rotation (angular error < 0.1°)
        let dot = input.rotation[0] * decoded.rotation[0]
            + input.rotation[1] * decoded.rotation[1]
            + input.rotation[2] * decoded.rotation[2]
            + input.rotation[3] * decoded.rotation[3];
        assert!(
            dot.abs() > 0.9999,
            "Nearly identical rotation: dot = {}",
            dot
        );

        // Verify position (f16 precision)
        assert!((decoded.position[0] - 1.5).abs() < 0.01);
        assert!((decoded.position[1] - 2.25).abs() < 0.01);
        assert!((decoded.position[2] - (-0.75)).abs() < 0.01);

        // Verify scale
        assert!(decoded.scale.iter().all(|&v| (v - 1.0).abs() < 0.001));
    }

    #[test]
    fn test_extreme_values_roundtrip() {
        let input = BoneTransform {
            rotation: [std::f32::consts::FRAC_1_SQRT_2, 0.0, std::f32::consts::FRAC_1_SQRT_2, 0.0], // 180° around [1,0,1]
            position: [1000.0, -500.0, 0.001],
            scale: [0.5, 1.0, 2.5], // Non-uniform scale
        };

        let encoded = encode_bone_transform(input.rotation, input.position, input.scale);
        let decoded = decode_bone_transform(&encoded);

        // Position precision degrades at large values (f16 limitation)
        assert!((decoded.position[0] - 1000.0).abs() < 1.0);
        assert!((decoded.position[1] - (-500.0)).abs() < 0.5);
        assert!((decoded.position[2] - 0.001).abs() < 0.001);

        // Verify non-uniform scale (XYZ)
        assert!((decoded.scale[0] - 0.5).abs() < 0.01);
        assert!((decoded.scale[1] - 1.0).abs() < 0.01);
        assert!((decoded.scale[2] - 2.5).abs() < 0.01);
    }

    #[test]
    fn test_byte_level_verification() {
        // Test encode→bytes→decode roundtrip at byte level
        let input = BoneTransform {
            rotation: [0.0, 0.0, 0.0, 1.0], // Identity
            position: [1.0, 2.0, -2.0],
            scale: [1.0, 1.0, 1.0],
        };

        let encoded = encode_bone_transform(input.rotation, input.position, input.scale);
        let bytes = encoded.to_bytes();
        let parsed = PlatformBoneKeyframe::from_bytes(&bytes);
        let decoded = decode_bone_transform(&parsed);

        // Verify position and scale are exactly preserved (f16 can represent these exactly)
        assert!((decoded.position[0] - 1.0).abs() < 0.001);
        assert!((decoded.position[1] - 2.0).abs() < 0.001);
        assert!((decoded.position[2] - (-2.0)).abs() < 0.001);
        assert!(decoded.scale.iter().all(|&v| (v - 1.0).abs() < 0.001));

        // Verify rotation roundtrip
        let dot = input.rotation[0] * decoded.rotation[0]
            + input.rotation[1] * decoded.rotation[1]
            + input.rotation[2] * decoded.rotation[2]
            + input.rotation[3] * decoded.rotation[3];
        assert!(
            dot.abs() > 0.999,
            "Rotation roundtrip failed: dot = {}",
            dot
        );
    }

    // ========================================================================
    // Edge Case Tests (from spec)
    // ========================================================================

    #[test]
    fn test_single_frame_animation() {
        // Minimum valid animation: 1 bone, 1 frame (20 bytes total)
        // Build it programmatically to avoid hardcoding implementation-specific values
        let header = NetherZXAnimationHeader::new(1, 1);
        let header_bytes = header.to_bytes();

        // Identity transform: no rotation, zero position, unit scale
        let keyframe = encode_bone_transform(
            [0.0, 0.0, 0.0, 1.0], // Identity quaternion
            [0.0, 0.0, 0.0],      // Zero position
            [1.0, 1.0, 1.0],      // Unit scale
        );
        let keyframe_bytes = keyframe.to_bytes();

        // Combine header + keyframe
        let mut data = Vec::with_capacity(20);
        data.extend_from_slice(&header_bytes);
        data.extend_from_slice(&keyframe_bytes);

        assert_eq!(data.len(), 20); // 4 header + 16 data

        // Verify we can parse it back
        let parsed_header = NetherZXAnimationHeader::from_bytes(&data).unwrap();
        assert_eq!(parsed_header.bone_count, 1);
        assert_eq!(parsed_header.frame_count, 1);
        assert!(parsed_header.validate());

        // Verify the keyframe roundtrips correctly
        let parsed_kf = PlatformBoneKeyframe::from_bytes(&data[4..]);
        let decoded = decode_bone_transform(&parsed_kf);
        assert!((decoded.rotation[3] - 1.0).abs() < 0.002, "w ≈ 1");
        assert!(
            decoded.position.iter().all(|&v| v.abs() < 0.001),
            "position ≈ 0"
        );
        assert!(
            decoded.scale.iter().all(|&v| (v - 1.0).abs() < 0.001),
            "scale ≈ 1"
        );
    }

    #[test]
    fn test_max_bone_count() {
        // Maximum: 255 bones
        let header = NetherZXAnimationHeader::new(255, 1);
        assert!(header.validate());
        assert_eq!(header.file_size(), 4 + 255 * 16);
    }

    #[test]
    fn test_max_frame_count() {
        // Maximum: 65535 frames (at 60fps = ~18 minutes)
        let header = NetherZXAnimationHeader::new(1, 65535);
        assert!(header.validate());
        let expected_size = 4 + (65535 * 16);
        assert_eq!(header.file_size(), expected_size);
        assert_eq!(expected_size, 1048564); // ~1MB for single bone
    }

    #[test]
    fn test_platform_keyframe_roundtrip() {
        // Create a keyframe via encode to ensure valid values
        let kf = encode_bone_transform(
            [0.0, 0.0, 0.0, 1.0], // Identity rotation
            [1.0, 2.0, -2.0],     // Position
            [1.0, 1.0, 1.0],      // Unit scale
        );

        let bytes = kf.to_bytes();
        let parsed = PlatformBoneKeyframe::from_bytes(&bytes);

        assert_eq!(parsed.rotation, kf.rotation);
        assert_eq!(parsed.position, kf.position);
        assert_eq!(parsed.scale, kf.scale);
    }
}
