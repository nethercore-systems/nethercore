//! Procedural Shapes Example
//!
//! Demonstrates all 6 procedural mesh generation functions:
//! - cube() — Box with flat normals
//! - sphere() — UV sphere with smooth normals
//! - cylinder() — Cylinder with caps
//! - cylinder() (cone variant) — Tapered cylinder
//! - plane() — Subdivided ground plane
//! - torus() — Donut shape
//! - capsule() — Pill shape
//!
//! Controls:
//! - A button: Cycle through shapes
//! - Left stick: Rotate shape
//! - Auto-rotates for visual inspection
//!
//! Features checkerboard texture to verify UV mapping and proper lighting for normals.

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

    // Procedural mesh generation
    fn cube(size_x: f32, size_y: f32, size_z: f32) -> u32;
    fn sphere(radius: f32, segments: u32, rings: u32) -> u32;
    fn cylinder(radius_bottom: f32, radius_top: f32, height: f32, segments: u32) -> u32;
    fn plane(size_x: f32, size_z: f32, subdivisions_x: u32, subdivisions_z: u32) -> u32;
    fn torus(major_radius: f32, minor_radius: f32, major_segments: u32, minor_segments: u32) -> u32;
    fn capsule(radius: f32, height: f32, segments: u32, rings: u32) -> u32;

    // Mesh drawing
    fn draw_mesh(handle: u32);

    // Transform
    fn push_identity();
    fn push_rotate_x(angle_deg: f32);
    fn push_rotate_y(angle_deg: f32);

    // Render state
    fn set_color(color: u32);
    fn depth_test(enabled: u32);

    // Sky (for lighting)
    fn set_sky(
        horizon_r: f32,
        horizon_g: f32,
        horizon_b: f32,
        zenith_r: f32,
        zenith_g: f32,
        zenith_b: f32,
        sun_dir_x: f32,
        sun_dir_y: f32,
        sun_dir_z: f32,
        sun_r: f32,
        sun_g: f32,
        sun_b: f32,
        sun_sharpness: f32,
    );

    // 2D drawing for UI
    fn draw_text(ptr: *const u8, len: u32, x: f32, y: f32, size: f32, color: u32);
}

// Input button constants
const BUTTON_A: u32 = 1;

/// Current shape index (0-6)
static mut CURRENT_SHAPE: u32 = 0;

/// Mesh handles for all 7 shapes
static mut MESH_HANDLES: [u32; 7] = [0; 7];

/// Texture handle
static mut TEXTURE: u32 = 0;

/// Current rotation angles
static mut ROTATION_X: f32 = 0.0;
static mut ROTATION_Y: f32 = 0.0;

/// Previous A button state (for edge detection)
static mut PREV_A_BUTTON: u32 = 0;

/// Shape names for UI display
static SHAPE_NAMES: [&str; 7] = [
    "Cube (1×1×1)",
    "Sphere (r=1.5, 32×16)",
    "Cylinder (r=1, h=2, 24 segs)",
    "Cone (r=1.5→0, h=2, 24 segs)",
    "Plane (3×3, 8×8 subdivs)",
    "Torus (R=1.5, r=0.5, 32×16)",
    "Capsule (r=0.8, h=2, 24×8)",
];

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
        // Configure rendering
        set_clear_color(0x1a1a2eFF); // Dark blue
        render_mode(0); // Unlit with normals (simple Lambert shading)
        depth_test(1); // Enable depth testing

        // Set up procedural sky for lighting
        set_sky(
            0.7, 0.8, 0.9, // horizon (light blue)
            0.3, 0.5, 0.9, // zenith (deeper blue)
            0.3, 0.8, 0.5, // sun direction (from upper-right)
            2.0, 1.9, 1.8, // sun color (warm white)
            200.0,         // sun sharpness
        );

        // Set up camera
        camera_set(0.0, 5.0, 10.0, 0.0, 0.0, 0.0);
        camera_fov(60.0);

        // Load checkerboard texture to verify UV mapping
        TEXTURE = load_texture(8, 8, CHECKERBOARD.as_ptr());
        texture_filter(0); // Nearest neighbor for crisp pixels

        // Generate all 7 procedural shapes
        MESH_HANDLES[0] = cube(1.0, 1.0, 1.0); // 2×2×2 cube
        MESH_HANDLES[1] = sphere(1.5, 32, 16); // High-quality sphere
        MESH_HANDLES[2] = cylinder(1.0, 1.0, 2.0, 24); // Uniform cylinder
        MESH_HANDLES[3] = cylinder(1.5, 0.0, 2.0, 24); // Cone (radius_top = 0)
        MESH_HANDLES[4] = plane(3.0, 3.0, 8, 8); // Subdivided plane
        MESH_HANDLES[5] = torus(1.5, 0.5, 32, 16); // Donut
        MESH_HANDLES[6] = capsule(0.8, 2.0, 24, 8); // Pill shape
    }
}

#[no_mangle]
pub extern "C" fn update() {
    unsafe {
        // Shape cycling (press A to change shape)
        let a_button = button_held(0, BUTTON_A);
        if a_button != 0 && PREV_A_BUTTON == 0 {
            // Button just pressed
            CURRENT_SHAPE = (CURRENT_SHAPE + 1) % 7;
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
        // Draw current shape
        push_identity();
        push_rotate_y(ROTATION_Y);
        push_rotate_x(ROTATION_X);

        // Special positioning for plane (tilt to be visible from camera)
        if CURRENT_SHAPE == 4 {
            push_rotate_x(-45.0); // Additional tilt for plane
        }

        texture_bind(TEXTURE);
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
    }
}
