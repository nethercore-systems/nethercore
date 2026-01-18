//! Environment Cells Inspector - Mode 1 Demo
//!
//! Demonstrates the unified Cells mode:
//! - Family 0: Particles (stars/snow/rain/embers/bubbles/warp)
//! - Family 1: Tiles/Lights (Truchet/Mondrian, buildings, bands, panels)

#![no_std]
#![no_main]

use core::panic::PanicInfo;
use examples_common::*;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    core::arch::wasm32::unreachable()
}

// Debug values (Mode 1: Cells) — matches the design sheet.
static mut FAMILY: u8 = 0; // 0=Particles, 1=Tiles
static mut VARIANT: u8 = 0; // 0-3 (family-specific)
static mut DENSITY: u8 = 120;
static mut SIZE_MIN: u8 = 2;
static mut SIZE_MAX: u8 = 10;
static mut INTENSITY: u8 = 200;
static mut SHAPE: u8 = 220;
static mut MOTION: u8 = 64;
static mut PARALLAX: u8 = 140;
static mut HEIGHT_BIAS: u8 = 100;
static mut CLUSTERING: u8 = 40;
static mut COLOR_A: u32 = 0xDDE6FFFF;
static mut COLOR_B: u32 = 0xFFF2C0FF;
static mut AXIS_X: f32 = 0.0;
static mut AXIS_Y: f32 = 1.0;
static mut AXIS_Z: f32 = 0.0;
static mut SEED: u32 = 0; // 0 = derive

static mut PHASE: u32 = 0;
static mut PHASE_RATE: u32 = 2700; // phase units/sec
static mut ANIMATE: u8 = 1;
static mut PRESET_INDEX: i32 = 0;

static mut SPHERE_MESH: u32 = 0;
static mut CUBE_MESH: u32 = 0;
static mut TORUS_MESH: u32 = 0;
static mut SHAPE_INDEX: i32 = 0;
static mut CAM_ANGLE: f32 = 0.0;
static mut CAM_ELEVATION: f32 = 0.0;

const SHAPE_COUNT: usize = 3;
const PRESET_COUNT: usize = 4;
const PRESET_NAMES: [&str; PRESET_COUNT] = [
    "Starfield Calm",
    "Snowfall",
    "Cyber City Windows",
    "Truchet Gallery Neon",
];

fn load_preset(index: usize) {
    unsafe {
        match index {
            0 => {
                // Starfield Calm — Particles/Stars
                FAMILY = 0;
                VARIANT = 0;
                DENSITY = 120;
                SIZE_MIN = 2;
                SIZE_MAX = 10;
                INTENSITY = 200;
                SHAPE = 220;
                MOTION = 64;
                PARALLAX = 140;
                HEIGHT_BIAS = 100;
                CLUSTERING = 40;
                COLOR_A = 0xDDE6FFFF;
                COLOR_B = 0xFFF2C0FF;
                AXIS_X = 0.0;
                AXIS_Y = 1.0;
                AXIS_Z = 0.0;
                SEED = 0;
                PHASE_RATE = 2700;
            }
            1 => {
                // Snowfall — Particles/Fall
                FAMILY = 0;
                VARIANT = 1;
                DENSITY = 170;
                SIZE_MIN = 8;
                SIZE_MAX = 22;
                INTENSITY = 150;
                SHAPE = 140;
                MOTION = 160;
                PARALLAX = 170;
                HEIGHT_BIAS = 170;
                CLUSTERING = 90;
                COLOR_A = 0xF2F7FFFF;
                COLOR_B = 0xCFE6FFFF;
                AXIS_X = 0.0;
                AXIS_Y = -1.0;
                AXIS_Z = 0.0;
                SEED = 0;
                PHASE_RATE = 6500;
            }
            2 => {
                // Cyber City Windows — Tiles/Buildings
                FAMILY = 1;
                VARIANT = 1;
                DENSITY = 210;
                SIZE_MIN = 28;
                SIZE_MAX = 96;
                INTENSITY = 230;
                SHAPE = 210;
                MOTION = 110;
                PARALLAX = 190;
                HEIGHT_BIAS = 230;
                CLUSTERING = 230;
                COLOR_A = 0x52D6FFFF;
                COLOR_B = 0xFFD36BFF;
                AXIS_X = 0.0;
                AXIS_Y = 1.0;
                AXIS_Z = 0.0;
                SEED = 0;
                PHASE_RATE = 11000;
            }
            _ => {
                // Truchet Gallery Neon — Tiles/Abstract
                FAMILY = 1;
                VARIANT = 0;
                DENSITY = 200;
                SIZE_MIN = 22;
                SIZE_MAX = 72;
                INTENSITY = 210;
                SHAPE = 170;
                MOTION = 90;
                PARALLAX = 80;
                HEIGHT_BIAS = 128;
                CLUSTERING = 180;
                COLOR_A = 0x8B5CFFFF;
                COLOR_B = 0x00FFD1FF;
                AXIS_X = 0.0;
                AXIS_Y = 1.0;
                AXIS_Z = 0.0;
                SEED = 0;
                PHASE_RATE = 5500;
            }
        }
    }
}

#[no_mangle]
pub extern "C" fn on_debug_change() {
    unsafe {
        PRESET_INDEX = PRESET_INDEX.clamp(0, PRESET_COUNT as i32 - 1);
        FAMILY = FAMILY.clamp(0, 1);
        VARIANT = VARIANT.clamp(0, 3);
        if SIZE_MIN > SIZE_MAX {
            core::mem::swap(&mut SIZE_MIN, &mut SIZE_MAX);
        }

        let len = libm::sqrtf(AXIS_X * AXIS_X + AXIS_Y * AXIS_Y + AXIS_Z * AXIS_Z);
        if len > 1e-6 {
            AXIS_X /= len;
            AXIS_Y /= len;
            AXIS_Z /= len;
        } else if FAMILY == 0 && VARIANT == 1 {
            // Fall defaults to "down" (rain/snow).
            AXIS_X = 0.0;
            AXIS_Y = -1.0;
            AXIS_Z = 0.0;
        } else {
            // Most variants default to Y-up.
            AXIS_X = 0.0;
            AXIS_Y = 1.0;
            AXIS_Z = 0.0;
        }
    }
}

#[no_mangle]
pub extern "C" fn init() {
    unsafe {
        set_clear_color(0x0A0A14FF);
        SPHERE_MESH = sphere(1.0, 32, 24);
        CUBE_MESH = cube(1.4, 1.4, 1.4);
        TORUS_MESH = torus(1.0, 0.4, 32, 16);
        load_preset(0);

        debug_group_begin(b"cells".as_ptr(), 5);
        debug_register_u8(b"family".as_ptr(), 6, &FAMILY);
        debug_register_u8(b"variant".as_ptr(), 7, &VARIANT);
        debug_register_u8(b"density".as_ptr(), 7, &DENSITY);
        debug_register_u8(b"size_min".as_ptr(), 8, &SIZE_MIN);
        debug_register_u8(b"size_max".as_ptr(), 8, &SIZE_MAX);
        debug_register_u8(b"intensity".as_ptr(), 9, &INTENSITY);
        debug_register_u8(b"shape".as_ptr(), 5, &SHAPE);
        debug_register_u8(b"motion".as_ptr(), 6, &MOTION);
        debug_register_u8(b"parallax".as_ptr(), 8, &PARALLAX);
        debug_register_u8(b"height".as_ptr(), 6, &HEIGHT_BIAS);
        debug_register_u8(b"cluster".as_ptr(), 7, &CLUSTERING);
        debug_register_color(b"color_a".as_ptr(), 7, &COLOR_A as *const u32 as *const u8);
        debug_register_color(b"color_b".as_ptr(), 7, &COLOR_B as *const u32 as *const u8);
        debug_register_u32(b"seed".as_ptr(), 4, &SEED as *const u32 as *const u8);
        debug_group_end();

        debug_group_begin(b"axis".as_ptr(), 4);
        debug_register_f32(b"x".as_ptr(), 1, &AXIS_X as *const f32 as *const u8);
        debug_register_f32(b"y".as_ptr(), 1, &AXIS_Y as *const f32 as *const u8);
        debug_register_f32(b"z".as_ptr(), 1, &AXIS_Z as *const f32 as *const u8);
        debug_group_end();

        debug_group_begin(b"animation".as_ptr(), 9);
        debug_register_u8(b"animate".as_ptr(), 7, &ANIMATE);
        debug_register_u32(b"phase".as_ptr(), 5, &PHASE as *const u32 as *const u8);
        debug_register_u32(b"rate".as_ptr(), 4, &PHASE_RATE as *const u32 as *const u8);
        debug_group_end();

        debug_group_begin(b"preset".as_ptr(), 6);
        debug_register_i32(b"index".as_ptr(), 5, &PRESET_INDEX as *const i32 as *const u8);
        debug_group_end();
    }
}

#[no_mangle]
pub extern "C" fn update() {
    unsafe {
        if button_pressed(0, button::A) != 0 {
            PRESET_INDEX = (PRESET_INDEX + 1) % PRESET_COUNT as i32;
            load_preset(PRESET_INDEX as usize);
        }
        if button_pressed(0, button::B) != 0 {
            SHAPE_INDEX = (SHAPE_INDEX + 1) % SHAPE_COUNT as i32;
        }

        if ANIMATE != 0 {
            let dt = delta_time();
            PHASE = PHASE.wrapping_add((PHASE_RATE as f32 * dt) as u32);
        }

        let stick_x = left_stick_x(0);
        let stick_y = left_stick_y(0);
        if stick_x.abs() > 0.1 {
            CAM_ANGLE += stick_x * 2.0;
        }
        if stick_y.abs() > 0.1 {
            CAM_ELEVATION = (CAM_ELEVATION - stick_y * 2.0).clamp(-60.0, 60.0);
        }
    }
}

#[no_mangle]
pub extern "C" fn render() {
    unsafe {
        let angle_rad = CAM_ANGLE * 0.0174533;
        let elev_rad = CAM_ELEVATION * 0.0174533;
        let dist = 5.0;

        camera_set(
            dist * libm::cosf(elev_rad) * libm::sinf(angle_rad),
            dist * libm::sinf(elev_rad),
            dist * libm::cosf(elev_rad) * libm::cosf(angle_rad),
            0.0,
            0.0,
            0.0,
        );
        camera_fov(60.0);

        env_cells(
            0, // base layer
            FAMILY as u32,
            VARIANT as u32,
            DENSITY as u32,
            SIZE_MIN as u32,
            SIZE_MAX as u32,
            INTENSITY as u32,
            SHAPE as u32,
            MOTION as u32,
            PARALLAX as u32,
            HEIGHT_BIAS as u32,
            CLUSTERING as u32,
            COLOR_A,
            COLOR_B,
            AXIS_X,
            AXIS_Y,
            AXIS_Z,
            PHASE,
            SEED,
        );
        draw_env();

        push_identity();
        set_color(0x333344FF);
        material_metallic(0.7);
        material_roughness(0.3);
        let mesh = match SHAPE_INDEX {
            0 => SPHERE_MESH,
            1 => CUBE_MESH,
            _ => TORUS_MESH,
        };
        draw_mesh(mesh);

        let title = b"Env Mode 1: Cells";
        set_color(0xFFFFFFFF);
        draw_text(title.as_ptr(), title.len() as u32, 10.0, 10.0, 20.0);

        let preset_name = PRESET_NAMES[PRESET_INDEX as usize];
        let mut label = [0u8; 40];
        let prefix = b"Preset: ";
        label[..prefix.len()].copy_from_slice(prefix);
        let name = preset_name.as_bytes();
        label[prefix.len()..prefix.len() + name.len()].copy_from_slice(name);
        set_color(0xCCCCCCFF);
        draw_text(label.as_ptr(), (prefix.len() + name.len()) as u32, 10.0, 40.0, 16.0);

        let hint = b"A: preset | B: shape | Stick: camera | F4: debug";
        set_color(0x888888FF);
        draw_text(hint.as_ptr(), hint.len() as u32, 10.0, 70.0, 14.0);
    }
}
