// ! Shader generation system for Emberware Z
//!
//! Generates WGSL shaders from templates by replacing placeholders based on:
//! - Render mode (0-3): Unlit, Matcap, PBR, Hybrid
//! - Vertex format flags (UV, COLOR, NORMAL, SKINNED)
//!
//! Total shader count: 40
//! - Mode 0: 16 shaders (all vertex formats)
//! - Modes 1-3: 8 shaders each (only formats with NORMAL flag)

use crate::graphics::{FORMAT_COLOR, FORMAT_NORMAL, FORMAT_SKINNED, FORMAT_UV};
use std::fmt;

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
// Shader Templates (embedded at compile time)
// ============================================================================

// Shared utilities
const COMMON: &str = include_str!("../shaders/common.wgsl");
const BLINNPHONG_COMMON: &str = include_str!("../shaders/blinnphong_common.wgsl");

// Mode-specific templates (modes 0-1 only; modes 2-3 generated from blinnphong_common)
const TEMPLATE_MODE0: &str = include_str!("../shaders/mode0_unlit.wgsl");
const TEMPLATE_MODE1: &str = include_str!("../shaders/mode1_matcap.wgsl");

// ============================================================================
// Placeholder Snippets
// ============================================================================

// Vertex input struct fields
const VIN_UV: &str = "@location(1) uv: vec2<f32>,";
const VIN_COLOR: &str = "@location(2) color: vec3<f32>,";
const VIN_NORMAL: &str = "@location(3) normal: vec3<f32>,";
const VIN_SKINNED: &str =
    "@location(4) bone_indices: vec4<u32>,\n    @location(5) bone_weights: vec4<f32>,";

// Vertex output struct fields
const VOUT_WORLD_NORMAL: &str = "@location(1) world_normal: vec3<f32>,";
const VOUT_VIEW_NORMAL: &str = "@location(2) view_normal: vec3<f32>,";
const VOUT_CAMERA_POS: &str = "@location(4) @interpolate(flat) camera_position: vec3<f32>,";
const VOUT_UV: &str = "@location(10) uv: vec2<f32>,";
const VOUT_COLOR: &str = "@location(11) color: vec3<f32>,";

// Vertex shader code
const VS_UV: &str = "out.uv = in.uv;";
const VS_COLOR: &str = "out.color = in.color;";
const VS_WORLD_NORMAL: &str =
    "let world_normal_raw = (model_matrix * vec4<f32>(in.normal, 0.0)).xyz;\n    out.world_normal = normalize(world_normal_raw);";
const VS_VIEW_NORMAL: &str =
    "let view_normal = (view_matrix * vec4<f32>(out.world_normal, 0.0)).xyz;\n    out.view_normal = normalize(view_normal);";
const VS_CAMERA_POS: &str = "out.camera_position = extract_camera_position(view_matrix);";

// Legacy constant (for backward compatibility if needed)
const VS_NORMAL: &str = r#"let world_normal_raw = (model_matrix * vec4<f32>(in.normal, 0.0)).xyz;
    out.world_normal = normalize(world_normal_raw);
    let view_normal = (view_matrix * vec4<f32>(world_normal_raw, 0.0)).xyz;
    out.view_normal = normalize(view_normal);"#;

const VS_SKINNED: &str = r#"// GPU skinning: compute skinned position and normal
    var skinned_pos = vec3<f32>(0.0, 0.0, 0.0);
    var skinned_normal = vec3<f32>(0.0, 0.0, 0.0);

    for (var i = 0u; i < 4u; i++) {
        let bone_idx = in.bone_indices[i];
        let weight = in.bone_weights[i];

        if (weight > 0.0 && bone_idx < 256u) {
            let bone_matrix = bones[bone_idx];
            skinned_pos += (bone_matrix * vec4<f32>(in.position, 1.0)).xyz * weight;
            //VS_SKINNED_NORMAL
        }
    }

    let final_position = skinned_pos;
    //VS_SKINNED_FINAL_NORMAL"#;

const VS_SKINNED_NORMAL: &str =
    "skinned_normal += (bone_matrix * vec4<f32>(in.normal, 0.0)).xyz * weight;";
const VS_SKINNED_FINAL_NORMAL: &str = "let final_normal = normalize(skinned_normal);";

const VS_POSITION_SKINNED: &str = "let world_pos = vec4<f32>(final_position, 1.0);";
const VS_POSITION_UNSKINNED: &str = "let world_pos = vec4<f32>(in.position, 1.0);";

// Fragment shader code (Mode 0 and Mode 1 - use "color" variable)
const FS_COLOR: &str = "color *= in.color;";
const FS_UV: &str = "let tex_sample = textureSample(slot0, tex_sampler, in.uv); color *= tex_sample.rgb; color *= tex_sample.a;";
const FS_AMBIENT: &str = "let ambient = color * sample_sky(in.world_normal, sky);";
const FS_NORMAL: &str = "color = ambient + lambert_diffuse(in.world_normal, sky.sun_direction, color, sky.sun_color);";

// Fragment shader code (Modes 2-3 - use "albedo" variable)
const FS_ALBEDO_COLOR: &str = "albedo *= in.color;";
const FS_ALBEDO_UV: &str = "let albedo_sample = textureSample(slot0, tex_sampler, in.uv); albedo *= albedo_sample.rgb; albedo *= albedo_sample.a;";

// Fragment shader code (Mode 3 - Blinn-Phong texture sampling)
const FS_MODE3_SLOT1: &str = "let slot1_sample = textureSample(slot1, tex_sampler, in.uv);\n    value0 = slot1_sample.r;\n    value1 = slot1_sample.g;\n    emissive = slot1_sample.b;";

// ============================================================================
// Shader Generation
// ============================================================================

/// Generate a shader from a template for a specific vertex format
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

    // Get the appropriate template (modes 0-1 only; modes 2-3 use BLINNPHONG_COMMON)
    let template = match mode {
        0 => TEMPLATE_MODE0,
        1 => TEMPLATE_MODE1,
        _ => "", // Modes 2-3 use BLINNPHONG_COMMON, not a template
    };

    // Check if this format is valid for the mode
    // Modes 1-3 require normals
    let has_normal = format & FORMAT_NORMAL != 0;
    if mode > 0 && !has_normal {
        return Err(ShaderGenError::MissingNormalFlag { mode, format });
    }

    // Extract format flags
    let has_uv = format & FORMAT_UV != 0;
    let has_color = format & FORMAT_COLOR != 0;
    let has_skinned = format & FORMAT_SKINNED != 0;

    // Build shader by combining common code + mode-specific template
    let mut shader = String::new();

    // Always include common bindings and utilities first
    shader.push_str(COMMON);
    shader.push('\n');

    // Include Blinn-Phong (modes 2-3) or mode-specific template (modes 0-1)
    if mode >= 2 {
        shader.push_str(BLINNPHONG_COMMON);
    } else {
        shader.push_str(template);
    }
    shader.push('\n');

    // Replace vertex input placeholders
    shader = shader.replace("//VIN_UV", if has_uv { VIN_UV } else { "" });
    shader = shader.replace("//VIN_COLOR", if has_color { VIN_COLOR } else { "" });
    shader = shader.replace("//VIN_NORMAL", if has_normal { VIN_NORMAL } else { "" });
    shader = shader.replace("//VIN_SKINNED", if has_skinned { VIN_SKINNED } else { "" });

    // Replace vertex output placeholders
    shader = shader.replace("//VOUT_UV", if has_uv { VOUT_UV } else { "" });
    shader = shader.replace("//VOUT_COLOR", if has_color { VOUT_COLOR } else { "" });
    shader = shader.replace("//VOUT_WORLD_NORMAL", if has_normal { VOUT_WORLD_NORMAL } else { "" });
    shader = shader.replace("//VOUT_VIEW_NORMAL", if has_normal { VOUT_VIEW_NORMAL } else { "" });
    shader = shader.replace("//VOUT_CAMERA_POS", if mode >= 2 { VOUT_CAMERA_POS } else { "" });

    // Replace vertex shader code placeholders
    shader = shader.replace("//VS_UV", if has_uv { VS_UV } else { "" });
    shader = shader.replace("//VS_COLOR", if has_color { VS_COLOR } else { "" });

    // Normal handling depends on skinning
    if has_normal && !has_skinned {
        shader = shader.replace("//VS_WORLD_NORMAL", VS_WORLD_NORMAL);
        shader = shader.replace("//VS_VIEW_NORMAL", VS_VIEW_NORMAL);
    } else if has_normal && has_skinned {
        // Skinned normals: use final_normal from skinning code
        let skinned_world_normal =
            "out.world_normal = normalize(final_normal);";
        let skinned_view_normal =
            "let view_normal = (view_matrix * vec4<f32>(final_normal, 0.0)).xyz;\n    out.view_normal = normalize(view_normal);";
        shader = shader.replace("//VS_WORLD_NORMAL", skinned_world_normal);
        shader = shader.replace("//VS_VIEW_NORMAL", skinned_view_normal);
    } else {
        shader = shader.replace("//VS_WORLD_NORMAL", "");
        shader = shader.replace("//VS_VIEW_NORMAL", "");
    }

    // Camera position extraction (modes 2-3 only)
    if mode >= 2 {
        shader = shader.replace("//VS_CAMERA_POS", VS_CAMERA_POS);
    } else {
        shader = shader.replace("//VS_CAMERA_POS", "");
    }

    // Handle skinning with nested replacements
    if has_skinned {
        let mut skinned_code = VS_SKINNED.to_string();
        skinned_code = skinned_code.replace(
            "//VS_SKINNED_NORMAL",
            if has_normal { VS_SKINNED_NORMAL } else { "" },
        );
        skinned_code = skinned_code.replace(
            "//VS_SKINNED_FINAL_NORMAL",
            if has_normal {
                VS_SKINNED_FINAL_NORMAL
            } else {
                ""
            },
        );
        shader = shader.replace("//VS_SKINNED", &skinned_code);
        shader = shader.replace("//VS_POSITION", VS_POSITION_SKINNED);
    } else {
        shader = shader.replace("//VS_SKINNED", "");
        shader = shader.replace("//VS_POSITION", VS_POSITION_UNSKINNED);
    }

    // Replace fragment shader placeholders (mode-specific)
    match mode {
        0 => {
            // Mode 0 (Unlit)
            shader = shader.replace("//FS_COLOR", if has_color { FS_COLOR } else { "" });
            shader = shader.replace("//FS_UV", if has_uv { FS_UV } else { "" });
            shader = shader.replace("//FS_AMBIENT", if has_normal { FS_AMBIENT } else { "" });
            shader = shader.replace("//FS_NORMAL", if has_normal { FS_NORMAL } else { "" });
        }
        1 => {
            // Mode 1 (Matcap)
            shader = shader.replace("//FS_COLOR", if has_color { FS_COLOR } else { "" });
            shader = shader.replace("//FS_UV", if has_uv { FS_UV } else { "" });
            shader = shader.replace("//FS_AMBIENT", "");  // Matcap doesn't use ambient
        }
        2 => {
            // Mode 2: Metallic-Roughness Blinn-Phong
            shader = shader.replace("//FS_COLOR", if has_color { FS_ALBEDO_COLOR } else { "" });
            shader = shader.replace("//FS_UV", if has_uv { FS_ALBEDO_UV } else { "" });
            shader = shader.replace("//FS_AMBIENT", "");  // Modes 2-3 handle ambient internally

            // Shininess: Compute from roughness (power curve)
            let mode2_shininess = "let shininess = pow(256.0, 1.0 - value1);";
            shader = shader.replace("//FS_MODE2_3_SHININESS", mode2_shininess);

            // Specular color: Derive from metallic (F0=0.04 dielectric)
            let mode2_specular = "let specular_color = mix(vec3<f32>(0.04), albedo, value0);";
            shader = shader.replace("//FS_MODE2_3_SPECULAR_COLOR", mode2_specular);

            // Texture overrides
            if has_uv {
                shader = shader.replace("//FS_MODE2_3_TEXTURES", "let mre_sample = textureSample(slot1, tex_sampler, in.uv);\n    value0 = mre_sample.r;\n    value1 = mre_sample.g;\n    emissive = mre_sample.b;");
            } else {
                shader = shader.replace("//FS_MODE2_3_TEXTURES", "");
            }
        }
        3 => {
            // Mode 3: Specular Blinn-Phong
            shader = shader.replace("//FS_COLOR", if has_color { FS_ALBEDO_COLOR } else { "" });
            shader = shader.replace("//FS_UV", if has_uv { FS_ALBEDO_UV } else { "" });
            shader = shader.replace("//FS_AMBIENT", "");  // Modes 2-3 handle ambient internally

            // Shininess: Direct linear mapping (0-1 → 1-256)
            let mode3_shininess = "let shininess = mix(1.0, 256.0, value1);";
            shader = shader.replace("//FS_MODE2_3_SHININESS", mode3_shininess);

            // Specular color: From texture (if UV) or uniform with intensity modulation
            let mode3_specular = if has_uv {
                "var specular_color = textureSample(slot2, tex_sampler, in.uv).rgb;\n    specular_color = specular_color * value0;"
            } else {
                "var specular_color = unpack_rgb8(shading.matcap_blend_modes);\n    specular_color = specular_color * value0;"
            };
            shader = shader.replace("//FS_MODE2_3_SPECULAR_COLOR", mode3_specular);

            // Texture overrides for slot 1 (Specular intensity-Shininess-Emissive)
            if has_uv {
                let mode3_textures = "let slot1_sample = textureSample(slot1, tex_sampler, in.uv);\n    value0 = slot1_sample.r;\n    value1 = slot1_sample.g;\n    emissive = slot1_sample.b;";
                shader = shader.replace("//FS_MODE2_3_TEXTURES", mode3_textures);
            } else {
                shader = shader.replace("//FS_MODE2_3_TEXTURES", "");
            }
        }
        _ => {}
    }

    Ok(shader)
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
        0 => "Unlit",
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
                    "//FS_MODE2_3_SHININESS",
                    "//FS_MODE2_3_SPECULAR_COLOR",
                    "//FS_MODE2_3_TEXTURES",
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
