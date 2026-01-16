//! Environment Silhouette Inspector - Mode 3 Demo
//!
//! Demonstrates the Mode 3 Silhouette environment: horizon shapes with
//! bounded layered depth and loopable phase-driven motion.

#![no_std]
#![no_main]

use core::panic::PanicInfo;
use examples_common::*;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    core::arch::wasm32::unreachable()
}

// Debug values (Mode 3: Silhouette) — matches the design sheet.
static mut FAMILY: u8 = 0; // 0=Mountains, 1=City, 2=Forest, 3=Waves/Coral
static mut JAGGEDNESS: u8 = 170;
static mut LAYER_COUNT: u8 = 3;
static mut COLOR_NEAR: u32 = 0x141422FF;
static mut COLOR_FAR: u32 = 0x2B2E45FF;
static mut SKY_ZENITH: u32 = 0x0B1538FF;
static mut SKY_HORIZON: u32 = 0xD9774FFF;
static mut PARALLAX_RATE: u8 = 170;
static mut SEED: u32 = 0;
static mut FOG: u8 = 160;
static mut WIND: u8 = 0;

static mut PHASE: u32 = 0;
static mut PHASE_RATE: u32 = 1024; // phase units/sec (65536 = one full loop)
static mut ANIMATE: u8 = 1;

static mut PRESET_INDEX: i32 = 0;

// Internal state
static mut SPHERE_MESH: u32 = 0;
static mut CUBE_MESH: u32 = 0;
static mut TORUS_MESH: u32 = 0;
static mut SHAPE_INDEX: i32 = 0;
static mut CAM_ANGLE: f32 = 0.0;
static mut CAM_ELEVATION: f32 = 10.0;

const SHAPE_COUNT: usize = 3;
const PRESET_COUNT: usize = 4;
const PRESET_NAMES: [&str; PRESET_COUNT] = [
    "Mountain Range (Dusk Layers)",
    "Cyber Skyline (Neon Night)",
    "Forest Canopy (Windy Twilight)",
    "Underwater Reef (Choppy Coral)",
];

fn load_preset(index: usize) {
    unsafe {
        match index {
            0 => {
                // Mountain Range (Dusk Layers)
                FAMILY = 0;
                JAGGEDNESS = 170;
                LAYER_COUNT = 3;
                COLOR_NEAR = 0x141422FF;
                COLOR_FAR = 0x2B2E45FF;
                SKY_ZENITH = 0x0B1538FF;
                SKY_HORIZON = 0xD9774FFF;
                PARALLAX_RATE = 170;
                SEED = 0;
                FOG = 160;
                WIND = 0;
                ANIMATE = 1;
                PHASE_RATE = 1024;
            }
            1 => {
                // Cyber Skyline (Neon Night)
                FAMILY = 1;
                JAGGEDNESS = 200;
                LAYER_COUNT = 2;
                COLOR_NEAR = 0x0A0B12FF;
                COLOR_FAR = 0x14182DFF;
                SKY_ZENITH = 0x050812FF;
                SKY_HORIZON = 0x2A0F4DFF;
                PARALLAX_RATE = 140;
                SEED = 0;
                FOG = 80;
                WIND = 0;
                ANIMATE = 1;
                PHASE_RATE = 2048;
            }
            2 => {
                // Forest Canopy (Windy Twilight)
                FAMILY = 2;
                JAGGEDNESS = 140;
                LAYER_COUNT = 3;
                COLOR_NEAR = 0x07110BFF;
                COLOR_FAR = 0x0D1F16FF;
                SKY_ZENITH = 0x091B2BFF;
                SKY_HORIZON = 0x2B6B5AFF;
                PARALLAX_RATE = 150;
                SEED = 0;
                FOG = 110;
                WIND = 140;
                ANIMATE = 1;
                PHASE_RATE = 4096;
            }
            _ => {
                // Underwater Reef (Choppy Coral)
                FAMILY = 3;
                JAGGEDNESS = 210;
                LAYER_COUNT = 2;
                COLOR_NEAR = 0x031018FF;
                COLOR_FAR = 0x04304CFF;
                SKY_ZENITH = 0x00121CFF;
                SKY_HORIZON = 0x1A6D7AFF;
                PARALLAX_RATE = 120;
                SEED = 0;
                FOG = 200;
                WIND = 180;
                ANIMATE = 1;
                PHASE_RATE = 3072;
            }
        }
    }
}

#[no_mangle]
pub extern "C" fn on_debug_change() {
    unsafe {
        PRESET_INDEX = PRESET_INDEX.clamp(0, PRESET_COUNT as i32 - 1);
        FAMILY = FAMILY.clamp(0, 3);
        LAYER_COUNT = LAYER_COUNT.clamp(1, 3);
    }
}

#[no_mangle]
pub extern "C" fn init() {
    unsafe {
        set_clear_color(0x000000FF);
        SPHERE_MESH = sphere(1.5, 32, 24);
        CUBE_MESH = cube(2.0, 2.0, 2.0);
        TORUS_MESH = torus(1.3, 0.5, 32, 16);
        load_preset(0);
        register_debug_values();
    }
}

unsafe fn register_debug_values() {
    debug_group_begin(b"silhouette".as_ptr(), 10);
    debug_register_u8(b"family".as_ptr(), 6, &FAMILY);
    debug_register_u8(b"jaggedness".as_ptr(), 10, &JAGGEDNESS);
    debug_register_u8(b"layer_count".as_ptr(), 11, &LAYER_COUNT);
    debug_register_u8(b"parallax".as_ptr(), 8, &PARALLAX_RATE);
    debug_register_u8(b"fog".as_ptr(), 3, &FOG);
    debug_register_u8(b"wind".as_ptr(), 4, &WIND);
    debug_register_u32(b"seed".as_ptr(), 4, &SEED as *const u32 as *const u8);
    debug_group_end();

    debug_group_begin(b"colors".as_ptr(), 6);
    debug_register_color(b"color_near".as_ptr(), 10, &COLOR_NEAR as *const u32 as *const u8);
    debug_register_color(b"color_far".as_ptr(), 9, &COLOR_FAR as *const u32 as *const u8);
    debug_register_color(b"sky_zenith".as_ptr(), 10, &SKY_ZENITH as *const u32 as *const u8);
    debug_register_color(
        b"sky_horizon".as_ptr(),
        11,
        &SKY_HORIZON as *const u32 as *const u8,
    );
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
        if button_pressed(0, button::X) != 0 {
            SEED = if SEED == 0 { 1 } else { SEED.wrapping_add(1) };
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

        let cos_e = libm::cosf(elev_rad);
        let sin_e = libm::sinf(elev_rad);
        let cos_a = libm::cosf(angle_rad);
        let sin_a = libm::sinf(angle_rad);

        camera_set(
            dist * cos_e * sin_a,
            dist * sin_e,
            dist * cos_e * cos_a,
            0.0,
            0.0,
            0.0,
        );
        camera_fov(60.0);

        env_silhouette(
            0, // base layer
            FAMILY as u32,
            JAGGEDNESS as u32,
            LAYER_COUNT as u32,
            COLOR_NEAR,
            COLOR_FAR,
            SKY_ZENITH,
            SKY_HORIZON,
            PARALLAX_RATE as u32,
            SEED,
            PHASE,
            FOG as u32,
            WIND as u32,
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
        set_color(0xFFFFFFFF);
        draw_text(title.as_ptr(), title.len() as u32, 10.0, 10.0, 20.0);

        let preset_name = PRESET_NAMES[PRESET_INDEX as usize];
        let mut label = [0u8; 64];
        let prefix = b"Preset: ";
        label[..prefix.len()].copy_from_slice(prefix);
        let name = preset_name.as_bytes();
        label[prefix.len()..prefix.len() + name.len()].copy_from_slice(name);
        set_color(0xCCCCCCFF);
        draw_text(
            label.as_ptr(),
            (prefix.len() + name.len()) as u32,
            10.0,
            40.0,
            16.0,
        );

        let hint = b"A: preset | B: shape | X: seed | F4: Debug Inspector";
        set_color(0x888888FF);
        draw_text(hint.as_ptr(), hint.len() as u32, 10.0, 70.0, 14.0);
    }
}
