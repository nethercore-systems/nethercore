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
#![allow(dead_code)]

mod buffer;
mod command_buffer;
mod matrix_packing;
mod pipeline;
mod quad_instance;
mod render_state;
mod texture_manager;
mod unified_shading_state;
mod vertex;

use hashbrown::HashMap;
use std::sync::Arc;

use anyhow::{Context, Result};
use glam::Mat4;
use winit::window::Window;

use emberware_core::console::Graphics;

// Re-export public types from submodules
pub use buffer::{BufferManager, GrowableBuffer, MeshHandle, RetainedMesh};
pub use command_buffer::{BufferSource, VirtualRenderPass};
pub use matrix_packing::{MvpIndex, MvpShadingIndices};
pub use quad_instance::{QuadInstance, QuadMode};
pub use render_state::{
    BlendMode, CullMode, MatcapBlendMode, RenderState, TextureFilter, TextureHandle,
};
pub use unified_shading_state::{
    pack_matcap_blend_modes, pack_octahedral_u32, pack_rgb8, pack_unorm8,
    unpack_matcap_blend_modes, PackedLight, PackedUnifiedShadingState, ShadingStateIndex,
};
pub use vertex::{
    vertex_stride, FORMAT_COLOR, FORMAT_NORMAL, FORMAT_SKINNED, FORMAT_UV, VERTEX_FORMAT_COUNT,
};

// Re-export for crate-internal use
pub(crate) use pipeline::PipelineEntry;

use pipeline::{PipelineCache, PipelineKey};
use texture_manager::TextureManager;

/// Convert pixel coordinates to NDC (Normalized Device Coordinates)
/// NDC range is -1 to 1 for both X and Y
#[inline]
fn pixel_to_ndc(pixel_x: f32, pixel_y: f32, width: f32, height: f32) -> (f32, f32) {
    let ndc_x = (pixel_x / (width * 0.5)) - 1.0;
    let ndc_y = 1.0 - (pixel_y / (height * 0.5));
    (ndc_x, ndc_y)
}

// MaterialCacheKey removed - obsolete with unified shading state system.
// Frame bind group is now identical for all draws (contains only buffers, no per-draw material data).

/// Emberware Z graphics backend
///
/// Manages wgpu device, textures, render state, and frame presentation.
/// Implements the vertex buffer architecture with one buffer per stride
/// and command buffer pattern for draw batching.
/// Offscreen render target for fixed internal resolution
struct RenderTarget {
    color_texture: wgpu::Texture,
    color_view: wgpu::TextureView,
    depth_texture: wgpu::Texture,
    depth_view: wgpu::TextureView,
    width: u32,
    height: u32,
}

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
    scale_mode: crate::config::ScaleMode,

    // Unit quad mesh for GPU-instanced rendering (billboards, sprites, etc.)
    unit_quad_format: u8,
    unit_quad_base_vertex: u32,
    unit_quad_first_index: u32,
    unit_quad_index_count: u32,
}

impl ZGraphics {
    /// Create a new ZGraphics instance
    ///
    /// This initializes wgpu with the given window and sets up all core resources.
    pub async fn new(window: Arc<Window>) -> Result<Self> {
        let size = window.inner_size();
        let width = size.width.max(1);
        let height = size.height.max(1);

        // Create wgpu instance
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        // Create surface
        let surface = instance
            .create_surface(window)
            .context("Failed to create surface")?;

        // Request adapter
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .context("Failed to find suitable GPU adapter")?;

        tracing::info!("Using GPU adapter: {:?}", adapter.get_info().name);

        // Request device and queue
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("Emberware Z Device"),
                    required_features: wgpu::Features::default(),
                    required_limits: wgpu::Limits::default(),
                    memory_hints: wgpu::MemoryHints::Performance,
                },
                None,
            )
            .await
            .context("Failed to create GPU device")?;

        // Configure surface
        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width,
            height,
            present_mode: wgpu::PresentMode::AutoVsync,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 1,
        };
        surface.configure(&device, &config);

        // Create depth buffer
        let (depth_texture, depth_view) = Self::create_depth_texture(&device, width, height);

        // Create samplers
        let sampler_nearest = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Sampler Nearest"),
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            address_mode_w: wgpu::AddressMode::Repeat,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let sampler_linear = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Sampler Linear"),
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            address_mode_w: wgpu::AddressMode::Repeat,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        // Create buffer manager (vertex/index buffers and mesh storage)
        let mut buffer_manager = BufferManager::new(&device);

        // Create bone storage buffer for GPU skinning (256 bones × 64 bytes = 16KB)
        let bone_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Bone Storage Buffer"),
            size: 256 * 64, // 256 matrices × 64 bytes per matrix
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // Create matrix storage buffers (per-frame arrays)
        let model_matrix_capacity = 1024;
        let model_matrix_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Model Matrices"),
            size: (model_matrix_capacity * std::mem::size_of::<Mat4>()) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let view_matrix_capacity = 16;
        let view_matrix_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("View Matrices"),
            size: (view_matrix_capacity * std::mem::size_of::<Mat4>()) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let proj_matrix_capacity = 16;
        let proj_matrix_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Projection Matrices"),
            size: (proj_matrix_capacity * std::mem::size_of::<Mat4>()) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // Create MVP indices buffer (2 × u32 per entry: packed MVP + shading_state_index)
        let mvp_indices_capacity = 1024;
        let mvp_indices_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("MVP Indices"),
            size: (mvp_indices_capacity * 2 * std::mem::size_of::<u32>()) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // Create shading state buffer (per-frame array of PackedUnifiedShadingState)
        let shading_state_capacity = 256;
        let shading_state_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Shading States"),
            size: (shading_state_capacity * std::mem::size_of::<PackedUnifiedShadingState>())
                as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // Create texture manager (handles fallback textures)
        let texture_manager = TextureManager::new(&device, &queue)?;

        // Create offscreen render target at default resolution (960×540)
        let render_target = Self::create_render_target(&device, 960, 540, surface_format);

        // Create blit pipeline for scaling render target to window
        let (blit_pipeline, blit_bind_group, blit_sampler) =
            Self::create_blit_pipeline(&device, surface_format, &render_target);

        // Create static unit quad mesh for GPU-instanced rendering
        // Format: POS_UV_COLOR (format bits: UV | COLOR = 0b011 = 3)
        use crate::graphics::vertex::{FORMAT_UV, FORMAT_COLOR, vertex_stride};
        let unit_quad_format = FORMAT_UV | FORMAT_COLOR;

        let unit_quad_vertices: Vec<f32> = vec![
            // pos_x, pos_y, pos_z, uv_u, uv_v, color_r, color_g, color_b
            -0.5, -0.5, 0.0,  0.0, 0.0,  1.0, 1.0, 1.0,  // Bottom-left
             0.5, -0.5, 0.0,  1.0, 0.0,  1.0, 1.0, 1.0,  // Bottom-right
             0.5,  0.5, 0.0,  1.0, 1.0,  1.0, 1.0, 1.0,  // Top-right
            -0.5,  0.5, 0.0,  0.0, 1.0,  1.0, 1.0, 1.0,  // Top-left
        ];

        let unit_quad_indices: Vec<u16> = vec![
            0, 1, 2,  // First triangle
            0, 2, 3,  // Second triangle
        ];

        // Upload unit quad to retained vertex buffer
        let vertex_bytes = bytemuck::cast_slice(&unit_quad_vertices);
        let stride = vertex_stride(unit_quad_format);

        // Get the current position in the vertex buffer (this will be our base_vertex)
        let unit_quad_base_vertex = {
            let retained_vertex_buf = buffer_manager.retained_vertex_buffer(unit_quad_format);
            (retained_vertex_buf.used() / stride as u64) as u32
        };

        // Write vertex data to buffer
        let retained_vertex_buf_mut = buffer_manager.retained_vertex_buffer_mut(unit_quad_format);
        retained_vertex_buf_mut.ensure_capacity(&device, vertex_bytes.len() as u64);
        retained_vertex_buf_mut.write(&queue, vertex_bytes);

        // Upload unit quad to retained index buffer
        let index_bytes = bytemuck::cast_slice(&unit_quad_indices);

        // Get the current position in the index buffer (this will be our first_index)
        let unit_quad_first_index = {
            let retained_index_buf = buffer_manager.retained_index_buffer(unit_quad_format);
            (retained_index_buf.used() / 2) as u32 // u16 = 2 bytes
        };

        // Write index data to buffer
        let retained_index_buf_mut = buffer_manager.retained_index_buffer_mut(unit_quad_format);
        retained_index_buf_mut.ensure_capacity(&device, index_bytes.len() as u64);
        retained_index_buf_mut.write(&queue, index_bytes);

        let unit_quad_index_count = 6;

        let graphics = Self {
            surface,
            device,
            queue,
            config,
            render_target,
            blit_pipeline,
            blit_bind_group,
            blit_sampler,
            depth_texture,
            depth_view,
            texture_manager,
            sampler_nearest,
            sampler_linear,
            render_state: RenderState::default(),
            bone_buffer,
            model_matrix_buffer,
            view_matrix_buffer,
            proj_matrix_buffer,
            mvp_indices_buffer,
            model_matrix_capacity,
            view_matrix_capacity,
            proj_matrix_capacity,
            mvp_indices_capacity,
            shading_state_buffer,
            shading_state_capacity,
            texture_bind_groups: HashMap::new(),
            current_frame: None,
            current_view: None,
            buffer_manager,
            command_buffer: VirtualRenderPass::new(),
            pipeline_cache: PipelineCache::new(),
            current_render_mode: 0,      // Default to Mode 0 (Unlit)
            current_resolution_index: 1, // 960×540 (default)
            scale_mode: crate::config::ScaleMode::default(), // Stretch by default
            unit_quad_format,
            unit_quad_base_vertex,
            unit_quad_first_index,
            unit_quad_index_count,
        };

        Ok(graphics)
    }

    /// Create a new ZGraphics instance (blocking version for sync contexts)
    pub fn new_blocking(window: Arc<Window>) -> Result<Self> {
        pollster::block_on(Self::new(window))
    }

    /// Create depth texture and view
    fn create_depth_texture(
        device: &wgpu::Device,
        width: u32,
        height: u32,
    ) -> (wgpu::Texture, wgpu::TextureView) {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Depth Texture"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        (texture, view)
    }

    /// Create offscreen render target at specified resolution
    fn create_render_target(
        device: &wgpu::Device,
        width: u32,
        height: u32,
        surface_format: wgpu::TextureFormat,
    ) -> RenderTarget {
        // Create color texture (render target)
        let color_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Render Target Color"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: surface_format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let color_view = color_texture.create_view(&wgpu::TextureViewDescriptor::default());

        // Create depth texture for render target
        let depth_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Render Target Depth"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });
        let depth_view = depth_texture.create_view(&wgpu::TextureViewDescriptor::default());

        RenderTarget {
            color_texture,
            color_view,
            depth_texture,
            depth_view,
            width,
            height,
        }
    }

    /// Create blit pipeline and resources for scaling render target to window
    fn create_blit_pipeline(
        device: &wgpu::Device,
        surface_format: wgpu::TextureFormat,
        render_target: &RenderTarget,
    ) -> (wgpu::RenderPipeline, wgpu::BindGroup, wgpu::Sampler) {
        // Create sampler for render target (nearest neighbor for pixel-perfect scaling)
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Blit Sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        // Load blit shader
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Blit Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../../shaders/blit.wgsl").into()),
        });

        // Create bind group layout
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Blit Bind Group Layout"),
            entries: &[
                // Render target texture
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                // Sampler
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

        // Create bind group
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Blit Bind Group"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&render_target.color_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
        });

        // Create pipeline layout
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Blit Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        // Create render pipeline
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Blit Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_format,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
            cache: None,
        });

        (pipeline, bind_group, sampler)
    }

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
    pub fn set_scale_mode(&mut self, scale_mode: crate::config::ScaleMode) {
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
    // Draw Command Processing
    // ========================================================================

    /// Process all draw commands from ZFFIState and execute them
    ///
    /// This method consumes draw commands from the ZFFIState and executes them
    /// on the GPU, directly translating FFI state into graphics calls without
    /// an intermediate unpacking/repacking step.
    ///
    /// This replaces the previous execute_draw_commands() function in app.rs,
    /// eliminating redundant data translation and simplifying the architecture.
    /// Process all draw commands from ZFFIState and execute them
    ///
    /// This method consumes draw commands from the ZFFIState and executes them
    /// on the GPU, directly translating FFI state into graphics calls without
    /// an intermediate unpacking/repacking step.
    ///
    /// This replaces the previous execute_draw_commands() function in app.rs,
    /// eliminating redundant data translation and simplifying the architecture.
    pub fn process_draw_commands(
        &mut self,
        z_state: &mut crate::state::ZFFIState,
        texture_map: &hashbrown::HashMap<u32, TextureHandle>,
    ) {
        use crate::console::RESOLUTIONS;
        use crate::state::DeferredCommand;

        // Apply init config to graphics (render mode, etc.)
        self.set_render_mode(z_state.init_config.render_mode);

        // Get the game's internal render resolution
        let res_idx = z_state.init_config.resolution_index as usize;
        let (render_width, render_height) = RESOLUTIONS.get(res_idx).copied().unwrap_or((960, 540)); // Default to 540p if invalid
        let render_width_f = render_width as f32;
        let render_height_f = render_height as f32;

        // 1. Swap the FFI-populated render pass into our command buffer
        // This efficiently transfers all immediate geometry (triangles, meshes)
        // without copying vectors. The old command buffer (now in z_state.render_pass)
        // will be cleared when z_state.clear_frame() is called.
        std::mem::swap(&mut self.command_buffer, &mut z_state.render_pass);

        // 1.1. Remap texture handles from FFI handles to graphics handles
        // FFI functions (draw_triangles, draw_mesh) store INVALID placeholders because they
        // don't have access to session.texture_map. Now we remap them using bound_textures.
        for cmd in self.command_buffer.commands_mut() {
            // Get mutable reference to texture_slots based on variant
            let texture_slots = match cmd {
                command_buffer::VRPCommand::Mesh { texture_slots, .. } => texture_slots,
                command_buffer::VRPCommand::IndexedMesh { texture_slots, .. } => texture_slots,
                command_buffer::VRPCommand::Quad { texture_slots, .. } => texture_slots,
            };

            if texture_slots[0] == TextureHandle::INVALID {
                *texture_slots = [
                    texture_map
                        .get(&z_state.bound_textures[0])
                        .copied()
                        .unwrap_or(TextureHandle::INVALID),
                    texture_map
                        .get(&z_state.bound_textures[1])
                        .copied()
                        .unwrap_or(TextureHandle::INVALID),
                    texture_map
                        .get(&z_state.bound_textures[2])
                        .copied()
                        .unwrap_or(TextureHandle::INVALID),
                    texture_map
                        .get(&z_state.bound_textures[3])
                        .copied()
                        .unwrap_or(TextureHandle::INVALID),
                ];
            }
        }

        // Ensure default shading state exists for deferred commands.
        // Deferred commands (billboards, sprites, text) use ShadingStateIndex(0) by default.
        // If the game only uses deferred drawing and never calls draw_mesh/draw_triangles,
        // the shading state pool would be empty (cleared by clear_frame), causing panics
        // during command sorting/rendering when accessing state 0.
        //
        // This ensures state 0 always exists, using the current render state defaults
        // (color, blend mode, material properties, etc.) from z_state.current_shading_state.
        if z_state.shading_states.is_empty() {
            z_state.add_shading_state();
        }

        // IMPORTANT: Ensure mvp_shading_states has at least one entry (index 0)
        // This is needed for text rendering and other deferred commands that use placeholder buffer_index 0.
        // Without this, sorting/rendering would panic when trying to access mvp_shading_states[0].
        if z_state.mvp_shading_states.is_empty() {
            z_state.add_mvp_shading_state();
        }

        // 1.5. Process GPU-instanced quads (billboards, sprites)
        // Upload quad instances to GPU and create instanced draw command
        if !z_state.quad_instances.is_empty() {
            tracing::info!("Processing {} quad instances", z_state.quad_instances.len());

            // DEBUG: Log first few instances to verify data
            for (i, inst) in z_state.quad_instances.iter().take(3).enumerate() {
                tracing::info!("  Instance[{}]: pos=({},{},{}), size=({},{}), mode={}, view_idx={}",
                    i, inst.position[0], inst.position[1], inst.position[2],
                    inst.size[0], inst.size[1], inst.mode, inst.view_index);
            }

            // DEBUG: Log view/projection matrices
            if !z_state.view_matrices.is_empty() && !z_state.proj_matrices.is_empty() {
                tracing::info!("  View matrix count: {}, Proj matrix count: {}",
                    z_state.view_matrices.len(), z_state.proj_matrices.len());
                tracing::info!("  View[0]: {:?}", z_state.view_matrices[0]);
                tracing::info!("  Proj[0]: {:?}", z_state.proj_matrices[0]);
            }

            // Upload instance data to GPU
            self.buffer_manager
                .upload_quad_instances(&self.device, &self.queue, &z_state.quad_instances)
                .expect("Failed to upload quad instances to GPU");

            // Create instanced draw command using the unit quad mesh
            // The GPU vertex shader will expand each instance into a quad
            let instance_count = z_state.quad_instances.len() as u32;

            tracing::info!(
                "Quad texture state: bound_textures={:?}, texture_map has {} entries",
                z_state.bound_textures,
                texture_map.len()
            );

            // Map FFI texture handles to graphics texture handles
            let texture_slots = [
                texture_map
                    .get(&z_state.bound_textures[0])
                    .copied()
                    .unwrap_or(TextureHandle::INVALID),
                texture_map
                    .get(&z_state.bound_textures[1])
                    .copied()
                    .unwrap_or(TextureHandle::INVALID),
                texture_map
                    .get(&z_state.bound_textures[2])
                    .copied()
                    .unwrap_or(TextureHandle::INVALID),
                texture_map
                    .get(&z_state.bound_textures[3])
                    .copied()
                    .unwrap_or(TextureHandle::INVALID),
            ];

            // Note: Quad instances contain their own shading_state_index in the instance data.
            // BufferSource::Quad has no buffer_index - quads read transforms and shading from instance data.
            self.command_buffer.add_command(command_buffer::VRPCommand::Quad {
                base_vertex: self.unit_quad_base_vertex,
                first_index: self.unit_quad_first_index,
                instance_count,
                texture_slots,
                depth_test: true, // Billboards typically use depth test (TODO: per-instance?)
                cull_mode: CullMode::None, // Quads are double-sided
            });
        }

        // 2. Process deferred commands (text only - billboards/sprites/rects now use QuadInstance)
        for cmd in z_state.deferred_commands.drain(..) {
            match cmd {
                DeferredCommand::DrawText {
                    text,
                    x,
                    y,
                    size,
                    color,
                    font,
                } => {
                    // Render text using built-in or custom font
                    let text_str = std::str::from_utf8(&text).unwrap_or("");

                    // Skip empty text
                    if text_str.is_empty() {
                        continue;
                    }

                    // Look up custom font if font handle != 0
                    let font_opt = if font == 0 {
                        None
                    } else {
                        let font_index = (font - 1) as usize;
                        z_state.fonts.get(font_index)
                    };

                    // Generate text quads (POS_UV_COLOR format = 3)
                    let (vertices, indices) = Self::generate_text_quads(
                        text_str,
                        x,
                        y,
                        size,
                        color,
                        font_opt,
                        render_width_f,
                        render_height_f,
                    );

                    // Skip if no vertices generated
                    if vertices.is_empty() || indices.is_empty() {
                        continue;
                    }

                    // Determine which texture to use
                    let font_texture = if let Some(custom_font) = font_opt {
                        // Use custom font's texture
                        if let Some(&graphics_handle) = texture_map.get(&custom_font.texture) {
                            graphics_handle
                        } else {
                            tracing::warn!(
                                "Custom font texture handle {} not found, using built-in font",
                                custom_font.texture
                            );
                            self.font_texture()
                        }
                    } else {
                        // Use built-in font texture
                        self.font_texture()
                    };

                    // Text uses POS_UV_COLOR format (format 3)
                    const TEXT_FORMAT: u8 = 3; // FORMAT_UV | FORMAT_COLOR

                    // Append vertex and index data
                    let base_vertex = self
                        .command_buffer
                        .append_vertex_data(TEXT_FORMAT, &vertices);
                    let first_index = self.command_buffer.append_index_data(TEXT_FORMAT, &indices);

                    // Create texture slots with font texture in slot 0
                    let mut texture_slots = [TextureHandle::INVALID; 4];
                    texture_slots[0] = font_texture;

                    // Add draw command for text rendering
                    // Text is always rendered in 2D screen space with identity transform
                    // Uses buffer_index 0 which is guaranteed to exist (safety check in render_frame)
                    // This works because we ensure mvp_shading_states has at least one entry before rendering
                    self.command_buffer.add_command(command_buffer::VRPCommand::IndexedMesh {
                        format: TEXT_FORMAT,
                        index_count: indices.len() as u32,
                        base_vertex,
                        first_index,
                        buffer_index: 0,
                        texture_slots,
                        depth_test: false, // 2D text doesn't use depth test
                        cull_mode: CullMode::None,
                    });
                }
            }
        }

        // Note: All per-frame cleanup (model_matrices, audio_commands, render_pass, deferred_commands)
        // happens AFTER render_frame completes in app.rs via z_state.clear_frame()
        // This keeps cleanup centralized and ensures matrices survive until GPU upload
    }

    /// Convert game matcap blend mode to graphics matcap blend mode
    fn convert_matcap_blend_mode(mode: u8) -> MatcapBlendMode {
        match mode {
            0 => MatcapBlendMode::Multiply,
            1 => MatcapBlendMode::Add,
            2 => MatcapBlendMode::HsvModulate,
            _ => MatcapBlendMode::Multiply,
        }
    }

    /// Map game texture handles to graphics texture handles
    fn map_texture_handles(
        texture_map: &hashbrown::HashMap<u32, TextureHandle>,
        bound_textures: &[u32; 4],
    ) -> [TextureHandle; 4] {
        let mut texture_slots = [TextureHandle::INVALID; 4];
        for (slot, &game_handle) in bound_textures.iter().enumerate() {
            if game_handle != 0 {
                if let Some(&graphics_handle) = texture_map.get(&game_handle) {
                    texture_slots[slot] = graphics_handle;
                }
            }
        }
        texture_slots
    }

    /// Convert game cull mode to graphics cull mode
    fn convert_cull_mode(mode: u8) -> CullMode {
        match mode {
            0 => CullMode::None,
            1 => CullMode::Back,
            2 => CullMode::Front,
            _ => CullMode::None,
        }
    }

    /// Convert game blend mode to graphics blend mode
    fn convert_blend_mode(mode: u8) -> BlendMode {
        match mode {
            0 => BlendMode::None,
            1 => BlendMode::Alpha,
            2 => BlendMode::Additive,
            3 => BlendMode::Multiply,
            _ => BlendMode::None,
        }
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

    // ========================================================================
    // Text Rendering
    // ========================================================================

    /// Generate vertex data for rendering text
    ///
    /// This method generates quads for each character in the text string.
    /// The text is rendered in screen space (2D), left-aligned.
    ///
    /// # Arguments
    /// * `text` - UTF-8 text string to render
    /// * `x` - Screen X coordinate in pixels (0 = left edge)
    /// * `y` - Screen Y coordinate in pixels (0 = top edge)
    /// * `size` - Font size in pixels (base size is 8x8, this scales it)
    /// * `color` - Text color as 0xRRGGBBAA
    ///
    /// # Returns
    /// A tuple of (vertices, indices) for POS_UV_COLOR format (format 3)
    ///
    /// # Notes
    /// - Characters outside ASCII 32-126 are rendered as spaces
    /// - This is a simple left-to-right, single-line renderer (no word wrap)
    pub fn generate_text_quads(
        text: &str,
        x: f32,
        y: f32,
        size: f32,
        color: u32,
        font_opt: Option<&crate::state::Font>,
        render_width: f32,
        render_height: f32,
    ) -> (Vec<f32>, Vec<u16>) {
        use crate::font;

        // Extract color components (0xRRGGBBAA)
        let r = ((color >> 24) & 0xFF) as f32 / 255.0;
        let g = ((color >> 16) & 0xFF) as f32 / 255.0;
        let b = ((color >> 8) & 0xFF) as f32 / 255.0;

        let char_count = text.chars().count();
        let mut vertices = Vec::with_capacity(char_count * 4 * 8); // 4 verts × 8 floats
        let mut indices: Vec<u16> = Vec::with_capacity(char_count * 6); // 6 indices per quad

        let mut cursor_x = x;
        let mut vertex_index = 0u16;

        if let Some(custom_font) = font_opt {
            // Custom font rendering
            let scale = size / custom_font.char_height as f32;
            let glyph_height = custom_font.char_height as f32 * scale;

            // Calculate texture atlas grid dimensions
            // Glyphs are arranged left-to-right, top-to-bottom in a 16-column grid
            let glyphs_per_row = 16;
            let max_glyph_width = custom_font
                .char_widths
                .as_ref()
                .map(|widths| *widths.iter().max().unwrap_or(&custom_font.char_width))
                .unwrap_or(custom_font.char_width);

            let atlas_width = glyphs_per_row * max_glyph_width as usize;
            let atlas_height = (custom_font.char_count as usize).div_ceil(glyphs_per_row)
                * custom_font.char_height as usize;

            for ch in text.chars() {
                let char_code = ch as u32;

                // Map character to glyph index
                let glyph_index = if char_code >= custom_font.first_codepoint
                    && char_code < custom_font.first_codepoint + custom_font.char_count
                {
                    (char_code - custom_font.first_codepoint) as usize
                } else {
                    0 // Fallback to first character
                };

                // Get glyph width
                let glyph_width_px = custom_font
                    .char_widths
                    .as_ref()
                    .and_then(|widths| widths.get(glyph_index).copied())
                    .unwrap_or(custom_font.char_width);
                let glyph_width = glyph_width_px as f32 * scale;

                // Calculate UV coordinates
                let col = glyph_index % glyphs_per_row;
                let row = glyph_index / glyphs_per_row;

                let u0 = (col * max_glyph_width as usize) as f32 / atlas_width as f32;
                let v0 = (row * custom_font.char_height as usize) as f32 / atlas_height as f32;
                let u1 = ((col * max_glyph_width as usize) + glyph_width_px as usize) as f32
                    / atlas_width as f32;
                let v1 =
                    ((row + 1) * custom_font.char_height as usize) as f32 / atlas_height as f32;

                // Convert pixel coordinates to NDC
                let (x0_ndc, y0_ndc) = pixel_to_ndc(cursor_x, y, render_width, render_height);
                let (x1_ndc, y1_ndc) = pixel_to_ndc(
                    cursor_x + glyph_width,
                    y + glyph_height,
                    render_width,
                    render_height,
                );

                // Screen-space quad vertices (2D) in NDC coordinates
                // Format: POS_UV_COLOR (format 3)
                // Each vertex: [x, y, z, u, v, r, g, b]

                // Top-left
                vertices.extend_from_slice(&[x0_ndc, y0_ndc, 0.0, u0, v0, r, g, b]);
                // Top-right
                vertices.extend_from_slice(&[x1_ndc, y0_ndc, 0.0, u1, v0, r, g, b]);
                // Bottom-right
                vertices.extend_from_slice(&[x1_ndc, y1_ndc, 0.0, u1, v1, r, g, b]);
                // Bottom-left
                vertices.extend_from_slice(&[x0_ndc, y1_ndc, 0.0, u0, v1, r, g, b]);

                // Indices for two triangles (quad)
                indices.extend_from_slice(&[
                    vertex_index,
                    vertex_index + 1,
                    vertex_index + 2,
                    vertex_index,
                    vertex_index + 2,
                    vertex_index + 3,
                ]);

                cursor_x += glyph_width;
                vertex_index += 4;
            }
        } else {
            // Built-in font rendering (scaled monospace)
            let scale = size / font::GLYPH_HEIGHT as f32;
            let glyph_width = font::GLYPH_WIDTH as f32 * scale;
            let glyph_height = font::GLYPH_HEIGHT as f32 * scale;

            for ch in text.chars() {
                let char_code = ch as u32;

                // Get UV coordinates for this character
                let (u0, v0, u1, v1) = font::get_glyph_uv(char_code);

                // Convert pixel coordinates to NDC
                let (x0_ndc, y0_ndc) = pixel_to_ndc(cursor_x, y, render_width, render_height);
                let (x1_ndc, y1_ndc) = pixel_to_ndc(
                    cursor_x + glyph_width,
                    y + glyph_height,
                    render_width,
                    render_height,
                );

                // Screen-space quad vertices (2D) in NDC coordinates
                // Format: POS_UV_COLOR (format 3)
                // Each vertex: [x, y, z, u, v, r, g, b]

                // Top-left
                vertices.extend_from_slice(&[x0_ndc, y0_ndc, 0.0, u0, v0, r, g, b]);
                // Top-right
                vertices.extend_from_slice(&[x1_ndc, y0_ndc, 0.0, u1, v0, r, g, b]);
                // Bottom-right
                vertices.extend_from_slice(&[x1_ndc, y1_ndc, 0.0, u1, v1, r, g, b]);
                // Bottom-left
                vertices.extend_from_slice(&[x0_ndc, y1_ndc, 0.0, u0, v1, r, g, b]);

                // Indices for two triangles (quad)
                indices.extend_from_slice(&[
                    vertex_index,
                    vertex_index + 1,
                    vertex_index + 2,
                    vertex_index,
                    vertex_index + 2,
                    vertex_index + 3,
                ]);

                cursor_x += glyph_width;
                vertex_index += 4;
            }
        }

        (vertices, indices)
    }

    /// Ensure model matrix buffer has sufficient capacity
    fn ensure_model_buffer_capacity(&mut self, count: usize) {
        if count <= self.model_matrix_capacity {
            return;
        }

        let new_capacity = (count * 2).next_power_of_two();
        tracing::debug!(
            "Growing model matrix buffer: {} → {}",
            self.model_matrix_capacity,
            new_capacity
        );

        self.model_matrix_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Model Matrices"),
            size: (new_capacity * std::mem::size_of::<Mat4>()) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        self.model_matrix_capacity = new_capacity;
    }

    /// Ensure view matrix buffer has sufficient capacity
    fn ensure_view_buffer_capacity(&mut self, count: usize) {
        if count <= self.view_matrix_capacity {
            return;
        }

        let new_capacity = (count * 2).next_power_of_two();
        tracing::debug!(
            "Growing view matrix buffer: {} → {}",
            self.view_matrix_capacity,
            new_capacity
        );

        self.view_matrix_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("View Matrices"),
            size: (new_capacity * std::mem::size_of::<Mat4>()) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        self.view_matrix_capacity = new_capacity;
    }

    /// Ensure projection matrix buffer has sufficient capacity
    fn ensure_proj_buffer_capacity(&mut self, count: usize) {
        if count <= self.proj_matrix_capacity {
            return;
        }

        let new_capacity = (count * 2).next_power_of_two();
        tracing::debug!(
            "Growing projection matrix buffer: {} → {}",
            self.proj_matrix_capacity,
            new_capacity
        );

        self.proj_matrix_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Projection Matrices"),
            size: (new_capacity * std::mem::size_of::<Mat4>()) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        self.proj_matrix_capacity = new_capacity;
    }

    /// Ensure MVP indices buffer has sufficient capacity
    fn ensure_mvp_indices_buffer_capacity(&mut self, count: usize) {
        if count <= self.mvp_indices_capacity {
            return;
        }

        let new_capacity = (count * 2).next_power_of_two();
        tracing::debug!(
            "Growing MVP indices buffer: {} → {}",
            self.mvp_indices_capacity,
            new_capacity
        );

        self.mvp_indices_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("MVP Indices"),
            size: (new_capacity * 2 * std::mem::size_of::<u32>()) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        self.mvp_indices_capacity = new_capacity;
    }

    fn ensure_shading_state_buffer_capacity(&mut self, count: usize) {
        if count <= self.shading_state_capacity {
            return;
        }

        let new_capacity = (count * 2).next_power_of_two();
        tracing::debug!(
            "Growing shading state buffer: {} → {}",
            self.shading_state_capacity,
            new_capacity
        );

        self.shading_state_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Shading States"),
            size: (new_capacity * std::mem::size_of::<PackedUnifiedShadingState>()) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        self.shading_state_capacity = new_capacity;
    }

    /// Render the command buffer contents to a texture view
    ///
    /// This is the core rendering function that takes buffered draw commands
    /// and issues GPU draw calls.
    ///
    /// # Arguments
    /// * `view` - The texture view to render to
    /// * `z_state` - The Z console FFI state containing matrix pools
    /// * `clear_color` - Background clear color (RGBA 0-1)
    pub fn render_frame(
        &mut self,
        view: &wgpu::TextureView,
        z_state: &crate::state::ZFFIState,
        clear_color: [f32; 4],
    ) {
        // Create command encoder
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Game Render Encoder"),
            });

        // If no commands, just clear render target and blit to window
        if self.command_buffer.commands().is_empty() {
            {
                let _render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Clear Pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &self.render_target.color_view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color {
                                r: clear_color[0] as f64,
                                g: clear_color[1] as f64,
                                b: clear_color[2] as f64,
                                a: clear_color[3] as f64,
                            }),
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                        view: &self.render_target.depth_view,
                        depth_ops: Some(wgpu::Operations {
                            load: wgpu::LoadOp::Clear(1.0),
                            store: wgpu::StoreOp::Store,
                        }),
                        stencil_ops: None,
                    }),
                    timestamp_writes: None,
                    occlusion_query_set: None,
                });
            }

            // Calculate viewport based on scale mode
            let (viewport_x, viewport_y, viewport_width, viewport_height) = match self.scale_mode {
                crate::config::ScaleMode::Stretch => {
                    // Stretch to fill window (may distort aspect ratio)
                    (
                        0.0,
                        0.0,
                        self.config.width as f32,
                        self.config.height as f32,
                    )
                }
                crate::config::ScaleMode::PixelPerfect => {
                    // Integer scaling with letterboxing (pixel-perfect)
                    let render_width = self.render_target.width as f32;
                    let render_height = self.render_target.height as f32;
                    let window_width = self.config.width as f32;
                    let window_height = self.config.height as f32;

                    // Calculate largest integer scale that fits in window
                    let scale_x = (window_width / render_width).floor();
                    let scale_y = (window_height / render_height).floor();
                    let scale = scale_x.min(scale_y).max(1.0); // At least 1x

                    // Calculate scaled dimensions
                    let scaled_width = render_width * scale;
                    let scaled_height = render_height * scale;

                    // Center the viewport
                    let x = (window_width - scaled_width) / 2.0;
                    let y = (window_height - scaled_height) / 2.0;

                    (x, y, scaled_width, scaled_height)
                }
            };

            // Blit to window
            {
                let mut blit_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Blit Pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    depth_stencil_attachment: None,
                    timestamp_writes: None,
                    occlusion_query_set: None,
                });
                blit_pass.set_pipeline(&self.blit_pipeline);
                blit_pass.set_bind_group(0, &self.blit_bind_group, &[]);

                // Set viewport for scaling mode
                blit_pass.set_viewport(
                    viewport_x,
                    viewport_y,
                    viewport_width,
                    viewport_height,
                    0.0,
                    1.0,
                );

                blit_pass.draw(0..3, 0..1);
            }
            self.queue.submit(std::iter::once(encoder.finish()));
            return;
        }

        // Upload vertex/index data from command buffer to GPU buffers
        for format in 0..VERTEX_FORMAT_COUNT as u8 {
            let vertex_data = self.command_buffer.vertex_data(format);
            if !vertex_data.is_empty() {
                self.buffer_manager
                    .vertex_buffer_mut(format)
                    .ensure_capacity(&self.device, vertex_data.len() as u64);
                self.buffer_manager
                    .vertex_buffer(format)
                    .write_at(&self.queue, 0, vertex_data);
            }

            let index_data = self.command_buffer.index_data(format);
            if !index_data.is_empty() {
                let index_bytes: &[u8] = bytemuck::cast_slice(index_data);
                self.buffer_manager
                    .index_buffer_mut(format)
                    .ensure_capacity(&self.device, index_bytes.len() as u64);
                self.buffer_manager
                    .index_buffer(format)
                    .write_at(&self.queue, 0, index_bytes);
            }
        }

        // OPTIMIZATION 3: Sort draw commands IN-PLACE by (pipeline_key, texture_slots) to minimize state changes
        // Commands are reset at the start of next frame, so no need to preserve original order or clone
        self.command_buffer
            .commands_mut()
            .sort_unstable_by_key(|cmd| {
                // Extract fields from command variant
                let (format, depth_test, cull_mode, texture_slots, buffer_index, is_quad) = match cmd {
                    command_buffer::VRPCommand::Mesh { format, depth_test, cull_mode, texture_slots, buffer_index, .. } => {
                        (*format, *depth_test, *cull_mode, *texture_slots, Some(*buffer_index), false)
                    }
                    command_buffer::VRPCommand::IndexedMesh { format, depth_test, cull_mode, texture_slots, buffer_index, .. } => {
                        (*format, *depth_test, *cull_mode, *texture_slots, Some(*buffer_index), false)
                    }
                    command_buffer::VRPCommand::Quad { depth_test, cull_mode, texture_slots, .. } => {
                        (self.unit_quad_format, *depth_test, *cull_mode, *texture_slots, None, true)
                    }
                };

                // Extract blend mode from shading state for sorting
                let blend_mode = if let Some(buffer_idx) = buffer_index {
                    // Get shading index from mvp_shading_states buffer (second element of tuple)
                    let indices = z_state.mvp_shading_states
                        .get(buffer_idx as usize)
                        .expect("Invalid buffer_index in VRPCommand - this indicates a bug in state tracking");
                    let shading_state = z_state.shading_states.get(indices.shading_idx as usize)
                        .expect("Invalid shading_state_index - this indicates a bug in state tracking");
                    BlendMode::from_u8((shading_state.blend_mode & 0xFF) as u8)
                } else {
                    // Quads store shading state in instance data, assume default blend mode for sorting
                    BlendMode::None
                };

                // Sort key: (render_mode, format, blend_mode, depth_test, cull_mode, texture_slots)
                // This groups commands by pipeline first, then by textures
                let state = RenderState {
                    depth_test,
                    cull_mode,
                    blend_mode,
                    texture_filter: self.render_state.texture_filter,
                };

                // Create sort key based on pipeline type (Regular vs Quad)
                let (render_mode, vertex_format, blend_mode_u8, depth_test_u8, cull_mode_u8) =
                    if is_quad {
                        // Quad pipeline: Use special values to group separately
                        let pipeline_key = PipelineKey::quad(&state);
                        match pipeline_key {
                            PipelineKey::Quad { blend_mode, depth_test } => {
                                (255u8, 255u8, blend_mode, depth_test as u8, 0u8)
                            }
                            _ => unreachable!(),
                        }
                    } else {
                        // Regular pipeline: Use actual values
                        let pipeline_key = PipelineKey::new(self.current_render_mode, format, &state);
                        match pipeline_key {
                            PipelineKey::Regular { render_mode, vertex_format, blend_mode, depth_test, cull_mode } => {
                                (render_mode, vertex_format, blend_mode, depth_test as u8, cull_mode)
                            }
                            _ => unreachable!(),
                        }
                    };

                (
                    render_mode,
                    vertex_format,
                    blend_mode_u8,
                    depth_test_u8,
                    cull_mode_u8,
                    texture_slots[0].0,
                    texture_slots[1].0,
                    texture_slots[2].0,
                    texture_slots[3].0,
                )
            });

        // Upload matrices from z_state to GPU storage buffers
        // 1. Upload model matrices
        if !z_state.model_matrices.is_empty() {
            self.ensure_model_buffer_capacity(z_state.model_matrices.len());
            let data = bytemuck::cast_slice(&z_state.model_matrices);
            self.queue.write_buffer(&self.model_matrix_buffer, 0, data);
        }

        // 2. Upload view matrices
        if !z_state.view_matrices.is_empty() {
            self.ensure_view_buffer_capacity(z_state.view_matrices.len());
            let data = bytemuck::cast_slice(&z_state.view_matrices);
            self.queue.write_buffer(&self.view_matrix_buffer, 0, data);
        }

        // 3. Upload projection matrices
        if !z_state.proj_matrices.is_empty() {
            self.ensure_proj_buffer_capacity(z_state.proj_matrices.len());
            let data = bytemuck::cast_slice(&z_state.proj_matrices);
            self.queue.write_buffer(&self.proj_matrix_buffer, 0, data);
        }

        // 4. Upload shading states (NEW - Phase 5)
        if !z_state.shading_states.is_empty() {
            self.ensure_shading_state_buffer_capacity(z_state.shading_states.len());
            let data = bytemuck::cast_slice(&z_state.shading_states);
            self.queue.write_buffer(&self.shading_state_buffer, 0, data);
        }

        // 5. Upload MVP + shading state indices (already deduplicated by add_mvp_shading_state)
        // WGSL: array<vec4<u32>> - unpacked indices use all 4 fields naturally (no bit-packing!)
        // Each entry is 4 × u32: [model_idx, view_idx, proj_idx, shading_idx]
        let state_count = z_state.mvp_shading_states.len();
        if state_count > 0 {
            self.ensure_mvp_indices_buffer_capacity(state_count);
            let data = bytemuck::cast_slice(&z_state.mvp_shading_states);
            self.queue.write_buffer(&self.mvp_indices_buffer, 0, data);
        }

        // Take texture cache out temporarily to avoid nested mutable borrows during render pass.
        // Cache is persistent across frames - entries are reused when keys match.
        let mut texture_bind_groups = std::mem::take(&mut self.texture_bind_groups);

        // Create frame bind group once per frame (same for all draws)
        // Get bind group layout from first pipeline (all pipelines have same frame layout)
        let frame_bind_group = if let Some(first_cmd) = self.command_buffer.commands().first() {
            // Extract fields from first command variant
            let (format, depth_test, cull_mode) = match first_cmd {
                command_buffer::VRPCommand::Mesh { format, depth_test, cull_mode, .. } => (*format, *depth_test, *cull_mode),
                command_buffer::VRPCommand::IndexedMesh { format, depth_test, cull_mode, .. } => (*format, *depth_test, *cull_mode),
                command_buffer::VRPCommand::Quad { depth_test, cull_mode, .. } => (self.unit_quad_format, *depth_test, *cull_mode),
            };

            let first_state = RenderState {
                depth_test,
                cull_mode,
                blend_mode: BlendMode::None, // Doesn't matter for layout
                texture_filter: self.render_state.texture_filter,
            };
            let pipeline_entry = self.pipeline_cache.get_or_create(
                &self.device,
                self.config.format,
                self.current_render_mode,
                format,
                &first_state,
            );

            self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Frame Bind Group (Unified)"),
                layout: &pipeline_entry.bind_group_layout_frame,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: self.model_matrix_buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: self.view_matrix_buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: self.proj_matrix_buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 3,
                        resource: self.shading_state_buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 4,
                        resource: self.mvp_indices_buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 5,
                        resource: self.bone_buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 6,
                        resource: self.buffer_manager.quad_instance_buffer().as_entire_binding(),
                    },
                ],
            })
        } else {
            // No commands to render, nothing to do
            return;
        };

        // Render pass - render game content to offscreen target
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Game Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &self.render_target.color_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: clear_color[0] as f64,
                            g: clear_color[1] as f64,
                            b: clear_color[2] as f64,
                            a: clear_color[3] as f64,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.render_target.depth_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            // State tracking to skip redundant GPU calls (commands are sorted by pipeline/texture)
            let mut bound_pipeline: Option<PipelineKey> = None;
            let mut bound_texture_slots: Option<[TextureHandle; 4]> = None;
            let mut bound_vertex_format: Option<(u8, BufferSource)> = None;
            let mut frame_bind_group_set = false;

            for cmd in self.command_buffer.commands() {
                // Destructure command variant to extract common fields
                let (format, depth_test, cull_mode, texture_slots, buffer_source, is_quad) = match cmd {
                    command_buffer::VRPCommand::Mesh { format, depth_test, cull_mode, texture_slots, buffer_index, .. } => {
                        (*format, *depth_test, *cull_mode, *texture_slots, BufferSource::Immediate(*buffer_index), false)
                    }
                    command_buffer::VRPCommand::IndexedMesh { format, depth_test, cull_mode, texture_slots, buffer_index, .. } => {
                        (*format, *depth_test, *cull_mode, *texture_slots, BufferSource::Retained(*buffer_index), false)
                    }
                    command_buffer::VRPCommand::Quad { depth_test, cull_mode, texture_slots, .. } => {
                        (self.unit_quad_format, *depth_test, *cull_mode, *texture_slots, BufferSource::Quad, true)
                    }
                };

                // Extract blend mode from shading state for rendering
                // For Immediate/Retained, get from mvp_shading_states buffer
                // For Quad, assume default (quads store shading in instance data)
                let blend_mode = match buffer_source {
                    BufferSource::Immediate(buffer_idx) | BufferSource::Retained(buffer_idx) => {
                        let indices = z_state.mvp_shading_states
                            .get(buffer_idx as usize)
                            .expect("Invalid buffer_index in VRPCommand - this indicates a bug in state tracking");
                        let shading_state = z_state.shading_states.get(indices.shading_idx as usize)
                            .expect("Invalid shading_state_index - this indicates a bug in state tracking");
                        BlendMode::from_u8((shading_state.blend_mode & 0xFF) as u8)
                    }
                    BufferSource::Quad => BlendMode::None, // Placeholder for quads
                };

                // Create render state from command + blend mode
                let state = RenderState {
                    depth_test,
                    cull_mode,
                    blend_mode,
                    texture_filter: self.render_state.texture_filter,
                };

                // Get/create pipeline - use quad pipeline for quad rendering, regular for others
                if is_quad {
                    // Quad rendering: Ensure quad pipeline exists
                    self.pipeline_cache.get_or_create_quad(
                        &self.device,
                        self.config.format,
                        &state,
                    );
                } else {
                    // Regular mesh rendering: Ensure format-specific pipeline exists
                    if !self
                        .pipeline_cache
                        .contains(self.current_render_mode, format, &state)
                    {
                        self.pipeline_cache.get_or_create(
                            &self.device,
                            self.config.format,
                            self.current_render_mode,
                            format,
                            &state,
                        );
                    }
                }

                // Now get immutable reference to pipeline entry (avoiding borrow issues)
                let pipeline_key = if is_quad {
                    PipelineKey::quad(&state)
                } else {
                    PipelineKey::new(self.current_render_mode, format, &state)
                };

                let pipeline_entry = self.pipeline_cache.get_by_key(&pipeline_key)
                    .expect("Pipeline should exist after get_or_create");

                // Get or create texture bind group (cached by texture slots)
                let texture_bind_group = texture_bind_groups
                    .entry(texture_slots)
                    .or_insert_with(|| {
                        // Get texture views for this command's bound textures
                        let tex_view_0 = if texture_slots[0] == TextureHandle::INVALID {
                            self.get_fallback_white_view()
                        } else {
                            self.get_texture_view(texture_slots[0])
                                .unwrap_or_else(|| self.get_fallback_checkerboard_view())
                        };
                        let tex_view_1 = if texture_slots[1] == TextureHandle::INVALID {
                            self.get_fallback_white_view()
                        } else {
                            self.get_texture_view(texture_slots[1])
                                .unwrap_or_else(|| self.get_fallback_checkerboard_view())
                        };
                        let tex_view_2 = if texture_slots[2] == TextureHandle::INVALID {
                            self.get_fallback_white_view()
                        } else {
                            self.get_texture_view(texture_slots[2])
                                .unwrap_or_else(|| self.get_fallback_checkerboard_view())
                        };
                        let tex_view_3 = if texture_slots[3] == TextureHandle::INVALID {
                            self.get_fallback_white_view()
                        } else {
                            self.get_texture_view(texture_slots[3])
                                .unwrap_or_else(|| self.get_fallback_checkerboard_view())
                        };

                        self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                            label: Some("Texture Bind Group"),
                            layout: &pipeline_entry.bind_group_layout_textures,
                            entries: &[
                                wgpu::BindGroupEntry {
                                    binding: 0,
                                    resource: wgpu::BindingResource::TextureView(tex_view_0),
                                },
                                wgpu::BindGroupEntry {
                                    binding: 1,
                                    resource: wgpu::BindingResource::TextureView(tex_view_1),
                                },
                                wgpu::BindGroupEntry {
                                    binding: 2,
                                    resource: wgpu::BindingResource::TextureView(tex_view_2),
                                },
                                wgpu::BindGroupEntry {
                                    binding: 3,
                                    resource: wgpu::BindingResource::TextureView(tex_view_3),
                                },
                                wgpu::BindGroupEntry {
                                    binding: 4,
                                    resource: wgpu::BindingResource::Sampler(
                                        self.current_sampler(),
                                    ),
                                },
                            ],
                        })
                    });

                // Set pipeline (only if changed)
                if bound_pipeline != Some(pipeline_key) {
                    render_pass.set_pipeline(&pipeline_entry.pipeline);
                    bound_pipeline = Some(pipeline_key);
                }

                // Set frame bind group once (unified across all draws)
                if !frame_bind_group_set {
                    render_pass.set_bind_group(0, &frame_bind_group, &[]);
                    frame_bind_group_set = true;
                }

                // Set texture bind group (only if changed)
                if bound_texture_slots != Some(texture_slots) {
                    render_pass.set_bind_group(1, &*texture_bind_group, &[]);
                    bound_texture_slots = Some(texture_slots);
                }

                // Set vertex buffer (only if format or buffer source changed)
                if bound_vertex_format != Some((format, buffer_source)) {
                    let vertex_buffer = match buffer_source {
                        BufferSource::Immediate(_) => self.buffer_manager.vertex_buffer(format),
                        BufferSource::Retained(_) => {
                            self.buffer_manager.retained_vertex_buffer(format)
                        }
                        BufferSource::Quad => {
                            // Quad instancing uses unit quad mesh (format: POS_UV_COLOR)
                            self.buffer_manager.retained_vertex_buffer(self.unit_quad_format)
                        }
                    };
                    if let Some(buffer) = vertex_buffer.buffer() {
                        render_pass.set_vertex_buffer(0, buffer.slice(..));
                    }
                    bound_vertex_format = Some((format, buffer_source));
                }

                // Handle different rendering paths based on command variant
                match cmd {
                    command_buffer::VRPCommand::Quad { instance_count, base_vertex, first_index, .. } => {
                        // Quad rendering: Instance data comes from storage buffer binding(6)
                        // The quad shader reads QuadInstance data via @builtin(instance_index)
                        // No per-instance vertex attributes needed (unlike old approach)
                        // Unit quad: 4 vertices, 6 indices (2 triangles)

                        const UNIT_QUAD_INDEX_COUNT: u32 = 6;

                        tracing::info!(
                            "Drawing {} quad instances (indices {}..{}, base_vertex {}, textures: {:?})",
                            instance_count,
                            first_index,
                            first_index + UNIT_QUAD_INDEX_COUNT,
                            base_vertex,
                            texture_slots
                        );

                        // Indexed draw with GPU instancing (quads always use indices)
                        let index_buffer = self.buffer_manager.retained_index_buffer(self.unit_quad_format);
                        if let Some(buffer) = index_buffer.buffer() {
                            render_pass.set_index_buffer(buffer.slice(..), wgpu::IndexFormat::Uint16);
                            render_pass.draw_indexed(
                                *first_index..*first_index + UNIT_QUAD_INDEX_COUNT,
                                *base_vertex as i32,
                                0..*instance_count,
                            );
                        } else {
                            tracing::error!("Quad index buffer is None!");
                        }
                    }
                    command_buffer::VRPCommand::IndexedMesh { index_count, base_vertex, first_index, buffer_index, .. } => {
                        // Indexed mesh: MVP instancing with storage buffer lookup
                        let index_buffer = match buffer_source {
                            BufferSource::Immediate(_) => self.buffer_manager.index_buffer(format),
                            BufferSource::Retained(_) => {
                                self.buffer_manager.retained_index_buffer(format)
                            }
                            BufferSource::Quad => unreachable!(),
                        };
                        if let Some(buffer) = index_buffer.buffer() {
                            render_pass.set_index_buffer(buffer.slice(..), wgpu::IndexFormat::Uint16);
                            render_pass.draw_indexed(
                                *first_index..*first_index + *index_count,
                                *base_vertex as i32,
                                *buffer_index..*buffer_index + 1,
                            );
                        }
                    }
                    command_buffer::VRPCommand::Mesh { vertex_count, base_vertex, buffer_index, .. } => {
                        // Non-indexed mesh: MVP instancing with storage buffer lookup
                        render_pass.draw(
                            *base_vertex..*base_vertex + *vertex_count,
                            *buffer_index..*buffer_index + 1,
                        );
                    }
                }
            }
        }

        // Move texture cache back into self (preserving allocations for next frame)
        self.texture_bind_groups = texture_bind_groups;

        // Calculate viewport based on scale mode
        let (viewport_x, viewport_y, viewport_width, viewport_height) = match self.scale_mode {
            crate::config::ScaleMode::Stretch => {
                // Stretch to fill window (may distort aspect ratio)
                (
                    0.0,
                    0.0,
                    self.config.width as f32,
                    self.config.height as f32,
                )
            }
            crate::config::ScaleMode::PixelPerfect => {
                // Integer scaling with letterboxing (pixel-perfect)
                let render_width = self.render_target.width as f32;
                let render_height = self.render_target.height as f32;
                let window_width = self.config.width as f32;
                let window_height = self.config.height as f32;

                // Calculate largest integer scale that fits in window
                let scale_x = (window_width / render_width).floor();
                let scale_y = (window_height / render_height).floor();
                let scale = scale_x.min(scale_y).max(1.0); // At least 1x

                // Calculate scaled dimensions
                let scaled_width = render_width * scale;
                let scaled_height = render_height * scale;

                // Center the viewport
                let x = (window_width - scaled_width) / 2.0;
                let y = (window_height - scaled_height) / 2.0;

                (x, y, scaled_width, scaled_height)
            }
        };

        // Blit pass - scale render target to window surface
        {
            let mut blit_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Blit Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            blit_pass.set_pipeline(&self.blit_pipeline);
            blit_pass.set_bind_group(0, &self.blit_bind_group, &[]);

            // Set viewport for scaling mode
            blit_pass.set_viewport(
                viewport_x,
                viewport_y,
                viewport_width,
                viewport_height,
                0.0,
                1.0,
            );

            blit_pass.draw(0..3, 0..1); // Fullscreen triangle
        }

        // Submit commands
        self.queue.submit(std::iter::once(encoder.finish()));
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_text_quads_empty() {
        let (vertices, indices) =
            ZGraphics::generate_text_quads("", 0.0, 0.0, 16.0, 0xFFFFFFFF, None, 960.0, 540.0);
        assert!(vertices.is_empty());
        assert!(indices.is_empty());
    }

    #[test]
    fn test_generate_text_quads_single_char() {
        let (vertices, indices) =
            ZGraphics::generate_text_quads("A", 0.0, 0.0, 16.0, 0xFFFFFFFF, None, 960.0, 540.0);
        assert_eq!(vertices.len(), 32);
        assert_eq!(indices.len(), 6);
    }

    #[test]
    fn test_generate_text_quads_multiple_chars() {
        let (vertices, indices) =
            ZGraphics::generate_text_quads("Hello", 0.0, 0.0, 8.0, 0xFFFFFFFF, None, 960.0, 540.0);
        assert_eq!(vertices.len(), 160);
        assert_eq!(indices.len(), 30);
    }

    #[test]
    fn test_generate_text_quads_color() {
        let (vertices, _) =
            ZGraphics::generate_text_quads("X", 0.0, 0.0, 8.0, 0xFF0000FF, None, 960.0, 540.0);
        assert!((vertices[5] - 1.0).abs() < 0.01);
        assert!((vertices[6] - 0.0).abs() < 0.01);
        assert!((vertices[7] - 0.0).abs() < 0.01);
    }

    #[test]
    fn test_generate_text_quads_position() {
        let (vertices, _) =
            ZGraphics::generate_text_quads("A", 100.0, 50.0, 16.0, 0xFFFFFFFF, None, 960.0, 540.0);
        // Vertices are in NDC (Normalized Device Coordinates), not pixel coordinates
        // x: (100.0 / (960.0 * 0.5)) - 1.0 ≈ -0.7917
        // y: 1.0 - (50.0 / (540.0 * 0.5)) ≈ 0.8148
        assert!((vertices[0] - (-0.7917)).abs() < 0.01);
        assert!((vertices[1] - 0.8148).abs() < 0.01);
        assert!((vertices[2] - 0.0).abs() < 0.01);
    }

    #[test]
    fn test_generate_text_quads_indices_valid() {
        let (_, indices) =
            ZGraphics::generate_text_quads("AB", 0.0, 0.0, 8.0, 0xFFFFFFFF, None, 960.0, 540.0);
        assert_eq!(indices[0..6], [0, 1, 2, 0, 2, 3]);
        assert_eq!(indices[6..12], [4, 5, 6, 4, 6, 7]);
    }

    // Matrix preservation test removed - implementation changed with unified shading state
}
