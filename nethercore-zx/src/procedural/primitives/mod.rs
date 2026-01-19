//! Procedural mesh primitives (no UVs)
//!
//! Functions for generating common 3D primitives with normals but no UV coordinates.
//! These are suitable for solid-color rendering.

mod complex;
mod simple;

pub use complex::{generate_capsule, generate_cylinder};
pub use simple::{generate_cube, generate_plane, generate_sphere, generate_torus};
