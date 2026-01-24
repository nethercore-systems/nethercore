//! Shader generation system for Nethercore ZX
//!
//! All 72 shader permutations are pregenerated at build time by `build.rs` and validated
//! with naga. This module provides access to the pregenerated shaders.
//!
//! - Render mode (0-3): Lambert, Matcap, MR-Blinn-Phong, Blinn-Phong
//! - Vertex format flags (UV, COLOR, NORMAL, SKINNED, TANGENT)
//!
//! Total shader count: 72
//! - Mode 0: 24 shaders (all vertex formats, tangent requires normal)
//! - Modes 1-3: 16 shaders each (formats with NORMAL, optionally TANGENT)
//!
//! TANGENT flag (bit 4) is only valid when NORMAL flag (bit 2) is also set,
//! as tangent-space normal mapping requires vertex normals for TBN construction.
//!
//! Additionally, `ENVIRONMENT_SHADER` and `QUAD_SHADER` are generated from templates
//! (env_template.wgsl, quad_template.wgsl) combined with the shared common WGSL utilities.

mod error;
mod formats;
mod pregenerated;
#[cfg(test)]
mod templates;

pub use error::ShaderGenError;
#[allow(unused_imports)] // Re-exported for debugging/tests; may be unused within this module.
pub use formats::{mode_name, valid_formats_for_mode};
#[cfg(test)]
pub use formats::shader_count_for_mode;
#[allow(unused_imports)] // Re-exported for debugging/tests; may be unused within this module.
pub use pregenerated::{
    ENVIRONMENT_SHADER, PREGENERATED_SHADERS, QUAD_SHADER, get_pregenerated_shader,
};
#[cfg(test)]
pub use templates::get_template;

use crate::graphics::FORMAT_NORMAL;

/// Get a pregenerated shader for a specific mode and vertex format
///
/// All shaders are pregenerated at build time and validated with naga.
/// This function returns the pregenerated shader source as a `&'static str`.
///
/// # Errors
///
/// Returns `ShaderGenError::InvalidRenderMode` if mode is not 0-3.
/// Returns `ShaderGenError::MissingNormalFlag` if modes 1-3 are used without NORMAL flag.
pub fn generate_shader(mode: u8, format: u8) -> Result<&'static str, ShaderGenError> {
    // Validate mode
    if mode > 3 {
        return Err(ShaderGenError::InvalidRenderMode(mode));
    }

    // Check if this format is valid for the mode
    // Modes 1-3 require normals
    let has_normal = format & FORMAT_NORMAL != 0;
    if mode > 0 && !has_normal {
        return Err(ShaderGenError::MissingNormalFlag { mode, format });
    }

    // Use pregenerated shader (validated at build time)
    let source = get_pregenerated_shader(mode, format)
        .expect("Pregenerated shader missing - this indicates a bug in build.rs");

    Ok(source)
}

#[cfg(test)]
mod tests;
