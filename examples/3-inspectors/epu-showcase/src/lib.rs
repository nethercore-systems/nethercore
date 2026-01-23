//! EPU Showcase - Environment Processing Unit Demo
//!
//! Demonstrates the instruction-based EPU system for procedural environment backgrounds.
//! The EPU uses 128-byte configurations (8 x 128-bit layers) to define complex environments
//! with minimal memory and deterministic rendering.
//!
//! # Format (128-bit instructions)
//!
//! Each layer is 128 bits (2 x u64) with the following layout:
//! - hi word: opcode(5), region(3), blend(3), meta5(5), color_a(24), color_b(24)
//!   - meta5 = (domain_id << 3) | variant_id for domain/variant selection
//! - lo word: intensity(8), param_a(8), param_b(8), param_c(8), param_d(8), direction(16), alpha_a(4), alpha_b(4)
//!
//! Features:
//! - Multiple preset environments
//! - Additional opcodes: CELL, PATCHES, APERTURE, TRACE, VEIL, ATMOSPHERE, PLANE, CELESTIAL, PORTAL
//! - Keyboard/gamepad cycling through presets
//! - Real-time environment background rendering via epu_draw()
//! - Layer breakdown showing opcode names for current preset
//!
//! Controls:
//! - A button: Cycle to next preset
//! - B button: Cycle to previous preset
//! - Left stick: Rotate camera around scene
//!
//! Press F4 to open the debug inspector.

#![no_std]
#![no_main]

use core::panic::PanicInfo;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    core::arch::wasm32::unreachable()
}

// ============================================================================
// Modules
// ============================================================================

mod constants;
mod presets;

use presets::{PRESETS, PRESET_COUNT, PRESET_NAMES};

// ============================================================================
// FFI Declarations
// ============================================================================

#[path = "../../../../include/zx.rs"]
mod ffi;
use ffi::*;

// ============================================================================
// Game State
// ============================================================================

static mut PRESET_INDEX: i32 = 0;
static mut CAM_ANGLE: f32 = 0.0;
static mut CAM_ELEVATION: f32 = 15.0;
static mut SPHERE_MESH: u32 = 0;
static mut TORUS_MESH: u32 = 0;
static mut CUBE_MESH: u32 = 0;
static mut SHAPE_INDEX: i32 = 0;
static mut MATERIAL_METALLIC_U8: i32 = 255; // ~1 * 255
static mut MATERIAL_ROUGHNESS_U8: i32 = 128; // ~0.50 * 255

const SHAPE_COUNT: i32 = 3;
const SHAPE_NAMES: [&str; 3] = ["Sphere", "Cube", "Torus"];

// ============================================================================
// Game Implementation
// ============================================================================

#[no_mangle]
pub extern "C" fn init() {
    unsafe {
        set_clear_color(0x000000FF);

        // Generate meshes for the scene
        SPHERE_MESH = sphere(1.0, 32, 24);
        CUBE_MESH = cube(1.2, 1.2, 1.2);
        TORUS_MESH = torus(1.0, 0.4, 32, 16);

        // Register debug values
        debug_group_begin(b"preset".as_ptr(), 6);
        debug_register_i32(b"index".as_ptr(), 5, &raw const PRESET_INDEX as *const u8);
        debug_group_end();

        debug_group_begin(b"camera".as_ptr(), 6);
        debug_register_f32(b"angle".as_ptr(), 5, &raw const CAM_ANGLE as *const u8);
        debug_register_f32(
            b"elevation".as_ptr(),
            9,
            &raw const CAM_ELEVATION as *const u8,
        );
        debug_group_end();

        debug_group_begin(b"shape".as_ptr(), 5);
        debug_register_i32(b"index".as_ptr(), 5, &raw const SHAPE_INDEX as *const u8);
        debug_group_end();

        debug_group_begin(b"material".as_ptr(), 8);
        debug_register_i32(
            b"metallic_u8".as_ptr(),
            11,
            &raw const MATERIAL_METALLIC_U8 as *const u8,
        );
        debug_register_i32(
            b"roughness_u8".as_ptr(),
            12,
            &raw const MATERIAL_ROUGHNESS_U8 as *const u8,
        );
        debug_group_end();
    }
}

#[no_mangle]
pub extern "C" fn on_debug_change() {
    unsafe {
        // Clamp preset index
        if PRESET_INDEX < 0 {
            PRESET_INDEX = 0;
        }
        if PRESET_INDEX >= PRESET_COUNT as i32 {
            PRESET_INDEX = PRESET_COUNT as i32 - 1;
        }

        // Clamp shape index
        if SHAPE_INDEX < 0 {
            SHAPE_INDEX = 0;
        }
        if SHAPE_INDEX >= SHAPE_COUNT {
            SHAPE_INDEX = SHAPE_COUNT - 1;
        }

        // Clamp camera elevation
        CAM_ELEVATION = CAM_ELEVATION.clamp(-60.0, 60.0);

        // Clamp material parameters (0..255)
        MATERIAL_METALLIC_U8 = MATERIAL_METALLIC_U8.clamp(0, 255);
        MATERIAL_ROUGHNESS_U8 = MATERIAL_ROUGHNESS_U8.clamp(0, 255);
    }
}

#[no_mangle]
pub extern "C" fn update() {
    unsafe {
        // Cycle presets with A/B buttons
        if button_pressed(0, button::A) != 0 {
            PRESET_INDEX = (PRESET_INDEX + 1) % PRESET_COUNT as i32;
        }
        if button_pressed(0, button::B) != 0 {
            PRESET_INDEX = (PRESET_INDEX + PRESET_COUNT as i32 - 1) % PRESET_COUNT as i32;
        }

        // Cycle shapes with X button
        if button_pressed(0, button::X) != 0 {
            SHAPE_INDEX = (SHAPE_INDEX + 1) % SHAPE_COUNT;
        }

        // Camera control via left stick
        let stick_x = left_stick_x(0);
        let stick_y = left_stick_y(0);

        if stick_x.abs() > 0.1 {
            CAM_ANGLE += stick_x * 2.0;
        }
        if stick_y.abs() > 0.1 {
            CAM_ELEVATION -= stick_y * 2.0;
            CAM_ELEVATION = CAM_ELEVATION.clamp(-60.0, 60.0);
        }
    }
}

#[no_mangle]
pub extern "C" fn render() {
    unsafe {
        // Calculate camera position
        let angle_rad = CAM_ANGLE * 0.0174533;
        let elev_rad = CAM_ELEVATION * 0.0174533;
        let dist = 5.0;

        let cam_x = dist * libm::cosf(elev_rad) * libm::sinf(angle_rad);
        let cam_y = dist * libm::sinf(elev_rad) + 1.0;
        let cam_z = dist * libm::cosf(elev_rad) * libm::cosf(angle_rad);

        camera_set(cam_x, cam_y, cam_z, 0.0, 0.0, 0.0);
        camera_fov(60.0);

        // Draw the EPU environment background (push-only)
        epu_draw(PRESETS[PRESET_INDEX as usize].as_ptr() as *const u64);

        // Draw a shape to show lighting from the environment
        push_identity();
        set_color(0x888899FF);
        material_metallic((MATERIAL_METALLIC_U8 as f32) / 255.0);
        material_roughness((MATERIAL_ROUGHNESS_U8 as f32) / 255.0);

        let mesh = match SHAPE_INDEX {
            0 => SPHERE_MESH,
            1 => CUBE_MESH,
            _ => TORUS_MESH,
        };
        draw_mesh(mesh);

        // Draw UI overlay
        draw_ui();
    }
}

/// Get opcode name from opcode number
fn opcode_name(opcode: u8) -> &'static [u8] {
    match opcode {
        0x00 => b"NOP",
        0x01 => b"RAMP",
        0x02 => b"LOBE/SECTOR",
        0x03 => b"BAND/SILHOUETTE",
        0x04 => b"FOG/SPLIT",
        0x05 => b"CELL",
        0x06 => b"PATCHES",
        0x07 => b"APERTURE",
        0x08 => b"DECAL",
        0x09 => b"GRID",
        0x0A => b"SCATTER",
        0x0B => b"FLOW",
        0x0C => b"TRACE",
        0x0D => b"VEIL",
        0x0E => b"ATMOSPHERE",
        0x0F => b"PLANE",
        0x10 => b"CELESTIAL",
        0x11 => b"PORTAL",
        0x12 => b"LOBE_RADIANCE",
        0x13 => b"BAND_RADIANCE",
        _ => b"???",
    }
}

/// Extract opcode from hi word (bits 63..59, which is bits 127..123 of the full instruction)
fn extract_opcode(hi: u64) -> u8 {
    ((hi >> 59) & 0x1F) as u8
}

unsafe fn draw_ui() {
    // Title
    let title = b"EPU Showcase";
    set_color(0xFFFFFFFF);
    draw_text(title.as_ptr(), title.len() as u32, 10.0, 10.0, 24.0);

    // Current preset name
    let preset_name = PRESET_NAMES[PRESET_INDEX as usize];
    let mut label = [0u8; 48];
    let prefix = b"Preset: ";
    label[..prefix.len()].copy_from_slice(prefix);
    let name = preset_name.as_bytes();
    let name_len = if name.len() > 40 { 40 } else { name.len() };
    label[prefix.len()..prefix.len() + name_len].copy_from_slice(&name[..name_len]);
    set_color(0xCCCCCCFF);
    draw_text(
        label.as_ptr(),
        (prefix.len() + name_len) as u32,
        10.0,
        42.0,
        18.0,
    );

    // Current shape name
    let shape_name = SHAPE_NAMES[SHAPE_INDEX as usize];
    let mut shape_label = [0u8; 32];
    let shape_prefix = b"Shape: ";
    shape_label[..shape_prefix.len()].copy_from_slice(shape_prefix);
    let sname = shape_name.as_bytes();
    shape_label[shape_prefix.len()..shape_prefix.len() + sname.len()].copy_from_slice(sname);
    set_color(0xAAAAAAFF);
    draw_text(
        shape_label.as_ptr(),
        (shape_prefix.len() + sname.len()) as u32,
        10.0,
        66.0,
        16.0,
    );

    // Instructions
    let hint1 = b"A/B: Cycle presets | X: Cycle shapes";
    set_color(0x888888FF);
    draw_text(hint1.as_ptr(), hint1.len() as u32, 10.0, 94.0, 14.0);

    let hint2 = b"Left stick: Orbit camera | F4: Debug panel";
    draw_text(hint2.as_ptr(), hint2.len() as u32, 10.0, 112.0, 14.0);

    // Preset index indicator (supports up to 99 presets)
    let mut idx_label = [0u8; 16];
    let current = PRESET_INDEX as u8 + 1;
    let total = PRESET_COUNT as u8;
    let mut pos = 0usize;

    idx_label[pos] = b'[';
    pos += 1;

    // Write current index (1-based)
    if current >= 10 {
        idx_label[pos] = b'0' + (current / 10);
        pos += 1;
    }
    idx_label[pos] = b'0' + (current % 10);
    pos += 1;

    idx_label[pos] = b'/';
    pos += 1;

    // Write total count
    if total >= 10 {
        idx_label[pos] = b'0' + (total / 10);
        pos += 1;
    }
    idx_label[pos] = b'0' + (total % 10);
    pos += 1;

    idx_label[pos] = b']';
    pos += 1;

    set_color(0x666666FF);
    draw_text(idx_label.as_ptr(), pos as u32, 10.0, 130.0, 12.0);

    // Layer breakdown - show active opcodes for current preset
    let layers_title = b"Layers:";
    set_color(0x888888FF);
    draw_text(
        layers_title.as_ptr(),
        layers_title.len() as u32,
        10.0,
        152.0,
        12.0,
    );

    let preset = &PRESETS[PRESET_INDEX as usize];
    let mut y_offset = 166.0f32;

    #[allow(clippy::needless_range_loop)]
    for layer_idx in 0..8 {
        let hi = preset[layer_idx][0];
        let opcode = extract_opcode(hi);

        // Skip NOP layers for cleaner display
        if opcode == 0 {
            continue;
        }

        let op_name = opcode_name(opcode);

        // Build layer line: "L0: RAMP" format
        let mut layer_line = [0u8; 24];
        layer_line[0] = b'L';
        layer_line[1] = b'0' + (layer_idx as u8);
        layer_line[2] = b':';
        layer_line[3] = b' ';
        let copy_len = if op_name.len() > 16 {
            16
        } else {
            op_name.len()
        };
        layer_line[4..4 + copy_len].copy_from_slice(&op_name[..copy_len]);

        set_color(0x669966FF);
        draw_text(
            layer_line.as_ptr(),
            (4 + copy_len) as u32,
            10.0,
            y_offset,
            11.0,
        );
        y_offset += 12.0;
    }
}
