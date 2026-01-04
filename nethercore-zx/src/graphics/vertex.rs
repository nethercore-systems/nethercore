//! Vertex format definitions and wgpu buffer layouts
//!
//! This module provides wgpu-specific vertex buffer layout information.
//! Format constants and stride functions are in z-common.

// Re-export format constants from zx-common
pub use zx_common::{
    FORMAT_COLOR, FORMAT_NORMAL, FORMAT_SKINNED, FORMAT_TANGENT, FORMAT_UV, vertex_stride,
    vertex_stride_packed,
};

/// All format flags combined (without tangent - tangent requires normal)
pub const FORMAT_ALL: u8 = FORMAT_UV | FORMAT_COLOR | FORMAT_NORMAL | FORMAT_SKINNED;

/// All format flags including tangent
#[allow(dead_code)]
pub const FORMAT_ALL_WITH_TANGENT: u8 =
    FORMAT_UV | FORMAT_COLOR | FORMAT_NORMAL | FORMAT_TANGENT | FORMAT_SKINNED;

/// Number of vertex format permutations (32: 0-31, includes tangent flag)
pub const VERTEX_FORMAT_COUNT: usize = 32;

/// Vertex format information for creating vertex buffer layouts
#[derive(Debug, Clone)]
pub struct VertexFormatInfo {
    /// Format flags (combination of FORMAT_* constants)
    pub format: u8,
    /// Stride in bytes
    pub stride: u32,
    /// Human-readable name for debugging
    pub name: &'static str,
}

impl VertexFormatInfo {
    /// Get vertex format info for a format index (0-31)
    ///
    /// Returns info for GPU vertex buffers, which always use packed formats.
    pub const fn for_format(format: u8) -> Self {
        let name = match format {
            0 => "POS",
            1 => "POS_UV",
            2 => "POS_COLOR",
            3 => "POS_UV_COLOR",
            4 => "POS_NORMAL",
            5 => "POS_UV_NORMAL",
            6 => "POS_COLOR_NORMAL",
            7 => "POS_UV_COLOR_NORMAL",
            8 => "POS_SKINNED",
            9 => "POS_UV_SKINNED",
            10 => "POS_COLOR_SKINNED",
            11 => "POS_UV_COLOR_SKINNED",
            12 => "POS_NORMAL_SKINNED",
            13 => "POS_UV_NORMAL_SKINNED",
            14 => "POS_COLOR_NORMAL_SKINNED",
            15 => "POS_UV_COLOR_NORMAL_SKINNED",
            // Formats 16-31: Same as 0-15 but with tangent flag (bit 4)
            // Note: Tangent requires normal, so formats 16-19 and 24-27 are invalid
            16 => "POS_TANGENT", // Invalid: tangent requires normal
            17 => "POS_UV_TANGENT", // Invalid
            18 => "POS_COLOR_TANGENT", // Invalid
            19 => "POS_UV_COLOR_TANGENT", // Invalid
            20 => "POS_NORMAL_TANGENT",
            21 => "POS_UV_NORMAL_TANGENT",
            22 => "POS_COLOR_NORMAL_TANGENT",
            23 => "POS_UV_COLOR_NORMAL_TANGENT",
            24 => "POS_TANGENT_SKINNED", // Invalid
            25 => "POS_UV_TANGENT_SKINNED", // Invalid
            26 => "POS_COLOR_TANGENT_SKINNED", // Invalid
            27 => "POS_UV_COLOR_TANGENT_SKINNED", // Invalid
            28 => "POS_NORMAL_TANGENT_SKINNED",
            29 => "POS_UV_NORMAL_TANGENT_SKINNED",
            30 => "POS_COLOR_NORMAL_TANGENT_SKINNED",
            31 => "POS_UV_COLOR_NORMAL_TANGENT_SKINNED",
            _ => "UNKNOWN",
        };

        Self {
            format,
            stride: vertex_stride_packed(format), // GPU always uses packed
            name,
        }
    }

    /// Check if this format has UV coordinates
    #[inline]
    pub const fn has_uv(&self) -> bool {
        self.format & FORMAT_UV != 0
    }

    /// Check if this format has per-vertex color
    #[inline]
    pub const fn has_color(&self) -> bool {
        self.format & FORMAT_COLOR != 0
    }

    /// Check if this format has normals
    #[inline]
    pub const fn has_normal(&self) -> bool {
        self.format & FORMAT_NORMAL != 0
    }

    /// Check if this format has tangent vectors
    #[inline]
    pub const fn has_tangent(&self) -> bool {
        self.format & FORMAT_TANGENT != 0
    }

    /// Check if this format has skinning data
    #[inline]
    pub const fn has_skinned(&self) -> bool {
        self.format & FORMAT_SKINNED != 0
    }

    /// Check if this is a valid format (tangent requires normal)
    #[inline]
    pub const fn is_valid(&self) -> bool {
        // Tangent requires normal
        if self.format & FORMAT_TANGENT != 0 && self.format & FORMAT_NORMAL == 0 {
            return false;
        }
        true
    }

    /// Creates a wgpu vertex buffer layout descriptor for this format.
    ///
    /// Returns a layout with:
    /// - `array_stride` set to this format's stride in bytes
    /// - `step_mode` set to per-vertex stepping
    /// - `attributes` built from the format flags
    ///
    /// The returned layout can be used when creating render pipelines.
    pub fn vertex_buffer_layout(&self) -> wgpu::VertexBufferLayout<'static> {
        // Build attribute list based on format
        let attributes = Self::build_attributes(self.format);

        wgpu::VertexBufferLayout {
            array_stride: self.stride as u64,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes,
        }
    }

    /// Builds packed vertex attributes for a format.
    ///
    /// GPU always uses packed formats.
    ///
    /// Shader locations are assigned in order:
    /// - Location 0: Position (Float16x4, padded)
    /// - Location 1: UV (if FORMAT_UV, Unorm16x2)
    /// - Location 2: Color (if FORMAT_COLOR, Unorm8x4)
    /// - Location 3: Normal (if FORMAT_NORMAL, Uint32 octahedral)
    /// - Location 4: Bone indices (if FORMAT_SKINNED, Uint8x4)
    /// - Location 5: Bone weights (if FORMAT_SKINNED, Unorm8x4)
    fn build_attributes(format: u8) -> &'static [wgpu::VertexAttribute] {
        VERTEX_ATTRIBUTES[format as usize]
    }
}

// ============================================================================
// wgpu-specific vertex attribute definitions (requires runtime feature)
// ============================================================================

mod wgpu_attrs {
    /// Attribute sizes in bytes for offset calculation (packed formats - GPU only)
    const SIZE_POS: u64 = 8; // Float16x4 (padded for alignment)
    const SIZE_UV: u64 = 4; // Unorm16x2
    const SIZE_COLOR: u64 = 4; // Unorm8x4
    const SIZE_NORMAL: u64 = 4; // Octahedral u32
    const SIZE_TANGENT: u64 = 4; // Octahedral u32 with sign bit
    const SIZE_BONE_INDICES: u64 = 4; // Uint8x4
    // Note: SIZE_BONE_WEIGHTS not needed - bone weights is always the last attribute
    // so its size never appears in offset calculations

    /// Shader locations for each attribute type
    const LOC_POS: u32 = 0;
    const LOC_UV: u32 = 1;
    const LOC_COLOR: u32 = 2;
    const LOC_NORMAL: u32 = 3;
    const LOC_BONE_INDICES: u32 = 4;
    const LOC_BONE_WEIGHTS: u32 = 5;
    const LOC_TANGENT: u32 = 6;

    /// Creates a position attribute at offset 0 (Float16x4, padded)
    const fn attr_pos() -> wgpu::VertexAttribute {
        wgpu::VertexAttribute {
            format: wgpu::VertexFormat::Float16x4,
            offset: 0,
            shader_location: LOC_POS,
        }
    }

    /// Creates a UV attribute at the given offset (Unorm16x2)
    const fn attr_uv(offset: u64) -> wgpu::VertexAttribute {
        wgpu::VertexAttribute {
            format: wgpu::VertexFormat::Unorm16x2,
            offset,
            shader_location: LOC_UV,
        }
    }

    /// Creates a color attribute at the given offset (Unorm8x4)
    const fn attr_color(offset: u64) -> wgpu::VertexAttribute {
        wgpu::VertexAttribute {
            format: wgpu::VertexFormat::Unorm8x4,
            offset,
            shader_location: LOC_COLOR,
        }
    }

    /// Creates a normal attribute at the given offset (Uint32 - octahedral encoded)
    const fn attr_normal(offset: u64) -> wgpu::VertexAttribute {
        wgpu::VertexAttribute {
            format: wgpu::VertexFormat::Uint32,
            offset,
            shader_location: LOC_NORMAL,
        }
    }

    /// Creates a tangent attribute at the given offset (Uint32 - octahedral with sign bit)
    const fn attr_tangent(offset: u64) -> wgpu::VertexAttribute {
        wgpu::VertexAttribute {
            format: wgpu::VertexFormat::Uint32,
            offset,
            shader_location: LOC_TANGENT,
        }
    }

    /// Creates bone indices attribute at the given offset (Uint8x4)
    const fn attr_bone_indices(offset: u64) -> wgpu::VertexAttribute {
        wgpu::VertexAttribute {
            format: wgpu::VertexFormat::Uint8x4,
            offset,
            shader_location: LOC_BONE_INDICES,
        }
    }

    /// Creates bone weights attribute at the given offset (Unorm8x4)
    const fn attr_bone_weights(offset: u64) -> wgpu::VertexAttribute {
        wgpu::VertexAttribute {
            format: wgpu::VertexFormat::Unorm8x4,
            offset,
            shader_location: LOC_BONE_WEIGHTS,
        }
    }

    /// Pre-computed vertex attribute arrays for all 32 formats.
    ///
    /// Vertex layout order: Position → UV → Color → Normal → Tangent → Bone Indices → Bone Weights
    /// Each attribute is only present if its corresponding flag is set.
    /// Offsets are computed based on which attributes precede each one.
    ///
    /// Note: Formats 16-19 and 24-27 have tangent but no normal - these are invalid
    /// but still defined to avoid runtime panics. They should never be used.
    pub static VERTEX_ATTRIBUTES: [&[wgpu::VertexAttribute]; 32] = [
        // ============================================================================
        // Formats 0-15: Without tangent (same as before)
        // ============================================================================
        // Format 0: POS
        &[attr_pos()],
        // Format 1: POS_UV
        &[attr_pos(), attr_uv(SIZE_POS)],
        // Format 2: POS_COLOR
        &[attr_pos(), attr_color(SIZE_POS)],
        // Format 3: POS_UV_COLOR
        &[
            attr_pos(),
            attr_uv(SIZE_POS),
            attr_color(SIZE_POS + SIZE_UV),
        ],
        // Format 4: POS_NORMAL
        &[attr_pos(), attr_normal(SIZE_POS)],
        // Format 5: POS_UV_NORMAL
        &[
            attr_pos(),
            attr_uv(SIZE_POS),
            attr_normal(SIZE_POS + SIZE_UV),
        ],
        // Format 6: POS_COLOR_NORMAL
        &[
            attr_pos(),
            attr_color(SIZE_POS),
            attr_normal(SIZE_POS + SIZE_COLOR),
        ],
        // Format 7: POS_UV_COLOR_NORMAL
        &[
            attr_pos(),
            attr_uv(SIZE_POS),
            attr_color(SIZE_POS + SIZE_UV),
            attr_normal(SIZE_POS + SIZE_UV + SIZE_COLOR),
        ],
        // Format 8: POS_SKINNED
        &[
            attr_pos(),
            attr_bone_indices(SIZE_POS),
            attr_bone_weights(SIZE_POS + SIZE_BONE_INDICES),
        ],
        // Format 9: POS_UV_SKINNED
        &[
            attr_pos(),
            attr_uv(SIZE_POS),
            attr_bone_indices(SIZE_POS + SIZE_UV),
            attr_bone_weights(SIZE_POS + SIZE_UV + SIZE_BONE_INDICES),
        ],
        // Format 10: POS_COLOR_SKINNED
        &[
            attr_pos(),
            attr_color(SIZE_POS),
            attr_bone_indices(SIZE_POS + SIZE_COLOR),
            attr_bone_weights(SIZE_POS + SIZE_COLOR + SIZE_BONE_INDICES),
        ],
        // Format 11: POS_UV_COLOR_SKINNED
        &[
            attr_pos(),
            attr_uv(SIZE_POS),
            attr_color(SIZE_POS + SIZE_UV),
            attr_bone_indices(SIZE_POS + SIZE_UV + SIZE_COLOR),
            attr_bone_weights(SIZE_POS + SIZE_UV + SIZE_COLOR + SIZE_BONE_INDICES),
        ],
        // Format 12: POS_NORMAL_SKINNED
        &[
            attr_pos(),
            attr_normal(SIZE_POS),
            attr_bone_indices(SIZE_POS + SIZE_NORMAL),
            attr_bone_weights(SIZE_POS + SIZE_NORMAL + SIZE_BONE_INDICES),
        ],
        // Format 13: POS_UV_NORMAL_SKINNED
        &[
            attr_pos(),
            attr_uv(SIZE_POS),
            attr_normal(SIZE_POS + SIZE_UV),
            attr_bone_indices(SIZE_POS + SIZE_UV + SIZE_NORMAL),
            attr_bone_weights(SIZE_POS + SIZE_UV + SIZE_NORMAL + SIZE_BONE_INDICES),
        ],
        // Format 14: POS_COLOR_NORMAL_SKINNED
        &[
            attr_pos(),
            attr_color(SIZE_POS),
            attr_normal(SIZE_POS + SIZE_COLOR),
            attr_bone_indices(SIZE_POS + SIZE_COLOR + SIZE_NORMAL),
            attr_bone_weights(SIZE_POS + SIZE_COLOR + SIZE_NORMAL + SIZE_BONE_INDICES),
        ],
        // Format 15: POS_UV_COLOR_NORMAL_SKINNED
        &[
            attr_pos(),
            attr_uv(SIZE_POS),
            attr_color(SIZE_POS + SIZE_UV),
            attr_normal(SIZE_POS + SIZE_UV + SIZE_COLOR),
            attr_bone_indices(SIZE_POS + SIZE_UV + SIZE_COLOR + SIZE_NORMAL),
            attr_bone_weights(SIZE_POS + SIZE_UV + SIZE_COLOR + SIZE_NORMAL + SIZE_BONE_INDICES),
        ],
        // ============================================================================
        // Formats 16-31: With tangent flag (bit 4)
        // ============================================================================
        // Format 16: POS_TANGENT (INVALID - tangent requires normal)
        &[attr_pos(), attr_tangent(SIZE_POS)],
        // Format 17: POS_UV_TANGENT (INVALID)
        &[attr_pos(), attr_uv(SIZE_POS), attr_tangent(SIZE_POS + SIZE_UV)],
        // Format 18: POS_COLOR_TANGENT (INVALID)
        &[attr_pos(), attr_color(SIZE_POS), attr_tangent(SIZE_POS + SIZE_COLOR)],
        // Format 19: POS_UV_COLOR_TANGENT (INVALID)
        &[
            attr_pos(),
            attr_uv(SIZE_POS),
            attr_color(SIZE_POS + SIZE_UV),
            attr_tangent(SIZE_POS + SIZE_UV + SIZE_COLOR),
        ],
        // Format 20: POS_NORMAL_TANGENT
        &[
            attr_pos(),
            attr_normal(SIZE_POS),
            attr_tangent(SIZE_POS + SIZE_NORMAL),
        ],
        // Format 21: POS_UV_NORMAL_TANGENT
        &[
            attr_pos(),
            attr_uv(SIZE_POS),
            attr_normal(SIZE_POS + SIZE_UV),
            attr_tangent(SIZE_POS + SIZE_UV + SIZE_NORMAL),
        ],
        // Format 22: POS_COLOR_NORMAL_TANGENT
        &[
            attr_pos(),
            attr_color(SIZE_POS),
            attr_normal(SIZE_POS + SIZE_COLOR),
            attr_tangent(SIZE_POS + SIZE_COLOR + SIZE_NORMAL),
        ],
        // Format 23: POS_UV_COLOR_NORMAL_TANGENT
        &[
            attr_pos(),
            attr_uv(SIZE_POS),
            attr_color(SIZE_POS + SIZE_UV),
            attr_normal(SIZE_POS + SIZE_UV + SIZE_COLOR),
            attr_tangent(SIZE_POS + SIZE_UV + SIZE_COLOR + SIZE_NORMAL),
        ],
        // Format 24: POS_TANGENT_SKINNED (INVALID)
        &[
            attr_pos(),
            attr_tangent(SIZE_POS),
            attr_bone_indices(SIZE_POS + SIZE_TANGENT),
            attr_bone_weights(SIZE_POS + SIZE_TANGENT + SIZE_BONE_INDICES),
        ],
        // Format 25: POS_UV_TANGENT_SKINNED (INVALID)
        &[
            attr_pos(),
            attr_uv(SIZE_POS),
            attr_tangent(SIZE_POS + SIZE_UV),
            attr_bone_indices(SIZE_POS + SIZE_UV + SIZE_TANGENT),
            attr_bone_weights(SIZE_POS + SIZE_UV + SIZE_TANGENT + SIZE_BONE_INDICES),
        ],
        // Format 26: POS_COLOR_TANGENT_SKINNED (INVALID)
        &[
            attr_pos(),
            attr_color(SIZE_POS),
            attr_tangent(SIZE_POS + SIZE_COLOR),
            attr_bone_indices(SIZE_POS + SIZE_COLOR + SIZE_TANGENT),
            attr_bone_weights(SIZE_POS + SIZE_COLOR + SIZE_TANGENT + SIZE_BONE_INDICES),
        ],
        // Format 27: POS_UV_COLOR_TANGENT_SKINNED (INVALID)
        &[
            attr_pos(),
            attr_uv(SIZE_POS),
            attr_color(SIZE_POS + SIZE_UV),
            attr_tangent(SIZE_POS + SIZE_UV + SIZE_COLOR),
            attr_bone_indices(SIZE_POS + SIZE_UV + SIZE_COLOR + SIZE_TANGENT),
            attr_bone_weights(SIZE_POS + SIZE_UV + SIZE_COLOR + SIZE_TANGENT + SIZE_BONE_INDICES),
        ],
        // Format 28: POS_NORMAL_TANGENT_SKINNED
        &[
            attr_pos(),
            attr_normal(SIZE_POS),
            attr_tangent(SIZE_POS + SIZE_NORMAL),
            attr_bone_indices(SIZE_POS + SIZE_NORMAL + SIZE_TANGENT),
            attr_bone_weights(SIZE_POS + SIZE_NORMAL + SIZE_TANGENT + SIZE_BONE_INDICES),
        ],
        // Format 29: POS_UV_NORMAL_TANGENT_SKINNED
        &[
            attr_pos(),
            attr_uv(SIZE_POS),
            attr_normal(SIZE_POS + SIZE_UV),
            attr_tangent(SIZE_POS + SIZE_UV + SIZE_NORMAL),
            attr_bone_indices(SIZE_POS + SIZE_UV + SIZE_NORMAL + SIZE_TANGENT),
            attr_bone_weights(SIZE_POS + SIZE_UV + SIZE_NORMAL + SIZE_TANGENT + SIZE_BONE_INDICES),
        ],
        // Format 30: POS_COLOR_NORMAL_TANGENT_SKINNED
        &[
            attr_pos(),
            attr_color(SIZE_POS),
            attr_normal(SIZE_POS + SIZE_COLOR),
            attr_tangent(SIZE_POS + SIZE_COLOR + SIZE_NORMAL),
            attr_bone_indices(SIZE_POS + SIZE_COLOR + SIZE_NORMAL + SIZE_TANGENT),
            attr_bone_weights(SIZE_POS + SIZE_COLOR + SIZE_NORMAL + SIZE_TANGENT + SIZE_BONE_INDICES),
        ],
        // Format 31: POS_UV_COLOR_NORMAL_TANGENT_SKINNED
        &[
            attr_pos(),
            attr_uv(SIZE_POS),
            attr_color(SIZE_POS + SIZE_UV),
            attr_normal(SIZE_POS + SIZE_UV + SIZE_COLOR),
            attr_tangent(SIZE_POS + SIZE_UV + SIZE_COLOR + SIZE_NORMAL),
            attr_bone_indices(SIZE_POS + SIZE_UV + SIZE_COLOR + SIZE_NORMAL + SIZE_TANGENT),
            attr_bone_weights(SIZE_POS + SIZE_UV + SIZE_COLOR + SIZE_NORMAL + SIZE_TANGENT + SIZE_BONE_INDICES),
        ],
    ];
}

use wgpu_attrs::VERTEX_ATTRIBUTES;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vertex_stride_pos_only() {
        // POS: 3 floats = 12 bytes
        assert_eq!(vertex_stride(0), 12);
    }

    #[test]
    fn test_vertex_stride_pos_uv() {
        // POS + UV: 3 + 2 floats = 20 bytes
        assert_eq!(vertex_stride(FORMAT_UV), 20);
    }

    #[test]
    fn test_vertex_stride_pos_color() {
        // POS + COLOR: 3 + 3 floats = 24 bytes
        assert_eq!(vertex_stride(FORMAT_COLOR), 24);
    }

    #[test]
    fn test_vertex_stride_pos_uv_color() {
        // POS + UV + COLOR: 3 + 2 + 3 floats = 32 bytes
        assert_eq!(vertex_stride(FORMAT_UV | FORMAT_COLOR), 32);
    }

    #[test]
    fn test_vertex_stride_pos_normal() {
        // POS + NORMAL: 3 + 3 floats = 24 bytes
        assert_eq!(vertex_stride(FORMAT_NORMAL), 24);
    }

    #[test]
    fn test_vertex_stride_pos_uv_normal() {
        // POS + UV + NORMAL: 3 + 2 + 3 floats = 32 bytes
        assert_eq!(vertex_stride(FORMAT_UV | FORMAT_NORMAL), 32);
    }

    #[test]
    fn test_vertex_stride_pos_color_normal() {
        // POS + COLOR + NORMAL: 3 + 3 + 3 floats = 36 bytes
        assert_eq!(vertex_stride(FORMAT_COLOR | FORMAT_NORMAL), 36);
    }

    #[test]
    fn test_vertex_stride_pos_uv_color_normal() {
        // POS + UV + COLOR + NORMAL: 3 + 2 + 3 + 3 floats = 44 bytes
        assert_eq!(vertex_stride(FORMAT_UV | FORMAT_COLOR | FORMAT_NORMAL), 44);
    }

    #[test]
    fn test_vertex_stride_pos_skinned() {
        // POS + SKINNED: 3 floats + 4 u8 + 4 floats = 12 + 4 + 16 = 32 bytes
        assert_eq!(vertex_stride(FORMAT_SKINNED), 32);
    }

    #[test]
    fn test_vertex_stride_full() {
        // All flags: POS + UV + COLOR + NORMAL + SKINNED
        // 12 + 8 + 12 + 12 + 20 = 64 bytes
        assert_eq!(vertex_stride(FORMAT_ALL), 64);
    }

    #[test]
    fn test_vertex_format_info_names() {
        assert_eq!(VertexFormatInfo::for_format(0).name, "POS");
        assert_eq!(VertexFormatInfo::for_format(1).name, "POS_UV");
        assert_eq!(VertexFormatInfo::for_format(2).name, "POS_COLOR");
        assert_eq!(VertexFormatInfo::for_format(3).name, "POS_UV_COLOR");
        assert_eq!(VertexFormatInfo::for_format(4).name, "POS_NORMAL");
        assert_eq!(VertexFormatInfo::for_format(5).name, "POS_UV_NORMAL");
        assert_eq!(VertexFormatInfo::for_format(6).name, "POS_COLOR_NORMAL");
        assert_eq!(VertexFormatInfo::for_format(7).name, "POS_UV_COLOR_NORMAL");
        assert_eq!(VertexFormatInfo::for_format(8).name, "POS_SKINNED");
        assert_eq!(
            VertexFormatInfo::for_format(15).name,
            "POS_UV_COLOR_NORMAL_SKINNED"
        );
    }

    #[test]
    fn test_vertex_format_info_flags() {
        let format = VertexFormatInfo::for_format(FORMAT_UV | FORMAT_NORMAL);
        assert!(format.has_uv());
        assert!(!format.has_color());
        assert!(format.has_normal());
        assert!(!format.has_skinned());
    }

    #[test]
    fn test_all_32_vertex_formats() {
        // Verify all 32 formats have valid packed strides
        for i in 0..VERTEX_FORMAT_COUNT {
            let info = VertexFormatInfo::for_format(i as u8);
            assert!(
                info.stride >= 8, // Minimum: position only (f16x4) = 8 bytes
                "Format {} has stride {} < 8",
                i,
                info.stride
            );
            assert!(
                info.stride <= 32, // Maximum: full format with tangent packed = 32 bytes
                "Format {} has stride {} > 32",
                i,
                info.stride
            );
        }
    }

    #[test]
    fn test_tangent_format_strides() {
        // Test tangent format strides (packed)
        // Format 20: POS_NORMAL_TANGENT = 8 + 4 + 4 = 16
        assert_eq!(vertex_stride_packed(FORMAT_NORMAL | FORMAT_TANGENT), 16);
        // Format 21: POS_UV_NORMAL_TANGENT = 8 + 4 + 4 + 4 = 20
        assert_eq!(vertex_stride_packed(FORMAT_UV | FORMAT_NORMAL | FORMAT_TANGENT), 20);
        // Format 31: Full with tangent = 8 + 4 + 4 + 4 + 4 + 8 = 32
        assert_eq!(vertex_stride_packed(FORMAT_ALL_WITH_TANGENT), 32);
    }

    #[test]
    fn test_tangent_format_names() {
        assert_eq!(VertexFormatInfo::for_format(20).name, "POS_NORMAL_TANGENT");
        assert_eq!(VertexFormatInfo::for_format(21).name, "POS_UV_NORMAL_TANGENT");
        assert_eq!(VertexFormatInfo::for_format(31).name, "POS_UV_COLOR_NORMAL_TANGENT_SKINNED");
    }

    #[test]
    fn test_tangent_requires_normal_validation() {
        // Formats with tangent but without normal should be invalid
        assert!(!VertexFormatInfo::for_format(16).is_valid()); // POS_TANGENT
        assert!(!VertexFormatInfo::for_format(17).is_valid()); // POS_UV_TANGENT
        assert!(!VertexFormatInfo::for_format(24).is_valid()); // POS_TANGENT_SKINNED

        // Formats with tangent AND normal should be valid
        assert!(VertexFormatInfo::for_format(20).is_valid()); // POS_NORMAL_TANGENT
        assert!(VertexFormatInfo::for_format(21).is_valid()); // POS_UV_NORMAL_TANGENT
        assert!(VertexFormatInfo::for_format(31).is_valid()); // Full with tangent
    }

    #[test]
    fn test_vertex_stride_pos_uv_skinned() {
        // POS + UV + SKINNED: 12 + 8 + 20 = 40 bytes
        assert_eq!(vertex_stride(FORMAT_UV | FORMAT_SKINNED), 40);
    }

    #[test]
    fn test_vertex_stride_pos_color_skinned() {
        // POS + COLOR + SKINNED: 12 + 12 + 20 = 44 bytes
        assert_eq!(vertex_stride(FORMAT_COLOR | FORMAT_SKINNED), 44);
    }

    #[test]
    fn test_vertex_stride_pos_normal_skinned() {
        // POS + NORMAL + SKINNED: 12 + 12 + 20 = 44 bytes
        assert_eq!(vertex_stride(FORMAT_NORMAL | FORMAT_SKINNED), 44);
    }

    #[test]
    fn test_vertex_stride_pos_uv_color_skinned() {
        // POS + UV + COLOR + SKINNED: 12 + 8 + 12 + 20 = 52 bytes
        assert_eq!(vertex_stride(FORMAT_UV | FORMAT_COLOR | FORMAT_SKINNED), 52);
    }

    #[test]
    fn test_vertex_stride_pos_uv_normal_skinned() {
        // POS + UV + NORMAL + SKINNED: 12 + 8 + 12 + 20 = 52 bytes
        assert_eq!(
            vertex_stride(FORMAT_UV | FORMAT_NORMAL | FORMAT_SKINNED),
            52
        );
    }

    #[test]
    fn test_vertex_stride_pos_color_normal_skinned() {
        // POS + COLOR + NORMAL + SKINNED: 12 + 12 + 12 + 20 = 56 bytes
        assert_eq!(
            vertex_stride(FORMAT_COLOR | FORMAT_NORMAL | FORMAT_SKINNED),
            56
        );
    }

    #[test]
    fn test_skinned_vertex_format_info() {
        let format = VertexFormatInfo::for_format(FORMAT_SKINNED);
        assert!(!format.has_uv());
        assert!(!format.has_color());
        assert!(!format.has_normal());
        assert!(format.has_skinned());
        assert_eq!(format.name, "POS_SKINNED");
        assert_eq!(format.stride, 16); // Packed: pos(8) + skinned(8) = 16
    }

    #[test]
    fn test_skinned_full_vertex_format_info() {
        let format = VertexFormatInfo::for_format(FORMAT_ALL);
        assert!(format.has_uv());
        assert!(format.has_color());
        assert!(format.has_normal());
        assert!(format.has_skinned());
        assert_eq!(format.name, "POS_UV_COLOR_NORMAL_SKINNED");
        assert_eq!(format.stride, 28); // Packed: pos(8) + uv(4) + color(4) + normal(4) + skinned(8) = 28
    }

    #[test]
    fn test_all_skinned_vertex_format_strides() {
        // Verify all 8 skinned variants have correct strides
        assert_eq!(vertex_stride(FORMAT_SKINNED), 12 + 20);
        assert_eq!(vertex_stride(FORMAT_UV | FORMAT_SKINNED), 20 + 20);
        assert_eq!(vertex_stride(FORMAT_COLOR | FORMAT_SKINNED), 24 + 20);
        assert_eq!(
            vertex_stride(FORMAT_UV | FORMAT_COLOR | FORMAT_SKINNED),
            32 + 20
        );
        assert_eq!(vertex_stride(FORMAT_NORMAL | FORMAT_SKINNED), 24 + 20);
        assert_eq!(
            vertex_stride(FORMAT_UV | FORMAT_NORMAL | FORMAT_SKINNED),
            32 + 20
        );
        assert_eq!(
            vertex_stride(FORMAT_COLOR | FORMAT_NORMAL | FORMAT_SKINNED),
            36 + 20
        );
        assert_eq!(vertex_stride(FORMAT_ALL), 44 + 20);
    }

    #[test]
    fn test_skinned_format_flags_isolation() {
        // Verify FORMAT_SKINNED doesn't interfere with other flags
        for base_format in 0..8u8 {
            let skinned_format = base_format | FORMAT_SKINNED;
            let base_info = VertexFormatInfo::for_format(base_format);
            let skinned_info = VertexFormatInfo::for_format(skinned_format);

            assert_eq!(base_info.has_uv(), skinned_info.has_uv());
            assert_eq!(base_info.has_color(), skinned_info.has_color());
            assert_eq!(base_info.has_normal(), skinned_info.has_normal());

            assert!(!base_info.has_skinned());
            assert!(skinned_info.has_skinned());

            assert_eq!(skinned_info.stride, base_info.stride + 8); // skinned = u8x4 indices + unorm8x4 weights = 8 bytes
        }
    }

    #[test]
    fn test_format_all_includes_skinned() {
        assert_eq!(
            FORMAT_ALL,
            FORMAT_UV | FORMAT_COLOR | FORMAT_NORMAL | FORMAT_SKINNED
        );
        assert_eq!(FORMAT_ALL, 15);
    }

    #[test]
    fn test_vertex_buffer_layout_pos_only() {
        let info = VertexFormatInfo::for_format(0);
        let layout = info.vertex_buffer_layout();
        assert_eq!(layout.array_stride, 8); // f16x4 (8 bytes) - PACKED format
        assert_eq!(layout.attributes.len(), 1);
        assert_eq!(layout.attributes[0].shader_location, 0);
    }

    #[test]
    fn test_vertex_buffer_layout_full() {
        let info = VertexFormatInfo::for_format(FORMAT_ALL);
        let layout = info.vertex_buffer_layout();
        assert_eq!(layout.array_stride, 28); // Packed: pos(8) + uv(4) + color(4) + normal(4) + skinned(8) = 28
        assert_eq!(layout.attributes.len(), 6);
    }

    #[test]
    fn test_vertex_attribute_offsets_pos_uv_color_normal() {
        let info = VertexFormatInfo::for_format(FORMAT_UV | FORMAT_COLOR | FORMAT_NORMAL);
        let layout = info.vertex_buffer_layout();
        // Packed offsets: pos(0-7), uv(8-11), color(12-15), normal(16-19)
        assert_eq!(layout.attributes[0].offset, 0); // Position at 0
        assert_eq!(layout.attributes[1].offset, 8); // UV at 8
        assert_eq!(layout.attributes[2].offset, 12); // Color at 12
        assert_eq!(layout.attributes[3].offset, 16); // Normal at 16
    }

    #[test]
    fn test_vertex_attribute_shader_locations() {
        let info = VertexFormatInfo::for_format(FORMAT_UV | FORMAT_NORMAL);
        let layout = info.vertex_buffer_layout();
        assert_eq!(layout.attributes[0].shader_location, 0);
        assert_eq!(layout.attributes[1].shader_location, 1);
        assert_eq!(layout.attributes[2].shader_location, 3);
    }
}
