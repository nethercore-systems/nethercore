// ============================================================================
// EPU COMPUTE: IMPORT CUBE FACES -> OCTAHEDRAL RADIANCE (mip 0)
//
// The face convention matches the documented runtime/ROM order:
//   +X, -X, +Y, -Y, +Z, -Z
//
// UV convention is the standard cube-map convention with image origin at the
// top-left of each face:
//   +X: ( Z, -Y)
//   -X: (-Z, -Y)
//   +Y: ( X, -Z)
//   -Y: ( X,  Z)
//   +Z: (-X, -Y)
//   -Z: ( X, -Y)
//
// Mip 0 is imported with a small filtered footprint rather than a single center
// tap so imported probes do not show obvious octahedral/fold stair-stepping at
// low roughness.
// ============================================================================

struct FrameUniforms {
    active_count: u32,
    map_size: u32,
    _pad0: u32,
    _pad1: u32,
}

@group(0) @binding(0) var cube_px: texture_2d<f32>;
@group(0) @binding(1) var cube_nx: texture_2d<f32>;
@group(0) @binding(2) var cube_py: texture_2d<f32>;
@group(0) @binding(3) var cube_ny: texture_2d<f32>;
@group(0) @binding(4) var cube_pz: texture_2d<f32>;
@group(0) @binding(5) var cube_nz: texture_2d<f32>;
@group(0) @binding(6) var cube_sampler: sampler;
@group(0) @binding(7) var<storage, read> epu_active_env_ids: array<u32>;
@group(0) @binding(8) var<uniform> epu_frame: FrameUniforms;
@group(0) @binding(9) var epu_out_sharp: texture_storage_2d_array<rgba16float, write>;

fn sign_not_zero(v: vec2f) -> vec2f {
    return vec2f(select(-1.0, 1.0, v.x >= 0.0), select(-1.0, 1.0, v.y >= 0.0));
}

fn octahedral_decode(oct: vec2f) -> vec3f {
    var v = vec3f(oct.x, oct.y, 1.0 - abs(oct.x) - abs(oct.y));
    if v.z < 0.0 {
        let folded = (1.0 - abs(v.yx)) * sign_not_zero(v.xy);
        v.x = folded.x;
        v.y = folded.y;
    }
    return normalize(v);
}

fn sample_cube_level(face: u32, uv: vec2f) -> vec3f {
    let uv_clamped = clamp(uv, vec2f(0.0), vec2f(1.0));
    switch face {
        case 0u: {
            return textureSampleLevel(cube_px, cube_sampler, uv_clamped, 0.0).rgb;
        }
        case 1u: {
            return textureSampleLevel(cube_nx, cube_sampler, uv_clamped, 0.0).rgb;
        }
        case 2u: {
            return textureSampleLevel(cube_py, cube_sampler, uv_clamped, 0.0).rgb;
        }
        case 3u: {
            return textureSampleLevel(cube_ny, cube_sampler, uv_clamped, 0.0).rgb;
        }
        case 4u: {
            return textureSampleLevel(cube_pz, cube_sampler, uv_clamped, 0.0).rgb;
        }
        default: {
            return textureSampleLevel(cube_nz, cube_sampler, uv_clamped, 0.0).rgb;
        }
    }
}

fn sample_cube_faces(dir: vec3f) -> vec3f {
    let abs_dir = abs(dir);

    if abs_dir.x >= abs_dir.y && abs_dir.x >= abs_dir.z {
        let inv = 1.0 / max(abs_dir.x, 1e-6);
        if dir.x > 0.0 {
            let uv = vec2f(dir.z, -dir.y) * inv * 0.5 + 0.5;
            return sample_cube_level(0u, uv);
        }

        let uv = vec2f(-dir.z, -dir.y) * inv * 0.5 + 0.5;
        return sample_cube_level(1u, uv);
    }

    if abs_dir.y >= abs_dir.z {
        let inv = 1.0 / max(abs_dir.y, 1e-6);
        if dir.y > 0.0 {
            let uv = vec2f(dir.x, -dir.z) * inv * 0.5 + 0.5;
            return sample_cube_level(2u, uv);
        }

        let uv = vec2f(dir.x, dir.z) * inv * 0.5 + 0.5;
        return sample_cube_level(3u, uv);
    }

    let inv = 1.0 / max(abs_dir.z, 1e-6);
    if dir.z > 0.0 {
        let uv = vec2f(-dir.x, -dir.y) * inv * 0.5 + 0.5;
        return sample_cube_level(4u, uv);
    }

    let uv = vec2f(dir.x, -dir.y) * inv * 0.5 + 0.5;
    return sample_cube_level(5u, uv);
}

fn sample_imported_radiance(oct_center: vec2f, texel_size: vec2f) -> vec3f {
    let offsets = array<vec2f, 9>(
        vec2f(-0.5, -0.5),
        vec2f( 0.0, -0.5),
        vec2f( 0.5, -0.5),
        vec2f(-0.5,  0.0),
        vec2f( 0.0,  0.0),
        vec2f( 0.5,  0.0),
        vec2f(-0.5,  0.5),
        vec2f( 0.0,  0.5),
        vec2f( 0.5,  0.5),
    );
    let weights = array<f32, 9>(
        1.0, 2.0, 1.0,
        2.0, 4.0, 2.0,
        1.0, 2.0, 1.0,
    );

    var accum = vec3f(0.0);
    var weight_sum = 0.0;

    for (var i = 0u; i < 9u; i = i + 1u) {
        let oct = oct_center + offsets[i] * texel_size;
        let dir = octahedral_decode(oct);
        let w = weights[i];
        accum += sample_cube_faces(dir) * w;
        weight_sum += w;
    }

    return accum / max(weight_sum, 1e-5);
}

@compute @workgroup_size(8, 8, 1)
fn epu_import_cube(@builtin(global_invocation_id) gid: vec3u) {
    let map_size = epu_frame.map_size;
    if gid.x >= map_size || gid.y >= map_size {
        return;
    }

    let env_id = epu_active_env_ids[0];
    let uv = (vec2f(gid.xy) + 0.5) / vec2f(f32(map_size));
    let oct = uv * 2.0 - 1.0;
    let texel_size = vec2f(2.0 / f32(map_size));
    let radiance = sample_imported_radiance(oct, texel_size);
    textureStore(epu_out_sharp, vec2u(gid.xy), i32(env_id), vec4f(radiance, 1.0));
}
