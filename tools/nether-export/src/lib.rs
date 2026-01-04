//! nether-export library
//!
//! Provides asset conversion functions for use by other tools (e.g., nether-cli pack command).

pub mod animation;
pub mod audio;
pub mod codegen;
pub mod formats;
pub mod manifest;
pub mod mesh;
pub mod skeleton;
pub mod texture;

// Re-export packing functions and vertex format constants from zx-common
pub use zx_common::{
    pack_bone_weights_unorm8, pack_color_rgba_unorm8, pack_normal_octahedral, pack_normal_snorm16,
    pack_position_f16, pack_tangent_f32x4, pack_uv_f16, pack_uv_unorm16, pack_vertex_data,
    unpack_octahedral_u32, vertex_stride, vertex_stride_packed, FORMAT_COLOR, FORMAT_NORMAL,
    FORMAT_SKINNED, FORMAT_TANGENT, FORMAT_UV,
};

// Re-export ROM format from shared (includes all extension constants)
pub use nethercore_shared::RomFormat;

// Re-export key types for mesh conversion
pub use mesh::{convert_gltf_to_memory, convert_obj_to_memory, ConvertedMesh};

// Re-export skeleton conversion types
pub use skeleton::{convert_gltf_skeleton_to_memory, ConvertedSkeleton};

// Re-export animation conversion types
pub use animation::{
    convert_gltf_animation_to_memory, get_animation_list, AnimationInfo, ConvertedAnimation,
};
