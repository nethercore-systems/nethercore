// ============================================================================
// EPU SAMPLING FUNCTIONS
// Used by render pipelines to sample background, reflection, and ambient.
// ============================================================================

// SH9 structure for diffuse irradiance lookup
struct EpuSh9 {
    c0: vec3f, _pad0: f32,
    c1: vec3f, _pad1: f32,
    c2: vec3f, _pad2: f32,
    c3: vec3f, _pad3: f32,
    c4: vec3f, _pad4: f32,
    c5: vec3f, _pad5: f32,
    c6: vec3f, _pad6: f32,
    c7: vec3f, _pad7: f32,
    c8: vec3f, _pad8: f32,
}

// WGSL `sign()` returns 0 for 0 inputs, which breaks octahedral fold math on the
// axes (producing visible "plus" seams). Use a non-zero sign instead.
fn sign_not_zero(v: vec2f) -> vec2f {
    return vec2f(select(-1.0, 1.0, v.x >= 0.0), select(-1.0, 1.0, v.y >= 0.0));
}

// Octahedral encode for sampling
fn epu_oct_encode(dir: vec3f) -> vec2f {
    let n = dir / (abs(dir.x) + abs(dir.y) + abs(dir.z));
    if n.z < 0.0 {
        return (1.0 - abs(n.yx)) * sign_not_zero(n.xy);
    }
    return n.xy;
}

fn epu_saturate(x: f32) -> f32 { return clamp(x, 0.0, 1.0); }

// ============================================================================
// BACKGROUND SAMPLING (EnvRadiance mip 0)
// ============================================================================

fn sample_background(
    env_radiance: texture_2d_array<f32>,
    env_samp: sampler,
    env_id: u32,
    view_dir: vec3f
) -> vec3f {
    let uv = epu_oct_encode(view_dir) * 0.5 + 0.5;
    return textureSampleLevel(env_radiance, env_samp, uv, i32(env_id), 0.0).rgb;
}

// ============================================================================
// REFLECTION SAMPLING (Continuous Roughness -> LOD)
// ============================================================================

fn sample_reflection(
    env_radiance: texture_2d_array<f32>,
    env_samp: sampler,
    env_id: u32,
    refl_dir: vec3f,
    roughness: f32
) -> vec3f {
    let uv = epu_oct_encode(refl_dir) * 0.5 + 0.5;

    // Use roughness^2 for a more perceptually linear blur ramp.
    let r = epu_saturate(roughness);
    let max_lod = max(0.0, f32(textureNumLevels(env_radiance) - 1));
    let lod = (r * r) * max_lod;

    // Manual mip lerp (keeps results smooth even if sampler mipmap_filter is Nearest).
    let lod0 = floor(lod);
    let lod1 = min(lod0 + 1.0, max_lod);
    let t = lod - lod0;

    let c0 = textureSampleLevel(env_radiance, env_samp, uv, i32(env_id), lod0).rgb;
    let c1 = textureSampleLevel(env_radiance, env_samp, uv, i32(env_id), lod1).rgb;
    return mix(c0, c1, t);
}

// ============================================================================
// AMBIENT LIGHTING (SH9 diffuse irradiance)
// ============================================================================

fn sample_ambient(
    sh9: ptr<storage, array<EpuSh9>, read>,
    env_id: u32,
    n: vec3f
) -> vec3f {
    let c = (*sh9)[env_id];

    let nn = normalize(n);
    let x = nn.x;
    let y = nn.y;
    let z = nn.z;

    let sh0 = 0.282095;
    let sh1 = 0.488603 * y;
    let sh2 = 0.488603 * z;
    let sh3 = 0.488603 * x;
    let sh4 = 1.092548 * x * y;
    let sh5 = 1.092548 * y * z;
    let sh6 = 0.315392 * (3.0 * z * z - 1.0);
    let sh7 = 1.092548 * x * z;
    let sh8 = 0.546274 * (x * x - y * y);

    let e = c.c0 * sh0
        + c.c1 * sh1
        + c.c2 * sh2
        + c.c3 * sh3
        + c.c4 * sh4
        + c.c5 * sh5
        + c.c6 * sh6
        + c.c7 * sh7
        + c.c8 * sh8;

    return max(e, vec3f(0.0));
}
