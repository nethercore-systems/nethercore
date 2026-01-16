//! Environment Nebula Inspector - Mode 4 Demo
//!
//! Demonstrates Nebula: fog/clouds/aurora/ink/plasma/kaleido (soft fields).

#![no_std]
#![no_main]

use core::panic::PanicInfo;
use examples_common::*;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    core::arch::wasm32::unreachable()
}

// Debug values (Mode 4: Nebula) — matches the design sheet.
static mut FAMILY: u8 = 0;
static mut COVERAGE: u8 = 170;
static mut SOFTNESS: u8 = 220;
static mut INTENSITY: u8 = 10;
static mut SCALE: u8 = 190;
static mut DETAIL: u8 = 40;
static mut WARP: u8 = 30;
static mut FLOW: u8 = 70;
static mut PARALLAX: u8 = 0;
static mut HEIGHT_BIAS: u8 = 128;
static mut CONTRAST: u8 = 35;
static mut COLOR_A: u32 = 0xA9B9C7FF;
static mut COLOR_B: u32 = 0xF2B59CFF;
static mut AXIS_X: f32 = 0.0;
static mut AXIS_Y: f32 = 1.0;
static mut AXIS_Z: f32 = 0.0;
static mut SEED: u32 = 0;

static mut PHASE: u32 = 0;
static mut PHASE_RATE: u32 = 512;
static mut ANIMATE: u8 = 1;
static mut PRESET_INDEX: i32 = 0;

static mut SPHERE_MESH: u32 = 0;
static mut CUBE_MESH: u32 = 0;
static mut TORUS_MESH: u32 = 0;
static mut SHAPE_INDEX: i32 = 0;
static mut CAM_ANGLE: f32 = 0.0;
static mut CAM_ELEVATION: f32 = 10.0;

const SHAPE_COUNT: usize = 3;
const PRESET_COUNT: usize = 4;
const PRESET_NAMES: [&str; PRESET_COUNT] = [
    "Foggy Dawn",
    "Storm Front",
    "Aurora Night",
    "Kaleido Galaxy",
];

fn load_preset(index: usize) {
    unsafe {
        match index {
            0 => {
                // Foggy Dawn
                FAMILY = 0;
                COVERAGE = 170;
                SOFTNESS = 220;
                INTENSITY = 10;
                SCALE = 190;
                DETAIL = 40;
                WARP = 30;
                FLOW = 70;
                PARALLAX = 0;
                HEIGHT_BIAS = 128;
                CONTRAST = 35;
                COLOR_A = 0xA9B9C7FF;
                COLOR_B = 0xF2B59CFF;
                AXIS_X = 0.0;
                AXIS_Y = 1.0;
                AXIS_Z = 0.0;
                SEED = 0;
                PHASE_RATE = 512;
            }
            1 => {
                // Storm Front
                FAMILY = 1;
                COVERAGE = 215;
                SOFTNESS = 160;
                INTENSITY = 0;
                SCALE = 150;
                DETAIL = 160;
                WARP = 120;
                FLOW = 110;
                PARALLAX = 0;
                HEIGHT_BIAS = 210;
                CONTRAST = 190;
                COLOR_A = 0x2E3440FF;
                COLOR_B = 0xBFC7D5FF;
                AXIS_X = 0.0;
                AXIS_Y = 1.0;
                AXIS_Z = 0.0;
                SEED = 0;
                PHASE_RATE = 1024;
            }
            2 => {
                // Aurora Night
                FAMILY = 2;
                COVERAGE = 150;
                SOFTNESS = 210;
                INTENSITY = 230;
                SCALE = 170;
                DETAIL = 120;
                WARP = 140;
                FLOW = 140;
                PARALLAX = 0;
                HEIGHT_BIAS = 170;
                CONTRAST = 200;
                COLOR_A = 0x00E8A8FF;
                COLOR_B = 0xB04BFFFF;
                AXIS_X = 0.15;
                AXIS_Y = 0.98;
                AXIS_Z = 0.12;
                SEED = 0;
                PHASE_RATE = 2048;
            }
            _ => {
                // Kaleido Galaxy
                FAMILY = 5;
                COVERAGE = 185;
                SOFTNESS = 140;
                INTENSITY = 120;
                SCALE = 160;
                DETAIL = 220;
                WARP = 200;
                FLOW = 90;
                PARALLAX = 0;
                HEIGHT_BIAS = 128;
                CONTRAST = 160;
                COLOR_A = 0x24103CFF;
                COLOR_B = 0x30D6FFFF;
                AXIS_X = 0.0;
                AXIS_Y = 1.0;
                AXIS_Z = 0.0;
                SEED = 0;
                PHASE_RATE = 3072;
            }
        }
    }
}

#[no_mangle]
pub extern "C" fn on_debug_change() {
    unsafe {
        PRESET_INDEX = PRESET_INDEX.clamp(0, PRESET_COUNT as i32 - 1);
        FAMILY = FAMILY.clamp(0, 5);

        // Normalize axis.
        let len = libm::sqrtf(AXIS_X * AXIS_X + AXIS_Y * AXIS_Y + AXIS_Z * AXIS_Z);
        if len > 0.01 {
            AXIS_X /= len;
            AXIS_Y /= len;
            AXIS_Z /= len;
        } else {
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
        SPHERE_MESH = sphere(1.5, 32, 24);
        CUBE_MESH = cube(2.0, 2.0, 2.0);
        TORUS_MESH = torus(1.3, 0.5, 32, 16);
        load_preset(0);

        debug_group_begin(b"nebula".as_ptr(), 6);
        debug_register_u8(b"family".as_ptr(), 6, &FAMILY);
        debug_register_u8(b"coverage".as_ptr(), 8, &COVERAGE);
        debug_register_u8(b"softness".as_ptr(), 8, &SOFTNESS);
        debug_register_u8(b"intensity".as_ptr(), 9, &INTENSITY);
        debug_register_u8(b"scale".as_ptr(), 5, &SCALE);
        debug_register_u8(b"detail".as_ptr(), 6, &DETAIL);
        debug_register_u8(b"warp".as_ptr(), 4, &WARP);
        debug_register_u8(b"flow".as_ptr(), 4, &FLOW);
        debug_register_u8(b"parallax".as_ptr(), 8, &PARALLAX);
        debug_register_u8(b"height".as_ptr(), 6, &HEIGHT_BIAS);
        debug_register_u8(b"contrast".as_ptr(), 8, &CONTRAST);
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
            CAM_ELEVATION = (CAM_ELEVATION - stick_y * 2.0).clamp(-20.0, 60.0);
        }
    }
}

#[no_mangle]
pub extern "C" fn render() {
    unsafe {
        let angle_rad = CAM_ANGLE * 0.0174533;
        let elev_rad = CAM_ELEVATION * 0.0174533;
        let dist = 6.0;

        camera_set(
            dist * libm::cosf(elev_rad) * libm::sinf(angle_rad),
            dist * libm::sinf(elev_rad),
            dist * libm::cosf(elev_rad) * libm::cosf(angle_rad),
            0.0,
            0.0,
            0.0,
        );
        camera_fov(60.0);

        env_nebula(
            0, // base layer
            FAMILY as u32,
            COVERAGE as u32,
            SOFTNESS as u32,
            INTENSITY as u32,
            SCALE as u32,
            DETAIL as u32,
            WARP as u32,
            FLOW as u32,
            PARALLAX as u32,
            HEIGHT_BIAS as u32,
            CONTRAST as u32,
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
        set_color(0x222233FF);
        material_metallic(0.9);
        material_roughness(0.1);
        let mesh = match SHAPE_INDEX {
            0 => SPHERE_MESH,
            1 => CUBE_MESH,
            _ => TORUS_MESH,
        };
        draw_mesh(mesh);

        let title = b"Env Mode 4: Nebula";
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

        let hint = b"A: preset | B: shape | Stick: camera | F4: Debug Inspector";
        set_color(0x888888FF);
        draw_text(hint.as_ptr(), hint.len() as u32, 10.0, 70.0, 14.0);
    }
}
