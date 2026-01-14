//! Encoding and decoding functions for animation data

use super::types::{BoneTransform, PlatformBoneKeyframe};
use half::f16;

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
