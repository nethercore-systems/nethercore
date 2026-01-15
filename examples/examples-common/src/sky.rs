//! Debug Sky - Sky and sun controls with debug integration
//!
//! Provides sky gradient and sun configuration with debug inspector integration.

use crate::ffi::*;
use crate::color;

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
            horizon: 0xFF804DFF,  // Orange
            zenith: 0x4D1A80FF,   // Purple
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
            horizon: 0x0D0D1AFF,  // Very dark blue
            zenith: 0x03030DFF,   // Almost black
            sun_dir_x: 0.0,
            sun_dir_y: -1.0,
            sun_dir_z: 0.0,
            sun_color: 0x1A1A26FF, // Dim moon
            sun_sharpness: 0.5,
        }
    }

    /// Apply sky settings and draw (call in render())
    pub fn apply_and_draw(&self) {
        unsafe {
            // Use env_gradient for 2-color sky (zenith -> horizon for sky, horizon -> same for ground)
            env_gradient(
                0,
                self.zenith,
                self.horizon,
                self.horizon,
                self.zenith,
                0.0, // sun azimuth
                0.0, // horizon shift
                0.0, // sun elevation
                0,   // sun disk
                0,   // sun halo
                0,   // sun intensity (disabled)
                0,   // horizon haze
                0,   // sun warmth
                0,   // cloudiness
            );
            // Use new lighting API
            light_set(0, self.sun_dir_x, self.sun_dir_y, self.sun_dir_z);
            light_color(0, self.sun_color);
            light_intensity(0, 1.0);
            draw_env();
        }
    }

    /// Just apply settings without drawing (for lighting calculations)
    pub fn apply(&self) {
        unsafe {
            // Use env_gradient for 2-color sky (zenith -> horizon for sky, horizon -> same for ground)
            env_gradient(
                0,
                self.zenith,
                self.horizon,
                self.horizon,
                self.zenith,
                0.0, // sun azimuth
                0.0, // horizon shift
                0.0, // sun elevation
                0,   // sun disk
                0,   // sun halo
                0,   // sun intensity (disabled)
                0,   // horizon haze
                0,   // sun warmth
                0,   // cloudiness
            );
            // Use new lighting API
            light_set(0, self.sun_dir_x, self.sun_dir_y, self.sun_dir_z);
            light_color(0, self.sun_color);
            light_intensity(0, 1.0);
        }
    }
}

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
