//! Vertex data packing utilities
//!
//! Provides functions to convert f32 vertex data to packed GPU formats:
//! - f32 → f16 (IEEE 754 half-float)
//! - f32 → snorm16 (signed normalized, -1.0 to 1.0)
//! - f32 → unorm8 (unsigned normalized, 0.0 to 1.0)
//!
//! Used by both `ember-export` (asset pipeline) and `emberware-zx` (runtime).

use bytemuck::cast_slice;
use half::f16;

// ============================================================================
// Vertex Format Constants
// ============================================================================

/// Vertex format flag: Has UV coordinates (2 floats)
pub const FORMAT_UV: u8 = 1;
/// Vertex format flag: Has per-vertex color (RGB, 3 floats)
pub const FORMAT_COLOR: u8 = 2;
/// Vertex format flag: Has normals (3 floats)
pub const FORMAT_NORMAL: u8 = 4;
/// Vertex format flag: Has bone indices/weights for skinning
pub const FORMAT_SKINNED: u8 = 8;

/// Calculate vertex stride in bytes for unpacked f32 format
#[inline]
pub const fn vertex_stride(format: u8) -> u32 {
    let mut stride = 12; // Position: Float32x3

    if format & FORMAT_UV != 0 {
        stride += 8; // UV: Float32x2
    }
    if format & FORMAT_COLOR != 0 {
        stride += 12; // Color: Float32x3
    }
    if format & FORMAT_NORMAL != 0 {
        stride += 12; // Normal: Float32x3
    }
    if format & FORMAT_SKINNED != 0 {
        stride += 20; // Bone indices (4 u8) + weights (4 f32)
    }

    stride
}

/// Calculate vertex stride in bytes for packed GPU format
#[inline]
pub const fn vertex_stride_packed(format: u8) -> u32 {
    let mut stride = 8; // Position: Float16x4

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

// ============================================================================
// Basic Conversion Functions
// ============================================================================

/// Convert f32 to signed normalized 16-bit integer (snorm16)
///
/// Maps f32 range [-1.0, 1.0] to i16 range [-32767, 32767].
#[inline]
pub fn f32_to_snorm16(value: f32) -> i16 {
    let clamped = value.clamp(-1.0, 1.0);
    (clamped * 32767.0) as i16
}

/// Convert f32 to unsigned normalized 8-bit integer (unorm8)
///
/// Maps f32 range [0.0, 1.0] to u8 range [0, 255].
#[inline]
pub fn f32_to_unorm8(value: f32) -> u8 {
    let clamped = value.clamp(0.0, 1.0);
    (clamped * 255.0) as u8
}

// ============================================================================
// Position Packing
// ============================================================================

/// Pack a 3D position (f32x3) to Float16x4 format (with w=1.0 padding)
#[inline]
pub fn pack_position_f16(x: f32, y: f32, z: f32) -> [f16; 4] {
    [
        f16::from_f32(x),
        f16::from_f32(y),
        f16::from_f32(z),
        f16::from_f32(1.0),
    ]
}

// ============================================================================
// UV Packing
// ============================================================================

/// Pack a 2D UV coordinate (f32x2) to Float16x2 format
#[inline]
pub fn pack_uv_f16(u: f32, v: f32) -> [f16; 2] {
    [f16::from_f32(u), f16::from_f32(v)]
}

/// Pack a 2D UV coordinate (f32x2) to Unorm16x2 format
///
/// Better precision than f16 for values in [0.0, 1.0] range.
#[inline]
pub fn pack_uv_unorm16(u: f32, v: f32) -> [u16; 2] {
    [
        (u.clamp(0.0, 1.0) * 65535.0) as u16,
        (v.clamp(0.0, 1.0) * 65535.0) as u16,
    ]
}

// ============================================================================
// Normal Packing
// ============================================================================

/// Pack a 3D normal (f32x3) to Snorm16x4 format (with w=0.0 padding)
#[inline]
pub fn pack_normal_snorm16(nx: f32, ny: f32, nz: f32) -> [i16; 4] {
    [
        f32_to_snorm16(nx),
        f32_to_snorm16(ny),
        f32_to_snorm16(nz),
        0,
    ]
}

/// Encode normalized direction to octahedral coordinates in [-1, 1]²
#[inline]
pub fn encode_octahedral(dir: glam::Vec3) -> (f32, f32) {
    let dir = dir.normalize_or_zero();

    let l1_norm = dir.x.abs() + dir.y.abs() + dir.z.abs();
    if l1_norm == 0.0 {
        return (0.0, 0.0);
    }

    let mut u = dir.x / l1_norm;
    let mut v = dir.y / l1_norm;

    if dir.z < 0.0 {
        let u_abs = u.abs();
        let v_abs = v.abs();
        u = (1.0 - v_abs) * if u >= 0.0 { 1.0 } else { -1.0 };
        v = (1.0 - u_abs) * if v >= 0.0 { 1.0 } else { -1.0 };
    }

    (u, v)
}

/// Decode octahedral coordinates in [-1, 1]² back to normalized direction
#[inline]
pub fn decode_octahedral(u: f32, v: f32) -> glam::Vec3 {
    let mut dir = glam::Vec3::new(u, v, 1.0 - u.abs() - v.abs());

    if dir.z < 0.0 {
        let old_x = dir.x;
        dir.x = (1.0 - dir.y.abs()) * if old_x >= 0.0 { 1.0 } else { -1.0 };
        dir.y = (1.0 - old_x.abs()) * if dir.y >= 0.0 { 1.0 } else { -1.0 };
    }

    dir.normalize_or_zero()
}

/// Pack Vec3 direction to u32 using octahedral encoding (2x snorm16)
#[inline]
pub fn pack_octahedral_u32(dir: glam::Vec3) -> u32 {
    let (u, v) = encode_octahedral(dir);
    let u_snorm = f32_to_snorm16(u);
    let v_snorm = f32_to_snorm16(v);
    (u_snorm as u16 as u32) | ((v_snorm as u16 as u32) << 16)
}

/// Unpack u32 to Vec3 direction using octahedral decoding
#[inline]
pub fn unpack_octahedral_u32(packed: u32) -> glam::Vec3 {
    let u_i16 = (packed & 0xFFFF) as i16;
    let v_i16 = (packed >> 16) as i16;
    let u = u_i16 as f32 / 32767.0;
    let v = v_i16 as f32 / 32767.0;
    decode_octahedral(u, v)
}

/// Pack a 3D normal to octahedral-encoded u32 (4 bytes)
#[inline]
pub fn pack_normal_octahedral(nx: f32, ny: f32, nz: f32) -> u32 {
    pack_octahedral_u32(glam::Vec3::new(nx, ny, nz))
}

// ============================================================================
// Color Packing
// ============================================================================

/// Pack an RGB color (f32x3) to Unorm8x4 format (with alpha=255)
#[inline]
pub fn pack_color_unorm8(r: f32, g: f32, b: f32) -> [u8; 4] {
    [f32_to_unorm8(r), f32_to_unorm8(g), f32_to_unorm8(b), 255]
}

/// Pack an RGBA color (f32x4) to Unorm8x4 format
#[inline]
pub fn pack_color_rgba_unorm8(r: f32, g: f32, b: f32, a: f32) -> [u8; 4] {
    [
        f32_to_unorm8(r),
        f32_to_unorm8(g),
        f32_to_unorm8(b),
        f32_to_unorm8(a),
    ]
}

// ============================================================================
// Bone Weight Packing
// ============================================================================

/// Pack bone weights as unorm8x4 (4 bytes)
#[inline]
pub fn pack_bone_weights_unorm8(weights: [f32; 4]) -> [u8; 4] {
    [
        f32_to_unorm8(weights[0]),
        f32_to_unorm8(weights[1]),
        f32_to_unorm8(weights[2]),
        f32_to_unorm8(weights[3]),
    ]
}

// ============================================================================
// Full Vertex Packing
// ============================================================================

/// Pack unpacked f32 vertex data to GPU-ready packed format
///
/// Converts f32 positions/UVs/normals/colors to packed formats based on format flags.
pub fn pack_vertex_data(data: &[f32], format: u8) -> Vec<u8> {
    let has_uv = format & FORMAT_UV != 0;
    let has_color = format & FORMAT_COLOR != 0;
    let has_normal = format & FORMAT_NORMAL != 0;
    let has_skinning = format & FORMAT_SKINNED != 0;

    let mut f32_stride = 3; // Position
    if has_uv {
        f32_stride += 2;
    }
    if has_color {
        f32_stride += 3;
    }
    if has_normal {
        f32_stride += 3;
    }
    if has_skinning {
        f32_stride += 5;
    }

    let vertex_count = data.len() / f32_stride;
    let packed_stride = vertex_stride_packed(format) as usize;
    let mut packed = Vec::with_capacity(vertex_count * packed_stride);

    for i in 0..vertex_count {
        let base = i * f32_stride;
        let mut offset = base;

        // Position: f32x3 → f16x4
        let pos = pack_position_f16(data[offset], data[offset + 1], data[offset + 2]);
        packed.extend_from_slice(cast_slice(&pos));
        offset += 3;

        // UV: f32x2 → unorm16x2
        if has_uv {
            let uv = pack_uv_unorm16(data[offset], data[offset + 1]);
            packed.extend_from_slice(cast_slice(&uv));
            offset += 2;
        }

        // Color: f32x3 → unorm8x4
        if has_color {
            let color =
                pack_color_rgba_unorm8(data[offset], data[offset + 1], data[offset + 2], 1.0);
            packed.extend_from_slice(cast_slice(&color));
            offset += 3;
        }

        // Normal: f32x3 → octahedral u32
        if has_normal {
            let normal = pack_normal_octahedral(data[offset], data[offset + 1], data[offset + 2]);
            packed.extend_from_slice(&normal.to_le_bytes());
            offset += 3;
        }

        // Skinning: bone indices + weights
        if has_skinning {
            let packed_indices_u32 = data[offset].to_bits();
            packed.extend_from_slice(&packed_indices_u32.to_le_bytes());
            offset += 1;

            let bone_weights = pack_bone_weights_unorm8([
                data[offset],
                data[offset + 1],
                data[offset + 2],
                data[offset + 3],
            ]);
            packed.extend_from_slice(&bone_weights);
            offset += 4;
        }
        let _ = offset;
    }

    packed
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pack_position_f16() {
        let packed = pack_position_f16(1.0, 2.0, 3.0);
        assert_eq!(packed[0], f16::from_f32(1.0));
        assert_eq!(packed[1], f16::from_f32(2.0));
        assert_eq!(packed[2], f16::from_f32(3.0));
        assert_eq!(packed[3], f16::from_f32(1.0));
    }

    #[test]
    fn test_f32_to_snorm16_range() {
        assert_eq!(f32_to_snorm16(-1.0), -32767);
        assert_eq!(f32_to_snorm16(0.0), 0);
        assert_eq!(f32_to_snorm16(1.0), 32767);
    }

    #[test]
    fn test_f32_to_unorm8_range() {
        assert_eq!(f32_to_unorm8(0.0), 0);
        assert_eq!(f32_to_unorm8(0.5), 127);
        assert_eq!(f32_to_unorm8(1.0), 255);
    }

    #[test]
    fn test_octahedral_roundtrip() {
        let test_dirs = [
            glam::Vec3::new(1.0, 0.0, 0.0),
            glam::Vec3::new(-1.0, 0.0, 0.0),
            glam::Vec3::new(0.0, 1.0, 0.0),
            glam::Vec3::new(0.0, 0.0, 1.0),
            glam::Vec3::new(0.577, 0.577, 0.577),
        ];

        for dir in test_dirs {
            let normalized = dir.normalize();
            let packed = pack_octahedral_u32(normalized);
            let decoded = unpack_octahedral_u32(packed);
            let error = (decoded - normalized).length();
            assert!(error < 0.01, "Roundtrip failed for {:?}", normalized);
        }
    }

    #[test]
    fn test_vertex_stride() {
        assert_eq!(vertex_stride(0), 12); // POS only
        assert_eq!(vertex_stride(FORMAT_UV), 20); // POS + UV
        assert_eq!(vertex_stride(FORMAT_UV | FORMAT_NORMAL), 32); // POS + UV + NORMAL
    }

    #[test]
    fn test_vertex_stride_packed() {
        assert_eq!(vertex_stride_packed(0), 8); // POS only
        assert_eq!(vertex_stride_packed(FORMAT_UV), 12); // POS + UV
        assert_eq!(vertex_stride_packed(FORMAT_UV | FORMAT_NORMAL), 16); // POS + UV + NORMAL
    }
}
