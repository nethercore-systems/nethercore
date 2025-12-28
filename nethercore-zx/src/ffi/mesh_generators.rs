//! Procedural mesh generation FFI functions
//!
//! These functions generate common 3D primitives and return mesh handles.
//! All procedural meshes use packed vertex formats (f16, snorm16, unorm8) for memory efficiency.
//!
//! Two variants provided:
//! - Base functions (cube, sphere, etc.): Format 4 (POS_NORMAL) - uniform colors
//! - _uv variants (sphere_uv, etc.): Format 5 (POS_UV_NORMAL) - textured rendering
//!
//! **IMPORTANT**: All procedural mesh functions are init-only. They queue meshes
//! for GPU upload, which must happen during init() to ensure deterministic rollback.

use anyhow::Result;
use tracing::{info, warn};
use wasmtime::{Caller, Linker};

use super::ZXGameContext;
use super::guards::check_init_only;
use crate::graphics::{FORMAT_NORMAL, FORMAT_UV};
use crate::procedural::{self, MeshData, MeshDataUV};
use crate::state::PendingMeshPacked;

/// Register procedural mesh generation FFI functions
pub fn register(linker: &mut Linker<ZXGameContext>) -> Result<()> {
    // Base procedural shapes (FORMAT_NORMAL - solid colors)
    linker.func_wrap("env", "cube", cube)?;
    linker.func_wrap("env", "sphere", sphere)?;
    linker.func_wrap("env", "cylinder", cylinder)?;
    linker.func_wrap("env", "plane", plane)?;
    linker.func_wrap("env", "torus", torus)?;
    linker.func_wrap("env", "capsule", capsule)?;

    // UV-enabled variants (FORMAT_UV | FORMAT_NORMAL - textured)
    linker.func_wrap("env", "sphere_uv", sphere_uv)?;
    linker.func_wrap("env", "plane_uv", plane_uv)?;
    linker.func_wrap("env", "cube_uv", cube_uv)?;
    linker.func_wrap("env", "cylinder_uv", cylinder_uv)?;
    linker.func_wrap("env", "torus_uv", torus_uv)?;
    linker.func_wrap("env", "capsule_uv", capsule_uv)?;

    Ok(())
}

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
fn cube(mut caller: Caller<'_, ZXGameContext>, size_x: f32, size_y: f32, size_z: f32) -> u32 {
    if let Err(e) = check_init_only(&caller, "cube") {
        warn!("{}", e);
        return 0;
    }

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
fn sphere(mut caller: Caller<'_, ZXGameContext>, radius: f32, segments: u32, rings: u32) -> u32 {
    if let Err(e) = check_init_only(&caller, "sphere") {
        warn!("{}", e);
        return 0;
    }

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
fn cylinder(
    mut caller: Caller<'_, ZXGameContext>,
    radius_bottom: f32,
    radius_top: f32,
    height: f32,
    segments: u32,
) -> u32 {
    if let Err(e) = check_init_only(&caller, "cylinder") {
        warn!("{}", e);
        return 0;
    }

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
    let mesh_data: MeshData = procedural::generate_cylinder(radius_bottom, radius_top, height, segments);

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
fn plane(
    mut caller: Caller<'_, ZXGameContext>,
    size_x: f32,
    size_z: f32,
    subdivisions_x: u32,
    subdivisions_z: u32,
) -> u32 {
    if let Err(e) = check_init_only(&caller, "plane") {
        warn!("{}", e);
        return 0;
    }

    // Validate parameters
    if size_x <= 0.0 || size_z <= 0.0 {
        warn!("plane: sizes must be > 0.0 (got {}, {})", size_x, size_z);
        return 0;
    }

    // Generate PACKED mesh data (Vec<u8>)
    let mesh_data: MeshData = procedural::generate_plane(size_x, size_z, subdivisions_x, subdivisions_z);

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
fn torus(
    mut caller: Caller<'_, ZXGameContext>,
    major_radius: f32,
    minor_radius: f32,
    major_segments: u32,
    minor_segments: u32,
) -> u32 {
    if let Err(e) = check_init_only(&caller, "torus") {
        warn!("{}", e);
        return 0;
    }

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
fn capsule(
    mut caller: Caller<'_, ZXGameContext>,
    radius: f32,
    height: f32,
    segments: u32,
    rings: u32,
) -> u32 {
    if let Err(e) = check_init_only(&caller, "capsule") {
        warn!("{}", e);
        return 0;
    }

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

// ============================================================================
// UV-Enabled Procedural Shapes (Format 5: POS_UV_NORMAL)
// ============================================================================

/// Generate a UV sphere mesh with equirectangular texture mapping
///
/// # Arguments
/// * `radius` - Sphere radius
/// * `segments` - Number of longitudinal divisions (clamped 3-256)
/// * `rings` - Number of latitudinal divisions (clamped 2-256)
///
/// Returns mesh handle (>0) on success, 0 on failure.
///
/// The sphere uses Format 5 (POS_UV_NORMAL) with equirectangular UV mapping:
/// - U wraps 0→1 around the equator (longitude)
/// - V maps 0→1 from north pole to south pole (latitude)
///
/// Perfect for skybox/environment mapping and earth-like textures.
///
/// **Init-only**: Must be called during `init()`.
fn sphere_uv(mut caller: Caller<'_, ZXGameContext>, radius: f32, segments: u32, rings: u32) -> u32 {
    if let Err(e) = check_init_only(&caller, "sphere_uv") {
        warn!("{}", e);
        return 0;
    }

    // Validate parameters
    if radius <= 0.0 {
        warn!("sphere_uv: radius must be > 0.0 (got {})", radius);
        return 0;
    }

    // Generate PACKED mesh data with UVs (clamping happens in procedural function)
    let mesh_data: MeshDataUV = procedural::generate_sphere_uv(radius, segments, rings);

    let vertex_count = mesh_data.vertices.len() / 16; // 16 bytes per POS_UV_NORMAL vertex
    let index_count = mesh_data.indices.len();

    // Allocate handle and queue mesh
    let state = &mut caller.data_mut().ffi;
    let handle = state.next_mesh_handle;
    state.next_mesh_handle += 1;

    state.pending_meshes_packed.push(PendingMeshPacked {
        handle,
        format: FORMAT_UV | FORMAT_NORMAL, // Base format (0-15, no FORMAT_PACKED flag)
        vertex_data: mesh_data.vertices,
        index_data: Some(mesh_data.indices),
    });

    info!(
        "sphere_uv: created mesh {} (radius={}, {}x{} segments, {} verts, {} indices, PACKED with UVs)",
        handle, radius, segments, rings, vertex_count, index_count
    );
    handle
}

/// Generate a subdivided plane mesh on the XZ plane with UV mapping
///
/// # Arguments
/// * `size_x` - Width along X axis
/// * `size_z` - Depth along Z axis
/// * `subdivisions_x` - Number of X subdivisions (clamped 1-256)
/// * `subdivisions_z` - Number of Z subdivisions (clamped 1-256)
///
/// Returns mesh handle (>0) on success, 0 on failure.
///
/// The plane uses Format 5 (POS_UV_NORMAL) with grid-based UV mapping:
/// - U maps 0→1 along X axis (left to right)
/// - V maps 0→1 along Z axis (front to back)
///
/// Perfect for ground planes, floors, and tiled textures.
///
/// **Init-only**: Must be called during `init()`.
fn plane_uv(
    mut caller: Caller<'_, ZXGameContext>,
    size_x: f32,
    size_z: f32,
    subdivisions_x: u32,
    subdivisions_z: u32,
) -> u32 {
    if let Err(e) = check_init_only(&caller, "plane_uv") {
        warn!("{}", e);
        return 0;
    }

    // Validate parameters
    if size_x <= 0.0 || size_z <= 0.0 {
        warn!("plane_uv: sizes must be > 0.0 (got {}, {})", size_x, size_z);
        return 0;
    }

    // Generate PACKED mesh data with UVs
    let mesh_data: MeshDataUV = procedural::generate_plane_uv(size_x, size_z, subdivisions_x, subdivisions_z);

    let vertex_count = mesh_data.vertices.len() / 16; // 16 bytes per POS_UV_NORMAL vertex
    let index_count = mesh_data.indices.len();

    // Allocate handle and queue mesh
    let state = &mut caller.data_mut().ffi;
    let handle = state.next_mesh_handle;
    state.next_mesh_handle += 1;

    state.pending_meshes_packed.push(PendingMeshPacked {
        handle,
        format: FORMAT_UV | FORMAT_NORMAL, // Base format (0-15, no FORMAT_PACKED flag)
        vertex_data: mesh_data.vertices,
        index_data: Some(mesh_data.indices),
    });

    info!(
        "plane_uv: created mesh {} ({}×{}, {}×{} subdivisions, {} verts, {} indices, PACKED with UVs)",
        handle, size_x, size_z, subdivisions_x, subdivisions_z, vertex_count, index_count
    );
    handle
}

/// Generate a cube mesh with box-unwrapped UV mapping
///
/// # Arguments
/// * `size_x` - Half-extent along X axis
/// * `size_y` - Half-extent along Y axis
/// * `size_z` - Half-extent along Z axis
///
/// Returns mesh handle (>0) on success, 0 on failure.
///
/// The cube uses Format 5 (POS_UV_NORMAL) with box-unwrapped UVs:
/// - Each face gets a quadrant in texture space (0-0.5, 0.5-1.0)
/// - Front/Back: U=[0.0-0.5], Top/Bottom: U=[0.5-1.0]
/// - +X/-X: V=[0.0-0.5], +Y/-Y: V=[0.5-1.0], +Z/-Z: mixed
///
/// Perfect for cubemaps and multi-texture cubes.
///
/// **Init-only**: Must be called during `init()`.
fn cube_uv(mut caller: Caller<'_, ZXGameContext>, size_x: f32, size_y: f32, size_z: f32) -> u32 {
    if let Err(e) = check_init_only(&caller, "cube_uv") {
        warn!("{}", e);
        return 0;
    }

    // Validate parameters
    if size_x <= 0.0 || size_y <= 0.0 || size_z <= 0.0 {
        warn!(
            "cube_uv: all sizes must be > 0.0 (got {}, {}, {})",
            size_x, size_y, size_z
        );
        return 0;
    }

    // Generate PACKED mesh data with UVs
    let mesh_data: MeshDataUV = procedural::generate_cube_uv(size_x, size_y, size_z);

    let vertex_count = mesh_data.vertices.len() / 16; // 16 bytes per POS_UV_NORMAL vertex
    let index_count = mesh_data.indices.len();

    // Allocate handle and queue mesh
    let state = &mut caller.data_mut().ffi;
    let handle = state.next_mesh_handle;
    state.next_mesh_handle += 1;

    state.pending_meshes_packed.push(PendingMeshPacked {
        handle,
        format: FORMAT_UV | FORMAT_NORMAL, // Base format (0-15, no FORMAT_PACKED flag)
        vertex_data: mesh_data.vertices,
        index_data: Some(mesh_data.indices),
    });

    info!(
        "cube_uv: created mesh {} ({}×{}×{}, {} verts, {} indices, PACKED with UVs)",
        handle,
        size_x * 2.0,
        size_y * 2.0,
        size_z * 2.0,
        vertex_count,
        index_count
    );
    handle
}

/// Generate a cylinder or cone mesh with cylindrical UV mapping
///
/// # Arguments
/// * `radius_bottom` - Bottom radius (>= 0.0)
/// * `radius_top` - Top radius (>= 0.0)
/// * `height` - Cylinder height
/// * `segments` - Number of radial divisions (clamped 3-256)
///
/// Returns mesh handle (>0) on success, 0 on failure.
///
/// The cylinder uses Format 5 (POS_UV_NORMAL) with cylindrical UV mapping:
/// - Body: U wraps 0→1 around circumference, V maps 0→1 along height
/// - Top cap: Radial mapping centered at U=0.5, V=0.75
/// - Bottom cap: Radial mapping centered at U=0.5, V=0.25
///
/// Perfect for barrel, can, pillar textures.
///
/// **Init-only**: Must be called during `init()`.
fn cylinder_uv(
    mut caller: Caller<'_, ZXGameContext>,
    radius_bottom: f32,
    radius_top: f32,
    height: f32,
    segments: u32,
) -> u32 {
    if let Err(e) = check_init_only(&caller, "cylinder_uv") {
        warn!("{}", e);
        return 0;
    }

    // Validate parameters
    if radius_bottom < 0.0 || radius_top < 0.0 {
        warn!(
            "cylinder_uv: radii must be >= 0.0 (got {}, {})",
            radius_bottom, radius_top
        );
        return 0;
    }

    if height <= 0.0 {
        warn!("cylinder_uv: height must be > 0.0 (got {})", height);
        return 0;
    }

    // Generate PACKED mesh data with UVs
    let mesh_data: MeshDataUV = procedural::generate_cylinder_uv(radius_bottom, radius_top, height, segments);

    let vertex_count = mesh_data.vertices.len() / 16; // 16 bytes per POS_UV_NORMAL vertex
    let index_count = mesh_data.indices.len();

    // Allocate handle and queue mesh
    let state = &mut caller.data_mut().ffi;
    let handle = state.next_mesh_handle;
    state.next_mesh_handle += 1;

    state.pending_meshes_packed.push(PendingMeshPacked {
        handle,
        format: FORMAT_UV | FORMAT_NORMAL, // Base format (0-15, no FORMAT_PACKED flag)
        vertex_data: mesh_data.vertices,
        index_data: Some(mesh_data.indices),
    });

    info!(
        "cylinder_uv: created mesh {} (radii={}/{}, height={}, {} segments, {} verts, {} indices, PACKED with UVs)",
        handle, radius_bottom, radius_top, height, segments, vertex_count, index_count
    );
    handle
}

/// Generate a torus (donut) mesh with wrapped UV mapping
///
/// # Arguments
/// * `major_radius` - Distance from torus center to tube center
/// * `minor_radius` - Tube radius
/// * `major_segments` - Segments around major circle (clamped 3-256)
/// * `minor_segments` - Segments around tube (clamped 3-256)
///
/// Returns mesh handle (>0) on success, 0 on failure.
///
/// The torus uses Format 5 (POS_UV_NORMAL) with wrapped UV mapping:
/// - U wraps 0→1 around the major circle (ring)
/// - V wraps 0→1 around the minor circle (tube)
///
/// Perfect for donut, ring, tire textures with repeating patterns.
///
/// **Init-only**: Must be called during `init()`.
fn torus_uv(
    mut caller: Caller<'_, ZXGameContext>,
    major_radius: f32,
    minor_radius: f32,
    major_segments: u32,
    minor_segments: u32,
) -> u32 {
    if let Err(e) = check_init_only(&caller, "torus_uv") {
        warn!("{}", e);
        return 0;
    }

    // Validate parameters
    if major_radius <= 0.0 || minor_radius <= 0.0 {
        warn!(
            "torus_uv: radii must be > 0.0 (got {}, {})",
            major_radius, minor_radius
        );
        return 0;
    }

    // Generate PACKED mesh data with UVs
    let mesh_data: MeshDataUV =
        procedural::generate_torus_uv(major_radius, minor_radius, major_segments, minor_segments);

    let vertex_count = mesh_data.vertices.len() / 16; // 16 bytes per POS_UV_NORMAL vertex
    let index_count = mesh_data.indices.len();

    // Allocate handle and queue mesh
    let state = &mut caller.data_mut().ffi;
    let handle = state.next_mesh_handle;
    state.next_mesh_handle += 1;

    state.pending_meshes_packed.push(PendingMeshPacked {
        handle,
        format: FORMAT_UV | FORMAT_NORMAL, // Base format (0-15, no FORMAT_PACKED flag)
        vertex_data: mesh_data.vertices,
        index_data: Some(mesh_data.indices),
    });

    info!(
        "torus_uv: created mesh {} (major={}, minor={}, {}×{} segments, {} verts, {} indices, PACKED with UVs)",
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

/// Generate a capsule (pill shape) mesh with hybrid UV mapping
///
/// # Arguments
/// * `radius` - Capsule radius
/// * `height` - Height of cylindrical section (>= 0.0)
/// * `segments` - Number of radial divisions (clamped 3-256)
/// * `rings` - Number of latitudinal divisions per hemisphere (clamped 1-128)
///
/// Returns mesh handle (>0) on success, 0 on failure.
///
/// The capsule uses Format 5 (POS_UV_NORMAL) with hybrid UV mapping:
/// - Bottom hemisphere: V=[0.0-0.25], equirectangular projection
/// - Cylindrical body: V=[0.25-0.75], wrapped around circumference
/// - Top hemisphere: V=[0.75-1.0], equirectangular projection
/// - U wraps 0→1 around circumference for all sections
///
/// Perfect for pill, barrel, character body textures.
///
/// **Init-only**: Must be called during `init()`.
fn capsule_uv(
    mut caller: Caller<'_, ZXGameContext>,
    radius: f32,
    height: f32,
    segments: u32,
    rings: u32,
) -> u32 {
    if let Err(e) = check_init_only(&caller, "capsule_uv") {
        warn!("{}", e);
        return 0;
    }

    // Validate parameters
    if radius <= 0.0 {
        warn!("capsule_uv: radius must be > 0.0 (got {})", radius);
        return 0;
    }

    if height < 0.0 {
        warn!("capsule_uv: height must be >= 0.0 (got {})", height);
        return 0;
    }

    // Generate PACKED mesh data with UVs
    let mesh_data: MeshDataUV = procedural::generate_capsule_uv(radius, height, segments, rings);

    let vertex_count = mesh_data.vertices.len() / 16; // 16 bytes per POS_UV_NORMAL vertex
    let index_count = mesh_data.indices.len();

    // Allocate handle and queue mesh
    let state = &mut caller.data_mut().ffi;
    let handle = state.next_mesh_handle;
    state.next_mesh_handle += 1;

    state.pending_meshes_packed.push(PendingMeshPacked {
        handle,
        format: FORMAT_UV | FORMAT_NORMAL, // Base format (0-15, no FORMAT_PACKED flag)
        vertex_data: mesh_data.vertices,
        index_data: Some(mesh_data.indices),
    });

    info!(
        "capsule_uv: created mesh {} (radius={}, height={}, {} segments, {} rings, {} verts, {} indices, PACKED with UVs)",
        handle, radius, height, segments, rings, vertex_count, index_count
    );
    handle
}
