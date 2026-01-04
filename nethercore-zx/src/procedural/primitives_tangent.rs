//! Procedural mesh primitives with tangent data for normal mapping
//!
//! Functions for generating common 3D primitives with normals, UVs, and tangent data.
//! These are suitable for normal-mapped rendering.

use glam::Vec3;
use std::f32::consts::PI;
use tracing::warn;

use super::types::MeshBuilderTangent;

/// Generate a UV sphere mesh with tangent data for normal mapping
///
/// # Arguments
/// * `radius` - Sphere radius
/// * `segments` - Number of longitudinal divisions (min 3, max 256)
/// * `rings` - Number of latitudinal divisions (min 2, max 256)
///
/// # Returns
/// Mesh with tangent data, Format 21 (POS_UV_NORMAL_TANGENT)
///
/// # Tangent Calculation
/// Tangent follows the direction of increasing U (longitude)
/// Bitangent follows the direction of increasing V (latitude)
pub fn generate_sphere_tangent<M: MeshBuilderTangent + Default>(
    radius: f32,
    segments: u32,
    rings: u32,
) -> M {
    // Validate and clamp parameters
    let radius = if radius <= 0.0 {
        warn!("generate_sphere_tangent: radius must be > 0.0, clamping to 0.001");
        0.001
    } else {
        radius
    };

    let segments = segments.clamp(3, 256);
    let rings = rings.clamp(2, 256);

    let mut mesh = M::default();

    // Generate vertices with tangent data
    for ring in 0..=rings {
        let phi = (ring as f32 / rings as f32) * PI; // 0 to PI (north pole to south pole)
        let v = ring as f32 / rings as f32;
        let y = radius * phi.cos();
        let ring_radius = radius * phi.sin();

        for seg in 0..=segments {
            let theta = (seg as f32 / segments as f32) * 2.0 * PI; // 0 to 2PI
            let u = seg as f32 / segments as f32;
            let x = ring_radius * theta.cos();
            let z = ring_radius * theta.sin();

            let position = Vec3::new(x, y, z);
            let normal = position.normalize();

            // Tangent is perpendicular to normal in the XZ plane (follows longitude)
            // At poles, we need a fallback tangent
            let tangent = if ring == 0 || ring == rings {
                // At poles, use a consistent tangent
                Vec3::new(1.0, 0.0, 0.0)
            } else {
                // Tangent follows the direction of increasing U (theta)
                // d/dtheta of (cos(theta), 0, sin(theta)) = (-sin(theta), 0, cos(theta))
                Vec3::new(-theta.sin(), 0.0, theta.cos()).normalize()
            };

            // Handedness: bitangent = cross(normal, tangent) for right-handed
            // For sphere, we use +1.0 handedness
            let handedness = 1.0;

            mesh.add_vertex_tangent(position, (u, v), normal, tangent, handedness);
        }
    }

    // Generate indices (same as UV sphere)
    let verts_per_ring = segments + 1;
    for ring in 0..rings {
        for seg in 0..segments {
            let i0 = (ring * verts_per_ring + seg) as u16;
            let i1 = (ring * verts_per_ring + seg + 1) as u16;
            let i2 = ((ring + 1) * verts_per_ring + seg) as u16;
            let i3 = ((ring + 1) * verts_per_ring + seg + 1) as u16;

            // Two triangles per quad (CCW winding)
            mesh.add_triangle(i0, i1, i3);
            mesh.add_triangle(i0, i3, i2);
        }
    }

    mesh
}

/// Generate a plane mesh with tangent data for normal mapping
///
/// # Arguments
/// * `size_x` - Width along X axis
/// * `size_z` - Depth along Z axis
/// * `subdivisions_x` - Number of X subdivisions (min 1, max 256)
/// * `subdivisions_z` - Number of Z subdivisions (min 1, max 256)
///
/// # Returns
/// Mesh with tangent data, Format 21 (POS_UV_NORMAL_TANGENT)
///
/// # Tangent Calculation
/// Tangent points along +X (direction of increasing U)
/// Bitangent points along +Z (direction of increasing V)
/// Normal points along +Y
pub fn generate_plane_tangent<M: MeshBuilderTangent + Default>(
    size_x: f32,
    size_z: f32,
    subdivisions_x: u32,
    subdivisions_z: u32,
) -> M {
    // Validate and clamp parameters
    let size_x = if size_x <= 0.0 {
        warn!("generate_plane_tangent: size_x must be > 0.0, clamping to 0.001");
        0.001
    } else {
        size_x
    };

    let size_z = if size_z <= 0.0 {
        warn!("generate_plane_tangent: size_z must be > 0.0, clamping to 0.001");
        0.001
    } else {
        size_z
    };

    let subdivisions_x = subdivisions_x.clamp(1, 256);
    let subdivisions_z = subdivisions_z.clamp(1, 256);

    let mut mesh = M::default();

    // Plane has constant normal (+Y) and tangent (+X)
    let normal = Vec3::new(0.0, 1.0, 0.0);
    let tangent = Vec3::new(1.0, 0.0, 0.0);
    let handedness = 1.0; // Bitangent = +Z

    // Generate vertices with tangent data
    for z in 0..=subdivisions_z {
        for x in 0..=subdivisions_x {
            let u = x as f32 / subdivisions_x as f32;
            let v = z as f32 / subdivisions_z as f32;

            let pos_x = -size_x * 0.5 + u * size_x;
            let pos_z = -size_z * 0.5 + v * size_z;

            let position = Vec3::new(pos_x, 0.0, pos_z);
            mesh.add_vertex_tangent(position, (u, v), normal, tangent, handedness);
        }
    }

    // Generate indices (same as UV plane)
    for z in 0..subdivisions_z {
        for x in 0..subdivisions_x {
            let i0 = (z * (subdivisions_x + 1) + x) as u16;
            let i1 = i0 + 1;
            let i2 = ((z + 1) * (subdivisions_x + 1) + x) as u16;
            let i3 = i2 + 1;

            // Two triangles per quad (CCW winding for +Y normal)
            mesh.add_triangle(i0, i2, i1);
            mesh.add_triangle(i1, i2, i3);
        }
    }

    mesh
}

/// Generate a cube mesh with tangent data for normal mapping
///
/// # Arguments
/// * `size_x` - Half-extent along X axis
/// * `size_y` - Half-extent along Y axis
/// * `size_z` - Half-extent along Z axis
///
/// # Returns
/// Mesh with 24 vertices (4 per face), Format 21 (POS_UV_NORMAL_TANGENT)
pub fn generate_cube_tangent<M: MeshBuilderTangent + Default>(
    size_x: f32,
    size_y: f32,
    size_z: f32,
) -> M {
    // Validate and clamp parameters
    let size_x = if size_x <= 0.0 {
        warn!("generate_cube_tangent: size_x must be > 0.0, clamping to 0.001");
        0.001
    } else {
        size_x
    };

    let size_y = if size_y <= 0.0 {
        warn!("generate_cube_tangent: size_y must be > 0.0, clamping to 0.001");
        0.001
    } else {
        size_y
    };

    let size_z = if size_z <= 0.0 {
        warn!("generate_cube_tangent: size_z must be > 0.0, clamping to 0.001");
        0.001
    } else {
        size_z
    };

    let mut mesh = M::default();

    // Face data: (normal, tangent, handedness, 4 vertex positions, 4 UVs)
    // For each face, tangent follows U direction, bitangent = normal × tangent * handedness follows V
    let faces: [(Vec3, Vec3, f32, [(f32, f32, f32); 4], [(f32, f32); 4]); 6] = [
        // Front (+Z): normal=+Z, tangent=+X, bitangent=+Y
        (
            Vec3::Z,
            Vec3::X,
            1.0,
            [
                (-size_x, -size_y, size_z),
                (size_x, -size_y, size_z),
                (size_x, size_y, size_z),
                (-size_x, size_y, size_z),
            ],
            [(0.0, 1.0), (1.0, 1.0), (1.0, 0.0), (0.0, 0.0)],
        ),
        // Back (-Z): normal=-Z, tangent=-X, bitangent=+Y
        (
            Vec3::NEG_Z,
            Vec3::NEG_X,
            1.0,
            [
                (size_x, -size_y, -size_z),
                (-size_x, -size_y, -size_z),
                (-size_x, size_y, -size_z),
                (size_x, size_y, -size_z),
            ],
            [(0.0, 1.0), (1.0, 1.0), (1.0, 0.0), (0.0, 0.0)],
        ),
        // Top (+Y): normal=+Y, tangent=+X, bitangent=-Z
        (
            Vec3::Y,
            Vec3::X,
            -1.0,
            [
                (-size_x, size_y, size_z),
                (size_x, size_y, size_z),
                (size_x, size_y, -size_z),
                (-size_x, size_y, -size_z),
            ],
            [(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 1.0)],
        ),
        // Bottom (-Y): normal=-Y, tangent=+X, bitangent=+Z
        (
            Vec3::NEG_Y,
            Vec3::X,
            1.0,
            [
                (-size_x, -size_y, -size_z),
                (size_x, -size_y, -size_z),
                (size_x, -size_y, size_z),
                (-size_x, -size_y, size_z),
            ],
            [(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 1.0)],
        ),
        // Right (+X): normal=+X, tangent=-Z, bitangent=+Y
        (
            Vec3::X,
            Vec3::NEG_Z,
            1.0,
            [
                (size_x, -size_y, size_z),
                (size_x, -size_y, -size_z),
                (size_x, size_y, -size_z),
                (size_x, size_y, size_z),
            ],
            [(0.0, 1.0), (1.0, 1.0), (1.0, 0.0), (0.0, 0.0)],
        ),
        // Left (-X): normal=-X, tangent=+Z, bitangent=+Y
        (
            Vec3::NEG_X,
            Vec3::Z,
            1.0,
            [
                (-size_x, -size_y, -size_z),
                (-size_x, -size_y, size_z),
                (-size_x, size_y, size_z),
                (-size_x, size_y, -size_z),
            ],
            [(0.0, 1.0), (1.0, 1.0), (1.0, 0.0), (0.0, 0.0)],
        ),
    ];

    for (normal, tangent, handedness, positions, uvs) in &faces {
        let base_idx = (mesh.add_vertex_tangent(
            Vec3::new(positions[0].0, positions[0].1, positions[0].2),
            uvs[0],
            *normal,
            *tangent,
            *handedness,
        )) as u16;

        mesh.add_vertex_tangent(
            Vec3::new(positions[1].0, positions[1].1, positions[1].2),
            uvs[1],
            *normal,
            *tangent,
            *handedness,
        );
        mesh.add_vertex_tangent(
            Vec3::new(positions[2].0, positions[2].1, positions[2].2),
            uvs[2],
            *normal,
            *tangent,
            *handedness,
        );
        mesh.add_vertex_tangent(
            Vec3::new(positions[3].0, positions[3].1, positions[3].2),
            uvs[3],
            *normal,
            *tangent,
            *handedness,
        );

        // Two triangles per face (CCW winding)
        mesh.add_triangle(base_idx, base_idx + 1, base_idx + 2);
        mesh.add_triangle(base_idx, base_idx + 2, base_idx + 3);
    }

    mesh
}

/// Generate a torus mesh with tangent data for normal mapping
///
/// # Arguments
/// * `major_radius` - Distance from torus center to tube center
/// * `minor_radius` - Tube radius
/// * `major_segments` - Segments around major circle (min 3, max 256)
/// * `minor_segments` - Segments around tube (min 3, max 256)
///
/// # Returns
/// Mesh with tangent data, Format 21 (POS_UV_NORMAL_TANGENT)
pub fn generate_torus_tangent<M: MeshBuilderTangent + Default>(
    major_radius: f32,
    minor_radius: f32,
    major_segments: u32,
    minor_segments: u32,
) -> M {
    // Validate and clamp parameters
    let major_radius = if major_radius <= 0.0 {
        warn!("generate_torus_tangent: major_radius must be > 0.0, clamping to 0.001");
        0.001
    } else {
        major_radius
    };

    let minor_radius = if minor_radius <= 0.0 {
        warn!("generate_torus_tangent: minor_radius must be > 0.0, clamping to 0.001");
        0.001
    } else {
        minor_radius
    };

    let major_segments = major_segments.clamp(3, 256);
    let minor_segments = minor_segments.clamp(3, 256);

    let mut mesh = M::default();

    // Generate vertices
    for major in 0..=major_segments {
        let theta = (major as f32 / major_segments as f32) * 2.0 * PI;
        let u = major as f32 / major_segments as f32;

        // Center of the tube at this major segment
        let tube_center = Vec3::new(major_radius * theta.cos(), 0.0, major_radius * theta.sin());

        // Tangent follows the major circle (direction of increasing U)
        let tangent = Vec3::new(-theta.sin(), 0.0, theta.cos());

        for minor in 0..=minor_segments {
            let phi = (minor as f32 / minor_segments as f32) * 2.0 * PI;
            let v = minor as f32 / minor_segments as f32;

            // Normal points outward from tube center
            let tube_normal = Vec3::new(theta.cos() * phi.cos(), phi.sin(), theta.sin() * phi.cos());

            let position = tube_center + tube_normal * minor_radius;
            let normal = tube_normal.normalize();

            // Handedness: +1.0 for right-handed TBN
            let handedness = 1.0;

            mesh.add_vertex_tangent(position, (u, v), normal, tangent, handedness);
        }
    }

    // Generate indices
    let verts_per_ring = minor_segments + 1;
    for major in 0..major_segments {
        for minor in 0..minor_segments {
            let i0 = (major * verts_per_ring + minor) as u16;
            let i1 = (major * verts_per_ring + minor + 1) as u16;
            let i2 = ((major + 1) * verts_per_ring + minor) as u16;
            let i3 = ((major + 1) * verts_per_ring + minor + 1) as u16;

            // Two triangles per quad (CCW winding)
            mesh.add_triangle(i0, i1, i3);
            mesh.add_triangle(i0, i3, i2);
        }
    }

    mesh
}
