//! Shared mesh generation helpers
//!
//! Utilities to reduce boilerplate in mesh generators across all showcase games.

use proc_gen::mesh::{write_obj, UnpackedMesh};
use std::path::Path;

/// Write a mesh to an OBJ file with consistent logging.
///
/// This helper encapsulates the common pattern:
/// 1. Print generation start message
/// 2. Write OBJ file
/// 3. Print completion stats (verts, tris)
pub fn write_mesh(mesh: &UnpackedMesh, name: &str, output_dir: &Path) {
    println!("  Generating: {}.obj", name);

    let path = output_dir.join(format!("{}.obj", name));
    write_obj(mesh, &path, name).expect("Failed to write OBJ file");

    println!(
        "    -> {} ({} verts, {} tris)",
        path.display(),
        mesh.positions.len(),
        mesh.indices.len() / 3
    );
}
