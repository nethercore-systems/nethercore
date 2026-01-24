use bytemuck::{Pod, Zeroable};

use super::light::PackedLight;
use super::quantization::{pack_matcap_blend_modes, pack_uniform_set_0, pack_unorm8};
use crate::graphics::render_state::MatcapBlendMode;

/// Unified per-draw shading state (80 bytes, POD, hashable)
/// Size breakdown: 16 bytes (header) + 48 bytes (lights) + 16 bytes (animation/environment)
///
/// # Mode-Specific Field Interpretation
///
/// The `uniform_set_0` and `uniform_set_1` fields are interpreted differently per render mode.
/// Each is a u32 containing 4 packed u8 values: [byte0, byte1, byte2, byte3].
///
/// Field layout per render mode:
///
/// | Mode | uniform_set_0 [b0, b1, b2, b3]                   | uniform_set_1 [b0, b1, b2, b3]           |
/// |------|--------------------------------------------------|------------------------------------------|
/// | 0    | [unused, unused, unused, Rim Intensity]          | [unused, unused, unused, Rim Power]      |
/// | 1    | [BlendMode0, BlendMode1, BlendMode2, BlendMode3] | [unused, unused, unused, unused]         |
/// | 2    | [Metallic, Roughness, Emissive, Rim Intensity]   | [unused, unused, unused, Rim Power]      |
/// | 3    | [SpecDamping*, Shininess, Emissive, RimIntens]   | [Spec R, Spec G, Spec B, Rim Power]      |
///
/// *SpecDamping is INVERTED: 0=full specular (default), 255=no specular.
/// This is beginner-friendly since the default of 0 gives visible highlights.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Pod, Zeroable)]
pub struct PackedUnifiedShadingState {
    /// Material color (RGBA8 packed)
    pub color_rgba8: u32,

    /// Mode-specific uniform data (4 bytes packed as 4 × u8)
    /// - Mode 0: [unused, unused, unused, rim_intensity]
    /// - Mode 1: [blend_mode_0, blend_mode_1, blend_mode_2, blend_mode_3]
    /// - Mode 2: [metallic, roughness, emissive, rim_intensity]
    /// - Mode 3: [spec_damping*, shininess, emissive, rim_intensity]
    /// * spec_damping is inverted: 0=full specular, 255=no specular
    pub uniform_set_0: u32,

    /// Mode-specific extended data (4 bytes packed as 4 × u8)
    /// - Mode 0: [unused, unused, unused, rim_power]
    /// - Mode 1: unused
    /// - Mode 2: [unused, unused, unused, rim_power]
    /// - Mode 3: [spec_r, spec_g, spec_b, rim_power]
    pub uniform_set_1: u32,

    /// Flags and reserved bits
    /// - Bit 0: skinning_mode (0 = raw, 1 = inverse bind mode)
    /// - Bits 1-31: reserved for future use
    pub flags: u32,

    pub lights: [PackedLight; 4], // 48 bytes (4 × 12-byte lights)

    // Animation system fields (12 bytes)
    /// Base offset into @binding(7) all_keyframes buffer
    /// Shader reads: all_keyframes[keyframe_base + bone_index]
    /// 0 = no keyframes bound (use bones buffer directly)
    pub keyframe_base: u32,

    /// Base offset into @binding(6) all_inverse_bind buffer
    /// Shader reads: inverse_bind[inverse_bind_base + bone_index]
    /// 0 = no skeleton bound (raw bone mode)
    pub inverse_bind_base: u32,

    /// Padding for struct alignment (animation_flags slot unused)
    pub _pad: u32,

    /// EPU environment ID (`env_id`) used for EnvRadiance / SH9 sampling.
    pub environment_index: u32,
}

impl Default for PackedUnifiedShadingState {
    fn default() -> Self {
        // uniform_set_0: [metallic=0, roughness=128, emissive=0, rim_intensity=0]
        let uniform_set_0 = pack_uniform_set_0(0, 128, 0, 0);
        // uniform_set_1: [spec_r=255, spec_g=255, spec_b=255, rim_power=0] (white specular)
        let uniform_set_1 = super::quantization::pack_uniform_set_1(255, 255, 255, 0);

        Self {
            color_rgba8: 0xFFFFFFFF, // White
            uniform_set_0,
            uniform_set_1,
            flags: DEFAULT_FLAGS, // uniform_alpha = 15 (opaque), other flags = 0
            lights: [PackedLight::default(); 4], // All lights disabled
            // Animation system fields (default to no animation)
            keyframe_base: 0,     // No keyframes bound
            inverse_bind_base: 0, // No skeleton bound (raw bone mode)
            _pad: 0,
            environment_index: 0, // Index 0 = default environment
        }
    }
}

/// Handle to interned shading state (newtype for clarity and type safety)
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ShadingStateIndex(pub u32);

impl ShadingStateIndex {
    pub const INVALID: Self = Self(u32::MAX);
}

impl crate::state::PoolIndex for ShadingStateIndex {
    fn from_raw(value: u32) -> Self {
        ShadingStateIndex(value)
    }

    fn as_raw(&self) -> u32 {
        self.0
    }
}

// ============================================================================
// PackedUnifiedShadingState Helpers
// ============================================================================

/// Flag bit for skinning mode in PackedUnifiedShadingState.flags
/// 0 = raw mode (matrices used as-is), 1 = inverse bind mode
pub const FLAG_SKINNING_MODE: u32 = 1 << 0;

/// Flag bit for texture filter mode in PackedUnifiedShadingState.flags
/// 0 = nearest (pixelated), 1 = linear (smooth)
pub const FLAG_TEXTURE_FILTER_LINEAR: u32 = 1 << 1;

// ============================================================================
// Animation system (Unified Buffer)
// ============================================================================
// NOTE: ANIMATION_FLAG_USE_IMMEDIATE removed - unified_animation buffer uses
// pre-computed offsets. The shader just reads from unified_animation[keyframe_base + bone_idx].

// ============================================================================
// Material Override Flags (bits 2-7)
// ============================================================================

/// Flag bit for uniform color override (bit 2)
pub const FLAG_USE_UNIFORM_COLOR: u32 = 1 << 2;
/// Flag bit for uniform metallic override (bit 3)
pub const FLAG_USE_UNIFORM_METALLIC: u32 = 1 << 3;
/// Flag bit for uniform roughness override (bit 4)
pub const FLAG_USE_UNIFORM_ROUGHNESS: u32 = 1 << 4;
/// Flag bit for uniform emissive override (bit 5)
pub const FLAG_USE_UNIFORM_EMISSIVE: u32 = 1 << 5;
/// Flag bit for uniform specular override (bit 6, Mode 3 only)
pub const FLAG_USE_UNIFORM_SPECULAR: u32 = 1 << 6;
/// Flag bit for matcap vs environment reflection (bit 7, Mode 1 only)
pub const FLAG_USE_MATCAP_REFLECTION: u32 = 1 << 7;

// ============================================================================
// Dither Transparency Flags (Bits 8-15)
// ============================================================================

/// Mask for uniform alpha level in flags (bits 8-11)
/// Values 0-15: 0 = fully transparent, 15 = fully opaque (default)
pub const FLAG_UNIFORM_ALPHA_MASK: u32 = 0xF << 8;
/// Bit shift for uniform alpha level
pub const FLAG_UNIFORM_ALPHA_SHIFT: u32 = 8;

/// Mask for dither offset X in flags (bits 12-13)
/// Values 0-3: pixel shift in X axis
pub const FLAG_DITHER_OFFSET_X_MASK: u32 = 0x3 << 12;
/// Bit shift for dither offset X
pub const FLAG_DITHER_OFFSET_X_SHIFT: u32 = 12;

/// Mask for dither offset Y in flags (bits 14-15)
/// Values 0-3: pixel shift in Y axis
pub const FLAG_DITHER_OFFSET_Y_MASK: u32 = 0x3 << 14;
/// Bit shift for dither offset Y
pub const FLAG_DITHER_OFFSET_Y_SHIFT: u32 = 14;

/// Default flags value with uniform_alpha = 15 (opaque)
pub const DEFAULT_FLAGS: u32 = 0xF << 8;

// ============================================================================
// Normal Mapping Flags (Bit 16)
// ============================================================================

/// Flag bit to disable normal map sampling in PackedUnifiedShadingState.flags
/// When NOT set (default) and mesh has tangent data: slot 3 is sampled as normal map
/// When SET: normal map sampling is skipped, vertex normal is used instead
/// This is an opt-out flag - normal mapping is enabled by default when tangent data exists
pub const FLAG_SKIP_NORMAL_MAP: u32 = 1 << 16;

impl PackedUnifiedShadingState {
    /// Create from all f32 parameters (used during FFI calls)
    /// For Mode 2: metallic, roughness, emissive packed into uniform_set_0
    /// rim_intensity defaults to 0, can be set via update methods
    pub fn from_floats(
        metallic: f32,
        roughness: f32,
        emissive: f32,
        color: u32,
        matcap_blend_modes: [MatcapBlendMode; 4],
        lights: [PackedLight; 4],
        environment_index: u32,
    ) -> Self {
        // Pack Mode 2 style: [metallic, roughness, emissive, rim_intensity=0]
        let uniform_set_0 = pack_uniform_set_0(
            pack_unorm8(metallic),
            pack_unorm8(roughness),
            pack_unorm8(emissive),
            0, // rim_intensity default
        );
        // uniform_set_1: for Mode 1 use matcap blend modes, Mode 3 use specular RGB
        let uniform_set_1 = pack_matcap_blend_modes(matcap_blend_modes);

        Self {
            uniform_set_0,
            color_rgba8: color,
            flags: DEFAULT_FLAGS, // uniform_alpha = 15 (opaque), other flags = 0
            uniform_set_1,
            lights,
            // Animation system fields - defaults
            keyframe_base: 0,
            inverse_bind_base: 0,
            _pad: 0,
            environment_index,
        }
    }

    /// Set skinning mode flag
    /// - false: raw mode (matrices used as-is)
    /// - true: inverse bind mode (GPU applies inverse bind matrices)
    #[inline]
    pub fn set_skinning_mode(&mut self, inverse_bind: bool) {
        if inverse_bind {
            self.flags |= FLAG_SKINNING_MODE;
        } else {
            self.flags &= !FLAG_SKINNING_MODE;
        }
    }

    /// Get skinning mode flag
    #[inline]
    pub fn skinning_mode(&self) -> bool {
        (self.flags & FLAG_SKINNING_MODE) != 0
    }

    /// Set skip normal map flag (opt-out)
    /// When set to true: normal map sampling is disabled, vertex normal is used
    /// When set to false (default): normal map is sampled from slot 3 (if tangent data exists)
    #[inline]
    pub fn set_skip_normal_map(&mut self, skip: bool) {
        if skip {
            self.flags |= FLAG_SKIP_NORMAL_MAP;
        } else {
            self.flags &= !FLAG_SKIP_NORMAL_MAP;
        }
    }

    /// Check if normal map sampling is skipped
    /// Returns true if normal map is disabled, false if enabled (default)
    #[inline]
    pub fn skips_normal_map(&self) -> bool {
        (self.flags & FLAG_SKIP_NORMAL_MAP) != 0
    }
}
