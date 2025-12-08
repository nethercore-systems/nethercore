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
/// Converts f32 positions/UVs/normals/colors to f16/snorm16/unorm8 based on format flags.
/// This is the core packing function used by both immediate draws and retained mesh loading.
///
/// # Format Layout (unpacked f32)
/// - Position: 3 f32 (x, y, z)
/// - UV (if FORMAT_UV): 2 f32 (u, v)
/// - Color (if FORMAT_COLOR): 3 f32 (r, g, b) - alpha added as 1.0
/// - Normal (if FORMAT_NORMAL): 3 f32 (nx, ny, nz)
/// - Skinning (if FORMAT_SKINNED): Currently placeholder (not yet implemented)
///
/// # Packed Layout (GPU format)
/// - Position: f16x4 (8 bytes, w=1.0 padding)
/// - UV (if FORMAT_UV): f16x2 (4 bytes)
/// - Color (if FORMAT_COLOR): unorm8x4 (4 bytes, alpha=255)
/// - Normal (if FORMAT_NORMAL): snorm16x4 (8 bytes, w=0 padding)
///
/// # Arguments
/// * `data` - Unpacked f32 vertex data (position + optional attributes)
/// * `format` - Vertex format flags (0-15: UV=1, COLOR=2, NORMAL=4, SKINNED=8)
///
/// # Returns
/// Packed vertex data ready for GPU upload as Vec<u8>
///
/// # Memory Savings
/// - POS_NORMAL: 24 bytes → 16 bytes (33% reduction)
/// - POS_UV_NORMAL: 32 bytes → 20 bytes (37.5% reduction)
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
        f32_stride += 9; // 4 bone indices (as f32) + 4 weights + padding (?)
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

        // UV: f32x2 → f16x2 (4 bytes)
        if has_uv {
            let uv = pack_uv_f16(data[offset], data[offset + 1]);
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

        // Normal: f32x3 → snorm16x4 (8 bytes)
        if has_normal {
            let normal = pack_normal_snorm16(data[offset], data[offset + 1], data[offset + 2]);
            packed.extend_from_slice(cast_slice(&normal));
            offset += 3;
        }

        // Skinning: Keep as-is (not packed)
        if has_skinning {
            // TODO: Implement skinning data packing when skinning is used
            // For now, this is a placeholder
            tracing::warn!("Skinning data packing not yet implemented");
        }
    }

    packed
}

/// Write a packed POS_UV_NORMAL vertex to a byte buffer (20 bytes)
///
/// Uses bytemuck to cast f16/i16 arrays to bytes efficiently.
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

    // UV: Float16x2 (4 bytes)
    let uv_packed = pack_uv_f16(uv[0], uv[1]);
    buf.extend_from_slice(cast_slice(&uv_packed)); // bytemuck: [f16; 2] -> &[u8]

    // Normal: Snorm16x4 (8 bytes)
    let norm_packed = pack_normal_snorm16(normal[0], normal[1], normal[2]);
    buf.extend_from_slice(cast_slice(&norm_packed)); // bytemuck: [i16; 4] -> &[u8]
}

/// Write a packed POS_NORMAL vertex to a byte buffer (16 bytes)
///
/// # Arguments
/// * `buf` - Byte buffer to write to
/// * `pos` - Position [x, y, z]
/// * `normal` - Normal [nx, ny, nz]
pub fn write_vertex_normal(buf: &mut Vec<u8>, pos: [f32; 3], normal: [f32; 3]) {
    // Position: Float16x4 (8 bytes)
    let pos_packed = pack_position_f16(pos[0], pos[1], pos[2]);
    buf.extend_from_slice(cast_slice(&pos_packed));

    // Normal: Snorm16x4 (8 bytes)
    let norm_packed = pack_normal_snorm16(normal[0], normal[1], normal[2]);
    buf.extend_from_slice(cast_slice(&norm_packed));
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
}
