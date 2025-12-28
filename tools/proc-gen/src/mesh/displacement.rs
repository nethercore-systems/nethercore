//! Noise displacement modifiers for meshes
//!
//! Adds organic variation to meshes by displacing vertices along their
//! normals using noise functions. Essential for making meshes look
//! less like perfect mathematical primitives.

use super::modifiers::{MeshModifier, SmoothNormals};
use nethercore_zx::procedural::UnpackedMesh;
use glam::Vec3;
use noise::{NoiseFn, Perlin};

/// Displace mesh vertices along normals using noise
///
/// This modifier adds subtle (or not so subtle) organic variation
/// to any mesh, making it look weathered, damaged, or natural.
pub struct NoiseDisplace {
    /// Displacement amplitude in world units
    pub amplitude: f32,
    /// Noise scale (larger = smoother bumps)
    pub scale: f32,
    /// Number of octaves for detail (1-4)
    pub octaves: u32,
    /// Persistence for multi-octave noise (0.0-1.0)
    pub persistence: f32,
    /// Random seed for deterministic generation
    pub seed: u32,
    /// Whether to recalculate normals after displacement
    pub recalculate_normals: bool,
}

impl Default for NoiseDisplace {
    fn default() -> Self {
        Self {
            amplitude: 0.05,
            scale: 1.0,
            octaves: 2,
            persistence: 0.5,
            seed: 0,
            recalculate_normals: true,
        }
    }
}

impl NoiseDisplace {
    /// Create subtle displacement for minor imperfections
    pub fn subtle(seed: u32) -> Self {
        Self {
            amplitude: 0.02,
            scale: 2.0,
            octaves: 2,
            persistence: 0.5,
            seed,
            recalculate_normals: true,
        }
    }

    /// Create moderate displacement for weathered surfaces
    pub fn weathered(seed: u32) -> Self {
        Self {
            amplitude: 0.05,
            scale: 1.5,
            octaves: 3,
            persistence: 0.6,
            seed,
            recalculate_normals: true,
        }
    }

    /// Create heavy displacement for damaged/organic surfaces
    pub fn heavy(seed: u32) -> Self {
        Self {
            amplitude: 0.1,
            scale: 1.0,
            octaves: 4,
            persistence: 0.65,
            seed,
            recalculate_normals: true,
        }
    }

    fn sample_fbm(&self, noise: &Perlin, x: f64, y: f64, z: f64) -> f64 {
        let mut total = 0.0;
        let mut amplitude = 1.0;
        let mut frequency = self.scale as f64;
        let mut max_value = 0.0;

        for _ in 0..self.octaves {
            total += noise.get([x * frequency, y * frequency, z * frequency]) * amplitude;
            max_value += amplitude;
            amplitude *= self.persistence as f64;
            frequency *= 2.0;
        }

        total / max_value
    }
}

impl MeshModifier for NoiseDisplace {
    fn apply(&self, mesh: &mut UnpackedMesh) {
        let perlin = Perlin::new(self.seed);

        for i in 0..mesh.positions.len() {
            let pos = Vec3::from(mesh.positions[i]);
            let normal = Vec3::from(mesh.normals[i]);

            // Sample noise at vertex position
            let noise_value = self.sample_fbm(
                &perlin,
                pos.x as f64,
                pos.y as f64,
                pos.z as f64,
            ) as f32;

            // Displace along normal
            let displaced = pos + normal * noise_value * self.amplitude;
            mesh.positions[i] = [displaced.x, displaced.y, displaced.z];
        }

        // Recalculate normals if requested
        if self.recalculate_normals {
            SmoothNormals::default().apply(mesh);
        }
    }
}

/// Directional displacement along a specific axis
pub struct DirectionalDisplace {
    /// Displacement amplitude
    pub amplitude: f32,
    /// Noise scale
    pub scale: f32,
    /// Displacement axis (will be normalized)
    pub axis: [f32; 3],
    /// Falloff from axis (0.0 = no falloff, 1.0 = full falloff at 90 degrees)
    pub falloff: f32,
    /// Random seed
    pub seed: u32,
}

impl Default for DirectionalDisplace {
    fn default() -> Self {
        Self {
            amplitude: 0.05,
            scale: 1.0,
            axis: [0.0, 1.0, 0.0], // Default: Y axis (up)
            falloff: 0.0,
            seed: 0,
        }
    }
}

impl MeshModifier for DirectionalDisplace {
    fn apply(&self, mesh: &mut UnpackedMesh) {
        let perlin = Perlin::new(self.seed);
        let axis = Vec3::from(self.axis).normalize();

        for i in 0..mesh.positions.len() {
            let pos = Vec3::from(mesh.positions[i]);
            let normal = Vec3::from(mesh.normals[i]);

            // Calculate falloff based on normal alignment with axis
            let alignment = normal.dot(axis).abs();
            let falloff_factor = 1.0 - (1.0 - alignment) * self.falloff;

            // Sample noise
            let noise_value = perlin.get([
                pos.x as f64 * self.scale as f64,
                pos.y as f64 * self.scale as f64,
                pos.z as f64 * self.scale as f64,
            ]) as f32;

            // Displace along axis
            let displaced = pos + axis * noise_value * self.amplitude * falloff_factor;
            mesh.positions[i] = [displaced.x, displaced.y, displaced.z];
        }

        SmoothNormals::default().apply(mesh);
    }
}

/// Bulge modifier - expands or contracts mesh along an axis
pub struct Bulge {
    /// Bulge amount (positive = expand, negative = contract)
    pub amount: f32,
    /// Axis to bulge along
    pub axis: BulgeAxis,
    /// Falloff from center (0.0 = uniform, 1.0 = only at center)
    pub falloff: f32,
    /// Center position along axis (0.0 to 1.0)
    pub center: f32,
}

/// Axis for bulge operation
#[derive(Clone, Copy, Default)]
pub enum BulgeAxis {
    X,
    #[default]
    Y,
    Z,
}

impl Default for Bulge {
    fn default() -> Self {
        Self {
            amount: 0.1,
            axis: BulgeAxis::Y,
            falloff: 0.8,
            center: 0.5,
        }
    }
}

impl MeshModifier for Bulge {
    fn apply(&self, mesh: &mut UnpackedMesh) {
        if mesh.positions.is_empty() {
            return;
        }

        // Find bounds along axis
        let axis_idx = match self.axis {
            BulgeAxis::X => 0,
            BulgeAxis::Y => 1,
            BulgeAxis::Z => 2,
        };

        let min = mesh.positions.iter().map(|p| p[axis_idx]).fold(f32::MAX, f32::min);
        let max = mesh.positions.iter().map(|p| p[axis_idx]).fold(f32::MIN, f32::max);
        let range = max - min;

        if range < 0.0001 {
            return;
        }

        for i in 0..mesh.positions.len() {
            let pos = Vec3::from(mesh.positions[i]);
            let normal = Vec3::from(mesh.normals[i]);

            // Calculate position along axis (0.0 to 1.0)
            let t = (mesh.positions[i][axis_idx] - min) / range;

            // Calculate bulge factor based on distance from center
            let dist_from_center = (t - self.center).abs();
            let bulge_factor = 1.0 - (dist_from_center * 2.0).min(1.0);
            let bulge_factor = bulge_factor.powf(1.0 / (1.0 - self.falloff + 0.01));

            // Apply bulge perpendicular to axis
            let axis_vec = match self.axis {
                BulgeAxis::X => Vec3::X,
                BulgeAxis::Y => Vec3::Y,
                BulgeAxis::Z => Vec3::Z,
            };

            // Project normal onto plane perpendicular to axis
            let perp_normal = (normal - axis_vec * normal.dot(axis_vec)).normalize_or_zero();

            let displaced = pos + perp_normal * self.amount * bulge_factor;
            mesh.positions[i] = [displaced.x, displaced.y, displaced.z];
        }

        SmoothNormals::default().apply(mesh);
    }
}

/// Twist modifier - rotates mesh around an axis
pub struct Twist {
    /// Total twist angle in degrees
    pub angle: f32,
    /// Axis to twist around
    pub axis: BulgeAxis,
    /// Twist distribution (0.0 = linear, 1.0 = concentrated at ends)
    pub distribution: f32,
}

impl Default for Twist {
    fn default() -> Self {
        Self {
            angle: 45.0,
            axis: BulgeAxis::Y,
            distribution: 0.0,
        }
    }
}

impl MeshModifier for Twist {
    fn apply(&self, mesh: &mut UnpackedMesh) {
        if mesh.positions.is_empty() {
            return;
        }

        let axis_idx = match self.axis {
            BulgeAxis::X => 0,
            BulgeAxis::Y => 1,
            BulgeAxis::Z => 2,
        };

        let min = mesh.positions.iter().map(|p| p[axis_idx]).fold(f32::MAX, f32::min);
        let max = mesh.positions.iter().map(|p| p[axis_idx]).fold(f32::MIN, f32::max);
        let range = max - min;

        if range < 0.0001 {
            return;
        }

        for i in 0..mesh.positions.len() {
            let pos = Vec3::from(mesh.positions[i]);

            // Calculate position along axis (0.0 to 1.0)
            let t = (mesh.positions[i][axis_idx] - min) / range;

            // Calculate twist angle at this position
            let twist_t = if self.distribution > 0.0 {
                // Non-linear distribution
                let pow = 1.0 + self.distribution * 2.0;
                t.powf(pow)
            } else {
                t
            };
            let angle_rad = (self.angle * twist_t - self.angle * 0.5).to_radians();

            // Rotate around axis
            let (sin_a, cos_a) = angle_rad.sin_cos();
            let rotated = match self.axis {
                BulgeAxis::X => Vec3::new(
                    pos.x,
                    pos.y * cos_a - pos.z * sin_a,
                    pos.y * sin_a + pos.z * cos_a,
                ),
                BulgeAxis::Y => Vec3::new(
                    pos.x * cos_a - pos.z * sin_a,
                    pos.y,
                    pos.x * sin_a + pos.z * cos_a,
                ),
                BulgeAxis::Z => Vec3::new(
                    pos.x * cos_a - pos.y * sin_a,
                    pos.x * sin_a + pos.y * cos_a,
                    pos.z,
                ),
            };

            mesh.positions[i] = [rotated.x, rotated.y, rotated.z];
        }

        SmoothNormals::default().apply(mesh);
    }
}

/// Taper modifier - scales mesh along an axis
pub struct Taper {
    /// Taper amount (0.0 = no taper, 1.0 = shrink to point)
    pub amount: f32,
    /// Axis to taper along
    pub axis: BulgeAxis,
    /// Which end to taper (0.0 = start, 1.0 = end, 0.5 = both)
    pub taper_end: f32,
}

impl Default for Taper {
    fn default() -> Self {
        Self {
            amount: 0.5,
            axis: BulgeAxis::Y,
            taper_end: 1.0,
        }
    }
}

impl MeshModifier for Taper {
    fn apply(&self, mesh: &mut UnpackedMesh) {
        if mesh.positions.is_empty() {
            return;
        }

        let axis_idx = match self.axis {
            BulgeAxis::X => 0,
            BulgeAxis::Y => 1,
            BulgeAxis::Z => 2,
        };

        let min = mesh.positions.iter().map(|p| p[axis_idx]).fold(f32::MAX, f32::min);
        let max = mesh.positions.iter().map(|p| p[axis_idx]).fold(f32::MIN, f32::max);
        let range = max - min;

        if range < 0.0001 {
            return;
        }

        for i in 0..mesh.positions.len() {
            // Calculate position along axis (0.0 to 1.0)
            let t = (mesh.positions[i][axis_idx] - min) / range;

            // Calculate scale factor
            let taper_factor = if self.taper_end < 0.5 {
                // Taper at start
                1.0 - (1.0 - t) * self.amount
            } else if self.taper_end > 0.5 {
                // Taper at end
                1.0 - t * self.amount
            } else {
                // Taper both ends
                let dist_from_center = (t - 0.5).abs() * 2.0;
                1.0 - dist_from_center * self.amount
            };

            // Scale perpendicular to axis
            match self.axis {
                BulgeAxis::X => {
                    mesh.positions[i][1] *= taper_factor;
                    mesh.positions[i][2] *= taper_factor;
                }
                BulgeAxis::Y => {
                    mesh.positions[i][0] *= taper_factor;
                    mesh.positions[i][2] *= taper_factor;
                }
                BulgeAxis::Z => {
                    mesh.positions[i][0] *= taper_factor;
                    mesh.positions[i][1] *= taper_factor;
                }
            }
        }

        SmoothNormals::default().apply(mesh);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nethercore_zx::procedural::generate_sphere;

    #[test]
    fn test_noise_displace() {
        let mut mesh: UnpackedMesh = generate_sphere(1.0, 8, 4);
        let original_positions = mesh.positions.clone();

        NoiseDisplace::subtle(42).apply(&mut mesh);

        // Positions should have changed
        assert_ne!(mesh.positions, original_positions);

        // Should still have valid vertex count
        assert_eq!(mesh.positions.len(), original_positions.len());
    }

    #[test]
    fn test_noise_displace_deterministic() {
        let mut mesh1: UnpackedMesh = generate_sphere(1.0, 8, 4);
        let mut mesh2: UnpackedMesh = generate_sphere(1.0, 8, 4);

        NoiseDisplace::subtle(42).apply(&mut mesh1);
        NoiseDisplace::subtle(42).apply(&mut mesh2);

        // Same seed should produce same result
        assert_eq!(mesh1.positions, mesh2.positions);
    }

    #[test]
    fn test_bulge() {
        let mut mesh: UnpackedMesh = generate_sphere(1.0, 8, 4);

        Bulge {
            amount: 0.2,
            axis: BulgeAxis::Y,
            falloff: 0.5,
            center: 0.5,
        }.apply(&mut mesh);

        // Mesh should still have valid structure
        assert!(!mesh.positions.is_empty());
        for &idx in &mesh.indices {
            assert!((idx as usize) < mesh.positions.len());
        }
    }

    #[test]
    fn test_twist() {
        let mut mesh: UnpackedMesh = generate_sphere(1.0, 8, 4);

        Twist {
            angle: 90.0,
            axis: BulgeAxis::Y,
            distribution: 0.0,
        }.apply(&mut mesh);

        // Mesh should still have valid structure
        assert!(!mesh.positions.is_empty());
    }

    #[test]
    fn test_taper() {
        let mut mesh: UnpackedMesh = generate_sphere(1.0, 8, 4);

        Taper {
            amount: 0.5,
            axis: BulgeAxis::Y,
            taper_end: 1.0,
        }.apply(&mut mesh);

        // Mesh should still have valid structure
        assert!(!mesh.positions.is_empty());
    }
}
