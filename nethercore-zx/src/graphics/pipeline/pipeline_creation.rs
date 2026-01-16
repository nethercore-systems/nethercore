//! Pipeline creation functions
//!
//! Functions for creating render pipelines for different rendering modes.

use super::super::render_state::{PassConfig, RenderState};
use super::super::vertex::VertexFormatInfo;
use super::bind_groups::{create_frame_bind_group_layout, create_texture_bind_group_layout};

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
    use super::super::vertex::{FORMAT_COLOR, FORMAT_UV};
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
        label: Some(if is_screen_space {
            "Screen-Space Quad Pipeline"
        } else {
            "Billboard Pipeline"
        }),
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

/// Create environment rendering pipeline for fullscreen procedural environment
pub(crate) fn create_environment_pipeline(
    device: &wgpu::Device,
    surface_format: wgpu::TextureFormat,
    pass_config: &PassConfig,
) -> PipelineEntry {
    // Load environment shader (generated from the common WGSL sources + env_template.wgsl by build.rs)
    use crate::shader_gen::ENVIRONMENT_SHADER;

    // Create shader module
    let shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Environment Shader"),
        source: wgpu::ShaderSource::Wgsl(ENVIRONMENT_SHADER.into()),
    });

    // Create bind group layouts (same as other pipelines)
    let bind_group_layout_frame = create_frame_bind_group_layout(device, 0);
    let bind_group_layout_textures = create_texture_bind_group_layout(device);

    // Create pipeline layout
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Environment Pipeline Layout"),
        bind_group_layouts: &[&bind_group_layout_frame, &bind_group_layout_textures],
        push_constant_ranges: &[],
    });

    // Create render pipeline
    let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Environment Pipeline"),
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
            depth_write_enabled: false, // Environment is infinitely far, don't write depth
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
