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

mod types;

#[cfg(test)]
mod tests;

pub use types::*;

use bitcode::{Decode, Encode};
use hashbrown::HashMap;
use serde::{Deserialize, Serialize};
use std::sync::OnceLock;

/// Nethercore ZX data pack
///
/// Contains all bundled assets for a Nethercore ZX ROM. Assets are stored
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
    #[allow(clippy::too_many_arguments)]
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
