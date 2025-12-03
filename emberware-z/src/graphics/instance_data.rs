//! Instance data for per-draw instanced rendering
//!
//! Each draw command gets one instance with its MVP and shading state indices.
//! This replaces the previous approach of using storage buffers for indices.

/// Per-instance data passed as vertex attributes
///
/// This structure is uploaded to a vertex buffer and bound with step_mode: Instance.
/// Each draw command gets exactly one instance containing its indices.
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct InstanceData {
    /// Packed MVP index (model: 16 bits, view: 8 bits, proj: 8 bits)
    pub mvp_index: u32,
    /// Index into shading states array
    pub shading_index: u32,
}

impl InstanceData {
    /// Create new instance data
    pub fn new(mvp_index: u32, shading_index: u32) -> Self {
        Self {
            mvp_index,
            shading_index,
        }
    }

    /// Get the vertex buffer layout for instance data
    pub fn layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<InstanceData>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                // mvp_index at location 10
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 10,
                    format: wgpu::VertexFormat::Uint32,
                },
                // shading_index at location 11
                wgpu::VertexAttribute {
                    offset: 4,
                    shader_location: 11,
                    format: wgpu::VertexFormat::Uint32,
                },
            ],
        }
    }
}
