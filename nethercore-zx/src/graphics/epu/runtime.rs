//! EPU GPU Runtime (v2 - 128-bit instructions)
//!
//! This module provides the GPU infrastructure to execute EPU compute shaders
//! and produce EnvSharp and EnvLight0 octahedral maps.
//!
//! # Architecture
//!
//! The EPU runtime manages:
//! - GPU buffers for environment states and frame uniforms
//! - Storage textures for EnvSharp and EnvLight0 output
//! - Compute pipeline and bind groups
//!
//! # v2 Changes
//!
//! EPU v2 uses 128-bit instructions with embedded RGB24 colors. The palette
//! buffer has been removed - colors are now packed directly into the
//! instruction format.
//!
//! # Usage
//!
//! ```ignore
//! let epu_runtime = EpuRuntime::new(&device);
//! epu_runtime.build_env(&device, &queue, &mut encoder, &config, time);
//! ```

use super::EpuConfig;

// EPU shader sources - included directly since build_support is not exposed in the crate.
const EPU_COMMON: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/shaders/epu/epu_common.wgsl"
));
const EPU_BOUNDS: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/shaders/epu/epu_bounds.wgsl"
));
const EPU_FEATURES: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/shaders/epu/epu_features.wgsl"
));
const EPU_COMPUTE_ENV: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/shaders/epu/epu_compute_env.wgsl"
));
const EPU_COMPUTE_BLUR: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/shaders/epu/epu_compute_blur.wgsl"
));
const EPU_COMPUTE_IRRAD: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/shaders/epu/epu_compute_irrad.wgsl"
));

/// Output map size in texels (64x64 octahedral).
pub const EPU_MAP_SIZE: u32 = 64;

/// Maximum number of environment states that can be processed.
pub const MAX_ENV_STATES: u32 = 256;

/// Maximum number of active environments per dispatch.
pub const MAX_ACTIVE_ENVS: u32 = 32;

/// Cache entry for dirty-state tracking of environment configurations.
///
/// Each entry stores the hash and time-dependency flag for an environment slot,
/// allowing the runtime to skip rebuilding unchanged static environments.
#[derive(Clone, Copy, Default)]
struct EpuCacheEntry {
    /// Hash of the EpuConfig used to detect changes
    state_hash: u64,
    /// Whether this config uses time-based animation
    time_dependent: bool,
    /// Whether this cache entry contains valid data
    valid: bool,
}

/// Thread-safe cache storage using RefCell for interior mutability.
///
/// This allows `build_envs()` to maintain its `&self` signature while still
/// updating the cache state.
struct EpuCache {
    entries: std::cell::RefCell<Vec<EpuCacheEntry>>,
    current_frame: std::cell::Cell<u64>,
}

impl EpuCache {
    fn new() -> Self {
        Self {
            entries: std::cell::RefCell::new(vec![
                EpuCacheEntry::default();
                MAX_ENV_STATES as usize
            ]),
            current_frame: std::cell::Cell::new(0),
        }
    }

    fn advance_frame(&self) {
        self.current_frame
            .set(self.current_frame.get().wrapping_add(1));
    }

    fn current_frame(&self) -> u64 {
        self.current_frame.get()
    }

    fn invalidate(&self, env_id: u32) {
        if let Some(entry) = self.entries.borrow_mut().get_mut(env_id as usize) {
            entry.valid = false;
        }
    }

    fn invalidate_all(&self) {
        for entry in self.entries.borrow_mut().iter_mut() {
            entry.valid = false;
        }
    }

    /// Check if an environment needs rebuilding and update cache.
    ///
    /// Returns `true` if the environment needs to be rebuilt.
    fn needs_rebuild(&self, env_id: u32, config: &EpuConfig) -> bool {
        let hash = config.state_hash();
        let time_dependent = config.is_time_dependent();
        let mut entries = self.entries.borrow_mut();

        if let Some(entry) = entries.get_mut(env_id as usize) {
            // Check if we can skip this environment
            if entry.valid && entry.state_hash == hash && !entry.time_dependent {
                // Cache hit: same config, not time-dependent
                return false;
            }

            // Cache miss or time-dependent: update cache and rebuild
            entry.state_hash = hash;
            entry.time_dependent = time_dependent;
            entry.valid = true;
        }

        true
    }
}

/// Result of collecting active environments with deduplication and capping.
#[derive(Debug, Clone)]
pub struct ActiveEnvList {
    /// Deduplicated and capped list of unique environment IDs.
    pub unique_ids: Vec<u32>,
    /// Maps original env_id to its slot index in `unique_ids`, or 0 for fallback.
    pub slot_map: std::collections::HashMap<u32, u32>,
    /// Number of environments that were dropped due to cap overflow.
    pub overflow_count: usize,
}

/// Collects unique environment IDs, caps to MAX_ACTIVE_ENVS, logs warning in debug builds if overflow.
///
/// Returns an `ActiveEnvList` containing:
/// - `unique_ids`: The deduplicated and capped list of environment IDs
/// - `slot_map`: Maps each env_id to its slot index (0-31), or 0 for envs that exceeded the cap
/// - `overflow_count`: Number of environments that were dropped due to exceeding the cap
///
/// # Arguments
/// * `env_ids` - Slice of environment IDs (may contain duplicates)
///
/// # Example
/// ```ignore
/// let env_ids = &[5, 2, 5, 10, 2, 7];
/// let result = collect_active_envs(env_ids);
/// // result.unique_ids = [2, 5, 7, 10] (sorted, deduplicated)
/// // result.slot_map = {2: 0, 5: 1, 7: 2, 10: 3}
/// ```
pub fn collect_active_envs(env_ids: &[u32]) -> ActiveEnvList {
    // Deduplicate
    let mut unique: Vec<u32> = env_ids.to_vec();
    unique.sort_unstable();
    unique.dedup();

    // Track overflow before capping
    let overflow_count = unique.len().saturating_sub(MAX_ACTIVE_ENVS as usize);

    // Cap and log warning in debug builds
    if unique.len() > MAX_ACTIVE_ENVS as usize {
        #[cfg(debug_assertions)]
        eprintln!(
            "EPU: {} unique envs exceed cap of {}, falling back to env_id=0 for {} envs",
            unique.len(),
            MAX_ACTIVE_ENVS,
            overflow_count
        );
        unique.truncate(MAX_ACTIVE_ENVS as usize);
    }

    // Build mapping: env_id -> slot index
    let mut slot_map = std::collections::HashMap::new();
    for (slot, &env_id) in unique.iter().enumerate() {
        slot_map.insert(env_id, slot as u32);
    }
    // Note: Any env_id not in slot_map should use slot 0 as fallback.
    // The caller can check with: slot_map.get(&env_id).copied().unwrap_or(0)

    ActiveEnvList {
        unique_ids: unique,
        slot_map,
        overflow_count,
    }
}

/// Frame uniforms structure matching the WGSL `FrameUniforms` struct.
#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct FrameUniforms {
    time: f32,
    active_count: u32,
    map_size: u32,
    _pad0: u32,
}

/// Blur uniforms structure matching the WGSL `BlurUniforms` struct.
#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct BlurUniforms {
    active_count: u32,
    map_size: u32,
    blur_offset: f32,
    _pad0: u32,
}

/// Irradiance uniforms structure matching the WGSL `IrradUniforms` struct.
#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct IrradUniforms {
    active_count: u32,
    _pad0: u32,
    _pad1: u32,
    _pad2: u32,
}

/// Ambient cube storage for 6-direction diffuse irradiance samples.
///
/// Stores irradiance sampled from the most blurred environment map level
/// in 6 axis-aligned directions for efficient diffuse lighting.
#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct AmbientCube {
    /// Irradiance in +X direction
    pub pos_x: [f32; 3],
    _pad0: f32,
    /// Irradiance in -X direction
    pub neg_x: [f32; 3],
    _pad1: f32,
    /// Irradiance in +Y direction
    pub pos_y: [f32; 3],
    _pad2: f32,
    /// Irradiance in -Y direction
    pub neg_y: [f32; 3],
    _pad3: f32,
    /// Irradiance in +Z direction
    pub pos_z: [f32; 3],
    _pad4: f32,
    /// Irradiance in -Z direction
    pub neg_z: [f32; 3],
    _pad5: f32,
}

/// GPU representation of an EPU environment state (v2 128-bit format).
///
/// Each layer is 128 bits = 4 x u32 for GPU compatibility.
/// The shader expects `array<vec4u, 8>` where each vec4u represents a 128-bit instruction.
///
/// WGSL vec4u layout: [w0, w1, w2, w3] where:
/// - w0 = bits 31..0   (lo.lo)
/// - w1 = bits 63..32  (lo.hi)
/// - w2 = bits 95..64  (hi.lo)
/// - w3 = bits 127..96 (hi.hi)
#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct GpuEnvironmentState {
    /// 8 layers stored as [lo_lo, lo_hi, hi_lo, hi_hi] quadruplets
    /// Total: 8 layers x 4 u32 = 32 u32 = 128 bytes
    layers: [[u32; 4]; 8],
}

impl From<&EpuConfig> for GpuEnvironmentState {
    fn from(config: &EpuConfig) -> Self {
        // EpuConfig stores [hi, lo] where hi=bits 127..64, lo=bits 63..0
        // WGSL vec4u needs [w0, w1, w2, w3] = [lo_lo, lo_hi, hi_lo, hi_hi]
        let layers = config.layers.map(|[hi, lo]| {
            [
                (lo & 0xFFFF_FFFF) as u32, // w0 = lo bits 31..0
                (lo >> 32) as u32,         // w1 = lo bits 63..32
                (hi & 0xFFFF_FFFF) as u32, // w2 = hi bits 31..0 (overall bits 95..64)
                (hi >> 32) as u32,         // w3 = hi bits 63..32 (overall bits 127..96)
            ]
        });
        Self { layers }
    }
}

/// EPU GPU runtime for environment map generation (v2).
///
/// Manages GPU resources and compute pipeline for generating EnvSharp and EnvLight0
/// octahedral maps from EPU configurations.
///
/// v2: Palette buffer removed - colors are embedded in 128-bit instructions.
pub struct EpuRuntime {
    // GPU buffers
    env_states_buffer: wgpu::Buffer,
    active_env_ids_buffer: wgpu::Buffer,
    frame_uniforms_buffer: wgpu::Buffer,

    // Output textures (64x64 octahedral, 256 array layers)
    env_sharp_texture: wgpu::Texture,
    env_light0_texture: wgpu::Texture,
    env_light1_texture: wgpu::Texture,
    env_light2_texture: wgpu::Texture,

    // Texture views for binding
    env_sharp_view: wgpu::TextureView,
    env_light0_view: wgpu::TextureView,
    env_light1_view: wgpu::TextureView,
    env_light2_view: wgpu::TextureView,

    // Pipeline resources
    pipeline: wgpu::ComputePipeline,
    bind_group: wgpu::BindGroup,

    // Blur pipeline resources
    blur_pipeline: wgpu::ComputePipeline,
    blur_uniforms_buffer: wgpu::Buffer,
    blur_sampler: wgpu::Sampler,
    blur_bind_group_layout: wgpu::BindGroupLayout,

    // Irradiance extraction resources
    ambient_cubes_buffer: wgpu::Buffer,
    irrad_pipeline: wgpu::ComputePipeline,
    irrad_uniforms_buffer: wgpu::Buffer,
    irrad_bind_group_layout: wgpu::BindGroupLayout,

    // Dirty-state cache for skipping unchanged static environments
    cache: EpuCache,
}

impl EpuRuntime {
    /// Create a new EPU runtime with all GPU resources.
    ///
    /// # Arguments
    /// * `device` - The wgpu device to create resources on
    pub fn new(device: &wgpu::Device) -> Self {
        // Create environment states buffer (256 environments x 128 bytes = 32KB)
        // v2: Each environment is 128 bytes (8 layers x 16 bytes per 128-bit instruction)
        let env_states_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("EPU Environment States"),
            size: (MAX_ENV_STATES as usize * std::mem::size_of::<GpuEnvironmentState>()) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // Create active environment IDs buffer (32 u32s)
        let active_env_ids_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("EPU Active Env IDs"),
            size: (MAX_ACTIVE_ENVS as usize * std::mem::size_of::<u32>()) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // Create frame uniforms buffer (16 bytes)
        let frame_uniforms_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("EPU Frame Uniforms"),
            size: std::mem::size_of::<FrameUniforms>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // Create EnvSharp texture (64x64 RGBA16Float, 256 array layers)
        let env_sharp_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("EPU EnvSharp"),
            size: wgpu::Extent3d {
                width: EPU_MAP_SIZE,
                height: EPU_MAP_SIZE,
                depth_or_array_layers: MAX_ENV_STATES,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba16Float,
            usage: wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

        // Create EnvLight0 texture (64x64 RGBA16Float, 256 array layers)
        let env_light0_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("EPU EnvLight0"),
            size: wgpu::Extent3d {
                width: EPU_MAP_SIZE,
                height: EPU_MAP_SIZE,
                depth_or_array_layers: MAX_ENV_STATES,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba16Float,
            usage: wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

        // Create EnvLight1 texture (64x64 RGBA16Float, 256 array layers) - first blur level
        let env_light1_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("EPU EnvLight1"),
            size: wgpu::Extent3d {
                width: EPU_MAP_SIZE,
                height: EPU_MAP_SIZE,
                depth_or_array_layers: MAX_ENV_STATES,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba16Float,
            usage: wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

        // Create EnvLight2 texture (64x64 RGBA16Float, 256 array layers) - second blur level
        let env_light2_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("EPU EnvLight2"),
            size: wgpu::Extent3d {
                width: EPU_MAP_SIZE,
                height: EPU_MAP_SIZE,
                depth_or_array_layers: MAX_ENV_STATES,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba16Float,
            usage: wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

        // Create texture views
        let env_sharp_view = env_sharp_texture.create_view(&wgpu::TextureViewDescriptor {
            label: Some("EPU EnvSharp View"),
            dimension: Some(wgpu::TextureViewDimension::D2Array),
            ..Default::default()
        });

        let env_light0_view = env_light0_texture.create_view(&wgpu::TextureViewDescriptor {
            label: Some("EPU EnvLight0 View"),
            dimension: Some(wgpu::TextureViewDimension::D2Array),
            ..Default::default()
        });

        let env_light1_view = env_light1_texture.create_view(&wgpu::TextureViewDescriptor {
            label: Some("EPU EnvLight1 View"),
            dimension: Some(wgpu::TextureViewDimension::D2Array),
            ..Default::default()
        });

        let env_light2_view = env_light2_texture.create_view(&wgpu::TextureViewDescriptor {
            label: Some("EPU EnvLight2 View"),
            dimension: Some(wgpu::TextureViewDimension::D2Array),
            ..Default::default()
        });

        // Concatenate shader sources in order
        let shader_source =
            format!("{EPU_COMMON}\n{EPU_BOUNDS}\n{EPU_FEATURES}\n{EPU_COMPUTE_ENV}");

        // Create shader module
        let shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("EPU Compute Env Shader"),
            source: wgpu::ShaderSource::Wgsl(shader_source.into()),
        });

        // Create bind group layout (v2: no palette buffer, bindings 0-4)
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
                // @binding(4) epu_out_light0: storage texture 2d array (write)
                wgpu::BindGroupLayoutEntry {
                    binding: 4,
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

        // Create bind group (v2: no palette, bindings 0-4)
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
                    resource: wgpu::BindingResource::TextureView(&env_sharp_view),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: wgpu::BindingResource::TextureView(&env_light0_view),
                },
            ],
        });

        // Create pipeline layout
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("EPU Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        // Create compute pipeline
        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("EPU Compute Env Pipeline"),
            layout: Some(&pipeline_layout),
            module: &shader_module,
            entry_point: Some("epu_build"),
            compilation_options: Default::default(),
            cache: None,
        });

        // =====================================================================
        // Blur pipeline resources
        // =====================================================================

        // Create blur uniforms buffer (16 bytes)
        let blur_uniforms_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("EPU Blur Uniforms"),
            size: std::mem::size_of::<BlurUniforms>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // Create linear filtering sampler for blur
        let blur_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("EPU Blur Sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        // Create blur bind group layout matching epu_compute_blur.wgsl bindings
        let blur_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("EPU Blur Bind Group Layout"),
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
                    // @binding(6) epu_samp: sampler
                    wgpu::BindGroupLayoutEntry {
                        binding: 6,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                    // @binding(7) epu_blur: uniform buffer
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

        // Create blur shader module
        let blur_shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("EPU Compute Blur Shader"),
            source: wgpu::ShaderSource::Wgsl(EPU_COMPUTE_BLUR.into()),
        });

        // Create blur pipeline layout
        let blur_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("EPU Blur Pipeline Layout"),
            bind_group_layouts: &[&blur_bind_group_layout],
            push_constant_ranges: &[],
        });

        // Create blur compute pipeline
        let blur_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("EPU Compute Blur Pipeline"),
            layout: Some(&blur_pipeline_layout),
            module: &blur_shader_module,
            entry_point: Some("epu_kawase_blur"),
            compilation_options: Default::default(),
            cache: None,
        });

        // =====================================================================
        // Irradiance extraction pipeline resources
        // =====================================================================

        // Create ambient cubes storage buffer (MAX_ENV_STATES * 96 bytes)
        let ambient_cubes_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("EPU Ambient Cubes"),
            size: (MAX_ENV_STATES as usize * std::mem::size_of::<AmbientCube>()) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });

        // Create irrad uniforms buffer (16 bytes)
        let irrad_uniforms_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("EPU Irrad Uniforms"),
            size: std::mem::size_of::<IrradUniforms>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // Create irrad bind group layout matching epu_compute_irrad.wgsl bindings
        let irrad_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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
                    // @binding(4) epu_blurred: texture_2d_array<f32> (EnvLight2)
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
                    // @binding(6) epu_ambient: storage buffer (read_write)
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

        // Create irrad shader module
        let irrad_shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("EPU Compute Irrad Shader"),
            source: wgpu::ShaderSource::Wgsl(EPU_COMPUTE_IRRAD.into()),
        });

        // Create irrad pipeline layout
        let irrad_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("EPU Irrad Pipeline Layout"),
                bind_group_layouts: &[&irrad_bind_group_layout],
                push_constant_ranges: &[],
            });

        // Create irrad compute pipeline
        let irrad_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("EPU Compute Irrad Pipeline"),
            layout: Some(&irrad_pipeline_layout),
            module: &irrad_shader_module,
            entry_point: Some("epu_extract_ambient"),
            compilation_options: Default::default(),
            cache: None,
        });

        Self {
            env_states_buffer,
            active_env_ids_buffer,
            frame_uniforms_buffer,
            env_sharp_texture,
            env_light0_texture,
            env_light1_texture,
            env_light2_texture,
            env_sharp_view,
            env_light0_view,
            env_light1_view,
            env_light2_view,
            pipeline,
            bind_group,
            blur_pipeline,
            blur_uniforms_buffer,
            blur_sampler,
            blur_bind_group_layout,
            ambient_cubes_buffer,
            irrad_pipeline,
            irrad_uniforms_buffer,
            irrad_bind_group_layout,
            cache: EpuCache::new(),
        }
    }

    /// Advance to the next frame for cache purposes.
    ///
    /// This should be called once per frame before `build_envs()` to ensure
    /// proper cache invalidation for time-dependent environments.
    pub fn advance_frame(&self) {
        self.cache.advance_frame();
    }

    /// Get the current frame counter.
    ///
    /// This is incremented by `advance_frame()` each frame.
    pub fn current_frame(&self) -> u64 {
        self.cache.current_frame()
    }

    /// Invalidate the cache entry for a specific environment ID.
    ///
    /// This forces the environment to be rebuilt on the next `build_envs()` call.
    pub fn invalidate_cache(&self, env_id: u32) {
        self.cache.invalidate(env_id);
    }

    /// Invalidate all cache entries.
    ///
    /// This forces all environments to be rebuilt on the next `build_envs()` call.
    pub fn invalidate_all_caches(&self) {
        self.cache.invalidate_all();
    }

    /// Dispatch a blur pass from input texture to output texture.
    ///
    /// This creates a dynamic bind group with the specified input/output textures
    /// and dispatches the Kawase blur compute shader.
    ///
    /// # Arguments
    /// * `device` - The wgpu device for creating bind groups
    /// * `queue` - The wgpu queue for buffer writes
    /// * `encoder` - Command encoder to record compute pass
    /// * `input_view` - Input texture view to sample from
    /// * `output_view` - Output storage texture view to write to
    /// * `blur_offset` - Blur kernel offset (1.0 for first pass, 2.0 for second)
    /// * `active_count` - Number of active environments to process
    #[allow(clippy::too_many_arguments)]
    fn dispatch_blur_pass(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        input_view: &wgpu::TextureView,
        output_view: &wgpu::TextureView,
        blur_offset: f32,
        active_count: u32,
    ) {
        // Update blur uniforms
        let blur_uniforms = BlurUniforms {
            active_count,
            map_size: EPU_MAP_SIZE,
            blur_offset,
            _pad0: 0,
        };
        queue.write_buffer(
            &self.blur_uniforms_buffer,
            0,
            bytemuck::cast_slice(&[blur_uniforms]),
        );

        // Create dynamic bind group with input/output textures
        let blur_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("EPU Blur Bind Group"),
            layout: &self.blur_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: self.active_env_ids_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: wgpu::BindingResource::TextureView(input_view),
                },
                wgpu::BindGroupEntry {
                    binding: 5,
                    resource: wgpu::BindingResource::TextureView(output_view),
                },
                wgpu::BindGroupEntry {
                    binding: 6,
                    resource: wgpu::BindingResource::Sampler(&self.blur_sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 7,
                    resource: self.blur_uniforms_buffer.as_entire_binding(),
                },
            ],
        });

        // Create blur compute pass
        let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("EPU Blur Pass"),
            timestamp_writes: None,
        });

        compute_pass.set_pipeline(&self.blur_pipeline);
        compute_pass.set_bind_group(0, &blur_bind_group, &[]);

        // Dispatch compute (8x8 workgroups)
        let workgroups_x = EPU_MAP_SIZE.div_ceil(8);
        let workgroups_y = EPU_MAP_SIZE.div_ceil(8);
        compute_pass.dispatch_workgroups(workgroups_x, workgroups_y, active_count);
    }

    /// Dispatch the irradiance extraction pass.
    ///
    /// This extracts 6-direction ambient cube samples from the most blurred
    /// light level (EnvLight2) and stores them in the ambient cubes buffer.
    ///
    /// # Arguments
    /// * `device` - The wgpu device for creating bind groups
    /// * `queue` - The wgpu queue for buffer writes
    /// * `encoder` - Command encoder to record compute pass
    /// * `active_count` - Number of active environments to process
    fn dispatch_irrad_pass(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        active_count: u32,
    ) {
        // Update irrad uniforms
        let irrad_uniforms = IrradUniforms {
            active_count,
            _pad0: 0,
            _pad1: 0,
            _pad2: 0,
        };
        queue.write_buffer(
            &self.irrad_uniforms_buffer,
            0,
            bytemuck::cast_slice(&[irrad_uniforms]),
        );

        // Create irrad bind group
        let irrad_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("EPU Irrad Bind Group"),
            layout: &self.irrad_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: self.active_env_ids_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: wgpu::BindingResource::TextureView(&self.env_light2_view),
                },
                wgpu::BindGroupEntry {
                    binding: 5,
                    resource: wgpu::BindingResource::Sampler(&self.blur_sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 6,
                    resource: self.ambient_cubes_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 7,
                    resource: self.irrad_uniforms_buffer.as_entire_binding(),
                },
            ],
        });

        // Create irrad compute pass
        let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("EPU Irrad Pass"),
            timestamp_writes: None,
        });

        compute_pass.set_pipeline(&self.irrad_pipeline);
        compute_pass.set_bind_group(0, &irrad_bind_group, &[]);

        // Dispatch compute (workgroup size 1,1,1, one per active env)
        compute_pass.dispatch_workgroups(1, 1, active_count);
    }

    /// Build environment maps for a single environment configuration.
    ///
    /// This dispatches the compute shader to generate EnvSharp, EnvLight0,
    /// EnvLight1, and EnvLight2 octahedral maps for the given configuration.
    /// The Light1 and Light2 textures are generated via Kawase blur pyramid.
    ///
    /// # Arguments
    /// * `device` - The wgpu device for creating bind groups
    /// * `queue` - The wgpu queue for buffer writes
    /// * `encoder` - Command encoder to record compute pass
    /// * `config` - The EPU configuration to evaluate
    /// * `time` - Current time for animation (in seconds)
    pub fn build_env(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        config: &EpuConfig,
        time: f32,
    ) {
        // Convert EpuConfig to GPU format and upload to slot 0
        let gpu_state = GpuEnvironmentState::from(config);
        queue.write_buffer(
            &self.env_states_buffer,
            0,
            bytemuck::cast_slice(&[gpu_state]),
        );

        // Upload frame uniforms
        let frame_uniforms = FrameUniforms {
            time,
            active_count: 1,
            map_size: EPU_MAP_SIZE,
            _pad0: 0,
        };
        queue.write_buffer(
            &self.frame_uniforms_buffer,
            0,
            bytemuck::cast_slice(&[frame_uniforms]),
        );

        // Set active_env_ids = [0]
        let active_ids: [u32; 1] = [0];
        queue.write_buffer(
            &self.active_env_ids_buffer,
            0,
            bytemuck::cast_slice(&active_ids),
        );

        // Create compute pass for env evaluation
        {
            let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("EPU Compute Env Pass"),
                timestamp_writes: None,
            });

            compute_pass.set_pipeline(&self.pipeline);
            compute_pass.set_bind_group(0, &self.bind_group, &[]);

            // Dispatch compute (8x8 workgroup, ceil(64/8)=8 workgroups per axis)
            // For a single environment (active_count=1), we dispatch (8, 8, 1)
            let workgroups_x = EPU_MAP_SIZE.div_ceil(8);
            let workgroups_y = EPU_MAP_SIZE.div_ceil(8);
            let workgroups_z = 1; // Single environment
            compute_pass.dispatch_workgroups(workgroups_x, workgroups_y, workgroups_z);
        }

        // Blur pyramid: Light0 -> Light1 (offset 1.0) -> Light2 (offset 2.0)
        self.dispatch_blur_pass(
            device,
            queue,
            encoder,
            &self.env_light0_view,
            &self.env_light1_view,
            1.0,
            1,
        );
        self.dispatch_blur_pass(
            device,
            queue,
            encoder,
            &self.env_light1_view,
            &self.env_light2_view,
            2.0,
            1,
        );

        // Extract ambient cube irradiance from EnvLight2
        self.dispatch_irrad_pass(device, queue, encoder, 1);
    }

    /// Build environment maps for multiple environments.
    ///
    /// This dispatches the compute shader to generate EnvSharp, EnvLight0,
    /// EnvLight1, and EnvLight2 octahedral maps for the given configurations.
    /// The Light1 and Light2 textures are generated via Kawase blur pyramid.
    ///
    /// Uses dirty-state caching to skip rebuilding unchanged static environments.
    /// Call `advance_frame()` once per frame before this method to ensure proper
    /// cache behavior for time-dependent environments.
    ///
    /// # Arguments
    /// * `device` - The wgpu device for creating bind groups
    /// * `queue` - The wgpu queue for buffer writes
    /// * `encoder` - Command encoder to record compute pass
    /// * `configs` - Slice of (env_id, config) pairs to evaluate
    /// * `time` - Current time for animation (in seconds)
    pub fn build_envs(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        configs: &[(u32, &EpuConfig)],
        time: f32,
    ) {
        if configs.is_empty() {
            return;
        }

        // Filter configs to only those that need rebuilding (cache miss or time-dependent)
        let dirty_configs: Vec<(u32, &EpuConfig)> = configs
            .iter()
            .take(MAX_ACTIVE_ENVS as usize)
            .filter(|(env_id, config)| self.cache.needs_rebuild(*env_id, config))
            .copied()
            .collect();

        // Early exit if all environments are cached
        if dirty_configs.is_empty() {
            return;
        }

        let active_count = dirty_configs.len() as u32;

        // Upload each dirty config to its corresponding slot
        for (env_id, config) in &dirty_configs {
            let gpu_state = GpuEnvironmentState::from(*config);
            let offset = (*env_id as usize) * std::mem::size_of::<GpuEnvironmentState>();
            queue.write_buffer(
                &self.env_states_buffer,
                offset as u64,
                bytemuck::cast_slice(&[gpu_state]),
            );
        }

        // Upload frame uniforms
        let frame_uniforms = FrameUniforms {
            time,
            active_count,
            map_size: EPU_MAP_SIZE,
            _pad0: 0,
        };
        queue.write_buffer(
            &self.frame_uniforms_buffer,
            0,
            bytemuck::cast_slice(&[frame_uniforms]),
        );

        // Upload active environment IDs (only dirty ones)
        let active_ids: Vec<u32> = dirty_configs.iter().map(|(id, _)| *id).collect();
        queue.write_buffer(
            &self.active_env_ids_buffer,
            0,
            bytemuck::cast_slice(&active_ids),
        );

        // Create compute pass for env evaluation
        {
            let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("EPU Compute Env Pass (Multi)"),
                timestamp_writes: None,
            });

            compute_pass.set_pipeline(&self.pipeline);
            compute_pass.set_bind_group(0, &self.bind_group, &[]);

            // Dispatch compute with z = active_count (only dirty envs)
            let workgroups_x = EPU_MAP_SIZE.div_ceil(8);
            let workgroups_y = EPU_MAP_SIZE.div_ceil(8);
            compute_pass.dispatch_workgroups(workgroups_x, workgroups_y, active_count);
        }

        // Blur pyramid: Light0 -> Light1 (offset 1.0) -> Light2 (offset 2.0)
        self.dispatch_blur_pass(
            device,
            queue,
            encoder,
            &self.env_light0_view,
            &self.env_light1_view,
            1.0,
            active_count,
        );
        self.dispatch_blur_pass(
            device,
            queue,
            encoder,
            &self.env_light1_view,
            &self.env_light2_view,
            2.0,
            active_count,
        );

        // Extract ambient cube irradiance from EnvLight2
        self.dispatch_irrad_pass(device, queue, encoder, active_count);
    }

    /// Get a reference to the EnvSharp texture for sampling.
    pub fn env_sharp_texture(&self) -> &wgpu::Texture {
        &self.env_sharp_texture
    }

    /// Get a reference to the EnvLight0 texture for sampling.
    pub fn env_light0_texture(&self) -> &wgpu::Texture {
        &self.env_light0_texture
    }

    /// Get the EnvSharp texture view for binding to render pipelines.
    pub fn env_sharp_view(&self) -> &wgpu::TextureView {
        &self.env_sharp_view
    }

    /// Get the EnvLight0 texture view for binding to render pipelines.
    pub fn env_light0_view(&self) -> &wgpu::TextureView {
        &self.env_light0_view
    }

    /// Get a reference to the EnvLight1 texture for sampling.
    pub fn env_light1_texture(&self) -> &wgpu::Texture {
        &self.env_light1_texture
    }

    /// Get the EnvLight1 texture view for binding to render pipelines.
    pub fn env_light1_view(&self) -> &wgpu::TextureView {
        &self.env_light1_view
    }

    /// Get a reference to the EnvLight2 texture for sampling.
    pub fn env_light2_texture(&self) -> &wgpu::Texture {
        &self.env_light2_texture
    }

    /// Get the EnvLight2 texture view for binding to render pipelines.
    pub fn env_light2_view(&self) -> &wgpu::TextureView {
        &self.env_light2_view
    }

    /// Get a reference to the ambient cubes storage buffer.
    ///
    /// This buffer contains the 6-direction ambient cube irradiance samples
    /// extracted from EnvLight2 for each environment.
    pub fn ambient_cubes_buffer(&self) -> &wgpu::Buffer {
        &self.ambient_cubes_buffer
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_frame_uniforms_size() {
        // FrameUniforms must be exactly 16 bytes (4 x u32/f32)
        assert_eq!(
            std::mem::size_of::<FrameUniforms>(),
            16,
            "FrameUniforms must be 16 bytes"
        );
    }

    #[test]
    fn test_blur_uniforms_size() {
        // BlurUniforms must be exactly 16 bytes (4 x u32/f32)
        assert_eq!(
            std::mem::size_of::<BlurUniforms>(),
            16,
            "BlurUniforms must be 16 bytes"
        );
    }

    #[test]
    fn test_gpu_environment_state_size() {
        // GpuEnvironmentState must be exactly 128 bytes (8 layers x 16 bytes)
        assert_eq!(
            std::mem::size_of::<GpuEnvironmentState>(),
            128,
            "GpuEnvironmentState must be 128 bytes"
        );
    }

    #[test]
    fn test_gpu_environment_state_conversion() {
        let config = EpuConfig {
            layers: [
                [0x1234_5678_9ABC_DEF0, 0xFEDC_BA98_7654_3210], // layer 0: [hi, lo]
                [0xAAAA_BBBB_CCCC_DDDD, 0x1111_2222_3333_4444], // layer 1: [hi, lo]
                [0, 0],
                [0, 0],
                [0, 0],
                [0, 0],
                [0, 0],
                [0, 0],
            ],
        };

        let gpu_state = GpuEnvironmentState::from(&config);

        // Check first layer [hi, lo] -> [lo_lo, lo_hi, hi_lo, hi_hi]
        // hi = 0x1234_5678_9ABC_DEF0 -> hi_hi=0x1234_5678, hi_lo=0x9ABC_DEF0
        // lo = 0xFEDC_BA98_7654_3210 -> lo_hi=0xFEDC_BA98, lo_lo=0x7654_3210
        assert_eq!(gpu_state.layers[0][0], 0x7654_3210); // w0 = lo_lo
        assert_eq!(gpu_state.layers[0][1], 0xFEDC_BA98); // w1 = lo_hi
        assert_eq!(gpu_state.layers[0][2], 0x9ABC_DEF0); // w2 = hi_lo
        assert_eq!(gpu_state.layers[0][3], 0x1234_5678); // w3 = hi_hi

        // Check second layer
        // hi = 0xAAAA_BBBB_CCCC_DDDD -> hi_hi=0xAAAA_BBBB, hi_lo=0xCCCC_DDDD
        // lo = 0x1111_2222_3333_4444 -> lo_hi=0x1111_2222, lo_lo=0x3333_4444
        assert_eq!(gpu_state.layers[1][0], 0x3333_4444); // w0 = lo_lo
        assert_eq!(gpu_state.layers[1][1], 0x1111_2222); // w1 = lo_hi
        assert_eq!(gpu_state.layers[1][2], 0xCCCC_DDDD); // w2 = hi_lo
        assert_eq!(gpu_state.layers[1][3], 0xAAAA_BBBB); // w3 = hi_hi

        // Rest should be zero
        for i in 2..8 {
            assert_eq!(gpu_state.layers[i], [0, 0, 0, 0]);
        }
    }

    #[test]
    fn test_irrad_uniforms_size() {
        // IrradUniforms must be exactly 16 bytes (4 x u32)
        assert_eq!(
            std::mem::size_of::<IrradUniforms>(),
            16,
            "IrradUniforms must be 16 bytes"
        );
    }

    #[test]
    fn test_ambient_cube_size() {
        // AmbientCube must be exactly 96 bytes (6 directions x 16 bytes each)
        assert_eq!(
            std::mem::size_of::<AmbientCube>(),
            96,
            "AmbientCube must be 96 bytes"
        );
    }

    #[test]
    fn test_collect_active_envs_deduplication() {
        use super::collect_active_envs;

        // Input with duplicates
        let env_ids = &[5, 2, 5, 10, 2, 7, 10, 5];
        let result = collect_active_envs(env_ids);

        // Should be sorted and deduplicated
        assert_eq!(result.unique_ids, vec![2, 5, 7, 10]);
        assert_eq!(result.overflow_count, 0);

        // Check slot mapping
        assert_eq!(result.slot_map.get(&2), Some(&0));
        assert_eq!(result.slot_map.get(&5), Some(&1));
        assert_eq!(result.slot_map.get(&7), Some(&2));
        assert_eq!(result.slot_map.get(&10), Some(&3));
    }

    #[test]
    fn test_collect_active_envs_capping() {
        use super::{MAX_ACTIVE_ENVS, collect_active_envs};

        // Create more than MAX_ACTIVE_ENVS unique IDs (0..40)
        let env_ids: Vec<u32> = (0..40).collect();
        let result = collect_active_envs(&env_ids);

        // Should be capped to MAX_ACTIVE_ENVS
        assert_eq!(result.unique_ids.len(), MAX_ACTIVE_ENVS as usize);
        assert_eq!(result.overflow_count, 40 - MAX_ACTIVE_ENVS as usize);

        // IDs should be sorted, so 0..31 should be kept
        for i in 0..MAX_ACTIVE_ENVS {
            assert!(result.unique_ids.contains(&i));
            assert_eq!(result.slot_map.get(&i), Some(&i));
        }

        // IDs 32..39 should NOT be in the mapping
        for i in MAX_ACTIVE_ENVS..40 {
            assert!(!result.slot_map.contains_key(&i));
        }
    }

    #[test]
    fn test_collect_active_envs_fallback_mapping() {
        use super::collect_active_envs;

        // Simple case with a few IDs
        let env_ids = &[100, 50, 25];
        let result = collect_active_envs(env_ids);

        // Sorted order: 25, 50, 100
        assert_eq!(result.unique_ids, vec![25, 50, 100]);

        // Verify slot mapping
        assert_eq!(result.slot_map.get(&25), Some(&0));
        assert_eq!(result.slot_map.get(&50), Some(&1));
        assert_eq!(result.slot_map.get(&100), Some(&2));

        // Unknown ID should return None (caller uses unwrap_or(0) for fallback)
        assert_eq!(result.slot_map.get(&999), None);
        assert_eq!(result.slot_map.get(&999).copied().unwrap_or(0), 0);
    }

    #[test]
    fn test_collect_active_envs_empty() {
        use super::collect_active_envs;

        let result = collect_active_envs(&[]);
        assert!(result.unique_ids.is_empty());
        assert!(result.slot_map.is_empty());
        assert_eq!(result.overflow_count, 0);
    }

    #[test]
    fn test_collect_active_envs_single() {
        use super::collect_active_envs;

        let result = collect_active_envs(&[42]);
        assert_eq!(result.unique_ids, vec![42]);
        assert_eq!(result.slot_map.get(&42), Some(&0));
        assert_eq!(result.overflow_count, 0);
    }

    #[test]
    fn test_collect_active_envs_exactly_at_cap() {
        use super::{MAX_ACTIVE_ENVS, collect_active_envs};

        // Exactly MAX_ACTIVE_ENVS unique IDs
        let env_ids: Vec<u32> = (0..MAX_ACTIVE_ENVS).collect();
        let result = collect_active_envs(&env_ids);

        assert_eq!(result.unique_ids.len(), MAX_ACTIVE_ENVS as usize);
        assert_eq!(result.overflow_count, 0);

        // All IDs should be mapped
        for i in 0..MAX_ACTIVE_ENVS {
            assert_eq!(result.slot_map.get(&i), Some(&i));
        }
    }

    // =========================================================================
    // Cache Tests
    // =========================================================================

    #[test]
    fn test_cache_entry_default() {
        let entry = super::EpuCacheEntry::default();
        assert_eq!(entry.state_hash, 0);
        assert!(!entry.time_dependent);
        assert!(!entry.valid);
    }

    #[test]
    fn test_epu_cache_advance_frame() {
        let cache = super::EpuCache::new();
        assert_eq!(cache.current_frame(), 0);

        cache.advance_frame();
        assert_eq!(cache.current_frame(), 1);

        cache.advance_frame();
        assert_eq!(cache.current_frame(), 2);
    }

    #[test]
    fn test_epu_cache_advance_frame_wrapping() {
        let cache = super::EpuCache::new();

        // Set to max value
        cache.current_frame.set(u64::MAX);
        assert_eq!(cache.current_frame(), u64::MAX);

        // Should wrap to 0
        cache.advance_frame();
        assert_eq!(cache.current_frame(), 0);
    }

    #[test]
    fn test_epu_cache_needs_rebuild_first_call() {
        let cache = super::EpuCache::new();
        let config = EpuConfig {
            layers: [
                [1, 2],
                [3, 4],
                [5, 6],
                [7, 8],
                [9, 10],
                [11, 12],
                [13, 14],
                [15, 16],
            ],
        };

        // First call should always need rebuild (cache not valid)
        assert!(cache.needs_rebuild(0, &config));
    }

    #[test]
    fn test_epu_cache_hit_static_config() {
        let cache = super::EpuCache::new();
        let config = EpuConfig {
            layers: [
                [1, 2],
                [3, 4],
                [5, 6],
                [7, 8],
                [9, 10],
                [11, 12],
                [13, 14],
                [15, 16],
            ], // Static config
        };

        // First call: cache miss
        assert!(cache.needs_rebuild(0, &config));

        // Second call with same config: cache hit
        assert!(!cache.needs_rebuild(0, &config));

        // Third call: still a hit
        assert!(!cache.needs_rebuild(0, &config));
    }

    #[test]
    fn test_epu_cache_miss_different_config() {
        let cache = super::EpuCache::new();
        let config1 = EpuConfig {
            layers: [
                [1, 2],
                [3, 4],
                [5, 6],
                [7, 8],
                [9, 10],
                [11, 12],
                [13, 14],
                [15, 16],
            ],
        };
        let config2 = EpuConfig {
            layers: [
                [1, 2],
                [3, 4],
                [5, 6],
                [7, 8],
                [9, 10],
                [11, 12],
                [13, 14],
                [15, 17],
            ], // Different
        };

        // First config
        assert!(cache.needs_rebuild(0, &config1));
        assert!(!cache.needs_rebuild(0, &config1));

        // Different config: cache miss
        assert!(cache.needs_rebuild(0, &config2));

        // Same different config: cache hit
        assert!(!cache.needs_rebuild(0, &config2));
    }

    #[test]
    fn test_epu_cache_miss_time_dependent() {
        use super::super::{FlowParams, epu_begin, epu_finish};

        let cache = super::EpuCache::new();

        // Create a time-dependent config (FLOW with speed > 0)
        let mut e = epu_begin();
        e.flow(FlowParams {
            speed: 20, // Time-dependent
            ..FlowParams::default()
        });
        let config = epu_finish(e);

        // Verify it's time-dependent
        assert!(config.is_time_dependent());

        // First call: needs rebuild
        assert!(cache.needs_rebuild(0, &config));

        // Second call: still needs rebuild (time-dependent always rebuilds)
        assert!(cache.needs_rebuild(0, &config));
    }

    #[test]
    fn test_epu_cache_invalidate_single() {
        let cache = super::EpuCache::new();
        let config = EpuConfig {
            layers: [
                [1, 2],
                [3, 4],
                [5, 6],
                [7, 8],
                [9, 10],
                [11, 12],
                [13, 14],
                [15, 16],
            ],
        };

        // Populate cache
        assert!(cache.needs_rebuild(0, &config));
        assert!(!cache.needs_rebuild(0, &config)); // Hit

        // Invalidate
        cache.invalidate(0);

        // Should need rebuild again
        assert!(cache.needs_rebuild(0, &config));
    }

    #[test]
    fn test_epu_cache_invalidate_all() {
        let cache = super::EpuCache::new();
        let config1 = EpuConfig {
            layers: [
                [1, 0],
                [0, 0],
                [0, 0],
                [0, 0],
                [0, 0],
                [0, 0],
                [0, 0],
                [0, 0],
            ],
        };
        let config2 = EpuConfig {
            layers: [
                [2, 0],
                [0, 0],
                [0, 0],
                [0, 0],
                [0, 0],
                [0, 0],
                [0, 0],
                [0, 0],
            ],
        };

        // Populate cache for multiple envs
        assert!(cache.needs_rebuild(0, &config1));
        assert!(cache.needs_rebuild(1, &config2));
        assert!(!cache.needs_rebuild(0, &config1)); // Hit
        assert!(!cache.needs_rebuild(1, &config2)); // Hit

        // Invalidate all
        cache.invalidate_all();

        // Both should need rebuild
        assert!(cache.needs_rebuild(0, &config1));
        assert!(cache.needs_rebuild(1, &config2));
    }

    #[test]
    fn test_epu_cache_multiple_env_ids() {
        let cache = super::EpuCache::new();
        let config_a = EpuConfig {
            layers: [
                [0xA, 0],
                [0, 0],
                [0, 0],
                [0, 0],
                [0, 0],
                [0, 0],
                [0, 0],
                [0, 0],
            ],
        };
        let config_b = EpuConfig {
            layers: [
                [0xB, 0],
                [0, 0],
                [0, 0],
                [0, 0],
                [0, 0],
                [0, 0],
                [0, 0],
                [0, 0],
            ],
        };

        // Different env IDs should have independent cache entries
        assert!(cache.needs_rebuild(10, &config_a));
        assert!(cache.needs_rebuild(20, &config_b));

        // Each should be cached independently
        assert!(!cache.needs_rebuild(10, &config_a));
        assert!(!cache.needs_rebuild(20, &config_b));

        // Changing one doesn't affect the other
        let config_a_modified = EpuConfig {
            layers: [
                [0xAA, 0],
                [0, 0],
                [0, 0],
                [0, 0],
                [0, 0],
                [0, 0],
                [0, 0],
                [0, 0],
            ],
        };
        assert!(cache.needs_rebuild(10, &config_a_modified));
        assert!(!cache.needs_rebuild(20, &config_b)); // Still cached
    }
}
