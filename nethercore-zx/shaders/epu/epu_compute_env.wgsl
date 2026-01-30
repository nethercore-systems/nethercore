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
    // Start with default enclosure (will be set by first bounds layer)
    var enc = EnclosureConfig(vec3f(0.0, 1.0, 0.0), 0.5, -0.5, 0.1);
    // Default regions should be direction-dependent, so presets can start with any bounds
    // opcode (SECTOR / SPLIT / SILHOUETTE / APERTURE) without requiring a leading RAMP.
    var regions = compute_region_weights(dir, enc);

    var radiance = vec3f(0.0);

    for (var i = 0u; i < 8u; i++) {
        let instr = st.layers[i];
        let opcode = instr_opcode(instr);
        if opcode == OP_NOP { continue; }

        let is_bounds = opcode < OP_FEATURE_MIN;

        let blend = instr_blend(instr);

        if is_bounds {
            // Bounds opcode: update enclosure, get sample + output regions
            enc = enclosure_from_layer(instr, opcode, enc);
            let bounds_result = evaluate_bounds_layer(dir, instr, opcode, enc, regions);
            regions = bounds_result.regions;  // Use output regions for subsequent features
            radiance = apply_blend(radiance, bounds_result.sample, blend);
        } else {
            // Feature opcode: just get sample using current regions
            let sample = evaluate_layer(dir, instr, enc, regions);
            radiance = apply_blend(radiance, sample, blend);
        }
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
    let radiance = evaluate_env_radiance(dir, st);

    textureStore(epu_out_sharp, vec2u(gid.xy), i32(env_id), vec4f(radiance, 1.0));
}
