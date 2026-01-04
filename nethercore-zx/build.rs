//! Build script for nethercore-zx
//!
//! Generates all 40 shader permutations at compile time and validates them with naga.
//! Also generates sky.wgsl from common utilities + sky_template.
//! This catches shader errors during `cargo build` rather than at runtime.
//!
//! The generated shaders are written to OUT_DIR as a Rust module that can be included
//! in shader_gen.rs for zero-cost shader loading at runtime.

use std::env;
use std::fs;
use std::path::Path;

// Vertex format flags (must match vertex.rs)
const FORMAT_UV: u8 = 1;
const FORMAT_COLOR: u8 = 2;
const FORMAT_NORMAL: u8 = 4;
const FORMAT_SKINNED: u8 = 8;
const FORMAT_TANGENT: u8 = 16;

// Shader template files (read at build time)
const COMMON: &str = include_str!("shaders/common.wgsl");
const BLINNPHONG_COMMON: &str = include_str!("shaders/blinnphong_common.wgsl");
const TEMPLATE_MODE0: &str = include_str!("shaders/mode0_lambert.wgsl");
const TEMPLATE_MODE1: &str = include_str!("shaders/mode1_matcap.wgsl");
const SKY_TEMPLATE: &str = include_str!("shaders/sky_template.wgsl");
const QUAD_TEMPLATE: &str = include_str!("shaders/quad_template.wgsl");

// Placeholder snippets (must match shader_gen.rs)
const VIN_UV: &str = "@location(1) uv: vec2<f32>,";
const VIN_COLOR: &str = "@location(2) color: vec3<f32>,";
const VIN_NORMAL: &str = "@location(3) normal_packed: u32,";
const VIN_SKINNED: &str =
    "@location(4) bone_indices: vec4<u32>,\n    @location(5) bone_weights: vec4<f32>,";
const VIN_TANGENT: &str = "@location(6) tangent_packed: u32,";

const VOUT_WORLD_NORMAL: &str = "@location(1) world_normal: vec3<f32>,";
const VOUT_VIEW_NORMAL: &str = "@location(2) view_normal: vec3<f32>,";
const VOUT_VIEW_POS: &str = "@location(4) view_position: vec3<f32>,";
const VOUT_CAMERA_POS: &str = "@location(5) @interpolate(flat) camera_position: vec3<f32>,";
const VOUT_UV: &str = "@location(10) uv: vec2<f32>,";
const VOUT_COLOR: &str = "@location(11) color: vec3<f32>,";
// Tangent vertex outputs: world-space tangent (location 6) + bitangent sign (location 7)
const VOUT_TANGENT: &str = "@location(6) world_tangent: vec3<f32>,\n    @location(7) @interpolate(flat) bitangent_sign: f32,";

// Mode 1: Additional view-space tangent output for matcap normal mapping (location 8)
const VOUT_VIEW_TANGENT: &str = "@location(8) view_tangent: vec3<f32>,";

const VS_UV: &str = "out.uv = in.uv;";
const VS_COLOR: &str = "out.color = in.color;";
const VS_WORLD_NORMAL: &str = "let normal = unpack_octahedral(in.normal_packed);\n    let world_normal_raw = (model_matrix * vec4<f32>(normal, 0.0)).xyz;\n    out.world_normal = normalize(world_normal_raw);";
const VS_VIEW_NORMAL: &str = "let view_rot = mat3x3<f32>(view_matrix[0].xyz, view_matrix[1].xyz, view_matrix[2].xyz);\n    out.view_normal = normalize(view_rot * out.world_normal);";
const VS_VIEW_POS: &str = "out.view_position = (view_matrix * model_pos).xyz;";
const VS_CAMERA_POS: &str = "out.camera_position = extract_camera_position(view_matrix);";

// Mode 1 with tangent: compute view-space tangent for matcap normal mapping
const VS_VIEW_TANGENT: &str = "out.view_tangent = normalize(view_rot * out.world_tangent);";
const VS_VIEW_TANGENT_SKINNED: &str = "out.view_tangent = normalize(view_rot * final_tangent);";

// Tangent vertex shader code: unpack and transform to world space
const VS_TANGENT: &str = r#"let tangent_data = unpack_tangent(in.tangent_packed);
    let world_tangent_raw = (model_matrix * vec4<f32>(tangent_data.xyz, 0.0)).xyz;
    out.world_tangent = normalize(world_tangent_raw);
    out.bitangent_sign = tangent_data.w;"#;

// Tangent vertex shader code for skinned meshes (use skinned tangent instead)
const VS_TANGENT_SKINNED: &str = "out.world_tangent = normalize(final_tangent);\n    out.bitangent_sign = final_tangent_sign;";

const VS_SKINNED: &str = r#"// GPU skinning: compute skinned position, normal, and tangent
    // Animation System v2 (Unified Buffer): keyframe_base and inverse_bind_base
    // point directly into unified_animation buffer (offsets pre-computed on CPU)
    // - Skinning mode (FLAG_SKINNING_MODE): 0 = raw, 1 = apply inverse bind
    let shading_state_for_skinning = shading_states[shading_state_idx];
    let use_inverse_bind = (shading_state_for_skinning.flags & FLAG_SKINNING_MODE) != 0u;
    let keyframe_base = shading_state_for_skinning.keyframe_base;
    let inverse_bind_base = shading_state_for_skinning.inverse_bind_base;

    var skinned_pos = vec3<f32>(0.0, 0.0, 0.0);
    var skinned_normal = vec3<f32>(0.0, 0.0, 0.0);
    //VS_SKINNED_UNPACK_NORMAL
    var skinned_tangent = vec3<f32>(0.0, 0.0, 0.0);
    //VS_SKINNED_UNPACK_TANGENT

    for (var i = 0u; i < 4u; i++) {
        let bone_idx = in.bone_indices[i];
        let weight = in.bone_weights[i];

        if (weight > 0.0 && bone_idx < 256u) {
            // Get bone matrix from unified_animation (CPU pre-computed keyframe_base)
            var bone_matrix = bone_to_mat4(unified_animation[keyframe_base + bone_idx]);

            // Apply inverse bind if in inverse bind mode (skeleton is bound via skeleton_bind)
            if (use_inverse_bind) {
                let inv_bind = bone_to_mat4(unified_animation[inverse_bind_base + bone_idx]);
                bone_matrix = bone_matrix * inv_bind;
            }

            skinned_pos += (bone_matrix * vec4<f32>(in.position, 1.0)).xyz * weight;
            //VS_SKINNED_NORMAL
            //VS_SKINNED_TANGENT
        }
    }

    let final_position = skinned_pos;
    //VS_SKINNED_FINAL_NORMAL
    //VS_SKINNED_FINAL_TANGENT"#;

const VS_SKINNED_UNPACK_NORMAL: &str = "let input_normal = unpack_octahedral(in.normal_packed);";
const VS_SKINNED_NORMAL: &str =
    "skinned_normal += (bone_matrix * vec4<f32>(input_normal, 0.0)).xyz * weight;";
const VS_SKINNED_FINAL_NORMAL: &str = "let final_normal = normalize(skinned_normal);";

// Tangent skinning placeholders
const VS_SKINNED_UNPACK_TANGENT: &str = r#"let input_tangent_data = unpack_tangent(in.tangent_packed);
    let input_tangent = input_tangent_data.xyz;
    let input_tangent_sign = input_tangent_data.w;"#;
const VS_SKINNED_TANGENT: &str =
    "skinned_tangent += (bone_matrix * vec4<f32>(input_tangent, 0.0)).xyz * weight;";
const VS_SKINNED_FINAL_TANGENT: &str = r#"let final_tangent = normalize(skinned_tangent);
    let final_tangent_sign = input_tangent_sign;"#;

const VS_POSITION_SKINNED: &str = "let world_pos = vec4<f32>(final_position, 1.0);";
const VS_POSITION_UNSKINNED: &str = "let world_pos = vec4<f32>(in.position, 1.0);";

const FS_COLOR: &str = "color *= in.color;";
// Mode 0/1: Color/albedo from texture, with uniform color override support
// When FLAG_USE_UNIFORM_COLOR is NOT set, use texture alpha for dithering
const FS_UV: &str = r#"if !has_flag(shading.flags, FLAG_USE_UNIFORM_COLOR) {
        let tex_sample = sample_filtered(slot0, shading.flags, in.uv);
        color *= tex_sample.rgb;
        base_alpha = tex_sample.a;
    }"#;
// Mode 0 Lambert: ambient from environment gradient + save albedo for lighting (no tangent)
const FS_AMBIENT: &str = r#"let shading_normal = in.world_normal;
    let ambient = color * sample_environment_ambient(shading.environment_index, shading_normal);
    let albedo = color;"#;

// Mode 0 Lambert: ambient with tangent/normal map support
const FS_AMBIENT_TANGENT: &str = r#"// Build TBN matrix and sample normal map
    let tbn = build_tbn(in.world_tangent, in.world_normal, in.bitangent_sign);
    let shading_normal = sample_normal_map(slot3, in.uv, tbn, shading.flags);
    let ambient = color * sample_environment_ambient(shading.environment_index, shading_normal);
    let albedo = color;"#;

// Mode 0 Lambert: 4 dynamic lights only (no sun direct lighting)
const FS_NORMAL: &str = r#"var final_color = ambient;

    // 4 dynamic lights (Lambert diffuse only)
    for (var i = 0u; i < 4u; i++) {
        let light_data = unpack_light(shading.lights[i]);
        if (light_data.enabled) {
            let light = compute_light(light_data, in.world_position);
            final_color += lambert_diffuse(shading_normal, light.direction, albedo, light.color);
        }
    }

    color = final_color;"#;

const FS_ALBEDO_COLOR: &str = "albedo *= in.color;";
// Mode 2/3: Albedo from texture, with uniform color override support
// When FLAG_USE_UNIFORM_COLOR is NOT set, use texture alpha for dithering
const FS_ALBEDO_UV: &str = r#"if !has_flag(shading.flags, FLAG_USE_UNIFORM_COLOR) {
        let albedo_sample = sample_filtered(slot0, shading.flags, in.uv);
        albedo *= albedo_sample.rgb;
        base_alpha = albedo_sample.a;
    }"#;

// Mode 2/3: Shading normal computation (no tangent)
const FS_SHADING_NORMAL: &str = "let shading_normal = in.world_normal;";

// Mode 2/3: Shading normal with tangent/normal map support
const FS_SHADING_NORMAL_TANGENT: &str = r#"// Build TBN matrix and sample normal map
    let tbn = build_tbn(in.world_tangent, in.world_normal, in.bitangent_sign);
    let shading_normal = sample_normal_map(slot3, in.uv, tbn, shading.flags);"#;

// Mode 1 Matcap: Shading normal (both world and view space) - no tangent
const FS_MATCAP_SHADING_NORMAL: &str = r#"let shading_world_normal = normalize(in.world_normal);
    let shading_view_normal = normalize(in.view_normal);"#;

// Mode 1 Matcap: Shading normal with tangent/normal map support
// When tangent data present, slot3 is used for normal map (not 4th matcap)
// Uses both world-space and view-space TBN for world and view normals
const FS_MATCAP_SHADING_NORMAL_TANGENT: &str = r#"// Build world-space TBN and sample normal map
    let world_tbn = build_tbn(in.world_tangent, in.world_normal, in.bitangent_sign);
    let shading_world_normal = sample_normal_map(slot3, in.uv, world_tbn, shading.flags);
    // Build view-space TBN for matcap UV calculation (view_tangent passed from VS)
    let view_tbn = build_tbn(in.view_tangent, in.view_normal, in.bitangent_sign);
    let shading_view_normal = sample_normal_map(slot3, in.uv, view_tbn, shading.flags);"#;

// Mode 2/3: MRE/material texture sampling with override flag support
// Shared between Mode 2 (MRE = metallic/roughness/emissive) and Mode 3 (SDE = spec_damping/shininess/emissive)
// The variable name "mat_sample" is generic to work for both modes
const FS_MODE2_3_TEXTURES_UV: &str = r#"let mat_sample = sample_filtered(slot1, shading.flags, in.uv);
    if !has_flag(shading.flags, FLAG_USE_UNIFORM_METALLIC) {
        value0 = mat_sample.r;
    }
    if !has_flag(shading.flags, FLAG_USE_UNIFORM_ROUGHNESS) {
        value1 = mat_sample.g;
    }
    if !has_flag(shading.flags, FLAG_USE_UNIFORM_EMISSIVE) {
        emissive = mat_sample.b;
    }"#;

/// Generate a shader for a specific mode and vertex format
fn generate_shader(mode: u8, format: u8) -> Result<String, String> {
    // Validate mode
    if mode > 3 {
        return Err(format!("Invalid render mode: {} (must be 0-3)", mode));
    }

    // Get the appropriate template
    let template = match mode {
        0 => TEMPLATE_MODE0,
        1 => TEMPLATE_MODE1,
        _ => "", // Modes 2-3 use BLINNPHONG_COMMON
    };

    // Extract format flags
    let has_uv = format & FORMAT_UV != 0;
    let has_color = format & FORMAT_COLOR != 0;
    let has_normal = format & FORMAT_NORMAL != 0;
    let has_skinned = format & FORMAT_SKINNED != 0;
    let has_tangent = format & FORMAT_TANGENT != 0;

    // Validate format constraints
    if mode > 0 && !has_normal {
        return Err(format!(
            "Render mode {} requires NORMAL flag, but format {} doesn't have it",
            mode, format
        ));
    }
    if has_tangent && !has_normal {
        return Err(format!(
            "TANGENT flag requires NORMAL flag, but format {} doesn't have it",
            format
        ));
    }

    // Build shader by combining common code + mode-specific template
    let mut shader = String::new();
    shader.push_str(COMMON);
    shader.push('\n');

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
    shader = shader.replace("//VIN_TANGENT", if has_tangent { VIN_TANGENT } else { "" });

    // Replace vertex output placeholders
    shader = shader.replace("//VOUT_UV", if has_uv { VOUT_UV } else { "" });
    shader = shader.replace("//VOUT_COLOR", if has_color { VOUT_COLOR } else { "" });
    shader = shader.replace(
        "//VOUT_WORLD_NORMAL",
        if has_normal { VOUT_WORLD_NORMAL } else { "" },
    );
    shader = shader.replace(
        "//VOUT_VIEW_NORMAL",
        if has_normal { VOUT_VIEW_NORMAL } else { "" },
    );
    shader = shader.replace(
        "//VOUT_VIEW_POS",
        if mode == 1 && has_normal {
            VOUT_VIEW_POS
        } else {
            ""
        },
    );
    shader = shader.replace(
        "//VOUT_CAMERA_POS",
        if mode >= 2 { VOUT_CAMERA_POS } else { "" },
    );
    shader = shader.replace(
        "//VOUT_TANGENT",
        if has_tangent { VOUT_TANGENT } else { "" },
    );
    // Mode 1 with tangent needs view-space tangent for matcap normal mapping
    shader = shader.replace(
        "//VOUT_VIEW_TANGENT",
        if mode == 1 && has_tangent { VOUT_VIEW_TANGENT } else { "" },
    );

    // Replace vertex shader code placeholders
    shader = shader.replace("//VS_UV", if has_uv { VS_UV } else { "" });
    shader = shader.replace("//VS_COLOR", if has_color { VS_COLOR } else { "" });

    // Normal handling depends on skinning
    if has_normal && !has_skinned {
        shader = shader.replace("//VS_WORLD_NORMAL", VS_WORLD_NORMAL);
        shader = shader.replace("//VS_VIEW_NORMAL", VS_VIEW_NORMAL);
    } else if has_normal && has_skinned {
        let skinned_world_normal = "out.world_normal = normalize(final_normal);";
        let skinned_view_normal = "let view_rot = mat3x3<f32>(view_matrix[0].xyz, view_matrix[1].xyz, view_matrix[2].xyz);\n    out.view_normal = normalize(view_rot * final_normal);";
        shader = shader.replace("//VS_WORLD_NORMAL", skinned_world_normal);
        shader = shader.replace("//VS_VIEW_NORMAL", skinned_view_normal);
    } else {
        shader = shader.replace("//VS_WORLD_NORMAL", "");
        shader = shader.replace("//VS_VIEW_NORMAL", "");
    }

    // Tangent handling depends on skinning
    if has_tangent && !has_skinned {
        shader = shader.replace("//VS_TANGENT", VS_TANGENT);
    } else if has_tangent && has_skinned {
        shader = shader.replace("//VS_TANGENT", VS_TANGENT_SKINNED);
    } else {
        shader = shader.replace("//VS_TANGENT", "");
    }

    // Mode 1 with tangent needs view-space tangent for matcap normal mapping
    if mode == 1 && has_tangent && !has_skinned {
        shader = shader.replace("//VS_VIEW_TANGENT", VS_VIEW_TANGENT);
    } else if mode == 1 && has_tangent && has_skinned {
        shader = shader.replace("//VS_VIEW_TANGENT", VS_VIEW_TANGENT_SKINNED);
    } else {
        shader = shader.replace("//VS_VIEW_TANGENT", "");
    }

    // View position (mode 1 only, for perspective-correct matcap)
    if mode == 1 && has_normal {
        shader = shader.replace("//VS_VIEW_POS", VS_VIEW_POS);
    } else {
        shader = shader.replace("//VS_VIEW_POS", "");
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
            "//VS_SKINNED_UNPACK_NORMAL",
            if has_normal {
                VS_SKINNED_UNPACK_NORMAL
            } else {
                ""
            },
        );
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
        // Tangent skinning placeholders
        skinned_code = skinned_code.replace(
            "//VS_SKINNED_UNPACK_TANGENT",
            if has_tangent {
                VS_SKINNED_UNPACK_TANGENT
            } else {
                ""
            },
        );
        skinned_code = skinned_code.replace(
            "//VS_SKINNED_TANGENT",
            if has_tangent {
                VS_SKINNED_TANGENT
            } else {
                ""
            },
        );
        skinned_code = skinned_code.replace(
            "//VS_SKINNED_FINAL_TANGENT",
            if has_tangent {
                VS_SKINNED_FINAL_TANGENT
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
            shader = shader.replace("//FS_COLOR", if has_color { FS_COLOR } else { "" });
            shader = shader.replace("//FS_UV", if has_uv { FS_UV } else { "" });
            // Tangent normal mapping requires both tangent data and UVs
            let ambient = if has_tangent && has_uv {
                FS_AMBIENT_TANGENT
            } else if has_normal {
                FS_AMBIENT
            } else {
                ""
            };
            shader = shader.replace("//FS_AMBIENT", ambient);
            shader = shader.replace("//FS_NORMAL", if has_normal { FS_NORMAL } else { "" });
        }
        1 => {
            shader = shader.replace("//FS_COLOR", if has_color { FS_COLOR } else { "" });
            shader = shader.replace("//FS_UV", if has_uv { FS_UV } else { "" });
            shader = shader.replace("//FS_AMBIENT", "");
            // Matcap shading normal: use TBN+normal map if tangent data and UVs are present
            let matcap_normal = if has_tangent && has_uv {
                FS_MATCAP_SHADING_NORMAL_TANGENT
            } else {
                FS_MATCAP_SHADING_NORMAL
            };
            shader = shader.replace("//FS_MATCAP_SHADING_NORMAL", matcap_normal);
        }
        2 => {
            shader = shader.replace("//FS_COLOR", if has_color { FS_ALBEDO_COLOR } else { "" });
            shader = shader.replace("//FS_UV", if has_uv { FS_ALBEDO_UV } else { "" });
            shader = shader.replace("//FS_AMBIENT", "");
            // Shading normal: use TBN+normal map if tangent data and UVs are present
            let shading_normal = if has_tangent && has_uv {
                FS_SHADING_NORMAL_TANGENT
            } else {
                FS_SHADING_NORMAL
            };
            shader = shader.replace("//FS_SHADING_NORMAL", shading_normal);
            shader = shader.replace(
                "//FS_MODE2_3_DIFFUSE_FACTOR",
                "let diffuse_factor = 1.0 - value0;",
            );
            shader = shader.replace(
                "//FS_MODE2_3_SHININESS",
                "let shininess = mix(1.0, 256.0, 1.0 - value1);",
            );
            shader = shader.replace("//FS_MODE2_3_ROUGHNESS", "let roughness = value1;");
            shader = shader.replace(
                "//FS_MODE2_3_SPECULAR_COLOR",
                "let specular_color = mix(vec3<f32>(0.04), albedo, value0);",
            );
            if has_uv {
                // Mode 2: MRE texture sampling with override flag support (uses shared constant)
                shader = shader.replace("//FS_MODE2_3_TEXTURES", FS_MODE2_3_TEXTURES_UV);
            } else {
                shader = shader.replace("//FS_MODE2_3_TEXTURES", "");
            }
            // Mode 2: Energy conservation - diffuse reduced by Fresnel (roughness-dependent)
            // value1 contains roughness at this point (before conversion to shininess)
            shader = shader.replace(
                "//FS_MODE2_3_DIFFUSE_FRESNEL",
                "let diffuse_fresnel = mix(one_minus_F, vec3<f32>(1.0), value1);",
            );
        }
        3 => {
            shader = shader.replace("//FS_COLOR", if has_color { FS_ALBEDO_COLOR } else { "" });
            shader = shader.replace("//FS_UV", if has_uv { FS_ALBEDO_UV } else { "" });
            shader = shader.replace("//FS_AMBIENT", "");
            // Shading normal: use TBN+normal map if tangent data and UVs are present
            let shading_normal = if has_tangent && has_uv {
                FS_SHADING_NORMAL_TANGENT
            } else {
                FS_SHADING_NORMAL
            };
            shader = shader.replace("//FS_SHADING_NORMAL", shading_normal);
            shader = shader.replace("//FS_MODE2_3_DIFFUSE_FACTOR", "let diffuse_factor = 1.0;");
            shader = shader.replace(
                "//FS_MODE2_3_SHININESS",
                "let shininess = mix(1.0, 256.0, value1);",
            );
            shader = shader.replace(
                "//FS_MODE2_3_ROUGHNESS",
                "let roughness = 1.0 - (shininess - 1.0) / 255.0;",
            );
            // Mode 3 uses INVERTED spec_damping: 0 = full specular, 255 = no specular
            // This is beginner-friendly: default of 0 gives visible highlights
            // uniform_set_1 format: 0xRRGGBBRP (big-endian, same as color_rgba8)
            // Mode 3 specular color: supports both texture and uniform sources
            // Uses unpack_specular_rgb() helper to avoid code duplication
            let specular = if has_uv {
                // With UV: check flag to decide between texture and uniform
                r#"var specular_color: vec3<f32>;
    if has_flag(shading.flags, FLAG_USE_UNIFORM_SPECULAR) {
        specular_color = unpack_specular_rgb(shading.uniform_set_1) * (1.0 - value0);
    } else {
        specular_color = sample_filtered(slot2, shading.flags, in.uv).rgb * (1.0 - value0);
    }"#
            } else {
                // Without UV: always use uniform specular color
                "var specular_color = unpack_specular_rgb(shading.uniform_set_1) * (1.0 - value0);"
            };
            shader = shader.replace("//FS_MODE2_3_SPECULAR_COLOR", specular);
            if has_uv {
                // Mode 3: slot1 texture sampling with override flag support (uses shared constant)
                shader = shader.replace("//FS_MODE2_3_TEXTURES", FS_MODE2_3_TEXTURES_UV);
            } else {
                shader = shader.replace("//FS_MODE2_3_TEXTURES", "");
            }
            // Mode 3: No energy conservation - full diffuse always (artistic freedom)
            shader = shader.replace(
                "//FS_MODE2_3_DIFFUSE_FRESNEL",
                "let diffuse_fresnel = vec3<f32>(1.0);",
            );
        }
        _ => {}
    }

    Ok(shader)
}

/// Validate a shader using naga
fn validate_shader(source: &str, mode: u8, format: u8) -> Result<(), String> {
    // Parse the WGSL source
    let module = naga::front::wgsl::parse_str(source).map_err(|e| {
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

/// Validate a shader using naga (generic version for named shaders)
fn validate_shader_generic(source: &str, name: &str) -> Result<(), String> {
    let module = naga::front::wgsl::parse_str(source)
        .map_err(|e| format!("WGSL parse error for {}: {:?}", name, e))?;

    let mut validator = naga::valid::Validator::new(
        naga::valid::ValidationFlags::all(),
        naga::valid::Capabilities::all(),
    );

    validator
        .validate(&module)
        .map_err(|e| format!("Validation error for {}: {:?}", name, e))?;

    Ok(())
}

/// Get valid formats for a render mode
/// Mode 0: all 32 formats (0-31)
/// Modes 1-3: 16 formats each (must have NORMAL, optionally TANGENT)
fn valid_formats_for_mode(mode: u8) -> Vec<u8> {
    // Tangent requires normal: filter out formats with TANGENT but without NORMAL
    let tangent_valid = |f: &u8| {
        let has_tangent = (*f & FORMAT_TANGENT) != 0;
        let has_normal = (*f & FORMAT_NORMAL) != 0;
        !has_tangent || has_normal // tangent requires normal
    };

    match mode {
        // Mode 0: all combinations except invalid tangent-without-normal
        // Valid: 0-15 (no tangent), 20-23 (tangent+normal), 28-31 (tangent+normal+skinned)
        0 => (0..32).filter(tangent_valid).collect(),
        // Modes 1-3: require NORMAL, plus tangent validation
        1..=3 => (0..32)
            .filter(|&f| f & FORMAT_NORMAL != 0)
            .filter(tangent_valid)
            .collect(),
        _ => vec![],
    }
}

/// Extract bindings section from common.wgsl (up to "// Data Unpacking Utilities")
fn extract_common_bindings() -> &'static str {
    // Find the section by looking for the section title (not the full header line)
    let marker = "// Data Unpacking Utilities";
    if let Some(marker_pos) = COMMON.find(marker) {
        // Find the start of the section divider line (=== line) before the marker
        let section_start = COMMON[..marker_pos].rfind("// ===").unwrap_or(marker_pos);
        &COMMON[..section_start]
    } else {
        panic!("Could not find '{}' in common.wgsl", marker);
    }
}

/// Extract utility functions from common.wgsl (from Data Unpacking to Unified Vertex Input)
fn extract_common_utilities() -> &'static str {
    let start_marker = "// Data Unpacking Utilities";
    let end_marker = "// Unified Vertex Input/Output";

    let start_pos = COMMON
        .find(start_marker)
        .expect("Could not find start marker in common.wgsl");
    let start = COMMON[..start_pos].rfind("// ===").unwrap_or(start_pos);

    let end_pos = COMMON
        .find(end_marker)
        .expect("Could not find end marker in common.wgsl");
    let end = COMMON[..end_pos].rfind("// ===").unwrap_or(end_pos);

    &COMMON[start..end]
}

/// Generate sky.wgsl from common + sky_template
fn generate_sky_shader() -> String {
    let mut shader = String::new();
    shader.push_str("// Auto-generated from common.wgsl + sky_template.wgsl\n");
    shader.push_str("// DO NOT EDIT - regenerate with cargo build\n\n");
    shader.push_str(extract_common_bindings());
    shader.push_str(extract_common_utilities());
    shader.push_str(SKY_TEMPLATE);
    shader
}

/// Generate quad.wgsl from common + quad_template
fn generate_quad_shader() -> String {
    let mut shader = String::new();
    shader.push_str("// Auto-generated from common.wgsl + quad_template.wgsl\n");
    shader.push_str("// DO NOT EDIT - regenerate with cargo build\n\n");
    shader.push_str(extract_common_bindings());
    shader.push_str(extract_common_utilities());
    shader.push_str(QUAD_TEMPLATE);
    shader
}

fn check_ffi_freshness() {
    let ffi_source = Path::new("../include/zx.rs");
    let c_header = Path::new("../include/zx.h");
    let zig_bindings = Path::new("../include/zx.zig");

    println!("cargo:rerun-if-changed=../include/zx.rs");

    let source_time = match fs::metadata(ffi_source).and_then(|m| m.modified()) {
        Ok(t) => t,
        Err(_) => return,
    };

    let c_time = fs::metadata(c_header).and_then(|m| m.modified()).ok();
    let zig_time = fs::metadata(zig_bindings).and_then(|m| m.modified()).ok();

    let c_stale = c_time.is_none_or(|t| source_time > t);
    let zig_stale = zig_time.is_none_or(|t| source_time > t);

    if c_stale || zig_stale {
        panic!("FFI bindings are stale! Run: cargo xtask ffi generate");
    }
}

fn main() {
    check_ffi_freshness();

    // Tell Cargo to rerun this if the shader templates change
    println!("cargo:rerun-if-changed=shaders/common.wgsl");
    println!("cargo:rerun-if-changed=shaders/blinnphong_common.wgsl");
    println!("cargo:rerun-if-changed=shaders/mode0_lambert.wgsl");
    println!("cargo:rerun-if-changed=shaders/mode1_matcap.wgsl");
    println!("cargo:rerun-if-changed=shaders/sky_template.wgsl");
    println!("cargo:rerun-if-changed=shaders/quad_template.wgsl");

    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("generated_shaders.rs");

    let mut generated_code = String::new();
    generated_code.push_str("// Auto-generated by build.rs - DO NOT EDIT\n");
    generated_code.push_str("// Contains all 72 pregenerated shader permutations (with tangent support)\n\n");

    // Generate array of 72 shader sources
    // Mode 0: 24 formats (0-15, 20-23, 28-31), Modes 1-3: 16 formats each = 24 + 16*3 = 72
    generated_code
        .push_str("/// Pregenerated shader sources indexed by shader_index(mode, format)\n");
    generated_code.push_str("pub const PREGENERATED_SHADERS: [&str; 72] = [\n");

    let mut shader_count = 0;
    let mut errors: Vec<String> = Vec::new();

    // Mode 0: valid formats (0-15, 20-23, 28-31) - tangent requires normal
    for format in valid_formats_for_mode(0) {
        match generate_shader(0, format) {
            Ok(source) => {
                if let Err(e) = validate_shader(&source, 0, format) {
                    errors.push(e);
                }
                generated_code.push_str(&format!("    // Mode 0, Format {}\n", format));
                generated_code.push_str(&format!("    r#\"{}\"#,\n", source));
                shader_count += 1;
            }
            Err(e) => errors.push(e),
        }
    }

    // Modes 1-3: only formats with NORMAL (4-7, 12-15, 20-23, 28-31)
    for mode in 1u8..=3 {
        for format in valid_formats_for_mode(mode) {
            match generate_shader(mode, format) {
                Ok(source) => {
                    if let Err(e) = validate_shader(&source, mode, format) {
                        errors.push(e);
                    }
                    generated_code.push_str(&format!("    // Mode {}, Format {}\n", mode, format));
                    generated_code.push_str(&format!("    r#\"{}\"#,\n", source));
                    shader_count += 1;
                }
                Err(e) => errors.push(e),
            }
        }
    }

    generated_code.push_str("];\n\n");

    // Generate index lookup function
    generated_code.push_str(
        r#"/// Get pregenerated shader source by mode and format
/// Returns None if the mode/format combination is invalid
pub fn get_pregenerated_shader(mode: u8, format: u8) -> Option<&'static str> {
    const FORMAT_NORMAL: u8 = 4;
    const FORMAT_SKINNED: u8 = 8;
    const FORMAT_TANGENT: u8 = 16;

    // Validate mode/format combination
    if mode > 3 {
        return None;
    }
    if format >= 32 {
        return None; // Invalid format
    }
    if mode > 0 && format & FORMAT_NORMAL == 0 {
        return None; // Modes 1-3 require NORMAL
    }
    if format & FORMAT_TANGENT != 0 && format & FORMAT_NORMAL == 0 {
        return None; // TANGENT requires NORMAL
    }

    // Index calculation:
    // Mode 0: 24 formats (0-15, 20-23, 28-31)
    //   - 0-15: indices 0-15
    //   - 20-23: indices 16-19
    //   - 28-31: indices 20-23
    // Modes 1-3: 16 formats each (4-7, 12-15, 20-23, 28-31)
    //   - Map format to 0-15 offset: (UV + COLOR*2) + SKINNED*4 + TANGENT*8
    let index = match mode {
        0 => {
            // Mode 0: direct index for 0-15, offset for tangent variants
            if format < 16 {
                format as usize
            } else if format >= 20 && format <= 23 {
                16 + (format - 20) as usize // tangent+normal (no skinned)
            } else if format >= 28 && format <= 31 {
                20 + (format - 28) as usize // tangent+normal+skinned
            } else {
                return None; // Invalid format for mode 0
            }
        }
        1 => {
            let base = 24usize; // After mode 0's 24 shaders
            let offset = (format & 0b0011) as usize
                + if format & FORMAT_SKINNED != 0 { 4 } else { 0 }
                + if format & FORMAT_TANGENT != 0 { 8 } else { 0 };
            base + offset
        }
        2 => {
            let base = 24 + 16; // After mode 0 and mode 1
            let offset = (format & 0b0011) as usize
                + if format & FORMAT_SKINNED != 0 { 4 } else { 0 }
                + if format & FORMAT_TANGENT != 0 { 8 } else { 0 };
            base + offset
        }
        3 => {
            let base = 24 + 16 + 16; // After mode 0, 1, and 2
            let offset = (format & 0b0011) as usize
                + if format & FORMAT_SKINNED != 0 { 4 } else { 0 }
                + if format & FORMAT_TANGENT != 0 { 8 } else { 0 };
            base + offset
        }
        _ => return None,
    };

    Some(PREGENERATED_SHADERS[index])
}
"#,
    );

    // Generate sky shader
    let sky_shader = generate_sky_shader();
    if let Err(e) = validate_shader_generic(&sky_shader, "sky") {
        errors.push(e);
    }
    generated_code.push_str("\n/// Generated sky shader source\n");
    generated_code.push_str(&format!(
        "pub const SKY_SHADER: &str = r#\"{}\"#;\n",
        sky_shader
    ));

    // Generate quad shader
    let quad_shader = generate_quad_shader();
    if let Err(e) = validate_shader_generic(&quad_shader, "quad_unlit") {
        errors.push(e);
    }
    generated_code.push_str("\n/// Generated quad unlit shader source\n");
    generated_code.push_str(&format!(
        "pub const QUAD_SHADER: &str = r#\"{}\"#;\n",
        quad_shader
    ));

    // Check for errors
    if !errors.is_empty() {
        panic!(
            "Shader generation failed with {} errors:\n{}",
            errors.len(),
            errors.join("\n")
        );
    }

    // Expected: 24 (mode 0) + 16*3 (modes 1-3) = 72 shaders
    // Mode 0: formats 0-15 + 20-23 + 28-31 (tangent requires normal)
    // Modes 1-3: formats 4-7, 12-15, 20-23, 28-31 (all require normal)
    assert_eq!(
        shader_count, 72,
        "Expected 72 shaders, got {}",
        shader_count
    );

    // Write the generated code
    fs::write(&dest_path, generated_code).expect("Failed to write generated_shaders.rs");

    println!(
        "cargo:warning=Generated {} shaders successfully (+ sky + quad)",
        shader_count
    );
}
