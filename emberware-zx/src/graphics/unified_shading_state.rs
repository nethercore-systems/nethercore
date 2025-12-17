use bytemuck::{Pod, Zeroable};
use glam::Vec3;
use half::f16;
use zx_common::{pack_octahedral_u32, unpack_octahedral_u32};

use super::render_state::MatcapBlendMode;

/// Quantized sky data for GPU upload (16 bytes)
/// Sun direction uses octahedral encoding for uniform precision
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Pod, Zeroable)]
pub struct PackedSky {
    pub horizon_color: u32,           // RGBA8 packed (4 bytes)
    pub zenith_color: u32,            // RGBA8 packed (4 bytes)
    pub sun_direction_oct: u32,       // Octahedral encoding (2x snorm16) - 4 bytes
    pub sun_color_and_sharpness: u32, // RGB8 + sharpness u8 (4 bytes)
}

/// Light type stored in bit 7 of data1
#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum LightType {
    #[default]
    Directional = 0,
    Point = 1,
}

impl LightType {
    pub fn from_bit(bit: bool) -> Self {
        if bit {
            LightType::Point
        } else {
            LightType::Directional
        }
    }

    pub fn to_bit(self) -> bool {
        matches!(self, LightType::Point)
    }
}

/// One packed light (12 bytes) - supports directional and point lights
///
/// # Format
///
/// **data0:**
/// - Directional: octahedral direction (snorm16x2)
/// - Point: position XY (f16x2)
///
/// **data1:** RGB8 (bits 31-8) + type (bit 7) + intensity (bits 6-0)
/// - Format: 0xRRGGBB_TI where T=type(1bit), I=intensity(7bits)
/// - Intensity maps 0-127 -> 0.0-8.0 for HDR support
///
/// **data2:**
/// - Directional: unused (0)
/// - Point: position Z (f16, bits 15-0) + range (f16, bits 31-16)
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Pod, Zeroable)]
pub struct PackedLight {
    /// Directional: octahedral direction (snorm16x2)
    /// Point: position XY (f16x2)
    pub data0: u32,

    /// RGB8 (bits 31-8) + type (bit 7) + intensity (bits 6-0)
    /// Format: 0xRRGGBB_TI where T=type(1bit), I=intensity(7bits)
    pub data1: u32,

    /// Directional: unused (0)
    /// Point: position Z (f16, bits 15-0) + range (f16, bits 31-16)
    pub data2: u32,
}

/// Unified per-draw shading state (96 bytes, POD, hashable)
/// Size breakdown: 16 bytes (header) + 16 bytes (sky) + 48 bytes (lights) + 16 bytes (animation)
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

    pub sky: PackedSky,           // 16 bytes
    pub lights: [PackedLight; 4], // 48 bytes (4 × 12-byte lights)

    // Animation System v2 fields (16 bytes total)
    /// Base offset into @binding(7) all_keyframes buffer
    /// Shader reads: all_keyframes[keyframe_base + bone_index]
    /// 0 = no keyframes bound (use bones buffer directly)
    pub keyframe_base: u32,

    /// Base offset into @binding(6) all_inverse_bind buffer
    /// Shader reads: inverse_bind[inverse_bind_base + bone_index]
    /// 0 = no skeleton bound (raw bone mode)
    pub inverse_bind_base: u32,

    /// Animation-specific flags (separate from main flags field)
    /// - Bit 0: use_static_keyframes (0 = immediate bones, 1 = static keyframes)
    /// - Bits 1-31: reserved for v2.1
    pub animation_flags: u32,

    /// Reserved for future animation features (zeroed)
    pub _animation_reserved: u32,
}

impl Default for PackedUnifiedShadingState {
    fn default() -> Self {
        // Natural Earth sky defaults - sun is the key light, sky provides subtle fill
        // Convention: light direction = direction rays travel (physically correct)
        // Users can customize via sky_set_gradient() and sky_set_sun() FFI calls
        let sky = PackedSky::from_floats(
            Vec3::new(0.25, 0.25, 0.3), // Horizon: dim warm gray (subtle fill, not key light)
            Vec3::new(0.15, 0.2, 0.35), // Zenith: darker blue (sky color, not light source)
            Vec3::new(-0.4, -0.7, -0.3).normalize(), // Sun direction: high in sky, slightly left
            Vec3::new(0.7, 0.67, 0.6),  // Sun color: softer daylight (reduced from 1.0)
            0.92,                       // Sun sharpness: visible disc
        );

        // uniform_set_0: [metallic=0, roughness=128, emissive=0, rim_intensity=0]
        let uniform_set_0 = pack_uniform_set_0(0, 128, 0, 0);
        // uniform_set_1: [spec_r=255, spec_g=255, spec_b=255, rim_power=0] (white specular)
        let uniform_set_1 = pack_uniform_set_1(255, 255, 255, 0);

        Self {
            color_rgba8: 0xFFFFFFFF, // White
            uniform_set_0,
            uniform_set_1,
            flags: DEFAULT_FLAGS, // uniform_alpha = 15 (opaque), other flags = 0
            sky,
            lights: [PackedLight::default(); 4], // All lights disabled
            // Animation System v2 fields (default to no animation)
            keyframe_base: 0,       // No keyframes bound
            inverse_bind_base: 0,   // No skeleton bound (raw bone mode)
            animation_flags: 0,     // Use immediate bones by default
            _animation_reserved: 0, // Zeroed
        }
    }
}

/// Handle to interned shading state (newtype for clarity and type safety)
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ShadingStateIndex(pub u32);

impl ShadingStateIndex {
    pub const INVALID: Self = Self(u32::MAX);
}

// ============================================================================
// Quantization Helper Functions
// ============================================================================

/// Pack an f32 color channel [0.0, 1.0] to u8 [0, 255]
#[inline]
pub fn pack_unorm8(value: f32) -> u8 {
    (value.clamp(0.0, 1.0) * 255.0).round() as u8
}

/// Unpack u8 [0, 255] to f32 [0.0, 1.0]
#[inline]
pub fn unpack_unorm8(value: u8) -> f32 {
    value as f32 / 255.0
}

/// Pack an f32 normalized value [-1.0, 1.0] to i16 snorm16 [-32767, 32767]
#[inline]
#[allow(dead_code)]
pub fn pack_snorm16(value: f32) -> i16 {
    (value.clamp(-1.0, 1.0) * 32767.0).round() as i16
}

/// Unpack snorm16 [-32767, 32767] to f32 [-1.0, 1.0]
#[inline]
#[allow(dead_code)]
pub fn unpack_snorm16(value: i16) -> f32 {
    value as f32 / 32767.0
}

/// Pack f32 to IEEE 754 half-precision float (f16) stored as u16
#[inline]
pub fn pack_f16(value: f32) -> u16 {
    f16::from_f32(value).to_bits()
}

/// Unpack IEEE 754 half-precision float (f16) from u16 to f32
#[inline]
pub fn unpack_f16(bits: u16) -> f32 {
    f16::from_bits(bits).to_f32()
}

/// Pack two f32 values into a u32 as f16x2
#[inline]
pub fn pack_f16x2(x: f32, y: f32) -> u32 {
    let x_bits = pack_f16(x) as u32;
    let y_bits = pack_f16(y) as u32;
    x_bits | (y_bits << 16)
}

/// Unpack u32 to two f32 values from f16x2
#[inline]
pub fn unpack_f16x2(packed: u32) -> (f32, f32) {
    let x = unpack_f16((packed & 0xFFFF) as u16);
    let y = unpack_f16((packed >> 16) as u16);
    (x, y)
}

/// Pack RGBA f32 [0.0, 1.0] to u32 RGBA8
/// Format: 0xRRGGBBAA (R in highest byte, A in lowest)
#[inline]
pub fn pack_rgba8(r: f32, g: f32, b: f32, a: f32) -> u32 {
    let r = pack_unorm8(r);
    let g = pack_unorm8(g);
    let b = pack_unorm8(b);
    let a = pack_unorm8(a);
    ((r as u32) << 24) | ((g as u32) << 16) | ((b as u32) << 8) | (a as u32)
}

/// Pack Vec3 color [0.0, 1.0] to u32 RGB8 (alpha = 255)
#[inline]
pub fn pack_rgb8(color: Vec3) -> u32 {
    pack_rgba8(color.x, color.y, color.z, 1.0)
}

/// Pack 4x MatcapBlendMode into u32 (4 bytes)
#[inline]
pub fn pack_matcap_blend_modes(modes: [MatcapBlendMode; 4]) -> u32 {
    (modes[0] as u32)
        | ((modes[1] as u32) << 8)
        | ((modes[2] as u32) << 16)
        | ((modes[3] as u32) << 24)
}

// ============================================================================
// Uniform Set Packing Helpers
// ============================================================================

/// Pack 4 u8 values into uniform_set_0
/// Layout: [byte0, byte1, byte2, byte3] where byte0 is in low bits
/// - Mode 0: [unused, unused, unused, rim_intensity]
/// - Mode 1: [blend_mode_0, blend_mode_1, blend_mode_2, blend_mode_3]
/// - Mode 2: [metallic, roughness, emissive, rim_intensity]
/// - Mode 3: [spec_intensity, shininess, emissive, rim_intensity]
#[inline]
pub fn pack_uniform_set_0(byte0: u8, byte1: u8, byte2: u8, byte3: u8) -> u32 {
    (byte0 as u32) | ((byte1 as u32) << 8) | ((byte2 as u32) << 16) | ((byte3 as u32) << 24)
}

/// Pack 4 u8 values into uniform_set_1
/// Layout: [byte0, byte1, byte2, byte3] where byte0 is in low bits
/// - Mode 0: [unused, unused, unused, rim_power]
/// - Mode 1: unused
/// - Mode 2: [unused, unused, unused, rim_power]
/// - Mode 3: [spec_r, spec_g, spec_b, rim_power]
#[inline]
pub fn pack_uniform_set_1(byte0: u8, byte1: u8, byte2: u8, byte3: u8) -> u32 {
    (byte0 as u32) | ((byte1 as u32) << 8) | ((byte2 as u32) << 16) | ((byte3 as u32) << 24)
}

/// Update a specific byte in a packed u32 value
///
/// # Arguments
/// * `current` - The current packed u32 value
/// * `byte_index` - Which byte to update (0-3, where 0 is lowest byte)
/// * `value` - The new byte value
#[inline]
pub fn update_u32_byte(current: u32, byte_index: u8, value: u8) -> u32 {
    let shift = byte_index as u32 * 8;
    let mask = !(0xFFu32 << shift);
    (current & mask) | ((value as u32) << shift)
}

// Backwards compatibility aliases
#[inline]
pub fn update_uniform_set_0_byte(current: u32, byte_index: u8, value: u8) -> u32 {
    update_u32_byte(current, byte_index, value)
}

#[inline]
pub fn update_uniform_set_1_byte(current: u32, byte_index: u8, value: u8) -> u32 {
    update_u32_byte(current, byte_index, value)
}

/// Unpack matcap blend modes from u32
pub fn unpack_matcap_blend_modes(packed: u32) -> [MatcapBlendMode; 4] {
    [
        MatcapBlendMode::from_u8((packed & 0xFF) as u8),
        MatcapBlendMode::from_u8(((packed >> 8) & 0xFF) as u8),
        MatcapBlendMode::from_u8(((packed >> 16) & 0xFF) as u8),
        MatcapBlendMode::from_u8(((packed >> 24) & 0xFF) as u8),
    ]
}

// ============================================================================
// PackedSky Helpers
// ============================================================================

impl PackedSky {
    /// Create a PackedSky from f32 parameters
    pub fn from_floats(
        horizon_color: Vec3,
        zenith_color: Vec3,
        sun_direction: Vec3,
        sun_color: Vec3,
        sun_sharpness: f32,
    ) -> Self {
        let horizon_rgba = pack_rgb8(horizon_color);
        let zenith_rgba = pack_rgb8(zenith_color);
        let sun_dir_oct = pack_octahedral_u32(sun_direction);

        let sun_r = pack_unorm8(sun_color.x);
        let sun_g = pack_unorm8(sun_color.y);
        let sun_b = pack_unorm8(sun_color.z);
        let sun_sharp = pack_unorm8(sun_sharpness);
        // Format: 0xRRGGBBSS (R in highest byte, sharpness in lowest)
        let sun_color_and_sharpness = ((sun_r as u32) << 24)
            | ((sun_g as u32) << 16)
            | ((sun_b as u32) << 8)
            | (sun_sharp as u32);

        Self {
            horizon_color: horizon_rgba,
            zenith_color: zenith_rgba,
            sun_direction_oct: sun_dir_oct,
            sun_color_and_sharpness,
        }
    }
}

// ============================================================================
// PackedLight Helpers
// ============================================================================

impl PackedLight {
    /// Create a directional light
    pub fn directional(direction: Vec3, color: Vec3, intensity: f32, enabled: bool) -> Self {
        let data0 = pack_octahedral_u32(direction.normalize_or_zero());
        let data1 =
            Self::pack_color_type_intensity(color, LightType::Directional, intensity, enabled);
        Self {
            data0,
            data1,
            data2: 0,
        }
    }

    /// Create a point light
    pub fn point(position: Vec3, color: Vec3, intensity: f32, range: f32, enabled: bool) -> Self {
        let data0 = pack_f16x2(position.x, position.y);
        let data1 = Self::pack_color_type_intensity(color, LightType::Point, intensity, enabled);
        let data2 = pack_f16x2(position.z, range);
        Self {
            data0,
            data1,
            data2,
        }
    }

    /// Pack color, type, and intensity into data1
    /// Format: 0xRRGGBB_TI where T=type(1bit), I=intensity(7bits)
    fn pack_color_type_intensity(
        color: Vec3,
        light_type: LightType,
        intensity: f32,
        enabled: bool,
    ) -> u32 {
        let r = pack_unorm8(color.x);
        let g = pack_unorm8(color.y);
        let b = pack_unorm8(color.z);

        // Intensity: 0.0-8.0 -> 0-127 (7 bits)
        // If disabled, set to 0
        let intensity_7bit = if enabled {
            ((intensity / 8.0).clamp(0.0, 1.0) * 127.0).round() as u8
        } else {
            0
        };

        // Type in bit 7, intensity in bits 0-6
        let type_intensity = ((light_type as u8) << 7) | (intensity_7bit & 0x7F);

        ((r as u32) << 24) | ((g as u32) << 16) | ((b as u32) << 8) | (type_intensity as u32)
    }

    /// Create a PackedLight from f32 parameters (directional light)
    /// Backward compatibility: delegates to directional()
    /// If enabled=false, intensity is set to 0 (which indicates disabled light)
    pub fn from_floats(direction: Vec3, color: Vec3, intensity: f32, enabled: bool) -> Self {
        Self::directional(direction, color, intensity, enabled)
    }

    /// Create a disabled light (all zeros)
    pub fn disabled() -> Self {
        Self::default()
    }

    /// Get light type (directional or point)
    pub fn get_type(&self) -> LightType {
        LightType::from_bit((self.data1 & 0x80) != 0)
    }

    /// Extract direction as f32 array (only valid for directional lights)
    /// Decodes the octahedral-encoded direction stored in data0.
    pub fn get_direction(&self) -> [f32; 3] {
        let dir = unpack_octahedral_u32(self.data0);
        [dir.x, dir.y, dir.z]
    }

    /// Get position (only valid for point lights)
    pub fn get_position(&self) -> [f32; 3] {
        let (x, y) = unpack_f16x2(self.data0);
        let (z, _) = unpack_f16x2(self.data2);
        [x, y, z]
    }

    /// Get range (only valid for point lights)
    pub fn get_range(&self) -> f32 {
        let (_, range) = unpack_f16x2(self.data2);
        range
    }

    /// Extract color as f32 array
    /// Format: 0xRRGGBB_TI (R in highest byte, type+intensity in lowest byte)
    pub fn get_color(&self) -> [f32; 3] {
        let r = unpack_unorm8(((self.data1 >> 24) & 0xFF) as u8);
        let g = unpack_unorm8(((self.data1 >> 16) & 0xFF) as u8);
        let b = unpack_unorm8(((self.data1 >> 8) & 0xFF) as u8);
        [r, g, b]
    }

    /// Extract intensity as f32 (0.0-8.0 range)
    /// Intensity is stored in bits 0-6 of data1
    pub fn get_intensity(&self) -> f32 {
        let intensity_7bit = (self.data1 & 0x7F) as f32;
        intensity_7bit / 127.0 * 8.0
    }

    /// Check if light is enabled (intensity > 0)
    pub fn is_enabled(&self) -> bool {
        (self.data1 & 0x7F) != 0
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
// Animation System v2 (Unified Buffer)
// ============================================================================
// NOTE: ANIMATION_FLAG_USE_IMMEDIATE removed - unified_animation buffer uses
// pre-computed offsets. The shader just reads from unified_animation[keyframe_base + bone_idx].
// The animation_flags field in PackedUnifiedShadingState is now unused but kept for
// struct layout compatibility.

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
/// Flag bit for matcap vs sky reflection (bit 7, Mode 1 only)
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
        sky: PackedSky,
        lights: [PackedLight; 4],
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
            sky,
            lights,
            // Animation System v2 fields - defaults
            keyframe_base: 0,
            inverse_bind_base: 0,
            animation_flags: 0,
            _animation_reserved: 0,
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use z_common::encode_octahedral;

    #[test]
    fn test_packed_sizes() {
        assert_eq!(std::mem::size_of::<PackedSky>(), 16);
        assert_eq!(std::mem::size_of::<PackedLight>(), 12); // 12 bytes for point light support
        assert_eq!(std::mem::size_of::<PackedUnifiedShadingState>(), 96); // 16 + 16 + 48 + 16 (animation)
    }

    #[test]
    fn test_quantization() {
        assert_eq!(pack_unorm8(0.0), 0);
        assert_eq!(pack_unorm8(1.0), 255);
        assert_eq!(pack_unorm8(0.5), 128);

        assert_eq!(pack_snorm16(0.0), 0);
        assert_eq!(pack_snorm16(1.0), 32767);
        assert_eq!(pack_snorm16(-1.0), -32767);
    }

    #[test]
    fn test_octahedral_cardinals() {
        // Test that cardinal directions encode/decode correctly
        let tests = [
            Vec3::new(1.0, 0.0, 0.0),  // +X
            Vec3::new(-1.0, 0.0, 0.0), // -X
            Vec3::new(0.0, 1.0, 0.0),  // +Y
            Vec3::new(0.0, -1.0, 0.0), // -Y
            Vec3::new(0.0, 0.0, 1.0),  // +Z
            Vec3::new(0.0, 0.0, -1.0), // -Z
        ];

        for dir in &tests {
            let (u, v) = encode_octahedral(*dir);
            assert!(u >= -1.0 && u <= 1.0, "u out of range for {:?}", dir);
            assert!(v >= -1.0 && v <= 1.0, "v out of range for {:?}", dir);

            // Verify packing doesn't panic and produces valid output
            let packed = pack_octahedral_u32(*dir);
            assert_ne!(packed, 0xFFFFFFFF, "invalid pack for {:?}", dir);
        }
    }

    #[test]
    fn test_octahedral_zero_vector() {
        let zero = Vec3::new(0.0, 0.0, 0.0);
        let (u, v) = encode_octahedral(zero);
        assert_eq!(u, 0.0);
        assert_eq!(v, 0.0);
    }

    #[test]
    fn test_octahedral_diagonal() {
        // Test diagonal directions (challenging for octahedral)
        let diag = Vec3::new(0.577, 0.577, 0.577).normalize();
        let (u, v) = encode_octahedral(diag);
        assert!(u >= -1.0 && u <= 1.0);
        assert!(v >= -1.0 && v <= 1.0);
    }

    #[test]
    fn test_pack_rgba8() {
        // Format: 0xRRGGBBAA (R in highest byte, A in lowest)
        let packed = pack_rgba8(1.0, 0.5, 0.25, 1.0);
        assert_eq!((packed >> 24) & 0xFF, 255); // R
        assert_eq!((packed >> 16) & 0xFF, 128); // G
        assert_eq!((packed >> 8) & 0xFF, 64); // B
        assert_eq!(packed & 0xFF, 255); // A
    }

    #[test]
    fn test_default_sky_is_black() {
        let sky = PackedSky::default();
        assert_eq!(sky.horizon_color, 0);
        assert_eq!(sky.zenith_color, 0);
        assert_eq!(sky.sun_direction_oct, 0);
        assert_eq!(sky.sun_color_and_sharpness, 0);
    }

    #[test]
    fn test_disabled_light() {
        let light = PackedLight::disabled();
        assert_eq!(light.data0, 0);
        assert_eq!(light.data1, 0);
        assert_eq!(light.data2, 0);
        assert!(!light.is_enabled());
    }

    #[test]
    fn test_directional_light_roundtrip() {
        let dir = Vec3::new(0.5, -0.7, 0.3).normalize();
        let color = Vec3::new(1.0, 0.5, 0.25);
        let intensity = 2.5;

        let light = PackedLight::directional(dir, color, intensity, true);

        assert_eq!(light.get_type(), LightType::Directional);
        assert!(light.is_enabled());

        let unpacked_dir = light.get_direction();
        assert!((unpacked_dir[0] - dir.x).abs() < 0.01);
        assert!((unpacked_dir[1] - dir.y).abs() < 0.01);
        assert!((unpacked_dir[2] - dir.z).abs() < 0.01);

        let unpacked_color = light.get_color();
        assert!((unpacked_color[0] - color.x).abs() < 0.01);
        assert!((unpacked_color[1] - color.y).abs() < 0.01);
        assert!((unpacked_color[2] - color.z).abs() < 0.01);

        // Intensity with 7-bit precision in 0-8 range
        let unpacked_intensity = light.get_intensity();
        assert!((unpacked_intensity - intensity).abs() < 0.1);
    }

    #[test]
    fn test_point_light_roundtrip() {
        let pos = Vec3::new(10.5, -5.25, 100.0);
        let color = Vec3::new(0.8, 0.6, 0.4);
        let intensity = 4.0;
        let range = 25.0;

        let light = PackedLight::point(pos, color, intensity, range, true);

        assert_eq!(light.get_type(), LightType::Point);
        assert!(light.is_enabled());

        let unpacked_pos = light.get_position();
        // f16 precision is about 3 decimal digits
        assert!((unpacked_pos[0] - pos.x).abs() < 0.1);
        assert!((unpacked_pos[1] - pos.y).abs() < 0.1);
        assert!((unpacked_pos[2] - pos.z).abs() < 1.0);

        let unpacked_range = light.get_range();
        assert!((unpacked_range - range).abs() < 0.5);
    }

    #[test]
    fn test_f16_packing() {
        let values = [0.0, 1.0, -1.0, 100.0, 0.001, 65504.0];
        for v in values {
            let packed = pack_f16(v);
            let unpacked = unpack_f16(packed);
            let error = (unpacked - v).abs() / v.abs().max(1.0);
            assert!(
                error < 0.01,
                "f16 roundtrip failed for {}: got {}",
                v,
                unpacked
            );
        }
    }

    #[test]
    fn test_f16x2_packing() {
        let (x, y) = (42.5, -17.25);
        let packed = pack_f16x2(x, y);
        let (ux, uy) = unpack_f16x2(packed);
        assert!((ux - x).abs() < 0.1);
        assert!((uy - y).abs() < 0.1);
    }

    #[test]
    fn test_intensity_range() {
        // Test intensity at various points in 0-8 range
        for intensity in [0.0, 1.0, 2.0, 4.0, 7.9] {
            let light =
                PackedLight::directional(Vec3::new(0.0, -1.0, 0.0), Vec3::ONE, intensity, true);
            let unpacked = light.get_intensity();
            assert!(
                (unpacked - intensity).abs() < 0.1,
                "intensity {} unpacked to {}",
                intensity,
                unpacked
            );
        }
    }

    #[test]
    fn test_disabled_directional_light() {
        let light = PackedLight::directional(
            Vec3::new(0.0, -1.0, 0.0),
            Vec3::ONE,
            1.0,
            false, // disabled
        );
        assert!(!light.is_enabled());
        assert_eq!(light.get_intensity(), 0.0);
    }

    #[test]
    fn test_texture_filter_flag() {
        let mut state = PackedUnifiedShadingState::default();
        // Default: nearest (flag not set)
        assert_eq!(state.flags & FLAG_TEXTURE_FILTER_LINEAR, 0);

        // Set to linear
        state.flags |= FLAG_TEXTURE_FILTER_LINEAR;
        assert_ne!(state.flags & FLAG_TEXTURE_FILTER_LINEAR, 0);

        // Set back to nearest
        state.flags &= !FLAG_TEXTURE_FILTER_LINEAR;
        assert_eq!(state.flags & FLAG_TEXTURE_FILTER_LINEAR, 0);
    }

    #[test]
    fn test_flags_independence() {
        // Verify texture_filter and skinning_mode flags don't interfere with each other
        let mut state = PackedUnifiedShadingState::default();

        // Set both flags
        state.flags = FLAG_SKINNING_MODE | FLAG_TEXTURE_FILTER_LINEAR;
        assert!(state.skinning_mode());
        assert_ne!(state.flags & FLAG_TEXTURE_FILTER_LINEAR, 0);

        // Clear skinning_mode, texture_filter should remain
        state.flags &= !FLAG_SKINNING_MODE;
        assert!(!state.skinning_mode());
        assert_ne!(state.flags & FLAG_TEXTURE_FILTER_LINEAR, 0);

        // Clear texture_filter, both should be clear
        state.flags &= !FLAG_TEXTURE_FILTER_LINEAR;
        assert!(!state.skinning_mode());
        assert_eq!(state.flags & FLAG_TEXTURE_FILTER_LINEAR, 0);
    }

    #[test]
    fn test_texture_filter_flag_bit_position() {
        // Verify the flag is at bit 1 (value 2)
        assert_eq!(FLAG_TEXTURE_FILTER_LINEAR, 2);
        assert_eq!(FLAG_TEXTURE_FILTER_LINEAR, 1 << 1);

        // Verify it's different from skinning_mode (bit 0)
        assert_ne!(FLAG_TEXTURE_FILTER_LINEAR, FLAG_SKINNING_MODE);
    }

    // ========================================================================
    // Dither Transparency Tests
    // ========================================================================

    #[test]
    fn test_uniform_alpha_packing() {
        // Test all 16 values pack/unpack correctly
        for alpha in 0..=15u32 {
            let flags = alpha << FLAG_UNIFORM_ALPHA_SHIFT;
            let unpacked = (flags & FLAG_UNIFORM_ALPHA_MASK) >> FLAG_UNIFORM_ALPHA_SHIFT;
            assert_eq!(unpacked, alpha);
        }
    }

    #[test]
    fn test_dither_offset_packing() {
        // Test all 16 offset combinations
        for x in 0..=3u32 {
            for y in 0..=3u32 {
                let flags = (x << FLAG_DITHER_OFFSET_X_SHIFT) | (y << FLAG_DITHER_OFFSET_Y_SHIFT);
                let unpacked_x = (flags & FLAG_DITHER_OFFSET_X_MASK) >> FLAG_DITHER_OFFSET_X_SHIFT;
                let unpacked_y = (flags & FLAG_DITHER_OFFSET_Y_MASK) >> FLAG_DITHER_OFFSET_Y_SHIFT;
                assert_eq!(unpacked_x, x);
                assert_eq!(unpacked_y, y);
            }
        }
    }

    #[test]
    fn test_default_flags_are_opaque() {
        let state = PackedUnifiedShadingState::default();
        let alpha = (state.flags & FLAG_UNIFORM_ALPHA_MASK) >> FLAG_UNIFORM_ALPHA_SHIFT;
        assert_eq!(alpha, 15, "Default uniform_alpha must be 15 (opaque)");
    }

    #[test]
    fn test_bayer_threshold_values() {
        // Verify Bayer matrix produces values in expected range
        const BAYER_4X4: [f32; 16] = [
            0.0 / 16.0,
            8.0 / 16.0,
            2.0 / 16.0,
            10.0 / 16.0,
            12.0 / 16.0,
            4.0 / 16.0,
            14.0 / 16.0,
            6.0 / 16.0,
            3.0 / 16.0,
            11.0 / 16.0,
            1.0 / 16.0,
            9.0 / 16.0,
            15.0 / 16.0,
            7.0 / 16.0,
            13.0 / 16.0,
            5.0 / 16.0,
        ];

        for (i, &threshold) in BAYER_4X4.iter().enumerate() {
            assert!(threshold >= 0.0, "Threshold {} is negative", i);
            assert!(threshold < 1.0, "Threshold {} >= 1.0", i);
        }

        // Verify we have 16 unique values
        let mut sorted = BAYER_4X4.to_vec();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
        for i in 0..15 {
            assert_ne!(sorted[i], sorted[i + 1], "Duplicate threshold values");
        }
    }

    #[test]
    fn test_dither_flags_independence() {
        // Verify dither flags don't interfere with other flags
        let mut state = PackedUnifiedShadingState::default();

        // Set skinning_mode and texture_filter
        state.flags |= FLAG_SKINNING_MODE | FLAG_TEXTURE_FILTER_LINEAR;

        // Set uniform_alpha to 8 (50% transparency)
        state.flags = (state.flags & !FLAG_UNIFORM_ALPHA_MASK) | (8u32 << FLAG_UNIFORM_ALPHA_SHIFT);

        // Set dither offset to (2, 3)
        state.flags = (state.flags & !FLAG_DITHER_OFFSET_X_MASK & !FLAG_DITHER_OFFSET_Y_MASK)
            | (2u32 << FLAG_DITHER_OFFSET_X_SHIFT)
            | (3u32 << FLAG_DITHER_OFFSET_Y_SHIFT);

        // Verify all flags are independent
        assert!(state.skinning_mode());
        assert_ne!(state.flags & FLAG_TEXTURE_FILTER_LINEAR, 0);
        assert_eq!(
            (state.flags & FLAG_UNIFORM_ALPHA_MASK) >> FLAG_UNIFORM_ALPHA_SHIFT,
            8
        );
        assert_eq!(
            (state.flags & FLAG_DITHER_OFFSET_X_MASK) >> FLAG_DITHER_OFFSET_X_SHIFT,
            2
        );
        assert_eq!(
            (state.flags & FLAG_DITHER_OFFSET_Y_MASK) >> FLAG_DITHER_OFFSET_Y_SHIFT,
            3
        );
    }
}
