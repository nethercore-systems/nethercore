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

/// Blend mode for alpha blending
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u32)]
pub enum BlendMode {
    /// No blending (opaque)
    #[default]
    None = 0,
    /// Standard alpha blending
    Alpha = 1,
    /// Additive blending
    Additive = 2,
    /// Multiply blending
    Multiply = 3,
}

impl BlendMode {
    pub fn from_u32(value: u32) -> Self {
        match value {
            0 => BlendMode::None,
            1 => BlendMode::Alpha,
            2 => BlendMode::Additive,
            3 => BlendMode::Multiply,
            _ => BlendMode::None,
        }
    }

    pub fn from_u8(value: u8) -> Self {
        match value {
            0 => BlendMode::None,
            1 => BlendMode::Alpha,
            2 => BlendMode::Additive,
            3 => BlendMode::Multiply,
            _ => BlendMode::None,
        }
    }

    pub fn to_wgpu(self) -> Option<wgpu::BlendState> {
        match self {
            BlendMode::None => None,
            BlendMode::Alpha => Some(wgpu::BlendState::ALPHA_BLENDING),
            BlendMode::Additive => Some(wgpu::BlendState {
                color: wgpu::BlendComponent {
                    src_factor: wgpu::BlendFactor::SrcAlpha,
                    dst_factor: wgpu::BlendFactor::One,
                    operation: wgpu::BlendOperation::Add,
                },
                alpha: wgpu::BlendComponent {
                    src_factor: wgpu::BlendFactor::One,
                    dst_factor: wgpu::BlendFactor::One,
                    operation: wgpu::BlendOperation::Add,
                },
            }),
            BlendMode::Multiply => Some(wgpu::BlendState {
                color: wgpu::BlendComponent {
                    src_factor: wgpu::BlendFactor::Dst,
                    dst_factor: wgpu::BlendFactor::Zero,
                    operation: wgpu::BlendOperation::Add,
                },
                alpha: wgpu::BlendComponent {
                    src_factor: wgpu::BlendFactor::DstAlpha,
                    dst_factor: wgpu::BlendFactor::Zero,
                    operation: wgpu::BlendOperation::Add,
                },
            }),
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
    /// Blending mode
    pub blend_mode: BlendMode,
}

impl Default for RenderState {
    fn default() -> Self {
        Self {
            depth_test: true,
            cull_mode: CullMode::Back,
            blend_mode: BlendMode::None,
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
        assert_eq!(state.cull_mode, CullMode::Back);
        assert_eq!(state.blend_mode, BlendMode::None);
        // Note: texture_filter is no longer part of RenderState
        // It's now in PackedUnifiedShadingState.flags (bit 1)
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
    fn test_blend_mode_conversion() {
        assert_eq!(BlendMode::from_u32(0), BlendMode::None);
        assert_eq!(BlendMode::from_u32(1), BlendMode::Alpha);
        assert_eq!(BlendMode::from_u32(2), BlendMode::Additive);
        assert_eq!(BlendMode::from_u32(3), BlendMode::Multiply);
        assert_eq!(BlendMode::from_u32(99), BlendMode::None);

        assert!(BlendMode::None.to_wgpu().is_none());
        assert!(BlendMode::Alpha.to_wgpu().is_some());
        assert!(BlendMode::Additive.to_wgpu().is_some());
        assert!(BlendMode::Multiply.to_wgpu().is_some());
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
        assert_eq!(state.cull_mode, CullMode::Back);
        state.cull_mode = CullMode::Front;
        assert_eq!(state.cull_mode, CullMode::Front);
        state.cull_mode = CullMode::None;
        assert_eq!(state.cull_mode, CullMode::None);
        state.cull_mode = CullMode::Back;
        assert_eq!(state.cull_mode, CullMode::Back);
    }

    #[test]
    fn test_render_state_blend_mode_switching() {
        let mut state = RenderState::default();
        assert_eq!(state.blend_mode, BlendMode::None);
        state.blend_mode = BlendMode::Alpha;
        assert_eq!(state.blend_mode, BlendMode::Alpha);
        state.blend_mode = BlendMode::Additive;
        assert_eq!(state.blend_mode, BlendMode::Additive);
        state.blend_mode = BlendMode::Multiply;
        assert_eq!(state.blend_mode, BlendMode::Multiply);
        state.blend_mode = BlendMode::None;
        assert_eq!(state.blend_mode, BlendMode::None);
    }

    // Note: test_render_state_texture_filter_switching removed - texture_filter
    // is now in PackedUnifiedShadingState.flags (bit 1), not RenderState

    #[test]
    fn test_render_state_equality() {
        let state1 = RenderState::default();
        let state2 = RenderState::default();
        let state3 = RenderState {
            blend_mode: BlendMode::Alpha,
            ..Default::default()
        };

        assert_eq!(state1, state2);
        assert_ne!(state1, state3);
    }
}
