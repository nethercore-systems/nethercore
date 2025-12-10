//! Emberware Z graphics backend (wgpu)
//!
//! Implements the `Graphics` trait from emberware-core with a wgpu-based
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

use emberware_core::console::Graphics;

// Re-export packing utilities from z-common (for FFI and backwards compat)
pub use z_common::{
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
pub use render_state::{
    BlendMode, CullMode, MatcapBlendMode, RenderState, TextureFilter, TextureHandle,
};
pub use unified_shading_state::{
    LightType, PackedLight, PackedUnifiedShadingState, ShadingStateIndex, FLAG_SKINNING_MODE,
    pack_f16, pack_f16x2, pack_matcap_blend_modes, pack_rgb8, pack_unorm8, unpack_f16, unpack_f16x2,
    unpack_matcap_blend_modes, update_uniform_set_0_byte, update_uniform_set_1_byte,
};
pub use vertex::{FORMAT_ALL, VERTEX_FORMAT_COUNT, VertexFormatInfo};

// Re-export for crate-internal use
pub(crate) use init::RenderTarget;
pub(crate) use pipeline::PipelineEntry;

use pipeline::PipelineCache;
use texture_manager::TextureManager;

/// Emberware Z graphics backend
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
    blit_sampler: wgpu::Sampler,

    // Depth buffer (for window-sized UI rendering, no longer used for game content)
    depth_texture: wgpu::Texture,
    depth_view: wgpu::TextureView,

    // Texture management (extracted to separate module)
    texture_manager: TextureManager,

    // Samplers
    sampler_nearest: wgpu::Sampler,
    sampler_linear: wgpu::Sampler,

    // Current render state
    render_state: RenderState,

    // Bone system (GPU skinning)
    bone_buffer: wgpu::Buffer,
    inverse_bind_buffer: wgpu::Buffer,

    // Matrix storage buffers (per-frame arrays)
    model_matrix_buffer: wgpu::Buffer,
    view_matrix_buffer: wgpu::Buffer,
    proj_matrix_buffer: wgpu::Buffer,
    mvp_indices_buffer: wgpu::Buffer,
    model_matrix_capacity: usize,
    view_matrix_capacity: usize,
    proj_matrix_capacity: usize,
    mvp_indices_capacity: usize,

    // Shading state storage buffer (per-frame array)
    shading_state_buffer: wgpu::Buffer,
    shading_state_capacity: usize,

    // Bind group caches (cleared and repopulated each frame)
    texture_bind_groups: HashMap<[TextureHandle; 4], wgpu::BindGroup>,

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

    // Current render target resolution (for detecting changes)
    current_resolution_index: u8,

    // Scaling mode for render target to window
    scale_mode: emberware_core::app::config::ScaleMode,

    // Unit quad mesh for GPU-instanced rendering (billboards, sprites, etc.)
    unit_quad_format: u8,
    unit_quad_base_vertex: u32,
    unit_quad_first_index: u32,

    // Screen dimensions uniform for screen-space quads (part of bind group 0)
    screen_dims_buffer: wgpu::Buffer,
}

impl ZGraphics {
    /// Update render target resolution if changed
    pub fn update_resolution(&mut self, resolution_index: u8) {
        if resolution_index != self.current_resolution_index {
            self.recreate_render_target(resolution_index);
        }
    }

    /// Update scaling mode for render target to window
    pub fn set_scale_mode(&mut self, scale_mode: emberware_core::app::config::ScaleMode) {
        self.scale_mode = scale_mode;
    }

    fn recreate_render_target(&mut self, resolution_index: u8) {
        use crate::console::RESOLUTIONS;

        let (width, height) = RESOLUTIONS
            .get(resolution_index as usize)
            .copied()
            .unwrap_or((960, 540));

        tracing::info!(
            "Recreating render target: {}×{} (index {})",
            width,
            height,
            resolution_index
        );

        self.render_target =
            Self::create_render_target(&self.device, width, height, self.config.format);

        let screen_dims_data: [f32; 2] = [width as f32, height as f32];
        self.queue.write_buffer(
            &self.screen_dims_buffer,
            0,
            bytemuck::cast_slice(&screen_dims_data),
        );

        let bind_group_layout = self.blit_pipeline.get_bind_group_layout(0);
        self.blit_bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Blit Bind Group"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&self.render_target.color_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&self.blit_sampler),
                },
            ],
        });

        self.current_resolution_index = resolution_index;
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

    // Render State
    pub fn set_depth_test(&mut self, enabled: bool) {
        self.render_state.depth_test = enabled;
    }

    pub fn set_cull_mode(&mut self, mode: CullMode) {
        self.render_state.cull_mode = mode;
    }

    pub fn set_blend_mode(&mut self, mode: BlendMode) {
        self.render_state.blend_mode = mode;
    }

    pub fn set_texture_filter(&mut self, filter: TextureFilter) {
        self.render_state.texture_filter = filter;
    }

    pub fn render_state(&self) -> &RenderState {
        &self.render_state
    }

    pub fn current_sampler(&self) -> &wgpu::Sampler {
        match self.render_state.texture_filter {
            TextureFilter::Nearest => &self.sampler_nearest,
            TextureFilter::Linear => &self.sampler_linear,
        }
    }

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

    pub fn get_pipeline(&mut self, format: u8, state: &RenderState) -> &PipelineEntry {
        self.pipeline_cache.get_or_create(
            &self.device,
            self.config.format,
            self.current_render_mode,
            format,
            state,
        )
    }

    pub fn clear_game_resources(&mut self) {
        self.buffer_manager.clear_game_meshes();
        self.texture_manager.clear_game_textures();
        tracing::info!("Cleared game resources for new game");
    }
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
