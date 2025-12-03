/// Packed MVP matrix indices (model: 16 bits, view: 8 bits, proj: 8 bits)
///
/// This allows addressing:
/// - 65,536 model matrices
/// - 256 view matrices
/// - 256 projection matrices
///
/// The packed format stores all three indices in a single u32:
/// - Bits 0-15: Model matrix index
/// - Bits 16-23: View matrix index
/// - Bits 24-31: Projection matrix index
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct MvpIndex(pub u32);

impl MvpIndex {
    /// Invalid/uninitialized MVP index
    pub const INVALID: Self = Self(0);

    /// Pack three matrix indices into a single u32
    ///
    /// # Panics
    /// In debug builds, panics if indices exceed their bit limits:
    /// - model must be < 65536 (16 bits)
    /// - view must be < 256 (8 bits)
    /// - proj must be < 256 (8 bits)
    pub fn new(model: u32, view: u32, proj: u32) -> Self {
        debug_assert!(
            model < 65536,
            "Model index must fit in 16 bits (got {})",
            model
        );
        debug_assert!(view < 256, "View index must fit in 8 bits (got {})", view);
        debug_assert!(
            proj < 256,
            "Projection index must fit in 8 bits (got {})",
            proj
        );

        Self((model & 0xFFFF) | ((view & 0xFF) << 16) | ((proj & 0xFF) << 24))
    }

    /// Unpack into (model, view, proj) indices
    #[inline]
    pub fn unpack(self) -> (u32, u32, u32) {
        let model = self.0 & 0xFFFF;
        let view = (self.0 >> 16) & 0xFF;
        let proj = (self.0 >> 24) & 0xFF;
        (model, view, proj)
    }

    /// Extract model matrix index (bits 0-15)
    #[inline]
    pub fn model_index(self) -> u32 {
        self.0 & 0xFFFF
    }

    /// Extract view matrix index (bits 16-23)
    #[inline]
    pub fn view_index(self) -> u32 {
        (self.0 >> 16) & 0xFF
    }

    /// Extract projection matrix index (bits 24-31)
    #[inline]
    pub fn proj_index(self) -> u32 {
        (self.0 >> 24) & 0xFF
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mvp_index_packing() {
        let idx = MvpIndex::new(12345, 67, 89);
        assert_eq!(idx.model_index(), 12345);
        assert_eq!(idx.view_index(), 67);
        assert_eq!(idx.proj_index(), 89);

        let (m, v, p) = idx.unpack();
        assert_eq!(m, 12345);
        assert_eq!(v, 67);
        assert_eq!(p, 89);
    }

    #[test]
    fn test_mvp_index_max_values() {
        let idx = MvpIndex::new(65535, 255, 255);
        assert_eq!(idx.model_index(), 65535);
        assert_eq!(idx.view_index(), 255);
        assert_eq!(idx.proj_index(), 255);
    }

    #[test]
    fn test_mvp_index_zero() {
        let idx = MvpIndex::new(0, 0, 0);
        assert_eq!(idx.model_index(), 0);
        assert_eq!(idx.view_index(), 0);
        assert_eq!(idx.proj_index(), 0);
    }
}
