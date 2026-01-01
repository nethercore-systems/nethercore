//! Environment Scatter Inspector - Mode 1 Demo
//!
//! Demonstrates scattered elements: stars, particles, snow, rain effects.

#![no_std]
#![no_main]

use core::panic::PanicInfo;
use examples_common::*;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    core::arch::wasm32::unreachable()
}

// Debug values
// Variant: 0=Stars(twinkle), 1=Vertical(rain/snow), 2=Horizontal(speed), 3=Warp(radial)
static mut VARIANT: u8 = 0;
static mut DENSITY: u8 = 180;
static mut SIZE: u8 = 15;
static mut GLOW: u8 = 64;
static mut STREAK_LENGTH: u8 = 0;
static mut COLOR_PRIMARY: u32 = 0xFFFFFFFF;
static mut COLOR_SECONDARY: u32 = 0x8888FFFF;
static mut PARALLAX_RATE: u8 = 128;
static mut PARALLAX_SIZE: u8 = 64;
static mut PHASE: u32 = 0;
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
const PRESET_NAMES: [&str; PRESET_COUNT] = ["Stars", "Snow", "Rain", "Particles"];

fn load_preset(index: usize) {
    unsafe {
        match index {
            0 => { // Stars - twinkle, no movement
                VARIANT = 0;  // twinkle
                DENSITY = 200;
                SIZE = 8;
                GLOW = 100;
                STREAK_LENGTH = 0;
                COLOR_PRIMARY = 0xFFFFFFFF;
                COLOR_SECONDARY = 0xAAAAFFFF;
                PARALLAX_RATE = 20;
                PARALLAX_SIZE = 50;
            }
            1 => { // Snow - vertical fall, soft dots
                VARIANT = 1;  // vertical movement
                DENSITY = 150;
                SIZE = 12;
                GLOW = 60;
                STREAK_LENGTH = 0;
                COLOR_PRIMARY = 0xFFFFFFFF;
                COLOR_SECONDARY = 0xDDDDFFFF;
                PARALLAX_RATE = 80;
                PARALLAX_SIZE = 60;
            }
            2 => { // Rain - vertical fall with streaks
                VARIANT = 1;  // vertical movement
                DENSITY = 200;
                SIZE = 6;
                GLOW = 30;
                STREAK_LENGTH = 40;
                COLOR_PRIMARY = 0x8899CCFF;
                COLOR_SECONDARY = 0x6677AAFF;
                PARALLAX_RATE = 200;
                PARALLAX_SIZE = 30;
            }
            _ => { // Particles - warp/radial movement
                VARIANT = 3;  // warp outward
                DENSITY = 180;
                SIZE = 10;
                GLOW = 150;
                STREAK_LENGTH = 0;
                COLOR_PRIMARY = 0xFF8800FF;
                COLOR_SECONDARY = 0xFFFF00FF;
                PARALLAX_RATE = 100;
                PARALLAX_SIZE = 50;
            }
        }
    }
}

#[no_mangle]
pub extern "C" fn on_debug_change() {
    unsafe {
        PRESET_INDEX = PRESET_INDEX.clamp(0, PRESET_COUNT as i32 - 1);
        VARIANT = VARIANT.clamp(0, 3);
    }
}

#[no_mangle]
pub extern "C" fn init() {
    unsafe {
        set_clear_color(0x0A0A14FF);
        render_mode(2);
        depth_test(1);
        SPHERE_MESH = sphere(1.0, 32, 24);
        CUBE_MESH = cube(1.4, 1.4, 1.4);
        TORUS_MESH = torus(1.0, 0.4, 32, 16);
        load_preset(0);

        debug_group_begin(b"scatter".as_ptr(), 7);
        debug_register_u8(b"variant".as_ptr(), 7, &VARIANT);
        debug_register_u8(b"density".as_ptr(), 7, &DENSITY);
        debug_register_u8(b"size".as_ptr(), 4, &SIZE);
        debug_register_u8(b"glow".as_ptr(), 4, &GLOW);
        debug_register_u8(b"streak_len".as_ptr(), 10, &STREAK_LENGTH);
        debug_register_color(b"primary".as_ptr(), 7, &COLOR_PRIMARY as *const u32 as *const u8);
        debug_register_color(b"secondary".as_ptr(), 9, &COLOR_SECONDARY as *const u32 as *const u8);
        debug_group_end();

        debug_group_begin(b"parallax".as_ptr(), 8);
        debug_register_u8(b"rate".as_ptr(), 4, &PARALLAX_RATE);
        debug_register_u8(b"size".as_ptr(), 4, &PARALLAX_SIZE);
        debug_group_end();

        debug_group_begin(b"animation".as_ptr(), 9);
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
            PHASE = PHASE.wrapping_add(100);
        }

        let stick_x = left_stick_x(0);
        let stick_y = left_stick_y(0);
        if stick_x.abs() > 0.1 { CAM_ANGLE += stick_x * 2.0; }
        if stick_y.abs() > 0.1 { CAM_ELEVATION = (CAM_ELEVATION - stick_y * 2.0).clamp(-60.0, 60.0); }
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
            0.0, 0.0, 0.0
        );
        camera_fov(60.0);

        env_scatter(
            0, // base layer
            VARIANT as u32,
            DENSITY as u32,
            SIZE as u32,
            GLOW as u32,
            STREAK_LENGTH as u32,
            COLOR_PRIMARY,
            COLOR_SECONDARY,
            PARALLAX_RATE as u32,
            PARALLAX_SIZE as u32,
            PHASE,
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

        let title = b"Env Mode 1: Scatter";
        set_color(0xFFFFFFFF);
        draw_text(title.as_ptr(), title.len() as u32, 10.0, 10.0, 20.0);

        let preset_name = PRESET_NAMES[PRESET_INDEX as usize];
        let mut label = [0u8; 32];
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
