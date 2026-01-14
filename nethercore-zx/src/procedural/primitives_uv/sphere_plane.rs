//! Simple parametric primitives with UV coordinates (sphere, plane)

use glam::Vec3;
use std::f32::consts::PI;
use tracing::warn;

use crate::procedural::types::MeshBuilderUV;

/// Generate a UV sphere mesh with smooth normals and equirectangular UV mapping
///
/// # Arguments
/// * `radius` - Sphere radius
/// * `segments` - Number of longitudinal divisions (min 3, max 256)
/// * `rings` - Number of latitudinal divisions (min 2, max 256)
///
/// # Returns
/// Mesh with `(rings + 1) × (segments + 1)` vertices, Format 5 (POS_UV_NORMAL)
///
/// # UV Mapping
/// - U (horizontal): Longitude (theta) wraps 0→1 around equator
/// - V (vertical): Latitude (phi) maps 0→1 from north pole to south pole
///
/// Note: Includes duplicate seam vertices at U=1.0 for correct texture wrapping.
pub fn generate_sphere_uv<M: MeshBuilderUV + Default>(radius: f32, segments: u32, rings: u32) -> M {
    // Validate and clamp parameters
    let radius = if radius <= 0.0 {
        warn!("generate_sphere_uv: radius must be > 0.0, clamping to 0.001");
        0.001
    } else {
        radius
    };

    let segments = segments.clamp(3, 256);
    let rings = rings.clamp(2, 256);

    let mut mesh = M::default();

    // Generate vertices with equirectangular UV mapping
    // Note: We generate segments+1 vertices per ring to create proper UV seam
    // The last column (seg=segments) has U=1.0 and duplicates positions of seg=0
    for ring in 0..=rings {
        let phi = (ring as f32 / rings as f32) * PI; // 0 to PI (north pole to south pole)
        let v = ring as f32 / rings as f32; // V coordinate: 0 at north pole, 1 at south pole
        let y = radius * phi.cos();
        let ring_radius = radius * phi.sin();

        for seg in 0..=segments {
            let theta = (seg as f32 / segments as f32) * 2.0 * PI; // 0 to 2PI
            let u = seg as f32 / segments as f32; // U coordinate: 0 to 1.0 inclusive
            let x = ring_radius * theta.cos();
            let z = ring_radius * theta.sin();

            let position = Vec3::new(x, y, z);
            let normal = position.normalize(); // Smooth normals point from center

            mesh.add_vertex_uv(position, (u, v), normal);
        }
    }

    // Generate indices
    // With segments+1 vertices per ring, we connect seg to seg+1 without modular wrap
    let verts_per_ring = segments + 1;
    for ring in 0..rings {
        for seg in 0..segments {
            let i0 = (ring * verts_per_ring + seg) as u16;
            let i1 = (ring * verts_per_ring + seg + 1) as u16;
            let i2 = ((ring + 1) * verts_per_ring + seg) as u16;
            let i3 = ((ring + 1) * verts_per_ring + seg + 1) as u16;

            // Two triangles per quad (CCW winding for outward-facing normals)
            mesh.add_triangle(i0, i1, i3);
            mesh.add_triangle(i0, i3, i2);
        }
    }

    mesh
}

/// Generate a plane mesh with UVs on the XZ plane (Y=0)
///
/// # Arguments
/// * `size_x` - Width along X axis
/// * `size_z` - Depth along Z axis
/// * `subdivisions_x` - Number of X subdivisions (min 1, max 256)
/// * `subdivisions_z` - Number of Z subdivisions (min 1, max 256)
///
/// # Returns
/// Mesh with `(subdivisions_x + 1) × (subdivisions_z + 1)` vertices, Format 5 (POS_UV_NORMAL)
///
/// # UV Mapping
/// - U maps 0→1 along X axis (left to right)
/// - V maps 0→1 along Z axis (front to back)
pub fn generate_plane_uv<M: MeshBuilderUV + Default>(
    size_x: f32,
    size_z: f32,
    subdivisions_x: u32,
    subdivisions_z: u32,
) -> M {
    // Validate and clamp parameters
    let size_x = if size_x <= 0.0 {
        warn!("generate_plane_uv: size_x must be > 0.0, clamping to 0.001");
        0.001
    } else {
        size_x
    };

    let size_z = if size_z <= 0.0 {
        warn!("generate_plane_uv: size_z must be > 0.0, clamping to 0.001");
        0.001
    } else {
        size_z
    };

    let subdivisions_x = subdivisions_x.clamp(1, 256);
    let subdivisions_z = subdivisions_z.clamp(1, 256);

    let mut mesh = M::default();
    let normal = Vec3::new(0.0, 1.0, 0.0); // Up

    // Generate vertices with UVs
    for z in 0..=subdivisions_z {
        for x in 0..=subdivisions_x {
            let u = x as f32 / subdivisions_x as f32; // 0-1 along X
            let v = z as f32 / subdivisions_z as f32; // 0-1 along Z

            let pos_x = -size_x * 0.5 + u * size_x;
            let pos_z = -size_z * 0.5 + v * size_z;

            let position = Vec3::new(pos_x, 0.0, pos_z);
            mesh.add_vertex_uv(position, (u, v), normal);
        }
    }

    // Generate indices (same topology as non-UV plane)
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
