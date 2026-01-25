//! EPU (Environment Processing Unit) Rust API
//!
//! This module provides the Rust-side EPU types and builder API that encode
//! semantic environment configuration into the 128-byte packed format consumed
//! by the GPU compute shaders.
//!
//! # Architecture
//!
//! The EPU produces a single directional radiance signal per environment.
//! That radiance is stored in `EnvRadiance` (mip 0) and then downsampled into
//! a true mip pyramid for roughness-based reflections. Diffuse ambient uses
//! SH9 coefficients extracted from a coarse mip level.
//!
//! # Format (128-bit instructions)
//!
//! Each environment is 128 bytes (8 x 128-bit instructions). The 128-bit format
//! provides direct RGB colors, per-color alpha, and region masks for more
//! flexible compositing.
//!
//! # Example
//!
//! ```ignore
//! let mut e = epu_begin();
//! e.ramp_enclosure(RampParams { ... });
//! e.sector_enclosure(SectorParams { ... });
//! e.decal(DecalParams { ..Default::default() });
//! e.lobe_radiance(LobeRadianceParams { ... });
//! let config = epu_finish(e);
//! ```

// Submodules for organized runtime code
mod cache;
mod pipelines;
pub mod runtime;
mod settings;
mod shaders;
mod types;

#[cfg(test)]
mod tests;

// Re-export runtime types
pub use cache::{ActiveEnvList, collect_active_envs};
pub use runtime::EpuRuntime;
pub use settings::{
    EPU_MAP_SIZE, EPU_MIN_MIP_SIZE, EpuRuntimeSettings, MAX_ACTIVE_ENVS, MAX_ENV_STATES,
};
pub use types::EpuSh9;

use glam::Vec3;

// =============================================================================
// Enums
// =============================================================================

/// EPU instruction opcodes (5-bit, 32 possible).
///
/// Opcode ranges:
/// - `0x00`: NOP (universal)
/// - `0x01..=0x07`: Enclosure ops
/// - `0x08..=0x1F`: Radiance ops
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum EpuOpcode {
    /// Disable layer
    #[default]
    Nop = 0x0,
    /// Enclosure gradient (sky/walls/floor)
    Ramp = 0x1,
    /// Angular wedge enclosure modifier
    Sector = 0x2,
    /// Skyline/horizon cutout enclosure modifier
    Silhouette = 0x3,
    /// Planar cut enclosure source
    Split = 0x4,
    /// Voronoi/mosaic cell enclosure source
    Cell = 0x5,
    /// Noise patch enclosure source
    Patches = 0x6,
    /// Shaped opening/viewport enclosure modifier
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

/// Waveforms for phase-driven modulation.
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum PhaseWaveform {
    /// No modulation (constant).
    #[default]
    Off = 0,
    /// Smooth sine modulation.
    Sine = 1,
    /// Linear up/down (triangle) modulation.
    Triangle = 2,
    /// Hard on/off modulation.
    Strobe = 3,
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
/// - Slots 0-3: Enclosure (RAMP + optional enclosure ops)
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
/// - Enclosure (RAMP + enclosure ops) goes to slots 0-3
/// - Radiance (feature ops) goes to slots 4-7
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
    // Enclosure (Bounds) Helpers
    // =========================================================================

    /// Set the enclosure gradient (RAMP) - always goes to slot 0.
    ///
    /// This establishes the base colors and enclosure weights used by all other layers.
    pub fn ramp_enclosure(&mut self, p: RampParams) {
        let layer = EpuLayer {
            opcode: EpuOpcode::Ramp,
            region_mask: REGION_ALL,
            blend: EpuBlend::Add,
            meta5: 0,
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

    /// Apply a SECTOR enclosure modifier.
    pub fn sector_enclosure(&mut self, p: SectorParams) {
        self.push_bounds(EpuLayer {
            opcode: EpuOpcode::Sector,
            region_mask: REGION_ALL,
            blend: EpuBlend::Add,
            meta5: pack_meta5(0, p.variant_id),
            color_a: p.sky_color,
            color_b: p.wall_color,
            alpha_a: 15,
            alpha_b: 15,
            intensity: p.strength,
            param_a: p.center_u01,
            param_b: p.width,
            param_c: 0,
            param_d: 0,
            direction: encode_direction_u16(p.up),
        });
    }

    /// Apply a SILHOUETTE enclosure modifier.
    pub fn silhouette_enclosure(&mut self, p: SilhouetteParams) {
        let param_c = ((p.octaves_q & 0x0F) << 4) | (p.drift_amount_q & 0x0F);
        self.push_bounds(EpuLayer {
            opcode: EpuOpcode::Silhouette,
            region_mask: REGION_ALL,
            blend: EpuBlend::Add,
            meta5: pack_meta5(0, p.variant_id),
            color_a: p.silhouette_color,
            color_b: p.background_color,
            alpha_a: p.strength,
            alpha_b: 0,
            intensity: p.edge_softness,
            param_a: p.horizon_bias,
            param_b: p.roughness,
            param_c,
            param_d: p.drift_speed,
            direction: encode_direction_u16(p.up),
        });
    }

    /// Apply a SPLIT enclosure source.
    pub fn split_enclosure(&mut self, p: SplitParams) {
        self.push_bounds(EpuLayer {
            opcode: EpuOpcode::Split,
            region_mask: REGION_ALL,
            blend: EpuBlend::Add,
            meta5: pack_meta5(0, p.variant_id),
            color_a: p.sky_color,
            color_b: p.wall_color,
            alpha_a: 15,
            alpha_b: 15,
            intensity: 0,
            param_a: p.blend_width,
            param_b: p.wedge_angle,
            param_c: p.count,
            param_d: p.offset,
            direction: encode_direction_u16(p.axis),
        });
    }

    /// Apply a CELL enclosure source.
    pub fn cell_enclosure(&mut self, p: CellParams) {
        self.push_bounds(EpuLayer {
            opcode: EpuOpcode::Cell,
            region_mask: REGION_ALL,
            blend: EpuBlend::Add,
            meta5: pack_meta5(0, p.variant_id),
            color_a: p.gap_color,
            color_b: p.wall_color,
            alpha_a: p.gap_alpha,
            alpha_b: p.outline_alpha,
            intensity: p.outline_brightness,
            param_a: p.density,
            param_b: p.fill_ratio,
            param_c: p.gap_width,
            param_d: p.seed,
            direction: encode_direction_u16(p.axis),
        });
    }

    /// Apply a PATCHES enclosure source.
    pub fn patches_enclosure(&mut self, p: PatchesParams) {
        self.push_bounds(EpuLayer {
            opcode: EpuOpcode::Patches,
            region_mask: REGION_ALL,
            blend: EpuBlend::Add,
            meta5: pack_meta5(p.domain_id, p.variant_id),
            color_a: p.sky_color,
            color_b: p.wall_color,
            alpha_a: p.sky_alpha,
            alpha_b: p.wall_alpha,
            intensity: 0,
            param_a: p.scale,
            param_b: p.coverage,
            param_c: p.sharpness,
            param_d: p.seed,
            direction: encode_direction_u16(p.axis),
        });
    }

    /// Apply an APERTURE enclosure modifier.
    pub fn aperture_enclosure(&mut self, p: ApertureParams) {
        self.push_bounds(EpuLayer {
            opcode: EpuOpcode::Aperture,
            region_mask: REGION_ALL,
            blend: EpuBlend::Add,
            meta5: pack_meta5(0, p.variant_id),
            color_a: p.opening_color,
            color_b: p.frame_color,
            alpha_a: 0,
            alpha_b: 0,
            intensity: p.edge_softness,
            param_a: p.half_width,
            param_b: p.half_height,
            param_c: p.frame_thickness,
            param_d: p.variant_param,
            direction: encode_direction_u16(p.dir),
        });
    }

    // =========================================================================
    // Feature Helpers
    // =========================================================================

    /// Add a decal shape (DECAL).
    pub fn decal(&mut self, p: DecalParams) {
        let param_a = ((p.shape as u8) << 4) | (p.softness_q & 0x0F);
        self.push_feature(EpuLayer {
            opcode: EpuOpcode::Decal,
            region_mask: p.region.to_mask(),
            blend: p.blend,
            meta5: 0,
            color_a: p.color,
            color_b: p.color_b,
            alpha_a: p.alpha,
            alpha_b: 15,
            intensity: p.intensity,
            param_a,
            param_b: p.size,
            param_c: p.glow_softness,
            param_d: p.phase,
            direction: encode_direction_u16(p.dir),
        });
    }

    /// Add scattered points (SCATTER).
    pub fn scatter(&mut self, p: ScatterParams) {
        let param_c = (p.twinkle_q & 0x0F) << 4;
        self.push_feature(EpuLayer {
            opcode: EpuOpcode::Scatter,
            region_mask: p.region.to_mask(),
            blend: p.blend,
            meta5: 0,
            color_a: p.color,
            color_b: [0, 0, 0],
            alpha_a: 15,
            alpha_b: 15,
            intensity: p.intensity,
            param_a: p.density,
            param_b: p.size,
            param_c,
            param_d: p.seed,
            direction: 0,
        });
    }

    /// Add a grid pattern (GRID).
    pub fn grid(&mut self, p: GridParams) {
        let param_c = ((p.pattern as u8) << 4) | (p.scroll_q & 0x0F);
        self.push_feature(EpuLayer {
            opcode: EpuOpcode::Grid,
            region_mask: p.region.to_mask(),
            blend: p.blend,
            meta5: 0,
            color_a: p.color,
            color_b: [0, 0, 0],
            alpha_a: 15,
            alpha_b: 15,
            intensity: p.intensity,
            param_a: p.scale,
            param_b: p.thickness,
            param_c,
            param_d: p.phase,
            direction: 0,
        });
    }

    /// Add animated flow (FLOW).
    pub fn flow(&mut self, p: FlowParams) {
        let param_c = ((p.octaves & 0x0F) << 4) | ((p.pattern as u8) & 0x0F);
        self.push_feature(EpuLayer {
            opcode: EpuOpcode::Flow,
            region_mask: p.region.to_mask(),
            blend: p.blend,
            meta5: 0,
            color_a: p.color,
            color_b: [0, 0, 0],
            alpha_a: 15,
            alpha_b: 15,
            intensity: p.intensity,
            param_a: p.scale,
            param_b: p.phase,
            param_c,
            param_d: p.turbulence,
            direction: encode_direction_u16(p.dir),
        });
    }

    /// Add a directional glow (LOBE_RADIANCE).
    pub fn lobe_radiance(&mut self, p: LobeRadianceParams) {
        self.push_feature(EpuLayer {
            opcode: EpuOpcode::LobeRadiance,
            region_mask: p.region.to_mask(),
            blend: p.blend,
            meta5: 0,
            color_a: p.color,
            color_b: p.edge_color,
            alpha_a: p.alpha,
            alpha_b: 0,
            intensity: p.intensity,
            param_a: p.exponent,
            param_b: p.falloff,
            param_c: p.waveform as u8,
            param_d: p.phase,
            direction: encode_direction_u16(p.dir),
        });
    }

    /// Add a horizon band (BAND_RADIANCE).
    pub fn band_radiance(&mut self, p: BandRadianceParams) {
        self.push_feature(EpuLayer {
            opcode: EpuOpcode::BandRadiance,
            region_mask: p.region.to_mask(),
            blend: p.blend,
            meta5: 0,
            color_a: p.color,
            color_b: p.edge_color,
            alpha_a: p.alpha,
            alpha_b: 0,
            intensity: p.intensity,
            param_a: p.width,
            param_b: p.offset,
            param_c: p.softness,
            param_d: p.phase,
            direction: encode_direction_u16(p.axis),
        });
    }

    /// Add atmospheric absorption/scattering (ATMOSPHERE).
    pub fn atmosphere(&mut self, p: AtmosphereParams) {
        self.push_feature(EpuLayer {
            opcode: EpuOpcode::Atmosphere,
            region_mask: p.region.to_mask(),
            blend: p.blend,
            meta5: pack_meta5(0, p.variant_id),
            color_a: p.zenith_color,
            color_b: p.horizon_color,
            alpha_a: p.alpha,
            alpha_b: 0,
            intensity: p.intensity,
            param_a: p.falloff_exponent,
            param_b: p.horizon_y,
            param_c: p.mie_concentration,
            param_d: p.mie_exponent,
            direction: encode_direction_u16(p.sun_dir),
        });
    }
}

// =============================================================================
// Parameter Structs (RGB colors)
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
        }
    }
}

/// Parameters for SECTOR enclosure.
#[derive(Clone, Copy, Debug)]
pub struct SectorParams {
    pub up: Vec3,
    pub sky_color: [u8; 3],
    pub wall_color: [u8; 3],
    pub strength: u8,
    pub center_u01: u8,
    pub width: u8,
    pub variant_id: u8,
}

impl Default for SectorParams {
    fn default() -> Self {
        Self {
            up: Vec3::Y,
            sky_color: [0, 0, 0],
            wall_color: [0, 0, 0],
            strength: 0,
            center_u01: 128,
            width: 64,
            variant_id: 0,
        }
    }
}

/// Parameters for SILHOUETTE enclosure.
#[derive(Clone, Copy, Debug)]
pub struct SilhouetteParams {
    pub up: Vec3,
    pub silhouette_color: [u8; 3],
    pub background_color: [u8; 3],
    pub edge_softness: u8,
    pub horizon_bias: u8,
    pub roughness: u8,
    pub octaves_q: u8,
    pub drift_amount_q: u8,
    pub drift_speed: u8,
    pub strength: u8,
    pub variant_id: u8,
}

impl Default for SilhouetteParams {
    fn default() -> Self {
        Self {
            up: Vec3::Y,
            silhouette_color: [0, 0, 0],
            background_color: [0, 0, 0],
            edge_softness: 32,
            horizon_bias: 128,
            roughness: 128,
            octaves_q: 4,
            drift_amount_q: 0,
            drift_speed: 0,
            strength: 15,
            variant_id: 0,
        }
    }
}

/// Parameters for SPLIT enclosure.
#[derive(Clone, Copy, Debug)]
pub struct SplitParams {
    pub axis: Vec3,
    pub sky_color: [u8; 3],
    pub wall_color: [u8; 3],
    pub blend_width: u8,
    pub wedge_angle: u8,
    pub count: u8,
    pub offset: u8,
    pub variant_id: u8,
}

impl Default for SplitParams {
    fn default() -> Self {
        Self {
            axis: Vec3::Y,
            sky_color: [0, 0, 0],
            wall_color: [0, 0, 0],
            blend_width: 16,
            wedge_angle: 128,
            count: 8,
            offset: 0,
            variant_id: 0,
        }
    }
}

/// Parameters for CELL enclosure.
#[derive(Clone, Copy, Debug)]
pub struct CellParams {
    pub axis: Vec3,
    pub gap_color: [u8; 3],
    pub wall_color: [u8; 3],
    pub outline_brightness: u8,
    pub density: u8,
    pub fill_ratio: u8,
    pub gap_width: u8,
    pub seed: u8,
    pub gap_alpha: u8,
    pub outline_alpha: u8,
    pub variant_id: u8,
}

impl Default for CellParams {
    fn default() -> Self {
        Self {
            axis: Vec3::Y,
            gap_color: [0, 0, 0],
            wall_color: [0, 0, 0],
            outline_brightness: 0,
            density: 64,
            fill_ratio: 255,
            gap_width: 0,
            seed: 0,
            gap_alpha: 15,
            outline_alpha: 15,
            variant_id: 0,
        }
    }
}

/// Parameters for PATCHES enclosure.
#[derive(Clone, Copy, Debug)]
pub struct PatchesParams {
    pub axis: Vec3,
    pub sky_color: [u8; 3],
    pub wall_color: [u8; 3],
    pub scale: u8,
    pub coverage: u8,
    pub sharpness: u8,
    pub seed: u8,
    pub sky_alpha: u8,
    pub wall_alpha: u8,
    pub domain_id: u8,
    pub variant_id: u8,
}

impl Default for PatchesParams {
    fn default() -> Self {
        Self {
            axis: Vec3::Y,
            sky_color: [0, 0, 0],
            wall_color: [0, 0, 0],
            scale: 32,
            coverage: 128,
            sharpness: 64,
            seed: 0,
            sky_alpha: 15,
            wall_alpha: 15,
            domain_id: 0,
            variant_id: 0,
        }
    }
}

/// Parameters for APERTURE enclosure.
#[derive(Clone, Copy, Debug)]
pub struct ApertureParams {
    pub dir: Vec3,
    pub opening_color: [u8; 3],
    pub frame_color: [u8; 3],
    pub edge_softness: u8,
    pub half_width: u8,
    pub half_height: u8,
    pub frame_thickness: u8,
    pub variant_param: u8,
    pub variant_id: u8,
}

impl Default for ApertureParams {
    fn default() -> Self {
        Self {
            dir: Vec3::Z,
            opening_color: [0, 0, 0],
            frame_color: [0, 0, 0],
            edge_softness: 16,
            half_width: 128,
            half_height: 128,
            frame_thickness: 64,
            variant_param: 0,
            variant_id: 0,
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
    /// Glow softness (0..255 maps to 0..0.2)
    pub glow_softness: u8,
    /// Looping animation phase (0..255 maps to 0..1).
    ///
    /// Advance this from your game (deterministic) to animate the decal.
    pub phase: u8,
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
            glow_softness: 64,
            phase: 0,
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
    /// Random seed (0..255)
    pub seed: u8,
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
    /// Looping animation phase (0..255 maps to 0..1).
    ///
    /// Advance this from your game (deterministic) to animate scrolling.
    pub phase: u8,
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
            phase: 0,
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
    /// Looping animation phase (0..255 maps to 0..1).
    ///
    /// Advance this from your game (deterministic) to animate the pattern.
    pub phase: u8,
    /// Noise octaves (0..4)
    pub octaves: u8,
    /// Pattern type
    pub pattern: FlowPattern,
    /// Turbulence amount (0..255)
    pub turbulence: u8,
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
            phase: 0,
            octaves: 2,
            pattern: FlowPattern::Noise,
            turbulence: 0,
        }
    }
}

/// Parameters for LOBE_RADIANCE feature.
#[derive(Clone, Copy, Debug)]
pub struct LobeRadianceParams {
    pub region: EpuRegion,
    pub blend: EpuBlend,
    pub dir: Vec3,
    pub color: [u8; 3],
    pub edge_color: [u8; 3],
    pub intensity: u8,
    pub exponent: u8,
    pub falloff: u8,
    /// How to interpret [`phase`](Self::phase) for modulation.
    pub waveform: PhaseWaveform,
    /// Looping animation phase (0..255 maps to 0..1).
    ///
    /// Advance this from your game (deterministic) to animate the lobe.
    pub phase: u8,
    pub alpha: u8,
}

impl Default for LobeRadianceParams {
    fn default() -> Self {
        Self {
            region: EpuRegion::All,
            blend: EpuBlend::Add,
            dir: Vec3::Y,
            color: [255, 255, 255],
            edge_color: [0, 0, 0],
            intensity: 255,
            exponent: 64,
            falloff: 64,
            waveform: PhaseWaveform::Off,
            phase: 0,
            alpha: 15,
        }
    }
}

/// Parameters for BAND_RADIANCE feature.
#[derive(Clone, Copy, Debug)]
pub struct BandRadianceParams {
    pub region: EpuRegion,
    pub blend: EpuBlend,
    pub axis: Vec3,
    pub color: [u8; 3],
    pub edge_color: [u8; 3],
    pub intensity: u8,
    pub width: u8,
    pub offset: u8,
    pub softness: u8,
    /// Looping modulation phase (0..255 maps to 0..1).
    ///
    /// Advance this from your game (deterministic) to scroll the modulation around the band.
    pub phase: u8,
    pub alpha: u8,
}

impl Default for BandRadianceParams {
    fn default() -> Self {
        Self {
            region: EpuRegion::All,
            blend: EpuBlend::Add,
            axis: Vec3::Y,
            color: [255, 255, 255],
            edge_color: [0, 0, 0],
            intensity: 255,
            width: 64,
            offset: 128,
            softness: 0,
            phase: 0,
            alpha: 15,
        }
    }
}

/// Parameters for ATMOSPHERE feature.
#[derive(Clone, Copy, Debug)]
pub struct AtmosphereParams {
    pub region: EpuRegion,
    pub blend: EpuBlend,
    pub zenith_color: [u8; 3],
    pub horizon_color: [u8; 3],
    pub intensity: u8,
    pub falloff_exponent: u8,
    pub horizon_y: u8,
    pub mie_concentration: u8,
    pub mie_exponent: u8,
    pub sun_dir: Vec3,
    pub alpha: u8,
    pub variant_id: u8,
}

impl Default for AtmosphereParams {
    fn default() -> Self {
        Self {
            region: EpuRegion::All,
            blend: EpuBlend::Add,
            zenith_color: [0, 0, 0],
            horizon_color: [0, 0, 0],
            intensity: 0,
            falloff_exponent: 128,
            horizon_y: 128,
            mie_concentration: 0,
            mie_exponent: 64,
            sun_dir: Vec3::Y,
            alpha: 15,
            variant_id: 0,
        }
    }
}
