//! Environment Inspector - Multi-Environment v3 Demo
//!
//! Demonstrates the new procedural environment system with gradient presets.
//!
//! Features:
//! - 3 shapes (sphere, cube, torus) to show environment reflections
//! - A/B buttons to cycle through gradient presets
//! - X button to cycle shapes
//! - Debug panel for real-time parameter tweaking
//!
//! Presets:
//! - Blue Sky: Classic blue atmosphere
//! - Sunset: Warm orange/purple gradient
//! - Underwater: Teal/turquoise depths
//! - Night: Dark starless sky
//! - Vapor: Synthwave magenta/cyan
//! - Desert: Dusty sandy atmosphere
//!
//! Usage:
//! 1. Run the game
//! 2. Press A/B to cycle presets forward/backward
//! 3. Press X to cycle shapes
//! 4. Press F4 to open debug panel for fine-tuning
//! 5. Left stick to rotate camera around object

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
    fn button_pressed(player: u32, button: u32) -> u32;

    // Time
    fn elapsed_time() -> f32;

    // Procedural mesh generation
    fn sphere(radius: f32, segments: u32, rings: u32) -> u32;
    fn cube(size_x: f32, size_y: f32, size_z: f32) -> u32;
    fn torus(major_radius: f32, minor_radius: f32, major_segments: u32, minor_segments: u32) -> u32;

    // Mesh drawing
    fn draw_mesh(handle: u32);

    // Transform
    fn push_identity();
    fn push_translate(x: f32, y: f32, z: f32);
    fn push_scale(x: f32, y: f32, z: f32);

    // Render state
    fn set_color(color: u32);
    fn depth_test(enabled: u32);
    fn cull_mode(mode: u32);

    // Environment (Multi-Environment v3)
    fn env_gradient(
        layer: u32,
        zenith: u32,
        sky_horizon: u32,
        ground_horizon: u32,
        nadir: u32,
        rotation: f32,
        shift: f32,
    );
    fn env_blend(mode: u32);
    fn draw_env();

    // Materials (Mode 2 for reflections)
    fn material_metallic(value: f32);
    fn material_roughness(value: f32);

    // 2D UI
    fn draw_text(ptr: *const u8, len: u32, x: f32, y: f32, size: f32, color: u32);

    // Debug inspection
    fn debug_group_begin(name: *const u8, name_len: u32);
    fn debug_group_end();
    fn debug_register_f32(name: *const u8, name_len: u32, ptr: *const f32);
    fn debug_register_i32(name: *const u8, name_len: u32, ptr: *const i32);
    fn debug_register_u8(name: *const u8, name_len: u32, ptr: *const u8);
    fn debug_register_color(name: *const u8, name_len: u32, ptr: *const u8);
}

// Button constants
const BUTTON_A: u32 = 4;
const BUTTON_B: u32 = 5;
const BUTTON_X: u32 = 6;

// Shape constants
const SHAPE_COUNT: usize = 3;
const SHAPE_NAMES: [&str; SHAPE_COUNT] = ["Sphere", "Cube", "Torus"];

// ============================================================================
// Environment Presets
// ============================================================================

const PRESET_COUNT: usize = 6;

const PRESET_NAMES: [&str; PRESET_COUNT] = [
    "Blue Sky",
    "Sunset",
    "Underwater",
    "Night",
    "Vapor",
    "Desert",
];

// Gradient colors: [zenith, sky_horizon, ground_horizon, nadir]
const PRESET_COLORS: [[u32; 4]; PRESET_COUNT] = [
    // Blue Sky
    [0x191970FF, 0x87CEEBFF, 0x228B22FF, 0x2F4F4FFF],
    // Sunset
    [0x4A00E0FF, 0xFF6B6BFF, 0x8B4513FF, 0x2F2F2FFF],
    // Underwater
    [0x006994FF, 0x40E0D0FF, 0x20B2AAFF, 0x003366FF],
    // Night
    [0x03030DFF, 0x0D0D1AFF, 0x0A0A14FF, 0x000000FF],
    // Vapor
    [0xFF00FFFF, 0x00FFFFFF, 0x8800FFFF, 0x000033FF],
    // Desert
    [0x4682B4FF, 0xFFE4B5FF, 0xD2B48CFF, 0x8B7355FF],
];

// Preset shifts
const PRESET_SHIFTS: [f32; PRESET_COUNT] = [
    0.0,  // Blue Sky
    0.1,  // Sunset
    -0.1, // Underwater
    0.0,  // Night
    0.0,  // Vapor
    0.05, // Desert
];

// ============================================================================
// Debug Values - Exposed in the debug panel
// ============================================================================

// Environment colors (editable)
static mut ZENITH_COLOR: u32 = 0x191970FF;
static mut SKY_HORIZON_COLOR: u32 = 0x87CEEBFF;
static mut GROUND_HORIZON_COLOR: u32 = 0x228B22FF;
static mut NADIR_COLOR: u32 = 0x2F4F4FFF;
static mut ROTATION: f32 = 0.0;
static mut SHIFT: f32 = 0.0;

// Shape material (u8 for easier debug panel editing: 0-255 maps to 0.0-1.0)
static mut METALLIC: u8 = 200;    // ~0.78
static mut ROUGHNESS: u8 = 50;    // ~0.20
static mut SHAPE_COLOR: u32 = 0xCCCCCCFF;

// Shape selection
static mut SHAPE_INDEX: i32 = 0;

// ============================================================================
// Internal State
// ============================================================================

static mut PRESET_INDEX: i32 = 0;
static mut SPHERE_MESH: u32 = 0;
static mut CUBE_MESH: u32 = 0;
static mut TORUS_MESH: u32 = 0;

// Camera orbit state
static mut CAM_ANGLE: f32 = 0.0;
static mut CAM_ELEVATION: f32 = 20.0;
static mut CAM_DISTANCE: f32 = 6.0;

// ============================================================================
// Game Implementation
// ============================================================================

#[no_mangle]
pub extern "C" fn on_debug_change() {
    unsafe {
        // Clamp shape index to valid range
        if SHAPE_INDEX < 0 {
            SHAPE_INDEX = 0;
        }
        if SHAPE_INDEX >= SHAPE_COUNT as i32 {
            SHAPE_INDEX = SHAPE_COUNT as i32 - 1;
        }
        // Clamp preset index to valid range
        if PRESET_INDEX < 0 {
            PRESET_INDEX = 0;
        }
        if PRESET_INDEX >= PRESET_COUNT as i32 {
            PRESET_INDEX = PRESET_COUNT as i32 - 1;
        }
    }
}

fn load_preset(index: usize) {
    unsafe {
        let colors = PRESET_COLORS[index];
        ZENITH_COLOR = colors[0];
        SKY_HORIZON_COLOR = colors[1];
        GROUND_HORIZON_COLOR = colors[2];
        NADIR_COLOR = colors[3];
        SHIFT = PRESET_SHIFTS[index];
        ROTATION = 0.0;
    }
}

#[no_mangle]
pub extern "C" fn init() {
    unsafe {
        // Dark clear color
        set_clear_color(0x0A0A0AFF);

        // Mode 2 (PBR) for reflections
        render_mode(2);

        // Enable depth testing
        depth_test(1);

        // Generate meshes
        SPHERE_MESH = sphere(1.5, 32, 24);
        CUBE_MESH = cube(2.0, 2.0, 2.0);
        TORUS_MESH = torus(1.2, 0.5, 32, 16);

        // Load initial preset
        load_preset(0);

        // Register debug values
        register_debug_values();
    }
}

unsafe fn register_debug_values() {
    // Environment group
    debug_group_begin(b"environment".as_ptr(), 11);
    debug_register_color(b"zenith".as_ptr(), 6, &ZENITH_COLOR as *const u32 as *const u8);
    debug_register_color(b"sky_horizon".as_ptr(), 11, &SKY_HORIZON_COLOR as *const u32 as *const u8);
    debug_register_color(b"ground_horizon".as_ptr(), 14, &GROUND_HORIZON_COLOR as *const u32 as *const u8);
    debug_register_color(b"nadir".as_ptr(), 5, &NADIR_COLOR as *const u32 as *const u8);
    debug_register_f32(b"rotation".as_ptr(), 8, &ROTATION);
    debug_register_f32(b"shift".as_ptr(), 5, &SHIFT);
    debug_group_end();

    // Material group (u8 values: 0-255 maps to 0.0-1.0)
    debug_group_begin(b"material".as_ptr(), 8);
    debug_register_u8(b"metallic".as_ptr(), 8, &METALLIC);
    debug_register_u8(b"roughness".as_ptr(), 9, &ROUGHNESS);
    debug_register_color(b"color".as_ptr(), 5, &SHAPE_COLOR as *const u32 as *const u8);
    debug_group_end();

    // Shape group
    debug_group_begin(b"shape".as_ptr(), 5);
    debug_register_i32(b"index (0-2)".as_ptr(), 11, &SHAPE_INDEX);
    debug_group_end();

    // Preset group
    debug_group_begin(b"preset".as_ptr(), 6);
    debug_register_i32(b"index".as_ptr(), 5, &PRESET_INDEX);
    debug_group_end();
}

#[no_mangle]
pub extern "C" fn update() {
    unsafe {
        // Preset cycling
        if button_pressed(0, BUTTON_A) != 0 {
            PRESET_INDEX = (PRESET_INDEX + 1) % PRESET_COUNT as i32;
            load_preset(PRESET_INDEX as usize);
        }
        if button_pressed(0, BUTTON_B) != 0 {
            PRESET_INDEX = if PRESET_INDEX == 0 {
                PRESET_COUNT as i32 - 1
            } else {
                PRESET_INDEX - 1
            };
            load_preset(PRESET_INDEX as usize);
        }

        // Shape cycling
        if button_pressed(0, BUTTON_X) != 0 {
            SHAPE_INDEX = (SHAPE_INDEX + 1) % SHAPE_COUNT as i32;
        }

        // Camera orbit via left stick
        let stick_x = left_stick_x(0);
        let stick_y = left_stick_y(0);

        if stick_x.abs() > 0.1 {
            CAM_ANGLE += stick_x * 2.0;
        }
        if stick_y.abs() > 0.1 {
            CAM_ELEVATION = (CAM_ELEVATION - stick_y * 2.0).clamp(-20.0, 60.0);
        }
    }
}

#[no_mangle]
pub extern "C" fn render() {
    unsafe {
        // Calculate camera position from orbit
        let angle_rad = CAM_ANGLE * core::f32::consts::PI / 180.0;
        let elev_rad = CAM_ELEVATION * core::f32::consts::PI / 180.0;

        let cos_elev = libm::cosf(elev_rad);
        let sin_elev = libm::sinf(elev_rad);
        let cos_angle = libm::cosf(angle_rad);
        let sin_angle = libm::sinf(angle_rad);

        let cam_x = CAM_DISTANCE * cos_elev * sin_angle;
        let cam_y = CAM_DISTANCE * sin_elev;
        let cam_z = CAM_DISTANCE * cos_elev * cos_angle;

        camera_set(cam_x, cam_y, cam_z, 0.0, 0.0, 0.0);
        camera_fov(60.0);

        // Configure environment
        env_blend(0);     // Alpha blend (not used with single layer)
        env_gradient(
            0, // base layer
            ZENITH_COLOR,
            SKY_HORIZON_COLOR,
            GROUND_HORIZON_COLOR,
            NADIR_COLOR,
            ROTATION,
            SHIFT,
        );

        // Draw environment
        draw_env();

        // Select mesh based on shape index
        let mesh = match SHAPE_INDEX {
            0 => SPHERE_MESH,
            1 => CUBE_MESH,
            2 => TORUS_MESH,
            _ => SPHERE_MESH,
        };

        // Draw reflective shape at origin
        push_identity();
        set_color(SHAPE_COLOR);
        // Convert u8 (0-255) to f32 (0.0-1.0)
        material_metallic(METALLIC as f32 / 255.0);
        material_roughness(ROUGHNESS as f32 / 255.0);
        draw_mesh(mesh);

        // Draw UI
        draw_ui();
    }
}

unsafe fn draw_ui() {
    // Title
    let title = b"Environment Inspector";
    draw_text(title.as_ptr(), title.len() as u32, 10.0, 10.0, 20.0, 0xFFFFFFFF);

    // Current preset name
    let preset_name = PRESET_NAMES[PRESET_INDEX as usize];
    let mut label = [0u8; 48];
    let prefix = b"Preset: ";
    label[..prefix.len()].copy_from_slice(prefix);
    let name_bytes = preset_name.as_bytes();
    label[prefix.len()..prefix.len() + name_bytes.len()].copy_from_slice(name_bytes);
    draw_text(
        label.as_ptr(),
        (prefix.len() + name_bytes.len()) as u32,
        10.0,
        40.0,
        16.0,
        0xCCCCCCFF,
    );

    // Current shape name
    let shape_name = SHAPE_NAMES[SHAPE_INDEX as usize];
    let mut shape_label = [0u8; 32];
    let shape_prefix = b"Shape: ";
    shape_label[..shape_prefix.len()].copy_from_slice(shape_prefix);
    let shape_name_bytes = shape_name.as_bytes();
    shape_label[shape_prefix.len()..shape_prefix.len() + shape_name_bytes.len()].copy_from_slice(shape_name_bytes);
    draw_text(
        shape_label.as_ptr(),
        (shape_prefix.len() + shape_name_bytes.len()) as u32,
        10.0,
        60.0,
        16.0,
        0xCCCCCCFF,
    );

    // Instructions
    let hint1 = b"A/B: Cycle presets";
    draw_text(hint1.as_ptr(), hint1.len() as u32, 10.0, 90.0, 14.0, 0x888888FF);

    let hint2 = b"X: Cycle shapes";
    draw_text(hint2.as_ptr(), hint2.len() as u32, 10.0, 110.0, 14.0, 0x888888FF);

    let hint3 = b"Left stick: Orbit camera";
    draw_text(hint3.as_ptr(), hint3.len() as u32, 10.0, 130.0, 14.0, 0x888888FF);

    let hint4 = b"F4: Debug panel";
    draw_text(hint4.as_ptr(), hint4.len() as u32, 10.0, 150.0, 14.0, 0x888888FF);
}
