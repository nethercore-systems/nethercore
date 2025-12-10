//! Debug Inspection Demo
//!
//! Demonstrates the debug inspection system for runtime value tweaking.
//!
//! Features:
//! - Register values for debug inspection via FFI
//! - Organize values into groups (player, world, effects)
//! - Various value types: f32, i32, bool, Vec2, Color
//! - Visual feedback showing effect of value changes
//!
//! Usage:
//! 1. Run the game
//! 2. Press F3 to open the debug panel
//! 3. Tweak values and see immediate visual changes
//! 4. Press F5 to pause, F6 to step frame
//! 5. Press F7/F8 to change time scale

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

    // Camera
    fn camera_set(x: f32, y: f32, z: f32, target_x: f32, target_y: f32, target_z: f32);
    fn camera_fov(fov_degrees: f32);

    // Procedural mesh generation
    fn cube(size_x: f32, size_y: f32, size_z: f32) -> u32;
    fn sphere(radius: f32, subdivisions: u32) -> u32;

    // Mesh drawing
    fn draw_mesh(handle: u32);

    // Transform
    fn push_identity();
    fn push_translate(x: f32, y: f32, z: f32);
    fn push_rotate_y(angle_deg: f32);
    fn push_scale(x: f32, y: f32, z: f32);
    fn pop_transform();

    // Render state
    fn set_color(color: u32);
    fn depth_test(enabled: u32);

    // Time
    fn elapsed_time() -> f32;

    // Debug inspection FFI (compile out in release builds)
    fn debug_group_begin(name: *const u8);
    fn debug_group_end();
    fn debug_register_f32(name: *const u8, ptr: *const f32);
    fn debug_register_i32(name: *const u8, ptr: *const i32);
    fn debug_register_bool(name: *const u8, ptr: *const u8);
    fn debug_register_color(name: *const u8, ptr: *const u8);
}

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
static mut PLAYER_COLOR: [u8; 4] = [255, 100, 100, 255]; // RGBA
static mut ORBIT_COLOR: [u8; 4] = [100, 100, 255, 255];

// Mesh handles
static mut CUBE_MESH: u32 = 0;
static mut SPHERE_MESH: u32 = 0;

// ============================================================================
// Game Implementation
// ============================================================================

#[no_mangle]
pub extern "C" fn init() {
    unsafe {
        // Dark background
        set_clear_color(0x1a1a2eFF);

        // Set up camera
        camera_set(0.0, 4.0, 8.0, 0.0, 0.0, 0.0);
        camera_fov(60.0);

        // Enable depth testing
        depth_test(1);

        // Generate meshes
        CUBE_MESH = cube(1.0, 1.0, 1.0);
        SPHERE_MESH = sphere(0.5, 2);

        // Register debug values
        register_debug_values();
    }
}

/// Register all tweakable values with the debug inspection system
unsafe fn register_debug_values() {
    // Player group
    debug_group_begin(b"player\0".as_ptr());
    debug_register_f32(b"x\0".as_ptr(), &PLAYER_X);
    debug_register_f32(b"y\0".as_ptr(), &PLAYER_Y);
    debug_register_f32(b"speed\0".as_ptr(), &PLAYER_SPEED);
    debug_register_f32(b"scale\0".as_ptr(), &PLAYER_SCALE);
    debug_register_color(b"color\0".as_ptr(), PLAYER_COLOR.as_ptr());
    debug_group_end();

    // World group
    debug_group_begin(b"world\0".as_ptr());
    debug_register_f32(b"rotation_speed\0".as_ptr(), &ROTATION_SPEED);
    debug_register_i32(b"object_count\0".as_ptr(), &OBJECT_COUNT);
    debug_register_f32(b"orbit_radius\0".as_ptr(), &ORBIT_RADIUS);
    debug_group_end();

    // Effects group
    debug_group_begin(b"effects\0".as_ptr());
    debug_register_bool(b"enable_rotation\0".as_ptr(), &ENABLE_ROTATION);
    debug_register_color(b"orbit_color\0".as_ptr(), ORBIT_COLOR.as_ptr());
    debug_group_end();
}

#[no_mangle]
pub extern "C" fn update() {
    // No update logic needed - all values are updated via debug panel
}

#[no_mangle]
pub extern "C" fn render() {
    unsafe {
        let time = elapsed_time();

        // Draw player cube at center
        push_identity();
        push_translate(PLAYER_X, PLAYER_Y, 0.0);
        push_scale(PLAYER_SCALE, PLAYER_SCALE, PLAYER_SCALE);

        // Convert RGBA to u32 color
        let player_color = color_to_u32(&PLAYER_COLOR);
        set_color(player_color);
        draw_mesh(CUBE_MESH);
        pop_transform();
        pop_transform();
        pop_transform();

        // Draw orbiting spheres
        let orbit_color = color_to_u32(&ORBIT_COLOR);
        set_color(orbit_color);

        let count = OBJECT_COUNT.max(0).min(8); // Clamp to reasonable range
        for i in 0..count {
            let angle_offset = (i as f32 / count as f32) * 360.0;
            let angle = if ENABLE_ROTATION != 0 {
                angle_offset + time * ROTATION_SPEED
            } else {
                angle_offset
            };

            let rad = angle * core::f32::consts::PI / 180.0;
            let x = ORBIT_RADIUS * cos_approx(rad);
            let z = ORBIT_RADIUS * sin_approx(rad);

            push_identity();
            push_translate(x, 0.0, z);
            draw_mesh(SPHERE_MESH);
            pop_transform();
            pop_transform();
        }
    }
}

/// Convert RGBA bytes to u32 color
fn color_to_u32(rgba: &[u8; 4]) -> u32 {
    ((rgba[0] as u32) << 24)
        | ((rgba[1] as u32) << 16)
        | ((rgba[2] as u32) << 8)
        | (rgba[3] as u32)
}

/// Simple sine approximation (Taylor series, good enough for visuals)
fn sin_approx(x: f32) -> f32 {
    // Normalize to [-PI, PI]
    let pi = core::f32::consts::PI;
    let mut x = x % (2.0 * pi);
    if x > pi {
        x -= 2.0 * pi;
    } else if x < -pi {
        x += 2.0 * pi;
    }
    // Taylor series: sin(x) ≈ x - x³/6 + x⁵/120
    let x2 = x * x;
    let x3 = x2 * x;
    let x5 = x3 * x2;
    x - x3 / 6.0 + x5 / 120.0
}

/// Simple cosine approximation
fn cos_approx(x: f32) -> f32 {
    sin_approx(x + core::f32::consts::FRAC_PI_2)
}
