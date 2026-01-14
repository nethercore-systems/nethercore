//! Procedural mesh generation FFI functions
//!
//! These functions generate common 3D primitives and return mesh handles.
//! All procedural meshes use packed vertex formats (f16, snorm16, unorm8) for memory efficiency.
//!
//! Three variants provided:
//! - Base functions (cube, sphere, etc.): Format 4 (POS_NORMAL) - uniform colors
//! - _uv variants (sphere_uv, etc.): Format 5 (POS_UV_NORMAL) - textured rendering
//! - _tangent variants (sphere_tangent, etc.): Format 21 (POS_UV_NORMAL_TANGENT) - normal mapped
//!
//! **IMPORTANT**: All procedural mesh functions are init-only. They queue meshes
//! for GPU upload, which must happen during init() to ensure deterministic rollback.

mod base_shapes;
mod uv_shapes;
mod tangent_shapes;

use anyhow::Result;
use wasmtime::Linker;

use super::ZXGameContext;

/// Register procedural mesh generation FFI functions
pub fn register(linker: &mut Linker<ZXGameContext>) -> Result<()> {
    // Base procedural shapes (FORMAT_NORMAL - solid colors)
    linker.func_wrap("env", "cube", base_shapes::cube)?;
    linker.func_wrap("env", "sphere", base_shapes::sphere)?;
    linker.func_wrap("env", "cylinder", base_shapes::cylinder)?;
    linker.func_wrap("env", "plane", base_shapes::plane)?;
    linker.func_wrap("env", "torus", base_shapes::torus)?;
    linker.func_wrap("env", "capsule", base_shapes::capsule)?;

    // UV-enabled variants (FORMAT_UV | FORMAT_NORMAL - textured)
    linker.func_wrap("env", "sphere_uv", uv_shapes::sphere_uv)?;
    linker.func_wrap("env", "plane_uv", uv_shapes::plane_uv)?;
    linker.func_wrap("env", "cube_uv", uv_shapes::cube_uv)?;
    linker.func_wrap("env", "cylinder_uv", uv_shapes::cylinder_uv)?;
    linker.func_wrap("env", "torus_uv", uv_shapes::torus_uv)?;
    linker.func_wrap("env", "capsule_uv", uv_shapes::capsule_uv)?;

    // Tangent-enabled variants (FORMAT_UV | FORMAT_NORMAL | FORMAT_TANGENT - normal mapped)
    linker.func_wrap("env", "sphere_tangent", tangent_shapes::sphere_tangent)?;
    linker.func_wrap("env", "plane_tangent", tangent_shapes::plane_tangent)?;
    linker.func_wrap("env", "cube_tangent", tangent_shapes::cube_tangent)?;
    linker.func_wrap("env", "torus_tangent", tangent_shapes::torus_tangent)?;

    Ok(())
}
