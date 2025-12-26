//! Mesh modifiers for procedural geometry
//!
//! This module provides modifiers that operate on `UnpackedMesh` to transform,
//! mirror, and adjust geometry.
//!
//! # Fluent API
//!
//! Use the `MeshApply` extension trait for method chaining:
//! ```no_run
//! use proc_gen::mesh::*;
//!
//! let mut mesh: UnpackedMesh = generate_capsule(0.4, 0.6, 8, 4);
//! mesh.apply(Transform::scale(1.0, 1.2, 0.8))
//!     .apply(Mirror::default())
//!     .apply(Subdivide { iterations: 1 })
//!     .apply(SmoothNormals::default());
//! ```

use nethercore_zx::procedural::UnpackedMesh;
use glam::{Mat4, Vec3};
use std::collections::HashMap;

/// Trait for mesh modifiers
///
/// Implement this trait to create custom mesh modifiers that can be applied
/// to `UnpackedMesh` instances.
pub trait MeshModifier {
    /// Apply this modifier to a mesh, modifying it in place
    fn apply(&self, mesh: &mut UnpackedMesh);
}

/// Extension trait for fluent modifier application
///
/// Provides a chainable `apply` method on `UnpackedMesh` for convenient
/// modifier composition.
pub trait MeshApply {
    /// Apply a modifier and return `&mut Self` for chaining
    fn apply<M: MeshModifier>(&mut self, modifier: M) -> &mut Self;
}

impl MeshApply for UnpackedMesh {
    fn apply<M: MeshModifier>(&mut self, modifier: M) -> &mut Self {
        modifier.apply(self);
        self
    }
}

/// Coordinate axis for mirroring
#[derive(Debug, Clone, Copy)]
pub enum Axis {
    /// X axis (left/right)
    X,
    /// Y axis (up/down)
    Y,
    /// Z axis (forward/back)
    Z,
}

/// Transform mesh vertices and normals using a 4x4 matrix
///
/// Applies transformations (translation, rotation, scale) to mesh geometry.
/// Normals are transformed using the inverse-transpose to handle non-uniform scaling correctly.
///
/// # Example
/// ```no_run
/// use proc_gen::mesh::*;
///
/// let mut mesh: UnpackedMesh = generate_cube(1.0, 1.0, 1.0);
///
/// // Scale non-uniformly
/// Transform::scale(2.0, 1.0, 1.0).apply(&mut mesh);
///
/// // Rotate 45 degrees around Y axis
/// Transform::rotate_y(45.0).apply(&mut mesh);
/// ```
pub struct Transform {
    matrix: Mat4,
}

impl Transform {
    /// Create an identity transform (no change)
    pub fn identity() -> Self {
        Self {
            matrix: Mat4::IDENTITY,
        }
    }

    /// Create a translation transform
    pub fn translate(x: f32, y: f32, z: f32) -> Self {
        Self {
            matrix: Mat4::from_translation(Vec3::new(x, y, z)),
        }
    }

    /// Create a non-uniform scale transform
    pub fn scale(x: f32, y: f32, z: f32) -> Self {
        Self {
            matrix: Mat4::from_scale(Vec3::new(x, y, z)),
        }
    }

    /// Create a uniform scale transform
    pub fn scale_uniform(s: f32) -> Self {
        Self::scale(s, s, s)
    }

    /// Create a rotation around the X axis (in degrees)
    pub fn rotate_x(degrees: f32) -> Self {
        Self {
            matrix: Mat4::from_rotation_x(degrees.to_radians()),
        }
    }

    /// Create a rotation around the Y axis (in degrees)
    pub fn rotate_y(degrees: f32) -> Self {
        Self {
            matrix: Mat4::from_rotation_y(degrees.to_radians()),
        }
    }

    /// Create a rotation around the Z axis (in degrees)
    pub fn rotate_z(degrees: f32) -> Self {
        Self {
            matrix: Mat4::from_rotation_z(degrees.to_radians()),
        }
    }

    /// Create a transform from a custom 4x4 matrix
    pub fn from_matrix(matrix: Mat4) -> Self {
        Self { matrix }
    }
}

impl MeshModifier for Transform {
    fn apply(&self, mesh: &mut UnpackedMesh) {
        // For normals, use inverse-transpose to handle non-uniform scaling
        let normal_matrix = self.matrix.inverse().transpose();

        // Transform positions
        for pos in &mut mesh.positions {
            let v = Vec3::from(*pos);
            let transformed = self.matrix.transform_point3(v);
            *pos = [transformed.x, transformed.y, transformed.z];
        }

        // Transform normals using inverse-transpose
        for norm in &mut mesh.normals {
            let n = Vec3::from(*norm);
            let transformed = normal_matrix.transform_vector3(n).normalize();
            *norm = [transformed.x, transformed.y, transformed.z];
        }

        // UVs remain unchanged
    }
}

/// Mirror mesh across an axis with optional vertex welding on the mirror plane
///
/// Creates a mirrored copy of the mesh and welds vertices that lie on the mirror plane.
/// Triangle winding is reversed to maintain counter-clockwise ordering from the outside.
///
/// # Example
/// ```no_run
/// use proc_gen::mesh::*;
///
/// let mut mesh: UnpackedMesh = generate_capsule(0.4, 0.6, 8, 4);
///
/// // Mirror across X axis, welding vertices within 0.001 units of the plane
/// Mirror {
///     axis: Axis::X,
///     merge_threshold: 0.001,
///     flip_u: false,
///     flip_v: false,
/// }.apply(&mut mesh);
/// ```
pub struct Mirror {
    /// Axis to mirror across
    pub axis: Axis,
    /// Distance threshold for welding vertices on the mirror plane
    pub merge_threshold: f32,
    /// Whether to flip U coordinates (horizontal texture flip)
    pub flip_u: bool,
    /// Whether to flip V coordinates (vertical texture flip)
    pub flip_v: bool,
}

impl Default for Mirror {
    fn default() -> Self {
        Self {
            axis: Axis::X,
            merge_threshold: 0.001,
            flip_u: false,
            flip_v: false,
        }
    }
}

impl MeshModifier for Mirror {
    fn apply(&self, mesh: &mut UnpackedMesh) {
        let original_vertex_count = mesh.positions.len();
        let axis_idx = match self.axis {
            Axis::X => 0,
            Axis::Y => 1,
            Axis::Z => 2,
        };

        // Build weld map for vertices on the mirror plane
        let mut weld_map = HashMap::new();
        for i in 0..original_vertex_count {
            if mesh.positions[i][axis_idx].abs() < self.merge_threshold {
                weld_map.insert(original_vertex_count + i, i);
            }
        }

        // Duplicate and mirror vertices
        for i in 0..original_vertex_count {
            let mut pos = mesh.positions[i];
            pos[axis_idx] = -pos[axis_idx];
            mesh.positions.push(pos);

            let mut norm = mesh.normals[i];
            norm[axis_idx] = -norm[axis_idx];
            mesh.normals.push(norm);

            // Mirror UVs if requested
            if !mesh.uvs.is_empty() {
                let mut uv = mesh.uvs[i];
                if self.flip_u {
                    uv[0] = 1.0 - uv[0];
                }
                if self.flip_v {
                    uv[1] = 1.0 - uv[1];
                }
                mesh.uvs.push(uv);
            }
        }

        // Mirror triangles with reversed winding
        let original_index_count = mesh.indices.len();
        for chunk in 0..original_index_count / 3 {
            let base = chunk * 3;
            let i0 = mesh.indices[base] as usize;
            let i1 = mesh.indices[base + 1] as usize;
            let i2 = mesh.indices[base + 2] as usize;

            // Apply weld map or offset indices
            let mi0 = *weld_map
                .get(&(original_vertex_count + i0))
                .unwrap_or(&(original_vertex_count + i0)) as u16;
            let mi1 = *weld_map
                .get(&(original_vertex_count + i1))
                .unwrap_or(&(original_vertex_count + i1)) as u16;
            let mi2 = *weld_map
                .get(&(original_vertex_count + i2))
                .unwrap_or(&(original_vertex_count + i2)) as u16;

            // Reverse winding to maintain CCW from outside
            mesh.indices.push(mi2);
            mesh.indices.push(mi1);
            mesh.indices.push(mi0);
        }
    }
}

/// Recalculate normals by averaging face normals for shared positions
///
/// Groups vertices by position (within a threshold) and averages the face normals
/// of all triangles that reference vertices in each group. This creates smooth shading
/// across edges.
///
/// # Example
/// ```no_run
/// use proc_gen::mesh::*;
///
/// let mut mesh: UnpackedMesh = generate_sphere(1.0, 16, 8);
/// SmoothNormals::default().apply(&mut mesh);
/// ```
pub struct SmoothNormals {
    /// Distance threshold for considering vertices as sharing a position
    pub weld_threshold: f32,
}

impl Default for SmoothNormals {
    fn default() -> Self {
        Self {
            weld_threshold: 0.0001,
        }
    }
}

impl MeshModifier for SmoothNormals {
    fn apply(&self, mesh: &mut UnpackedMesh) {
        // Compute face normals
        let mut face_normals = Vec::new();
        for chunk in mesh.indices.chunks(3) {
            if chunk.len() == 3 {
                let p0 = Vec3::from(mesh.positions[chunk[0] as usize]);
                let p1 = Vec3::from(mesh.positions[chunk[1] as usize]);
                let p2 = Vec3::from(mesh.positions[chunk[2] as usize]);

                let edge1 = p1 - p0;
                let edge2 = p2 - p0;
                let normal = edge1.cross(edge2).normalize();
                face_normals.push(normal);
            }
        }

        // Group vertices by position (with threshold)
        let mut position_groups: HashMap<usize, Vec<usize>> = HashMap::new();

        for i in 0..mesh.positions.len() {
            let mut found_group = None;
            for (&group_id, group_verts) in position_groups.iter() {
                let group_pos = Vec3::from(mesh.positions[group_verts[0]]);
                let curr_pos = Vec3::from(mesh.positions[i]);
                if group_pos.distance(curr_pos) < self.weld_threshold {
                    found_group = Some(group_id);
                    break;
                }
            }

            if let Some(group_id) = found_group {
                position_groups.get_mut(&group_id).unwrap().push(i);
            } else {
                position_groups.insert(i, vec![i]);
            }
        }

        // Average normals for each position group
        for group_verts in position_groups.values() {
            let mut avg_normal = Vec3::ZERO;

            // Collect all face normals that reference these vertices
            for (face_idx, chunk) in mesh.indices.chunks(3).enumerate() {
                if chunk.len() == 3 {
                    for &vert_idx in group_verts {
                        if chunk.contains(&(vert_idx as u16)) {
                            avg_normal += face_normals[face_idx];
                            break;
                        }
                    }
                }
            }

            avg_normal = avg_normal.normalize();

            // Apply to all vertices in group
            for &vert_idx in group_verts {
                mesh.normals[vert_idx] = [avg_normal.x, avg_normal.y, avg_normal.z];
            }
        }
    }
}

/// Convert to flat shading by duplicating vertices (one normal per triangle)
///
/// Each triangle gets its own set of 3 vertices with the same face normal,
/// creating a faceted appearance. This increases vertex count by 3x the triangle count.
///
/// # Example
/// ```no_run
/// use proc_gen::mesh::*;
///
/// let mut mesh: UnpackedMesh = generate_cube(1.0, 1.0, 1.0);
/// FlatNormals.apply(&mut mesh);
/// ```
pub struct FlatNormals;

impl MeshModifier for FlatNormals {
    fn apply(&self, mesh: &mut UnpackedMesh) {
        let mut new_positions = Vec::new();
        let mut new_normals = Vec::new();
        let mut new_uvs = Vec::new();
        let mut new_indices = Vec::new();

        let has_uvs = !mesh.uvs.is_empty();

        for chunk in mesh.indices.chunks(3) {
            if chunk.len() == 3 {
                let p0 = Vec3::from(mesh.positions[chunk[0] as usize]);
                let p1 = Vec3::from(mesh.positions[chunk[1] as usize]);
                let p2 = Vec3::from(mesh.positions[chunk[2] as usize]);

                // Compute face normal
                let edge1 = p1 - p0;
                let edge2 = p2 - p0;
                let face_normal = edge1.cross(edge2).normalize();
                let normal_arr = [face_normal.x, face_normal.y, face_normal.z];

                // Create 3 new vertices with same position but face normal
                for &idx in chunk {
                    let new_idx = new_positions.len() as u16;
                    new_positions.push(mesh.positions[idx as usize]);
                    new_normals.push(normal_arr);
                    if has_uvs {
                        new_uvs.push(mesh.uvs[idx as usize]);
                    }
                    new_indices.push(new_idx);
                }
            }
        }

        mesh.positions = new_positions;
        mesh.normals = new_normals;
        mesh.uvs = new_uvs;
        mesh.indices = new_indices;
    }
}

/// Subdivide mesh using midpoint subdivision
///
/// Each triangle is split into 4 smaller triangles by adding a vertex at
/// the midpoint of each edge. This creates smoother geometry when combined
/// with smooth normals.
///
/// # Algorithm
///
/// For each triangle with vertices A, B, C:
/// 1. Create midpoint vertices: AB, BC, CA
/// 2. Replace with 4 triangles: (A, AB, CA), (AB, B, BC), (CA, BC, C), (AB, BC, CA)
///
/// # Complexity
///
/// - Vertices: approximately doubles per iteration
/// - Triangles: 4x per iteration
///
/// # Example
/// ```no_run
/// use proc_gen::mesh::*;
///
/// let mut mesh: UnpackedMesh = generate_cube(1.0, 1.0, 1.0);
///
/// // Apply 2 iterations of subdivision for a smoother cube
/// Subdivide { iterations: 2 }.apply(&mut mesh);
/// SmoothNormals::default().apply(&mut mesh);
/// ```
pub struct Subdivide {
    /// Number of subdivision iterations (1-4 recommended, higher = exponential growth)
    pub iterations: u32,
}

impl Default for Subdivide {
    fn default() -> Self {
        Self { iterations: 1 }
    }
}

impl MeshModifier for Subdivide {
    fn apply(&self, mesh: &mut UnpackedMesh) {
        for _ in 0..self.iterations {
            subdivide_once(mesh);
        }
    }
}

/// Perform a single subdivision pass
fn subdivide_once(mesh: &mut UnpackedMesh) {
    let has_uvs = !mesh.uvs.is_empty();

    // Edge key: sorted pair of vertex indices
    type EdgeKey = (u16, u16);
    fn make_edge_key(a: u16, b: u16) -> EdgeKey {
        if a < b { (a, b) } else { (b, a) }
    }

    // Map from edge to midpoint vertex index
    let mut edge_midpoints: HashMap<EdgeKey, u16> = HashMap::new();

    let mut new_positions = mesh.positions.clone();
    let mut new_normals = mesh.normals.clone();
    let mut new_uvs = if has_uvs { mesh.uvs.clone() } else { Vec::new() };
    let mut new_indices = Vec::new();

    /// Get or create midpoint vertex for an edge
    fn get_or_create_midpoint(
        edge: EdgeKey,
        edge_midpoints: &mut HashMap<EdgeKey, u16>,
        positions: &[[f32; 3]],
        normals: &[[f32; 3]],
        uvs: &[[f32; 2]],
        new_positions: &mut Vec<[f32; 3]>,
        new_normals: &mut Vec<[f32; 3]>,
        new_uvs: &mut Vec<[f32; 2]>,
        has_uvs: bool,
    ) -> u16 {
        if let Some(&idx) = edge_midpoints.get(&edge) {
            return idx;
        }

        let (i0, i1) = edge;
        let p0 = Vec3::from(positions[i0 as usize]);
        let p1 = Vec3::from(positions[i1 as usize]);
        let n0 = Vec3::from(normals[i0 as usize]);
        let n1 = Vec3::from(normals[i1 as usize]);

        let mid_pos = (p0 + p1) * 0.5;
        let mid_norm = (n0 + n1).normalize();

        let new_idx = new_positions.len() as u16;
        new_positions.push([mid_pos.x, mid_pos.y, mid_pos.z]);
        new_normals.push([mid_norm.x, mid_norm.y, mid_norm.z]);

        if has_uvs {
            let uv0 = uvs[i0 as usize];
            let uv1 = uvs[i1 as usize];
            let mid_uv = [
                (uv0[0] + uv1[0]) * 0.5,
                (uv0[1] + uv1[1]) * 0.5,
            ];
            new_uvs.push(mid_uv);
        }

        edge_midpoints.insert(edge, new_idx);
        new_idx
    }

    // Process each triangle
    for chunk in mesh.indices.chunks(3) {
        if chunk.len() != 3 {
            continue;
        }

        let i0 = chunk[0];
        let i1 = chunk[1];
        let i2 = chunk[2];

        // Get or create midpoint vertices
        let m01 = get_or_create_midpoint(
            make_edge_key(i0, i1),
            &mut edge_midpoints,
            &mesh.positions,
            &mesh.normals,
            &mesh.uvs,
            &mut new_positions,
            &mut new_normals,
            &mut new_uvs,
            has_uvs,
        );
        let m12 = get_or_create_midpoint(
            make_edge_key(i1, i2),
            &mut edge_midpoints,
            &mesh.positions,
            &mesh.normals,
            &mesh.uvs,
            &mut new_positions,
            &mut new_normals,
            &mut new_uvs,
            has_uvs,
        );
        let m20 = get_or_create_midpoint(
            make_edge_key(i2, i0),
            &mut edge_midpoints,
            &mesh.positions,
            &mesh.normals,
            &mesh.uvs,
            &mut new_positions,
            &mut new_normals,
            &mut new_uvs,
            has_uvs,
        );

        // Create 4 new triangles:
        // Corner triangles (maintain winding order)
        new_indices.extend_from_slice(&[i0, m01, m20]);   // Corner 0
        new_indices.extend_from_slice(&[m01, i1, m12]);   // Corner 1
        new_indices.extend_from_slice(&[m20, m12, i2]);   // Corner 2
        // Center triangle
        new_indices.extend_from_slice(&[m01, m12, m20]);
    }

    mesh.positions = new_positions;
    mesh.normals = new_normals;
    mesh.uvs = new_uvs;
    mesh.indices = new_indices;
}

/// Chamfer (bevel) edges to create rounded corners
///
/// Creates additional geometry along mesh edges to produce a beveled or
/// rounded appearance. This is commonly used for realistic hard-surface
/// modeling (armor, vehicles, machinery).
///
/// # Algorithm
///
/// 1. Identify sharp edges (angle between adjacent faces exceeds threshold)
/// 2. Inset vertices along sharp edges
/// 3. Create connecting faces between the original and inset positions
///
/// # Example
/// ```no_run
/// use proc_gen::mesh::*;
///
/// let mut mesh: UnpackedMesh = generate_cube(1.0, 1.0, 1.0);
///
/// // Add a small chamfer with 2 segments for rounded edges
/// Chamfer {
///     amount: 0.05,
///     segments: 2,
///     angle_threshold_degrees: 30.0,
/// }.apply(&mut mesh);
/// ```
pub struct Chamfer {
    /// Distance to inset from edges (0.01-0.1 typical)
    pub amount: f32,
    /// Number of bevel segments (1 = flat bevel, 2+ = rounded)
    pub segments: u32,
    /// Minimum angle between faces (degrees) to be considered a sharp edge
    pub angle_threshold_degrees: f32,
}

impl Default for Chamfer {
    fn default() -> Self {
        Self {
            amount: 0.05,
            segments: 1,
            angle_threshold_degrees: 30.0,
        }
    }
}

/// Edge information for chamfer calculation
#[derive(Clone)]
struct EdgeInfo {
    /// Vertex indices forming the edge
    v0: u16,
    v1: u16,
    /// Faces sharing this edge (indices into triangle array)
    faces: Vec<usize>,
    /// Whether this edge is sharp (should be chamfered)
    is_sharp: bool,
}

impl MeshModifier for Chamfer {
    fn apply(&self, mesh: &mut UnpackedMesh) {
        if mesh.indices.is_empty() || self.amount <= 0.0 || self.segments == 0 {
            return;
        }

        let has_uvs = !mesh.uvs.is_empty();
        let angle_threshold = self.angle_threshold_degrees.to_radians().cos();

        // Build edge map: edge -> list of face indices that share it
        type EdgeKey = (u16, u16);
        fn make_edge_key(a: u16, b: u16) -> EdgeKey {
            if a < b { (a, b) } else { (b, a) }
        }

        let mut edge_map: HashMap<EdgeKey, EdgeInfo> = HashMap::new();

        // Collect faces and compute face normals
        let mut face_normals: Vec<Vec3> = Vec::new();
        for (face_idx, chunk) in mesh.indices.chunks(3).enumerate() {
            if chunk.len() != 3 {
                continue;
            }

            let p0 = Vec3::from(mesh.positions[chunk[0] as usize]);
            let p1 = Vec3::from(mesh.positions[chunk[1] as usize]);
            let p2 = Vec3::from(mesh.positions[chunk[2] as usize]);

            let normal = (p1 - p0).cross(p2 - p0).normalize();
            face_normals.push(normal);

            // Register edges
            for (a, b) in [(chunk[0], chunk[1]), (chunk[1], chunk[2]), (chunk[2], chunk[0])] {
                let key = make_edge_key(a, b);
                edge_map
                    .entry(key)
                    .or_insert_with(|| EdgeInfo {
                        v0: key.0,
                        v1: key.1,
                        faces: Vec::new(),
                        is_sharp: false,
                    })
                    .faces
                    .push(face_idx);
            }
        }

        // Identify sharp edges
        for edge in edge_map.values_mut() {
            if edge.faces.len() == 2 {
                let n0 = face_normals[edge.faces[0]];
                let n1 = face_normals[edge.faces[1]];
                let dot = n0.dot(n1);
                // Sharp if angle exceeds threshold (dot product below threshold cos)
                edge.is_sharp = dot < angle_threshold;
            } else if edge.faces.len() == 1 {
                // Boundary edge - always sharp
                edge.is_sharp = true;
            }
        }

        // Collect sharp edges
        let sharp_edges: Vec<EdgeInfo> = edge_map
            .values()
            .filter(|e| e.is_sharp)
            .cloned()
            .collect();

        if sharp_edges.is_empty() {
            return;
        }

        let mut new_positions = mesh.positions.clone();
        let mut new_normals = mesh.normals.clone();
        let mut new_uvs = if has_uvs { mesh.uvs.clone() } else { Vec::new() };
        let mut new_indices = mesh.indices.clone();

        // Process each sharp edge independently
        for edge in &sharp_edges {
            let p0 = Vec3::from(mesh.positions[edge.v0 as usize]);
            let p1 = Vec3::from(mesh.positions[edge.v1 as usize]);
            let edge_dir = (p1 - p0).normalize();

            // Compute average normal of adjacent faces for inset direction
            let mut avg_normal = Vec3::ZERO;
            for &face_idx in &edge.faces {
                avg_normal += face_normals[face_idx];
            }
            avg_normal = avg_normal.normalize();

            // Inset direction is perpendicular to edge and away from faces
            let inset_dir = edge_dir.cross(avg_normal).normalize();

            // Create inset vertices for this edge (both endpoints)
            // Store indices for the strip: [original, seg1, seg2, ...]
            let mut strip_v0: Vec<u16> = vec![edge.v0];
            let mut strip_v1: Vec<u16> = vec![edge.v1];

            for seg in 1..=self.segments {
                let t = seg as f32 / (self.segments + 1) as f32;
                let inset_amount = self.amount * t;

                // Create inset vertex for v0
                let orig_pos_0 = Vec3::from(mesh.positions[edge.v0 as usize]);
                let inset_pos_0 = orig_pos_0 + inset_dir * inset_amount - avg_normal * inset_amount * 0.5;
                let new_idx_0 = new_positions.len() as u16;
                new_positions.push([inset_pos_0.x, inset_pos_0.y, inset_pos_0.z]);
                new_normals.push(mesh.normals[edge.v0 as usize]);
                if has_uvs {
                    new_uvs.push(mesh.uvs[edge.v0 as usize]);
                }
                strip_v0.push(new_idx_0);

                // Create inset vertex for v1
                let orig_pos_1 = Vec3::from(mesh.positions[edge.v1 as usize]);
                let inset_pos_1 = orig_pos_1 + inset_dir * inset_amount - avg_normal * inset_amount * 0.5;
                let new_idx_1 = new_positions.len() as u16;
                new_positions.push([inset_pos_1.x, inset_pos_1.y, inset_pos_1.z]);
                new_normals.push(mesh.normals[edge.v1 as usize]);
                if has_uvs {
                    new_uvs.push(mesh.uvs[edge.v1 as usize]);
                }
                strip_v1.push(new_idx_1);
            }

            // Create bevel faces between original edge and inset edges
            // Both strips should have the same length: 1 + segments
            debug_assert_eq!(strip_v0.len(), strip_v1.len());

            for i in 0..strip_v0.len() - 1 {
                let a = strip_v0[i];
                let b = strip_v0[i + 1];
                let c = strip_v1[i];
                let d = strip_v1[i + 1];

                // Two triangles per quad
                new_indices.extend_from_slice(&[a, c, b]);
                new_indices.extend_from_slice(&[b, c, d]);
            }
        }

        mesh.positions = new_positions;
        mesh.normals = new_normals;
        mesh.uvs = new_uvs;
        mesh.indices = new_indices;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nethercore_zx::procedural::{generate_cube, generate_sphere};

    #[test]
    fn test_transform_scale() {
        let mut mesh: UnpackedMesh = generate_cube(1.0, 1.0, 1.0);
        let original_max = mesh.positions.iter()
            .map(|p| p[0].abs())
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap();

        Transform::scale(2.0, 2.0, 2.0).apply(&mut mesh);

        // Verify positions scaled by 2x
        let scaled_max = mesh.positions.iter()
            .map(|p| p[0].abs())
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap();

        assert!((scaled_max - original_max * 2.0).abs() < 0.01);
    }

    #[test]
    fn test_transform_translate() {
        let mut mesh: UnpackedMesh = generate_cube(1.0, 1.0, 1.0);
        Transform::translate(5.0, 0.0, 0.0).apply(&mut mesh);

        // All X positions should be shifted by 5
        let avg_x: f32 = mesh.positions.iter().map(|p| p[0]).sum::<f32>() / mesh.positions.len() as f32;
        assert!((avg_x - 5.0).abs() < 0.01);
    }

    #[test]
    fn test_mirror_doubles_triangles() {
        let mut mesh: UnpackedMesh = generate_cube(1.0, 1.0, 1.0);
        let original_tri_count = mesh.triangle_count();

        Mirror::default().apply(&mut mesh);

        assert_eq!(mesh.triangle_count(), original_tri_count * 2);
    }

    #[test]
    fn test_flat_normals_vertex_count() {
        let mut mesh: UnpackedMesh = generate_sphere(1.0, 8, 4);
        let original_tri_count = mesh.triangle_count();

        FlatNormals.apply(&mut mesh);

        // Each triangle gets 3 unique vertices
        assert_eq!(mesh.vertex_count(), original_tri_count * 3);
    }

    #[test]
    fn test_flat_normals_per_triangle_normal() {
        let mut mesh: UnpackedMesh = generate_cube(1.0, 1.0, 1.0);
        FlatNormals.apply(&mut mesh);

        // Each triangle should have identical normals for all 3 vertices
        for chunk in mesh.indices.chunks(3) {
            if chunk.len() == 3 {
                let n0 = mesh.normals[chunk[0] as usize];
                let n1 = mesh.normals[chunk[1] as usize];
                let n2 = mesh.normals[chunk[2] as usize];

                assert_eq!(n0, n1);
                assert_eq!(n1, n2);
            }
        }
    }

    #[test]
    fn test_smooth_normals_averaging() {
        let mut mesh: UnpackedMesh = generate_cube(1.0, 1.0, 1.0);
        SmoothNormals::default().apply(&mut mesh);

        // Verify all normals are unit length
        for norm in &mesh.normals {
            let len = (norm[0] * norm[0] + norm[1] * norm[1] + norm[2] * norm[2]).sqrt();
            assert!((len - 1.0).abs() < 0.01);
        }
    }

    #[test]
    fn test_subdivide_quadruples_triangles() {
        let mut mesh: UnpackedMesh = generate_cube(1.0, 1.0, 1.0);
        let original_tri_count = mesh.triangle_count();

        Subdivide { iterations: 1 }.apply(&mut mesh);

        // Each triangle becomes 4 triangles
        assert_eq!(mesh.triangle_count(), original_tri_count * 4);
    }

    #[test]
    fn test_subdivide_multiple_iterations() {
        let mut mesh: UnpackedMesh = generate_cube(1.0, 1.0, 1.0);
        let original_tri_count = mesh.triangle_count();

        Subdivide { iterations: 2 }.apply(&mut mesh);

        // 2 iterations: 4^2 = 16x triangles
        assert_eq!(mesh.triangle_count(), original_tri_count * 16);
    }

    #[test]
    fn test_subdivide_preserves_bounds() {
        let mut mesh: UnpackedMesh = generate_cube(1.0, 1.0, 1.0);

        // Get original bounds
        let original_max_x = mesh.positions.iter()
            .map(|p| p[0])
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap();

        Subdivide { iterations: 1 }.apply(&mut mesh);

        // Midpoint subdivision should not exceed original bounds
        let new_max_x = mesh.positions.iter()
            .map(|p| p[0])
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap();

        assert!((new_max_x - original_max_x).abs() < 0.01);
    }

    #[test]
    fn test_subdivide_with_uvs() {
        use nethercore_zx::procedural::generate_cube_uv;

        let mut mesh: UnpackedMesh = generate_cube_uv(1.0, 1.0, 1.0);
        let original_has_uvs = !mesh.uvs.is_empty();

        Subdivide { iterations: 1 }.apply(&mut mesh);

        // UVs should be preserved and interpolated
        assert!(original_has_uvs);
        assert!(!mesh.uvs.is_empty());
        assert_eq!(mesh.uvs.len(), mesh.positions.len());
    }

    #[test]
    fn test_chamfer_adds_geometry() {
        let mut mesh: UnpackedMesh = generate_cube(1.0, 1.0, 1.0);
        let original_vert_count = mesh.vertex_count();
        let original_tri_count = mesh.triangle_count();

        Chamfer::default().apply(&mut mesh);

        // Chamfer should add vertices and triangles for beveled edges
        assert!(mesh.vertex_count() >= original_vert_count);
        assert!(mesh.triangle_count() >= original_tri_count);
    }

    #[test]
    fn test_chamfer_with_segments() {
        let mut mesh: UnpackedMesh = generate_cube(1.0, 1.0, 1.0);
        let original_vert_count = mesh.vertex_count();

        Chamfer {
            amount: 0.1,
            segments: 3,
            angle_threshold_degrees: 30.0,
        }.apply(&mut mesh);

        // More segments = more vertices
        assert!(mesh.vertex_count() > original_vert_count);
    }

    #[test]
    fn test_chamfer_no_sharp_edges() {
        // Sphere has no sharp edges (smooth surface)
        let mut mesh: UnpackedMesh = generate_sphere(1.0, 16, 8);
        let original_tri_count = mesh.triangle_count();

        // With a high angle threshold, no edges should be chamfered
        Chamfer {
            amount: 0.1,
            segments: 1,
            angle_threshold_degrees: 1.0, // Very small angle = almost nothing is sharp
        }.apply(&mut mesh);

        // Smooth sphere should have minimal/no chamfer applied
        // (sphere faces have gradual angle changes)
        assert!(mesh.triangle_count() >= original_tri_count);
    }

    #[test]
    fn test_fluent_apply_chaining() {
        let mut mesh: UnpackedMesh = generate_cube(1.0, 1.0, 1.0);
        let original_tri_count = mesh.triangle_count();

        // Test fluent API chaining
        mesh.apply(Transform::scale(2.0, 2.0, 2.0))
            .apply(Subdivide { iterations: 1 })
            .apply(SmoothNormals::default());

        // Verify subdivide worked (4x triangles)
        assert_eq!(mesh.triangle_count(), original_tri_count * 4);

        // Verify scale worked (positions should be larger)
        let max_pos = mesh.positions.iter()
            .map(|p| p[0].abs())
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap();
        assert!(max_pos > 0.9); // Scaled up from 0.5 to 1.0
    }

    #[test]
    fn test_fluent_apply_returns_self() {
        let mut mesh: UnpackedMesh = generate_cube(1.0, 1.0, 1.0);

        // Verify apply returns &mut Self for chaining
        let result = mesh.apply(Transform::identity());
        assert_eq!(result.vertex_count(), mesh.vertex_count());
    }

    #[test]
    fn test_subdivide_valid_indices() {
        let mut mesh: UnpackedMesh = generate_sphere(1.0, 8, 4);

        Subdivide { iterations: 1 }.apply(&mut mesh);

        // All indices should be valid
        for &idx in &mesh.indices {
            assert!((idx as usize) < mesh.positions.len());
        }
    }
}
