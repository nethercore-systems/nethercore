//! Shared types and utilities for Emberware Z console
//!
//! This crate provides Z-specific utilities shared between:
//! - `emberware-z` (runtime)
//! - `ember-export` (asset pipeline)
//! - `ember-cli` (build tools)
//!
//! # Modules
//!
//! - [`packing`] - Vertex data packing utilities (f32 → f16/snorm16/unorm8)
//! - [`formats`] - Z-specific binary asset and ROM formats

pub mod formats;
pub mod packing;

// Re-export commonly used packing items
pub use packing::{
    encode_octahedral, pack_bone_weights_unorm8, pack_color_rgba_unorm8, pack_normal_octahedral,
    pack_normal_snorm16, pack_octahedral_u32, pack_position_f16, pack_uv_f16, pack_uv_unorm16,
    pack_vertex_data, unpack_octahedral_u32, vertex_stride, vertex_stride_packed, FORMAT_COLOR,
    FORMAT_NORMAL, FORMAT_SKINNED, FORMAT_UV,
};

// Re-export commonly used format items
pub use formats::{
    EmberZAnimationHeader, EmberZMeshHeader, EmberZSkeletonHeader, EmberZSoundHeader,
    EmberZTextureHeader, PackedData, PackedFont, PackedGlyph, PackedMesh, PackedSkeleton,
    PackedSound, PackedTexture, TextureFormat, ZDataPack, ZMetadata, ZRom, BONE_TRANSFORM_SIZE,
    EWZ_ANIMATION_EXT, EWZ_MAGIC, EWZ_MESH_EXT, EWZ_SKELETON_EXT, EWZ_SOUND_EXT, EWZ_TEXTURE_EXT,
    EWZ_VERSION, INVERSE_BIND_MATRIX_SIZE, SAMPLE_RATE,
};
