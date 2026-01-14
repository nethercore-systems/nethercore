//! Procedural mesh generation
//!
//! Functions for generating common 3D primitives with proper normals.
//!
//! All procedural meshes generate PACKED vertex data for memory efficiency:
//! - Format 4 (POS_NORMAL): 12 bytes/vertex (f16x4 + octahedral u32)
//! - Format 5 (POS_UV_NORMAL): 16 bytes/vertex (f16x4 + unorm16x2 + octahedral u32)

mod export;
mod primitives;
mod primitives_tangent;
mod primitives_uv;
mod types;

#[cfg(test)]
mod tests;

// Re-export types (used by generator return types, fields accessed externally)
#[allow(unused_imports)] // Types accessed via return type inference
pub use types::{
    MeshBuilder, MeshBuilderTangent, MeshBuilderUV, MeshData, MeshDataTangent, MeshDataUV,
    UnpackedMesh,
};

// Re-export OBJ export
pub use export::write_obj;

// Re-export non-UV primitives
pub use primitives::{
    generate_capsule, generate_cube, generate_cylinder, generate_plane, generate_sphere,
    generate_torus,
};

// Re-export UV primitives
pub use primitives_uv::{
    generate_capsule_uv, generate_cube_uv, generate_cylinder_uv, generate_plane_uv,
    generate_sphere_uv, generate_torus_uv,
};

// Re-export tangent primitives (for normal mapping)
pub use primitives_tangent::{
    generate_cube_tangent, generate_plane_tangent, generate_sphere_tangent, generate_torus_tangent,
};
