//! Procedural mesh generation
//!
//! Functions for generating common 3D primitives with proper normals.
//!
//! All procedural meshes generate PACKED vertex data for memory efficiency:
//! - Format 4 (POS_NORMAL): 12 bytes/vertex (f16x4 + octahedral u32)
//! - Format 5 (POS_UV_NORMAL): 16 bytes/vertex (f16x4 + unorm16x2 + octahedral u32)

mod primitives;
mod primitives_uv;
mod types;

// Re-export types (used by generator return types, fields accessed externally)
#[allow(unused_imports)] // Types accessed via return type inference
pub use types::{MeshData, MeshDataUV};

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graphics::unpack_octahedral_u32;
    use half::f16;

    // Helper functions to unpack packed vertex data for testing
    // Packed format: Position (f16x4, 8 bytes) + Normal (octahedral u32, 4 bytes) = 12 bytes/vertex

    /// Unpack position from packed vertex data
    fn unpack_position(data: &[u8], vertex_idx: usize) -> [f32; 3] {
        let base = vertex_idx * 12; // 12 bytes per vertex (POS_NORMAL packed)
        let pos_bytes = &data[base..base + 8]; // f16x4 position (8 bytes)
        let pos: &[f16; 4] = bytemuck::from_bytes(&pos_bytes[0..8]);
        [pos[0].to_f32(), pos[1].to_f32(), pos[2].to_f32()]
    }

    /// Unpack normal from packed vertex data (octahedral encoded)
    fn unpack_normal(data: &[u8], vertex_idx: usize) -> [f32; 3] {
        let base = vertex_idx * 12 + 8; // Skip position (8 bytes)
        let norm_bytes = &data[base..base + 4]; // octahedral u32 (4 bytes)
        let packed =
            u32::from_le_bytes([norm_bytes[0], norm_bytes[1], norm_bytes[2], norm_bytes[3]]);
        let normal = unpack_octahedral_u32(packed);
        [normal.x, normal.y, normal.z]
    }

    #[test]
    fn test_cube_counts() {
        let mesh = generate_cube(1.0, 1.0, 1.0);
        assert_eq!(mesh.vertices.len(), 24 * 12); // 24 vertices × 12 bytes (POS_NORMAL packed)
        assert_eq!(mesh.indices.len(), 36); // 6 faces × 2 triangles × 3
    }

    #[test]
    fn test_sphere_counts() {
        let mesh = generate_sphere(1.0, 16, 8);
        let expected_verts = (8 + 1) * 16; // (rings + 1) × segments
        let expected_indices = 8 * 16 * 6; // rings × segments × 6
        assert_eq!(mesh.vertices.len(), expected_verts * 12); // 12 bytes per vertex (packed)
        assert_eq!(mesh.indices.len(), expected_indices);
    }

    #[test]
    fn test_plane_counts() {
        let mesh = generate_plane(2.0, 2.0, 4, 4);
        let expected_verts = (4 + 1) * (4 + 1); // (subdivisions_x + 1) × (subdivisions_z + 1)
        let expected_indices = 4 * 4 * 6; // subdivisions_x × subdivisions_z × 6
        assert_eq!(mesh.vertices.len(), expected_verts * 12); // 12 bytes per vertex (packed)
        assert_eq!(mesh.indices.len(), expected_indices);
    }

    #[test]
    fn test_normals_normalized() {
        let mesh = generate_sphere(1.0, 16, 8);
        let vertex_count = mesh.vertices.len() / 12; // 12 bytes per vertex

        // Check every normal is unit length
        for i in 0..vertex_count {
            let normal = unpack_normal(&mesh.vertices, i);
            let length =
                (normal[0] * normal[0] + normal[1] * normal[1] + normal[2] * normal[2]).sqrt();
            assert!(
                (length - 1.0).abs() < 0.02, // Increased tolerance for packed format
                "Normal not normalized: {}",
                length
            );
        }
    }

    #[test]
    fn test_cube_flat_normals() {
        let mesh = generate_cube(1.0, 1.0, 1.0);

        // First 4 vertices (front face) should all have normal (0, 0, 1)
        for i in 0..4 {
            let normal = unpack_normal(&mesh.vertices, i);
            assert!((normal[0] - 0.0).abs() < 0.02); // nx = 0
            assert!((normal[1] - 0.0).abs() < 0.02); // ny = 0
            assert!((normal[2] - 1.0).abs() < 0.02); // nz = 1
        }
    }

    #[test]
    fn test_invalid_params_safe() {
        // Should not panic, should clamp
        let _ = generate_cube(0.0, 1.0, 1.0);
        let _ = generate_sphere(-1.0, 3, 2);
        let _ = generate_cylinder(1.0, 1.0, 1.0, 2);
        let _ = generate_plane(1.0, 1.0, 0, 0);
        let _ = generate_torus(0.5, 1.0, 3, 3);
        let _ = generate_capsule(1.0, 0.0, 3, 1);
    }

    // ========================================================================
    // Winding Order Verification Tests
    // ========================================================================
    //
    // These tests verify that all triangles have correct CCW winding order
    // (counter-clockwise when viewed from outside the mesh).
    //
    // For each triangle, we:
    // 1. Compute face normal via cross product: N = (v1-v0) × (v2-v0)
    // 2. Compute centroid of the triangle
    // 3. Verify normal points outward (away from mesh center)
    //
    // A positive dot product between the normal and the outward direction
    // confirms CCW winding.

    /// Helper: extract vertex position from mesh at given index
    fn get_vertex_pos(mesh: &MeshData, vertex_idx: u16) -> [f32; 3] {
        unpack_position(&mesh.vertices, vertex_idx as usize)
    }

    /// Helper: compute cross product of two 3D vectors
    fn cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
        [
            a[1] * b[2] - a[2] * b[1],
            a[2] * b[0] - a[0] * b[2],
            a[0] * b[1] - a[1] * b[0],
        ]
    }

    /// Helper: compute dot product of two 3D vectors
    fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
        a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
    }

    /// Helper: subtract two vectors (a - b)
    fn sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
        [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
    }

    /// Helper: normalize a vector
    fn normalize(v: [f32; 3]) -> [f32; 3] {
        let len = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
        if len < 0.0001 {
            return [0.0, 1.0, 0.0]; // fallback
        }
        [v[0] / len, v[1] / len, v[2] / len]
    }

    /// Helper: compute magnitude of a vector
    fn magnitude(v: [f32; 3]) -> f32 {
        (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt()
    }

    /// Helper: compute face normal from triangle indices (CCW winding gives outward normal)
    /// Returns None for degenerate triangles (zero-area)
    fn compute_face_normal(mesh: &MeshData, i0: u16, i1: u16, i2: u16) -> Option<[f32; 3]> {
        let v0 = get_vertex_pos(mesh, i0);
        let v1 = get_vertex_pos(mesh, i1);
        let v2 = get_vertex_pos(mesh, i2);
        let edge1 = sub(v1, v0);
        let edge2 = sub(v2, v0);
        let cross_prod = cross(edge1, edge2);
        let len = magnitude(cross_prod);

        // Skip degenerate triangles (zero-area)
        if len < 0.0001 {
            return None;
        }

        Some([
            cross_prod[0] / len,
            cross_prod[1] / len,
            cross_prod[2] / len,
        ])
    }

    /// Helper: compute triangle centroid
    fn triangle_centroid(mesh: &MeshData, i0: u16, i1: u16, i2: u16) -> [f32; 3] {
        let v0 = get_vertex_pos(mesh, i0);
        let v1 = get_vertex_pos(mesh, i1);
        let v2 = get_vertex_pos(mesh, i2);
        [
            (v0[0] + v1[0] + v2[0]) / 3.0,
            (v0[1] + v1[1] + v2[1]) / 3.0,
            (v0[2] + v1[2] + v2[2]) / 3.0,
        ]
    }

    /// Verify all triangles have outward-facing normals (for meshes centered at origin)
    /// Returns (passed, failed, skipped, total) counts
    /// Skipped triangles are degenerate (zero-area)
    fn verify_outward_normals(mesh: &MeshData, center: [f32; 3]) -> (usize, usize, usize, usize) {
        let mut passed = 0;
        let mut failed = 0;
        let mut skipped = 0;

        for tri in mesh.indices.chunks(3) {
            let i0 = tri[0];
            let i1 = tri[1];
            let i2 = tri[2];

            let face_normal = match compute_face_normal(mesh, i0, i1, i2) {
                Some(n) => n,
                None => {
                    skipped += 1;
                    continue;
                }
            };
            let centroid = triangle_centroid(mesh, i0, i1, i2);

            // Direction from mesh center to triangle centroid
            let outward_dir = normalize(sub(centroid, center));

            // Face normal should point in same general direction as outward
            let alignment = dot(face_normal, outward_dir);

            if alignment > 0.0 {
                passed += 1;
            } else {
                failed += 1;
            }
        }

        (passed, failed, skipped, passed + failed + skipped)
    }

    #[test]
    fn test_winding_cube() {
        let mesh = generate_cube(2.0, 2.0, 2.0);
        let (_passed, failed, skipped, total) = verify_outward_normals(&mesh, [0.0, 0.0, 0.0]);

        assert_eq!(
            failed, 0,
            "Cube winding: {}/{} triangles have incorrect winding (normals point inward)",
            failed, total
        );
        assert_eq!(skipped, 0, "Cube should have no degenerate triangles");
        assert_eq!(total, 12, "Cube should have 12 triangles (6 faces × 2)");
    }

    #[test]
    fn test_winding_sphere() {
        let mesh = generate_sphere(1.0, 32, 16);
        let (passed, failed, skipped, _total) = verify_outward_normals(&mesh, [0.0, 0.0, 0.0]);

        // Sphere has degenerate triangles at poles where all vertices share the same position
        // (top pole: ring 0, bottom pole: ring N)
        assert_eq!(
            failed, 0,
            "Sphere winding: {}/{} non-degenerate triangles have incorrect winding (skipped {} degenerate)",
            failed, passed + failed, skipped
        );
        assert!(passed > 0, "Sphere should have valid triangles");
        // 32 segments × 2 poles = 64 degenerate triangles at poles
        assert!(
            skipped <= 64,
            "Sphere should have at most 64 degenerate pole triangles, got {}",
            skipped
        );
    }

    #[test]
    fn test_winding_plane() {
        let mesh = generate_plane(2.0, 2.0, 4, 4);

        // For plane, all normals should point +Y (upward)
        let mut correct = 0;
        let mut wrong = 0;
        let mut skipped = 0;

        for tri in mesh.indices.chunks(3) {
            let face_normal = match compute_face_normal(&mesh, tri[0], tri[1], tri[2]) {
                Some(n) => n,
                None => {
                    skipped += 1;
                    continue;
                }
            };
            // Should point +Y
            if face_normal[1] > 0.9 {
                correct += 1;
            } else {
                wrong += 1;
            }
        }

        assert_eq!(skipped, 0, "Plane should have no degenerate triangles");
        assert_eq!(
            wrong,
            0,
            "Plane winding: {}/{} triangles have incorrect winding (normals should point +Y)",
            wrong,
            correct + wrong
        );
    }

    #[test]
    fn test_winding_cylinder() {
        let mesh = generate_cylinder(1.0, 1.0, 2.0, 32);

        // Cylinder is centered at origin with height along Y
        // Body normals should point radially outward (XZ plane)
        // Top cap normals should point +Y
        // Bottom cap normals should point -Y

        let mut body_correct = 0;
        let mut body_wrong = 0;
        let mut cap_correct = 0;
        let mut cap_wrong = 0;
        let mut skipped = 0;

        for tri in mesh.indices.chunks(3) {
            let face_normal = match compute_face_normal(&mesh, tri[0], tri[1], tri[2]) {
                Some(n) => n,
                None => {
                    skipped += 1;
                    continue;
                }
            };
            let centroid = triangle_centroid(&mesh, tri[0], tri[1], tri[2]);

            // Determine if this is a cap or body triangle based on Y position
            let y = centroid[1];
            let is_top_cap = y > 0.9;
            let is_bottom_cap = y < -0.9;

            if is_top_cap {
                // Top cap should point +Y
                if face_normal[1] > 0.9 {
                    cap_correct += 1;
                } else {
                    cap_wrong += 1;
                }
            } else if is_bottom_cap {
                // Bottom cap should point -Y
                if face_normal[1] < -0.9 {
                    cap_correct += 1;
                } else {
                    cap_wrong += 1;
                }
            } else {
                // Body - normal should point radially outward in XZ plane
                let radial_dir = normalize([centroid[0], 0.0, centroid[2]]);
                let alignment = dot(face_normal, radial_dir);
                if alignment > 0.0 {
                    body_correct += 1;
                } else {
                    body_wrong += 1;
                }
            }
        }

        assert_eq!(
            body_wrong,
            0,
            "Cylinder body winding: {}/{} triangles have incorrect winding",
            body_wrong,
            body_correct + body_wrong
        );
        assert_eq!(
            cap_wrong,
            0,
            "Cylinder cap winding: {}/{} triangles have incorrect winding",
            cap_wrong,
            cap_correct + cap_wrong
        );
        // Cylinder caps have center vertices, so some triangles at center may be degenerate
        assert!(
            skipped <= 2,
            "Cylinder should have at most 2 degenerate triangles, got {}",
            skipped
        );
    }

    #[test]
    fn test_winding_torus() {
        // Use proper torus proportions: major > minor to avoid self-intersection
        let major_radius = 1.0; // Distance from torus center to tube center
        let minor_radius = 0.3; // Tube radius
        let mesh = generate_torus(major_radius, minor_radius, 32, 16);

        // Torus: major radius in XZ plane, minor radius forms the tube
        // For each triangle, the normal should point away from the tube center
        let mut correct = 0;
        let mut wrong = 0;
        let mut skipped = 0;

        for tri in mesh.indices.chunks(3) {
            let face_normal = match compute_face_normal(&mesh, tri[0], tri[1], tri[2]) {
                Some(n) => n,
                None => {
                    skipped += 1;
                    continue;
                }
            };
            let centroid = triangle_centroid(&mesh, tri[0], tri[1], tri[2]);

            // Find the closest point on the major circle (tube center)
            let xz_len = (centroid[0] * centroid[0] + centroid[2] * centroid[2]).sqrt();
            if xz_len > 0.01 {
                let tube_center = [
                    centroid[0] * major_radius / xz_len,
                    0.0,
                    centroid[2] * major_radius / xz_len,
                ];

                // Outward direction is from tube center to triangle centroid
                let outward = normalize(sub(centroid, tube_center));
                let alignment = dot(face_normal, outward);

                if alignment > 0.0 {
                    correct += 1;
                } else {
                    wrong += 1;
                }
            } else {
                correct += 1; // Edge case, assume correct
            }
        }

        assert_eq!(skipped, 0, "Torus should have no degenerate triangles");
        assert_eq!(
            wrong,
            0,
            "Torus winding: {}/{} triangles have incorrect winding",
            wrong,
            correct + wrong
        );
    }

    #[test]
    fn test_winding_capsule() {
        let mesh = generate_capsule(0.5, 2.0, 32, 16);

        // Capsule: cylinder body with hemisphere caps
        // All normals should point radially outward from the capsule axis

        // For capsule, we use a more specific test
        let mut correct = 0;
        let mut wrong = 0;
        let mut skipped = 0;

        for tri in mesh.indices.chunks(3) {
            let face_normal = match compute_face_normal(&mesh, tri[0], tri[1], tri[2]) {
                Some(n) => n,
                None => {
                    skipped += 1;
                    continue;
                }
            };
            let centroid = triangle_centroid(&mesh, tri[0], tri[1], tri[2]);

            // For capsule along Y axis, outward is radial from Y axis
            let radial_xz = (centroid[0] * centroid[0] + centroid[2] * centroid[2]).sqrt();

            if radial_xz > 0.01 {
                // Not on the axis - check radial direction
                let radial_dir = normalize([centroid[0], 0.0, centroid[2]]);
                let horizontal_component =
                    face_normal[0] * radial_dir[0] + face_normal[2] * radial_dir[2];

                // Most of the normal's horizontal component should point outward
                // (Allow for caps where vertical component dominates)
                let normal_horizontal_len =
                    (face_normal[0] * face_normal[0] + face_normal[2] * face_normal[2]).sqrt();

                if normal_horizontal_len > 0.3 {
                    // Significant horizontal component
                    if horizontal_component > 0.0 {
                        correct += 1;
                    } else {
                        wrong += 1;
                    }
                } else {
                    // Mostly vertical (cap area) - check Y direction matches position
                    if (centroid[1] > 0.0 && face_normal[1] > 0.0)
                        || (centroid[1] < 0.0 && face_normal[1] < 0.0)
                        || centroid[1].abs() < 0.1
                    {
                        correct += 1;
                    } else {
                        wrong += 1;
                    }
                }
            } else {
                // On the axis (pole) - normal should point up or down based on Y
                if (centroid[1] > 0.0 && face_normal[1] > 0.0)
                    || (centroid[1] < 0.0 && face_normal[1] < 0.0)
                {
                    correct += 1;
                } else {
                    wrong += 1;
                }
            }
        }

        // Capsule has degenerate triangles at hemisphere poles
        // 32 segments × 2 poles = 64 max degenerate triangles
        assert!(
            skipped <= 64,
            "Capsule should have at most 64 degenerate pole triangles, got {}",
            skipped
        );
        assert_eq!(
            wrong,
            0,
            "Capsule winding: {}/{} triangles have incorrect winding (skipped {} degenerate)",
            wrong,
            correct + wrong,
            skipped
        );
    }

    /// Test that stored vertex normals point outward for sphere
    /// This verifies the actual normal data, not just the winding order
    #[test]
    fn test_sphere_vertex_normals_point_outward() {
        let mesh = generate_sphere(1.0, 16, 8);
        let vertex_count = mesh.vertices.len() / 12; // 12 bytes per vertex (POS_NORMAL packed)

        // For a unit sphere centered at origin, the vertex normal should equal
        // the normalized vertex position (pointing outward from center)
        for i in 0..vertex_count {
            let pos = unpack_position(&mesh.vertices, i);
            let normal = unpack_normal(&mesh.vertices, i);

            let px = pos[0];
            let py = pos[1];
            let pz = pos[2];
            let nx = normal[0];
            let ny = normal[1];
            let nz = normal[2];

            // Compute expected normal (normalized position for unit sphere at origin)
            let pos_len = (px * px + py * py + pz * pz).sqrt();
            if pos_len < 0.001 {
                continue; // Skip degenerate vertices at poles
            }

            let expected_nx = px / pos_len;
            let expected_ny = py / pos_len;
            let expected_nz = pz / pos_len;

            // Verify stored normal matches expected (points outward)
            // Increased tolerance for packed format (f16/snorm16)
            assert!(
                (nx - expected_nx).abs() < 0.02,
                "Sphere vertex normal X mismatch at vertex {}: stored={}, expected={}",
                i,
                nx,
                expected_nx
            );
            assert!(
                (ny - expected_ny).abs() < 0.02,
                "Sphere vertex normal Y mismatch at vertex {}: stored={}, expected={}",
                i,
                ny,
                expected_ny
            );
            assert!(
                (nz - expected_nz).abs() < 0.02,
                "Sphere vertex normal Z mismatch at vertex {}: stored={}, expected={}",
                i,
                nz,
                expected_nz
            );

            // Also verify dot product is positive (normal points same direction as position)
            let dot_product = px * nx + py * ny + pz * nz;
            assert!(
                dot_product > 0.0,
                "Sphere vertex {} normal points inward! dot(pos, normal) = {}",
                i,
                dot_product
            );
        }
    }

    /// Test that cube vertex normals match face directions
    #[test]
    fn test_cube_vertex_normals_match_faces() {
        let mesh = generate_cube(1.0, 1.0, 1.0);

        // Each face has 4 vertices, all with the same normal
        // Faces in order: +Z, -Z, +Y, -Y, +X, -X
        let expected_normals: [[f32; 3]; 6] = [
            [0.0, 0.0, 1.0],  // Front (+Z)
            [0.0, 0.0, -1.0], // Back (-Z)
            [0.0, 1.0, 0.0],  // Top (+Y)
            [0.0, -1.0, 0.0], // Bottom (-Y)
            [1.0, 0.0, 0.0],  // Right (+X)
            [-1.0, 0.0, 0.0], // Left (-X)
        ];

        for face in 0..6 {
            let expected = expected_normals[face];
            for vert in 0..4 {
                let vertex_idx = face * 4 + vert;
                let normal = unpack_normal(&mesh.vertices, vertex_idx);

                // Increased tolerance for packed format (snorm16)
                assert!(
                    (normal[0] - expected[0]).abs() < 0.02,
                    "Cube face {} vertex {} normal X mismatch: {} vs expected {}",
                    face,
                    vert,
                    normal[0],
                    expected[0]
                );
                assert!(
                    (normal[1] - expected[1]).abs() < 0.02,
                    "Cube face {} vertex {} normal Y mismatch: {} vs expected {}",
                    face,
                    vert,
                    normal[1],
                    expected[1]
                );
                assert!(
                    (normal[2] - expected[2]).abs() < 0.02,
                    "Cube face {} vertex {} normal Z mismatch: {} vs expected {}",
                    face,
                    vert,
                    normal[2],
                    expected[2]
                );
            }
        }
    }

    /// Test that verifies ALL procedural shapes at once with a simple outward test
    #[test]
    fn test_winding_all_shapes_simple() {
        // This is a comprehensive sanity check using the simple center-based test
        // Note: This test uses center-to-centroid which doesn't work perfectly for torus,
        // so torus is excluded here (it has a dedicated test with tube-center logic)

        let shapes: Vec<(&str, MeshData)> = vec![
            ("cube", generate_cube(2.0, 2.0, 2.0)),
            ("sphere", generate_sphere(1.0, 16, 8)),
            ("cylinder", generate_cylinder(1.0, 1.0, 2.0, 16)),
            ("capsule", generate_capsule(0.5, 2.0, 16, 8)),
        ];

        for (name, mesh) in shapes {
            let (passed, failed, skipped, _total) = verify_outward_normals(&mesh, [0.0, 0.0, 0.0]);

            // Degenerate triangles (at poles) are expected and skipped
            // Only non-degenerate triangles should be verified
            let non_degenerate = passed + failed;
            if non_degenerate > 0 {
                assert_eq!(
                    failed, 0,
                    "{}: {}/{} non-degenerate triangles have incorrect winding (skipped {} degenerate)",
                    name, failed, non_degenerate, skipped
                );
            }
        }
    }
}
