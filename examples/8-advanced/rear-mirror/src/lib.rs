//! Rear-View Mirror Example
//!
//! Demonstrates a driving/racing scenario with a rear-view mirror
//! using viewport for the mirror view.
//!
//! Controls:
//! - Left stick: Steer and accelerate
//! - A button: Toggle mirror visibility

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

// Screen dimensions
const SCREEN_WIDTH: u32 = 960;
const SCREEN_HEIGHT: u32 = 540;

// Mirror dimensions (positioned at top center)
const MIRROR_WIDTH: u32 = 300;
const MIRROR_HEIGHT: u32 = 100;
const MIRROR_X: u32 = (SCREEN_WIDTH - MIRROR_WIDTH) / 2;
const MIRROR_Y: u32 = 10;

// Mesh handles
static mut CAR_BODY: u32 = 0;
static mut CAR_WHEEL: u32 = 0;
static mut ROAD: u32 = 0;
static mut TREE: u32 = 0;
static mut SPHERE_MESH: u32 = 0;

// State
static mut PLAYER_X: f32 = 0.0;
static mut PLAYER_Z: f32 = 0.0;
static mut PLAYER_ANGLE: f32 = 0.0;
static mut SPEED: f32 = 0.0;
static mut SHOW_MIRROR: bool = true;
static mut PREV_A_BUTTON: u32 = 0;
static mut TIME: f32 = 0.0;

// Chasing "enemy" car
static mut ENEMY_Z: f32 = -30.0;
static mut ENEMY_OFFSET: f32 = 0.0;

#[no_mangle]
pub extern "C" fn init() {
    unsafe {
        set_clear_color(0x87CEEBFF);
        render_mode(0);
        depth_test(1);

        // Generate meshes
        CAR_BODY = cube(2.0, 0.8, 4.0);      // Car body
        CAR_WHEEL = cylinder(0.4, 0.4, 0.3, 12); // Wheel
        ROAD = plane(20.0, 200.0, 4, 20);    // Long road
        TREE = cylinder(0.3, 0.0, 3.0, 8);   // Tree trunk (cone)
        SPHERE_MESH = sphere(1.5, 12, 8);    // Tree foliage
    }
}

#[no_mangle]
pub extern "C" fn update() {
    unsafe {
        TIME += 1.0 / 60.0;

        // Toggle mirror with A
        let a_button = button_held(0, BUTTON_A);
        if a_button != 0 && PREV_A_BUTTON == 0 {
            SHOW_MIRROR = !SHOW_MIRROR;
        }
        PREV_A_BUTTON = a_button;

        // Controls
        let stick_x = left_stick_x(0);
        let stick_y = left_stick_y(0);

        // Accelerate/brake with stick Y
        SPEED += stick_y * 0.2;
        SPEED *= 0.98; // Friction
        if SPEED > 10.0 { SPEED = 10.0; }
        if SPEED < -5.0 { SPEED = -5.0; }

        // Steering with stick X (only when moving)
        if SPEED.abs() > 0.5 {
            PLAYER_ANGLE -= stick_x * 2.0 * (SPEED / 10.0);
        }

        // Move forward
        let angle_rad = PLAYER_ANGLE.to_radians();
        PLAYER_X += libm::sinf(angle_rad) * SPEED * 0.1;
        PLAYER_Z += libm::cosf(angle_rad) * SPEED * 0.1;

        // Keep player on road
        if PLAYER_X > 8.0 { PLAYER_X = 8.0; }
        if PLAYER_X < -8.0 { PLAYER_X = -8.0; }

        // Enemy car follows behind
        let target_z = PLAYER_Z - 15.0;
        ENEMY_Z += (target_z - ENEMY_Z) * 0.02;
        ENEMY_OFFSET = libm::sinf(TIME * 2.0) * 3.0; // Weave back and forth
    }
}

/// Draw the scene (road, trees, enemy car)
unsafe fn draw_scene(is_mirror: bool) {
    // Road
    push_identity();
    push_translate(0.0, 0.0, PLAYER_Z);
    set_color(0x333333FF);
    draw_mesh(ROAD);

    // Road center line
    push_identity();
    push_translate(0.0, 0.01, PLAYER_Z);
    push_scale(0.5, 1.0, 1.0);
    set_color(0xFFFF00FF);
    draw_mesh(ROAD);

    // Trees on sides
    for i in 0..20 {
        let z = (i as f32) * 10.0 - 50.0 + PLAYER_Z;
        let wrapped_z = z - (libm::floorf((z - PLAYER_Z + 50.0) / 200.0) * 200.0);

        // Left side trees
        push_identity();
        push_translate(-12.0, 0.0, wrapped_z);
        set_color(0x8B4513FF); // Brown trunk
        draw_mesh(TREE);

        push_identity();
        push_translate(-12.0, 3.0, wrapped_z);
        set_color(0x228B22FF); // Green foliage
        draw_mesh(SPHERE_MESH);

        // Right side trees
        push_identity();
        push_translate(12.0, 0.0, wrapped_z);
        set_color(0x8B4513FF);
        draw_mesh(TREE);

        push_identity();
        push_translate(12.0, 3.0, wrapped_z);
        set_color(0x228B22FF);
        draw_mesh(SPHERE_MESH);
    }

    // Enemy car (behind player)
    push_identity();
    push_translate(ENEMY_OFFSET, 0.5, ENEMY_Z);
    set_color(0xFF0000FF); // Red car
    draw_mesh(CAR_BODY);

    // Enemy wheels
    for (dx, dz) in [(-1.2, 1.5), (1.2, 1.5), (-1.2, -1.5), (1.2, -1.5)].iter() {
        push_identity();
        push_translate(ENEMY_OFFSET + dx, 0.3, ENEMY_Z + dz);
        push_rotate_y(90.0);
        set_color(0x222222FF);
        draw_mesh(CAR_WHEEL);
    }

    // Player's car hood (visible from cockpit view, not in mirror)
    if !is_mirror {
        push_identity();
        push_translate(PLAYER_X, 0.3, PLAYER_Z + 2.0);
        push_rotate_y(PLAYER_ANGLE);
        set_color(0x0066CCFF); // Blue hood
        push_scale(1.0, 0.3, 1.0);
        draw_mesh(CAR_BODY);
    }
}

/// Draw from driver's perspective (looking forward)
unsafe fn draw_forward_view() {
    let angle_rad = PLAYER_ANGLE.to_radians();
    let look_x = PLAYER_X + libm::sinf(angle_rad) * 10.0;
    let look_z = PLAYER_Z + libm::cosf(angle_rad) * 10.0;

    camera_set(PLAYER_X, 2.0, PLAYER_Z, look_x, 1.5, look_z);
    camera_fov(75.0);

    draw_scene(false);
}

/// Draw from mirror perspective (looking backward)
unsafe fn draw_mirror_view() {
    let angle_rad = PLAYER_ANGLE.to_radians();
    let look_x = PLAYER_X - libm::sinf(angle_rad) * 10.0;
    let look_z = PLAYER_Z - libm::cosf(angle_rad) * 10.0;

    camera_set(PLAYER_X, 2.5, PLAYER_Z, look_x, 2.0, look_z);
    camera_fov(60.0);

    draw_scene(true);
}

#[no_mangle]
pub extern "C" fn render() {
    unsafe {
        // Main view (full screen)
        viewport_clear();
        draw_forward_view();

        // Mirror view (small viewport at top center)
        if SHOW_MIRROR {
            // Draw mirror frame/border
            viewport_clear();
            set_color(0x222222FF,
            );
        draw_rect(
                MIRROR_X as f32 - 4.0, MIRROR_Y as f32 - 4.0, MIRROR_WIDTH as f32 + 8.0, MIRROR_HEIGHT as f32 + 8.0);

            // Draw mirror contents
            viewport(MIRROR_X, MIRROR_Y, MIRROR_WIDTH, MIRROR_HEIGHT);
            draw_mirror_view();

            // Mirror label
            viewport_clear();
            let mirror_text = "REAR VIEW";
            set_color(0x888888FF,
            );
        draw_text(
                mirror_text.as_ptr(), mirror_text.len() as u32, MIRROR_X as f32 + MIRROR_WIDTH as f32 / 2.0 - 40.0, MIRROR_Y as f32 + MIRROR_HEIGHT as f32 + 8.0, 12.0);
        }

        // HUD
        viewport_clear();

        // Title
        let title = "REAR-VIEW MIRROR DEMO";
        set_color(0xFFFFFFFF,
        );
        draw_text(
            title.as_ptr(), title.len() as u32, 10.0, 10.0, 24.0);

        // Explanation
        let explain = "Uses viewport() for picture-in-picture mirror view";
        set_color(0x888888FF,
        );
        draw_text(
            explain.as_ptr(), explain.len() as u32, 10.0, 40.0, 12.0);

        // Mirror status
        let mirror_status = if SHOW_MIRROR { "Mirror: ON" } else { "Mirror: OFF" };
        set_color(if SHOW_MIRROR { 0x88FF88FF } else { 0xFF8888FF },
        );
        draw_text(
            mirror_status.as_ptr(), mirror_status.len() as u32, 10.0, 60.0, 14.0);

        // Speed display
        let speed_label = "Speed: ";
        set_color(0xAAAAAAFF,
        );
        draw_text(
            speed_label.as_ptr(), speed_label.len() as u32, SCREEN_WIDTH as f32 - 150.0, SCREEN_HEIGHT as f32 - 60.0, 16.0);

        // Controls at bottom
        let controls = "Controls: Left Stick = Steer/Accelerate | A = Toggle Mirror";
        set_color(0xAAAAAAFF,
        );
        draw_text(
            controls.as_ptr(), controls.len() as u32, 10.0, SCREEN_HEIGHT as f32 - 30.0, 14.0);

        // Red car warning if visible in mirror
        if SHOW_MIRROR {
            let warning = "Watch for red car behind you!";
            set_color(0xFF4444FF,
            );
        draw_text(
                warning.as_ptr(), warning.len() as u32, SCREEN_WIDTH as f32 / 2.0 - 100.0, SCREEN_HEIGHT as f32 - 50.0, 12.0);
        }
    }
}
