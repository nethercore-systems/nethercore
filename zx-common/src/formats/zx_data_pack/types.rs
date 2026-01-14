//! ZX data pack asset types
//!
//! Contains all asset type definitions (textures, meshes, sounds, etc.)
//! and related enums/helpers.

use bitcode::{Decode, Encode};
use nethercore_shared::math::BoneMatrix3x4;
use serde::{Deserialize, Serialize};

/// Texture compression format
///
/// Determined by the `compress_textures` flag in nether.toml at pack time.
/// - compress_textures = false: RGBA8 — pixel-perfect, full alpha
/// - compress_textures = true: BC7 — 4× compression, stipple transparency
/// - Normal maps: BC5 — 2-channel RG, optimal for tangent-space normals
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, Encode, Decode)]
pub enum TextureFormat {
    /// Uncompressed RGBA8 (4 bytes per pixel)
    /// Used for Mode 0 (Lambert) and procedural textures
    #[default]
    Rgba8,

    /// BC7 compressed (8 bits per pixel, linear color space)
    /// Used for all textures in modes 1-3
    Bc7,

    /// BC5 compressed (8 bits per pixel, 2-channel RG)
    /// Used for normal maps — Z reconstructed in shader: z = sqrt(1 - x² - y²)
    Bc5,
}

impl TextureFormat {
    /// Check if this format is BC7 (compressed color)
    pub fn is_bc7(&self) -> bool {
        matches!(self, TextureFormat::Bc7)
    }

    /// Check if this format is BC5 (compressed 2-channel)
    pub fn is_bc5(&self) -> bool {
        matches!(self, TextureFormat::Bc5)
    }

    /// Check if this format is compressed (BC5 or BC7)
    pub fn is_compressed(&self) -> bool {
        matches!(self, TextureFormat::Bc7 | TextureFormat::Bc5)
    }

    /// Calculate data size for given dimensions
    pub fn data_size(&self, width: u16, height: u16) -> usize {
        let w = width as usize;
        let h = height as usize;

        match self {
            TextureFormat::Rgba8 => w * h * 4,
            TextureFormat::Bc7 | TextureFormat::Bc5 => {
                // Both BC7 and BC5 use 16 bytes per 4x4 block
                let blocks_x = w.div_ceil(4);
                let blocks_y = h.div_ceil(4);
                blocks_x * blocks_y * 16
            }
        }
    }

    /// Get wgpu-compatible format name (for debugging/logging)
    pub fn wgpu_format_name(&self) -> &'static str {
        match self {
            TextureFormat::Rgba8 => "Rgba8Unorm",
            TextureFormat::Bc7 => "Bc7RgbaUnorm",
            TextureFormat::Bc5 => "Bc5RgUnorm",
        }
    }
}

/// Packed texture (RGBA8 or BC7 compressed)
///
/// Ready for direct GPU upload via `wgpu::Queue::write_texture()`.
/// Format is determined by the `compress_textures` flag in nether.toml at pack time.
#[derive(Debug, Clone, Serialize, Deserialize, Encode, Decode)]
pub struct PackedTexture {
    /// Asset ID (e.g., "player_idle", "stage1_tileset")
    pub id: String,

    /// Texture width in pixels (max 65535)
    pub width: u16,

    /// Texture height in pixels (max 65535)
    pub height: u16,

    /// Texture format (RGBA8 or BC7)
    #[serde(default)]
    pub format: TextureFormat,

    /// Pixel data (RGBA8) or compressed blocks (BC7)
    pub data: Vec<u8>,
}

impl PackedTexture {
    /// Create a new RGBA8 packed texture
    pub fn new(id: impl Into<String>, width: u32, height: u32, data: Vec<u8>) -> Self {
        Self {
            id: id.into(),
            width: width as u16,
            height: height as u16,
            format: TextureFormat::Rgba8,
            data,
        }
    }

    /// Create a new packed texture with explicit format
    pub fn with_format(
        id: impl Into<String>,
        width: u16,
        height: u16,
        format: TextureFormat,
        data: Vec<u8>,
    ) -> Self {
        Self {
            id: id.into(),
            width,
            height,
            format,
            data,
        }
    }

    /// Get expected data size based on format
    pub fn expected_size(&self) -> usize {
        self.format.data_size(self.width, self.height)
    }

    /// Validate that data size matches dimensions and format
    pub fn validate(&self) -> bool {
        self.data.len() == self.expected_size()
    }

    /// Check if texture is BC7 compressed
    pub fn is_bc7(&self) -> bool {
        self.format.is_bc7()
    }

    /// Get dimensions as u32 tuple (for wgpu compatibility)
    pub fn dimensions_u32(&self) -> (u32, u32) {
        (self.width as u32, self.height as u32)
    }
}

/// Packed mesh (GPU-ready vertices + indices)
///
/// Vertices are packed according to the vertex format flags (see asset-pipeline.md).
/// Ready for direct GPU buffer upload.
#[derive(Debug, Clone, Serialize, Deserialize, Encode, Decode)]
pub struct PackedMesh {
    /// Asset ID (e.g., "player_mesh", "stage1")
    pub id: String,

    /// Vertex format flags (0-15):
    /// - Bit 0: Has UV coordinates
    /// - Bit 1: Has vertex colors
    /// - Bit 2: Has normals
    /// - Bit 3: Has bone weights (skinned)
    pub format: u8,

    /// Number of vertices
    pub vertex_count: u32,

    /// Number of indices
    pub index_count: u32,

    /// GPU-ready packed vertex data
    pub vertex_data: Vec<u8>,

    /// Index buffer (u16 indices)
    pub index_data: Vec<u16>,
}

impl PackedMesh {
    /// Check if mesh has UV coordinates
    pub fn has_uv(&self) -> bool {
        self.format & 0x01 != 0
    }

    /// Check if mesh has vertex colors
    pub fn has_color(&self) -> bool {
        self.format & 0x02 != 0
    }

    /// Check if mesh has normals
    pub fn has_normal(&self) -> bool {
        self.format & 0x04 != 0
    }

    /// Check if mesh is skinned (has bone weights)
    pub fn is_skinned(&self) -> bool {
        self.format & 0x08 != 0
    }

    /// Get the stride (bytes per vertex) for this format
    ///
    /// Returns the packed stride matching the GPU vertex layout:
    /// - Position: f16x4 = 8 bytes
    /// - UV: unorm16x2 = 4 bytes
    /// - Color: unorm8x4 = 4 bytes
    /// - Normal: octahedral u32 = 4 bytes
    /// - Skinning: u8x4 + unorm8x4 = 8 bytes
    pub fn stride(&self) -> usize {
        crate::vertex_stride_packed(self.format) as usize
    }
}

/// Packed skeleton (inverse bind matrices only)
///
/// Contains ONLY the inverse bind matrices needed for GPU skinning.
/// Bone names, hierarchy, and rest pose belong in WASM memory (generated
/// by nether-export as Rust const arrays).
#[derive(Debug, Clone, Serialize, Deserialize, Encode, Decode)]
pub struct PackedSkeleton {
    /// Asset ID (e.g., "player_skeleton", "enemy_rig")
    pub id: String,

    /// Number of bones
    pub bone_count: u32,

    /// Inverse bind matrices (one per bone, GPU-ready)
    pub inverse_bind_matrices: Vec<BoneMatrix3x4>,
}

impl PackedSkeleton {
    /// Create a new packed skeleton
    pub fn new(id: impl Into<String>, inverse_bind_matrices: Vec<BoneMatrix3x4>) -> Self {
        let bone_count = inverse_bind_matrices.len() as u32;
        Self {
            id: id.into(),
            bone_count,
            inverse_bind_matrices,
        }
    }

    /// Validate that bone count matches matrices
    pub fn validate(&self) -> bool {
        self.inverse_bind_matrices.len() == self.bone_count as usize
    }
}

/// Packed keyframe collection (animation clip)
///
/// Contains keyframe data in platform format (16 bytes per bone per frame).
/// Data is stored in ROM and accessed via `rom_keyframes()` or `keyframes_load()`.
#[derive(Debug, Clone, Serialize, Deserialize, Encode, Decode)]
pub struct PackedKeyframes {
    /// Asset ID (e.g., "walk", "run", "idle")
    pub id: String,

    /// Number of bones per frame
    pub bone_count: u8,

    /// Number of frames
    pub frame_count: u16,

    /// Raw platform format data (frame_count × bone_count × 16 bytes)
    pub data: Vec<u8>,
}

impl PackedKeyframes {
    /// Create a new packed keyframes collection
    pub fn new(id: impl Into<String>, bone_count: u8, frame_count: u16, data: Vec<u8>) -> Self {
        Self {
            id: id.into(),
            bone_count,
            frame_count,
            data,
        }
    }

    /// Validate that data size matches header
    pub fn validate(&self) -> bool {
        let expected = self.bone_count as usize * self.frame_count as usize * 16;
        self.bone_count > 0 && self.frame_count > 0 && self.data.len() == expected
    }

    /// Get frame data as a slice
    pub fn frame_data(&self, frame_index: u16) -> Option<&[u8]> {
        if frame_index >= self.frame_count {
            return None;
        }
        let frame_size = self.bone_count as usize * 16;
        let start = frame_index as usize * frame_size;
        let end = start + frame_size;
        Some(&self.data[start..end])
    }
}

/// Packed font (bitmap atlas + glyph metrics)
///
/// The atlas texture is embedded in the font asset. When loaded via `rom_font()`,
/// the host uploads the atlas to VRAM internally.
#[derive(Debug, Clone, Serialize, Deserialize, Encode, Decode)]
pub struct PackedFont {
    /// Asset ID (e.g., "pixel_font", "title_font")
    pub id: String,

    /// Atlas texture width in pixels
    pub atlas_width: u32,

    /// Atlas texture height in pixels
    pub atlas_height: u32,

    /// RGBA8 bitmap atlas data
    pub atlas_data: Vec<u8>,

    /// Line height in pixels
    pub line_height: f32,

    /// Baseline offset from top in pixels
    pub baseline: f32,

    /// Glyph metrics
    pub glyphs: Vec<PackedGlyph>,
}

impl PackedFont {
    /// Find glyph by codepoint
    pub fn find_glyph(&self, codepoint: u32) -> Option<&PackedGlyph> {
        self.glyphs.iter().find(|g| g.codepoint == codepoint)
    }

    /// Get expected atlas data size
    pub fn expected_atlas_size(&self) -> usize {
        (self.atlas_width * self.atlas_height * 4) as usize
    }
}

/// Glyph metrics within a font atlas
#[derive(Debug, Clone, Serialize, Deserialize, Encode, Decode)]
pub struct PackedGlyph {
    /// Unicode codepoint
    pub codepoint: u32,

    /// X position in atlas (pixels)
    pub x: u16,

    /// Y position in atlas (pixels)
    pub y: u16,

    /// Width in atlas (pixels)
    pub w: u16,

    /// Height in atlas (pixels)
    pub h: u16,

    /// Horizontal render offset
    pub x_offset: f32,

    /// Vertical render offset
    pub y_offset: f32,

    /// Horizontal advance (to next glyph)
    pub advance: f32,
}

/// Packed sound (PCM audio data)
///
/// Audio is stored as 22050Hz mono i16 PCM. Sample count is derived from data.len().
#[derive(Debug, Clone, Serialize, Deserialize, Encode, Decode)]
pub struct PackedSound {
    /// Asset ID (e.g., "jump", "explosion", "bgm_level1")
    pub id: String,

    /// PCM audio samples (22050Hz mono i16)
    pub data: Vec<i16>,
}

impl PackedSound {
    /// Create a new packed sound
    pub fn new(id: impl Into<String>, data: Vec<i16>) -> Self {
        Self {
            id: id.into(),
            data,
        }
    }

    /// Get sample count
    pub fn sample_count(&self) -> usize {
        self.data.len()
    }

    /// Get duration in seconds (at 22050Hz)
    pub fn duration_seconds(&self) -> f32 {
        self.data.len() as f32 / 22050.0
    }
}

/// Packed raw data (levels, dialogue, custom formats)
///
/// Opaque byte data that the game interprets. Use for levels, dialogue,
/// configuration, or any custom format.
#[derive(Debug, Clone, Serialize, Deserialize, Encode, Decode)]
pub struct PackedData {
    /// Asset ID (e.g., "level1", "dialogue_en", "config")
    pub id: String,

    /// Raw byte data
    pub data: Vec<u8>,
}

impl PackedData {
    /// Create new packed data
    pub fn new(id: impl Into<String>, data: Vec<u8>) -> Self {
        Self {
            id: id.into(),
            data,
        }
    }

    /// Get data size in bytes
    pub fn size(&self) -> usize {
        self.data.len()
    }
}

/// Tracker format (XM or IT)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Encode, Decode, Default)]
#[repr(u8)]
pub enum TrackerFormat {
    /// Extended Module format (FastTracker II)
    #[default]
    Xm = 0,
    /// Impulse Tracker format
    It = 1,
}

/// Packed tracker module (XM/IT pattern data + sample mapping)
///
/// Contains tracker pattern data with sample references resolved at load time.
/// Samples are loaded separately via `rom_sound()` and mapped by ID.
#[derive(Debug, Clone, Serialize, Deserialize, Encode, Decode)]
pub struct PackedTracker {
    /// Asset ID (e.g., "level1_music", "boss_theme")
    pub id: String,

    /// Tracker format (XM or IT)
    #[serde(default)]
    pub format: TrackerFormat,

    /// Pattern data (samples stripped, patterns + instrument metadata only)
    /// - XM: NCXM format (minimal XM)
    /// - IT: NCIT format (minimal IT)
    pub pattern_data: Vec<u8>,

    /// Instrument index -> ROM sample ID mapping
    /// e.g., ["kick", "snare", "bass"] means:
    /// - Instrument 0 uses ROM sample "kick"
    /// - Instrument 1 uses ROM sample "snare"
    /// - Instrument 2 uses ROM sample "bass"
    pub sample_ids: Vec<String>,
}

impl PackedTracker {
    /// Create a new packed tracker
    pub fn new(
        id: impl Into<String>,
        format: TrackerFormat,
        pattern_data: Vec<u8>,
        sample_ids: Vec<String>,
    ) -> Self {
        Self {
            id: id.into(),
            format,
            pattern_data,
            sample_ids,
        }
    }

    /// Get the number of instruments/samples
    pub fn instrument_count(&self) -> usize {
        self.sample_ids.len()
    }

    /// Get pattern data size in bytes
    pub fn pattern_data_size(&self) -> usize {
        self.pattern_data.len()
    }
}
