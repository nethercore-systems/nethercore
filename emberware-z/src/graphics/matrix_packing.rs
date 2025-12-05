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

/// DEPRECATED: Packed MVP matrix indices (model: 16 bits, view: 8 bits, proj: 8 bits)
/// Kept for compatibility but prefer MvpShadingIndices for GPU uploads
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct MvpIndex(pub u32);

impl MvpIndex {
    pub const INVALID: Self = Self(0);

    /// Pack three matrix indices into a single u32
    pub fn new(model: u32, view: u32, proj: u32) -> Self {
        debug_assert!(model < 65536, "Model index must fit in 16 bits");
        debug_assert!(view < 256, "View index must fit in 8 bits");
        debug_assert!(proj < 256, "Projection index must fit in 8 bits");

        Self((model & 0xFFFF) | ((view & 0xFF) << 16) | ((proj & 0xFF) << 24))
    }

    /// Unpack into (model, view, proj) indices
    pub fn unpack(self) -> (u32, u32, u32) {
        let model = self.0 & 0xFFFF;
        let view = (self.0 >> 16) & 0xFF;
        let proj = (self.0 >> 24) & 0xFF;
        (model, view, proj)
    }

    pub fn model_index(self) -> u32 {
        self.0 & 0xFFFF
    }

    pub fn view_index(self) -> u32 {
        (self.0 >> 16) & 0xFF
    }

    pub fn proj_index(self) -> u32 {
        (self.0 >> 24) & 0xFF
    }
}
