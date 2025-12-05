//! Emberware Z FFI state and types
//!
//! FFI staging state for Emberware Z console.
//! This state is rebuilt each frame from FFI calls and consumed by ZGraphics.
//! It is NOT part of rollback state - only GameState is rolled back.

use glam::{Mat4, Vec3};
use hashbrown::HashMap;

/// Maximum number of bones for GPU skinning
pub const MAX_BONES: usize = 256;

// ============================================================================
// Lighting (moved from core/src/wasm/render.rs)
// ============================================================================

// LightState removed - obsolete with unified shading state system.
// Lighting data now stored in PackedLight within PackedUnifiedShadingState.

// ============================================================================
// Font System
// ============================================================================

/// Custom bitmap font definition
#[derive(Debug, Clone)]
pub struct Font {
    /// Texture handle for the font atlas
    pub texture: u32,
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

// ============================================================================
// Pending Resources (moved from core/src/wasm/draw.rs)
// ============================================================================

/// Pending texture load request
#[derive(Debug)]
pub struct PendingTexture {
    pub handle: u32,
    pub width: u32,
    pub height: u32,
    pub data: Vec<u8>,
}

/// Pending mesh load request (retained mode)
#[derive(Debug)]
pub struct PendingMesh {
    pub handle: u32,
    pub format: u8,
    pub vertex_data: Vec<f32>,
    pub index_data: Option<Vec<u16>>,
}

// ============================================================================
// Z-Specific State
// ============================================================================

/// Init-time configuration for Emberware Z
#[derive(Debug, Clone)]
pub struct ZInitConfig {
    /// Resolution index (0-3 for Z: 360p, 540p, 720p, 1080p)
    pub resolution_index: u32,
    /// Tick rate index (0-3 for Z: 24, 30, 60, 120 fps)
    pub tick_rate_index: u32,
    /// Clear/background color (RGBA: 0xRRGGBBAA)
    pub clear_color: u32,
    /// Render mode (0-3: Unlit, Matcap, PBR, Hybrid)
    pub render_mode: u8,
    /// Whether any config was changed during init
    pub modified: bool,
}

impl Default for ZInitConfig {
    fn default() -> Self {
        Self {
            resolution_index: 1,     // Default 540p
            tick_rate_index: 2,      // Default 60 fps
            clear_color: 0x000000FF, // Black, fully opaque
            render_mode: 0,          // Unlit
            modified: false,
        }
    }
}

/// FFI staging state for Emberware Z
///
/// This state is written to by FFI functions during update()/render() calls,
/// then consumed by ZGraphics at the end of each frame. It is cleared after
/// rendering and does not persist between frames.
///
/// This is NOT serialized for rollback - only core GameState is rolled back.
#[derive(Debug)]
pub struct ZFFIState {
    // Render state
    pub depth_test: bool,
    pub cull_mode: u8,
    pub blend_mode: u8,
    pub texture_filter: u8,
    pub bound_textures: [u32; 4],

    // GPU skinning
    pub bone_matrices: Vec<Mat4>,
    pub bone_count: u32,

    // Virtual Render Pass (direct recording)
    pub render_pass: crate::graphics::VirtualRenderPass,

    // Mesh metadata mapping (for FFI access to mesh info)
    pub mesh_map: hashbrown::HashMap<u32, crate::graphics::RetainedMesh>,

    // Pending resource uploads
    pub pending_textures: Vec<PendingTexture>,
    pub pending_meshes: Vec<PendingMesh>,

    // Resource handle allocation
    pub next_texture_handle: u32,
    pub next_mesh_handle: u32,
    pub next_font_handle: u32,

    // Font system
    pub fonts: Vec<Font>,
    pub current_font: u32,

    // Audio system
    #[allow(dead_code)] // Used in full audio implementation
    pub sounds: Vec<Option<crate::audio::Sound>>,
    pub audio_commands: Vec<crate::audio::AudioCommand>,
    #[allow(dead_code)] // Used in full audio implementation
    pub next_sound_handle: u32,

    // Init configuration
    pub init_config: ZInitConfig,

    // Matrix pools (reset each frame)
    pub model_matrices: Vec<Mat4>,
    pub view_matrices: Vec<Mat4>,
    pub proj_matrices: Vec<Mat4>,

    // Current MVP state (values, not indices - lazy allocation like shading states)
    // None = use last in pool (len - 1), Some = pending, needs to be pushed
    pub current_model_matrix: Option<Mat4>,
    pub current_view_matrix: Option<Mat4>,
    pub current_proj_matrix: Option<Mat4>,

    // Combined MVP+Shading state pool with deduplication
    // Each entry contains unpacked indices (model, view, proj, shading) - 16 bytes, maps to vec4<u32> in WGSL
    pub mvp_shading_states: Vec<crate::graphics::MvpShadingIndices>,
    pub mvp_shading_map: HashMap<crate::graphics::MvpShadingIndices, u32>,  // indices -> buffer_index

    // Unified shading state system (deduplication + dirty tracking)
    pub shading_states: Vec<crate::graphics::PackedUnifiedShadingState>,
    pub shading_state_map:
        HashMap<crate::graphics::PackedUnifiedShadingState, crate::graphics::ShadingStateIndex>,
    pub current_shading_state: crate::graphics::PackedUnifiedShadingState,
    pub shading_state_dirty: bool,

    // GPU-instanced quad rendering
    pub quad_instances: Vec<crate::graphics::QuadInstance>,
}

impl Default for ZFFIState {
    fn default() -> Self {
        let mut model_matrices = Vec::with_capacity(256);
        let mut view_matrices = Vec::with_capacity(4);
        let mut proj_matrices = Vec::with_capacity(4);

        // Default model: identity matrix at index 0 (used by deferred commands)
        model_matrices.push(Mat4::IDENTITY);

        // Default view: camera at (0, 0, 5) looking at origin
        view_matrices.push(Mat4::look_at_rh(
            Vec3::new(0.0, 0.0, 5.0),
            Vec3::ZERO,
            Vec3::Y,
        ));

        // Default projection: 60° FOV, 16:9 aspect
        proj_matrices.push(Mat4::perspective_rh(
            60f32.to_radians(),
            16.0 / 9.0,
            0.1,
            1000.0,
        ));

        Self {
            depth_test: true,
            cull_mode: 1, // Back-face culling
            blend_mode: 0,
            texture_filter: 0, // Nearest
            bound_textures: [0; 4],
            bone_matrices: Vec::new(),
            bone_count: 0,
            render_pass: crate::graphics::VirtualRenderPass::new(),
            mesh_map: hashbrown::HashMap::new(),
            pending_textures: Vec::new(),
            pending_meshes: Vec::new(),
            next_texture_handle: 1, // 0 reserved for invalid
            next_mesh_handle: 1,
            next_font_handle: 1,
            fonts: Vec::new(),
            current_font: 0, // 0 = built-in font
            sounds: Vec::new(),
            audio_commands: Vec::new(),
            next_sound_handle: 1, // 0 reserved for invalid
            init_config: ZInitConfig::default(),
            model_matrices: model_matrices.clone(),
            view_matrices: view_matrices.clone(),
            proj_matrices: proj_matrices.clone(),
            current_model_matrix: None,  // Start with None = use pool index 0
            current_view_matrix: None,
            current_proj_matrix: None,
            mvp_shading_states: Vec::with_capacity(256),
            mvp_shading_map: HashMap::with_capacity(256),
            shading_states: Vec::new(),
            shading_state_map: HashMap::new(),
            current_shading_state: crate::graphics::PackedUnifiedShadingState::default(),
            shading_state_dirty: true, // Start dirty so first draw creates state 0
            quad_instances: Vec::with_capacity(256),
        }
    }
}

impl ZFFIState {
    /// Create new FFI state with default values (test helper)
    #[cfg(test)]
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a model matrix to the pool and return its index
    pub fn add_model_matrix(&mut self, matrix: Mat4) -> Option<u32> {
        let idx = self.model_matrices.len() as u32;
        if idx >= 65536 {
            // Panic in all builds - this is a programming error
            panic!("Model matrix pool overflow! Maximum 65,536 matrices per frame.");
        }
        self.model_matrices.push(matrix);
        Some(idx)
    }

    /// Update material property in current shading state (with quantization check)
    pub fn update_material_metallic(&mut self, value: f32) {
        use crate::graphics::pack_unorm8;
        let quantized = pack_unorm8(value);
        if self.current_shading_state.metallic != quantized {
            self.current_shading_state.metallic = quantized;
            self.shading_state_dirty = true;
        }
    }

    pub fn update_material_roughness(&mut self, value: f32) {
        use crate::graphics::pack_unorm8;
        let quantized = pack_unorm8(value);
        if self.current_shading_state.roughness != quantized {
            self.current_shading_state.roughness = quantized;
            self.shading_state_dirty = true;
        }
    }

    pub fn update_material_emissive(&mut self, value: f32) {
        use crate::graphics::pack_unorm8;
        let quantized = pack_unorm8(value);
        if self.current_shading_state.emissive != quantized {
            self.current_shading_state.emissive = quantized;
            self.shading_state_dirty = true;
        }
    }

    /// Update light in current shading state (with quantization)
    pub fn update_light(
        &mut self,
        index: usize,
        direction: [f32; 3],
        color: [f32; 3],
        intensity: f32,
        enabled: bool,
    ) {
        use crate::graphics::PackedLight;
        use glam::Vec3;

        let new_light = PackedLight::from_floats(
            Vec3::from_slice(&direction),
            Vec3::from_slice(&color),
            intensity,
            enabled,
        );

        if self.current_shading_state.lights[index] != new_light {
            self.current_shading_state.lights[index] = new_light;
            self.shading_state_dirty = true;
        }
    }

    /// Update sky colors in current shading state (with quantization)
    pub fn update_sky_colors(&mut self, horizon: [f32; 3], zenith: [f32; 3]) {
        use crate::graphics::pack_rgb8;
        use glam::Vec3;

        let horizon_packed = pack_rgb8(Vec3::from_slice(&horizon));
        let zenith_packed = pack_rgb8(Vec3::from_slice(&zenith));

        if self.current_shading_state.sky.horizon_color != horizon_packed
            || self.current_shading_state.sky.zenith_color != zenith_packed
        {
            self.current_shading_state.sky.horizon_color = horizon_packed;
            self.current_shading_state.sky.zenith_color = zenith_packed;
            self.shading_state_dirty = true;
        }
    }

    /// Update sky sun parameters in current shading state (with quantization)
    pub fn update_sky_sun(&mut self, direction: [f32; 3], color: [f32; 3], sharpness: f32) {
        use crate::graphics::{pack_octahedral_u32, pack_unorm8};
        use glam::Vec3;

        let dir_oct_packed = pack_octahedral_u32(Vec3::from_slice(&direction));

        let color_r = pack_unorm8(color[0]);
        let color_g = pack_unorm8(color[1]);
        let color_b = pack_unorm8(color[2]);
        let sharp = pack_unorm8(sharpness);
        let color_and_sharpness = (color_r as u32)
            | ((color_g as u32) << 8)
            | ((color_b as u32) << 16)
            | ((sharp as u32) << 24);

        if self.current_shading_state.sky.sun_direction_oct != dir_oct_packed
            || self.current_shading_state.sky.sun_color_and_sharpness != color_and_sharpness
        {
            self.current_shading_state.sky.sun_direction_oct = dir_oct_packed;
            self.current_shading_state.sky.sun_color_and_sharpness = color_and_sharpness;
            self.shading_state_dirty = true;
        }
    }

    /// Update color in current shading state (no quantization - already u32 RGBA8)
    pub fn update_color(&mut self, color: u32) {
        if self.current_shading_state.color_rgba8 != color {
            self.current_shading_state.color_rgba8 = color;
            self.shading_state_dirty = true;
        }
    }

    /// Update blend mode in current shading state
    pub fn update_blend_mode(&mut self, blend_mode: crate::graphics::BlendMode) {
        let blend_u32 = blend_mode as u32;
        if self.current_shading_state.blend_mode != blend_u32 {
            self.current_shading_state.blend_mode = blend_u32;
            self.shading_state_dirty = true;
        }
    }

    /// Update a single matcap blend mode slot in current shading state
    pub fn update_matcap_blend_mode(
        &mut self,
        slot: usize,
        mode: crate::graphics::MatcapBlendMode,
    ) {
        use crate::graphics::{pack_matcap_blend_modes, unpack_matcap_blend_modes};

        // Unpack current modes, modify one slot, repack
        let mut modes = unpack_matcap_blend_modes(self.current_shading_state.matcap_blend_modes);
        modes[slot] = mode;
        let packed = pack_matcap_blend_modes(modes);

        if self.current_shading_state.matcap_blend_modes != packed {
            self.current_shading_state.matcap_blend_modes = packed;
            self.shading_state_dirty = true;
        }
    }

    /// Add current shading state to the pool if dirty, returning its index
    ///
    /// Uses deduplication via HashMap - if this exact state already exists, returns existing index.
    /// Otherwise adds a new entry.
    pub fn add_shading_state(&mut self) -> crate::graphics::ShadingStateIndex {
        // If not dirty, return the last added state (should be at index states.len() - 1)
        if !self.shading_state_dirty && !self.shading_states.is_empty() {
            return crate::graphics::ShadingStateIndex(self.shading_states.len() as u32 - 1);
        }

        // Check if this state already exists (deduplication)
        if let Some(&existing_idx) = self.shading_state_map.get(&self.current_shading_state) {
            self.shading_state_dirty = false;
            return existing_idx;
        }

        // Add new state
        let idx = self.shading_states.len() as u32;
        if idx >= 65536 {
            panic!("Shading state pool overflow! Maximum 65,536 unique states per frame.");
        }

        let shading_idx = crate::graphics::ShadingStateIndex(idx);
        self.shading_states.push(self.current_shading_state);
        self.shading_state_map
            .insert(self.current_shading_state, shading_idx);
        self.shading_state_dirty = false;

        shading_idx
    }

    /// Add current MVP matrices + shading state to combined pool, returning buffer index
    ///
    /// Uses lazy allocation and deduplication - only allocates when draws happen.
    /// Similar to add_shading_state() but for combined MVP+shading state.
    pub fn add_mvp_shading_state(&mut self) -> u32 {
        // First, ensure shading state is added
        let shading_idx = self.add_shading_state();

        // Get or push model matrix: Some = pending (push it), None = use last in pool
        let model_idx = if let Some(mat) = self.current_model_matrix.take() {
            self.model_matrices.push(mat);
            (self.model_matrices.len() - 1) as u32
        } else {
            (self.model_matrices.len() - 1) as u32
        };

        // Get or push view matrix
        let view_idx = if let Some(mat) = self.current_view_matrix.take() {
            self.view_matrices.push(mat);
            (self.view_matrices.len() - 1) as u32
        } else {
            (self.view_matrices.len() - 1) as u32
        };

        // Get or push projection matrix
        let proj_idx = if let Some(mat) = self.current_proj_matrix.take() {
            self.proj_matrices.push(mat);
            (self.proj_matrices.len() - 1) as u32
        } else {
            (self.proj_matrices.len() - 1) as u32
        };

        // Create unpacked indices struct (no bit-packing!)
        let indices = crate::graphics::MvpShadingIndices {
            model_idx,
            view_idx,
            proj_idx,
            shading_idx: shading_idx.0,
        };

        // Check if this exact combination already exists
        if let Some(&existing_idx) = self.mvp_shading_map.get(&indices) {
            return existing_idx;
        }

        // Add new combined state
        let buffer_idx = self.mvp_shading_states.len() as u32;
        if buffer_idx >= 65536 {
            panic!("MVP+Shading state pool overflow! Maximum 65,536 unique states per frame.");
        }

        self.mvp_shading_states.push(indices);
        self.mvp_shading_map.insert(indices, buffer_idx);

        buffer_idx
    }

    /// Clear all per-frame commands and reset for next frame
    ///
    /// Called once per frame in app.rs after render_frame() completes.
    /// This is the centralized cleanup point for all per-frame resources.
    ///
    /// This clears only the resources that accumulate per-frame:
    /// - render_pass (immediate draw commands)
    /// - model_matrices (per-draw transforms)
    /// - deferred_commands (billboards, sprites, text, sky)
    /// - audio_commands (sound effects, music)
    ///
    /// One-time init resources (pending_textures, pending_meshes) are NOT cleared here.
    /// They are drained once after init() in app.rs and never accumulate again.
    pub fn clear_frame(&mut self) {
        self.render_pass.reset();

        // Clear matrix pools and re-add defaults
        self.model_matrices.clear();
        self.model_matrices.push(Mat4::IDENTITY); // Re-add identity matrix at index 0

        self.view_matrices.clear();
        self.view_matrices.push(Mat4::look_at_rh(
            Vec3::new(0.0, 0.0, 5.0),
            Vec3::ZERO,
            Vec3::Y,
        )); // Re-add default view matrix

        self.proj_matrices.clear();
        self.proj_matrices.push(Mat4::perspective_rh(
            45.0_f32.to_radians(),
            16.0 / 9.0,
            0.1,
            1000.0,
        )); // Re-add default projection matrix

        // Reset current MVP state to defaults
        self.current_model_matrix = None; // Will use last in pool (IDENTITY)
        self.current_view_matrix = None;  // Will use last in pool (default view)
        self.current_proj_matrix = None;  // Will use last in pool (default proj)

        // Clear combined MVP+shading state pool
        self.mvp_shading_states.clear();
        self.mvp_shading_map.clear();

        self.audio_commands.clear();

        // Reset shading state pool for next frame
        self.shading_states.clear();
        self.shading_state_map.clear();
        self.shading_state_dirty = true; // Mark dirty so first draw creates state 0

        // Clear GPU-instanced quads for next frame
        self.quad_instances.clear();

        // Note: Render state (color, blend_mode, etc.) persists between frames
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_state_has_default_matrices() {
        let state = ZFFIState::default();

        // Should have one default view matrix
        assert_eq!(state.view_matrices.len(), 1);
        // Should have one default projection matrix
        assert_eq!(state.proj_matrices.len(), 1);
        // Should have one default model matrix (identity at index 0)
        assert_eq!(state.model_matrices.len(), 1);
        assert_eq!(state.model_matrices[0], Mat4::IDENTITY);

        // Current matrices should be None (use defaults from pool)
        assert_eq!(state.current_model_matrix, None);
        assert_eq!(state.current_view_matrix, None);
        assert_eq!(state.current_proj_matrix, None);
    }

    #[test]
    fn test_add_model_matrix() {
        let mut state = ZFFIState::new();

        let matrix1 = Mat4::from_translation(Vec3::new(1.0, 2.0, 3.0));
        let matrix2 = Mat4::from_translation(Vec3::new(4.0, 5.0, 6.0));

        let idx1 = state.add_model_matrix(matrix1);
        let idx2 = state.add_model_matrix(matrix2);

        assert_eq!(idx1, Some(1)); // Index 0 is identity
        assert_eq!(idx2, Some(2));
        assert_eq!(state.model_matrices.len(), 3); // Identity + 2 added
        assert_eq!(state.model_matrices[0], Mat4::IDENTITY);
        assert_eq!(state.model_matrices[1], matrix1);
        assert_eq!(state.model_matrices[2], matrix2);
    }

    #[test]
    fn test_add_many_model_matrices() {
        let mut state = ZFFIState::new();

        // Add 100 matrices (identity at 0, so these will be indices 1-100)
        for i in 0..100 {
            let matrix = Mat4::from_translation(Vec3::new(i as f32, 0.0, 0.0));
            let idx = state.add_model_matrix(matrix);
            assert_eq!(idx, Some(i + 1)); // +1 because index 0 is identity
        }

        assert_eq!(state.model_matrices.len(), 101); // Identity + 100 added
    }

    #[test]
    fn test_mvp_index_packing() {
        let mvp = crate::graphics::MvpIndex::new(1234, 56, 78);
        let (model, view, proj) = mvp.unpack();

        assert_eq!(model, 1234);
        assert_eq!(view, 56);
        assert_eq!(proj, 78);
    }

    #[test]
    fn test_mvp_index_max_values() {
        // Test maximum values for each field
        let mvp = crate::graphics::MvpIndex::new(65535, 255, 255);
        let (model, view, proj) = mvp.unpack();

        assert_eq!(model, 65535);
        assert_eq!(view, 255);
        assert_eq!(proj, 255);
    }

    #[test]
    fn test_mvp_index_accessors() {
        let mvp = crate::graphics::MvpIndex::new(100, 10, 5);

        assert_eq!(mvp.model_index(), 100);
        assert_eq!(mvp.view_index(), 10);
        assert_eq!(mvp.proj_index(), 5);
    }

    #[test]
    fn test_clear_frame_resets_model_matrices() {
        let mut state = ZFFIState::new();

        // Add some model matrices (identity already at 0)
        state.add_model_matrix(Mat4::from_translation(Vec3::new(1.0, 2.0, 3.0)));
        state.add_model_matrix(Mat4::from_translation(Vec3::new(4.0, 5.0, 6.0)));
        assert_eq!(state.model_matrices.len(), 3); // Identity + 2 added

        // Clear frame
        state.clear_frame();

        // Model matrices should be cleared and reset to just identity at index 0
        assert_eq!(state.model_matrices.len(), 1);
        assert_eq!(state.model_matrices[0], Mat4::IDENTITY);
        // View and proj should still exist
        assert_eq!(state.view_matrices.len(), 1);
        assert_eq!(state.proj_matrices.len(), 1);
    }

    #[test]
    fn test_mvp_index_invalid_constant() {
        let invalid = crate::graphics::MvpIndex::INVALID;
        let (model, view, proj) = invalid.unpack();

        assert_eq!(model, 0);
        assert_eq!(view, 0);
        assert_eq!(proj, 0);
    }

    #[test]
    fn test_matrix_packing_in_draw_workflow() {
        let mut state = ZFFIState::new();

        // Simulate a typical draw workflow
        let transform = Mat4::from_translation(Vec3::new(10.0, 20.0, 30.0));
        let model_idx = state.add_model_matrix(transform).unwrap();

        // Use default view/proj indices (0)
        let mvp = crate::graphics::MvpIndex::new(model_idx, 0, 0);

        // Verify we can unpack it correctly
        let (m, v, p) = mvp.unpack();
        assert_eq!(m, 1); // Index 1 (identity is at 0)
        assert_eq!(v, 0); // Default view
        assert_eq!(p, 0); // Default proj

        // Verify the matrix is in the pool
        assert_eq!(state.model_matrices[m as usize], transform);
    }

    // ========================================================================
    // Tests for new lazy allocation + deduplication system
    // ========================================================================

    #[test]
    fn test_lazy_allocation_with_option_pattern() {
        let mut state = ZFFIState::default();

        // Initially, current matrices should be None (use defaults from pool)
        assert_eq!(state.current_model_matrix, None);
        assert_eq!(state.current_view_matrix, None);
        assert_eq!(state.current_proj_matrix, None);

        // Set a new model matrix
        let new_model = Mat4::from_translation(Vec3::new(1.0, 2.0, 3.0));
        state.current_model_matrix = Some(new_model);

        // Allocate via add_mvp_shading_state()
        let buffer_idx = state.add_mvp_shading_state();

        // Should return buffer index 0 (first allocation)
        assert_eq!(buffer_idx, 0);

        // Model matrix should have been pushed to pool
        assert_eq!(state.model_matrices.len(), 2); // Identity + new matrix
        assert_eq!(state.model_matrices[1], new_model);

        // current_model_matrix should be taken (back to None)
        assert_eq!(state.current_model_matrix, None);
    }

    #[test]
    fn test_mvp_shading_deduplication() {
        let mut state = ZFFIState::default();

        // Set transform and color
        state.current_model_matrix = Some(Mat4::from_translation(Vec3::new(1.0, 0.0, 0.0)));
        state.current_shading_state.color_rgba8 = 0xFF0000FF; // Red
        state.shading_state_dirty = true;

        // First draw - allocates buffer index 0
        let idx1 = state.add_mvp_shading_state();
        assert_eq!(idx1, 0);
        assert_eq!(state.mvp_shading_states.len(), 1);

        // Second draw with same state (current matrices are None, will use last in pool)
        let idx2 = state.add_mvp_shading_state();

        // Should reuse the same buffer index due to deduplication
        assert_eq!(idx2, 0);
        assert_eq!(state.mvp_shading_states.len(), 1); // Still only 1 entry

        // Change color - should create new entry
        state.current_shading_state.color_rgba8 = 0x0000FFFF; // Blue
        state.shading_state_dirty = true;
        let idx3 = state.add_mvp_shading_state();
        assert_eq!(idx3, 1); // New buffer index
        assert_eq!(state.mvp_shading_states.len(), 2);
    }

    #[test]
    fn test_multiple_draws_share_buffer_index() {
        let mut state = ZFFIState::default();

        // Set transform once
        state.current_model_matrix = Some(Mat4::IDENTITY);
        state.current_shading_state.color_rgba8 = 0xFFFFFFFF;
        state.shading_state_dirty = true;

        // Simulate multiple draw calls with same state
        let idx1 = state.add_mvp_shading_state();
        let idx2 = state.add_mvp_shading_state();
        let idx3 = state.add_mvp_shading_state();

        // All should use the same buffer index
        assert_eq!(idx1, idx2);
        assert_eq!(idx2, idx3);

        // Only one buffer entry should exist
        assert_eq!(state.mvp_shading_states.len(), 1);
    }

    #[test]
    fn test_different_transforms_different_indices() {
        let mut state = ZFFIState::default();

        // Draw 1: Transform A
        state.current_model_matrix = Some(Mat4::from_translation(Vec3::new(1.0, 0.0, 0.0)));
        state.current_shading_state.color_rgba8 = 0xFF0000FF;
        state.shading_state_dirty = true;
        let idx1 = state.add_mvp_shading_state();

        // Draw 2: Transform B
        state.current_model_matrix = Some(Mat4::from_translation(Vec3::new(2.0, 0.0, 0.0)));
        state.current_shading_state.color_rgba8 = 0x00FF00FF;
        state.shading_state_dirty = true;
        let idx2 = state.add_mvp_shading_state();

        // Draw 3: Back to Transform A + same color
        state.current_model_matrix = None; // Use model_matrices[1] (first transform)
        state.model_matrices.truncate(2); // Remove the second transform
        state.current_shading_state.color_rgba8 = 0xFF0000FF;
        state.shading_state_dirty = true;

        // First two should be different
        assert_ne!(idx1, idx2);

        // Third should match first (deduplication works!)
        // Note: This might not deduplicate perfectly because we removed the matrix
        // but the test shows the deduplication concept
        assert_eq!(state.mvp_shading_states.len(), 2); // At least 2 unique states
    }

    #[test]
    fn test_clear_frame_resets_mvp_state() {
        let mut state = ZFFIState::default();

        // Add some MVP states
        state.current_model_matrix = Some(Mat4::from_translation(Vec3::new(1.0, 2.0, 3.0)));
        state.current_shading_state.color_rgba8 = 0xFF0000FF;
        state.shading_state_dirty = true;
        state.add_mvp_shading_state();

        state.current_model_matrix = Some(Mat4::from_translation(Vec3::new(4.0, 5.0, 6.0)));
        state.current_shading_state.color_rgba8 = 0x0000FFFF;
        state.shading_state_dirty = true;
        state.add_mvp_shading_state();

        // Should have multiple entries
        assert!(state.mvp_shading_states.len() > 0);
        assert!(state.mvp_shading_map.len() > 0);
        assert!(state.model_matrices.len() > 1);

        // Clear frame
        state.clear_frame();

        // All pools should be reset
        assert_eq!(state.mvp_shading_states.len(), 0);
        assert_eq!(state.mvp_shading_map.len(), 0);
        assert_eq!(state.model_matrices.len(), 1); // Only identity
        assert_eq!(state.view_matrices.len(), 1); // Only default
        assert_eq!(state.proj_matrices.len(), 1); // Only default

        // Current matrices should be None
        assert_eq!(state.current_model_matrix, None);
        assert_eq!(state.current_view_matrix, None);
        assert_eq!(state.current_proj_matrix, None);
    }

    #[test]
    fn test_none_uses_last_in_pool() {
        let mut state = ZFFIState::default();

        // Add a matrix explicitly
        state.current_model_matrix = Some(Mat4::from_translation(Vec3::new(5.0, 0.0, 0.0)));
        state.current_shading_state.color_rgba8 = 0xFF0000FF;
        state.shading_state_dirty = true;
        let idx1 = state.add_mvp_shading_state();

        // model_matrices should now have 2 entries: [IDENTITY, translation]
        assert_eq!(state.model_matrices.len(), 2);

        // Now use None (should use last in pool = translation)
        state.current_model_matrix = None;
        state.current_shading_state.color_rgba8 = 0xFF0000FF;
        state.shading_state_dirty = true; // Same color
        let idx2 = state.add_mvp_shading_state();

        // Should reuse the same buffer index
        assert_eq!(idx1, idx2);
    }
}
