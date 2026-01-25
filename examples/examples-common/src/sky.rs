//! Debug Sky - Sky and sun controls with debug integration
//!
//! Provides sky gradient and sun configuration with debug inspector integration.
//! Now uses the EPU (Environment Processing Unit) API with 128-bit instructions.
//!
//! # EPU Format
//!
//! Each layer is 128 bits (2 x u64) with direct RGB colors.
//! See `docs/book/src/api/epu.md` and `include/zx.rs` for the canonical layout.

use crate::color;
use crate::ffi::*;

/// Sky state for debug control
#[derive(Clone, Copy)]
pub struct DebugSky {
    /// Horizon color (u32 RGBA)
    pub horizon: u32,
    /// Zenith color (u32 RGBA)
    pub zenith: u32,
    /// Sun direction (will be normalized)
    pub sun_dir_x: f32,
    pub sun_dir_y: f32,
    pub sun_dir_z: f32,
    /// Sun color (u32 RGBA)
    pub sun_color: u32,
    /// Sun disc sharpness (0-1)
    pub sun_sharpness: f32,
}

impl Default for DebugSky {
    fn default() -> Self {
        Self {
            // Default: pleasant blue sky
            horizon: color::SKY_HORIZON,
            zenith: color::SKY_ZENITH,
            sun_dir_x: 0.5,
            sun_dir_y: -0.7,
            sun_dir_z: 0.5,
            sun_color: color::SUN_DEFAULT,
            sun_sharpness: 0.98,
        }
    }
}

impl DebugSky {
    /// Create a new sky with custom colors
    pub fn new(horizon: u32, zenith: u32) -> Self {
        Self {
            horizon,
            zenith,
            ..Default::default()
        }
    }

    /// Create a sunset sky preset
    pub fn sunset() -> Self {
        Self {
            horizon: 0xFF804DFF, // Orange
            zenith: 0x4D1A80FF,  // Purple
            sun_dir_x: 0.8,
            sun_dir_y: -0.2,
            sun_dir_z: 0.0,
            sun_color: 0xFFE673FF, // Golden
            sun_sharpness: 0.95,
        }
    }

    /// Create a night sky preset
    pub fn night() -> Self {
        Self {
            horizon: 0x0D0D1AFF, // Very dark blue
            zenith: 0x03030DFF,  // Almost black
            sun_dir_x: 0.0,
            sun_dir_y: -1.0,
            sun_dir_z: 0.0,
            sun_color: 0x1A1A26FF, // Dim moon
            sun_sharpness: 0.5,
        }
    }

    /// Apply sky settings and draw (call in render()).
    ///
    /// Uses the EPU API with a simple RAMP layer for sky gradient.
    pub fn apply_and_draw(&self) {
        // Use the shared EPU_SKY preset
        unsafe {
            // Use lighting API
            light_set(0, self.sun_dir_x, self.sun_dir_y, self.sun_dir_z);
            light_color(0, self.sun_color);
            light_intensity(0, 1.0);

            epu_set(EPU_SKY.as_ptr() as *const u64);
            draw_epu();
        }
    }

    /// Apply settings (does not draw environment).
    pub fn apply(&self) {
        unsafe {
            // Use lighting API
            light_set(0, self.sun_dir_x, self.sun_dir_y, self.sun_dir_z);
            light_color(0, self.sun_color);
            light_intensity(0, 1.0);

            epu_set(EPU_SKY.as_ptr() as *const u64);
        }
    }

    /// Draw the environment background for the current viewport/pass.
    pub fn draw(&self) {
        unsafe { draw_epu() }
    }
}

// =============================================================================
// EPU Preset for DebugSky
// =============================================================================

// EPU helper functions (shared by examples)
const fn epu_hi(
    opcode: u64,
    region: u64,
    blend: u64,
    meta5: u64,
    color_a: u64,
    color_b: u64,
) -> u64 {
    ((opcode & 0x1F) << 59)
        | ((region & 0x7) << 56)
        | ((blend & 0x7) << 53)
        | ((meta5 & 0x1F) << 48)
        | ((color_a & 0xFFFFFF) << 24)
        | (color_b & 0xFFFFFF)
}

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

// Opcodes
const OP_RAMP: u64 = 0x01;

// Region masks
const REGION_ALL: u64 = 0b111;

// Blend modes
const BLEND_ADD: u64 = 0;

// Direction for +Y (up)
const DIR_UP: u64 = 0x80FF;

/// EPU sky preset: blue sky (0x6496DC) to green ground (0x285028)
/// 8 x 128-bit layers, each as [hi, lo]
static EPU_SKY: [[u64; 2]; 8] = [
    // Layer 0: RAMP gradient
    [
        epu_hi(OP_RAMP, REGION_ALL, BLEND_ADD, 0, 0x6496DC, 0x285028),
        epu_lo(180, 200, 180, 0xA5, 0, DIR_UP, 15, 15),
    ],
    [0, 0], // NOP
    [0, 0], // NOP
    [0, 0], // NOP
    [0, 0], // NOP
    [0, 0], // NOP
    [0, 0], // NOP
    [0, 0], // NOP
];

/// Register sky debug values
///
/// Call this in init() with pointers to your static sky state
pub unsafe fn register_sky_debug(
    horizon: *const u8,
    zenith: *const u8,
    sun_dir_x: *const f32,
    sun_dir_y: *const f32,
    sun_dir_z: *const f32,
    sun_color: *const u8,
    sun_sharpness: *const f32,
) {
    debug_group_begin(b"sky".as_ptr(), 3);
    debug_register_color(b"horizon".as_ptr(), 7, horizon);
    debug_register_color(b"zenith".as_ptr(), 6, zenith);
    debug_group_end();

    debug_group_begin(b"sun".as_ptr(), 3);
    debug_register_f32(b"dir_x".as_ptr(), 5, sun_dir_x as *const u8);
    debug_register_f32(b"dir_y".as_ptr(), 5, sun_dir_y as *const u8);
    debug_register_f32(b"dir_z".as_ptr(), 5, sun_dir_z as *const u8);
    debug_register_color(b"color".as_ptr(), 5, sun_color);
    debug_register_f32(b"sharpness".as_ptr(), 9, sun_sharpness as *const u8);
    debug_group_end();
}
