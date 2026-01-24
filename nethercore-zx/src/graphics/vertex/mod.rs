//! Vertex format definitions and wgpu buffer layouts
//!
//! This module provides wgpu-specific vertex buffer layout information.
//! Format constants and stride functions are in z-common.

// Re-export format constants from zx-common
pub use zx_common::{
    FORMAT_COLOR, FORMAT_NORMAL, FORMAT_SKINNED, FORMAT_TANGENT, FORMAT_UV, vertex_stride,
    vertex_stride_packed,
};

mod attributes;
#[cfg(test)]
mod tests;

pub use attributes::VERTEX_ATTRIBUTES;

/// All format flags combined (without tangent - tangent requires normal)
pub const FORMAT_ALL: u8 = FORMAT_UV | FORMAT_COLOR | FORMAT_NORMAL | FORMAT_SKINNED;

/// All format flags including tangent (test helper).
#[cfg(test)]
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
            16 => "POS_TANGENT",          // Invalid: tangent requires normal
            17 => "POS_UV_TANGENT",       // Invalid
            18 => "POS_COLOR_TANGENT",    // Invalid
            19 => "POS_UV_COLOR_TANGENT", // Invalid
            20 => "POS_NORMAL_TANGENT",
            21 => "POS_UV_NORMAL_TANGENT",
            22 => "POS_COLOR_NORMAL_TANGENT",
            23 => "POS_UV_COLOR_NORMAL_TANGENT",
            24 => "POS_TANGENT_SKINNED",          // Invalid
            25 => "POS_UV_TANGENT_SKINNED",       // Invalid
            26 => "POS_COLOR_TANGENT_SKINNED",    // Invalid
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
