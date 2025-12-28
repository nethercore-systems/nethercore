//! Vertex color generation for meshes
//!
//! Provides ambient occlusion baking, curvature-based coloring, and
//! other vertex color techniques essential for PS1/PS2/N64-style graphics.
//! These techniques were THE signature of that era.

use super::modifiers::MeshModifier;
use nethercore_zx::procedural::UnpackedMesh;
use glam::Vec3;

/// Vertex color channel selection
#[derive(Clone, Copy, Debug, Default)]
pub enum ColorChannel {
    #[default]
    R,
    G,
    B,
    A,
    /// Store in all RGB channels (grayscale)
    RGB,
}

/// Bake ambient occlusion into vertex colors
///
/// This is THE signature technique for PS1/PS2/N64-era graphics.
/// Pre-computed shadows give depth without runtime lighting cost.
pub struct BakeVertexAO {
    /// Number of sample rays per vertex (more = better quality, slower)
    pub samples: u32,
    /// Maximum ray distance (relative to mesh size)
    pub max_distance: f32,
    /// AO intensity (0.0 to 1.0)
    pub intensity: f32,
    /// Which channel to store AO
    pub channel: ColorChannel,
    /// Bias to push sample origin slightly outside surface
    pub bias: f32,
}

impl Default for BakeVertexAO {
    fn default() -> Self {
        Self {
            samples: 32,
            max_distance: 0.5,
            intensity: 1.0,
            channel: ColorChannel::RGB,
            bias: 0.001,
        }
    }
}

impl BakeVertexAO {
    /// Quick low-quality AO for previews
    pub fn quick() -> Self {
        Self {
            samples: 8,
            max_distance: 0.3,
            intensity: 0.8,
            ..Default::default()
        }
    }

    /// High quality AO for final assets
    pub fn high_quality() -> Self {
        Self {
            samples: 64,
            max_distance: 0.5,
            intensity: 1.0,
            ..Default::default()
        }
    }
}

impl MeshModifier for BakeVertexAO {
    fn apply(&self, mesh: &mut UnpackedMesh) {
        if mesh.positions.is_empty() {
            return;
        }

        // Calculate mesh bounds for distance scaling
        let bounds = calculate_bounds(mesh);
        let size = (bounds.1 - bounds.0).length();
        let actual_max_dist = size * self.max_distance;

        // Build simple spatial structure for ray testing
        let triangles = build_triangles(mesh);

        // Ensure vertex colors exist
        ensure_vertex_colors(mesh);

        // Process each vertex
        for i in 0..mesh.positions.len() {
            let pos = Vec3::from(mesh.positions[i]);
            let normal = Vec3::from(mesh.normals[i]).normalize();

            // Sample hemisphere
            let mut occlusion = 0.0;
            for s in 0..self.samples {
                let ray_dir = hemisphere_sample(normal, s, self.samples);
                let ray_origin = pos + normal * self.bias;

                if ray_hits_any(&triangles, ray_origin, ray_dir, actual_max_dist) {
                    occlusion += 1.0;
                }
            }

            // Convert to AO value (1.0 = fully lit, 0.0 = fully occluded)
            let ao = 1.0 - (occlusion / self.samples as f32) * self.intensity;
            let ao_byte = (ao.clamp(0.0, 1.0) * 255.0) as u8;

            set_vertex_color_channel(&mut mesh.colors, i, self.channel, ao_byte);
        }
    }
}

/// Bake curvature into vertex colors
///
/// High curvature (convex edges) gets high values - useful for edge wear
/// Low curvature (concave corners) gets low values - useful for dirt accumulation
pub struct BakeVertexCurvature {
    /// Which channel to store curvature
    pub channel: ColorChannel,
    /// Invert values (edges become dark, corners become light)
    pub invert: bool,
    /// Smoothing iterations
    pub smooth_iterations: u32,
}

impl Default for BakeVertexCurvature {
    fn default() -> Self {
        Self {
            channel: ColorChannel::G,
            invert: false,
            smooth_iterations: 1,
        }
    }
}

impl MeshModifier for BakeVertexCurvature {
    fn apply(&self, mesh: &mut UnpackedMesh) {
        if mesh.positions.is_empty() || mesh.indices.len() < 3 {
            return;
        }

        // Calculate curvature at each vertex
        let mut curvatures = vec![0.0f32; mesh.positions.len()];

        // Build adjacency (which faces share each vertex)
        let mut vertex_faces: Vec<Vec<usize>> = vec![Vec::new(); mesh.positions.len()];
        for (face_idx, chunk) in mesh.indices.chunks(3).enumerate() {
            if chunk.len() == 3 {
                vertex_faces[chunk[0] as usize].push(face_idx);
                vertex_faces[chunk[1] as usize].push(face_idx);
                vertex_faces[chunk[2] as usize].push(face_idx);
            }
        }

        // Calculate face normals
        let mut face_normals = Vec::new();
        for chunk in mesh.indices.chunks(3) {
            if chunk.len() == 3 {
                let p0 = Vec3::from(mesh.positions[chunk[0] as usize]);
                let p1 = Vec3::from(mesh.positions[chunk[1] as usize]);
                let p2 = Vec3::from(mesh.positions[chunk[2] as usize]);
                let normal = (p1 - p0).cross(p2 - p0).normalize();
                face_normals.push(normal);
            }
        }

        // Calculate curvature as variance of face normals around each vertex
        for i in 0..mesh.positions.len() {
            let faces = &vertex_faces[i];
            if faces.len() < 2 {
                curvatures[i] = 0.5; // Neutral for boundary vertices
                continue;
            }

            let vertex_normal = Vec3::from(mesh.normals[i]).normalize();

            // Calculate variance of face normals from vertex normal
            let mut variance = 0.0;
            for &face_idx in faces {
                if face_idx < face_normals.len() {
                    let face_normal = face_normals[face_idx];
                    let dot = vertex_normal.dot(face_normal).clamp(-1.0, 1.0);
                    // Higher variance = higher curvature
                    variance += 1.0 - dot.abs();
                }
            }
            variance /= faces.len() as f32;

            // Also consider angle between adjacent faces
            let mut max_angle = 0.0f32;
            for i_face in 0..faces.len() {
                for j_face in (i_face + 1)..faces.len() {
                    if faces[i_face] < face_normals.len() && faces[j_face] < face_normals.len() {
                        let dot = face_normals[faces[i_face]].dot(face_normals[faces[j_face]]);
                        let angle = dot.clamp(-1.0, 1.0).acos();
                        max_angle = max_angle.max(angle);
                    }
                }
            }

            // Combine variance and max angle
            curvatures[i] = (variance * 2.0 + max_angle / std::f32::consts::PI) / 3.0;
        }

        // Smooth curvatures
        for _ in 0..self.smooth_iterations {
            let mut smoothed = curvatures.clone();
            for i in 0..mesh.positions.len() {
                let faces = &vertex_faces[i];
                let mut sum = curvatures[i];
                let mut count = 1;

                for &face_idx in faces {
                    let face_start = face_idx * 3;
                    if face_start + 2 < mesh.indices.len() {
                        for j in 0..3 {
                            let neighbor = mesh.indices[face_start + j] as usize;
                            if neighbor != i {
                                sum += curvatures[neighbor];
                                count += 1;
                            }
                        }
                    }
                }

                smoothed[i] = sum / count as f32;
            }
            curvatures = smoothed;
        }

        // Apply to vertex colors
        ensure_vertex_colors(mesh);

        for i in 0..mesh.positions.len() {
            let mut value = curvatures[i].clamp(0.0, 1.0);
            if self.invert {
                value = 1.0 - value;
            }
            let byte = (value * 255.0) as u8;
            set_vertex_color_channel(&mut mesh.colors, i, self.channel, byte);
        }
    }
}

/// Gradient vertex color along an axis
pub struct VertexColorGradient {
    /// Axis to apply gradient along
    pub axis: GradientAxis,
    /// Start color (at min position)
    pub color_start: [u8; 4],
    /// End color (at max position)
    pub color_end: [u8; 4],
    /// Gradient curve (1.0 = linear, <1 = ease in, >1 = ease out)
    pub curve: f32,
}

#[derive(Clone, Copy, Default)]
pub enum GradientAxis {
    X,
    #[default]
    Y,
    Z,
}

impl Default for VertexColorGradient {
    fn default() -> Self {
        Self {
            axis: GradientAxis::Y,
            color_start: [100, 100, 100, 255],
            color_end: [255, 255, 255, 255],
            curve: 1.0,
        }
    }
}

impl MeshModifier for VertexColorGradient {
    fn apply(&self, mesh: &mut UnpackedMesh) {
        if mesh.positions.is_empty() {
            return;
        }

        let axis_idx = match self.axis {
            GradientAxis::X => 0,
            GradientAxis::Y => 1,
            GradientAxis::Z => 2,
        };

        let min = mesh.positions.iter().map(|p| p[axis_idx]).fold(f32::MAX, f32::min);
        let max = mesh.positions.iter().map(|p| p[axis_idx]).fold(f32::MIN, f32::max);
        let range = max - min;

        if range < 0.0001 {
            return;
        }

        ensure_vertex_colors(mesh);

        for i in 0..mesh.positions.len() {
            let t = (mesh.positions[i][axis_idx] - min) / range;
            let t = t.powf(self.curve).clamp(0.0, 1.0);

            mesh.colors[i] = [
                lerp_u8(self.color_start[0], self.color_end[0], t),
                lerp_u8(self.color_start[1], self.color_end[1], t),
                lerp_u8(self.color_start[2], self.color_end[2], t),
                lerp_u8(self.color_start[3], self.color_end[3], t),
            ];
        }
    }
}

/// Bake simple directional lighting into vertex colors
pub struct BakeDirectionalLight {
    /// Light direction (normalized)
    pub direction: [f32; 3],
    /// Light color
    pub light_color: [u8; 4],
    /// Shadow color
    pub shadow_color: [u8; 4],
    /// Ambient amount (0.0 to 1.0)
    pub ambient: f32,
}

impl Default for BakeDirectionalLight {
    fn default() -> Self {
        Self {
            direction: [0.5, 1.0, 0.3],
            light_color: [255, 250, 240, 255],
            shadow_color: [80, 90, 100, 255],
            ambient: 0.3,
        }
    }
}

impl MeshModifier for BakeDirectionalLight {
    fn apply(&self, mesh: &mut UnpackedMesh) {
        if mesh.positions.is_empty() {
            return;
        }

        let light_dir = Vec3::from(self.direction).normalize();

        ensure_vertex_colors(mesh);

        for i in 0..mesh.positions.len() {
            let normal = Vec3::from(mesh.normals[i]).normalize();
            let ndotl = normal.dot(light_dir).max(0.0);
            let light = self.ambient + ndotl * (1.0 - self.ambient);

            mesh.colors[i] = [
                lerp_u8(self.shadow_color[0], self.light_color[0], light),
                lerp_u8(self.shadow_color[1], self.light_color[1], light),
                lerp_u8(self.shadow_color[2], self.light_color[2], light),
                255,
            ];
        }
    }
}

/// Multiply existing vertex colors with AO
pub struct MultiplyAO {
    /// AO intensity multiplier
    pub intensity: f32,
}

impl Default for MultiplyAO {
    fn default() -> Self {
        Self { intensity: 1.0 }
    }
}

impl MeshModifier for MultiplyAO {
    fn apply(&self, mesh: &mut UnpackedMesh) {
        if mesh.positions.is_empty() || mesh.colors.is_empty() {
            return;
        }

        // First bake AO into a temporary buffer
        let mut ao_values = vec![1.0f32; mesh.positions.len()];

        let bounds = calculate_bounds(mesh);
        let size = (bounds.1 - bounds.0).length();
        let max_dist = size * 0.5;
        let triangles = build_triangles(mesh);

        for i in 0..mesh.positions.len() {
            let pos = Vec3::from(mesh.positions[i]);
            let normal = Vec3::from(mesh.normals[i]).normalize();

            let mut occlusion = 0.0;
            let samples = 16;
            for s in 0..samples {
                let ray_dir = hemisphere_sample(normal, s, samples);
                if ray_hits_any(&triangles, pos + normal * 0.001, ray_dir, max_dist) {
                    occlusion += 1.0;
                }
            }

            ao_values[i] = 1.0 - (occlusion / samples as f32) * self.intensity;
        }

        // Multiply with existing colors
        for i in 0..mesh.colors.len() {
            let ao = ao_values[i].clamp(0.0, 1.0);
            mesh.colors[i][0] = (mesh.colors[i][0] as f32 * ao) as u8;
            mesh.colors[i][1] = (mesh.colors[i][1] as f32 * ao) as u8;
            mesh.colors[i][2] = (mesh.colors[i][2] as f32 * ao) as u8;
        }
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

    (min, max)
}

fn ensure_vertex_colors(mesh: &mut UnpackedMesh) {
    if mesh.colors.len() != mesh.positions.len() {
        mesh.colors = vec![[255, 255, 255, 255]; mesh.positions.len()];
    }
}

fn set_vertex_color_channel(colors: &mut Vec<[u8; 4]>, idx: usize, channel: ColorChannel, value: u8) {
    if idx >= colors.len() {
        return;
    }

    match channel {
        ColorChannel::R => colors[idx][0] = value,
        ColorChannel::G => colors[idx][1] = value,
        ColorChannel::B => colors[idx][2] = value,
        ColorChannel::A => colors[idx][3] = value,
        ColorChannel::RGB => {
            colors[idx][0] = value;
            colors[idx][1] = value;
            colors[idx][2] = value;
        }
    }
}

fn lerp_u8(a: u8, b: u8, t: f32) -> u8 {
    (a as f32 + (b as f32 - a as f32) * t).clamp(0.0, 255.0) as u8
}

/// Simple triangle structure for ray testing
struct Triangle {
    v0: Vec3,
    v1: Vec3,
    v2: Vec3,
}

fn build_triangles(mesh: &UnpackedMesh) -> Vec<Triangle> {
    let mut triangles = Vec::new();

    for chunk in mesh.indices.chunks(3) {
        if chunk.len() == 3 {
            triangles.push(Triangle {
                v0: Vec3::from(mesh.positions[chunk[0] as usize]),
                v1: Vec3::from(mesh.positions[chunk[1] as usize]),
                v2: Vec3::from(mesh.positions[chunk[2] as usize]),
            });
        }
    }

    triangles
}

/// Generate point on hemisphere aligned with normal
fn hemisphere_sample(normal: Vec3, sample_idx: u32, total_samples: u32) -> Vec3 {
    // Simple Fibonacci hemisphere sampling
    let phi = std::f32::consts::PI * (3.0 - 5.0f32.sqrt()); // Golden angle
    let y = 1.0 - (sample_idx as f32 / (total_samples - 1) as f32).clamp(0.0, 1.0);
    let radius = (1.0 - y * y).sqrt();
    let theta = phi * sample_idx as f32;

    let x = theta.cos() * radius;
    let z = theta.sin() * radius;

    // Create basis from normal
    let up = if normal.y.abs() < 0.99 { Vec3::Y } else { Vec3::X };
    let tangent = normal.cross(up).normalize();
    let bitangent = normal.cross(tangent);

    // Transform to normal's hemisphere
    (tangent * x + normal * y + bitangent * z).normalize()
}

/// Check if ray hits any triangle
fn ray_hits_any(triangles: &[Triangle], origin: Vec3, dir: Vec3, max_dist: f32) -> bool {
    for tri in triangles {
        if ray_triangle_intersect(origin, dir, tri, max_dist) {
            return true;
        }
    }
    false
}

/// Möller–Trumbore ray-triangle intersection
fn ray_triangle_intersect(origin: Vec3, dir: Vec3, tri: &Triangle, max_dist: f32) -> bool {
    let edge1 = tri.v1 - tri.v0;
    let edge2 = tri.v2 - tri.v0;
    let h = dir.cross(edge2);
    let a = edge1.dot(h);

    if a.abs() < 0.00001 {
        return false;
    }

    let f = 1.0 / a;
    let s = origin - tri.v0;
    let u = f * s.dot(h);

    if u < 0.0 || u > 1.0 {
        return false;
    }

    let q = s.cross(edge1);
    let v = f * dir.dot(q);

    if v < 0.0 || u + v > 1.0 {
        return false;
    }

    let t = f * edge2.dot(q);

    t > 0.001 && t < max_dist
}

#[cfg(test)]
mod tests {
    use super::*;
    use nethercore_zx::procedural::generate_cube;

    #[test]
    fn test_bake_vertex_ao() {
        let mut mesh: UnpackedMesh = generate_cube(1.0, 1.0, 1.0);

        BakeVertexAO::quick().apply(&mut mesh);

        // Colors should be initialized for all vertices
        assert_eq!(mesh.colors.len(), mesh.positions.len());

        // On a convex shape like a cube, most vertices should be fully lit (high AO values)
        // since there's nothing to occlude them. Check that colors are bright (near white).
        for color in &mesh.colors {
            // AO stored in RGB channels should be reasonably high for convex mesh
            assert!(color[0] >= 200, "AO value too low for convex mesh: {}", color[0]);
            assert!(color[3] == 255, "Alpha should be 255");
        }
    }

    #[test]
    fn test_bake_curvature() {
        let mut mesh: UnpackedMesh = generate_cube(1.0, 1.0, 1.0);

        BakeVertexCurvature::default().apply(&mut mesh);

        assert_eq!(mesh.colors.len(), mesh.positions.len());
    }

    #[test]
    fn test_vertex_color_gradient() {
        let mut mesh: UnpackedMesh = generate_cube(1.0, 1.0, 1.0);

        VertexColorGradient {
            axis: GradientAxis::Y,
            color_start: [0, 0, 0, 255],
            color_end: [255, 255, 255, 255],
            curve: 1.0,
        }.apply(&mut mesh);

        assert_eq!(mesh.colors.len(), mesh.positions.len());

        // Bottom vertices should be darker than top
        // (This depends on cube orientation)
    }

    #[test]
    fn test_bake_directional_light() {
        let mut mesh: UnpackedMesh = generate_cube(1.0, 1.0, 1.0);

        BakeDirectionalLight::default().apply(&mut mesh);

        assert_eq!(mesh.colors.len(), mesh.positions.len());

        // Should have lighting variation
        let first = mesh.colors[0];
        let has_variation = mesh.colors.iter().any(|c| *c != first);
        assert!(has_variation);
    }

    #[test]
    fn test_hemisphere_sample_produces_unit_vectors() {
        let normal = Vec3::Y;
        for i in 0..16 {
            let sample = hemisphere_sample(normal, i, 16);
            let len = sample.length();
            assert!((len - 1.0).abs() < 0.01, "Sample {} has length {}", i, len);
        }
    }

    #[test]
    fn test_hemisphere_samples_are_in_hemisphere() {
        let normal = Vec3::Y;
        for i in 0..16 {
            let sample = hemisphere_sample(normal, i, 16);
            assert!(sample.dot(normal) >= -0.01, "Sample {} is below hemisphere", i);
        }
    }
}
