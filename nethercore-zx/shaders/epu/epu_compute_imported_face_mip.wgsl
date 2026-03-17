// ============================================================================
// EPU COMPUTE: IMPORTED FACE ARRAY MIP GENERATION
//
// Builds ordinary per-face mip levels for the imported direct face cache.
// Unlike EnvRadiance, these layers are plain cubemap faces, so no octahedral
// fold weighting is applied here.
// ============================================================================

@group(0) @binding(2) var<storage, read> epu_active_face_layers: array<u32>;
@group(0) @binding(4) var epu_in: texture_2d_array<f32>;
@group(0) @binding(5) var epu_out: texture_storage_2d_array<rgba16float, write>;

@compute @workgroup_size(8, 8, 1)
fn epu_downsample_imported_face_mip(@builtin(global_invocation_id) gid: vec3u) {
    let face_layer = epu_active_face_layers[gid.z];

    let dst_dim = textureDimensions(epu_out);
    if gid.x >= dst_dim.x || gid.y >= dst_dim.y {
        return;
    }

    let src_xy = vec2u(gid.xy) * 2u;
    let base = vec2i(i32(src_xy.x), i32(src_xy.y));
    let layer = i32(face_layer);

    let c00 = textureLoad(epu_in, base + vec2i(0, 0), layer, 0).rgb;
    let c10 = textureLoad(epu_in, base + vec2i(1, 0), layer, 0).rgb;
    let c01 = textureLoad(epu_in, base + vec2i(0, 1), layer, 0).rgb;
    let c11 = textureLoad(epu_in, base + vec2i(1, 1), layer, 0).rgb;
    let color = (c00 + c10 + c01 + c11) * 0.25;

    textureStore(epu_out, vec2u(gid.xy), layer, vec4f(color, 1.0));
}
