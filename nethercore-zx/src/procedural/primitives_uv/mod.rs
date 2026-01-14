//! Procedural mesh primitives with UV coordinates
//!
//! Functions for generating common 3D primitives with normals and UV mapping.
//! These are suitable for textured rendering.

mod sphere_plane;
mod cube_torus;
mod cylinder_capsule;

// Re-export all public functions
pub use sphere_plane::{generate_sphere_uv, generate_plane_uv};
pub use cube_torus::{generate_cube_uv, generate_torus_uv};
pub use cylinder_capsule::{generate_cylinder_uv, generate_capsule_uv};
