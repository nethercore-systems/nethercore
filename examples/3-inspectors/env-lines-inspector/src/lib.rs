//! Environment Lines Inspector - Mode 2 Demo
//!
//! Demonstrates line-based patterns: grids, lanes, scanlines, caustic bands.

#![no_std]
#![no_main]

use core::panic::PanicInfo;
use examples_common::*;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    core::arch::wasm32::unreachable()
}

// Debug values (Mode 2: Lines) — matches the design sheet.
static mut VARIANT: u8 = 0; // 0=Floor, 1=Ceiling, 2=Sphere
static mut LINE_TYPE: u8 = 2; // 0=Horizontal, 1=Vertical, 2=Grid
static mut THICKNESS: u8 = 18;
static mut SPACING: f32 = 2.25;
static mut FADE_DISTANCE: f32 = 80.0;
static mut PARALLAX: u8 = 0;
static mut COLOR_PRIMARY: u32 = 0x00FFB0C0;
static mut COLOR_ACCENT: u32 = 0xFF3AF0FF;
static mut ACCENT_EVERY: u8 = 8;
static mut PROFILE: u8 = 0; // 0=Grid, 1=Lanes, 2=Scanlines, 3=Caustic
static mut WARP: u8 = 24;
static mut WOBBLE: u8 = 0;
static mut GLOW: u8 = 96;
static mut AXIS_X: f32 = 0.0;
static mut AXIS_Y: f32 = 0.0;
static mut AXIS_Z: f32 = 1.0;
static mut SEED: u32 = 0x4D2F5A10;

static mut PHASE: u32 = 0;
static mut PHASE_RATE: u32 = 8192; // phase units/sec (65536 = one full loop)
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
    "Synth Grid",
    "Racing Lanes",
    "CRT Scanlines",
    "Underwater Bands",
];

fn load_preset(index: usize) {
    unsafe {
        match index {
            0 => {
                // Synth Grid (Anchor or Overlay)
                VARIANT = 0;
                LINE_TYPE = 2;
                THICKNESS = 18;
                SPACING = 2.25;
                FADE_DISTANCE = 80.0;
                PARALLAX = 0;
                COLOR_PRIMARY = 0x00FFB0C0;
                COLOR_ACCENT = 0xFF3AF0FF;
                ACCENT_EVERY = 8;
                PROFILE = 0;
                WARP = 24;
                WOBBLE = 0;
                GLOW = 96;
                AXIS_X = 0.0;
                AXIS_Y = 0.0;
                AXIS_Z = 1.0;
                SEED = 0x4D2F5A10;
                PHASE_RATE = 8192;
            }
            1 => {
                // Racing Lanes (Overlay)
                VARIANT = 0;
                LINE_TYPE = 1;
                THICKNESS = 40;
                SPACING = 1.35;
                FADE_DISTANCE = 140.0;
                PARALLAX = 0;
                COLOR_PRIMARY = 0xFFFFFFC0;
                COLOR_ACCENT = 0xFFB000FF;
                ACCENT_EVERY = 4;
                PROFILE = 1;
                WARP = 8;
                WOBBLE = 0;
                GLOW = 160;
                AXIS_X = 0.0;
                AXIS_Y = 0.0;
                AXIS_Z = 1.0;
                SEED = 0x00C0FFEE;
                PHASE_RATE = 32768;
            }
            2 => {
                // CRT Scanlines (Overlay)
                VARIANT = 2;
                LINE_TYPE = 0;
                THICKNESS = 14;
                SPACING = 0.075;
                FADE_DISTANCE = 1.0;
                PARALLAX = 0;
                COLOR_PRIMARY = 0x0A0F14A0;
                COLOR_ACCENT = 0x101824A0;
                ACCENT_EVERY = 16;
                PROFILE = 2;
                WARP = 96;
                WOBBLE = 22;
                GLOW = 0;
                AXIS_X = 0.0;
                AXIS_Y = 0.0;
                AXIS_Z = 1.0;
                SEED = 0x1A2B3C4D;
                PHASE_RATE = 16384;
            }
            _ => {
                // Underwater Bands (Overlay)
                VARIANT = 2;
                LINE_TYPE = 0;
                THICKNESS = 96;
                SPACING = 0.45;
                FADE_DISTANCE = 1.0;
                PARALLAX = 0;
                COLOR_PRIMARY = 0x00BFE0A0;
                COLOR_ACCENT = 0xE0FFFFA0;
                ACCENT_EVERY = 3;
                PROFILE = 3;
                WARP = 32;
                WOBBLE = 200;
                GLOW = 96;
                AXIS_X = 0.0;
                AXIS_Y = 0.0;
                AXIS_Z = 1.0;
                SEED = 0xBADC0FFE;
                PHASE_RATE = 8192;
            }
        }
    }
}

#[no_mangle]
pub extern "C" fn on_debug_change() {
    unsafe {
        PRESET_INDEX = PRESET_INDEX.clamp(0, PRESET_COUNT as i32 - 1);
        VARIANT = VARIANT.clamp(0, 2);
        LINE_TYPE = LINE_TYPE.clamp(0, 2);
        PROFILE = PROFILE.clamp(0, 3);
        SPACING = SPACING.clamp(0.0001, 1000.0);
        FADE_DISTANCE = FADE_DISTANCE.clamp(0.01, 1000.0);
        ACCENT_EVERY = ACCENT_EVERY.max(1);

        // Normalize axis.
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

        debug_group_begin(b"lines".as_ptr(), 5);
        debug_register_u8(b"variant".as_ptr(), 7, &VARIANT);
        debug_register_u8(b"line_type".as_ptr(), 9, &LINE_TYPE);
        debug_register_u8(b"profile".as_ptr(), 7, &PROFILE);
        debug_register_u8(b"thickness".as_ptr(), 9, &THICKNESS);
        debug_register_f32(b"spacing".as_ptr(), 7, &SPACING as *const f32 as *const u8);
        debug_register_f32(b"fade_dist".as_ptr(), 9, &FADE_DISTANCE as *const f32 as *const u8);
        debug_register_u8(b"parallax".as_ptr(), 8, &PARALLAX);
        debug_register_color(b"primary".as_ptr(), 7, &COLOR_PRIMARY as *const u32 as *const u8);
        debug_register_color(b"accent".as_ptr(), 6, &COLOR_ACCENT as *const u32 as *const u8);
        debug_register_u8(b"accent_n".as_ptr(), 8, &ACCENT_EVERY);
        debug_register_u8(b"warp".as_ptr(), 4, &WARP);
        debug_register_u8(b"wobble".as_ptr(), 6, &WOBBLE);
        debug_register_u8(b"glow".as_ptr(), 4, &GLOW);
        debug_group_end();

        debug_group_begin(b"axis".as_ptr(), 4);
        debug_register_f32(b"x".as_ptr(), 1, &AXIS_X as *const f32 as *const u8);
        debug_register_f32(b"y".as_ptr(), 1, &AXIS_Y as *const f32 as *const u8);
        debug_register_f32(b"z".as_ptr(), 1, &AXIS_Z as *const f32 as *const u8);
        debug_register_u32(b"seed".as_ptr(), 4, &SEED as *const u32 as *const u8);
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
        if stick_x.abs() > 0.1 {
            CAM_ANGLE += stick_x * 2.0;
        }
        if stick_y.abs() > 0.1 {
            CAM_ELEVATION = (CAM_ELEVATION - stick_y * 2.0).clamp(-60.0, 60.0);
        }
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
            0.0,
            0.0,
            0.0,
        );
        camera_fov(60.0);

        env_lines(
            0, // base layer
            VARIANT as u32,
            LINE_TYPE as u32,
            THICKNESS as u32,
            SPACING,
            FADE_DISTANCE,
            PARALLAX as u32,
            COLOR_PRIMARY,
            COLOR_ACCENT,
            ACCENT_EVERY as u32,
            PHASE,
            PROFILE as u32,
            WARP as u32,
            WOBBLE as u32,
            GLOW as u32,
            AXIS_X,
            AXIS_Y,
            AXIS_Z,
            SEED,
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
