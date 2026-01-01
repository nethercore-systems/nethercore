//! Mode 0 Inspector - Lambert Shading
//!
//! Demonstrates the debug inspection system for Mode 0 rendering.
//! Mode 0 provides Lambert diffuse shading with up to 4 dynamic lights.
//!
//! Features:
//! - Shape cycling (Sphere, Cube, Torus)
//! - Sky gradient configuration (horizon/zenith colors)
//! - Sun direction, color, and sharpness control
//! - Real-time parameter tweaking via debug panel
//!
//! Usage:
//! 1. Run the game
//! 2. Press F4 to open the Debug Inspector
//! 3. Tweak values and see immediate visual changes
//! 4. Press F5 to pause, F6 to step frame

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
// Debug Values - Exposed in the debug panel
// ============================================================================

// Shape settings
static mut SHAPE_INDEX: i32 = 0; // 0=Sphere, 1=Cube, 2=Torus
static mut ROTATION_SPEED: f32 = 30.0; // degrees per second
static mut OBJECT_COLOR: u32 = 0xFFFFFFFF; // White

// Sky settings
static mut HORIZON_COLOR: u32 = 0xB2D8F2FF; // Light blue
static mut ZENITH_COLOR: u32 = 0x3366B2FF; // Darker blue

// Sun settings
static mut SUN_DIR_X: f32 = 0.5;
static mut SUN_DIR_Y: f32 = -0.7;
static mut SUN_DIR_Z: f32 = 0.5;
static mut SUN_COLOR: u32 = 0xFFF2E6FF; // Warm white
static mut SUN_SHARPNESS: f32 = 0.98;

// Light settings (4 dynamic lights)
static mut LIGHT_DIRS: [[f32; 3]; 4] = [
    [-0.5, -0.8, -0.3],  // Light 0: key light (from upper-right-front)
    [0.7, -0.3, -0.5],   // Light 1: fill light (from upper-left)
    [-0.3, 0.5, -0.7],   // Light 2: back light (from lower-front)
    [0.3, -0.6, 0.5],    // Light 3: rim light (from upper-back)
];
static mut LIGHT_COLORS: [u32; 4] = [
    0xFFF2E6FF,  // Light 0: warm white
    0x99B3FFFF,  // Light 1: cool blue
    0xFFB380FF,  // Light 2: orange
    0xB3FFB3FF,  // Light 3: green
];
static mut LIGHT_ENABLED: [u8; 4] = [1, 1, 0, 0];  // First two enabled by default
static mut LIGHT_INTENSITY: [f32; 4] = [1.5, 1.0, 0.8, 0.6];

// ============================================================================
// Internal State
// ============================================================================

// Mesh handles
static mut SPHERE_MESH: u32 = 0;
static mut CUBE_MESH: u32 = 0;
static mut TORUS_MESH: u32 = 0;

// Rotation state
static mut ROTATION_X: f32 = 0.0;
static mut ROTATION_Y: f32 = 0.0;

// Shape names for display
const SHAPE_NAMES: [&str; 3] = ["Sphere", "Cube", "Torus"];

// ============================================================================
// Game Implementation
// ============================================================================

#[no_mangle]
pub extern "C" fn on_debug_change() {
    // Clamp shape index to valid range
    unsafe {
        if SHAPE_INDEX < 0 {
            SHAPE_INDEX = 0;
        }
        if SHAPE_INDEX > 2 {
            SHAPE_INDEX = 2;
        }
    }
}

#[no_mangle]
pub extern "C" fn init() {
    unsafe {
        // Dark background
        set_clear_color(0x1a1a2eFF);

        // Set render mode 0 (Lambert diffuse shading)
        render_mode(0);

        // Enable depth testing
        depth_test(1);

        // Generate meshes
        SPHERE_MESH = sphere(1.5, 32, 16);
        CUBE_MESH = cube(1.2, 1.2, 1.2);
        TORUS_MESH = torus(1.2, 0.5, 32, 16);

        // Initialize 4 lights
        for i in 0..4u32 {
            let dir = LIGHT_DIRS[i as usize];
            light_set(i, dir[0], dir[1], dir[2]);
            light_color(i, LIGHT_COLORS[i as usize]);
            light_intensity(i, LIGHT_INTENSITY[i as usize]);
            if LIGHT_ENABLED[i as usize] != 0 {
                light_enable(i);
            } else {
                light_disable(i);
            }
        }

        // Register debug values
        register_debug_values();
    }
}

unsafe fn register_debug_values() {
    // Shape group
    debug_group_begin(b"shape".as_ptr(), 5);
    debug_register_i32(b"index (0-2)".as_ptr(), 11, &SHAPE_INDEX as *const i32 as *const u8);
    debug_register_f32(b"rotation_speed".as_ptr(), 14, &ROTATION_SPEED as *const f32 as *const u8);
    debug_register_color(b"color".as_ptr(), 5, &OBJECT_COLOR as *const u32 as *const u8);
    debug_group_end();

    // Sky group
    debug_group_begin(b"sky".as_ptr(), 3);
    debug_register_color(b"horizon".as_ptr(), 7, &HORIZON_COLOR as *const u32 as *const u8);
    debug_register_color(b"zenith".as_ptr(), 6, &ZENITH_COLOR as *const u32 as *const u8);
    debug_group_end();

    // Sun group
    debug_group_begin(b"sun".as_ptr(), 3);
    debug_register_f32(b"dir_x".as_ptr(), 5, &SUN_DIR_X as *const f32 as *const u8);
    debug_register_f32(b"dir_y".as_ptr(), 5, &SUN_DIR_Y as *const f32 as *const u8);
    debug_register_f32(b"dir_z".as_ptr(), 5, &SUN_DIR_Z as *const f32 as *const u8);
    debug_register_color(b"color".as_ptr(), 5, &SUN_COLOR as *const u32 as *const u8);
    debug_register_f32(b"sharpness".as_ptr(), 9, &SUN_SHARPNESS as *const f32 as *const u8);
    debug_group_end();

    // Light groups (4 lights)
    register_light_debug(0, b"light0");
    register_light_debug(1, b"light1");
    register_light_debug(2, b"light2");
    register_light_debug(3, b"light3");
}

unsafe fn register_light_debug(index: usize, name: &[u8]) {
    debug_group_begin(name.as_ptr(), name.len() as u32);
    debug_register_u8(b"enabled".as_ptr(), 7, &LIGHT_ENABLED[index]);
    debug_register_f32(b"dir_x".as_ptr(), 5, &LIGHT_DIRS[index][0] as *const f32 as *const u8);
    debug_register_f32(b"dir_y".as_ptr(), 5, &LIGHT_DIRS[index][1] as *const f32 as *const u8);
    debug_register_f32(b"dir_z".as_ptr(), 5, &LIGHT_DIRS[index][2] as *const f32 as *const u8);
    debug_register_color(b"color".as_ptr(), 5, &LIGHT_COLORS[index] as *const u32 as *const u8);
    debug_register_f32(b"intensity".as_ptr(), 9, &LIGHT_INTENSITY[index] as *const f32 as *const u8);
    debug_group_end();
}

#[no_mangle]
pub extern "C" fn update() {
    unsafe {
        // Shape cycling with A button
        if button_pressed(0, button::A) != 0 {
            SHAPE_INDEX = (SHAPE_INDEX + 1) % 3;
        }

        // Rotation control via left stick
        let stick_x = left_stick_x(0);
        let stick_y = left_stick_y(0);

        if stick_x.abs() > 0.1 || stick_y.abs() > 0.1 {
            ROTATION_Y += stick_x * 3.0;
            ROTATION_X += stick_y * 3.0;
        } else {
            // Auto-rotate when idle
            ROTATION_Y += ROTATION_SPEED * (1.0 / 60.0);
        }
    }
}

#[no_mangle]
pub extern "C" fn render() {
    unsafe {
        // Set camera every frame (immediate mode)
        camera_set(0.0, 2.0, 6.0, 0.0, 0.0, 0.0);
        camera_fov(60.0);

        // Configure and draw environment
        // Use gradient with 2-color sky (zenith -> horizon for sky, horizon -> darker for ground)
        env_gradient(0, ZENITH_COLOR, HORIZON_COLOR, HORIZON_COLOR, 0x2A2A2AFF, 0.0, 0.0);
        light_set(0, SUN_DIR_X, SUN_DIR_Y, SUN_DIR_Z);
        light_color(0, SUN_COLOR);
        light_intensity(0, 1.0);
        draw_env();

        // Update lights from debug values
        for i in 0..4u32 {
            let dir = LIGHT_DIRS[i as usize];
            light_set(i, dir[0], dir[1], dir[2]);
            light_color(i, LIGHT_COLORS[i as usize]);
            light_intensity(i, LIGHT_INTENSITY[i as usize]);
            if LIGHT_ENABLED[i as usize] != 0 {
                light_enable(i);
            } else {
                light_disable(i);
            }
        }

        // Draw current shape
        push_identity();
        push_rotate_y(ROTATION_Y);
        push_rotate_x(ROTATION_X);
        set_color(OBJECT_COLOR);

        let mesh = match SHAPE_INDEX {
            0 => SPHERE_MESH,
            1 => CUBE_MESH,
            2 => TORUS_MESH,
            _ => SPHERE_MESH,
        };
        draw_mesh(mesh);

        // Draw UI
        draw_ui();
    }
}

unsafe fn draw_ui() {
    // Title
    let title = b"Mode 0: Lambert";
    set_color(0xFFFFFFFF);
        draw_text(title.as_ptr(), title.len() as u32, 10.0, 10.0, 20.0);

    // Current shape
    let shape_name = SHAPE_NAMES[SHAPE_INDEX as usize];
    let mut label = [0u8; 32];
    let prefix = b"Shape: ";
    label[..prefix.len()].copy_from_slice(prefix);
    let name_bytes = shape_name.as_bytes();
    label[prefix.len()..prefix.len() + name_bytes.len()].copy_from_slice(name_bytes);
    set_color(0xCCCCCCFF,
    );
        draw_text(
        label.as_ptr(), (prefix.len() + name_bytes.len()) as u32, 10.0, 40.0, 16.0);

    // Instructions - comprehensive controls
    let hint1 = b"A: Cycle shapes | Left stick: Rotate";
    set_color(0x888888FF);
        draw_text(hint1.as_ptr(), hint1.len() as u32, 10.0, 70.0, 14.0);

    let hint2 = b"Right stick: Orbit camera";
    set_color(0x888888FF);
        draw_text(hint2.as_ptr(), hint2.len() as u32, 10.0, 90.0, 14.0);

    let hint3 = b"F4: Debug Inspector (edit material properties)";
    set_color(0x888888FF);
        draw_text(hint3.as_ptr(), hint3.len() as u32, 10.0, 110.0, 14.0);
}
