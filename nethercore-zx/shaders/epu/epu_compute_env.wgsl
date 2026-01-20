// ============================================================================
// EPU COMPUTE: ENVIRONMENT EVALUATION
// Builds EnvRadiance (mip 0) for all active environments.
// EPU v2: 128-bit instructions with embedded RGB24 colors (no palette)
// ============================================================================

// PackedEnvironmentState: 8 layers stored as vec4u (128-bit per instruction)
// Each layer is 128 bits: [w0, w1, w2, w3] where w3 contains opcode/region/blend
struct PackedEnvironmentState {
    layers: array<vec4u, 8>,
}

struct FrameUniforms {
    time: f32,
    active_count: u32,
    map_size: u32,
    _pad0: u32,
}

// Bindings (v2): No palette buffer - colors are embedded in instructions
@group(0) @binding(0) var<storage, read> epu_states: array<PackedEnvironmentState>;
@group(0) @binding(1) var<storage, read> epu_active_env_ids: array<u32>;
@group(0) @binding(2) var<uniform> epu_frame: FrameUniforms;

@group(0) @binding(3) var epu_out_sharp: texture_storage_2d_array<rgba16float, write>;

fn evaluate_env_radiance(dir: vec3f, st: PackedEnvironmentState, time: f32) -> vec3f {
    // Layer 0 is assumed to be RAMP for enclosure weights (recommended).
    let layer0 = st.layers[0];
    let enc = enclosure_from_ramp(layer0);
    let regions = compute_region_weights(dir, enc);

    var radiance = vec3f(0.0);

    for (var i = 0u; i < 8u; i++) {
        let instr = st.layers[i];
        let opcode = instr_opcode(instr);
        if opcode == OP_NOP { continue; }

        let blend = instr_blend(instr);
        let sample = evaluate_layer(dir, instr, enc, regions, time);

        radiance = apply_blend(radiance, sample, blend);
    }

    return radiance;
}

@compute @workgroup_size(8, 8, 1)
fn epu_build(@builtin(global_invocation_id) gid: vec3u) {
    let env_slot = gid.z;
    if env_slot >= epu_frame.active_count { return; }

    let env_id = epu_active_env_ids[env_slot];
    let map_size = epu_frame.map_size;
    if gid.x >= map_size || gid.y >= map_size { return; }

    // Texel -> oct coord -> direction
    let uv = (vec2f(gid.xy) + 0.5) / vec2f(f32(map_size));
    let oct = uv * 2.0 - 1.0;
    let dir = octahedral_decode(oct);

    let st = epu_states[env_id];
    let radiance = evaluate_env_radiance(dir, st, epu_frame.time);

    textureStore(epu_out_sharp, vec2u(gid.xy), i32(env_id), vec4f(radiance, 1.0));
}
