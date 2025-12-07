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
//!
//! Note: Many public APIs here are designed for game rendering and are not yet
//! fully wired up. Dead code warnings are suppressed at module level.
//!
//! # Vertex Buffer Architecture
//!
//! Each vertex format gets its own buffer to avoid padding waste:
//! - FORMAT_UV (1): Has UV coordinates
//! - FORMAT_COLOR (2): Has per-vertex color (RGB, 3 floats)
//! - FORMAT_NORMAL (4): Has normals
//! - FORMAT_SKINNED (8): Has bone indices/weights
//!
//! All 16 combinations are supported (8 base + 8 skinned variants).
//!
//! # Command Buffer Pattern
//!
//! Immediate-mode draws are buffered on the CPU side and flushed once per frame
//! to minimize draw calls. Retained meshes are stored separately.
//!
//! # Resource Cleanup Strategy
//!
//! Graphics resources are automatically cleaned up when the owning structures are dropped:
//!
//! - **Textures** (`TextureEntry`): Stored in `ZGraphics::textures` HashMap. When the
//!   HashMap entry is removed or ZGraphics is dropped, wgpu::Texture/TextureView implement
//!   Drop and release GPU resources automatically.
//!
//! - **Retained Meshes** (`RetainedMesh`): Stored in `ZGraphics::retained_meshes` HashMap.
//!   Mesh data lives in shared vertex/index buffers per format, so individual mesh removal
//!   doesn't free GPU memory (buffer compaction not implemented). Full cleanup occurs when
//!   ZGraphics is dropped.
//!
//! - **Vertex/Index Buffers** (`GrowableBuffer`): One per vertex format (16 total).
//!   Automatically dropped when ZGraphics is dropped; wgpu::Buffer implements Drop.
//!
//! - **Pipelines**: Cached in `ZGraphics::pipelines` HashMap. Dropped when ZGraphics drops.
//!
//! - **Per-Frame Resources**: Command buffer resets each frame via `reset_command_buffer()`.
//!   No GPU allocations for immediate-mode draws between frames.
//!
//! **Game Lifecycle**: When a game exits (mode changes from Playing to Library), the game's
//! `GameInstance` is dropped, which clears pending textures/meshes in `GameState`. However,
//! ZGraphics remains alive across game switches to avoid expensive GPU reinitialization.
//! This means textures/meshes loaded by a previous game persist in GPU memory until
//! explicitly removed or the application exits. For the intended use case (single game per
//! session), this is acceptable. Future versions may add explicit resource invalidation on
//! game switch if memory pressure becomes a concern.

// Many public APIs are designed for game rendering but not yet fully wired up
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
use glam::Mat4;

use emberware_core::console::Graphics;

// Re-export public types from submodules
pub use buffer::{BufferManager, GrowableBuffer, MeshHandle, RetainedMesh};
pub use command_buffer::{VirtualRenderPass, VRPCommand};
pub use matrix_packing::MvpShadingIndices;
pub use quad_instance::{QuadInstance, QuadMode};
pub use render_state::{
    BlendMode, CullMode, MatcapBlendMode, RenderState, TextureFilter, TextureHandle,
};
pub use unified_shading_state::{
    pack_matcap_blend_modes, pack_octahedral_u32, pack_rgb8, pack_unorm8,
    unpack_matcap_blend_modes, PackedLight, PackedUnifiedShadingState, ShadingStateIndex,
};
pub use vertex::{vertex_stride, FORMAT_COLOR, FORMAT_NORMAL, FORMAT_SKINNED, FORMAT_UV};

// Re-export for crate-internal use
pub(crate) use init::RenderTarget;
pub(crate) use pipeline::PipelineEntry;

use pipeline::PipelineCache;
use texture_manager::TextureManager;

// MaterialCacheKey removed - obsolete with unified shading state system.
// Frame bind group is now identical for all draws (contains only buffers, no per-draw material data).

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
    ///
    /// Called by app layer when init_config.resolution_index changes
    pub fn update_resolution(&mut self, resolution_index: u8) {
        if resolution_index != self.current_resolution_index {
            self.recreate_render_target(resolution_index);
        }
    }

    /// Update scaling mode for render target to window
    ///
    /// Called by app layer when config.video.scale_mode changes
    pub fn set_scale_mode(&mut self, scale_mode: emberware_core::app::config::ScaleMode) {
        self.scale_mode = scale_mode;
    }

    /// Recreate render target at new resolution
    ///
    /// Called when game changes resolution via init_config.resolution_index
    fn recreate_render_target(&mut self, resolution_index: u8) {
        use crate::console::RESOLUTIONS;

        let (width, height) = RESOLUTIONS
            .get(resolution_index as usize)
            .copied()
            .unwrap_or((960, 540)); // Fallback to default

        tracing::info!(
            "Recreating render target: {}×{} (index {})",
            width,
            height,
            resolution_index
        );

        // Create new render target
        self.render_target =
            Self::create_render_target(&self.device, width, height, self.config.format);

        // Update screen dimensions uniform buffer
        let screen_dims_data: [f32; 2] = [width as f32, height as f32];
        self.queue.write_buffer(
            &self.screen_dims_buffer,
            0,
            bytemuck::cast_slice(&screen_dims_data),
        );

        // Recreate blit bind group with new render target texture
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

    // ========================================================================
    // Texture Management (delegated to TextureManager)
    // ========================================================================

    /// Load a texture from RGBA8 pixel data
    ///
    /// Returns a TextureHandle or an error if VRAM budget is exceeded.
    pub fn load_texture(
        &mut self,
        width: u32,
        height: u32,
        pixels: &[u8],
    ) -> Result<TextureHandle> {
        self.texture_manager
            .load_texture(&self.device, &self.queue, width, height, pixels)
    }

    /// Get texture view by handle
    pub fn get_texture_view(&self, handle: TextureHandle) -> Option<&wgpu::TextureView> {
        self.texture_manager.get_texture_view(handle)
    }

    /// Get fallback checkerboard texture view
    pub fn get_fallback_checkerboard_view(&self) -> &wgpu::TextureView {
        self.texture_manager.get_fallback_checkerboard_view()
    }

    /// Get fallback white texture view
    pub fn get_fallback_white_view(&self) -> &wgpu::TextureView {
        self.texture_manager.get_fallback_white_view()
    }

    /// Get font texture handle
    pub fn font_texture(&self) -> TextureHandle {
        self.texture_manager.font_texture()
    }

    /// Get white fallback texture handle
    pub fn white_texture(&self) -> TextureHandle {
        self.texture_manager.white_texture()
    }

    /// Get font texture view
    pub fn get_font_texture_view(&self) -> &wgpu::TextureView {
        self.texture_manager.get_font_texture_view()
    }

    /// Get texture view for a handle, returning fallback if invalid
    /// Note: Texture binding is now managed in ZFFIState.bound_textures
    pub fn get_texture_view_or_fallback(&self, handle: TextureHandle) -> &wgpu::TextureView {
        if handle == TextureHandle::INVALID {
            self.get_fallback_white_view()
        } else {
            self.get_texture_view(handle)
                .unwrap_or_else(|| self.get_fallback_checkerboard_view())
        }
    }

    /// Get VRAM usage in bytes
    pub fn vram_used(&self) -> usize {
        self.texture_manager.vram_used()
    }

    /// Get VRAM limit in bytes
    pub fn vram_limit(&self) -> usize {
        self.texture_manager.vram_limit()
    }

    // ========================================================================
    // Render State
    // ========================================================================
    // Note: Color and texture binding are now managed in ZFFIState
    // These methods have been removed as they're redundant with FFI layer state management

    /// Enable or disable depth testing
    pub fn set_depth_test(&mut self, enabled: bool) {
        self.render_state.depth_test = enabled;
    }

    /// Set face culling mode
    pub fn set_cull_mode(&mut self, mode: CullMode) {
        self.render_state.cull_mode = mode;
    }

    /// Set blend mode
    pub fn set_blend_mode(&mut self, mode: BlendMode) {
        self.render_state.blend_mode = mode;
    }

    /// Set texture filter mode
    pub fn set_texture_filter(&mut self, filter: TextureFilter) {
        self.render_state.texture_filter = filter;
    }

    /// Get current render state
    pub fn render_state(&self) -> &RenderState {
        &self.render_state
    }

    /// Get current sampler based on texture filter setting
    pub fn current_sampler(&self) -> &wgpu::Sampler {
        match self.render_state.texture_filter {
            TextureFilter::Nearest => &self.sampler_nearest,
            TextureFilter::Linear => &self.sampler_linear,
        }
    }

    // ========================================================================
    // Retained Mesh Loading (delegated to BufferManager)
    // ========================================================================

    /// Load a non-indexed mesh (retained mode)
    ///
    /// The mesh is stored in the appropriate vertex buffer based on format.
    /// Returns a MeshHandle for use with draw_mesh().
    pub fn load_mesh(&mut self, data: &[f32], format: u8) -> Result<MeshHandle> {
        self.buffer_manager
            .load_mesh(&self.device, &self.queue, data, format)
    }

    /// Load an indexed mesh (retained mode)
    ///
    /// The mesh is stored in the appropriate vertex and index buffers based on format.
    /// Returns a MeshHandle for use with draw_mesh().
    pub fn load_mesh_indexed(
        &mut self,
        data: &[f32],
        indices: &[u16],
        format: u8,
    ) -> Result<MeshHandle> {
        self.buffer_manager
            .load_mesh_indexed(&self.device, &self.queue, data, indices, format)
    }

    /// Get mesh info by handle
    pub fn get_mesh(&self, handle: MeshHandle) -> Option<&RetainedMesh> {
        self.buffer_manager.get_mesh(handle)
    }

    // ========================================================================
    // Command Buffer Access
    // ========================================================================

    /// Get the command buffer (for flush/rendering)
    pub fn command_buffer(&self) -> &VirtualRenderPass {
        &self.command_buffer
    }

    /// Get mutable command buffer
    pub fn command_buffer_mut(&mut self) -> &mut VirtualRenderPass {
        &mut self.command_buffer
    }

    /// Reset the command buffer for the next frame
    ///
    /// Called automatically at the start of begin_frame, but can be called
    /// manually if needed.
    pub fn reset_command_buffer(&mut self) {
        self.command_buffer.reset();
    }

    // ========================================================================
    // Buffer Access (delegated to BufferManager)
    // ========================================================================

    /// Get vertex buffer for a format
    pub fn vertex_buffer(&self, format: u8) -> &GrowableBuffer {
        self.buffer_manager.vertex_buffer(format)
    }

    /// Get index buffer for a format
    pub fn index_buffer(&self, format: u8) -> &GrowableBuffer {
        self.buffer_manager.index_buffer(format)
    }

    // ========================================================================
    // Device Access
    // ========================================================================

    /// Get wgpu device reference
    pub fn device(&self) -> &wgpu::Device {
        &self.device
    }

    /// Get wgpu queue reference
    pub fn queue(&self) -> &wgpu::Queue {
        &self.queue
    }

    /// Get surface format
    pub fn surface_format(&self) -> wgpu::TextureFormat {
        self.config.format
    }

    /// Get depth format
    pub fn depth_format(&self) -> wgpu::TextureFormat {
        wgpu::TextureFormat::Depth32Float
    }

    /// Get depth texture view
    pub fn depth_view(&self) -> &wgpu::TextureView {
        &self.depth_view
    }

    /// Get current surface dimensions
    pub fn dimensions(&self) -> (u32, u32) {
        (self.config.width, self.config.height)
    }

    /// Get current surface width
    pub fn width(&self) -> u32 {
        self.config.width
    }

    /// Get current surface height
    pub fn height(&self) -> u32 {
        self.config.height
    }

    /// Get the current surface texture for rendering
    pub fn get_current_texture(&mut self) -> Result<wgpu::SurfaceTexture> {
        self.surface
            .get_current_texture()
            .context("Failed to acquire next surface texture")
    }

    // ========================================================================
    // Shader Compilation and Pipeline Management
    // ========================================================================

    /// Set the current render mode (0-3)
    ///
    /// This determines which shader templates are used for rendering.
    /// Must be called in init() before any rendering.
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

    /// Get the current render mode
    pub fn render_mode(&self) -> u8 {
        self.current_render_mode
    }

    /// Get or create a pipeline for the given state
    ///
    /// This caches pipelines to avoid recompilation.
    pub fn get_pipeline(&mut self, format: u8, state: &RenderState) -> &PipelineEntry {
        self.pipeline_cache.get_or_create(
            &self.device,
            self.config.format,
            self.current_render_mode,
            format,
            state,
        )
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

        // Recreate depth buffer
        let (depth_texture, depth_view) = Self::create_depth_texture(&self.device, width, height);
        self.depth_texture = depth_texture;
        self.depth_view = depth_view;

        tracing::debug!("Resized graphics to {}x{}", width, height);
    }

    fn begin_frame(&mut self) {
        // Reset command buffer for new frame
        self.command_buffer.reset();

        // Acquire next frame
        let frame = match self.surface.get_current_texture() {
            Ok(frame) => frame,
            Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                // Reconfigure surface and try again
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
        // Present frame
        if let Some(frame) = self.current_frame.take() {
            frame.present();
        }
        self.current_view = None;
    }

    fn set_bones(&mut self, bones: &[Mat4]) {
        if bones.is_empty() {
            return;
        }

        // Clamp to maximum 256 bones
        let bone_count = bones.len().min(256);

        // Convert Mat4 matrices to flat f32 array for GPU upload
        // Each Mat4 is 16 floats (64 bytes)
        let mut bone_data: Vec<f32> = Vec::with_capacity(bone_count * 16);
        for matrix in &bones[..bone_count] {
            bone_data.extend_from_slice(&matrix.to_cols_array());
        }

        // Upload to GPU bone storage buffer
        self.queue
            .write_buffer(&self.bone_buffer, 0, bytemuck::cast_slice(&bone_data));

        tracing::trace!("Uploaded {} bone matrices to GPU", bone_count);
    }
}

// Tests removed - generate_text_quads was replaced by GPU-instanced quad rendering system
