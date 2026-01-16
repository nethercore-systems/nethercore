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
static mut FAMILY: u8 = 0; // 0=Portal, 1=Tunnel, 2=Hypnotic, 3=Radar
static mut RING_COUNT: u8 = 48;
static mut THICKNESS: u8 = 28;
static mut COLOR_A: u32 = 0x2EE7FFFF;
static mut COLOR_B: u32 = 0x0B2B4CFF;
static mut CENTER_COLOR: u32 = 0xE8FFFFFF;
static mut CENTER_FALLOFF: u8 = 190;
static mut SPIRAL_TWIST: f32 = 25.0;
static mut AXIS_X: f32 = 0.0;
static mut AXIS_Y: f32 = 0.0;
static mut AXIS_Z: f32 = 1.0;
static mut PHASE: u32 = 0;
static mut ANIMATE: u8 = 1;
static mut PHASE_RATE: u32 = 7864; // phase units/sec
static mut PRESET_INDEX: i32 = 0;

static mut WOBBLE: u32 = 9000;
static mut NOISE: u8 = 32;
static mut DASH: u8 = 24;
static mut GLOW: u8 = 160;
static mut SEED: u8 = 41;

static mut SPHERE_MESH: u32 = 0;
static mut CUBE_MESH: u32 = 0;
static mut TORUS_MESH: u32 = 0;
static mut SHAPE_INDEX: i32 = 0;
static mut CAM_ANGLE: f32 = 0.0;
static mut CAM_ELEVATION: f32 = 0.0;

const SHAPE_COUNT: usize = 3;
const PRESET_COUNT: usize = 4;
const PRESET_NAMES: [&str; PRESET_COUNT] = ["Stargate Portal", "Hyperspace Tunnel", "Op-Art Vortex", "Radar Sweep"];

fn load_preset(index: usize) {
    unsafe {
        match index {
            0 => { // Stargate Portal
                FAMILY = 0;
                RING_COUNT = 48;
                THICKNESS = 28;
                COLOR_A = 0x2EE7FFFF;
                COLOR_B = 0x0B2B4CFF;
                CENTER_COLOR = 0xE8FFFFFF;
                CENTER_FALLOFF = 190;
                SPIRAL_TWIST = 25.0;
                AXIS_X = 0.0;
                AXIS_Y = 0.0;
                AXIS_Z = 1.0;
                WOBBLE = 9000;
                NOISE = 32;
                DASH = 24;
                GLOW = 160;
                SEED = 41;
                PHASE_RATE = 7864;
            }
            1 => { // Hyperspace Tunnel
                FAMILY = 1;
                RING_COUNT = 96;
                THICKNESS = 18;
                COLOR_A = 0x66A3FFFF;
                COLOR_B = 0xA24DFFFF;
                CENTER_COLOR = 0x00000000;
                CENTER_FALLOFF = 0;
                SPIRAL_TWIST = 8.0;
                AXIS_X = 0.0;
                AXIS_Y = 0.0;
                AXIS_Z = 1.0;
                WOBBLE = 16000;
                NOISE = 96;
                DASH = 0;
                GLOW = 220;
                SEED = 13;
                PHASE_RATE = 22938;
            }
            2 => { // Op-Art Vortex
                FAMILY = 2;
                RING_COUNT = 64;
                THICKNESS = 22;
                COLOR_A = 0xFFF0B8FF;
                COLOR_B = 0x1B0A29FF;
                CENTER_COLOR = 0xFFF0B8FF;
                CENTER_FALLOFF = 140;
                SPIRAL_TWIST = 85.0;
                AXIS_X = 0.15;
                AXIS_Y = 0.10;
                AXIS_Z = 0.98;
                WOBBLE = 22000;
                NOISE = 140;
                DASH = 160;
                GLOW = 180;
                SEED = 77;
                PHASE_RATE = 5243;
            }
            _ => { // Radar Sweep
                FAMILY = 3;
                RING_COUNT = 24;
                THICKNESS = 10;
                COLOR_A = 0x38FF9CFF;
                COLOR_B = 0x0B2B1BFF;
                CENTER_COLOR = 0x38FF9C80;
                CENTER_FALLOFF = 40;
                SPIRAL_TWIST = 0.0;
                AXIS_X = 0.0;
                AXIS_Y = 0.0;
                AXIS_Z = 1.0;
                WOBBLE = 0;
                NOISE = 0;
                DASH = 220;
                GLOW = 96;
                SEED = 99;
                PHASE_RATE = 32768;
            }
        }
    }
}

#[no_mangle]
pub extern "C" fn on_debug_change() {
    unsafe {
        PRESET_INDEX = PRESET_INDEX.clamp(0, PRESET_COUNT as i32 - 1);
        FAMILY = FAMILY.clamp(0, 3);
        // Normalize axis
        let len = libm::sqrtf(AXIS_X * AXIS_X + AXIS_Y * AXIS_Y + AXIS_Z * AXIS_Z);
        if len > 0.01 {
            AXIS_X /= len;
            AXIS_Y /= len;
            AXIS_Z /= len;
        } else {
            AXIS_X = 0.0;
            AXIS_Y = 0.0;
            AXIS_Z = 1.0;
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

        debug_group_begin(b"rings".as_ptr(), 5);
        debug_register_u8(b"family".as_ptr(), 6, &FAMILY);
        debug_register_u8(b"count".as_ptr(), 5, &RING_COUNT);
        debug_register_u8(b"thickness".as_ptr(), 9, &THICKNESS);
        debug_register_color(b"color_a".as_ptr(), 7, &COLOR_A as *const u32 as *const u8);
        debug_register_color(b"color_b".as_ptr(), 7, &COLOR_B as *const u32 as *const u8);
        debug_register_color(b"center".as_ptr(), 6, &CENTER_COLOR as *const u32 as *const u8);
        debug_register_u8(b"center_fall".as_ptr(), 11, &CENTER_FALLOFF);
        debug_register_f32(b"spiral".as_ptr(), 6, &SPIRAL_TWIST as *const f32 as *const u8);
        debug_register_u32(b"wobble".as_ptr(), 6, &WOBBLE as *const u32 as *const u8);
        debug_register_u8(b"noise".as_ptr(), 5, &NOISE);
        debug_register_u8(b"dash".as_ptr(), 4, &DASH);
        debug_register_u8(b"glow".as_ptr(), 4, &GLOW);
        debug_register_u8(b"seed".as_ptr(), 4, &SEED);
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

        env_rings(
            0, // base layer
            FAMILY as u32,
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
            WOBBLE,
            NOISE as u32,
            DASH as u32,
            GLOW as u32,
            SEED as u32,
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

        let title = b"Env Mode 7: Rings";
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

        let hint = b"A: preset | B: shape | Stick: camera | F4: Debug Inspector";
        set_color(0x888888FF);
        draw_text(hint.as_ptr(), hint.len() as u32, 10.0, 70.0, 14.0);
    }
}
