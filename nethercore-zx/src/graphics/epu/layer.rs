//! Core EPU layer types, opcodes, and encoding utilities.
//!
//! This module contains the fundamental types for EPU instruction layers,
//! including opcodes, blend modes, region masks, and the packed configuration format.

use glam::Vec3;

// =============================================================================
// Enums
// =============================================================================

/// EPU instruction opcodes (5-bit, 32 possible).
///
/// Opcode ranges:
/// - `0x00`: NOP (universal)
/// - `0x01..=0x07`: Bounds ops
/// - `0x08..=0x1F`: Radiance ops
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum EpuOpcode {
    /// Disable layer
    #[default]
    Nop = 0x0,
    /// Bounds gradient (sky/walls/floor)
    Ramp = 0x1,
    /// Angular wedge bounds modifier
    Sector = 0x2,
    /// Skyline/horizon cutout bounds modifier
    Silhouette = 0x3,
    /// Planar cut bounds source
    Split = 0x4,
    /// Voronoi/mosaic cell bounds source
    Cell = 0x5,
    /// Noise patch bounds source
    Patches = 0x6,
    /// Shaped opening/viewport bounds modifier
    Aperture = 0x7,
    /// Sharp SDF shape (disk/ring/rect/line)
    Decal = 0x8,
    /// Repeating lines/panels
    Grid = 0x9,
    /// Point field (stars/dust/bubbles)
    Scatter = 0xA,
    /// Animated noise/streaks/caustics
    Flow = 0xB,
    /// Procedural line/crack patterns
    Trace = 0xC,
    /// Curtain/ribbon effects
    Veil = 0xD,
    /// Atmospheric absorption + scattering
    Atmosphere = 0xE,
    /// Planar textures/patterns
    Plane = 0xF,
    /// Moon/sun/planet bodies
    Celestial = 0x10,
    /// Portal/vortex effects
    Portal = 0x11,
    /// Region-masked directional glow
    LobeRadiance = 0x12,
    /// Region-masked horizon band
    BandRadiance = 0x13,
}

// =============================================================================
// Region Mask Constants (3-bit bitfield)
// =============================================================================

/// Sky/ceiling region bit
pub const REGION_SKY: u8 = 0b100;
/// Wall/horizon belt region bit
pub const REGION_WALLS: u8 = 0b010;
/// Floor/ground region bit
pub const REGION_FLOOR: u8 = 0b001;
/// All regions combined (sky + walls + floor)
pub const REGION_ALL: u8 = 0b111;
/// No regions (layer disabled)
pub const REGION_NONE: u8 = 0b000;

/// Convenience enum for EPU region masks (3-bit mask).
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum EpuRegion {
    /// Apply everywhere
    #[default]
    All = 0b111,
    /// Sky/ceiling only
    Sky = 0b100,
    /// Wall/horizon belt only
    Walls = 0b010,
    /// Floor/ground only
    Floor = 0b001,
}

impl EpuRegion {
    /// Convert to 3-bit region mask
    #[inline]
    pub fn to_mask(self) -> u8 {
        self as u8
    }
}

/// EPU blend mode (3-bit, 8 modes)
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum EpuBlend {
    /// dst += src * a (additive blending)
    #[default]
    Add = 0,
    /// dst *= mix(1, src, a) (multiplicative/absorption)
    Multiply = 1,
    /// dst = max(dst, src * a)
    Max = 2,
    /// dst = mix(dst, src, a) (alpha blend)
    Lerp = 3,
    /// 1 - (1-dst)*(1-src*a) (screen blend)
    Screen = 4,
    /// HSV shift dst by src
    HsvMod = 5,
    /// dst = min(dst, src * a)
    Min = 6,
    /// Photoshop-style overlay
    Overlay = 7,
}

// =============================================================================
// Core Types
// =============================================================================

/// A single EPU instruction layer (unpacked form for building).
///
/// Use `encode()` to convert to the 128-bit packed format (two u64 values).
///
/// # Format
///
/// The format uses 128 bits per layer, providing:
/// - Direct RGB colors (no palette)
/// - Per-color alpha (4-bit each)
/// - Region masks (3-bit, combinable)
/// - 8 blend modes
#[derive(Clone, Copy, Debug)]
pub struct EpuLayer {
    /// Which algorithm to run (5-bit opcode)
    pub opcode: EpuOpcode,
    /// Region mask (3-bit bitfield: SKY=4, WALLS=2, FLOOR=1)
    pub region_mask: u8,
    /// How to combine layer output (3-bit, 8 modes)
    pub blend: EpuBlend,
    /// Meta header bits (5-bit): (domain_id << 3) | variant_id
    ///
    /// Set to 0 when unused.
    pub meta5: u8,
    /// Primary RGB color
    pub color_a: [u8; 3],
    /// Secondary RGB color
    pub color_b: [u8; 3],
    /// Primary alpha (0-15)
    pub alpha_a: u8,
    /// Secondary alpha (0-15)
    pub alpha_b: u8,
    /// Opcode-specific (usually brightness)
    pub intensity: u8,
    /// Opcode-specific parameter A
    pub param_a: u8,
    /// Opcode-specific parameter B
    pub param_b: u8,
    /// Opcode-specific parameter C
    pub param_c: u8,
    /// Opcode-specific parameter D
    pub param_d: u8,
    /// Octahedral-encoded direction (u8,u8)
    pub direction: u16,
}

impl Default for EpuLayer {
    fn default() -> Self {
        Self::nop()
    }
}

impl EpuLayer {
    /// Create a NOP (disabled) layer
    #[inline]
    pub fn nop() -> Self {
        Self {
            opcode: EpuOpcode::Nop,
            region_mask: REGION_ALL,
            blend: EpuBlend::Add,
            meta5: 0,
            color_a: [0, 0, 0],
            color_b: [0, 0, 0],
            alpha_a: 15,
            alpha_b: 15,
            intensity: 0,
            param_a: 0,
            param_b: 0,
            param_c: 0,
            param_d: 0,
            direction: 0,
        }
    }

    /// Encode this layer to the 128-bit packed format.
    ///
    /// Returns `[hi, lo]` where:
    ///
    /// ```text
    /// u64 hi [bits 127..64]:
    ///   bits 63..59: opcode     (5)
    ///   bits 58..56: region     (3)
    ///   bits 55..53: blend      (3)
    ///   bits 52..48: meta5      (5) - (domain_id<<3)|variant_id
    ///   bits 47..24: color_a    (24) RGB
    ///   bits 23..0:  color_b    (24) RGB
    ///
    /// u64 lo [bits 63..0]:
    ///   bits 63..56: intensity  (8)
    ///   bits 55..48: param_a    (8)
    ///   bits 47..40: param_b    (8)
    ///   bits 39..32: param_c    (8)
    ///   bits 31..24: param_d    (8)
    ///   bits 23..8:  direction  (16)
    ///   bits 7..4:   alpha_a    (4)
    ///   bits 3..0:   alpha_b    (4)
    /// ```
    #[inline]
    pub fn encode(self) -> [u64; 2] {
        let meta5 = (self.meta5 as u64) & 0x1F;
        let meta_hi = (meta5 >> 1) & 0xF;
        let meta_lo = meta5 & 0x1;

        // Pack color_a as RGB24
        let color_a_packed = ((self.color_a[0] as u64) << 16)
            | ((self.color_a[1] as u64) << 8)
            | (self.color_a[2] as u64);

        // Pack color_b as RGB24
        let color_b_packed = ((self.color_b[0] as u64) << 16)
            | ((self.color_b[1] as u64) << 8)
            | (self.color_b[2] as u64);

        // Build hi word
        let hi = ((self.opcode as u64 & 0x1F) << 59)
            | ((self.region_mask as u64 & 0x7) << 56)
            | ((self.blend as u64 & 0x7) << 53)
            | ((meta_hi & 0xF) << 49)
            | ((meta_lo & 0x1) << 48)
            | (color_a_packed << 24)
            | color_b_packed;

        // Build lo word
        let lo = ((self.intensity as u64) << 56)
            | ((self.param_a as u64) << 48)
            | ((self.param_b as u64) << 40)
            | ((self.param_c as u64) << 32)
            | ((self.param_d as u64) << 24)
            | ((self.direction as u64) << 8)
            | ((self.alpha_a as u64 & 0xF) << 4)
            | (self.alpha_b as u64 & 0xF);

        [hi, lo]
    }

    /// Create a layer with the given region (using `EpuRegion`).
    #[inline]
    pub fn with_region(mut self, region: EpuRegion) -> Self {
        self.region_mask = region.to_mask();
        self
    }
}

/// Packed EPU configuration (128 bytes = 8 x 128-bit instructions).
///
/// This is the GPU-consumable format. Each environment state is exactly
/// 8 layers packed into 128 bytes (each layer is 2 x u64 = 16 bytes).
///
/// Recommended slot usage:
/// - Slots 0-3: Bounds (RAMP + optional bounds ops)
/// - Slots 4-7: Radiance (DECAL/GRID/SCATTER/FLOW + radiance ops)
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct EpuConfig {
    /// 8 packed 128-bit instructions (each as [hi, lo])
    pub layers: [[u64; 2]; 8],
}

impl EpuConfig {
    /// Compute stable hash of the 128-byte config for dirty-state caching.
    ///
    /// This hash is used to detect when an environment configuration has changed
    /// and needs to be rebuilt on the GPU.
    pub fn state_hash(&self) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut hasher = DefaultHasher::new();
        self.layers.hash(&mut hasher);
        hasher.finish()
    }
}

// =============================================================================
// Direction Encoding
// =============================================================================

/// Encode a direction vector to octahedral u16 format.
///
/// Uses unsigned byte components scaled from [0, 255] representing [-1, 1].
/// This matches the WGSL decode function in the EPU compute shader.
///
/// # Arguments
/// * `dir` - Direction vector (will be normalized)
///
/// # Returns
/// Packed u16 with low byte = u, high byte = v
#[inline]
pub fn encode_direction_u16(dir: Vec3) -> u16 {
    let n = dir.normalize_or_zero();
    if n == Vec3::ZERO {
        // Default to +Y if zero vector
        return encode_direction_u16(Vec3::Y);
    }

    let denom = n.x.abs() + n.y.abs() + n.z.abs();
    let mut p = glam::Vec2::new(n.x, n.y) / denom;

    if n.z < 0.0 {
        let sign_x = if p.x >= 0.0 { 1.0 } else { -1.0 };
        let sign_y = if p.y >= 0.0 { 1.0 } else { -1.0 };
        p = glam::Vec2::new((1.0 - p.y.abs()) * sign_x, (1.0 - p.x.abs()) * sign_y);
    }

    // Map [-1, 1] to [0, 255]
    let u = ((p.x * 0.5 + 0.5) * 255.0).round().clamp(0.0, 255.0) as u16;
    let v = ((p.y * 0.5 + 0.5) * 255.0).round().clamp(0.0, 255.0) as u16;
    (u & 0xFF) | ((v & 0xFF) << 8)
}

/// Pack ceiling and floor Y thresholds into a single byte.
///
/// Each threshold is a 4-bit value (0..15) that maps to [-1, 1].
///
/// # Arguments
/// * `ceil_y_q` - Ceiling threshold quantized (0..15)
/// * `floor_y_q` - Floor threshold quantized (0..15)
#[inline]
pub fn pack_thresholds(ceil_y_q: u8, floor_y_q: u8) -> u8 {
    ((ceil_y_q & 0x0F) << 4) | (floor_y_q & 0x0F)
}

/// Pack `(domain_id, variant_id)` into the 5-bit `meta5` field.
///
/// `meta5 = (domain_id << 3) | variant_id`.
#[inline]
pub const fn pack_meta5(domain_id: u8, variant_id: u8) -> u8 {
    ((domain_id & 0x03) << 3) | (variant_id & 0x07)
}
