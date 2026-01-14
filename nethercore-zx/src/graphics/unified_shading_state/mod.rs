// ============================================================================
// Unified Shading State Module
// ============================================================================
//
// This module contains all types and utilities for the unified shading system,
// including environment states, lights, and per-draw shading configurations.
//
// The module is organized into the following submodules:
// - environment: PackedEnvironmentState and environment mode configurations
// - light: PackedLight and light types (directional, point)
// - quantization: Helper functions for packing/unpacking data
// - shading_state: PackedUnifiedShadingState and flags
// - tests: Comprehensive test suite

mod environment;
mod light;
mod quantization;
mod shading_state;

#[cfg(test)]
mod tests;

// Re-export all public types and functions to preserve API
pub use environment::{
    blend_mode, env_mode, CurtainsConfig, EnvironmentIndex, GradientConfig, LinesConfig,
    PackedEnvironmentState, RectanglesConfig, RingsConfig, RoomConfig, ScatterConfig,
    SilhouetteConfig,
};

pub use light::{LightType, PackedLight};

// Note: Some exports may appear unused in lib but are used by FFI/external code
#[allow(unused_imports)]
pub use quantization::{
    pack_f16, pack_f16x2, pack_matcap_blend_modes, pack_rgb8, pack_rgba8, pack_snorm16,
    pack_uniform_set_0, pack_uniform_set_1, pack_unorm8, unpack_f16, unpack_f16x2,
    unpack_matcap_blend_modes, unpack_snorm16, unpack_unorm8, update_u32_byte,
    update_uniform_set_0_byte, update_uniform_set_1_byte,
};

pub use shading_state::{
    PackedUnifiedShadingState, ShadingStateIndex, DEFAULT_FLAGS, FLAG_DITHER_OFFSET_X_MASK,
    FLAG_DITHER_OFFSET_X_SHIFT, FLAG_DITHER_OFFSET_Y_MASK, FLAG_DITHER_OFFSET_Y_SHIFT,
    FLAG_SKIP_NORMAL_MAP, FLAG_SKINNING_MODE, FLAG_TEXTURE_FILTER_LINEAR,
    FLAG_UNIFORM_ALPHA_MASK, FLAG_UNIFORM_ALPHA_SHIFT, FLAG_USE_MATCAP_REFLECTION,
    FLAG_USE_UNIFORM_COLOR, FLAG_USE_UNIFORM_EMISSIVE, FLAG_USE_UNIFORM_METALLIC,
    FLAG_USE_UNIFORM_ROUGHNESS, FLAG_USE_UNIFORM_SPECULAR,
};
