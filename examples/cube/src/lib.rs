//! Cube Example
//!
//! Demonstrates the procedural cube() function and retained mode 3D drawing.
//! A textured cube rotates based on analog stick input.
//!
//! Features:
//! - `cube()` to procedurally generate a cube mesh
//! - `draw_mesh()` to render the retained mesh
//! - Camera setup with `camera_set()` and `camera_fov()`
//! - Interactive rotation via analog stick
//! - Mode 0 with normals for simple Lambert shading
//! - Procedural sky for lighting
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

#[link(wasm_import_module = "env")]
extern "C" {
    // Configuration
    fn set_clear_color(color: u32);

    // Camera
    fn camera_set(x: f32, y: f32, z: f32, target_x: f32, target_y: f32, target_z: f32);
    fn camera_fov(fov_degrees: f32);

    // Input
    fn left_stick_x(player: u32) -> f32;
    fn left_stick_y(player: u32) -> f32;

    // Textures
    fn load_texture(width: u32, height: u32, pixels: *const u8) -> u32;
    fn texture_bind(handle: u32);
    fn texture_filter(filter: u32);

    // Procedural mesh generation
    fn cube(size_x: f32, size_y: f32, size_z: f32) -> u32;

    // Mesh drawing
    fn draw_mesh(handle: u32);

    // Transform
    fn push_identity();
    fn push_rotate_x(angle_deg: f32);
    fn push_rotate_y(angle_deg: f32);

    // Render state
    fn set_color(color: u32);
    fn depth_test(enabled: u32);
}

/// Cube mesh handle
static mut CUBE_MESH: u32 = 0;

/// Texture handle
static mut TEXTURE: u32 = 0;

/// Current rotation angles (degrees)
static mut ROTATION_X: f32 = 0.0;
static mut ROTATION_Y: f32 = 0.0;

/// 8x8 checkerboard texture (RGBA8)
const CHECKERBOARD: [u8; 8 * 8 * 4] = {
    let mut pixels = [0u8; 256];
    let white = [0xFF, 0xFF, 0xFF, 0xFF];
    let gray = [0x80, 0x80, 0x80, 0xFF];

    let mut y = 0;
    while y < 8 {
        let mut x = 0;
        while x < 8 {
            let idx = (y * 8 + x) * 4;
            let color = if (x + y) % 2 == 0 { white } else { gray };
            pixels[idx] = color[0];
            pixels[idx + 1] = color[1];
            pixels[idx + 2] = color[2];
            pixels[idx + 3] = color[3];
            x += 1;
        }
        y += 1;
    }
    pixels
};

#[no_mangle]
pub extern "C" fn init() {
    unsafe {
        // Dark blue background
        set_clear_color(0x1a1a2eFF);

        // Note: Sky uses reasonable defaults (blue gradient with sun) from the renderer
        // No need to set sky explicitly unless you want custom sky settings

        // Set up camera
        camera_set(0.0, 0.0, 5.0, 0.0, 0.0, 0.0);
        camera_fov(60.0);

        // Enable depth testing for proper 3D rendering
        depth_test(1);

        // Load checkerboard texture
        TEXTURE = load_texture(8, 8, CHECKERBOARD.as_ptr());
        texture_filter(0); // Nearest neighbor for crisp pixels

        // Generate cube mesh procedurally (2×2×2 cube)
        CUBE_MESH = cube(1.0, 1.0, 1.0);
    }
}

#[no_mangle]
pub extern "C" fn update() {
    unsafe {
        // Read analog stick input for rotation
        let stick_x = left_stick_x(0);
        let stick_y = left_stick_y(0);

        // Rotate based on stick input (90 degrees per second at full deflection)
        ROTATION_Y += stick_x * 2.0;
        ROTATION_X += stick_y * 2.0;

        // Also slowly auto-rotate when stick is centered
        if stick_x.abs() < 0.1 && stick_y.abs() < 0.1 {
            ROTATION_Y += 0.5;
            ROTATION_X += 0.3;
        }
    }
}

#[no_mangle]
pub extern "C" fn render() {
    unsafe {
        // Apply rotation accumulated in update()
        push_identity();
        push_rotate_y(ROTATION_Y);
        push_rotate_x(ROTATION_X);

        // Bind texture and set color
        texture_bind(TEXTURE);
        set_color(0xFFFFFFFF); // White (no tint)

        // Draw the cube
        draw_mesh(CUBE_MESH);
    }
}
