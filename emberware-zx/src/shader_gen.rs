//! Shader generation system for Emberware Z
//!
//! All 40 shader permutations are pregenerated at build time by build.rs and validated
//! with naga. This module provides access to the pregenerated shaders.
//!
//! - Render mode (0-3): Lambert, Matcap, MR-Blinn-Phong, Blinn-Phong
//! - Vertex format flags (UV, COLOR, NORMAL, SKINNED)
//!
//! Total shader count: 40
//! - Mode 0: 16 shaders (all vertex formats)
//! - Modes 1-3: 8 shaders each (only formats with NORMAL flag)
//!
//! Additionally, SKY_SHADER and QUAD_SHADER are generated from templates
//! (sky_template.wgsl, quad_template.wgsl) combined with common.wgsl utilities.

use crate::graphics::FORMAT_NORMAL;
use std::fmt;

// Include pregenerated shaders from build.rs
// This provides: PREGENERATED_SHADERS, get_pregenerated_shader(), SKY_SHADER, QUAD_SHADER
include!(concat!(env!("OUT_DIR"), "/generated_shaders.rs"));

// ============================================================================
// Error Types
// ============================================================================

/// Error type for shader generation failures
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ShaderGenError {
    /// Invalid render mode (must be 0-3)
    InvalidRenderMode(u8),
    /// Render mode requires NORMAL flag but format doesn't have it
    MissingNormalFlag { mode: u8, format: u8 },
}

impl fmt::Display for ShaderGenError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ShaderGenError::InvalidRenderMode(mode) => {
                write!(f, "Invalid render mode: {} (must be 0-3)", mode)
            }
            ShaderGenError::MissingNormalFlag { mode, format } => {
                write!(
                    f,
                    "Render mode {} requires NORMAL flag, but format {} doesn't have it",
                    mode, format
                )
            }
        }
    }
}

impl std::error::Error for ShaderGenError {}

// ============================================================================
// Shader Templates (embedded for inspection/debugging via get_template())
// ============================================================================

const BLINNPHONG_COMMON: &str = include_str!("../shaders/blinnphong_common.wgsl");
const TEMPLATE_MODE0: &str = include_str!("../shaders/mode0_unlit.wgsl");
const TEMPLATE_MODE1: &str = include_str!("../shaders/mode1_matcap.wgsl");

// ============================================================================
// Shader Generation
// ============================================================================

/// Get a pregenerated shader for a specific mode and vertex format
///
/// All shaders are pregenerated at build time and validated with naga.
/// This function returns the pregenerated shader source as a &'static str.
///
/// # Errors
///
/// Returns `ShaderGenError::InvalidRenderMode` if mode is not 0-3.
/// Returns `ShaderGenError::MissingNormalFlag` if modes 1-3 are used without NORMAL flag.
pub fn generate_shader(mode: u8, format: u8) -> Result<String, ShaderGenError> {
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

    Ok(source.to_string())
}

/// Get the template for a given render mode (for debugging/inspection)
///
/// Note: Modes 2-3 don't have separate templates; they're generated from blinnphong_common.
///
/// # Errors
///
/// Returns `ShaderGenError::InvalidRenderMode` if mode is not 0-3.
#[allow(dead_code)] // Debugging/inspection helper
pub fn get_template(mode: u8) -> Result<&'static str, ShaderGenError> {
    match mode {
        0 => Ok(TEMPLATE_MODE0),
        1 => Ok(TEMPLATE_MODE1),
        2 => Ok(BLINNPHONG_COMMON), // Generated from common files
        3 => Ok(BLINNPHONG_COMMON), // Generated from common files
        _ => Err(ShaderGenError::InvalidRenderMode(mode)),
    }
}

/// Get human-readable name for a render mode
pub fn mode_name(mode: u8) -> &'static str {
    match mode {
        0 => "Lambert",
        1 => "Matcap",
        2 => "MR-Blinn-Phong",
        3 => "Blinn-Phong",
        _ => "Unknown",
    }
}

/// Get the number of shader permutations for a render mode
#[allow(dead_code)] // Debugging/testing helper
pub fn shader_count_for_mode(mode: u8) -> usize {
    match mode {
        0 => 16,    // All vertex formats
        1..=3 => 8, // Only formats with NORMAL
        _ => 0,
    }
}

/// Get all valid vertex formats for a render mode
#[allow(dead_code)] // Debugging/testing helper
pub fn valid_formats_for_mode(mode: u8) -> Vec<u8> {
    match mode {
        0 => (0..16).collect(), // All formats
        1..=3 => {
            // Only formats with NORMAL flag (formats 4-7 and 12-15)
            (0..16).filter(|&f| f & FORMAT_NORMAL != 0).collect()
        }
        _ => vec![],
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graphics::{FORMAT_COLOR, FORMAT_UV};

    #[test]
    fn test_shader_generation_mode0() {
        // Mode 0 should support all 16 formats
        for format in 0..16 {
            let shader = generate_shader(0, format).expect("Mode 0 should support all formats");
            assert!(!shader.is_empty());
            assert!(shader.contains("@vertex"));
            assert!(shader.contains("@fragment"));
        }
    }

    #[test]
    fn test_shader_generation_mode1() {
        // Mode 1 should only support formats with NORMAL
        for format in valid_formats_for_mode(1) {
            let shader =
                generate_shader(1, format).expect("Mode 1 should support formats with NORMAL");
            assert!(!shader.is_empty());
            assert!(shader.contains("matcap"));
        }
    }

    #[test]
    fn test_mode1_without_normals_returns_error() {
        // Mode 1 without normals should return an error
        let result = generate_shader(1, 0); // Format 0 has no NORMAL flag
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            ShaderGenError::MissingNormalFlag { mode: 1, format: 0 }
        );
    }

    #[test]
    fn test_invalid_render_mode_returns_error() {
        // Invalid render modes should return an error
        let result = generate_shader(4, 0);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), ShaderGenError::InvalidRenderMode(4));

        let result = generate_shader(255, 0);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), ShaderGenError::InvalidRenderMode(255));
    }

    #[test]
    fn test_get_template_returns_error_for_invalid_mode() {
        assert!(get_template(0).is_ok());
        assert!(get_template(3).is_ok());
        assert_eq!(
            get_template(4).unwrap_err(),
            ShaderGenError::InvalidRenderMode(4)
        );
        assert_eq!(
            get_template(100).unwrap_err(),
            ShaderGenError::InvalidRenderMode(100)
        );
    }

    #[test]
    fn test_placeholder_replacement() {
        // Test that placeholders are replaced correctly
        let shader = generate_shader(0, FORMAT_UV | FORMAT_COLOR).unwrap();
        assert!(shader.contains("@location(1) uv"));
        assert!(shader.contains("@location(2) color"));
        assert!(!shader.contains("//VIN_UV"));
        assert!(!shader.contains("//VIN_COLOR"));
    }

    #[test]
    fn test_shader_counts() {
        assert_eq!(shader_count_for_mode(0), 16);
        assert_eq!(shader_count_for_mode(1), 8);
        assert_eq!(shader_count_for_mode(2), 8);
        assert_eq!(shader_count_for_mode(3), 8);
    }

    #[test]
    fn test_total_shader_count() {
        let total: usize = (0..4).map(shader_count_for_mode).sum();
        assert_eq!(total, 40); // 16 + 8 + 8 + 8 = 40
    }

    // ========================================================================
    // Shader Compilation Tests (using naga)
    // ========================================================================

    /// Helper to compile a WGSL shader and validate it using naga
    fn compile_and_validate_shader(mode: u8, format: u8) -> Result<(), String> {
        let shader_source = generate_shader(mode, format).map_err(|e| {
            format!(
                "Shader generation error for mode {} format {}: {}",
                mode, format, e
            )
        })?;

        // Parse the WGSL source
        let module = naga::front::wgsl::parse_str(&shader_source).map_err(|e| {
            format!(
                "WGSL parse error for mode {} format {}: {:?}",
                mode, format, e
            )
        })?;

        // Validate the module
        let mut validator = naga::valid::Validator::new(
            naga::valid::ValidationFlags::all(),
            naga::valid::Capabilities::all(),
        );

        validator.validate(&module).map_err(|e| {
            format!(
                "Validation error for mode {} format {}: {:?}",
                mode, format, e
            )
        })?;

        Ok(())
    }

    #[test]
    fn test_compile_all_40_shaders() {
        let mut errors = Vec::new();

        // Mode 0: All 16 vertex formats
        for format in 0u8..16 {
            if let Err(e) = compile_and_validate_shader(0, format) {
                errors.push(e);
            }
        }

        // Modes 1-3: Only formats with NORMAL flag
        for mode in 1u8..=3 {
            for format in valid_formats_for_mode(mode) {
                if let Err(e) = compile_and_validate_shader(mode, format) {
                    errors.push(e);
                }
            }
        }

        if !errors.is_empty() {
            panic!(
                "Shader compilation failed for {} shaders:\n{}",
                errors.len(),
                errors.join("\n")
            );
        }
    }

    #[test]
    fn test_compile_mode0_all_formats() {
        for format in 0u8..16 {
            compile_and_validate_shader(0, format).unwrap_or_else(|e| panic!("{}", e));
        }
    }

    #[test]
    fn test_compile_mode1_matcap() {
        for format in valid_formats_for_mode(1) {
            compile_and_validate_shader(1, format).unwrap_or_else(|e| panic!("{}", e));
        }
    }

    #[test]
    fn test_compile_mode2_pbr() {
        for format in valid_formats_for_mode(2) {
            compile_and_validate_shader(2, format).unwrap_or_else(|e| panic!("{}", e));
        }
    }

    #[test]
    fn test_compile_mode3_blinnphong() {
        for format in valid_formats_for_mode(3) {
            compile_and_validate_shader(3, format).unwrap_or_else(|e| panic!("{}", e));
        }
    }

    #[test]
    fn test_compile_skinned_variants() {
        // Test all skinned formats (formats 8-15)
        // Mode 0 supports all skinned formats
        for format in 8u8..16 {
            compile_and_validate_shader(0, format).unwrap_or_else(|e| panic!("{}", e));
        }

        // Modes 1-3 only support skinned formats with NORMAL (12-15)
        for mode in 1u8..=3 {
            for format in [12, 13, 14, 15] {
                compile_and_validate_shader(mode, format).unwrap_or_else(|e| panic!("{}", e));
            }
        }
    }

    #[test]
    fn test_shader_has_vertex_entry() {
        for mode in 0u8..=3 {
            for format in valid_formats_for_mode(mode) {
                let shader = generate_shader(mode, format).unwrap();
                assert!(
                    shader.contains("fn vs("),
                    "Mode {} format {} missing vertex entry point 'vs'",
                    mode,
                    format
                );
            }
        }
    }

    #[test]
    fn test_shader_has_fragment_entry() {
        for mode in 0u8..=3 {
            for format in valid_formats_for_mode(mode) {
                let shader = generate_shader(mode, format).unwrap();
                assert!(
                    shader.contains("fn fs("),
                    "Mode {} format {} missing fragment entry point 'fs'",
                    mode,
                    format
                );
            }
        }
    }

    #[test]
    fn test_no_unreplaced_placeholders() {
        for mode in 0u8..=3 {
            for format in valid_formats_for_mode(mode) {
                let shader = generate_shader(mode, format).unwrap();
                let placeholders = [
                    "//VIN_UV",
                    "//VIN_COLOR",
                    "//VIN_NORMAL",
                    "//VIN_SKINNED",
                    "//VOUT_UV",
                    "//VOUT_COLOR",
                    "//VOUT_WORLD_NORMAL",
                    "//VOUT_VIEW_NORMAL",
                    "//VOUT_CAMERA_POS",
                    "//VS_UV",
                    "//VS_COLOR",
                    "//VS_WORLD_NORMAL",
                    "//VS_VIEW_NORMAL",
                    "//VS_CAMERA_POS",
                    "//VS_SKINNED",
                    "//VS_POSITION",
                    "//FS_COLOR",
                    "//FS_UV",
                    "//FS_AMBIENT",
                    "//FS_NORMAL",
                    "//FS_MRE",
                    "//FS_MODE2_3_DIFFUSE_FACTOR",
                    "//FS_MODE2_3_SHININESS",
                    "//FS_MODE2_3_SPECULAR_COLOR",
                    "//FS_MODE2_3_TEXTURES",
                    "//FS_MODE2_3_ROUGHNESS",
                ];

                for placeholder in placeholders {
                    assert!(
                        !shader.contains(placeholder),
                        "Mode {} format {} has unreplaced placeholder '{}'",
                        mode,
                        format,
                        placeholder
                    );
                }
            }
        }
    }
}
