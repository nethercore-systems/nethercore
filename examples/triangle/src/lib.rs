//! Triangle Example
//!
//! Minimal example demonstrating immediate mode 3D drawing with `draw_triangles`.
//! A colorful triangle spins around the Y axis.

#![no_std]
#![no_main]

use core::panic::PanicInfo;

#[panic_handler]
fn panic(_: &PanicInfo) -> ! {
    loop {}
}

#[link(wasm_import_module = "env")]
extern "C" {
    fn set_clear_color(color: u32);
    fn elapsed_time() -> f32;
    fn camera_set(x: f32, y: f32, z: f32, target_x: f32, target_y: f32, target_z: f32);
    fn transform_identity();
    fn transform_rotate(angle_deg: f32, x: f32, y: f32, z: f32);
    fn draw_triangles(data: *const f32, vertex_count: u32, format: u32);
}

/// Vertex format: POS_COLOR = 2
/// Each vertex: position (3 floats) + color (3 floats) = 6 floats
const FORMAT_POS_COLOR: u32 = 2;

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
        // Position camera to view the triangle
        camera_set(0.0, 0.0, 3.0, 0.0, 0.0, 0.0);
    }
}

#[no_mangle]
pub extern "C" fn update() {
    // No game state to update - animation is driven by elapsed_time in render
}

#[no_mangle]
pub extern "C" fn render() {
    unsafe {
        // Reset transform to identity
        transform_identity();

        // Rotate around Y axis based on elapsed time (60 degrees per second)
        let angle = elapsed_time() * 60.0;
        transform_rotate(angle, 0.0, 1.0, 0.0);

        // Draw the triangle with POS_COLOR format
        draw_triangles(TRIANGLE.as_ptr(), 3, FORMAT_POS_COLOR);
    }
}
