//! Vertex data packing utilities
//!
//! Provides functions to convert f32 vertex data to packed formats:
//! - f32 → f16 (IEEE 754 half-float, 16-bit via `half` crate)
//! - f32 → snorm16 (signed normalized, -1.0 to 1.0)
//! - f32 → unorm8 (unsigned normalized, 0.0 to 1.0)
//!
//! These are used by the convenience FFI API to automatically pack vertex data.

use bytemuck::cast_slice;
use half::f16;

/// Convert f32 to signed normalized 16-bit integer (snorm16)
///
/// Maps f32 range [-1.0, 1.0] to i16 range [-32767, 32767].
/// Values outside [-1.0, 1.0] are clamped.
///
/// # Arguments
/// * `value` - f32 value to convert (ideally in range [-1.0, 1.0])
///
/// # Returns
/// i16 containing the signed normalized value
#[inline]
pub fn f32_to_snorm16(value: f32) -> i16 {
    let clamped = value.clamp(-1.0, 1.0);
    (clamped * 32767.0) as i16
}

/// Convert f32 to unsigned normalized 8-bit integer (unorm8)
///
/// Maps f32 range [0.0, 1.0] to u8 range [0, 255].
/// Values outside [0.0, 1.0] are clamped.
///
/// # Arguments
/// * `value` - f32 value to convert (ideally in range [0.0, 1.0])
///
/// # Returns
/// u8 containing the unsigned normalized value
#[inline]
pub fn f32_to_unorm8(value: f32) -> u8 {
    let clamped = value.clamp(0.0, 1.0);
    (clamped * 255.0) as u8
}

/// Pack a 3D position (f32x3) to Float16x4 format (with w=1.0 padding)
///
/// # Arguments
/// * `x`, `y`, `z` - Position coordinates
///
/// # Returns
/// [f16; 4] containing packed f16 values [x, y, z, 1.0]
#[inline]
pub fn pack_position_f16(x: f32, y: f32, z: f32) -> [f16; 4] {
    [
        f16::from_f32(x),
        f16::from_f32(y),
        f16::from_f32(z),
        f16::from_f32(1.0), // W component padding
    ]
}

/// Pack a 2D UV coordinate (f32x2) to Float16x2 format
///
/// # Arguments
/// * `u`, `v` - UV coordinates
///
/// # Returns
/// [f16; 2] containing packed f16 values [u, v]
#[inline]
pub fn pack_uv_f16(u: f32, v: f32) -> [f16; 2] {
    [f16::from_f32(u), f16::from_f32(v)]
}

/// Pack a 2D UV coordinate (f32x2) to Unorm16x2 format
///
/// Better precision than f16 for values in the [0.0, 1.0] range.
/// Unorm16 provides 65536 distinct values uniformly distributed in [0, 1],
/// while f16 has non-uniform precision that's worse near 0.
///
/// # Arguments
/// * `u`, `v` - UV coordinates (ideally in range [0.0, 1.0])
///
/// # Returns
/// [u16; 2] containing packed unorm16 values [u, v]
#[inline]
pub fn pack_uv_unorm16(u: f32, v: f32) -> [u16; 2] {
    [
        (u.clamp(0.0, 1.0) * 65535.0) as u16,
        (v.clamp(0.0, 1.0) * 65535.0) as u16,
    ]
}

/// Pack a 3D normal (f32x3) to Snorm16x4 format (with w=0.0 padding)
///
/// # Arguments
/// * `nx`, `ny`, `nz` - Normal coordinates (should be normalized)
///
/// # Returns
/// [i16; 4] containing packed snorm16 values [nx, ny, nz, 0]
#[inline]
pub fn pack_normal_snorm16(nx: f32, ny: f32, nz: f32) -> [i16; 4] {
    [
        f32_to_snorm16(nx),
        f32_to_snorm16(ny),
        f32_to_snorm16(nz),
        0, // W component padding
    ]
}

// ============================================================================
// Octahedral Normal Encoding
// ============================================================================
// Octahedral encoding provides better quality than snorm16x3 with uniform
// angular precision (~0.02° worst-case error). It maps 3D unit vectors to
// a 2D square [-1,1]² then packs as 2x snorm16 into a u32.

/// Encode normalized direction to octahedral coordinates in [-1, 1]²
///
/// Uses signed octahedral mapping for uniform precision distribution across the sphere.
/// More accurate than XY+reconstructed-Z approaches, especially near poles.
#[inline]
pub fn encode_octahedral(dir: glam::Vec3) -> (f32, f32) {
    let dir = dir.normalize_or_zero();

    // Project to octahedron via L1 normalization
    let l1_norm = dir.x.abs() + dir.y.abs() + dir.z.abs();
    if l1_norm == 0.0 {
        return (0.0, 0.0);
    }

    let mut u = dir.x / l1_norm;
    let mut v = dir.y / l1_norm;

    // Fold lower hemisphere (z < 0) into upper square
    if dir.z < 0.0 {
        let u_abs = u.abs();
        let v_abs = v.abs();
        u = (1.0 - v_abs) * if u >= 0.0 { 1.0 } else { -1.0 };
        v = (1.0 - u_abs) * if v >= 0.0 { 1.0 } else { -1.0 };
    }

    (u, v) // Both in [-1, 1]
}

/// Decode octahedral coordinates in [-1, 1]² back to normalized direction
///
/// Reverses the encoding operation to reconstruct the 3D direction vector.
#[inline]
pub fn decode_octahedral(u: f32, v: f32) -> glam::Vec3 {
    let mut dir = glam::Vec3::new(u, v, 1.0 - u.abs() - v.abs());

    // Unfold lower hemisphere (z < 0 case)
    if dir.z < 0.0 {
        let old_x = dir.x;
        dir.x = (1.0 - dir.y.abs()) * if old_x >= 0.0 { 1.0 } else { -1.0 };
        dir.y = (1.0 - old_x.abs()) * if dir.y >= 0.0 { 1.0 } else { -1.0 };
    }

    dir.normalize_or_zero()
}

/// Pack Vec3 direction to u32 using octahedral encoding (2x snorm16)
///
/// Provides uniform angular precision (~0.02° worst-case error with 16-bit components).
/// More compact (4 bytes) than snorm16x4 (8 bytes) with better quality.
///
/// # Arguments
/// * `dir` - Direction vector (will be normalized)
///
/// # Returns
/// u32 containing packed octahedral coordinates [u: low 16 bits][v: high 16 bits]
#[inline]
pub fn pack_octahedral_u32(dir: glam::Vec3) -> u32 {
    let (u, v) = encode_octahedral(dir);
    let u_snorm = f32_to_snorm16(u);
    let v_snorm = f32_to_snorm16(v);
    // Pack as [u: i16 low 16 bits][v: i16 high 16 bits]
    (u_snorm as u16 as u32) | ((v_snorm as u16 as u32) << 16)
}

/// Unpack u32 to Vec3 direction using octahedral decoding (2x snorm16)
///
/// Reverses pack_octahedral_u32() to extract the original direction.
#[inline]
pub fn unpack_octahedral_u32(packed: u32) -> glam::Vec3 {
    // Extract i16 components with sign extension
    let u_i16 = (packed & 0xFFFF) as i16;
    let v_i16 = (packed >> 16) as i16;

    // Convert snorm16 to float [-1, 1]
    let u = u_i16 as f32 / 32767.0;
    let v = v_i16 as f32 / 32767.0;

    decode_octahedral(u, v)
}

/// Pack a 3D normal to octahedral-encoded u32 (4 bytes)
///
/// Convenience wrapper for vertex normal packing. Better quality than
/// snorm16x3 with uniform angular precision and smaller size (4 vs 8 bytes).
///
/// # Arguments
/// * `nx`, `ny`, `nz` - Normal coordinates (should be normalized)
///
/// # Returns
/// u32 containing octahedral-encoded normal
#[inline]
pub fn pack_normal_octahedral(nx: f32, ny: f32, nz: f32) -> u32 {
    pack_octahedral_u32(glam::Vec3::new(nx, ny, nz))
}

// ============================================================================
// Bone Weight Packing
// ============================================================================

/// Pack bone weights as unorm8x4 (4 bytes)
///
/// Reduces from 16 bytes (Float32x4) to 4 bytes. The precision loss is
/// acceptable for bone weights since they're interpolation factors.
///
/// # Arguments
/// * `weights` - Array of 4 bone weights, each in range [0.0, 1.0]
///
/// # Returns
/// [u8; 4] containing packed unorm8 values
#[inline]
pub fn pack_bone_weights_unorm8(weights: [f32; 4]) -> [u8; 4] {
    [
        f32_to_unorm8(weights[0]),
        f32_to_unorm8(weights[1]),
        f32_to_unorm8(weights[2]),
        f32_to_unorm8(weights[3]),
    ]
}

/// Pack an RGB color (f32x3) to Unorm8x4 format (with alpha=255)
///
/// # Arguments
/// * `r`, `g`, `b` - Color components in range [0.0, 1.0]
///
/// # Returns
/// [u8; 4] containing packed unorm8 values [r, g, b, 255]
#[inline]
pub fn pack_color_unorm8(r: f32, g: f32, b: f32) -> [u8; 4] {
    [
        f32_to_unorm8(r),
        f32_to_unorm8(g),
        f32_to_unorm8(b),
        255, // Alpha = 1.0
    ]
}

/// Pack an RGBA color (f32x4) to Unorm8x4 format
///
/// # Arguments
/// * `r`, `g`, `b`, `a` - Color components in range [0.0, 1.0]
///
/// # Returns
/// [u8; 4] containing packed unorm8 values [r, g, b, a]
#[inline]
pub fn pack_color_rgba_unorm8(r: f32, g: f32, b: f32, a: f32) -> [u8; 4] {
    [
        f32_to_unorm8(r),
        f32_to_unorm8(g),
        f32_to_unorm8(b),
        f32_to_unorm8(a),
    ]
}

/// Pack unpacked f32 vertex data to GPU-ready packed format
///
/// Converts f32 positions/UVs/normals/colors to packed formats based on format flags.
/// This is the core packing function used by both immediate draws and retained mesh loading.
///
/// # Format Layout (unpacked f32)
/// - Position: 3 f32 (x, y, z)
/// - UV (if FORMAT_UV): 2 f32 (u, v)
/// - Color (if FORMAT_COLOR): 3 f32 (r, g, b) - alpha added as 1.0
/// - Normal (if FORMAT_NORMAL): 3 f32 (nx, ny, nz)
/// - Skinning (if FORMAT_SKINNED): 4 bone indices + 4 weights
///
/// # Packed Layout (GPU format)
/// - Position: f16x4 (8 bytes, w=1.0 padding)
/// - UV (if FORMAT_UV): unorm16x2 (4 bytes) - better precision than f16 in [0,1]
/// - Color (if FORMAT_COLOR): unorm8x4 (4 bytes, alpha=255)
/// - Normal (if FORMAT_NORMAL): octahedral u32 (4 bytes) - better quality than snorm16x4
/// - Skinning (if FORMAT_SKINNED): u8x4 indices (4 bytes) + unorm8x4 weights (4 bytes)
///
/// # Arguments
/// * `data` - Unpacked f32 vertex data (position + optional attributes)
/// * `format` - Vertex format flags (0-15: UV=1, COLOR=2, NORMAL=4, SKINNED=8)
///
/// # Returns
/// Packed vertex data ready for GPU upload as Vec<u8>
///
/// # Memory Savings
/// - POS_NORMAL: 24 bytes → 12 bytes (50% reduction)
/// - POS_UV_NORMAL: 32 bytes → 16 bytes (50% reduction)
pub fn pack_vertex_data(data: &[f32], format: u8) -> Vec<u8> {
    use crate::graphics::{vertex_stride_packed, FORMAT_COLOR, FORMAT_NORMAL, FORMAT_SKINNED, FORMAT_UV};

    let has_uv = format & FORMAT_UV != 0;
    let has_color = format & FORMAT_COLOR != 0;
    let has_normal = format & FORMAT_NORMAL != 0;
    let has_skinning = format & FORMAT_SKINNED != 0;

    // Calculate unpacked stride (how many f32s per vertex)
    let mut f32_stride = 3; // Position (x, y, z)
    if has_uv {
        f32_stride += 2; // UV (u, v)
    }
    if has_color {
        f32_stride += 3; // Color (r, g, b) - alpha added as 1.0
    }
    if has_normal {
        f32_stride += 3; // Normal (nx, ny, nz)
    }
    if has_skinning {
        f32_stride += 8; // 4 bone indices (as f32) + 4 weights + padding (?)
                         // NOTE: Skinning layout needs verification
    }

    let vertex_count = data.len() / f32_stride;
    let packed_stride = vertex_stride_packed(format) as usize;
    let mut packed = Vec::with_capacity(vertex_count * packed_stride);

    for i in 0..vertex_count {
        let base = i * f32_stride;
        let mut offset = base;

        // Position: f32x3 → f16x4 (8 bytes)
        let pos = pack_position_f16(data[offset], data[offset + 1], data[offset + 2]);
        packed.extend_from_slice(cast_slice(&pos));
        offset += 3;

        // UV: f32x2 → unorm16x2 (4 bytes)
        if has_uv {
            let uv = pack_uv_unorm16(data[offset], data[offset + 1]);
            packed.extend_from_slice(cast_slice(&uv));
            offset += 2;
        }

        // Color: f32x3 → unorm8x4 (4 bytes, alpha=255)
        if has_color {
            let color = pack_color_rgba_unorm8(
                data[offset],
                data[offset + 1],
                data[offset + 2],
                1.0,
            );
            packed.extend_from_slice(cast_slice(&color));
            offset += 3;
        }

        // Normal: f32x3 → octahedral u32 (4 bytes)
        if has_normal {
            let normal = pack_normal_octahedral(data[offset], data[offset + 1], data[offset + 2]);
            packed.extend_from_slice(&normal.to_le_bytes());
            offset += 3;
        }

        // Skinning: bone indices (u8x4) + bone weights (unorm8x4)
        if has_skinning {
            // Bone indices: 4 f32 → u8x4 (4 bytes)
            let bone_indices = [
                data[offset] as u8,
                data[offset + 1] as u8,
                data[offset + 2] as u8,
                data[offset + 3] as u8,
            ];
            packed.extend_from_slice(&bone_indices);
            offset += 4;

            // Bone weights: 4 f32 → unorm8x4 (4 bytes)
            let bone_weights = pack_bone_weights_unorm8([
                data[offset],
                data[offset + 1],
                data[offset + 2],
                data[offset + 3],
            ]);
            packed.extend_from_slice(&bone_weights);
            offset += 4;
        }
        let _ = offset; // Suppress unused warning
    }

    packed
}

/// Write a packed POS_UV_NORMAL vertex to a byte buffer (16 bytes)
///
/// Uses bytemuck to cast arrays to bytes efficiently.
///
/// # Arguments
/// * `buf` - Byte buffer to write to
/// * `pos` - Position [x, y, z]
/// * `uv` - UV coordinates [u, v]
/// * `normal` - Normal [nx, ny, nz]
pub fn write_vertex_uv_normal(buf: &mut Vec<u8>, pos: [f32; 3], uv: [f32; 2], normal: [f32; 3]) {
    // Position: Float16x4 (8 bytes)
    let pos_packed = pack_position_f16(pos[0], pos[1], pos[2]);
    buf.extend_from_slice(cast_slice(&pos_packed)); // bytemuck: [f16; 4] -> &[u8]

    // UV: Unorm16x2 (4 bytes)
    let uv_packed = pack_uv_unorm16(uv[0], uv[1]);
    buf.extend_from_slice(cast_slice(&uv_packed)); // bytemuck: [u16; 2] -> &[u8]

    // Normal: Octahedral u32 (4 bytes)
    let norm_packed = pack_normal_octahedral(normal[0], normal[1], normal[2]);
    buf.extend_from_slice(&norm_packed.to_le_bytes());
}

/// Write a packed POS_NORMAL vertex to a byte buffer (12 bytes)
///
/// # Arguments
/// * `buf` - Byte buffer to write to
/// * `pos` - Position [x, y, z]
/// * `normal` - Normal [nx, ny, nz]
pub fn write_vertex_normal(buf: &mut Vec<u8>, pos: [f32; 3], normal: [f32; 3]) {
    // Position: Float16x4 (8 bytes)
    let pos_packed = pack_position_f16(pos[0], pos[1], pos[2]);
    buf.extend_from_slice(cast_slice(&pos_packed));

    // Normal: Octahedral u32 (4 bytes)
    let norm_packed = pack_normal_octahedral(normal[0], normal[1], normal[2]);
    buf.extend_from_slice(&norm_packed.to_le_bytes());
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
        assert_eq!(packed[3], f16::from_f32(1.0)); // W padding
    }

    #[test]
    fn test_pack_uv_f16() {
        let packed = pack_uv_f16(0.5, 0.75);
        assert_eq!(packed[0], f16::from_f32(0.5));
        assert_eq!(packed[1], f16::from_f32(0.75));
    }

    #[test]
    fn test_bytemuck_cast() {
        let packed = pack_position_f16(1.0, 2.0, 3.0);
        let bytes: &[u8] = cast_slice(&packed);
        assert_eq!(bytes.len(), 8); // 4 × f16 (2 bytes each)
    }

    #[test]
    fn test_f32_to_snorm16_range() {
        assert_eq!(f32_to_snorm16(-1.0), -32767);
        assert_eq!(f32_to_snorm16(0.0), 0);
        assert_eq!(f32_to_snorm16(1.0), 32767);
    }

    #[test]
    fn test_f32_to_snorm16_clamping() {
        assert_eq!(f32_to_snorm16(-2.0), -32767); // Clamped
        assert_eq!(f32_to_snorm16(2.0), 32767); // Clamped
    }

    #[test]
    fn test_f32_to_unorm8_range() {
        assert_eq!(f32_to_unorm8(0.0), 0);
        assert_eq!(f32_to_unorm8(0.5), 127);
        assert_eq!(f32_to_unorm8(1.0), 255);
    }

    #[test]
    fn test_f32_to_unorm8_clamping() {
        assert_eq!(f32_to_unorm8(-1.0), 0); // Clamped
        assert_eq!(f32_to_unorm8(2.0), 255); // Clamped
    }

    #[test]
    fn test_pack_normal_snorm16() {
        let packed = pack_normal_snorm16(0.0, 1.0, 0.0);
        assert_eq!(packed[0], 0);
        assert_eq!(packed[1], 32767);
        assert_eq!(packed[2], 0);
        assert_eq!(packed[3], 0); // W padding
    }

    #[test]
    fn test_pack_color_unorm8() {
        let packed = pack_color_unorm8(1.0, 0.5, 0.0);
        assert_eq!(packed[0], 255);
        assert_eq!(packed[1], 127);
        assert_eq!(packed[2], 0);
        assert_eq!(packed[3], 255); // Alpha
    }

    #[test]
    fn test_pack_color_rgba_unorm8() {
        let packed = pack_color_rgba_unorm8(1.0, 0.5, 0.0, 0.75);
        assert_eq!(packed[0], 255);
        assert_eq!(packed[1], 127);
        assert_eq!(packed[2], 0);
        assert_eq!(packed[3], 191); // 0.75 * 255 ≈ 191
    }

    /// Test octahedral encoding/decoding roundtrip using WGSL-compatible unpacking logic.
    /// This simulates exactly what the GPU shader does to catch any sign extension issues.
    #[test]
    fn test_octahedral_wgsl_roundtrip() {
        // Simulate WGSL unpack logic exactly
        // WGSL: let u_i16 = i32((packed & 0xFFFFu) << 16u) >> 16;
        // WGSL: let v_i16 = i32(packed) >> 16;
        fn wgsl_unpack(packed: u32) -> (f32, f32) {
            // For u: extract low 16 bits, shift to high position, bitcast to i32, arithmetic shift back
            let u_shifted = (packed & 0xFFFF) << 16;
            let u_i32 = u_shifted as i32; // bitcast u32 to i32
            let u_i16 = u_i32 >> 16; // arithmetic shift sign-extends

            // For v: bitcast entire u32 to i32, then arithmetic shift right by 16
            let v_i32 = packed as i32; // bitcast u32 to i32
            let v_i16 = v_i32 >> 16; // arithmetic shift sign-extends

            (u_i16 as f32 / 32767.0, v_i16 as f32 / 32767.0)
        }

        // Test cases including negative values in all octants
        let test_dirs = [
            glam::Vec3::new(1.0, 0.0, 0.0),   // +X
            glam::Vec3::new(-1.0, 0.0, 0.0),  // -X
            glam::Vec3::new(0.0, 1.0, 0.0),   // +Y
            glam::Vec3::new(0.0, -1.0, 0.0),  // -Y
            glam::Vec3::new(0.0, 0.0, 1.0),   // +Z (upper hemisphere center)
            glam::Vec3::new(0.0, 0.0, -1.0),  // -Z (lower hemisphere center)
            glam::Vec3::new(-0.707, 0.707, 0.0),   // XY plane, negative X
            glam::Vec3::new(0.707, -0.707, 0.0),   // XY plane, negative Y
            glam::Vec3::new(0.577, 0.577, 0.577),  // Diagonal +X+Y+Z
            glam::Vec3::new(-0.577, 0.577, 0.577), // Diagonal -X+Y+Z
            glam::Vec3::new(0.577, -0.577, 0.577), // Diagonal +X-Y+Z
            glam::Vec3::new(-0.577, -0.577, 0.577), // Diagonal -X-Y+Z
            glam::Vec3::new(0.577, 0.577, -0.577),  // Lower hemisphere
            glam::Vec3::new(-0.577, -0.577, -0.577), // Lower hemisphere opposite
        ];

        for dir in test_dirs {
            let normalized = dir.normalize();
            let packed = pack_octahedral_u32(normalized);
            let (u, v) = wgsl_unpack(packed);
            let decoded = decode_octahedral(u, v);

            let error = (decoded - normalized).length();
            assert!(
                error < 0.01,
                "WGSL roundtrip failed for {:?}:\n  packed=0x{:08X}\n  u={}, v={}\n  decoded={:?}\n  error={}",
                normalized, packed, u, v, decoded, error
            );
        }
    }

    /// Test that negative octahedral coordinates are correctly sign-extended
    #[test]
    fn test_octahedral_negative_sign_extension() {
        // Test a direction that produces negative u value: (-1, 0, 0)
        let dir = glam::Vec3::new(-1.0, 0.0, 0.0);
        let packed = pack_octahedral_u32(dir);

        // The low 16 bits should represent a negative snorm16 (sign bit set)
        let u_raw = packed & 0xFFFF;
        println!("Testing direction {:?}", dir);
        println!("  packed = 0x{:08X}", packed);
        println!("  u_raw (low 16 bits) = 0x{:04X} = {} as u16", u_raw, u_raw);

        // If u_raw has bit 15 set, it's a negative snorm16
        if u_raw & 0x8000 != 0 {
            println!("  u is negative (sign bit set)");

            // Verify sign extension works correctly
            let u_shifted = u_raw << 16;
            let u_i32 = u_shifted as i32;
            let u_i16 = u_i32 >> 16;
            println!("  WGSL-style: shifted=0x{:08X}, as_i32={}, shifted_back={}", u_shifted, u_i32, u_i16);

            // u_i16 should be negative (around -32767 for unit vector component)
            assert!(u_i16 < 0, "Sign extension failed: u_i16={} should be negative", u_i16);
        }
    }
}
