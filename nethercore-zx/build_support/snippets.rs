//! WGSL snippet strings inserted into templates by the build script.
//!
//! These are kept in a dedicated module so `build.rs` stays readable.

// Placeholder snippets (must match runtime shader_gen)
pub(crate) const VIN_UV: &str = "@location(1) uv: vec2<f32>,";
pub(crate) const VIN_COLOR: &str = "@location(2) color: vec3<f32>,";
pub(crate) const VIN_NORMAL: &str = "@location(3) normal_packed: u32,";
pub(crate) const VIN_SKINNED: &str =
    "@location(4) bone_indices: vec4<u32>,\n    @location(5) bone_weights: vec4<f32>,";
pub(crate) const VIN_TANGENT: &str = "@location(6) tangent_packed: u32,";

pub(crate) const VOUT_WORLD_NORMAL: &str = "@location(1) world_normal: vec3<f32>,";
pub(crate) const VOUT_VIEW_NORMAL: &str = "@location(2) view_normal: vec3<f32>,";
pub(crate) const VOUT_VIEW_POS: &str = "@location(4) view_position: vec3<f32>,";
pub(crate) const VOUT_CAMERA_POS: &str =
    "@location(5) @interpolate(flat) camera_position: vec3<f32>,";
pub(crate) const VOUT_UV: &str = "@location(10) uv: vec2<f32>,";
pub(crate) const VOUT_COLOR: &str = "@location(11) color: vec3<f32>,";
// Tangent vertex outputs: world-space tangent (location 6) + bitangent sign (location 7)
pub(crate) const VOUT_TANGENT: &str = "@location(6) world_tangent: vec3<f32>,\n    @location(7) @interpolate(flat) bitangent_sign: f32,";
// Mode 1: Additional view-space tangent output for matcap normal mapping (location 8)
pub(crate) const VOUT_VIEW_TANGENT: &str = "@location(8) view_tangent: vec3<f32>,";

pub(crate) const VS_UV: &str = "out.uv = in.uv;";
pub(crate) const VS_COLOR: &str = "out.color = in.color;";
pub(crate) const VS_WORLD_NORMAL: &str = "let normal = unpack_octahedral(in.normal_packed);\n    let world_normal_raw = (model_matrix * vec4<f32>(normal, 0.0)).xyz;\n    out.world_normal = normalize(world_normal_raw);";
pub(crate) const VS_VIEW_NORMAL: &str = "let view_rot = mat3x3<f32>(view_matrix[0].xyz, view_matrix[1].xyz, view_matrix[2].xyz);\n    out.view_normal = normalize(view_rot * out.world_normal);";
pub(crate) const VS_VIEW_POS: &str = "out.view_position = (view_matrix * model_pos).xyz;";
pub(crate) const VS_CAMERA_POS: &str =
    "out.camera_position = extract_camera_position(view_matrix);";

// Skinned variants of normal outputs.
pub(crate) const VS_WORLD_NORMAL_SKINNED: &str = "out.world_normal = normalize(final_normal);";
pub(crate) const VS_VIEW_NORMAL_SKINNED: &str = "let view_rot = mat3x3<f32>(view_matrix[0].xyz, view_matrix[1].xyz, view_matrix[2].xyz);\n    out.view_normal = normalize(view_rot * final_normal);";

// Mode 1 with tangent: compute view-space tangent for matcap normal mapping
pub(crate) const VS_VIEW_TANGENT: &str =
    "out.view_tangent = normalize(view_rot * out.world_tangent);";
pub(crate) const VS_VIEW_TANGENT_SKINNED: &str =
    "out.view_tangent = normalize(view_rot * final_tangent);";

// Tangent vertex shader code: unpack and transform to world space
pub(crate) const VS_TANGENT: &str = r#"let tangent_data = unpack_tangent(in.tangent_packed);
    let world_tangent_raw = (model_matrix * vec4<f32>(tangent_data.xyz, 0.0)).xyz;
    out.world_tangent = normalize(world_tangent_raw);
    out.bitangent_sign = tangent_data.w;"#;

// Tangent vertex shader code for skinned meshes (use skinned tangent instead)
pub(crate) const VS_TANGENT_SKINNED: &str =
    "out.world_tangent = normalize(final_tangent);\n    out.bitangent_sign = final_tangent_sign;";

pub(crate) const VS_SKINNED: &str = r#"// GPU skinning: compute skinned position, normal, and tangent
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

pub(crate) const VS_SKINNED_UNPACK_NORMAL: &str =
    "let input_normal = unpack_octahedral(in.normal_packed);";
pub(crate) const VS_SKINNED_NORMAL: &str =
    "skinned_normal += (bone_matrix * vec4<f32>(input_normal, 0.0)).xyz * weight;";
pub(crate) const VS_SKINNED_FINAL_NORMAL: &str = "let final_normal = normalize(skinned_normal);";

// Tangent skinning placeholders
pub(crate) const VS_SKINNED_UNPACK_TANGENT: &str = r#"let input_tangent_data = unpack_tangent(in.tangent_packed);
    let input_tangent = input_tangent_data.xyz;
    let input_tangent_sign = input_tangent_data.w;"#;
pub(crate) const VS_SKINNED_TANGENT: &str =
    "skinned_tangent += (bone_matrix * vec4<f32>(input_tangent, 0.0)).xyz * weight;";
pub(crate) const VS_SKINNED_FINAL_TANGENT: &str = r#"let final_tangent = normalize(skinned_tangent);
    let final_tangent_sign = input_tangent_sign;"#;

pub(crate) const VS_POSITION_SKINNED: &str = "let world_pos = vec4<f32>(final_position, 1.0);";
pub(crate) const VS_POSITION_UNSKINNED: &str = "let world_pos = vec4<f32>(in.position, 1.0);";

pub(crate) const FS_COLOR: &str = "color *= in.color;";
// Mode 0/1: Color/albedo from texture, with uniform color override support
// When FLAG_USE_UNIFORM_COLOR is NOT set, use texture alpha for dithering
pub(crate) const FS_UV: &str = r#"if !has_flag(shading.flags, FLAG_USE_UNIFORM_COLOR) {
        let tex_sample = sample_filtered(slot0, shading.flags, in.uv);
        color *= tex_sample.rgb;
        base_alpha = tex_sample.a;
    }"#;
// Mode 0 Lambert: ambient from environment gradient + save albedo for lighting (no tangent)
pub(crate) const FS_AMBIENT: &str = r#"let shading_normal = in.world_normal;
    let ambient = color * sample_environment_mode0_ambient(shading.environment_index, shading_normal, in.camera_position - in.world_position);
    let albedo = color;"#;

// Mode 0 Lambert: ambient with tangent/normal map support
pub(crate) const FS_AMBIENT_TANGENT: &str = r#"// Build TBN matrix and sample normal map
    let tbn = build_tbn(in.world_tangent, in.world_normal, in.bitangent_sign);
    let shading_normal = sample_normal_map(slot3, in.uv, tbn, shading.flags);
    let ambient = color * sample_environment_mode0_ambient(shading.environment_index, shading_normal, in.camera_position - in.world_position);
    let albedo = color;"#;

// Mode 0 Lambert: 4 dynamic lights only (no sun direct lighting)
pub(crate) const FS_NORMAL: &str = r#"var final_color = ambient;

    // 4 dynamic lights (Lambert diffuse only)
    for (var i = 0u; i < 4u; i++) {
        let light_data = unpack_light(shading.lights[i]);
        if (light_data.enabled) {
            let light = compute_light(light_data, in.world_position);
            final_color += lambert_diffuse(shading_normal, light.direction, albedo, light.color);
        }
    }

    color = final_color;"#;

pub(crate) const FS_ALBEDO_COLOR: &str = "albedo *= in.color;";
// Mode 2/3: Albedo from texture, with uniform color override support
// When FLAG_USE_UNIFORM_COLOR is NOT set, use texture alpha for dithering
pub(crate) const FS_ALBEDO_UV: &str = r#"if !has_flag(shading.flags, FLAG_USE_UNIFORM_COLOR) {
        let albedo_sample = sample_filtered(slot0, shading.flags, in.uv);
        albedo *= albedo_sample.rgb;
        base_alpha = albedo_sample.a;
    }"#;

// Mode 2/3: Shading normal computation (no tangent)
pub(crate) const FS_SHADING_NORMAL: &str = "let shading_normal = in.world_normal;";

// Mode 2/3: Shading normal with tangent/normal map support
pub(crate) const FS_SHADING_NORMAL_TANGENT: &str = r#"// Build TBN matrix and sample normal map
    let tbn = build_tbn(in.world_tangent, in.world_normal, in.bitangent_sign);
    let shading_normal = sample_normal_map(slot3, in.uv, tbn, shading.flags);"#;

// Mode 1 Matcap: Shading normal (both world and view space) - no tangent
pub(crate) const FS_MATCAP_SHADING_NORMAL: &str = r#"let shading_world_normal = normalize(in.world_normal);
    let shading_view_normal = normalize(in.view_normal);"#;

// Mode 1 Matcap: Shading normal with tangent/normal map support
// When tangent data present, slot3 is used for normal map (not 4th matcap)
// Uses both world-space and view-space TBN for world and view normals
pub(crate) const FS_MATCAP_SHADING_NORMAL_TANGENT: &str = r#"// Build world-space TBN and sample normal map
    let world_tbn = build_tbn(in.world_tangent, in.world_normal, in.bitangent_sign);
    let shading_world_normal = sample_normal_map(slot3, in.uv, world_tbn, shading.flags);
    // Build view-space TBN for matcap UV calculation (view_tangent passed from VS)
    let view_tbn = build_tbn(in.view_tangent, in.view_normal, in.bitangent_sign);
    let shading_view_normal = sample_normal_map(slot3, in.uv, view_tbn, shading.flags);"#;

// Mode 2/3: MRE/material texture sampling with override flag support
// Shared between Mode 2 (MRE = metallic/roughness/emissive) and Mode 3 (SDE = spec_damping/shininess/emissive)
// The variable name "mat_sample" is generic to work for both modes
pub(crate) const FS_MODE2_3_TEXTURES_UV: &str = r#"let mat_sample = sample_filtered(slot1, shading.flags, in.uv);
    if !has_flag(shading.flags, FLAG_USE_UNIFORM_METALLIC) {
        value0 = mat_sample.r;
    }
    if !has_flag(shading.flags, FLAG_USE_UNIFORM_ROUGHNESS) {
        value1 = mat_sample.g;
    }
    if !has_flag(shading.flags, FLAG_USE_UNIFORM_EMISSIVE) {
        emissive = mat_sample.b;
    }"#;
