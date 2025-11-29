use std::fs;
use std::io;

use orb_common::flags;

pub fn main() -> Result<(), io::Error> {
    println!("Running shader_gen...");
    let template = fs::read_to_string("./orb_tools/src/shader_gen/template.wgsl")?;

    for index in 0..=flags::ALL {
        let shader = generate_shader(&template, index);

        let filename = format!("./orb_render/shaders/3d_{index}.wgsl");
        fs::write(&filename, shader)?;
        println!("Generated {filename}.");
    }

    println!("Shader generation done.");
    Ok(())
}

/// Generate a shader from the template for a given flag combination.
///
/// The `flags` parameter is a bitmask combining:
/// - `flags::VERTEX_COLOR` (1) - Include vertex color input/output
/// - `flags::UV_TEXTURE` (2) - Include UV coordinate input/output
/// - `flags::CUBEMAP` (4) - Include cubemap reflection sampling
/// - `flags::MATCAP` (8) - Include matcap material sampling
pub fn generate_shader(template: &str, flags: u8) -> String {
    let mut shader = template.to_string();

    let has_color = flags & flags::VERTEX_COLOR != 0;
    let has_uv = flags & flags::UV_TEXTURE != 0;
    let has_cubemap = flags & flags::CUBEMAP != 0;
    let has_matcap = flags & flags::MATCAP != 0;
    let has_normal = has_cubemap || has_matcap;

    // Vertex Input
    shader = shader.replace("//VIN_COLOR", if has_color { VIN_COLOR } else { "" });
    shader = shader.replace("//VIN_UV", if has_uv { VIN_UV } else { "" });
    shader = shader.replace("//VIN_NORMAL", if has_normal { VIN_NORMAL } else { "" });

    // Vertex Output
    shader = shader.replace("//VOUT_COLOR", if has_color { VOUT_COLOR } else { "" });
    shader = shader.replace("//VOUT_UV", if has_uv { VOUT_UV } else { "" });
    shader = shader.replace(
        "//VOUT_WORLD_NORMAL",
        if has_normal { VOUT_WORLD_NORMAL } else { "" },
    );
    shader = shader.replace("//VOUT_MATCAP", if has_matcap { VOUT_MATCAP } else { "" });

    // Vertex Shader
    shader = shader.replace("//VS_COLOR", if has_color { VS_COLOR } else { "" });
    shader = shader.replace("//VS_UV", if has_uv { VS_UV } else { "" });
    shader = shader.replace(
        "//VS_WORLD_NORMAL",
        if has_normal { VS_WORLD_NORMAL } else { "" },
    );
    shader = shader.replace("//VS_MATCAP", if has_matcap { VS_MATCAP } else { "" });

    // Fragment Shader
    shader = shader.replace("//FS_COLOR", if has_color { FS_COLOR } else { "" });
    shader = shader.replace("//FS_UV", if has_uv { FS_UV } else { "" });
    shader = shader.replace("//FS_CUBEMAP", if has_cubemap { FS_CUBEMAP } else { "" });
    shader = shader.replace("//FS_MATCAP", if has_matcap { FS_MATCAP } else { "" });

    shader
}

// Vertex Inputs
const VIN_COLOR: &str = "@location(1) color: vec3<f32>,";
const VIN_UV: &str = "@location(2) uv: vec2<f32>,";
const VIN_NORMAL: &str = "@location(3) normals: vec3<f32>,";

// Vertex Outputs
const VOUT_COLOR: &str = "@location(0) color: vec3<f32>,";
const VOUT_UV: &str = "@location(1) uv: vec2<f32>,";
const VOUT_WORLD_NORMAL: &str = "@location(2) world_normal: vec3<f32>,";
const VOUT_MATCAP: &str = "@location(3) view_pos: vec3<f32>,
    @location(4) view_normal: vec3<f32>,";

// Vertex Shader
const VS_COLOR: &str = "out.color = model.color;";
const VS_UV: &str = "out.uv = model.uv;";
const VS_WORLD_NORMAL: &str =
    "out.world_normal = normalize((model_matrix * vec4<f32>(model.normals, 0.0)).xyz);";
const VS_MATCAP: &str = "out.view_pos = view_position.xyz;
    out.view_normal = normalize((view_matrix * model_matrix * vec4<f32>(model.normals, 0.0)).xyz);";

// Fragment Shader
const FS_COLOR: &str = "out_color = blend_colors(out_color, in.color.rgb, 0);";
const FS_UV: &str =
    "out_color = blend_colors(out_color, textureSample(texture, texture_2d_sampler, in.uv).rgb, 1);";
const FS_CUBEMAP: &str = "out_color = blend_colors(out_color, textureSample(cubemap1, texture_3d_sampler, in.world_normal).rgb, 2);
    out_color = blend_colors(out_color, textureSample(cubemap2, texture_3d_sampler, in.world_normal).rgb, 3);";
const FS_MATCAP: &str = "let matcap_uv = matcap_uv(in.view_pos, in.view_normal);
    out_color = blend_colors(out_color, textureSample(matcap1, texture_3d_sampler, matcap_uv).rgb, 4);
    out_color = blend_colors(out_color, textureSample(matcap2, texture_3d_sampler, matcap_uv).rgb, 5);
    out_color = blend_colors(out_color, textureSample(matcap3, texture_3d_sampler, matcap_uv).rgb, 6);
    out_color = blend_colors(out_color, textureSample(matcap4, texture_3d_sampler, matcap_uv).rgb, 7);";

#[cfg(test)]
mod tests {
    use super::*;

    // Minimal template with all placeholder markers for testing
    const TEST_TEMPLATE: &str = r#"struct VertexIn {
    @location(0) position: vec3<f32>,
    //VIN_COLOR
    //VIN_UV
    //VIN_NORMAL
};

struct VertexOut {
    @builtin(position) clip_position: vec4<f32>,
    //VOUT_COLOR
    //VOUT_UV
    //VOUT_WORLD_NORMAL
    //VOUT_MATCAP
};

@vertex
fn vs(model: VertexIn) -> VertexOut {
    var out: VertexOut;
    //VS_COLOR
    //VS_UV
    //VS_WORLD_NORMAL
    //VS_MATCAP
    return out;
}

@fragment
fn fs(in: VertexOut) -> @location(0) vec4<f32> {
    var out_color = vec3<f32>(1.0);
    //FS_COLOR
    //FS_UV
    //FS_CUBEMAP
    //FS_MATCAP
    return vec4<f32>(out_color, 1.0);
}"#;

    // === Flag combination tests ===

    #[test]
    fn test_flags_no_features() {
        let shader = generate_shader(TEST_TEMPLATE, 0);

        // Should not contain any feature-specific code
        assert!(!shader.contains("@location(1) color"));
        assert!(!shader.contains("@location(2) uv"));
        assert!(!shader.contains("@location(3) normals"));
        assert!(!shader.contains("out.color"));
        assert!(!shader.contains("out.uv"));
        assert!(!shader.contains("blend_colors"));
    }

    #[test]
    fn test_flags_vertex_color_only() {
        let shader = generate_shader(TEST_TEMPLATE, flags::VERTEX_COLOR);

        // Should have color input/output
        assert!(shader.contains("@location(1) color: vec3<f32>,"));
        assert!(shader.contains("out.color = model.color;"));
        assert!(shader.contains("blend_colors(out_color, in.color.rgb, 0)"));

        // Should NOT have UV, normal, cubemap, matcap
        assert!(!shader.contains("@location(2) uv"));
        assert!(!shader.contains("@location(3) normals"));
        assert!(!shader.contains("world_normal"));
        assert!(!shader.contains("cubemap"));
        assert!(!shader.contains("matcap"));
    }

    #[test]
    fn test_flags_uv_texture_only() {
        let shader = generate_shader(TEST_TEMPLATE, flags::UV_TEXTURE);

        // Should have UV input/output
        assert!(shader.contains("@location(2) uv: vec2<f32>,"));
        assert!(shader.contains("out.uv = model.uv;"));
        assert!(shader.contains("textureSample(texture, texture_2d_sampler, in.uv)"));

        // Should NOT have color, normal, cubemap, matcap
        assert!(!shader.contains("@location(1) color"));
        assert!(!shader.contains("@location(3) normals"));
        assert!(!shader.contains("world_normal"));
        assert!(!shader.contains("cubemap"));
        assert!(!shader.contains("matcap_uv"));
    }

    #[test]
    fn test_flags_cubemap_includes_normal() {
        let shader = generate_shader(TEST_TEMPLATE, flags::CUBEMAP);

        // Cubemap requires normals
        assert!(shader.contains("@location(3) normals: vec3<f32>,"));
        assert!(shader.contains("world_normal"));
        assert!(shader.contains("cubemap1"));
        assert!(shader.contains("cubemap2"));

        // Should NOT have matcap-specific code
        assert!(!shader.contains("view_pos"));
        assert!(!shader.contains("view_normal"));
        assert!(!shader.contains("matcap_uv"));
    }

    #[test]
    fn test_flags_matcap_includes_normal_and_view() {
        let shader = generate_shader(TEST_TEMPLATE, flags::MATCAP);

        // Matcap requires normals AND view position/normal
        assert!(shader.contains("@location(3) normals: vec3<f32>,"));
        assert!(shader.contains("world_normal"));
        assert!(shader.contains("view_pos"));
        assert!(shader.contains("view_normal"));
        assert!(shader.contains("matcap_uv"));
        assert!(shader.contains("matcap1"));
        assert!(shader.contains("matcap4"));
    }

    #[test]
    fn test_flags_all_features() {
        let shader = generate_shader(TEST_TEMPLATE, flags::ALL);

        // All features should be present
        assert!(shader.contains("@location(1) color: vec3<f32>,"));
        assert!(shader.contains("@location(2) uv: vec2<f32>,"));
        assert!(shader.contains("@location(3) normals: vec3<f32>,"));
        assert!(shader.contains("out.color = model.color;"));
        assert!(shader.contains("out.uv = model.uv;"));
        assert!(shader.contains("world_normal"));
        assert!(shader.contains("view_pos"));
        assert!(shader.contains("cubemap1"));
        assert!(shader.contains("matcap1"));
    }

    #[test]
    fn test_flags_color_and_uv() {
        let shader = generate_shader(TEST_TEMPLATE, flags::VERTEX_COLOR | flags::UV_TEXTURE);

        // Both color and UV
        assert!(shader.contains("@location(1) color: vec3<f32>,"));
        assert!(shader.contains("@location(2) uv: vec2<f32>,"));
        assert!(shader.contains("out.color"));
        assert!(shader.contains("out.uv"));

        // No normal-requiring features
        assert!(!shader.contains("@location(3) normals"));
        assert!(!shader.contains("cubemap"));
        assert!(!shader.contains("matcap_uv"));
    }

    #[test]
    fn test_flags_cubemap_and_matcap() {
        let shader = generate_shader(TEST_TEMPLATE, flags::CUBEMAP | flags::MATCAP);

        // Both cubemap and matcap use normals
        assert!(shader.contains("@location(3) normals: vec3<f32>,"));
        assert!(shader.contains("world_normal"));
        assert!(shader.contains("cubemap1"));
        assert!(shader.contains("matcap1"));
        assert!(shader.contains("view_pos"));
    }

    // === Permutation count test ===

    #[test]
    fn test_all_16_permutations_generate() {
        // flags::ALL is 15 (0b1111), so we have 16 permutations (0-15)
        for i in 0..=flags::ALL {
            let shader = generate_shader(TEST_TEMPLATE, i);
            // Basic sanity: should contain vertex shader and fragment shader
            assert!(
                shader.contains("@vertex"),
                "Permutation {} missing @vertex",
                i
            );
            assert!(
                shader.contains("@fragment"),
                "Permutation {} missing @fragment",
                i
            );
            // Placeholders should be replaced (no // comments for features)
            assert!(
                !shader.contains("//VIN_"),
                "Permutation {} has unreplaced VIN placeholder",
                i
            );
            assert!(
                !shader.contains("//VOUT_"),
                "Permutation {} has unreplaced VOUT placeholder",
                i
            );
            assert!(
                !shader.contains("//VS_"),
                "Permutation {} has unreplaced VS placeholder",
                i
            );
            assert!(
                !shader.contains("//FS_"),
                "Permutation {} has unreplaced FS placeholder",
                i
            );
        }
    }

    // === Placeholder replacement tests ===

    #[test]
    fn test_placeholders_fully_replaced() {
        let placeholders = [
            "//VIN_COLOR",
            "//VIN_UV",
            "//VIN_NORMAL",
            "//VOUT_COLOR",
            "//VOUT_UV",
            "//VOUT_WORLD_NORMAL",
            "//VOUT_MATCAP",
            "//VS_COLOR",
            "//VS_UV",
            "//VS_WORLD_NORMAL",
            "//VS_MATCAP",
            "//FS_COLOR",
            "//FS_UV",
            "//FS_CUBEMAP",
            "//FS_MATCAP",
        ];

        // With all flags, all placeholders should be replaced with content
        let shader = generate_shader(TEST_TEMPLATE, flags::ALL);
        for placeholder in placeholders {
            assert!(
                !shader.contains(placeholder),
                "Placeholder {} was not replaced",
                placeholder
            );
        }

        // With no flags, all placeholders should be replaced with empty string
        let shader = generate_shader(TEST_TEMPLATE, 0);
        for placeholder in placeholders {
            assert!(
                !shader.contains(placeholder),
                "Placeholder {} was not replaced (flags=0)",
                placeholder
            );
        }
    }

    // === Output correctness tests ===

    #[test]
    fn test_vertex_color_uses_correct_location() {
        let shader = generate_shader(TEST_TEMPLATE, flags::VERTEX_COLOR);
        assert!(shader.contains("@location(1) color: vec3<f32>"));
    }

    #[test]
    fn test_uv_uses_correct_location() {
        let shader = generate_shader(TEST_TEMPLATE, flags::UV_TEXTURE);
        assert!(shader.contains("@location(2) uv: vec2<f32>"));
    }

    #[test]
    fn test_normal_uses_correct_location() {
        let shader = generate_shader(TEST_TEMPLATE, flags::CUBEMAP);
        assert!(shader.contains("@location(3) normals: vec3<f32>"));
    }

    #[test]
    fn test_fragment_blend_indices_are_correct() {
        let shader = generate_shader(TEST_TEMPLATE, flags::ALL);

        // Verify blend order matches TASKS.md specification:
        // 0: Vertex Color, 1: Texture, 2-3: Cubemap, 4-7: Matcap
        assert!(shader.contains("blend_colors(out_color, in.color.rgb, 0)"));
        assert!(shader.contains("texture_2d_sampler, in.uv).rgb, 1)"));
        assert!(shader.contains("cubemap1, texture_3d_sampler, in.world_normal).rgb, 2)"));
        assert!(shader.contains("cubemap2, texture_3d_sampler, in.world_normal).rgb, 3)"));
        assert!(shader.contains("matcap1, texture_3d_sampler, matcap_uv).rgb, 4)"));
        assert!(shader.contains("matcap2, texture_3d_sampler, matcap_uv).rgb, 5)"));
        assert!(shader.contains("matcap3, texture_3d_sampler, matcap_uv).rgb, 6)"));
        assert!(shader.contains("matcap4, texture_3d_sampler, matcap_uv).rgb, 7)"));
    }

    // === Constants validation tests ===

    #[test]
    fn test_vin_constants_valid_wgsl() {
        // All VIN constants should be valid WGSL attribute declarations
        assert!(VIN_COLOR.starts_with("@location("));
        assert!(VIN_COLOR.ends_with(","));
        assert!(VIN_UV.starts_with("@location("));
        assert!(VIN_UV.ends_with(","));
        assert!(VIN_NORMAL.starts_with("@location("));
        assert!(VIN_NORMAL.ends_with(","));
    }

    #[test]
    fn test_fs_color_references_blend_layer_0() {
        // Vertex color should use blend index 0
        assert!(FS_COLOR.contains(", 0)"));
    }

    #[test]
    fn test_fs_uv_references_blend_layer_1() {
        // UV texture should use blend index 1
        assert!(FS_UV.contains(", 1)"));
    }

    #[test]
    fn test_fs_cubemap_references_blend_layers_2_and_3() {
        // Cubemaps should use blend indices 2 and 3
        assert!(FS_CUBEMAP.contains(", 2)"));
        assert!(FS_CUBEMAP.contains(", 3)"));
    }

    #[test]
    fn test_fs_matcap_references_blend_layers_4_to_7() {
        // Matcaps should use blend indices 4-7
        assert!(FS_MATCAP.contains(", 4)"));
        assert!(FS_MATCAP.contains(", 5)"));
        assert!(FS_MATCAP.contains(", 6)"));
        assert!(FS_MATCAP.contains(", 7)"));
    }
}
