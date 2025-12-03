/// Packed MVP matrix indices (model: 16 bits, view: 8 bits, proj: 8 bits)
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
