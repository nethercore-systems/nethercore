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
// Variant: 0=Floor, 1=Ceiling, 2=Sphere (WHERE lines appear)
// Line Type: 0=Horizontal, 1=Vertical, 2=Grid (PATTERN of lines)
static mut VARIANT: u8 = 0;
static mut LINE_TYPE: u8 = 2;
static mut THICKNESS: u8 = 20;
static mut SPACING: f32 = 0.5;
static mut FADE_DISTANCE: f32 = 5.0;
static mut COLOR_PRIMARY: u32 = 0x00FF88FF;
static mut COLOR_ACCENT: u32 = 0xFF00AAFF;
static mut ACCENT_EVERY: u8 = 5;
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
const PRESET_COUNT: usize = 4;
const PRESET_NAMES: [&str; PRESET_COUNT] = ["Grid", "Scanlines", "Horizon", "Ceiling"];

fn load_preset(index: usize) {
    unsafe {
        match index {
            0 => { // Grid - floor grid (synthwave style)
                VARIANT = 0;      // Floor
                LINE_TYPE = 2;    // Grid pattern
                THICKNESS = 15;
                SPACING = 0.5;
                FADE_DISTANCE = 8.0;
                COLOR_PRIMARY = 0x00FF88FF;
                COLOR_ACCENT = 0xFFFFFFFF;
                ACCENT_EVERY = 5;
            }
            1 => { // Scanlines - sphere horizontal lines
                VARIANT = 2;      // Sphere
                LINE_TYPE = 0;    // Horizontal
                THICKNESS = 8;
                SPACING = 0.15;
                FADE_DISTANCE = 10.0;
                COLOR_PRIMARY = 0x33FF33FF;
                COLOR_ACCENT = 0x88FF88FF;
                ACCENT_EVERY = 10;
            }
            2 => { // Horizon - floor horizontal lines
                VARIANT = 0;      // Floor
                LINE_TYPE = 0;    // Horizontal
                THICKNESS = 25;
                SPACING = 0.4;
                FADE_DISTANCE = 12.0;
                COLOR_PRIMARY = 0xFF6600FF;
                COLOR_ACCENT = 0xFFFF00FF;
                ACCENT_EVERY = 4;
            }
            _ => { // Ceiling - ceiling grid
                VARIANT = 1;      // Ceiling
                LINE_TYPE = 2;    // Grid pattern
                THICKNESS = 20;
                SPACING = 0.3;
                FADE_DISTANCE = 6.0;
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
        VARIANT = VARIANT.clamp(0, 2);     // 0=Floor, 1=Ceiling, 2=Sphere
        LINE_TYPE = LINE_TYPE.clamp(0, 2); // 0=Horizontal, 1=Vertical, 2=Grid
        SPACING = SPACING.clamp(0.05, 2.0);
        FADE_DISTANCE = FADE_DISTANCE.clamp(1.0, 20.0);
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

        debug_group_begin(b"lines".as_ptr(), 5);
        debug_register_u8(b"variant".as_ptr(), 7, &VARIANT);
        debug_register_u8(b"line_type".as_ptr(), 9, &LINE_TYPE);
        debug_register_u8(b"thickness".as_ptr(), 9, &THICKNESS);
        debug_register_f32(b"spacing".as_ptr(), 7, &SPACING as *const f32 as *const u8);
        debug_register_f32(b"fade_dist".as_ptr(), 9, &FADE_DISTANCE as *const f32 as *const u8);
        debug_register_color(b"primary".as_ptr(), 7, &COLOR_PRIMARY as *const u32 as *const u8);
        debug_register_color(b"accent".as_ptr(), 6, &COLOR_ACCENT as *const u32 as *const u8);
        debug_register_u8(b"accent_n".as_ptr(), 8, &ACCENT_EVERY);
        debug_group_end();

        debug_group_begin(b"animation".as_ptr(), 9);
        debug_register_u8(b"animate".as_ptr(), 7, &ANIMATE);
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

        env_lines(
            0, // base layer
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

        let title = b"Env Mode 2: Lines";
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
