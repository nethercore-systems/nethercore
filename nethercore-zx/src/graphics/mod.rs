//! Nethercore ZX graphics backend (wgpu)
//!
//! Implements the `Graphics` trait from nethercore-core with a wgpu-based
//! renderer featuring PS1/N64 aesthetic (vertex jitter, affine textures).
//!
//! # Architecture
//!
//! **ZXFFIState** (staging) → **ZXGraphics** (GPU execution)
//!
//! - FFI functions write draw commands, transforms, and render state to ZXFFIState
//! - App.rs passes ZXFFIState to ZXGraphics each frame
//! - ZXGraphics consumes commands and executes them on the GPU
//! - ZXGraphics owns all actual GPU resources (textures, meshes, buffers, pipelines)

mod buffer;
mod command_buffer;
mod draw;
pub mod epu;
mod frame;
mod init;
mod matrix_packing;
mod pipeline;
mod quad_instance;
mod render_state;
mod texture_manager;
mod trait_impls;
pub(crate) mod unified_shading_state;
mod vertex;
mod viewport;
mod zx_graphics;

// Re-export packing utilities from zx-common (for FFI and tooling)
pub use zx_common::{
    FORMAT_COLOR, FORMAT_NORMAL, FORMAT_SKINNED, FORMAT_TANGENT, FORMAT_UV,
    pack_bone_weights_unorm8, pack_color_rgba_unorm8, pack_normal_octahedral, pack_normal_snorm16,
    pack_octahedral_u32, pack_position_f16, pack_tangent, pack_uv_f16, pack_uv_unorm16,
    pack_vertex_data, unpack_octahedral_u32, unpack_tangent, vertex_stride, vertex_stride_packed,
};

// Re-export public types from submodules
pub use buffer::{BufferManager, GrowableBuffer, MeshHandle, RetainedMesh};
pub use command_buffer::{CommandSortKey, VRPCommand, VirtualRenderPass};
pub use matrix_packing::MvpShadingIndices;
pub use quad_instance::{QuadInstance, QuadMode};
pub use render_state::{
    CullMode, MatcapBlendMode, PassConfig, RenderState, TextureFilter, TextureHandle,
};
pub use unified_shading_state::{
    DEFAULT_FLAGS, FLAG_DITHER_OFFSET_X_MASK, FLAG_DITHER_OFFSET_X_SHIFT,
    FLAG_DITHER_OFFSET_Y_MASK, FLAG_DITHER_OFFSET_Y_SHIFT, FLAG_SKINNING_MODE,
    FLAG_SKIP_NORMAL_MAP, FLAG_TEXTURE_FILTER_LINEAR, FLAG_UNIFORM_ALPHA_MASK,
    FLAG_UNIFORM_ALPHA_SHIFT, FLAG_USE_MATCAP_REFLECTION, FLAG_USE_UNIFORM_COLOR,
    FLAG_USE_UNIFORM_EMISSIVE, FLAG_USE_UNIFORM_METALLIC, FLAG_USE_UNIFORM_ROUGHNESS,
    FLAG_USE_UNIFORM_SPECULAR, LightType, PackedLight, PackedUnifiedShadingState,
    ShadingStateIndex, pack_f16, pack_f16x2, pack_matcap_blend_modes, pack_rgb8, pack_unorm8,
    unpack_f16, unpack_f16x2, unpack_matcap_blend_modes, update_u32_byte,
};
pub use vertex::{FORMAT_ALL, VERTEX_FORMAT_COUNT, VertexFormatInfo};
pub use viewport::Viewport;
pub use zx_graphics::ZXGraphics;

// =============================================================================
// QUAD BATCH INFO (for GPU draw command creation)
// =============================================================================

/// Temporary data for processing a batch of quads during frame rendering.
///
/// This is an internal scratch structure used between quad instance upload
/// and draw command creation. It captures all the state needed to create
/// VRPCommand::Quad draw calls.
#[derive(Debug, Clone, Copy)]
pub(crate) struct QuadBatchInfo {
    /// Starting instance index in the instance buffer
    pub base_instance: u32,
    /// Number of quad instances in this batch
    pub instance_count: u32,
    /// FFI texture handles (resolved to TextureHandle during command creation)
    pub textures: [u32; 4],
    /// True if this batch contains screen-space quads (2D)
    pub is_screen_space: bool,
    /// Viewport for this batch
    pub viewport: Viewport,
    /// Pass ID for render pass ordering (execution barrier)
    pub pass_id: u32,
    /// Z-index for 2D ordering within a pass (higher = closer to camera)
    pub z_index: u32,
}
