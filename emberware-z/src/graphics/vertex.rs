//! Vertex format definitions and utilities
//!
//! Defines vertex format flags, stride calculations, and vertex buffer layouts
//! for all 16 vertex format permutations (8 base + 8 skinned variants).

/// Vertex format flag: Has UV coordinates (2 floats)
pub const FORMAT_UV: u8 = 1;
/// Vertex format flag: Has per-vertex color (RGB, 3 floats)
pub const FORMAT_COLOR: u8 = 2;
/// Vertex format flag: Has normals (3 floats)
pub const FORMAT_NORMAL: u8 = 4;
/// Vertex format flag: Has bone indices/weights for skinning (4 u8 + 4 floats)
pub const FORMAT_SKINNED: u8 = 8;

/// All format flags combined
pub const FORMAT_ALL: u8 = FORMAT_UV | FORMAT_COLOR | FORMAT_NORMAL | FORMAT_SKINNED;

/// Number of vertex format permutations (16: 0-15)
/// GPU always uses packed vertex formats (f16, snorm16, unorm8).
pub const VERTEX_FORMAT_COUNT: usize = 16;

/// Calculate vertex stride in bytes for unpacked f32 format (convenience API)
///
/// Used when game code provides Vec<f32> vertex data that needs packing before GPU upload.
/// Format values are 0-15 (base format).
#[inline]
pub const fn vertex_stride(format: u8) -> u32 {
    // Position: Float32x3 (12 bytes)
    let mut stride = 12;

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

/// Calculate vertex stride in bytes for packed format (used by power user API)
///
/// All formats are packed since GPU buffers only use packed data.
/// Format values are 0-15 (base format, no FORMAT_PACKED flag).
#[inline]
pub const fn vertex_stride_packed(format: u8) -> u32 {
    // Position: Float16x4 (8 bytes)
    let mut stride = 8;

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
    /// Get vertex format info for a format index (0-15)
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

    /// Check if this format has skinning data
    #[inline]
    pub const fn has_skinned(&self) -> bool {
        self.format & FORMAT_SKINNED != 0
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

/// Attribute sizes in bytes for offset calculation (packed formats - GPU only)
const SIZE_POS: u64 = 8; // Float16x4 (padded for alignment)
const SIZE_UV: u64 = 4; // Unorm16x2
const SIZE_COLOR: u64 = 4; // Unorm8x4
const SIZE_NORMAL: u64 = 4; // Octahedral u32
const SIZE_BONE_INDICES: u64 = 4; // Uint8x4
const SIZE_BONE_WEIGHTS: u64 = 4; // Unorm8x4

/// Shader locations for each attribute type
const LOC_POS: u32 = 0;
const LOC_UV: u32 = 1;
const LOC_COLOR: u32 = 2;
const LOC_NORMAL: u32 = 3;
const LOC_BONE_INDICES: u32 = 4;
const LOC_BONE_WEIGHTS: u32 = 5;

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

/// Pre-computed vertex attribute arrays for all 16 formats.
///
/// Vertex layout order: Position → UV → Color → Normal → Bone Indices → Bone Weights
/// Each attribute is only present if its corresponding flag is set.
/// Offsets are computed based on which attributes precede each one.
static VERTEX_ATTRIBUTES: [&[wgpu::VertexAttribute]; 16] = [
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
];

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
    fn test_all_16_vertex_formats() {
        // Verify all 16 formats have valid packed strides
        for i in 0..VERTEX_FORMAT_COUNT {
            let info = VertexFormatInfo::for_format(i as u8);
            assert!(
                info.stride >= 8, // Minimum: position only (f16x4) = 8 bytes
                "Format {} has stride {} < 8",
                i,
                info.stride
            );
            assert!(
                info.stride <= 44, // Maximum: full format packed = 44 bytes
                "Format {} has stride {} > 44",
                i,
                info.stride
            );
        }
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
