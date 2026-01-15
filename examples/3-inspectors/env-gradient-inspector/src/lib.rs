//! Environment Gradient Inspector - Mode 0 Demo
//!
//! Demonstrates 4-point gradient backgrounds with featured-sky controls (sun + haze + bands).

#![no_std]
#![no_main]

use core::panic::PanicInfo;
use examples_common::*;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    core::arch::wasm32::unreachable()
}

// Debug values
static mut ZENITH_COLOR: u32 = 0x000033FF;
static mut SKY_HORIZON: u32 = 0xFF6600FF;
static mut GROUND_HORIZON: u32 = 0x663300FF;
static mut NADIR_COLOR: u32 = 0x1A0A00FF;
static mut SHIFT: f32 = 0.0;
static mut ANIMATE: u8 = 1;
static mut PRESET_INDEX: i32 = 0;

// Featured sky controls
static mut SUN_AZIMUTH_DEG: f32 = 0.0;
static mut SUN_ELEVATION_DEG: f32 = 25.0;
static mut SUN_DISK: u8 = 32;      // 0-255 (mapped in shader)
static mut SUN_HALO: u8 = 140;     // 0-255 (mapped in shader)
static mut SUN_INTENSITY: u8 = 80; // 0-255 (0 disables sun)
static mut SUN_WARMTH: u8 = 120;   // 0-255
static mut SKY_HAZE: u8 = 80;      // 0-255
static mut CLOUDINESS: u8 = 80;    // 0-255 (bands)

static mut SPHERE_MESH: u32 = 0;
static mut CUBE_MESH: u32 = 0;
static mut TORUS_MESH: u32 = 0;
static mut SHAPE_INDEX: i32 = 0;
static mut CAM_ANGLE: f32 = 0.0;
static mut CAM_ELEVATION: f32 = 0.0;

const SHAPE_COUNT: usize = 3;
const SHAPE_NAMES: [&str; SHAPE_COUNT] = ["Sphere", "Cube", "Torus"];

const PRESET_COUNT: usize = 4;
const PRESET_NAMES: [&str; PRESET_COUNT] = ["Day", "Sunset", "Stormy", "Alien"];

fn load_preset(index: usize) {
    unsafe {
        match index {
            0 => { // Day
                ZENITH_COLOR = 0x1E90FFFF;
                SKY_HORIZON = 0x87CEEBFF;
                GROUND_HORIZON = 0x8B7355FF;
                NADIR_COLOR = 0x4A3728FF;

                SUN_AZIMUTH_DEG = 35.0;
                SUN_ELEVATION_DEG = 35.0;
                SUN_DISK = 24;
                SUN_HALO = 120;
                SUN_INTENSITY = 90;
                SUN_WARMTH = 40;
                SKY_HAZE = 60;
                CLOUDINESS = 40;
            }
            1 => { // Sunset
                ZENITH_COLOR = 0x000033FF;
                SKY_HORIZON = 0xFF6600FF;
                GROUND_HORIZON = 0x663300FF;
                NADIR_COLOR = 0x1A0A00FF;

                SUN_AZIMUTH_DEG = 95.0;
                SUN_ELEVATION_DEG = 8.0;
                SUN_DISK = 42;
                SUN_HALO = 170;
                SUN_INTENSITY = 130;
                SUN_WARMTH = 180;
                SKY_HAZE = 170;
                CLOUDINESS = 140;
            }
            2 => { // Stormy
                ZENITH_COLOR = 0x1A2433FF;
                SKY_HORIZON = 0x4A6A80FF;
                GROUND_HORIZON = 0x1A1F24FF;
                NADIR_COLOR = 0x0A0C0FFF;

                SUN_AZIMUTH_DEG = 10.0;
                SUN_ELEVATION_DEG = 15.0;
                SUN_DISK = 10;
                SUN_HALO = 220;
                SUN_INTENSITY = 35;
                SUN_WARMTH = 20;
                SKY_HAZE = 220;
                CLOUDINESS = 220;
            }
            _ => { // Alien
                ZENITH_COLOR = 0xFF00FFFF;
                SKY_HORIZON = 0x00FF00FF;
                GROUND_HORIZON = 0x00FFFFFF;
                NADIR_COLOR = 0x0000FFFF;

                SUN_AZIMUTH_DEG = 210.0;
                SUN_ELEVATION_DEG = 55.0;
                SUN_DISK = 80;
                SUN_HALO = 255;
                SUN_INTENSITY = 160;
                SUN_WARMTH = 0;
                SKY_HAZE = 80;
                CLOUDINESS = 200;
            }
        }
        SHIFT = 0.0;
    }
}

#[no_mangle]
pub extern "C" fn on_debug_change() {
    unsafe {
        PRESET_INDEX = PRESET_INDEX.clamp(0, PRESET_COUNT as i32 - 1);
        SHIFT = SHIFT.clamp(-1.0, 1.0);

        SUN_AZIMUTH_DEG = SUN_AZIMUTH_DEG % 360.0;
        SUN_ELEVATION_DEG = SUN_ELEVATION_DEG.clamp(-89.0, 89.0);
    }
}

#[no_mangle]
pub extern "C" fn init() {
    unsafe {
        set_clear_color(0x000000FF);
        SPHERE_MESH = sphere(1.0, 32, 24);
        CUBE_MESH = cube(1.4, 1.4, 1.4);
        TORUS_MESH = torus(1.0, 0.4, 32, 16);
        load_preset(0);

        debug_group_begin(b"gradient".as_ptr(), 8);
        debug_register_color(b"zenith".as_ptr(), 6, &ZENITH_COLOR as *const u32 as *const u8);
        debug_register_color(b"sky_horiz".as_ptr(), 9, &SKY_HORIZON as *const u32 as *const u8);
        debug_register_color(b"gnd_horiz".as_ptr(), 9, &GROUND_HORIZON as *const u32 as *const u8);
        debug_register_color(b"nadir".as_ptr(), 5, &NADIR_COLOR as *const u32 as *const u8);
        debug_register_f32(b"shift".as_ptr(), 5, &SHIFT as *const f32 as *const u8);
        debug_group_end();

        debug_group_begin(b"sun".as_ptr(), 3);
        debug_register_f32(b"az_deg".as_ptr(), 6, &SUN_AZIMUTH_DEG as *const f32 as *const u8);
        debug_register_f32(b"el_deg".as_ptr(), 6, &SUN_ELEVATION_DEG as *const f32 as *const u8);
        debug_register_u8(b"disk".as_ptr(), 4, &SUN_DISK);
        debug_register_u8(b"halo".as_ptr(), 4, &SUN_HALO);
        debug_register_u8(b"intensity".as_ptr(), 9, &SUN_INTENSITY);
        debug_register_u8(b"warmth".as_ptr(), 6, &SUN_WARMTH);
        debug_group_end();

        debug_group_begin(b"atmos".as_ptr(), 5);
        debug_register_u8(b"haze".as_ptr(), 4, &SKY_HAZE);
        debug_register_u8(b"bands".as_ptr(), 5, &CLOUDINESS);
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
            SUN_AZIMUTH_DEG = (SUN_AZIMUTH_DEG + 0.2) % 360.0;
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
        let deg2rad = 0.0174532925;
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

        let sun_azimuth = SUN_AZIMUTH_DEG * deg2rad;
        let sun_elevation = SUN_ELEVATION_DEG * deg2rad;

        env_gradient(
            0, // base layer
            ZENITH_COLOR,
            SKY_HORIZON,
            GROUND_HORIZON,
            NADIR_COLOR,
            sun_azimuth,
            SHIFT,
            sun_elevation,
            SUN_DISK as u32,
            SUN_HALO as u32,
            SUN_INTENSITY as u32,
            SKY_HAZE as u32,
            SUN_WARMTH as u32,
            CLOUDINESS as u32,
        );

        draw_env();

        push_identity();
        set_color(0x444455FF);
        material_metallic(0.8);
        material_roughness(0.2);
        let mesh = match SHAPE_INDEX {
            0 => SPHERE_MESH,
            1 => CUBE_MESH,
            _ => TORUS_MESH,
        };
        draw_mesh(mesh);

        let title = b"Env Mode 0: Featured Sky";
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
