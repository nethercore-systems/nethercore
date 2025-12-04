use bytemuck::{Pod, Zeroable};
use glam::{Vec3, Vec4};

use super::render_state::{BlendMode, MatcapBlendMode};

/// Quantized sky data for GPU upload (16 bytes)
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Pod, Zeroable)]
pub struct PackedSky {
    pub horizon_color: u32,              // RGBA8 packed
    pub zenith_color: u32,               // RGBA8 packed
    pub sun_direction: [i16; 4],         // snorm16x4 (w unused, set to 0)
    pub sun_color_and_sharpness: u32,    // RGB8 + sharpness u8
}

/// One packed light (16 bytes)
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Pod, Zeroable)]
pub struct PackedLight {
    pub direction: [i16; 4],             // snorm16x4 (w = enabled flag: 0x7FFF if enabled, 0 if disabled)
    pub color_and_intensity: u32,        // RGB8 + intensity u8
}

/// Unified per-draw shading state (96 bytes, POD, hashable)
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Pod, Zeroable)]
pub struct PackedUnifiedShadingState {
    pub metallic: u8,
    pub roughness: u8,
    pub emissive: u8,
    pub pad0: u8,
    pub color_rgba8: u32,
    pub blend_mode: u32,                 // BlendMode as u32
    pub matcap_blend_modes: u32,         // 4x MatcapBlendMode packed as u8s
    pub pad1: u32,
    pub sky: PackedSky,                  // 16 bytes
    pub lights: [PackedLight; 4],        // 64 bytes
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

/// Pack an f32 normalized value [-1.0, 1.0] to i16 snorm16 [-32767, 32767]
#[inline]
pub fn pack_snorm16(value: f32) -> i16 {
    (value.clamp(-1.0, 1.0) * 32767.0).round() as i16
}

/// Pack RGBA f32 [0.0, 1.0] to u32 RGBA8
#[inline]
pub fn pack_rgba8(r: f32, g: f32, b: f32, a: f32) -> u32 {
    let r = pack_unorm8(r);
    let g = pack_unorm8(g);
    let b = pack_unorm8(b);
    let a = pack_unorm8(a);
    (r as u32) | ((g as u32) << 8) | ((b as u32) << 16) | ((a as u32) << 24)
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

/// Pack Vec3 direction [-1.0, 1.0] to [i16; 4] snorm16 (w = 0)
#[inline]
pub fn pack_direction3(dir: Vec3) -> [i16; 4] {
    [
        pack_snorm16(dir.x),
        pack_snorm16(dir.y),
        pack_snorm16(dir.z),
        0,
    ]
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
        let sun_dir_packed = pack_direction3(sun_direction.normalize_or_zero());

        let sun_r = pack_unorm8(sun_color.x);
        let sun_g = pack_unorm8(sun_color.y);
        let sun_b = pack_unorm8(sun_color.z);
        let sun_sharp = pack_unorm8(sun_sharpness);
        let sun_color_and_sharpness = (sun_r as u32)
            | ((sun_g as u32) << 8)
            | ((sun_b as u32) << 16)
            | ((sun_sharp as u32) << 24);

        Self {
            horizon_color: horizon_rgba,
            zenith_color: zenith_rgba,
            sun_direction: sun_dir_packed,
            sun_color_and_sharpness,
        }
    }
}

// ============================================================================
// PackedLight Helpers
// ============================================================================

impl PackedLight {
    /// Create a PackedLight from f32 parameters
    pub fn from_floats(direction: Vec3, color: Vec3, intensity: f32, enabled: bool) -> Self {
        let mut dir_packed = pack_direction3(direction.normalize_or_zero());
        // Use w component as enabled flag: 0x7FFF if enabled, 0 if disabled
        dir_packed[3] = if enabled { 0x7FFF } else { 0 };

        let r = pack_unorm8(color.x);
        let g = pack_unorm8(color.y);
        let b = pack_unorm8(color.z);
        let intens = pack_unorm8(intensity);
        let color_and_intensity = (r as u32)
            | ((g as u32) << 8)
            | ((b as u32) << 16)
            | ((intens as u32) << 24);

        Self {
            direction: dir_packed,
            color_and_intensity,
        }
    }

    /// Create a disabled light (all zeros)
    pub fn disabled() -> Self {
        Self::default()
    }
}

// ============================================================================
// PackedUnifiedShadingState Helpers
// ============================================================================

impl PackedUnifiedShadingState {
    /// Create from all f32 parameters (used during FFI calls)
    #[allow(clippy::too_many_arguments)]
    pub fn from_floats(
        metallic: f32,
        roughness: f32,
        emissive: f32,
        color: u32,
        blend_mode: BlendMode,
        matcap_blend_modes: [MatcapBlendMode; 4],
        sky: PackedSky,
        lights: [PackedLight; 4],
    ) -> Self {
        Self {
            metallic: pack_unorm8(metallic),
            roughness: pack_unorm8(roughness),
            emissive: pack_unorm8(emissive),
            pad0: 0,
            color_rgba8: color,
            blend_mode: blend_mode as u32,
            matcap_blend_modes: pack_matcap_blend_modes(matcap_blend_modes),
            pad1: 0,
            sky,
            lights,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_packed_sizes() {
        assert_eq!(std::mem::size_of::<PackedSky>(), 16);
        assert_eq!(std::mem::size_of::<PackedLight>(), 16);
        assert_eq!(std::mem::size_of::<PackedUnifiedShadingState>(), 96);
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
    fn test_pack_rgba8() {
        let packed = pack_rgba8(1.0, 0.5, 0.25, 1.0);
        assert_eq!(packed & 0xFF, 255);           // R
        assert_eq!((packed >> 8) & 0xFF, 128);    // G
        assert_eq!((packed >> 16) & 0xFF, 64);    // B
        assert_eq!((packed >> 24) & 0xFF, 255);   // A
    }

    #[test]
    fn test_default_sky_is_black() {
        let sky = PackedSky::default();
        assert_eq!(sky.horizon_color, 0);
        assert_eq!(sky.zenith_color, 0);
        assert_eq!(sky.sun_direction, [0, 0, 0, 0]);
        assert_eq!(sky.sun_color_and_sharpness, 0);
    }

    #[test]
    fn test_disabled_light() {
        let light = PackedLight::disabled();
        assert_eq!(light.direction, [0, 0, 0, 0]);
        assert_eq!(light.color_and_intensity, 0);
    }
}
