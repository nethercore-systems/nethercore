//! Procedural mesh generation
//!
//! Functions for generating common 3D primitives with proper normals.
//! All meshes use vertex format 4 (POS_NORMAL): [x, y, z, nx, ny, nz]
//!
//! Procedural meshes rely on uniform colors from material properties,
//! not texture sampling.

use glam::Vec3;
use std::f32::consts::PI;
use tracing::warn;

/// Vertex with position and normal (no UVs - procedural meshes use uniform colors)
#[derive(Clone, Copy, Debug)]
struct Vertex {
    position: Vec3,
    normal: Vec3,
}

impl Vertex {
    /// Create a new vertex
    fn new(position: Vec3, normal: Vec3) -> Self {
        Self { position, normal }
    }

    /// Convert to flat f32 array for FFI
    /// Format: [x, y, z, nx, ny, nz]
    fn to_floats(&self) -> [f32; 6] {
        [
            self.position.x,
            self.position.y,
            self.position.z,
            self.normal.x,
            self.normal.y,
            self.normal.z,
        ]
    }
}

/// Generated mesh data (vertices + indices)
pub struct MeshData {
    /// Flat vertex data: [x, y, z, nx, ny, nz] per vertex
    pub vertices: Vec<f32>,
    /// Triangle indices (u16 for GPU compatibility)
    pub indices: Vec<u16>,
}

impl MeshData {
    /// Create empty mesh data
    fn new() -> Self {
        Self {
            vertices: Vec::new(),
            indices: Vec::new(),
        }
    }

    /// Add a vertex and return its index
    fn add_vertex(&mut self, vertex: Vertex) -> u16 {
        let index = (self.vertices.len() / 6) as u16;
        self.vertices.extend_from_slice(&vertex.to_floats());
        index
    }

    /// Add a triangle (3 vertex indices)
    fn add_triangle(&mut self, i0: u16, i1: u16, i2: u16) {
        self.indices.push(i0);
        self.indices.push(i1);
        self.indices.push(i2);
    }
}

/// Generate a cube mesh with flat normals
///
/// # Arguments
/// * `size_x` - Half-extent along X axis
/// * `size_y` - Half-extent along Y axis
/// * `size_z` - Half-extent along Z axis
///
/// # Returns
/// Mesh with 24 vertices (4 per face) and 36 indices (6 faces × 2 triangles × 3)
pub fn generate_cube(size_x: f32, size_y: f32, size_z: f32) -> MeshData {
    // Validate and clamp parameters
    let size_x = if size_x <= 0.0 {
        warn!("generate_cube: size_x must be > 0.0, clamping to 0.001");
        0.001
    } else {
        size_x
    };

    let size_y = if size_y <= 0.0 {
        warn!("generate_cube: size_y must be > 0.0, clamping to 0.001");
        0.001
    } else {
        size_y
    };

    let size_z = if size_z <= 0.0 {
        warn!("generate_cube: size_z must be > 0.0, clamping to 0.001");
        0.001
    } else {
        size_z
    };

    let mut mesh = MeshData::new();

    // Helper to add a quad (4 vertices, 2 triangles)
    let add_quad = |mesh: &mut MeshData,
                    v0: Vec3,
                    v1: Vec3,
                    v2: Vec3,
                    v3: Vec3,
                    normal: Vec3| {
        let i0 = mesh.add_vertex(Vertex::new(v0, normal));
        let i1 = mesh.add_vertex(Vertex::new(v1, normal));
        let i2 = mesh.add_vertex(Vertex::new(v2, normal));
        let i3 = mesh.add_vertex(Vertex::new(v3, normal));

        // Two triangles with CCW winding (right-handed, Y-up)
        mesh.add_triangle(i0, i2, i1);
        mesh.add_triangle(i1, i2, i3);
    };

    // Front face (z = +size_z, facing +Z)
    add_quad(
        &mut mesh,
        Vec3::new(-size_x, -size_y, size_z),
        Vec3::new(size_x, -size_y, size_z),
        Vec3::new(size_x, size_y, size_z),
        Vec3::new(-size_x, size_y, size_z),
        Vec3::new(0.0, 0.0, 1.0),
    );

    // Back face (z = -size_z, facing -Z)
    add_quad(
        &mut mesh,
        Vec3::new(size_x, -size_y, -size_z),
        Vec3::new(-size_x, -size_y, -size_z),
        Vec3::new(-size_x, size_y, -size_z),
        Vec3::new(size_x, size_y, -size_z),
        Vec3::new(0.0, 0.0, -1.0),
    );

    // Top face (y = +size_y, facing +Y)
    add_quad(
        &mut mesh,
        Vec3::new(-size_x, size_y, size_z),
        Vec3::new(size_x, size_y, size_z),
        Vec3::new(size_x, size_y, -size_z),
        Vec3::new(-size_x, size_y, -size_z),
        Vec3::new(0.0, 1.0, 0.0),
    );

    // Bottom face (y = -size_y, facing -Y)
    add_quad(
        &mut mesh,
        Vec3::new(-size_x, -size_y, -size_z),
        Vec3::new(size_x, -size_y, -size_z),
        Vec3::new(size_x, -size_y, size_z),
        Vec3::new(-size_x, -size_y, size_z),
        Vec3::new(0.0, -1.0, 0.0),
    );

    // Right face (x = +size_x, facing +X)
    add_quad(
        &mut mesh,
        Vec3::new(size_x, -size_y, size_z),
        Vec3::new(size_x, -size_y, -size_z),
        Vec3::new(size_x, size_y, -size_z),
        Vec3::new(size_x, size_y, size_z),
        Vec3::new(1.0, 0.0, 0.0),
    );

    // Left face (x = -size_x, facing -X)
    add_quad(
        &mut mesh,
        Vec3::new(-size_x, -size_y, -size_z),
        Vec3::new(-size_x, -size_y, size_z),
        Vec3::new(-size_x, size_y, size_z),
        Vec3::new(-size_x, size_y, -size_z),
        Vec3::new(-1.0, 0.0, 0.0),
    );

    mesh
}

/// Generate a UV sphere mesh with smooth normals
///
/// # Arguments
/// * `radius` - Sphere radius
/// * `segments` - Number of longitudinal divisions (min 3, max 256)
/// * `rings` - Number of latitudinal divisions (min 2, max 256)
///
/// # Returns
/// Mesh with `(rings + 1) × segments` vertices
pub fn generate_sphere(radius: f32, segments: u32, rings: u32) -> MeshData {
    // Validate and clamp parameters
    let radius = if radius <= 0.0 {
        warn!("generate_sphere: radius must be > 0.0, clamping to 0.001");
        0.001
    } else {
        radius
    };

    let segments = segments.clamp(3, 256);
    let rings = rings.clamp(2, 256);

    let mut mesh = MeshData::new();

    // Generate vertices
    for ring in 0..=rings {
        let phi = (ring as f32 / rings as f32) * PI; // 0 to PI
        let y = radius * phi.cos();
        let ring_radius = radius * phi.sin();

        for seg in 0..segments {
            let theta = (seg as f32 / segments as f32) * 2.0 * PI; // 0 to 2PI
            let x = ring_radius * theta.cos();
            let z = ring_radius * theta.sin();

            let position = Vec3::new(x, y, z);
            let normal = position.normalize(); // Smooth normals point from center

            mesh.add_vertex(Vertex::new(position, normal));
        }
    }

    // Generate indices
    for ring in 0..rings {
        for seg in 0..segments {
            let next_seg = (seg + 1) % segments;

            let i0 = (ring * segments + seg) as u16;
            let i1 = (ring * segments + next_seg) as u16;
            let i2 = ((ring + 1) * segments + seg) as u16;
            let i3 = ((ring + 1) * segments + next_seg) as u16;

            // Two triangles per quad
            mesh.add_triangle(i0, i2, i1);
            mesh.add_triangle(i1, i2, i3);
        }
    }

    mesh
}

/// Generate a plane mesh on the XZ plane (Y=0)
///
/// # Arguments
/// * `size_x` - Width along X axis
/// * `size_z` - Depth along Z axis
/// * `subdivisions_x` - Number of X subdivisions (min 1, max 256)
/// * `subdivisions_z` - Number of Z subdivisions (min 1, max 256)
///
/// # Returns
/// Mesh with `(subdivisions_x + 1) × (subdivisions_z + 1)` vertices
pub fn generate_plane(
    size_x: f32,
    size_z: f32,
    subdivisions_x: u32,
    subdivisions_z: u32,
) -> MeshData {
    // Validate and clamp parameters
    let size_x = if size_x <= 0.0 {
        warn!("generate_plane: size_x must be > 0.0, clamping to 0.001");
        0.001
    } else {
        size_x
    };

    let size_z = if size_z <= 0.0 {
        warn!("generate_plane: size_z must be > 0.0, clamping to 0.001");
        0.001
    } else {
        size_z
    };

    let subdivisions_x = subdivisions_x.clamp(1, 256);
    let subdivisions_z = subdivisions_z.clamp(1, 256);

    let mut mesh = MeshData::new();
    let normal = Vec3::new(0.0, 1.0, 0.0); // Up

    // Generate vertices
    for z in 0..=subdivisions_z {
        for x in 0..=subdivisions_x {
            let u = x as f32 / subdivisions_x as f32;
            let v = z as f32 / subdivisions_z as f32;

            let pos_x = -size_x * 0.5 + u * size_x;
            let pos_z = -size_z * 0.5 + v * size_z;

            let position = Vec3::new(pos_x, 0.0, pos_z);
            mesh.add_vertex(Vertex::new(position, normal));
        }
    }

    // Generate indices
    for z in 0..subdivisions_z {
        for x in 0..subdivisions_x {
            let i0 = (z * (subdivisions_x + 1) + x) as u16;
            let i1 = i0 + 1;
            let i2 = ((z + 1) * (subdivisions_x + 1) + x) as u16;
            let i3 = i2 + 1;

            // Two triangles per quad
            mesh.add_triangle(i0, i2, i1);
            mesh.add_triangle(i1, i2, i3);
        }
    }

    mesh
}

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
pub fn generate_cylinder(
    radius_bottom: f32,
    radius_top: f32,
    height: f32,
    segments: u32,
) -> MeshData {
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

    let mut mesh = MeshData::new();
    let half_height = height * 0.5;

    // Generate body vertices (two rings: bottom and top)
    let body_start_index = mesh.vertices.len() / 8;

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

        mesh.add_vertex(Vertex::new(bottom_pos, normal));
        mesh.add_vertex(Vertex::new(top_pos, normal));
    }

    // Generate body indices
    for i in 0..segments {
        let next_i = (i + 1) % segments;

        let i0 = (body_start_index + (i * 2) as usize) as u16;
        let i1 = i0 + 1;
        let i2 = (body_start_index + (next_i * 2) as usize) as u16;
        let i3 = i2 + 1;

        // Two triangles per quad
        mesh.add_triangle(i0, i2, i1);
        mesh.add_triangle(i1, i2, i3);
    }

    // Generate bottom cap (if radius > 0)
    if radius_bottom > 0.0 {
        let cap_center_index = mesh.add_vertex(Vertex::new(
            Vec3::new(0.0, -half_height, 0.0),
            Vec3::new(0.0, -1.0, 0.0),
        ));

        for i in 0..segments {
            let next_i = (i + 1) % segments;
            let theta = (i as f32 / segments as f32) * 2.0 * PI;
            let next_theta = (next_i as f32 / segments as f32) * 2.0 * PI;

            let i0 = mesh.add_vertex(Vertex::new(
                Vec3::new(
                    radius_bottom * theta.cos(),
                    -half_height,
                    radius_bottom * theta.sin(),
                ),
                Vec3::new(0.0, -1.0, 0.0),
            ));

            let i1 = mesh.add_vertex(Vertex::new(
                Vec3::new(
                    radius_bottom * next_theta.cos(),
                    -half_height,
                    radius_bottom * next_theta.sin(),
                ),
                Vec3::new(0.0, -1.0, 0.0),
            ));

            mesh.add_triangle(cap_center_index, i0, i1);
        }
    }

    // Generate top cap (if radius > 0)
    if radius_top > 0.0 {
        let cap_center_index = mesh.add_vertex(Vertex::new(
            Vec3::new(0.0, half_height, 0.0),
            Vec3::new(0.0, 1.0, 0.0),
        ));

        for i in 0..segments {
            let next_i = (i + 1) % segments;
            let theta = (i as f32 / segments as f32) * 2.0 * PI;
            let next_theta = (next_i as f32 / segments as f32) * 2.0 * PI;

            let i0 = mesh.add_vertex(Vertex::new(
                Vec3::new(
                    radius_top * theta.cos(),
                    half_height,
                    radius_top * theta.sin(),
                ),
                Vec3::new(0.0, 1.0, 0.0),
            ));

            let i1 = mesh.add_vertex(Vertex::new(
                Vec3::new(
                    radius_top * next_theta.cos(),
                    half_height,
                    radius_top * next_theta.sin(),
                ),
                Vec3::new(0.0, 1.0, 0.0),
            ));

            mesh.add_triangle(cap_center_index, i1, i0);
        }
    }

    mesh
}

/// Generate a torus mesh
///
/// # Arguments
/// * `major_radius` - Distance from torus center to tube center
/// * `minor_radius` - Tube radius
/// * `major_segments` - Segments around major circle (min 3, max 256)
/// * `minor_segments` - Segments around tube (min 3, max 256)
///
/// # Returns
/// Mesh with `major_segments × minor_segments` vertices
pub fn generate_torus(
    major_radius: f32,
    minor_radius: f32,
    major_segments: u32,
    minor_segments: u32,
) -> MeshData {
    // Validate and clamp parameters
    let major_radius = if major_radius <= 0.0 {
        warn!("generate_torus: major_radius must be > 0.0, clamping to 0.001");
        0.001
    } else {
        major_radius
    };

    let minor_radius = if minor_radius <= 0.0 {
        warn!("generate_torus: minor_radius must be > 0.0, clamping to 0.001");
        0.001
    } else {
        minor_radius
    };

    let major_segments = major_segments.clamp(3, 256);
    let minor_segments = minor_segments.clamp(3, 256);

    let mut mesh = MeshData::new();

    // Generate vertices
    for i in 0..major_segments {
        let theta = (i as f32 / major_segments as f32) * 2.0 * PI;
        let cos_theta = theta.cos();
        let sin_theta = theta.sin();

        for j in 0..minor_segments {
            let phi = (j as f32 / minor_segments as f32) * 2.0 * PI;
            let cos_phi = phi.cos();
            let sin_phi = phi.sin();

            // Position on torus surface
            let x = (major_radius + minor_radius * cos_phi) * cos_theta;
            let y = minor_radius * sin_phi;
            let z = (major_radius + minor_radius * cos_phi) * sin_theta;

            let position = Vec3::new(x, y, z);

            // Normal points radially from tube center
            let tube_center = Vec3::new(major_radius * cos_theta, 0.0, major_radius * sin_theta);
            let normal = (position - tube_center).normalize();

            mesh.add_vertex(Vertex::new(position, normal));
        }
    }

    // Generate indices
    for i in 0..major_segments {
        let next_i = (i + 1) % major_segments;

        for j in 0..minor_segments {
            let next_j = (j + 1) % minor_segments;

            let i0 = (i * minor_segments + j) as u16;
            let i1 = (i * minor_segments + next_j) as u16;
            let i2 = (next_i * minor_segments + j) as u16;
            let i3 = (next_i * minor_segments + next_j) as u16;

            // Two triangles per quad
            mesh.add_triangle(i0, i2, i1);
            mesh.add_triangle(i1, i2, i3);
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
pub fn generate_capsule(radius: f32, height: f32, segments: u32, rings: u32) -> MeshData {
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

    let mut mesh = MeshData::new();
    let half_height = height * 0.5;

    // If height is 0, just generate a sphere
    if height == 0.0 {
        return generate_sphere(radius, segments, rings * 2);
    }

    // Generate cylinder body vertices (two rings)
    let body_start_index = mesh.vertices.len() / 8;

    for i in 0..segments {
        let theta = (i as f32 / segments as f32) * 2.0 * PI;
        let cos_theta = theta.cos();
        let sin_theta = theta.sin();

        let bottom_pos = Vec3::new(radius * cos_theta, -half_height, radius * sin_theta);
        let top_pos = Vec3::new(radius * cos_theta, half_height, radius * sin_theta);

        let normal = Vec3::new(cos_theta, 0.0, sin_theta); // Radial normal

        mesh.add_vertex(Vertex::new(bottom_pos, normal));
        mesh.add_vertex(Vertex::new(top_pos, normal));
    }

    // Generate cylinder body indices
    for i in 0..segments {
        let next_i = (i + 1) % segments;

        let i0 = (body_start_index + (i * 2) as usize) as u16;
        let i1 = i0 + 1;
        let i2 = (body_start_index + (next_i * 2) as usize) as u16;
        let i3 = i2 + 1;

        mesh.add_triangle(i0, i2, i1);
        mesh.add_triangle(i1, i2, i3);
    }

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

            mesh.add_vertex(Vertex::new(position, normal));
        }
    }

    // Generate top hemisphere indices
    let top_hemisphere_start = (body_start_index + (segments * 2) as usize) as u32;

    for ring in 0..rings {
        for seg in 0..segments {
            let next_seg = (seg + 1) % segments;

            let i0 = (top_hemisphere_start + ring * segments + seg) as u16;
            let i1 = (top_hemisphere_start + ring * segments + next_seg) as u16;
            let i2 = (top_hemisphere_start + (ring + 1) * segments + seg) as u16;
            let i3 = (top_hemisphere_start + (ring + 1) * segments + next_seg) as u16;

            mesh.add_triangle(i0, i2, i1);
            mesh.add_triangle(i1, i2, i3);
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

            mesh.add_vertex(Vertex::new(position, normal));
        }
    }

    // Generate bottom hemisphere indices
    let bottom_hemisphere_start =
        (top_hemisphere_start + (rings + 1) * segments) as u32;

    for ring in 0..rings {
        for seg in 0..segments {
            let next_seg = (seg + 1) % segments;

            let i0 = (bottom_hemisphere_start + ring * segments + seg) as u16;
            let i1 = (bottom_hemisphere_start + ring * segments + next_seg) as u16;
            let i2 = (bottom_hemisphere_start + (ring + 1) * segments + seg) as u16;
            let i3 = (bottom_hemisphere_start + (ring + 1) * segments + next_seg) as u16;

            mesh.add_triangle(i0, i2, i1);
            mesh.add_triangle(i1, i2, i3);
        }
    }

    mesh
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cube_counts() {
        let mesh = generate_cube(1.0, 1.0, 1.0);
        assert_eq!(mesh.vertices.len(), 24 * 6); // 24 vertices × 6 floats (POS_NORMAL)
        assert_eq!(mesh.indices.len(), 36); // 6 faces × 2 triangles × 3
    }

    #[test]
    fn test_sphere_counts() {
        let mesh = generate_sphere(1.0, 16, 8);
        let expected_verts = (8 + 1) * 16; // (rings + 1) × segments
        let expected_indices = 8 * 16 * 6; // rings × segments × 6
        assert_eq!(mesh.vertices.len(), expected_verts * 6);
        assert_eq!(mesh.indices.len(), expected_indices);
    }

    #[test]
    fn test_plane_counts() {
        let mesh = generate_plane(2.0, 2.0, 4, 4);
        let expected_verts = (4 + 1) * (4 + 1); // (subdivisions_x + 1) × (subdivisions_z + 1)
        let expected_indices = 4 * 4 * 6; // subdivisions_x × subdivisions_z × 6
        assert_eq!(mesh.vertices.len(), expected_verts * 6);
        assert_eq!(mesh.indices.len(), expected_indices);
    }

    #[test]
    fn test_normals_normalized() {
        let mesh = generate_sphere(1.0, 16, 8);

        // Check every normal is unit length
        for i in (0..mesh.vertices.len()).step_by(6) {
            let nx = mesh.vertices[i + 3];
            let ny = mesh.vertices[i + 4];
            let nz = mesh.vertices[i + 5];
            let length = (nx * nx + ny * ny + nz * nz).sqrt();
            assert!(
                (length - 1.0).abs() < 0.001,
                "Normal not normalized: {}",
                length
            );
        }
    }

    #[test]
    fn test_cube_flat_normals() {
        let mesh = generate_cube(1.0, 1.0, 1.0);

        // First 4 vertices (front face) should all have normal (0, 0, 1)
        for i in 0..4 {
            let idx = i * 6;
            assert!((mesh.vertices[idx + 3] - 0.0).abs() < 0.001); // nx = 0
            assert!((mesh.vertices[idx + 4] - 0.0).abs() < 0.001); // ny = 0
            assert!((mesh.vertices[idx + 5] - 1.0).abs() < 0.001); // nz = 1
        }
    }

    #[test]
    fn test_invalid_params_safe() {
        // Should not panic, should clamp
        let _ = generate_cube(0.0, 1.0, 1.0);
        let _ = generate_sphere(-1.0, 3, 2);
        let _ = generate_cylinder(1.0, 1.0, 1.0, 2);
        let _ = generate_plane(1.0, 1.0, 0, 0);
        let _ = generate_torus(0.5, 1.0, 3, 3);
        let _ = generate_capsule(1.0, 0.0, 3, 1);
    }
}
