//! EPU GPU pipeline creation.
//!
//! This module contains helper functions for creating wgpu pipelines and
//! bind groups used by the EPU runtime.

use super::settings::MAX_ACTIVE_ENVS;
use super::settings::MAX_ENV_STATES;
use super::shaders::{
    EPU_BOUNDS, EPU_COMMON, EPU_COMPUTE_BLUR, EPU_COMPUTE_ENV, EPU_COMPUTE_IRRAD, EPU_FEATURES,
};
use super::types::{EpuSh9, FrameUniforms, GpuEnvironmentState, IrradUniforms};

/// Create GPU buffers for environment states, active IDs, and frame uniforms.
pub(super) fn create_buffers(device: &wgpu::Device) -> (wgpu::Buffer, wgpu::Buffer, wgpu::Buffer) {
    // Environment states buffer (256 environments x 128 bytes = 32KB)
    let env_states_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("EPU Environment States"),
        size: (MAX_ENV_STATES as usize * std::mem::size_of::<GpuEnvironmentState>()) as u64,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    // Active environment IDs buffer (32 u32s)
    let active_env_ids_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("EPU Active Env IDs"),
        size: (MAX_ACTIVE_ENVS as usize * std::mem::size_of::<u32>()) as u64,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    // Frame uniforms buffer (16 bytes)
    let frame_uniforms_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("EPU Frame Uniforms"),
        size: std::mem::size_of::<FrameUniforms>() as u64,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    (
        env_states_buffer,
        active_env_ids_buffer,
        frame_uniforms_buffer,
    )
}

/// Create the radiance texture with mip pyramid and associated views.
pub(super) fn create_radiance_texture(
    device: &wgpu::Device,
    map_size: u32,
    mip_level_count: u32,
    initial_layers: u32,
) -> (wgpu::Texture, wgpu::TextureView, Vec<wgpu::TextureView>) {
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("EPU EnvRadiance"),
        size: wgpu::Extent3d {
            width: map_size,
            height: map_size,
            depth_or_array_layers: initial_layers,
        },
        mip_level_count,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba16Float,
        usage: wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    });

    // Full view (all mips) for sampling in render
    let full_view = texture.create_view(&wgpu::TextureViewDescriptor {
        label: Some("EPU EnvRadiance View"),
        dimension: Some(wgpu::TextureViewDimension::D2Array),
        ..Default::default()
    });

    // Per-mip views (single mip) for compute passes
    let mip_views: Vec<wgpu::TextureView> = (0..mip_level_count)
        .map(|mip| {
            texture.create_view(&wgpu::TextureViewDescriptor {
                dimension: Some(wgpu::TextureViewDimension::D2Array),
                base_mip_level: mip,
                mip_level_count: Some(1),
                ..Default::default()
            })
        })
        .collect();

    (texture, full_view, mip_views)
}

/// Create the main environment build compute pipeline.
pub(super) fn create_main_pipeline(
    device: &wgpu::Device,
    env_states_buffer: &wgpu::Buffer,
    active_env_ids_buffer: &wgpu::Buffer,
    frame_uniforms_buffer: &wgpu::Buffer,
    mip0_view: &wgpu::TextureView,
) -> (
    wgpu::ComputePipeline,
    wgpu::BindGroupLayout,
    wgpu::BindGroup,
) {
    // Concatenate shader sources
    let shader_source = format!("{EPU_COMMON}\n{EPU_BOUNDS}\n{EPU_FEATURES}\n{EPU_COMPUTE_ENV}");

    let shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("EPU Compute Env Shader"),
        source: wgpu::ShaderSource::Wgsl(shader_source.into()),
    });

    // Bind group layout (v2: no palette buffer, bindings 0-3)
    let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("EPU Bind Group Layout"),
        entries: &[
            // @binding(0) epu_states: storage buffer of PackedEnvironmentState
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            // @binding(1) epu_active_env_ids: storage buffer of u32
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            // @binding(2) epu_frame: uniform buffer
            wgpu::BindGroupLayoutEntry {
                binding: 2,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            // @binding(3) epu_out_sharp: storage texture 2d array (write)
            wgpu::BindGroupLayoutEntry {
                binding: 3,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::StorageTexture {
                    access: wgpu::StorageTextureAccess::WriteOnly,
                    format: wgpu::TextureFormat::Rgba16Float,
                    view_dimension: wgpu::TextureViewDimension::D2Array,
                },
                count: None,
            },
        ],
    });

    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("EPU Bind Group"),
        layout: &bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: env_states_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: active_env_ids_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: frame_uniforms_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 3,
                resource: wgpu::BindingResource::TextureView(mip0_view),
            },
        ],
    });

    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("EPU Pipeline Layout"),
        bind_group_layouts: &[&bind_group_layout],
        push_constant_ranges: &[],
    });

    let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some("EPU Compute Env Pipeline"),
        layout: Some(&pipeline_layout),
        module: &shader_module,
        entry_point: Some("epu_build"),
        compilation_options: Default::default(),
        cache: None,
    });

    (pipeline, bind_group_layout, bind_group)
}

/// Create the mip downsample pipeline.
pub(super) fn create_mip_pipeline(
    device: &wgpu::Device,
) -> (wgpu::ComputePipeline, wgpu::BindGroupLayout, wgpu::Sampler) {
    // Sampler for compute sampling
    let compute_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some("EPU Compute Sampler"),
        address_mode_u: wgpu::AddressMode::ClampToEdge,
        address_mode_v: wgpu::AddressMode::ClampToEdge,
        address_mode_w: wgpu::AddressMode::ClampToEdge,
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        mipmap_filter: wgpu::FilterMode::Nearest,
        ..Default::default()
    });

    let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("EPU Mip Bind Group Layout"),
        entries: &[
            // @binding(2) epu_active_env_ids: storage buffer of u32
            wgpu::BindGroupLayoutEntry {
                binding: 2,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            // @binding(4) epu_in: texture_2d_array<f32>
            wgpu::BindGroupLayoutEntry {
                binding: 4,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    view_dimension: wgpu::TextureViewDimension::D2Array,
                    multisampled: false,
                },
                count: None,
            },
            // @binding(5) epu_out: texture_storage_2d_array<rgba16float, write>
            wgpu::BindGroupLayoutEntry {
                binding: 5,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::StorageTexture {
                    access: wgpu::StorageTextureAccess::WriteOnly,
                    format: wgpu::TextureFormat::Rgba16Float,
                    view_dimension: wgpu::TextureViewDimension::D2Array,
                },
                count: None,
            },
        ],
    });

    let shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("EPU Compute Mip Shader"),
        source: wgpu::ShaderSource::Wgsl(EPU_COMPUTE_BLUR.into()),
    });

    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("EPU Mip Pipeline Layout"),
        bind_group_layouts: &[&bind_group_layout],
        push_constant_ranges: &[],
    });

    let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some("EPU Compute Mip Pipeline"),
        layout: Some(&pipeline_layout),
        module: &shader_module,
        entry_point: Some("epu_downsample_mip"),
        compilation_options: Default::default(),
        cache: None,
    });

    (pipeline, bind_group_layout, compute_sampler)
}

/// Create the irradiance extraction pipeline.
pub(super) fn create_irrad_pipeline(
    device: &wgpu::Device,
) -> (
    wgpu::Buffer,
    wgpu::ComputePipeline,
    wgpu::Buffer,
    wgpu::BindGroupLayout,
) {
    // SH9 storage buffer (MAX_ENV_STATES * 144 bytes)
    let sh9_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("EPU SH9"),
        size: (MAX_ENV_STATES as usize * std::mem::size_of::<EpuSh9>()) as u64,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
        mapped_at_creation: false,
    });

    // Irrad uniforms buffer (16 bytes)
    let irrad_uniforms_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("EPU Irrad Uniforms"),
        size: std::mem::size_of::<IrradUniforms>() as u64,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("EPU Irrad Bind Group Layout"),
        entries: &[
            // @binding(2) epu_active_env_ids: storage buffer of u32
            wgpu::BindGroupLayoutEntry {
                binding: 2,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            // @binding(4) epu_blurred: texture_2d_array<f32> (coarse EnvRadiance mip)
            wgpu::BindGroupLayoutEntry {
                binding: 4,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    view_dimension: wgpu::TextureViewDimension::D2Array,
                    multisampled: false,
                },
                count: None,
            },
            // @binding(5) epu_samp: sampler
            wgpu::BindGroupLayoutEntry {
                binding: 5,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            },
            // @binding(6) epu_sh9: storage buffer (read_write)
            wgpu::BindGroupLayoutEntry {
                binding: 6,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: false },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            // @binding(7) epu_irrad: uniform buffer
            wgpu::BindGroupLayoutEntry {
                binding: 7,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
        ],
    });

    let shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("EPU Compute Irrad Shader"),
        source: wgpu::ShaderSource::Wgsl(EPU_COMPUTE_IRRAD.into()),
    });

    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("EPU Irrad Pipeline Layout"),
        bind_group_layouts: &[&bind_group_layout],
        push_constant_ranges: &[],
    });

    let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some("EPU Compute Irrad Pipeline"),
        layout: Some(&pipeline_layout),
        module: &shader_module,
        entry_point: Some("epu_extract_sh9"),
        compilation_options: Default::default(),
        cache: None,
    });

    (
        sh9_buffer,
        pipeline,
        irrad_uniforms_buffer,
        bind_group_layout,
    )
}
