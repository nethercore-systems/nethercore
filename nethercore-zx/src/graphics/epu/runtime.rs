//! EPU GPU Runtime (128-bit instructions)
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
//! # Format Notes
//!
//! EPU uses 128-bit instructions with embedded RGB24 colors. The palette
//! buffer has been removed - colors are now packed directly into the
//! instruction format.
//!
//! # Usage
//!
//! ```ignore
//! let epu_runtime = EpuRuntime::new(&device);
//! epu_runtime.build_env(&device, &queue, &mut encoder, &config);
//! ```

use super::EpuConfig;
use super::cache::EpuCache;
use super::pipelines;
use super::settings::{
    EPU_INITIAL_LAYERS, EPU_IRRAD_TARGET_SIZE, EpuRuntimeSettings, MAX_ACTIVE_ENVS, MAX_ENV_STATES,
    calc_mip_sizes, choose_irrad_mip_level,
};
use super::types::{FrameUniforms, GpuEnvironmentState, IrradUniforms};

use std::sync::atomic::{AtomicU32, Ordering};

static EPU_BUILD_DEBUG_COUNT: AtomicU32 = AtomicU32::new(0);

/// EPU GPU runtime for environment map generation.
///
/// Manages GPU resources and compute pipeline for generating EnvRadiance
/// octahedral maps (with a downsample mip pyramid) from EPU configurations.
///
/// Palette buffer removed - colors are embedded in 128-bit instructions.
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

        // Create GPU buffers
        let (env_states_buffer, active_env_ids_buffer, frame_uniforms_buffer) =
            pipelines::create_buffers(device);

        // Create radiance texture and views
        let env_mip_sizes = calc_mip_sizes(settings.map_size, settings.min_mip_size);
        let mip_level_count = env_mip_sizes.len() as u32;
        let irrad_source_mip = choose_irrad_mip_level(&env_mip_sizes, EPU_IRRAD_TARGET_SIZE);

        let (env_radiance_texture, env_radiance_view, env_radiance_mip_views) =
            pipelines::create_radiance_texture(
                device,
                settings.map_size,
                mip_level_count,
                EPU_INITIAL_LAYERS,
            );

        // Create main compute pipeline
        let (pipeline, bind_group_layout, bind_group) = pipelines::create_main_pipeline(
            device,
            &env_states_buffer,
            &active_env_ids_buffer,
            &frame_uniforms_buffer,
            &env_radiance_mip_views[0],
        );

        // Create mip downsample pipeline
        let (mip_pipeline, mip_bind_group_layout, compute_sampler) =
            pipelines::create_mip_pipeline(device);

        // Create irradiance extraction pipeline
        let (sh9_buffer, irrad_pipeline, irrad_uniforms_buffer, irrad_bind_group_layout) =
            pipelines::create_irrad_pipeline(device);

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

    /// Invalidate the cache entry for a specific environment ID.
    ///
    /// This forces the environment to be rebuilt on the next `build_envs()` call.
    pub fn invalidate_cache(&mut self, env_id: u32) {
        self.cache.invalidate(env_id);
    }

    /// Invalidate all cache entries.
    ///
    /// This forces all environments to be rebuilt on the next `build_envs()` call.
    pub fn invalidate_all_caches(&mut self) {
        self.cache.invalidate_all();
    }

    // =========================================================================
    // Texture Growth
    // =========================================================================

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
            "Growing EPU texture layers: {} -> {}",
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
        self.bind_group = pipelines::create_main_bind_group(
            device,
            &self.bind_group_layout,
            &self.env_states_buffer,
            &self.active_env_ids_buffer,
            &self.frame_uniforms_buffer,
            &self.env_radiance_mip_views[0],
        );

        self.env_layer_capacity = new_capacity;
        self.resource_version = self.resource_version.wrapping_add(1);

        // Invalidate cache since textures were recreated
        self.cache.invalidate_all();

        true
    }

    // =========================================================================
    // Dispatch Helpers
    // =========================================================================

    /// Dispatch a single downsample pass from mip i to mip i+1.
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

        let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("EPU Irrad Pass"),
            timestamp_writes: None,
        });

        compute_pass.set_pipeline(&self.irrad_pipeline);
        compute_pass.set_bind_group(0, &irrad_bind_group, &[]);
        compute_pass.dispatch_workgroups(1, 1, active_count);
    }

    // =========================================================================
    // Build Methods
    // =========================================================================

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
    pub fn build_env(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        config: &EpuConfig,
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
            active_count: 1,
            map_size: self.settings.map_size,
            _pad0: 0,
            _pad1: 0,
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
    /// Uses dirty-state caching to skip rebuilding unchanged environments.
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
    pub fn build_envs(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        configs: &[(u32, &EpuConfig)],
    ) {
        if configs.is_empty() {
            return;
        }

        // Ensure texture arrays have enough layers for all env_ids
        let max_env_id = configs.iter().map(|(id, _)| *id).max().unwrap_or(0);
        self.ensure_layer_capacity(device, max_env_id + 1);

        // Filter configs to only those that need rebuilding
        let dirty_configs: Vec<(u32, &EpuConfig)> = {
            let cache = &mut self.cache;
            configs
                .iter()
                .take(MAX_ACTIVE_ENVS as usize)
                .filter(|(env_id, config)| cache.needs_rebuild(*env_id, config))
                .copied()
                .collect()
        };

        if std::env::var("NETHERCORE_EPU_DEBUG_BUILD").as_deref() == Ok("1") {
            let n = EPU_BUILD_DEBUG_COUNT.fetch_add(1, Ordering::Relaxed);
            if n < 64 {
                let (env0_id, env0_hash, env0_d0, env0_w0) = dirty_configs
                    .get(0)
                    .map(|(id, cfg)| {
                        let d0 = ((cfg.layers[0][1] >> 24) & 0xFF) as u8;
                        let gpu0 = GpuEnvironmentState::from(*cfg);
                        (*id, cfg.state_hash(), d0, gpu0.layers[0][0])
                    })
                    .unwrap_or((0, 0, 0, 0));

                tracing::info!(
                    "epu_build debug: call={}, input_configs={}, dirty_configs={}, first_dirty=(env_id={}, d0={}, w0=0x{:08x}, hash=0x{:016x})",
                    n,
                    configs.len(),
                    dirty_configs.len(),
                    env0_id,
                    env0_d0,
                    env0_w0,
                    env0_hash
                );
            }
        }

        // Always upload frame uniforms for rendering
        let frame_uniforms = FrameUniforms {
            active_count: dirty_configs.len() as u32,
            map_size: self.settings.map_size,
            _pad0: 0,
            _pad1: 0,
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

    // =========================================================================
    // Accessors
    // =========================================================================

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
    pub fn sh9_buffer(&self) -> &wgpu::Buffer {
        &self.sh9_buffer
    }

    /// Get the current runtime settings (map size and mip configuration).
    pub fn settings(&self) -> EpuRuntimeSettings {
        self.settings
    }

    /// Resource version for render-bindable EPU outputs.
    pub fn resource_version(&self) -> u64 {
        self.resource_version
    }

    /// Get a reference to the packed environment states buffer (read-only in render).
    pub fn env_states_buffer(&self) -> &wgpu::Buffer {
        &self.env_states_buffer
    }

    /// Get a reference to the frame uniforms buffer (active_count + map sizing).
    pub fn frame_uniforms_buffer(&self) -> &wgpu::Buffer {
        &self.frame_uniforms_buffer
    }
}
