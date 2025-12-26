//! Mesh generation and modification

// Re-export types from nethercore-zx
pub use nethercore_zx::procedural::{
    MeshBuilder, MeshBuilderUV, MeshData, MeshDataUV, UnpackedMesh,
};

// Re-export primitives
pub use nethercore_zx::procedural::{
    generate_cube, generate_sphere, generate_cylinder,
    generate_plane, generate_torus, generate_capsule,
    generate_cube_uv, generate_sphere_uv, generate_cylinder_uv,
    generate_plane_uv, generate_torus_uv, generate_capsule_uv,
};

// Re-export OBJ export
pub use nethercore_zx::procedural::write_obj;

// Local modules
pub mod modifiers;
pub mod combine;

// Convenience re-exports
pub use modifiers::{
    MeshModifier, MeshApply, Transform, Mirror, SmoothNormals, FlatNormals,
    Subdivide, Chamfer, Axis,
};
pub use combine::{combine, combine_transformed};
