//! Procedural Shapes Example
//!
//! Demonstrates all procedural mesh generation functions with optional texture mapping.
//! Supports both non-UV and UV-enabled variants, toggled with the B button.
//!
//! Shapes:
//! - cube() / cube_uv() — Box with flat normals
//! - sphere() / sphere_uv() — UV sphere with smooth normals
//! - cylinder() / cylinder_uv() — Cylinder with caps
//! - cylinder() (cone variant) — Tapered cylinder (plain mode only)
//! - plane() / plane_uv() — Subdivided ground plane
//! - torus() / torus_uv() — Donut shape
//! - capsule() / capsule_uv() — Pill shape
//!
//! Controls:
//! - A button: Cycle through shapes
//! - B button: Toggle texture mode (plain/textured)
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

    // Procedural mesh generation (non-UV)
    fn cube(size_x: f32, size_y: f32, size_z: f32) -> u32;
    fn sphere(radius: f32, segments: u32, rings: u32) -> u32;
    fn cylinder(radius_bottom: f32, radius_top: f32, height: f32, segments: u32) -> u32;
    fn plane(size_x: f32, size_z: f32, subdivisions_x: u32, subdivisions_z: u32) -> u32;
    fn torus(major_radius: f32, minor_radius: f32, major_segments: u32, minor_segments: u32) -> u32;
    fn capsule(radius: f32, height: f32, segments: u32, rings: u32) -> u32;

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
const BUTTON_A: u32 = 4;
const BUTTON_B: u32 = 5;

/// Current shape index (0-6 in plain mode, 0-5 in textured mode)
static mut CURRENT_SHAPE: u32 = 0;

/// Texture mode toggle
static mut TEXTURED_MODE: bool = false;

/// Mesh handles for plain shapes (7 shapes)
static mut MESH_HANDLES_PLAIN: [u32; 7] = [0; 7];

/// Mesh handles for UV-enabled shapes (6 shapes, no cone UV variant)
static mut MESH_HANDLES_UV: [u32; 6] = [0; 6];

/// Texture handle for UV debug texture
static mut TEXTURE_HANDLE: u32 = 0;

/// Current rotation angles
static mut ROTATION_X: f32 = 0.0;
static mut ROTATION_Y: f32 = 0.0;

/// Previous button states (for edge detection)
static mut PREV_A_BUTTON: u32 = 0;
static mut PREV_B_BUTTON: u32 = 0;

/// Shape names for plain mode
static SHAPE_NAMES_PLAIN: [&str; 7] = [
    "Cube (1×1×1)",
    "Sphere (r=1.5, 32×16)",
    "Cylinder (r=1, h=2, 24 segs)",
    "Cone (r=1.5→0, h=2, 24 segs)",
    "Plane (3×3, 8×8 subdivs)",
    "Torus (R=1.5, r=0.5, 32×16)",
    "Capsule (r=0.8, h=2, 24×8)",
];

/// Shape names for UV mode
static SHAPE_NAMES_UV: [&str; 6] = [
    "Cube (UV box unwrap)",
    "Sphere (UV equirectangular)",
    "Cylinder (UV cylindrical)",
    "Plane (UV grid)",
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
        render_mode(0); // Lambert
        depth_test(1); // Enable depth testing

        // Generate all 7 plain procedural shapes
        MESH_HANDLES_PLAIN[0] = cube(1.0, 1.0, 1.0);
        MESH_HANDLES_PLAIN[1] = sphere(1.5, 32, 16);
        MESH_HANDLES_PLAIN[2] = cylinder(1.0, 1.0, 2.0, 24);
        MESH_HANDLES_PLAIN[3] = cylinder(1.5, 0.0, 2.0, 24); // Cone
        MESH_HANDLES_PLAIN[4] = plane(3.0, 3.0, 8, 8);
        MESH_HANDLES_PLAIN[5] = torus(1.5, 0.5, 32, 16);
        MESH_HANDLES_PLAIN[6] = capsule(0.8, 2.0, 24, 8);

        // Generate all 6 UV-enabled procedural shapes
        // Order matches shape names (cube, sphere, cylinder, plane, torus, capsule)
        MESH_HANDLES_UV[0] = cube_uv(1.0, 1.0, 1.0);
        MESH_HANDLES_UV[1] = sphere_uv(1.5, 32, 16);
        MESH_HANDLES_UV[2] = cylinder_uv(1.0, 1.0, 2.0, 24);
        MESH_HANDLES_UV[3] = plane_uv(3.0, 3.0, 8, 8);
        MESH_HANDLES_UV[4] = torus_uv(1.5, 0.5, 32, 16);
        MESH_HANDLES_UV[5] = capsule_uv(0.8, 2.0, 24, 8);

        // Generate and load UV debug texture
        let texture_pixels = generate_uv_debug_texture();
        TEXTURE_HANDLE = load_texture(64, 64, texture_pixels.as_ptr());
    }
}

#[no_mangle]
pub extern "C" fn update() {
    unsafe {
        // B button toggles texture mode
        let b_button = button_held(0, BUTTON_B);
        if b_button != 0 && PREV_B_BUTTON == 0 {
            TEXTURED_MODE = !TEXTURED_MODE;

            // Reset to shape 0 when switching modes to avoid invalid index
            // (cone is only available in plain mode)
            if TEXTURED_MODE && CURRENT_SHAPE >= 6 {
                CURRENT_SHAPE = 0;
            }
        }
        PREV_B_BUTTON = b_button;

        // A button cycles through shapes
        let a_button = button_held(0, BUTTON_A);
        if a_button != 0 && PREV_A_BUTTON == 0 {
            let max_shapes = if TEXTURED_MODE { 6 } else { 7 };
            CURRENT_SHAPE = (CURRENT_SHAPE + 1) % max_shapes;
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
        camera_set(0.0, 5.0, 10.0, 0.0, 0.0, 0.0);
        camera_fov(60.0);

        // Bind texture if in textured mode
        if TEXTURED_MODE {
            texture_bind(TEXTURE_HANDLE);
        }

        // Draw current shape
        push_identity();
        push_rotate_y(ROTATION_Y);
        push_rotate_x(ROTATION_X);

        // Special positioning for plane (tilt to be visible from camera)
        let is_plane = if TEXTURED_MODE {
            CURRENT_SHAPE == 3 // Plane is index 3 in UV mode
        } else {
            CURRENT_SHAPE == 4 // Plane is index 4 in plain mode
        };

        if is_plane {
            push_rotate_x(-45.0); // Additional tilt for plane
        }

        set_color(0xFFFFFFFF); // White (no tint)

        // Draw from appropriate mesh array based on mode
        if TEXTURED_MODE {
            draw_mesh(MESH_HANDLES_UV[CURRENT_SHAPE as usize]);
        } else {
            draw_mesh(MESH_HANDLES_PLAIN[CURRENT_SHAPE as usize]);
        }

        // Draw UI - mode indicator
        let mode_text = if TEXTURED_MODE {
            "Mode: TEXTURED"
        } else {
            "Mode: PLAIN"
        };
        draw_text(
            mode_text.as_ptr(),
            mode_text.len() as u32,
            10.0,
            10.0,
            20.0,
            if TEXTURED_MODE { 0x88FF88FF } else { 0xFFFFFFFF },
        );

        // Draw shape name
        let shape_name = if TEXTURED_MODE {
            SHAPE_NAMES_UV[CURRENT_SHAPE as usize]
        } else {
            SHAPE_NAMES_PLAIN[CURRENT_SHAPE as usize]
        };
        draw_text(
            shape_name.as_ptr(),
            shape_name.len() as u32,
            10.0,
            35.0,
            18.0,
            0xFFFFFFFF,
        );

        // Draw controls
        let instruction = "A: cycle shapes | B: toggle texture | Stick: rotate";
        draw_text(
            instruction.as_ptr(),
            instruction.len() as u32,
            10.0,
            60.0,
            14.0,
            0xAAAAAAFF,
        );

        // Draw UV info when in textured mode
        if TEXTURED_MODE {
            let uv_info = "UV Debug: Red=U, Green=V, Blue=Checker";
            draw_text(
                uv_info.as_ptr(),
                uv_info.len() as u32,
                10.0,
                85.0,
                12.0,
                0x88FF88FF,
            );
        }
    }
}
