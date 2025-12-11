//! Shared types and utilities for Emberware Z console
//!
//! This crate provides Z-specific utilities shared between:
//! - `emberware-z` (runtime)
//! - `ember-export` (asset pipeline)
//!
//! # Modules
//!
//! - [`packing`] - Vertex data packing utilities (f32 → f16/snorm16/unorm8)

pub mod packing;

// Re-export commonly used items
pub use packing::{
    FORMAT_COLOR, FORMAT_NORMAL, FORMAT_SKINNED, FORMAT_UV, encode_octahedral,
    pack_bone_weights_unorm8, pack_color_rgba_unorm8, pack_normal_octahedral, pack_normal_snorm16,
    pack_octahedral_u32, pack_position_f16, pack_uv_f16, pack_uv_unorm16, pack_vertex_data,
    unpack_octahedral_u32, vertex_stride, vertex_stride_packed,
};
