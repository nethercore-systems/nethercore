//! Environment Room Inspector - Mode 5 Demo
//!
//! Demonstrates interior room environment with viewer position tracking.

#![no_std]
#![no_main]

use core::panic::PanicInfo;
use examples_common::*;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    core::arch::wasm32::unreachable()
}

// Debug values
static mut COLOR_CEILING: u32 = 0x404040FF;
static mut COLOR_FLOOR: u32 = 0x2A2A2AFF;
static mut COLOR_WALLS: u32 = 0x333333FF;
static mut PANEL_SIZE: f32 = 0.5;
static mut PANEL_GAP: u8 = 30;
static mut LIGHT_INTENSITY: u8 = 200;
static mut CORNER_DARKEN: u8 = 100;
static mut ROOM_SCALE: f32 = 2.0;
static mut VIEWER_X: i32 = 0;
static mut VIEWER_Y: i32 = 0;
static mut VIEWER_Z: i32 = 0;

static mut SPHERE_MESH: u32 = 0;
static mut CUBE_MESH: u32 = 0;
static mut TORUS_MESH: u32 = 0;
static mut SHAPE_INDEX: i32 = 0;
static mut CAM_ANGLE: f32 = 0.0;
static mut CAM_ELEVATION: f32 = 10.0;

const SHAPE_COUNT: usize = 3;

#[no_mangle]
pub extern "C" fn on_debug_change() {
    unsafe {
        VIEWER_X = VIEWER_X.clamp(-127, 127);
        VIEWER_Y = VIEWER_Y.clamp(-127, 127);
        VIEWER_Z = VIEWER_Z.clamp(-127, 127);
        ROOM_SCALE = ROOM_SCALE.clamp(0.5, 10.0);
        PANEL_SIZE = PANEL_SIZE.clamp(0.1, 2.0);
    }
}

#[no_mangle]
pub extern "C" fn init() {
    unsafe {
        set_clear_color(0x000000FF);
        render_mode(2);
        SPHERE_MESH = sphere(0.8, 24, 16);
        CUBE_MESH = cube(1.0, 1.0, 1.0);
        TORUS_MESH = torus(0.7, 0.25, 32, 16);

        debug_group_begin(b"room".as_ptr(), 4);
        debug_register_color(b"ceiling".as_ptr(), 7, &COLOR_CEILING as *const u32 as *const u8);
        debug_register_color(b"floor".as_ptr(), 5, &COLOR_FLOOR as *const u32 as *const u8);
        debug_register_color(b"walls".as_ptr(), 5, &COLOR_WALLS as *const u32 as *const u8);
        debug_register_f32(b"panel_size".as_ptr(), 10, &PANEL_SIZE as *const f32 as *const u8);
        debug_register_u8(b"panel_gap".as_ptr(), 9, &PANEL_GAP);
        debug_register_u8(b"light".as_ptr(), 5, &LIGHT_INTENSITY);
        debug_register_u8(b"corner_dark".as_ptr(), 11, &CORNER_DARKEN);
        debug_register_f32(b"scale".as_ptr(), 5, &ROOM_SCALE as *const f32 as *const u8);
        debug_group_end();

        debug_group_begin(b"viewer".as_ptr(), 6);
        debug_register_i32(b"x".as_ptr(), 1, &VIEWER_X as *const i32 as *const u8);
        debug_register_i32(b"y".as_ptr(), 1, &VIEWER_Y as *const i32 as *const u8);
        debug_register_i32(b"z".as_ptr(), 1, &VIEWER_Z as *const i32 as *const u8);
        debug_group_end();
    }
}

#[no_mangle]
pub extern "C" fn update() {
    unsafe {
        if button_pressed(0, button::B) != 0 {
            SHAPE_INDEX = (SHAPE_INDEX + 1) % SHAPE_COUNT as i32;
        }

        // Move viewer with left stick
        let stick_x = left_stick_x(0);
        let stick_y = left_stick_y(0);

        if stick_x.abs() > 0.1 {
            VIEWER_X = (VIEWER_X + (stick_x * 2.0) as i32).clamp(-127, 127);
        }
        if stick_y.abs() > 0.1 {
            VIEWER_Z = (VIEWER_Z - (stick_y * 2.0) as i32).clamp(-127, 127);
        }

        // Camera orbit with right stick
        let r_stick_x = right_stick_x(0);
        let r_stick_y = right_stick_y(0);
        if r_stick_x.abs() > 0.1 { CAM_ANGLE += r_stick_x * 2.0; }
        if r_stick_y.abs() > 0.1 { CAM_ELEVATION = (CAM_ELEVATION - r_stick_y * 2.0).clamp(-60.0, 60.0); }

        // Vertical movement with triggers
        let lt = trigger_left(0);
        let rt = trigger_right(0);
        if lt > 0.1 { VIEWER_Y = (VIEWER_Y - 1).max(-127); }
        if rt > 0.1 { VIEWER_Y = (VIEWER_Y + 1).min(127); }
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

        env_room(
            0, // base layer
            COLOR_CEILING,
            COLOR_FLOOR,
            COLOR_WALLS,
            PANEL_SIZE,
            PANEL_GAP as u32,
            0.3, -0.8, 0.5, // light direction
            LIGHT_INTENSITY as u32,
            CORNER_DARKEN as u32,
            ROOM_SCALE,
            VIEWER_X,
            VIEWER_Y,
            VIEWER_Z,
        );
        draw_env();

        // Draw sphere at viewer position (scaled to room)
        push_identity();
        let scale = ROOM_SCALE / 127.0;
        push_translate(
            VIEWER_X as f32 * scale,
            VIEWER_Y as f32 * scale,
            VIEWER_Z as f32 * scale
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

        let hint = b"L-stick: XZ | Triggers: Y | R-stick: cam | B: shape | F4: Debug Inspector";
        set_color(0x888888FF);
        draw_text(hint.as_ptr(), hint.len() as u32, 10.0, 40.0, 14.0);
    }
}
