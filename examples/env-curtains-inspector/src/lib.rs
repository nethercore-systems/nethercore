//! Environment Curtains Inspector - Mode 6 Demo
//!
//! Demonstrates vertical structures (pillars, trees, neon bars).

#![no_std]
#![no_main]

use core::panic::PanicInfo;
use examples_common::*;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    core::arch::wasm32::unreachable()
}

// Debug values
static mut LAYER_COUNT: u8 = 3;
static mut DENSITY: u8 = 200;
static mut HEIGHT_MIN: u8 = 20;
static mut HEIGHT_MAX: u8 = 50;
static mut WIDTH: u8 = 15;
static mut SPACING: u8 = 10;
static mut WAVINESS: u8 = 50;
static mut COLOR_NEAR: u32 = 0x00FF88FF;
static mut COLOR_FAR: u32 = 0x004422FF;
static mut GLOW: u8 = 100;
static mut PARALLAX_RATE: u8 = 128;
static mut PHASE: u32 = 0;
static mut ANIMATE: u8 = 1;
static mut PRESET_INDEX: i32 = 0;

static mut SPHERE_MESH: u32 = 0;
static mut CUBE_MESH: u32 = 0;
static mut TORUS_MESH: u32 = 0;
static mut SHAPE_INDEX: i32 = 0;
static mut CAM_ANGLE: f32 = 0.0;
static mut CAM_ELEVATION: f32 = 10.0;

const SHAPE_COUNT: usize = 3;
const PRESET_COUNT: usize = 3;
const PRESET_NAMES: [&str; PRESET_COUNT] = ["Neon", "Forest", "Pillars"];

fn load_preset(index: usize) {
    unsafe {
        match index {
            0 => { // Neon
                LAYER_COUNT = 2;
                DENSITY = 150;
                HEIGHT_MIN = 30;
                HEIGHT_MAX = 60;
                WIDTH = 5;
                SPACING = 15;
                WAVINESS = 30;
                COLOR_NEAR = 0xFF00FFFF;
                COLOR_FAR = 0x00FFFFFF;
                GLOW = 200;
            }
            1 => { // Forest
                LAYER_COUNT = 3;
                DENSITY = 220;
                HEIGHT_MIN = 20;
                HEIGHT_MAX = 50;
                WIDTH = 8;
                SPACING = 8;
                WAVINESS = 60;
                COLOR_NEAR = 0x2D5A27FF;
                COLOR_FAR = 0x1A3316FF;
                GLOW = 0;
            }
            _ => { // Pillars
                LAYER_COUNT = 2;
                DENSITY = 100;
                HEIGHT_MIN = 40;
                HEIGHT_MAX = 63;
                WIDTH = 20;
                SPACING = 25;
                WAVINESS = 0;
                COLOR_NEAR = 0xCCCCCCFF;
                COLOR_FAR = 0x666666FF;
                GLOW = 30;
            }
        }
    }
}

#[no_mangle]
pub extern "C" fn on_debug_change() {
    unsafe {
        PRESET_INDEX = PRESET_INDEX.clamp(0, PRESET_COUNT as i32 - 1);
    }
}

#[no_mangle]
pub extern "C" fn init() {
    unsafe {
        set_clear_color(0x0A0A1AFF);
        render_mode(2);
        depth_test(1);
        SPHERE_MESH = sphere(1.5, 32, 24);
        CUBE_MESH = cube(2.0, 2.0, 2.0);
        TORUS_MESH = torus(1.3, 0.5, 32, 16);
        load_preset(0);

        debug_group_begin(b"curtains".as_ptr(), 8);
        debug_register_u8(b"layers".as_ptr(), 6, &LAYER_COUNT);
        debug_register_u8(b"density".as_ptr(), 7, &DENSITY);
        debug_register_u8(b"height_min".as_ptr(), 10, &HEIGHT_MIN);
        debug_register_u8(b"height_max".as_ptr(), 10, &HEIGHT_MAX);
        debug_register_u8(b"width".as_ptr(), 5, &WIDTH);
        debug_register_u8(b"spacing".as_ptr(), 7, &SPACING);
        debug_register_u8(b"waviness".as_ptr(), 8, &WAVINESS);
        debug_register_color(b"near".as_ptr(), 4, &COLOR_NEAR as *const u32 as *const u8);
        debug_register_color(b"far".as_ptr(), 3, &COLOR_FAR as *const u32 as *const u8);
        debug_register_u8(b"glow".as_ptr(), 4, &GLOW);
        debug_register_u8(b"animate".as_ptr(), 7, &ANIMATE);
        debug_group_end();

        debug_group_begin(b"preset".as_ptr(), 6);
        debug_register_i32(b"index".as_ptr(), 5, &PRESET_INDEX);
        debug_group_end();
    }
}

#[no_mangle]
pub extern "C" fn update() {
    unsafe {
        if button_pressed(0, BUTTON_A) != 0 {
            PRESET_INDEX = (PRESET_INDEX + 1) % PRESET_COUNT as i32;
            load_preset(PRESET_INDEX as usize);
        }
        if button_pressed(0, BUTTON_B) != 0 {
            SHAPE_INDEX = (SHAPE_INDEX + 1) % SHAPE_COUNT as i32;
        }

        if ANIMATE != 0 {
            PHASE = PHASE.wrapping_add(50);
        }

        let stick_x = left_stick_x(0);
        let stick_y = left_stick_y(0);
        if stick_x.abs() > 0.1 { CAM_ANGLE += stick_x * 2.0; }
        if stick_y.abs() > 0.1 { CAM_ELEVATION = (CAM_ELEVATION - stick_y * 2.0).clamp(-20.0, 60.0); }
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
            0.0, 0.0, 0.0
        );
        camera_fov(60.0);

        env_curtains(
            0, // base layer
            LAYER_COUNT as u32,
            DENSITY as u32,
            HEIGHT_MIN as u32,
            HEIGHT_MAX as u32,
            WIDTH as u32,
            SPACING as u32,
            WAVINESS as u32,
            COLOR_NEAR,
            COLOR_FAR,
            GLOW as u32,
            PARALLAX_RATE as u32,
            PHASE,
        );
        draw_env();

        push_identity();
        set_color(0x333344FF);
        material_metallic(0.8);
        material_roughness(0.2);
        let mesh = match SHAPE_INDEX {
            0 => SPHERE_MESH,
            1 => CUBE_MESH,
            _ => TORUS_MESH,
        };
        draw_mesh(mesh);

        let title = b"Env Mode 6: Curtains";
        draw_text(title.as_ptr(), title.len() as u32, 10.0, 10.0, 20.0, 0xFFFFFFFF);

        let preset_name = PRESET_NAMES[PRESET_INDEX as usize];
        let mut label = [0u8; 32];
        let prefix = b"Preset: ";
        label[..prefix.len()].copy_from_slice(prefix);
        let name = preset_name.as_bytes();
        label[prefix.len()..prefix.len() + name.len()].copy_from_slice(name);
        draw_text(label.as_ptr(), (prefix.len() + name.len()) as u32, 10.0, 40.0, 16.0, 0xCCCCCCFF);

        let hint = b"A: preset | B: shape | Stick: camera | F4: debug";
        draw_text(hint.as_ptr(), hint.len() as u32, 10.0, 70.0, 14.0, 0x888888FF);
    }
}
