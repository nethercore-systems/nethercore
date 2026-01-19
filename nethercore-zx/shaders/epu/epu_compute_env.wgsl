// ============================================================================
// EPU COMPUTE: ENVIRONMENT EVALUATION
// Builds EnvSharp and EnvLight0 for all active environments.
// ============================================================================

// PackedEnvironmentState: 8 layers stored as pairs of u32 (lo, hi) for u64 emulation
// Each layer is 64 bits, stored as [lo0, hi0, lo1, hi1, ..., lo7, hi7]
struct PackedEnvironmentState {
    layers: array<vec2u, 8>,
}

struct FrameUniforms {
    time: f32,
    active_count: u32,
    map_size: u32,
    _pad0: u32,
}

@group(0) @binding(0) var<storage, read> epu_palette: array<vec4f>;
@group(0) @binding(1) var<storage, read> epu_states: array<PackedEnvironmentState>;
@group(0) @binding(2) var<storage, read> epu_active_env_ids: array<u32>;
@group(0) @binding(3) var<uniform> epu_frame: FrameUniforms;

@group(0) @binding(4) var epu_out_sharp: texture_storage_2d_array<rgba16float, write>;
@group(0) @binding(5) var epu_out_light0: texture_storage_2d_array<rgba16float, write>;

fn feature_is_emissive(lo: u32, hi: u32) -> bool {
    // Policy: emissive iff ADD blend.
    return instr_blend(lo, hi) == BLEND_ADD;
}

struct DualRadiance {
    sharp: vec3f,
    light0: vec3f,
}

fn evaluate_env_dual(dir: vec3f, st: PackedEnvironmentState, time: f32) -> DualRadiance {
    // Layer 0 is assumed to be RAMP for enclosure weights (recommended).
    let layer0 = st.layers[0];
    let enc = enclosure_from_ramp(layer0.x, layer0.y);
    let regions = compute_region_weights(dir, enc);

    var sharp = vec3f(0.0);
    var light0 = vec3f(0.0);

    for (var i = 0u; i < 8u; i++) {
        let layer = st.layers[i];
        let lo = layer.x;
        let hi = layer.y;
        let opcode = instr_opcode(lo, hi);
        if opcode == OP_NOP { continue; }

        let blend = instr_blend(lo, hi);

        let sample = evaluate_layer(dir, lo, hi, enc, regions, time, &epu_palette);

        // Sharp always receives all layers.
        sharp = apply_blend(sharp, sample, blend);

        // Light0 receives bounds + emissive features only.
        let is_feature = opcode >= OP_DECAL;
        if !is_feature || feature_is_emissive(lo, hi) {
            light0 = apply_blend(light0, sample, blend);
        }
    }

    return DualRadiance(sharp, light0);
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
    let dual = evaluate_env_dual(dir, st, epu_frame.time);

    textureStore(epu_out_sharp, vec2u(gid.xy), i32(env_id), vec4f(dual.sharp, 1.0));
    textureStore(epu_out_light0, vec2u(gid.xy), i32(env_id), vec4f(dual.light0, 1.0));
}
