//! Debug Shapes - Shape cycling and management
//!
//! Provides shape generation and cycling for inspector examples.

use crate::ffi::*;

/// Shape type enum
#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum ShapeType {
    Sphere = 0,
    Cube = 1,
    Torus = 2,
}

impl ShapeType {
    /// Get shape from index (clamped)
    pub fn from_index(index: i32) -> Self {
        match index {
            0 => ShapeType::Sphere,
            1 => ShapeType::Cube,
            2 => ShapeType::Torus,
            _ => ShapeType::Sphere,
        }
    }

    /// Get display name
    pub fn name(&self) -> &'static str {
        match self {
            ShapeType::Sphere => "Sphere",
            ShapeType::Cube => "Cube",
            ShapeType::Torus => "Torus",
        }
    }

    /// Number of available shapes
    pub const COUNT: i32 = 3;
}

/// Shape mesh handles
pub struct ShapeMeshes {
    pub sphere: u32,
    pub cube: u32,
    pub torus: u32,
}

impl ShapeMeshes {
    /// Generate all shape meshes (call in init())
    pub fn generate() -> Self {
        unsafe {
            Self {
                sphere: sphere(1.5, 32, 16),
                cube: cube(1.2, 1.2, 1.2),
                torus: torus(1.2, 0.5, 32, 16),
            }
        }
    }

    /// Get mesh handle for shape type
    pub fn get(&self, shape: ShapeType) -> u32 {
        match shape {
            ShapeType::Sphere => self.sphere,
            ShapeType::Cube => self.cube,
            ShapeType::Torus => self.torus,
        }
    }

    /// Get mesh handle by index
    pub fn get_by_index(&self, index: i32) -> u32 {
        self.get(ShapeType::from_index(index))
    }
}

/// Shape rotation state
pub struct ShapeRotation {
    pub x: f32,
    pub y: f32,
    pub speed: f32, // degrees per second
}

impl Default for ShapeRotation {
    fn default() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            speed: 30.0,
        }
    }
}

impl ShapeRotation {
    /// Update rotation based on input (call in update())
    pub fn update(&mut self) {
        unsafe {
            let stick_x = left_stick_x(0);
            let stick_y = left_stick_y(0);

            if stick_x.abs() > 0.1 || stick_y.abs() > 0.1 {
                self.y += stick_x * 3.0;
                self.x += stick_y * 3.0;
            } else {
                // Auto-rotate
                self.y += self.speed * (1.0 / 60.0);
            }

            // Wrap angles
            if self.y >= 360.0 {
                self.y -= 360.0;
            }
            if self.x >= 360.0 {
                self.x -= 360.0;
            }
        }
    }

    /// Apply rotation transform (call before draw_mesh)
    pub fn apply(&self) {
        unsafe {
            push_identity();
            push_rotate_y(self.y);
            push_rotate_x(self.x);
        }
    }
}

/// Handle shape cycling with A button (call in update())
/// Returns true if shape changed
pub fn handle_shape_cycling(shape_index: &mut i32) -> bool {
    unsafe {
        if button_pressed(0, BUTTON_A) != 0 {
            *shape_index = (*shape_index + 1) % ShapeType::COUNT;
            true
        } else {
            false
        }
    }
}

/// Clamp shape index to valid range
pub fn clamp_shape_index(index: &mut i32) {
    if *index < 0 {
        *index = 0;
    }
    if *index >= ShapeType::COUNT {
        *index = ShapeType::COUNT - 1;
    }
}

/// Register shape debug values
pub unsafe fn register_shape_debug(
    shape_index: *const i32,
    rotation_speed: *const f32,
    color: *const u8,
) {
    debug_group_begin(b"shape".as_ptr(), 5);
    debug_register_i32(b"index (0-2)".as_ptr(), 11, shape_index);
    debug_register_f32(b"rotation_speed".as_ptr(), 14, rotation_speed);
    debug_register_color(b"color".as_ptr(), 5, color);
    debug_group_end();
}
