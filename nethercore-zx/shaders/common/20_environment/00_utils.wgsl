// ============================================================================
// Environment Sampling (Multi-Environment v4)
// ============================================================================

// ============================================================================
// Hash Functions (for procedural randomness)
// ============================================================================

// Fast integer hash → float in [0,1] (for discrete randomness)
fn hash21(p: vec2<u32>) -> f32 {
    var n = p.x * 1597u + p.y * 2549u;
    n = n ^ (n >> 13u);
    n = n * 1013904223u;
    return f32(n) * (1.0 / 4294967295.0);
}

fn hash11(p: u32) -> f32 {
    var n = p * 1597u;
    n = n ^ (n >> 13u);
    n = n * 1013904223u;
    return f32(n) * (1.0 / 4294967295.0);
}

// Hash vec3 to float
fn hash31(p: vec3<f32>) -> f32 {
    var p3 = fract(p * 0.1031);
    let yzx = p3.yzx;
    p3 = p3 + dot(p3, yzx + 33.33);
    return fract((p3.x + p3.y) * p3.z);
}

// Cheap triangle wave in [0,1]
fn triwave(x: f32) -> f32 {
    return abs(fract(x) - 0.5) * 2.0;
}

// Triangle wave in [-1, 1] (exactly periodic, trig-free)
fn tri(x: f32) -> f32 {
    return 1.0 - 4.0 * abs(fract(x) - 0.5);
}

fn saturate(x: f32) -> f32 {
    return clamp(x, 0.0, 1.0);
}

fn safe_normalize(v: vec3<f32>, fallback: vec3<f32>) -> vec3<f32> {
    let len2 = dot(v, v);
    if (len2 > 1e-12) {
        return v * inverseSqrt(len2);
    }
    return fallback;
}

fn safe_normalize2(v: vec2<f32>, fallback: vec2<f32>) -> vec2<f32> {
    let len2 = dot(v, v);
    if (len2 > 1e-12) {
        return v * inverseSqrt(len2);
    }
    return fallback;
}

// Signed distance on a wrapping [0,1) domain (returned as absolute distance in [0, 0.5]).
fn wrap_dist01(a: f32, b: f32) -> f32 {
    return abs(fract(a - b + 0.5) - 0.5);
}

// Octahedral mapping: dir → uv in [-1, 1]^2 (no trig).
fn dir_to_oct_uv(dir: vec3<f32>) -> vec2<f32> {
    let n = safe_normalize(dir, vec3<f32>(0.0, 0.0, 1.0));
    var p = n / (abs(n.x) + abs(n.y) + abs(n.z) + 1e-12);
    var uv = p.xz;
    if (p.y < 0.0) {
        let yx = uv.yx;
        uv = (1.0 - abs(yx)) * sign(uv);
    }
    return uv;
}

fn dir_to_oct_uv01(dir: vec3<f32>) -> vec2<f32> {
    return dir_to_oct_uv(dir) * 0.5 + vec2<f32>(0.5);
}

struct OrthoBasis {
    t: vec3<f32>,
    b: vec3<f32>,
    n: vec3<f32>,
}

fn basis_from_axis(axis: vec3<f32>) -> OrthoBasis {
    let n = safe_normalize(axis, vec3<f32>(0.0, 1.0, 0.0));
    let s = select(vec3<f32>(0.0, 1.0, 0.0), vec3<f32>(1.0, 0.0, 0.0), abs(n.y) > 0.95);
    let t = safe_normalize(cross(s, n), vec3<f32>(1.0, 0.0, 0.0));
    let b = cross(n, t);
    return OrthoBasis(t, b, n);
}

// Trig-free pseudo-angle (diamond angle) mapping p→[0,1) with wrap.
// p should be a 2D vector in a plane (need not be normalized).
fn pseudo_angle01(p: vec2<f32>) -> f32 {
    let ax = abs(p.x);
    let ay = abs(p.y);
    let t = ay / (ax + ay + 1e-12); // 0..1, avoids 0/0
    var q = select(t, 2.0 - t, p.x < 0.0);
    q = select(q, 4.0 - q, p.y < 0.0);
    return q * 0.25;
}

fn hash_u32(x: u32) -> u32 {
    var n = x;
    n = n ^ (n >> 16u);
    n = n * 0x7feb352du;
    n = n ^ (n >> 15u);
    n = n * 0x846ca68bu;
    n = n ^ (n >> 16u);
    return n;
}

fn hash01_u32(x: u32) -> f32 {
    return f32(hash_u32(x)) * (1.0 / 4294967295.0);
}

fn hash22_u32(x: u32) -> vec2<f32> {
    let a = hash_u32(x);
    let b = hash_u32(x ^ 0x9e3779b9u);
    return vec2<f32>(f32(a) * (1.0 / 4294967295.0), f32(b) * (1.0 / 4294967295.0));
}

// 2D value noise (bilinear), stable and cheap (uses hash21)
fn value_noise2(uv: vec2<f32>) -> f32 {
    let i = floor(uv);
    let f = fract(uv);
    let u = f * f * (3.0 - 2.0 * f);

    let ix = i32(i.x);
    let iy = i32(i.y);

    let a = hash21(vec2<u32>(u32(ix + 1024) & 0xFFFFu, u32(iy + 1024) & 0xFFFFu));
    let b = hash21(vec2<u32>(u32(ix + 1025) & 0xFFFFu, u32(iy + 1024) & 0xFFFFu));
    let c = hash21(vec2<u32>(u32(ix + 1024) & 0xFFFFu, u32(iy + 1025) & 0xFFFFu));
    let d = hash21(vec2<u32>(u32(ix + 1025) & 0xFFFFu, u32(iy + 1025) & 0xFFFFu));

    return mix(mix(a, b, u.x), mix(c, d, u.x), u.y);
}

