//! UV manipulation and quality utilities
//!
//! Provides pixel-aware UV snapping, texel density normalization,
//! and UV projection techniques for professional-quality texturing.

use super::modifiers::MeshModifier;
use nethercore_zx::procedural::UnpackedMesh;
use glam::Vec3;

/// Snap UVs to pixel grid for clean texturing
///
/// This technique was used in PS1/PS2 games to ensure
/// texture pixels aligned cleanly with polygon edges.
pub struct PixelSnapUVs {
    /// Target texture resolution (power of 2)
    pub resolution: u32,
    /// Whether to snap to half-pixels (for better filtering)
    pub half_pixel_offset: bool,
}

impl Default for PixelSnapUVs {
    fn default() -> Self {
        Self {
            resolution: 256,
            half_pixel_offset: false,
        }
    }
}

impl MeshModifier for PixelSnapUVs {
    fn apply(&self, mesh: &mut UnpackedMesh) {
        if mesh.uvs.is_empty() {
            return;
        }

        let texel = 1.0 / self.resolution as f32;
        let half_texel = texel * 0.5;

        for uv in &mut mesh.uvs {
            // Snap to texel grid
            uv[0] = (uv[0] / texel).round() * texel;
            uv[1] = (uv[1] / texel).round() * texel;

            // Add half-pixel offset for better bilinear filtering
            if self.half_pixel_offset {
                uv[0] += half_texel;
                uv[1] += half_texel;
            }
        }
    }
}

/// Normalize texel density across UV islands
///
/// Ensures consistent texture detail regardless of face size.
pub struct NormalizeTexelDensity {
    /// Target texels per world unit
    pub texels_per_unit: f32,
    /// Minimum island scale (prevents tiny islands)
    pub min_scale: f32,
    /// Maximum island scale (prevents huge islands)
    pub max_scale: f32,
}

impl Default for NormalizeTexelDensity {
    fn default() -> Self {
        Self {
            texels_per_unit: 256.0,
            min_scale: 0.1,
            max_scale: 10.0,
        }
    }
}

impl MeshModifier for NormalizeTexelDensity {
    fn apply(&self, mesh: &mut UnpackedMesh) {
        if mesh.uvs.is_empty() || mesh.indices.len() < 3 {
            return;
        }

        // Calculate current texel density for each triangle
        let mut scale_factors: Vec<f32> = Vec::new();

        for chunk in mesh.indices.chunks(3) {
            if chunk.len() == 3 {
                let i0 = chunk[0] as usize;
                let i1 = chunk[1] as usize;
                let i2 = chunk[2] as usize;

                // World space area
                let p0 = Vec3::from(mesh.positions[i0]);
                let p1 = Vec3::from(mesh.positions[i1]);
                let p2 = Vec3::from(mesh.positions[i2]);
                let world_area = (p1 - p0).cross(p2 - p0).length() * 0.5;

                // UV space area
                let uv0 = mesh.uvs[i0];
                let uv1 = mesh.uvs[i1];
                let uv2 = mesh.uvs[i2];
                let uv_area = ((uv1[0] - uv0[0]) * (uv2[1] - uv0[1])
                    - (uv2[0] - uv0[0]) * (uv1[1] - uv0[1])).abs() * 0.5;

                if uv_area > 0.00001 && world_area > 0.00001 {
                    // Current texel density
                    let current_density = (uv_area / world_area).sqrt();
                    let target_density = 1.0 / self.texels_per_unit;
                    let scale = target_density / current_density;
                    scale_factors.push(scale.clamp(self.min_scale, self.max_scale));
                }
            }
        }

        // Apply average scale to all UVs (simple version)
        // A more sophisticated version would handle UV islands separately
        if !scale_factors.is_empty() {
            let avg_scale: f32 = scale_factors.iter().sum::<f32>() / scale_factors.len() as f32;

            // Find UV center
            let mut center_u = 0.0f32;
            let mut center_v = 0.0f32;
            for uv in &mesh.uvs {
                center_u += uv[0];
                center_v += uv[1];
            }
            center_u /= mesh.uvs.len() as f32;
            center_v /= mesh.uvs.len() as f32;

            // Scale around center
            for uv in &mut mesh.uvs {
                uv[0] = center_u + (uv[0] - center_u) * avg_scale;
                uv[1] = center_v + (uv[1] - center_v) * avg_scale;
            }
        }
    }
}

/// UV projection mode
#[derive(Clone, Copy, Default)]
pub enum UVProjection {
    /// Project from +X axis
    PlanarX,
    /// Project from +Y axis
    PlanarY,
    /// Project from +Z axis
    #[default]
    PlanarZ,
    /// Cylindrical projection around Y axis
    Cylindrical,
    /// Spherical projection
    Spherical,
    /// Box/triplanar projection (uses dominant normal axis)
    Box,
}

/// Apply UV projection to mesh
pub struct ProjectUVs {
    /// Projection mode
    pub projection: UVProjection,
    /// Scale factor
    pub scale: f32,
    /// Offset
    pub offset: [f32; 2],
}

impl Default for ProjectUVs {
    fn default() -> Self {
        Self {
            projection: UVProjection::PlanarZ,
            scale: 1.0,
            offset: [0.0, 0.0],
        }
    }
}

impl MeshModifier for ProjectUVs {
    fn apply(&self, mesh: &mut UnpackedMesh) {
        if mesh.positions.is_empty() {
            return;
        }

        // Ensure UV array exists
        mesh.uvs.resize(mesh.positions.len(), [0.0, 0.0]);

        // Calculate bounds for normalization
        let bounds = calculate_bounds(mesh);
        let size = bounds.1 - bounds.0;
        let center = (bounds.0 + bounds.1) * 0.5;

        for i in 0..mesh.positions.len() {
            let pos = Vec3::from(mesh.positions[i]);
            let normal = Vec3::from(mesh.normals[i]).normalize();
            let local = pos - center;

            let (u, v) = match self.projection {
                UVProjection::PlanarX => (local.z / size.z, local.y / size.y),
                UVProjection::PlanarY => (local.x / size.x, local.z / size.z),
                UVProjection::PlanarZ => (local.x / size.x, local.y / size.y),
                UVProjection::Cylindrical => {
                    let angle = local.x.atan2(local.z);
                    let u = (angle / std::f32::consts::PI + 1.0) * 0.5;
                    let v = local.y / size.y + 0.5;
                    (u, v)
                }
                UVProjection::Spherical => {
                    let normalized = local.normalize();
                    let u = (normalized.x.atan2(normalized.z) / std::f32::consts::PI + 1.0) * 0.5;
                    let v = normalized.y * 0.5 + 0.5;
                    (u, v)
                }
                UVProjection::Box => {
                    // Use dominant normal axis
                    let abs_normal = Vec3::new(normal.x.abs(), normal.y.abs(), normal.z.abs());
                    if abs_normal.x > abs_normal.y && abs_normal.x > abs_normal.z {
                        (local.z / size.z + 0.5, local.y / size.y + 0.5)
                    } else if abs_normal.y > abs_normal.z {
                        (local.x / size.x + 0.5, local.z / size.z + 0.5)
                    } else {
                        (local.x / size.x + 0.5, local.y / size.y + 0.5)
                    }
                }
            };

            mesh.uvs[i] = [
                u * self.scale + self.offset[0],
                v * self.scale + self.offset[1],
            ];
        }
    }
}

/// Fix cylindrical UV seams
///
/// Cylindrical projection creates a seam where U wraps from 1 to 0.
/// This modifier duplicates vertices along the seam to prevent texture stretching.
pub struct FixCylindricalSeam {
    /// U threshold for detecting seam (vertices near 0 and 1)
    pub threshold: f32,
}

impl Default for FixCylindricalSeam {
    fn default() -> Self {
        Self { threshold: 0.1 }
    }
}

impl MeshModifier for FixCylindricalSeam {
    fn apply(&self, mesh: &mut UnpackedMesh) {
        if mesh.uvs.is_empty() || mesh.indices.len() < 3 {
            return;
        }

        let mut new_positions = mesh.positions.clone();
        let mut new_normals = mesh.normals.clone();
        let mut new_uvs = mesh.uvs.clone();
        let mut new_colors = mesh.colors.clone();
        let mut new_indices = Vec::new();

        // Process each triangle
        for chunk in mesh.indices.chunks(3) {
            if chunk.len() != 3 {
                continue;
            }

            let mut tri_indices = [chunk[0] as usize, chunk[1] as usize, chunk[2] as usize];

            // Check for seam crossing (big U difference)
            let u0 = mesh.uvs[tri_indices[0]][0];
            let u1 = mesh.uvs[tri_indices[1]][0];
            let u2 = mesh.uvs[tri_indices[2]][0];

            let u_min = u0.min(u1).min(u2);
            let u_max = u0.max(u1).max(u2);

            if u_max - u_min > 0.5 {
                // This triangle crosses the seam
                // Duplicate vertices on the "wrong" side

                for j in 0..3 {
                    let u = mesh.uvs[tri_indices[j]][0];

                    // If this vertex is on the low side of the seam
                    if u < self.threshold {
                        // Duplicate with U shifted by 1
                        let new_idx = new_positions.len();
                        new_positions.push(mesh.positions[tri_indices[j]]);
                        new_normals.push(mesh.normals[tri_indices[j]]);
                        new_uvs.push([mesh.uvs[tri_indices[j]][0] + 1.0, mesh.uvs[tri_indices[j]][1]]);
                        if !mesh.colors.is_empty() {
                            new_colors.push(mesh.colors[tri_indices[j]]);
                        }
                        tri_indices[j] = new_idx;
                    }
                }
            }

            new_indices.push(tri_indices[0] as u16);
            new_indices.push(tri_indices[1] as u16);
            new_indices.push(tri_indices[2] as u16);
        }

        mesh.positions = new_positions;
        mesh.normals = new_normals;
        mesh.uvs = new_uvs;
        mesh.colors = new_colors;
        mesh.indices = new_indices;
    }
}

/// Scale UVs
pub struct ScaleUVs {
    pub scale_u: f32,
    pub scale_v: f32,
}

impl Default for ScaleUVs {
    fn default() -> Self {
        Self {
            scale_u: 1.0,
            scale_v: 1.0,
        }
    }
}

impl MeshModifier for ScaleUVs {
    fn apply(&self, mesh: &mut UnpackedMesh) {
        for uv in &mut mesh.uvs {
            uv[0] *= self.scale_u;
            uv[1] *= self.scale_v;
        }
    }
}

/// Offset UVs
pub struct OffsetUVs {
    pub offset_u: f32,
    pub offset_v: f32,
}

impl Default for OffsetUVs {
    fn default() -> Self {
        Self {
            offset_u: 0.0,
            offset_v: 0.0,
        }
    }
}

impl MeshModifier for OffsetUVs {
    fn apply(&self, mesh: &mut UnpackedMesh) {
        for uv in &mut mesh.uvs {
            uv[0] += self.offset_u;
            uv[1] += self.offset_v;
        }
    }
}

/// Rotate UVs around center
pub struct RotateUVs {
    /// Rotation angle in degrees
    pub angle: f32,
}

impl Default for RotateUVs {
    fn default() -> Self {
        Self { angle: 0.0 }
    }
}

impl MeshModifier for RotateUVs {
    fn apply(&self, mesh: &mut UnpackedMesh) {
        if mesh.uvs.is_empty() {
            return;
        }

        // Find UV center
        let mut center_u = 0.0f32;
        let mut center_v = 0.0f32;
        for uv in &mesh.uvs {
            center_u += uv[0];
            center_v += uv[1];
        }
        center_u /= mesh.uvs.len() as f32;
        center_v /= mesh.uvs.len() as f32;

        let rad = self.angle.to_radians();
        let cos_a = rad.cos();
        let sin_a = rad.sin();

        for uv in &mut mesh.uvs {
            let u = uv[0] - center_u;
            let v = uv[1] - center_v;

            uv[0] = u * cos_a - v * sin_a + center_u;
            uv[1] = u * sin_a + v * cos_a + center_v;
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

    if min.x > max.x {
        // Empty mesh
        (Vec3::ZERO, Vec3::ONE)
    } else {
        (min, max)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nethercore_zx::procedural::generate_cube_uv;

    #[test]
    fn test_pixel_snap_uvs() {
        let mut mesh: UnpackedMesh = generate_cube_uv(1.0, 1.0, 1.0);

        PixelSnapUVs { resolution: 16, half_pixel_offset: false }.apply(&mut mesh);

        // All UVs should be on 1/16 grid
        let texel = 1.0 / 16.0;
        for uv in &mesh.uvs {
            let u_snapped = (uv[0] / texel).round() * texel;
            let v_snapped = (uv[1] / texel).round() * texel;
            assert!((uv[0] - u_snapped).abs() < 0.001, "U not snapped: {}", uv[0]);
            assert!((uv[1] - v_snapped).abs() < 0.001, "V not snapped: {}", uv[1]);
        }
    }

    #[test]
    fn test_project_uvs_planar() {
        let mut mesh: UnpackedMesh = generate_cube_uv(1.0, 1.0, 1.0);

        ProjectUVs {
            projection: UVProjection::PlanarZ,
            scale: 1.0,
            offset: [0.0, 0.0],
        }.apply(&mut mesh);

        // All UVs should be valid
        for uv in &mesh.uvs {
            assert!(uv[0].is_finite());
            assert!(uv[1].is_finite());
        }
    }

    #[test]
    fn test_scale_uvs() {
        let mut mesh: UnpackedMesh = generate_cube_uv(1.0, 1.0, 1.0);
        let original = mesh.uvs.clone();

        ScaleUVs { scale_u: 2.0, scale_v: 0.5 }.apply(&mut mesh);

        for (i, uv) in mesh.uvs.iter().enumerate() {
            assert!((uv[0] - original[i][0] * 2.0).abs() < 0.001);
            assert!((uv[1] - original[i][1] * 0.5).abs() < 0.001);
        }
    }

    #[test]
    fn test_rotate_uvs() {
        let mut mesh: UnpackedMesh = generate_cube_uv(1.0, 1.0, 1.0);

        RotateUVs { angle: 90.0 }.apply(&mut mesh);

        // UVs should still be valid after rotation
        for uv in &mesh.uvs {
            assert!(uv[0].is_finite());
            assert!(uv[1].is_finite());
        }
    }
}
