//! Dither Transparency Demo
//!
//! Demonstrates the always-on dither transparency system with:
//! - Multiple meshes at different alpha levels (0-15)
//! - Overlapping meshes with different dither offsets
//! - Debug controls for real-time tweaking
//!
//! Controls:
//! - Left stick: Rotate scene
//! - F3: Open debug panel to tweak alpha levels and offsets
//! - F5: Pause, F6: Step frame
//!
//! The dither system uses a 4x4 Bayer matrix for classic PS1/Saturn-style
//! screen-door transparency. Alpha level 15 is fully opaque (default),
//! while 0 is fully transparent.

#![no_std]
#![no_main]

use core::panic::PanicInfo;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    core::arch::wasm32::unreachable()
}

// ============================================================================
// FFI Declarations
// ============================================================================

#[link(wasm_import_module = "env")]
extern "C" {
    // Configuration
    fn set_clear_color(color: u32);
    fn render_mode(mode: u32);

    // Camera
    fn camera_set(x: f32, y: f32, z: f32, target_x: f32, target_y: f32, target_z: f32);
    fn camera_fov(fov_degrees: f32);

    // Input
    fn left_stick_x(player: u32) -> f32;
    fn left_stick_y(player: u32) -> f32;

    // Procedural mesh generation
    fn cube(size_x: f32, size_y: f32, size_z: f32) -> u32;
    fn sphere(radius: f32, segments: u32, rings: u32) -> u32;
    fn torus(major_radius: f32, minor_radius: f32, major_segments: u32, minor_segments: u32) -> u32;
    fn plane(size_x: f32, size_z: f32, subdivisions_x: u32, subdivisions_z: u32) -> u32;

    // Mesh drawing
    fn draw_mesh(handle: u32);

    // Transform
    fn push_identity();
    fn push_translate(x: f32, y: f32, z: f32);
    fn push_rotate_x(angle_deg: f32);
    fn push_rotate_y(angle_deg: f32);
    fn push_scale(x: f32, y: f32, z: f32);

    // Render state
    fn set_color(color: u32);
    fn depth_test(enabled: u32);

    // Dither transparency
    fn uniform_alpha(level: u32);
    fn dither_offset(x: u32, y: u32);

    // Sky (for lighting)
    fn sky_set_colors(horizon_color: u32, zenith_color: u32);
    fn sky_set_sun(dir_x: f32, dir_y: f32, dir_z: f32, color: u32, sharpness: f32);

    // Time
    fn elapsed_time() -> f32;

    // 2D drawing for UI
    fn draw_text(ptr: *const u8, len: u32, x: f32, y: f32, size: f32, color: u32);

    // Debug inspection
    fn debug_group_begin(name: *const u8, name_len: u32);
    fn debug_group_end();
    fn debug_register_i32(name: *const u8, name_len: u32, ptr: *const i32);
    fn debug_register_f32(name: *const u8, name_len: u32, ptr: *const f32);
    fn debug_register_bool(name: *const u8, name_len: u32, ptr: *const u8);
}

// ============================================================================
// State
// ============================================================================

// Mesh handles
static mut SPHERE_MESH: u32 = 0;
static mut CUBE_MESH: u32 = 0;
static mut TORUS_MESH: u32 = 0;
static mut GROUND_MESH: u32 = 0;

// Scene rotation
static mut ROTATION_Y: f32 = 0.0;
static mut ROTATION_X: f32 = 15.0;

// Debug-tweakable values
static mut ALPHA_SPHERE_1: i32 = 15; // Front sphere - opaque
static mut ALPHA_SPHERE_2: i32 = 10; // Middle sphere - 63% opaque
static mut ALPHA_SPHERE_3: i32 = 5;  // Back sphere - 31% opaque
static mut ALPHA_CUBE: i32 = 8;      // Cube - 50% opaque
static mut ALPHA_TORUS: i32 = 12;    // Torus - 75% opaque
static mut ALPHA_GROUND: i32 = 15;   // Ground - opaque

// Dither offsets for overlapping objects
static mut OFFSET_SPHERE_1: i32 = 0;
static mut OFFSET_SPHERE_2: i32 = 1;
static mut OFFSET_SPHERE_3: i32 = 2;

// Animation
static mut AUTO_ROTATE: u8 = 1;
static mut ROTATION_SPEED: f32 = 20.0;

// ============================================================================
// Implementation
// ============================================================================

#[no_mangle]
pub extern "C" fn init() {
    unsafe {
        // Dark blue background
        set_clear_color(0x1a1a3eFF);

        // Use unlit mode with normals for clear visibility
        render_mode(0);

        // Enable depth testing
        depth_test(1);

        // Set up camera
        camera_set(0.0, 4.0, 10.0, 0.0, 0.0, 0.0);
        camera_fov(50.0);

        // Set up sky for ambient lighting
        sky_set_colors(0x404060FF, 0x202040FF);
        sky_set_sun(-0.5, -1.0, -0.5, 0xFFEEDDFF, 0.8);

        // Generate meshes
        SPHERE_MESH = sphere(1.0, 24, 12);
        CUBE_MESH = cube(1.5, 1.5, 1.5);
        TORUS_MESH = torus(1.2, 0.4, 24, 12);
        GROUND_MESH = plane(8.0, 8.0, 1, 1);

        // Register debug values
        register_debug_values();
    }
}

unsafe fn register_debug_values() {
    // Alpha levels group
    debug_group_begin(b"alpha levels".as_ptr(), 12);
    debug_register_i32(b"sphere_front (0-15)".as_ptr(), 18, &ALPHA_SPHERE_1);
    debug_register_i32(b"sphere_mid (0-15)".as_ptr(), 16, &ALPHA_SPHERE_2);
    debug_register_i32(b"sphere_back (0-15)".as_ptr(), 17, &ALPHA_SPHERE_3);
    debug_register_i32(b"cube (0-15)".as_ptr(), 10, &ALPHA_CUBE);
    debug_register_i32(b"torus (0-15)".as_ptr(), 11, &ALPHA_TORUS);
    debug_register_i32(b"ground (0-15)".as_ptr(), 12, &ALPHA_GROUND);
    debug_group_end();

    // Dither offsets group (for overlapping spheres)
    debug_group_begin(b"dither offsets".as_ptr(), 14);
    debug_register_i32(b"sphere_front (0-3)".as_ptr(), 17, &OFFSET_SPHERE_1);
    debug_register_i32(b"sphere_mid (0-3)".as_ptr(), 15, &OFFSET_SPHERE_2);
    debug_register_i32(b"sphere_back (0-3)".as_ptr(), 16, &OFFSET_SPHERE_3);
    debug_group_end();

    // Animation group
    debug_group_begin(b"animation".as_ptr(), 9);
    debug_register_bool(b"auto_rotate".as_ptr(), 11, &AUTO_ROTATE);
    debug_register_f32(b"speed (deg/s)".as_ptr(), 12, &ROTATION_SPEED);
    debug_group_end();
}

#[no_mangle]
pub extern "C" fn update() {
    unsafe {
        // Manual rotation with left stick
        let stick_x = left_stick_x(0);
        let stick_y = left_stick_y(0);

        if stick_x.abs() > 0.1 || stick_y.abs() > 0.1 {
            ROTATION_Y += stick_x * 3.0;
            ROTATION_X += stick_y * 2.0;
        } else if AUTO_ROTATE != 0 {
            // Auto-rotate
            ROTATION_Y += ROTATION_SPEED * (1.0 / 60.0);
        }

        // Clamp rotation_x to avoid gimbal issues
        if ROTATION_X > 89.0 {
            ROTATION_X = 89.0;
        }
        if ROTATION_X < -89.0 {
            ROTATION_X = -89.0;
        }

        // Wrap rotation_y
        if ROTATION_Y >= 360.0 {
            ROTATION_Y -= 360.0;
        }
        if ROTATION_Y < 0.0 {
            ROTATION_Y += 360.0;
        }
    }
}

#[no_mangle]
pub extern "C" fn render() {
    unsafe {
        let time = elapsed_time();

        // ====================================================================
        // Draw ground plane (opaque by default, can be tweaked)
        // ====================================================================
        push_identity();
        push_translate(0.0, -2.0, 0.0);
        set_color(0x505070FF);
        uniform_alpha(clamp_alpha(ALPHA_GROUND));
        dither_offset(0, 0);
        draw_mesh(GROUND_MESH);

        // ====================================================================
        // Draw torus (rotating, partially transparent)
        // ====================================================================
        push_identity();
        push_rotate_y(ROTATION_Y);
        push_rotate_x(ROTATION_X);
        push_translate(0.0, 0.0, 0.0);
        push_rotate_y(time * 30.0); // Additional spin

        set_color(0xFFAA66FF); // Orange
        uniform_alpha(clamp_alpha(ALPHA_TORUS));
        dither_offset(0, 0);
        draw_mesh(TORUS_MESH);

        // ====================================================================
        // Draw three overlapping spheres to demonstrate dither offset
        // Without different offsets, overlapping dithered objects cancel out
        // ====================================================================

        // Back sphere (most transparent)
        push_identity();
        push_rotate_y(ROTATION_Y);
        push_rotate_x(ROTATION_X);
        push_translate(0.0, 0.5, -1.5);

        set_color(0x6666FFFF); // Blue
        uniform_alpha(clamp_alpha(ALPHA_SPHERE_3));
        dither_offset(
            clamp_offset(OFFSET_SPHERE_3) as u32,
            clamp_offset(OFFSET_SPHERE_3) as u32,
        );
        draw_mesh(SPHERE_MESH);

        // Middle sphere
        push_identity();
        push_rotate_y(ROTATION_Y);
        push_rotate_x(ROTATION_X);
        push_translate(0.0, 0.5, 0.0);

        set_color(0x66FF66FF); // Green
        uniform_alpha(clamp_alpha(ALPHA_SPHERE_2));
        dither_offset(
            clamp_offset(OFFSET_SPHERE_2) as u32,
            clamp_offset(OFFSET_SPHERE_2) as u32,
        );
        draw_mesh(SPHERE_MESH);

        // Front sphere
        push_identity();
        push_rotate_y(ROTATION_Y);
        push_rotate_x(ROTATION_X);
        push_translate(0.0, 0.5, 1.5);

        set_color(0xFF6666FF); // Red
        uniform_alpha(clamp_alpha(ALPHA_SPHERE_1));
        dither_offset(
            clamp_offset(OFFSET_SPHERE_1) as u32,
            clamp_offset(OFFSET_SPHERE_1) as u32,
        );
        draw_mesh(SPHERE_MESH);

        // ====================================================================
        // Draw cube (off to the side)
        // ====================================================================
        push_identity();
        push_rotate_y(ROTATION_Y);
        push_rotate_x(ROTATION_X);
        push_translate(3.0, 0.0, 0.0);
        push_rotate_y(time * 45.0);

        set_color(0xFFFF66FF); // Yellow
        uniform_alpha(clamp_alpha(ALPHA_CUBE));
        dither_offset(0, 0);
        draw_mesh(CUBE_MESH);

        // ====================================================================
        // Reset to opaque for UI
        // ====================================================================
        uniform_alpha(15);
        dither_offset(0, 0);

        // Draw UI instructions
        let title = "Dither Transparency Demo";
        draw_text(title.as_ptr(), title.len() as u32, 10.0, 10.0, 24.0, 0xFFFFFFFF);

        let instructions = "F3: Debug Panel | Left Stick: Rotate | Try changing alpha levels!";
        draw_text(
            instructions.as_ptr(),
            instructions.len() as u32,
            10.0,
            40.0,
            14.0,
            0xAAAAAAFF,
        );

        let tip = "Tip: Set sphere offsets to same value to see pattern cancellation";
        draw_text(tip.as_ptr(), tip.len() as u32, 10.0, 60.0, 12.0, 0x888888FF);
    }
}

/// Clamp alpha to valid range 0-15
fn clamp_alpha(alpha: i32) -> u32 {
    if alpha < 0 {
        0
    } else if alpha > 15 {
        15
    } else {
        alpha as u32
    }
}

/// Clamp offset to valid range 0-3
fn clamp_offset(offset: i32) -> i32 {
    if offset < 0 {
        0
    } else if offset > 3 {
        3
    } else {
        offset
    }
}
