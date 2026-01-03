//! Mode 3 Inspector - Classic Blinn-Phong (Specular-Shininess)
//!
//! Demonstrates the debug inspection system for Mode 3 (Blinn-Phong) rendering.
//! Mode 3 provides classic lighting with explicit specular color control.
//!
//! Features:
//! - Shape cycling (Sphere, Cube, Torus)
//! - Material properties (shininess, specular color, emissive, rim)
//! - Sky gradient and sun configuration
//! - 2 directional lights + 2 point lights
//! - Visual spheres showing point light positions
//!
//! Usage:
//! 1. Run the game
//! 2. Press F4 to open the Debug Inspector
//! 3. Tweak values and see immediate visual changes

#![no_std]
#![no_main]

use core::panic::PanicInfo;
use examples_common::*;
use libm::{cosf, sinf};

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    core::arch::wasm32::unreachable()
}

// ============================================================================
// Debug Values - Exposed in the debug panel
// ============================================================================

// Shape settings
static mut SHAPE_INDEX: i32 = 0;
static mut ROTATION_SPEED: f32 = 30.0;
static mut OBJECT_COLOR: u32 = 0xE69933FF; // Golden base

// Material settings (Mode 3)
static mut SHININESS: f32 = 0.75;
static mut SPECULAR_COLOR: u32 = 0xFFCC66FF; // Golden specular
static mut EMISSIVE: f32 = 0.0;
static mut RIM_INTENSITY: f32 = 0.2;
static mut RIM_POWER: f32 = 0.15;

// Sky settings
static mut HORIZON_COLOR: u32 = 0xB2D8F2FF; // Light blue horizon
static mut ZENITH_COLOR: u32 = 0x3366B2FF;  // Deep blue zenith

// Sun settings
static mut SUN_DIR_X: f32 = 0.5;
static mut SUN_DIR_Y: f32 = -0.7;
static mut SUN_DIR_Z: f32 = 0.5;
static mut SUN_COLOR: u32 = 0xFFF2E6FF; // Warm white
static mut SUN_SHARPNESS: f32 = 0.98;

// Directional Light 0
static mut DIR0_ENABLED: u8 = 1;
static mut DIR0_X: f32 = -0.5;
static mut DIR0_Y: f32 = -0.8;
static mut DIR0_Z: f32 = -0.3;
static mut DIR0_COLOR: u32 = 0xFFF2E6FF; // Warm white
static mut DIR0_INTENSITY: f32 = 1.0;

// Directional Light 1
static mut DIR1_ENABLED: u8 = 0;
static mut DIR1_X: f32 = 0.7;
static mut DIR1_Y: f32 = -0.3;
static mut DIR1_Z: f32 = -0.5;
static mut DIR1_COLOR: u32 = 0x99B3FFFF; // Cool blue
static mut DIR1_INTENSITY: f32 = 0.5;

// Point Light 0
static mut POINT0_ENABLED: u8 = 1;
static mut POINT0_X: f32 = 3.0;
static mut POINT0_Y: f32 = 2.0;
static mut POINT0_Z: f32 = 0.0;
static mut POINT0_COLOR: u32 = 0xFF4D4DFF; // Red
static mut POINT0_INTENSITY: f32 = 2.0;
static mut POINT0_RANGE: f32 = 10.0;

// Point Light 1
static mut POINT1_ENABLED: u8 = 0;
static mut POINT1_X: f32 = -3.0;
static mut POINT1_Y: f32 = 1.0;
static mut POINT1_Z: f32 = 2.0;
static mut POINT1_COLOR: u32 = 0x4D4DFFFF; // Blue
static mut POINT1_INTENSITY: f32 = 2.0;
static mut POINT1_RANGE: f32 = 10.0;

// ============================================================================
// Internal State
// ============================================================================

static mut SPHERE_MESH: u32 = 0;
static mut CUBE_MESH: u32 = 0;
static mut TORUS_MESH: u32 = 0;
static mut LIGHT_INDICATOR_MESH: u32 = 0;

static mut ROTATION_X: f32 = 0.0;
static mut ROTATION_Y: f32 = 0.0;

/// Camera for orbit control
static mut CAMERA: DebugCamera = DebugCamera {
    target_x: 0.0,
    target_y: 0.0,
    target_z: 0.0,
    distance: 6.0,
    elevation: 20.0,
    azimuth: 0.0,
    auto_orbit_speed: 15.0,
    stick_control: StickControl::RightStick,
    fov: 60.0,
};

const SHAPE_NAMES: [&str; 3] = ["Sphere", "Cube", "Torus"];

// ============================================================================
// Game Implementation
// ============================================================================

#[no_mangle]
pub extern "C" fn on_debug_change() {
    unsafe {
        // Clamp shape index
        if SHAPE_INDEX < 0 { SHAPE_INDEX = 0; }
        if SHAPE_INDEX > 2 { SHAPE_INDEX = 2; }

        // Clamp material values
        SHININESS = SHININESS.clamp(0.0, 1.0);
        EMISSIVE = EMISSIVE.clamp(0.0, 2.0);
        RIM_INTENSITY = RIM_INTENSITY.clamp(0.0, 1.0);
        RIM_POWER = RIM_POWER.clamp(0.0, 1.0);

        // Clamp intensities
        DIR0_INTENSITY = DIR0_INTENSITY.clamp(0.0, 8.0);
        DIR1_INTENSITY = DIR1_INTENSITY.clamp(0.0, 8.0);
        POINT0_INTENSITY = POINT0_INTENSITY.clamp(0.0, 8.0);
        POINT1_INTENSITY = POINT1_INTENSITY.clamp(0.0, 8.0);
        POINT0_RANGE = POINT0_RANGE.clamp(0.1, 50.0);
        POINT1_RANGE = POINT1_RANGE.clamp(0.1, 50.0);
    }
}

#[no_mangle]
pub extern "C" fn init() {
    unsafe {
        set_clear_color(0x1a1a2eFF);
        render_mode(3); // Blinn-Phong (Specular-Shininess)

        // Generate meshes
        SPHERE_MESH = sphere(1.5, 32, 16);
        CUBE_MESH = cube(1.2, 1.2, 1.2);
        TORUS_MESH = torus(1.2, 0.5, 32, 16);
        LIGHT_INDICATOR_MESH = sphere(0.15, 8, 8);

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

    // Material group (Mode 3 specific)
    debug_group_begin(b"material".as_ptr(), 8);
    debug_register_f32(b"shininess".as_ptr(), 9, &SHININESS as *const f32 as *const u8);
    debug_register_color(b"specular_color".as_ptr(), 14, &SPECULAR_COLOR as *const u32 as *const u8);
    debug_register_f32(b"emissive".as_ptr(), 8, &EMISSIVE as *const f32 as *const u8);
    debug_register_f32(b"rim_intensity".as_ptr(), 13, &RIM_INTENSITY as *const f32 as *const u8);
    debug_register_f32(b"rim_power".as_ptr(), 9, &RIM_POWER as *const f32 as *const u8);
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

    // Directional Light 0
    debug_group_begin(b"dir_light_0".as_ptr(), 11);
    debug_register_bool(b"enabled".as_ptr(), 7, &DIR0_ENABLED);
    debug_register_f32(b"dir_x".as_ptr(), 5, &DIR0_X as *const f32 as *const u8);
    debug_register_f32(b"dir_y".as_ptr(), 5, &DIR0_Y as *const f32 as *const u8);
    debug_register_f32(b"dir_z".as_ptr(), 5, &DIR0_Z as *const f32 as *const u8);
    debug_register_color(b"color".as_ptr(), 5, &DIR0_COLOR as *const u32 as *const u8);
    debug_register_f32(b"intensity".as_ptr(), 9, &DIR0_INTENSITY as *const f32 as *const u8);
    debug_group_end();

    // Directional Light 1
    debug_group_begin(b"dir_light_1".as_ptr(), 11);
    debug_register_bool(b"enabled".as_ptr(), 7, &DIR1_ENABLED);
    debug_register_f32(b"dir_x".as_ptr(), 5, &DIR1_X as *const f32 as *const u8);
    debug_register_f32(b"dir_y".as_ptr(), 5, &DIR1_Y as *const f32 as *const u8);
    debug_register_f32(b"dir_z".as_ptr(), 5, &DIR1_Z as *const f32 as *const u8);
    debug_register_color(b"color".as_ptr(), 5, &DIR1_COLOR as *const u32 as *const u8);
    debug_register_f32(b"intensity".as_ptr(), 9, &DIR1_INTENSITY as *const f32 as *const u8);
    debug_group_end();

    // Point Light 0
    debug_group_begin(b"point_light_0".as_ptr(), 13);
    debug_register_bool(b"enabled".as_ptr(), 7, &POINT0_ENABLED);
    debug_register_f32(b"pos_x".as_ptr(), 5, &POINT0_X as *const f32 as *const u8);
    debug_register_f32(b"pos_y".as_ptr(), 5, &POINT0_Y as *const f32 as *const u8);
    debug_register_f32(b"pos_z".as_ptr(), 5, &POINT0_Z as *const f32 as *const u8);
    debug_register_color(b"color".as_ptr(), 5, &POINT0_COLOR as *const u32 as *const u8);
    debug_register_f32(b"intensity".as_ptr(), 9, &POINT0_INTENSITY as *const f32 as *const u8);
    debug_register_f32(b"range".as_ptr(), 5, &POINT0_RANGE as *const f32 as *const u8);
    debug_group_end();

    // Point Light 1
    debug_group_begin(b"point_light_1".as_ptr(), 13);
    debug_register_bool(b"enabled".as_ptr(), 7, &POINT1_ENABLED);
    debug_register_f32(b"pos_x".as_ptr(), 5, &POINT1_X as *const f32 as *const u8);
    debug_register_f32(b"pos_y".as_ptr(), 5, &POINT1_Y as *const f32 as *const u8);
    debug_register_f32(b"pos_z".as_ptr(), 5, &POINT1_Z as *const f32 as *const u8);
    debug_register_color(b"color".as_ptr(), 5, &POINT1_COLOR as *const u32 as *const u8);
    debug_register_f32(b"intensity".as_ptr(), 9, &POINT1_INTENSITY as *const f32 as *const u8);
    debug_register_f32(b"range".as_ptr(), 5, &POINT1_RANGE as *const f32 as *const u8);
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
            ROTATION_Y += ROTATION_SPEED * (1.0 / 60.0);
        }

        // Update camera
        CAMERA.update();
    }
}

#[no_mangle]
pub extern "C" fn render() {
    unsafe {
        // Apply camera
        CAMERA.apply();

        // Configure and draw environment
        env_gradient(0, ZENITH_COLOR, HORIZON_COLOR, HORIZON_COLOR, 0x2A2A2AFF, 0.0, 0.0);
        light_set(0, SUN_DIR_X, SUN_DIR_Y, SUN_DIR_Z);
        light_color(0, SUN_COLOR);
        light_intensity(0, 1.0);
        draw_env();

        // Apply lights
        apply_lights();

        // Apply material (Mode 3 specific)
        material_shininess(SHININESS);
        material_specular(SPECULAR_COLOR);
        material_emissive(EMISSIVE);
        material_rim(RIM_INTENSITY, RIM_POWER);

        // Draw main shape
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

        // Draw light indicators
        draw_light_indicators();

        // Draw UI
        draw_ui();
    }
}

unsafe fn apply_lights() {
    // Directional Light 0
    if DIR0_ENABLED != 0 {
        light_set(0, DIR0_X, DIR0_Y, DIR0_Z);
        light_color(0, DIR0_COLOR);
        light_intensity(0, DIR0_INTENSITY);
        light_enable(0);
    } else {
        light_disable(0);
    }

    // Directional Light 1
    if DIR1_ENABLED != 0 {
        light_set(1, DIR1_X, DIR1_Y, DIR1_Z);
        light_color(1, DIR1_COLOR);
        light_intensity(1, DIR1_INTENSITY);
        light_enable(1);
    } else {
        light_disable(1);
    }

    // Point Light 0
    if POINT0_ENABLED != 0 {
        light_set_point(2, POINT0_X, POINT0_Y, POINT0_Z);
        light_color(2, POINT0_COLOR);
        light_intensity(2, POINT0_INTENSITY);
        light_range(2, POINT0_RANGE);
        light_enable(2);
    } else {
        light_disable(2);
    }

    // Point Light 1
    if POINT1_ENABLED != 0 {
        light_set_point(3, POINT1_X, POINT1_Y, POINT1_Z);
        light_color(3, POINT1_COLOR);
        light_intensity(3, POINT1_INTENSITY);
        light_range(3, POINT1_RANGE);
        light_enable(3);
    } else {
        light_disable(3);
    }
}

unsafe fn draw_light_indicators() {
    // Reset material to emissive-only for light indicators
    material_shininess(0.0);

    // Point Light 0 indicator
    if POINT0_ENABLED != 0 {
        material_emissive(2.0);
        push_identity();
        push_translate(POINT0_X, POINT0_Y, POINT0_Z);
        set_color(POINT0_COLOR);
        draw_mesh(LIGHT_INDICATOR_MESH);
    }

    // Point Light 1 indicator
    if POINT1_ENABLED != 0 {
        material_emissive(2.0);
        push_identity();
        push_translate(POINT1_X, POINT1_Y, POINT1_Z);
        set_color(POINT1_COLOR);
        draw_mesh(LIGHT_INDICATOR_MESH);
    }

    // Reset emissive
    material_emissive(0.0);
}

unsafe fn draw_ui() {
    // Title
    let title = b"Mode 3: Blinn-Phong (Specular-Shininess)";
    set_color(0xFFFFFFFF);
        draw_text(title.as_ptr(), title.len() as u32, 10.0, 10.0, 20.0);

    // Current shape
    let shape_name = SHAPE_NAMES[SHAPE_INDEX as usize];
    let mut label = [0u8; 32];
    let prefix = b"Shape: ";
    label[..prefix.len()].copy_from_slice(prefix);
    let name_bytes = shape_name.as_bytes();
    label[prefix.len()..prefix.len() + name_bytes.len()].copy_from_slice(name_bytes);
    set_color(0xCCCCCCFF);
        draw_text(label.as_ptr(), (prefix.len() + name_bytes.len()) as u32, 10.0, 40.0, 16.0);

    // Material info
    let mat_info = b"Explicit specular color control";
    set_color(0xFFAAAAFF);
        draw_text(mat_info.as_ptr(), mat_info.len() as u32, 10.0, 60.0, 14.0);

    // Instructions - comprehensive controls
    let hint1 = b"A: Cycle shapes | Left stick: Rotate";
    set_color(0x888888FF);
        draw_text(hint1.as_ptr(), hint1.len() as u32, 10.0, 90.0, 14.0);

    let hint2 = b"Right stick: Orbit camera";
    set_color(0x888888FF);
        draw_text(hint2.as_ptr(), hint2.len() as u32, 10.0, 110.0, 14.0);

    let hint3 = b"F4: Debug Inspector (edit specular, shininess, lights)";
    set_color(0x888888FF);
        draw_text(hint3.as_ptr(), hint3.len() as u32, 10.0, 130.0, 14.0);
}
