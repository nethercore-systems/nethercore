//! Shared types and utilities for Nethercore ZX console
//!
//! This crate provides ZX-specific utilities shared between:
//! - `nethercore-zx` (runtime)
//! - `nether-export` (asset pipeline)
//! - `nether-cli` (build tools)
//!
//! # Modules
//!
//! - [`packing`] - Vertex data packing utilities (f32 → f16/snorm16/unorm8)
//! - [`formats`] - ZX-specific binary asset and ROM formats
//! - [`loader`] - ROM loader for Nethercore ZX ROM files

pub mod formats;
#[cfg(feature = "loader")]
pub mod loader;
pub mod packing;

// Re-export the ROM loader
#[cfg(feature = "loader")]
pub use loader::ZXRomLoader;

// Re-export commonly used packing items
pub use packing::{
    FORMAT_COLOR, FORMAT_NORMAL, FORMAT_SKINNED, FORMAT_TANGENT, FORMAT_UV, encode_octahedral,
    pack_bone_weights_unorm8, pack_color_rgba_unorm8, pack_normal_octahedral, pack_normal_snorm16,
    pack_octahedral_u16, pack_octahedral_u32, pack_position_f16, pack_tangent, pack_tangent_f32x4,
    pack_uv_f16, pack_uv_unorm16, pack_vertex_data, unpack_octahedral_u16, unpack_octahedral_u32,
    unpack_tangent, vertex_stride, vertex_stride_packed,
};

// Re-export commonly used format items
pub use formats::{
    BONE_TRANSFORM_SIZE,
    BoneTransform,
    INVERSE_BIND_MATRIX_SIZE,
    NetherZXAnimationHeader,
    // Mesh/texture/skeleton types
    NetherZXMeshHeader,
    NetherZXSkeletonHeader,
    NetherZXTextureHeader,
    PLATFORM_BONE_KEYFRAME_SIZE,
    // Data pack types
    PackedData,
    PackedFont,
    PackedGlyph,
    PackedKeyframes,
    PackedMesh,
    PackedSkeleton,
    PackedSound,
    PackedTexture,
    PackedTracker,
    TrackerFormat,
    PlatformBoneKeyframe,
    // ROM format constants (from nethercore_shared)
    RomFormat,
    SAMPLE_RATE,
    TextureFormat,
    // ROM types
    ZMetadata,
    ZX_ROM_FORMAT,
    ZXDataPack,
    ZXRom,
    // Animation types
    decode_bone_transform,
    decode_quat_smallest_three,
    encode_bone_transform,
    encode_quat_smallest_three,
    f16_to_f32,
    f32_to_f16,
};
