// ============================================================================
// EPU COMPUTE: IRRADIANCE EXTRACTION
// Extracts 6-direction ambient cube from the most blurred light level.
// ============================================================================

struct AmbientCube {
    pos_x: vec3f, _pad0: f32,
    neg_x: vec3f, _pad1: f32,
    pos_y: vec3f, _pad2: f32,
    neg_y: vec3f, _pad3: f32,
    pos_z: vec3f, _pad4: f32,
    neg_z: vec3f, _pad5: f32,
}

struct IrradUniforms {
    active_count: u32,
    _pad0: u32,
    _pad1: u32,
    _pad2: u32,
}

@group(0) @binding(2) var<storage, read> epu_active_env_ids: array<u32>;
@group(0) @binding(4) var epu_blurred: texture_2d_array<f32>;
@group(0) @binding(5) var epu_samp: sampler;
@group(0) @binding(6) var<storage, read_write> epu_ambient: array<AmbientCube>;
@group(0) @binding(7) var<uniform> epu_irrad: IrradUniforms;

// Octahedral encode for sampling - duplicated here for standalone compute shader
fn oct_encode_local(dir: vec3f) -> vec2f {
    let n = dir / (abs(dir.x) + abs(dir.y) + abs(dir.z));
    if n.z < 0.0 {
        return (1.0 - abs(n.yx)) * sign(n.xy);
    }
    return n.xy;
}

@compute @workgroup_size(1, 1, 1)
fn epu_extract_ambient(@builtin(global_invocation_id) gid: vec3u) {
    let env_slot = gid.z;
    if env_slot >= epu_irrad.active_count { return; }

    let env_id = epu_active_env_ids[env_slot];

    let dirs = array<vec3f, 6>(
        vec3f( 1.0,  0.0,  0.0),
        vec3f(-1.0,  0.0,  0.0),
        vec3f( 0.0,  1.0,  0.0),
        vec3f( 0.0, -1.0,  0.0),
        vec3f( 0.0,  0.0,  1.0),
        vec3f( 0.0,  0.0, -1.0),
    );

    var out: AmbientCube;

    for (var i = 0u; i < 6u; i++) {
        let uv = oct_encode_local(dirs[i]) * 0.5 + 0.5;
        let s = textureSampleLevel(epu_blurred, epu_samp, uv, i32(env_id), 0.0).rgb;
        switch i {
            case 0u: { out.pos_x = s; }
            case 1u: { out.neg_x = s; }
            case 2u: { out.pos_y = s; }
            case 3u: { out.neg_y = s; }
            case 4u: { out.pos_z = s; }
            case 5u: { out.neg_z = s; }
            default: {}
        }
    }

    epu_ambient[env_id] = out;
}
