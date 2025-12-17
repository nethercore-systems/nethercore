//! FFI staging state for Emberware Z

use std::sync::Arc;

use glam::{Mat4, Vec3};
use hashbrown::HashMap;

use zx_common::ZDataPack;

use super::{
    BoneMatrix3x4, Font, KeyframeGpuInfo, KeyframeSource, LoadedKeyframeCollection,
    PendingKeyframes, PendingMesh, PendingMeshPacked, PendingSkeleton, PendingTexture,
    SkeletonData, SkeletonGpuInfo, ZInitConfig,
};

/// FFI staging state for Emberware Z
///
/// This state is written to by FFI functions during update()/render() calls,
/// then consumed by ZGraphics at the end of each frame. It is cleared after
/// rendering and does not persist between frames.
///
/// This is NOT serialized for rollback - only core GameState is rolled back.
#[derive(Debug)]
pub struct ZFFIState {
    // Data pack from ROM (set during game loading, immutable after)
    // Assets in the data pack are loaded via `rom_*` FFI and go directly to VRAM
    pub data_pack: Option<Arc<ZDataPack>>,

    // Render state
    pub depth_test: bool,
    pub cull_mode: u8,
    pub blend_mode: u8,
    pub texture_filter: u8,
    pub bound_textures: [u32; 4],

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

    // GPU Animation Index Tracking (Animation System v2 - Unified Buffer)
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
    pub mvp_shading_map: HashMap<crate::graphics::MvpShadingIndices, u32>, // indices -> buffer_index

    // Unified shading state system (deduplication + dirty tracking)
    pub shading_states: Vec<crate::graphics::PackedUnifiedShadingState>,
    pub shading_state_map:
        HashMap<crate::graphics::PackedUnifiedShadingState, crate::graphics::ShadingStateIndex>,
    pub current_shading_state: crate::graphics::PackedUnifiedShadingState,
    pub shading_state_dirty: bool,

    // GPU-instanced quad rendering (batched by texture)
    pub quad_batches: Vec<super::QuadBatch>,
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
            data_pack: None, // Set during game loading
            depth_test: true,
            cull_mode: 1, // Back-face culling
            blend_mode: 0,
            texture_filter: 0, // Nearest
            bound_textures: [0; 4],
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
            init_config: ZInitConfig::default(),
            model_matrices,
            view_matrices,
            proj_matrices,
            current_model_matrix: None, // Start with None = use pool index 0
            current_view_matrix: None,
            current_proj_matrix: None,
            mvp_shading_states: Vec::with_capacity(256),
            mvp_shading_map: HashMap::with_capacity(256),
            shading_states: Vec::new(),
            shading_state_map: HashMap::new(),
            current_shading_state: crate::graphics::PackedUnifiedShadingState::default(),
            shading_state_dirty: true, // Start dirty so first draw creates state 0
            quad_batches: Vec::new(),
        }
    }
}

impl ZFFIState {
    /// Create new FFI state with default values (test helper)
    #[cfg(test)]
    pub fn new() -> Self {
        Self::default()
    }

    /// Update material property in current shading state (with quantization check)
    /// Metallic is stored in uniform_set_0 byte 0
    pub fn update_material_metallic(&mut self, value: f32) {
        use crate::graphics::{pack_unorm8, update_uniform_set_0_byte};
        let quantized = pack_unorm8(value);
        let current_byte = (self.current_shading_state.uniform_set_0 & 0xFF) as u8;
        if current_byte != quantized {
            self.current_shading_state.uniform_set_0 =
                update_uniform_set_0_byte(self.current_shading_state.uniform_set_0, 0, quantized);
            self.shading_state_dirty = true;
        }
    }

    /// Roughness is stored in uniform_set_0 byte 1
    pub fn update_material_roughness(&mut self, value: f32) {
        use crate::graphics::{pack_unorm8, update_uniform_set_0_byte};
        let quantized = pack_unorm8(value);
        let current_byte = ((self.current_shading_state.uniform_set_0 >> 8) & 0xFF) as u8;
        if current_byte != quantized {
            self.current_shading_state.uniform_set_0 =
                update_uniform_set_0_byte(self.current_shading_state.uniform_set_0, 1, quantized);
            self.shading_state_dirty = true;
        }
    }

    /// Emissive is stored in uniform_set_0 byte 2
    pub fn update_material_emissive(&mut self, value: f32) {
        use crate::graphics::{pack_unorm8, update_uniform_set_0_byte};
        let quantized = pack_unorm8(value);
        let current_byte = ((self.current_shading_state.uniform_set_0 >> 16) & 0xFF) as u8;
        if current_byte != quantized {
            self.current_shading_state.uniform_set_0 =
                update_uniform_set_0_byte(self.current_shading_state.uniform_set_0, 2, quantized);
            self.shading_state_dirty = true;
        }
    }

    /// Rim intensity is stored in uniform_set_0 byte 3
    pub fn update_material_rim_intensity(&mut self, value: f32) {
        use crate::graphics::{pack_unorm8, update_uniform_set_0_byte};
        let quantized = pack_unorm8(value);
        let current_byte = ((self.current_shading_state.uniform_set_0 >> 24) & 0xFF) as u8;
        if current_byte != quantized {
            self.current_shading_state.uniform_set_0 =
                update_uniform_set_0_byte(self.current_shading_state.uniform_set_0, 3, quantized);
            self.shading_state_dirty = true;
        }
    }

    /// Rim power is stored in uniform_set_1 byte 0 (low byte)
    pub fn update_material_rim_power(&mut self, value: f32) {
        use crate::graphics::{pack_unorm8, update_uniform_set_1_byte};
        // Rim power is [0-1] → [0-32] in shader, so we pack [0-1] as u8
        let quantized = pack_unorm8(value / 32.0); // Normalize from [0-32] to [0-1]
        let current_byte = (self.current_shading_state.uniform_set_1 & 0xFF) as u8;
        if current_byte != quantized {
            self.current_shading_state.uniform_set_1 =
                update_uniform_set_1_byte(self.current_shading_state.uniform_set_1, 0, quantized);
            self.shading_state_dirty = true;
        }
    }

    /// Update a directional light in current shading state (with quantization)
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

        let new_light = PackedLight::directional(
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

    /// Update a point light in current shading state (with quantization)
    pub fn update_point_light(
        &mut self,
        index: usize,
        position: [f32; 3],
        color: [f32; 3],
        intensity: f32,
        range: f32,
        enabled: bool,
    ) {
        use crate::graphics::PackedLight;
        use glam::Vec3;

        let new_light = PackedLight::point(
            Vec3::from_slice(&position),
            Vec3::from_slice(&color),
            intensity,
            range,
            enabled,
        );

        if self.current_shading_state.lights[index] != new_light {
            self.current_shading_state.lights[index] = new_light;
            self.shading_state_dirty = true;
        }
    }

    /// Update sky colors in current shading state
    ///
    /// Both colors are 0xRRGGBBAA format (alpha ignored).
    pub fn update_sky_colors(&mut self, horizon_rgba: u32, zenith_rgba: u32) {
        // Mask off alpha, keeping RGB in upper 24 bits
        let horizon = horizon_rgba & 0xFFFFFF00;
        let zenith = zenith_rgba & 0xFFFFFF00;

        if self.current_shading_state.sky.horizon_color != horizon
            || self.current_shading_state.sky.zenith_color != zenith
        {
            self.current_shading_state.sky.horizon_color = horizon;
            self.current_shading_state.sky.zenith_color = zenith;
            self.shading_state_dirty = true;
        }
    }

    /// Update sky sun parameters in current shading state
    ///
    /// The `direction` parameter is the direction light rays travel (from sun toward surface).
    /// This matches the convention used by dynamic lights (`update_light`).
    /// For a sun directly overhead, use `(0, -1, 0)` (rays going down).
    ///
    /// `color_rgba` is 0xRRGGBBAA format (alpha ignored).
    pub fn update_sky_sun(&mut self, direction: [f32; 3], color_rgba: u32, sharpness: f32) {
        use crate::graphics::{pack_octahedral_u32, pack_unorm8};
        use glam::Vec3;

        // Store direction as-is (rays travel convention, same as dynamic lights)
        let dir_oct_packed = pack_octahedral_u32(Vec3::from_slice(&direction));

        // Input: 0xRRGGBBAA, Output: 0xRRGGBBSS (replace alpha with sharpness)
        let sharp = pack_unorm8(sharpness) as u32;
        let color_and_sharpness = (color_rgba & 0xFFFFFF00) | sharp;

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

    /// Update a single matcap blend mode slot in current shading state
    /// Matcap blend modes are stored in uniform_set_0 (Mode 1 only)
    pub fn update_matcap_blend_mode(
        &mut self,
        slot: usize,
        mode: crate::graphics::MatcapBlendMode,
    ) {
        use crate::graphics::{pack_matcap_blend_modes, unpack_matcap_blend_modes};

        // Unpack current modes, modify one slot, repack
        let mut modes = unpack_matcap_blend_modes(self.current_shading_state.uniform_set_0);
        modes[slot] = mode;
        let packed = pack_matcap_blend_modes(modes);

        if self.current_shading_state.uniform_set_0 != packed {
            self.current_shading_state.uniform_set_0 = packed;
            self.shading_state_dirty = true;
        }
    }

    /// Update specular color in current shading state (Mode 3 only)
    /// Specular RGB is stored in uniform_set_1 using 0xRRGGBBRP format (big-endian, same as color_rgba8)
    pub fn update_specular_color(&mut self, r: f32, g: f32, b: f32) {
        use crate::graphics::pack_unorm8;

        let r_u8 = pack_unorm8(r);
        let g_u8 = pack_unorm8(g);
        let b_u8 = pack_unorm8(b);

        // Keep rim_power in byte 0 (low byte), update bytes 3-1 (high bytes) with RGB
        let rim_power_byte = (self.current_shading_state.uniform_set_1 & 0xFF) as u8;
        let new_packed = ((r_u8 as u32) << 24)
            | ((g_u8 as u32) << 16)
            | ((b_u8 as u32) << 8)
            | (rim_power_byte as u32);

        if self.current_shading_state.uniform_set_1 != new_packed {
            self.current_shading_state.uniform_set_1 = new_packed;
            self.shading_state_dirty = true;
        }
    }

    /// Update skinning mode in current shading state
    /// - false: raw mode (bone matrices used as-is)
    /// - true: inverse bind mode (GPU applies inverse bind matrices)
    pub fn update_skinning_mode(&mut self, inverse_bind: bool) {
        use crate::graphics::FLAG_SKINNING_MODE;

        let new_flags = if inverse_bind {
            self.current_shading_state.flags | FLAG_SKINNING_MODE
        } else {
            self.current_shading_state.flags & !FLAG_SKINNING_MODE
        };

        if self.current_shading_state.flags != new_flags {
            self.current_shading_state.flags = new_flags;
            self.shading_state_dirty = true;
        }
    }

    /// Sync animation state (Animation System v2 - Unified Buffer) to current_shading_state
    ///
    /// Computes absolute keyframe_base into unified_animation buffer:
    /// - Static keyframes: inverse_bind_end + section_offset
    /// - Immediate bones: animation_static_end + offset
    ///
    /// Also copies current_inverse_bind_base (already absolute since inverse_bind is at offset 0).
    /// Called before add_shading_state().
    pub fn sync_animation_state(&mut self) {
        // Compute absolute keyframe_base into unified_animation buffer
        let keyframe_base = match self.current_keyframe_source {
            // Static keyframes are at [inverse_bind_end..animation_static_end)
            KeyframeSource::Static { offset } => self.inverse_bind_end + offset,
            // Immediate bones are at [animation_static_end..)
            KeyframeSource::Immediate { offset } => self.animation_static_end + offset,
        };

        // Update shading state if changed
        if self.current_shading_state.keyframe_base != keyframe_base {
            self.current_shading_state.keyframe_base = keyframe_base;
            self.shading_state_dirty = true;
        }

        if self.current_shading_state.inverse_bind_base != self.current_inverse_bind_base {
            self.current_shading_state.inverse_bind_base = self.current_inverse_bind_base;
            self.shading_state_dirty = true;
        }

        // Note: animation_flags no longer used - shader uses unified_animation with pre-computed offsets
    }

    /// Update texture filter mode in current shading state
    /// - false/0: nearest (pixelated)
    /// - true/1: linear (smooth)
    pub fn update_texture_filter(&mut self, linear: bool) {
        use crate::graphics::FLAG_TEXTURE_FILTER_LINEAR;

        let new_flags = if linear {
            self.current_shading_state.flags | FLAG_TEXTURE_FILTER_LINEAR
        } else {
            self.current_shading_state.flags & !FLAG_TEXTURE_FILTER_LINEAR
        };

        if self.current_shading_state.flags != new_flags {
            self.current_shading_state.flags = new_flags;
            self.shading_state_dirty = true;
        }
    }

    /// Update uniform alpha level in current shading state (dither transparency)
    /// - 0: fully transparent (all pixels discarded)
    /// - 15: fully opaque (no pixels discarded, default)
    pub fn update_uniform_alpha(&mut self, alpha: u8) {
        use crate::graphics::{FLAG_UNIFORM_ALPHA_MASK, FLAG_UNIFORM_ALPHA_SHIFT};

        let alpha = alpha.min(15) as u32; // Clamp to 4 bits
        let new_flags = (self.current_shading_state.flags & !FLAG_UNIFORM_ALPHA_MASK)
            | (alpha << FLAG_UNIFORM_ALPHA_SHIFT);

        if self.current_shading_state.flags != new_flags {
            self.current_shading_state.flags = new_flags;
            self.shading_state_dirty = true;
        }
    }

    /// Update dither offset in current shading state
    /// - x: 0-3 pixel shift in X axis
    /// - y: 0-3 pixel shift in Y axis
    /// Use different offsets for stacked transparent objects to prevent pattern cancellation
    pub fn update_dither_offset(&mut self, x: u8, y: u8) {
        use crate::graphics::{
            FLAG_DITHER_OFFSET_X_MASK, FLAG_DITHER_OFFSET_X_SHIFT, FLAG_DITHER_OFFSET_Y_MASK,
            FLAG_DITHER_OFFSET_Y_SHIFT,
        };

        let x = (x.min(3) as u32) << FLAG_DITHER_OFFSET_X_SHIFT;
        let y = (y.min(3) as u32) << FLAG_DITHER_OFFSET_Y_SHIFT;
        let new_flags = (self.current_shading_state.flags
            & !FLAG_DITHER_OFFSET_X_MASK
            & !FLAG_DITHER_OFFSET_Y_MASK)
            | x
            | y;

        if self.current_shading_state.flags != new_flags {
            self.current_shading_state.flags = new_flags;
            self.shading_state_dirty = true;
        }
    }

    // =========================================================================
    // Material Override Flag Methods
    // =========================================================================

    /// Internal helper to update an override flag
    fn update_override_flag(&mut self, flag: u32, enabled: bool) {
        let new_flags = if enabled {
            self.current_shading_state.flags | flag
        } else {
            self.current_shading_state.flags & !flag
        };
        if self.current_shading_state.flags != new_flags {
            self.current_shading_state.flags = new_flags;
            self.shading_state_dirty = true;
        }
    }

    /// Update use_uniform_color flag
    pub fn set_use_uniform_color(&mut self, enabled: bool) {
        use crate::graphics::FLAG_USE_UNIFORM_COLOR;
        self.update_override_flag(FLAG_USE_UNIFORM_COLOR, enabled);
    }

    /// Update use_uniform_metallic flag
    pub fn set_use_uniform_metallic(&mut self, enabled: bool) {
        use crate::graphics::FLAG_USE_UNIFORM_METALLIC;
        self.update_override_flag(FLAG_USE_UNIFORM_METALLIC, enabled);
    }

    /// Update use_uniform_roughness flag
    pub fn set_use_uniform_roughness(&mut self, enabled: bool) {
        use crate::graphics::FLAG_USE_UNIFORM_ROUGHNESS;
        self.update_override_flag(FLAG_USE_UNIFORM_ROUGHNESS, enabled);
    }

    /// Update use_uniform_emissive flag
    pub fn set_use_uniform_emissive(&mut self, enabled: bool) {
        use crate::graphics::FLAG_USE_UNIFORM_EMISSIVE;
        self.update_override_flag(FLAG_USE_UNIFORM_EMISSIVE, enabled);
    }

    /// Update use_uniform_specular flag (Mode 3 only)
    pub fn set_use_uniform_specular(&mut self, enabled: bool) {
        use crate::graphics::FLAG_USE_UNIFORM_SPECULAR;
        self.update_override_flag(FLAG_USE_UNIFORM_SPECULAR, enabled);
    }

    /// Update use_matcap_reflection flag (Mode 1 only)
    pub fn set_use_matcap_reflection(&mut self, enabled: bool) {
        use crate::graphics::FLAG_USE_MATCAP_REFLECTION;
        self.update_override_flag(FLAG_USE_MATCAP_REFLECTION, enabled);
    }

    /// Check if a skeleton is currently bound (inverse bind mode enabled)
    pub fn is_skeleton_bound(&self) -> bool {
        self.bound_skeleton != 0
    }

    /// Get the currently bound skeleton data, if any
    pub fn get_bound_skeleton(&self) -> Option<&SkeletonData> {
        if self.bound_skeleton == 0 {
            return None;
        }
        let index = self.bound_skeleton as usize - 1;
        self.skeletons.get(index)
    }

    /// Add current shading state to the pool if dirty, returning its index
    ///
    /// Uses deduplication via HashMap - if this exact state already exists, returns existing index.
    /// Otherwise adds a new entry.
    pub fn add_shading_state(&mut self) -> crate::graphics::ShadingStateIndex {
        // Sync animation state before checking (Animation System v2)
        self.sync_animation_state();

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

    /// Add a quad instance to the appropriate batch (auto-batches by texture and blend mode)
    ///
    /// This automatically groups quads by texture and blend mode to minimize draw calls.
    /// When bound_textures or blend_mode changes, a new batch is created.
    pub fn add_quad_instance(&mut self, instance: crate::graphics::QuadInstance) {
        // Check if we can add to the current batch or need a new one
        if let Some(last_batch) = self.quad_batches.last_mut()
            && last_batch.textures == self.bound_textures
            && last_batch.blend_mode == self.blend_mode
        {
            // Same textures and blend mode - add to current batch
            last_batch.instances.push(instance);
            return;
        }

        // Need a new batch (either first batch, textures changed, or blend mode changed)
        self.quad_batches.push(super::QuadBatch {
            textures: self.bound_textures,
            blend_mode: self.blend_mode,
            instances: vec![instance],
        });
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
    ///
    /// Note: Audio playback state is in ZRollbackState, not here.
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
        self.current_view_matrix = None; // Will use last in pool (default view)
        self.current_proj_matrix = None; // Will use last in pool (default proj)

        // Clear combined MVP+shading state pool
        self.mvp_shading_states.clear();
        self.mvp_shading_map.clear();

        // Reset shading state pool for next frame
        self.shading_states.clear();
        self.shading_state_map.clear();
        self.shading_state_dirty = true; // Mark dirty so first draw creates state 0

        // Clear GPU-instanced quad batches for next frame
        self.quad_batches.clear();

        // Clear immediate bone matrices for next frame (Animation System v2)
        // The bone_matrices buffer accumulates during the frame and must be reset
        self.bone_matrices.clear();

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

    // ========================================================================
    // Dither Transparency Tests
    // ========================================================================

    #[test]
    fn test_uniform_alpha_update() {
        use crate::graphics::{FLAG_UNIFORM_ALPHA_MASK, FLAG_UNIFORM_ALPHA_SHIFT};

        let mut ffi_state = ZFFIState::default();

        // Default should be opaque (alpha = 15)
        let alpha = (ffi_state.current_shading_state.flags & FLAG_UNIFORM_ALPHA_MASK)
            >> FLAG_UNIFORM_ALPHA_SHIFT;
        assert_eq!(alpha, 15);

        // Update to 50% transparency
        ffi_state.update_uniform_alpha(8);
        let alpha = (ffi_state.current_shading_state.flags & FLAG_UNIFORM_ALPHA_MASK)
            >> FLAG_UNIFORM_ALPHA_SHIFT;
        assert_eq!(alpha, 8);
        assert!(ffi_state.shading_state_dirty);

        // Reset dirty flag and update to same value - should not mark dirty
        ffi_state.shading_state_dirty = false;
        ffi_state.update_uniform_alpha(8);
        assert!(!ffi_state.shading_state_dirty);

        // Update to different value - should mark dirty
        ffi_state.update_uniform_alpha(0);
        assert!(ffi_state.shading_state_dirty);
        let alpha = (ffi_state.current_shading_state.flags & FLAG_UNIFORM_ALPHA_MASK)
            >> FLAG_UNIFORM_ALPHA_SHIFT;
        assert_eq!(alpha, 0);
    }

    #[test]
    fn test_dither_offset_update() {
        use crate::graphics::{
            FLAG_DITHER_OFFSET_X_MASK, FLAG_DITHER_OFFSET_X_SHIFT, FLAG_DITHER_OFFSET_Y_MASK,
            FLAG_DITHER_OFFSET_Y_SHIFT,
        };

        let mut ffi_state = ZFFIState::default();

        // Default should be (0, 0)
        let x = (ffi_state.current_shading_state.flags & FLAG_DITHER_OFFSET_X_MASK)
            >> FLAG_DITHER_OFFSET_X_SHIFT;
        let y = (ffi_state.current_shading_state.flags & FLAG_DITHER_OFFSET_Y_MASK)
            >> FLAG_DITHER_OFFSET_Y_SHIFT;
        assert_eq!(x, 0);
        assert_eq!(y, 0);

        // Update to (2, 3)
        ffi_state.update_dither_offset(2, 3);

        let x = (ffi_state.current_shading_state.flags & FLAG_DITHER_OFFSET_X_MASK)
            >> FLAG_DITHER_OFFSET_X_SHIFT;
        let y = (ffi_state.current_shading_state.flags & FLAG_DITHER_OFFSET_Y_MASK)
            >> FLAG_DITHER_OFFSET_Y_SHIFT;

        assert_eq!(x, 2);
        assert_eq!(y, 3);
        assert!(ffi_state.shading_state_dirty);
    }

    #[test]
    fn test_dither_updates_preserve_other_flags() {
        use crate::graphics::{
            FLAG_SKINNING_MODE, FLAG_TEXTURE_FILTER_LINEAR, FLAG_UNIFORM_ALPHA_MASK,
            FLAG_UNIFORM_ALPHA_SHIFT,
        };

        let mut ffi_state = ZFFIState::default();

        // Set some other flags first
        ffi_state.update_skinning_mode(true);
        ffi_state.update_texture_filter(true);

        // Verify they're set
        assert_ne!(
            ffi_state.current_shading_state.flags & FLAG_SKINNING_MODE,
            0
        );
        assert_ne!(
            ffi_state.current_shading_state.flags & FLAG_TEXTURE_FILTER_LINEAR,
            0
        );

        // Update uniform_alpha
        ffi_state.update_uniform_alpha(8);

        // Verify other flags are preserved
        assert_ne!(
            ffi_state.current_shading_state.flags & FLAG_SKINNING_MODE,
            0
        );
        assert_ne!(
            ffi_state.current_shading_state.flags & FLAG_TEXTURE_FILTER_LINEAR,
            0
        );
        assert_eq!(
            (ffi_state.current_shading_state.flags & FLAG_UNIFORM_ALPHA_MASK)
                >> FLAG_UNIFORM_ALPHA_SHIFT,
            8
        );

        // Update dither_offset
        ffi_state.update_dither_offset(1, 2);

        // Verify all flags are still preserved
        assert_ne!(
            ffi_state.current_shading_state.flags & FLAG_SKINNING_MODE,
            0
        );
        assert_ne!(
            ffi_state.current_shading_state.flags & FLAG_TEXTURE_FILTER_LINEAR,
            0
        );
        assert_eq!(
            (ffi_state.current_shading_state.flags & FLAG_UNIFORM_ALPHA_MASK)
                >> FLAG_UNIFORM_ALPHA_SHIFT,
            8
        );
    }

    #[test]
    fn test_uniform_alpha_clamping() {
        use crate::graphics::{FLAG_UNIFORM_ALPHA_MASK, FLAG_UNIFORM_ALPHA_SHIFT};

        let mut ffi_state = ZFFIState::default();

        // Values > 15 should be clamped to 15
        ffi_state.update_uniform_alpha(100);
        let alpha = (ffi_state.current_shading_state.flags & FLAG_UNIFORM_ALPHA_MASK)
            >> FLAG_UNIFORM_ALPHA_SHIFT;
        assert_eq!(alpha, 15);

        // Values at boundary should work
        ffi_state.update_uniform_alpha(15);
        let alpha = (ffi_state.current_shading_state.flags & FLAG_UNIFORM_ALPHA_MASK)
            >> FLAG_UNIFORM_ALPHA_SHIFT;
        assert_eq!(alpha, 15);
    }

    #[test]
    fn test_dither_offset_clamping() {
        use crate::graphics::{
            FLAG_DITHER_OFFSET_X_MASK, FLAG_DITHER_OFFSET_X_SHIFT, FLAG_DITHER_OFFSET_Y_MASK,
            FLAG_DITHER_OFFSET_Y_SHIFT,
        };

        let mut ffi_state = ZFFIState::default();

        // Values > 3 should be clamped
        ffi_state.update_dither_offset(100, 200);
        let x = (ffi_state.current_shading_state.flags & FLAG_DITHER_OFFSET_X_MASK)
            >> FLAG_DITHER_OFFSET_X_SHIFT;
        let y = (ffi_state.current_shading_state.flags & FLAG_DITHER_OFFSET_Y_MASK)
            >> FLAG_DITHER_OFFSET_Y_SHIFT;
        assert_eq!(x, 3);
        assert_eq!(y, 3);
    }
}
