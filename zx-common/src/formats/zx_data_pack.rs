//! Nethercore ZX data pack format
//!
//! Contains GPU-ready asset data bundled with the ROM. Assets loaded via `rom_*` FFI
//! go directly to VRAM/audio memory on the host, bypassing WASM linear memory.
//!
//! # Design Principles
//!
//! - **STRICTLY GPU-ready POD data only** — No metadata that belongs in game code
//! - **String IDs** — Assets referenced by name for ergonomics
//! - **Hash lookup** — FxHash internally for O(1) runtime access
//! - **Console-specific** — Prevents mixing data between consoles

use bitcode::{Decode, Encode};
use hashbrown::HashMap;
use serde::{Deserialize, Serialize};
use std::sync::OnceLock;

/// Texture compression format
///
/// Determined by render mode:
/// - Mode 0 (Unlit): RGBA8 — pixel-perfect, full alpha
/// - Mode 1-3 (Matcap/MRBP/SSBP): BC7 — 4× compression, stipple transparency
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, Encode, Decode)]
pub enum TextureFormat {
    /// Uncompressed RGBA8 (4 bytes per pixel)
    /// Used for Mode 0 (Unlit) and procedural textures
    #[default]
    Rgba8,

    /// BC7 compressed (8 bits per pixel, linear color space)
    /// Used for all textures in modes 1-3
    Bc7,
}

impl TextureFormat {
    /// Check if this format is BC7 (compressed)
    pub fn is_bc7(&self) -> bool {
        matches!(self, TextureFormat::Bc7)
    }

    /// Calculate data size for given dimensions
    pub fn data_size(&self, width: u16, height: u16) -> usize {
        let w = width as usize;
        let h = height as usize;

        match self {
            TextureFormat::Rgba8 => w * h * 4,
            TextureFormat::Bc7 => {
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
        }
    }
}

use nethercore_shared::math::BoneMatrix3x4;

/// Nethercore ZX data pack
///
/// Contains all bundled assets for an Nethercore ZX ROM. Assets are stored
/// in GPU-ready formats and loaded directly to VRAM/audio memory.
#[derive(Debug, Clone, Default, Serialize, Deserialize, Encode, Decode)]
pub struct ZXDataPack {
    /// Textures (RGBA8 pixel data)
    pub textures: Vec<PackedTexture>,

    /// Meshes (GPU-ready packed vertices + indices)
    pub meshes: Vec<PackedMesh>,

    /// Skeletons (inverse bind matrices only — GPU-ready)
    pub skeletons: Vec<PackedSkeleton>,

    /// Keyframe collections (animation clips)
    pub keyframes: Vec<PackedKeyframes>,

    /// Fonts (bitmap atlas + glyph metrics)
    pub fonts: Vec<PackedFont>,

    /// Sounds (PCM audio data)
    pub sounds: Vec<PackedSound>,

    /// Raw data (levels, dialogue, custom formats)
    pub data: Vec<PackedData>,

    /// Tracker modules (XM pattern data + sample mappings)
    pub trackers: Vec<PackedTracker>,

    // ========================================================================
    // Index caches for O(1) lookup (built lazily on first access)
    // ========================================================================
    #[serde(skip)]
    #[bitcode(skip)]
    texture_index: OnceLock<HashMap<String, usize>>,

    #[serde(skip)]
    #[bitcode(skip)]
    mesh_index: OnceLock<HashMap<String, usize>>,

    #[serde(skip)]
    #[bitcode(skip)]
    skeleton_index: OnceLock<HashMap<String, usize>>,

    #[serde(skip)]
    #[bitcode(skip)]
    keyframes_index: OnceLock<HashMap<String, usize>>,

    #[serde(skip)]
    #[bitcode(skip)]
    font_index: OnceLock<HashMap<String, usize>>,

    #[serde(skip)]
    #[bitcode(skip)]
    sound_index: OnceLock<HashMap<String, usize>>,

    #[serde(skip)]
    #[bitcode(skip)]
    data_index: OnceLock<HashMap<String, usize>>,

    #[serde(skip)]
    #[bitcode(skip)]
    tracker_index: OnceLock<HashMap<String, usize>>,
}

impl ZXDataPack {
    /// Create an empty data pack
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a data pack with the given assets
    pub fn with_assets(
        textures: Vec<PackedTexture>,
        meshes: Vec<PackedMesh>,
        skeletons: Vec<PackedSkeleton>,
        keyframes: Vec<PackedKeyframes>,
        fonts: Vec<PackedFont>,
        sounds: Vec<PackedSound>,
        data: Vec<PackedData>,
        trackers: Vec<PackedTracker>,
    ) -> Self {
        Self {
            textures,
            meshes,
            skeletons,
            keyframes,
            fonts,
            sounds,
            data,
            trackers,
            // Index caches will be lazily initialized on first lookup
            texture_index: OnceLock::new(),
            mesh_index: OnceLock::new(),
            skeleton_index: OnceLock::new(),
            keyframes_index: OnceLock::new(),
            font_index: OnceLock::new(),
            sound_index: OnceLock::new(),
            data_index: OnceLock::new(),
            tracker_index: OnceLock::new(),
        }
    }

    /// Check if the data pack is empty
    pub fn is_empty(&self) -> bool {
        self.textures.is_empty()
            && self.meshes.is_empty()
            && self.skeletons.is_empty()
            && self.keyframes.is_empty()
            && self.fonts.is_empty()
            && self.sounds.is_empty()
            && self.data.is_empty()
            && self.trackers.is_empty()
    }

    /// Get total asset count
    pub fn asset_count(&self) -> usize {
        self.textures.len()
            + self.meshes.len()
            + self.skeletons.len()
            + self.keyframes.len()
            + self.fonts.len()
            + self.sounds.len()
            + self.data.len()
            + self.trackers.len()
    }

    /// Find a texture by ID (O(1) lookup via lazy-initialized hash index)
    pub fn find_texture(&self, id: &str) -> Option<&PackedTexture> {
        let index = self
            .texture_index
            .get_or_init(|| build_index(&self.textures, |t| &t.id));
        index.get(id).map(|&i| &self.textures[i])
    }

    /// Find a mesh by ID (O(1) lookup via lazy-initialized hash index)
    pub fn find_mesh(&self, id: &str) -> Option<&PackedMesh> {
        let index = self
            .mesh_index
            .get_or_init(|| build_index(&self.meshes, |m| &m.id));
        index.get(id).map(|&i| &self.meshes[i])
    }

    /// Find a skeleton by ID (O(1) lookup via lazy-initialized hash index)
    pub fn find_skeleton(&self, id: &str) -> Option<&PackedSkeleton> {
        let index = self
            .skeleton_index
            .get_or_init(|| build_index(&self.skeletons, |s| &s.id));
        index.get(id).map(|&i| &self.skeletons[i])
    }

    /// Find a keyframe collection by ID (O(1) lookup via lazy-initialized hash index)
    pub fn find_keyframes(&self, id: &str) -> Option<&PackedKeyframes> {
        let index = self
            .keyframes_index
            .get_or_init(|| build_index(&self.keyframes, |k| &k.id));
        index.get(id).map(|&i| &self.keyframes[i])
    }

    /// Find a font by ID (O(1) lookup via lazy-initialized hash index)
    pub fn find_font(&self, id: &str) -> Option<&PackedFont> {
        let index = self
            .font_index
            .get_or_init(|| build_index(&self.fonts, |f| &f.id));
        index.get(id).map(|&i| &self.fonts[i])
    }

    /// Find a sound by ID (O(1) lookup via lazy-initialized hash index)
    pub fn find_sound(&self, id: &str) -> Option<&PackedSound> {
        let index = self
            .sound_index
            .get_or_init(|| build_index(&self.sounds, |s| &s.id));
        index.get(id).map(|&i| &self.sounds[i])
    }

    /// Find raw data by ID (O(1) lookup via lazy-initialized hash index)
    pub fn find_data(&self, id: &str) -> Option<&PackedData> {
        let index = self
            .data_index
            .get_or_init(|| build_index(&self.data, |d| &d.id));
        index.get(id).map(|&i| &self.data[i])
    }

    /// Find a tracker by ID (O(1) lookup via lazy-initialized hash index)
    pub fn find_tracker(&self, id: &str) -> Option<&PackedTracker> {
        let index = self
            .tracker_index
            .get_or_init(|| build_index(&self.trackers, |t| &t.id));
        index.get(id).map(|&i| &self.trackers[i])
    }
}

/// Build a hash map index from a vector of items with string IDs
fn build_index<T, F>(items: &[T], get_id: F) -> HashMap<String, usize>
where
    F: Fn(&T) -> &String,
{
    items
        .iter()
        .enumerate()
        .map(|(i, item)| (get_id(item).clone(), i))
        .collect()
}

/// Packed texture (RGBA8 or BC7 compressed)
///
/// Ready for direct GPU upload via `wgpu::Queue::write_texture()`.
/// Format is determined by render mode at build time.
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
    pub fn stride(&self) -> usize {
        let mut stride = 12; // Position: 3 * f32

        if self.has_uv() {
            stride += 8; // UV: 2 * f32
        }
        if self.has_color() {
            stride += 4; // Color: 4 * u8 (RGBA)
        }
        if self.has_normal() {
            stride += 12; // Normal: 3 * f32
        }
        if self.is_skinned() {
            stride += 8; // Bone indices: 4 * u8 + bone weights: 4 * u8
        }

        stride
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

/// Packed tracker module (XM pattern data + sample mapping)
///
/// Contains XM pattern data with sample references resolved at load time.
/// Samples are loaded separately via `rom_sound()` and mapped by ID.
#[derive(Debug, Clone, Serialize, Deserialize, Encode, Decode)]
pub struct PackedTracker {
    /// Asset ID (e.g., "level1_music", "boss_theme")
    pub id: String,

    /// XM pattern data (samples stripped, patterns + instrument metadata only)
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
    pub fn new(id: impl Into<String>, pattern_data: Vec<u8>, sample_ids: Vec<String>) -> Self {
        Self {
            id: id.into(),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_data_pack() {
        let pack = ZXDataPack::new();
        assert!(pack.is_empty());
        assert_eq!(pack.asset_count(), 0);
    }

    #[test]
    fn test_data_pack_with_assets() {
        let mut pack = ZXDataPack::new();
        pack.textures
            .push(PackedTexture::new("test", 2, 2, vec![0; 16]));
        pack.meshes.push(PackedMesh {
            id: "mesh".to_string(),
            format: 0,
            vertex_count: 3,
            index_count: 3,
            vertex_data: vec![0; 36],
            index_data: vec![0, 1, 2],
        });

        assert!(!pack.is_empty());
        assert_eq!(pack.asset_count(), 2);
    }

    #[test]
    fn test_find_texture() {
        let mut pack = ZXDataPack::new();
        pack.textures
            .push(PackedTexture::new("player", 32, 32, vec![0; 32 * 32 * 4]));
        pack.textures
            .push(PackedTexture::new("enemy", 16, 16, vec![0; 16 * 16 * 4]));

        assert!(pack.find_texture("player").is_some());
        assert!(pack.find_texture("enemy").is_some());
        assert!(pack.find_texture("missing").is_none());
    }

    #[test]
    fn test_packed_texture_validation() {
        let valid = PackedTexture::new("test", 2, 2, vec![0; 16]);
        assert!(valid.validate());

        let invalid = PackedTexture::new("test", 2, 2, vec![0; 10]); // Wrong size
        assert!(!invalid.validate());
    }

    #[test]
    fn test_mesh_format_flags() {
        let mesh = PackedMesh {
            id: "test".to_string(),
            format: 0b1111, // All flags
            vertex_count: 1,
            index_count: 0,
            vertex_data: vec![],
            index_data: vec![],
        };

        assert!(mesh.has_uv());
        assert!(mesh.has_color());
        assert!(mesh.has_normal());
        assert!(mesh.is_skinned());
    }

    #[test]
    fn test_mesh_stride() {
        // Position only
        let pos_only = PackedMesh {
            id: "test".to_string(),
            format: 0,
            vertex_count: 1,
            index_count: 0,
            vertex_data: vec![],
            index_data: vec![],
        };
        assert_eq!(pos_only.stride(), 12);

        // Position + UV + Normal
        let pos_uv_norm = PackedMesh {
            id: "test".to_string(),
            format: 0b0101, // UV + Normal
            vertex_count: 1,
            index_count: 0,
            vertex_data: vec![],
            index_data: vec![],
        };
        assert_eq!(pos_uv_norm.stride(), 12 + 8 + 12);

        // Full skinned
        let skinned = PackedMesh {
            id: "test".to_string(),
            format: 0b1111, // All flags
            vertex_count: 1,
            index_count: 0,
            vertex_data: vec![],
            index_data: vec![],
        };
        assert_eq!(skinned.stride(), 12 + 8 + 4 + 12 + 8);
    }

    #[test]
    fn test_packed_sound_duration() {
        let sound = PackedSound::new("test", vec![0; 22050]); // 1 second
        assert!((sound.duration_seconds() - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_packed_skeleton() {
        let skeleton = PackedSkeleton::new(
            "test",
            vec![BoneMatrix3x4::IDENTITY, BoneMatrix3x4::IDENTITY],
        );
        assert_eq!(skeleton.bone_count, 2);
        assert!(skeleton.validate());
    }

    #[test]
    fn test_find_mesh() {
        let mut pack = ZXDataPack::new();
        pack.meshes.push(PackedMesh {
            id: "cube".to_string(),
            format: 0b0001, // UV only
            vertex_count: 24,
            index_count: 36,
            vertex_data: vec![0; 24 * 20], // pos + uv
            index_data: vec![0; 36],
        });
        pack.meshes.push(PackedMesh {
            id: "sphere".to_string(),
            format: 0,
            vertex_count: 100,
            index_count: 200,
            vertex_data: vec![0; 100 * 12],
            index_data: vec![0; 200],
        });

        let cube = pack.find_mesh("cube");
        assert!(cube.is_some());
        assert_eq!(cube.unwrap().vertex_count, 24);

        let sphere = pack.find_mesh("sphere");
        assert!(sphere.is_some());
        assert_eq!(sphere.unwrap().vertex_count, 100);

        assert!(pack.find_mesh("missing").is_none());
    }

    #[test]
    fn test_find_skeleton() {
        let mut pack = ZXDataPack::new();
        pack.skeletons.push(PackedSkeleton::new(
            "player_rig",
            vec![BoneMatrix3x4::IDENTITY; 20],
        ));
        pack.skeletons.push(PackedSkeleton::new(
            "enemy_rig",
            vec![BoneMatrix3x4::IDENTITY; 10],
        ));

        let player = pack.find_skeleton("player_rig");
        assert!(player.is_some());
        assert_eq!(player.unwrap().bone_count, 20);

        let enemy = pack.find_skeleton("enemy_rig");
        assert!(enemy.is_some());
        assert_eq!(enemy.unwrap().bone_count, 10);

        assert!(pack.find_skeleton("missing").is_none());
    }

    #[test]
    fn test_find_font() {
        let mut pack = ZXDataPack::new();
        pack.fonts.push(PackedFont {
            id: "pixel".to_string(),
            atlas_width: 256,
            atlas_height: 256,
            atlas_data: vec![0; 256 * 256 * 4],
            glyphs: vec![
                PackedGlyph {
                    codepoint: 'A' as u32,
                    x: 0,
                    y: 0,
                    w: 8,
                    h: 8,
                    x_offset: 0.0,
                    y_offset: 0.0,
                    advance: 8.0,
                },
                PackedGlyph {
                    codepoint: 'B' as u32,
                    x: 8,
                    y: 0,
                    w: 8,
                    h: 8,
                    x_offset: 0.0,
                    y_offset: 0.0,
                    advance: 8.0,
                },
            ],
            line_height: 10.0,
            baseline: 8.0,
        });

        let font = pack.find_font("pixel");
        assert!(font.is_some());
        assert_eq!(font.unwrap().glyphs.len(), 2);
        assert!((font.unwrap().line_height - 10.0).abs() < 0.001);

        assert!(pack.find_font("missing").is_none());
    }

    #[test]
    fn test_find_sound() {
        let mut pack = ZXDataPack::new();
        pack.sounds.push(PackedSound::new("jump", vec![0i16; 2205])); // 0.1 sec
        pack.sounds
            .push(PackedSound::new("explosion", vec![0i16; 22050])); // 1 sec

        let jump = pack.find_sound("jump");
        assert!(jump.is_some());
        assert_eq!(jump.unwrap().data.len(), 2205);

        let explosion = pack.find_sound("explosion");
        assert!(explosion.is_some());
        assert!((explosion.unwrap().duration_seconds() - 1.0).abs() < 0.001);

        assert!(pack.find_sound("missing").is_none());
    }

    #[test]
    fn test_find_data() {
        let mut pack = ZXDataPack::new();
        pack.data
            .push(PackedData::new("level1", vec![1, 2, 3, 4, 5]));
        pack.data.push(PackedData::new("config", vec![0xFF; 100]));

        let level = pack.find_data("level1");
        assert!(level.is_some());
        assert_eq!(level.unwrap().data, vec![1, 2, 3, 4, 5]);

        let config = pack.find_data("config");
        assert!(config.is_some());
        assert_eq!(config.unwrap().data.len(), 100);

        assert!(pack.find_data("missing").is_none());
    }

    #[test]
    fn test_find_keyframes() {
        let mut pack = ZXDataPack::new();

        // Create 2 bones, 3 frames animation (2 * 3 * 16 = 96 bytes)
        let walk_data = vec![0u8; 2 * 3 * 16];
        pack.keyframes
            .push(PackedKeyframes::new("walk", 2, 3, walk_data));

        // Create 4 bones, 10 frames animation (4 * 10 * 16 = 640 bytes)
        let run_data = vec![0u8; 4 * 10 * 16];
        pack.keyframes
            .push(PackedKeyframes::new("run", 4, 10, run_data));

        let walk = pack.find_keyframes("walk");
        assert!(walk.is_some());
        assert_eq!(walk.unwrap().bone_count, 2);
        assert_eq!(walk.unwrap().frame_count, 3);
        assert!(walk.unwrap().validate());

        let run = pack.find_keyframes("run");
        assert!(run.is_some());
        assert_eq!(run.unwrap().bone_count, 4);
        assert_eq!(run.unwrap().frame_count, 10);
        assert!(run.unwrap().validate());

        assert!(pack.find_keyframes("missing").is_none());
    }

    #[test]
    fn test_packed_keyframes_frame_data() {
        // 2 bones, 2 frames (2 * 2 * 16 = 64 bytes)
        let mut data = vec![0u8; 64];
        // Mark first frame's first bone
        data[0] = 0xFF;
        // Mark second frame's first bone
        data[32] = 0xAA;

        let kf = PackedKeyframes::new("test", 2, 2, data);
        assert!(kf.validate());

        let frame0 = kf.frame_data(0).unwrap();
        assert_eq!(frame0.len(), 32); // 2 bones * 16 bytes
        assert_eq!(frame0[0], 0xFF);

        let frame1 = kf.frame_data(1).unwrap();
        assert_eq!(frame1.len(), 32);
        assert_eq!(frame1[0], 0xAA);

        // Out of bounds
        assert!(kf.frame_data(2).is_none());
    }

    #[test]
    fn test_serialization_roundtrip() {
        let mut pack = ZXDataPack::new();

        // Add one of each asset type
        pack.textures
            .push(PackedTexture::new("tex", 4, 4, vec![0xAB; 64]));
        pack.meshes.push(PackedMesh {
            id: "mesh".to_string(),
            format: 0b0101,
            vertex_count: 3,
            index_count: 3,
            vertex_data: vec![1, 2, 3],
            index_data: vec![0, 1, 2],
        });
        pack.skeletons
            .push(PackedSkeleton::new("skel", vec![BoneMatrix3x4::IDENTITY]));
        pack.keyframes
            .push(PackedKeyframes::new("anim", 2, 5, vec![0; 2 * 5 * 16]));
        pack.fonts.push(PackedFont {
            id: "font".to_string(),
            atlas_width: 64,
            atlas_height: 64,
            atlas_data: vec![0; 64 * 64 * 4],
            glyphs: vec![PackedGlyph {
                codepoint: 'X' as u32,
                x: 0,
                y: 0,
                w: 8,
                h: 8,
                x_offset: 0.0,
                y_offset: 0.0,
                advance: 8.0,
            }],
            line_height: 12.0,
            baseline: 10.0,
        });
        pack.sounds.push(PackedSound::new("sfx", vec![100i16; 100]));
        pack.data.push(PackedData::new("raw", vec![9, 8, 7]));

        // Serialize with bitcode
        let encoded = bitcode::encode(&pack);

        // Deserialize
        let decoded: ZXDataPack = bitcode::decode(&encoded).expect("decode failed");

        // Verify all assets survived
        assert_eq!(decoded.asset_count(), 7);
        assert_eq!(decoded.textures.len(), 1);
        assert_eq!(decoded.meshes.len(), 1);
        assert_eq!(decoded.skeletons.len(), 1);
        assert_eq!(decoded.keyframes.len(), 1);
        assert_eq!(decoded.fonts.len(), 1);
        assert_eq!(decoded.sounds.len(), 1);
        assert_eq!(decoded.data.len(), 1);

        // Verify content
        assert_eq!(decoded.find_texture("tex").unwrap().width, 4);
        assert_eq!(decoded.find_mesh("mesh").unwrap().format, 0b0101);
        assert_eq!(decoded.find_skeleton("skel").unwrap().bone_count, 1);
        assert_eq!(decoded.find_keyframes("anim").unwrap().bone_count, 2);
        assert_eq!(decoded.find_keyframes("anim").unwrap().frame_count, 5);
        assert!((decoded.find_font("font").unwrap().line_height - 12.0).abs() < 0.001);
        assert_eq!(decoded.find_sound("sfx").unwrap().data.len(), 100);
        assert_eq!(decoded.find_data("raw").unwrap().data, vec![9, 8, 7]);
    }

    // ========================================================================
    // TextureFormat tests
    // ========================================================================

    #[test]
    fn test_texture_format_default() {
        let tex = PackedTexture::new("test", 64, 64, vec![0; 64 * 64 * 4]);
        assert_eq!(tex.format, TextureFormat::Rgba8);
    }

    #[test]
    fn test_texture_format_equality() {
        assert_eq!(TextureFormat::Rgba8, TextureFormat::Rgba8);
        assert_eq!(TextureFormat::Bc7, TextureFormat::Bc7);

        assert_ne!(TextureFormat::Rgba8, TextureFormat::Bc7);
    }

    #[test]
    fn test_texture_format_is_bc7() {
        assert!(!TextureFormat::Rgba8.is_bc7());
        assert!(TextureFormat::Bc7.is_bc7());
    }

    #[test]
    fn test_texture_format_data_size_rgba8() {
        assert_eq!(TextureFormat::Rgba8.data_size(64, 64), 64 * 64 * 4);
        assert_eq!(TextureFormat::Rgba8.data_size(32, 32), 32 * 32 * 4);
        assert_eq!(TextureFormat::Rgba8.data_size(128, 64), 128 * 64 * 4);
    }

    #[test]
    fn test_texture_format_data_size_bc7() {
        // 64×64 = 16×16 blocks × 16 bytes = 4096 bytes
        assert_eq!(TextureFormat::Bc7.data_size(64, 64), 4096);

        // 32×32 = 8×8 blocks × 16 bytes = 1024 bytes
        assert_eq!(TextureFormat::Bc7.data_size(32, 32), 1024);

        // 128×128 = 32×32 blocks × 16 bytes = 16384 bytes
        assert_eq!(TextureFormat::Bc7.data_size(128, 128), 16384);
    }

    #[test]
    fn test_texture_format_data_size_bc7_non_aligned() {
        // 30×30 → 8×8 blocks (rounds up) × 16 bytes = 1024 bytes
        assert_eq!(TextureFormat::Bc7.data_size(30, 30), 8 * 8 * 16);

        // 1×1 → 1×1 blocks × 16 bytes = 16 bytes
        assert_eq!(TextureFormat::Bc7.data_size(1, 1), 16);

        // 5×7 → 2×2 blocks × 16 bytes = 64 bytes
        assert_eq!(TextureFormat::Bc7.data_size(5, 7), 2 * 2 * 16);
    }

    #[test]
    fn test_bc7_compression_ratio() {
        let w: u16 = 64;
        let h: u16 = 64;
        let rgba8 = TextureFormat::Rgba8.data_size(w, h);
        let bc7 = TextureFormat::Bc7.data_size(w, h);
        assert_eq!(rgba8 / bc7, 4); // 4× compression ratio
    }

    #[test]
    fn test_packed_texture_with_format() {
        let tex = PackedTexture::with_format(
            "material",
            64,
            64,
            TextureFormat::Bc7,
            vec![0; 4096], // BC7 size for 64×64
        );

        assert_eq!(tex.width, 64);
        assert_eq!(tex.height, 64);
        assert_eq!(tex.format, TextureFormat::Bc7);
        assert!(tex.is_bc7());
        assert!(tex.validate());
    }

    #[test]
    fn test_packed_texture_dimensions_u32() {
        let tex = PackedTexture::new("test", 256, 128, vec![0; 256 * 128 * 4]);
        let (w, h) = tex.dimensions_u32();
        assert_eq!(w, 256);
        assert_eq!(h, 128);
    }

    #[test]
    fn test_packed_data() {
        let data = PackedData::new("level", vec![0x01, 0x02, 0x03, 0x04]);
        assert_eq!(data.id, "level");
        assert_eq!(data.data.len(), 4);
    }

    #[test]
    fn test_packed_glyph() {
        let glyph = PackedGlyph {
            codepoint: 'A' as u32,
            x: 10,
            y: 20,
            w: 8,
            h: 16,
            x_offset: 1.0,
            y_offset: 2.0,
            advance: 10.0,
        };
        assert_eq!(glyph.codepoint, 65);
        assert!((glyph.advance - 10.0).abs() < 0.001);
        assert_eq!(glyph.w, 8);
        assert_eq!(glyph.h, 16);
    }

    #[test]
    fn test_find_tracker() {
        let mut pack = ZXDataPack::new();
        pack.trackers.push(PackedTracker::new(
            "level1_music",
            vec![0x45, 0x78, 0x74], // Dummy XM data
            vec!["kick".to_string(), "snare".to_string()],
        ));
        pack.trackers.push(PackedTracker::new(
            "boss_theme",
            vec![0x01, 0x02, 0x03],
            vec!["bass".to_string()],
        ));

        let level = pack.find_tracker("level1_music");
        assert!(level.is_some());
        assert_eq!(level.unwrap().instrument_count(), 2);
        assert_eq!(level.unwrap().sample_ids[0], "kick");

        let boss = pack.find_tracker("boss_theme");
        assert!(boss.is_some());
        assert_eq!(boss.unwrap().instrument_count(), 1);

        assert!(pack.find_tracker("missing").is_none());
    }

    #[test]
    fn test_packed_tracker() {
        let tracker = PackedTracker::new(
            "test_song",
            vec![0; 1024],
            vec!["drum".to_string(), "bass".to_string(), "lead".to_string()],
        );

        assert_eq!(tracker.id, "test_song");
        assert_eq!(tracker.instrument_count(), 3);
        assert_eq!(tracker.pattern_data_size(), 1024);
    }
}
