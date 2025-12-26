//! Procedural mesh types
//!
//! Shared types for procedural mesh generation.

use bytemuck::cast_slice;
use glam::Vec3;

use crate::graphics::{pack_normal_octahedral, pack_position_f16, pack_uv_unorm16};

/// Trait for mesh construction - enables generic geometry generation
///
/// This trait allows procedural generation functions to work with both:
/// - `MeshData`: Packed GPU format for runtime
/// - `UnpackedMesh`: f32 format for OBJ export and modifiers
pub trait MeshBuilder: Default {
    /// Add a vertex with position and normal, returning its index
    fn add_vertex(&mut self, position: Vec3, normal: Vec3) -> u16;

    /// Add a triangle using three vertex indices
    fn add_triangle(&mut self, i0: u16, i1: u16, i2: u16);
}

/// Trait extension for UV-mapped meshes
pub trait MeshBuilderUV: MeshBuilder {
    /// Add a vertex with position, UV coordinates, and normal, returning its index
    fn add_vertex_uv(&mut self, position: Vec3, uv: (f32, f32), normal: Vec3) -> u16;
}

/// Vertex with position and normal (no UVs - for solid color rendering)
#[derive(Clone, Copy, Debug)]
pub(super) struct Vertex {
    pub position: Vec3,
    pub normal: Vec3,
}

impl Vertex {
    /// Create a new vertex
    pub fn new(position: Vec3, normal: Vec3) -> Self {
        Self { position, normal }
    }
}

/// Vertex with position, UV coordinates, and normal (for textured rendering)
#[derive(Clone, Copy, Debug)]
pub(super) struct VertexUV {
    pub position: Vec3,
    pub uv: (f32, f32),
    pub normal: Vec3,
}

impl VertexUV {
    /// Create a new UV vertex
    pub fn new(position: Vec3, uv: (f32, f32), normal: Vec3) -> Self {
        Self {
            position,
            uv,
            normal,
        }
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
    pub fn new() -> Self {
        Self {
            vertices: Vec::new(),
            indices: Vec::new(),
        }
    }

    /// Add a packed vertex (POS_NORMAL) and return its index
    pub(super) fn add_vertex_internal(&mut self, vertex: Vertex) -> u16 {
        let index = (self.vertices.len() / 12) as u16;

        // Pack position as [f16; 4] and cast to bytes using bytemuck
        let pos_packed = pack_position_f16(vertex.position.x, vertex.position.y, vertex.position.z);
        self.vertices.extend_from_slice(cast_slice(&pos_packed)); // [f16; 4] → &[u8]

        // Pack normal as octahedral u32 (4 bytes)
        let norm_packed = pack_normal_octahedral(vertex.normal.x, vertex.normal.y, vertex.normal.z);
        self.vertices.extend_from_slice(&norm_packed.to_le_bytes()); // u32 → &[u8; 4]

        index
    }
}

impl Default for MeshData {
    fn default() -> Self {
        Self::new()
    }
}

impl MeshBuilder for MeshData {
    fn add_vertex(&mut self, position: Vec3, normal: Vec3) -> u16 {
        self.add_vertex_internal(Vertex::new(position, normal))
    }

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
    pub fn new() -> Self {
        Self {
            vertices: Vec::new(),
            indices: Vec::new(),
        }
    }

    /// Add a packed UV vertex and return its index
    pub(super) fn add_vertex_internal(&mut self, vertex: VertexUV) -> u16 {
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
}

impl Default for MeshDataUV {
    fn default() -> Self {
        Self::new()
    }
}

impl MeshBuilder for MeshDataUV {
    fn add_vertex(&mut self, position: Vec3, normal: Vec3) -> u16 {
        self.add_vertex_internal(VertexUV::new(position, (0.0, 0.0), normal))
    }

    fn add_triangle(&mut self, i0: u16, i1: u16, i2: u16) {
        self.indices.push(i0);
        self.indices.push(i1);
        self.indices.push(i2);
    }
}

impl MeshBuilderUV for MeshDataUV {
    fn add_vertex_uv(&mut self, position: Vec3, uv: (f32, f32), normal: Vec3) -> u16 {
        self.add_vertex_internal(VertexUV::new(position, uv, normal))
    }
}

/// Unpacked mesh data (f32 format) for export and modifiers
///
/// Unlike `MeshData` which uses packed formats (f16, octahedral), this stores
/// full-precision f32 values suitable for:
/// - OBJ file export
/// - Mesh modifiers (subdivision, chamfer, etc.)
/// - Advanced geometry processing
#[derive(Clone)]
pub struct UnpackedMesh {
    /// Vertex positions as [x, y, z]
    pub positions: Vec<[f32; 3]>,
    /// Vertex normals as [x, y, z]
    pub normals: Vec<[f32; 3]>,
    /// UV coordinates as [u, v] (empty if no UVs)
    pub uvs: Vec<[f32; 2]>,
    /// Triangle indices (u16 for GPU compatibility)
    pub indices: Vec<u16>,
}

impl UnpackedMesh {
    /// Create empty unpacked mesh
    pub fn new() -> Self {
        Self {
            positions: Vec::new(),
            normals: Vec::new(),
            uvs: Vec::new(),
            indices: Vec::new(),
        }
    }

    /// Get vertex count
    pub fn vertex_count(&self) -> usize {
        self.positions.len()
    }

    /// Get triangle count
    pub fn triangle_count(&self) -> usize {
        self.indices.len() / 3
    }
}

impl Default for UnpackedMesh {
    fn default() -> Self {
        Self::new()
    }
}

impl MeshBuilder for UnpackedMesh {
    fn add_vertex(&mut self, position: Vec3, normal: Vec3) -> u16 {
        let index = self.positions.len() as u16;
        self.positions.push([position.x, position.y, position.z]);
        self.normals.push([normal.x, normal.y, normal.z]);
        index
    }

    fn add_triangle(&mut self, i0: u16, i1: u16, i2: u16) {
        self.indices.push(i0);
        self.indices.push(i1);
        self.indices.push(i2);
    }
}

impl MeshBuilderUV for UnpackedMesh {
    fn add_vertex_uv(&mut self, position: Vec3, uv: (f32, f32), normal: Vec3) -> u16 {
        let index = self.positions.len() as u16;
        self.positions.push([position.x, position.y, position.z]);
        self.normals.push([normal.x, normal.y, normal.z]);
        self.uvs.push([uv.0, uv.1]);
        index
    }
}
