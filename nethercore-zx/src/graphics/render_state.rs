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

// =============================================================================
// PassConfig - Render Pass System
// =============================================================================

/// Configuration for a render pass with depth/stencil state.
///
/// Each pass has its own depth and stencil configuration. Passes provide
/// execution barriers - commands in pass N are guaranteed to complete before
/// commands in pass N+1 begin.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PassConfig {
    /// Depth comparison function
    pub depth_compare: wgpu::CompareFunction,
    /// Whether to write to depth buffer
    pub depth_write: bool,
    /// Whether to clear depth buffer at pass start
    pub depth_clear: bool,
    /// Stencil comparison function
    pub stencil_compare: wgpu::CompareFunction,
    /// Stencil reference value (0-255)
    pub stencil_ref: u8,
    /// Stencil operation when stencil test passes
    pub stencil_pass: wgpu::StencilOperation,
    /// Stencil operation when stencil test fails
    pub stencil_fail: wgpu::StencilOperation,
    /// Stencil operation when depth test fails
    pub stencil_depth_fail: wgpu::StencilOperation,
}

impl Default for PassConfig {
    fn default() -> Self {
        Self {
            depth_compare: wgpu::CompareFunction::Less,
            depth_write: true,
            depth_clear: false,
            stencil_compare: wgpu::CompareFunction::Always,
            stencil_ref: 0,
            stencil_pass: wgpu::StencilOperation::Keep,
            stencil_fail: wgpu::StencilOperation::Keep,
            stencil_depth_fail: wgpu::StencilOperation::Keep,
        }
    }
}

impl PassConfig {
    /// Standard pass with optional depth clear.
    /// Depth: compare=LESS, write=ON
    /// Stencil: disabled
    pub fn standard(depth_clear: bool) -> Self {
        Self {
            depth_clear,
            ..Default::default()
        }
    }

    /// Stencil write pass (mask creation).
    /// Depth: compare=ALWAYS, write=OFF (prevents mask polluting depth buffer)
    /// Stencil: write ref_value on pass/depth_fail
    /// Color: disabled (mask creation only)
    pub fn stencil_write(ref_value: u8, depth_clear: bool) -> Self {
        Self {
            depth_compare: wgpu::CompareFunction::Always,
            depth_write: false,
            depth_clear,
            stencil_compare: wgpu::CompareFunction::Always,
            stencil_ref: ref_value,
            stencil_pass: wgpu::StencilOperation::Replace,
            stencil_fail: wgpu::StencilOperation::Keep,
            stencil_depth_fail: wgpu::StencilOperation::Replace,
        }
    }

    /// Stencil test pass (render inside mask).
    /// Depth: compare=LESS, write=ON
    /// Stencil: only render where stencil == ref_value
    pub fn stencil_test(ref_value: u8, depth_clear: bool) -> Self {
        Self {
            depth_clear,
            stencil_compare: wgpu::CompareFunction::Equal,
            stencil_ref: ref_value,
            ..Default::default()
        }
    }

    /// Returns true if this pass writes to stencil buffer
    pub fn writes_stencil(&self) -> bool {
        self.stencil_pass != wgpu::StencilOperation::Keep
            || self.stencil_fail != wgpu::StencilOperation::Keep
            || self.stencil_depth_fail != wgpu::StencilOperation::Keep
    }

    /// Returns true if this pass tests against stencil buffer
    pub fn tests_stencil(&self) -> bool {
        self.stencil_compare != wgpu::CompareFunction::Always
    }

    /// Returns true if stencil operations are active
    pub fn is_stencil_active(&self) -> bool {
        self.writes_stencil() || self.tests_stencil()
    }

    /// Get color write mask (disabled during stencil-only writing)
    pub fn color_write_mask(&self) -> wgpu::ColorWrites {
        // Disable color output when writing to stencil but not testing
        if self.writes_stencil() && !self.tests_stencil() {
            wgpu::ColorWrites::empty()
        } else {
            wgpu::ColorWrites::ALL
        }
    }

    /// Generate wgpu StencilState from this config
    pub fn to_wgpu_stencil_state(&self) -> wgpu::StencilState {
        let face_state = wgpu::StencilFaceState {
            compare: self.stencil_compare,
            fail_op: self.stencil_fail,
            depth_fail_op: self.stencil_depth_fail,
            pass_op: self.stencil_pass,
        };

        wgpu::StencilState {
            front: face_state,
            back: face_state,
            read_mask: 0xFF,
            write_mask: if self.writes_stencil() { 0xFF } else { 0x00 },
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
