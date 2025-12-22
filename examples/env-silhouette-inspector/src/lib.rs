//! Environment Silhouette Inspector - Mode 3 Demo
//!
//! Demonstrates the silhouette environment mode with layered terrain.
//!
//! Features:
//! - Multiple preset terrain styles (Mountains, Cityscape, Forest)
//! - Adjustable jaggedness and layer count
//! - Debug panel for real-time parameter tweaking

#![no_std]
#![no_main]

use core::panic::PanicInfo;
use examples_common::*;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    core::arch::wasm32::unreachable()
}

// Debug values
static mut JAGGEDNESS: u8 = 150;
static mut LAYER_COUNT: u8 = 3;
static mut COLOR_NEAR: u32 = 0x1A1A2EFF;
static mut COLOR_FAR: u32 = 0x4A4A6AFF;
static mut SKY_ZENITH: u32 = 0xFF6B35FF;
static mut SKY_HORIZON: u32 = 0xFFA07AFF;
static mut PARALLAX_RATE: u8 = 128;
static mut SEED: u32 = 12345;
static mut PRESET_INDEX: i32 = 0;

// Internal state
static mut SPHERE_MESH: u32 = 0;
static mut CUBE_MESH: u32 = 0;
static mut TORUS_MESH: u32 = 0;
static mut SHAPE_INDEX: i32 = 0;
static mut CAM_ANGLE: f32 = 0.0;
static mut CAM_ELEVATION: f32 = 10.0;

const SHAPE_COUNT: usize = 3;
const PRESET_COUNT: usize = 3;
const PRESET_NAMES: [&str; PRESET_COUNT] = ["Mountains", "Cityscape", "Forest"];

#[no_mangle]
pub extern "C" fn on_debug_change() {
    unsafe {
        PRESET_INDEX = PRESET_INDEX.clamp(0, PRESET_COUNT as i32 - 1);
    }
}

fn load_preset(index: usize) {
    unsafe {
        match index {
            0 => { // Mountains
                JAGGEDNESS = 200;
                LAYER_COUNT = 3;
                COLOR_NEAR = 0x2F4F4FFF;
                COLOR_FAR = 0x708090FF;
                SKY_ZENITH = 0x87CEEBFF;
                SKY_HORIZON = 0xFFE4B5FF;
                PARALLAX_RATE = 150;
            }
            1 => { // Cityscape
                JAGGEDNESS = 50;
                LAYER_COUNT = 2;
                COLOR_NEAR = 0x1A1A2EFF;
                COLOR_FAR = 0x3D3D5CFF;
                SKY_ZENITH = 0x0D0D1AFF;
                SKY_HORIZON = 0x4A0080FF;
                PARALLAX_RATE = 100;
            }
            _ => { // Forest
                JAGGEDNESS = 100;
                LAYER_COUNT = 3;
                COLOR_NEAR = 0x228B22FF;
                COLOR_FAR = 0x556B2FFF;
                SKY_ZENITH = 0xFF6B35FF;
                SKY_HORIZON = 0xFFA07AFF;
                PARALLAX_RATE = 120;
            }
        }
    }
}

#[no_mangle]
pub extern "C" fn init() {
    unsafe {
        set_clear_color(0x000000FF);
        render_mode(2);
        depth_test(1);
        SPHERE_MESH = sphere(1.5, 32, 24);
        CUBE_MESH = cube(2.0, 2.0, 2.0);
        TORUS_MESH = torus(1.3, 0.5, 32, 16);
        load_preset(0);
        register_debug_values();
    }
}

unsafe fn register_debug_values() {
    debug_group_begin(b"silhouette".as_ptr(), 10);
    debug_register_u8(b"jaggedness".as_ptr(), 10, &JAGGEDNESS);
    debug_register_u8(b"layer_count".as_ptr(), 11, &LAYER_COUNT);
    debug_register_color(b"color_near".as_ptr(), 10, &COLOR_NEAR as *const u32 as *const u8);
    debug_register_color(b"color_far".as_ptr(), 9, &COLOR_FAR as *const u32 as *const u8);
    debug_register_color(b"sky_zenith".as_ptr(), 10, &SKY_ZENITH as *const u32 as *const u8);
    debug_register_color(b"sky_horizon".as_ptr(), 11, &SKY_HORIZON as *const u32 as *const u8);
    debug_register_u8(b"parallax".as_ptr(), 8, &PARALLAX_RATE);
    debug_group_end();

    debug_group_begin(b"preset".as_ptr(), 6);
    debug_register_i32(b"index".as_ptr(), 5, &PRESET_INDEX);
    debug_group_end();
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
        if button_pressed(0, BUTTON_X) != 0 {
            SEED = SEED.wrapping_add(1);
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

        let cos_e = libm::cosf(elev_rad);
        let sin_e = libm::sinf(elev_rad);
        let cos_a = libm::cosf(angle_rad);
        let sin_a = libm::sinf(angle_rad);

        camera_set(dist * cos_e * sin_a, dist * sin_e, dist * cos_e * cos_a, 0.0, 0.0, 0.0);
        camera_fov(60.0);

        env_silhouette(
            0, // base layer
            JAGGEDNESS as u32,
            LAYER_COUNT as u32,
            COLOR_NEAR,
            COLOR_FAR,
            SKY_ZENITH,
            SKY_HORIZON,
            PARALLAX_RATE as u32,
            SEED,
        );
        draw_env();

        push_identity();
        set_color(0xCCCCCCFF);
        material_metallic(0.8);
        material_roughness(0.2);
        let mesh = match SHAPE_INDEX {
            0 => SPHERE_MESH,
            1 => CUBE_MESH,
            _ => TORUS_MESH,
        };
        draw_mesh(mesh);

        let title = b"Env Mode 3: Silhouette";
        draw_text(title.as_ptr(), title.len() as u32, 10.0, 10.0, 20.0, 0xFFFFFFFF);

        let preset_name = PRESET_NAMES[PRESET_INDEX as usize];
        let mut label = [0u8; 32];
        let prefix = b"Preset: ";
        label[..prefix.len()].copy_from_slice(prefix);
        let name = preset_name.as_bytes();
        label[prefix.len()..prefix.len() + name.len()].copy_from_slice(name);
        draw_text(label.as_ptr(), (prefix.len() + name.len()) as u32, 10.0, 40.0, 16.0, 0xCCCCCCFF);

        let hint = b"A: preset | B: shape | X: seed | F4: debug";
        draw_text(hint.as_ptr(), hint.len() as u32, 10.0, 70.0, 14.0, 0x888888FF);
    }
}
