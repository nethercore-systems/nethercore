// ============================================================================
// EPU COMPUTE: MIP PYRAMID GENERATION (Downsample)
//
// Builds a true mip-style downsample pyramid from a single octahedral radiance
// map (mip 0). Each pass downsamples mip i -> mip i+1 with a 2x2 box filter.
//
// This replaces full-resolution direction-space Kawase blur. The pyramid is
// used for roughness-based reflections (LOD from roughness) and diffuse SH9
// extraction from a coarse level.
// ============================================================================

@group(0) @binding(2) var<storage, read> epu_active_env_ids: array<u32>;
@group(0) @binding(4) var epu_in: texture_2d_array<f32>;
@group(0) @binding(5) var epu_out: texture_storage_2d_array<rgba16float, write>;

@compute @workgroup_size(8, 8, 1)
fn epu_downsample_mip(@builtin(global_invocation_id) gid: vec3u) {
    let env_id = epu_active_env_ids[gid.z];

    let dst_dim = textureDimensions(epu_out);
    if gid.x >= dst_dim.x || gid.y >= dst_dim.y {
        return;
    }

    let src_dim = textureDimensions(epu_in);
    let src_xy = vec2u(gid.xy) * 2u;
    let base = vec2<i32>(i32(src_xy.x), i32(src_xy.y));
    let layer = i32(env_id);

    var c = vec3f(0.0);
    c += textureLoad(epu_in, base + vec2i(0, 0), layer, 0).rgb;
    c += textureLoad(epu_in, base + vec2i(1, 0), layer, 0).rgb;
    c += textureLoad(epu_in, base + vec2i(0, 1), layer, 0).rgb;
    c += textureLoad(epu_in, base + vec2i(1, 1), layer, 0).rgb;
    c *= 0.25;

    textureStore(epu_out, vec2u(gid.xy), layer, vec4f(c, 1.0));
}
