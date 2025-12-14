//! Textured Procedural Shapes Example
//!
//! Demonstrates UV-enabled procedural mesh generation with texture mapping.
//! Uses a colorful UV debug texture to show how UVs are mapped on each shape.
//!
//! Controls:
//! - A button: Cycle through shapes
//! - Left stick: Rotate shape
//! - Auto-rotates for visual inspection
//!

#![no_std]
#![no_main]

use core::panic::PanicInfo;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    core::arch::wasm32::unreachable()
}

#[link(wasm_import_module = "env")]
extern "C" {
    // Configuration
    fn set_clear_color(color: u32);
    fn render_mode(mode: u32);

    // Camera
    fn camera_set(x: f32, y: f32, z: f32, target_x: f32, target_y: f32, target_z: f32);
    fn camera_fov(fov_degrees: f32);

    // Input
    fn button_held(player: u32, button: u32) -> u32;
    fn left_stick_x(player: u32) -> f32;
    fn left_stick_y(player: u32) -> f32;

    // Textures
    fn load_texture(width: u32, height: u32, pixels: *const u8) -> u32;
    fn texture_bind(handle: u32);
    fn texture_filter(filter: u32);

    // UV-enabled procedural mesh generation
    fn sphere_uv(radius: f32, segments: u32, rings: u32) -> u32;
    fn plane_uv(size_x: f32, size_z: f32, subdivisions_x: u32, subdivisions_z: u32) -> u32;
    fn cube_uv(size_x: f32, size_y: f32, size_z: f32) -> u32;
    fn cylinder_uv(radius_bottom: f32, radius_top: f32, height: f32, segments: u32) -> u32;
    fn torus_uv(major_radius: f32, minor_radius: f32, major_segments: u32, minor_segments: u32) -> u32;
    fn capsule_uv(radius: f32, height: f32, segments: u32, rings: u32) -> u32;

    // Mesh drawing
    fn draw_mesh(handle: u32);

    // Transform
    fn push_identity();
    fn push_rotate_x(angle_deg: f32);
    fn push_rotate_y(angle_deg: f32);

    // Render state
    fn set_color(color: u32);
    fn depth_test(enabled: u32);

    // 2D drawing for UI
    fn draw_text(ptr: *const u8, len: u32, x: f32, y: f32, size: f32, color: u32);
}

// Input button constants
const BUTTON_A: u32 = 1;

/// Current shape index (0-5)
static mut CURRENT_SHAPE: u32 = 0;

/// Mesh handles for all 6 UV-enabled shapes
static mut MESH_HANDLES: [u32; 6] = [0; 6];

/// Texture handle for UV debug texture
static mut TEXTURE_HANDLE: u32 = 0;

/// Current rotation angles
static mut ROTATION_X: f32 = 0.0;
static mut ROTATION_Y: f32 = 0.0;

/// Previous A button state (for edge detection)
static mut PREV_A_BUTTON: u32 = 0;

/// Shape names for UI display
static SHAPE_NAMES: [&str; 6] = [
    "Sphere (UV equirectangular)",
    "Plane (UV grid)",
    "Cube (UV box unwrap)",
    "Cylinder (UV cylindrical)",
    "Torus (UV wrapped)",
    "Capsule (UV hybrid)",
];

/// Generate a colorful UV debug texture (64x64)
/// - Red channel increases left to right (U axis)
/// - Green channel increases bottom to top (V axis)
/// - Blue channel is a checker pattern
fn generate_uv_debug_texture() -> [u8; 64 * 64 * 4] {
    let mut pixels = [0u8; 64 * 64 * 4];
    let checker_size = 8; // 8x8 pixel checkers

    for y in 0..64 {
        for x in 0..64 {
            let idx = (y * 64 + x) * 4;

            // Red increases left to right (U)
            let r = ((x * 255) / 63) as u8;

            // Green increases bottom to top (V)
            let g = ((y * 255) / 63) as u8;

            // Blue is a checker pattern
            let checker_x = x / checker_size;
            let checker_y = y / checker_size;
            let is_checker = (checker_x + checker_y) % 2 == 0;
            let b = if is_checker { 255 } else { 64 };

            pixels[idx] = r;
            pixels[idx + 1] = g;
            pixels[idx + 2] = b;
            pixels[idx + 3] = 255; // Alpha
        }
    }

    pixels
}

#[no_mangle]
pub extern "C" fn init() {
    unsafe {
        // Configure rendering
        set_clear_color(0x1a1a2eFF); // Dark blue
        render_mode(0); // Unlit (textured)
        depth_test(1); // Enable depth testing

        // Generate and load UV debug texture
        let texture_pixels = generate_uv_debug_texture();
        TEXTURE_HANDLE = load_texture(64, 64, texture_pixels.as_ptr());

        // Generate all 6 UV-enabled procedural shapes
        MESH_HANDLES[0] = sphere_uv(1.5, 32, 16); // High-quality sphere
        MESH_HANDLES[1] = plane_uv(3.0, 3.0, 8, 8); // Subdivided plane
        MESH_HANDLES[2] = cube_uv(1.0, 1.0, 1.0); // 2×2×2 cube
        MESH_HANDLES[3] = cylinder_uv(1.0, 1.0, 2.0, 24); // Uniform cylinder
        MESH_HANDLES[4] = torus_uv(1.5, 0.5, 32, 16); // Donut
        MESH_HANDLES[5] = capsule_uv(0.8, 2.0, 24, 8); // Pill shape
    }
}

#[no_mangle]
pub extern "C" fn update() {
    unsafe {
        // Shape cycling (press A to change shape)
        let a_button = button_held(0, BUTTON_A);
        if a_button != 0 && PREV_A_BUTTON == 0 {
            // Button just pressed
            CURRENT_SHAPE = (CURRENT_SHAPE + 1) % 6;
        }
        PREV_A_BUTTON = a_button;

        // Rotation control (left stick)
        let stick_x = left_stick_x(0);
        let stick_y = left_stick_y(0);

        if stick_x.abs() > 0.1 || stick_y.abs() > 0.1 {
            ROTATION_Y += stick_x * 2.0;
            ROTATION_X += stick_y * 2.0;
        } else {
            // Auto-rotate when idle
            ROTATION_Y += 0.5;
            ROTATION_X += 0.3;
        }
    }
}

#[no_mangle]
pub extern "C" fn render() {
    unsafe {
        // Set camera every frame (immediate mode)
        camera_set(0.0, 3.0, 8.0, 0.0, 0.0, 0.0);
        camera_fov(60.0);

        // Bind UV debug texture
        texture_bind(TEXTURE_HANDLE);

        // Draw current shape
        push_identity();
        push_rotate_y(ROTATION_Y);
        push_rotate_x(ROTATION_X);

        // Special positioning for plane (tilt to be visible from camera)
        if CURRENT_SHAPE == 1 {
            push_rotate_x(-45.0); // Additional tilt for plane
        }

        set_color(0xFFFFFFFF); // White (no tint)
        draw_mesh(MESH_HANDLES[CURRENT_SHAPE as usize]);

        // Draw UI - shape name
        let shape_name = SHAPE_NAMES[CURRENT_SHAPE as usize];
        draw_text(
            shape_name.as_ptr(),
            shape_name.len() as u32,
            10.0,
            10.0,
            24.0,
            0xFFFFFFFF,
        );

        // Draw instruction
        let instruction = "Press A to cycle shapes | Left stick to rotate";
        draw_text(
            instruction.as_ptr(),
            instruction.len() as u32,
            10.0,
            40.0,
            16.0,
            0xAAAAAAFF,
        );

        // Draw UV mapping info
        let uv_info = "UV Debug Texture: Red=U, Green=V, Blue=Checker";
        draw_text(
            uv_info.as_ptr(),
            uv_info.len() as u32,
            10.0,
            70.0,
            14.0,
            0x88FF88FF,
        );
    }
}
