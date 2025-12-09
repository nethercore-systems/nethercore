//! Procedural mesh generation
//!
//! Functions for generating common 3D primitives with proper normals.
//!
//! All procedural meshes generate PACKED vertex data for memory efficiency:
//! - Format 4 (POS_NORMAL): 12 bytes/vertex (f16x4 + octahedral u32)
//! - Format 5 (POS_UV_NORMAL): 16 bytes/vertex (f16x4 + unorm16x2 + octahedral u32)

use bytemuck::cast_slice;
use glam::Vec3;
use std::f32::consts::PI;
use tracing::warn;

use crate::graphics::{pack_position_f16, pack_uv_unorm16, pack_normal_octahedral};

/// Vertex with position and normal (no UVs - for solid color rendering)
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
    /// Format 4 (POS_NORMAL): [x, y, z, nx, ny, nz]
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

/// Vertex with position, UV coordinates, and normal (for textured rendering)
#[derive(Clone, Copy, Debug)]
struct VertexUV {
    position: Vec3,
    uv: (f32, f32),
    normal: Vec3,
}

impl VertexUV {
    /// Create a new UV vertex
    fn new(position: Vec3, uv: (f32, f32), normal: Vec3) -> Self {
        Self {
            position,
            uv,
            normal,
        }
    }

    /// Convert to flat f32 array for FFI
    /// Format 5 (POS_UV_NORMAL): [x, y, z, u, v, nx, ny, nz]
    fn to_floats(&self) -> [f32; 8] {
        [
            self.position.x,
            self.position.y,
            self.position.z,
            self.uv.0,
            self.uv.1,
            self.normal.x,
            self.normal.y,
            self.normal.z,
        ]
    }
}

/// Generated mesh data (PACKED FORMAT - POS_NORMAL)
pub struct MeshData {
    /// Packed vertex data: [f16x4, octahedral u32] = 12 bytes per vertex
    pub vertices: Vec<u8>,
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

    /// Add a packed vertex (POS_NORMAL) and return its index
    fn add_vertex(&mut self, vertex: Vertex) -> u16 {
        let index = (self.vertices.len() / 12) as u16;

        // Pack position as [f16; 4] and cast to bytes using bytemuck
        let pos_packed = pack_position_f16(vertex.position.x, vertex.position.y, vertex.position.z);
        self.vertices.extend_from_slice(cast_slice(&pos_packed)); // [f16; 4] → &[u8]

        // Pack normal as octahedral u32 (4 bytes)
        let norm_packed = pack_normal_octahedral(vertex.normal.x, vertex.normal.y, vertex.normal.z);
        self.vertices.extend_from_slice(&norm_packed.to_le_bytes()); // u32 → &[u8; 4]

        index
    }

    /// Add a triangle (3 vertex indices)
    fn add_triangle(&mut self, i0: u16, i1: u16, i2: u16) {
        self.indices.push(i0);
        self.indices.push(i1);
        self.indices.push(i2);
    }
}

/// Generated mesh data with UVs (PACKED FORMAT)
pub struct MeshDataUV {
    /// Packed vertex data: [f16x4, unorm16x2, octahedral u32] = 16 bytes per vertex
    pub vertices: Vec<u8>,
    /// Triangle indices (u16 for GPU compatibility)
    pub indices: Vec<u16>,
}

impl MeshDataUV {
    /// Create empty mesh data
    fn new() -> Self {
        Self {
            vertices: Vec::new(),
            indices: Vec::new(),
        }
    }

    /// Add a packed UV vertex and return its index
    fn add_vertex(&mut self, vertex: VertexUV) -> u16 {
        let index = (self.vertices.len() / 16) as u16;

        // Pack position as [f16; 4] and cast to bytes using bytemuck
        let pos_packed = pack_position_f16(vertex.position.x, vertex.position.y, vertex.position.z);
        self.vertices.extend_from_slice(cast_slice(&pos_packed)); // [f16; 4] → &[u8]

        // Pack UV as [u16; 2] (unorm16) and cast to bytes using bytemuck
        let uv_packed = pack_uv_unorm16(vertex.uv.0, vertex.uv.1);
        self.vertices.extend_from_slice(cast_slice(&uv_packed)); // [u16; 2] → &[u8]

        // Pack normal as octahedral u32 (4 bytes)
        let norm_packed = pack_normal_octahedral(vertex.normal.x, vertex.normal.y, vertex.normal.z);
        self.vertices.extend_from_slice(&norm_packed.to_le_bytes()); // u32 → &[u8; 4]

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
    let add_quad = |mesh: &mut MeshData, v0: Vec3, v1: Vec3, v2: Vec3, v3: Vec3, normal: Vec3| {
        let i0 = mesh.add_vertex(Vertex::new(v0, normal));
        let i1 = mesh.add_vertex(Vertex::new(v1, normal));
        let i2 = mesh.add_vertex(Vertex::new(v2, normal));
        let i3 = mesh.add_vertex(Vertex::new(v3, normal));

        // Two triangles with CCW winding when viewed from front
        // For a quad: v0=BL, v1=BR, v2=TR, v3=TL
        mesh.add_triangle(i0, i1, i2);
        mesh.add_triangle(i0, i2, i3);
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

            // Two triangles per quad (CCW winding for outward-facing normals)
            // Vertex layout: i0=TR, i1=TL (at ring/higher y), i2=BR, i3=BL (at ring+1/lower y)
            // CCW order when viewed from outside: i0→i1→i3→i2
            mesh.add_triangle(i0, i1, i3);
            mesh.add_triangle(i0, i3, i2);
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

            // Two triangles per quad (CCW winding for +Y normal)
            // Vertex layout: i0/i1 at z, i2/i3 at z+1
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
    let body_start_index = mesh.vertices.len() / 12;

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

        // Two triangles per quad (CCW winding for outward normals)
        // Vertex layout: i0=BR, i1=TR (seg i), i2=BL, i3=TL (seg i+1)
        // CCW order when viewed from outside: i0→i1→i3→i2
        mesh.add_triangle(i0, i1, i3);
        mesh.add_triangle(i0, i3, i2);
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

            // CCW winding for -Y normal (viewed from below)
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

            // Two triangles per quad (CCW winding for outward normals)
            // Vertex layout: i0/i1 at major i, i2/i3 at major i+1
            // As phi increases (j → j+1), we move CCW around tube cross-section
            // As theta increases (i → i+1), we move CCW around main ring
            mesh.add_triangle(i0, i1, i3);
            mesh.add_triangle(i0, i3, i2);
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
    let body_start_index = mesh.vertices.len() / 12;

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

        // Two triangles per quad (CCW winding for outward normals)
        // Same layout as cylinder body: i0=BR, i1=TR, i2=BL, i3=TL
        // CCW order when viewed from outside: i0→i1→i3→i2
        mesh.add_triangle(i0, i1, i3);
        mesh.add_triangle(i0, i3, i2);
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

            mesh.add_vertex(Vertex::new(position, normal));
        }
    }

    // Generate bottom hemisphere indices
    let bottom_hemisphere_start = (top_hemisphere_start + (rings + 1) * segments) as u32;

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

// ============================================================================
// UV-Enabled Procedural Shapes (Format 5: POS_UV_NORMAL)
// ============================================================================

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
    let top_hemi_start = (segments * 2) as u32;
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graphics::unpack_octahedral_u32;
    use half::f16;

    // Helper functions to unpack packed vertex data for testing
    // Packed format: Position (f16x4, 8 bytes) + Normal (octahedral u32, 4 bytes) = 12 bytes/vertex

    /// Unpack position from packed vertex data
    fn unpack_position(data: &[u8], vertex_idx: usize) -> [f32; 3] {
        let base = vertex_idx * 12; // 12 bytes per vertex (POS_NORMAL packed)
        let pos_bytes = &data[base..base + 8]; // f16x4 position (8 bytes)
        let pos: &[f16; 4] = bytemuck::from_bytes(&pos_bytes[0..8]);
        [pos[0].to_f32(), pos[1].to_f32(), pos[2].to_f32()]
    }

    /// Unpack normal from packed vertex data (octahedral encoded)
    fn unpack_normal(data: &[u8], vertex_idx: usize) -> [f32; 3] {
        let base = vertex_idx * 12 + 8; // Skip position (8 bytes)
        let norm_bytes = &data[base..base + 4]; // octahedral u32 (4 bytes)
        let packed = u32::from_le_bytes([norm_bytes[0], norm_bytes[1], norm_bytes[2], norm_bytes[3]]);
        let normal = unpack_octahedral_u32(packed);
        [normal.x, normal.y, normal.z]
    }

    #[test]
    fn test_cube_counts() {
        let mesh = generate_cube(1.0, 1.0, 1.0);
        assert_eq!(mesh.vertices.len(), 24 * 12); // 24 vertices × 12 bytes (POS_NORMAL packed)
        assert_eq!(mesh.indices.len(), 36); // 6 faces × 2 triangles × 3
    }

    #[test]
    fn test_sphere_counts() {
        let mesh = generate_sphere(1.0, 16, 8);
        let expected_verts = (8 + 1) * 16; // (rings + 1) × segments
        let expected_indices = 8 * 16 * 6; // rings × segments × 6
        assert_eq!(mesh.vertices.len(), expected_verts * 12); // 12 bytes per vertex (packed)
        assert_eq!(mesh.indices.len(), expected_indices);
    }

    #[test]
    fn test_plane_counts() {
        let mesh = generate_plane(2.0, 2.0, 4, 4);
        let expected_verts = (4 + 1) * (4 + 1); // (subdivisions_x + 1) × (subdivisions_z + 1)
        let expected_indices = 4 * 4 * 6; // subdivisions_x × subdivisions_z × 6
        assert_eq!(mesh.vertices.len(), expected_verts * 12); // 12 bytes per vertex (packed)
        assert_eq!(mesh.indices.len(), expected_indices);
    }

    #[test]
    fn test_normals_normalized() {
        let mesh = generate_sphere(1.0, 16, 8);
        let vertex_count = mesh.vertices.len() / 12; // 12 bytes per vertex

        // Check every normal is unit length
        for i in 0..vertex_count {
            let normal = unpack_normal(&mesh.vertices, i);
            let length = (normal[0] * normal[0] + normal[1] * normal[1] + normal[2] * normal[2]).sqrt();
            assert!(
                (length - 1.0).abs() < 0.02, // Increased tolerance for packed format
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
            let normal = unpack_normal(&mesh.vertices, i);
            assert!((normal[0] - 0.0).abs() < 0.02); // nx = 0
            assert!((normal[1] - 0.0).abs() < 0.02); // ny = 0
            assert!((normal[2] - 1.0).abs() < 0.02); // nz = 1
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

    // ========================================================================
    // Winding Order Verification Tests
    // ========================================================================
    //
    // These tests verify that all triangles have correct CCW winding order
    // (counter-clockwise when viewed from outside the mesh).
    //
    // For each triangle, we:
    // 1. Compute face normal via cross product: N = (v1-v0) × (v2-v0)
    // 2. Compute centroid of the triangle
    // 3. Verify normal points outward (away from mesh center)
    //
    // A positive dot product between the normal and the outward direction
    // confirms CCW winding.

    /// Helper: extract vertex position from mesh at given index
    fn get_vertex_pos(mesh: &MeshData, vertex_idx: u16) -> [f32; 3] {
        unpack_position(&mesh.vertices, vertex_idx as usize)
    }

    /// Helper: compute cross product of two 3D vectors
    fn cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
        [
            a[1] * b[2] - a[2] * b[1],
            a[2] * b[0] - a[0] * b[2],
            a[0] * b[1] - a[1] * b[0],
        ]
    }

    /// Helper: compute dot product of two 3D vectors
    fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
        a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
    }

    /// Helper: subtract two vectors (a - b)
    fn sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
        [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
    }

    /// Helper: normalize a vector
    fn normalize(v: [f32; 3]) -> [f32; 3] {
        let len = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
        if len < 0.0001 {
            return [0.0, 1.0, 0.0]; // fallback
        }
        [v[0] / len, v[1] / len, v[2] / len]
    }

    /// Helper: compute magnitude of a vector
    fn magnitude(v: [f32; 3]) -> f32 {
        (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt()
    }

    /// Helper: compute face normal from triangle indices (CCW winding gives outward normal)
    /// Returns None for degenerate triangles (zero-area)
    fn compute_face_normal(mesh: &MeshData, i0: u16, i1: u16, i2: u16) -> Option<[f32; 3]> {
        let v0 = get_vertex_pos(mesh, i0);
        let v1 = get_vertex_pos(mesh, i1);
        let v2 = get_vertex_pos(mesh, i2);
        let edge1 = sub(v1, v0);
        let edge2 = sub(v2, v0);
        let cross_prod = cross(edge1, edge2);
        let len = magnitude(cross_prod);

        // Skip degenerate triangles (zero-area)
        if len < 0.0001 {
            return None;
        }

        Some([
            cross_prod[0] / len,
            cross_prod[1] / len,
            cross_prod[2] / len,
        ])
    }

    /// Helper: compute triangle centroid
    fn triangle_centroid(mesh: &MeshData, i0: u16, i1: u16, i2: u16) -> [f32; 3] {
        let v0 = get_vertex_pos(mesh, i0);
        let v1 = get_vertex_pos(mesh, i1);
        let v2 = get_vertex_pos(mesh, i2);
        [
            (v0[0] + v1[0] + v2[0]) / 3.0,
            (v0[1] + v1[1] + v2[1]) / 3.0,
            (v0[2] + v1[2] + v2[2]) / 3.0,
        ]
    }

    /// Verify all triangles have outward-facing normals (for meshes centered at origin)
    /// Returns (passed, failed, skipped, total) counts
    /// Skipped triangles are degenerate (zero-area)
    fn verify_outward_normals(mesh: &MeshData, center: [f32; 3]) -> (usize, usize, usize, usize) {
        let mut passed = 0;
        let mut failed = 0;
        let mut skipped = 0;

        for tri in mesh.indices.chunks(3) {
            let i0 = tri[0];
            let i1 = tri[1];
            let i2 = tri[2];

            let face_normal = match compute_face_normal(mesh, i0, i1, i2) {
                Some(n) => n,
                None => {
                    skipped += 1;
                    continue;
                }
            };
            let centroid = triangle_centroid(mesh, i0, i1, i2);

            // Direction from mesh center to triangle centroid
            let outward_dir = normalize(sub(centroid, center));

            // Face normal should point in same general direction as outward
            let alignment = dot(face_normal, outward_dir);

            if alignment > 0.0 {
                passed += 1;
            } else {
                failed += 1;
            }
        }

        (passed, failed, skipped, passed + failed + skipped)
    }

    #[test]
    fn test_winding_cube() {
        let mesh = generate_cube(2.0, 2.0, 2.0);
        let (_passed, failed, skipped, total) = verify_outward_normals(&mesh, [0.0, 0.0, 0.0]);

        assert_eq!(
            failed, 0,
            "Cube winding: {}/{} triangles have incorrect winding (normals point inward)",
            failed, total
        );
        assert_eq!(skipped, 0, "Cube should have no degenerate triangles");
        assert_eq!(total, 12, "Cube should have 12 triangles (6 faces × 2)");
    }

    #[test]
    fn test_winding_sphere() {
        let mesh = generate_sphere(1.0, 32, 16);
        let (passed, failed, skipped, _total) = verify_outward_normals(&mesh, [0.0, 0.0, 0.0]);

        // Sphere has degenerate triangles at poles where all vertices share the same position
        // (top pole: ring 0, bottom pole: ring N)
        assert_eq!(
            failed, 0,
            "Sphere winding: {}/{} non-degenerate triangles have incorrect winding (skipped {} degenerate)",
            failed, passed + failed, skipped
        );
        assert!(passed > 0, "Sphere should have valid triangles");
        // 32 segments × 2 poles = 64 degenerate triangles at poles
        assert!(
            skipped <= 64,
            "Sphere should have at most 64 degenerate pole triangles, got {}",
            skipped
        );
    }

    #[test]
    fn test_winding_plane() {
        let mesh = generate_plane(2.0, 2.0, 4, 4);

        // For plane, all normals should point +Y (upward)
        let mut correct = 0;
        let mut wrong = 0;
        let mut skipped = 0;

        for tri in mesh.indices.chunks(3) {
            let face_normal = match compute_face_normal(&mesh, tri[0], tri[1], tri[2]) {
                Some(n) => n,
                None => {
                    skipped += 1;
                    continue;
                }
            };
            // Should point +Y
            if face_normal[1] > 0.9 {
                correct += 1;
            } else {
                wrong += 1;
            }
        }

        assert_eq!(skipped, 0, "Plane should have no degenerate triangles");
        assert_eq!(
            wrong,
            0,
            "Plane winding: {}/{} triangles have incorrect winding (normals should point +Y)",
            wrong,
            correct + wrong
        );
    }

    #[test]
    fn test_winding_cylinder() {
        let mesh = generate_cylinder(1.0, 1.0, 2.0, 32);

        // Cylinder is centered at origin with height along Y
        // Body normals should point radially outward (XZ plane)
        // Top cap normals should point +Y
        // Bottom cap normals should point -Y

        let mut body_correct = 0;
        let mut body_wrong = 0;
        let mut cap_correct = 0;
        let mut cap_wrong = 0;
        let mut skipped = 0;

        for tri in mesh.indices.chunks(3) {
            let face_normal = match compute_face_normal(&mesh, tri[0], tri[1], tri[2]) {
                Some(n) => n,
                None => {
                    skipped += 1;
                    continue;
                }
            };
            let centroid = triangle_centroid(&mesh, tri[0], tri[1], tri[2]);

            // Determine if this is a cap or body triangle based on Y position
            let y = centroid[1];
            let is_top_cap = y > 0.9;
            let is_bottom_cap = y < -0.9;

            if is_top_cap {
                // Top cap should point +Y
                if face_normal[1] > 0.9 {
                    cap_correct += 1;
                } else {
                    cap_wrong += 1;
                }
            } else if is_bottom_cap {
                // Bottom cap should point -Y
                if face_normal[1] < -0.9 {
                    cap_correct += 1;
                } else {
                    cap_wrong += 1;
                }
            } else {
                // Body - normal should point radially outward in XZ plane
                let radial_dir = normalize([centroid[0], 0.0, centroid[2]]);
                let alignment = dot(face_normal, radial_dir);
                if alignment > 0.0 {
                    body_correct += 1;
                } else {
                    body_wrong += 1;
                }
            }
        }

        assert_eq!(
            body_wrong,
            0,
            "Cylinder body winding: {}/{} triangles have incorrect winding",
            body_wrong,
            body_correct + body_wrong
        );
        assert_eq!(
            cap_wrong,
            0,
            "Cylinder cap winding: {}/{} triangles have incorrect winding",
            cap_wrong,
            cap_correct + cap_wrong
        );
        // Cylinder caps have center vertices, so some triangles at center may be degenerate
        assert!(
            skipped <= 2,
            "Cylinder should have at most 2 degenerate triangles, got {}",
            skipped
        );
    }

    #[test]
    fn test_winding_torus() {
        // Use proper torus proportions: major > minor to avoid self-intersection
        let major_radius = 1.0; // Distance from torus center to tube center
        let minor_radius = 0.3; // Tube radius
        let mesh = generate_torus(major_radius, minor_radius, 32, 16);

        // Torus: major radius in XZ plane, minor radius forms the tube
        // For each triangle, the normal should point away from the tube center
        let mut correct = 0;
        let mut wrong = 0;
        let mut skipped = 0;

        for tri in mesh.indices.chunks(3) {
            let face_normal = match compute_face_normal(&mesh, tri[0], tri[1], tri[2]) {
                Some(n) => n,
                None => {
                    skipped += 1;
                    continue;
                }
            };
            let centroid = triangle_centroid(&mesh, tri[0], tri[1], tri[2]);

            // Find the closest point on the major circle (tube center)
            let xz_len = (centroid[0] * centroid[0] + centroid[2] * centroid[2]).sqrt();
            if xz_len > 0.01 {
                let tube_center = [
                    centroid[0] * major_radius / xz_len,
                    0.0,
                    centroid[2] * major_radius / xz_len,
                ];

                // Outward direction is from tube center to triangle centroid
                let outward = normalize(sub(centroid, tube_center));
                let alignment = dot(face_normal, outward);

                if alignment > 0.0 {
                    correct += 1;
                } else {
                    wrong += 1;
                }
            } else {
                correct += 1; // Edge case, assume correct
            }
        }

        assert_eq!(skipped, 0, "Torus should have no degenerate triangles");
        assert_eq!(
            wrong,
            0,
            "Torus winding: {}/{} triangles have incorrect winding",
            wrong,
            correct + wrong
        );
    }

    #[test]
    fn test_winding_capsule() {
        let mesh = generate_capsule(0.5, 2.0, 32, 16);

        // Capsule: cylinder body with hemisphere caps
        // All normals should point radially outward from the capsule axis

        // For capsule, we use a more specific test
        let mut correct = 0;
        let mut wrong = 0;
        let mut skipped = 0;

        for tri in mesh.indices.chunks(3) {
            let face_normal = match compute_face_normal(&mesh, tri[0], tri[1], tri[2]) {
                Some(n) => n,
                None => {
                    skipped += 1;
                    continue;
                }
            };
            let centroid = triangle_centroid(&mesh, tri[0], tri[1], tri[2]);

            // For capsule along Y axis, outward is radial from Y axis
            let radial_xz = (centroid[0] * centroid[0] + centroid[2] * centroid[2]).sqrt();

            if radial_xz > 0.01 {
                // Not on the axis - check radial direction
                let radial_dir = normalize([centroid[0], 0.0, centroid[2]]);
                let horizontal_component =
                    face_normal[0] * radial_dir[0] + face_normal[2] * radial_dir[2];

                // Most of the normal's horizontal component should point outward
                // (Allow for caps where vertical component dominates)
                let normal_horizontal_len =
                    (face_normal[0] * face_normal[0] + face_normal[2] * face_normal[2]).sqrt();

                if normal_horizontal_len > 0.3 {
                    // Significant horizontal component
                    if horizontal_component > 0.0 {
                        correct += 1;
                    } else {
                        wrong += 1;
                    }
                } else {
                    // Mostly vertical (cap area) - check Y direction matches position
                    if (centroid[1] > 0.0 && face_normal[1] > 0.0)
                        || (centroid[1] < 0.0 && face_normal[1] < 0.0)
                        || centroid[1].abs() < 0.1
                    {
                        correct += 1;
                    } else {
                        wrong += 1;
                    }
                }
            } else {
                // On the axis (pole) - normal should point up or down based on Y
                if (centroid[1] > 0.0 && face_normal[1] > 0.0)
                    || (centroid[1] < 0.0 && face_normal[1] < 0.0)
                {
                    correct += 1;
                } else {
                    wrong += 1;
                }
            }
        }

        // Capsule has degenerate triangles at hemisphere poles
        // 32 segments × 2 poles = 64 max degenerate triangles
        assert!(
            skipped <= 64,
            "Capsule should have at most 64 degenerate pole triangles, got {}",
            skipped
        );
        assert_eq!(
            wrong,
            0,
            "Capsule winding: {}/{} triangles have incorrect winding (skipped {} degenerate)",
            wrong,
            correct + wrong,
            skipped
        );
    }

    /// Test that stored vertex normals point outward for sphere
    /// This verifies the actual normal data, not just the winding order
    #[test]
    fn test_sphere_vertex_normals_point_outward() {
        let mesh = generate_sphere(1.0, 16, 8);
        let vertex_count = mesh.vertices.len() / 12; // 12 bytes per vertex (POS_NORMAL packed)

        // For a unit sphere centered at origin, the vertex normal should equal
        // the normalized vertex position (pointing outward from center)
        for i in 0..vertex_count {
            let pos = unpack_position(&mesh.vertices, i);
            let normal = unpack_normal(&mesh.vertices, i);

            let px = pos[0];
            let py = pos[1];
            let pz = pos[2];
            let nx = normal[0];
            let ny = normal[1];
            let nz = normal[2];

            // Compute expected normal (normalized position for unit sphere at origin)
            let pos_len = (px * px + py * py + pz * pz).sqrt();
            if pos_len < 0.001 {
                continue; // Skip degenerate vertices at poles
            }

            let expected_nx = px / pos_len;
            let expected_ny = py / pos_len;
            let expected_nz = pz / pos_len;

            // Verify stored normal matches expected (points outward)
            // Increased tolerance for packed format (f16/snorm16)
            assert!(
                (nx - expected_nx).abs() < 0.02,
                "Sphere vertex normal X mismatch at vertex {}: stored={}, expected={}",
                i,
                nx,
                expected_nx
            );
            assert!(
                (ny - expected_ny).abs() < 0.02,
                "Sphere vertex normal Y mismatch at vertex {}: stored={}, expected={}",
                i,
                ny,
                expected_ny
            );
            assert!(
                (nz - expected_nz).abs() < 0.02,
                "Sphere vertex normal Z mismatch at vertex {}: stored={}, expected={}",
                i,
                nz,
                expected_nz
            );

            // Also verify dot product is positive (normal points same direction as position)
            let dot_product = px * nx + py * ny + pz * nz;
            assert!(
                dot_product > 0.0,
                "Sphere vertex {} normal points inward! dot(pos, normal) = {}",
                i,
                dot_product
            );
        }
    }

    /// Test that cube vertex normals match face directions
    #[test]
    fn test_cube_vertex_normals_match_faces() {
        let mesh = generate_cube(1.0, 1.0, 1.0);

        // Each face has 4 vertices, all with the same normal
        // Faces in order: +Z, -Z, +Y, -Y, +X, -X
        let expected_normals: [[f32; 3]; 6] = [
            [0.0, 0.0, 1.0],  // Front (+Z)
            [0.0, 0.0, -1.0], // Back (-Z)
            [0.0, 1.0, 0.0],  // Top (+Y)
            [0.0, -1.0, 0.0], // Bottom (-Y)
            [1.0, 0.0, 0.0],  // Right (+X)
            [-1.0, 0.0, 0.0], // Left (-X)
        ];

        for face in 0..6 {
            let expected = expected_normals[face];
            for vert in 0..4 {
                let vertex_idx = face * 4 + vert;
                let normal = unpack_normal(&mesh.vertices, vertex_idx);

                // Increased tolerance for packed format (snorm16)
                assert!(
                    (normal[0] - expected[0]).abs() < 0.02,
                    "Cube face {} vertex {} normal X mismatch: {} vs expected {}",
                    face,
                    vert,
                    normal[0],
                    expected[0]
                );
                assert!(
                    (normal[1] - expected[1]).abs() < 0.02,
                    "Cube face {} vertex {} normal Y mismatch: {} vs expected {}",
                    face,
                    vert,
                    normal[1],
                    expected[1]
                );
                assert!(
                    (normal[2] - expected[2]).abs() < 0.02,
                    "Cube face {} vertex {} normal Z mismatch: {} vs expected {}",
                    face,
                    vert,
                    normal[2],
                    expected[2]
                );
            }
        }
    }

    /// Test that verifies ALL procedural shapes at once with a simple outward test
    #[test]
    fn test_winding_all_shapes_simple() {
        // This is a comprehensive sanity check using the simple center-based test
        // Note: This test uses center-to-centroid which doesn't work perfectly for torus,
        // so torus is excluded here (it has a dedicated test with tube-center logic)

        let shapes: Vec<(&str, MeshData)> = vec![
            ("cube", generate_cube(2.0, 2.0, 2.0)),
            ("sphere", generate_sphere(1.0, 16, 8)),
            ("cylinder", generate_cylinder(1.0, 1.0, 2.0, 16)),
            ("capsule", generate_capsule(0.5, 2.0, 16, 8)),
        ];

        for (name, mesh) in shapes {
            let (passed, failed, skipped, _total) = verify_outward_normals(&mesh, [0.0, 0.0, 0.0]);

            // Degenerate triangles (at poles) are expected and skipped
            // Only non-degenerate triangles should be verified
            let non_degenerate = passed + failed;
            if non_degenerate > 0 {
                assert_eq!(
                    failed, 0,
                    "{}: {}/{} non-degenerate triangles have incorrect winding (skipped {} degenerate)",
                    name, failed, non_degenerate, skipped
                );
            }
        }
    }
}
