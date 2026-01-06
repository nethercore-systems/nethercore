//! Shader pipeline management
//!
//! Handles shader compilation, pipeline caching, and bind group layout creation
//! for all render mode and vertex format combinations.

use hashbrown::HashMap;

use super::render_state::{PassConfig, RenderState};
use super::vertex::VertexFormatInfo;

/// Key for pipeline cache lookup
///
/// Pipeline keys are derived from PassConfig to enable caching by
/// depth/stencil configuration. The pass_config_hash captures the
/// unique combination of depth/stencil settings.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum PipelineKey {
    /// Regular mesh rendering pipeline
    Regular {
        render_mode: u8,
        vertex_format: u8,
        depth_test: bool,
        cull_mode: u8,
        /// Hash of PassConfig fields that affect pipeline state
        pass_config_hash: u64,
    },
    /// GPU-instanced quad rendering pipeline (billboards, sprites)
    Quad {
        depth_test: bool,
        /// Hash of PassConfig fields that affect pipeline state
        pass_config_hash: u64,
        /// True for screen-space quads (always write depth), false for billboards (use PassConfig)
        is_screen_space: bool,
    },
    /// Procedural sky rendering pipeline (always renders behind)
    Sky {
        /// Hash of PassConfig fields that affect pipeline state
        pass_config_hash: u64,
    },
}

/// Compute a hash of PassConfig fields that affect pipeline state
fn pass_config_hash(config: &PassConfig) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    config.hash(&mut hasher);
    hasher.finish()
}

impl PipelineKey {
    /// Create a new regular pipeline key from render state and pass config
    pub fn new(
        render_mode: u8,
        format: u8,
        state: &RenderState,
        pass_config: &PassConfig,
    ) -> Self {
        Self::Regular {
            render_mode,
            vertex_format: format,
            depth_test: state.depth_test,
            cull_mode: state.cull_mode as u8,
            pass_config_hash: pass_config_hash(pass_config),
        }
    }

    /// Create a quad pipeline key
    pub fn quad(state: &RenderState, pass_config: &PassConfig, is_screen_space: bool) -> Self {
        Self::Quad {
            depth_test: state.depth_test,
            pass_config_hash: pass_config_hash(pass_config),
            is_screen_space,
        }
    }

    /// Create a sky pipeline key
    pub fn sky(pass_config: &PassConfig) -> Self {
        Self::Sky {
            pass_config_hash: pass_config_hash(pass_config),
        }
    }
}

/// Cached pipeline entry with bind group layouts
pub(crate) struct PipelineEntry {
    pub pipeline: wgpu::RenderPipeline,
    pub bind_group_layout_frame: wgpu::BindGroupLayout,
    pub bind_group_layout_textures: wgpu::BindGroupLayout,
}

// Note: Stencil state and color write mask are obtained from PassConfig methods:
// - PassConfig::to_wgpu_stencil_state()
// - PassConfig::color_write_mask()
// - PassConfig depth_compare and depth_write fields

/// Create a new pipeline for the given vertex format and render state
pub(crate) fn create_pipeline(
    device: &wgpu::Device,
    surface_format: wgpu::TextureFormat,
    render_mode: u8,
    format: u8,
    state: &RenderState,
    pass_config: &PassConfig,
) -> PipelineEntry {
    use crate::shader_gen::generate_shader;

    // Generate shader source, falling back to Mode 0 if the requested mode/format is invalid
    let shader_source = match generate_shader(render_mode, format) {
        Ok(source) => source,
        Err(e) => {
            tracing::warn!(
                "Shader generation failed for mode {} format {}: {}. Falling back to Mode 0 (Lambert).",
                render_mode,
                format,
                e
            );
            // Fallback to Mode 0 (Lambert) which supports all formats
            generate_shader(0, format).expect("Mode 0 should support all vertex formats")
        }
    };

    // Create shader module
    let shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some(&format!("Shader Mode{} Format{}", render_mode, format)),
        source: wgpu::ShaderSource::Wgsl(shader_source.into()),
    });

    // Create bind group layouts
    let bind_group_layout_frame = create_frame_bind_group_layout(device, render_mode);
    let bind_group_layout_textures = create_texture_bind_group_layout(device);

    // Create pipeline layout (no push constants needed - using MVP indices buffer instead)
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Pipeline Layout"),
        bind_group_layouts: &[&bind_group_layout_frame, &bind_group_layout_textures],
        push_constant_ranges: &[],
    });

    // Get vertex format info
    let vertex_info = VertexFormatInfo::for_format(format);

    // Create render pipeline
    let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some(&format!("Pipeline Mode{} Format{}", render_mode, format)),
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader_module,
            entry_point: Some("vs"),
            buffers: &[vertex_info.vertex_buffer_layout()],
            compilation_options: Default::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader_module,
            entry_point: Some("fs"),
            targets: &[Some(wgpu::ColorTargetState {
                format: surface_format,
                blend: None, // All rendering is opaque (dithering used for transparency)
                write_mask: pass_config.color_write_mask(),
            })],
            compilation_options: Default::default(),
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: state.cull_mode.to_wgpu(),
            unclipped_depth: false,
            polygon_mode: wgpu::PolygonMode::Fill,
            conservative: false,
        },
        depth_stencil: Some(wgpu::DepthStencilState {
            format: wgpu::TextureFormat::Depth24PlusStencil8,
            depth_write_enabled: pass_config.depth_write,
            depth_compare: pass_config.depth_compare,
            stencil: pass_config.to_wgpu_stencil_state(),
            bias: wgpu::DepthBiasState::default(),
        }),
        multisample: wgpu::MultisampleState {
            count: 1,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
        multiview: None,
        cache: None,
    });

    PipelineEntry {
        pipeline,
        bind_group_layout_frame,
        bind_group_layout_textures,
    }
}

/// Create a quad pipeline for GPU-instanced rendering
///
/// `is_screen_space` determines depth behavior:
/// - true: Screen-space quads always write depth at 0 for early-z optimization
/// - false: Billboards use PassConfig depth settings (they're 3D positioned)
pub(crate) fn create_quad_pipeline(
    device: &wgpu::Device,
    surface_format: wgpu::TextureFormat,
    pass_config: &PassConfig,
    is_screen_space: bool,
) -> PipelineEntry {
    // Load quad shader (generated from quad_template.wgsl by build.rs)
    use crate::shader_gen::QUAD_SHADER;

    // Create shader module
    let shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Quad Shader"),
        source: wgpu::ShaderSource::Wgsl(QUAD_SHADER.into()),
    });

    // Create bind group layouts (same as regular pipelines)
    let bind_group_layout_frame = create_frame_bind_group_layout(device, 0);
    let bind_group_layout_textures = create_texture_bind_group_layout(device);

    // Create pipeline layout
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Quad Pipeline Layout"),
        bind_group_layouts: &[&bind_group_layout_frame, &bind_group_layout_textures],
        push_constant_ranges: &[],
    });

    // Define quad vertex format (POS_UV_COLOR: position, uv, color)
    use super::vertex::{FORMAT_COLOR, FORMAT_UV};
    let quad_format = FORMAT_UV | FORMAT_COLOR;
    let vertex_info = VertexFormatInfo::for_format(quad_format);

    // Screen-space quads use Always depth compare to allow later quads to overwrite earlier ones
    // (painter's algorithm). Depth writes remain enabled for early-z optimization against 3D.
    // Billboard quads use PassConfig settings since they're 3D-positioned and need proper depth testing.
    let (depth_write_enabled, depth_compare) = if is_screen_space {
        (true, wgpu::CompareFunction::Always)
    } else {
        (pass_config.depth_write, pass_config.depth_compare)
    };

    // Create render pipeline
    let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some(if is_screen_space { "Screen-Space Quad Pipeline" } else { "Billboard Pipeline" }),
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader_module,
            entry_point: Some("vs"),
            buffers: &[vertex_info.vertex_buffer_layout()],
            compilation_options: Default::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader_module,
            entry_point: Some("fs"),
            targets: &[Some(wgpu::ColorTargetState {
                format: surface_format,
                blend: None, // All rendering is opaque (dithering used for transparency)
                write_mask: pass_config.color_write_mask(),
            })],
            compilation_options: Default::default(),
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: None, // Quads are always double-sided
            unclipped_depth: false,
            polygon_mode: wgpu::PolygonMode::Fill,
            conservative: false,
        },
        depth_stencil: Some(wgpu::DepthStencilState {
            format: wgpu::TextureFormat::Depth24PlusStencil8,
            depth_write_enabled,
            depth_compare,
            stencil: pass_config.to_wgpu_stencil_state(),
            bias: wgpu::DepthBiasState::default(),
        }),
        multisample: wgpu::MultisampleState {
            count: 1,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
        multiview: None,
        cache: None,
    });

    PipelineEntry {
        pipeline,
        bind_group_layout_frame,
        bind_group_layout_textures,
    }
}

/// Create sky rendering pipeline for fullscreen procedural sky
pub(crate) fn create_sky_pipeline(
    device: &wgpu::Device,
    surface_format: wgpu::TextureFormat,
    pass_config: &PassConfig,
) -> PipelineEntry {
    // Load sky shader (generated from common.wgsl + sky_template.wgsl by build.rs)
    use crate::shader_gen::SKY_SHADER;

    // Create shader module
    let shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Sky Shader"),
        source: wgpu::ShaderSource::Wgsl(SKY_SHADER.into()),
    });

    // Create bind group layouts (same as other pipelines)
    let bind_group_layout_frame = create_frame_bind_group_layout(device, 0);
    let bind_group_layout_textures = create_texture_bind_group_layout(device);

    // Create pipeline layout
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Sky Pipeline Layout"),
        bind_group_layouts: &[&bind_group_layout_frame, &bind_group_layout_textures],
        push_constant_ranges: &[],
    });

    // Create render pipeline
    let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Sky Pipeline"),
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader_module,
            entry_point: Some("vs"),
            buffers: &[], // No vertex buffer - generated in shader
            compilation_options: Default::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader_module,
            entry_point: Some("fs"),
            targets: &[Some(wgpu::ColorTargetState {
                format: surface_format,
                blend: None, // No blending - opaque background
                write_mask: pass_config.color_write_mask(),
            })],
            compilation_options: Default::default(),
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: None, // Fullscreen triangle, no culling needed
            unclipped_depth: false,
            polygon_mode: wgpu::PolygonMode::Fill,
            conservative: false,
        },
        depth_stencil: Some(wgpu::DepthStencilState {
            format: wgpu::TextureFormat::Depth24PlusStencil8,
            depth_write_enabled: false, // Sky is infinitely far, don't write depth
            depth_compare: wgpu::CompareFunction::LessEqual, // Only render where depth == 1.0 (cleared, nothing drew)
            stencil: pass_config.to_wgpu_stencil_state(),
            bias: wgpu::DepthBiasState::default(),
        }),
        multisample: wgpu::MultisampleState {
            count: 1,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
        multiview: None,
        cache: None,
    });

    PipelineEntry {
        pipeline,
        bind_group_layout_frame,
        bind_group_layout_textures,
    }
}

/// Create bind group layout for per-frame uniforms (group 0)
///
/// UNIFIED BUFFER ARCHITECTURE (6 bindings, all storage, grouped by purpose):
/// - Binding 0-1: Transforms (unified_transforms, mvp_indices)
/// - Binding 2: Shading (shading_states)
/// - Binding 3: Animation (unified_animation)
/// - Binding 4: Environment (environment_states) - Multi-Environment v3
/// - Binding 5: Quad rendering (quad_instances)
///
/// CPU pre-computes absolute indices into unified_transforms (no frame_offsets needed).
/// Screen dimensions eliminated - resolution_index packed into QuadInstance.mode.
fn create_frame_bind_group_layout(
    device: &wgpu::Device,
    _render_mode: u8,
) -> wgpu::BindGroupLayout {
    let bindings = vec![
        // =====================================================================
        // TRANSFORMS (bindings 0-1)
        // =====================================================================

        // Binding 0: unified_transforms - all mat4x4 matrices [models | views | projs]
        // VERTEX_FRAGMENT: sky shader needs view/proj matrices in fragment
        wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Storage { read_only: true },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        },
        // Binding 1: mvp_indices - absolute indices [model_idx, view_idx, proj_idx, shading_idx]
        // view_idx and proj_idx are pre-offset by CPU to point directly into unified_transforms
        wgpu::BindGroupLayoutEntry {
            binding: 1,
            visibility: wgpu::ShaderStages::VERTEX,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Storage { read_only: true },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        },
        // =====================================================================
        // SHADING (binding 2)
        // =====================================================================

        // Binding 2: shading_states - per-draw shading state array
        wgpu::BindGroupLayoutEntry {
            binding: 2,
            visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Storage { read_only: true },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        },
        // =====================================================================
        // ANIMATION (binding 3)
        // =====================================================================

        // Binding 3: unified_animation - all mat3x4 [inverse_bind | keyframes | immediate]
        wgpu::BindGroupLayoutEntry {
            binding: 3,
            visibility: wgpu::ShaderStages::VERTEX,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Storage { read_only: true },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        },
        // =====================================================================
        // ENVIRONMENT (binding 4) - Multi-Environment v3
        // =====================================================================

        // Binding 4: environment_states - per-frame array of PackedEnvironmentState
        // Used by sky shader for procedural environment rendering
        wgpu::BindGroupLayoutEntry {
            binding: 4,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Storage { read_only: true },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        },
        // =====================================================================
        // QUAD RENDERING (binding 5)
        // =====================================================================

        // Binding 5: quad_instances - for GPU-instanced quad rendering
        // Screen dimensions eliminated - resolution_index packed into mode field
        wgpu::BindGroupLayoutEntry {
            binding: 5,
            visibility: wgpu::ShaderStages::VERTEX,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Storage { read_only: true },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        },
    ];

    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("Frame Bind Group Layout (Unified)"),
        entries: &bindings,
    })
}

/// Create bind group layout for textures (group 1)
fn create_texture_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("Texture Bind Group Layout"),
        entries: &[
            // Slot 0: Albedo texture
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
            // Slot 1: MRE or Matcap
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            },
            // Slot 2: Environment matcap or Matcap
            wgpu::BindGroupLayoutEntry {
                binding: 2,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            },
            // Slot 3: Matcap (modes 1, 3)
            wgpu::BindGroupLayoutEntry {
                binding: 3,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            },
            // Sampler (nearest)
            wgpu::BindGroupLayoutEntry {
                binding: 4,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            },
            // Sampler (linear) - for per-draw filter selection via shading state flag
            wgpu::BindGroupLayoutEntry {
                binding: 5,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            },
        ],
    })
}

/// Cache for render pipelines
///
/// Stores compiled pipelines keyed by their render state configuration.
/// Pipelines are created on-demand and reused across frames.
/// Shader modules are precompiled at startup for all 40 permutations.
pub struct PipelineCache {
    pipelines: HashMap<PipelineKey, PipelineEntry>,
    /// Precompiled shader modules for all 40 mode/format combinations
    /// Index = mode * 16 + format for mode 0, or calculated index for modes 1-3
    shader_modules: Option<Vec<wgpu::ShaderModule>>,
}

impl PipelineCache {
    /// Create an empty pipeline cache
    pub fn new() -> Self {
        Self {
            pipelines: HashMap::new(),
            shader_modules: None,
        }
    }

    /// Precompile all 40 shader modules at startup
    ///
    /// This should be called during graphics initialization to ensure all shaders
    /// compile successfully. Panics on any shader compilation failure, indicating
    /// a bug in shader generation.
    pub fn precompile_all_shaders(&mut self, device: &wgpu::Device) {
        use crate::graphics::FORMAT_NORMAL;
        use crate::shader_gen::generate_shader;

        tracing::info!("Precompiling all 40 shader modules...");

        let mut modules = Vec::with_capacity(40);

        // Mode 0: 16 shaders (all formats)
        for format in 0u8..16 {
            let source = generate_shader(0, format)
                .expect("Mode 0 shader generation should succeed for all formats");
            let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some(&format!("Mode0_Format{}", format)),
                source: wgpu::ShaderSource::Wgsl(source.into()),
            });
            modules.push(module);
        }

        // Modes 1-3: 8 shaders each (only formats with NORMAL)
        for mode in 1u8..=3 {
            for format in (0u8..16).filter(|f| f & FORMAT_NORMAL != 0) {
                let source = generate_shader(mode, format).unwrap_or_else(|e| {
                    panic!(
                        "Mode {} format {} shader generation failed: {}",
                        mode, format, e
                    )
                });
                let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
                    label: Some(&format!("Mode{}_Format{}", mode, format)),
                    source: wgpu::ShaderSource::Wgsl(source.into()),
                });
                modules.push(module);
            }
        }

        assert_eq!(
            modules.len(),
            40,
            "Expected 40 shader modules, got {}",
            modules.len()
        );
        tracing::info!("Successfully precompiled all 40 shader modules");

        self.shader_modules = Some(modules);
    }

    /// Check if shaders have been precompiled
    #[allow(dead_code)] // Useful for testing/debugging
    pub fn shaders_precompiled(&self) -> bool {
        self.shader_modules.is_some()
    }

    /// Get or create a pipeline for the given state
    ///
    /// Returns a reference to the cached pipeline, creating it if necessary.
    pub fn get_or_create(
        &mut self,
        device: &wgpu::Device,
        surface_format: wgpu::TextureFormat,
        render_mode: u8,
        format: u8,
        state: &RenderState,
        pass_config: &PassConfig,
    ) -> &PipelineEntry {
        let key = PipelineKey::new(render_mode, format, state, pass_config);

        // Return existing pipeline if cached
        if self.pipelines.contains_key(&key) {
            return &self.pipelines[&key];
        }

        // Otherwise, create a new pipeline
        tracing::debug!(
            "Creating pipeline: mode={}, format={}, depth={}, cull={:?}, pass_config={:?}",
            render_mode,
            format,
            state.depth_test,
            state.cull_mode,
            pass_config
        );

        let entry = create_pipeline(
            device,
            surface_format,
            render_mode,
            format,
            state,
            pass_config,
        );
        self.pipelines.insert(key, entry);
        &self.pipelines[&key]
    }

    /// Check if a pipeline exists in the cache
    pub fn contains(
        &self,
        render_mode: u8,
        format: u8,
        state: &RenderState,
        pass_config: &PassConfig,
    ) -> bool {
        let key = PipelineKey::new(render_mode, format, state, pass_config);
        self.pipelines.contains_key(&key)
    }

    /// Get or create a quad pipeline
    ///
    /// Returns a reference to the cached quad pipeline, creating it if necessary.
    /// `is_screen_space` determines depth behavior:
    /// - true (screen-space): always writes depth at 0 for early-z optimization
    /// - false (billboard): uses PassConfig depth settings
    pub fn get_or_create_quad(
        &mut self,
        device: &wgpu::Device,
        surface_format: wgpu::TextureFormat,
        state: &RenderState,
        pass_config: &PassConfig,
        is_screen_space: bool,
    ) -> &PipelineEntry {
        let key = PipelineKey::quad(state, pass_config, is_screen_space);

        // Return existing pipeline if cached
        if self.pipelines.contains_key(&key) {
            return &self.pipelines[&key];
        }

        // Otherwise, create a new quad pipeline
        tracing::debug!(
            "Creating quad pipeline: depth={}, is_screen_space={}, pass_config={:?}",
            state.depth_test,
            is_screen_space,
            pass_config
        );

        let entry = create_quad_pipeline(device, surface_format, pass_config, is_screen_space);
        self.pipelines.insert(key, entry);
        &self.pipelines[&key]
    }

    /// Get or create a sky pipeline
    ///
    /// Returns a reference to the cached sky pipeline, creating it if necessary.
    pub fn get_or_create_sky(
        &mut self,
        device: &wgpu::Device,
        surface_format: wgpu::TextureFormat,
        pass_config: &PassConfig,
    ) -> &PipelineEntry {
        let key = PipelineKey::sky(pass_config);

        // Return existing pipeline if cached
        if self.pipelines.contains_key(&key) {
            return &self.pipelines[&key];
        }

        // Otherwise, create a new sky pipeline
        tracing::debug!("Creating sky pipeline: pass_config={:?}", pass_config);

        let entry = create_sky_pipeline(device, surface_format, pass_config);
        self.pipelines.insert(key, entry);
        &self.pipelines[&key]
    }

    /// Get a pipeline by key (works for both Regular and Quad)
    pub fn get_by_key(&self, key: &PipelineKey) -> Option<&PipelineEntry> {
        self.pipelines.get(key)
    }
}

impl Default for PipelineCache {
    fn default() -> Self {
        Self::new()
    }
}
