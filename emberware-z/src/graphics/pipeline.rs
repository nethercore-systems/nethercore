//! Shader pipeline management
//!
//! Handles shader compilation, pipeline caching, and bind group layout creation
//! for all render mode and vertex format combinations.

use hashbrown::HashMap;

use super::render_state::RenderState;
use super::vertex::VertexFormatInfo;

/// Key for pipeline cache lookup
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum PipelineKey {
    /// Regular mesh rendering pipeline
    Regular {
        render_mode: u8,
        vertex_format: u8,
        blend_mode: u8,
        depth_test: bool,
        cull_mode: u8,
    },
    /// GPU-instanced quad rendering pipeline (billboards, sprites)
    Quad { blend_mode: u8, depth_test: bool },
    /// Procedural sky rendering pipeline
    Sky,
}

impl PipelineKey {
    /// Create a new regular pipeline key from render state
    pub fn new(render_mode: u8, format: u8, state: &RenderState) -> Self {
        Self::Regular {
            render_mode,
            vertex_format: format,
            blend_mode: state.blend_mode as u8,
            depth_test: state.depth_test,
            cull_mode: state.cull_mode as u8,
        }
    }

    /// Create a quad pipeline key
    pub fn quad(state: &RenderState) -> Self {
        Self::Quad {
            blend_mode: state.blend_mode as u8,
            depth_test: state.depth_test,
        }
    }
}

/// Cached pipeline entry with bind group layouts
pub(crate) struct PipelineEntry {
    pub pipeline: wgpu::RenderPipeline,
    pub bind_group_layout_frame: wgpu::BindGroupLayout,
    pub bind_group_layout_textures: wgpu::BindGroupLayout,
}

/// Create a new pipeline for the given vertex format and render state
pub(crate) fn create_pipeline(
    device: &wgpu::Device,
    surface_format: wgpu::TextureFormat,
    render_mode: u8,
    format: u8,
    state: &RenderState,
) -> PipelineEntry {
    use crate::shader_gen::generate_shader;

    // Generate shader source, falling back to Mode 0 if the requested mode/format is invalid
    let shader_source = match generate_shader(render_mode, format) {
        Ok(source) => source,
        Err(e) => {
            tracing::warn!(
                "Shader generation failed for mode {} format {}: {}. Falling back to Mode 0 (unlit).",
                render_mode,
                format,
                e
            );
            // Fallback to Mode 0 (unlit) which supports all formats
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
                blend: state.blend_mode.to_wgpu(),
                write_mask: wgpu::ColorWrites::ALL,
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
            format: wgpu::TextureFormat::Depth32Float,
            depth_write_enabled: state.depth_test,
            depth_compare: if state.depth_test {
                wgpu::CompareFunction::Less
            } else {
                wgpu::CompareFunction::Always
            },
            stencil: wgpu::StencilState::default(),
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
pub(crate) fn create_quad_pipeline(
    device: &wgpu::Device,
    surface_format: wgpu::TextureFormat,
    state: &RenderState,
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

    // Create render pipeline
    let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Quad Pipeline"),
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
                blend: state.blend_mode.to_wgpu(),
                write_mask: wgpu::ColorWrites::ALL,
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
            format: wgpu::TextureFormat::Depth32Float,
            depth_write_enabled: state.depth_test,
            depth_compare: if state.depth_test {
                wgpu::CompareFunction::Less
            } else {
                wgpu::CompareFunction::Always
            },
            stencil: wgpu::StencilState::default(),
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
                write_mask: wgpu::ColorWrites::ALL,
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
            format: wgpu::TextureFormat::Depth32Float,
            depth_write_enabled: false, // Don't write depth
            depth_compare: wgpu::CompareFunction::Always, // Always pass
            stencil: wgpu::StencilState::default(),
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
/// Unified layout for all render modes (0-3)
fn create_frame_bind_group_layout(
    device: &wgpu::Device,
    _render_mode: u8,
) -> wgpu::BindGroupLayout {
    // Unified binding layout (0-5) - same for all modes
    let bindings = vec![
        // Binding 0: Model matrices storage buffer (per-frame array)
        wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::VERTEX,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Storage { read_only: true },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        },
        // Binding 1: View matrices storage buffer (per-frame array)
        // VERTEX_FRAGMENT: sky shader computes view rays per-pixel in fragment
        wgpu::BindGroupLayoutEntry {
            binding: 1,
            visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Storage { read_only: true },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        },
        // Binding 2: Projection matrices storage buffer (per-frame array)
        // VERTEX_FRAGMENT: sky shader extracts FOV in fragment
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
        // Binding 3: Shading states storage buffer (per-draw shading state array)
        wgpu::BindGroupLayoutEntry {
            binding: 3,
            visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Storage { read_only: true },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        },
        // Binding 4: MVP + shading indices storage buffer (vec2<u32>: packed MVP + shading state index)
        wgpu::BindGroupLayoutEntry {
            binding: 4,
            visibility: wgpu::ShaderStages::VERTEX,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Storage { read_only: true },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        },
        // Binding 5: Bone storage buffer for GPU skinning
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
        // Binding 6: Inverse bind storage buffer for skeletal animation
        wgpu::BindGroupLayoutEntry {
            binding: 6,
            visibility: wgpu::ShaderStages::VERTEX,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Storage { read_only: true },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        },
        // Binding 7: Quad instances storage buffer (for GPU-instanced quad rendering)
        wgpu::BindGroupLayoutEntry {
            binding: 7,
            visibility: wgpu::ShaderStages::VERTEX,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Storage { read_only: true },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        },
        // Binding 8: Screen dimensions uniform (for screen-space quad NDC conversion)
        wgpu::BindGroupLayoutEntry {
            binding: 8,
            visibility: wgpu::ShaderStages::VERTEX,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: wgpu::BufferSize::new(8), // 2 × f32 = 8 bytes
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
    ) -> &PipelineEntry {
        let key = PipelineKey::new(render_mode, format, state);

        // Return existing pipeline if cached
        if self.pipelines.contains_key(&key) {
            return &self.pipelines[&key];
        }

        // Otherwise, create a new pipeline
        tracing::debug!(
            "Creating pipeline: mode={}, format={}, blend={:?}, depth={}, cull={:?}",
            render_mode,
            format,
            state.blend_mode,
            state.depth_test,
            state.cull_mode
        );

        let entry = create_pipeline(device, surface_format, render_mode, format, state);
        self.pipelines.insert(key, entry);
        &self.pipelines[&key]
    }

    /// Check if a pipeline exists in the cache
    pub fn contains(&self, render_mode: u8, format: u8, state: &RenderState) -> bool {
        let key = PipelineKey::new(render_mode, format, state);
        self.pipelines.contains_key(&key)
    }

    /// Get or create a quad pipeline
    ///
    /// Returns a reference to the cached quad pipeline, creating it if necessary.
    pub fn get_or_create_quad(
        &mut self,
        device: &wgpu::Device,
        surface_format: wgpu::TextureFormat,
        state: &RenderState,
    ) -> &PipelineEntry {
        let key = PipelineKey::quad(state);

        // Return existing pipeline if cached
        if self.pipelines.contains_key(&key) {
            return &self.pipelines[&key];
        }

        // Otherwise, create a new quad pipeline
        tracing::debug!(
            "Creating quad pipeline: blend={:?}, depth={}",
            state.blend_mode,
            state.depth_test
        );

        let entry = create_quad_pipeline(device, surface_format, state);
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
    ) -> &PipelineEntry {
        let key = PipelineKey::Sky;

        // Return existing pipeline if cached
        if self.pipelines.contains_key(&key) {
            return &self.pipelines[&key];
        }

        // Otherwise, create a new sky pipeline
        tracing::debug!("Creating sky pipeline");

        let entry = create_sky_pipeline(device, surface_format);
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
