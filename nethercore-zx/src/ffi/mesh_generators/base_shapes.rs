//! Base procedural mesh generation (FORMAT_NORMAL)
//!
//! These functions generate common 3D primitives with position and normal data only,
//! suitable for solid color rendering.

use tracing::{info, warn};
use wasmtime::Caller;

use crate::ffi::ZXGameContext;
use crate::ffi::guards::guard_init_only;
use crate::graphics::FORMAT_NORMAL;
use crate::procedural::{self, MeshData};
use crate::state::PendingMeshPacked;

/// Generate a cube mesh
///
/// # Arguments
/// * `size_x` - Half-extent along X axis
/// * `size_y` - Half-extent along Y axis
/// * `size_z` - Half-extent along Z axis
///
/// Returns mesh handle (>0) on success, 0 on failure.
///
/// The cube has 24 vertices (4 per face) with flat normals and box-unwrapped UVs.
///
/// **Init-only**: Must be called during `init()`.
pub fn cube(mut caller: Caller<'_, ZXGameContext>, size_x: f32, size_y: f32, size_z: f32) -> u32 {
    guard_init_only!(caller, "cube");

    // Validate parameters
    if size_x <= 0.0 || size_y <= 0.0 || size_z <= 0.0 {
        warn!(
            "cube: all sizes must be > 0.0 (got {}, {}, {})",
            size_x, size_y, size_z
        );
        return 0;
    }

    // Generate PACKED mesh data (Vec<u8>)
    let mesh_data: MeshData = procedural::generate_cube(size_x, size_y, size_z);

    let vertex_count = mesh_data.vertices.len() / 16; // 16 bytes per POS_NORMAL vertex
    let index_count = mesh_data.indices.len();

    // Allocate handle and queue mesh
    let state = &mut caller.data_mut().ffi;
    let handle = state.next_mesh_handle;
    state.next_mesh_handle += 1;

    state.pending_meshes_packed.push(PendingMeshPacked {
        handle,
        format: FORMAT_NORMAL, // Base format (0-15, no FORMAT_PACKED flag)
        vertex_data: mesh_data.vertices,
        index_data: Some(mesh_data.indices),
    });

    info!(
        "cube: created mesh {} ({}×{}×{}, {} verts, {} indices, PACKED)",
        handle,
        size_x * 2.0,
        size_y * 2.0,
        size_z * 2.0,
        vertex_count,
        index_count
    );
    handle
}

/// Generate a UV sphere mesh
///
/// # Arguments
/// * `radius` - Sphere radius
/// * `segments` - Number of longitudinal divisions (clamped 3-256)
/// * `rings` - Number of latitudinal divisions (clamped 2-256)
///
/// Returns mesh handle (>0) on success, 0 on failure.
///
/// The sphere uses equirectangular UV mapping and smooth normals.
///
/// **Init-only**: Must be called during `init()`.
pub fn sphere(
    mut caller: Caller<'_, ZXGameContext>,
    radius: f32,
    segments: u32,
    rings: u32,
) -> u32 {
    guard_init_only!(caller, "sphere");

    // Validate parameters
    if radius <= 0.0 {
        warn!("sphere: radius must be > 0.0 (got {})", radius);
        return 0;
    }

    // Generate PACKED mesh data (Vec<u8>)
    let mesh_data: MeshData = procedural::generate_sphere(radius, segments, rings);

    let vertex_count = mesh_data.vertices.len() / 16; // 16 bytes per POS_NORMAL vertex
    let index_count = mesh_data.indices.len();

    // Allocate handle and queue mesh
    let state = &mut caller.data_mut().ffi;
    let handle = state.next_mesh_handle;
    state.next_mesh_handle += 1;

    state.pending_meshes_packed.push(PendingMeshPacked {
        handle,
        format: FORMAT_NORMAL, // Base format (0-15, no FORMAT_PACKED flag)
        vertex_data: mesh_data.vertices,
        index_data: Some(mesh_data.indices),
    });

    info!(
        "sphere: created mesh {} (radius={}, {}x{} segments, {} verts, {} indices, PACKED)",
        handle, radius, segments, rings, vertex_count, index_count
    );
    handle
}

/// Generate a cylinder or cone mesh
///
/// # Arguments
/// * `radius_bottom` - Bottom radius (>= 0.0)
/// * `radius_top` - Top radius (>= 0.0)
/// * `height` - Cylinder height
/// * `segments` - Number of radial divisions (clamped 3-256)
///
/// Returns mesh handle (>0) on success, 0 on failure.
///
/// If radius_bottom != radius_top, creates a tapered cylinder or cone.
/// Includes top and bottom caps (omitted if radius is 0).
///
/// **Init-only**: Must be called during `init()`.
pub fn cylinder(
    mut caller: Caller<'_, ZXGameContext>,
    radius_bottom: f32,
    radius_top: f32,
    height: f32,
    segments: u32,
) -> u32 {
    guard_init_only!(caller, "cylinder");

    // Validate parameters
    if radius_bottom < 0.0 || radius_top < 0.0 {
        warn!(
            "cylinder: radii must be >= 0.0 (got {}, {})",
            radius_bottom, radius_top
        );
        return 0;
    }

    if height <= 0.0 {
        warn!("cylinder: height must be > 0.0 (got {})", height);
        return 0;
    }

    // Generate PACKED mesh data (Vec<u8>)
    let mesh_data: MeshData =
        procedural::generate_cylinder(radius_bottom, radius_top, height, segments);

    let vertex_count = mesh_data.vertices.len() / 16; // 16 bytes per POS_NORMAL vertex
    let index_count = mesh_data.indices.len();

    // Allocate handle and queue mesh
    let state = &mut caller.data_mut().ffi;
    let handle = state.next_mesh_handle;
    state.next_mesh_handle += 1;

    state.pending_meshes_packed.push(PendingMeshPacked {
        handle,
        format: FORMAT_NORMAL, // Base format (0-15, no FORMAT_PACKED flag)
        vertex_data: mesh_data.vertices,
        index_data: Some(mesh_data.indices),
    });

    info!(
        "cylinder: created mesh {} (radii={}/{}, height={}, {} segments, {} verts, {} indices, PACKED)",
        handle, radius_bottom, radius_top, height, segments, vertex_count, index_count
    );
    handle
}

/// Generate a subdivided plane mesh on the XZ plane
///
/// # Arguments
/// * `size_x` - Width along X axis
/// * `size_z` - Depth along Z axis
/// * `subdivisions_x` - Number of X subdivisions (clamped 1-256)
/// * `subdivisions_z` - Number of Z subdivisions (clamped 1-256)
///
/// Returns mesh handle (>0) on success, 0 on failure.
///
/// The plane is centered at the origin with Y=0, facing up (+Y).
///
/// **Init-only**: Must be called during `init()`.
pub fn plane(
    mut caller: Caller<'_, ZXGameContext>,
    size_x: f32,
    size_z: f32,
    subdivisions_x: u32,
    subdivisions_z: u32,
) -> u32 {
    guard_init_only!(caller, "plane");

    // Validate parameters
    if size_x <= 0.0 || size_z <= 0.0 {
        warn!("plane: sizes must be > 0.0 (got {}, {})", size_x, size_z);
        return 0;
    }

    // Generate PACKED mesh data (Vec<u8>)
    let mesh_data: MeshData =
        procedural::generate_plane(size_x, size_z, subdivisions_x, subdivisions_z);

    let vertex_count = mesh_data.vertices.len() / 16; // 16 bytes per POS_NORMAL vertex
    let index_count = mesh_data.indices.len();

    // Allocate handle and queue mesh
    let state = &mut caller.data_mut().ffi;
    let handle = state.next_mesh_handle;
    state.next_mesh_handle += 1;

    state.pending_meshes_packed.push(PendingMeshPacked {
        handle,
        format: FORMAT_NORMAL, // Base format (0-15, no FORMAT_PACKED flag)
        vertex_data: mesh_data.vertices,
        index_data: Some(mesh_data.indices),
    });

    info!(
        "plane: created mesh {} ({}×{}, {}×{} subdivisions, {} verts, {} indices, PACKED)",
        handle, size_x, size_z, subdivisions_x, subdivisions_z, vertex_count, index_count
    );
    handle
}

/// Generate a torus (donut) mesh
///
/// # Arguments
/// * `major_radius` - Distance from torus center to tube center
/// * `minor_radius` - Tube radius
/// * `major_segments` - Segments around major circle (clamped 3-256)
/// * `minor_segments` - Segments around tube (clamped 3-256)
///
/// Returns mesh handle (>0) on success, 0 on failure.
///
/// The torus lies in the XZ plane with smooth normals and wrapped UVs.
///
/// **Init-only**: Must be called during `init()`.
pub fn torus(
    mut caller: Caller<'_, ZXGameContext>,
    major_radius: f32,
    minor_radius: f32,
    major_segments: u32,
    minor_segments: u32,
) -> u32 {
    guard_init_only!(caller, "torus");

    // Validate parameters
    if major_radius <= 0.0 || minor_radius <= 0.0 {
        warn!(
            "torus: radii must be > 0.0 (got {}, {})",
            major_radius, minor_radius
        );
        return 0;
    }

    // Generate PACKED mesh data (Vec<u8>)
    let mesh_data: MeshData =
        procedural::generate_torus(major_radius, minor_radius, major_segments, minor_segments);

    let vertex_count = mesh_data.vertices.len() / 16; // 16 bytes per POS_NORMAL vertex
    let index_count = mesh_data.indices.len();

    // Allocate handle and queue mesh
    let state = &mut caller.data_mut().ffi;
    let handle = state.next_mesh_handle;
    state.next_mesh_handle += 1;

    state.pending_meshes_packed.push(PendingMeshPacked {
        handle,
        format: FORMAT_NORMAL, // Base format (0-15, no FORMAT_PACKED flag)
        vertex_data: mesh_data.vertices,
        index_data: Some(mesh_data.indices),
    });

    info!(
        "torus: created mesh {} (major={}, minor={}, {}×{} segments, {} verts, {} indices, PACKED)",
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

/// Generate a capsule (pill shape) mesh
///
/// # Arguments
/// * `radius` - Capsule radius
/// * `height` - Height of cylindrical section (>= 0.0)
/// * `segments` - Number of radial divisions (clamped 3-256)
/// * `rings` - Number of latitudinal divisions per hemisphere (clamped 1-128)
///
/// Returns mesh handle (>0) on success, 0 on failure.
///
/// Total capsule height = height + 2 * radius.
/// If height is 0, generates a sphere instead.
///
/// **Init-only**: Must be called during `init()`.
pub fn capsule(
    mut caller: Caller<'_, ZXGameContext>,
    radius: f32,
    height: f32,
    segments: u32,
    rings: u32,
) -> u32 {
    guard_init_only!(caller, "capsule");

    // Validate parameters
    if radius <= 0.0 {
        warn!("capsule: radius must be > 0.0 (got {})", radius);
        return 0;
    }

    if height < 0.0 {
        warn!("capsule: height must be >= 0.0 (got {})", height);
        return 0;
    }

    // Generate PACKED mesh data (Vec<u8>)
    let mesh_data: MeshData = procedural::generate_capsule(radius, height, segments, rings);

    let vertex_count = mesh_data.vertices.len() / 16; // 16 bytes per POS_NORMAL vertex
    let index_count = mesh_data.indices.len();

    // Allocate handle and queue mesh
    let state = &mut caller.data_mut().ffi;
    let handle = state.next_mesh_handle;
    state.next_mesh_handle += 1;

    state.pending_meshes_packed.push(PendingMeshPacked {
        handle,
        format: FORMAT_NORMAL, // Base format (0-15, no FORMAT_PACKED flag)
        vertex_data: mesh_data.vertices,
        index_data: Some(mesh_data.indices),
    });

    info!(
        "capsule: created mesh {} (radius={}, height={}, {} segments, {} rings, {} verts, {} indices, PACKED)",
        handle, radius, height, segments, rings, vertex_count, index_count
    );
    handle
}
