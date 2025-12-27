//! Render state management
//!
//! Defines render state enums (cull mode, blend mode, texture filter),
//! texture handles, sky uniforms, and the overall render state struct.

/// Handle to a loaded texture
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TextureHandle(pub u32);

impl TextureHandle {
    /// Invalid/null texture handle
    pub const INVALID: TextureHandle = TextureHandle(0);
}

/// Cull mode for face culling
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u32)]
pub enum CullMode {
    /// No face culling
    #[default]
    None = 0,
    /// Cull back faces
    Back = 1,
    /// Cull front faces
    Front = 2,
}

impl CullMode {
    pub fn from_u32(value: u32) -> Self {
        match value {
            0 => CullMode::None,
            1 => CullMode::Back,
            2 => CullMode::Front,
            _ => CullMode::None,
        }
    }

    pub fn from_u8(value: u8) -> Self {
        match value {
            0 => CullMode::None,
            1 => CullMode::Back,
            2 => CullMode::Front,
            _ => CullMode::None,
        }
    }

    pub fn to_wgpu(self) -> Option<wgpu::Face> {
        match self {
            CullMode::None => None,
            CullMode::Back => Some(wgpu::Face::Back),
            CullMode::Front => Some(wgpu::Face::Front),
        }
    }
}

/// Texture filter mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u32)]
pub enum TextureFilter {
    /// Nearest neighbor (pixelated)
    #[default]
    Nearest = 0,
    /// Linear interpolation (smooth)
    Linear = 1,
}

impl TextureFilter {
    pub fn from_u32(value: u32) -> Self {
        match value {
            0 => TextureFilter::Nearest,
            1 => TextureFilter::Linear,
            _ => TextureFilter::Nearest,
        }
    }

    pub fn from_u8(value: u8) -> Self {
        match value {
            0 => TextureFilter::Nearest,
            1 => TextureFilter::Linear,
            _ => TextureFilter::Nearest,
        }
    }

    pub fn to_wgpu(self) -> wgpu::FilterMode {
        match self {
            TextureFilter::Nearest => wgpu::FilterMode::Nearest,
            TextureFilter::Linear => wgpu::FilterMode::Linear,
        }
    }
}

/// Matcap blend mode (Mode 1 only, slots 1-3)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u8)]
pub enum MatcapBlendMode {
    /// Multiply (default)
    #[default]
    Multiply = 0,
    /// Add (glow/emission)
    Add = 1,
    /// HSV Modulate (hue shift/iridescence)
    HsvModulate = 2,
}

impl MatcapBlendMode {
    pub fn from_u32(value: u32) -> Option<Self> {
        match value {
            0 => Some(MatcapBlendMode::Multiply),
            1 => Some(MatcapBlendMode::Add),
            2 => Some(MatcapBlendMode::HsvModulate),
            _ => None,
        }
    }

    pub fn from_u8(value: u8) -> Self {
        match value {
            0 => MatcapBlendMode::Multiply,
            1 => MatcapBlendMode::Add,
            2 => MatcapBlendMode::HsvModulate,
            _ => MatcapBlendMode::Multiply,
        }
    }
}

/// Stencil mode for masked rendering (split-screen, portals, scopes)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default, Hash)]
#[repr(u8)]
pub enum StencilMode {
    /// Normal rendering, no stencil operations
    #[default]
    Disabled = 0,
    /// Write to stencil buffer (mask creation), no color output
    Writing = 1,
    /// Only render where stencil == 1 (inside mask)
    Testing = 2,
    /// Only render where stencil == 0 (outside mask)
    TestingInverted = 3,
}

impl StencilMode {
    pub fn from_u8(value: u8) -> Self {
        match value {
            0 => StencilMode::Disabled,
            1 => StencilMode::Writing,
            2 => StencilMode::Testing,
            3 => StencilMode::TestingInverted,
            _ => StencilMode::Disabled,
        }
    }

    /// Returns true if this mode requires stencil operations
    pub fn is_active(&self) -> bool {
        !matches!(self, StencilMode::Disabled)
    }

    /// Returns true if this mode writes to stencil buffer
    pub fn writes_stencil(&self) -> bool {
        matches!(self, StencilMode::Writing)
    }

    /// Returns true if this mode tests against stencil buffer
    pub fn tests_stencil(&self) -> bool {
        matches!(self, StencilMode::Testing | StencilMode::TestingInverted)
    }

    /// Get the wgpu stencil state for this mode
    pub fn to_wgpu_stencil_state(&self) -> wgpu::StencilState {
        match self {
            StencilMode::Disabled => wgpu::StencilState::default(),
            StencilMode::Writing => wgpu::StencilState {
                front: wgpu::StencilFaceState {
                    compare: wgpu::CompareFunction::Always,
                    fail_op: wgpu::StencilOperation::Keep,
                    depth_fail_op: wgpu::StencilOperation::Keep,
                    pass_op: wgpu::StencilOperation::Replace,
                },
                back: wgpu::StencilFaceState {
                    compare: wgpu::CompareFunction::Always,
                    fail_op: wgpu::StencilOperation::Keep,
                    depth_fail_op: wgpu::StencilOperation::Keep,
                    pass_op: wgpu::StencilOperation::Replace,
                },
                read_mask: 0xFF,
                write_mask: 0xFF,
            },
            StencilMode::Testing => wgpu::StencilState {
                front: wgpu::StencilFaceState {
                    compare: wgpu::CompareFunction::Equal,
                    fail_op: wgpu::StencilOperation::Keep,
                    depth_fail_op: wgpu::StencilOperation::Keep,
                    pass_op: wgpu::StencilOperation::Keep,
                },
                back: wgpu::StencilFaceState {
                    compare: wgpu::CompareFunction::Equal,
                    fail_op: wgpu::StencilOperation::Keep,
                    depth_fail_op: wgpu::StencilOperation::Keep,
                    pass_op: wgpu::StencilOperation::Keep,
                },
                read_mask: 0xFF,
                write_mask: 0x00,
            },
            StencilMode::TestingInverted => wgpu::StencilState {
                front: wgpu::StencilFaceState {
                    compare: wgpu::CompareFunction::NotEqual,
                    fail_op: wgpu::StencilOperation::Keep,
                    depth_fail_op: wgpu::StencilOperation::Keep,
                    pass_op: wgpu::StencilOperation::Keep,
                },
                back: wgpu::StencilFaceState {
                    compare: wgpu::CompareFunction::NotEqual,
                    fail_op: wgpu::StencilOperation::Keep,
                    depth_fail_op: wgpu::StencilOperation::Keep,
                    pass_op: wgpu::StencilOperation::Keep,
                },
                read_mask: 0xFF,
                write_mask: 0x00,
            },
        }
    }

    /// Returns the color write mask for this mode
    pub fn color_write_mask(&self) -> wgpu::ColorWrites {
        match self {
            // When writing to stencil, don't write color (mask creation only)
            StencilMode::Writing => wgpu::ColorWrites::empty(),
            // All other modes write color normally
            _ => wgpu::ColorWrites::ALL,
        }
    }
}

/// Current render state (tracks what needs pipeline changes)
///
/// Note: texture_filter is not part of this struct - it's stored in
/// PackedUnifiedShadingState.flags (bit 1) for per-draw selection
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RenderState {
    /// Depth test enabled
    pub depth_test: bool,
    /// Face culling mode
    pub cull_mode: CullMode,
}

impl Default for RenderState {
    fn default() -> Self {
        Self {
            depth_test: true,
            cull_mode: CullMode::None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_state_default() {
        let state = RenderState::default();
        assert!(state.depth_test);
        assert_eq!(state.cull_mode, CullMode::None);
        // Note: texture_filter is not part of RenderState
        // It's in PackedUnifiedShadingState.flags (bit 1)
    }

    #[test]
    fn test_cull_mode_conversion() {
        assert_eq!(CullMode::from_u32(0), CullMode::None);
        assert_eq!(CullMode::from_u32(1), CullMode::Back);
        assert_eq!(CullMode::from_u32(2), CullMode::Front);
        assert_eq!(CullMode::from_u32(99), CullMode::None);

        assert!(CullMode::None.to_wgpu().is_none());
        assert_eq!(CullMode::Back.to_wgpu(), Some(wgpu::Face::Back));
        assert_eq!(CullMode::Front.to_wgpu(), Some(wgpu::Face::Front));
    }

    #[test]
    fn test_texture_filter_conversion() {
        assert_eq!(TextureFilter::from_u32(0), TextureFilter::Nearest);
        assert_eq!(TextureFilter::from_u32(1), TextureFilter::Linear);
        assert_eq!(TextureFilter::from_u32(99), TextureFilter::Nearest);

        assert_eq!(TextureFilter::Nearest.to_wgpu(), wgpu::FilterMode::Nearest);
        assert_eq!(TextureFilter::Linear.to_wgpu(), wgpu::FilterMode::Linear);
    }

    #[test]
    fn test_render_state_depth_test_toggle() {
        let mut state = RenderState::default();
        assert!(state.depth_test);
        state.depth_test = false;
        assert!(!state.depth_test);
        state.depth_test = true;
        assert!(state.depth_test);
    }

    #[test]
    fn test_render_state_cull_mode_switching() {
        let mut state = RenderState::default();
        assert_eq!(state.cull_mode, CullMode::None);
        state.cull_mode = CullMode::Back;
        assert_eq!(state.cull_mode, CullMode::Back);
        state.cull_mode = CullMode::Front;
        assert_eq!(state.cull_mode, CullMode::Front);
        state.cull_mode = CullMode::None;
        assert_eq!(state.cull_mode, CullMode::None);
    }

    // Note: test_render_state_texture_filter_switching removed - texture_filter
    // is now in PackedUnifiedShadingState.flags (bit 1), not RenderState

    #[test]
    fn test_render_state_equality() {
        let state1 = RenderState::default();
        let state2 = RenderState::default();
        let state3 = RenderState {
            cull_mode: CullMode::Back,
            ..Default::default()
        };

        assert_eq!(state1, state2);
        assert_ne!(state1, state3);
    }
}
