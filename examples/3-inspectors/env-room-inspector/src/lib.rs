//! Environment Room Inspector - Mode 5 Demo
//!
//! Demonstrates the Mode 5 Room environment: axis-aligned interior box
//! with viewer parallax, directional lighting, and loopable accent animation.

#![no_std]
#![no_main]

use core::panic::PanicInfo;
use examples_common::*;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    core::arch::wasm32::unreachable()
}

// Debug values (Mode 5: Room) — matches the design sheet.
static mut COLOR_CEILING: u32 = 0x1A1A1A00;
static mut COLOR_FLOOR: u32 = 0x241A1400;
static mut COLOR_WALLS: u32 = 0x2E2B2700;
static mut PANEL_SIZE: f32 = 1.6;
static mut PANEL_GAP: u8 = 18;
static mut CORNER_DARKEN: u8 = 190;

static mut LIGHT_DIR_X: f32 = 0.35;
static mut LIGHT_DIR_Y: f32 = -0.80;
static mut LIGHT_DIR_Z: f32 = 0.48;
static mut LIGHT_INTENSITY: u8 = 175;
static mut LIGHT_TINT: u32 = 0xFFB06000;

static mut ROOM_SCALE: f32 = 3.0;
static mut VIEWER_X: i32 = 0;
static mut VIEWER_Y: i32 = 0;
static mut VIEWER_Z: i32 = 0;

static mut ACCENT: u8 = 0;
static mut ACCENT_MODE: u8 = 0; // 0=Seams, 1=Sweep, 2=Seams+Sweep, 3=Pulse
static mut ROUGHNESS: u8 = 230;

static mut PHASE: u32 = 0;
static mut PHASE_RATE: u32 = 0; // phase units/sec (65536 = one full loop)
static mut ANIMATE: u8 = 0;

static mut PRESET_INDEX: i32 = 0;

// Scene state
static mut SPHERE_MESH: u32 = 0;
static mut CUBE_MESH: u32 = 0;
static mut TORUS_MESH: u32 = 0;
static mut SHAPE_INDEX: i32 = 0;
static mut CAM_ANGLE: f32 = 0.0;
static mut CAM_ELEVATION: f32 = 10.0;

const SHAPE_COUNT: usize = 3;
const PRESET_COUNT: usize = 4;
const PRESET_NAMES: [&str; PRESET_COUNT] = [
    "Dungeon Stone",
    "Sterile Lab",
    "Neon Bay",
    "Underwater Tank",
];

fn load_preset(index: usize) {
    unsafe {
        match index {
            0 => {
                // Dungeon Stone
                COLOR_CEILING = 0x1A1A1A00;
                COLOR_FLOOR = 0x241A1400;
                COLOR_WALLS = 0x2E2B2700;
                PANEL_SIZE = 1.6;
                PANEL_GAP = 18;
                CORNER_DARKEN = 190;
                LIGHT_DIR_X = 0.35;
                LIGHT_DIR_Y = -0.80;
                LIGHT_DIR_Z = 0.48;
                LIGHT_INTENSITY = 175;
                LIGHT_TINT = 0xFFB06000;
                ROOM_SCALE = 3.0;
                VIEWER_X = 0;
                VIEWER_Y = -20;
                VIEWER_Z = 40;
                ACCENT = 0;
                ACCENT_MODE = 0;
                ROUGHNESS = 230;
                ANIMATE = 0;
                PHASE_RATE = 0;
            }
            1 => {
                // Sterile Lab
                COLOR_CEILING = 0xEAF4FF00;
                COLOR_FLOOR = 0xC7CCD200;
                COLOR_WALLS = 0xE2E6EA00;
                PANEL_SIZE = 0.65;
                PANEL_GAP = 42;
                CORNER_DARKEN = 35;
                LIGHT_DIR_X = 0.0;
                LIGHT_DIR_Y = -1.0;
                LIGHT_DIR_Z = 0.0;
                LIGHT_INTENSITY = 210;
                LIGHT_TINT = 0xD8F4FF00;
                ROOM_SCALE = 2.6;
                VIEWER_X = 0;
                VIEWER_Y = 0;
                VIEWER_Z = 0;
                ACCENT = 80;
                ACCENT_MODE = 0;
                ROUGHNESS = 90;
                ANIMATE = 1;
                PHASE_RATE = 8192;
            }
            2 => {
                // Neon Bay
                COLOR_CEILING = 0x060A1200;
                COLOR_FLOOR = 0x09040C00;
                COLOR_WALLS = 0x12061A00;
                PANEL_SIZE = 0.95;
                PANEL_GAP = 28;
                CORNER_DARKEN = 105;
                LIGHT_DIR_X = 0.62;
                LIGHT_DIR_Y = -0.28;
                LIGHT_DIR_Z = 0.73;
                LIGHT_INTENSITY = 235;
                LIGHT_TINT = 0xFF40FF00;
                ROOM_SCALE = 4.2;
                VIEWER_X = 25;
                VIEWER_Y = -10;
                VIEWER_Z = 60;
                ACCENT = 230;
                ACCENT_MODE = 2;
                ROUGHNESS = 35;
                ANIMATE = 1;
                PHASE_RATE = 21845;
            }
            _ => {
                // Underwater Tank
                COLOR_CEILING = 0x06101A00;
                COLOR_FLOOR = 0x04131A00;
                COLOR_WALLS = 0x05202A00;
                PANEL_SIZE = 8.0;
                PANEL_GAP = 0;
                CORNER_DARKEN = 70;
                LIGHT_DIR_X = 0.10;
                LIGHT_DIR_Y = -0.70;
                LIGHT_DIR_Z = 0.70;
                LIGHT_INTENSITY = 165;
                LIGHT_TINT = 0x40FFD000;
                ROOM_SCALE = 6.0;
                VIEWER_X = 0;
                VIEWER_Y = -30;
                VIEWER_Z = 20;
                ACCENT = 110;
                ACCENT_MODE = 1;
                ROUGHNESS = 180;
                ANIMATE = 1;
                PHASE_RATE = 4096;
            }
        }
    }
}

#[no_mangle]
pub extern "C" fn on_debug_change() {
    unsafe {
        PRESET_INDEX = PRESET_INDEX.clamp(0, PRESET_COUNT as i32 - 1);
        ACCENT_MODE = ACCENT_MODE.clamp(0, 3);

        VIEWER_X = VIEWER_X.clamp(-128, 127);
        VIEWER_Y = VIEWER_Y.clamp(-128, 127);
        VIEWER_Z = VIEWER_Z.clamp(-128, 127);
        ROOM_SCALE = ROOM_SCALE.clamp(0.1, 25.5);
        PANEL_SIZE = PANEL_SIZE.clamp(0.05, 32.0);

        // Normalize light dir (fallback to downward).
        let len = libm::sqrtf(
            LIGHT_DIR_X * LIGHT_DIR_X + LIGHT_DIR_Y * LIGHT_DIR_Y + LIGHT_DIR_Z * LIGHT_DIR_Z,
        );
        if len > 0.01 {
            LIGHT_DIR_X /= len;
            LIGHT_DIR_Y /= len;
            LIGHT_DIR_Z /= len;
        } else {
            LIGHT_DIR_X = 0.0;
            LIGHT_DIR_Y = -1.0;
            LIGHT_DIR_Z = 0.0;
        }
    }
}

#[no_mangle]
pub extern "C" fn init() {
    unsafe {
        set_clear_color(0x000000FF);
        SPHERE_MESH = sphere(0.8, 24, 16);
        CUBE_MESH = cube(1.0, 1.0, 1.0);
        TORUS_MESH = torus(0.7, 0.25, 32, 16);
        load_preset(0);

        debug_group_begin(b"room".as_ptr(), 4);
        debug_register_color(
            b"ceiling".as_ptr(),
            7,
            &COLOR_CEILING as *const u32 as *const u8,
        );
        debug_register_color(b"floor".as_ptr(), 5, &COLOR_FLOOR as *const u32 as *const u8);
        debug_register_color(b"walls".as_ptr(), 5, &COLOR_WALLS as *const u32 as *const u8);
        debug_register_color(
            b"light_tint".as_ptr(),
            10,
            &LIGHT_TINT as *const u32 as *const u8,
        );
        debug_register_f32(b"panel_size".as_ptr(), 10, &PANEL_SIZE as *const f32 as *const u8);
        debug_register_u8(b"panel_gap".as_ptr(), 9, &PANEL_GAP);
        debug_register_u8(b"corner_dark".as_ptr(), 11, &CORNER_DARKEN);
        debug_register_f32(b"ldx".as_ptr(), 3, &LIGHT_DIR_X as *const f32 as *const u8);
        debug_register_f32(b"ldy".as_ptr(), 3, &LIGHT_DIR_Y as *const f32 as *const u8);
        debug_register_f32(b"ldz".as_ptr(), 3, &LIGHT_DIR_Z as *const f32 as *const u8);
        debug_register_u8(b"light".as_ptr(), 5, &LIGHT_INTENSITY);
        debug_register_u8(b"rough".as_ptr(), 5, &ROUGHNESS);
        debug_register_f32(b"scale".as_ptr(), 5, &ROOM_SCALE as *const f32 as *const u8);
        debug_register_u8(b"accent".as_ptr(), 6, &ACCENT);
        debug_register_u8(b"accent_m".as_ptr(), 8, &ACCENT_MODE);
        debug_group_end();

        debug_group_begin(b"viewer".as_ptr(), 6);
        debug_register_i32(b"x".as_ptr(), 1, &VIEWER_X as *const i32 as *const u8);
        debug_register_i32(b"y".as_ptr(), 1, &VIEWER_Y as *const i32 as *const u8);
        debug_register_i32(b"z".as_ptr(), 1, &VIEWER_Z as *const i32 as *const u8);
        debug_group_end();

        debug_group_begin(b"animation".as_ptr(), 9);
        debug_register_u8(b"animate".as_ptr(), 7, &ANIMATE);
        debug_register_u32(b"phase".as_ptr(), 5, &PHASE as *const u32 as *const u8);
        debug_register_u32(b"rate".as_ptr(), 4, &PHASE_RATE as *const u32 as *const u8);
        debug_group_end();

        debug_group_begin(b"preset".as_ptr(), 6);
        debug_register_i32(
            b"index".as_ptr(),
            5,
            &PRESET_INDEX as *const i32 as *const u8,
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

        // Move viewer with left stick (XZ) and triggers (Y).
        let stick_x = left_stick_x(0);
        let stick_y = left_stick_y(0);
        if stick_x.abs() > 0.1 {
            VIEWER_X = (VIEWER_X + (stick_x * 2.0) as i32).clamp(-128, 127);
        }
        if stick_y.abs() > 0.1 {
            VIEWER_Z = (VIEWER_Z - (stick_y * 2.0) as i32).clamp(-128, 127);
        }

        let lt = trigger_left(0);
        let rt = trigger_right(0);
        if lt > 0.1 {
            VIEWER_Y = (VIEWER_Y - 1).max(-128);
        }
        if rt > 0.1 {
            VIEWER_Y = (VIEWER_Y + 1).min(127);
        }

        // Camera orbit with right stick.
        let r_stick_x = right_stick_x(0);
        let r_stick_y = right_stick_y(0);
        if r_stick_x.abs() > 0.1 {
            CAM_ANGLE += r_stick_x * 2.0;
        }
        if r_stick_y.abs() > 0.1 {
            CAM_ELEVATION = (CAM_ELEVATION - r_stick_y * 2.0).clamp(-60.0, 60.0);
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

        env_room(
            0, // base layer
            COLOR_CEILING,
            COLOR_FLOOR,
            COLOR_WALLS,
            PANEL_SIZE,
            PANEL_GAP as u32,
            LIGHT_DIR_X,
            LIGHT_DIR_Y,
            LIGHT_DIR_Z,
            LIGHT_INTENSITY as u32,
            LIGHT_TINT,
            CORNER_DARKEN as u32,
            ROOM_SCALE,
            VIEWER_X,
            VIEWER_Y,
            VIEWER_Z,
            ACCENT as u32,
            ACCENT_MODE as u32,
            ROUGHNESS as u32,
            PHASE,
        );
        draw_env();

        // Draw a marker mesh at the viewer position (scaled to room).
        push_identity();
        let scale = ROOM_SCALE / 127.0;
        push_translate(
            VIEWER_X as f32 * scale,
            VIEWER_Y as f32 * scale,
            VIEWER_Z as f32 * scale,
        );
        set_color(0xFFCC00FF);
        material_metallic(0.6);
        material_roughness(0.3);
        let mesh = match SHAPE_INDEX {
            0 => SPHERE_MESH,
            1 => CUBE_MESH,
            _ => TORUS_MESH,
        };
        draw_mesh(mesh);

        let title = b"Env Mode 5: Room";
        set_color(0xFFFFFFFF);
        draw_text(title.as_ptr(), title.len() as u32, 10.0, 10.0, 20.0);

        let preset_name = PRESET_NAMES[PRESET_INDEX as usize];
        let mut label = [0u8; 40];
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

        let hint = b"A: preset | L-stick: XZ | Triggers: Y | R-stick: cam | B: shape | F4: Debug";
        set_color(0x888888FF);
        draw_text(hint.as_ptr(), hint.len() as u32, 10.0, 70.0, 14.0);
    }
}
