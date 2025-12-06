// ============================================================================
// BLINN-PHONG LIGHTING SYSTEM
// Modes 2-3: Normalized Blinn-Phong with mode-specific parameters
// Reference: Gotanda 2010 - "Practical Implementation at tri-Ace"
// ============================================================================

// NOTE: Vertex shader (VertexIn/VertexOut structs and @vertex fn) is injected by shader_gen.rs from common.wgsl
// NOTE: Common bindings, structures, and utilities are injected by shader_gen.rs from common.wgsl

// ============================================================================
// Lighting Functions
// ============================================================================

// Gotanda 2010 linear approximation (Equation 12)
// Fitted for shininess 0-1000, extrapolates fine beyond
// Energy-conserving normalization factor for Blinn-Phong BRDF
fn gotanda_normalization(shininess: f32) -> f32 {
    return shininess * 0.0397436 + 0.0856832;
}

// Normalized Blinn-Phong specular lighting
// Convention: light_dir = direction rays travel (negate for lighting calculations)
// No geometry term - era-authentic, classical Blinn-Phong didn't have it
fn normalized_blinn_phong_specular(
    N: vec3<f32>,               // Surface normal (normalized)
    V: vec3<f32>,               // View direction (normalized, surface TO camera)
    light_dir: vec3<f32>,       // Light direction (direction rays travel)
    shininess: f32,             // 1-256 range (mapped from roughness/texture)
    specular_color: vec3<f32>,  // Specular highlight color
    light_color: vec3<f32>,
) -> vec3<f32> {
    // Negate light_dir because it represents "direction rays travel"
    let L = -light_dir;
    let H = normalize(L + V);

    let NdotH = max(dot(N, H), 0.0);
    let NdotL = max(dot(N, L), 0.0);

    // Gotanda normalization for energy conservation
    let norm = gotanda_normalization(shininess);
    let spec = norm * pow(NdotH, shininess);

    return specular_color * spec * light_color * NdotL;
}

// Rim lighting (edge highlights)
// Uses provided color for coherent scene lighting
fn rim_lighting(
    N: vec3<f32>,
    V: vec3<f32>,
    rim_color: vec3<f32>,
    rim_intensity: f32,
    rim_power: f32,
) -> vec3<f32> {
    let NdotV = max(dot(N, V), 0.0);
    let rim_factor = pow(1.0 - NdotV, rim_power);
    return rim_color * rim_factor * rim_intensity;
}

// ============================================================================
// Fragment Shader
// ============================================================================

@fragment
fn fs(in: VertexOut) -> @location(0) vec4<f32> {
    let shading = shading_states[in.shading_state_index];
    let material_color = unpack_rgba8(shading.color_rgba8);
    let sky = unpack_sky(shading.sky);

    // Unpack uniforms from packed field
    let packed_values = shading.metallic_roughness_emissive_pad;
    let uniform0 = unpack_unorm8_from_u32(packed_values & 0xFFu);
    let uniform1 = unpack_unorm8_from_u32((packed_values >> 8u) & 0xFFu);
    var emissive = unpack_unorm8_from_u32((packed_values >> 16u) & 0xFFu);

    var albedo = material_color.rgb;
    //FS_COLOR
    //FS_UV

    // Rim intensity always comes from uniform (never overridden by texture)
    let rim_intensity = uniform0;

    var value0 = uniform0;  // metallic (mode 2) or specular_intensity (mode 3), may be overridden by texture
    var value1 = uniform1;  // roughness (mode 2) or shininess (mode 3)

    //FS_MODE2_3_TEXTURES  // Texture overrides for mode-specific slots

    // MODE-SPECIFIC: Shininess computation (injected by shader_gen.rs)
    //FS_MODE2_3_SHININESS

    // MODE-SPECIFIC: Specular color computation (injected by shader_gen.rs)
    //FS_MODE2_3_SPECULAR_COLOR

    let view_dir = normalize(in.camera_position - in.world_position);

    // Unpack rim power from matcap_blend_modes byte 3
    let rim_power_raw = unpack_unorm8_from_u32(shading.matcap_blend_modes >> 24u);
    let rim_power = rim_power_raw * 32.0;

    // Diffuse factor: Mode 2 reduces diffuse for metallic surfaces (metals don't have diffuse)
    // Mode 3 has no metallic, so diffuse_factor = 1.0
    //FS_MODE2_3_DIFFUSE_FACTOR

    let glow = albedo * emissive;
    var final_color = glow;

    // Indirect ambient: sample sky in direction of surface normal (IBL-lite, no cosine term)
    // Gotanda normalization reduces ambient as shininess increases (energy conservation)
    let spec_norm = gotanda_normalization(shininess);
    let ambient_factor = 1.0 / sqrt(1.0 + spec_norm);
    let ambient_color = sample_sky(in.world_normal, sky);
    final_color += ambient_color * albedo * ambient_factor;

    // Direct sun: sample sky in the direction of the sun (all colors from sky)
    let sun_color = sample_sky(-sky.sun_direction, sky);

    // Sun diffuse (direct, with metallic reduction)
    final_color += lambert_diffuse(in.world_normal, sky.sun_direction, albedo, sun_color) * diffuse_factor;

    // Sun specular
    final_color += normalized_blinn_phong_specular(
        in.world_normal, view_dir, sky.sun_direction, shininess, specular_color, sun_color
    );

    // 4 dynamic lights (direct illumination)
    for (var i = 0u; i < 4u; i++) {
        let light = unpack_light(shading.lights[i]);
        if (light.enabled) {
            let light_color = light.color * light.intensity;
            final_color += lambert_diffuse(in.world_normal, light.direction, albedo, light_color) * diffuse_factor;
            final_color += normalized_blinn_phong_specular(
                in.world_normal, view_dir, light.direction, shininess, specular_color, light_color
            );
        }
    }

    // Rim lighting (always uses rim_intensity from uniform, never from texture)
    let rim = rim_lighting(in.world_normal, view_dir, sun_color, rim_intensity, rim_power);
    final_color += rim;

    return vec4<f32>(final_color, material_color.a);
}
