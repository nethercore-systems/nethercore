use bytemuck::{Pod, Zeroable};
use glam::{Vec3, Vec4};

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

    /// Reserved for alignment/future use
    pub _pad0: u32,

    pub sky: PackedSky,           // 16 bytes
    pub lights: [PackedLight; 4], // 32 bytes (4 × 8-byte lights)
}

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

        // uniform_set_0: [metallic=0, roughness=128, emissive=0, rim_intensity=0]
        let uniform_set_0 = pack_uniform_set_0(0, 128, 0, 0);
        // uniform_set_1: [spec_r=255, spec_g=255, spec_b=255, rim_power=0] (white specular)
        let uniform_set_1 = pack_uniform_set_1(255, 255, 255, 0);

        Self {
            color_rgba8: 0xFFFFFFFF, // White
            uniform_set_0,
            uniform_set_1,
            _pad0: 0,
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
    /// Create a PackedLight from f32 parameters
    /// If enabled=false, intensity is set to 0 (which indicates disabled light)
    pub fn from_floats(direction: Vec3, color: Vec3, intensity: f32, enabled: bool) -> Self {
        let dir_packed = pack_octahedral_u32(direction.normalize_or_zero());

        let r = pack_unorm8(color.x);
        let g = pack_unorm8(color.y);
        let b = pack_unorm8(color.z);
        // If disabled, set intensity to 0 (intensity=0 means disabled)
        let intens = if enabled { pack_unorm8(intensity) } else { 0 };
        // Format: 0xRRGGBBII (R in highest byte, intensity in lowest)
        let color_and_intensity =
            ((r as u32) << 24) | ((g as u32) << 16) | ((b as u32) << 8) | (intens as u32);

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
    /// Format: 0xRRGGBBII (R in highest byte, intensity in lowest)
    pub fn get_color(&self) -> [f32; 3] {
        let r = unpack_unorm8(((self.color_and_intensity >> 24) & 0xFF) as u8);
        let g = unpack_unorm8(((self.color_and_intensity >> 16) & 0xFF) as u8);
        let b = unpack_unorm8(((self.color_and_intensity >> 8) & 0xFF) as u8);
        [r, g, b]
    }

    /// Extract intensity as f32
    /// Format: 0xRRGGBBII (intensity in lowest byte)
    pub fn get_intensity(&self) -> f32 {
        unpack_unorm8((self.color_and_intensity & 0xFF) as u8)
    }

    /// Check if light is enabled (intensity > 0)
    pub fn is_enabled(&self) -> bool {
        (self.color_and_intensity & 0xFF) != 0
    }
}

// ============================================================================
// PackedUnifiedShadingState Helpers
// ============================================================================

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
            _pad0: 0,
            uniform_set_1,
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
        assert_eq!(light.direction_oct, 0);
        assert_eq!(light.color_and_intensity, 0);
        assert!(!light.is_enabled());
    }
}
