//! Dither Transparency Demo
//!
//! Demonstrates the always-on dither transparency system with:
//! - Side-by-side comparison: same offsets vs unique offsets
//! - Multiple meshes at different alpha levels (0-15)
//! - Debug controls for real-time tweaking
//!
//! Controls:
//! - Left stick: Rotate scene
//! - A button: Toggle between comparison mode and single group
//! - F4: Open Debug Inspector to tweak alpha levels and offsets
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

// Import the canonical FFI bindings
#[path = "../../../../include/zx.rs"]
mod ffi;
use ffi::*;

// ============================================================================
// State
// ============================================================================

// Mesh handles
static mut SPHERE_MESH: u32 = 0;
static mut CUBE_MESH: u32 = 0;
static mut GROUND_MESH: u32 = 0;

// Scene rotation
static mut ROTATION_Y: f32 = 0.0;
static mut ROTATION_X: f32 = 20.0;

// Display mode: 0 = comparison (side-by-side), 1 = single group with tweakable offsets
static mut DISPLAY_MODE: i32 = 0;

// Debug-tweakable values
static mut ALPHA_FRONT: i32 = 8;  // Front sphere - 50% opaque
static mut ALPHA_MID: i32 = 8;    // Middle sphere - 50% opaque
static mut ALPHA_BACK: i32 = 8;   // Back sphere - 50% opaque
static mut ALPHA_GROUND: i32 = 15; // Ground - opaque

// Dither offsets for single-group mode
static mut OFFSET_FRONT: i32 = 0;
static mut OFFSET_MID: i32 = 1;
static mut OFFSET_BACK: i32 = 2;

// Toggle for single-group mode: use unique offsets or all same
static mut USE_UNIQUE_OFFSETS: u8 = 1;

// Animation
static mut AUTO_ROTATE: u8 = 1;
static mut ROTATION_SPEED: f32 = 15.0;

// ============================================================================
// Implementation
// ============================================================================

#[no_mangle]
pub extern "C" fn init() {
    unsafe {
        // Dark blue background
        set_clear_color(0x1a1a3eFF);

        // Use Lambert mode with normals for clear visibility
        render_mode(0);

        // Set up lighting
        light_set(0, -0.5, -1.0, -0.5);
        light_color(0, 0xFFEEDDFF);
        light_intensity(0, 1.0);

        // Generate meshes
        SPHERE_MESH = sphere(0.8, 24, 12);
        CUBE_MESH = cube(1.2, 1.2, 1.2);
        GROUND_MESH = plane(12.0, 8.0, 1, 1);

        // Register debug values
        register_debug_values();
    }
}

unsafe fn register_debug_values() {
    // Display mode
    debug_group_begin(b"display".as_ptr(), 7);
    debug_register_i32(b"mode (0=compare, 1=single)".as_ptr(), 25, &DISPLAY_MODE as *const i32 as *const u8);
    debug_group_end();

    // Alpha levels group
    debug_group_begin(b"alpha levels".as_ptr(), 12);
    debug_register_i32(b"front (0-15)".as_ptr(), 11, &ALPHA_FRONT as *const i32 as *const u8);
    debug_register_i32(b"mid (0-15)".as_ptr(), 9, &ALPHA_MID as *const i32 as *const u8);
    debug_register_i32(b"back (0-15)".as_ptr(), 10, &ALPHA_BACK as *const i32 as *const u8);
    debug_register_i32(b"ground (0-15)".as_ptr(), 12, &ALPHA_GROUND as *const i32 as *const u8);
    debug_group_end();

    // Dither offsets group (for single-group mode)
    debug_group_begin(b"offsets (single mode)".as_ptr(), 20);
    debug_register_bool(b"use_unique_offsets".as_ptr(), 18, &USE_UNIQUE_OFFSETS as *const u8);
    debug_register_i32(b"front (0-3)".as_ptr(), 10, &OFFSET_FRONT as *const i32 as *const u8);
    debug_register_i32(b"mid (0-3)".as_ptr(), 8, &OFFSET_MID as *const i32 as *const u8);
    debug_register_i32(b"back (0-3)".as_ptr(), 9, &OFFSET_BACK as *const i32 as *const u8);
    debug_group_end();

    // Animation group
    debug_group_begin(b"animation".as_ptr(), 9);
    debug_register_bool(b"auto_rotate".as_ptr(), 11, &AUTO_ROTATE as *const u8);
    debug_register_f32(b"speed (deg/s)".as_ptr(), 12, &ROTATION_SPEED as *const f32 as *const u8);
    debug_group_end();
}

#[no_mangle]
pub extern "C" fn update() {
    unsafe {
        // Toggle display mode with A button
        if button_pressed(0, button::A) != 0 {
            DISPLAY_MODE = if DISPLAY_MODE == 0 { 1 } else { 0 };
        }

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
        // Set camera every frame (immediate mode)
        camera_set(0.0, 5.0, 12.0, 0.0, 0.0, 0.0);
        camera_fov(50.0);

        // Draw ground plane
        push_identity();
        push_translate(0.0, -2.0, 0.0);
        set_color(0x404055FF);
        uniform_alpha(clamp_alpha(ALPHA_GROUND));
        dither_offset(0, 0);
        draw_mesh(GROUND_MESH);

        if DISPLAY_MODE == 0 {
            render_comparison_mode();
        } else {
            render_single_mode();
        }

        // Reset to opaque for UI
        uniform_alpha(15);
        dither_offset(0, 0);

        // Draw UI
        render_ui();
    }
}

/// Render side-by-side comparison: same offsets vs unique offsets
unsafe fn render_comparison_mode() {
    // ========================================================================
    // LEFT GROUP: Same offset (0,0) - shows pattern cancellation
    // ========================================================================
    let left_x = -3.0;

    // Back sphere (blue)
    push_identity();
    push_rotate_y(ROTATION_Y);
    push_rotate_x(ROTATION_X);
    push_translate(left_x, 0.5, -1.2);
    set_color(0x6688FFFF);
    uniform_alpha(clamp_alpha(ALPHA_BACK));
    dither_offset(0, 0); // Same offset!
    draw_mesh(SPHERE_MESH);

    // Middle sphere (green)
    push_identity();
    push_rotate_y(ROTATION_Y);
    push_rotate_x(ROTATION_X);
    push_translate(left_x, 0.5, 0.0);
    set_color(0x66FF88FF);
    uniform_alpha(clamp_alpha(ALPHA_MID));
    dither_offset(0, 0); // Same offset!
    draw_mesh(SPHERE_MESH);

    // Front sphere (red)
    push_identity();
    push_rotate_y(ROTATION_Y);
    push_rotate_x(ROTATION_X);
    push_translate(left_x, 0.5, 1.2);
    set_color(0xFF6688FF);
    uniform_alpha(clamp_alpha(ALPHA_FRONT));
    dither_offset(0, 0); // Same offset!
    draw_mesh(SPHERE_MESH);

    // ========================================================================
    // RIGHT GROUP: Unique offsets - correct layering
    // ========================================================================
    let right_x = 3.0;

    // Back sphere (blue)
    push_identity();
    push_rotate_y(ROTATION_Y);
    push_rotate_x(ROTATION_X);
    push_translate(right_x, 0.5, -1.2);
    set_color(0x6688FFFF);
    uniform_alpha(clamp_alpha(ALPHA_BACK));
    dither_offset(2, 2); // Unique offset
    draw_mesh(SPHERE_MESH);

    // Middle sphere (green)
    push_identity();
    push_rotate_y(ROTATION_Y);
    push_rotate_x(ROTATION_X);
    push_translate(right_x, 0.5, 0.0);
    set_color(0x66FF88FF);
    uniform_alpha(clamp_alpha(ALPHA_MID));
    dither_offset(1, 1); // Unique offset
    draw_mesh(SPHERE_MESH);

    // Front sphere (red)
    push_identity();
    push_rotate_y(ROTATION_Y);
    push_rotate_x(ROTATION_X);
    push_translate(right_x, 0.5, 1.2);
    set_color(0xFF6688FF);
    uniform_alpha(clamp_alpha(ALPHA_FRONT));
    dither_offset(0, 0); // Unique offset
    draw_mesh(SPHERE_MESH);
}

/// Render single group with tweakable offsets
unsafe fn render_single_mode() {
    let center_x = 0.0;

    // Determine offsets based on toggle
    let (off_front, off_mid, off_back) = if USE_UNIQUE_OFFSETS != 0 {
        (
            clamp_offset(OFFSET_FRONT) as u32,
            clamp_offset(OFFSET_MID) as u32,
            clamp_offset(OFFSET_BACK) as u32,
        )
    } else {
        (0, 0, 0) // All same
    };

    // Back sphere (blue)
    push_identity();
    push_rotate_y(ROTATION_Y);
    push_rotate_x(ROTATION_X);
    push_translate(center_x, 0.5, -1.2);
    set_color(0x6688FFFF);
    uniform_alpha(clamp_alpha(ALPHA_BACK));
    dither_offset(off_back, off_back);
    draw_mesh(SPHERE_MESH);

    // Middle sphere (green)
    push_identity();
    push_rotate_y(ROTATION_Y);
    push_rotate_x(ROTATION_X);
    push_translate(center_x, 0.5, 0.0);
    set_color(0x66FF88FF);
    uniform_alpha(clamp_alpha(ALPHA_MID));
    dither_offset(off_mid, off_mid);
    draw_mesh(SPHERE_MESH);

    // Front sphere (red)
    push_identity();
    push_rotate_y(ROTATION_Y);
    push_rotate_x(ROTATION_X);
    push_translate(center_x, 0.5, 1.2);
    set_color(0xFF6688FF);
    uniform_alpha(clamp_alpha(ALPHA_FRONT));
    dither_offset(off_front, off_front);
    draw_mesh(SPHERE_MESH);
}

/// Render UI text
unsafe fn render_ui() {
    let title = "Dither Transparency Demo";
    set_color(0xFFFFFFFF);
        draw_text(title.as_ptr(), title.len() as u32, 10.0, 10.0, 24.0);

    if DISPLAY_MODE == 0 {
        // Comparison mode labels
        let left_label = "SAME OFFSETS";
        set_color(0xFF8888FF);
        draw_text(left_label.as_ptr(), left_label.len() as u32, 80.0, 420.0, 18.0);

        let right_label = "UNIQUE OFFSETS";
        set_color(0x88FF88FF);
        draw_text(right_label.as_ptr(), right_label.len() as u32, 440.0, 420.0, 18.0);

        let left_desc = "(pattern cancellation)";
        set_color(0xAAAAFFFF);
        draw_text(left_desc.as_ptr(), left_desc.len() as u32, 60.0, 445.0, 12.0);

        let right_desc = "(correct layering)";
        set_color(0xAAFFAAFF);
        draw_text(right_desc.as_ptr(), right_desc.len() as u32, 450.0, 445.0, 12.0);

        let instruction = "Press A to switch to single-group mode";
        set_color(0xAAAAAAFF);
        draw_text(instruction.as_ptr(), instruction.len() as u32, 10.0, 40.0, 14.0);
    } else {
        // Single mode
        let mode_text = if USE_UNIQUE_OFFSETS != 0 {
            "Mode: UNIQUE OFFSETS (use F3 to toggle)"
        } else {
            "Mode: SAME OFFSETS (use F3 to toggle)"
        };
        set_color(0xAAAAAAFF);
        draw_text(mode_text.as_ptr(), mode_text.len() as u32, 10.0, 40.0, 14.0);

        let instruction = "Press A to switch to comparison mode";
        set_color(0x888888FF);
        draw_text(instruction.as_ptr(), instruction.len() as u32, 10.0, 60.0, 14.0);
    }

    let controls = "Left Stick: Rotate | F4: Debug Inspector";
    set_color(0x666666FF);
        draw_text(controls.as_ptr(), controls.len() as u32, 10.0, 80.0, 12.0);
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
