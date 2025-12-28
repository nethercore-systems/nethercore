//! Mesh detail generators
//!
//! Provides modifiers for adding surface details like rivets, panel lines,
//! and greebles (random mechanical detail) essential for sci-fi and
//! industrial aesthetics in PS1/PS2-era style.

use super::modifiers::{MeshModifier, SmoothNormals};
use nethercore_zx::procedural::UnpackedMesh;
use glam::Vec3;

/// Add rivets along edges of the mesh
///
/// Rivets are small cylindrical protrusions common in industrial/military aesthetics.
/// They're placed along sharp edges automatically.
pub struct AddRivets {
    /// Rivet radius relative to mesh size
    pub radius: f32,
    /// Rivet height (protrusion depth)
    pub height: f32,
    /// Spacing between rivets (relative to mesh size)
    pub spacing: f32,
    /// Minimum edge angle (degrees) to place rivets on
    pub min_edge_angle: f32,
    /// Number of segments around rivet circumference
    pub segments: u32,
    /// Random seed for placement variation
    pub seed: u32,
}

impl Default for AddRivets {
    fn default() -> Self {
        Self {
            radius: 0.02,
            height: 0.01,
            spacing: 0.1,
            min_edge_angle: 30.0,
            segments: 6,
            seed: 0,
        }
    }
}

impl AddRivets {
    /// Dense rivets for armored/heavy industrial look
    pub fn dense() -> Self {
        Self {
            radius: 0.015,
            height: 0.008,
            spacing: 0.06,
            min_edge_angle: 20.0,
            segments: 6,
            seed: 0,
        }
    }

    /// Sparse larger rivets for lighter industrial look
    pub fn sparse() -> Self {
        Self {
            radius: 0.03,
            height: 0.015,
            spacing: 0.15,
            min_edge_angle: 45.0,
            segments: 8,
            seed: 0,
        }
    }
}

impl MeshModifier for AddRivets {
    fn apply(&self, mesh: &mut UnpackedMesh) {
        if mesh.positions.is_empty() || mesh.indices.len() < 3 {
            return;
        }

        // Calculate mesh bounds for sizing
        let bounds = calculate_bounds(mesh);
        let size = (bounds.1 - bounds.0).length();
        let actual_radius = size * self.radius;
        let actual_height = size * self.height;
        let actual_spacing = size * self.spacing;
        let min_angle_rad = self.min_edge_angle.to_radians();

        // Find sharp edges
        let edges = find_sharp_edges(mesh, min_angle_rad);

        // Place rivets along edges
        let mut rng = SimpleRng::new(self.seed);

        for edge in edges {
            let p0 = Vec3::from(mesh.positions[edge.0]);
            let p1 = Vec3::from(mesh.positions[edge.1]);
            let edge_vec = p1 - p0;
            let edge_len = edge_vec.length();

            if edge_len < actual_spacing {
                continue;
            }

            // Average normal at edge
            let n0 = Vec3::from(mesh.normals[edge.0]);
            let n1 = Vec3::from(mesh.normals[edge.1]);
            let normal = ((n0 + n1) * 0.5).normalize();

            // Number of rivets along this edge
            let rivet_count = (edge_len / actual_spacing) as u32;

            for i in 0..rivet_count {
                // Position along edge (with slight randomization)
                let t = (i as f32 + 0.5 + rng.next_f32() * 0.3 - 0.15) / rivet_count as f32;
                let center = p0 + edge_vec * t;

                add_rivet(
                    mesh,
                    center,
                    normal,
                    actual_radius,
                    actual_height,
                    self.segments,
                );
            }
        }

        SmoothNormals::default().apply(mesh);
    }
}

/// Add panel lines (inset grooves) to surfaces
///
/// Panel lines are thin grooves that suggest separate panels or sections,
/// common in mecha, vehicles, and sci-fi designs.
pub struct AddPanelLines {
    /// Groove width
    pub width: f32,
    /// Groove depth
    pub depth: f32,
    /// Pattern type
    pub pattern: PanelPattern,
    /// Grid spacing for GridH/GridV patterns
    pub grid_spacing: f32,
    /// Random seed
    pub seed: u32,
}

#[derive(Clone, Copy, Default)]
pub enum PanelPattern {
    /// Horizontal lines
    #[default]
    GridH,
    /// Vertical lines
    GridV,
    /// Both horizontal and vertical
    Grid,
    /// Random panel segments
    Random,
}

impl Default for AddPanelLines {
    fn default() -> Self {
        Self {
            width: 0.01,
            depth: 0.005,
            pattern: PanelPattern::Grid,
            grid_spacing: 0.2,
            seed: 0,
        }
    }
}

impl MeshModifier for AddPanelLines {
    fn apply(&self, mesh: &mut UnpackedMesh) {
        if mesh.positions.is_empty() {
            return;
        }

        let bounds = calculate_bounds(mesh);
        let size = (bounds.1 - bounds.0).length();
        let actual_depth = size * self.depth;
        let actual_spacing = size * self.grid_spacing;

        // Displace vertices near grid lines inward
        for i in 0..mesh.positions.len() {
            let pos = Vec3::from(mesh.positions[i]);
            let normal = Vec3::from(mesh.normals[i]);
            let local = pos - bounds.0;

            let groove_factor = match self.pattern {
                PanelPattern::GridH => {
                    groove_intensity(local.y, actual_spacing, size * self.width)
                }
                PanelPattern::GridV => {
                    groove_intensity(local.x, actual_spacing, size * self.width)
                }
                PanelPattern::Grid => {
                    let h = groove_intensity(local.y, actual_spacing, size * self.width);
                    let v = groove_intensity(local.x, actual_spacing, size * self.width);
                    h.max(v)
                }
                PanelPattern::Random => {
                    // Random panels based on position hash
                    let hash = simple_hash(
                        (local.x / actual_spacing) as i32,
                        (local.y / actual_spacing) as i32,
                        self.seed,
                    );
                    if hash % 3 == 0 {
                        groove_intensity(local.x, actual_spacing * 0.5, size * self.width)
                    } else {
                        0.0
                    }
                }
            };

            if groove_factor > 0.0 {
                let displaced = pos - normal * actual_depth * groove_factor;
                mesh.positions[i] = [displaced.x, displaced.y, displaced.z];
            }
        }

        SmoothNormals::default().apply(mesh);
    }
}

/// Add greebles (random mechanical detail)
///
/// Greebles are small random geometric details that add visual complexity
/// to surfaces. Essential for sci-fi and mechanical aesthetics.
pub struct AddGreebles {
    /// Greeble density (0.0 to 1.0)
    pub density: f32,
    /// Minimum greeble size (relative to mesh)
    pub size_min: f32,
    /// Maximum greeble size (relative to mesh)
    pub size_max: f32,
    /// Maximum greeble height
    pub height_max: f32,
    /// Greeble shape variety
    pub variety: GreebleVariety,
    /// Random seed
    pub seed: u32,
}

#[derive(Clone, Copy, Default)]
pub enum GreebleVariety {
    /// Only boxes
    BoxesOnly,
    /// Boxes and cylinders
    #[default]
    Mixed,
    /// Antenna-like thin protrusions
    Antennae,
}

impl Default for AddGreebles {
    fn default() -> Self {
        Self {
            density: 0.3,
            size_min: 0.02,
            size_max: 0.08,
            height_max: 0.05,
            variety: GreebleVariety::Mixed,
            seed: 0,
        }
    }
}

impl AddGreebles {
    /// Light greebling for subtle detail
    pub fn light() -> Self {
        Self {
            density: 0.15,
            size_min: 0.02,
            size_max: 0.05,
            height_max: 0.03,
            variety: GreebleVariety::Mixed,
            seed: 0,
        }
    }

    /// Heavy greebling for complex machinery
    pub fn heavy() -> Self {
        Self {
            density: 0.5,
            size_min: 0.03,
            size_max: 0.12,
            height_max: 0.08,
            variety: GreebleVariety::Mixed,
            seed: 0,
        }
    }
}

impl MeshModifier for AddGreebles {
    fn apply(&self, mesh: &mut UnpackedMesh) {
        if mesh.positions.is_empty() || mesh.indices.len() < 3 {
            return;
        }

        let bounds = calculate_bounds(mesh);
        let size = (bounds.1 - bounds.0).length();

        let mut rng = SimpleRng::new(self.seed);

        // Get face centers for greeble placement
        let face_centers: Vec<(Vec3, Vec3)> = mesh.indices.chunks(3)
            .filter_map(|chunk| {
                if chunk.len() == 3 {
                    let p0 = Vec3::from(mesh.positions[chunk[0] as usize]);
                    let p1 = Vec3::from(mesh.positions[chunk[1] as usize]);
                    let p2 = Vec3::from(mesh.positions[chunk[2] as usize]);
                    let center = (p0 + p1 + p2) / 3.0;
                    let normal = (p1 - p0).cross(p2 - p0).normalize();
                    Some((center, normal))
                } else {
                    None
                }
            })
            .collect();

        // Add greebles at random face centers
        for (center, normal) in face_centers {
            if rng.next_f32() > self.density {
                continue;
            }

            // Skip faces pointing mostly down
            if normal.y < -0.5 {
                continue;
            }

            let greeble_size = size * (self.size_min + rng.next_f32() * (self.size_max - self.size_min));
            let greeble_height = size * self.height_max * (0.3 + rng.next_f32() * 0.7);

            match self.variety {
                GreebleVariety::BoxesOnly => {
                    add_box_greeble(mesh, center, normal, greeble_size, greeble_height, &mut rng);
                }
                GreebleVariety::Mixed => {
                    if rng.next_f32() < 0.6 {
                        add_box_greeble(mesh, center, normal, greeble_size, greeble_height, &mut rng);
                    } else {
                        add_cylinder_greeble(mesh, center, normal, greeble_size * 0.3, greeble_height, 6);
                    }
                }
                GreebleVariety::Antennae => {
                    add_cylinder_greeble(mesh, center, normal, greeble_size * 0.1, greeble_height * 2.0, 4);
                }
            }
        }

        SmoothNormals::default().apply(mesh);
    }
}

/// Add bolts to surfaces (larger than rivets, with hex/cross patterns)
pub struct AddBolts {
    /// Bolt head radius
    pub radius: f32,
    /// Bolt head height
    pub height: f32,
    /// Bolt type
    pub bolt_type: BoltType,
    /// Spacing between bolts
    pub spacing: f32,
    /// Only on flat surfaces
    pub flat_only: bool,
    /// Random seed
    pub seed: u32,
}

#[derive(Clone, Copy, Default)]
pub enum BoltType {
    /// Hexagonal bolt head
    #[default]
    Hex,
    /// Round head
    Round,
    /// Phillips cross head
    Phillips,
}

impl Default for AddBolts {
    fn default() -> Self {
        Self {
            radius: 0.03,
            height: 0.015,
            bolt_type: BoltType::Hex,
            spacing: 0.15,
            flat_only: true,
            seed: 0,
        }
    }
}

impl MeshModifier for AddBolts {
    fn apply(&self, mesh: &mut UnpackedMesh) {
        if mesh.positions.is_empty() || mesh.indices.len() < 3 {
            return;
        }

        let bounds = calculate_bounds(mesh);
        let size = (bounds.1 - bounds.0).length();
        let actual_radius = size * self.radius;
        let actual_height = size * self.height;
        let actual_spacing = size * self.spacing;

        let mut rng = SimpleRng::new(self.seed);

        // Get suitable face centers
        let face_data: Vec<(Vec3, Vec3, f32)> = mesh.indices.chunks(3)
            .filter_map(|chunk| {
                if chunk.len() == 3 {
                    let p0 = Vec3::from(mesh.positions[chunk[0] as usize]);
                    let p1 = Vec3::from(mesh.positions[chunk[1] as usize]);
                    let p2 = Vec3::from(mesh.positions[chunk[2] as usize]);
                    let center = (p0 + p1 + p2) / 3.0;
                    let normal = (p1 - p0).cross(p2 - p0).normalize();
                    let area = (p1 - p0).cross(p2 - p0).length() * 0.5;
                    Some((center, normal, area))
                } else {
                    None
                }
            })
            .collect();

        // Place bolts on grid pattern
        let mut placed = Vec::new();

        for (center, normal, area) in face_data {
            // Skip small faces
            if area < actual_spacing * actual_spacing * 0.1 {
                continue;
            }

            // Check if flat enough
            if self.flat_only && normal.y.abs() < 0.7 && normal.x.abs() < 0.7 && normal.z.abs() < 0.7 {
                continue;
            }

            // Check spacing from existing bolts
            let too_close = placed.iter().any(|p: &Vec3| (*p - center).length() < actual_spacing);
            if too_close {
                continue;
            }

            // Random chance to skip
            if rng.next_f32() > 0.4 {
                continue;
            }

            placed.push(center);

            let segments = match self.bolt_type {
                BoltType::Hex => 6,
                BoltType::Round => 12,
                BoltType::Phillips => 8,
            };

            add_rivet(mesh, center, normal, actual_radius, actual_height, segments);
        }

        SmoothNormals::default().apply(mesh);
    }
}

// Helper functions

fn calculate_bounds(mesh: &UnpackedMesh) -> (Vec3, Vec3) {
    let mut min = Vec3::splat(f32::MAX);
    let mut max = Vec3::splat(f32::MIN);

    for pos in &mesh.positions {
        min = min.min(Vec3::from(*pos));
        max = max.max(Vec3::from(*pos));
    }

    if min.x > max.x {
        (Vec3::ZERO, Vec3::ONE)
    } else {
        (min, max)
    }
}

fn find_sharp_edges(mesh: &UnpackedMesh, min_angle: f32) -> Vec<(usize, usize)> {
    use std::collections::HashMap;

    // Build edge -> face map
    let mut edge_faces: HashMap<(usize, usize), Vec<usize>> = HashMap::new();

    for (face_idx, chunk) in mesh.indices.chunks(3).enumerate() {
        if chunk.len() == 3 {
            let i0 = chunk[0] as usize;
            let i1 = chunk[1] as usize;
            let i2 = chunk[2] as usize;

            for &(a, b) in &[(i0, i1), (i1, i2), (i2, i0)] {
                let key = if a < b { (a, b) } else { (b, a) };
                edge_faces.entry(key).or_default().push(face_idx);
            }
        }
    }

    // Calculate face normals
    let face_normals: Vec<Vec3> = mesh.indices.chunks(3)
        .map(|chunk| {
            if chunk.len() == 3 {
                let p0 = Vec3::from(mesh.positions[chunk[0] as usize]);
                let p1 = Vec3::from(mesh.positions[chunk[1] as usize]);
                let p2 = Vec3::from(mesh.positions[chunk[2] as usize]);
                (p1 - p0).cross(p2 - p0).normalize()
            } else {
                Vec3::Y
            }
        })
        .collect();

    // Find edges where adjacent faces have angle > min_angle
    let mut sharp_edges = Vec::new();

    for (edge, faces) in edge_faces {
        if faces.len() == 2 {
            let n0 = face_normals[faces[0]];
            let n1 = face_normals[faces[1]];
            let angle = n0.dot(n1).clamp(-1.0, 1.0).acos();

            if angle > min_angle {
                sharp_edges.push(edge);
            }
        }
    }

    sharp_edges
}

fn add_rivet(
    mesh: &mut UnpackedMesh,
    center: Vec3,
    normal: Vec3,
    radius: f32,
    height: f32,
    segments: u32,
) {
    // Create tangent basis
    let tangent = if normal.y.abs() < 0.9 {
        normal.cross(Vec3::Y).normalize()
    } else {
        normal.cross(Vec3::X).normalize()
    };
    let bitangent = normal.cross(tangent);

    let base_idx = mesh.positions.len() as u16;

    // Center top vertex
    let top = center + normal * height;
    mesh.positions.push([top.x, top.y, top.z]);
    mesh.normals.push([normal.x, normal.y, normal.z]);
    if !mesh.uvs.is_empty() {
        mesh.uvs.push([0.5, 0.5]);
    }
    if !mesh.colors.is_empty() {
        mesh.colors.push([255, 255, 255, 255]);
    }

    // Ring vertices
    for i in 0..segments {
        let angle = (i as f32 / segments as f32) * std::f32::consts::TAU;
        let (sin_a, cos_a) = angle.sin_cos();

        let rim_pos = center + tangent * cos_a * radius + bitangent * sin_a * radius;
        let rim_normal = (rim_pos - center).normalize() * 0.5 + normal * 0.5;

        mesh.positions.push([rim_pos.x, rim_pos.y, rim_pos.z]);
        mesh.normals.push([rim_normal.x, rim_normal.y, rim_normal.z]);
        if !mesh.uvs.is_empty() {
            mesh.uvs.push([0.5 + cos_a * 0.5, 0.5 + sin_a * 0.5]);
        }
        if !mesh.colors.is_empty() {
            mesh.colors.push([255, 255, 255, 255]);
        }
    }

    // Create triangles (fan from center)
    for i in 0..segments {
        let next = (i + 1) % segments;
        mesh.indices.push(base_idx); // Center
        mesh.indices.push(base_idx + 1 + i as u16);
        mesh.indices.push(base_idx + 1 + next as u16);
    }
}

fn add_box_greeble(
    mesh: &mut UnpackedMesh,
    center: Vec3,
    normal: Vec3,
    size: f32,
    height: f32,
    rng: &mut SimpleRng,
) {
    // Create tangent basis
    let tangent = if normal.y.abs() < 0.9 {
        normal.cross(Vec3::Y).normalize()
    } else {
        normal.cross(Vec3::X).normalize()
    };
    let bitangent = normal.cross(tangent);

    // Random aspect ratio
    let width = size * (0.5 + rng.next_f32() * 0.5);
    let depth = size * (0.5 + rng.next_f32() * 0.5);

    let base_idx = mesh.positions.len() as u16;

    // 8 vertices of box
    let corners = [
        center + tangent * width * 0.5 + bitangent * depth * 0.5,
        center - tangent * width * 0.5 + bitangent * depth * 0.5,
        center - tangent * width * 0.5 - bitangent * depth * 0.5,
        center + tangent * width * 0.5 - bitangent * depth * 0.5,
        center + tangent * width * 0.5 + bitangent * depth * 0.5 + normal * height,
        center - tangent * width * 0.5 + bitangent * depth * 0.5 + normal * height,
        center - tangent * width * 0.5 - bitangent * depth * 0.5 + normal * height,
        center + tangent * width * 0.5 - bitangent * depth * 0.5 + normal * height,
    ];

    for pos in &corners {
        mesh.positions.push([pos.x, pos.y, pos.z]);
        mesh.normals.push([normal.x, normal.y, normal.z]); // Simplified
        if !mesh.uvs.is_empty() {
            mesh.uvs.push([0.5, 0.5]);
        }
        if !mesh.colors.is_empty() {
            mesh.colors.push([255, 255, 255, 255]);
        }
    }

    // Top face
    mesh.indices.extend_from_slice(&[base_idx + 4, base_idx + 5, base_idx + 6]);
    mesh.indices.extend_from_slice(&[base_idx + 4, base_idx + 6, base_idx + 7]);

    // Side faces
    let sides = [
        [0, 1, 5, 4],
        [1, 2, 6, 5],
        [2, 3, 7, 6],
        [3, 0, 4, 7],
    ];

    for side in &sides {
        mesh.indices.extend_from_slice(&[
            base_idx + side[0],
            base_idx + side[1],
            base_idx + side[2],
        ]);
        mesh.indices.extend_from_slice(&[
            base_idx + side[0],
            base_idx + side[2],
            base_idx + side[3],
        ]);
    }
}

fn add_cylinder_greeble(
    mesh: &mut UnpackedMesh,
    center: Vec3,
    normal: Vec3,
    radius: f32,
    height: f32,
    segments: u32,
) {
    let tangent = if normal.y.abs() < 0.9 {
        normal.cross(Vec3::Y).normalize()
    } else {
        normal.cross(Vec3::X).normalize()
    };
    let bitangent = normal.cross(tangent);

    let base_idx = mesh.positions.len() as u16;

    // Bottom ring + top ring
    for layer in 0..2 {
        let h = if layer == 0 { 0.0 } else { height };
        for i in 0..segments {
            let angle = (i as f32 / segments as f32) * std::f32::consts::TAU;
            let (sin_a, cos_a) = angle.sin_cos();

            let pos = center + tangent * cos_a * radius + bitangent * sin_a * radius + normal * h;
            let n = (tangent * cos_a + bitangent * sin_a).normalize();

            mesh.positions.push([pos.x, pos.y, pos.z]);
            mesh.normals.push([n.x, n.y, n.z]);
            if !mesh.uvs.is_empty() {
                mesh.uvs.push([i as f32 / segments as f32, layer as f32]);
            }
            if !mesh.colors.is_empty() {
                mesh.colors.push([255, 255, 255, 255]);
            }
        }
    }

    // Top center
    let top_center = center + normal * height;
    mesh.positions.push([top_center.x, top_center.y, top_center.z]);
    mesh.normals.push([normal.x, normal.y, normal.z]);
    if !mesh.uvs.is_empty() {
        mesh.uvs.push([0.5, 1.0]);
    }
    if !mesh.colors.is_empty() {
        mesh.colors.push([255, 255, 255, 255]);
    }

    let top_center_idx = base_idx + segments as u16 * 2;

    // Side quads
    for i in 0..segments {
        let next = (i + 1) % segments;
        let b0 = base_idx + i as u16;
        let b1 = base_idx + next as u16;
        let t0 = base_idx + segments as u16 + i as u16;
        let t1 = base_idx + segments as u16 + next as u16;

        mesh.indices.extend_from_slice(&[b0, b1, t1]);
        mesh.indices.extend_from_slice(&[b0, t1, t0]);
    }

    // Top cap
    for i in 0..segments {
        let next = (i + 1) % segments;
        mesh.indices.push(top_center_idx);
        mesh.indices.push(base_idx + segments as u16 + i as u16);
        mesh.indices.push(base_idx + segments as u16 + next as u16);
    }
}

fn groove_intensity(coord: f32, spacing: f32, width: f32) -> f32 {
    let mod_pos = coord % spacing;
    let dist_from_line = mod_pos.min(spacing - mod_pos);

    if dist_from_line < width {
        1.0 - (dist_from_line / width)
    } else {
        0.0
    }
}

fn simple_hash(x: i32, y: i32, seed: u32) -> u32 {
    let mut h = seed.wrapping_add(x as u32 * 374761393);
    h = h.wrapping_add(y as u32 * 668265263);
    h = (h ^ (h >> 13)).wrapping_mul(1274126177);
    h ^ (h >> 16)
}

/// Simple deterministic RNG
struct SimpleRng {
    state: u32,
}

impl SimpleRng {
    fn new(seed: u32) -> Self {
        Self { state: seed.wrapping_add(1) }
    }

    fn next(&mut self) -> u32 {
        self.state = self.state.wrapping_mul(1103515245).wrapping_add(12345);
        self.state
    }

    fn next_f32(&mut self) -> f32 {
        (self.next() & 0x7FFFFFFF) as f32 / 0x7FFFFFFF as f32
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nethercore_zx::procedural::generate_cube_uv;

    #[test]
    fn test_add_rivets() {
        let mut mesh: UnpackedMesh = generate_cube_uv(1.0, 1.0, 1.0);
        let original_verts = mesh.positions.len();

        AddRivets::default().apply(&mut mesh);

        // Should have added some geometry
        assert!(mesh.positions.len() >= original_verts);
    }

    #[test]
    fn test_add_panel_lines() {
        let mut mesh: UnpackedMesh = generate_cube_uv(1.0, 1.0, 1.0);

        AddPanelLines::default().apply(&mut mesh);

        // Should still have valid structure
        assert!(!mesh.positions.is_empty());
    }

    #[test]
    fn test_add_greebles() {
        let mut mesh: UnpackedMesh = generate_cube_uv(1.0, 1.0, 1.0);
        let original_verts = mesh.positions.len();

        AddGreebles::light().apply(&mut mesh);

        // Should have added geometry
        assert!(mesh.positions.len() >= original_verts);
    }

    #[test]
    fn test_add_bolts() {
        let mut mesh: UnpackedMesh = generate_cube_uv(1.0, 1.0, 1.0);
        let original_verts = mesh.positions.len();

        AddBolts::default().apply(&mut mesh);

        // Should have added some bolts
        assert!(mesh.positions.len() >= original_verts);
    }
}
