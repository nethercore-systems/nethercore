//! Resource types pending GPU upload

use super::BoneMatrix3x4;
use zx_common::TextureFormat;

// ============================================================================
// GPU Animation Index Tracking (Animation System)
// ============================================================================

/// Tracks where a skeleton's inverse bind matrices live in the global GPU buffer
/// Used to index into @binding(6) all_inverse_bind_mats
#[derive(Debug, Clone, Copy, Default)]
pub struct SkeletonGpuInfo {
    /// Start index in the all_inverse_bind_mats buffer
    pub inverse_bind_offset: u32,
    /// Number of bones in this skeleton
    pub bone_count: u8,
}

/// Tracks where an animation's keyframes live in the global GPU buffer
/// Used to index into @binding(7) all_keyframes
#[derive(Debug, Clone, Copy, Default)]
pub struct KeyframeGpuInfo {
    /// Start index in the all_keyframes buffer
    pub keyframe_base_offset: u32,
    /// Number of bones per frame
    pub bone_count: u8,
    /// Number of frames in the animation
    pub frame_count: u16,
}

/// Source of bone matrices for a draw call (Animation System)
/// Determines whether to read from static GPU buffers or dynamic per-frame uploads
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyframeSource {
    /// Use static keyframes from @binding(7) all_keyframes
    /// Offset points to pre-decoded matrices in the global buffer
    Static { offset: u32 },
    /// Use dynamic bones from @binding(5) immediate_bones (existing bone_matrices buffer)
    /// Offset points to bone matrices appended this frame
    /// For procedural animation, IK, physics-driven bones, etc.
    Immediate { offset: u32 },
}

impl Default for KeyframeSource {
    fn default() -> Self {
        KeyframeSource::Static { offset: 0 }
    }
}

/// Pending skeleton load request (created during init)
#[derive(Debug)]
pub struct PendingSkeleton {
    pub handle: u32,
    pub inverse_bind: Vec<BoneMatrix3x4>,
    pub bone_count: u32,
}

/// Custom bitmap font definition
#[derive(Debug, Clone)]
pub struct Font {
    /// Texture handle for the font atlas
    pub texture: u32,
    /// Width of the texture atlas in pixels
    pub atlas_width: u32,
    /// Height of the texture atlas in pixels
    pub atlas_height: u32,
    /// Width of each glyph in pixels (for fixed-width fonts)
    pub char_width: u8,
    /// Height of each glyph in pixels
    pub char_height: u8,
    /// First codepoint in the font
    pub first_codepoint: u32,
    /// Number of characters in the font
    pub char_count: u32,
    /// Optional per-character widths for variable-width fonts (None = fixed-width)
    pub char_widths: Option<Vec<u8>>,
}

/// Pending texture load request
///
/// Supports both RGBA8 (uncompressed) and BC7 (compressed) texture formats.
#[derive(Debug)]
pub struct PendingTexture {
    pub handle: u32,
    pub width: u32,
    pub height: u32,
    /// Texture format (RGBA8, BC7, or BC7Linear)
    pub format: TextureFormat,
    /// Pixel data (RGBA8) or compressed blocks (BC7)
    pub data: Vec<u8>,
}

/// Pending mesh load request (unpacked f32 data from user)
#[derive(Debug)]
pub struct PendingMesh {
    pub handle: u32,
    pub format: u8,            // Vertex format flags (0-15, NO FORMAT_PACKED)
    pub vertex_data: Vec<f32>, // Unpacked f32 data
    pub index_data: Option<Vec<u16>>,
}

/// Pending packed mesh load request (packed bytes from procedural gen or power users)
#[derive(Debug)]
pub struct PendingMeshPacked {
    pub handle: u32,
    pub format: u8,           // Vertex format flags (0-15, NO FORMAT_PACKED)
    pub vertex_data: Vec<u8>, // Packed bytes (f16, snorm16, unorm8)
    pub index_data: Option<Vec<u16>>,
}

/// Pending keyframe collection load request (created during init)
#[derive(Debug)]
pub struct PendingKeyframes {
    pub handle: u32,
    pub bone_count: u8,
    pub frame_count: u16,
    pub data: Vec<u8>, // Platform format (16 bytes per bone per frame)
}
