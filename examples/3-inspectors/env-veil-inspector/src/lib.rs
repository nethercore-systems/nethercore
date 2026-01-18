//! Environment Veil Inspector - Mode 6 Demo
//!
//! Demonstrates Veil: direction-based SDF ribbons/pillars (bounded depth slices).

#![no_std]
#![no_main]

use core::panic::PanicInfo;
use examples_common::*;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    core::arch::wasm32::unreachable()
}

// Debug values (Mode 6: Veil) — matches the design sheet.
static mut FAMILY: u8 = 0; // 0=Pillars, 1=Drapes, 2=Shards, 3=Soft Veils
static mut DENSITY: u8 = 210;
static mut WIDTH: u8 = 38;
static mut TAPER: u8 = 60;
static mut CURVATURE: u8 = 80;
static mut EDGE_SOFT: u8 = 130;
static mut HEIGHT_MIN: u8 = 96;
static mut HEIGHT_MAX: u8 = 232;
static mut COLOR_NEAR: u32 = 0x2D5A27FF;
static mut COLOR_FAR: u32 = 0x142A12FF;
static mut GLOW: u8 = 0;
static mut PARALLAX: u8 = 160;
static mut AXIS_X: f32 = 0.0;
static mut AXIS_Y: f32 = 1.0;
static mut AXIS_Z: f32 = 0.0;
static mut SEED: u32 = 0;

static mut PHASE: u32 = 0;
static mut PHASE_RATE: u32 = 2400;
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
    "Forest Trunks",
    "Temple Colonnade",
    "Neon Drapes",
    "Crystal Shards",
];

fn load_preset(index: usize) {
    unsafe {
        match index {
            0 => {
                // Forest Trunks
                FAMILY = 0;
                DENSITY = 210;
                WIDTH = 38;
                TAPER = 60;
                CURVATURE = 80;
                EDGE_SOFT = 130;
                HEIGHT_MIN = 96;
                HEIGHT_MAX = 232;
                COLOR_NEAR = 0x2D5A27FF;
                COLOR_FAR = 0x142A12FF;
                GLOW = 0;
                PARALLAX = 160;
                AXIS_X = 0.0;
                AXIS_Y = 1.0;
                AXIS_Z = 0.0;
                SEED = 0;
                PHASE_RATE = 2400; // 40/frame @ 60fps
            }
            1 => {
                // Temple Colonnade
                FAMILY = 0;
                DENSITY = 96;
                WIDTH = 80;
                TAPER = 20;
                CURVATURE = 10;
                EDGE_SOFT = 90;
                HEIGHT_MIN = 104;
                HEIGHT_MAX = 248;
                COLOR_NEAR = 0xC8C2B6FF;
                COLOR_FAR = 0x6F6A60FF;
                GLOW = 10;
                PARALLAX = 120;
                AXIS_X = 0.0;
                AXIS_Y = 1.0;
                AXIS_Z = 0.0;
                SEED = 0;
                PHASE_RATE = 480; // 8/frame @ 60fps
            }
            2 => {
                // Neon Drapes
                FAMILY = 1;
                DENSITY = 140;
                WIDTH = 28;
                TAPER = 190;
                CURVATURE = 170;
                EDGE_SOFT = 80;
                HEIGHT_MIN = 88;
                HEIGHT_MAX = 248;
                COLOR_NEAR = 0xFF2BD6FF;
                COLOR_FAR = 0x00E5FFFF;
                GLOW = 220;
                PARALLAX = 200;
                AXIS_X = 0.15;
                AXIS_Y = 0.97;
                AXIS_Z = 0.18;
                SEED = 0;
                PHASE_RATE = 7200; // 120/frame @ 60fps
            }
            _ => {
                // Crystal Shards
                FAMILY = 2;
                DENSITY = 110;
                WIDTH = 44;
                TAPER = 230;
                CURVATURE = 35;
                EDGE_SOFT = 35;
                HEIGHT_MIN = 80;
                HEIGHT_MAX = 255;
                COLOR_NEAR = 0x8FE7FFFF;
                COLOR_FAR = 0x3A1E66FF;
                GLOW = 90;
                PARALLAX = 180;
                AXIS_X = 0.25;
                AXIS_Y = 0.96;
                AXIS_Z = 0.10;
                SEED = 0;
                PHASE_RATE = 3600; // 60/frame @ 60fps
            }
        }
    }
}

#[no_mangle]
pub extern "C" fn on_debug_change() {
    unsafe {
        PRESET_INDEX = PRESET_INDEX.clamp(0, PRESET_COUNT as i32 - 1);
        FAMILY = FAMILY.clamp(0, 3);
        if HEIGHT_MIN > HEIGHT_MAX {
            let tmp = HEIGHT_MIN;
            HEIGHT_MIN = HEIGHT_MAX;
            HEIGHT_MAX = tmp;
        }

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
        set_clear_color(0x0A0A1AFF);
        SPHERE_MESH = sphere(1.5, 32, 24);
        CUBE_MESH = cube(2.0, 2.0, 2.0);
        TORUS_MESH = torus(1.3, 0.5, 32, 16);
        load_preset(0);

        debug_group_begin(b"veil".as_ptr(), 4);
        debug_register_u8(b"family".as_ptr(), 6, core::ptr::addr_of!(FAMILY));
        debug_register_u8(b"density".as_ptr(), 7, core::ptr::addr_of!(DENSITY));
        debug_register_u8(b"width".as_ptr(), 5, core::ptr::addr_of!(WIDTH));
        debug_register_u8(b"taper".as_ptr(), 5, core::ptr::addr_of!(TAPER));
        debug_register_u8(b"curv".as_ptr(), 4, core::ptr::addr_of!(CURVATURE));
        debug_register_u8(b"edge_soft".as_ptr(), 9, core::ptr::addr_of!(EDGE_SOFT));
        debug_register_u8(b"h_min".as_ptr(), 5, core::ptr::addr_of!(HEIGHT_MIN));
        debug_register_u8(b"h_max".as_ptr(), 5, core::ptr::addr_of!(HEIGHT_MAX));
        debug_register_color(
            b"near".as_ptr(),
            4,
            core::ptr::addr_of!(COLOR_NEAR) as *const u8,
        );
        debug_register_color(
            b"far".as_ptr(),
            3,
            core::ptr::addr_of!(COLOR_FAR) as *const u8,
        );
        debug_register_u8(b"glow".as_ptr(), 4, core::ptr::addr_of!(GLOW));
        debug_register_u8(b"parallax".as_ptr(), 8, core::ptr::addr_of!(PARALLAX));
        debug_register_u32(b"seed".as_ptr(), 4, core::ptr::addr_of!(SEED) as *const u8);
        debug_group_end();

        debug_group_begin(b"axis".as_ptr(), 4);
        debug_register_f32(b"x".as_ptr(), 1, core::ptr::addr_of!(AXIS_X) as *const u8);
        debug_register_f32(b"y".as_ptr(), 1, core::ptr::addr_of!(AXIS_Y) as *const u8);
        debug_register_f32(b"z".as_ptr(), 1, core::ptr::addr_of!(AXIS_Z) as *const u8);
        debug_group_end();

        debug_group_begin(b"animation".as_ptr(), 9);
        debug_register_u8(b"animate".as_ptr(), 7, core::ptr::addr_of!(ANIMATE));
        debug_register_u32(b"phase".as_ptr(), 5, core::ptr::addr_of!(PHASE) as *const u8);
        debug_register_u32(b"rate".as_ptr(), 4, core::ptr::addr_of!(PHASE_RATE) as *const u8);
        debug_group_end();

        debug_group_begin(b"preset".as_ptr(), 6);
        debug_register_i32(
            b"index".as_ptr(),
            5,
            core::ptr::addr_of!(PRESET_INDEX) as *const u8,
        );
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

        env_veil(
            0, // base layer
            FAMILY as u32,
            DENSITY as u32,
            WIDTH as u32,
            TAPER as u32,
            CURVATURE as u32,
            EDGE_SOFT as u32,
            HEIGHT_MIN as u32,
            HEIGHT_MAX as u32,
            COLOR_NEAR,
            COLOR_FAR,
            GLOW as u32,
            PARALLAX as u32,
            AXIS_X,
            AXIS_Y,
            AXIS_Z,
            PHASE,
            SEED,
        );
        draw_env();

        push_identity();
        set_color(0xFFFFFFFF);
        material_metallic(0.8);
        material_roughness(0.4);
        let mesh = match SHAPE_INDEX {
            0 => SPHERE_MESH,
            1 => CUBE_MESH,
            _ => TORUS_MESH,
        };
        draw_mesh(mesh);

        let title = b"Env Mode 6: Veil";
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
