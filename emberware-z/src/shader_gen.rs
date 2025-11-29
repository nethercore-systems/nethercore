// ! Shader generation system for Emberware Z
//!
//! Generates WGSL shaders from templates by replacing placeholders based on:
//! - Render mode (0-3): Unlit, Matcap, PBR, Hybrid
//! - Vertex format flags (UV, COLOR, NORMAL, SKINNED)
//!
//! Total shader count: 40
//! - Mode 0: 16 shaders (all vertex formats)
//! - Modes 1-3: 8 shaders each (only formats with NORMAL flag)

use crate::graphics::{FORMAT_UV, FORMAT_COLOR, FORMAT_NORMAL, FORMAT_SKINNED};

// ============================================================================
// Shader Templates (embedded at compile time)
// ============================================================================

const TEMPLATE_MODE0: &str = include_str!("../shaders/mode0_unlit.wgsl");
const TEMPLATE_MODE1: &str = include_str!("../shaders/mode1_matcap.wgsl");
const TEMPLATE_MODE2: &str = include_str!("../shaders/mode2_pbr.wgsl");
const TEMPLATE_MODE3: &str = include_str!("../shaders/mode3_hybrid.wgsl");

// ============================================================================
// Placeholder Snippets
// ============================================================================

// Vertex input struct fields
const VIN_UV: &str = "@location(1) uv: vec2<f32>,";
const VIN_COLOR: &str = "@location(2) color: vec3<f32>,";
const VIN_NORMAL: &str = "@location(3) normal: vec3<f32>,";
const VIN_SKINNED: &str = "@location(4) bone_indices: vec4<u32>,\n    @location(5) bone_weights: vec4<f32>,";

// Vertex output struct fields
const VOUT_UV: &str = "@location(10) uv: vec2<f32>,";
const VOUT_COLOR: &str = "@location(11) color: vec3<f32>,";
const VOUT_NORMAL: &str = "@location(12) world_normal: vec3<f32>,\n    @location(13) view_normal: vec3<f32>,";

// Vertex shader code
const VS_UV: &str = "out.uv = in.uv;";
const VS_COLOR: &str = "out.color = in.color;";
const VS_NORMAL: &str = r#"out.world_normal = normalize(in.normal);
    let view_normal = (view_matrix * vec4<f32>(in.normal, 0.0)).xyz;
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

const VS_SKINNED_NORMAL: &str = "skinned_normal += (bone_matrix * vec4<f32>(in.normal, 0.0)).xyz * weight;";
const VS_SKINNED_FINAL_NORMAL: &str = "let final_normal = normalize(skinned_normal);";

const VS_POSITION_SKINNED: &str = "let world_pos = vec4<f32>(final_position, 1.0);";
const VS_POSITION_UNSKINNED: &str = "let world_pos = vec4<f32>(in.position, 1.0);";

// Fragment shader code (Mode 0 and Mode 1 - use "color" variable)
const FS_COLOR: &str = "color *= in.color;";
const FS_UV: &str = "color *= textureSample(slot0, tex_sampler, in.uv).rgb;";
const FS_NORMAL: &str = "color *= sky_lambert(in.world_normal);";

// Fragment shader code (Modes 2-3 - use "albedo" variable)
const FS_ALBEDO_COLOR: &str = "albedo *= in.color;";
const FS_ALBEDO_UV: &str = "albedo *= textureSample(slot0, tex_sampler, in.uv).rgb;";

// ============================================================================
// Shader Generation
// ============================================================================

/// Generate a shader from a template for a specific vertex format
pub fn generate_shader(mode: u8, format: u8) -> String {
    // Get the appropriate template
    let template = match mode {
        0 => TEMPLATE_MODE0,
        1 => TEMPLATE_MODE1,
        2 => TEMPLATE_MODE2,
        3 => TEMPLATE_MODE3,
        _ => panic!("Invalid render mode: {}", mode),
    };

    // Check if this format is valid for the mode
    // Modes 1-3 require normals
    let has_normal = format & FORMAT_NORMAL != 0;
    if mode > 0 && !has_normal {
        panic!("Render mode {} requires NORMAL flag, but format {} doesn't have it", mode, format);
    }

    // Extract format flags
    let has_uv = format & FORMAT_UV != 0;
    let has_color = format & FORMAT_COLOR != 0;
    let has_skinned = format & FORMAT_SKINNED != 0;

    // Start with template
    let mut shader = template.to_string();

    // Replace vertex input placeholders
    shader = shader.replace("//VIN_UV", if has_uv { VIN_UV } else { "" });
    shader = shader.replace("//VIN_COLOR", if has_color { VIN_COLOR } else { "" });
    shader = shader.replace("//VIN_NORMAL", if has_normal { VIN_NORMAL } else { "" });
    shader = shader.replace("//VIN_SKINNED", if has_skinned { VIN_SKINNED } else { "" });

    // Replace vertex output placeholders
    shader = shader.replace("//VOUT_UV", if has_uv { VOUT_UV } else { "" });
    shader = shader.replace("//VOUT_COLOR", if has_color { VOUT_COLOR } else { "" });
    shader = shader.replace("//VOUT_NORMAL", if has_normal { VOUT_NORMAL } else { "" });

    // Replace vertex shader code placeholders
    shader = shader.replace("//VS_UV", if has_uv { VS_UV } else { "" });
    shader = shader.replace("//VS_COLOR", if has_color { VS_COLOR } else { "" });

    // Normal handling depends on skinning
    if has_normal && !has_skinned {
        shader = shader.replace("//VS_NORMAL", VS_NORMAL);
    } else if has_normal && has_skinned {
        // Skinned normals are handled differently
        let skinned_normal = r#"out.world_normal = normalize(final_normal);
    let view_normal = (view_matrix * vec4<f32>(final_normal, 0.0)).xyz;
    out.view_normal = normalize(view_normal);"#;
        shader = shader.replace("//VS_NORMAL", skinned_normal);
    } else {
        shader = shader.replace("//VS_NORMAL", "");
    }

    // Handle skinning with nested replacements
    if has_skinned {
        let mut skinned_code = VS_SKINNED.to_string();
        skinned_code = skinned_code.replace("//VS_SKINNED_NORMAL", if has_normal { VS_SKINNED_NORMAL } else { "" });
        skinned_code = skinned_code.replace("//VS_SKINNED_FINAL_NORMAL", if has_normal { VS_SKINNED_FINAL_NORMAL } else { "" });
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
            shader = shader.replace("//FS_NORMAL", if has_normal { FS_NORMAL } else { "" });
        }
        1 => {
            // Mode 1 (Matcap)
            shader = shader.replace("//FS_COLOR", if has_color { FS_COLOR } else { "" });
            shader = shader.replace("//FS_UV", if has_uv { FS_UV } else { "" });
        }
        2 | 3 => {
            // Mode 2 (PBR) and Mode 3 (Hybrid) - use "albedo" variable
            shader = shader.replace("//FS_COLOR", if has_color { FS_ALBEDO_COLOR } else { "" });
            shader = shader.replace("//FS_UV", if has_uv { FS_ALBEDO_UV } else { "" });

            // MRE texture sampling (conditionally based on UV)
            if has_uv {
                shader = shader.replace("//FS_MRE", "let mre_sample = textureSample(slot1, tex_sampler, in.uv);\n    mre = vec3<f32>(mre_sample.r, mre_sample.g, mre_sample.b);");
            } else {
                shader = shader.replace("//FS_MRE", "");
            }
        }
        _ => {}
    }

    shader
}

/// Get the template for a given render mode (for debugging/inspection)
pub fn get_template(mode: u8) -> &'static str {
    match mode {
        0 => TEMPLATE_MODE0,
        1 => TEMPLATE_MODE1,
        2 => TEMPLATE_MODE2,
        3 => TEMPLATE_MODE3,
        _ => panic!("Invalid render mode: {}", mode),
    }
}

/// Get human-readable name for a render mode
pub fn mode_name(mode: u8) -> &'static str {
    match mode {
        0 => "Unlit",
        1 => "Matcap",
        2 => "PBR",
        3 => "Hybrid",
        _ => "Unknown",
    }
}

/// Get the number of shader permutations for a render mode
pub fn shader_count_for_mode(mode: u8) -> usize {
    match mode {
        0 => 16,  // All vertex formats
        1..=3 => 8,   // Only formats with NORMAL
        _ => 0,
    }
}

/// Get all valid vertex formats for a render mode
pub fn valid_formats_for_mode(mode: u8) -> Vec<u8> {
    match mode {
        0 => (0..16).collect(),  // All formats
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
            let shader = generate_shader(0, format);
            assert!(!shader.is_empty());
            assert!(shader.contains("@vertex"));
            assert!(shader.contains("@fragment"));
        }
    }

    #[test]
    fn test_shader_generation_mode1() {
        // Mode 1 should only support formats with NORMAL
        for format in valid_formats_for_mode(1) {
            let shader = generate_shader(1, format);
            assert!(!shader.is_empty());
            assert!(shader.contains("matcap"));
        }
    }

    #[test]
    #[should_panic]
    fn test_mode1_without_normals_panics() {
        // Mode 1 without normals should panic
        generate_shader(1, 0);  // Format 0 has no NORMAL flag
    }

    #[test]
    fn test_placeholder_replacement() {
        // Test that placeholders are replaced correctly
        let shader = generate_shader(0, FORMAT_UV | FORMAT_COLOR);
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
        let total: usize = (0..4).map(|m| shader_count_for_mode(m)).sum();
        assert_eq!(total, 40);  // 16 + 8 + 8 + 8 = 40
    }

    // ========================================================================
    // Shader Compilation Tests (using naga)
    // ========================================================================

    /// Helper to compile a WGSL shader and validate it using naga
    fn compile_and_validate_shader(mode: u8, format: u8) -> Result<(), String> {
        let shader_source = generate_shader(mode, format);

        // Parse the WGSL source
        let module = naga::front::wgsl::parse_str(&shader_source)
            .map_err(|e| format!("WGSL parse error for mode {} format {}: {:?}", mode, format, e))?;

        // Validate the module
        let mut validator = naga::valid::Validator::new(
            naga::valid::ValidationFlags::all(),
            naga::valid::Capabilities::all(),
        );

        validator.validate(&module)
            .map_err(|e| format!("Validation error for mode {} format {}: {:?}", mode, format, e))?;

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
            compile_and_validate_shader(0, format)
                .unwrap_or_else(|e| panic!("{}", e));
        }
    }

    #[test]
    fn test_compile_mode1_matcap() {
        for format in valid_formats_for_mode(1) {
            compile_and_validate_shader(1, format)
                .unwrap_or_else(|e| panic!("{}", e));
        }
    }

    #[test]
    fn test_compile_mode2_pbr() {
        for format in valid_formats_for_mode(2) {
            compile_and_validate_shader(2, format)
                .unwrap_or_else(|e| panic!("{}", e));
        }
    }

    #[test]
    fn test_compile_mode3_hybrid() {
        for format in valid_formats_for_mode(3) {
            compile_and_validate_shader(3, format)
                .unwrap_or_else(|e| panic!("{}", e));
        }
    }

    #[test]
    fn test_compile_skinned_variants() {
        // Test all skinned formats (formats 8-15)
        // Mode 0 supports all skinned formats
        for format in 8u8..16 {
            compile_and_validate_shader(0, format)
                .unwrap_or_else(|e| panic!("{}", e));
        }

        // Modes 1-3 only support skinned formats with NORMAL (12-15)
        for mode in 1u8..=3 {
            for format in [12, 13, 14, 15] {
                compile_and_validate_shader(mode, format)
                    .unwrap_or_else(|e| panic!("{}", e));
            }
        }
    }

    #[test]
    fn test_shader_has_vertex_entry() {
        for mode in 0u8..=3 {
            for format in valid_formats_for_mode(mode) {
                let shader = generate_shader(mode, format);
                assert!(
                    shader.contains("fn vs("),
                    "Mode {} format {} missing vertex entry point 'vs'",
                    mode, format
                );
            }
        }
    }

    #[test]
    fn test_shader_has_fragment_entry() {
        for mode in 0u8..=3 {
            for format in valid_formats_for_mode(mode) {
                let shader = generate_shader(mode, format);
                assert!(
                    shader.contains("fn fs("),
                    "Mode {} format {} missing fragment entry point 'fs'",
                    mode, format
                );
            }
        }
    }

    #[test]
    fn test_no_unreplaced_placeholders() {
        for mode in 0u8..=3 {
            for format in valid_formats_for_mode(mode) {
                let shader = generate_shader(mode, format);
                let placeholders = [
                    "//VIN_UV", "//VIN_COLOR", "//VIN_NORMAL", "//VIN_SKINNED",
                    "//VOUT_UV", "//VOUT_COLOR", "//VOUT_NORMAL",
                    "//VS_UV", "//VS_COLOR", "//VS_NORMAL", "//VS_SKINNED",
                    "//VS_POSITION",
                    "//FS_COLOR", "//FS_UV", "//FS_NORMAL", "//FS_MRE",
                ];

                for placeholder in placeholders {
                    assert!(
                        !shader.contains(placeholder),
                        "Mode {} format {} has unreplaced placeholder '{}'",
                        mode, format, placeholder
                    );
                }
            }
        }
    }
}
