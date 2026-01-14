//! Medium complexity primitives with UV coordinates (cube, torus)

use glam::Vec3;
use std::f32::consts::PI;
use tracing::warn;

use crate::procedural::types::MeshBuilderUV;

/// Generate a cube mesh with box-unwrapped UVs
///
/// # Arguments
/// * `size_x` - Half-extent along X axis
/// * `size_y` - Half-extent along Y axis
/// * `size_z` - Half-extent along Z axis
///
/// # Returns
/// Mesh with 24 vertices (4 per face), Format 5 (POS_UV_NORMAL)
///
/// # UV Mapping (Box Unwrap)
/// Each face gets a quadrant of UV space:
/// - Front (+Z): U [0.0, 0.5], V [0.0, 0.5]
/// - Back (-Z): U [0.5, 1.0], V [0.0, 0.5]
/// - Top (+Y): U [0.0, 0.5], V [0.5, 1.0]
/// - Bottom (-Y): U [0.5, 1.0], V [0.5, 1.0]
/// - Right (+X): Wraps to front-right corner
/// - Left (-X): Wraps to front-left corner
pub fn generate_cube_uv<M: MeshBuilderUV + Default>(size_x: f32, size_y: f32, size_z: f32) -> M {
    // Validate and clamp parameters
    let size_x = if size_x <= 0.0 {
        warn!("generate_cube_uv: size_x must be > 0.0, clamping to 0.001");
        0.001
    } else {
        size_x
    };

    let size_y = if size_y <= 0.0 {
        warn!("generate_cube_uv: size_y must be > 0.0, clamping to 0.001");
        0.001
    } else {
        size_y
    };

    let size_z = if size_z <= 0.0 {
        warn!("generate_cube_uv: size_z must be > 0.0, clamping to 0.001");
        0.001
    } else {
        size_z
    };

    let mut mesh = M::default();

    // Helper to add a quad with UVs
    #[allow(clippy::too_many_arguments)]
    let add_quad = |mesh: &mut M,
                    v0: Vec3,
                    v1: Vec3,
                    v2: Vec3,
                    v3: Vec3,
                    normal: Vec3,
                    uv0: (f32, f32),
                    uv1: (f32, f32),
                    uv2: (f32, f32),
                    uv3: (f32, f32)| {
        let i0 = mesh.add_vertex_uv(v0, uv0, normal);
        let i1 = mesh.add_vertex_uv(v1, uv1, normal);
        let i2 = mesh.add_vertex_uv(v2, uv2, normal);
        let i3 = mesh.add_vertex_uv(v3, uv3, normal);

        mesh.add_triangle(i0, i1, i2);
        mesh.add_triangle(i0, i2, i3);
    };

    // Front face (+Z): Bottom-left quadrant
    add_quad(
        &mut mesh,
        Vec3::new(-size_x, -size_y, size_z),
        Vec3::new(size_x, -size_y, size_z),
        Vec3::new(size_x, size_y, size_z),
        Vec3::new(-size_x, size_y, size_z),
        Vec3::new(0.0, 0.0, 1.0),
        (0.0, 0.0),
        (0.5, 0.0),
        (0.5, 0.5),
        (0.0, 0.5),
    );

    // Back face (-Z): Bottom-right quadrant
    add_quad(
        &mut mesh,
        Vec3::new(size_x, -size_y, -size_z),
        Vec3::new(-size_x, -size_y, -size_z),
        Vec3::new(-size_x, size_y, -size_z),
        Vec3::new(size_x, size_y, -size_z),
        Vec3::new(0.0, 0.0, -1.0),
        (0.5, 0.0),
        (1.0, 0.0),
        (1.0, 0.5),
        (0.5, 0.5),
    );

    // Top face (+Y): Top-left quadrant
    add_quad(
        &mut mesh,
        Vec3::new(-size_x, size_y, size_z),
        Vec3::new(size_x, size_y, size_z),
        Vec3::new(size_x, size_y, -size_z),
        Vec3::new(-size_x, size_y, -size_z),
        Vec3::new(0.0, 1.0, 0.0),
        (0.0, 0.5),
        (0.5, 0.5),
        (0.5, 1.0),
        (0.0, 1.0),
    );

    // Bottom face (-Y): Top-right quadrant
    add_quad(
        &mut mesh,
        Vec3::new(-size_x, -size_y, -size_z),
        Vec3::new(size_x, -size_y, -size_z),
        Vec3::new(size_x, -size_y, size_z),
        Vec3::new(-size_x, -size_y, size_z),
        Vec3::new(0.0, -1.0, 0.0),
        (0.5, 0.5),
        (1.0, 0.5),
        (1.0, 1.0),
        (0.5, 1.0),
    );

    // Right face (+X)
    add_quad(
        &mut mesh,
        Vec3::new(size_x, -size_y, size_z),
        Vec3::new(size_x, -size_y, -size_z),
        Vec3::new(size_x, size_y, -size_z),
        Vec3::new(size_x, size_y, size_z),
        Vec3::new(1.0, 0.0, 0.0),
        (0.0, 0.0),
        (1.0, 0.0),
        (1.0, 1.0),
        (0.0, 1.0),
    );

    // Left face (-X)
    add_quad(
        &mut mesh,
        Vec3::new(-size_x, -size_y, -size_z),
        Vec3::new(-size_x, -size_y, size_z),
        Vec3::new(-size_x, size_y, size_z),
        Vec3::new(-size_x, size_y, -size_z),
        Vec3::new(-1.0, 0.0, 0.0),
        (0.0, 0.0),
        (1.0, 0.0),
        (1.0, 1.0),
        (0.0, 1.0),
    );

    mesh
}

/// Generate a torus with wrapped UV mapping
///
/// # Arguments
/// * `major_radius` - Distance from torus center to tube center
/// * `minor_radius` - Tube radius
/// * `major_segments` - Segments around major circle (min 3, max 256)
/// * `minor_segments` - Segments around tube (min 3, max 256)
///
/// # Returns
/// Mesh with `(major_segments + 1) × (minor_segments + 1)` vertices, Format 5 (POS_UV_NORMAL)
///
/// # UV Mapping
/// - U wraps 0→1 around major circle (XZ plane)
/// - V wraps 0→1 around minor circle (tube cross-section)
///
/// Note: Includes duplicate seam vertices at U=1.0 and V=1.0 for correct texture wrapping.
pub fn generate_torus_uv<M: MeshBuilderUV + Default>(
    major_radius: f32,
    minor_radius: f32,
    major_segments: u32,
    minor_segments: u32,
) -> M {
    let major_radius = if major_radius <= 0.0 {
        warn!("generate_torus_uv: major_radius must be > 0.0, clamping to 0.001");
        0.001
    } else {
        major_radius
    };

    let minor_radius = if minor_radius <= 0.0 {
        warn!("generate_torus_uv: minor_radius must be > 0.0, clamping to 0.001");
        0.001
    } else {
        minor_radius
    };

    let major_segments = major_segments.clamp(3, 256);
    let minor_segments = minor_segments.clamp(3, 256);

    let mut mesh = M::default();

    // Generate vertices with wrapped UVs
    // Note: We generate (major+1) × (minor+1) vertices for proper UV seams
    for i in 0..=major_segments {
        let theta = (i as f32 / major_segments as f32) * 2.0 * PI;
        let u = i as f32 / major_segments as f32; // Major circle UV (0 to 1.0 inclusive)
        let cos_theta = theta.cos();
        let sin_theta = theta.sin();

        for j in 0..=minor_segments {
            let phi = (j as f32 / minor_segments as f32) * 2.0 * PI;
            let v = j as f32 / minor_segments as f32; // Minor circle UV (0 to 1.0 inclusive)
            let cos_phi = phi.cos();
            let sin_phi = phi.sin();

            let x = (major_radius + minor_radius * cos_phi) * cos_theta;
            let y = minor_radius * sin_phi;
            let z = (major_radius + minor_radius * cos_phi) * sin_theta;

            let position = Vec3::new(x, y, z);
            let tube_center = Vec3::new(major_radius * cos_theta, 0.0, major_radius * sin_theta);
            let normal = (position - tube_center).normalize();

            mesh.add_vertex_uv(position, (u, v), normal);
        }
    }

    // Generate indices
    // With (major+1) × (minor+1) vertices, we connect without modular wrap
    let verts_per_ring = minor_segments + 1;
    for i in 0..major_segments {
        for j in 0..minor_segments {
            let i0 = (i * verts_per_ring + j) as u16;
            let i1 = (i * verts_per_ring + j + 1) as u16;
            let i2 = ((i + 1) * verts_per_ring + j) as u16;
            let i3 = ((i + 1) * verts_per_ring + j + 1) as u16;

            mesh.add_triangle(i0, i1, i3);
            mesh.add_triangle(i0, i3, i2);
        }
    }

    mesh
}
