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

const VS_SKINNED: &str = r#"// GPU skinning (placeholder - will be implemented)
    // TODO: Apply bone transforms to position and normal"#;

// Fragment shader code (Mode 0)
const FS_COLOR: &str = "color *= in.color;";
const FS_UV: &str = "color *= textureSample(slot0, tex_sampler, in.uv).rgb;";
const FS_NORMAL: &str = "color *= sky_lambert(in.world_normal);";

// Fragment shader code (Modes 1-3 for MRE texture)
const FS_MRE: &str = r#"// Sample MRE texture if UV present
    #ifdef HAS_UV
    let mre_sample = textureSample(slot1, tex_sampler, in.uv);
    mre = vec3<f32>(mre_sample.r, mre_sample.g, mre_sample.b);
    #endif"#;

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
    shader = shader.replace("//VS_NORMAL", if has_normal { VS_NORMAL } else { "" });
    shader = shader.replace("//VS_SKINNED", if has_skinned { VS_SKINNED } else { "" });

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
            // Mode 2 (PBR) and Mode 3 (Hybrid)
            shader = shader.replace("//FS_COLOR", if has_color { FS_COLOR } else { "" });
            shader = shader.replace("//FS_UV", if has_uv { FS_UV } else { "" });

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
}
