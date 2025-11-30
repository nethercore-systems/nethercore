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
pub const VERTEX_FORMAT_COUNT: usize = 16;

/// Calculate vertex stride in bytes for a given format
#[inline]
pub const fn vertex_stride(format: u8) -> u32 {
    let mut stride = 3 * 4; // Position: 3 floats = 12 bytes

    if format & FORMAT_UV != 0 {
        stride += 2 * 4; // UV: 2 floats = 8 bytes
    }
    if format & FORMAT_COLOR != 0 {
        stride += 3 * 4; // Color: 3 floats = 12 bytes
    }
    if format & FORMAT_NORMAL != 0 {
        stride += 3 * 4; // Normal: 3 floats = 12 bytes
    }
    if format & FORMAT_SKINNED != 0 {
        stride += 4 + 4 * 4; // Bone indices (4 u8 = 4 bytes) + weights (4 floats = 16 bytes) = 20 bytes
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
    /// Get vertex format info for a format index
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
            stride: vertex_stride(format),
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

    /// Builds vertex attributes for a format, returning a static slice.
    ///
    /// Shader locations are assigned in order:
    /// - Location 0: Position (always present, Float32x3)
    /// - Location 1: UV (if FORMAT_UV, Float32x2)
    /// - Location 2: Color (if FORMAT_COLOR, Float32x3)
    /// - Location 3: Normal (if FORMAT_NORMAL, Float32x3)
    /// - Location 4: Bone indices (if FORMAT_SKINNED, Uint8x4)
    /// - Location 5: Bone weights (if FORMAT_SKINNED, Float32x4)
    fn build_attributes(format: u8) -> &'static [wgpu::VertexAttribute] {
        // Pre-computed attribute arrays for each format
        // Position is always at location 0
        match format {
            0 => &[
                // POS only
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x3,
                    offset: 0,
                    shader_location: 0,
                },
            ],
            1 => &[
                // POS_UV
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x3,
                    offset: 0,
                    shader_location: 0,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x2,
                    offset: 12,
                    shader_location: 1,
                },
            ],
            2 => &[
                // POS_COLOR
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x3,
                    offset: 0,
                    shader_location: 0,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x3,
                    offset: 12,
                    shader_location: 2,
                },
            ],
            3 => &[
                // POS_UV_COLOR
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x3,
                    offset: 0,
                    shader_location: 0,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x2,
                    offset: 12,
                    shader_location: 1,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x3,
                    offset: 20,
                    shader_location: 2,
                },
            ],
            4 => &[
                // POS_NORMAL
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x3,
                    offset: 0,
                    shader_location: 0,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x3,
                    offset: 12,
                    shader_location: 3,
                },
            ],
            5 => &[
                // POS_UV_NORMAL
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x3,
                    offset: 0,
                    shader_location: 0,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x2,
                    offset: 12,
                    shader_location: 1,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x3,
                    offset: 20,
                    shader_location: 3,
                },
            ],
            6 => &[
                // POS_COLOR_NORMAL
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x3,
                    offset: 0,
                    shader_location: 0,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x3,
                    offset: 12,
                    shader_location: 2,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x3,
                    offset: 24,
                    shader_location: 3,
                },
            ],
            7 => &[
                // POS_UV_COLOR_NORMAL
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x3,
                    offset: 0,
                    shader_location: 0,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x2,
                    offset: 12,
                    shader_location: 1,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x3,
                    offset: 20,
                    shader_location: 2,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x3,
                    offset: 32,
                    shader_location: 3,
                },
            ],
            8 => &[
                // POS_SKINNED
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x3,
                    offset: 0,
                    shader_location: 0,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Uint8x4,
                    offset: 12,
                    shader_location: 4,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x4,
                    offset: 16,
                    shader_location: 5,
                },
            ],
            9 => &[
                // POS_UV_SKINNED
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x3,
                    offset: 0,
                    shader_location: 0,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x2,
                    offset: 12,
                    shader_location: 1,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Uint8x4,
                    offset: 20,
                    shader_location: 4,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x4,
                    offset: 24,
                    shader_location: 5,
                },
            ],
            10 => &[
                // POS_COLOR_SKINNED
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x3,
                    offset: 0,
                    shader_location: 0,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x3,
                    offset: 12,
                    shader_location: 2,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Uint8x4,
                    offset: 24,
                    shader_location: 4,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x4,
                    offset: 28,
                    shader_location: 5,
                },
            ],
            11 => &[
                // POS_UV_COLOR_SKINNED
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x3,
                    offset: 0,
                    shader_location: 0,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x2,
                    offset: 12,
                    shader_location: 1,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x3,
                    offset: 20,
                    shader_location: 2,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Uint8x4,
                    offset: 32,
                    shader_location: 4,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x4,
                    offset: 36,
                    shader_location: 5,
                },
            ],
            12 => &[
                // POS_NORMAL_SKINNED
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x3,
                    offset: 0,
                    shader_location: 0,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x3,
                    offset: 12,
                    shader_location: 3,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Uint8x4,
                    offset: 24,
                    shader_location: 4,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x4,
                    offset: 28,
                    shader_location: 5,
                },
            ],
            13 => &[
                // POS_UV_NORMAL_SKINNED
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x3,
                    offset: 0,
                    shader_location: 0,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x2,
                    offset: 12,
                    shader_location: 1,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x3,
                    offset: 20,
                    shader_location: 3,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Uint8x4,
                    offset: 32,
                    shader_location: 4,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x4,
                    offset: 36,
                    shader_location: 5,
                },
            ],
            14 => &[
                // POS_COLOR_NORMAL_SKINNED
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x3,
                    offset: 0,
                    shader_location: 0,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x3,
                    offset: 12,
                    shader_location: 2,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x3,
                    offset: 24,
                    shader_location: 3,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Uint8x4,
                    offset: 36,
                    shader_location: 4,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x4,
                    offset: 40,
                    shader_location: 5,
                },
            ],
            15 => &[
                // POS_UV_COLOR_NORMAL_SKINNED
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x3,
                    offset: 0,
                    shader_location: 0,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x2,
                    offset: 12,
                    shader_location: 1,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x3,
                    offset: 20,
                    shader_location: 2,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x3,
                    offset: 32,
                    shader_location: 3,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Uint8x4,
                    offset: 44,
                    shader_location: 4,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x4,
                    offset: 48,
                    shader_location: 5,
                },
            ],
            _ => &[
                // Fallback: POS only
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x3,
                    offset: 0,
                    shader_location: 0,
                },
            ],
        }
    }
}

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
        assert_eq!(VertexFormatInfo::for_format(15).name, "POS_UV_COLOR_NORMAL_SKINNED");
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
        // Verify all 16 formats have valid strides
        for i in 0..VERTEX_FORMAT_COUNT {
            let info = VertexFormatInfo::for_format(i as u8);
            assert!(info.stride >= 12, "Format {} has stride {} < 12", i, info.stride);
            assert!(info.stride <= 64, "Format {} has stride {} > 64", i, info.stride);
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
        assert_eq!(vertex_stride(FORMAT_UV | FORMAT_NORMAL | FORMAT_SKINNED), 52);
    }

    #[test]
    fn test_vertex_stride_pos_color_normal_skinned() {
        // POS + COLOR + NORMAL + SKINNED: 12 + 12 + 12 + 20 = 56 bytes
        assert_eq!(vertex_stride(FORMAT_COLOR | FORMAT_NORMAL | FORMAT_SKINNED), 56);
    }

    #[test]
    fn test_skinned_vertex_format_info() {
        let format = VertexFormatInfo::for_format(FORMAT_SKINNED);
        assert!(!format.has_uv());
        assert!(!format.has_color());
        assert!(!format.has_normal());
        assert!(format.has_skinned());
        assert_eq!(format.name, "POS_SKINNED");
        assert_eq!(format.stride, 32); // 12 + 20
    }

    #[test]
    fn test_skinned_full_vertex_format_info() {
        let format = VertexFormatInfo::for_format(FORMAT_ALL);
        assert!(format.has_uv());
        assert!(format.has_color());
        assert!(format.has_normal());
        assert!(format.has_skinned());
        assert_eq!(format.name, "POS_UV_COLOR_NORMAL_SKINNED");
        assert_eq!(format.stride, 64); // 12 + 8 + 12 + 12 + 20
    }

    #[test]
    fn test_all_skinned_vertex_format_strides() {
        // Verify all 8 skinned variants have correct strides
        assert_eq!(vertex_stride(FORMAT_SKINNED), 12 + 20);
        assert_eq!(vertex_stride(FORMAT_UV | FORMAT_SKINNED), 20 + 20);
        assert_eq!(vertex_stride(FORMAT_COLOR | FORMAT_SKINNED), 24 + 20);
        assert_eq!(vertex_stride(FORMAT_UV | FORMAT_COLOR | FORMAT_SKINNED), 32 + 20);
        assert_eq!(vertex_stride(FORMAT_NORMAL | FORMAT_SKINNED), 24 + 20);
        assert_eq!(vertex_stride(FORMAT_UV | FORMAT_NORMAL | FORMAT_SKINNED), 32 + 20);
        assert_eq!(vertex_stride(FORMAT_COLOR | FORMAT_NORMAL | FORMAT_SKINNED), 36 + 20);
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

            assert_eq!(skinned_info.stride, base_info.stride + 20);
        }
    }

    #[test]
    fn test_format_all_includes_skinned() {
        assert_eq!(FORMAT_ALL, FORMAT_UV | FORMAT_COLOR | FORMAT_NORMAL | FORMAT_SKINNED);
        assert_eq!(FORMAT_ALL, 15);
    }

    #[test]
    fn test_vertex_buffer_layout_pos_only() {
        let info = VertexFormatInfo::for_format(0);
        let layout = info.vertex_buffer_layout();
        assert_eq!(layout.array_stride, 12);
        assert_eq!(layout.attributes.len(), 1);
        assert_eq!(layout.attributes[0].shader_location, 0);
    }

    #[test]
    fn test_vertex_buffer_layout_full() {
        let info = VertexFormatInfo::for_format(FORMAT_ALL);
        let layout = info.vertex_buffer_layout();
        assert_eq!(layout.array_stride, 64);
        assert_eq!(layout.attributes.len(), 6);
    }

    #[test]
    fn test_vertex_attribute_offsets_pos_uv_color_normal() {
        let info = VertexFormatInfo::for_format(FORMAT_UV | FORMAT_COLOR | FORMAT_NORMAL);
        let layout = info.vertex_buffer_layout();
        assert_eq!(layout.attributes[0].offset, 0);
        assert_eq!(layout.attributes[1].offset, 12);
        assert_eq!(layout.attributes[2].offset, 20);
        assert_eq!(layout.attributes[3].offset, 32);
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
