// Sample a single environment mode
fn sample_mode(mode: u32, data: array<u32, 14>, offset: u32, direction: vec3<f32>) -> vec4<f32> {
    switch (mode) {
        case 0u: { return sample_gradient(data, offset, direction); }
        case 1u: { return sample_cells(data, offset, direction); }
        case 2u: { return sample_lines(data, offset, direction); }
        case 3u: { return sample_silhouette(data, offset, direction); }
        case 4u: { return sample_nebula(data, offset, direction); }
        case 5u: { return sample_room(data, offset, direction); }
        case 6u: { return sample_veil(data, offset, direction); }
        case 7u: { return sample_rings(data, offset, direction); }
        default: { return sample_gradient(data, offset, direction); }
    }
}

// Blend two layers together
fn blend_layers(base: vec4<f32>, overlay: vec4<f32>, mode: u32) -> vec4<f32> {
    switch (mode) {
        case 0u: { return mix(base, overlay, overlay.a); }  // Alpha blend
        case 1u: { return base + overlay; }                  // Add
        case 2u: { return base * overlay; }                  // Multiply
        case 3u: {
            // Screen: 1 - (1-base) * (1-overlay)
            return vec4<f32>(1.0) - (vec4<f32>(1.0) - base) * (vec4<f32>(1.0) - overlay);
        }
        default: { return base; }
    }
}

// Sample complete environment (base + overlay with blend)
fn sample_environment(env_index: u32, direction: vec3<f32>) -> vec4<f32> {
    let env = environment_states[env_index];
    let base_mode = env.header & 0x7u;
    let overlay_mode = (env.header >> 3u) & 0x7u;
    let blend_mode = (env.header >> 6u) & 0x3u;

    let base_color = sample_mode(base_mode, env.data, 0u, direction);

    let overlay_color = sample_mode(overlay_mode, env.data, 7u, direction);
    return blend_layers(base_color, overlay_color, blend_mode);
}

// Sample environment ambient (used for material lighting)
fn sample_environment_ambient(env_index: u32, direction: vec3<f32>) -> vec3<f32> {
    let env_color = sample_environment(env_index, direction);
    return env_color.rgb;
}

// ============================================================================
// EPU BACKGROUND SAMPLING
// ============================================================================
// Sample from precomputed EPU EnvSharp octahedral texture array.
// Used for background rendering (env_template.wgsl).

// Octahedral encode for EPU texture sampling (direction -> UV)
// WGSL `sign()` returns 0 for 0 inputs, which breaks octahedral fold math on the
// axes (producing visible "plus" seams). Use a non-zero sign instead.
fn sign_not_zero(v: vec2<f32>) -> vec2<f32> {
    return vec2<f32>(
        select(-1.0, 1.0, v.x >= 0.0),
        select(-1.0, 1.0, v.y >= 0.0)
    );
}

fn epu_octahedral_encode(dir: vec3<f32>) -> vec2<f32> {
    let n = dir / (abs(dir.x) + abs(dir.y) + abs(dir.z));
    if n.z < 0.0 {
        return (1.0 - abs(n.yx)) * sign_not_zero(n.xy);
    }
    return n.xy;
}

// Sample background from EPU EnvSharp texture
fn sample_epu_background(env_index: u32, direction: vec3<f32>) -> vec4<f32> {
    // Octahedral encode direction to UV coordinates [0, 1]
    let oct = epu_octahedral_encode(normalize(direction));
    let uv = oct * 0.5 + 0.5;

    // Sample from EPU EnvSharp texture array
    return textureSampleLevel(epu_env_sharp, epu_sampler, uv, i32(env_index), 0.0);
}

// ============================================================================
// EPU REFLECTION SAMPLING (Sharp-vs-Blur Contract)
// ============================================================================
// Sample from EPU blur pyramid for roughness-based reflections.
// Uses sharp-vs-blur contract:
// - roughness <= 0.15: use EnvSharp (full detail)
// - roughness > 0.15: interpolate through Light0 -> Light1 -> Light2

fn sample_epu_reflection(env_id: u32, refl_dir: vec3f, roughness: f32) -> vec3f {
    let uv = epu_octahedral_encode(normalize(refl_dir)) * 0.5 + 0.5;
    let sharp_cut = 0.15;

    // For very smooth surfaces, use the sharp environment map
    if roughness <= sharp_cut {
        return textureSampleLevel(epu_env_sharp, epu_sampler, uv, i32(env_id), 0.0).rgb;
    }

    // Remap roughness above sharp_cut to [0, 1] range
    let r = saturate((roughness - sharp_cut) / (1.0 - sharp_cut));

    // Map to blur pyramid levels: t in [0, 2] spans Light0 -> Light1 -> Light2
    let t = r * 2.0;

    if t <= 1.0 {
        // Interpolate between Light0 and Light1
        let c0 = textureSampleLevel(epu_env_light0, epu_sampler, uv, i32(env_id), 0.0).rgb;
        let c1 = textureSampleLevel(epu_env_light1, epu_sampler, uv, i32(env_id), 0.0).rgb;
        return mix(c0, c1, t);
    } else {
        // Interpolate between Light1 and Light2
        let c1 = textureSampleLevel(epu_env_light1, epu_sampler, uv, i32(env_id), 0.0).rgb;
        let c2 = textureSampleLevel(epu_env_light2, epu_sampler, uv, i32(env_id), 0.0).rgb;
        return mix(c1, c2, t - 1.0);
    }
}

// ============================================================================
// EPU AMBIENT CUBE SAMPLING (6-Direction Diffuse Irradiance)
// ============================================================================
// Sample from pre-computed ambient cubes for efficient diffuse lighting.
// Much faster than texture sampling for diffuse irradiance.

fn sample_epu_ambient(env_id: u32, n: vec3f) -> vec3f {
    let c = epu_ambient_cubes[env_id];
    // Separate positive and negative direction weights
    let pos = vec3f(max(n.x, 0.0), max(n.y, 0.0), max(n.z, 0.0));
    let neg = vec3f(max(-n.x, 0.0), max(-n.y, 0.0), max(-n.z, 0.0));
    // Weighted sum of all 6 directions based on normal orientation
    return c.pos_x * pos.x + c.neg_x * neg.x
         + c.pos_y * pos.y + c.neg_y * neg.y
         + c.pos_z * pos.z + c.neg_z * neg.z;
}
