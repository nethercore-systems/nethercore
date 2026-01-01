//! Asset viewers for ZX preview mode
//!
//! This module provides specialized viewers for each asset type in a ZX ROM data pack.
//! Each viewer handles rendering, interaction, and playback for its specific asset type.

use std::sync::Arc;

use half::f16;

use crate::audio::{AudioOutput, OUTPUT_SAMPLE_RATE, Sound};
use crate::console::NethercoreZX;
use crate::graphics::ZXGraphics;
use crate::state::{TrackerState, tracker_flags};
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
                        format!(
                            "XM Tracker, {} instruments, {} bytes pattern data",
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

    // === Texture viewer controls ===

    /// Zoom in on texture preview
    pub fn texture_zoom_in(&mut self) {
        self.texture_zoom = (self.texture_zoom * 1.5).min(10.0);
    }

    /// Zoom out on texture preview
    pub fn texture_zoom_out(&mut self) {
        self.texture_zoom = (self.texture_zoom / 1.5).max(0.1);
    }

    /// Reset texture zoom to 1:1
    pub fn texture_reset_zoom(&mut self) {
        self.texture_zoom = 1.0;
        self.texture_pan = (0.0, 0.0);
    }

    /// Pan texture preview
    pub fn texture_pan(&mut self, dx: f32, dy: f32) {
        self.texture_pan.0 += dx;
        self.texture_pan.1 += dy;
    }

    /// Get current texture zoom level
    pub fn texture_zoom(&self) -> f32 {
        self.texture_zoom
    }

    /// Get current texture pan offset
    pub fn texture_pan_offset(&self) -> (f32, f32) {
        self.texture_pan
    }

    /// Update texture cache if needed and return the cached handle.
    ///
    /// This is a helper method to reduce duplication between texture and font preview.
    /// Creates or updates the cached egui texture handle based on the cache_id.
    fn update_texture_cache(
        &mut self,
        ctx: &egui::Context,
        cache_id: &str,
        texture_name: &str,
        width: u32,
        height: u32,
        rgba_data: &[u8],
    ) {
        if self.cached_texture_id.as_ref() != Some(&cache_id.to_string()) {
            let color_image = egui::ColorImage::from_rgba_unmultiplied(
                [width as usize, height as usize],
                rgba_data,
            );

            let filter = if self.texture_linear_filter {
                egui::TextureFilter::Linear
            } else {
                egui::TextureFilter::Nearest
            };

            let mut options = egui::TextureOptions::default();
            options.magnification = filter;
            options.minification = filter;

            self.cached_texture = Some(ctx.load_texture(texture_name, color_image, options));
            self.cached_texture_id = Some(cache_id.to_string());
        }
    }

    // === Sound viewer controls ===

    /// Toggle sound playback
    pub fn sound_toggle_play(&mut self) {
        if self.sound_playing {
            // Stop playback
            self.sound_stop();
        } else {
            // Get sound data
            let sound_data = self.selected_sound().map(|s| s.data.clone());

            if let Some(data) = sound_data {
                // Initialize audio output if not already created
                if self.audio_output.is_none() {
                    match AudioOutput::new() {
                        Ok(output) => {
                            self.audio_output = Some(output);
                        }
                        Err(e) => {
                            eprintln!("Failed to initialize audio: {}", e);
                            return;
                        }
                    }
                }

                // Convert mono i16 to stereo f32 and upsample 22050->44100
                let mut stereo_samples = Vec::with_capacity(data.len() * 4); // *2 for upsample, *2 for stereo
                for sample in data.iter() {
                    let f_sample = *sample as f32 / i16::MAX as f32;
                    // Upsample: duplicate each sample for 44100 Hz output
                    stereo_samples.push(f_sample); // Left
                    stereo_samples.push(f_sample); // Right
                    stereo_samples.push(f_sample); // Left (duplicate)
                    stereo_samples.push(f_sample); // Right (duplicate)
                }

                // Push entire sound to buffer at once
                if let Some(ref mut audio_output) = self.audio_output {
                    audio_output.push_samples(&stereo_samples);
                }

                // Start playing
                self.sound_playing = true;
                self.sound_position = 0;
            }
        }
    }

    /// Stop sound and reset position
    pub fn sound_stop(&mut self) {
        self.sound_playing = false;
        self.sound_position = 0;
    }

    // === Tracker viewer controls ===

    /// Start tracker playback
    pub fn start_tracker_playback(&mut self) {
        // Get tracker data (clone what we need to avoid borrow conflicts)
        let (pattern_data, sample_ids) = match self.selected_tracker() {
            Some(tracker) => (tracker.pattern_data.clone(), tracker.sample_ids.clone()),
            None => return,
        };

        // Parse XM module (supports both full XM and minimal NCXM formats)
        let xm_module = match nether_xm::parse_xm_minimal(&pattern_data) {
            Ok(m) => m,
            Err(e) => {
                eprintln!("Failed to parse XM: {:?}", e);
                return;
            }
        };

        // Initialize audio output if needed
        if self.audio_output.is_none() {
            match AudioOutput::new() {
                Ok(output) => {
                    self.audio_output = Some(output);
                }
                Err(e) => {
                    eprintln!("Failed to initialize audio: {}", e);
                    return;
                }
            }
        }

        // Load sounds from data pack using tracker's sample_ids
        // Sound handles are 1-indexed (0 = no sound)
        let mut sounds: Vec<Option<Sound>> = vec![None]; // Index 0 is unused
        let mut sound_handles: Vec<u32> = Vec::new();

        eprintln!("DEBUG: Starting tracker playback");
        eprintln!(
            "DEBUG: Tracker has {} sample_ids: {:?}",
            sample_ids.len(),
            sample_ids
        );
        eprintln!(
            "DEBUG: Data pack has {} sounds: {:?}",
            self.data_pack.sounds.len(),
            self.data_pack
                .sounds
                .iter()
                .map(|s| &s.id)
                .collect::<Vec<_>>()
        );

        for sample_id in &sample_ids {
            if let Some(packed_sound) = self.data_pack.sounds.iter().find(|s| &s.id == sample_id) {
                // Convert PackedSound to Sound
                let sound = Sound {
                    data: Arc::new(packed_sound.data.clone()),
                };
                eprintln!(
                    "DEBUG: Loaded sample '{}' with {} samples, handle={}",
                    sample_id,
                    packed_sound.data.len(),
                    sounds.len()
                );
                sounds.push(Some(sound));
                sound_handles.push(sounds.len() as u32 - 1); // Handle points to index
            } else {
                eprintln!(
                    "Warning: tracker sample '{}' not found in data pack",
                    sample_id
                );
                sounds.push(None);
                sound_handles.push(sounds.len() as u32 - 1);
            }
        }

        eprintln!(
            "DEBUG: Total sounds loaded: {}, sound_handles: {:?}",
            sounds.len(),
            sound_handles
        );
        self.tracker_sounds = sounds;

        // Initialize tracker engine
        let mut engine = TrackerEngine::new();
        let handle = engine.load_xm_module(xm_module.clone(), sound_handles);

        // Initialize tracker state
        let mut state = TrackerState::default();
        state.handle = handle;
        state.flags = tracker_flags::PLAYING;
        state.volume = 256; // Full volume

        self.tracker_engine = Some(engine);
        self.tracker_state = Some(state);
        self.tracker_playing = true;
    }

    /// Seek sound to position (0.0 - 1.0)
    pub fn sound_seek(&mut self, position: f32) {
        if let Some(id) = &self.selected_id
            && let Some(sound) = self.data_pack.sounds.iter().find(|s| &s.id == id)
        {
            self.sound_position = ((position.clamp(0.0, 1.0) * sound.data.len() as f32) as usize)
                .min(sound.data.len().saturating_sub(1));
        }
    }

    /// Get sound playback progress (0.0 - 1.0)
    pub fn sound_progress(&self) -> f32 {
        if let Some(id) = &self.selected_id
            && let Some(sound) = self.data_pack.sounds.iter().find(|s| &s.id == id)
        {
            if sound.data.is_empty() {
                return 0.0;
            }
            return self.sound_position as f32 / sound.data.len() as f32;
        }
        0.0
    }

    /// Check if sound is playing
    pub fn sound_is_playing(&self) -> bool {
        self.sound_playing
    }

    // === Animation viewer controls ===

    /// Toggle animation playback
    pub fn animation_toggle_play(&mut self) {
        self.animation_playing = !self.animation_playing;
    }

    /// Set animation frame
    pub fn animation_set_frame(&mut self, frame: u16) {
        if let Some(id) = &self.selected_id
            && let Some(anim) = self.data_pack.keyframes.iter().find(|k| &k.id == id)
        {
            self.animation_frame = frame.min(anim.frame_count.saturating_sub(1));
        }
    }

    /// Step animation forward one frame
    pub fn animation_step_forward(&mut self) {
        if let Some(id) = &self.selected_id
            && let Some(anim) = self.data_pack.keyframes.iter().find(|k| &k.id == id)
        {
            self.animation_frame = (self.animation_frame + 1) % anim.frame_count.max(1);
        }
    }

    /// Step animation backward one frame
    pub fn animation_step_back(&mut self) {
        if let Some(id) = &self.selected_id
            && let Some(anim) = self.data_pack.keyframes.iter().find(|k| &k.id == id)
        {
            if self.animation_frame == 0 {
                self.animation_frame = anim.frame_count.saturating_sub(1);
            } else {
                self.animation_frame -= 1;
            }
        }
    }

    /// Get current animation frame
    pub fn animation_frame(&self) -> u16 {
        self.animation_frame
    }

    /// Check if animation is playing
    pub fn animation_is_playing(&self) -> bool {
        self.animation_playing
    }

    /// Set animation playback speed
    pub fn animation_set_speed(&mut self, speed: f32) {
        self.animation_speed = speed.clamp(0.1, 4.0);
    }

    // === Mesh viewer controls ===

    /// Rotate mesh view
    pub fn mesh_rotate(&mut self, dyaw: f32, dpitch: f32) {
        self.mesh_rotation.0 += dyaw;
        self.mesh_rotation.1 = (self.mesh_rotation.1 + dpitch).clamp(-89.0, 89.0);
    }

    /// Zoom mesh view
    pub fn mesh_zoom(&mut self, delta: f32) {
        self.mesh_distance = (self.mesh_distance - delta).clamp(0.5, 50.0);
    }

    /// Reset mesh view
    pub fn mesh_reset_view(&mut self) {
        self.mesh_rotation = (0.0, 0.0);
        self.mesh_distance = 5.0;
    }

    /// Toggle wireframe overlay
    pub fn mesh_toggle_wireframe(&mut self) {
        self.mesh_wireframe = !self.mesh_wireframe;
    }

    /// Get mesh rotation (yaw, pitch)
    pub fn mesh_rotation(&self) -> (f32, f32) {
        self.mesh_rotation
    }

    /// Get mesh camera distance
    pub fn mesh_distance(&self) -> f32 {
        self.mesh_distance
    }

    /// Check if wireframe is enabled
    pub fn mesh_wireframe(&self) -> bool {
        self.mesh_wireframe
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
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Asset Preview");

            if let Some(id) = &self.selected_id {
                let id_owned = id.clone(); // Clone to avoid borrow issues
                ui.label(format!("Selected: {}", id_owned));

                if let Some(info) = self.selected_info() {
                    ui.label(info);
                }

                ui.separator();

                match self.selected_category {
                    AssetCategory::Textures => {
                        ui.heading("Texture Preview");

                        // Extract values to avoid borrow issues
                        let zoom = self.texture_zoom;

                        ui.horizontal(|ui| {
                            if ui.button("Zoom In").clicked() {
                                self.texture_zoom_in();
                            }
                            if ui.button("Zoom Out").clicked() {
                                self.texture_zoom_out();
                            }
                            if ui.button("Reset").clicked() {
                                self.texture_reset_zoom();
                            }
                            ui.label(format!("Zoom: {:.1}x", zoom));

                            ui.separator();

                            if ui
                                .button(if self.texture_linear_filter {
                                    "Linear"
                                } else {
                                    "Nearest"
                                })
                                .clicked()
                            {
                                self.texture_linear_filter = !self.texture_linear_filter;
                                // Invalidate cache to recreate texture with new filter
                                self.cached_texture = None;
                                self.cached_texture_id = None;
                            }
                        });

                        ui.separator();

                        // Render the texture - use cached texture to avoid recreating every frame
                        if let Some(texture) = self.selected_texture() {
                            let width = texture.width;
                            let height = texture.height;
                            let texture_data = texture.data.clone();

                            // Update cache if needed
                            self.update_texture_cache(
                                ctx,
                                &id_owned,
                                &format!("preview_{}", id_owned),
                                width as u32,
                                height as u32,
                                &texture_data,
                            );

                            // Use the cached texture
                            if let Some(ref texture_handle) = self.cached_texture {
                                let display_size =
                                    egui::vec2(width as f32 * zoom, height as f32 * zoom);
                                ui.add(
                                    egui::Image::new(texture_handle)
                                        .fit_to_exact_size(display_size),
                                );
                            }
                        } else {
                            ui.label("Failed to load texture");
                        }
                    }
                    AssetCategory::Sounds => {
                        ui.heading("Sound Preview");

                        // Extract sound data to avoid borrow issues
                        let sound_data = self.selected_sound().map(|s| s.data.clone());
                        let is_playing = self.sound_playing;
                        let progress = self.sound_progress();

                        if let Some(sound_samples) = sound_data {
                            ui.horizontal(|ui| {
                                if is_playing {
                                    if ui.button("⏸ Stop").clicked() {
                                        self.sound_stop();
                                    }
                                } else if ui.button("▶ Play").clicked() {
                                    self.sound_toggle_play();
                                }
                                ui.label(format!("Position: {:.1}%", progress * 100.0));
                            });

                            ui.separator();

                            // Draw waveform
                            let available_width = ui.available_width();
                            let waveform_height = 200.0;

                            let (rect, _response) = ui.allocate_exact_size(
                                egui::vec2(available_width, waveform_height),
                                egui::Sense::hover(),
                            );

                            if ui.is_rect_visible(rect) {
                                let painter = ui.painter();

                                // Background
                                painter.rect_filled(rect, 0.0, egui::Color32::from_gray(20));

                                // Center line
                                let center_y = rect.center().y;
                                painter.line_segment(
                                    [
                                        rect.left_top() + egui::vec2(0.0, waveform_height / 2.0),
                                        rect.right_top() + egui::vec2(0.0, waveform_height / 2.0),
                                    ],
                                    egui::Stroke::new(1.0, egui::Color32::from_gray(60)),
                                );

                                // Draw waveform
                                let samples_per_pixel =
                                    (sound_samples.len() as f32 / available_width).max(1.0)
                                        as usize;

                                for x in 0..(available_width as usize) {
                                    let sample_idx = x * samples_per_pixel;
                                    if sample_idx >= sound_samples.len() {
                                        break;
                                    }

                                    // Get min/max samples in this pixel range
                                    let mut min_sample = i16::MAX;
                                    let mut max_sample = i16::MIN;

                                    for i in 0..samples_per_pixel {
                                        let idx = sample_idx + i;
                                        if idx >= sound_samples.len() {
                                            break;
                                        }
                                        let sample = sound_samples[idx];
                                        min_sample = min_sample.min(sample);
                                        max_sample = max_sample.max(sample);
                                    }

                                    // Normalize to -1.0 to 1.0
                                    let min_norm = min_sample as f32 / i16::MAX as f32;
                                    let max_norm = max_sample as f32 / i16::MAX as f32;

                                    // Convert to screen space
                                    let min_y = center_y - min_norm * (waveform_height / 2.0);
                                    let max_y = center_y - max_norm * (waveform_height / 2.0);

                                    let x_pos = rect.left() + x as f32;

                                    // Draw line for this pixel column
                                    painter.line_segment(
                                        [egui::pos2(x_pos, min_y), egui::pos2(x_pos, max_y)],
                                        egui::Stroke::new(
                                            1.0,
                                            egui::Color32::from_rgb(100, 200, 255),
                                        ),
                                    );
                                }

                                // Draw playback position
                                if is_playing {
                                    let pos_x = rect.left() + progress * available_width;
                                    painter.line_segment(
                                        [
                                            egui::pos2(pos_x, rect.top()),
                                            egui::pos2(pos_x, rect.bottom()),
                                        ],
                                        egui::Stroke::new(
                                            2.0,
                                            egui::Color32::from_rgb(255, 100, 100),
                                        ),
                                    );
                                }
                            }
                        } else {
                            ui.label("Failed to load sound");
                        }
                    }
                    AssetCategory::Meshes => {
                        ui.heading("Mesh Preview");

                        if let Some(mesh) = self.selected_mesh() {
                            ui.label(format!("Vertices: {}", mesh.vertex_count));
                            ui.label(format!("Indices: {}", mesh.index_count));
                            ui.label(format!("Stride: {} bytes", mesh.stride()));
                            ui.label(format!(
                                "Total size: {} bytes",
                                mesh.vertex_data.len() + mesh.index_data.len() * 2
                            ));

                            let mut flags = Vec::new();
                            if mesh.has_uv() {
                                flags.push("UV");
                            }
                            if mesh.has_color() {
                                flags.push("Color");
                            }
                            if mesh.has_normal() {
                                flags.push("Normal");
                            }
                            if mesh.is_skinned() {
                                flags.push("Skinned");
                            }
                            ui.label(format!("Features: {}", flags.join(", ")));

                            ui.separator();

                            // Show vertex data format
                            ui.label("Vertex Format:");
                            ui.indent("vertex_format", |ui| {
                                ui.label("• Position (xyz): 12 bytes");
                                if mesh.has_uv() {
                                    ui.label("• UV (uv): 8 bytes");
                                }
                                if mesh.has_color() {
                                    ui.label("• Color (rgba): 4 bytes");
                                }
                                if mesh.has_normal() {
                                    ui.label("• Normal (xyz): 12 bytes");
                                }
                                if mesh.is_skinned() {
                                    ui.label("• Bone Indices (4×u8): 4 bytes");
                                    ui.label("• Bone Weights (4×u8): 4 bytes");
                                }
                            });

                            ui.separator();

                            // Show sample vertices
                            ui.label("Sample Vertex Data (first 3 vertices):");
                            egui::ScrollArea::vertical()
                                .max_height(200.0)
                                .show(ui, |ui| {
                                    let stride = mesh.stride();
                                    for i in 0..3.min(mesh.vertex_count as usize) {
                                        let offset = i * stride;
                                        if offset + 8 <= mesh.vertex_data.len() {
                                            ui.collapsing(format!("Vertex {}", i), |ui| {
                                                // Read position (first 8 bytes: f16x4)
                                                let pos_bytes =
                                                    &mesh.vertex_data[offset..offset + 8];
                                                let x = f16::from_le_bytes([
                                                    pos_bytes[0],
                                                    pos_bytes[1],
                                                ])
                                                .to_f32();
                                                let y = f16::from_le_bytes([
                                                    pos_bytes[2],
                                                    pos_bytes[3],
                                                ])
                                                .to_f32();
                                                let z = f16::from_le_bytes([
                                                    pos_bytes[4],
                                                    pos_bytes[5],
                                                ])
                                                .to_f32();
                                                ui.code(format!(
                                                    "pos: ({:.3}, {:.3}, {:.3})",
                                                    x, y, z
                                                ));
                                            });
                                        }
                                    }
                                });

                            ui.separator();

                            // Show sample indices
                            ui.label("Index Data (first 12 indices):");
                            let sample_indices: Vec<String> = mesh
                                .index_data
                                .iter()
                                .take(12)
                                .map(|idx| format!("{}", idx))
                                .collect();
                            ui.code(sample_indices.join(", "));
                            if mesh.index_data.len() > 12 {
                                ui.label("...");
                            }
                        } else {
                            ui.label("Failed to load mesh");
                        }
                    }
                    AssetCategory::Animations => {
                        ui.heading("Animation Preview");

                        // Extract animation data to avoid borrow issues
                        let anim_data = self
                            .selected_animation()
                            .map(|a| (a.frame_count, a.bone_count, a.data.clone()));

                        if let Some((frame_count, bone_count, keyframe_data)) = anim_data {
                            let keyframe_data_size = keyframe_data.len();
                            let is_playing = self.animation_playing;
                            let current_frame = self.animation_frame;

                            ui.horizontal(|ui| {
                                if ui
                                    .button(if is_playing { "⏸ Pause" } else { "▶ Play" })
                                    .clicked()
                                {
                                    self.animation_playing = !self.animation_playing;
                                }
                                ui.label(format!("Frame: {}/{}", current_frame, frame_count));
                            });

                            ui.separator();

                            ui.label(format!("Bones: {}", bone_count));
                            ui.label(format!("Frames: {}", frame_count));
                            ui.label(format!(
                                "Duration: {:.2}s @ 30fps",
                                frame_count as f32 / 30.0
                            ));
                            ui.label(format!("Keyframe data: {} bytes", keyframe_data_size));

                            ui.separator();

                            // Show animation format info
                            ui.label("Format:");
                            ui.indent("anim_format", |ui| {
                                ui.label(format!("• {} bone matrices per frame", bone_count));
                                ui.label("• 12 floats per matrix (3×4)");
                                ui.label(format!("• {} bytes per frame", bone_count * 12 * 4));
                            });

                            ui.separator();

                            // Progress bar
                            ui.label("Playback Progress:");
                            let progress = if frame_count > 0 {
                                current_frame as f32 / frame_count as f32
                            } else {
                                0.0
                            };
                            ui.add(egui::ProgressBar::new(progress).show_percentage());

                            ui.separator();

                            // Show sample bone transform from current frame
                            ui.label(format!("Sample: Bone 0 at frame {}", current_frame));
                            if bone_count > 0 && frame_count > 0 {
                                let frame_idx = (current_frame as usize) % (frame_count as usize);
                                let bone_idx = 0;
                                let matrix_offset =
                                    (frame_idx * bone_count as usize + bone_idx) * 12 * 4;

                                if matrix_offset + 48 <= keyframe_data_size {
                                    ui.collapsing("Transform Matrix", |ui| {
                                        let data =
                                            &keyframe_data[matrix_offset..matrix_offset + 48];
                                        let mut floats = Vec::new();
                                        for chunk in data.chunks(4) {
                                            floats.push(f32::from_le_bytes([
                                                chunk[0], chunk[1], chunk[2], chunk[3],
                                            ]));
                                        }
                                        ui.code(format!(
                                            "[ {:.3}, {:.3}, {:.3}, {:.3} ]\n\
                                             [ {:.3}, {:.3}, {:.3}, {:.3} ]\n\
                                             [ {:.3}, {:.3}, {:.3}, {:.3} ]",
                                            floats[0],
                                            floats[1],
                                            floats[2],
                                            floats[3],
                                            floats[4],
                                            floats[5],
                                            floats[6],
                                            floats[7],
                                            floats[8],
                                            floats[9],
                                            floats[10],
                                            floats[11],
                                        ));
                                    });
                                }
                            }
                        } else {
                            ui.label("Failed to load animation");
                        }
                    }
                    AssetCategory::Trackers => {
                        ui.heading("Tracker Preview");

                        if let Some(tracker) = self.selected_tracker() {
                            ui.label(format!("Instruments: {}", tracker.instrument_count()));
                            ui.label(format!(
                                "Pattern data: {} bytes",
                                tracker.pattern_data_size()
                            ));

                            ui.separator();

                            let is_playing = self.tracker_playing;

                            ui.horizontal(|ui| {
                                if is_playing {
                                    if ui.button("⏸ Stop").clicked() {
                                        self.tracker_playing = false;
                                    }
                                } else if ui.button("▶ Play").clicked() {
                                    self.start_tracker_playback();
                                }

                                // Show position if playing
                                if let Some(ref state) = self.tracker_state {
                                    ui.label(format!("Order: {}", state.order_position));
                                    ui.label(format!("Row: {}", state.row));
                                }
                            });
                        }
                    }
                    AssetCategory::Skeletons => {
                        ui.heading("Skeleton Preview");
                        if let Some(skeleton) = self.selected_skeleton() {
                            ui.label(format!("Bones: {}", skeleton.bone_count));
                            ui.label(format!(
                                "Inverse bind matrices: {}",
                                skeleton.inverse_bind_matrices.len()
                            ));

                            ui.separator();

                            // Show bone matrices in a scrollable area
                            ui.label("Bone Transforms:");
                            egui::ScrollArea::vertical()
                                .max_height(300.0)
                                .show(ui, |ui| {
                                    for (i, matrix) in
                                        skeleton.inverse_bind_matrices.iter().enumerate()
                                    {
                                        ui.collapsing(format!("Bone {}", i), |ui| {
                                            ui.code(format!(
                                                "[ {:.3}, {:.3}, {:.3}, {:.3} ]\n\
                                             [ {:.3}, {:.3}, {:.3}, {:.3} ]\n\
                                             [ {:.3}, {:.3}, {:.3}, {:.3} ]",
                                                matrix.row0[0],
                                                matrix.row0[1],
                                                matrix.row0[2],
                                                matrix.row0[3],
                                                matrix.row1[0],
                                                matrix.row1[1],
                                                matrix.row1[2],
                                                matrix.row1[3],
                                                matrix.row2[0],
                                                matrix.row2[1],
                                                matrix.row2[2],
                                                matrix.row2[3],
                                            ));
                                        });
                                    }
                                });
                        } else {
                            ui.label("Failed to load skeleton");
                        }
                    }
                    AssetCategory::Fonts => {
                        ui.heading("Font Preview");

                        // Extract font data to avoid borrow issues
                        let font_data = self.selected_font().map(|f| {
                            (
                                f.atlas_width,
                                f.atlas_height,
                                f.line_height,
                                f.baseline,
                                f.glyphs.len(),
                                f.atlas_data.clone(),
                                f.glyphs
                                    .iter()
                                    .take(20)
                                    .map(|g| g.codepoint)
                                    .collect::<Vec<_>>(),
                            )
                        });

                        if let Some((
                            width,
                            height,
                            line_height,
                            baseline,
                            glyph_count,
                            atlas_data,
                            sample_glyphs,
                        )) = font_data
                        {
                            ui.label(format!("Atlas: {}x{}", width, height));
                            ui.label(format!("Glyphs: {}", glyph_count));
                            ui.label(format!("Line height: {:.1}", line_height));
                            ui.label(format!("Baseline: {:.1}", baseline));

                            ui.separator();

                            // Extract zoom control value
                            let zoom = self.texture_zoom;

                            ui.horizontal(|ui| {
                                if ui.button("Zoom In").clicked() {
                                    self.texture_zoom_in();
                                }
                                if ui.button("Zoom Out").clicked() {
                                    self.texture_zoom_out();
                                }
                                if ui.button("Reset").clicked() {
                                    self.texture_reset_zoom();
                                }
                                ui.label(format!("Zoom: {:.1}x", zoom));

                                ui.separator();

                                if ui
                                    .button(if self.texture_linear_filter {
                                        "Linear"
                                    } else {
                                        "Nearest"
                                    })
                                    .clicked()
                                {
                                    self.texture_linear_filter = !self.texture_linear_filter;
                                    // Invalidate cache to recreate texture with new filter
                                    self.cached_texture = None;
                                    self.cached_texture_id = None;
                                }
                            });

                            ui.separator();

                            // Update cache if needed
                            let font_id = format!("font_{}", id_owned);
                            self.update_texture_cache(
                                ctx,
                                &font_id,
                                &format!("font_atlas_{}", id_owned),
                                width,
                                height,
                                &atlas_data,
                            );

                            // Use the cached texture
                            if let Some(ref texture_handle) = self.cached_texture {
                                let display_size =
                                    egui::vec2(width as f32 * zoom, height as f32 * zoom);
                                ui.add(
                                    egui::Image::new(texture_handle)
                                        .fit_to_exact_size(display_size),
                                );
                            }

                            ui.separator();

                            // Show some sample glyphs
                            ui.label("Sample Characters:");
                            ui.horizontal_wrapped(|ui| {
                                for codepoint in &sample_glyphs {
                                    if let Some(ch) = char::from_u32(*codepoint) {
                                        ui.label(format!("'{}'", ch));
                                    }
                                }
                                if glyph_count > 20 {
                                    ui.label("...");
                                }
                            });
                        } else {
                            ui.label("Failed to load font");
                        }
                    }
                    AssetCategory::Data => {
                        ui.heading("Data Preview");
                        if let Some(data) = self.selected_data() {
                            ui.label(format!("Size: {} bytes", data.data.len()));

                            // Show hex dump of first bytes
                            let preview_bytes = data.data.len().min(256);
                            ui.label(format!("First {} bytes (hex):", preview_bytes));

                            egui::ScrollArea::vertical()
                                .max_height(300.0)
                                .show(ui, |ui| {
                                    ui.code(
                                        data.data[..preview_bytes]
                                            .chunks(16)
                                            .enumerate()
                                            .map(|(i, chunk)| {
                                                format!(
                                                    "{:04x}: {}",
                                                    i * 16,
                                                    chunk
                                                        .iter()
                                                        .map(|b| format!("{:02x}", b))
                                                        .collect::<Vec<_>>()
                                                        .join(" ")
                                                )
                                            })
                                            .collect::<Vec<_>>()
                                            .join("\n"),
                                    );
                                });
                        }
                    }
                }
            } else {
                ui.label("No asset selected");
                ui.label("Select an asset from the category tabs above");
            }
        });
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
            }

            // Debug: print once per second approximately
            static mut DEBUG_COUNTER: u32 = 0;
            unsafe {
                DEBUG_COUNTER += 1;
                if DEBUG_COUNTER.is_multiple_of(30) {
                    eprintln!(
                        "DEBUG render: {} samples, max_sample={:.4}, tracker_sounds.len()={}",
                        samples_to_generate,
                        max_sample,
                        self.tracker_sounds.len()
                    );
                }
            }

            // Push to audio output
            if let Some(audio_output) = &mut self.audio_output {
                audio_output.push_samples(&stereo_samples);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use zx_common::{PackedSound, PackedTexture};

    fn create_test_data() -> PreviewData<ZXDataPack> {
        let mut data_pack = ZXDataPack::default();

        // Add a test texture
        data_pack
            .textures
            .push(PackedTexture::new("test_tex", 64, 64, vec![0; 64 * 64 * 4]));

        // Add a test sound (0.5 seconds)
        data_pack
            .sounds
            .push(PackedSound::new("test_sfx", vec![0i16; 11025]));

        PreviewData {
            data_pack,
            metadata: super::super::PreviewMetadata {
                id: "test".to_string(),
                title: "Test Game".to_string(),
                author: "Test".to_string(),
                version: "1.0.0".to_string(),
            },
        }
    }

    #[test]
    fn test_viewer_creation() {
        let data = create_test_data();
        let viewer = ZXAssetViewer::new(&data);

        assert_eq!(viewer.asset_count(AssetCategory::Textures), 1);
        assert_eq!(viewer.asset_count(AssetCategory::Sounds), 1);
        assert_eq!(viewer.asset_count(AssetCategory::Meshes), 0);
    }

    #[test]
    fn test_asset_selection() {
        let data = create_test_data();
        let mut viewer = ZXAssetViewer::new(&data);

        viewer.select_asset(AssetCategory::Textures, "test_tex");

        assert_eq!(viewer.selected_category(), AssetCategory::Textures);
        assert_eq!(viewer.selected_id(), Some("test_tex"));
        assert!(viewer.selected_texture().is_some());
    }

    #[test]
    fn test_texture_controls() {
        let data = create_test_data();
        let mut viewer = ZXAssetViewer::new(&data);

        viewer.texture_zoom_in();
        assert!(viewer.texture_zoom() > 1.0);

        viewer.texture_reset_zoom();
        assert!((viewer.texture_zoom() - 1.0).abs() < f32::EPSILON);

        viewer.texture_pan(10.0, 5.0);
        assert_eq!(viewer.texture_pan_offset(), (10.0, 5.0));
    }

    #[test]
    fn test_sound_controls() {
        let data = create_test_data();
        let mut viewer = ZXAssetViewer::new(&data);

        viewer.select_asset(AssetCategory::Sounds, "test_sfx");

        assert!(!viewer.sound_is_playing());
        viewer.sound_toggle_play();
        assert!(viewer.sound_is_playing());

        viewer.sound_seek(0.5);
        assert!((viewer.sound_progress() - 0.5).abs() < 0.01);

        viewer.sound_stop();
        assert!(!viewer.sound_is_playing());
        assert!((viewer.sound_progress()).abs() < f32::EPSILON);
    }
}
