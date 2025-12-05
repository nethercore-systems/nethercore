/// Unpacked MVP + shading indices for GPU upload (16 bytes, vec4<u32> in WGSL)
/// Uses all 4 × u32 fields naturally without bit-packing
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct MvpShadingIndices {
    pub model_idx: u32,
    pub view_idx: u32,
    pub proj_idx: u32,
    pub shading_idx: u32,
}

// Safety: MvpShadingIndices is repr(C) with only u32 fields
unsafe impl bytemuck::Pod for MvpShadingIndices {}
unsafe impl bytemuck::Zeroable for MvpShadingIndices {}
