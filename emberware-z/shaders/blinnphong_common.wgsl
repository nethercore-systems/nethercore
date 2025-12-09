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

// Spherical Gaussian approximation for Fresnel (faster than pow)
fn fresnel_schlick_sg(cos_theta: f32, F0: vec3<f32>) -> vec3<f32> {
    return F0 + (1.0 - F0) * exp2((-5.55473 * cos_theta - 6.98316) * cos_theta);
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

    // Unpack uniforms from uniform_set_0
    // Mode 2: [metallic, roughness, emissive, rim_intensity]
    // Mode 3: [spec_damping*, shininess, emissive, rim_intensity]
    // *spec_damping is inverted: 0=full specular (default), 255=no specular
    let uniform0 = unpack_unorm8_from_u32(shading.uniform_set_0 & 0xFFu); // byte 0
    let uniform1 = unpack_unorm8_from_u32((shading.uniform_set_0 >> 8u) & 0xFFu); // byte 1
    var emissive = unpack_unorm8_from_u32((shading.uniform_set_0 >> 16u) & 0xFFu); // byte 2

    var albedo = material_color.rgb;
    //FS_COLOR
    //FS_UV

    // Rim intensity from byte 3 of uniform_set_0 (separate from metallic/spec_damping)
    let rim_intensity = unpack_unorm8_from_u32((shading.uniform_set_0 >> 24u) & 0xFFu);

    var value0 = uniform0;  // metallic (mode 2) or spec_damping (mode 3), may be overridden by texture
    var value1 = uniform1;  // roughness (mode 2) or shininess (mode 3)

    //FS_MODE2_3_TEXTURES  // Texture overrides for mode-specific slots

    // MODE-SPECIFIC: Shininess computation (injected by shader_gen.rs)
    //FS_MODE2_3_SHININESS

    // MODE-SPECIFIC: Specular color computation (injected by shader_gen.rs)
    //FS_MODE2_3_SPECULAR_COLOR

    let view_dir = normalize(in.camera_position - in.world_position);

    // Rim power from byte 0 of uniform_set_1 (low byte)
    let rim_power_raw = unpack_unorm8_from_u32(shading.uniform_set_1 & 0xFFu);
    let rim_power = rim_power_raw * 32.0;

    let glow = albedo * emissive;
    var final_color = glow;

    // Diffuse factor: Mode 2 reduces diffuse for metallic surfaces (metals don't have diffuse)
    // Mode 3 has no metallic, so diffuse_factor = 1.0
    //FS_MODE2_3_DIFFUSE_FACTOR

    // === Environment Lighting (IBL-lite) ===
    // Re-normalize after interpolation - interpolated normals become shorter than unit length
    let N = normalize(in.world_normal);
    let NdotV = max(dot(N, view_dir), 0.0);

    // Fresnel: F0 = specular_color (works for both MR and SS workflows)
    let fresnel = fresnel_schlick_sg(NdotV, specular_color);
    let one_minus_F = vec3<f32>(1.0) - fresnel;

    // MODE-SPECIFIC: Diffuse Fresnel multiplier
    // Mode 2: diffuse_fresnel = one_minus_F (energy conservation)
    // Mode 3: diffuse_fresnel = vec3(1.0) (no diffuse reduction, artistic freedom)
    //FS_MODE2_3_DIFFUSE_FRESNEL

    // MODE-SPECIFIC: Roughness (injected by shader_gen.rs)
    // Mode 2: direct from value1, Mode 3: derived from shininess
    //FS_MODE2_3_ROUGHNESS

    // Diffuse ambient (normal direction) - use gradient only, sun handled by direct lighting
    let diffuse_env = sample_sky_ambient(N, sky);

    // Specular reflection (reflection direction) - use gradient only, sun specular via blinn-phong
    let R = reflect(-view_dir, N);
    let specular_env = sample_sky_ambient(R, sky);

    // Rough surfaces have dimmer reflections (energy scatters)
    let reflection_strength = (1.0 - roughness) * (1.0 - roughness);  // squared falloff

    // Energy conservation factor
    let spec_norm = gotanda_normalization(shininess);
    let ambient_factor = 1.0 / sqrt(1.0 + spec_norm);

    // Diffuse ambient
    final_color += diffuse_env * albedo * ambient_factor * diffuse_factor * diffuse_fresnel;

    // Specular environment reflection (attenuated by roughness)
    final_color += specular_env * fresnel * reflection_strength;

    // Direct sun - use stored sun color, not sample_sky (which adds gradient on top)
    let sun_color = sky.sun_color;

    // Sun diffuse
    final_color += lambert_diffuse(N, sky.sun_direction, albedo, sun_color) * diffuse_factor * diffuse_fresnel;

    // Sun specular
    final_color += normalized_blinn_phong_specular(
        N, view_dir, sky.sun_direction, shininess, specular_color, sun_color
    );

    // 4 dynamic lights (direct illumination)
    for (var i = 0u; i < 4u; i++) {
        let light = unpack_light(shading.lights[i]);
        if (light.enabled) {
            let light_color = light.color * light.intensity;
            final_color += lambert_diffuse(N, light.direction, albedo, light_color) * diffuse_factor * diffuse_fresnel;
            final_color += normalized_blinn_phong_specular(
                N, view_dir, light.direction, shininess, specular_color, light_color
            );
        }
    }

    // Rim lighting (always uses rim_intensity from uniform, never from texture)
    // Specular-tinted rim, modulated by sun color for scene coherence
    // - Material character from specular_color (gold stays gold, holo stays magenta)
    // - Scene coherence from sun_color (red lights = red tint on everything)
    let rim_color = specular_color * sun_color;
    let rim = rim_lighting(N, view_dir, rim_color, rim_intensity, rim_power);
    final_color += rim;

    return vec4<f32>(final_color, material_color.a);
}
