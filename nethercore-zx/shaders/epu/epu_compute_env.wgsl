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
    // Start with default bounds direction (will be updated by bounds layers)
    var bounds_dir = vec3f(0.0, 1.0, 0.0);
    // Default regions: all-sky (bounds layers will compute their own regions)
    var regions = RegionWeights(1.0, 0.0, 0.0);
    var bounds_count = 0u;
    var shared_bounds_dir_set = false;

    var radiance = vec3f(0.0);

    for (var i = 0u; i < 8u; i++) {
        let instr = st.layers[i];
        let opcode = instr_opcode(instr);
        if opcode == OP_NOP { continue; }

        let is_bounds = opcode < OP_FEATURE_MIN;

        let blend = instr_blend(instr);

        if is_bounds {
            // Bounds opcodes may evaluate against their own decoded axis, but
            // only the first structural organizer should redefine the shared
            // downstream axis used by later layers.
            let eval_bounds_dir = bounds_dir_from_layer(instr, opcode, bounds_dir);
            if !shared_bounds_dir_set {
                bounds_dir = eval_bounds_dir;
                shared_bounds_dir_set = true;
            }
            let bounds_result = evaluate_bounds_layer(dir, instr, opcode, eval_bounds_dir, regions);
            var region_mix = bounds_result.region_mix;
            if bounds_count > 0u {
                let organizer_bounds =
                    opcode == OP_SECTOR
                    || opcode == OP_SPLIT
                    || opcode == OP_CELL
                    || opcode == OP_APERTURE;
                let retag_scale = select(0.72, 0.38, organizer_bounds);
                region_mix *= retag_scale;
            }
            regions = compose_bounds_regions(regions, bounds_result.regions, region_mix);
            radiance = apply_blend(radiance, bounds_result.sample, blend);
            bounds_count += 1u;
        } else {
            // Feature opcode: just get sample using current regions
            let sample = evaluate_layer(dir, instr, bounds_dir, regions);
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
    let oct_l1 = abs(oct.x) + abs(oct.y);
    let fold_stress = 1.0 - smoothstep(0.025, 0.11, abs(oct_l1 - 1.0));
    let corner_stress =
        smoothstep(0.74, 0.95, max(abs(oct.x), abs(oct.y)))
        * smoothstep(1.02, 1.32, oct_l1);
    let oct_stress = max(fold_stress, corner_stress);

    var final_radiance = radiance;
    if oct_stress > 0.001 {
        let oct_step = 2.0 / f32(map_size);
        let sample_axis = select(
            vec2f(oct_step, 0.0),
            vec2f(0.0, oct_step),
            abs(oct.x) > abs(oct.y)
        );
        let dir_a = octahedral_decode(clamp(oct + sample_axis, vec2f(-1.0), vec2f(1.0)));
        let dir_b = octahedral_decode(clamp(oct - sample_axis, vec2f(-1.0), vec2f(1.0)));
        let avg_radiance = (
            radiance
            + evaluate_env_radiance(dir_a, st)
            + evaluate_env_radiance(dir_b, st)
        ) / 3.0;
        final_radiance = mix(radiance, avg_radiance, oct_stress * 0.35);
    }

    textureStore(epu_out_sharp, vec2u(gid.xy), i32(env_id), vec4f(final_radiance, 1.0));
}
