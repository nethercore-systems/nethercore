//! EPU Multi Reflections
//!
//! Enhanced demo showing multiple EPU environments with orbit camera and interactive controls.
//!
//! Features:
//! - Four distinct EPU environment presets (Neon City, Ember Glow, Frozen, Void)
//! - Orbit camera with right stick + triggers for zoom
//! - Shape cycling (Sphere, Cube, Torus)
//! - Environment cycling per object
//! - Auto-rotation toggle
//!
//! Controls:
//! - Right Stick: Orbit camera
//! - Triggers: Zoom in/out
//! - D-Pad Left/Right: Cycle left object environment
//! - D-Pad Up/Down: Cycle right object environment
//! - A: Cycle both shapes
//! - X: Cycle left shape only
//! - Y: Cycle right shape only
//! - B: Toggle auto-rotation

#![no_std]
#![no_main]

use core::panic::PanicInfo;
use examples_common::*;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    core::arch::wasm32::unreachable()
}

// =============================================================================
// EPU Constants
// =============================================================================

const fn epu_hi(
    opcode: u64,
    region: u64,
    blend: u64,
    meta5: u64,
    color_a: u64,
    color_b: u64,
) -> u64 {
    ((opcode & 0x1F) << 59)
        | ((region & 0x7) << 56)
        | ((blend & 0x7) << 53)
        | ((meta5 & 0x1F) << 48)
        | ((color_a & 0xFFFFFF) << 24)
        | (color_b & 0xFFFFFF)
}

const fn epu_lo(
    intensity: u64,
    param_a: u64,
    param_b: u64,
    param_c: u64,
    param_d: u64,
    direction: u64,
    alpha_a: u64,
    alpha_b: u64,
) -> u64 {
    ((intensity & 0xFF) << 56)
        | ((param_a & 0xFF) << 48)
        | ((param_b & 0xFF) << 40)
        | ((param_c & 0xFF) << 32)
        | ((param_d & 0xFF) << 24)
        | ((direction & 0xFFFF) << 8)
        | ((alpha_a & 0xF) << 4)
        | (alpha_b & 0xF)
}

// Opcodes
const OP_RAMP: u64 = 0x01;
const OP_GRID: u64 = 0x09;
const OP_SCATTER: u64 = 0x0A;
const OP_FLOW: u64 = 0x0B;

// Region masks
const REGION_ALL: u64 = 0b111;
const REGION_SKY: u64 = 0b100;
const REGION_WALLS: u64 = 0b010;

// Blend modes
const BLEND_ADD: u64 = 0;

// Direction for +Y (up)
const DIR_UP: u64 = 0x80FF;

// =============================================================================
// EPU Presets (2 layers each: 1 bounds + 1 feature)
// =============================================================================

/// Neon City: Purple sky to dark floor with cyan grid on walls
static EPU_NEON_CITY: [[u64; 2]; 8] = [
    // Layer 0: RAMP - Purple sky (#1A0A2E) to dark floor (#000000), gray walls (#404060)
    [
        epu_hi(OP_RAMP, REGION_ALL, BLEND_ADD, 0, 0x1A0A2E, 0x000000),
        epu_lo(220, 0x40, 0x40, 0x60, 0xA5, DIR_UP, 15, 15),
    ],
    // Layer 1: GRID - Cyan grid lines on walls (#00FFFF)
    [
        epu_hi(OP_GRID, REGION_WALLS, BLEND_ADD, 0, 0x00FFFF, 0x00FFFF),
        epu_lo(180, 32, 32, 4, 0, 0, 12, 12),
    ],
    [0, 0], // NOP
    [0, 0], // NOP
    [0, 0], // NOP
    [0, 0], // NOP
    [0, 0], // NOP
    [0, 0], // NOP
];

/// Ember Glow: Orange sky to dark red floor with yellow-orange embers
static EPU_EMBER_GLOW: [[u64; 2]; 8] = [
    // Layer 0: RAMP - Orange sky (#FF6600) to dark red floor (#1A0000), dark orange walls (#401800)
    [
        epu_hi(OP_RAMP, REGION_ALL, BLEND_ADD, 0, 0xFF6600, 0x1A0000),
        epu_lo(220, 0x40, 0x18, 0x00, 0xA5, DIR_UP, 15, 15),
    ],
    // Layer 1: SCATTER - Yellow-orange embers (#FFCC00)
    [
        epu_hi(OP_SCATTER, REGION_SKY, BLEND_ADD, 0, 0xFFCC00, 0xFF6600),
        epu_lo(200, 64, 32, 8, 4, 0, 15, 10),
    ],
    [0, 0], // NOP
    [0, 0], // NOP
    [0, 0], // NOP
    [0, 0], // NOP
    [0, 0], // NOP
    [0, 0], // NOP
];

/// Frozen: Pale blue sky to white floor with white snow particles
static EPU_FROZEN: [[u64; 2]; 8] = [
    // Layer 0: RAMP - Pale blue sky (#CCE6FF) to white floor (#F0F8FF), bright icy walls (#C8D8E8)
    [
        epu_hi(OP_RAMP, REGION_ALL, BLEND_ADD, 0, 0xCCE6FF, 0xF0F8FF),
        epu_lo(230, 0xC8, 0xD8, 0xE8, 0xA5, DIR_UP, 15, 15),
    ],
    // Layer 1: FLOW - White snow particles (#FFFFFF)
    [
        epu_hi(OP_FLOW, REGION_SKY, BLEND_ADD, 0, 0xFFFFFF, 0xE0E8F0),
        epu_lo(160, 48, 8, 16, 24, 0x40C0, 12, 8),
    ],
    [0, 0], // NOP
    [0, 0], // NOP
    [0, 0], // NOP
    [0, 0], // NOP
    [0, 0], // NOP
    [0, 0], // NOP
];

/// Void: Deep blue sky to black floor with white stars
static EPU_VOID: [[u64; 2]; 8] = [
    // Layer 0: RAMP - Deep blue sky (#000D1A) to black floor (#000008), dark gray walls (#080810)
    [
        epu_hi(OP_RAMP, REGION_ALL, BLEND_ADD, 0, 0x000D1A, 0x000008),
        epu_lo(220, 0x08, 0x08, 0x10, 0xA5, DIR_UP, 15, 15),
    ],
    // Layer 1: SCATTER - White stars (#FFFFFF)
    [
        epu_hi(OP_SCATTER, REGION_SKY, BLEND_ADD, 0, 0xFFFFFF, 0x8080FF),
        epu_lo(255, 128, 16, 2, 1, 0, 15, 8),
    ],
    [0, 0], // NOP
    [0, 0], // NOP
    [0, 0], // NOP
    [0, 0], // NOP
    [0, 0], // NOP
    [0, 0], // NOP
];

// =============================================================================
// Game State
// =============================================================================

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

static mut MESHES: Option<ShapeMeshes> = None;

// Shape indices (0=Sphere, 1=Cube, 2=Torus)
static mut LEFT_SHAPE: i32 = 0;
static mut RIGHT_SHAPE: i32 = 0;

// Environment indices (0=Neon City, 1=Ember Glow, 2=Frozen, 3=Void)
static mut LEFT_ENV: i32 = 0;
static mut RIGHT_ENV: i32 = 1;

// Rotation
static mut ROTATION_Y: f32 = 0.0;
static mut AUTO_ROTATE: bool = true;

const ENV_COUNT: i32 = 4;
const ENV_NAMES: [&str; 4] = ["Neon City", "Ember Glow", "Frozen", "Void"];
const SHAPE_NAMES: [&str; 3] = ["Sphere", "Cube", "Torus"];

// =============================================================================
// Helper Functions
// =============================================================================

fn get_epu_preset(index: i32) -> *const u64 {
    match index {
        0 => EPU_NEON_CITY.as_ptr() as *const u64,
        1 => EPU_EMBER_GLOW.as_ptr() as *const u64,
        2 => EPU_FROZEN.as_ptr() as *const u64,
        3 => EPU_VOID.as_ptr() as *const u64,
        _ => EPU_NEON_CITY.as_ptr() as *const u64,
    }
}

// =============================================================================
// Game Implementation
// =============================================================================

#[no_mangle]
pub extern "C" fn init() {
    unsafe {
        set_clear_color(0x000000FF);

        // Generate shape meshes
        MESHES = Some(ShapeMeshes::generate());

        // Shiny metal for visible environment reflections
        material_metallic(1.0);
        material_roughness(0.05);
        material_emissive(0.0);

        // Use uniform material values (no textures)
        use_uniform_color(1);
        use_uniform_metallic(1);
        use_uniform_roughness(1);
        use_uniform_emissive(1);

        // Key light
        light_set(0, -0.4, -1.0, -0.2);
        light_color(0, 0xFFFFFFFF);
        light_intensity(0, 1.5);
    }
}

#[no_mangle]
pub extern "C" fn update() {
    unsafe {
        // Update camera (right stick + triggers)
        CAMERA.update();

        // Shape cycling
        if button_pressed(0, A) != 0 {
            // A: cycle both shapes
            LEFT_SHAPE = (LEFT_SHAPE + 1) % ShapeType::COUNT;
            RIGHT_SHAPE = (RIGHT_SHAPE + 1) % ShapeType::COUNT;
        }
        if button_pressed(0, X) != 0 {
            // X: cycle left shape only
            LEFT_SHAPE = (LEFT_SHAPE + 1) % ShapeType::COUNT;
        }
        if button_pressed(0, Y) != 0 {
            // Y: cycle right shape only
            RIGHT_SHAPE = (RIGHT_SHAPE + 1) % ShapeType::COUNT;
        }

        // Environment cycling
        if button_pressed(0, LEFT) != 0 {
            LEFT_ENV = (LEFT_ENV + ENV_COUNT - 1) % ENV_COUNT;
        }
        if button_pressed(0, RIGHT) != 0 {
            LEFT_ENV = (LEFT_ENV + 1) % ENV_COUNT;
        }
        if button_pressed(0, UP) != 0 {
            RIGHT_ENV = (RIGHT_ENV + ENV_COUNT - 1) % ENV_COUNT;
        }
        if button_pressed(0, DOWN) != 0 {
            RIGHT_ENV = (RIGHT_ENV + 1) % ENV_COUNT;
        }

        // Auto-rotation toggle
        if button_pressed(0, B) != 0 {
            AUTO_ROTATE = !AUTO_ROTATE;
        }

        // Update rotation
        if AUTO_ROTATE {
            ROTATION_Y += 30.0 * delta_time();
            if ROTATION_Y >= 360.0 {
                ROTATION_Y -= 360.0;
            }
        }
    }
}

#[no_mangle]
pub extern "C" fn render() {
    unsafe {
        // Apply camera
        CAMERA.apply();

        let meshes = MESHES.as_ref().unwrap();

        // Push two environments this frame
        environment_index(0);
        epu_set(get_epu_preset(LEFT_ENV));
        environment_index(1);
        epu_set(get_epu_preset(RIGHT_ENV));

        // Left object: env 0
        environment_index(0);
        set_color(0xFFFFFFFF);
        push_identity();
        push_translate(-1.6, 0.0, 0.0);
        push_rotate_y(ROTATION_Y);
        draw_mesh(meshes.get_by_index(LEFT_SHAPE));

        // Right object: env 1
        environment_index(1);
        set_color(0xFFFFFFFF);
        push_identity();
        push_translate(1.6, 0.0, 0.0);
        push_rotate_y(-ROTATION_Y); // Opposite rotation for visual interest
        draw_mesh(meshes.get_by_index(RIGHT_SHAPE));

        // Draw environment background (use left env for background)
        environment_index(0);
        draw_epu();

        // Draw UI
        draw_ui();
    }
}

unsafe fn draw_ui() {
    // Title
    set_color(0xFFFFFFFF);
    let title = b"EPU Multi-Reflections";
    draw_text(title.as_ptr(), title.len() as u32, 10.0, 10.0, 20.0);

    // Current state
    set_color(0xCCCCCCFF);

    // Left object info
    let left_env_name = ENV_NAMES[LEFT_ENV as usize];
    let left_shape_name = SHAPE_NAMES[LEFT_SHAPE as usize];
    let mut left_label = [0u8; 64];
    let left_prefix = b"Left: ";
    left_label[..left_prefix.len()].copy_from_slice(left_prefix);
    let mut pos = left_prefix.len();
    let env_bytes = left_env_name.as_bytes();
    left_label[pos..pos + env_bytes.len()].copy_from_slice(env_bytes);
    pos += env_bytes.len();
    let bracket_open = b" [";
    left_label[pos..pos + bracket_open.len()].copy_from_slice(bracket_open);
    pos += bracket_open.len();
    let shape_bytes = left_shape_name.as_bytes();
    left_label[pos..pos + shape_bytes.len()].copy_from_slice(shape_bytes);
    pos += shape_bytes.len();
    let bracket_close = b"]";
    left_label[pos..pos + bracket_close.len()].copy_from_slice(bracket_close);
    pos += bracket_close.len();
    draw_text(left_label.as_ptr(), pos as u32, 10.0, 40.0, 16.0);

    // Right object info
    let right_env_name = ENV_NAMES[RIGHT_ENV as usize];
    let right_shape_name = SHAPE_NAMES[RIGHT_SHAPE as usize];
    let mut right_label = [0u8; 64];
    let right_prefix = b"Right: ";
    right_label[..right_prefix.len()].copy_from_slice(right_prefix);
    pos = right_prefix.len();
    let env_bytes = right_env_name.as_bytes();
    right_label[pos..pos + env_bytes.len()].copy_from_slice(env_bytes);
    pos += env_bytes.len();
    let bracket_open = b" [";
    right_label[pos..pos + bracket_open.len()].copy_from_slice(bracket_open);
    pos += bracket_open.len();
    let shape_bytes = right_shape_name.as_bytes();
    right_label[pos..pos + shape_bytes.len()].copy_from_slice(shape_bytes);
    pos += shape_bytes.len();
    let bracket_close = b"]";
    right_label[pos..pos + bracket_close.len()].copy_from_slice(bracket_close);
    pos += bracket_close.len();
    draw_text(right_label.as_ptr(), pos as u32, 10.0, 60.0, 16.0);

    // Instructions
    set_color(0x888888FF);
    let hint1 = b"D-Pad L/R: Left env | D-Pad U/D: Right env";
    draw_text(hint1.as_ptr(), hint1.len() as u32, 10.0, 90.0, 14.0);

    let hint2 = b"A: Shapes | X: Left shape | Y: Right shape";
    draw_text(hint2.as_ptr(), hint2.len() as u32, 10.0, 110.0, 14.0);

    let hint3 = b"Right Stick: Orbit | Triggers: Zoom | B: Rotation";
    draw_text(hint3.as_ptr(), hint3.len() as u32, 10.0, 130.0, 14.0);

    // Rotation state
    if AUTO_ROTATE {
        set_color(0x88FF88FF);
        let rot_on = b"Rotation: ON";
        draw_text(rot_on.as_ptr(), rot_on.len() as u32, 10.0, 160.0, 14.0);
    } else {
        set_color(0xFF8888FF);
        let rot_off = b"Rotation: OFF";
        draw_text(rot_off.as_ptr(), rot_off.len() as u32, 10.0, 160.0, 14.0);
    }
}
