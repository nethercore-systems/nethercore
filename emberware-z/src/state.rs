//! Emberware Z FFI state and types
//!
//! FFI staging state for Emberware Z console.
//! This state is rebuilt each frame from FFI calls and consumed by ZGraphics.
//! It is NOT part of rollback state - only GameState is rolled back.

use glam::{Mat4, Vec3};

/// Maximum number of bones for GPU skinning
pub const MAX_BONES: usize = 256;

/// Maximum transform stack depth
pub const MAX_TRANSFORM_STACK: usize = 32;

// ============================================================================
// Camera (moved from core/src/wasm/camera.rs)
// ============================================================================

/// Default camera field of view in degrees
pub const DEFAULT_CAMERA_FOV: f32 = 60.0;

/// Camera state for 3D rendering
#[derive(Debug, Clone, Copy)]
pub struct CameraState {
    /// Camera position in world space
    pub position: Vec3,
    /// Camera target (look-at point) in world space
    pub target: Vec3,
    /// Field of view in degrees
    pub fov: f32,
    /// Near clipping plane
    pub near: f32,
    /// Far clipping plane
    pub far: f32,
}

impl Default for CameraState {
    fn default() -> Self {
        Self {
            position: Vec3::new(0.0, 0.0, 5.0),
            target: Vec3::ZERO,
            fov: DEFAULT_CAMERA_FOV,
            near: 0.1,
            far: 1000.0,
        }
    }
}

impl CameraState {
    /// Compute the view matrix (world-to-camera transform)
    #[inline]
    pub fn view_matrix(&self) -> Mat4 {
        Mat4::look_at_rh(self.position, self.target, Vec3::Y)
    }

    /// Compute the projection matrix for a given aspect ratio
    #[inline]
    pub fn projection_matrix(&self, aspect_ratio: f32) -> Mat4 {
        Mat4::perspective_rh(self.fov.to_radians(), aspect_ratio, self.near, self.far)
    }

    /// Compute the combined view-projection matrix
    #[inline]
    #[allow(dead_code)] // Public API for games, currently unused internally
    pub fn view_projection_matrix(&self, aspect_ratio: f32) -> Mat4 {
        self.projection_matrix(aspect_ratio) * self.view_matrix()
    }
}

// ============================================================================
// Lighting (moved from core/src/wasm/render.rs)
// ============================================================================

/// Light state for Mode 2/3 (PBR/Hybrid)
#[derive(Debug, Clone, Copy)]
pub struct LightState {
    /// Light enabled
    pub enabled: bool,
    /// Light direction (normalized)
    pub direction: [f32; 3],
    /// Light color (RGB, linear)
    pub color: [f32; 3],
    /// Light intensity multiplier
    pub intensity: f32,
}

impl Default for LightState {
    fn default() -> Self {
        Self {
            enabled: false,
            direction: [0.0, -1.0, 0.0], // Default: downward
            color: [1.0, 1.0, 1.0],      // Default: white
            intensity: 1.0,
        }
    }
}

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
// Draw Commands (moved from core/src/wasm/draw.rs)
// ============================================================================

// ============================================================================
// Draw Commands (moved from core/src/wasm/draw.rs)
// ============================================================================

/// Deferred draw command for special objects (billboards, sprites, text)
///
/// These commands generate geometry at render time (need camera info for billboards,
/// screen size for sprites) or are 2D overlays. They are processed after the
/// main 3D render pass.
#[derive(Debug)]
pub enum DeferredCommand {
    /// Draw a billboard (camera-facing quad)
    DrawBillboard {
        width: f32,
        height: f32,
        mode: u8,
        uv_rect: Option<(f32, f32, f32, f32)>,
        transform: Mat4,
        color: u32,
        depth_test: bool,
        cull_mode: u8,
        blend_mode: u8,
        bound_textures: [u32; 4],
    },
    /// Draw a 2D sprite in screen space
    DrawSprite {
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        uv_rect: Option<(f32, f32, f32, f32)>,
        origin: Option<(f32, f32)>,
        rotation: f32,
        color: u32,
        blend_mode: u8,
        bound_textures: [u32; 4],
    },
    /// Draw a 2D rectangle in screen space
    DrawRect {
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        color: u32,
        blend_mode: u8,
    },
    /// Draw text in screen space
    DrawText {
        text: Vec<u8>,
        x: f32,
        y: f32,
        size: f32,
        color: u32,
        blend_mode: u8,
        font: u32, // 0 = built-in font, >0 = custom font handle
    },
    /// Set procedural sky parameters
    SetSky {
        horizon_color: [f32; 3],
        zenith_color: [f32; 3],
        sun_direction: [f32; 3],
        sun_color: [f32; 3],
        sun_sharpness: f32,
    },
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
    // 3D Camera
    pub camera: CameraState,

    // 3D Transform stack
    pub transform_stack: Vec<Mat4>,
    pub current_transform: Mat4,

    // Render state
    pub color: u32,
    pub depth_test: bool,
    pub cull_mode: u8,
    pub blend_mode: u8,
    pub texture_filter: u8,
    pub bound_textures: [u32; 4],

    // Z-specific rendering modes
    pub matcap_blend_modes: [u8; 4],
    pub material_metallic: f32,
    pub material_roughness: f32,
    pub material_emissive: f32,

    // PBR lighting
    pub lights: [LightState; 4],

    // GPU skinning
    pub bone_matrices: Vec<Mat4>,
    pub bone_count: u32,

    // Virtual Render Pass (direct recording)
    pub render_pass: crate::graphics::VirtualRenderPass,

    // Deferred commands (billboards, sprites, text)
    pub deferred_commands: Vec<DeferredCommand>,

    // Resource mappings (injected by App before frame)
    pub texture_map: hashbrown::HashMap<u32, crate::graphics::TextureHandle>,
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

    // Current matrix indices
    #[allow(dead_code)] // Reserved for future advanced matrix management
    pub current_model_idx: u32,
    pub current_view_idx: u32,
    pub current_proj_idx: u32,
}

impl Default for ZFFIState {
    fn default() -> Self {
        let mut model_matrices = Vec::with_capacity(256);
        let mut view_matrices = Vec::with_capacity(4);
        let mut proj_matrices = Vec::with_capacity(4);

        // Default model: identity matrix at index 0 (used by deferred commands)
        model_matrices.push(Mat4::IDENTITY);

        // Default view: camera at (0, 0, 5) looking at origin
        view_matrices.push(Mat4::look_at_rh(Vec3::new(0.0, 0.0, 5.0), Vec3::ZERO, Vec3::Y));

        // Default projection: 60° FOV, 16:9 aspect
        proj_matrices.push(Mat4::perspective_rh(60f32.to_radians(), 16.0 / 9.0, 0.1, 1000.0));

        Self {
            camera: CameraState::default(),
            transform_stack: Vec::with_capacity(MAX_TRANSFORM_STACK),
            current_transform: Mat4::IDENTITY,
            color: 0xFFFFFFFF,
            depth_test: true,
            cull_mode: 1, // Back-face culling
            blend_mode: 0,
            texture_filter: 0, // Nearest
            bound_textures: [0; 4],
            matcap_blend_modes: [0; 4],
            material_metallic: 0.0,
            material_roughness: 0.5,
            material_emissive: 0.0,
            lights: [LightState::default(); 4],
            bone_matrices: Vec::new(),
            bone_count: 0,
            render_pass: crate::graphics::VirtualRenderPass::new(),
            deferred_commands: Vec::new(),
            texture_map: hashbrown::HashMap::new(),
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
            model_matrices,
            view_matrices,
            proj_matrices,
            current_model_idx: 0,
            current_view_idx: 0,
            current_proj_idx: 0,
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

    /// Pack current matrix indices into MvpIndex
    #[allow(dead_code)] // Reserved for future advanced matrix management
    pub fn pack_current_mvp(&self) -> crate::graphics::MvpIndex {
        crate::graphics::MvpIndex::new(
            self.current_model_idx,
            self.current_view_idx,
            self.current_proj_idx,
        )
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
        self.model_matrices.clear();
        // Re-add identity matrix at index 0 for deferred commands
        self.model_matrices.push(Mat4::IDENTITY);
        self.deferred_commands.clear();
        self.audio_commands.clear();
        // Note: Camera, transforms, render state persist between frames
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

        // Indices should start at 0
        assert_eq!(state.current_view_idx, 0);
        assert_eq!(state.current_proj_idx, 0);
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
    fn test_pack_current_mvp() {
        let mut state = ZFFIState::new();

        // Add a model matrix (will be at index 1, identity is at 0)
        let model_idx = state.add_model_matrix(Mat4::IDENTITY).unwrap();
        state.current_model_idx = model_idx;
        state.current_view_idx = 0;
        state.current_proj_idx = 0;

        let mvp = state.pack_current_mvp();
        let (m, v, p) = mvp.unpack();

        assert_eq!(m, 1); // Index 1, since identity is at 0
        assert_eq!(v, 0);
        assert_eq!(p, 0);
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

        let mvp = crate::graphics::MvpIndex::new(
            model_idx,
            state.current_view_idx,
            state.current_proj_idx,
        );

        // Verify we can unpack it correctly
        let (m, v, p) = mvp.unpack();
        assert_eq!(m, 1); // Index 1 (identity is at 0)
        assert_eq!(v, 0); // Default view
        assert_eq!(p, 0); // Default proj

        // Verify the matrix is in the pool
        assert_eq!(state.model_matrices[m as usize], transform);
    }
}
