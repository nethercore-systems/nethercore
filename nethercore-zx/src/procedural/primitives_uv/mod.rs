//! Procedural mesh primitives with UV coordinates
//!
//! Functions for generating common 3D primitives with normals and UV mapping.
//! These are suitable for textured rendering.

mod cube_torus;
mod cylinder_capsule;
mod sphere_plane;

// Re-export all public functions
pub use cube_torus::{generate_cube_uv, generate_torus_uv};
pub use cylinder_capsule::{generate_capsule_uv, generate_cylinder_uv};
pub use sphere_plane::{generate_plane_uv, generate_sphere_uv};
