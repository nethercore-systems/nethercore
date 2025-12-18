//! Environment Rings Inspector - Mode 7 Demo
//!
//! Demonstrates concentric rings for portals, tunnels, and vortex effects.

#![no_std]
#![no_main]

use core::panic::PanicInfo;
use examples_common::*;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    core::arch::wasm32::unreachable()
}

// Debug values
static mut RING_COUNT: u8 = 12;
static mut THICKNESS: u8 = 128;
static mut COLOR_A: u32 = 0x6600FFFF;
static mut COLOR_B: u32 = 0x00FFCCFF;
static mut CENTER_COLOR: u32 = 0xFFFFFFFF;
static mut CENTER_FALLOFF: u8 = 50;
static mut SPIRAL_TWIST: f32 = 0.0;
static mut AXIS_X: f32 = 0.0;
static mut AXIS_Y: f32 = 0.0;
static mut AXIS_Z: f32 = -1.0;
static mut PHASE: u32 = 0;
static mut ANIMATE: u8 = 1;
static mut SPIN_SPEED: f32 = 2.0;
static mut PRESET_INDEX: i32 = 0;

static mut SPHERE_MESH: u32 = 0;
static mut CAM_ANGLE: f32 = 0.0;
static mut CAM_ELEVATION: f32 = 0.0;

const PRESET_COUNT: usize = 4;
const PRESET_NAMES: [&str; PRESET_COUNT] = ["Portal", "Tunnel", "Hypnotic", "Spiral"];

fn load_preset(index: usize) {
    unsafe {
        match index {
            0 => { // Portal
                RING_COUNT = 8;
                THICKNESS = 150;
                COLOR_A = 0x6600FFFF;
                COLOR_B = 0x00FFCCFF;
                CENTER_COLOR = 0xFFFFFFFF;
                CENTER_FALLOFF = 80;
                SPIRAL_TWIST = 0.0;
                SPIN_SPEED = 2.0;
            }
            1 => { // Tunnel
                RING_COUNT = 20;
                THICKNESS = 100;
                COLOR_A = 0x333333FF;
                COLOR_B = 0x666666FF;
                CENTER_COLOR = 0xFFFFFFFF;
                CENTER_FALLOFF = 30;
                SPIRAL_TWIST = 0.0;
                SPIN_SPEED = 0.5;
            }
            2 => { // Hypnotic
                RING_COUNT = 6;
                THICKNESS = 200;
                COLOR_A = 0x000000FF;
                COLOR_B = 0xFFFFFFFF;
                CENTER_COLOR = 0xFF0000FF;
                CENTER_FALLOFF = 100;
                SPIRAL_TWIST = 0.0;
                SPIN_SPEED = 5.0;
            }
            _ => { // Spiral
                RING_COUNT = 12;
                THICKNESS = 128;
                COLOR_A = 0xFF00AAFF;
                COLOR_B = 0x00AAFFFF;
                CENTER_COLOR = 0xFFFF00FF;
                CENTER_FALLOFF = 60;
                SPIRAL_TWIST = 180.0;
                SPIN_SPEED = 3.0;
            }
        }
        AXIS_X = 0.0;
        AXIS_Y = 0.0;
        AXIS_Z = -1.0;
    }
}

#[no_mangle]
pub extern "C" fn on_debug_change() {
    unsafe {
        PRESET_INDEX = PRESET_INDEX.clamp(0, PRESET_COUNT as i32 - 1);
        // Normalize axis
        let len = libm::sqrtf(AXIS_X * AXIS_X + AXIS_Y * AXIS_Y + AXIS_Z * AXIS_Z);
        if len > 0.01 {
            AXIS_X /= len;
            AXIS_Y /= len;
            AXIS_Z /= len;
        } else {
            AXIS_Z = -1.0;
        }
    }
}

#[no_mangle]
pub extern "C" fn init() {
    unsafe {
        set_clear_color(0x0A0A14FF);
        render_mode(2);
        depth_test(1);
        SPHERE_MESH = sphere(1.0, 32, 24);
        load_preset(0);

        debug_group_begin(b"rings".as_ptr(), 5);
        debug_register_u8(b"count".as_ptr(), 5, &RING_COUNT);
        debug_register_u8(b"thickness".as_ptr(), 9, &THICKNESS);
        debug_register_color(b"color_a".as_ptr(), 7, &COLOR_A as *const u32 as *const u8);
        debug_register_color(b"color_b".as_ptr(), 7, &COLOR_B as *const u32 as *const u8);
        debug_register_color(b"center".as_ptr(), 6, &CENTER_COLOR as *const u32 as *const u8);
        debug_register_u8(b"center_fall".as_ptr(), 11, &CENTER_FALLOFF);
        debug_register_f32(b"spiral".as_ptr(), 6, &SPIRAL_TWIST);
        debug_group_end();

        debug_group_begin(b"axis".as_ptr(), 4);
        debug_register_f32(b"x".as_ptr(), 1, &AXIS_X);
        debug_register_f32(b"y".as_ptr(), 1, &AXIS_Y);
        debug_register_f32(b"z".as_ptr(), 1, &AXIS_Z);
        debug_group_end();

        debug_group_begin(b"animation".as_ptr(), 9);
        debug_register_u8(b"animate".as_ptr(), 7, &ANIMATE);
        debug_register_f32(b"speed".as_ptr(), 5, &SPIN_SPEED);
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

        if ANIMATE != 0 {
            PHASE = PHASE.wrapping_add((SPIN_SPEED * 1000.0) as u32);
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

        env_select_pair(7, 7); // Rings mode
        env_rings_set(
            RING_COUNT as u32,
            THICKNESS as u32,
            COLOR_A,
            COLOR_B,
            CENTER_COLOR,
            CENTER_FALLOFF as u32,
            SPIRAL_TWIST,
            AXIS_X,
            AXIS_Y,
            AXIS_Z,
            PHASE,
        );
        draw_sky();

        push_identity();
        set_color(0x222233FF);
        material_metallic(0.9);
        material_roughness(0.1);
        draw_mesh(SPHERE_MESH);

        let title = b"Env Mode 7: Rings";
        draw_text(title.as_ptr(), title.len() as u32, 10.0, 10.0, 20.0, 0xFFFFFFFF);

        let preset_name = PRESET_NAMES[PRESET_INDEX as usize];
        let mut label = [0u8; 32];
        let prefix = b"Preset: ";
        label[..prefix.len()].copy_from_slice(prefix);
        let name = preset_name.as_bytes();
        label[prefix.len()..prefix.len() + name.len()].copy_from_slice(name);
        draw_text(label.as_ptr(), (prefix.len() + name.len()) as u32, 10.0, 40.0, 16.0, 0xCCCCCCFF);

        let hint = b"A: cycle presets | Left stick: camera | F3: debug";
        draw_text(hint.as_ptr(), hint.len() as u32, 10.0, 70.0, 14.0, 0x888888FF);
    }
}
