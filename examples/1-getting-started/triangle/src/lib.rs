//! Triangle Example
//!
//! Minimal example demonstrating immediate mode 3D drawing with `draw_triangles`.
//! A colorful triangle spins around the Y axis.
//!
//! Note: Rollback state is automatic (entire WASM memory is snapshotted). No save_state/load_state needed.

#![no_std]
#![no_main]

use core::panic::PanicInfo;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    // Trigger a WASM trap so runtime can catch the error
    // instead of infinite loop which freezes the game
    core::arch::wasm32::unreachable()
}

// Import the canonical FFI bindings
#[path = "../../../../include/zx.rs"]
mod ffi;
use ffi::*;

/// Triangle vertices: 3 vertices × 6 floats = 18 floats
/// Colors: red, green, blue at each corner
static TRIANGLE: [f32; 18] = [
    // Vertex 0: top (red)
    0.0, 1.0, 0.0, // position
    1.0, 0.0, 0.0, // color (red)
    // Vertex 1: bottom-left (green)
    -0.866, -0.5, 0.0, // position
    0.0, 1.0, 0.0,     // color (green)
    // Vertex 2: bottom-right (blue)
    0.866, -0.5, 0.0, // position
    0.0, 0.0, 1.0,    // color (blue)
];

#[no_mangle]
pub extern "C" fn init() {
    unsafe {
        // Dark blue-gray background
        set_clear_color(0x1a1a2eFF);
    }
}

#[no_mangle]
pub extern "C" fn update() {
    // No game state to update - animation is driven by elapsed_time in render
}

#[no_mangle]
pub extern "C" fn render() {
    unsafe {
        // Set camera every frame (immediate mode)
        camera_set(0.0, 0.0, 3.0, 0.0, 0.0, 0.0);

        // Rotate triangle around Y axis based on elapsed time
        push_identity();
        let time = elapsed_time();
        let rotation_speed = 45.0; // degrees per second
        push_rotate(time * rotation_speed, 0.0, 1.0, 0.0);

        // Draw the triangle with POS_COLOR format
        draw_triangles(TRIANGLE.as_ptr(), 3, format::POS_COLOR as u32);
    }
}
