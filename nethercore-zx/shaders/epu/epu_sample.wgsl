// ============================================================================
// EPU SAMPLING FUNCTIONS
// Used by render pipelines to sample background, reflection, and ambient.
// ============================================================================

// AmbientCube structure for ambient lighting lookup
struct EpuAmbientCube {
    pos_x: vec3f, _pad0: f32,
    neg_x: vec3f, _pad1: f32,
    pos_y: vec3f, _pad2: f32,
    neg_y: vec3f, _pad3: f32,
    pos_z: vec3f, _pad4: f32,
    neg_z: vec3f, _pad5: f32,
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
// BACKGROUND SAMPLING (EnvSharp)
// ============================================================================

fn sample_background(
    env_sharp: texture_2d_array<f32>,
    env_samp: sampler,
    env_id: u32,
    view_dir: vec3f
) -> vec3f {
    let uv = epu_oct_encode(view_dir) * 0.5 + 0.5;
    return textureSampleLevel(env_sharp, env_samp, uv, i32(env_id), 0.0).rgb;
}

// ============================================================================
// REFLECTION SAMPLING (Roughness -> Blur Level)
// Contract: avoid "double images" by sampling *either* EnvSharp *or* the
// blurred EnvLight* chain.
// ============================================================================

fn sample_reflection(
    env_sharp: texture_2d_array<f32>,
    env_light0: texture_2d_array<f32>,
    env_light1: texture_2d_array<f32>,
    env_light2: texture_2d_array<f32>,
    env_samp: sampler,
    env_id: u32,
    refl_dir: vec3f,
    roughness: f32
) -> vec3f {
    let uv = epu_oct_encode(refl_dir) * 0.5 + 0.5;

    // Threshold chosen to avoid ghosting between sharp and blurred representations.
    let sharp_cut = 0.15;
    if roughness <= sharp_cut {
        return textureSampleLevel(env_sharp, env_samp, uv, i32(env_id), 0.0).rgb;
    }

    // Remap [sharp_cut..1] -> [0..1] for blurred selection.
    let r = epu_saturate((roughness - sharp_cut) / (1.0 - sharp_cut));

    // 3-level example: Light0, Light1, Light2
    let t = r * 2.0;
    if t <= 1.0 {
        let a = t;
        let c0 = textureSampleLevel(env_light0, env_samp, uv, i32(env_id), 0.0).rgb;
        let c1 = textureSampleLevel(env_light1, env_samp, uv, i32(env_id), 0.0).rgb;
        return mix(c0, c1, a);
    } else {
        let a = t - 1.0;
        let c1 = textureSampleLevel(env_light1, env_samp, uv, i32(env_id), 0.0).rgb;
        let c2 = textureSampleLevel(env_light2, env_samp, uv, i32(env_id), 0.0).rgb;
        return mix(c1, c2, a);
    }
}

// ============================================================================
// AMBIENT LIGHTING (6-direction cube lookup)
// ============================================================================

fn sample_ambient(
    ambient_cubes: ptr<storage, array<EpuAmbientCube>, read>,
    env_id: u32,
    n: vec3f
) -> vec3f {
    let c = (*ambient_cubes)[env_id];

    let pos = vec3f(max(n.x, 0.0), max(n.y, 0.0), max(n.z, 0.0));
    let neg = vec3f(max(-n.x, 0.0), max(-n.y, 0.0), max(-n.z, 0.0));

    var a = vec3f(0.0);
    a += c.pos_x * pos.x;
    a += c.neg_x * neg.x;
    a += c.pos_y * pos.y;
    a += c.neg_y * neg.y;
    a += c.pos_z * pos.z;
    a += c.neg_z * neg.z;

    return a;
}
