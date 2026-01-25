//! FFI staging state for Nethercore ZX

use std::sync::Arc;

use glam::{Mat4, Vec3};
use hashbrown::HashMap;

use zx_common::ZXDataPack;

use super::{
    BoneMatrix3x4, Font, KeyframeGpuInfo, KeyframeSource, LoadedKeyframeCollection,
    PendingKeyframes, PendingMesh, PendingMeshPacked, PendingSkeleton, PendingTexture,
    SkeletonData, SkeletonGpuInfo, StatePool, ZXInitConfig,
};

use crate::graphics::epu::EpuConfig;

// Re-export submodules
mod material;
mod rendering;
mod resource;

#[cfg(test)]
mod tests;

/// Default z-index for 2D rendering (background layer)
///
/// Z-index 0 represents the back-most layer. Higher values render on top.
/// This is the default z-index that's reset each frame.
pub const DEFAULT_Z_INDEX: u32 = 0;

/// FFI staging state for Nethercore ZX
///
/// This state is written to by FFI functions during update()/render() calls,
/// then consumed by ZXGraphics at the end of each frame. It is cleared after
/// rendering and does not persist between frames.
///
/// This is NOT serialized for rollback - only core GameState is rolled back.
#[derive(Debug)]
pub struct ZXFFIState {
    // Data pack from ROM (set during game loading, immutable after)
    // Assets in the data pack are loaded via `rom_*` FFI and go directly to VRAM
    pub data_pack: Option<Arc<ZXDataPack>>,

    // Render state
    pub cull_mode: crate::graphics::CullMode,
    pub texture_filter: crate::graphics::TextureFilter,
    pub bound_textures: [u32; 4],
    /// Current z-index for 2D draw ordering (higher = closer to camera)
    pub current_z_index: u32,
    /// Current viewport for split-screen rendering (default: fullscreen)
    pub current_viewport: crate::graphics::Viewport,

    // Render pass system (replaces stencil_mode/stencil_group/depth_test)
    /// Current pass ID (increments on each begin_pass_*() call)
    pub current_pass_id: u32,
    /// Pass configurations (indexed by pass_id)
    pub pass_configs: Vec<crate::graphics::PassConfig>,

    // GPU skinning (3x4 matrices for 25% memory savings)
    pub bone_matrices: Vec<BoneMatrix3x4>,
    pub bone_count: u32,

    // Skeleton system (inverse bind matrices for GPU skinning)
    /// Loaded skeletons (index 0 is reserved, handles are 1-indexed)
    pub skeletons: Vec<SkeletonData>,
    /// Currently bound skeleton handle (0 = no skeleton bound, raw mode)
    pub bound_skeleton: u32,
    /// Next skeleton handle to allocate
    pub next_skeleton_handle: u32,

    // Keyframe system (animation clips stored on host)
    /// Loaded keyframe collections (handles are 1-indexed)
    pub keyframes: Vec<LoadedKeyframeCollection>,
    /// Pending keyframe loads (processed after init())
    pub pending_keyframes: Vec<PendingKeyframes>,
    /// Next keyframe handle to allocate
    pub next_keyframe_handle: u32,

    // GPU Animation Index Tracking (Unified Buffer)
    /// Tracks where each skeleton's inverse bind matrices are in unified_animation
    /// Index = skeleton_handle - 1 (handles are 1-indexed)
    pub skeleton_gpu_info: Vec<SkeletonGpuInfo>,
    /// Tracks where each keyframe collection's data is in unified_animation
    /// Index = keyframe_handle - 1 (handles are 1-indexed)
    pub keyframe_gpu_info: Vec<KeyframeGpuInfo>,
    /// Current keyframe source for skinned draws (Static or Immediate)
    pub current_keyframe_source: KeyframeSource,
    /// Current inverse bind base offset (already absolute - inverse_bind at offset 0)
    pub current_inverse_bind_base: u32,
    /// Offset where inverse_bind section ends = keyframes section starts (set after static upload)
    pub inverse_bind_end: u32,
    /// Offset where static data ends = immediate section starts (set after static upload)
    pub animation_static_end: u32,

    // Virtual Render Pass (direct recording)
    pub render_pass: crate::graphics::VirtualRenderPass,

    // Mesh metadata mapping (for FFI access to mesh info)
    pub mesh_map: hashbrown::HashMap<u32, crate::graphics::RetainedMesh>,

    // Pending resource uploads (processed after init())
    pub pending_textures: Vec<PendingTexture>,
    pub pending_meshes: Vec<PendingMesh>,
    pub pending_meshes_packed: Vec<PendingMeshPacked>,
    pub pending_skeletons: Vec<PendingSkeleton>,

    // Resource handle allocation
    pub next_texture_handle: u32,
    pub next_mesh_handle: u32,
    pub next_font_handle: u32,

    // Font system
    pub fonts: Vec<Font>,
    pub current_font: u32,

    // Audio system (sounds stored here for FFI access, playback state in ZRollbackState)
    pub sounds: Vec<Option<crate::audio::Sound>>,
    pub next_sound_handle: u32,
    /// Sound ID -> handle mapping (for tracker sample resolution)
    pub sound_id_to_handle: HashMap<String, u32>,

    // Tracker system (XM module playback, state in ZRollbackState, engine here)
    pub tracker_engine: crate::tracker::TrackerEngine,

    // Init configuration
    pub init_config: ZXInitConfig,

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
    pub mvp_shading_map: HashMap<crate::graphics::MvpShadingIndices, u32>, // indices -> buffer_index

    // Unified shading state system (deduplication + dirty tracking)
    pub shading_pool:
        StatePool<crate::graphics::PackedUnifiedShadingState, crate::graphics::ShadingStateIndex>,
    pub current_shading_state: crate::graphics::PackedUnifiedShadingState,
    pub shading_state_dirty: bool,

    // GPU-instanced quad rendering (batched by texture)
    pub quad_batches: Vec<super::QuadBatch>,

    // Diagnostics (reset each frame)
    pub mvp_shading_overflowed_this_frame: bool,
    pub mvp_shading_overflow_count: u32,

    // EPU (Environment Processing Unit) state - instruction-based API (push-only)
    /// EPU configs pushed for this frame, keyed by `env_id`.
    ///
    /// - `epu_set(...)` stores a config for the **currently selected** `environment_index(...)`.
    /// - `epu_set_env(env_id, ...)` stores a config for an explicit `env_id` without drawing.
    /// - `draw_epu()` records a background draw request for the current viewport/pass.
    ///
    /// Any `env_id` that does not have an explicit config falls back to:
    /// 1) `env_id = 0` if present, else
    /// 2) the built-in default environment config.
    pub epu_frame_configs: HashMap<u32, EpuConfig>,
    /// EPU draw requests for this frame.
    ///
    /// Keyed by (viewport, pass_id) so split-screen and multi-pass rendering can
    /// request an environment draw per pass. If `draw_epu()` is called multiple
    /// times for the same key, only the last call wins.
    ///
    /// The value is an index into `mvp_shading_indices` (instance_index) so the
    /// environment shader uses the correct view/proj + shading state.
    pub epu_frame_draws: HashMap<(crate::graphics::Viewport, u32), u32>,
    // NOTE: epu_ambient_cubes was removed - GPU readback would break rollback determinism
}

impl Default for ZXFFIState {
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
            data_pack: None, // Set during game loading
            cull_mode: crate::graphics::CullMode::None,
            texture_filter: crate::graphics::TextureFilter::Nearest,
            bound_textures: [0; 4],
            current_z_index: DEFAULT_Z_INDEX,
            current_viewport: crate::graphics::Viewport::FULLSCREEN,
            // Render pass system - pass 0 is always the default pass
            current_pass_id: 0,
            pass_configs: vec![crate::graphics::PassConfig::default()],
            bone_matrices: Vec::new(),
            bone_count: 0,
            skeletons: Vec::new(),
            bound_skeleton: 0,
            next_skeleton_handle: 1, // 0 reserved for "no skeleton"
            keyframes: Vec::new(),
            pending_keyframes: Vec::new(),
            next_keyframe_handle: 1, // 0 reserved for "invalid"
            skeleton_gpu_info: Vec::new(),
            keyframe_gpu_info: Vec::new(),
            current_keyframe_source: KeyframeSource::default(),
            current_inverse_bind_base: 0,
            inverse_bind_end: 0,
            animation_static_end: 0,
            render_pass: crate::graphics::VirtualRenderPass::new(),
            mesh_map: hashbrown::HashMap::new(),
            pending_textures: Vec::new(),
            pending_meshes: Vec::new(),
            pending_meshes_packed: Vec::new(),
            pending_skeletons: Vec::new(),
            next_texture_handle: 1, // 0 reserved for invalid
            next_mesh_handle: 1,
            next_font_handle: 1,
            fonts: Vec::new(),
            current_font: 0, // 0 = built-in font
            sounds: Vec::new(),
            next_sound_handle: 1, // 0 reserved for invalid
            sound_id_to_handle: HashMap::new(),
            tracker_engine: crate::tracker::TrackerEngine::new(),
            init_config: ZXInitConfig::default(),
            model_matrices,
            view_matrices,
            proj_matrices,
            current_model_matrix: None, // Start with None = use pool index 0
            current_view_matrix: None,
            current_proj_matrix: None,
            mvp_shading_states: Vec::with_capacity(256),
            mvp_shading_map: HashMap::with_capacity(256),
            shading_pool: StatePool::new("Shading state", 65536),
            current_shading_state: crate::graphics::PackedUnifiedShadingState::default(),
            shading_state_dirty: true, // Start dirty so first draw creates state 0
            quad_batches: Vec::new(),
            mvp_shading_overflowed_this_frame: false,
            mvp_shading_overflow_count: 0,
            // EPU (instruction-based) state (push-only)
            epu_frame_configs: HashMap::new(),
            epu_frame_draws: HashMap::new(),
        }
    }
}

impl ZXFFIState {
    /// Create new FFI state with default values (test helper)
    #[cfg(test)]
    pub fn new() -> Self {
        Self::default()
    }
}
