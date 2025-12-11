//! Procedural mesh primitives with UV coordinates
//!
//! Functions for generating common 3D primitives with normals and UV mapping.
//! These are suitable for textured rendering.

use glam::Vec3;
use std::f32::consts::PI;
use tracing::warn;

use super::types::{MeshDataUV, VertexUV};

/// Generate a UV sphere mesh with smooth normals and equirectangular UV mapping
///
/// # Arguments
/// * `radius` - Sphere radius
/// * `segments` - Number of longitudinal divisions (min 3, max 256)
/// * `rings` - Number of latitudinal divisions (min 2, max 256)
///
/// # Returns
/// Mesh with `(rings + 1) × segments` vertices, Format 5 (POS_UV_NORMAL)
///
/// # UV Mapping
/// - U (horizontal): Longitude (theta) wraps 0→1 around equator
/// - V (vertical): Latitude (phi) maps 0→1 from north pole to south pole
pub fn generate_sphere_uv(radius: f32, segments: u32, rings: u32) -> MeshDataUV {
    // Validate and clamp parameters
    let radius = if radius <= 0.0 {
        warn!("generate_sphere_uv: radius must be > 0.0, clamping to 0.001");
        0.001
    } else {
        radius
    };

    let segments = segments.clamp(3, 256);
    let rings = rings.clamp(2, 256);

    let mut mesh = MeshDataUV::new();

    // Generate vertices with equirectangular UV mapping
    for ring in 0..=rings {
        let phi = (ring as f32 / rings as f32) * PI; // 0 to PI (north pole to south pole)
        let v = ring as f32 / rings as f32; // V coordinate: 0 at north pole, 1 at south pole
        let y = radius * phi.cos();
        let ring_radius = radius * phi.sin();

        for seg in 0..segments {
            let theta = (seg as f32 / segments as f32) * 2.0 * PI; // 0 to 2PI
            let u = seg as f32 / segments as f32; // U coordinate: 0-1 wrapping around
            let x = ring_radius * theta.cos();
            let z = ring_radius * theta.sin();

            let position = Vec3::new(x, y, z);
            let normal = position.normalize(); // Smooth normals point from center

            mesh.add_vertex(VertexUV::new(position, (u, v), normal));
        }
    }

    // Generate indices (same topology as non-UV sphere)
    for ring in 0..rings {
        for seg in 0..segments {
            let next_seg = (seg + 1) % segments;

            let i0 = (ring * segments + seg) as u16;
            let i1 = (ring * segments + next_seg) as u16;
            let i2 = ((ring + 1) * segments + seg) as u16;
            let i3 = ((ring + 1) * segments + next_seg) as u16;

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
pub fn generate_plane_uv(
    size_x: f32,
    size_z: f32,
    subdivisions_x: u32,
    subdivisions_z: u32,
) -> MeshDataUV {
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

    let mut mesh = MeshDataUV::new();
    let normal = Vec3::new(0.0, 1.0, 0.0); // Up

    // Generate vertices with UVs
    for z in 0..=subdivisions_z {
        for x in 0..=subdivisions_x {
            let u = x as f32 / subdivisions_x as f32; // 0-1 along X
            let v = z as f32 / subdivisions_z as f32; // 0-1 along Z

            let pos_x = -size_x * 0.5 + u * size_x;
            let pos_z = -size_z * 0.5 + v * size_z;

            let position = Vec3::new(pos_x, 0.0, pos_z);
            mesh.add_vertex(VertexUV::new(position, (u, v), normal));
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
pub fn generate_cube_uv(size_x: f32, size_y: f32, size_z: f32) -> MeshDataUV {
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

    let mut mesh = MeshDataUV::new();

    // Helper to add a quad with UVs
    #[allow(clippy::too_many_arguments)]
    let add_quad = |mesh: &mut MeshDataUV,
                    v0: Vec3,
                    v1: Vec3,
                    v2: Vec3,
                    v3: Vec3,
                    normal: Vec3,
                    uv0: (f32, f32),
                    uv1: (f32, f32),
                    uv2: (f32, f32),
                    uv3: (f32, f32)| {
        let i0 = mesh.add_vertex(VertexUV::new(v0, uv0, normal));
        let i1 = mesh.add_vertex(VertexUV::new(v1, uv1, normal));
        let i2 = mesh.add_vertex(VertexUV::new(v2, uv2, normal));
        let i3 = mesh.add_vertex(VertexUV::new(v3, uv3, normal));

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
pub fn generate_cylinder_uv(
    radius_bottom: f32,
    radius_top: f32,
    height: f32,
    segments: u32,
) -> MeshDataUV {
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

    let mut mesh = MeshDataUV::new();
    let half_height = height * 0.5;

    // Generate body vertices with cylindrical UVs
    for i in 0..segments {
        let theta = (i as f32 / segments as f32) * 2.0 * PI;
        let u = i as f32 / segments as f32; // Wrap around 0-1
        let cos_theta = theta.cos();
        let sin_theta = theta.sin();

        let bottom_pos = Vec3::new(
            radius_bottom * cos_theta,
            -half_height,
            radius_bottom * sin_theta,
        );
        let top_pos = Vec3::new(radius_top * cos_theta, half_height, radius_top * sin_theta);

        let tangent = Vec3::new(cos_theta, 0.0, sin_theta);
        let slope = Vec3::new(0.0, radius_bottom - radius_top, 0.0);
        let normal = (tangent + slope.normalize_or_zero()).normalize();

        mesh.add_vertex(VertexUV::new(bottom_pos, (u, 0.0), normal));
        mesh.add_vertex(VertexUV::new(top_pos, (u, 1.0), normal));
    }

    // Generate body indices
    for i in 0..segments {
        let next_i = (i + 1) % segments;
        let i0 = (i * 2) as u16;
        let i1 = i0 + 1;
        let i2 = (next_i * 2) as u16;
        let i3 = i2 + 1;

        mesh.add_triangle(i0, i1, i3);
        mesh.add_triangle(i0, i3, i2);
    }

    // Bottom cap (if radius > 0)
    if radius_bottom > 0.0 {
        let center_idx = mesh.add_vertex(VertexUV::new(
            Vec3::new(0.0, -half_height, 0.0),
            (0.5, 0.5),
            Vec3::new(0.0, -1.0, 0.0),
        ));

        for i in 0..segments {
            let next_i = (i + 1) % segments;
            let theta = (i as f32 / segments as f32) * 2.0 * PI;
            let next_theta = (next_i as f32 / segments as f32) * 2.0 * PI;

            let u0 = 0.5 + 0.5 * theta.cos();
            let v0 = 0.5 + 0.5 * theta.sin();
            let u1 = 0.5 + 0.5 * next_theta.cos();
            let v1 = 0.5 + 0.5 * next_theta.sin();

            let i0 = mesh.add_vertex(VertexUV::new(
                Vec3::new(
                    radius_bottom * theta.cos(),
                    -half_height,
                    radius_bottom * theta.sin(),
                ),
                (u0, v0),
                Vec3::new(0.0, -1.0, 0.0),
            ));

            let i1 = mesh.add_vertex(VertexUV::new(
                Vec3::new(
                    radius_bottom * next_theta.cos(),
                    -half_height,
                    radius_bottom * next_theta.sin(),
                ),
                (u1, v1),
                Vec3::new(0.0, -1.0, 0.0),
            ));

            mesh.add_triangle(center_idx, i0, i1);
        }
    }

    // Top cap (if radius > 0)
    if radius_top > 0.0 {
        let center_idx = mesh.add_vertex(VertexUV::new(
            Vec3::new(0.0, half_height, 0.0),
            (0.5, 0.5),
            Vec3::new(0.0, 1.0, 0.0),
        ));

        for i in 0..segments {
            let next_i = (i + 1) % segments;
            let theta = (i as f32 / segments as f32) * 2.0 * PI;
            let next_theta = (next_i as f32 / segments as f32) * 2.0 * PI;

            let u0 = 0.5 + 0.5 * theta.cos();
            let v0 = 0.5 + 0.5 * theta.sin();
            let u1 = 0.5 + 0.5 * next_theta.cos();
            let v1 = 0.5 + 0.5 * next_theta.sin();

            let i0 = mesh.add_vertex(VertexUV::new(
                Vec3::new(
                    radius_top * theta.cos(),
                    half_height,
                    radius_top * theta.sin(),
                ),
                (u0, v0),
                Vec3::new(0.0, 1.0, 0.0),
            ));

            let i1 = mesh.add_vertex(VertexUV::new(
                Vec3::new(
                    radius_top * next_theta.cos(),
                    half_height,
                    radius_top * next_theta.sin(),
                ),
                (u1, v1),
                Vec3::new(0.0, 1.0, 0.0),
            ));

            mesh.add_triangle(center_idx, i1, i0);
        }
    }

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
/// Mesh with `major_segments × minor_segments` vertices, Format 5 (POS_UV_NORMAL)
///
/// # UV Mapping
/// - U wraps 0→1 around major circle (XZ plane)
/// - V wraps 0→1 around minor circle (tube cross-section)
pub fn generate_torus_uv(
    major_radius: f32,
    minor_radius: f32,
    major_segments: u32,
    minor_segments: u32,
) -> MeshDataUV {
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

    let mut mesh = MeshDataUV::new();

    // Generate vertices with wrapped UVs
    for i in 0..major_segments {
        let theta = (i as f32 / major_segments as f32) * 2.0 * PI;
        let u = i as f32 / major_segments as f32; // Major circle UV (0-1)
        let cos_theta = theta.cos();
        let sin_theta = theta.sin();

        for j in 0..minor_segments {
            let phi = (j as f32 / minor_segments as f32) * 2.0 * PI;
            let v = j as f32 / minor_segments as f32; // Minor circle UV (0-1)
            let cos_phi = phi.cos();
            let sin_phi = phi.sin();

            let x = (major_radius + minor_radius * cos_phi) * cos_theta;
            let y = minor_radius * sin_phi;
            let z = (major_radius + minor_radius * cos_phi) * sin_theta;

            let position = Vec3::new(x, y, z);
            let tube_center = Vec3::new(major_radius * cos_theta, 0.0, major_radius * sin_theta);
            let normal = (position - tube_center).normalize();

            mesh.add_vertex(VertexUV::new(position, (u, v), normal));
        }
    }

    // Generate indices (same topology as non-UV torus)
    for i in 0..major_segments {
        let next_i = (i + 1) % major_segments;

        for j in 0..minor_segments {
            let next_j = (j + 1) % minor_segments;

            let i0 = (i * minor_segments + j) as u16;
            let i1 = (i * minor_segments + next_j) as u16;
            let i2 = (next_i * minor_segments + j) as u16;
            let i3 = (next_i * minor_segments + next_j) as u16;

            mesh.add_triangle(i0, i1, i3);
            mesh.add_triangle(i0, i3, i2);
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
pub fn generate_capsule_uv(radius: f32, height: f32, segments: u32, rings: u32) -> MeshDataUV {
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

    let mut mesh = MeshDataUV::new();
    let half_height = height * 0.5;

    // If height is 0, just generate a sphere with full UV range
    if height == 0.0 {
        return generate_sphere_uv(radius, segments, rings * 2);
    }

    // Generate cylinder body (V range: 0.25 to 0.75)
    for i in 0..segments {
        let theta = (i as f32 / segments as f32) * 2.0 * PI;
        let u = i as f32 / segments as f32;
        let cos_theta = theta.cos();
        let sin_theta = theta.sin();

        let bottom_pos = Vec3::new(radius * cos_theta, -half_height, radius * sin_theta);
        let top_pos = Vec3::new(radius * cos_theta, half_height, radius * sin_theta);
        let normal = Vec3::new(cos_theta, 0.0, sin_theta);

        mesh.add_vertex(VertexUV::new(bottom_pos, (u, 0.25), normal));
        mesh.add_vertex(VertexUV::new(top_pos, (u, 0.75), normal));
    }

    // Generate cylinder body indices
    for i in 0..segments {
        let next_i = (i + 1) % segments;
        let i0 = (i * 2) as u16;
        let i1 = i0 + 1;
        let i2 = (next_i * 2) as u16;
        let i3 = i2 + 1;

        mesh.add_triangle(i0, i1, i3);
        mesh.add_triangle(i0, i3, i2);
    }

    // Top hemisphere (V range: 0.75 to 1.0)
    for ring in 0..=rings {
        let phi = (ring as f32 / rings as f32) * (PI * 0.5);
        let v = 0.75 + 0.25 * (ring as f32 / rings as f32); // Map to 0.75-1.0
        let y = half_height + radius * phi.cos();
        let ring_radius = radius * phi.sin();

        for seg in 0..segments {
            let theta = (seg as f32 / segments as f32) * 2.0 * PI;
            let u = seg as f32 / segments as f32;
            let x = ring_radius * theta.cos();
            let z = ring_radius * theta.sin();

            let position = Vec3::new(x, y, z);
            let sphere_center = Vec3::new(0.0, half_height, 0.0);
            let normal = (position - sphere_center).normalize();

            mesh.add_vertex(VertexUV::new(position, (u, v), normal));
        }
    }

    // Top hemisphere indices
    let top_hemi_start = segments * 2;
    for ring in 0..rings {
        for seg in 0..segments {
            let next_seg = (seg + 1) % segments;

            let i0 = (top_hemi_start + ring * segments + seg) as u16;
            let i1 = (top_hemi_start + ring * segments + next_seg) as u16;
            let i2 = (top_hemi_start + (ring + 1) * segments + seg) as u16;
            let i3 = (top_hemi_start + (ring + 1) * segments + next_seg) as u16;

            mesh.add_triangle(i0, i1, i3);
            mesh.add_triangle(i0, i3, i2);
        }
    }

    // Bottom hemisphere (V range: 0.0 to 0.25)
    for ring in 0..=rings {
        let phi = (ring as f32 / rings as f32) * (PI * 0.5);
        let v = 0.25 * (1.0 - ring as f32 / rings as f32); // Map to 0.25-0.0
        let y = -half_height - radius * phi.cos();
        let ring_radius = radius * phi.sin();

        for seg in 0..segments {
            let theta = (seg as f32 / segments as f32) * 2.0 * PI;
            let u = seg as f32 / segments as f32;
            let x = ring_radius * theta.cos();
            let z = ring_radius * theta.sin();

            let position = Vec3::new(x, y, z);
            let sphere_center = Vec3::new(0.0, -half_height, 0.0);
            let normal = (position - sphere_center).normalize();

            mesh.add_vertex(VertexUV::new(position, (u, v), normal));
        }
    }

    // Bottom hemisphere indices
    let bottom_hemi_start = top_hemi_start + (rings + 1) * segments;
    for ring in 0..rings {
        for seg in 0..segments {
            let next_seg = (seg + 1) % segments;

            let i0 = (bottom_hemi_start + ring * segments + seg) as u16;
            let i1 = (bottom_hemi_start + ring * segments + next_seg) as u16;
            let i2 = (bottom_hemi_start + (ring + 1) * segments + seg) as u16;
            let i3 = (bottom_hemi_start + (ring + 1) * segments + next_seg) as u16;

            mesh.add_triangle(i0, i2, i1);
            mesh.add_triangle(i1, i2, i3);
        }
    }

    mesh
}
