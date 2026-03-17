// ============================================================================
// EPU COMPUTE: COPY IMPORTED CUBE FACES INTO ACTIVE-FRAME FACE ARRAY
//
// This keeps a direct, non-octahedral source around for imported environments
// so render-time background and low-roughness reflections can sample the faces
// without going through octa mip0.
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
@group(0) @binding(8) var<storage, read> epu_imported_face_base_layers: array<u32>;
@group(0) @binding(9) var<uniform> epu_frame: FrameUniforms;
@group(0) @binding(10) var epu_imported_faces: texture_storage_2d_array<rgba16float, write>;

fn remap_texel(dst_xy: vec2u, dst_size: u32, src_dim: vec2u) -> vec2i {
    let src_extent = max(src_dim, vec2u(1u, 1u));
    let dst_extent = max(dst_size, 1u);
    let src_xy = min((dst_xy * src_extent) / vec2u(dst_extent, dst_extent), src_extent - vec2u(1u, 1u));
    return vec2i(src_xy);
}

fn sample_face(face_index: u32, dst_xy: vec2u, dst_size: u32) -> vec4f {
    switch face_index {
        case 0u: {
            let src_dim = textureDimensions(cube_px, 0);
            return textureLoad(cube_px, remap_texel(dst_xy, dst_size, src_dim), 0);
        }
        case 1u: {
            let src_dim = textureDimensions(cube_nx, 0);
            return textureLoad(cube_nx, remap_texel(dst_xy, dst_size, src_dim), 0);
        }
        case 2u: {
            let src_dim = textureDimensions(cube_py, 0);
            return textureLoad(cube_py, remap_texel(dst_xy, dst_size, src_dim), 0);
        }
        case 3u: {
            let src_dim = textureDimensions(cube_ny, 0);
            return textureLoad(cube_ny, remap_texel(dst_xy, dst_size, src_dim), 0);
        }
        case 4u: {
            let src_dim = textureDimensions(cube_pz, 0);
            return textureLoad(cube_pz, remap_texel(dst_xy, dst_size, src_dim), 0);
        }
        default: {
            let src_dim = textureDimensions(cube_nz, 0);
            return textureLoad(cube_nz, remap_texel(dst_xy, dst_size, src_dim), 0);
        }
    }
}

@compute @workgroup_size(8, 8, 1)
fn epu_copy_cube_faces(@builtin(global_invocation_id) gid: vec3u) {
    let face_size = epu_frame.map_size;
    if gid.x >= face_size || gid.y >= face_size || gid.z >= 6u {
        return;
    }

    let env_id = epu_active_env_ids[0];
    let face_base = epu_imported_face_base_layers[env_id];
    let texel = sample_face(gid.z, gid.xy, face_size);
    textureStore(epu_imported_faces, vec2u(gid.xy), i32(face_base + gid.z), texel);
}
