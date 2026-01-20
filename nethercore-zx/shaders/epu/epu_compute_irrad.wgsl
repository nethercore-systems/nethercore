// ============================================================================
// EPU COMPUTE: DIFFUSE IRRADIANCE EXTRACTION (SH9)
// Extracts L2 spherical harmonics coefficients from a coarse radiance mip.
// These coefficients are evaluated per-pixel for smooth
// diffuse ambient lighting.
// ============================================================================

const PI: f32 = 3.141592653589793;
const TAU: f32 = 6.283185307179586;
const GOLDEN_RATIO_CONJ: f32 = 0.6180339887498949;
const SH_SAMPLES: u32 = 64u;

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

struct IrradUniforms {
    active_count: u32,
    _pad0: u32,
    _pad1: u32,
    _pad2: u32,
}

@group(0) @binding(2) var<storage, read> epu_active_env_ids: array<u32>;
@group(0) @binding(4) var epu_blurred: texture_2d_array<f32>;
@group(0) @binding(5) var epu_samp: sampler;
@group(0) @binding(6) var<storage, read_write> epu_sh9: array<EpuSh9>;
@group(0) @binding(7) var<uniform> epu_irrad: IrradUniforms;

// Octahedral encode for sampling - duplicated here for standalone compute shader
// WGSL `sign()` returns 0 for 0 inputs, which breaks octahedral fold math on the
// axes (producing visible "plus" seams). Use a non-zero sign instead.
fn sign_not_zero(v: vec2f) -> vec2f {
    return vec2f(select(-1.0, 1.0, v.x >= 0.0), select(-1.0, 1.0, v.y >= 0.0));
}

fn oct_encode_local(dir: vec3f) -> vec2f {
    let n = dir / (abs(dir.x) + abs(dir.y) + abs(dir.z));
    if n.z < 0.0 {
        return (1.0 - abs(n.yx)) * sign_not_zero(n.xy);
    }
    return n.xy;
}

// Uniform sphere sampling via spherical Fibonacci points (deterministic).
fn fibonacci_dir(i: u32, n: u32) -> vec3f {
    let k = (f32(i) + 0.5) / f32(n);
    let z = 1.0 - 2.0 * k;
    let r = sqrt(max(0.0, 1.0 - z * z));
    let phi = TAU * fract((f32(i) + 0.5) * GOLDEN_RATIO_CONJ);
    return vec3f(cos(phi) * r, sin(phi) * r, z);
}

@compute @workgroup_size(1, 1, 1)
fn epu_extract_sh9(@builtin(global_invocation_id) gid: vec3u) {
    let env_slot = gid.z;
    if env_slot >= epu_irrad.active_count { return; }

    let env_id = epu_active_env_ids[env_slot];

    // Accumulate radiance SH coefficients (real SH, L2).
    var c0 = vec3f(0.0);
    var c1 = vec3f(0.0);
    var c2 = vec3f(0.0);
    var c3 = vec3f(0.0);
    var c4 = vec3f(0.0);
    var c5 = vec3f(0.0);
    var c6 = vec3f(0.0);
    var c7 = vec3f(0.0);
    var c8 = vec3f(0.0);

    for (var i = 0u; i < SH_SAMPLES; i++) {
        let dir = fibonacci_dir(i, SH_SAMPLES);
        let uv = oct_encode_local(dir) * 0.5 + 0.5;
        let l = textureSampleLevel(epu_blurred, epu_samp, uv, i32(env_id), 0.0).rgb;

        let x = dir.x;
        let y = dir.y;
        let z = dir.z;

        let sh0 = 0.282095;
        let sh1 = 0.488603 * y;
        let sh2 = 0.488603 * z;
        let sh3 = 0.488603 * x;
        let sh4 = 1.092548 * x * y;
        let sh5 = 1.092548 * y * z;
        let sh6 = 0.315392 * (3.0 * z * z - 1.0);
        let sh7 = 1.092548 * x * z;
        let sh8 = 0.546274 * (x * x - y * y);

        c0 += l * sh0;
        c1 += l * sh1;
        c2 += l * sh2;
        c3 += l * sh3;
        c4 += l * sh4;
        c5 += l * sh5;
        c6 += l * sh6;
        c7 += l * sh7;
        c8 += l * sh8;
    }

    // Convert sum to integral over sphere.
    let w = (4.0 * PI) / f32(SH_SAMPLES);

    // Lambertian convolution kernel (irradiance) per band.
    let a0 = PI;
    let a1 = (2.0 * PI) / 3.0;
    let a2 = PI / 4.0;

    c0 *= w * a0;
    c1 *= w * a1;
    c2 *= w * a1;
    c3 *= w * a1;
    c4 *= w * a2;
    c5 *= w * a2;
    c6 *= w * a2;
    c7 *= w * a2;
    c8 *= w * a2;

    var out: EpuSh9;
    out.c0 = c0;
    out.c1 = c1;
    out.c2 = c2;
    out.c3 = c3;
    out.c4 = c4;
    out.c5 = c5;
    out.c6 = c6;
    out.c7 = c7;
    out.c8 = c8;

    epu_sh9[env_id] = out;
}
