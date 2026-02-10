//! Asset viewers for ZX preview mode
//!
//! This module provides specialized viewers for each asset type in a ZX ROM data pack.
//! Each viewer handles rendering, interaction, and playback for its specific asset type.

mod animation_viewer;
mod mesh_viewer;
mod sound_viewer;
mod texture_viewer;
mod tracker_viewer;
mod ui;

#[cfg(test)]
mod tests;

use crate::audio::{AudioOutput, OUTPUT_SAMPLE_RATE, Sound};
use crate::console::NethercoreZX;
use crate::graphics::ZXGraphics;
use crate::state::TrackerState;
use crate::tracker::TrackerEngine;
use nethercore_core::app::preview::{
    AssetCategory as CoreAssetCategory, AssetViewer as CoreAssetViewer,
    PreviewData as CorePreviewData,
};
use zx_common::ZXDataPack;

use super::{AssetCategory, PreviewData};

/// Main asset viewer for ZX console
///
/// Manages the currently selected asset and provides viewing functionality
/// for all asset types in a ZX data pack.
pub struct ZXAssetViewer {
    /// The loaded data pack
    data_pack: ZXDataPack,

    /// Currently selected category
    selected_category: AssetCategory,

    /// Currently selected asset ID within the category
    selected_id: Option<String>,

    // === Texture viewer state ===
    /// Zoom level for texture preview (1.0 = 100%)
    texture_zoom: f32,

    /// Pan offset for texture preview
    texture_pan: (f32, f32),

    /// Texture filtering mode (true = Linear, false = Nearest/Point)
    texture_linear_filter: bool,

    /// Cached texture handle for current texture/font (to avoid recreating every frame)
    cached_texture: Option<egui::TextureHandle>,

    /// ID of the currently cached texture
    cached_texture_id: Option<String>,

    // === Sound viewer state ===
    /// Whether sound is currently playing
    sound_playing: bool,

    /// Current playback position in samples
    sound_position: usize,

    /// Audio output for sound playback
    audio_output: Option<AudioOutput>,

    // === Tracker viewer state ===
    /// Tracker engine for XM playback
    tracker_engine: Option<TrackerEngine>,
    /// Tracker state for XM playback
    tracker_state: Option<TrackerState>,
    /// Whether tracker is playing
    tracker_playing: bool,
    /// Loaded sounds for tracker playback (indexed by sound handle)
    tracker_sounds: Vec<Option<Sound>>,

    // === Animation viewer state ===
    /// Current animation frame
    animation_frame: u16,

    /// Whether animation is playing
    animation_playing: bool,

    /// Animation playback speed multiplier
    animation_speed: f32,

    // === Mesh viewer state ===
    /// Camera rotation for mesh preview (yaw, pitch)
    mesh_rotation: (f32, f32),

    /// Camera distance for mesh preview
    mesh_distance: f32,

    /// Whether to show wireframe overlay
    mesh_wireframe: bool,
}

impl ZXAssetViewer {
    /// Create a new asset viewer from loaded preview data
    pub fn new(data: &PreviewData<ZXDataPack>) -> Self {
        Self {
            data_pack: data.data_pack.clone(),
            selected_category: AssetCategory::Textures,
            selected_id: None,

            // Texture state
            texture_zoom: 1.0,
            texture_pan: (0.0, 0.0),
            texture_linear_filter: false, // Start with nearest/point filtering
            cached_texture: None,
            cached_texture_id: None,

            // Sound state
            sound_playing: false,
            sound_position: 0,
            audio_output: None,

            // Tracker state
            tracker_engine: None,
            tracker_state: None,
            tracker_playing: false,
            tracker_sounds: Vec::new(),

            // Animation state
            animation_frame: 0,
            animation_playing: false,
            animation_speed: 1.0,

            // Mesh state
            mesh_rotation: (0.0, 0.0),
            mesh_distance: 5.0,
            mesh_wireframe: false,
        }
    }

    /// Get all asset IDs for a given category
    pub fn asset_ids(&self, category: AssetCategory) -> Vec<String> {
        match category {
            AssetCategory::Textures => self
                .data_pack
                .textures
                .iter()
                .map(|t| t.id.clone())
                .collect(),
            AssetCategory::Meshes => self.data_pack.meshes.iter().map(|m| m.id.clone()).collect(),
            AssetCategory::Sounds => self.data_pack.sounds.iter().map(|s| s.id.clone()).collect(),
            AssetCategory::Trackers => self
                .data_pack
                .trackers
                .iter()
                .map(|t| t.id.clone())
                .collect(),
            AssetCategory::Animations => self
                .data_pack
                .keyframes
                .iter()
                .map(|k| k.id.clone())
                .collect(),
            AssetCategory::Skeletons => self
                .data_pack
                .skeletons
                .iter()
                .map(|s| s.id.clone())
                .collect(),
            AssetCategory::Fonts => self.data_pack.fonts.iter().map(|f| f.id.clone()).collect(),
            AssetCategory::Data => self.data_pack.data.iter().map(|d| d.id.clone()).collect(),
        }
    }

    /// Get the count of assets in a category
    pub fn asset_count(&self, category: AssetCategory) -> usize {
        match category {
            AssetCategory::Textures => self.data_pack.textures.len(),
            AssetCategory::Meshes => self.data_pack.meshes.len(),
            AssetCategory::Sounds => self.data_pack.sounds.len(),
            AssetCategory::Trackers => self.data_pack.trackers.len(),
            AssetCategory::Animations => self.data_pack.keyframes.len(),
            AssetCategory::Skeletons => self.data_pack.skeletons.len(),
            AssetCategory::Fonts => self.data_pack.fonts.len(),
            AssetCategory::Data => self.data_pack.data.len(),
        }
    }

    /// Select an asset for viewing
    pub fn select_asset(&mut self, category: AssetCategory, id: &str) {
        self.selected_category = category;
        self.selected_id = Some(id.to_string());

        // Reset viewer state for the new asset
        self.texture_zoom = 1.0;
        self.texture_pan = (0.0, 0.0);
        self.cached_texture = None;
        self.cached_texture_id = None;
        self.sound_playing = false;
        self.sound_position = 0;
        self.animation_frame = 0;
        self.animation_playing = false;
        self.mesh_rotation = (0.0, 0.0);
    }

    /// Get currently selected category
    pub fn selected_category(&self) -> AssetCategory {
        self.selected_category
    }

    /// Get currently selected asset ID
    pub fn selected_id(&self) -> Option<&str> {
        self.selected_id.as_deref()
    }

    /// Get information string for the currently selected asset
    pub fn selected_info(&self) -> Option<String> {
        let id = self.selected_id.as_ref()?;

        match self.selected_category {
            AssetCategory::Textures => {
                self.data_pack
                    .textures
                    .iter()
                    .find(|t| &t.id == id)
                    .map(|t| {
                        format!(
                            "{}x{} {:?} ({} bytes)",
                            t.width,
                            t.height,
                            t.format,
                            t.data.len()
                        )
                    })
            }
            AssetCategory::Meshes => self.data_pack.meshes.iter().find(|m| &m.id == id).map(|m| {
                let mut flags = Vec::new();
                if m.has_uv() {
                    flags.push("UV");
                }
                if m.has_color() {
                    flags.push("Color");
                }
                if m.has_normal() {
                    flags.push("Normal");
                }
                if m.is_skinned() {
                    flags.push("Skinned");
                }
                format!(
                    "{} verts, {} indices, stride {} [{}]",
                    m.vertex_count,
                    m.index_count,
                    m.stride(),
                    flags.join(", ")
                )
            }),
            AssetCategory::Sounds => self.data_pack.sounds.iter().find(|s| &s.id == id).map(|s| {
                format!(
                    "{} samples ({:.2}s @ 22050Hz)",
                    s.data.len(),
                    s.duration_seconds()
                )
            }),
            AssetCategory::Trackers => {
                self.data_pack
                    .trackers
                    .iter()
                    .find(|t| &t.id == id)
                    .map(|t| {
                        let format_name = match t.format {
                            zx_common::TrackerFormat::Xm => "XM",
                            zx_common::TrackerFormat::It => "IT",
                        };
                        format!(
                            "{} Tracker, {} instruments, {} bytes pattern data",
                            format_name,
                            t.instrument_count(),
                            t.pattern_data_size()
                        )
                    })
            }
            AssetCategory::Animations => {
                self.data_pack
                    .keyframes
                    .iter()
                    .find(|k| &k.id == id)
                    .map(|k| {
                        format!(
                            "{} bones, {} frames ({} bytes)",
                            k.bone_count,
                            k.frame_count,
                            k.data.len()
                        )
                    })
            }
            AssetCategory::Skeletons => {
                self.data_pack
                    .skeletons
                    .iter()
                    .find(|s| &s.id == id)
                    .map(|s| {
                        format!(
                            "{} bones, {} inverse bind matrices",
                            s.bone_count,
                            s.inverse_bind_matrices.len()
                        )
                    })
            }
            AssetCategory::Fonts => self.data_pack.fonts.iter().find(|f| &f.id == id).map(|f| {
                format!(
                    "{}x{} atlas, {} glyphs, line height {:.1}",
                    f.atlas_width,
                    f.atlas_height,
                    f.glyphs.len(),
                    f.line_height
                )
            }),
            AssetCategory::Data => self
                .data_pack
                .data
                .iter()
                .find(|d| &d.id == id)
                .map(|d| format!("{} bytes", d.data.len())),
        }
    }

    /// Update the viewer state (called each frame)
    pub fn update(&mut self, dt: f32) {
        // Update animation playback
        if self.animation_playing
            && let Some(id) = &self.selected_id
            && let Some(anim) = self.data_pack.keyframes.iter().find(|k| &k.id == id)
        {
            // Advance frame based on delta time (assuming 30 fps animations)
            let frames_per_second = 30.0 * self.animation_speed;
            let frame_advance = dt * frames_per_second;

            self.animation_frame =
                ((self.animation_frame as f32 + frame_advance) as u16) % anim.frame_count.max(1);
        }

        // Update sound playback position
        if self.sound_playing
            && let Some(id) = &self.selected_id
            && let Some(sound) = self.data_pack.sounds.iter().find(|s| &s.id == id)
        {
            // Advance position based on delta time (22050 Hz sample rate)
            let samples_advance = (dt * 22050.0) as usize;
            self.sound_position += samples_advance;

            // Loop or stop at end
            if self.sound_position >= sound.data.len() {
                self.sound_position = 0;
                self.sound_playing = false;
            }
        }
    }

    // === Data access for rendering ===

    /// Get reference to the data pack
    pub fn data_pack(&self) -> &ZXDataPack {
        &self.data_pack
    }

    /// Get the currently selected texture (if any)
    pub fn selected_texture(&self) -> Option<&zx_common::PackedTexture> {
        if self.selected_category != AssetCategory::Textures {
            return None;
        }
        let id = self.selected_id.as_ref()?;
        self.data_pack.textures.iter().find(|t| &t.id == id)
    }

    /// Get the currently selected mesh (if any)
    pub fn selected_mesh(&self) -> Option<&zx_common::PackedMesh> {
        if self.selected_category != AssetCategory::Meshes {
            return None;
        }
        let id = self.selected_id.as_ref()?;
        self.data_pack.meshes.iter().find(|m| &m.id == id)
    }

    /// Get the currently selected sound (if any)
    pub fn selected_sound(&self) -> Option<&zx_common::PackedSound> {
        if self.selected_category != AssetCategory::Sounds {
            return None;
        }
        let id = self.selected_id.as_ref()?;
        self.data_pack.sounds.iter().find(|s| &s.id == id)
    }

    /// Get the currently selected tracker (if any)
    pub fn selected_tracker(&self) -> Option<&zx_common::PackedTracker> {
        if self.selected_category != AssetCategory::Trackers {
            return None;
        }
        let id = self.selected_id.as_ref()?;
        self.data_pack.trackers.iter().find(|t| &t.id == id)
    }

    /// Get the currently selected animation (if any)
    pub fn selected_animation(&self) -> Option<&zx_common::PackedKeyframes> {
        if self.selected_category != AssetCategory::Animations {
            return None;
        }
        let id = self.selected_id.as_ref()?;
        self.data_pack.keyframes.iter().find(|k| &k.id == id)
    }

    /// Get the currently selected skeleton (if any)
    pub fn selected_skeleton(&self) -> Option<&zx_common::PackedSkeleton> {
        if self.selected_category != AssetCategory::Skeletons {
            return None;
        }
        let id = self.selected_id.as_ref()?;
        self.data_pack.skeletons.iter().find(|s| &s.id == id)
    }

    /// Get the currently selected font (if any)
    pub fn selected_font(&self) -> Option<&zx_common::PackedFont> {
        if self.selected_category != AssetCategory::Fonts {
            return None;
        }
        let id = self.selected_id.as_ref()?;
        self.data_pack.fonts.iter().find(|f| &f.id == id)
    }

    /// Get the currently selected data blob (if any)
    pub fn selected_data(&self) -> Option<&zx_common::PackedData> {
        if self.selected_category != AssetCategory::Data {
            return None;
        }
        let id = self.selected_id.as_ref()?;
        self.data_pack.data.iter().find(|d| &d.id == id)
    }
}

// Convert between local and core AssetCategory
fn _to_core_category(cat: AssetCategory) -> CoreAssetCategory {
    match cat {
        AssetCategory::Textures => CoreAssetCategory::Textures,
        AssetCategory::Meshes => CoreAssetCategory::Meshes,
        AssetCategory::Sounds => CoreAssetCategory::Sounds,
        AssetCategory::Trackers => CoreAssetCategory::Trackers,
        AssetCategory::Animations => CoreAssetCategory::Animations,
        AssetCategory::Skeletons => CoreAssetCategory::Fonts, // Note: Core doesn't have Skeletons
        AssetCategory::Fonts => CoreAssetCategory::Fonts,
        AssetCategory::Data => CoreAssetCategory::Data,
    }
}

fn from_core_category(cat: CoreAssetCategory) -> AssetCategory {
    match cat {
        CoreAssetCategory::Textures => AssetCategory::Textures,
        CoreAssetCategory::Meshes => AssetCategory::Meshes,
        CoreAssetCategory::Sounds => AssetCategory::Sounds,
        CoreAssetCategory::Trackers => AssetCategory::Trackers,
        CoreAssetCategory::Animations => AssetCategory::Animations,
        CoreAssetCategory::Fonts => AssetCategory::Fonts,
        CoreAssetCategory::Data => AssetCategory::Data,
    }
}

// Implement the core AssetViewer trait
impl CoreAssetViewer<NethercoreZX, ZXDataPack> for ZXAssetViewer {
    fn new(data: &CorePreviewData<ZXDataPack>) -> Self {
        // Convert core PreviewData to local PreviewData
        let local_data = PreviewData {
            data_pack: data.data_pack.clone(),
            metadata: super::PreviewMetadata {
                id: data.metadata.id.clone(),
                title: data.metadata.title.clone(),
                author: data.metadata.author.clone(),
                version: data.metadata.version.clone(),
            },
        };

        ZXAssetViewer::new(&local_data)
    }

    fn asset_ids(&self, category: CoreAssetCategory) -> Vec<String> {
        self.asset_ids(from_core_category(category))
    }

    fn asset_count(&self, category: CoreAssetCategory) -> usize {
        self.asset_count(from_core_category(category))
    }

    fn select_asset(&mut self, category: CoreAssetCategory, id: &str) {
        self.select_asset(from_core_category(category), id);
    }

    fn selected_info(&self) -> Option<String> {
        self.selected_info()
    }

    fn render_ui(&mut self, ctx: &egui::Context, _graphics: &mut ZXGraphics) {
        self.render_ui_impl(ctx);
    }

    fn update(&mut self, dt: f32) {
        // Update animation state
        if self.animation_playing {
            self.animation_frame = self.animation_frame.wrapping_add(1);
        }

        // Update sound playback position (just for UI visualization)
        if self.sound_playing
            && let Some(id) = &self.selected_id
            && let Some(sound) = self.data_pack.sounds.iter().find(|s| &s.id == id)
        {
            // Track position for waveform visualization
            let samples_per_second = 22050.0;
            let samples_to_advance = (samples_per_second * dt) as usize;
            self.sound_position += samples_to_advance;

            if self.sound_position >= sound.data.len() {
                self.sound_playing = false;
                self.sound_position = 0;
            }
        }

        // Update tracker playback - generate and push samples
        if self.tracker_playing {
            // Calculate how many samples to generate for this frame
            let samples_to_generate = (dt * OUTPUT_SAMPLE_RATE as f32) as usize;

            // Generate samples using loaded sounds from data pack
            let mut stereo_samples = Vec::with_capacity(samples_to_generate * 2);
            let mut max_sample = 0.0f32;
            let mut still_playing = true;

            if let (Some(engine), Some(state)) = (&mut self.tracker_engine, &mut self.tracker_state)
            {
                for _ in 0..samples_to_generate {
                    let (left, right) = engine.render_sample_and_advance(
                        state,
                        &self.tracker_sounds,
                        OUTPUT_SAMPLE_RATE,
                    );
                    max_sample = max_sample.max(left.abs()).max(right.abs());
                    stereo_samples.push(left);
                    stereo_samples.push(right);
                }
                still_playing = (state.flags & crate::state::tracker_flags::PLAYING) != 0;
            }

            // Debug: log once per second approximately
            static DEBUG_COUNTER: std::sync::atomic::AtomicU32 =
                std::sync::atomic::AtomicU32::new(0);
            let counter = DEBUG_COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed) + 1;
            if counter.is_multiple_of(30) {
                tracing::debug!(
                    samples_to_generate,
                    max_sample,
                    tracker_sounds_len = self.tracker_sounds.len(),
                    "preview audio render"
                );
            }

            // Push to audio output
            if let Some(audio_output) = &mut self.audio_output {
                audio_output.push_samples(&stereo_samples);
            }

            if !still_playing {
                self.tracker_playing = false;
            }
        }
    }
}
