//! Bind group layout creation
//!
//! Creates bind group layouts for frame uniforms and textures.

/// Create bind group layout for per-frame uniforms (group 0)
///
/// UNIFIED BUFFER ARCHITECTURE (grouped by purpose):
/// - Binding 0-1: Transforms (unified_transforms, mvp_indices)
/// - Binding 2: Shading (shading_states)
/// - Binding 3: Animation (unified_animation)
/// - Binding 5: Quad rendering (quad_instances)
/// - Binding 6-7: EPU textures (env_radiance, sampler)
/// - Binding 8-9: EPU state + frame uniforms
/// - Binding 11: EPU SH9 (diffuse irradiance)
///
/// CPU pre-computes absolute indices into unified_transforms (no frame_offsets needed).
/// Screen dimensions eliminated - resolution_index packed into QuadInstance.mode.
pub(crate) fn create_frame_bind_group_layout(
    device: &wgpu::Device,
    _render_mode: u8,
) -> wgpu::BindGroupLayout {
    let bindings = vec![
        // =====================================================================
        // TRANSFORMS (bindings 0-1)
        // =====================================================================

        // Binding 0: unified_transforms - all mat4x4 matrices [models | views | projs]
        // VERTEX_FRAGMENT: environment shader needs view/proj matrices in fragment
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
        // =====================================================================
        // EPU TEXTURES (bindings 6-7)
        // =====================================================================

        // Binding 6: EPU EnvRadiance texture array (octahedral, 256 layers)
        // Mip-mapped; used for background + roughness-based reflection sampling.
        wgpu::BindGroupLayoutEntry {
            binding: 6,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Texture {
                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                view_dimension: wgpu::TextureViewDimension::D2Array,
                multisampled: false,
            },
            count: None,
        },
        // Binding 7: EPU linear sampler for environment map sampling
        wgpu::BindGroupLayoutEntry {
            binding: 7,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
            count: None,
        },
        // =====================================================================
        // EPU STATE + FRAME UNIFORMS (bindings 8-9)
        // =====================================================================

        // Binding 8: Packed EPU environment states (storage, read-only)
        // Used for procedural sky/background and specular residual evaluation.
        wgpu::BindGroupLayoutEntry {
            binding: 8,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Storage { read_only: true },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        },
        // Binding 9: EPU frame uniforms (uniform)
        wgpu::BindGroupLayoutEntry {
            binding: 9,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        },
        // =====================================================================
        // EPU SH9 (binding 11)
        // =====================================================================

        // Binding 11: EPU SH9 storage buffer (256 entries, 144 bytes each)
        // Pre-computed L2 (9 coefficient) diffuse irradiance extracted from a coarse radiance mip.
        wgpu::BindGroupLayoutEntry {
            binding: 11,
            visibility: wgpu::ShaderStages::FRAGMENT,
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
pub(crate) fn create_texture_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
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
