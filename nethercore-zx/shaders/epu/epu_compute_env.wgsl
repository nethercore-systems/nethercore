// ============================================================================
// EPU COMPUTE: ENVIRONMENT EVALUATION
// Builds EnvSharp and EnvLight0 for all active environments.
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
@group(0) @binding(4) var epu_out_light0: texture_storage_2d_array<rgba16float, write>;

// EPU v2 emissive policy:
// - emissive = 0: Layer doesn't contribute to L_light0 (decorative only)
// - emissive = 1-14: Scaled contribution (6.7% - 93.3%)
// - emissive = 15: Full contribution (100%)
// Returns the emissive scale factor (0-1) for this instruction.

struct DualRadiance {
    sharp: vec3f,
    light0: vec3f,
}

fn evaluate_env_dual(dir: vec3f, st: PackedEnvironmentState, time: f32) -> DualRadiance {
    // Layer 0 is assumed to be RAMP for enclosure weights (recommended).
    let layer0 = st.layers[0];
    let enc = enclosure_from_ramp(layer0);
    let regions = compute_region_weights(dir, enc);

    var sharp = vec3f(0.0);
    var light0 = vec3f(0.0);

    for (var i = 0u; i < 8u; i++) {
        let instr = st.layers[i];
        let opcode = instr_opcode(instr);
        if opcode == OP_NOP { continue; }

        let blend = instr_blend(instr);
        let sample = evaluate_layer(dir, instr, enc, regions, time);

        // Sharp always receives all layers.
        sharp = apply_blend(sharp, sample, blend);

        // Light0 contribution is scaled by the emissive field.
        // emissive=0 means no contribution, emissive=15 means full contribution.
        let emissive_scale = instr_emissive_f32(instr);
        if emissive_scale > 0.0 {
            // Scale the sample weight by emissive factor for light0
            let scaled_sample = LayerSample(sample.rgb, sample.w * emissive_scale);
            light0 = apply_blend(light0, scaled_sample, blend);
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
