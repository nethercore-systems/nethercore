//! Mesh combining utilities
//!
//! Functions for merging multiple meshes into a single mesh.

use nethercore_zx::procedural::UnpackedMesh;
use glam::Mat4;

/// Combine multiple meshes into one
///
/// Merges multiple meshes by concatenating their vertex and index data.
/// Index offsets are adjusted automatically. If any mesh has UVs, the output
/// will have UVs (meshes without UVs get zero UVs).
///
/// # Panics
/// Panics if the total vertex count exceeds `u16::MAX` (65,535).
///
/// # Example
/// ```no_run
/// use proc_gen::mesh::*;
///
/// let mesh1: UnpackedMesh = generate_cube(1.0, 1.0, 1.0);
/// let mesh2: UnpackedMesh = generate_sphere(0.5, 8, 4);
///
/// let combined = combine(&[&mesh1, &mesh2]);
/// ```
pub fn combine(meshes: &[&UnpackedMesh]) -> UnpackedMesh {
    let mut result = UnpackedMesh::new();

    // Calculate total sizes
    let total_vertices: usize = meshes.iter().map(|m| m.positions.len()).sum();
    let total_indices: usize = meshes.iter().map(|m| m.indices.len()).sum();
    let has_any_uvs = meshes.iter().any(|m| !m.uvs.is_empty());

    if total_vertices > u16::MAX as usize {
        panic!(
            "Combined vertex count {} exceeds u16::MAX ({}). \
             Consider splitting into multiple meshes.",
            total_vertices,
            u16::MAX
        );
    }

    // Pre-allocate
    result.positions.reserve(total_vertices);
    result.normals.reserve(total_vertices);
    result.indices.reserve(total_indices);
    if has_any_uvs {
        result.uvs.reserve(total_vertices);
    }

    // Merge meshes
    for mesh in meshes {
        if mesh.positions.is_empty() {
            continue;
        }

        let vertex_offset = result.positions.len() as u16;

        // Copy vertices
        result.positions.extend_from_slice(&mesh.positions);
        result.normals.extend_from_slice(&mesh.normals);

        // Handle UVs
        if has_any_uvs {
            if mesh.uvs.is_empty() {
                // Pad with zero UVs
                result.uvs.resize(result.positions.len(), [0.0, 0.0]);
            } else {
                result.uvs.extend_from_slice(&mesh.uvs);
            }
        }

        // Copy indices with offset
        for &idx in &mesh.indices {
            result.indices.push(vertex_offset + idx);
        }
    }

    result
}

/// Combine multiple meshes with per-mesh transforms
///
/// Applies a transformation matrix to each mesh before combining them.
/// Useful for building complex assemblies from simpler parts.
///
/// # Example
/// ```no_run
/// use proc_gen::mesh::*;
/// use glam::Mat4;
///
/// let wheel: UnpackedMesh = generate_cylinder(0.3, 0.3, 0.1, 16);
/// let body: UnpackedMesh = generate_cube(2.0, 1.0, 1.0);
///
/// let combined = combine_transformed(&[
///     (&wheel, Mat4::from_translation([0.8, -0.5, 0.5].into())),
///     (&wheel, Mat4::from_translation([-0.8, -0.5, 0.5].into())),
///     (&body, Mat4::IDENTITY),
/// ]);
/// ```
pub fn combine_transformed(meshes: &[(&UnpackedMesh, Mat4)]) -> UnpackedMesh {
    use super::modifiers::{Transform, MeshModifier};

    let transformed_meshes: Vec<UnpackedMesh> = meshes
        .iter()
        .map(|(mesh, matrix)| {
            let mut transformed = (*mesh).clone();
            Transform::from_matrix(*matrix).apply(&mut transformed);
            transformed
        })
        .collect();

    let mesh_refs: Vec<&UnpackedMesh> = transformed_meshes.iter().collect();
    combine(&mesh_refs)
}

#[cfg(test)]
mod tests {
    use super::*;
    use nethercore_zx::procedural::generate_cube;

    #[test]
    fn test_combine_empty() {
        let result = combine(&[]);
        assert_eq!(result.positions.len(), 0);
        assert_eq!(result.indices.len(), 0);
    }

    #[test]
    fn test_combine_index_offset() {
        let mesh1: UnpackedMesh = generate_cube(1.0, 1.0, 1.0);
        let mesh2: UnpackedMesh = generate_cube(0.5, 0.5, 0.5);

        let combined = combine(&[&mesh1, &mesh2]);

        assert_eq!(
            combined.positions.len(),
            mesh1.positions.len() + mesh2.positions.len()
        );
        assert_eq!(
            combined.indices.len(),
            mesh1.indices.len() + mesh2.indices.len()
        );

        // Verify indices are valid
        for &idx in &combined.indices {
            assert!((idx as usize) < combined.positions.len());
        }
    }

    #[test]
    fn test_combine_single_mesh() {
        let mesh: UnpackedMesh = generate_cube(1.0, 1.0, 1.0);
        let original_vert_count = mesh.positions.len();

        let combined = combine(&[&mesh]);

        assert_eq!(combined.positions.len(), original_vert_count);
    }

    #[test]
    fn test_combine_multiple_meshes() {
        let m1: UnpackedMesh = generate_cube(1.0, 1.0, 1.0);
        let m2: UnpackedMesh = generate_cube(0.5, 0.5, 0.5);
        let m3: UnpackedMesh = generate_cube(0.25, 0.25, 0.25);

        let combined = combine(&[&m1, &m2, &m3]);

        let expected_verts = m1.positions.len() + m2.positions.len() + m3.positions.len();
        let expected_indices = m1.indices.len() + m2.indices.len() + m3.indices.len();

        assert_eq!(combined.positions.len(), expected_verts);
        assert_eq!(combined.indices.len(), expected_indices);
    }

    #[test]
    fn test_combine_transformed() {
        let mesh: UnpackedMesh = generate_cube(1.0, 1.0, 1.0);

        let combined = combine_transformed(&[
            (&mesh, Mat4::from_translation([1.0, 0.0, 0.0].into())),
            (&mesh, Mat4::from_translation([-1.0, 0.0, 0.0].into())),
        ]);

        // Should have 2x the vertices
        assert_eq!(combined.positions.len(), mesh.positions.len() * 2);
    }
}
