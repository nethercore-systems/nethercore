//! Types and constants for mesh conversion

/// Maximum index value for u16 indices (65535)
/// Meshes with more vertices must be split before export.
pub(crate) const MAX_INDEX_VALUE: u32 = u16::MAX as u32;

/// Maximum joint index for u8 storage (255)
/// Skeletons with more bones are not supported.
pub(crate) const MAX_JOINT_INDEX: u16 = u8::MAX as u16;

/// Skinning data: tuple of (bone indices, bone weights)
pub(crate) type SkinningData<'a> = (&'a [[u8; 4]], &'a [[f32; 4]]);

/// Result of in-memory mesh conversion
pub struct ConvertedMesh {
    /// Format flags (UV, normal, etc.)
    pub format: u8,
    /// Number of vertices
    pub vertex_count: u32,
    /// Number of indices
    pub index_count: u32,
    /// Packed vertex data
    pub vertex_data: Vec<u8>,
    /// Index data (u16)
    pub indices: Vec<u16>,
}
