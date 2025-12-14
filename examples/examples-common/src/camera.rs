//! Debug Camera - Orbiting camera with debug controls
//!
//! Provides an orbiting camera around a target point with:
//! - Auto-orbit when idle
//! - Manual orbit control via right stick
//! - Distance adjustment via triggers

use crate::ffi::*;
use libm::{cosf, sinf};

/// Which stick controls camera orbit
#[derive(Copy, Clone)]
pub enum StickControl {
    LeftStick,   // For animation examples
    RightStick,  // Default for inspector examples
}

/// Camera state for debug orbiting
pub struct DebugCamera {
    /// Target point the camera looks at
    pub target_x: f32,
    pub target_y: f32,
    pub target_z: f32,

    /// Orbit parameters
    pub distance: f32,
    pub elevation: f32, // degrees above horizon
    pub azimuth: f32,   // degrees around Y axis

    /// Auto-orbit speed (degrees per second, 0 to disable)
    pub auto_orbit_speed: f32,

    /// Which stick controls camera orbit (default: RightStick)
    pub stick_control: StickControl,

    /// Field of view
    pub fov: f32,
}

impl Default for DebugCamera {
    fn default() -> Self {
        Self {
            target_x: 0.0,
            target_y: 0.0,
            target_z: 0.0,
            distance: 5.0,
            elevation: 20.0,
            azimuth: 0.0,
            auto_orbit_speed: 15.0,
            stick_control: StickControl::RightStick,
            fov: 60.0,
        }
    }
}

impl DebugCamera {
    /// Create a new debug camera with custom settings
    pub fn new(distance: f32, elevation: f32) -> Self {
        Self {
            distance,
            elevation,
            ..Default::default()
        }
    }

    /// Create camera with left stick control (for animation examples)
    pub fn new_left_stick(distance: f32, elevation: f32) -> Self {
        Self {
            distance,
            elevation,
            stick_control: StickControl::LeftStick,
            ..Default::default()
        }
    }

    /// Update camera based on input (call in update())
    pub fn update(&mut self) {
        unsafe {
            // Read stick based on configuration
            let (stick_x, stick_y) = match self.stick_control {
                StickControl::LeftStick => (left_stick_x(0), left_stick_y(0)),
                StickControl::RightStick => (right_stick_x(0), right_stick_y(0)),
            };

            if stick_x.abs() > 0.1 || stick_y.abs() > 0.1 {
                self.azimuth += stick_x * 2.0;
                self.elevation -= stick_y * 2.0;
                self.elevation = self.elevation.clamp(-80.0, 80.0);
            } else if self.auto_orbit_speed > 0.0 {
                // Auto-orbit when stick is centered
                self.azimuth += self.auto_orbit_speed * (1.0 / 60.0);
            }

            // Wrap azimuth
            if self.azimuth >= 360.0 {
                self.azimuth -= 360.0;
            }
            if self.azimuth < 0.0 {
                self.azimuth += 360.0;
            }

            // Triggers for distance (optional)
            let lt = trigger_left(0);
            let rt = trigger_right(0);
            if lt > 0.1 {
                self.distance += lt * 0.1;
            }
            if rt > 0.1 {
                self.distance -= rt * 0.1;
            }
            self.distance = self.distance.clamp(2.0, 20.0);
        }
    }

    /// Apply camera transform (call at start of render())
    pub fn apply(&self) {
        let azimuth_rad = self.azimuth * core::f32::consts::PI / 180.0;
        let elevation_rad = self.elevation * core::f32::consts::PI / 180.0;

        // Calculate camera position on sphere around target
        let horizontal_dist = self.distance * cosf(elevation_rad);
        let cam_x = self.target_x + horizontal_dist * sinf(azimuth_rad);
        let cam_y = self.target_y + self.distance * sinf(elevation_rad);
        let cam_z = self.target_z + horizontal_dist * cosf(azimuth_rad);

        unsafe {
            camera_set(cam_x, cam_y, cam_z, self.target_x, self.target_y, self.target_z);
            camera_fov(self.fov);
        }
    }

    /// Register debug values for the camera
    pub fn register_debug(&self, distance: &f32, elevation: &f32, azimuth: &f32, auto_speed: &f32) {
        unsafe {
            debug_group_begin(b"camera".as_ptr(), 6);
            debug_register_f32(b"distance".as_ptr(), 8, distance);
            debug_register_f32(b"elevation".as_ptr(), 9, elevation);
            debug_register_f32(b"azimuth".as_ptr(), 7, azimuth);
            debug_register_f32(b"auto_orbit".as_ptr(), 10, auto_speed);
            debug_group_end();
        }
    }
}
