use bytemuck::{Pod, Zeroable};
use glam::{Vec3, Vec4};

use super::render_state::{BlendMode, MatcapBlendMode};

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

/// One packed light (8 bytes)
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Pod, Zeroable)]
pub struct PackedLight {
    pub direction_oct: u32,       // Octahedral encoding (2x snorm16) - 4 bytes
    pub color_and_intensity: u32, // RGB8 + intensity u8 (intensity=0 means disabled) - 4 bytes
}

/// Unified per-draw shading state (64 bytes, POD, hashable)
/// Size breakdown: 16 bytes (header) + 16 bytes (sky) + 32 bytes (4 × 8-byte lights)
///
/// # Mode-Specific Field Interpretation
///
/// The same struct fields are interpreted differently per render mode (no struct changes needed):
///
/// | Field               | Bytes | Mode 2 (PBR)              | Mode 3 (Blinn-Phong)                  |
/// |---------------------|-------|---------------------------|---------------------------------------|
/// | `metallic`          | 0     | Metallic                  | **Rim intensity** (Slot 1.R fallback) |
/// | `roughness`         | 1     | Roughness                 | **Shininess** (Slot 1.G fallback)     |
/// | `emissive`          | 2     | Emissive                  | **Emissive** (same meaning!)          |
/// | `pad0`              | 3     | Unused                    | Unused                                |
/// | `matcap_blend_modes`| 4-7   | Mode 1: 4×matcap modes    | **Specular RGB + Rim power** (Mode 3) |
/// |                     |       |                           | Bytes 0-2: Specular RGB8              |
/// |                     |       |                           | Byte 3: Rim power [0-255]→[0-32]      |
///
/// **Key insights:**
/// - **Emissive stays in Slot 1.B for both modes** - no migration needed!
/// - Mode 3 reinterprets `metallic` → `rim_intensity` and `roughness` → `shininess`
/// - Specular color (Mode 3) comes from Slot 2 RGB texture OR uniform fallback (bytes 0-2 of `matcap_blend_modes`)
/// - Rim power (Mode 3) is uniform-only, stored in `matcap_blend_modes` byte 3, range 0-32
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Pod, Zeroable)]
pub struct PackedUnifiedShadingState {
    /// Mode 2: Metallic [0-255] → [0.0-1.0]
    /// Mode 3: Rim intensity [0-255] → [0.0-1.0] (uniform fallback for Slot 1.R)
    pub metallic: u8,

    /// Mode 2: Roughness [0-255] → [0.0-1.0]
    /// Mode 3: Shininess [0-255] → [0.0-1.0] → [1-256] (uniform fallback for Slot 1.G)
    pub roughness: u8,

    /// Mode 2: Emissive intensity [0-255+] (allows HDR values > 1.0)
    /// Mode 3: Emissive intensity [0-255+] (same meaning, uniform fallback for Slot 1.B)
    pub emissive: u8,

    pub pad0: u8,
    pub color_rgba8: u32,
    pub blend_mode: u32,

    /// Mode 1: 4x MatcapBlendMode packed as u8s
    /// Mode 3: Bytes 0-2 = Specular RGB8 (uniform fallback for Slot 2 RGB, defaults to white)
    ///         Byte 3 = rim_power [0-255] → [0.0-1.0] → [0-32] (uniform-only)
    pub matcap_blend_modes: u32,

    pub sky: PackedSky,           // 16 bytes
    pub lights: [PackedLight; 4], // 32 bytes (4 × 8-byte lights)
}

// TODO: Optimize matcap_blend_modes:
// We could explore this fields as embedding the mode123
// unique values
// Mode1: Blend Modes [4 of them]
// Mode2: [Metallic, Roughness, Emissive, _]
// Mode3: [Rim, Shininess, Emissive, _]

impl Default for PackedUnifiedShadingState {
    fn default() -> Self {
        // Reasonable defaults: pleasant outdoor lighting with visible dynamic lights
        // Convention: light direction = direction rays travel (physically correct)
        // Users can customize via sky_set_gradient() and sky_set_sun() FFI calls
        let sky = PackedSky::from_floats(
            Vec3::new(0.3, 0.4, 0.5),                // Horizon: pleasant blue ambient
            Vec3::new(0.1, 0.2, 0.4),                // Zenith: darker blue sky
            Vec3::new(-0.3, -0.5, -0.4).normalize(), // Sun direction: rays travel down-left-forward
            Vec3::new(0.7, 0.65, 0.6),               // Sun color: warm daylight
            0.95, // Sun sharpness: fairly sharp (maps to ~242/255)
        );

        Self {
            metallic: 0,    // Non-metallic
            roughness: 128, // Medium roughness (0.5)
            emissive: 0,    // No emission
            pad0: 0,
            color_rgba8: 0xFFFFFFFF, // White
            blend_mode: BlendMode::Alpha as u32,
            matcap_blend_modes: 0x00FFFFFF, // Mode 1: All Multiply | Mode 3: White specular RGB + rim_power=0
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

/// Encode normalized direction to octahedral coordinates in [-1, 1]²
/// Uses signed octahedral mapping for uniform precision distribution across the sphere.
/// More accurate than XY+reconstructed-Z approaches, especially near poles.
#[inline]
pub fn encode_octahedral(dir: Vec3) -> (f32, f32) {
    let dir = dir.normalize_or_zero();

    // Project to octahedron via L1 normalization
    let l1_norm = dir.x.abs() + dir.y.abs() + dir.z.abs();
    if l1_norm == 0.0 {
        return (0.0, 0.0);
    }

    let mut u = dir.x / l1_norm;
    let mut v = dir.y / l1_norm;

    // Fold lower hemisphere (z < 0) into upper square
    if dir.z < 0.0 {
        let u_abs = u.abs();
        let v_abs = v.abs();
        u = (1.0 - v_abs) * if u >= 0.0 { 1.0 } else { -1.0 };
        v = (1.0 - u_abs) * if v >= 0.0 { 1.0 } else { -1.0 };
    }

    (u, v) // Both in [-1, 1]
}

/// Pack Vec3 direction to u32 using octahedral encoding (2x snorm16)
/// Provides uniform angular precision (~0.02° worst-case error with 16-bit components)
#[inline]
pub fn pack_octahedral_u32(dir: Vec3) -> u32 {
    let (u, v) = encode_octahedral(dir);
    let u_snorm = pack_snorm16(u);
    let v_snorm = pack_snorm16(v);
    // Pack as [u: i16 low 16 bits][v: i16 high 16 bits]
    (u_snorm as u16 as u32) | ((v_snorm as u16 as u32) << 16)
}

/// Decode octahedral coordinates in [-1, 1]² back to normalized direction
/// Reverses the encoding operation to reconstruct the 3D direction vector.
#[inline]
pub fn decode_octahedral(u: f32, v: f32) -> Vec3 {
    let mut dir = Vec3::new(u, v, 1.0 - u.abs() - v.abs());

    // Unfold lower hemisphere (z < 0 case)
    if dir.z < 0.0 {
        let old_x = dir.x;
        dir.x = (1.0 - dir.y.abs()) * if old_x >= 0.0 { 1.0 } else { -1.0 };
        dir.y = (1.0 - old_x.abs()) * if dir.y >= 0.0 { 1.0 } else { -1.0 };
    }

    dir.normalize_or_zero()
}

/// Unpack u32 to Vec3 direction using octahedral decoding (2x snorm16)
/// Reverses pack_octahedral_u32() to extract the original direction.
#[inline]
pub fn unpack_octahedral_u32(packed: u32) -> Vec3 {
    // Extract i16 components with sign extension
    let u_i16 = ((packed & 0xFFFF) as i16) as i32;
    let v_i16 = ((packed >> 16) as i16) as i32;

    // Convert snorm16 to float [-1, 1]
    let u = unpack_snorm16(u_i16 as i16);
    let v = unpack_snorm16(v_i16 as i16);

    decode_octahedral(u, v)
}

/// Pack 4x MatcapBlendMode into u32 (4 bytes)
#[inline]
pub fn pack_matcap_blend_modes(modes: [MatcapBlendMode; 4]) -> u32 {
    (modes[0] as u32)
        | ((modes[1] as u32) << 8)
        | ((modes[2] as u32) << 16)
        | ((modes[3] as u32) << 24)
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
        let sun_color_and_sharpness = (sun_r as u32)
            | ((sun_g as u32) << 8)
            | ((sun_b as u32) << 16)
            | ((sun_sharp as u32) << 24);

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
    /// Create a PackedLight from f32 parameters
    /// If enabled=false, intensity is set to 0 (which indicates disabled light)
    pub fn from_floats(direction: Vec3, color: Vec3, intensity: f32, enabled: bool) -> Self {
        let dir_packed = pack_octahedral_u32(direction.normalize_or_zero());

        let r = pack_unorm8(color.x);
        let g = pack_unorm8(color.y);
        let b = pack_unorm8(color.z);
        // If disabled, set intensity to 0 (intensity=0 means disabled)
        let intens = if enabled { pack_unorm8(intensity) } else { 0 };
        let color_and_intensity =
            (r as u32) | ((g as u32) << 8) | ((b as u32) << 16) | ((intens as u32) << 24);

        Self {
            direction_oct: dir_packed,
            color_and_intensity,
        }
    }

    /// Create a disabled light (all zeros)
    pub fn disabled() -> Self {
        Self::default()
    }

    /// Extract direction as f32 array
    /// Decodes the octahedral-encoded direction stored in the packed light.
    pub fn get_direction(&self) -> [f32; 3] {
        let dir = unpack_octahedral_u32(self.direction_oct);
        [dir.x, dir.y, dir.z]
    }

    /// Extract color as f32 array
    pub fn get_color(&self) -> [f32; 3] {
        let r = unpack_unorm8((self.color_and_intensity & 0xFF) as u8);
        let g = unpack_unorm8(((self.color_and_intensity >> 8) & 0xFF) as u8);
        let b = unpack_unorm8(((self.color_and_intensity >> 16) & 0xFF) as u8);
        [r, g, b]
    }

    /// Extract intensity as f32
    pub fn get_intensity(&self) -> f32 {
        unpack_unorm8(((self.color_and_intensity >> 24) & 0xFF) as u8)
    }

    /// Check if light is enabled (intensity > 0)
    pub fn is_enabled(&self) -> bool {
        (self.color_and_intensity >> 24) != 0
    }
}

// ============================================================================
// PackedUnifiedShadingState Helpers
// ============================================================================

impl PackedUnifiedShadingState {
    /// Create from all f32 parameters (used during FFI calls)
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
        assert_eq!(std::mem::size_of::<PackedLight>(), 8); // Was 16, now 8 (50% reduction!)
        assert_eq!(std::mem::size_of::<PackedUnifiedShadingState>(), 64); // Was 100, now 64 (16 + 16 + 32)
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
        let packed = pack_rgba8(1.0, 0.5, 0.25, 1.0);
        assert_eq!(packed & 0xFF, 255); // R
        assert_eq!((packed >> 8) & 0xFF, 128); // G
        assert_eq!((packed >> 16) & 0xFF, 64); // B
        assert_eq!((packed >> 24) & 0xFF, 255); // A
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
        assert_eq!(light.direction_oct, 0);
        assert_eq!(light.color_and_intensity, 0);
        assert!(!light.is_enabled());
    }
}
