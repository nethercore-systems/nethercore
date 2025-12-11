use bytemuck::{Pod, Zeroable};
use glam::{Vec3, Vec4};
use half::f16;
use z_common::{encode_octahedral, pack_octahedral_u32, unpack_octahedral_u32};

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

/// Unified per-draw shading state (80 bytes, POD, hashable)
/// Size breakdown: 16 bytes (header) + 16 bytes (sky) + 48 bytes (4 × 12-byte lights)
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
            flags: 0, // skinning_mode = 0 (raw mode)
            sky,
            lights: [PackedLight::default(); 4], // All lights disabled
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
pub fn pack_snorm16(value: f32) -> i16 {
    (value.clamp(-1.0, 1.0) * 32767.0).round() as i16
}

/// Unpack snorm16 [-32767, 32767] to f32 [-1.0, 1.0]
#[inline]
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

/// Pack Vec4 color [0.0, 1.0] to u32 RGBA8
#[inline]
pub fn pack_rgba8_vec4(color: Vec4) -> u32 {
    pack_rgba8(color.x, color.y, color.z, color.w)
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

/// Update a specific byte in uniform_set_0
#[inline]
pub fn update_uniform_set_0_byte(current: u32, byte_index: u8, value: u8) -> u32 {
    let shift = byte_index as u32 * 8;
    let mask = !(0xFFu32 << shift);
    (current & mask) | ((value as u32) << shift)
}

/// Update a specific byte in uniform_set_1
#[inline]
pub fn update_uniform_set_1_byte(current: u32, byte_index: u8, value: u8) -> u32 {
    let shift = byte_index as u32 * 8;
    let mask = !(0xFFu32 << shift);
    (current & mask) | ((value as u32) << shift)
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
            flags: 0, // skinning_mode = 0 (raw mode)
            uniform_set_1,
            sky,
            lights,
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

    #[test]
    fn test_packed_sizes() {
        assert_eq!(std::mem::size_of::<PackedSky>(), 16);
        assert_eq!(std::mem::size_of::<PackedLight>(), 12); // 12 bytes for point light support
        assert_eq!(std::mem::size_of::<PackedUnifiedShadingState>(), 80); // 16 + 16 + 48
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
}
