//! Nethercore ZX graphics backend (wgpu)
//!
//! Implements the `Graphics` trait from nethercore-core with a wgpu-based
//! renderer featuring PS1/N64 aesthetic (vertex jitter, affine textures).
//!
//! # Architecture
//!
//! **ZFFIState** (staging) → **ZGraphics** (GPU execution)
//!
//! - FFI functions write draw commands, transforms, and render state to ZFFIState
//! - App.rs passes ZFFIState to ZGraphics each frame
//! - ZGraphics consumes commands and executes them on the GPU
//! - ZGraphics owns all actual GPU resources (textures, meshes, buffers, pipelines)

mod buffer;
mod command_buffer;
mod draw;
mod frame;
mod init;
mod matrix_packing;
mod pipeline;
mod quad_instance;
mod render_state;
mod texture_manager;
mod unified_shading_state;
mod vertex;

use hashbrown::HashMap;

use anyhow::{Context, Result};

use nethercore_core::console::Graphics;

// Re-export packing utilities from zx-common (for FFI and backwards compat)
pub use zx_common::{
    FORMAT_COLOR, FORMAT_NORMAL, FORMAT_SKINNED, FORMAT_UV, pack_bone_weights_unorm8,
    pack_color_rgba_unorm8, pack_normal_octahedral, pack_normal_snorm16, pack_octahedral_u32,
    pack_position_f16, pack_uv_f16, pack_uv_unorm16, pack_vertex_data, unpack_octahedral_u32,
    vertex_stride, vertex_stride_packed,
};

// Re-export public types from submodules
pub use buffer::{BufferManager, GrowableBuffer, MeshHandle, RetainedMesh};
pub use command_buffer::{VRPCommand, VirtualRenderPass};
pub use matrix_packing::MvpShadingIndices;
pub use quad_instance::{QuadInstance, QuadMode};
pub use render_state::{CullMode, MatcapBlendMode, RenderState, TextureFilter, TextureHandle};
pub use unified_shading_state::{
    DEFAULT_FLAGS,
    EnvironmentIndex,
    FLAG_DITHER_OFFSET_X_MASK,
    FLAG_DITHER_OFFSET_X_SHIFT,
    FLAG_DITHER_OFFSET_Y_MASK,
    FLAG_DITHER_OFFSET_Y_SHIFT,
    FLAG_SKINNING_MODE,
    FLAG_TEXTURE_FILTER_LINEAR,
    FLAG_UNIFORM_ALPHA_MASK,
    FLAG_UNIFORM_ALPHA_SHIFT,
    FLAG_USE_MATCAP_REFLECTION,
    FLAG_USE_UNIFORM_COLOR,
    FLAG_USE_UNIFORM_EMISSIVE,
    FLAG_USE_UNIFORM_METALLIC,
    FLAG_USE_UNIFORM_ROUGHNESS,
    FLAG_USE_UNIFORM_SPECULAR,
    LightType,
    // Multi-Environment v3
    PackedEnvironmentState,
    PackedLight,
    PackedUnifiedShadingState,
    ShadingStateIndex,
    blend_mode,
    env_mode,
    pack_f16,
    pack_f16x2,
    pack_matcap_blend_modes,
    pack_rgb8,
    pack_unorm8,
    unpack_f16,
    unpack_f16x2,
    unpack_matcap_blend_modes,
    update_uniform_set_0_byte,
    update_uniform_set_1_byte,
};
pub use vertex::{FORMAT_ALL, VERTEX_FORMAT_COUNT, VertexFormatInfo};

// Re-export for crate-internal use
pub(crate) use init::RenderTarget;

use pipeline::PipelineCache;
use texture_manager::TextureManager;

/// Nethercore ZX graphics backend
///
/// Manages wgpu device, textures, render state, and frame presentation.
/// Implements the vertex buffer architecture with one buffer per stride
/// and command buffer pattern for draw batching.
pub struct ZGraphics {
    // Core wgpu objects
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,

    // Offscreen render target (game renders at fixed resolution)
    render_target: RenderTarget,

    // Blit pipeline (for scaling render target to window)
    blit_pipeline: wgpu::RenderPipeline,
    blit_bind_group: wgpu::BindGroup,

    // Depth buffer (for window-sized UI rendering, no longer used for game content)
    depth_texture: wgpu::Texture,
    depth_view: wgpu::TextureView,

    // Texture management (extracted to separate module)
    texture_manager: TextureManager,

    // Samplers
    sampler_nearest: wgpu::Sampler,
    sampler_linear: wgpu::Sampler,

    // =================================================================
    // UNIFIED BUFFER ARCHITECTURE
    // =================================================================
    // Merges similar matrix buffers to reduce storage buffer count from 9 to 4.

    // Unified transforms (@binding(0)): [models | views | projs]
    // All mat4x4 matrices uploaded each frame
    unified_transforms_buffer: wgpu::Buffer,
    unified_transforms_capacity: usize, // in mat4x4 count

    // Unified animation (@binding(3)): [inverse_bind | keyframes | immediate]
    // - Inverse bind: static, uploaded once after init
    // - Keyframes: static, uploaded once after init
    // - Immediate: per-frame, uploaded each frame
    unified_animation_buffer: wgpu::Buffer,
    unified_animation_capacity: usize, // in mat3x4 count
    /// Where inverse bind section ends in unified_animation (pub for state sync)
    pub inverse_bind_end: usize,
    /// Where static data ends in unified_animation (pub for state sync)
    pub animation_static_end: usize,

    // MVP indices buffer (@binding(1)) - absolute indices pre-computed by CPU
    mvp_indices_buffer: wgpu::Buffer,
    mvp_indices_capacity: usize,

    // Shading state storage buffer (per-frame array)
    shading_state_buffer: wgpu::Buffer,
    shading_state_capacity: usize,

    // Environment state storage buffer (Multi-Environment v3)
    // @binding(4) - per-frame array of PackedEnvironmentState
    environment_states_buffer: wgpu::Buffer,
    environment_states_capacity: usize,

    // Bind group caches
    texture_bind_groups: HashMap<[TextureHandle; 4], wgpu::BindGroup>,

    /// Cached frame bind group (@group(0)) - only recreated when buffers change
    /// This avoids wasteful GPU descriptor set creation every frame.
    cached_frame_bind_group: Option<wgpu::BindGroup>,

    /// Hash of buffer sizes/addresses to detect when bind group needs recreation
    cached_frame_bind_group_hash: u64,

    // Frame state
    current_frame: Option<wgpu::SurfaceTexture>,
    current_view: Option<wgpu::TextureView>,

    // Buffer management (vertex/index buffers and retained meshes)
    buffer_manager: BufferManager,

    // Command buffer for immediate mode draws
    command_buffer: VirtualRenderPass,

    // Shader and pipeline cache
    pipeline_cache: PipelineCache,
    current_render_mode: u8,

    // Scaling mode for render target to window
    scale_mode: nethercore_core::app::config::ScaleMode,

    // Unit quad mesh for GPU-instanced rendering (billboards, sprites, etc.)
    unit_quad_format: u8,
    unit_quad_base_vertex: u32,
    unit_quad_first_index: u32,

    // Persistent buffers for quad instance processing (avoids per-frame allocation)
    quad_instance_scratch: Vec<QuadInstance>,
    /// (base_instance, instance_count, textures, is_screen_space)
    quad_batch_scratch: Vec<(u32, u32, [u32; 4], bool)>,
}

impl ZGraphics {
    /// Update render target resolution if changed
    /// Note: Nethercore ZX uses a fixed 540p resolution, so this is a no-op
    pub fn update_resolution(&mut self) {
        // Fixed resolution - no dynamic changes needed
    }

    /// Update scaling mode for render target to window
    pub fn set_scale_mode(&mut self, scale_mode: nethercore_core::app::config::ScaleMode) {
        self.scale_mode = scale_mode;
    }

    /// Invalidate cached frame bind group, forcing recreation on next frame.
    /// Call this when buffers are recreated (e.g., after init animation data upload).
    pub fn invalidate_frame_bind_group(&mut self) {
        self.cached_frame_bind_group = None;
        self.cached_frame_bind_group_hash = 0;
    }

    // Texture Management
    pub fn load_texture(
        &mut self,
        width: u32,
        height: u32,
        pixels: &[u8],
    ) -> Result<TextureHandle> {
        self.texture_manager
            .load_texture(&self.device, &self.queue, width, height, pixels)
    }

    /// Load a texture with explicit format (RGBA8 or BC7)
    pub fn load_texture_with_format(
        &mut self,
        width: u32,
        height: u32,
        data: &[u8],
        format: zx_common::TextureFormat,
    ) -> Result<TextureHandle> {
        self.texture_manager.load_texture_with_format(
            &self.device,
            &self.queue,
            width,
            height,
            data,
            format,
        )
    }

    pub fn get_texture_view(&self, handle: TextureHandle) -> Option<&wgpu::TextureView> {
        self.texture_manager.get_texture_view(handle)
    }

    pub fn get_fallback_checkerboard_view(&self) -> &wgpu::TextureView {
        self.texture_manager.get_fallback_checkerboard_view()
    }

    pub fn get_fallback_white_view(&self) -> &wgpu::TextureView {
        self.texture_manager.get_fallback_white_view()
    }

    pub fn font_texture(&self) -> TextureHandle {
        self.texture_manager.font_texture()
    }

    pub fn white_texture(&self) -> TextureHandle {
        self.texture_manager.white_texture()
    }

    pub fn get_font_texture_view(&self) -> &wgpu::TextureView {
        self.texture_manager.get_font_texture_view()
    }

    pub fn get_texture_view_or_fallback(&self, handle: TextureHandle) -> &wgpu::TextureView {
        if handle == TextureHandle::INVALID {
            self.get_fallback_white_view()
        } else {
            self.get_texture_view(handle)
                .unwrap_or_else(|| self.get_fallback_checkerboard_view())
        }
    }

    pub fn vram_used(&self) -> usize {
        self.texture_manager.vram_used()
    }

    pub fn vram_limit(&self) -> usize {
        self.texture_manager.vram_limit()
    }

    // Note: Render state methods (set_depth_test, set_cull_mode, set_blend_mode,
    // set_texture_filter, render_state(), current_sampler()) have been removed.
    // Render state is now captured per-command from ZFFIState in draw.rs.
    // Texture filter is stored in PackedUnifiedShadingState.flags (bit 1) for
    // per-draw shader selection via sample_filtered() helper.

    // Mesh Loading
    pub fn load_mesh(&mut self, data: &[f32], format: u8) -> Result<MeshHandle> {
        self.buffer_manager
            .load_mesh(&self.device, &self.queue, data, format)
    }

    pub fn load_mesh_indexed(
        &mut self,
        data: &[f32],
        indices: &[u16],
        format: u8,
    ) -> Result<MeshHandle> {
        self.buffer_manager
            .load_mesh_indexed(&self.device, &self.queue, data, indices, format)
    }

    pub fn load_mesh_packed(&mut self, data: &[u8], format: u8) -> Result<MeshHandle> {
        self.buffer_manager
            .load_mesh_packed(&self.device, &self.queue, data, format)
    }

    pub fn load_mesh_indexed_packed(
        &mut self,
        data: &[u8],
        indices: &[u16],
        format: u8,
    ) -> Result<MeshHandle> {
        self.buffer_manager.load_mesh_indexed_packed(
            &self.device,
            &self.queue,
            data,
            indices,
            format,
        )
    }

    pub fn get_mesh(&self, handle: MeshHandle) -> Option<&RetainedMesh> {
        self.buffer_manager.get_mesh(handle)
    }

    // Command Buffer
    pub fn command_buffer(&self) -> &VirtualRenderPass {
        &self.command_buffer
    }

    pub fn command_buffer_mut(&mut self) -> &mut VirtualRenderPass {
        &mut self.command_buffer
    }

    pub fn reset_command_buffer(&mut self) {
        self.command_buffer.reset();
    }

    // Buffer Access
    pub fn vertex_buffer(&self, format: u8) -> &GrowableBuffer {
        self.buffer_manager.vertex_buffer(format)
    }

    pub fn index_buffer(&self, format: u8) -> &GrowableBuffer {
        self.buffer_manager.index_buffer(format)
    }

    // Device Access
    pub fn device(&self) -> &wgpu::Device {
        &self.device
    }

    pub fn queue(&self) -> &wgpu::Queue {
        &self.queue
    }

    // Capture Support (screenshot/GIF)
    /// Returns the dimensions of the render target (game resolution)
    pub fn render_target_dimensions(&self) -> (u32, u32) {
        (self.render_target.width, self.render_target.height)
    }

    /// Returns a reference to the render target color texture for capture
    pub fn render_target_texture(&self) -> &wgpu::Texture {
        &self.render_target.color_texture
    }

    pub fn surface_format(&self) -> wgpu::TextureFormat {
        self.config.format
    }

    pub fn depth_format(&self) -> wgpu::TextureFormat {
        wgpu::TextureFormat::Depth32Float
    }

    pub fn depth_view(&self) -> &wgpu::TextureView {
        &self.depth_view
    }

    pub fn dimensions(&self) -> (u32, u32) {
        (self.config.width, self.config.height)
    }

    pub fn width(&self) -> u32 {
        self.config.width
    }

    pub fn height(&self) -> u32 {
        self.config.height
    }

    pub fn get_current_texture(&mut self) -> Result<wgpu::SurfaceTexture> {
        self.surface
            .get_current_texture()
            .context("Failed to acquire next surface texture")
    }

    // Pipeline Management
    pub fn set_render_mode(&mut self, mode: u8) {
        if mode > 3 {
            tracing::warn!("Invalid render mode: {}, clamping to 3", mode);
            self.current_render_mode = 3;
        } else {
            self.current_render_mode = mode;
            tracing::info!(
                "Set render mode to {} ({})",
                mode,
                crate::shader_gen::mode_name(mode)
            );
        }
    }

    pub fn render_mode(&self) -> u8 {
        self.current_render_mode
    }

    pub fn clear_game_resources(&mut self) {
        self.buffer_manager.clear_game_meshes();
        self.texture_manager.clear_game_textures();
        self.command_buffer.reset();
        self.texture_bind_groups.clear(); // Clear cached bind groups!

        // Reset animation buffer metadata - prevents stale offsets from previous game
        // affecting the new game's animation data layout
        self.inverse_bind_end = 0;
        self.animation_static_end = 0;

        // Invalidate cached frame bind group since buffer contents changed
        self.invalidate_frame_bind_group();

        tracing::info!("Cleared game resources for new game");
    }

    // =================================================================
    // UNIFIED BUFFER: Static Upload Methods
    // =================================================================

    /// Upload all inverse bind matrices to the unified animation buffer
    ///
    /// Called once after init() when all skeletons have been loaded.
    /// Writes to section [0..I) of unified_animation buffer.
    /// Sets inverse_bind_end to track where inverse bind section ends.
    pub fn upload_static_inverse_bind(&mut self, all_matrices: &[crate::state::BoneMatrix3x4]) {
        let matrix_count = all_matrices.len();
        if matrix_count == 0 {
            self.inverse_bind_end = 0;
            return;
        }

        const BONE_MATRIX_SIZE: usize = 48; // 3×4 f32 = 12 floats × 4 bytes

        // Check if we need to grow the unified animation buffer
        // Required: inverse_bind + keyframes + immediate (256 max)
        let required_total = matrix_count + 8192 + 256; // generous estimate
        if required_total > self.unified_animation_capacity {
            self.unified_animation_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Unified Animation (@binding(3))"),
                size: (required_total * BONE_MATRIX_SIZE) as u64,
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            self.unified_animation_capacity = required_total;
            tracing::info!(
                "Resized unified animation buffer: {} matrices ({} bytes)",
                required_total,
                required_total * BONE_MATRIX_SIZE
            );
            // Invalidate cached bind group since buffer was recreated
            self.invalidate_frame_bind_group();
        }

        // Write inverse bind matrices at offset 0 (first section)
        let bytes = bone_matrices_to_bytes(all_matrices);
        self.queue
            .write_buffer(&self.unified_animation_buffer, 0, &bytes);

        // Track where inverse bind section ends (= keyframes section starts)
        self.inverse_bind_end = matrix_count;

        tracing::debug!(
            "Uploaded {} inverse bind matrices to unified_animation[0..{}]",
            matrix_count,
            matrix_count
        );
    }

    /// Upload all pre-decoded keyframe matrices to the unified animation buffer
    ///
    /// Called once after init() when all keyframes have been loaded and decoded.
    /// Writes to section [I..I+K) of unified_animation buffer.
    /// Sets animation_static_end to track where static data ends.
    pub fn upload_static_keyframes(&mut self, all_matrices: &[crate::state::BoneMatrix3x4]) {
        let matrix_count = all_matrices.len();
        if matrix_count == 0 {
            // Static end is just after inverse bind
            self.animation_static_end = self.inverse_bind_end;
            return;
        }

        const BONE_MATRIX_SIZE: usize = 48; // 3×4 f32 = 12 floats × 4 bytes
        let byte_offset = self.inverse_bind_end * BONE_MATRIX_SIZE;

        // Check if we need to grow the unified animation buffer
        let required_total = self.inverse_bind_end + matrix_count + 256; // +256 for immediate
        if required_total > self.unified_animation_capacity {
            // Need to recreate buffer and re-upload inverse bind matrices first
            // This is rare - only happens if we severely underestimated
            self.unified_animation_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Unified Animation (@binding(3))"),
                size: (required_total * BONE_MATRIX_SIZE) as u64,
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            self.unified_animation_capacity = required_total;
            tracing::warn!(
                "Had to resize unified animation buffer during keyframe upload: {} matrices",
                required_total
            );
            // Invalidate cached bind group since buffer was recreated
            self.invalidate_frame_bind_group();
            // NOTE: inverse bind data is lost! Caller must re-upload.
            // This is a rare edge case - normally inverse_bind is uploaded first
            // with enough buffer space.
        }

        // Write keyframe matrices after inverse bind section
        let bytes = bone_matrices_to_bytes(all_matrices);
        self.queue
            .write_buffer(&self.unified_animation_buffer, byte_offset as u64, &bytes);

        // Track where static data ends (= immediate section starts)
        self.animation_static_end = self.inverse_bind_end + matrix_count;

        tracing::debug!(
            "Uploaded {} keyframe matrices to unified_animation[{}..{}]",
            matrix_count,
            self.inverse_bind_end,
            self.animation_static_end
        );
    }

    /// Get the offset where immediate bone matrices should be written
    pub fn immediate_bone_offset(&self) -> usize {
        self.animation_static_end
    }
}

/// Convert a slice of BoneMatrix3x4 to bytes for GPU upload
///
/// BoneMatrix3x4 is #[repr(C)] with three [f32; 4] arrays (48 bytes total).
/// This is safe because the struct is fully POD-compatible.
fn bone_matrices_to_bytes(matrices: &[crate::state::BoneMatrix3x4]) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(matrices.len() * 48);
    for m in matrices {
        bytes.extend_from_slice(bytemuck::cast_slice(&m.row0));
        bytes.extend_from_slice(bytemuck::cast_slice(&m.row1));
        bytes.extend_from_slice(bytemuck::cast_slice(&m.row2));
    }
    bytes
}

impl Graphics for ZGraphics {
    fn resize(&mut self, width: u32, height: u32) {
        if width == 0 || height == 0 {
            return;
        }

        self.config.width = width;
        self.config.height = height;
        self.surface.configure(&self.device, &self.config);

        let (depth_texture, depth_view) = Self::create_depth_texture(&self.device, width, height);
        self.depth_texture = depth_texture;
        self.depth_view = depth_view;

        tracing::debug!("Resized graphics to {}x{}", width, height);
    }

    fn begin_frame(&mut self) {
        self.command_buffer.reset();

        let frame = match self.surface.get_current_texture() {
            Ok(frame) => frame,
            Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                self.surface.configure(&self.device, &self.config);
                match self.surface.get_current_texture() {
                    Ok(frame) => frame,
                    Err(e) => {
                        tracing::error!("Failed to acquire frame after reconfigure: {:?}", e);
                        return;
                    }
                }
            }
            Err(e) => {
                tracing::error!("Failed to acquire frame: {:?}", e);
                return;
            }
        };

        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        self.current_frame = Some(frame);
        self.current_view = Some(view);
    }

    fn end_frame(&mut self) {
        if let Some(frame) = self.current_frame.take() {
            frame.present();
        }
        self.current_view = None;
    }
}

impl nethercore_core::capture::CaptureSupport for ZGraphics {
    fn device(&self) -> &wgpu::Device {
        &self.device
    }

    fn queue(&self) -> &wgpu::Queue {
        &self.queue
    }

    fn render_target_texture(&self) -> &wgpu::Texture {
        &self.render_target.color_texture
    }

    fn render_target_dimensions(&self) -> (u32, u32) {
        (self.render_target.width, self.render_target.height)
    }
}

impl nethercore_core::app::StandaloneGraphicsSupport for ZGraphics {
    fn surface_format(&self) -> wgpu::TextureFormat {
        self.config.format
    }

    fn width(&self) -> u32 {
        self.config.width
    }

    fn height(&self) -> u32 {
        self.config.height
    }

    fn get_current_texture(&mut self) -> Result<wgpu::SurfaceTexture, wgpu::SurfaceError> {
        self.surface.get_current_texture()
    }

    fn blit_to_window(&self, encoder: &mut wgpu::CommandEncoder, view: &wgpu::TextureView) {
        // Delegate to existing method in frame.rs
        ZGraphics::blit_to_window(self, encoder, view)
    }

    fn set_scale_mode(&mut self, mode: nethercore_core::app::config::ScaleMode) {
        self.scale_mode = mode;
    }
}
