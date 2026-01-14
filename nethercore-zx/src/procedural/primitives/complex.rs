//! Complex primitive shapes: cylinder, capsule

use glam::Vec3;
use std::f32::consts::PI;
use tracing::warn;

use crate::procedural::types::MeshBuilder;

use super::simple::generate_sphere;

/// Generate a cylinder or cone mesh
///
/// # Arguments
/// * `radius_bottom` - Bottom radius (>= 0.0)
/// * `radius_top` - Top radius (>= 0.0)
/// * `height` - Cylinder height
/// * `segments` - Number of radial divisions (min 3, max 256)
///
/// # Returns
/// Mesh with body and caps (if radii > 0)
pub fn generate_cylinder<M: MeshBuilder + Default>(
    radius_bottom: f32,
    radius_top: f32,
    height: f32,
    segments: u32,
) -> M {
    // Validate and clamp parameters
    let radius_bottom = if radius_bottom < 0.0 {
        warn!("generate_cylinder: radius_bottom must be >= 0.0, clamping to 0.0");
        0.0
    } else {
        radius_bottom
    };

    let radius_top = if radius_top < 0.0 {
        warn!("generate_cylinder: radius_top must be >= 0.0, clamping to 0.0");
        0.0
    } else {
        radius_top
    };

    let height = if height <= 0.0 {
        warn!("generate_cylinder: height must be > 0.0, clamping to 0.001");
        0.001
    } else {
        height
    };

    let segments = segments.clamp(3, 256);

    let mut mesh = M::default();
    let half_height = height * 0.5;

    // Generate body vertices (two rings: bottom and top)
    let mut body_indices = Vec::with_capacity((segments * 2) as usize);

    for i in 0..segments {
        let theta = (i as f32 / segments as f32) * 2.0 * PI;
        let cos_theta = theta.cos();
        let sin_theta = theta.sin();

        // Bottom vertex
        let bottom_pos = Vec3::new(
            radius_bottom * cos_theta,
            -half_height,
            radius_bottom * sin_theta,
        );

        // Top vertex
        let top_pos = Vec3::new(radius_top * cos_theta, half_height, radius_top * sin_theta);

        // Calculate normal for cylinder/cone surface
        // For a cone, normals tilt based on slope
        let tangent = Vec3::new(cos_theta, 0.0, sin_theta);
        let slope = Vec3::new(0.0, radius_bottom - radius_top, 0.0);
        let normal = (tangent + slope.normalize_or_zero()).normalize();

        let bottom_idx = mesh.add_vertex(bottom_pos, normal);
        let top_idx = mesh.add_vertex(top_pos, normal);
        body_indices.push(bottom_idx);
        body_indices.push(top_idx);
    }

    // Generate body indices
    for i in 0..segments {
        let next_i = (i + 1) % segments;

        let i0 = body_indices[(i * 2) as usize];
        let i1 = body_indices[(i * 2 + 1) as usize];
        let i2 = body_indices[(next_i * 2) as usize];
        let i3 = body_indices[(next_i * 2 + 1) as usize];

        // Two triangles per quad (CCW winding for outward normals)
        // Vertex layout: i0=BR, i1=TR (seg i), i2=BL, i3=TL (seg i+1)
        // CCW order when viewed from outside: i0→i1→i3→i2
        mesh.add_triangle(i0, i1, i3);
        mesh.add_triangle(i0, i3, i2);
    }

    // Generate bottom cap (if radius > 0)
    if radius_bottom > 0.0 {
        let cap_center_index =
            mesh.add_vertex(Vec3::new(0.0, -half_height, 0.0), Vec3::new(0.0, -1.0, 0.0));

        for i in 0..segments {
            let next_i = (i + 1) % segments;
            let theta = (i as f32 / segments as f32) * 2.0 * PI;
            let next_theta = (next_i as f32 / segments as f32) * 2.0 * PI;

            let i0 = mesh.add_vertex(
                Vec3::new(
                    radius_bottom * theta.cos(),
                    -half_height,
                    radius_bottom * theta.sin(),
                ),
                Vec3::new(0.0, -1.0, 0.0),
            );

            let i1 = mesh.add_vertex(
                Vec3::new(
                    radius_bottom * next_theta.cos(),
                    -half_height,
                    radius_bottom * next_theta.sin(),
                ),
                Vec3::new(0.0, -1.0, 0.0),
            );

            // CCW winding for -Y normal (viewed from below)
            mesh.add_triangle(cap_center_index, i0, i1);
        }
    }

    // Generate top cap (if radius > 0)
    if radius_top > 0.0 {
        let cap_center_index =
            mesh.add_vertex(Vec3::new(0.0, half_height, 0.0), Vec3::new(0.0, 1.0, 0.0));

        for i in 0..segments {
            let next_i = (i + 1) % segments;
            let theta = (i as f32 / segments as f32) * 2.0 * PI;
            let next_theta = (next_i as f32 / segments as f32) * 2.0 * PI;

            let i0 = mesh.add_vertex(
                Vec3::new(
                    radius_top * theta.cos(),
                    half_height,
                    radius_top * theta.sin(),
                ),
                Vec3::new(0.0, 1.0, 0.0),
            );

            let i1 = mesh.add_vertex(
                Vec3::new(
                    radius_top * next_theta.cos(),
                    half_height,
                    radius_top * next_theta.sin(),
                ),
                Vec3::new(0.0, 1.0, 0.0),
            );

            mesh.add_triangle(cap_center_index, i1, i0);
        }
    }

    mesh
}

/// Generate a capsule mesh (cylinder with hemispherical caps)
///
/// # Arguments
/// * `radius` - Capsule radius
/// * `height` - Height of cylindrical section (>= 0.0)
/// * `segments` - Number of radial divisions (min 3, max 256)
/// * `rings` - Number of latitudinal divisions per hemisphere (min 1, max 128)
///
/// # Returns
/// Mesh with cylinder body and two hemispheres
/// Total height = height + 2 * radius
pub fn generate_capsule<M: MeshBuilder + Default>(
    radius: f32,
    height: f32,
    segments: u32,
    rings: u32,
) -> M {
    // Validate and clamp parameters
    let radius = if radius <= 0.0 {
        warn!("generate_capsule: radius must be > 0.0, clamping to 0.001");
        0.001
    } else {
        radius
    };

    let height = if height < 0.0 {
        warn!("generate_capsule: height must be >= 0.0, clamping to 0.0");
        0.0
    } else {
        height
    };

    let segments = segments.clamp(3, 256);
    let rings = rings.clamp(1, 128);

    let mut mesh = M::default();
    let half_height = height * 0.5;

    // If height is 0, just generate a sphere
    if height == 0.0 {
        return generate_sphere(radius, segments, rings * 2);
    }

    // Generate cylinder body vertices (two rings)
    let mut body_indices = Vec::with_capacity((segments * 2) as usize);

    for i in 0..segments {
        let theta = (i as f32 / segments as f32) * 2.0 * PI;
        let cos_theta = theta.cos();
        let sin_theta = theta.sin();

        let bottom_pos = Vec3::new(radius * cos_theta, -half_height, radius * sin_theta);
        let top_pos = Vec3::new(radius * cos_theta, half_height, radius * sin_theta);

        let normal = Vec3::new(cos_theta, 0.0, sin_theta); // Radial normal

        let bottom_idx = mesh.add_vertex(bottom_pos, normal);
        let top_idx = mesh.add_vertex(top_pos, normal);
        body_indices.push(bottom_idx);
        body_indices.push(top_idx);
    }

    // Generate cylinder body indices
    for i in 0..segments {
        let next_i = (i + 1) % segments;

        let i0 = body_indices[(i * 2) as usize];
        let i1 = body_indices[(i * 2 + 1) as usize];
        let i2 = body_indices[(next_i * 2) as usize];
        let i3 = body_indices[(next_i * 2 + 1) as usize];

        // Two triangles per quad (CCW winding for outward normals)
        // Same layout as cylinder body: i0=BR, i1=TR, i2=BL, i3=TL
        // CCW order when viewed from outside: i0→i1→i3→i2
        mesh.add_triangle(i0, i1, i3);
        mesh.add_triangle(i0, i3, i2);
    }

    // Track starting index for top hemisphere
    let top_hemisphere_start = segments * 2;

    // Generate top hemisphere
    for ring in 0..=rings {
        let phi = (ring as f32 / rings as f32) * (PI * 0.5); // 0 to PI/2
        let y = half_height + radius * phi.cos();
        let ring_radius = radius * phi.sin();

        for seg in 0..segments {
            let theta = (seg as f32 / segments as f32) * 2.0 * PI;
            let x = ring_radius * theta.cos();
            let z = ring_radius * theta.sin();

            let position = Vec3::new(x, y, z);
            let sphere_center = Vec3::new(0.0, half_height, 0.0);
            let normal = (position - sphere_center).normalize();

            mesh.add_vertex(position, normal);
        }
    }

    // Generate top hemisphere indices

    for ring in 0..rings {
        for seg in 0..segments {
            let next_seg = (seg + 1) % segments;

            let i0 = (top_hemisphere_start + ring * segments + seg) as u16;
            let i1 = (top_hemisphere_start + ring * segments + next_seg) as u16;
            let i2 = (top_hemisphere_start + (ring + 1) * segments + seg) as u16;
            let i3 = (top_hemisphere_start + (ring + 1) * segments + next_seg) as u16;

            // Two triangles per quad (CCW winding for outward normals)
            // Same layout as sphere: i0=TR, i1=TL, i2=BR, i3=BL
            // CCW order when viewed from outside: i0→i1→i3→i2
            mesh.add_triangle(i0, i1, i3);
            mesh.add_triangle(i0, i3, i2);
        }
    }

    // Generate bottom hemisphere
    for ring in 0..=rings {
        let phi = (ring as f32 / rings as f32) * (PI * 0.5); // 0 to PI/2
        let y = -half_height - radius * phi.cos();
        let ring_radius = radius * phi.sin();

        for seg in 0..segments {
            let theta = (seg as f32 / segments as f32) * 2.0 * PI;
            let x = ring_radius * theta.cos();
            let z = ring_radius * theta.sin();

            let position = Vec3::new(x, y, z);
            let sphere_center = Vec3::new(0.0, -half_height, 0.0);
            let normal = (position - sphere_center).normalize();

            mesh.add_vertex(position, normal);
        }
    }

    // Generate bottom hemisphere indices
    let bottom_hemisphere_start = top_hemisphere_start + (rings + 1) * segments;

    for ring in 0..rings {
        for seg in 0..segments {
            let next_seg = (seg + 1) % segments;

            let i0 = (bottom_hemisphere_start + ring * segments + seg) as u16;
            let i1 = (bottom_hemisphere_start + ring * segments + next_seg) as u16;
            let i2 = (bottom_hemisphere_start + (ring + 1) * segments + seg) as u16;
            let i3 = (bottom_hemisphere_start + (ring + 1) * segments + next_seg) as u16;

            // Two triangles per quad (CCW winding for outward normals)
            // INVERTED layout vs sphere: ring 0 at pole (lower y), ring+1 closer to equator (higher y)
            // So i0=BR, i1=BL (lower y), i2=TR, i3=TL (higher y)
            // This pattern is correct for this inverted layout
            mesh.add_triangle(i0, i2, i1);
            mesh.add_triangle(i1, i2, i3);
        }
    }

    mesh
}
