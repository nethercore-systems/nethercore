use bytemuck::{Pod, Zeroable};
use glam::Vec3;

use super::quantization::{pack_f16x2, pack_unorm8, unpack_f16x2, unpack_unorm8};
use zx_common::{pack_octahedral_u32, unpack_octahedral_u32};

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
///
/// - Directional: octahedral direction (snorm16x2)
/// - Point: position XY (f16x2)
///
/// **data1:** RGB8 (bits 31-8) + type (bit 7) + intensity (bits 6-0)
///
/// - Format: 0xRRGGBB_TI where T=type(1bit), I=intensity(7bits)
/// - Intensity maps 0-127 -> 0.0-8.0 for HDR support
///
/// **data2:**
///
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
