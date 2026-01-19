//! EPU (Environment Processing Unit) Rust API
//!
//! This module provides the Rust-side EPU types and builder API that encode
//! semantic environment configuration into the 128-byte packed format consumed
//! by the GPU compute shaders.
//!
//! # Architecture
//!
//! The EPU produces two directional radiance signals per environment:
//! - `L_sharp`: Bounds + all Features (for background + glossy reflections)
//! - `L_light0`: Bounds + emissive Features (for lighting/blur pyramid)
//!
//! # v2 Format (128-bit instructions)
//!
//! Each environment is 128 bytes (8 x 128-bit instructions). The 128-bit format
//! provides direct RGB colors, explicit emissive control, per-color alpha, and
//! region masks for more flexible compositing.
//!
//! # Example
//!
//! ```ignore
//! let mut e = epu_begin();
//! e.ramp_enclosure(RampParams { ... });
//! e.lobe(LobeParams { ... });
//! e.decal(DecalParams { ... });
//! let config = epu_finish(e);
//! ```

pub mod runtime;

#[cfg(test)]
mod tests;

// Re-export runtime types
pub use runtime::{
    ActiveEnvList, AmbientCube, EPU_MAP_SIZE, EpuRuntime, MAX_ACTIVE_ENVS, MAX_ENV_STATES,
    collect_active_envs,
};

use glam::Vec3;

// =============================================================================
// Enums
// =============================================================================

/// EPU instruction opcodes (5-bit, supports up to 32 opcodes)
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum EpuOpcode {
    /// Disable layer
    #[default]
    Nop = 0x0,
    /// Enclosure gradient (sky/walls/floor)
    Ramp = 0x1,
    /// Directional glow (sun, lamp, neon spill)
    Lobe = 0x2,
    /// Horizon band / ring
    Band = 0x3,
    /// Atmospheric absorption (use MULTIPLY blend)
    Fog = 0x4,
    /// Sharp SDF shape (disk/ring/rect/line)
    Decal = 0x5,
    /// Repeating lines/panels
    Grid = 0x6,
    /// Point field (stars/dust/bubbles)
    Scatter = 0x7,
    /// Animated noise/streaks/caustics
    Flow = 0x8,
}

// =============================================================================
// Region Mask Constants (v2 3-bit bitfield)
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

/// EPU region mask for features (v1 compatibility enum, maps to v2 mask)
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
    /// Convert to v2 region mask
    #[inline]
    pub fn to_mask(self) -> u8 {
        self as u8
    }
}

/// EPU blend mode (v2: 3-bit, 8 modes)
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

/// Decal shape types
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum DecalShape {
    /// Circular disk
    #[default]
    Disk = 0,
    /// Ring/annulus
    Ring = 1,
    /// Rectangle
    Rect = 2,
    /// Vertical line
    Line = 3,
}

/// Grid pattern types
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum GridPattern {
    /// Vertical stripes
    #[default]
    Stripes = 0,
    /// Crosshatch grid
    Grid = 1,
    /// Checkerboard
    Checker = 2,
}

/// Flow pattern types
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum FlowPattern {
    /// Perlin-like noise
    #[default]
    Noise = 0,
    /// Directional streaks
    Streaks = 1,
    /// Underwater caustic
    Caustic = 2,
}

// =============================================================================
// Core Types
// =============================================================================

/// A single EPU instruction layer (unpacked form for building).
///
/// Use `encode()` to convert to the 128-bit packed format (two u64 values).
///
/// # v2 Format
///
/// The v2 format uses 128 bits per layer, providing:
/// - Direct RGB colors (no palette)
/// - Explicit emissive control (4-bit)
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
    /// Emissive contribution to L_light0 (0-15, 0=decorative only)
    pub emissive: u8,
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
            emissive: 0,
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
    ///   bits 52..49: emissive   (4)
    ///   bit  48:     reserved   (1)
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
            | ((self.emissive as u64 & 0xF) << 49)
            // bit 48 reserved
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

    /// Create a layer with the given region (using EpuRegion enum for v1 compatibility)
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
/// - Slots 0-3: Bounds (RAMP, LOBE, BAND, FOG)
/// - Slots 4-7: Features (DECAL, GRID, SCATTER, FLOW)
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

    /// Check if any layer uses time-based animation.
    ///
    /// Returns `true` if the configuration contains any animated features
    /// that depend on the time uniform (e.g., pulsing, scrolling, twinkling).
    /// Time-dependent environments must be rebuilt every frame.
    pub fn is_time_dependent(&self) -> bool {
        for [hi, lo] in &self.layers {
            // Extract opcode from hi word bits 63..59
            let opcode = (hi >> 59) & 0x1F;
            // Extract params from lo word
            let param_b = ((lo >> 40) & 0xFF) as u8;
            let param_c = ((lo >> 32) & 0xFF) as u8;

            match opcode {
                // LOBE: anim_mode in param_c (0=none, 1=pulse, 2=flicker)
                0x2 => {
                    if param_c != 0 {
                        return true;
                    }
                }
                // BAND: scroll_speed in param_c
                0x3 => {
                    if param_c != 0 {
                        return true;
                    }
                }
                // DECAL: pulse_speed in param_c
                0x5 => {
                    if param_c != 0 {
                        return true;
                    }
                }
                // GRID: scroll_q in lower 4 bits of param_c
                0x6 => {
                    if param_c & 0x0F != 0 {
                        return true;
                    }
                }
                // SCATTER: twinkle_q in upper 4 bits of param_c
                0x7 => {
                    if (param_c >> 4) & 0x0F != 0 {
                        return true;
                    }
                }
                // FLOW: speed in param_b
                0x8 => {
                    if param_b != 0 {
                        return true;
                    }
                }
                _ => {}
            }
        }
        false
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

// =============================================================================
// Builder API
// =============================================================================

/// Begin building an EPU configuration.
///
/// Returns an `EpuBuilder` that can be used to add bounds and feature layers.
#[inline]
pub fn epu_begin() -> EpuBuilder {
    EpuBuilder::new()
}

/// Finish building and return the packed `EpuConfig`.
#[inline]
pub fn epu_finish(builder: EpuBuilder) -> EpuConfig {
    builder.finish()
}

/// Builder for constructing EPU configurations with semantic methods.
///
/// Automatically manages layer slot allocation:
/// - Bounds (RAMP, LOBE, BAND, FOG) go to slots 0-3
/// - Features (DECAL, GRID, SCATTER, FLOW) go to slots 4-7
pub struct EpuBuilder {
    cfg: EpuConfig,
    next_bounds: usize,
    next_feature: usize,
}

impl Default for EpuBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl EpuBuilder {
    /// Create a new builder with all layers initialized to NOP.
    #[inline]
    pub fn new() -> Self {
        Self {
            cfg: EpuConfig::default(),
            next_bounds: 0,
            next_feature: 4,
        }
    }

    /// Finish building and return the packed configuration.
    #[inline]
    pub fn finish(self) -> EpuConfig {
        self.cfg
    }

    /// Push a bounds layer (slots 0-3). Silently ignored if full.
    fn push_bounds(&mut self, layer: EpuLayer) {
        if self.next_bounds >= 4 {
            return;
        }
        self.cfg.layers[self.next_bounds] = layer.encode();
        self.next_bounds += 1;
    }

    /// Push a feature layer (slots 4-7). Silently ignored if full.
    fn push_feature(&mut self, layer: EpuLayer) {
        if self.next_feature >= 8 {
            return;
        }
        self.cfg.layers[self.next_feature] = layer.encode();
        self.next_feature += 1;
    }

    // =========================================================================
    // Bounds Helpers
    // =========================================================================

    /// Set the enclosure gradient (RAMP) - always goes to slot 0.
    ///
    /// This establishes the base colors and enclosure weights used by all other layers.
    pub fn ramp_enclosure(&mut self, p: RampParams) {
        let layer = EpuLayer {
            opcode: EpuOpcode::Ramp,
            region_mask: REGION_ALL,
            blend: EpuBlend::Add,
            emissive: p.emissive,
            color_a: p.sky_color,
            color_b: p.floor_color,
            alpha_a: 15,
            alpha_b: 15,
            intensity: p.softness,
            param_a: p.wall_color[0], // Wall R (for gradient mixing)
            param_b: p.wall_color[1], // Wall G
            param_c: p.wall_color[2], // Wall B
            param_d: pack_thresholds(p.ceil_q, p.floor_q),
            direction: encode_direction_u16(p.up),
        };
        // RAMP always goes to slot 0
        self.cfg.layers[0] = layer.encode();
        self.next_bounds = self.next_bounds.max(1);
    }

    /// Add a directional glow (LOBE).
    ///
    /// # Arguments
    /// * `dir` - Lobe center direction
    /// * `color` - RGB color for glow
    /// * `intensity` - Brightness (0..255)
    /// * `exponent` - Sharpness (0..255 maps to 1..64)
    /// * `anim_speed` - Animation speed (0..255 maps to 0..10)
    /// * `anim_mode` - Animation mode (0=none, 1=pulse, 2=flicker)
    pub fn lobe(
        &mut self,
        dir: Vec3,
        color: [u8; 3],
        intensity: u8,
        exponent: u8,
        anim_speed: u8,
        anim_mode: u8,
    ) {
        self.push_bounds(EpuLayer {
            opcode: EpuOpcode::Lobe,
            region_mask: REGION_ALL,
            blend: EpuBlend::Add,
            emissive: 15, // Lobes are emissive by default
            color_a: color,
            color_b: [0, 0, 0], // Edge tint (unused for now)
            alpha_a: 15,
            alpha_b: 15,
            intensity,
            param_a: exponent,
            param_b: anim_speed,
            param_c: anim_mode,
            param_d: 0,
            direction: encode_direction_u16(dir),
        });
    }

    /// Add a horizon band (BAND).
    ///
    /// # Arguments
    /// * `normal` - Band normal axis
    /// * `color` - RGB color for band
    /// * `intensity` - Brightness (0..255)
    /// * `width` - Band width (0..255 maps to 0.005..0.5)
    /// * `offset` - Vertical offset (0..255 maps to -0.5..0.5)
    /// * `scroll_speed` - Scroll speed (0..255 maps to 0..1)
    pub fn band(
        &mut self,
        normal: Vec3,
        color: [u8; 3],
        intensity: u8,
        width: u8,
        offset: u8,
        scroll_speed: u8,
    ) {
        self.push_bounds(EpuLayer {
            opcode: EpuOpcode::Band,
            region_mask: REGION_ALL,
            blend: EpuBlend::Add,
            emissive: 8, // Moderate emissive by default
            color_a: color,
            color_b: [0, 0, 0],
            alpha_a: 15,
            alpha_b: 15,
            intensity,
            param_a: width,
            param_b: offset,
            param_c: scroll_speed,
            param_d: 0,
            direction: encode_direction_u16(normal),
        });
    }

    /// Add atmospheric fog/absorption (FOG).
    ///
    /// Uses MULTIPLY blend mode for absorption effect.
    ///
    /// # Arguments
    /// * `up` - Up vector for vertical bias
    /// * `fog_color` - RGB color for fog tint
    /// * `density` - Fog density (0..255)
    /// * `vertical_bias` - Vertical bias (0..255 maps to -1..1)
    /// * `falloff` - Falloff curve (0..255 maps to 0.5..4.0)
    pub fn fog(
        &mut self,
        up: Vec3,
        fog_color: [u8; 3],
        density: u8,
        vertical_bias: u8,
        falloff: u8,
    ) {
        self.push_bounds(EpuLayer {
            opcode: EpuOpcode::Fog,
            region_mask: REGION_ALL,
            blend: EpuBlend::Multiply,
            emissive: 0, // Fog is not emissive
            color_a: fog_color,
            color_b: [255, 255, 255], // Clear color for fog
            alpha_a: 15,
            alpha_b: 15,
            intensity: density,
            param_a: vertical_bias,
            param_b: falloff,
            param_c: 0,
            param_d: 0,
            direction: encode_direction_u16(up),
        });
    }

    // =========================================================================
    // Feature Helpers
    // =========================================================================

    /// Add a decal shape (DECAL).
    ///
    /// Emissive contribution is controlled by the `emissive` field in DecalParams.
    pub fn decal(&mut self, p: DecalParams) {
        let param_a = ((p.shape as u8) << 4) | (p.softness_q & 0x0F);
        self.push_feature(EpuLayer {
            opcode: EpuOpcode::Decal,
            region_mask: p.region.to_mask(),
            blend: p.blend,
            emissive: p.emissive,
            color_a: p.color,
            color_b: p.color_b,
            alpha_a: p.alpha,
            alpha_b: 15,
            intensity: p.intensity,
            param_a,
            param_b: p.size,
            param_c: p.pulse_speed,
            param_d: 0,
            direction: encode_direction_u16(p.dir),
        });
    }

    /// Add scattered points (SCATTER).
    ///
    /// Emissive contribution is controlled by the `emissive` field in ScatterParams.
    pub fn scatter(&mut self, p: ScatterParams) {
        let param_c = ((p.twinkle_q & 0x0F) << 4) | (p.seed & 0x0F);
        self.push_feature(EpuLayer {
            opcode: EpuOpcode::Scatter,
            region_mask: p.region.to_mask(),
            blend: p.blend,
            emissive: p.emissive,
            color_a: p.color,
            color_b: [0, 0, 0],
            alpha_a: 15,
            alpha_b: 15,
            intensity: p.intensity,
            param_a: p.density,
            param_b: p.size,
            param_c,
            param_d: 0,
            direction: 0,
        });
    }

    /// Add a grid pattern (GRID).
    ///
    /// Emissive contribution is controlled by the `emissive` field in GridParams.
    pub fn grid(&mut self, p: GridParams) {
        let param_c = ((p.pattern as u8) << 4) | (p.scroll_q & 0x0F);
        self.push_feature(EpuLayer {
            opcode: EpuOpcode::Grid,
            region_mask: p.region.to_mask(),
            blend: p.blend,
            emissive: p.emissive,
            color_a: p.color,
            color_b: [0, 0, 0],
            alpha_a: 15,
            alpha_b: 15,
            intensity: p.intensity,
            param_a: p.scale,
            param_b: p.thickness,
            param_c,
            param_d: 0,
            direction: 0,
        });
    }

    /// Add animated flow (FLOW).
    ///
    /// Emissive contribution is controlled by the `emissive` field in FlowParams.
    pub fn flow(&mut self, p: FlowParams) {
        let param_c = ((p.octaves & 0x0F) << 4) | ((p.pattern as u8) & 0x0F);
        self.push_feature(EpuLayer {
            opcode: EpuOpcode::Flow,
            region_mask: p.region.to_mask(),
            blend: p.blend,
            emissive: p.emissive,
            color_a: p.color,
            color_b: [0, 0, 0],
            alpha_a: 15,
            alpha_b: 15,
            intensity: p.intensity,
            param_a: p.scale,
            param_b: p.speed,
            param_c,
            param_d: 0,
            direction: encode_direction_u16(p.dir),
        });
    }
}

// =============================================================================
// Parameter Structs (v2 - RGB colors)
// =============================================================================

/// Parameters for RAMP enclosure.
#[derive(Clone, Copy, Debug)]
pub struct RampParams {
    /// Up vector defining the enclosure orientation
    pub up: Vec3,
    /// RGB color for wall/horizon
    pub wall_color: [u8; 3],
    /// RGB color for sky/ceiling
    pub sky_color: [u8; 3],
    /// RGB color for floor/ground
    pub floor_color: [u8; 3],
    /// Ceiling threshold (0..15 maps to -1..1)
    pub ceil_q: u8,
    /// Floor threshold (0..15 maps to -1..1)
    pub floor_q: u8,
    /// Transition softness (0..255)
    pub softness: u8,
    /// Emissive contribution (0-15, 0=decorative only)
    pub emissive: u8,
}

impl Default for RampParams {
    fn default() -> Self {
        Self {
            up: Vec3::Y,
            wall_color: [0, 0, 0],
            sky_color: [0, 0, 0],
            floor_color: [0, 0, 0],
            ceil_q: 8,
            floor_q: 8,
            softness: 128,
            emissive: 0,
        }
    }
}

/// Parameters for DECAL feature.
#[derive(Clone, Copy, Debug)]
pub struct DecalParams {
    /// Region mask
    pub region: EpuRegion,
    /// Blend mode
    pub blend: EpuBlend,
    /// Shape type
    pub shape: DecalShape,
    /// Shape center direction
    pub dir: Vec3,
    /// RGB color for shape (primary)
    pub color: [u8; 3],
    /// RGB color for outline/glow (secondary)
    pub color_b: [u8; 3],
    /// Brightness (0..255)
    pub intensity: u8,
    /// Edge softness (0..15)
    pub softness_q: u8,
    /// Size (0..255 maps to 0..0.5 rad)
    pub size: u8,
    /// Pulse animation speed (0..255 maps to 0..10)
    pub pulse_speed: u8,
    /// Emissive contribution (0-15, 0=decorative only)
    pub emissive: u8,
    /// Alpha (0-15)
    pub alpha: u8,
}

impl Default for DecalParams {
    fn default() -> Self {
        Self {
            region: EpuRegion::All,
            blend: EpuBlend::Add,
            shape: DecalShape::Disk,
            dir: Vec3::Y,
            color: [255, 255, 255],
            color_b: [0, 0, 0],
            intensity: 255,
            softness_q: 2,
            size: 20,
            pulse_speed: 0,
            emissive: 15,
            alpha: 15,
        }
    }
}

/// Parameters for SCATTER feature.
#[derive(Clone, Copy, Debug)]
pub struct ScatterParams {
    /// Region mask
    pub region: EpuRegion,
    /// Blend mode
    pub blend: EpuBlend,
    /// RGB color for points
    pub color: [u8; 3],
    /// Brightness (0..255)
    pub intensity: u8,
    /// Point density (0..255 maps to 1..256)
    pub density: u8,
    /// Point size (0..255 maps to 0.001..0.05 rad)
    pub size: u8,
    /// Twinkle amount (0..15)
    pub twinkle_q: u8,
    /// Random seed (0..15)
    pub seed: u8,
    /// Emissive contribution (0-15, 0=decorative only)
    pub emissive: u8,
}

impl Default for ScatterParams {
    fn default() -> Self {
        Self {
            region: EpuRegion::All,
            blend: EpuBlend::Add,
            color: [255, 255, 255],
            intensity: 255,
            density: 200,
            size: 20,
            twinkle_q: 8,
            seed: 0,
            emissive: 15,
        }
    }
}

/// Parameters for GRID feature.
#[derive(Clone, Copy, Debug)]
pub struct GridParams {
    /// Region mask
    pub region: EpuRegion,
    /// Blend mode
    pub blend: EpuBlend,
    /// RGB color for lines
    pub color: [u8; 3],
    /// Brightness (0..255)
    pub intensity: u8,
    /// Grid scale (0..255 maps to 1..64)
    pub scale: u8,
    /// Line thickness (0..255 maps to 0.001..0.1)
    pub thickness: u8,
    /// Pattern type
    pub pattern: GridPattern,
    /// Scroll speed (0..15 maps to 0..2)
    pub scroll_q: u8,
    /// Emissive contribution (0-15, 0=decorative only)
    pub emissive: u8,
}

impl Default for GridParams {
    fn default() -> Self {
        Self {
            region: EpuRegion::Walls,
            blend: EpuBlend::Add,
            color: [64, 64, 64],
            intensity: 128,
            scale: 32,
            thickness: 20,
            pattern: GridPattern::Grid,
            scroll_q: 0,
            emissive: 8,
        }
    }
}

/// Parameters for FLOW feature.
#[derive(Clone, Copy, Debug)]
pub struct FlowParams {
    /// Region mask
    pub region: EpuRegion,
    /// Blend mode
    pub blend: EpuBlend,
    /// Flow direction
    pub dir: Vec3,
    /// RGB color for flow
    pub color: [u8; 3],
    /// Brightness (0..255)
    pub intensity: u8,
    /// Noise scale (0..255 maps to 1..16)
    pub scale: u8,
    /// Animation speed (0..255 maps to 0..2)
    pub speed: u8,
    /// Noise octaves (0..4)
    pub octaves: u8,
    /// Pattern type
    pub pattern: FlowPattern,
    /// Emissive contribution (0-15, 0=decorative only)
    pub emissive: u8,
}

impl Default for FlowParams {
    fn default() -> Self {
        Self {
            region: EpuRegion::Sky,
            blend: EpuBlend::Lerp,
            dir: Vec3::X,
            color: [128, 128, 128],
            intensity: 60,
            scale: 32,
            speed: 20,
            octaves: 2,
            pattern: FlowPattern::Noise,
            emissive: 0,
        }
    }
}
