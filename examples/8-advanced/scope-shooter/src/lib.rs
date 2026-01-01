//! Scope Shooter Example
//!
//! Demonstrates a sniper scope mechanic using stencil masking.
//!
//! Controls:
//! - Left stick: Look around
//! - Right trigger (or B button): Hold to scope
//! - A button: Fire (visual feedback only)

#![no_std]
#![no_main]

use core::panic::PanicInfo;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    core::arch::wasm32::unreachable()
}

// Import the canonical FFI bindings
#[path = "../../../../include/zx.rs"]
mod ffi;
use ffi::*;


// Input (button indices from zx.rs)
const BUTTON_A: u32 = 4;
const BUTTON_B: u32 = 5;

// Screen dimensions
const SCREEN_WIDTH: f32 = 960.0;
const SCREEN_HEIGHT: f32 = 540.0;
const CENTER_X: f32 = SCREEN_WIDTH / 2.0;
const CENTER_Y: f32 = SCREEN_HEIGHT / 2.0;

// Scope settings
const SCOPE_RADIUS: f32 = 200.0;
const NORMAL_FOV: f32 = 60.0;
const SCOPED_FOV: f32 = 15.0;

// Mesh handles
static mut CUBE_MESH: u32 = 0;
static mut SPHERE_MESH: u32 = 0;
static mut FLOOR_MESH: u32 = 0;

// State
static mut CAMERA_YAW: f32 = 0.0;
static mut CAMERA_PITCH: f32 = 0.0;
static mut IS_SCOPED: bool = false;
static mut PREV_B_BUTTON: u32 = 0;
static mut FIRE_FLASH: f32 = 0.0;

#[no_mangle]
pub extern "C" fn init() {
    unsafe {
        set_clear_color(0x87CEEBFF); // Sky blue
        render_mode(0);
        depth_test(1);

        // Generate scene objects
        CUBE_MESH = cube(2.0, 2.0, 2.0);
        SPHERE_MESH = sphere(1.0, 16, 8);
        FLOOR_MESH = plane(100.0, 100.0, 8, 8);
    }
}

#[no_mangle]
pub extern "C" fn update() {
    unsafe {
        // Camera control with left stick
        let stick_x = left_stick_x(0);
        let stick_y = left_stick_y(0);

        let sensitivity = if IS_SCOPED { 0.5 } else { 2.0 };
        CAMERA_YAW -= stick_x * sensitivity; // Negate: right stick = turn right
        CAMERA_PITCH -= stick_y * sensitivity;

        // Clamp pitch
        if CAMERA_PITCH > 60.0 {
            CAMERA_PITCH = 60.0;
        }
        if CAMERA_PITCH < -60.0 {
            CAMERA_PITCH = -60.0;
        }

        // Scope control: Hold right trigger OR hold B button
        let b_button = button_held(0, BUTTON_B);
        let trigger = trigger_right(0);

        // Hold to scope (not toggle)
        IS_SCOPED = trigger > 0.5 || (b_button != 0);

        PREV_B_BUTTON = b_button;

        // Fire with A button
        let a_button = button_held(0, BUTTON_A);
        if a_button != 0 && FIRE_FLASH <= 0.0 {
            FIRE_FLASH = 0.2;
        }

        // Decay fire flash
        if FIRE_FLASH > 0.0 {
            FIRE_FLASH -= 1.0 / 60.0;
        }
    }
}

/// Get camera look direction
unsafe fn get_camera_target() -> (f32, f32, f32) {
    let yaw_rad = CAMERA_YAW.to_radians();
    let pitch_rad = CAMERA_PITCH.to_radians();

    let cos_pitch = libm::cosf(pitch_rad);
    let sin_pitch = libm::sinf(pitch_rad);
    let cos_yaw = libm::cosf(yaw_rad);
    let sin_yaw = libm::sinf(yaw_rad);

    let tx = sin_yaw * cos_pitch;
    let ty = sin_pitch;
    let tz = cos_yaw * cos_pitch;

    (tx, ty, tz)
}

/// Draw the 3D scene with targets
unsafe fn draw_scene(fov: f32) {
    let (tx, ty, tz) = get_camera_target();
    camera_set(0.0, 2.0, 0.0, tx, 2.0 + ty, tz);
    camera_fov(fov);

    // Ground
    push_identity();
    push_translate(0.0, 0.0, 0.0);
    set_color(0x228B22FF); // Forest green
    draw_mesh(FLOOR_MESH);

    // Target cubes at various distances
    let targets: [(f32, f32, f32, u32); 6] = [
        (10.0, 1.0, 10.0, 0xFF0000FF),   // Red - close
        (-15.0, 1.0, 20.0, 0x00FF00FF),  // Green
        (25.0, 2.0, 30.0, 0x0000FFFF),   // Blue
        (-20.0, 1.5, 40.0, 0xFFFF00FF),  // Yellow
        (0.0, 3.0, 50.0, 0xFF00FFFF),    // Magenta
        (30.0, 1.0, 60.0, 0x00FFFFFF),   // Cyan - far
    ];

    for (x, y, z, color) in targets {
        push_identity();
        push_translate(x, y, z);
        set_color(color);
        draw_mesh(CUBE_MESH);

        // Add sphere on top
        push_identity();
        push_translate(x, y + 2.0, z);
        set_color(0xFFFFFFFF);
        draw_mesh(SPHERE_MESH);
    }
}

/// Draw the scope circle mask
unsafe fn draw_scope_mask() {
    set_color(0xFFFFFFFF);
    draw_circle(CENTER_X, CENTER_Y, SCOPE_RADIUS);
}

/// Draw scope reticle (crosshairs)
unsafe fn draw_reticle() {
    let reticle_color = 0x00FF00FF; // Green
    let thickness = 2.0;
    let gap = 20.0;
    let length = 40.0;

    set_color(reticle_color);

    // Horizontal lines
    draw_line(
        CENTER_X - SCOPE_RADIUS + 30.0,
        CENTER_Y,
        CENTER_X - gap,
        CENTER_Y,
        thickness,
    );
    draw_line(
        CENTER_X + gap,
        CENTER_Y,
        CENTER_X + SCOPE_RADIUS - 30.0,
        CENTER_Y,
        thickness,
    );

    // Vertical lines
    draw_line(
        CENTER_X,
        CENTER_Y - SCOPE_RADIUS + 30.0,
        CENTER_X,
        CENTER_Y - gap,
        thickness,
    );
    draw_line(
        CENTER_X,
        CENTER_Y + gap,
        CENTER_X,
        CENTER_Y + SCOPE_RADIUS - 30.0,
        thickness,
    );

    // Center dot
    set_color(reticle_color);
        draw_rect(CENTER_X - 2.0, CENTER_Y - 2.0, 4.0, 4.0);

    // Mil-dots on horizontal line
    for i in 1..5 {
        let offset = i as f32 * 30.0;
        set_color(reticle_color,
        );
        draw_rect(
            CENTER_X + offset - 1.5, CENTER_Y - 4.0, 3.0, 8.0);
        set_color(reticle_color,
        );
        draw_rect(
            CENTER_X - offset - 1.5, CENTER_Y - 4.0, 3.0, 8.0);
    }
}

/// Draw scope border ring
unsafe fn draw_scope_border() {
    let border_color = 0x111111FF;

    // Draw black vignette outside scope (non-overlapping rectangles)
    // Top
    set_color(border_color);
        draw_rect(0.0, 0.0, SCREEN_WIDTH, CENTER_Y - SCOPE_RADIUS);
    // Bottom
    set_color(border_color,
    );
        draw_rect(
        0.0, CENTER_Y + SCOPE_RADIUS, SCREEN_WIDTH, SCREEN_HEIGHT - (CENTER_Y + SCOPE_RADIUS));
    // Left (only the middle band to avoid overlap)
    set_color(border_color,
    );
        draw_rect(
        0.0, CENTER_Y - SCOPE_RADIUS, CENTER_X - SCOPE_RADIUS, SCOPE_RADIUS * 2.0);
    // Right (only the middle band to avoid overlap)
    set_color(border_color,
    );
        draw_rect(
        CENTER_X + SCOPE_RADIUS, CENTER_Y - SCOPE_RADIUS, SCREEN_WIDTH - (CENTER_X + SCOPE_RADIUS), SCOPE_RADIUS * 2.0);
}

#[no_mangle]
pub extern "C" fn render() {
    unsafe {
        if IS_SCOPED {
            // Draw black border outside scope using inverted stencil
            stencil_begin();
            draw_scope_mask();
            stencil_invert(); // Render OUTSIDE the circle
            set_color(0x111111FF);
        draw_rect(0.0, 0.0, SCREEN_WIDTH, SCREEN_HEIGHT);
            stencil_clear();

            // Draw zoomed scene inside scope
            stencil_begin();
            draw_scope_mask();
            stencil_end();
            draw_scene(SCOPED_FOV);
            stencil_clear();

            // Draw reticle on top
            draw_reticle();

            // Scope frame text
            let zoom_text = "8x ZOOM";
            set_color(0x00FF00AA,
            );
        draw_text(
                zoom_text.as_ptr(), zoom_text.len() as u32, CENTER_X - SCOPE_RADIUS + 20.0, CENTER_Y + SCOPE_RADIUS - 30.0, 14.0);
        } else {
            // Normal unscoped view
            draw_scene(NORMAL_FOV);

            // Simple crosshair
            set_color(0xFFFFFFFF);
            draw_line(
                CENTER_X - 10.0,
                CENTER_Y,
                CENTER_X + 10.0,
                CENTER_Y,
                2.0,
            );
            draw_line(
                CENTER_X,
                CENTER_Y - 10.0,
                CENTER_X,
                CENTER_Y + 10.0,
                2.0,
            );
        }

        // Fire flash effect
        if FIRE_FLASH > 0.0 {
            let alpha = (FIRE_FLASH * 255.0 * 2.0) as u32;
            let flash_color = 0xFFFF00 << 8 | alpha.min(255);
            set_color(flash_color);
        draw_rect(0.0, 0.0, SCREEN_WIDTH, SCREEN_HEIGHT);
        }

        // UI - Title (top left, only when not scoped)
        if !IS_SCOPED {
            let title = "SNIPER SCOPE DEMO";
            set_color(0xFFFFFFFF,
            );
        draw_text(
                title.as_ptr(), title.len() as u32, 10.0, 10.0, 24.0);

            // Explanation
            let explain = "Stencil masks the scope view to a circle";
            set_color(0x888888FF,
            );
        draw_text(
                explain.as_ptr(), explain.len() as u32, 10.0, 40.0, 12.0);
        }

        // Controls at bottom
        let controls = if IS_SCOPED {
            "Controls: Left Stick = Aim | Release B/Right Trigger = Unscope | A = Fire"
        } else {
            "Controls: Left Stick = Look | Hold B or Right Trigger = Scope | A = Fire"
        };
        set_color(0xAAAAAAFF,
        );
        draw_text(
            controls.as_ptr(), controls.len() as u32, 10.0, SCREEN_HEIGHT - 30.0, 14.0);

        // Status indicator
        let status = if IS_SCOPED { "SCOPED - 8x Zoom" } else { "Hip Fire" };
        set_color(if IS_SCOPED { 0x00FF00FF } else { 0xFFFFFFFF },
        );
        draw_text(
            status.as_ptr(), status.len() as u32, SCREEN_WIDTH - 150.0, 10.0, 16.0);
    }
}
