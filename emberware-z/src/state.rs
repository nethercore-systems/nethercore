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
// Pending Resources (moved from core/src/wasm/draw.rs)
// ============================================================================

/// Pending texture load request
#[derive(Debug, Clone)]
pub struct PendingTexture {
    pub handle: u32,
    pub width: u32,
    pub height: u32,
    pub data: Vec<u8>,
}

/// Pending mesh load request (retained mode)
#[derive(Debug, Clone)]
pub struct PendingMesh {
    pub handle: u32,
    pub format: u8,
    pub vertex_data: Vec<f32>,
    pub index_data: Option<Vec<u32>>,
}

// ============================================================================
// Draw Commands (moved from core/src/wasm/draw.rs)
// ============================================================================

/// Draw command for Z-specific immediate mode drawing
#[derive(Debug, Clone)]
pub enum ZDrawCommand {
    /// Draw triangles with immediate data (non-indexed)
    DrawTriangles {
        format: u8,
        vertex_data: Vec<f32>,
        transform: Mat4,
        color: u32,
        depth_test: bool,
        cull_mode: u8,
        blend_mode: u8,
        bound_textures: [u32; 4],
    },
    /// Draw indexed triangles with immediate data
    DrawTrianglesIndexed {
        format: u8,
        vertex_data: Vec<f32>,
        index_data: Vec<u32>,
        transform: Mat4,
        color: u32,
        depth_test: bool,
        cull_mode: u8,
        blend_mode: u8,
        bound_textures: [u32; 4],
    },
    /// Draw a retained mesh by handle
    DrawMesh {
        handle: u32,
        transform: Mat4,
        color: u32,
        depth_test: bool,
        cull_mode: u8,
        blend_mode: u8,
        bound_textures: [u32; 4],
    },
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
#[derive(Debug, Clone)]
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
    pub render_mode: u8,
    pub matcap_blend_modes: [u8; 4],
    pub material_metallic: f32,
    pub material_roughness: f32,
    pub material_emissive: f32,

    // PBR lighting
    pub lights: [LightState; 4],

    // GPU skinning
    pub bone_matrices: Vec<Mat4>,
    pub bone_count: u32,

    // Draw commands for this frame
    pub draw_commands: Vec<ZDrawCommand>,

    // Pending resource uploads
    pub pending_textures: Vec<PendingTexture>,
    pub pending_meshes: Vec<PendingMesh>,

    // Resource handle allocation
    pub next_texture_handle: u32,
    pub next_mesh_handle: u32,

    // Init configuration
    pub init_config: ZInitConfig,
}

impl Default for ZFFIState {
    fn default() -> Self {
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
            render_mode: 0,
            matcap_blend_modes: [0; 4],
            material_metallic: 0.0,
            material_roughness: 0.5,
            material_emissive: 0.0,
            lights: [LightState::default(); 4],
            bone_matrices: Vec::new(),
            bone_count: 0,
            draw_commands: Vec::new(),
            pending_textures: Vec::new(),
            pending_meshes: Vec::new(),
            next_texture_handle: 1, // 0 reserved for invalid
            next_mesh_handle: 1,
            init_config: ZInitConfig::default(),
        }
    }
}

impl ZFFIState {
    /// Create new FFI state with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Clear all commands and reset for next frame
    pub fn clear_frame(&mut self) {
        self.draw_commands.clear();
        self.pending_textures.clear();
        self.pending_meshes.clear();
        // Note: Camera, transforms, render state persist between clear_frame calls
        // within a single frame, but the entire ZFFIState is rebuilt each game frame
    }
}
