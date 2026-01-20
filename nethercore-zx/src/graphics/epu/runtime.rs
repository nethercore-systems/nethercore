//! EPU GPU Runtime (v2 - 128-bit instructions)
//!
//! This module provides the GPU infrastructure to execute EPU compute shaders
//! and produce an EnvRadiance octahedral map with a true mip pyramid, plus SH9
//! coefficients for diffuse ambient lighting.
//!
//! # Architecture
//!
//! The EPU runtime manages:
//! - GPU buffers for environment states and frame uniforms
//! - Storage texture for EnvRadiance output (mip-mapped)
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

/// Default output map size in texels (octahedral).
///
/// Override via [`EpuRuntimeSettings`] or `NETHERCORE_EPU_MAP_SIZE`.
pub const EPU_MAP_SIZE: u32 = 128;

/// Minimum mip size for the EPU radiance pyramid.
///
/// Mips smaller than this provide little value for stylized IBL and can be
/// disproportionately expensive to manage (more passes, tiny dispatches).
///
/// Override via [`EpuRuntimeSettings`] or `NETHERCORE_EPU_MIN_MIP_SIZE`.
pub const EPU_MIN_MIP_SIZE: u32 = 4;

/// Target mip size for diffuse irradiance (SH9) extraction.
///
/// The SH9 pass samples many directions; using a coarser mip reduces noise and
/// better matches "diffuse = low frequency".
const EPU_IRRAD_TARGET_SIZE: u32 = 16;

/// Runtime knobs for EPU radiance generation.
///
/// `map_size` and `min_mip_size` are intentionally exposed to make it easy to
/// experiment with quality/perf tradeoffs.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EpuRuntimeSettings {
    /// Base EnvRadiance resolution (width == height).
    pub map_size: u32,
    /// Smallest mip level to generate (inclusive).
    pub min_mip_size: u32,
}

impl Default for EpuRuntimeSettings {
    fn default() -> Self {
        Self {
            map_size: EPU_MAP_SIZE,
            min_mip_size: EPU_MIN_MIP_SIZE,
        }
    }
}

impl EpuRuntimeSettings {
    /// Read runtime overrides from environment variables:
    /// - `NETHERCORE_EPU_MAP_SIZE`
    /// - `NETHERCORE_EPU_MIN_MIP_SIZE`
    ///
    /// Invalid values fall back to defaults.
    pub fn from_env() -> Self {
        fn parse_u32(var: &str) -> Option<u32> {
            std::env::var(var).ok()?.parse::<u32>().ok()
        }

        let mut settings = Self::default();

        if let Some(v) = parse_u32("NETHERCORE_EPU_MAP_SIZE") {
            settings.map_size = v;
        }
        if let Some(v) = parse_u32("NETHERCORE_EPU_MIN_MIP_SIZE") {
            settings.min_mip_size = v;
        }

        settings.sanitized()
    }

    /// Clamp/repair settings into a valid state (power-of-two, min<=base).
    #[must_use]
    pub fn sanitized(self) -> Self {
        let mut out = self;

        if out.map_size < 1 {
            out.map_size = EPU_MAP_SIZE;
        }
        if out.min_mip_size < 1 {
            out.min_mip_size = EPU_MIN_MIP_SIZE;
        }

        if !out.map_size.is_power_of_two() {
            out.map_size = out.map_size.next_power_of_two().max(1);
        }
        if !out.min_mip_size.is_power_of_two() {
            out.min_mip_size = out.min_mip_size.next_power_of_two().max(1);
        }

        if out.min_mip_size > out.map_size {
            out.min_mip_size = out.map_size;
        }

        out
    }
}

fn calc_mip_sizes(base_size: u32, min_size: u32) -> Vec<u32> {
    debug_assert!(base_size >= 1);
    debug_assert!(min_size >= 1);
    debug_assert!(
        base_size.is_power_of_two() && min_size.is_power_of_two(),
        "EPU mip pyramid assumes power-of-two sizing (base={base_size}, min={min_size})"
    );
    debug_assert!(
        min_size <= base_size,
        "min mip size must be <= base size (base={base_size}, min={min_size})"
    );

    let mut sizes = vec![base_size];
    let mut size = base_size;
    while size > min_size {
        size /= 2;
        sizes.push(size);
    }
    sizes
}

fn choose_irrad_mip_level(mip_sizes: &[u32], target_size: u32) -> u32 {
    debug_assert!(!mip_sizes.is_empty());
    mip_sizes
        .iter()
        .position(|&s| s <= target_size)
        .unwrap_or(mip_sizes.len().saturating_sub(1)) as u32
}

/// Maximum number of environment states that can be processed.
pub const MAX_ENV_STATES: u32 = 256;

/// Initial number of texture array layers (grows on demand).
/// Starting small saves VRAM - most games use < 16 environments.
const EPU_INITIAL_LAYERS: u32 = 8;

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

/// Irradiance uniforms structure matching the WGSL `IrradUniforms` struct.
#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct IrradUniforms {
    active_count: u32,
    _pad0: u32,
    _pad1: u32,
    _pad2: u32,
}

/// SH9 (L2) diffuse irradiance coefficients.
///
/// These are Lambertian-convolved coefficients in the real SH basis, stored in
/// the following order:
/// `[Y00, Y1-1, Y10, Y11, Y2-2, Y2-1, Y20, Y21, Y22]`.
#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct EpuSh9 {
    pub c0: [f32; 3],
    _pad0: f32,
    pub c1: [f32; 3],
    _pad1: f32,
    pub c2: [f32; 3],
    _pad2: f32,
    pub c3: [f32; 3],
    _pad3: f32,
    pub c4: [f32; 3],
    _pad4: f32,
    pub c5: [f32; 3],
    _pad5: f32,
    pub c6: [f32; 3],
    _pad6: f32,
    pub c7: [f32; 3],
    _pad7: f32,
    pub c8: [f32; 3],
    _pad8: f32,
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
/// Manages GPU resources and compute pipeline for generating EnvRadiance
/// octahedral maps (with a downsample mip pyramid) from EPU configurations.
///
/// v2: Palette buffer removed - colors are embedded in 128-bit instructions.
///
/// # Texture Array Growth
///
/// The EPU textures use growable array layers starting at `EPU_INITIAL_LAYERS` (8)
/// and growing to `MAX_ENV_STATES` (256) on demand. This reduces VRAM usage for
/// games that only use a few environments while still supporting the full range.
pub struct EpuRuntime {
    settings: EpuRuntimeSettings,
    /// Incremented whenever any render-bound EPU resources are recreated.
    resource_version: u64,
    // GPU buffers
    env_states_buffer: wgpu::Buffer,
    active_env_ids_buffer: wgpu::Buffer,
    frame_uniforms_buffer: wgpu::Buffer,

    // Output texture: octahedral radiance map with a true mip-style pyramid.
    // The texture is a 2D array indexed by env_id.
    env_radiance_texture: wgpu::Texture,

    // Full view (all mips) for sampling in render.
    env_radiance_view: wgpu::TextureView,

    // Per-mip views (single mip) for compute passes (build + downsample chain).
    env_radiance_mip_views: Vec<wgpu::TextureView>,

    // Cached mip sizes for dispatch (level 0 is base resolution).
    env_mip_sizes: Vec<u32>,

    // Mip level used as source for SH9 irradiance extraction.
    irrad_source_mip: u32,

    // Current texture array layer capacity (grows on demand)
    env_layer_capacity: u32,

    // Pipeline resources
    pipeline: wgpu::ComputePipeline,
    bind_group_layout: wgpu::BindGroupLayout,
    bind_group: wgpu::BindGroup,

    // Mip pyramid generation resources (downsample chain).
    mip_pipeline: wgpu::ComputePipeline,
    mip_bind_group_layout: wgpu::BindGroupLayout,

    // Compute sampler shared by irradiance extraction (and any compute sampling).
    compute_sampler: wgpu::Sampler,

    // Irradiance extraction resources
    sh9_buffer: wgpu::Buffer,
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
        Self::new_with_settings(device, EpuRuntimeSettings::from_env())
    }

    pub fn new_with_settings(device: &wgpu::Device, settings: EpuRuntimeSettings) -> Self {
        let settings = settings.sanitized();
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

        // Create octahedral radiance texture with a downsampled mip pyramid.
        let env_mip_sizes = calc_mip_sizes(settings.map_size, settings.min_mip_size);
        let mip_level_count = env_mip_sizes.len() as u32;
        let irrad_source_mip = choose_irrad_mip_level(&env_mip_sizes, EPU_IRRAD_TARGET_SIZE);

        let env_radiance_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("EPU EnvRadiance"),
            size: wgpu::Extent3d {
                width: settings.map_size,
                height: settings.map_size,
                depth_or_array_layers: EPU_INITIAL_LAYERS,
            },
            mip_level_count,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba16Float,
            usage: wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

        // Full view (all mips) for sampling in render.
        let env_radiance_view = env_radiance_texture.create_view(&wgpu::TextureViewDescriptor {
            label: Some("EPU EnvRadiance View"),
            dimension: Some(wgpu::TextureViewDimension::D2Array),
            ..Default::default()
        });

        // Per-mip views (single mip) for compute passes.
        let env_radiance_mip_views: Vec<wgpu::TextureView> = (0..mip_level_count)
            .map(|mip| {
                env_radiance_texture.create_view(&wgpu::TextureViewDescriptor {
                    dimension: Some(wgpu::TextureViewDimension::D2Array),
                    base_mip_level: mip,
                    mip_level_count: Some(1),
                    ..Default::default()
                })
            })
            .collect();

        // Concatenate shader sources in order
        let shader_source =
            format!("{EPU_COMMON}\n{EPU_BOUNDS}\n{EPU_FEATURES}\n{EPU_COMPUTE_ENV}");

        // Create shader module
        let shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("EPU Compute Env Shader"),
            source: wgpu::ShaderSource::Wgsl(shader_source.into()),
        });

        // Create bind group layout (v2: no palette buffer, bindings 0-3)
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

        // Create bind group (v2: no palette, bindings 0-3)
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
                    resource: wgpu::BindingResource::TextureView(&env_radiance_mip_views[0]),
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
        // Mip pyramid generation resources (downsample chain)
        // =====================================================================

        // Sampler for compute sampling (used by irradiance extraction).
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

        // Create mip bind group layout matching epu_compute_blur.wgsl bindings
        let mip_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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

        // Create mip shader module
        let mip_shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("EPU Compute Mip Shader"),
            source: wgpu::ShaderSource::Wgsl(EPU_COMPUTE_BLUR.into()),
        });

        // Create mip pipeline layout
        let mip_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("EPU Mip Pipeline Layout"),
            bind_group_layouts: &[&mip_bind_group_layout],
            push_constant_ranges: &[],
        });

        // Create mip compute pipeline
        let mip_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("EPU Compute Mip Pipeline"),
            layout: Some(&mip_pipeline_layout),
            module: &mip_shader_module,
            entry_point: Some("epu_downsample_mip"),
            compilation_options: Default::default(),
            cache: None,
        });

        // =====================================================================
        // Irradiance extraction pipeline resources
        // =====================================================================

        // Create SH9 storage buffer (MAX_ENV_STATES * 144 bytes)
        let sh9_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("EPU SH9"),
            size: (MAX_ENV_STATES as usize * std::mem::size_of::<EpuSh9>()) as u64,
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
            entry_point: Some("epu_extract_sh9"),
            compilation_options: Default::default(),
            cache: None,
        });

        Self {
            settings,
            resource_version: 0,
            env_states_buffer,
            active_env_ids_buffer,
            frame_uniforms_buffer,
            env_radiance_texture,
            env_radiance_view,
            env_radiance_mip_views,
            env_mip_sizes,
            irrad_source_mip,
            env_layer_capacity: EPU_INITIAL_LAYERS,
            pipeline,
            bind_group_layout,
            bind_group,
            mip_pipeline,
            mip_bind_group_layout,
            compute_sampler,
            sh9_buffer,
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

    /// Get the current texture array layer capacity.
    pub fn layer_capacity(&self) -> u32 {
        self.env_layer_capacity
    }

    /// Ensure EPU textures have sufficient layer capacity.
    ///
    /// If the required capacity exceeds current capacity, the radiance texture
    /// array is recreated with a larger layer count (keeping the mip pyramid
    /// structure). The new capacity is the next power of two that fits the
    /// required count, capped at MAX_ENV_STATES (256).
    ///
    /// # Arguments
    /// * `device` - The wgpu device for creating textures
    /// * `required` - Minimum number of layers needed
    ///
    /// # Returns
    /// `true` if textures were recreated, `false` if no change was needed.
    pub fn ensure_layer_capacity(&mut self, device: &wgpu::Device, required: u32) -> bool {
        if required <= self.env_layer_capacity {
            return false;
        }

        // Grow to next power of two, capped at MAX_ENV_STATES
        let new_capacity = (required.max(1))
            .checked_next_power_of_two()
            .unwrap_or(MAX_ENV_STATES)
            .min(MAX_ENV_STATES);

        tracing::debug!(
            "Growing EPU texture layers: {} → {}",
            self.env_layer_capacity,
            new_capacity
        );

        // Recreate radiance texture with new layer count
        self.env_radiance_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("EPU EnvRadiance"),
            size: wgpu::Extent3d {
                width: self.settings.map_size,
                height: self.settings.map_size,
                depth_or_array_layers: new_capacity,
            },
            mip_level_count: self.env_mip_sizes.len() as u32,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba16Float,
            usage: wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

        // Recreate full sampling view (all mips)
        self.env_radiance_view =
            self.env_radiance_texture
                .create_view(&wgpu::TextureViewDescriptor {
                    label: Some("EPU EnvRadiance View"),
                    dimension: Some(wgpu::TextureViewDimension::D2Array),
                    ..Default::default()
                });

        // Recreate per-mip views (single mip each)
        let mip_level_count = self.env_mip_sizes.len() as u32;
        self.env_radiance_mip_views = (0..mip_level_count)
            .map(|mip| {
                self.env_radiance_texture
                    .create_view(&wgpu::TextureViewDescriptor {
                        dimension: Some(wgpu::TextureViewDimension::D2Array),
                        base_mip_level: mip,
                        mip_level_count: Some(1),
                        ..Default::default()
                    })
            })
            .collect();

        // Recreate main bind group with new texture views
        self.bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("EPU Bind Group"),
            layout: &self.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: self.env_states_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: self.active_env_ids_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: self.frame_uniforms_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::TextureView(&self.env_radiance_mip_views[0]),
                },
            ],
        });

        self.env_layer_capacity = new_capacity;
        self.resource_version = self.resource_version.wrapping_add(1);

        // Invalidate cache since textures were recreated
        self.cache.invalidate_all();

        true
    }

    /// Dispatch a single downsample pass from mip i to mip i+1.
    ///
    /// The source and destination are views of the same radiance texture at
    /// different mip levels.
    fn dispatch_mip_pass(
        &self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        input_view: &wgpu::TextureView,
        output_view: &wgpu::TextureView,
        output_size: u32,
        active_count: u32,
    ) {
        let mip_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("EPU Mip Bind Group"),
            layout: &self.mip_bind_group_layout,
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
            ],
        });

        let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("EPU Mip Pass"),
            timestamp_writes: None,
        });

        compute_pass.set_pipeline(&self.mip_pipeline);
        compute_pass.set_bind_group(0, &mip_bind_group, &[]);

        let workgroups_x = output_size.div_ceil(8);
        let workgroups_y = output_size.div_ceil(8);
        compute_pass.dispatch_workgroups(workgroups_x, workgroups_y, active_count);
    }

    /// Dispatch the irradiance extraction pass.
    ///
    /// This extracts SH9 coefficients from a coarse radiance mip level and
    /// stores them in the SH9 buffer.
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
        let irrad_source_view = &self.env_radiance_mip_views[self.irrad_source_mip as usize];
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
                    resource: wgpu::BindingResource::TextureView(irrad_source_view),
                },
                wgpu::BindGroupEntry {
                    binding: 5,
                    resource: wgpu::BindingResource::Sampler(&self.compute_sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 6,
                    resource: self.sh9_buffer.as_entire_binding(),
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
    /// This dispatches the compute shader to generate EnvRadiance (mip 0) and then builds
    /// a downsampled mip pyramid from that radiance.
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
            map_size: self.settings.map_size,
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
            let workgroups_x = self.settings.map_size.div_ceil(8);
            let workgroups_y = self.settings.map_size.div_ceil(8);
            let workgroups_z = 1; // Single environment
            compute_pass.dispatch_workgroups(workgroups_x, workgroups_y, workgroups_z);
        }

        // Mip pyramid: mip0 -> mip1 -> ...
        for mip in 0..self.env_mip_sizes.len().saturating_sub(1) {
            let output_size = self.env_mip_sizes[mip + 1];
            self.dispatch_mip_pass(
                device,
                encoder,
                &self.env_radiance_mip_views[mip],
                &self.env_radiance_mip_views[mip + 1],
                output_size,
                1,
            );
        }

        // Extract SH9 diffuse irradiance from a coarse radiance mip
        self.dispatch_irrad_pass(device, queue, encoder, 1);
    }

    /// Build environment maps for multiple environments.
    ///
    /// This dispatches the compute shader to generate radiance (mip 0) and then
    /// builds a downsampled mip pyramid for rough reflections and diffuse SH9.
    ///
    /// Uses dirty-state caching to skip rebuilding unchanged static environments.
    /// Call `advance_frame()` once per frame before this method to ensure proper
    /// cache behavior for time-dependent environments.
    ///
    /// # Texture Growth
    ///
    /// If any `env_id` in `configs` exceeds the current texture array capacity,
    /// the textures will be automatically grown to accommodate it. This is a
    /// one-time cost that happens rarely as games typically use few environments.
    ///
    /// # Arguments
    /// * `device` - The wgpu device for creating bind groups (and textures if growing)
    /// * `queue` - The wgpu queue for buffer writes
    /// * `encoder` - Command encoder to record compute pass
    /// * `configs` - Slice of (env_id, config) pairs to evaluate
    /// * `time` - Current time for animation (in seconds)
    pub fn build_envs(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        configs: &[(u32, &EpuConfig)],
        time: f32,
    ) {
        if configs.is_empty() {
            return;
        }

        // Ensure texture arrays have enough layers for all env_ids
        // The +1 is because env_id is 0-indexed, so env_id=7 needs 8 layers
        let max_env_id = configs.iter().map(|(id, _)| *id).max().unwrap_or(0);
        self.ensure_layer_capacity(device, max_env_id + 1);

        // Filter configs to only those that need rebuilding (cache miss or time-dependent)
        let dirty_configs: Vec<(u32, &EpuConfig)> = configs
            .iter()
            .take(MAX_ACTIVE_ENVS as usize)
            .filter(|(env_id, config)| self.cache.needs_rebuild(*env_id, config))
            .copied()
            .collect();

        // Always upload frame uniforms for rendering; procedural sky/reflections
        // consume the same time value as the compute pass.
        let frame_uniforms = FrameUniforms {
            time,
            active_count: dirty_configs.len() as u32,
            map_size: self.settings.map_size,
            _pad0: 0,
        };
        queue.write_buffer(
            &self.frame_uniforms_buffer,
            0,
            bytemuck::cast_slice(&[frame_uniforms]),
        );

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
            let workgroups_x = self.settings.map_size.div_ceil(8);
            let workgroups_y = self.settings.map_size.div_ceil(8);
            compute_pass.dispatch_workgroups(workgroups_x, workgroups_y, active_count);
        }

        // Mip pyramid: mip0 -> mip1 -> ...
        for mip in 0..self.env_mip_sizes.len().saturating_sub(1) {
            let output_size = self.env_mip_sizes[mip + 1];
            self.dispatch_mip_pass(
                device,
                encoder,
                &self.env_radiance_mip_views[mip],
                &self.env_radiance_mip_views[mip + 1],
                output_size,
                active_count,
            );
        }

        // Extract SH9 diffuse irradiance from a coarse radiance mip
        self.dispatch_irrad_pass(device, queue, encoder, active_count);
    }

    /// Get a reference to the radiance texture (mip-mapped) for sampling.
    pub fn env_radiance_texture(&self) -> &wgpu::Texture {
        &self.env_radiance_texture
    }

    /// Get the radiance view for binding to render pipelines (includes all mips).
    pub fn env_radiance_view(&self) -> &wgpu::TextureView {
        &self.env_radiance_view
    }

    /// Get a single-mip radiance view (useful for debug/inspection tooling).
    pub fn env_radiance_mip_view(&self, mip: u32) -> Option<&wgpu::TextureView> {
        self.env_radiance_mip_views.get(mip as usize)
    }

    /// Get a reference to the SH9 storage buffer.
    ///
    /// This buffer contains the L2 diffuse irradiance SH coefficients extracted
    /// from a coarse radiance mip for each environment.
    pub fn sh9_buffer(&self) -> &wgpu::Buffer {
        &self.sh9_buffer
    }

    /// Get the current runtime settings (map size and mip configuration).
    pub fn settings(&self) -> EpuRuntimeSettings {
        self.settings
    }

    /// Resource version for render-bindable EPU outputs.
    ///
    /// This changes when the EnvRadiance texture/view is recreated (e.g. layer growth).
    pub fn resource_version(&self) -> u64 {
        self.resource_version
    }

    /// Get a reference to the packed environment states buffer (read-only in render).
    pub fn env_states_buffer(&self) -> &wgpu::Buffer {
        &self.env_states_buffer
    }

    /// Get a reference to the frame uniforms buffer (time + map sizing).
    pub fn frame_uniforms_buffer(&self) -> &wgpu::Buffer {
        &self.frame_uniforms_buffer
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
    fn test_calc_mip_sizes() {
        let sizes = calc_mip_sizes(128, 4);
        assert_eq!(sizes, vec![128, 64, 32, 16, 8, 4]);
        assert!(sizes.iter().all(|&s| s.is_power_of_two()));
        assert!(sizes.windows(2).all(|w| w[0] > w[1]));
    }

    #[test]
    fn test_choose_irrad_mip_level() {
        let sizes = calc_mip_sizes(128, 4);
        assert_eq!(choose_irrad_mip_level(&sizes, 16), 3);
        assert_eq!(choose_irrad_mip_level(&sizes, 8), 4);
        assert_eq!(choose_irrad_mip_level(&sizes, 4), 5);
        // If target is smaller than the smallest generated mip, clamp to last.
        assert_eq!(choose_irrad_mip_level(&sizes, 2), 5);
        // If target is larger than base, pick mip 0.
        assert_eq!(choose_irrad_mip_level(&sizes, 256), 0);
    }

    #[test]
    fn test_settings_sanitized_power_of_two() {
        let s = EpuRuntimeSettings {
            map_size: 300,
            min_mip_size: 7,
        }
        .sanitized();

        assert!(s.map_size.is_power_of_two());
        assert!(s.min_mip_size.is_power_of_two());
        assert!(s.min_mip_size <= s.map_size);
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
    fn test_sh9_size() {
        // EpuSh9 must be exactly 144 bytes (9 coefficients x 16 bytes each)
        assert_eq!(
            std::mem::size_of::<EpuSh9>(),
            144,
            "EpuSh9 must be 144 bytes"
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
