//! Tangent-enabled procedural mesh generation (FORMAT_UV | FORMAT_NORMAL | FORMAT_TANGENT)
//!
//! These functions generate common 3D primitives with position, UV, normal, and tangent data,
//! suitable for normal-mapped rendering.

use tracing::{info, warn};
use wasmtime::Caller;

use crate::ffi::ZXGameContext;
use crate::ffi::guards::check_init_only;
use crate::graphics::{FORMAT_NORMAL, FORMAT_TANGENT, FORMAT_UV};
use crate::procedural::{self, MeshDataTangent};
use crate::state::PendingMeshPacked;

/// Generate a UV sphere mesh with tangent data for normal mapping
///
/// # Arguments
/// * `radius` - Sphere radius
/// * `segments` - Number of longitudinal divisions (clamped 3-256)
/// * `rings` - Number of latitudinal divisions (clamped 2-256)
///
/// Returns mesh handle (>0) on success, 0 on failure.
///
/// **Init-only**: Must be called during `init()`.
pub fn sphere_tangent(
    mut caller: Caller<'_, ZXGameContext>,
    radius: f32,
    segments: u32,
    rings: u32,
) -> u32 {
    if let Err(e) = check_init_only(&caller, "sphere_tangent") {
        warn!("{}", e);
        return 0;
    }

    if radius <= 0.0 {
        warn!("sphere_tangent: radius must be > 0.0 (got {})", radius);
        return 0;
    }

    let mesh_data: MeshDataTangent = procedural::generate_sphere_tangent(radius, segments, rings);

    let vertex_count = mesh_data.vertices.len() / 20; // 20 bytes per POS_UV_NORMAL_TANGENT vertex
    let index_count = mesh_data.indices.len();

    let state = &mut caller.data_mut().ffi;
    let handle = state.next_mesh_handle;
    state.next_mesh_handle += 1;

    state.pending_meshes_packed.push(PendingMeshPacked {
        handle,
        format: FORMAT_UV | FORMAT_NORMAL | FORMAT_TANGENT,
        vertex_data: mesh_data.vertices,
        index_data: Some(mesh_data.indices),
    });

    info!(
        "sphere_tangent: created mesh {} (radius={}, {}x{} segments, {} verts, {} indices, PACKED with tangents)",
        handle, radius, segments, rings, vertex_count, index_count
    );
    handle
}

/// Generate a plane mesh with tangent data for normal mapping
///
/// # Arguments
/// * `size_x` - Width along X axis
/// * `size_z` - Depth along Z axis
/// * `subdivisions_x` - Number of X subdivisions (clamped 1-256)
/// * `subdivisions_z` - Number of Z subdivisions (clamped 1-256)
///
/// Returns mesh handle (>0) on success, 0 on failure.
///
/// **Init-only**: Must be called during `init()`.
pub fn plane_tangent(
    mut caller: Caller<'_, ZXGameContext>,
    size_x: f32,
    size_z: f32,
    subdivisions_x: u32,
    subdivisions_z: u32,
) -> u32 {
    if let Err(e) = check_init_only(&caller, "plane_tangent") {
        warn!("{}", e);
        return 0;
    }

    if size_x <= 0.0 || size_z <= 0.0 {
        warn!(
            "plane_tangent: sizes must be > 0.0 (got {}, {})",
            size_x, size_z
        );
        return 0;
    }

    let mesh_data: MeshDataTangent =
        procedural::generate_plane_tangent(size_x, size_z, subdivisions_x, subdivisions_z);

    let vertex_count = mesh_data.vertices.len() / 20;
    let index_count = mesh_data.indices.len();

    let state = &mut caller.data_mut().ffi;
    let handle = state.next_mesh_handle;
    state.next_mesh_handle += 1;

    state.pending_meshes_packed.push(PendingMeshPacked {
        handle,
        format: FORMAT_UV | FORMAT_NORMAL | FORMAT_TANGENT,
        vertex_data: mesh_data.vertices,
        index_data: Some(mesh_data.indices),
    });

    info!(
        "plane_tangent: created mesh {} ({}×{}, {}×{} subdivisions, {} verts, {} indices, PACKED with tangents)",
        handle, size_x, size_z, subdivisions_x, subdivisions_z, vertex_count, index_count
    );
    handle
}

/// Generate a cube mesh with tangent data for normal mapping
///
/// # Arguments
/// * `size_x` - Half-extent along X axis
/// * `size_y` - Half-extent along Y axis
/// * `size_z` - Half-extent along Z axis
///
/// Returns mesh handle (>0) on success, 0 on failure.
///
/// **Init-only**: Must be called during `init()`.
pub fn cube_tangent(
    mut caller: Caller<'_, ZXGameContext>,
    size_x: f32,
    size_y: f32,
    size_z: f32,
) -> u32 {
    if let Err(e) = check_init_only(&caller, "cube_tangent") {
        warn!("{}", e);
        return 0;
    }

    if size_x <= 0.0 || size_y <= 0.0 || size_z <= 0.0 {
        warn!(
            "cube_tangent: all sizes must be > 0.0 (got {}, {}, {})",
            size_x, size_y, size_z
        );
        return 0;
    }

    let mesh_data: MeshDataTangent = procedural::generate_cube_tangent(size_x, size_y, size_z);

    let vertex_count = mesh_data.vertices.len() / 20;
    let index_count = mesh_data.indices.len();

    let state = &mut caller.data_mut().ffi;
    let handle = state.next_mesh_handle;
    state.next_mesh_handle += 1;

    state.pending_meshes_packed.push(PendingMeshPacked {
        handle,
        format: FORMAT_UV | FORMAT_NORMAL | FORMAT_TANGENT,
        vertex_data: mesh_data.vertices,
        index_data: Some(mesh_data.indices),
    });

    info!(
        "cube_tangent: created mesh {} ({}×{}×{}, {} verts, {} indices, PACKED with tangents)",
        handle,
        size_x * 2.0,
        size_y * 2.0,
        size_z * 2.0,
        vertex_count,
        index_count
    );
    handle
}

/// Generate a torus mesh with tangent data for normal mapping
///
/// # Arguments
/// * `major_radius` - Distance from torus center to tube center
/// * `minor_radius` - Tube radius
/// * `major_segments` - Segments around major circle (clamped 3-256)
/// * `minor_segments` - Segments around tube (clamped 3-256)
///
/// Returns mesh handle (>0) on success, 0 on failure.
///
/// **Init-only**: Must be called during `init()`.
pub fn torus_tangent(
    mut caller: Caller<'_, ZXGameContext>,
    major_radius: f32,
    minor_radius: f32,
    major_segments: u32,
    minor_segments: u32,
) -> u32 {
    if let Err(e) = check_init_only(&caller, "torus_tangent") {
        warn!("{}", e);
        return 0;
    }

    if major_radius <= 0.0 || minor_radius <= 0.0 {
        warn!(
            "torus_tangent: radii must be > 0.0 (got {}, {})",
            major_radius, minor_radius
        );
        return 0;
    }

    let mesh_data: MeshDataTangent = procedural::generate_torus_tangent(
        major_radius,
        minor_radius,
        major_segments,
        minor_segments,
    );

    let vertex_count = mesh_data.vertices.len() / 20;
    let index_count = mesh_data.indices.len();

    let state = &mut caller.data_mut().ffi;
    let handle = state.next_mesh_handle;
    state.next_mesh_handle += 1;

    state.pending_meshes_packed.push(PendingMeshPacked {
        handle,
        format: FORMAT_UV | FORMAT_NORMAL | FORMAT_TANGENT,
        vertex_data: mesh_data.vertices,
        index_data: Some(mesh_data.indices),
    });

    info!(
        "torus_tangent: created mesh {} (major={}, minor={}, {}×{} segments, {} verts, {} indices, PACKED with tangents)",
        handle,
        major_radius,
        minor_radius,
        major_segments,
        minor_segments,
        vertex_count,
        index_count
    );
    handle
}
