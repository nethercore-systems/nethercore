//! Complex primitives with UV coordinates and caps (cylinder, capsule)

use glam::Vec3;
use std::f32::consts::PI;
use tracing::warn;

use crate::procedural::types::MeshBuilderUV;

/// Generate a cylinder with cylindrical UV mapping
///
/// # Arguments
/// * `radius_bottom` - Bottom radius (>= 0.0)
/// * `radius_top` - Top radius (>= 0.0)
/// * `height` - Cylinder height
/// * `segments` - Number of radial divisions (min 3, max 256)
///
/// # Returns
/// Mesh with body and caps, Format 5 (POS_UV_NORMAL)
///
/// # UV Mapping
/// - Body: U wraps 0→1 around circumference, V maps 0→1 from bottom to top
/// - Caps: Radial mapping from center (0.5, 0.5)
///
/// Note: Includes duplicate seam vertices at U=1.0 for correct texture wrapping.
pub fn generate_cylinder_uv<M: MeshBuilderUV + Default>(
    radius_bottom: f32,
    radius_top: f32,
    height: f32,
    segments: u32,
) -> M {
    let radius_bottom = if radius_bottom < 0.0 {
        warn!("generate_cylinder_uv: radius_bottom must be >= 0.0, clamping to 0.0");
        0.0
    } else {
        radius_bottom
    };

    let radius_top = if radius_top < 0.0 {
        warn!("generate_cylinder_uv: radius_top must be >= 0.0, clamping to 0.0");
        0.0
    } else {
        radius_top
    };

    let height = if height <= 0.0 {
        warn!("generate_cylinder_uv: height must be > 0.0, clamping to 0.001");
        0.001
    } else {
        height
    };

    let segments = segments.clamp(3, 256);

    let mut mesh = M::default();
    let half_height = height * 0.5;

    // Calculate correct slant normal for tapered cylinders (cones)
    // The normal is perpendicular to the surface, pointing outward
    let radius_diff = radius_bottom - radius_top;
    let slant_length = (height * height + radius_diff * radius_diff).sqrt();
    let ny = radius_diff / slant_length; // Y component of normal (positive if bottom wider)
    let nr = height / slant_length; // Radial component of normal

    // Generate body vertices with cylindrical UVs
    // Note: We generate segments+1 vertices for proper UV seam at U=1.0
    for i in 0..=segments {
        let theta = (i as f32 / segments as f32) * 2.0 * PI;
        let u = i as f32 / segments as f32; // U from 0 to 1.0 inclusive
        let cos_theta = theta.cos();
        let sin_theta = theta.sin();

        let bottom_pos = Vec3::new(
            radius_bottom * cos_theta,
            -half_height,
            radius_bottom * sin_theta,
        );
        let top_pos = Vec3::new(radius_top * cos_theta, half_height, radius_top * sin_theta);

        // Correct slant normal: radial component scaled by nr, height component by ny
        let normal = Vec3::new(nr * cos_theta, ny, nr * sin_theta).normalize();

        mesh.add_vertex_uv(bottom_pos, (u, 0.0), normal);
        mesh.add_vertex_uv(top_pos, (u, 1.0), normal);
    }

    // Generate body indices
    // With segments+1 vertex columns, we connect i to i+1 without modular wrap
    for i in 0..segments {
        let i0 = (i * 2) as u16;
        let i1 = i0 + 1;
        let i2 = ((i + 1) * 2) as u16;
        let i3 = i2 + 1;

        mesh.add_triangle(i0, i1, i3);
        mesh.add_triangle(i0, i3, i2);
    }

    // Bottom cap (if radius > 0)
    if radius_bottom > 0.0 {
        let center_idx = mesh.add_vertex_uv(
            Vec3::new(0.0, -half_height, 0.0),
            (0.5, 0.5),
            Vec3::new(0.0, -1.0, 0.0),
        );

        for i in 0..segments {
            let next_i = (i + 1) % segments;
            let theta = (i as f32 / segments as f32) * 2.0 * PI;
            let next_theta = (next_i as f32 / segments as f32) * 2.0 * PI;

            let u0 = 0.5 + 0.5 * theta.cos();
            let v0 = 0.5 + 0.5 * theta.sin();
            let u1 = 0.5 + 0.5 * next_theta.cos();
            let v1 = 0.5 + 0.5 * next_theta.sin();

            let i0 = mesh.add_vertex_uv(
                Vec3::new(
                    radius_bottom * theta.cos(),
                    -half_height,
                    radius_bottom * theta.sin(),
                ),
                (u0, v0),
                Vec3::new(0.0, -1.0, 0.0),
            );

            let i1 = mesh.add_vertex_uv(
                Vec3::new(
                    radius_bottom * next_theta.cos(),
                    -half_height,
                    radius_bottom * next_theta.sin(),
                ),
                (u1, v1),
                Vec3::new(0.0, -1.0, 0.0),
            );

            mesh.add_triangle(center_idx, i0, i1);
        }
    }

    // Top cap (if radius > 0)
    if radius_top > 0.0 {
        let center_idx = mesh.add_vertex_uv(
            Vec3::new(0.0, half_height, 0.0),
            (0.5, 0.5),
            Vec3::new(0.0, 1.0, 0.0),
        );

        for i in 0..segments {
            let next_i = (i + 1) % segments;
            let theta = (i as f32 / segments as f32) * 2.0 * PI;
            let next_theta = (next_i as f32 / segments as f32) * 2.0 * PI;

            let u0 = 0.5 + 0.5 * theta.cos();
            let v0 = 0.5 + 0.5 * theta.sin();
            let u1 = 0.5 + 0.5 * next_theta.cos();
            let v1 = 0.5 + 0.5 * next_theta.sin();

            let i0 = mesh.add_vertex_uv(
                Vec3::new(
                    radius_top * theta.cos(),
                    half_height,
                    radius_top * theta.sin(),
                ),
                (u0, v0),
                Vec3::new(0.0, 1.0, 0.0),
            );

            let i1 = mesh.add_vertex_uv(
                Vec3::new(
                    radius_top * next_theta.cos(),
                    half_height,
                    radius_top * next_theta.sin(),
                ),
                (u1, v1),
                Vec3::new(0.0, 1.0, 0.0),
            );

            mesh.add_triangle(center_idx, i1, i0);
        }
    }

    mesh
}

/// Generate a capsule with hybrid UV mapping
///
/// # Arguments
/// * `radius` - Capsule radius
/// * `height` - Height of cylindrical section (>= 0.0)
/// * `segments` - Number of radial divisions (min 3, max 256)
/// * `rings` - Number of latitudinal divisions per hemisphere (min 1, max 128)
///
/// # Returns
/// Mesh with cylinder body and two hemispheres, Format 5 (POS_UV_NORMAL)
///
/// # UV Mapping
/// - Cylinder body: U wraps 0→1, V maps from 0.25→0.75
/// - Top hemisphere: V maps from 0.75→1.0
/// - Bottom hemisphere: V maps from 0.0→0.25
///
/// Note: Includes duplicate seam vertices at U=1.0 for correct texture wrapping.
pub fn generate_capsule_uv<M: MeshBuilderUV + Default>(
    radius: f32,
    height: f32,
    segments: u32,
    rings: u32,
) -> M {
    let radius = if radius <= 0.0 {
        warn!("generate_capsule_uv: radius must be > 0.0, clamping to 0.001");
        0.001
    } else {
        radius
    };

    let height = if height < 0.0 {
        warn!("generate_capsule_uv: height must be >= 0.0, clamping to 0.0");
        0.0
    } else {
        height
    };

    let segments = segments.clamp(3, 256);
    let rings = rings.clamp(1, 128);

    let mut mesh = M::default();
    let half_height = height * 0.5;

    // If height is 0, just generate a sphere with full UV range
    if height == 0.0 {
        return super::sphere_plane::generate_sphere_uv(radius, segments, rings * 2);
    }

    // Generate cylinder body (V range: 0.25 to 0.75)
    // Note: We generate segments+1 vertices for proper UV seam at U=1.0
    for i in 0..=segments {
        let theta = (i as f32 / segments as f32) * 2.0 * PI;
        let u = i as f32 / segments as f32; // U from 0 to 1.0 inclusive
        let cos_theta = theta.cos();
        let sin_theta = theta.sin();

        let bottom_pos = Vec3::new(radius * cos_theta, -half_height, radius * sin_theta);
        let top_pos = Vec3::new(radius * cos_theta, half_height, radius * sin_theta);
        let normal = Vec3::new(cos_theta, 0.0, sin_theta);

        mesh.add_vertex_uv(bottom_pos, (u, 0.25), normal);
        mesh.add_vertex_uv(top_pos, (u, 0.75), normal);
    }

    // Generate cylinder body indices
    // With segments+1 vertex columns, we connect i to i+1 without modular wrap
    for i in 0..segments {
        let i0 = (i * 2) as u16;
        let i1 = i0 + 1;
        let i2 = ((i + 1) * 2) as u16;
        let i3 = i2 + 1;

        mesh.add_triangle(i0, i1, i3);
        mesh.add_triangle(i0, i3, i2);
    }

    // Top hemisphere (V range: 0.75 to 1.0)
    // Note: We generate segments+1 vertices per ring for proper UV seam
    let verts_per_ring = segments + 1;
    for ring in 0..=rings {
        let phi = (ring as f32 / rings as f32) * (PI * 0.5);
        let v = 0.75 + 0.25 * (ring as f32 / rings as f32); // Map to 0.75-1.0
        let y = half_height + radius * phi.cos();
        let ring_radius = radius * phi.sin();

        for seg in 0..=segments {
            let theta = (seg as f32 / segments as f32) * 2.0 * PI;
            let u = seg as f32 / segments as f32; // U from 0 to 1.0 inclusive
            let x = ring_radius * theta.cos();
            let z = ring_radius * theta.sin();

            let position = Vec3::new(x, y, z);
            let sphere_center = Vec3::new(0.0, half_height, 0.0);
            let normal = (position - sphere_center).normalize();

            mesh.add_vertex_uv(position, (u, v), normal);
        }
    }

    // Top hemisphere indices
    // Body has (segments+1) * 2 vertices
    let top_hemi_start = verts_per_ring * 2;
    for ring in 0..rings {
        for seg in 0..segments {
            let i0 = (top_hemi_start + ring * verts_per_ring + seg) as u16;
            let i1 = (top_hemi_start + ring * verts_per_ring + seg + 1) as u16;
            let i2 = (top_hemi_start + (ring + 1) * verts_per_ring + seg) as u16;
            let i3 = (top_hemi_start + (ring + 1) * verts_per_ring + seg + 1) as u16;

            mesh.add_triangle(i0, i1, i3);
            mesh.add_triangle(i0, i3, i2);
        }
    }

    // Bottom hemisphere (V range: 0.0 to 0.25)
    // Note: We generate segments+1 vertices per ring for proper UV seam
    for ring in 0..=rings {
        let phi = (ring as f32 / rings as f32) * (PI * 0.5);
        let v = 0.25 * (1.0 - ring as f32 / rings as f32); // Map to 0.25-0.0
        let y = -half_height - radius * phi.cos();
        let ring_radius = radius * phi.sin();

        for seg in 0..=segments {
            let theta = (seg as f32 / segments as f32) * 2.0 * PI;
            let u = seg as f32 / segments as f32; // U from 0 to 1.0 inclusive
            let x = ring_radius * theta.cos();
            let z = ring_radius * theta.sin();

            let position = Vec3::new(x, y, z);
            let sphere_center = Vec3::new(0.0, -half_height, 0.0);
            let normal = (position - sphere_center).normalize();

            mesh.add_vertex_uv(position, (u, v), normal);
        }
    }

    // Bottom hemisphere indices
    let bottom_hemi_start = top_hemi_start + (rings + 1) * verts_per_ring;
    for ring in 0..rings {
        for seg in 0..segments {
            let i0 = (bottom_hemi_start + ring * verts_per_ring + seg) as u16;
            let i1 = (bottom_hemi_start + ring * verts_per_ring + seg + 1) as u16;
            let i2 = (bottom_hemi_start + (ring + 1) * verts_per_ring + seg) as u16;
            let i3 = (bottom_hemi_start + (ring + 1) * verts_per_ring + seg + 1) as u16;

            mesh.add_triangle(i0, i2, i1);
            mesh.add_triangle(i1, i2, i3);
        }
    }

    mesh
}
