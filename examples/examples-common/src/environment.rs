//! Debug Environment helpers (Multi-Environment v4).
//!
//! This module provides environment configuration for examples using the new
//! EPU (Environment Processing Unit) API. The legacy env_* functions have been
//! removed; use epu_set() and epu_draw() instead.
//!
//! # EPU v2 Format (128-bit instructions)
//!
//! Each environment is 128 bytes (8 x 128-bit instructions). Each 128-bit
//! instruction is stored as two u64 values `[hi, lo]`:
//!
//! ```text
//! u64 hi [bits 127..64]:
//!   bits 63..59: opcode     (5)   - NOP=0, RAMP=1, LOBE=2, BAND=3, FOG=4, DECAL=5, GRID=6, SCATTER=7, FLOW=8
//!   bits 58..56: region     (3)   - Bitfield: SKY=0b100, WALLS=0b010, FLOOR=0b001, ALL=0b111
//!   bits 55..53: blend      (3)   - ADD=0, MULTIPLY=1, MAX=2, LERP=3, SCREEN=4, HSV_MOD=5, MIN=6, OVERLAY=7
//!   bits 52..49: emissive   (4)   - L_light0 contribution (0=decorative, 15=full)
//!   bit  48:     reserved   (1)
//!   bits 47..24: color_a    (24)  - RGB24 primary color
//!   bits 23..0:  color_b    (24)  - RGB24 secondary color
//!
//! u64 lo [bits 63..0]:
//!   bits 63..56: intensity  (8)
//!   bits 55..48: param_a    (8)
//!   bits 47..40: param_b    (8)
//!   bits 39..32: param_c    (8)
//!   bits 31..24: param_d    (8)
//!   bits 23..8:  direction  (16)  - Octahedral encoded
//!   bits 7..4:   alpha_a    (4)   - color_a alpha (0-15)
//!   bits 3..0:   alpha_b    (4)   - color_b alpha (0-15)
//! ```
//!
//! Slots 0-3: Bounds layers (RAMP, LOBE, BAND, FOG)
//! Slots 4-7: Feature layers (DECAL, GRID, SCATTER, FLOW)

use crate::ffi::*;

/// Environment mode constants (0–7).
/// Note: These are kept for compatibility but the underlying implementation
/// now uses EPU presets instead of the legacy env_* functions.
pub mod env_mode {
    pub const GRADIENT: u32 = 0;
    pub const CELLS: u32 = 1;
    pub const LINES: u32 = 2;
    pub const SILHOUETTE: u32 = 3;
    pub const NEBULA: u32 = 4;
    pub const ROOM: u32 = 5;
    pub const VEIL: u32 = 6;
    pub const RINGS: u32 = 7;

    // Backward-compat aliases (legacy names).
    pub const SCATTER: u32 = CELLS;
    pub const RECTANGLES: u32 = NEBULA;
    pub const CURTAINS: u32 = VEIL;
}

/// Blend mode constants for environment layering.
/// Note: EPU uses different blend semantics; these are kept for API compatibility.
pub mod blend_mode {
    pub const ALPHA: u32 = 0;
    pub const ADD: u32 = 1;
    pub const MULTIPLY: u32 = 2;
    pub const SCREEN: u32 = 3;
}

#[derive(Clone, Copy)]
pub struct GradientParams {
    pub zenith: u32,
    pub sky_horizon: u32,
    pub ground_horizon: u32,
    pub nadir: u32,
    pub rotation: f32,
    pub shift: f32,
    pub sun_elevation: f32,
    pub sun_disk: u32,
    pub sun_halo: u32,
    pub sun_intensity: u32,
    pub horizon_haze: u32,
    pub sun_warmth: u32,
    pub cloudiness: u32,
    pub cloud_phase: u32,
}

impl GradientParams {
    pub fn clear_day() -> Self {
        Self {
            zenith: 0x2E65FFFF,
            sky_horizon: 0xA9D8FFFF,
            ground_horizon: 0x4D8B4DFF,
            nadir: 0x102010FF,
            rotation: 0.35,
            shift: 0.00,
            sun_elevation: 0.95,
            sun_disk: 10,
            sun_halo: 72,
            sun_intensity: 230,
            horizon_haze: 32,
            sun_warmth: 24,
            cloudiness: 40,
            cloud_phase: 0,
        }
    }
}

impl Default for GradientParams {
    fn default() -> Self {
        Self::clear_day()
    }
}

#[derive(Clone, Copy)]
pub struct CellsParams {
    pub family: u32,
    pub variant: u32,
    pub density: u32,
    pub size_min: u32,
    pub size_max: u32,
    pub intensity: u32,
    pub shape: u32,
    pub motion: u32,
    pub parallax: u32,
    pub height_bias: u32,
    pub clustering: u32,
    pub color_a: u32,
    pub color_b: u32,
    pub axis_x: f32,
    pub axis_y: f32,
    pub axis_z: f32,
    pub phase: u16,
    pub seed: u32,
}

impl CellsParams {
    pub fn disabled() -> Self {
        Self {
            family: 0,
            variant: 0,
            density: 0,
            size_min: 0,
            size_max: 0,
            intensity: 0,
            shape: 0,
            motion: 0,
            parallax: 0,
            height_bias: 0,
            clustering: 0,
            color_a: 0,
            color_b: 0,
            axis_x: 0.0,
            axis_y: 1.0,
            axis_z: 0.0,
            phase: 0,
            seed: 0,
        }
    }

    pub fn starfield_calm() -> Self {
        Self {
            family: 0,
            variant: 0,
            density: 120,
            size_min: 2,
            size_max: 10,
            intensity: 200,
            shape: 220,
            motion: 64,
            parallax: 140,
            height_bias: 100,
            clustering: 40,
            color_a: 0xDDE6FFFF,
            color_b: 0xFFF2C0FF,
            axis_x: 0.0,
            axis_y: 1.0,
            axis_z: 0.0,
            phase: 0,
            seed: 0,
        }
    }
}

impl Default for CellsParams {
    fn default() -> Self {
        Self::starfield_calm()
    }
}

#[derive(Clone, Copy)]
pub struct LinesParams {
    pub variant: u32,
    pub line_type: u32,
    pub thickness: u32,
    pub spacing: f32,
    pub fade_distance: f32,
    pub parallax: u32,
    pub color_primary: u32,
    pub color_accent: u32,
    pub accent_every: u32,
    pub phase: u16,
    pub profile: u32,
    pub warp: u32,
    pub wobble: u32,
    pub glow: u32,
    pub axis_x: f32,
    pub axis_y: f32,
    pub axis_z: f32,
    pub seed: u32,
}

impl LinesParams {
    pub fn synth_grid() -> Self {
        Self {
            variant: 0,
            line_type: 2,
            thickness: 18,
            spacing: 2.25,
            fade_distance: 80.0,
            parallax: 0,
            color_primary: 0x00FFB0C0,
            color_accent: 0xFF3AF0FF,
            accent_every: 8,
            phase: 0,
            profile: 0,
            warp: 24,
            wobble: 0,
            glow: 96,
            axis_x: 0.0,
            axis_y: 0.0,
            axis_z: 1.0,
            seed: 0x4D2F5A10,
        }
    }
}

impl Default for LinesParams {
    fn default() -> Self {
        Self::synth_grid()
    }
}

#[derive(Clone, Copy)]
pub struct RingsParams {
    pub family: u32,
    pub ring_count: u32,
    pub thickness: u32,
    pub color_a: u32,
    pub color_b: u32,
    pub center_color: u32,
    pub center_falloff: u32,
    pub spiral_twist: f32,
    pub axis_x: f32,
    pub axis_y: f32,
    pub axis_z: f32,
    pub phase: u16,
    pub wobble: u16,
    pub noise: u32,
    pub dash: u32,
    pub glow: u32,
    pub seed: u32,
}

impl RingsParams {
    pub fn stargate_portal() -> Self {
        Self {
            family: 0,
            ring_count: 48,
            thickness: 28,
            color_a: 0x2EE7FFFF,
            color_b: 0x0B2B4CFF,
            center_color: 0xE8FFFFFF,
            center_falloff: 190,
            spiral_twist: 25.0,
            axis_x: 0.0,
            axis_y: 0.0,
            axis_z: 1.0,
            phase: 0,
            wobble: 9000,
            noise: 32,
            dash: 24,
            glow: 160,
            seed: 41,
        }
    }
}

impl Default for RingsParams {
    fn default() -> Self {
        Self::stargate_portal()
    }
}

/// Small environment state for examples.
#[derive(Clone, Copy)]
pub struct DebugEnvironment {
    pub base_mode: u32,
    pub overlay_mode: u32,
    pub blend_mode: u32,
    pub gradient: GradientParams,
    pub cells: CellsParams,
    pub lines: LinesParams,
    pub rings: RingsParams,
}

impl Default for DebugEnvironment {
    fn default() -> Self {
        Self {
            base_mode: env_mode::GRADIENT,
            overlay_mode: env_mode::CELLS,
            blend_mode: blend_mode::SCREEN,
            gradient: GradientParams::default(),
            cells: CellsParams::starfield_calm(),
            lines: LinesParams::synth_grid(),
            rings: RingsParams::stargate_portal(),
        }
    }
}

// =============================================================================
// EPU v2 Helper Functions (const-friendly)
// =============================================================================

// EPU Opcodes (v2 spec: 5-bit)
// Not all opcodes are currently used in presets, but they are available.
#[allow(dead_code)]
const OP_NOP: u64 = 0x00;
const OP_RAMP: u64 = 0x01;
const OP_LOBE: u64 = 0x02;
const OP_BAND: u64 = 0x03;
#[allow(dead_code)]
const OP_FOG: u64 = 0x04;
const OP_DECAL: u64 = 0x05;
const OP_GRID: u64 = 0x06;
const OP_SCATTER: u64 = 0x07;
#[allow(dead_code)]
const OP_FLOW: u64 = 0x08;

// Region masks (3-bit bitfield)
const REGION_ALL: u64 = 0b111;
const REGION_SKY: u64 = 0b100;
const REGION_WALLS: u64 = 0b010;
const REGION_FLOOR: u64 = 0b001;

// Blend modes (3-bit)
// Not all blend modes are currently used in presets, but they are available.
const BLEND_ADD: u64 = 0;
#[allow(dead_code)]
const BLEND_MULTIPLY: u64 = 1;
#[allow(dead_code)]
const BLEND_MAX: u64 = 2;
#[allow(dead_code)]
const BLEND_LERP: u64 = 3;
#[allow(dead_code)]
const BLEND_SCREEN: u64 = 4;

// Common directions (octahedral encoded: u8, v8)
const DIR_UP: u64 = 0x80FF; // +Y direction

/// Build v2 hi word: opcode(5), region(3), blend(3), emissive(4), reserved(1), color_a(24), color_b(24)
const fn epu_hi(
    opcode: u64,
    region: u64,
    blend: u64,
    emissive: u64,
    color_a: u64,
    color_b: u64,
) -> u64 {
    ((opcode & 0x1F) << 59)
        | ((region & 0x7) << 56)
        | ((blend & 0x7) << 53)
        | ((emissive & 0xF) << 49)
        | ((color_a & 0xFFFFFF) << 24)
        | (color_b & 0xFFFFFF)
}

/// Build v2 lo word: intensity(8), param_a(8), param_b(8), param_c(8), param_d(8), direction(16), alpha_a(4), alpha_b(4)
const fn epu_lo(
    intensity: u64,
    param_a: u64,
    param_b: u64,
    param_c: u64,
    param_d: u64,
    direction: u64,
    alpha_a: u64,
    alpha_b: u64,
) -> u64 {
    ((intensity & 0xFF) << 56)
        | ((param_a & 0xFF) << 48)
        | ((param_b & 0xFF) << 40)
        | ((param_c & 0xFF) << 32)
        | ((param_d & 0xFF) << 24)
        | ((direction & 0xFFFF) << 8)
        | ((alpha_a & 0xF) << 4)
        | (alpha_b & 0xF)
}

/// NOP layer (disabled)
const NOP_LAYER: [u64; 2] = [0, 0];

// =============================================================================
// EPU v2 Preset Configurations (128-bit per layer)
// =============================================================================

/// Simple blue sky gradient (RAMP only)
/// sky = light blue (0x6496DC), floor = dark green (0x285028)
static EPU_GRADIENT: [[u64; 2]; 8] = [
    // RAMP: sky/wall/floor gradient with blue sky and green ground
    [
        epu_hi(OP_RAMP, REGION_ALL, BLEND_ADD, 8, 0x6496DC, 0x285028),
        epu_lo(180, 200, 180, 0xA5, 0, DIR_UP, 15, 15),
    ],
    NOP_LAYER,
    NOP_LAYER,
    NOP_LAYER,
    NOP_LAYER,
    NOP_LAYER,
    NOP_LAYER,
    NOP_LAYER,
];

/// Starfield with scatter points (black void + white stars)
static EPU_CELLS: [[u64; 2]; 8] = [
    // RAMP: black void
    [
        epu_hi(OP_RAMP, REGION_ALL, BLEND_ADD, 0, 0x000008, 0x000010),
        epu_lo(20, 0, 0, 0xF0, 0, DIR_UP, 15, 15),
    ],
    NOP_LAYER,
    NOP_LAYER,
    NOP_LAYER,
    // SCATTER: white stars (emissive)
    [
        epu_hi(OP_SCATTER, REGION_ALL, BLEND_ADD, 15, 0xFFFFFF, 0xAABBFF),
        epu_lo(255, 180, 15, 0x83, 0, 0, 15, 10),
    ],
    // SCATTER: blue distant stars
    [
        epu_hi(OP_SCATTER, REGION_ALL, BLEND_ADD, 10, 0x8888FF, 0x4444AA),
        epu_lo(160, 250, 6, 0x41, 0, 0, 12, 8),
    ],
    NOP_LAYER,
    NOP_LAYER,
];

/// Grid lines pattern (dark background + neon grid)
static EPU_LINES: [[u64; 2]; 8] = [
    // RAMP: dark purple/blue background
    [
        epu_hi(OP_RAMP, REGION_ALL, BLEND_ADD, 0, 0x101030, 0x080818),
        epu_lo(80, 20, 20, 0xA5, 0, DIR_UP, 15, 15),
    ],
    NOP_LAYER,
    NOP_LAYER,
    NOP_LAYER,
    // GRID: cyan neon lines on floor
    [
        epu_hi(OP_GRID, REGION_FLOOR, BLEND_ADD, 12, 0x00FFAA, 0x008866),
        epu_lo(180, 48, 8, 0x00, 0, 0, 15, 10),
    ],
    // GRID: magenta accent on walls
    [
        epu_hi(OP_GRID, REGION_WALLS, BLEND_ADD, 10, 0xFF00FF, 0x880088),
        epu_lo(120, 32, 4, 0x00, 0, 0, 12, 8),
    ],
    NOP_LAYER,
    NOP_LAYER,
];

/// Rings/portal pattern (black void + blue glow + cyan band)
static EPU_RINGS: [[u64; 2]; 8] = [
    // RAMP: black void
    [
        epu_hi(OP_RAMP, REGION_ALL, BLEND_ADD, 0, 0x000008, 0x000010),
        epu_lo(20, 0, 0, 0xF0, 0, DIR_UP, 15, 15),
    ],
    // LOBE: blue glow (emissive)
    [
        epu_hi(OP_LOBE, REGION_ALL, BLEND_ADD, 14, 0x2288FF, 0x0044AA),
        epu_lo(200, 28, 0, 0, 0, 0x80C0, 15, 12),
    ],
    // BAND: cyan accent ring
    [
        epu_hi(OP_BAND, REGION_ALL, BLEND_ADD, 10, 0x00FFFF, 0x008888),
        epu_lo(160, 0x20, 0x10, 0, 0, DIR_UP, 15, 10),
    ],
    NOP_LAYER,
    // DECAL: bright center disk
    [
        epu_hi(OP_DECAL, REGION_SKY, BLEND_ADD, 15, 0xFFFFFF, 0x88DDFF),
        epu_lo(255, 0x02, 16, 0, 0, 0x8080, 15, 15),
    ],
    NOP_LAYER,
    NOP_LAYER,
    NOP_LAYER,
];

impl DebugEnvironment {
    /// Advance loop phases (call in `update()`).
    pub fn tick(&mut self, delta_speed: f32) {
        let delta = (delta_speed * 100.0) as u16;
        self.cells.phase = self.cells.phase.wrapping_add(delta);
        self.lines.phase = self.lines.phase.wrapping_add(delta);
        self.rings.phase = self.rings.phase.wrapping_add(delta);
    }

    /// Apply environment settings using EPU (call in `render()` before `epu_draw()`).
    ///
    /// This implementation uses pre-built EPU v2 configurations (128-bit per layer)
    /// that approximate the legacy env_* modes. For full control, use epu_set() directly.
    pub fn apply(&self) {
        unsafe {
            // Select EPU preset based on base_mode
            // v2 format: [[u64; 2]; 8] = 128 bytes (8 x 128-bit layers)
            let preset: &[[u64; 2]; 8] = match self.base_mode {
                env_mode::GRADIENT => &EPU_GRADIENT,
                env_mode::CELLS => &EPU_CELLS,
                env_mode::LINES => &EPU_LINES,
                env_mode::RINGS => &EPU_RINGS,
                _ => &EPU_GRADIENT, // Default to gradient for unsupported modes
            };
            // Cast to *const u64 for FFI (the memory layout is contiguous)
            epu_set(0, preset.as_ptr() as *const u64);
        }
    }

    /// Apply and draw the environment
    pub fn apply_and_draw(&self) {
        self.apply();
        unsafe {
            epu_draw(0);
        }
    }
}
