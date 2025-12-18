//! Environment Rectangles Inspector - Mode 4 Demo
//!
//! Demonstrates rectangular light sources (windows, screens, panels).

#![no_std]
#![no_main]

use core::panic::PanicInfo;
use examples_common::*;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    core::arch::wasm32::unreachable()
}

// Debug values
static mut VARIANT: u8 = 1; // Buildings
static mut DENSITY: u8 = 180;
static mut LIT_RATIO: u8 = 200;
static mut SIZE_MIN: u8 = 10;
static mut SIZE_MAX: u8 = 30;
static mut ASPECT: u8 = 2;
static mut COLOR_PRIMARY: u32 = 0xFFFF99FF;
static mut COLOR_VARIATION: u32 = 0xFF9966FF;
static mut PARALLAX_RATE: u8 = 50;
static mut PHASE: u32 = 0;
static mut ANIMATE: u8 = 1;

static mut SPHERE_MESH: u32 = 0;
static mut CAM_ANGLE: f32 = 0.0;
static mut CAM_ELEVATION: f32 = 10.0;

const VARIANT_NAMES: [&str; 4] = ["Scatter", "Buildings", "Bands", "Panels"];

#[no_mangle]
pub extern "C" fn on_debug_change() {
    unsafe {
        VARIANT = VARIANT.min(3);
    }
}

#[no_mangle]
pub extern "C" fn init() {
    unsafe {
        set_clear_color(0x0A0A14FF);
        render_mode(2);
        depth_test(1);
        SPHERE_MESH = sphere(1.5, 32, 24);

        debug_group_begin(b"rectangles".as_ptr(), 10);
        debug_register_u8(b"variant".as_ptr(), 7, &VARIANT);
        debug_register_u8(b"density".as_ptr(), 7, &DENSITY);
        debug_register_u8(b"lit_ratio".as_ptr(), 9, &LIT_RATIO);
        debug_register_u8(b"size_min".as_ptr(), 8, &SIZE_MIN);
        debug_register_u8(b"size_max".as_ptr(), 8, &SIZE_MAX);
        debug_register_u8(b"aspect".as_ptr(), 6, &ASPECT);
        debug_register_color(b"color".as_ptr(), 5, &COLOR_PRIMARY as *const u32 as *const u8);
        debug_register_color(b"variation".as_ptr(), 9, &COLOR_VARIATION as *const u32 as *const u8);
        debug_register_u8(b"animate".as_ptr(), 7, &ANIMATE);
        debug_group_end();
    }
}

#[no_mangle]
pub extern "C" fn update() {
    unsafe {
        if button_pressed(0, BUTTON_A) != 0 {
            VARIANT = (VARIANT + 1) % 4;
        }

        if ANIMATE != 0 {
            PHASE = PHASE.wrapping_add(100);
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

        env_select_pair(4, 4); // Rectangles mode
        env_rectangles_set(
            VARIANT as u32,
            DENSITY as u32,
            LIT_RATIO as u32,
            SIZE_MIN as u32,
            SIZE_MAX as u32,
            ASPECT as u32,
            COLOR_PRIMARY,
            COLOR_VARIATION,
            PARALLAX_RATE as u32,
            PHASE,
        );
        draw_sky();

        push_identity();
        set_color(0x222233FF);
        material_metallic(0.9);
        material_roughness(0.1);
        draw_mesh(SPHERE_MESH);

        let title = b"Env Mode 4: Rectangles";
        draw_text(title.as_ptr(), title.len() as u32, 10.0, 10.0, 20.0, 0xFFFFFFFF);

        let variant_name = VARIANT_NAMES[VARIANT as usize];
        let mut label = [0u8; 32];
        let prefix = b"Variant: ";
        label[..prefix.len()].copy_from_slice(prefix);
        let name = variant_name.as_bytes();
        label[prefix.len()..prefix.len() + name.len()].copy_from_slice(name);
        draw_text(label.as_ptr(), (prefix.len() + name.len()) as u32, 10.0, 40.0, 16.0, 0xCCCCCCFF);

        let hint = b"A: cycle variants | F3: debug panel";
        draw_text(hint.as_ptr(), hint.len() as u32, 10.0, 70.0, 14.0, 0x888888FF);
    }
}
