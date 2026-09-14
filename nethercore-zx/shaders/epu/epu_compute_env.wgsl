// ============================================================================
// EPU COMPUTE: ENVIRONMENT EVALUATION
// Builds EnvRadiance (mip 0) for all active environments.
// EPU: 128-bit instructions with embedded RGB24 colors (no palette)
// ============================================================================

// PackedEnvironmentState: 8 layers stored as vec4u (128-bit per instruction)
// Each layer is 128 bits: [w0, w1, w2, w3] where w3 contains opcode/region/blend
struct PackedEnvironmentState {
    layers: array<vec4u, 8>,
}

struct FrameUniforms {
    active_count: u32,
    map_size: u32,
    _pad0: u32,
    _pad1: u32,
}

// Bindings: No palette buffer - colors are embedded in instructions
@group(0) @binding(0) var<storage, read> epu_states: array<PackedEnvironmentState>;
@group(0) @binding(1) var<storage, read> epu_active_env_ids: array<u32>;
@group(0) @binding(2) var<uniform> epu_frame: FrameUniforms;

@group(0) @binding(3) var epu_out_sharp: texture_storage_2d_array<rgba16float, write>;

fn evaluate_env_radiance(dir: vec3f, st: PackedEnvironmentState) -> vec3f {
    return evaluate_epu_layers(dir, st.layers);
}

// Linear source filtering boundary, before runtime storage saturation.
fn evaluate_env_texel(pixel: vec2u, map_size: u32, st: PackedEnvironmentState) -> vec3f {
    // Integrate inside the angular cell, rather than sampling its center or
    // blending selected fold neighbors. The oct solid-angle Jacobian is
    // 1/length(oct_surface)^3 = (abs(dir.x)+abs(dir.y)+abs(dir.z))^3.
    var radiance = vec3f(0.0);
    var weight_sum = 0.0;
    // ponytail: four evaluations per source texel; cost acceptance is deferred.
    for (var y = 0u; y < 2u; y++) {
        for (var x = 0u; x < 2u; x++) {
            let offset = (vec2f(f32(x), f32(y)) + 0.5) / 2.0;
            let oct = 2.0 * (vec2f(pixel) + offset) / f32(map_size) - 1.0;
            let dir = octahedral_decode(oct);
            let l1 = dot(abs(dir), vec3f(1.0));
            let weight = l1 * l1 * l1;
            radiance += evaluate_env_radiance(dir, st) * weight;
            weight_sum += weight;
        }
    }
    return radiance / weight_sum;
}

@compute @workgroup_size(8, 8, 1)
fn epu_build(@builtin(global_invocation_id) gid: vec3u) {
    if gid.z >= epu_frame.active_count { return; }
    let env_id = epu_active_env_ids[gid.z];
    let map_size = epu_frame.map_size;
    if gid.x >= map_size || gid.y >= map_size { return; }
    let final_radiance = evaluate_env_texel(gid.xy, map_size, epu_states[env_id]);
    textureStore(
        epu_out_sharp,
        vec2u(gid.xy),
        i32(env_id),
        vec4f(epu_saturate3(final_radiance), 1.0)
    );
}
