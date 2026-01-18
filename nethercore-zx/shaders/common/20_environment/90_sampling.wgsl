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
