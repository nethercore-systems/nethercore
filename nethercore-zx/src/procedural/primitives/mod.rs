//! Procedural mesh primitives (no UVs)
//!
//! Functions for generating common 3D primitives with normals but no UV coordinates.
//! These are suitable for solid-color rendering.

mod simple;
mod complex;

pub use simple::{generate_cube, generate_sphere, generate_plane, generate_torus};
pub use complex::{generate_cylinder, generate_capsule};
