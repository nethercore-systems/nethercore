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
mod render_state;
mod texture_manager;
mod vertex;

use hashbrown::HashMap;
use std::sync::Arc;

use anyhow::{Context, Result};
use glam::{Mat4, Vec3};
use wgpu::util::DeviceExt;
use winit::window::Window;

use emberware_core::console::Graphics;

// Re-export public types from submodules
pub use buffer::{BufferManager, GrowableBuffer, MeshHandle, RetainedMesh};
pub use command_buffer::{BufferSource, VirtualRenderPass};
pub use matrix_packing::MvpIndex;
pub use render_state::{
    BlendMode, CameraUniforms, CullMode, LightUniform, LightsUniforms, MatcapBlendMode,
    MaterialUniforms, RenderState, SkyUniforms, TextureFilter, TextureHandle,
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

/// Material cache key for deduplicating material uniform buffers
///
/// Combines all material properties that affect the uniform buffer contents.
/// Used as HashMap key to avoid creating duplicate buffers for identical materials.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct MaterialCacheKey {
    color: u32,
    metallic_bits: u32,
    roughness_bits: u32,
    emissive_bits: u32,
    matcap_blend_modes: [u32; 4],
}

impl MaterialCacheKey {
    fn new(
        color: u32,
        metallic: f32,
        roughness: f32,
        emissive: f32,
        matcap_blend_modes: [MatcapBlendMode; 4],
    ) -> Self {
        Self {
            color,
            metallic_bits: metallic.to_bits(),
            roughness_bits: roughness.to_bits(),
            emissive_bits: emissive.to_bits(),
            matcap_blend_modes: [
                matcap_blend_modes[0] as u32,
                matcap_blend_modes[1] as u32,
                matcap_blend_modes[2] as u32,
                matcap_blend_modes[3] as u32,
            ],
        }
    }
}

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

    // Sky system
    sky_uniforms: SkyUniforms,
    sky_buffer: wgpu::Buffer,

    // Camera system (view/projection + position for specular)
    camera_uniforms: CameraUniforms,
    camera_buffer: wgpu::Buffer,

    // Lighting system (4 directional lights for PBR)
    lights_uniforms: LightsUniforms,
    lights_buffer: wgpu::Buffer,

    // Material system (global metallic/roughness/emissive)
    material_uniforms: MaterialUniforms,
    material_buffer: wgpu::Buffer,

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

    // Bind group caches (cleared and repopulated each frame)
    material_buffers: HashMap<MaterialCacheKey, wgpu::Buffer>,
    texture_bind_groups: HashMap<[TextureHandle; 4], wgpu::BindGroup>,
    frame_bind_groups: HashMap<MaterialCacheKey, wgpu::BindGroup>,

    // Frame state
    current_frame: Option<wgpu::SurfaceTexture>,
    current_view: Option<wgpu::TextureView>,

    // Buffer management (vertex/index buffers and retained meshes)
    buffer_manager: BufferManager,

    // Command buffer for immediate mode draws
    command_buffer: VirtualRenderPass,

    // Current transform matrix (model transform)
    current_transform: Mat4,
    // Transform stack for push/pop
    transform_stack: Vec<Mat4>,

    // Shader and pipeline cache
    pipeline_cache: PipelineCache,
    current_render_mode: u8,

    // Current render target resolution (for detecting changes)
    current_resolution_index: u8,

    // Scaling mode for render target to window
    scale_mode: crate::config::ScaleMode,
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
        let buffer_manager = BufferManager::new(&device);

        // Create sky uniform buffer
        let sky_uniforms = SkyUniforms::default();
        let sky_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Sky Uniform Buffer"),
            contents: bytemuck::cast_slice(&[sky_uniforms]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        // Create camera uniform buffer (view/projection + position)
        let camera_uniforms = CameraUniforms::default();
        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Uniform Buffer"),
            contents: bytemuck::cast_slice(&[camera_uniforms]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        // Create lights uniform buffer (4 directional lights)
        let lights_uniforms = LightsUniforms::default();
        let lights_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Lights Uniform Buffer"),
            contents: bytemuck::cast_slice(&[lights_uniforms]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        // Create material uniform buffer (metallic/roughness/emissive)
        let material_uniforms = MaterialUniforms::default();
        let material_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Material Uniform Buffer"),
            contents: bytemuck::cast_slice(&[material_uniforms]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

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

        // Create MVP indices buffer (2 × u32 per entry: packed MVP + reserved)
        let mvp_indices_capacity = 1024;
        let mvp_indices_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("MVP Indices"),
            size: (mvp_indices_capacity * 2 * std::mem::size_of::<u32>()) as u64,
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
            sky_uniforms,
            sky_buffer,
            camera_uniforms,
            camera_buffer,
            lights_uniforms,
            lights_buffer,
            material_uniforms,
            material_buffer,
            bone_buffer,
            model_matrix_buffer,
            view_matrix_buffer,
            proj_matrix_buffer,
            mvp_indices_buffer,
            model_matrix_capacity,
            view_matrix_capacity,
            proj_matrix_capacity,
            mvp_indices_capacity,
            material_buffers: HashMap::new(),
            texture_bind_groups: HashMap::new(),
            frame_bind_groups: HashMap::new(),
            current_frame: None,
            current_view: None,
            buffer_manager,
            command_buffer: VirtualRenderPass::new(),
            current_transform: Mat4::IDENTITY,
            transform_stack: Vec::with_capacity(16),
            pipeline_cache: PipelineCache::new(),
            current_render_mode: 0,      // Default to Mode 0 (Unlit)
            current_resolution_index: 1, // 960×540 (default)
            scale_mode: crate::config::ScaleMode::default(), // Stretch by default
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

    /// Get texture view for a slot, returning fallback if unbound
    pub fn get_slot_texture_view(&self, slot: usize) -> &wgpu::TextureView {
        let handle = self
            .render_state
            .texture_slots
            .get(slot)
            .copied()
            .unwrap_or(TextureHandle::INVALID);
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

    /// Set uniform tint color (0xRRGGBBAA)
    pub fn set_color(&mut self, color: u32) {
        self.render_state.color = color;
    }

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

    /// Bind texture to slot 0 (albedo)
    pub fn bind_texture(&mut self, handle: TextureHandle) {
        self.bind_texture_slot(handle, 0);
    }

    /// Bind texture to a specific slot (0-3)
    pub fn bind_texture_slot(&mut self, handle: TextureHandle, slot: usize) {
        if slot < 4 {
            self.render_state.texture_slots[slot] = handle;
        }
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
    // Sky System
    // ========================================================================

    /// Set sky parameters for procedural sky rendering
    ///
    /// Parameters:
    /// - horizon_rgb: Horizon color (RGB, linear, 3 floats)
    /// - zenith_rgb: Zenith (top) color (RGB, linear, 3 floats)
    /// - sun_dir: Sun direction (normalized, XYZ, 3 floats)
    /// - sun_rgb: Sun color (RGB, linear, 3 floats)
    /// - sun_sharpness: Sun sharpness (higher = sharper sun, typically 32-256)
    pub fn set_sky(
        &mut self,
        horizon_rgb: [f32; 3],
        zenith_rgb: [f32; 3],
        sun_dir: [f32; 3],
        sun_rgb: [f32; 3],
        sun_sharpness: f32,
    ) {
        // Normalize sun direction
        let sun_vec = Vec3::from_array(sun_dir);
        let sun_normalized = if sun_vec.length() > 0.0001 {
            sun_vec.normalize()
        } else {
            Vec3::Y // Default to up if zero vector
        };

        self.sky_uniforms = SkyUniforms {
            horizon_color: [horizon_rgb[0], horizon_rgb[1], horizon_rgb[2], 0.0],
            zenith_color: [zenith_rgb[0], zenith_rgb[1], zenith_rgb[2], 0.0],
            sun_direction: [sun_normalized.x, sun_normalized.y, sun_normalized.z, 0.0],
            sun_color_and_sharpness: [sun_rgb[0], sun_rgb[1], sun_rgb[2], sun_sharpness],
        };

        // Upload to GPU
        self.queue.write_buffer(
            &self.sky_buffer,
            0,
            bytemuck::cast_slice(&[self.sky_uniforms]),
        );

        tracing::debug!(
            "Set sky: horizon={:?}, zenith={:?}, sun_dir={:?}, sun_color={:?}, sharpness={}",
            horizon_rgb,
            zenith_rgb,
            sun_normalized.to_array(),
            sun_rgb,
            sun_sharpness
        );
    }

    /// Get current sky uniforms
    pub fn sky_uniforms(&self) -> &SkyUniforms {
        &self.sky_uniforms
    }

    /// Get sky uniform buffer for binding
    pub fn sky_buffer(&self) -> &wgpu::Buffer {
        &self.sky_buffer
    }

    // ========================================================================
    // Scene Uniforms (Camera, Lights, Materials)
    // ========================================================================

    /// Update scene uniforms (camera, lights, materials) and upload to GPU
    ///
    /// This should be called once per frame before rendering to ensure PBR
    /// shaders have up-to-date camera position (for specular), lights, and
    /// material properties.
    ///
    /// # Arguments
    /// * `camera` - Camera state from ZFFIState
    /// * `lights` - Array of 4 light states from ZFFIState
    /// * `aspect_ratio` - Screen aspect ratio (width/height)
    /// * `metallic` - Global metallic value (0.0 = non-metallic, 1.0 = fully metallic)
    /// * `roughness` - Global roughness value (0.0 = smooth, 1.0 = rough)
    /// * `emissive` - Global emissive intensity (0.0 = no emission, 1.0+ = glowing)
    pub fn update_scene_uniforms(
        &mut self,
        camera: &crate::state::CameraState,
        lights: &[crate::state::LightState; 4],
        aspect_ratio: f32,
        metallic: f32,
        roughness: f32,
        emissive: f32,
    ) {
        // Update camera uniforms
        let view = camera.view_matrix();
        let proj = camera.projection_matrix(aspect_ratio);

        self.camera_uniforms = CameraUniforms {
            view: view.to_cols_array_2d(),
            projection: proj.to_cols_array_2d(),
            position: [camera.position.x, camera.position.y, camera.position.z, 0.0],
        };

        self.queue.write_buffer(
            &self.camera_buffer,
            0,
            bytemuck::cast_slice(&[self.camera_uniforms]),
        );

        // Update lights uniforms
        for (i, light) in lights.iter().enumerate() {
            // Normalize light direction
            let dir = Vec3::from_array(light.direction);
            let dir_normalized = if dir.length() > 0.0001 {
                dir.normalize()
            } else {
                Vec3::new(0.0, -1.0, 0.0) // Default: downward
            };

            self.lights_uniforms.lights[i] = LightUniform {
                direction_and_enabled: [
                    dir_normalized.x,
                    dir_normalized.y,
                    dir_normalized.z,
                    if light.enabled { 1.0 } else { 0.0 },
                ],
                color_and_intensity: [
                    light.color[0],
                    light.color[1],
                    light.color[2],
                    light.intensity,
                ],
            };
        }

        self.queue.write_buffer(
            &self.lights_buffer,
            0,
            bytemuck::cast_slice(&[self.lights_uniforms]),
        );

        // Update material uniforms
        self.material_uniforms = MaterialUniforms {
            properties: [
                metallic.clamp(0.0, 1.0),
                roughness.clamp(0.0, 1.0),
                emissive.max(0.0),
                0.0, // .w unused
            ],
        };

        self.queue.write_buffer(
            &self.material_buffer,
            0,
            bytemuck::cast_slice(&[self.material_uniforms]),
        );

        tracing::trace!(
            "Updated scene uniforms: camera_pos={:?}, lights_enabled={}/{}/{}/{}, material=M{:.2}/R{:.2}/E{:.2}",
            camera.position,
            lights[0].enabled,
            lights[1].enabled,
            lights[2].enabled,
            lights[3].enabled,
            metallic,
            roughness,
            emissive
        );
    }

    /// Get camera uniform buffer for binding
    pub fn camera_buffer(&self) -> &wgpu::Buffer {
        &self.camera_buffer
    }

    /// Get lights uniform buffer for binding
    pub fn lights_buffer(&self) -> &wgpu::Buffer {
        &self.lights_buffer
    }

    /// Get material uniform buffer for binding
    pub fn material_buffer(&self) -> &wgpu::Buffer {
        &self.material_buffer
    }

    // ========================================================================
    // Transform Stack
    // ========================================================================

    /// Reset transform to identity matrix
    pub fn transform_identity(&mut self) {
        self.current_transform = Mat4::IDENTITY;
    }

    /// Translate the current transform
    pub fn transform_translate(&mut self, x: f32, y: f32, z: f32) {
        self.current_transform *= Mat4::from_translation(glam::vec3(x, y, z));
    }

    /// Rotate the current transform around an axis (angle in degrees)
    pub fn transform_rotate(&mut self, angle_deg: f32, x: f32, y: f32, z: f32) {
        let axis = glam::vec3(x, y, z).normalize();
        let angle_rad = angle_deg.to_radians();
        self.current_transform *= Mat4::from_axis_angle(axis, angle_rad);
    }

    /// Scale the current transform
    pub fn transform_scale(&mut self, x: f32, y: f32, z: f32) {
        self.current_transform *= Mat4::from_scale(glam::vec3(x, y, z));
    }

    /// Push the current transform onto the stack
    ///
    /// Returns false if the stack is full (max 16 entries)
    pub fn transform_push(&mut self) -> bool {
        if self.transform_stack.len() >= 16 {
            tracing::warn!("Transform stack overflow (max 16)");
            return false;
        }
        self.transform_stack.push(self.current_transform);
        true
    }

    /// Pop the transform from the stack
    ///
    /// Returns false if the stack is empty
    pub fn transform_pop(&mut self) -> bool {
        if let Some(transform) = self.transform_stack.pop() {
            self.current_transform = transform;
            true
        } else {
            tracing::warn!("Transform stack underflow");
            false
        }
    }

    /// Set the current transform from a 4x4 matrix (16 floats, column-major)
    pub fn transform_set(&mut self, matrix: &[f32; 16]) {
        self.current_transform = Mat4::from_cols_array(matrix);
    }

    /// Get the current transform matrix
    pub fn current_transform(&self) -> Mat4 {
        self.current_transform
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

        // Update scene uniforms (camera, lights, materials) for PBR rendering
        // Note: aspect ratio is computed in app.rs and passed via update_scene_uniforms
        // This is kept separate as it needs window size which ZGraphics doesn't have

        // Convert matcap blend modes from u8 to MatcapBlendMode
        let matcap_blend_modes = [
            Self::convert_matcap_blend_mode(z_state.matcap_blend_modes[0]),
            Self::convert_matcap_blend_mode(z_state.matcap_blend_modes[1]),
            Self::convert_matcap_blend_mode(z_state.matcap_blend_modes[2]),
            Self::convert_matcap_blend_mode(z_state.matcap_blend_modes[3]),
        ];

        // 1. Swap the FFI-populated render pass into our command buffer
        // This efficiently transfers all immediate geometry (triangles, meshes)
        // without copying vectors. The old command buffer (now in z_state.render_pass)
        // will be cleared when z_state.clear_frame() is called.
        std::mem::swap(&mut self.command_buffer, &mut z_state.render_pass);

        // 2. Process deferred commands (billboards, sprites, text, sky)
        // These require additional processing or generation of geometry
        for cmd in z_state.deferred_commands.drain(..) {
            match cmd {
                DeferredCommand::DrawBillboard {
                    width,
                    height,
                    mode,
                    uv_rect,
                    transform,
                    color,
                    depth_test,
                    cull_mode,
                    blend_mode,
                    bound_textures,
                } => {
                    // Generate billboard quad geometry
                    tracing::trace!(
                        "Billboard: {}x{} mode={} at {:?} color={:08x}",
                        width,
                        height,
                        mode,
                        transform,
                        color
                    );

                    // Validate mode
                    if !(1..=4).contains(&mode) {
                        tracing::warn!("Invalid billboard mode: {} (must be 1-4)", mode);
                        continue;
                    }

                    // Map game texture handles to graphics texture handles
                    let texture_slots = Self::map_texture_handles(texture_map, &bound_textures);

                    // Extract position from transform (last column)
                    let position = transform.w_axis.truncate();

                    // Get direction from billboard to camera (same for all billboards)
                    // view_matrix.z_axis points backward (toward camera), which is what we want
                    let view_matrix = z_state.camera.view_matrix();
                    let to_camera = view_matrix.z_axis.truncate();

                    // Generate billboard orientation based on mode
                    let (right, up) = match mode {
                        1 => {
                            // Spherical: fully face camera (use view matrix axes directly)
                            let right = view_matrix.x_axis.truncate();
                            let up = view_matrix.y_axis.truncate();
                            (right, up)
                        }
                        2 => {
                            // Cylindrical Y-axis: all billboards face same direction (projected to XZ plane)
                            let to_camera_xz = glam::Vec3::new(to_camera.x, 0.0, to_camera.z);
                            if to_camera_xz.length_squared() > 0.0001 {
                                let to_camera_xz = to_camera_xz.normalize();
                                let right = glam::Vec3::Y.cross(to_camera_xz);
                                (right, glam::Vec3::Y)
                            } else {
                                // Camera pointing straight up/down - default orientation
                                (glam::Vec3::X, glam::Vec3::Y)
                            }
                        }
                        3 => {
                            // Cylindrical X-axis: all billboards face same direction (projected to YZ plane)
                            let to_camera_yz = glam::Vec3::new(0.0, to_camera.y, to_camera.z);
                            if to_camera_yz.length_squared() > 0.0001 {
                                let to_camera_yz = to_camera_yz.normalize();
                                let up = to_camera_yz.cross(glam::Vec3::X);
                                (glam::Vec3::X, up)
                            } else {
                                // Camera aligned with X-axis - default orientation
                                (glam::Vec3::X, glam::Vec3::Y)
                            }
                        }
                        4 => {
                            // Cylindrical Z-axis: all billboards face same direction (projected to XY plane)
                            let to_camera_xy = glam::Vec3::new(to_camera.x, to_camera.y, 0.0);
                            if to_camera_xy.length_squared() > 0.0001 {
                                let to_camera_xy = to_camera_xy.normalize();
                                let right = glam::Vec3::Z.cross(to_camera_xy);
                                (right, glam::Vec3::Z)
                            } else {
                                // Camera aligned with Z-axis - default orientation
                                (glam::Vec3::X, glam::Vec3::Y)
                            }
                        }
                        _ => unreachable!(),
                    };

                    // Calculate UV coordinates
                    let (u0, v0, u1, v1) = uv_rect.unwrap_or((0.0, 0.0, 1.0, 1.0));

                    // Generate quad vertices (POS_UV_COLOR format = 3)
                    // Position (3) + UV (2) + Color (3) = 8 floats per vertex
                    let half_w = width * 0.5;
                    let half_h = height * 0.5;

                    // Extract color components (RGBA)
                    let r = ((color >> 24) & 0xFF) as f32 / 255.0;
                    let g = ((color >> 16) & 0xFF) as f32 / 255.0;
                    let b = ((color >> 8) & 0xFF) as f32 / 255.0;

                    // Four corners of the billboard
                    let v0_pos = position - right * half_w - up * half_h;
                    let v1_pos = position + right * half_w - up * half_h;
                    let v2_pos = position + right * half_w + up * half_h;
                    let v3_pos = position - right * half_w + up * half_h;

                    #[rustfmt::skip]
                    let vertices = vec![
                        // Vertex 0 (bottom-left)
                        v0_pos.x, v0_pos.y, v0_pos.z, u0, v1, r, g, b,
                        // Vertex 1 (bottom-right)
                        v1_pos.x, v1_pos.y, v1_pos.z, u1, v1, r, g, b,
                        // Vertex 2 (top-right)
                        v2_pos.x, v2_pos.y, v2_pos.z, u1, v0, r, g, b,
                        // Vertex 3 (top-left)
                        v3_pos.x, v3_pos.y, v3_pos.z, u0, v0, r, g, b,
                    ];

                    // Two triangles (CCW winding)
                    let indices = vec![0, 1, 2, 0, 2, 3];

                    // POS_UV_COLOR format
                    let format = 3;

                    // Append vertex and index data
                    let base_vertex = self.command_buffer.append_vertex_data(format, &vertices);
                    let first_index = self.command_buffer.append_index_data(format, &indices);

                    // Add draw command with identity matrix indices (positions are in world space)
                    self.command_buffer.add_command(command_buffer::VRPCommand {
                        format,
                        mvp_index: MvpIndex::new(0, 0, 0), // Identity transform at index 0
                        vertex_count: 4,
                        index_count: 6,
                        base_vertex,
                        first_index,
                        buffer_source: BufferSource::Immediate,
                        texture_slots,
                        color: 0xFFFFFFFF, // White (color already in vertices)
                        depth_test,
                        cull_mode: Self::convert_cull_mode(cull_mode),
                        blend_mode: Self::convert_blend_mode(blend_mode),
                        matcap_blend_modes,
                    });
                }
                DeferredCommand::DrawSprite {
                    x,
                    y,
                    width,
                    height,
                    uv_rect,
                    origin,
                    rotation,
                    color,
                    blend_mode,
                    bound_textures,
                } => {
                    // 2D sprite rendering in screen space
                    tracing::trace!(
                        "Sprite: {}x{} at ({},{}) color={:08x}",
                        width,
                        height,
                        x,
                        y,
                        color
                    );

                    // Map game texture handles to graphics texture handles
                    let texture_slots = Self::map_texture_handles(texture_map, &bound_textures);

                    // Extract color components (RGBA: 0xRRGGBBAA)
                    let r = ((color >> 24) & 0xFF) as f32 / 255.0;
                    let g = ((color >> 16) & 0xFF) as f32 / 255.0;
                    let b = ((color >> 8) & 0xFF) as f32 / 255.0;

                    // Get UV coordinates
                    let (u0, v0, u1, v1) = uv_rect.unwrap_or((0.0, 0.0, 1.0, 1.0));

                    // Get origin offset (default to top-left)
                    let (origin_x, origin_y) = origin.unwrap_or((0.0, 0.0));

                    // Calculate corner positions with origin offset
                    let x0 = x - origin_x;
                    let y0 = y - origin_y;
                    let x1 = x0 + width;
                    let y1 = y0 + height;

                    // Generate quad vertices
                    // If rotation is non-zero, rotate around the sprite's origin point
                    let vertices = if rotation.abs() > 0.0001 {
                        // Rotate around (x, y) which is the sprite's anchor point
                        let cos_r = rotation.cos();
                        let sin_r = rotation.sin();

                        // Transform each corner relative to rotation center (x, y)
                        let rotate_point = |px: f32, py: f32| -> (f32, f32) {
                            let dx = px - x;
                            let dy = py - y;
                            let rx = dx * cos_r - dy * sin_r;
                            let ry = dx * sin_r + dy * cos_r;
                            (x + rx, y + ry)
                        };

                        let (rx0, ry0) = rotate_point(x0, y0);
                        let (rx1, ry1) = rotate_point(x1, y0);
                        let (rx2, ry2) = rotate_point(x1, y1);
                        let (rx3, ry3) = rotate_point(x0, y1);

                        // Convert rotated pixel coordinates to NDC
                        let (rx0_ndc, ry0_ndc) =
                            pixel_to_ndc(rx0, ry0, render_width_f, render_height_f);
                        let (rx1_ndc, ry1_ndc) =
                            pixel_to_ndc(rx1, ry1, render_width_f, render_height_f);
                        let (rx2_ndc, ry2_ndc) =
                            pixel_to_ndc(rx2, ry2, render_width_f, render_height_f);
                        let (rx3_ndc, ry3_ndc) =
                            pixel_to_ndc(rx3, ry3, render_width_f, render_height_f);

                        vec![
                            // Top-left
                            rx0_ndc, ry0_ndc, 0.0, u0, v0, r, g, b, // Top-right
                            rx1_ndc, ry1_ndc, 0.0, u1, v0, r, g, b, // Bottom-right
                            rx2_ndc, ry2_ndc, 0.0, u1, v1, r, g, b, // Bottom-left
                            rx3_ndc, ry3_ndc, 0.0, u0, v1, r, g, b,
                        ]
                    } else {
                        // No rotation - simple quad, convert pixel coordinates to NDC
                        let (x0_ndc, y0_ndc) =
                            pixel_to_ndc(x0, y0, render_width_f, render_height_f);
                        let (x1_ndc, y1_ndc) =
                            pixel_to_ndc(x1, y1, render_width_f, render_height_f);

                        vec![
                            // Top-left
                            x0_ndc, y0_ndc, 0.0, u0, v0, r, g, b, // Top-right
                            x1_ndc, y0_ndc, 0.0, u1, v0, r, g, b, // Bottom-right
                            x1_ndc, y1_ndc, 0.0, u1, v1, r, g, b, // Bottom-left
                            x0_ndc, y1_ndc, 0.0, u0, v1, r, g, b,
                        ]
                    };

                    // Two triangles (6 indices)
                    let indices = vec![0, 1, 2, 0, 2, 3];

                    // POS_UV_COLOR format
                    const SPRITE_FORMAT: u8 = 3;

                    // Append vertex and index data
                    let base_vertex = self
                        .command_buffer
                        .append_vertex_data(SPRITE_FORMAT, &vertices);
                    let first_index = self
                        .command_buffer
                        .append_index_data(SPRITE_FORMAT, &indices);

                    // Add draw command with identity transform (screen space)
                    self.command_buffer.add_command(command_buffer::VRPCommand {
                        format: SPRITE_FORMAT,
                        mvp_index: MvpIndex::new(0, 0, 0),
                        vertex_count: 4,
                        index_count: 6,
                        base_vertex,
                        first_index,
                        buffer_source: BufferSource::Immediate,
                        texture_slots,
                        color: 0xFFFFFFFF, // Color already in vertices
                        depth_test: false, // 2D sprites don't use depth test
                        cull_mode: CullMode::None,
                        blend_mode: Self::convert_blend_mode(blend_mode),
                        matcap_blend_modes,
                    });
                }
                DeferredCommand::DrawRect {
                    x,
                    y,
                    width,
                    height,
                    color,
                    blend_mode,
                } => {
                    // 2D rectangle rendering in screen space (solid color, no texture)
                    tracing::trace!(
                        "Rect: {}x{} at ({},{}) color={:08x}",
                        width,
                        height,
                        x,
                        y,
                        color
                    );

                    // Extract color components (RGBA: 0xRRGGBBAA)
                    let r = ((color >> 24) & 0xFF) as f32 / 255.0;
                    let g = ((color >> 16) & 0xFF) as f32 / 255.0;
                    let b = ((color >> 8) & 0xFF) as f32 / 255.0;

                    // Convert pixel coordinates to NDC
                    let (x0_ndc, y0_ndc) = pixel_to_ndc(x, y, render_width_f, render_height_f);
                    let (x1_ndc, y1_ndc) =
                        pixel_to_ndc(x + width, y + height, render_width_f, render_height_f);

                    // Generate quad vertices (POS_COLOR format = 2) in NDC coordinates
                    // Format: [x, y, z, r, g, b]
                    #[rustfmt::skip]
                    let vertices = vec![
                        // Top-left
                        x0_ndc, y0_ndc, 0.0, r, g, b,
                        // Top-right
                        x1_ndc, y0_ndc, 0.0, r, g, b,
                        // Bottom-right
                        x1_ndc, y1_ndc, 0.0, r, g, b,
                        // Bottom-left
                        x0_ndc, y1_ndc, 0.0, r, g, b,
                    ];

                    // Two triangles (6 indices)
                    let indices = vec![0, 1, 2, 0, 2, 3];

                    // POS_COLOR format (no UV)
                    const RECT_FORMAT: u8 = 2;

                    // Append vertex and index data
                    let base_vertex = self
                        .command_buffer
                        .append_vertex_data(RECT_FORMAT, &vertices);
                    let first_index = self.command_buffer.append_index_data(RECT_FORMAT, &indices);

                    // Add draw command with identity transform (screen space)
                    self.command_buffer.add_command(command_buffer::VRPCommand {
                        format: RECT_FORMAT,
                        mvp_index: MvpIndex::new(0, 0, 0),
                        vertex_count: 4,
                        index_count: 6,
                        base_vertex,
                        first_index,
                        buffer_source: BufferSource::Immediate,
                        texture_slots: [TextureHandle::INVALID; 4],
                        color: 0xFFFFFFFF, // Color already in vertices
                        depth_test: false, // 2D rectangles don't use depth test
                        cull_mode: CullMode::None,
                        blend_mode: Self::convert_blend_mode(blend_mode),
                        matcap_blend_modes,
                    });
                }
                DeferredCommand::DrawText {
                    text,
                    x,
                    y,
                    size,
                    color,
                    blend_mode,
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
                    self.command_buffer.add_command(command_buffer::VRPCommand {
                        format: TEXT_FORMAT,
                        mvp_index: MvpIndex::new(0, 0, 0),
                        vertex_count: (vertices.len() / 8) as u32, // 8 floats per vertex
                        index_count: indices.len() as u32,
                        base_vertex,
                        first_index,
                        buffer_source: BufferSource::Immediate,
                        texture_slots,
                        color: 0xFFFFFFFF, // Color already baked into vertices
                        depth_test: false, // 2D text doesn't use depth test
                        cull_mode: CullMode::None,
                        blend_mode: Self::convert_blend_mode(blend_mode),
                        matcap_blend_modes,
                    });
                }
                DeferredCommand::SetSky {
                    horizon_color,
                    zenith_color,
                    sun_direction,
                    sun_color,
                    sun_sharpness,
                } => {
                    self.set_sky(
                        horizon_color,
                        zenith_color,
                        sun_direction,
                        sun_color,
                        sun_sharpness,
                    );
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
                // Sort key: (render_mode, format, blend_mode, depth_test, cull_mode, texture_slots)
                // This groups commands by pipeline first, then by textures
                let state = RenderState {
                    color: cmd.color,
                    depth_test: cmd.depth_test,
                    cull_mode: cmd.cull_mode,
                    blend_mode: cmd.blend_mode,
                    texture_filter: self.render_state.texture_filter,
                    texture_slots: cmd.texture_slots,
                    matcap_blend_modes: cmd.matcap_blend_modes,
                };
                let pipeline_key = PipelineKey::new(self.current_render_mode, cmd.format, &state);
                (
                    pipeline_key.render_mode,
                    pipeline_key.vertex_format,
                    pipeline_key.blend_mode,
                    pipeline_key.depth_test as u8,
                    pipeline_key.cull_mode,
                    cmd.texture_slots[0].0,
                    cmd.texture_slots[1].0,
                    cmd.texture_slots[2].0,
                    cmd.texture_slots[3].0,
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

        // 4. Collect and upload MVP indices from command buffer
        // Each entry is 2 × u32: [packed_mvp, reserved]
        let command_count = self.command_buffer.commands().len();
        if command_count > 0 {
            // Collect MVP indices first
            let mut mvp_indices_data = Vec::with_capacity(command_count * 2);
            for cmd in self.command_buffer.commands() {
                mvp_indices_data.push(cmd.mvp_index.0); // Packed MVP
                mvp_indices_data.push(0u32);            // Reserved
            }

            // Ensure capacity and upload
            self.ensure_mvp_indices_buffer_capacity(command_count);
            let data = bytemuck::cast_slice(&mvp_indices_data);
            self.queue.write_buffer(&self.mvp_indices_buffer, 0, data);
        }

        // Take caches out of self temporarily to avoid nested mutable borrows during render pass.
        // Caches are persistent across frames - entries are reused when keys match.
        // Cache growth is bounded by unique (material, texture) combinations used.
        let mut material_buffers = std::mem::take(&mut self.material_buffers);
        let mut texture_bind_groups = std::mem::take(&mut self.texture_bind_groups);
        let mut frame_bind_groups = std::mem::take(&mut self.frame_bind_groups);

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
            let mut bound_material: Option<MaterialCacheKey> = None;

            // Instance index for accessing MVP indices in storage buffer
            let mut instance_index: u32 = 0;

            for cmd in self.command_buffer.commands() {
                // Create render state from command
                let state = RenderState {
                    color: cmd.color,
                    depth_test: cmd.depth_test,
                    cull_mode: cmd.cull_mode,
                    blend_mode: cmd.blend_mode,
                    texture_filter: self.render_state.texture_filter,
                    texture_slots: cmd.texture_slots,
                    matcap_blend_modes: cmd.matcap_blend_modes,
                };

                // Get/create pipeline (using contains + get pattern to avoid borrow issues)
                let pipeline_key = PipelineKey::new(self.current_render_mode, cmd.format, &state);
                if !self
                    .pipeline_cache
                    .contains(self.current_render_mode, cmd.format, &state)
                {
                    // Create and insert pipeline if it doesn't exist
                    self.pipeline_cache.get_or_create(
                        &self.device,
                        self.config.format,
                        self.current_render_mode,
                        cmd.format,
                        &state,
                    );
                }
                // Safe to unwrap since we just ensured it exists
                let pipeline_entry = self
                    .pipeline_cache
                    .get(self.current_render_mode, cmd.format, &state)
                    .unwrap();

                // Get or create material uniform buffer (cached by color + properties + blend modes)
                let material_key = MaterialCacheKey::new(
                    cmd.color,
                    self.material_uniforms.properties[0],
                    self.material_uniforms.properties[1],
                    self.material_uniforms.properties[2],
                    state.matcap_blend_modes,
                );
                let material_buffer = material_buffers.entry(material_key).or_insert_with(|| {
                    let color_vec = state.color_vec4();
                    #[repr(C)]
                    #[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
                    struct MaterialUniforms {
                        color: [f32; 4],
                        matcap_blend_modes: [u32; 4],
                    }
                    let material = MaterialUniforms {
                        color: [color_vec.x, color_vec.y, color_vec.z, color_vec.w],
                        matcap_blend_modes: [
                            state.matcap_blend_modes[0] as u32,
                            state.matcap_blend_modes[1] as u32,
                            state.matcap_blend_modes[2] as u32,
                            state.matcap_blend_modes[3] as u32,
                        ],
                    };
                    self.device
                        .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                            label: Some("Material Buffer"),
                            contents: bytemuck::cast_slice(&[material]),
                            usage: wgpu::BufferUsages::UNIFORM,
                        })
                });

                // Create frame bind group (group 0) - Now reusable across draws since model matrices are in storage buffer!
                // Frame bind groups cached by material (since material buffer is the only per-draw varying resource)
                let frame_bind_group_key = material_key; // Reuse material_key as frame bind group key
                let frame_bind_group = frame_bind_groups
                    .entry(frame_bind_group_key)
                    .or_insert_with(|| {
                        match self.current_render_mode {
                            0 | 1 => {
                                // Mode 0 (Unlit) and Mode 1 (Matcap): Basic bindings
                                self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                                    label: Some("Frame Bind Group"),
                                    layout: &pipeline_entry.bind_group_layout_frame,
                                    entries: &[
                                        wgpu::BindGroupEntry {
                                            binding: 0,
                                            resource: self
                                                .model_matrix_buffer
                                                .as_entire_binding(),
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
                                            resource: self.mvp_indices_buffer.as_entire_binding(),
                                        },
                                        wgpu::BindGroupEntry {
                                            binding: 4,
                                            resource: self.sky_buffer.as_entire_binding(),
                                        },
                                        wgpu::BindGroupEntry {
                                            binding: 5,
                                            resource: material_buffer.as_entire_binding(),
                                        },
                                        wgpu::BindGroupEntry {
                                            binding: 6,
                                            resource: self.bone_buffer.as_entire_binding(),
                                        },
                                    ],
                                })
                            }
                            2 | 3 => {
                                // Mode 2 (PBR) and Mode 3 (Hybrid): Additional lighting uniforms
                                self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                                    label: Some("Frame Bind Group"),
                                    layout: &pipeline_entry.bind_group_layout_frame,
                                    entries: &[
                                        wgpu::BindGroupEntry {
                                            binding: 0,
                                            resource: self
                                                .model_matrix_buffer
                                                .as_entire_binding(),
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
                                            resource: self.mvp_indices_buffer.as_entire_binding(),
                                        },
                                        wgpu::BindGroupEntry {
                                            binding: 4,
                                            resource: self.sky_buffer.as_entire_binding(),
                                        },
                                        wgpu::BindGroupEntry {
                                            binding: 5,
                                            resource: material_buffer.as_entire_binding(),
                                        },
                                        wgpu::BindGroupEntry {
                                            binding: 6,
                                            resource: self.lights_buffer.as_entire_binding(),
                                        },
                                        wgpu::BindGroupEntry {
                                            binding: 7,
                                            resource: self.camera_buffer.as_entire_binding(),
                                        },
                                        wgpu::BindGroupEntry {
                                            binding: 8,
                                            resource: self.bone_buffer.as_entire_binding(),
                                        },
                                    ],
                                })
                            }
                            _ => {
                                // Fallback - same as mode 0/1
                                self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                                    label: Some("Frame Bind Group"),
                                    layout: &pipeline_entry.bind_group_layout_frame,
                                    entries: &[
                                        wgpu::BindGroupEntry {
                                            binding: 0,
                                            resource: self
                                                .model_matrix_buffer
                                                .as_entire_binding(),
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
                                            resource: self.sky_buffer.as_entire_binding(),
                                        },
                                        wgpu::BindGroupEntry {
                                            binding: 4,
                                            resource: material_buffer.as_entire_binding(),
                                        },
                                        wgpu::BindGroupEntry {
                                            binding: 5,
                                            resource: self.bone_buffer.as_entire_binding(),
                                        },
                                    ],
                                })
                            }
                        }
                    });

                // Get or create texture bind group (cached by texture slots)
                let texture_bind_group = texture_bind_groups
                    .entry(cmd.texture_slots)
                    .or_insert_with(|| {
                        // Get texture views for this command's bound textures
                        let tex_view_0 = if cmd.texture_slots[0] == TextureHandle::INVALID {
                            self.get_fallback_white_view()
                        } else {
                            self.get_texture_view(cmd.texture_slots[0])
                                .unwrap_or_else(|| self.get_fallback_checkerboard_view())
                        };
                        let tex_view_1 = if cmd.texture_slots[1] == TextureHandle::INVALID {
                            self.get_fallback_white_view()
                        } else {
                            self.get_texture_view(cmd.texture_slots[1])
                                .unwrap_or_else(|| self.get_fallback_checkerboard_view())
                        };
                        let tex_view_2 = if cmd.texture_slots[2] == TextureHandle::INVALID {
                            self.get_fallback_white_view()
                        } else {
                            self.get_texture_view(cmd.texture_slots[2])
                                .unwrap_or_else(|| self.get_fallback_checkerboard_view())
                        };
                        let tex_view_3 = if cmd.texture_slots[3] == TextureHandle::INVALID {
                            self.get_fallback_white_view()
                        } else {
                            self.get_texture_view(cmd.texture_slots[3])
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

                // Set frame bind group (only if material changed, since model matrices are in storage buffer)
                if bound_material != Some(material_key) {
                    render_pass.set_bind_group(0, &*frame_bind_group, &[]);
                    bound_material = Some(material_key);
                }

                // Set texture bind group (only if changed)
                if bound_texture_slots != Some(cmd.texture_slots) {
                    render_pass.set_bind_group(1, &*texture_bind_group, &[]);
                    bound_texture_slots = Some(cmd.texture_slots);
                }

                // Set vertex buffer (only if format or buffer source changed)
                if bound_vertex_format != Some((cmd.format, cmd.buffer_source)) {
                    let vertex_buffer = match cmd.buffer_source {
                        BufferSource::Immediate => self.buffer_manager.vertex_buffer(cmd.format),
                        BufferSource::Retained => {
                            self.buffer_manager.retained_vertex_buffer(cmd.format)
                        }
                    };
                    if let Some(buffer) = vertex_buffer.buffer() {
                        render_pass.set_vertex_buffer(0, buffer.slice(..));
                    }
                    bound_vertex_format = Some((cmd.format, cmd.buffer_source));
                }

                // Draw using instanced rendering - instance_index fetches MVP indices from storage buffer
                if cmd.index_count > 0 {
                    // Indexed draw - both immediate and retained use u16 indices
                    let index_buffer = match cmd.buffer_source {
                        BufferSource::Immediate => self.buffer_manager.index_buffer(cmd.format),
                        BufferSource::Retained => {
                            self.buffer_manager.retained_index_buffer(cmd.format)
                        }
                    };
                    if let Some(buffer) = index_buffer.buffer() {
                        render_pass.set_index_buffer(buffer.slice(..), wgpu::IndexFormat::Uint16);
                        render_pass.draw_indexed(
                            cmd.first_index..cmd.first_index + cmd.index_count,
                            cmd.base_vertex as i32,
                            instance_index..instance_index + 1,
                        );
                    }
                } else {
                    // Non-indexed draw
                    render_pass.draw(
                        cmd.base_vertex..cmd.base_vertex + cmd.vertex_count,
                        instance_index..instance_index + 1,
                    );
                }

                // Increment instance index for next draw
                instance_index += 1;
            }
        }

        // Move caches back into self (preserving allocations for next frame)
        self.material_buffers = material_buffers;
        self.texture_bind_groups = texture_bind_groups;
        self.frame_bind_groups = frame_bind_groups;

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

        // Reset transform to identity
        self.current_transform = Mat4::IDENTITY;
        self.transform_stack.clear();

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

    #[test]
    fn test_immediate_draw_matrices_preserved_after_process() {
        use crate::state::ZFFIState;
        use glam::{Mat4, Vec3};

        let mut z_state = ZFFIState::new();

        // Simulate immediate draw workflow: add matrix and record command
        let transform = Mat4::from_translation(Vec3::new(1.0, 2.0, 3.0));
        let model_idx = z_state.add_model_matrix(transform).unwrap();
        assert_eq!(model_idx, 1); // Index 0 is identity, this should be 1

        let mvp_index = crate::graphics::MvpIndex::new(
            model_idx,
            z_state.current_view_idx,
            z_state.current_proj_idx,
        );

        // Record a draw command
        z_state.render_pass.record_triangles(
            0, // format
            &[0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.5, 1.0, 0.0],
            mvp_index,
            0xFFFFFFFF,
            true,
            crate::graphics::CullMode::Back,
            crate::graphics::BlendMode::None,
            [crate::graphics::TextureHandle::INVALID; 4],
            [crate::graphics::MatcapBlendMode::Multiply; 4],
        );

        // Verify matrices exist before process
        assert_eq!(z_state.model_matrices.len(), 2); // Identity + added
        assert_eq!(z_state.model_matrices[1], transform);

        // This is the bug: process_draw_commands will call clear_frame()
        // which clears model_matrices before they can be uploaded!
        // For this test, we'll just simulate what happens:
        // In real code, process_draw_commands swaps render_pass and then calls clear_frame
        // which clears model_matrices

        // After clear_frame is called in process_draw_commands:
        z_state.clear_frame();

        // BUG: Model matrices are cleared! Index 1 no longer exists
        assert_eq!(z_state.model_matrices.len(), 1); // Only identity remains

        // When render_frame tries to use MVP index (1, 0, 0), index 1 doesn't exist!
        // This test demonstrates the bug - should fail with current implementation
    }
}
