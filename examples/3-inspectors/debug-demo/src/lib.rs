//! Debug Inspection Demo
//!
//! Demonstrates the debug inspection system for runtime value tweaking.
//!
//! Features:
//! - Register values for debug inspection via FFI
//! - Organize values into groups (player, world, effects)
//! - Various value types: f32, i32, bool, Vec2, Color
//! - Action buttons that call WASM functions when clicked
//! - Change callback for derived value updates
//! - Visual feedback showing effect of value changes
//!
//! Usage:
//! 1. Run the game
//! 2. Press F4 to open the debug panel
//! 3. Tweak values and see immediate visual changes
//! 4. Click action buttons to trigger functions
//! 5. Press F5 to pause, F6 to step frame
//! 6. Press F7/F8 to change time scale

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
// Debug Values - These will be exposed in the debug panel
// ============================================================================

// Player settings
static mut PLAYER_X: f32 = 0.0;
static mut PLAYER_Y: f32 = 0.0;
static mut PLAYER_SPEED: f32 = 2.0;
static mut PLAYER_SCALE: f32 = 1.0;

// World settings
static mut ROTATION_SPEED: f32 = 45.0; // degrees per second
static mut OBJECT_COUNT: i32 = 3;
static mut ORBIT_RADIUS: f32 = 3.0;

// Visual effects
static mut ENABLE_ROTATION: u8 = 1; // bool stored as u8
static mut PLAYER_COLOR: u32 = 0xFF6464FF; // Red
static mut ORBIT_COLOR: u32 = 0x6464FFFF; // Blue

// Derived/computed values (updated by change callback)
static mut CHANGE_COUNT: i32 = 0; // Tracks how many times debug values changed
static mut TOTAL_SPHERES: i32 = 3; // Derived from OBJECT_COUNT (clamped)

// Action button counter
static mut ACTION_CLICK_COUNT: i32 = 0; // Tracks clicks on the simple action button

// Mesh handles
static mut CUBE_MESH: u32 = 0;
static mut SPHERE_MESH: u32 = 0;

// ============================================================================
// Game Implementation
// ============================================================================

/// Optional callback invoked when any debug value changes
/// Updates derived values that depend on debug-tweakable values
///
/// This is automatically called by the console when debug values are modified.
/// Export this function to react to changes (similar to init/update/render).
#[no_mangle]
pub extern "C" fn on_debug_change() {
    unsafe {
        // Track how many times values have been changed
        CHANGE_COUNT += 1;

        // Recalculate derived values
        // TOTAL_SPHERES is OBJECT_COUNT clamped to valid range
        TOTAL_SPHERES = OBJECT_COUNT.max(0).min(8);
    }
}

// ============================================================================
// Debug Action Functions - Called when action buttons are clicked
// ============================================================================

/// Simple action with no parameters
/// Increments a click counter each time it's called
#[no_mangle]
pub extern "C" fn debug_function() {
    unsafe {
        ACTION_CLICK_COUNT += 1;
    }
}

/// Action with parameters demonstrating the param system
/// Sets the player position to the given x and y coordinates
#[no_mangle]
pub extern "C" fn set_position(x: i32, y: i32) {
    unsafe {
        PLAYER_X = x as f32;
        PLAYER_Y = y as f32;
    }
}

#[no_mangle]
pub extern "C" fn init() {
    unsafe {
        // Dark background
        set_clear_color(0x1a1a2eFF);

        // Generate meshes
        CUBE_MESH = cube(1.0, 1.0, 1.0);
        SPHERE_MESH = sphere(0.5, 8, 8);

        // Register debug values
        register_debug_values();
    }
}

/// Register all tweakable values with the debug inspection system
unsafe fn register_debug_values() {
    // Player group
    debug_group_begin(b"player".as_ptr(), 6);
    debug_register_f32(b"x".as_ptr(), 1, &PLAYER_X as *const f32 as *const u8);
    debug_register_f32(b"y".as_ptr(), 1, &PLAYER_Y as *const f32 as *const u8);
    debug_register_f32(b"speed".as_ptr(), 5, &PLAYER_SPEED as *const f32 as *const u8);
    debug_register_f32(b"scale".as_ptr(), 5, &PLAYER_SCALE as *const f32 as *const u8);
    debug_register_color(b"color".as_ptr(), 5, &PLAYER_COLOR as *const u32 as *const u8);
    debug_group_end();

    // World group
    debug_group_begin(b"world".as_ptr(), 5);
    debug_register_f32(b"rotation_speed".as_ptr(), 14, &ROTATION_SPEED as *const f32 as *const u8);
    debug_register_i32(b"object_count".as_ptr(), 12, &OBJECT_COUNT as *const i32 as *const u8);
    debug_register_f32(b"orbit_radius".as_ptr(), 12, &ORBIT_RADIUS as *const f32 as *const u8);
    debug_group_end();

    // Effects group
    debug_group_begin(b"effects".as_ptr(), 7);
    debug_register_bool(b"enable_rotation".as_ptr(), 15, &ENABLE_ROTATION);
    debug_register_color(b"orbit_color".as_ptr(), 11, &ORBIT_COLOR as *const u32 as *const u8);
    debug_group_end();

    // Stats group (derived values, updated by change callback)
    // These use debug_watch_* (read-only) since they're computed, not editable
    debug_group_begin(b"stats".as_ptr(), 5);
    debug_watch_i32(b"change_count".as_ptr(), 12, &CHANGE_COUNT as *const i32 as *const u8);
    debug_watch_i32(b"total_spheres".as_ptr(), 13, &TOTAL_SPHERES as *const i32 as *const u8);
    debug_group_end();

    // Actions group - demonstrates debug action buttons
    debug_group_begin(b"actions".as_ptr(), 7);

    // Simple action with no parameters - just click and it calls the function
    debug_register_action(
        b"Debug Function".as_ptr(),
        14,
        b"debug_function".as_ptr(),
        14,
    );

    // Action with parameters - shows input fields before the button
    // Clicking this will move the player cube to the specified position
    debug_action_begin(b"Set Position".as_ptr(), 12, b"set_position".as_ptr(), 12);
    debug_action_param_i32(b"x".as_ptr(), 1, 0);
    debug_action_param_i32(b"y".as_ptr(), 1, 2);
    debug_action_end();

    // Watch value to show action results
    debug_watch_i32(b"click_count".as_ptr(), 11, &ACTION_CLICK_COUNT as *const i32 as *const u8);

    debug_group_end();
}

#[no_mangle]
pub extern "C" fn update() {
    // No update logic needed - all values are updated via debug panel
}

#[no_mangle]
pub extern "C" fn render() {
    unsafe {
        // Set camera every frame (immediate mode)
        camera_set(0.0, 4.0, 8.0, 0.0, 0.0, 0.0);
        camera_fov(60.0);

        let time = elapsed_time();

        // Draw player cube at center
        // push_identity() resets transform, then push_* compose onto it
        push_identity();
        push_translate(PLAYER_X, PLAYER_Y, 0.0);
        push_scale(PLAYER_SCALE, PLAYER_SCALE, PLAYER_SCALE);

        set_color(PLAYER_COLOR);
        draw_mesh(CUBE_MESH);

        // Draw orbiting spheres
        set_color(ORBIT_COLOR);

        let count = OBJECT_COUNT.max(0).min(8); // Clamp to reasonable range
        for i in 0..count {
            let angle_offset = (i as f32 / count as f32) * 360.0;
            let angle = if ENABLE_ROTATION != 0 {
                angle_offset + time * ROTATION_SPEED
            } else {
                angle_offset
            };

            let rad = angle * core::f32::consts::PI / 180.0;
            let x = ORBIT_RADIUS * libm::cosf(rad);
            let z = ORBIT_RADIUS * libm::sinf(rad);

            // Each push_identity() resets for a new object
            push_identity();
            push_translate(x, 0.0, z);
            draw_mesh(SPHERE_MESH);
        }
    }
}
