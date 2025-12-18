//! Environment Lines Inspector - Mode 2 Demo
//!
//! Demonstrates line-based patterns: grids, horizons, scan lines, lasers.

#![no_std]
#![no_main]

use core::panic::PanicInfo;
use examples_common::*;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    core::arch::wasm32::unreachable()
}

// Debug values
static mut VARIANT: u8 = 0;       // 0=horizontal, 1=vertical, 2=grid, 3=radial
static mut LINE_TYPE: u8 = 0;     // 0=solid, 1=dashed, 2=dotted, 3=glow
static mut THICKNESS: u8 = 20;
static mut SPACING: f32 = 0.1;
static mut FADE_DISTANCE: f32 = 1.0;
static mut COLOR_PRIMARY: u32 = 0x00FF88FF;
static mut COLOR_ACCENT: u32 = 0xFF00AAFF;
static mut ACCENT_EVERY: u8 = 5;
static mut PHASE: u32 = 0;
static mut ANIMATE: u8 = 1;
static mut PRESET_INDEX: i32 = 0;

static mut SPHERE_MESH: u32 = 0;
static mut CAM_ANGLE: f32 = 0.0;
static mut CAM_ELEVATION: f32 = 10.0;

const PRESET_COUNT: usize = 4;
const PRESET_NAMES: [&str; PRESET_COUNT] = ["Grid", "Scanlines", "Horizon", "Laser"];

fn load_preset(index: usize) {
    unsafe {
        match index {
            0 => { // Grid
                VARIANT = 2;      // grid
                LINE_TYPE = 0;    // solid
                THICKNESS = 15;
                SPACING = 0.08;
                FADE_DISTANCE = 0.8;
                COLOR_PRIMARY = 0x00FF88FF;
                COLOR_ACCENT = 0xFFFFFFFF;
                ACCENT_EVERY = 5;
            }
            1 => { // Scanlines
                VARIANT = 0;      // horizontal
                LINE_TYPE = 0;    // solid
                THICKNESS = 8;
                SPACING = 0.02;
                FADE_DISTANCE = 0.0;
                COLOR_PRIMARY = 0x33FF33FF;
                COLOR_ACCENT = 0x88FF88FF;
                ACCENT_EVERY = 10;
            }
            2 => { // Horizon
                VARIANT = 0;      // horizontal
                LINE_TYPE = 3;    // glow
                THICKNESS = 40;
                SPACING = 0.15;
                FADE_DISTANCE = 1.2;
                COLOR_PRIMARY = 0xFF6600FF;
                COLOR_ACCENT = 0xFFFF00FF;
                ACCENT_EVERY = 4;
            }
            _ => { // Laser
                VARIANT = 3;      // radial
                LINE_TYPE = 3;    // glow
                THICKNESS = 25;
                SPACING = 0.2;
                FADE_DISTANCE = 0.5;
                COLOR_PRIMARY = 0xFF0066FF;
                COLOR_ACCENT = 0x00FFFFFF;
                ACCENT_EVERY = 3;
            }
        }
    }
}

#[no_mangle]
pub extern "C" fn on_debug_change() {
    unsafe {
        PRESET_INDEX = PRESET_INDEX.clamp(0, PRESET_COUNT as i32 - 1);
        VARIANT = VARIANT.clamp(0, 3);
        LINE_TYPE = LINE_TYPE.clamp(0, 3);
        SPACING = SPACING.clamp(0.01, 1.0);
        FADE_DISTANCE = FADE_DISTANCE.clamp(0.0, 2.0);
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

        debug_group_begin(b"lines".as_ptr(), 5);
        debug_register_u8(b"variant".as_ptr(), 7, &VARIANT);
        debug_register_u8(b"line_type".as_ptr(), 9, &LINE_TYPE);
        debug_register_u8(b"thickness".as_ptr(), 9, &THICKNESS);
        debug_register_f32(b"spacing".as_ptr(), 7, &SPACING);
        debug_register_f32(b"fade_dist".as_ptr(), 9, &FADE_DISTANCE);
        debug_register_color(b"primary".as_ptr(), 7, &COLOR_PRIMARY as *const u32 as *const u8);
        debug_register_color(b"accent".as_ptr(), 6, &COLOR_ACCENT as *const u32 as *const u8);
        debug_register_u8(b"accent_n".as_ptr(), 8, &ACCENT_EVERY);
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

        if ANIMATE != 0 {
            PHASE = PHASE.wrapping_add(80);
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

        env_select_pair(2, 2); // Lines mode
        env_lines_set(
            VARIANT as u32,
            LINE_TYPE as u32,
            THICKNESS as u32,
            SPACING,
            FADE_DISTANCE,
            COLOR_PRIMARY,
            COLOR_ACCENT,
            ACCENT_EVERY as u32,
            PHASE,
        );
        draw_sky();

        push_identity();
        set_color(0x222233FF);
        material_metallic(0.9);
        material_roughness(0.1);
        draw_mesh(SPHERE_MESH);

        let title = b"Env Mode 2: Lines";
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
