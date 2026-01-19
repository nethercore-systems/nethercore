// ============================================================================
// EPU COMPUTE: BLUR PYRAMID GENERATION
// Generates synthetic mipmaps by blurring EnvLight0 -> EnvLight1 -> EnvLight2...
// Uses Kawase blur (5-tap) for energy-conserving blur.
// ============================================================================

struct BlurUniforms {
    active_count: u32,
    map_size: u32,
    blur_offset: f32,
    _pad0: u32,
}

@group(0) @binding(2) var<storage, read> epu_active_env_ids: array<u32>;
@group(0) @binding(4) var epu_in: texture_2d_array<f32>;
@group(0) @binding(5) var epu_out: texture_storage_2d_array<rgba16float, write>;
@group(0) @binding(6) var epu_samp: sampler;
@group(0) @binding(7) var<uniform> epu_blur: BlurUniforms;

@compute @workgroup_size(8, 8, 1)
fn epu_kawase_blur(@builtin(global_invocation_id) gid: vec3u) {
    let env_slot = gid.z;
    if env_slot >= epu_blur.active_count { return; }

    let env_id = epu_active_env_ids[env_slot];
    let map_size = epu_blur.map_size;
    if gid.x >= map_size || gid.y >= map_size { return; }

    let resolution = vec2f(f32(map_size));
    let uv = (vec2f(gid.xy) + 0.5) / resolution;
    let texel = 1.0 / resolution;
    let o = epu_blur.blur_offset * texel;

    var c = textureSampleLevel(epu_in, epu_samp, uv, i32(env_id), 0.0).rgb;
    c += textureSampleLevel(epu_in, epu_samp, uv + vec2f(-o.x, -o.y), i32(env_id), 0.0).rgb;
    c += textureSampleLevel(epu_in, epu_samp, uv + vec2f( o.x, -o.y), i32(env_id), 0.0).rgb;
    c += textureSampleLevel(epu_in, epu_samp, uv + vec2f(-o.x,  o.y), i32(env_id), 0.0).rgb;
    c += textureSampleLevel(epu_in, epu_samp, uv + vec2f( o.x,  o.y), i32(env_id), 0.0).rgb;
    c /= 5.0;

    textureStore(epu_out, vec2u(gid.xy), i32(env_id), vec4f(c, 1.0));
}
