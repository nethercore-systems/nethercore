//! Graphics initialization and setup
//!
//! This module contains all initialization logic for ZGraphics including:
//! - Creating the wgpu instance, device, and surface
//! - Setting up render targets and depth buffers
//! - Creating the blit pipeline for scaling render target to window
//! - Initializing vertex buffers and GPU resources

use anyhow::{Context, Result};
use glam::Mat4;
use std::sync::Arc;
use wgpu::util::DeviceExt;
use winit::window::Window;

use super::vertex::{vertex_stride, FORMAT_COLOR, FORMAT_UV};
use super::ZGraphics;

/// Offscreen render target for fixed internal resolution
///
/// Game content renders at a fixed resolution (e.g., 960×540) to this target,
/// then gets scaled/blitted to the window surface (which can be any size).
///
/// # Architecture Note
///
/// The `color_texture` and `depth_texture` fields are never directly accessed
/// after creation, but they MUST be stored here because wgpu::TextureView does
/// not own the underlying texture. Dropping the texture would invalidate the views.
///
/// This is separate from `ZGraphics::depth_texture/depth_view` which is used for
/// window-sized UI rendering (not game content). The separation allows:
/// - Game renders at fixed resolution (pixel-perfect, stable coordinates)
/// - UI renders at window resolution (crisp regardless of window size)
/// - Blit pipeline scales game to window with configurable filtering
pub(crate) struct RenderTarget {
    #[allow(dead_code)] // Needed to keep texture alive for color_view
    pub(super) color_texture: wgpu::Texture,
    pub(super) color_view: wgpu::TextureView,
    #[allow(dead_code)] // Needed to keep texture alive for depth_view
    pub(super) depth_texture: wgpu::Texture,
    pub(super) depth_view: wgpu::TextureView,
    pub(super) width: u32,
    pub(super) height: u32,
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
        let mut buffer_manager = super::BufferManager::new(&device);

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
            size: (shading_state_capacity * std::mem::size_of::<super::PackedUnifiedShadingState>())
                as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // Create texture manager (handles fallback textures)
        let texture_manager = super::texture_manager::TextureManager::new(&device, &queue)?;

        // Create offscreen render target at default resolution (960×540)
        let render_target = Self::create_render_target(&device, 960, 540, surface_format);

        // Create screen dimensions uniform buffer (for screen-space quad rendering)
        let screen_dims_data: [f32; 2] = [render_target.width as f32, render_target.height as f32];
        let screen_dims_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Screen Dimensions Uniform"),
            contents: bytemuck::cast_slice(&screen_dims_data),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        // Create blit pipeline for scaling render target to window
        let (blit_pipeline, blit_bind_group, blit_sampler) =
            Self::create_blit_pipeline(&device, surface_format, &render_target);

        // Create static unit quad mesh for GPU-instanced rendering
        // Format: POS_UV_COLOR (format bits: UV | COLOR = 0b011 = 3)
        let unit_quad_format = FORMAT_UV | FORMAT_COLOR;

        let unit_quad_vertices: Vec<f32> = vec![
            // pos_x, pos_y, pos_z, uv_u, uv_v, color_r, color_g, color_b
            -0.5, -0.5, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0, // Bottom-left
            0.5, -0.5, 0.0, 1.0, 0.0, 1.0, 1.0, 1.0, // Bottom-right
            0.5, 0.5, 0.0, 1.0, 1.0, 1.0, 1.0, 1.0, // Top-right
            -0.5, 0.5, 0.0, 0.0, 1.0, 1.0, 1.0, 1.0, // Top-left
        ];

        let unit_quad_indices: Vec<u16> = vec![
            0, 1, 2, // First triangle
            0, 2, 3, // Second triangle
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
            render_state: super::RenderState::default(),
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
            texture_bind_groups: hashbrown::HashMap::new(),
            current_frame: None,
            current_view: None,
            buffer_manager,
            command_buffer: super::VirtualRenderPass::new(),
            pipeline_cache: super::pipeline::PipelineCache::new(),
            current_render_mode: 0,      // Default to Mode 0 (Unlit)
            current_resolution_index: 1, // 960×540 (default)
            scale_mode: emberware_core::app::config::ScaleMode::default(), // Stretch by default
            unit_quad_format,
            unit_quad_base_vertex,
            unit_quad_first_index,
            screen_dims_buffer,
        };

        Ok(graphics)
    }

    /// Create a new ZGraphics instance (blocking version for sync contexts)
    pub fn new_blocking(window: Arc<Window>) -> Result<Self> {
        pollster::block_on(Self::new(window))
    }

    /// Create depth texture and view
    pub(super) fn create_depth_texture(
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
    pub(super) fn create_render_target(
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
    pub(super) fn create_blit_pipeline(
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
}
