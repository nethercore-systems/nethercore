// ============================================================================
// EPU COMPUTE: DIFFUSE IRRADIANCE EXTRACTION (SH9)
// Extracts L2 spherical harmonics coefficients from source radiance (mip 0).
// These coefficients are evaluated per-pixel for smooth
// diffuse ambient lighting.
// ============================================================================

const PI: f32 = 3.141592653589793;

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
@group(0) @binding(4) var epu_radiance: texture_2d_array<f32>;
@group(0) @binding(5) var epu_samp: sampler;
@group(0) @binding(6) var<storage, read_write> epu_sh9: array<EpuSh9>;
@group(0) @binding(7) var<uniform> epu_irrad: IrradUniforms;

// Unnormalized octahedral surface point. The solid-angle Jacobian on each
// octahedron face is proportional to 1 / length(point)^3.
fn oct_surface(uv: vec2f) -> vec3f {
    var v = vec3f(uv, 1.0 - abs(uv.x) - abs(uv.y));
    if v.z < 0.0 {
        let s = vec2f(select(-1.0, 1.0, v.x >= 0.0), select(-1.0, 1.0, v.y >= 0.0));
        v = vec3f((1.0 - abs(v.yx)) * s, v.z);
    }
    return v;
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

    // ponytail: integrate every cached texel on a dirty environment, O(map_size^2).
    // A parallel reduction can replace this loop if the deferred cost gate requires it.
    let size = textureDimensions(epu_radiance, 0);
    var total_weight = 0.0;
    for (var i = 0u; i < size.x * size.y; i++) {
        let uv = (vec2f(f32(i % size.x), f32(i / size.x)) + 0.5) / vec2f(size);
        let surface = oct_surface(uv * 2.0 - 1.0);
        let inverse_length = inverseSqrt(dot(surface, surface));
        let dir = surface * inverse_length;
        let weight = inverse_length * inverse_length * inverse_length;
        total_weight += weight;
        let l = textureSampleLevel(epu_radiance, epu_samp, uv, i32(env_id), 0.0).rgb * weight;

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

    // Normalize the midpoint solid-angle quadrature; constant radiance retains its energy.
    let w = (4.0 * PI) / total_weight;

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
