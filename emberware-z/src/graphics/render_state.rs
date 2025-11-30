//! Render state management
//!
//! Defines render state enums (cull mode, blend mode, texture filter),
//! texture handles, sky uniforms, and the overall render state struct.

use glam::Vec4;

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

    pub fn to_wgpu(self) -> wgpu::FilterMode {
        match self {
            TextureFilter::Nearest => wgpu::FilterMode::Nearest,
            TextureFilter::Linear => wgpu::FilterMode::Linear,
        }
    }
}

/// Sky rendering parameters for procedural sky system
///
/// All zeros = black sky (no sun, no lighting)
/// Call set_sky() in init() to configure
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct SkyUniforms {
    /// Horizon color (RGB, linear) - .w unused
    pub horizon_color: [f32; 4],
    /// Zenith color (RGB, linear) - .w unused
    pub zenith_color: [f32; 4],
    /// Sun direction (normalized vector) - .w unused
    pub sun_direction: [f32; 4],
    /// Sun color and sharpness - .xyz = color (RGB, linear), .w = sharpness
    pub sun_color_and_sharpness: [f32; 4],
}

impl Default for SkyUniforms {
    fn default() -> Self {
        Self {
            horizon_color: [0.0, 0.0, 0.0, 0.0],
            zenith_color: [0.0, 0.0, 0.0, 0.0],
            sun_direction: [0.0, 1.0, 0.0, 0.0], // Up by default, .w unused
            sun_color_and_sharpness: [0.0, 0.0, 0.0, 0.0],
        }
    }
}

// SAFETY: SkyUniforms is #[repr(C)] with only primitive types (f32 arrays).
// All bit patterns are valid for f32, satisfying Pod and Zeroable requirements.
// Vec4 types ensure proper GPU alignment (16-byte boundaries).
unsafe impl bytemuck::Pod for SkyUniforms {}
unsafe impl bytemuck::Zeroable for SkyUniforms {}

/// Camera uniforms for view/projection and specular calculations
///
/// Provides view matrix, projection matrix, and camera position for PBR rendering
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct CameraUniforms {
    /// View matrix (world-to-camera transform)
    pub view: [[f32; 4]; 4],
    /// Projection matrix
    pub projection: [[f32; 4]; 4],
    /// Camera world position - .xyz = position, .w unused
    pub position: [f32; 4],
}

impl Default for CameraUniforms {
    fn default() -> Self {
        Self {
            view: [[0.0; 4]; 4],
            projection: [[0.0; 4]; 4],
            position: [0.0, 0.0, 0.0, 0.0],
        }
    }
}

// SAFETY: CameraUniforms is #[repr(C)] with only primitive types (f32 arrays).
// All bit patterns are valid for f32, satisfying Pod and Zeroable requirements.
unsafe impl bytemuck::Pod for CameraUniforms {}
unsafe impl bytemuck::Zeroable for CameraUniforms {}

/// Single light uniform (directional light)
///
/// Used in array of 4 lights for PBR rendering
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct LightUniform {
    /// Light direction (normalized) - .xyz = direction, .w = enabled (1.0 or 0.0)
    pub direction_and_enabled: [f32; 4],
    /// Light color and intensity - .xyz = color (RGB, linear), .w = intensity
    pub color_and_intensity: [f32; 4],
}

impl Default for LightUniform {
    fn default() -> Self {
        Self {
            direction_and_enabled: [0.0, -1.0, 0.0, 0.0], // Downward, disabled
            color_and_intensity: [1.0, 1.0, 1.0, 1.0],    // White, full intensity
        }
    }
}

// SAFETY: LightUniform is #[repr(C)] with only primitive types (f32 arrays).
unsafe impl bytemuck::Pod for LightUniform {}
unsafe impl bytemuck::Zeroable for LightUniform {}

/// Lights uniforms buffer (4 directional lights for PBR)
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct LightsUniforms {
    /// Array of 4 directional lights
    pub lights: [LightUniform; 4],
}

// SAFETY: LightsUniforms is #[repr(C)] with array of Pod types.
unsafe impl bytemuck::Pod for LightsUniforms {}
unsafe impl bytemuck::Zeroable for LightsUniforms {}

/// Material properties for PBR rendering
///
/// Stores global material properties (metallic, roughness, emissive)
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct MaterialUniforms {
    /// Material properties - .x = metallic, .y = roughness, .z = emissive, .w unused
    pub properties: [f32; 4],
}

impl Default for MaterialUniforms {
    fn default() -> Self {
        Self {
            properties: [0.0, 0.5, 0.0, 0.0], // Non-metallic, medium roughness, no emissive
        }
    }
}

// SAFETY: MaterialUniforms is #[repr(C)] with only primitive types (f32 arrays).
unsafe impl bytemuck::Pod for MaterialUniforms {}
unsafe impl bytemuck::Zeroable for MaterialUniforms {}

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
}

/// Current render state (tracks what needs pipeline changes)
#[derive(Debug, Clone, Copy, PartialEq)]
/// Uniform tint color (0xRRGGBBAA)
pub struct RenderState {
    pub color: u32,
    /// Depth test enabled
    pub depth_test: bool,
    /// Face culling mode
    pub cull_mode: CullMode,
    /// Blending mode
    pub blend_mode: BlendMode,
    /// Texture filter mode
    pub texture_filter: TextureFilter,
    /// Bound textures per slot (0-3)
    pub texture_slots: [TextureHandle; 4],
    /// Matcap blend modes for slots 1-3 (Mode 1 only, [0] unused)
    pub matcap_blend_modes: [MatcapBlendMode; 4],
}

impl Default for RenderState {
    fn default() -> Self {
        Self {
            color: 0xFFFFFFFF, // White, fully opaque
            depth_test: true,
            cull_mode: CullMode::Back,
            blend_mode: BlendMode::None,
            texture_filter: TextureFilter::Nearest,
            texture_slots: [TextureHandle::INVALID; 4],
            matcap_blend_modes: [MatcapBlendMode::Multiply; 4],
        }
    }
}

impl RenderState {
    /// Get color as Vec4 (RGBA, 0.0-1.0)
    pub fn color_vec4(&self) -> Vec4 {
        Vec4::new(
            ((self.color >> 24) & 0xFF) as f32 / 255.0,
            ((self.color >> 16) & 0xFF) as f32 / 255.0,
            ((self.color >> 8) & 0xFF) as f32 / 255.0,
            (self.color & 0xFF) as f32 / 255.0,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_texture_handle_invalid() {
        assert_eq!(TextureHandle::INVALID, TextureHandle(0));
    }

    #[test]
    fn test_render_state_default() {
        let state = RenderState::default();
        assert_eq!(state.color, 0xFFFFFFFF);
        assert!(state.depth_test);
        assert_eq!(state.cull_mode, CullMode::Back);
        assert_eq!(state.blend_mode, BlendMode::None);
        assert_eq!(state.texture_filter, TextureFilter::Nearest);
        assert_eq!(state.texture_slots, [TextureHandle::INVALID; 4]);
    }

    #[test]
    fn test_render_state_color_vec4() {
        let state = RenderState {
            color: 0xFF8040C0,
            ..Default::default()
        };
        let v = state.color_vec4();
        assert!((v.x - 1.0).abs() < 0.01);
        assert!((v.y - 0.502).abs() < 0.01);
        assert!((v.z - 0.251).abs() < 0.01);
        assert!((v.w - 0.753).abs() < 0.01);
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
    fn test_sky_uniforms_default() {
        let sky = SkyUniforms::default();
        assert_eq!(sky.horizon_color, [0.0, 0.0, 0.0, 0.0]);
        assert_eq!(sky.zenith_color, [0.0, 0.0, 0.0, 0.0]);
        assert_eq!(sky.sun_direction, [0.0, 1.0, 0.0, 0.0]);
        assert_eq!(sky.sun_color_and_sharpness, [0.0, 0.0, 0.0, 0.0]);
    }

    #[test]
    fn test_sky_uniforms_custom() {
        let sky = SkyUniforms {
            horizon_color: [1.0, 0.5, 0.2, 0.0],
            zenith_color: [0.2, 0.4, 1.0, 0.0],
            sun_direction: [0.577, 0.577, 0.577, 0.0],
            sun_color_and_sharpness: [1.5, 1.4, 1.0, 64.0],
        };
        assert_eq!(sky.horizon_color, [1.0, 0.5, 0.2, 0.0]);
        assert_eq!(sky.zenith_color, [0.2, 0.4, 1.0, 0.0]);
        assert_eq!(sky.sun_color_and_sharpness[3], 64.0);
    }

    #[test]
    fn test_sky_uniforms_size() {
        assert_eq!(std::mem::size_of::<SkyUniforms>(), 64);
    }

    #[test]
    fn test_sky_uniforms_alignment() {
        assert!(std::mem::align_of::<SkyUniforms>() <= 16);
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

    #[test]
    fn test_render_state_color_changes() {
        let mut state = RenderState::default();
        assert_eq!(state.color, 0xFFFFFFFF);

        state.color = 0xFF0000FF;
        let v = state.color_vec4();
        assert!((v.x - 1.0).abs() < 0.01);
        assert!((v.y - 0.0).abs() < 0.01);
        assert!((v.z - 0.0).abs() < 0.01);
        assert!((v.w - 1.0).abs() < 0.01);

        state.color = 0x00000000;
        let v = state.color_vec4();
        assert!((v.x - 0.0).abs() < 0.01);
        assert!((v.y - 0.0).abs() < 0.01);
        assert!((v.z - 0.0).abs() < 0.01);
        assert!((v.w - 0.0).abs() < 0.01);
    }

    #[test]
    fn test_render_state_texture_filter_switching() {
        let mut state = RenderState::default();
        assert_eq!(state.texture_filter, TextureFilter::Nearest);
        state.texture_filter = TextureFilter::Linear;
        assert_eq!(state.texture_filter, TextureFilter::Linear);
        assert_eq!(state.texture_filter.to_wgpu(), wgpu::FilterMode::Linear);
        state.texture_filter = TextureFilter::Nearest;
        assert_eq!(state.texture_filter, TextureFilter::Nearest);
        assert_eq!(state.texture_filter.to_wgpu(), wgpu::FilterMode::Nearest);
    }

    #[test]
    fn test_render_state_equality() {
        let state1 = RenderState::default();
        let state2 = RenderState::default();
        let state3 = RenderState {
            color: 0xFF0000FF,
            ..Default::default()
        };

        assert_eq!(state1, state2);
        assert_ne!(state1, state3);
    }

    #[test]
    fn test_render_state_clone() {
        let state1 = RenderState {
            color: 0x12345678,
            depth_test: false,
            cull_mode: CullMode::Front,
            blend_mode: BlendMode::Additive,
            texture_filter: TextureFilter::Linear,
            texture_slots: [
                TextureHandle(1),
                TextureHandle(2),
                TextureHandle(3),
                TextureHandle(4),
            ],
            matcap_blend_modes: [MatcapBlendMode::Multiply; 4],
        };

        let state2 = state1;
        assert_eq!(state1.color, state2.color);
        assert_eq!(state1.depth_test, state2.depth_test);
        assert_eq!(state1.cull_mode, state2.cull_mode);
        assert_eq!(state1.blend_mode, state2.blend_mode);
        assert_eq!(state1.texture_filter, state2.texture_filter);
        assert_eq!(state1.texture_slots, state2.texture_slots);
    }

    #[test]
    fn test_texture_slot_binding() {
        let mut state = RenderState::default();

        for slot in 0..4 {
            assert_eq!(state.texture_slots[slot], TextureHandle::INVALID);
        }

        state.texture_slots[0] = TextureHandle(1);
        assert_eq!(state.texture_slots[0], TextureHandle(1));
        assert_eq!(state.texture_slots[1], TextureHandle::INVALID);

        state.texture_slots[2] = TextureHandle(5);
        assert_eq!(state.texture_slots[2], TextureHandle(5));
    }

    #[test]
    fn test_texture_slot_rebinding() {
        let mut state = RenderState::default();

        state.texture_slots[0] = TextureHandle(1);
        assert_eq!(state.texture_slots[0], TextureHandle(1));

        state.texture_slots[0] = TextureHandle(2);
        assert_eq!(state.texture_slots[0], TextureHandle(2));

        state.texture_slots[0] = TextureHandle::INVALID;
        assert_eq!(state.texture_slots[0], TextureHandle::INVALID);
    }

    #[test]
    fn test_texture_slots_all_bound() {
        let mut state = RenderState::default();

        for slot in 0..4 {
            state.texture_slots[slot] = TextureHandle((slot + 1) as u32);
        }

        for slot in 0..4 {
            assert_eq!(state.texture_slots[slot], TextureHandle((slot + 1) as u32));
        }
    }
}
