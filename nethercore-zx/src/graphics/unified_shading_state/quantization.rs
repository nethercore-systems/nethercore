use glam::Vec3;
use half::f16;

use super::super::render_state::MatcapBlendMode;

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

/// Pack an f32 normalized value [-1.0, 1.0] to i16 snorm16 [-32767, 32767] (test helper).
#[cfg(test)]
#[inline]
pub fn pack_snorm16(value: f32) -> i16 {
    (value.clamp(-1.0, 1.0) * 32767.0).round() as i16
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
