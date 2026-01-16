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

// Debug values (Mode 0: Gradient) — matches the design sheet.
static mut ZENITH_COLOR: u32 = 0x2E65FFFF;
static mut SKY_HORIZON: u32 = 0xA9D8FFFF;
static mut GROUND_HORIZON: u32 = 0x4D8B4DFF;
static mut NADIR_COLOR: u32 = 0x102010FF;

// Orientation + horizon
static mut ROTATION: f32 = 0.35;      // radians
static mut SHIFT: f32 = 0.0;          // -1..1
static mut SUN_ELEVATION: f32 = 0.95; // radians

static mut PRESET_INDEX: i32 = 0;

// Featured sky controls
static mut SUN_DISK: u8 = 10;       // 0-255
static mut SUN_HALO: u8 = 72;       // 0-255
static mut SUN_INTENSITY: u8 = 230; // 0-255 (0 disables sun)
static mut HORIZON_HAZE: u8 = 32;   // 0-255
static mut SUN_WARMTH: u8 = 24;     // 0-255
static mut CLOUDINESS: u8 = 40;     // 0-255 (bands)
static mut CLOUD_PHASE: u32 = 0;    // 0..65535, wraps

// Animation controls (not packed directly; used to drive rotation + cloud_phase).
static mut ANIMATE_ROTATION: u8 = 1;
static mut ROTATION_RATE: f32 = 0.026_179_938; // rad/sec (≈ 240s loop)
static mut ANIMATE_CLOUDS: u8 = 1;
static mut CLOUD_PHASE_RATE: u32 = 728; // phase units/sec (≈ 90s loop)

static mut SPHERE_MESH: u32 = 0;
static mut CUBE_MESH: u32 = 0;
static mut TORUS_MESH: u32 = 0;
static mut SHAPE_INDEX: i32 = 0;
static mut CAM_ANGLE: f32 = 0.0;
static mut CAM_ELEVATION: f32 = 0.0;

const SHAPE_COUNT: usize = 3;
const SHAPE_NAMES: [&str; SHAPE_COUNT] = ["Sphere", "Cube", "Torus"];

const PRESET_COUNT: usize = 4;
const PRESET_NAMES: [&str; PRESET_COUNT] = [
    "Clear Day",
    "Sunset Haze",
    "Fog Bank Morning",
    "Deep Ocean Bands",
];

fn load_preset(index: usize) {
    unsafe {
        match index {
            0 => { // Clear Day
                ZENITH_COLOR = 0x2E65FFFF;
                SKY_HORIZON = 0xA9D8FFFF;
                GROUND_HORIZON = 0x4D8B4DFF;
                NADIR_COLOR = 0x102010FF;

                ROTATION = 0.35;
                SHIFT = 0.00;
                SUN_ELEVATION = 0.95;
                SUN_DISK = 10;
                SUN_HALO = 72;
                SUN_INTENSITY = 230;
                HORIZON_HAZE = 32;
                SUN_WARMTH = 24;
                CLOUDINESS = 40;
                CLOUD_PHASE = 0;

                ANIMATE_ROTATION = 1;
                ROTATION_RATE = 0.026_179_938; // 240s loop
                ANIMATE_CLOUDS = 1;
                CLOUD_PHASE_RATE = 728; // 90s loop
            }
            1 => { // Sunset Haze
                ZENITH_COLOR = 0x2D2B5DFF;
                SKY_HORIZON = 0xFF8A3DFF;
                GROUND_HORIZON = 0x3A1B14FF;
                NADIR_COLOR = 0x080408FF;

                ROTATION = 1.70;
                SHIFT = 0.02;
                SUN_ELEVATION = 0.18;
                SUN_DISK = 14;
                SUN_HALO = 150;
                SUN_INTENSITY = 255;
                HORIZON_HAZE = 200;
                SUN_WARMTH = 220;
                CLOUDINESS = 90;
                CLOUD_PHASE = 0;

                ANIMATE_ROTATION = 1;
                ROTATION_RATE = 0.013_089_969; // 480s loop
                ANIMATE_CLOUDS = 1;
                CLOUD_PHASE_RATE = 546; // 120s loop
            }
            2 => { // Fog Bank Morning
                ZENITH_COLOR = 0x8899A8FF;
                SKY_HORIZON = 0xD7DEE2FF;
                GROUND_HORIZON = 0x9A9F9CFF;
                NADIR_COLOR = 0x1A1C1DFF;

                ROTATION = 0.80;
                SHIFT = 0.08;
                SUN_ELEVATION = 0.30;
                SUN_DISK = 6;
                SUN_HALO = 200;
                SUN_INTENSITY = 120;
                HORIZON_HAZE = 255;
                SUN_WARMTH = 80;
                CLOUDINESS = 20;
                CLOUD_PHASE = 0;

                ANIMATE_ROTATION = 1;
                ROTATION_RATE = 0.010_471_975; // 600s loop
                ANIMATE_CLOUDS = 1;
                CLOUD_PHASE_RATE = 364; // 180s loop
            }
            _ => { // Deep Ocean Bands
                ZENITH_COLOR = 0x1C79A3FF;
                SKY_HORIZON = 0x0A3556FF;
                GROUND_HORIZON = 0x041C30FF;
                NADIR_COLOR = 0x01060BFF;

                ROTATION = 0.00;
                SHIFT = -0.25;
                SUN_ELEVATION = 1.30;
                SUN_DISK = 0;
                SUN_HALO = 0;
                SUN_INTENSITY = 0;
                HORIZON_HAZE = 60;
                SUN_WARMTH = 0;
                CLOUDINESS = 180;
                CLOUD_PHASE = 0;

                ANIMATE_ROTATION = 0;
                ROTATION_RATE = 0.0;
                ANIMATE_CLOUDS = 1;
                CLOUD_PHASE_RATE = 2184; // 30s loop
            }
        }
    }
}

#[no_mangle]
pub extern "C" fn on_debug_change() {
    unsafe {
        PRESET_INDEX = PRESET_INDEX.clamp(0, PRESET_COUNT as i32 - 1);
        SHIFT = SHIFT.clamp(-1.0, 1.0);
        SUN_ELEVATION = SUN_ELEVATION.clamp(0.0, 1.570_796_4);

        // Wrap rotation to [0, 2π) without using std.
        const TAU: f32 = 6.283_185_5;
        ROTATION = ROTATION % TAU;
        if ROTATION < 0.0 {
            ROTATION += TAU;
        }
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
        debug_register_f32(b"rotation".as_ptr(), 8, &ROTATION as *const f32 as *const u8);
        debug_register_f32(b"shift".as_ptr(), 5, &SHIFT as *const f32 as *const u8);
        debug_group_end();

        debug_group_begin(b"sun".as_ptr(), 3);
        debug_register_f32(b"elevation".as_ptr(), 9, &SUN_ELEVATION as *const f32 as *const u8);
        debug_register_u8(b"disk".as_ptr(), 4, &SUN_DISK);
        debug_register_u8(b"halo".as_ptr(), 4, &SUN_HALO);
        debug_register_u8(b"intensity".as_ptr(), 9, &SUN_INTENSITY);
        debug_register_u8(b"warmth".as_ptr(), 6, &SUN_WARMTH);
        debug_register_u8(b"haze".as_ptr(), 4, &HORIZON_HAZE);
        debug_group_end();

        debug_group_begin(b"clouds".as_ptr(), 6);
        debug_register_u8(b"bands".as_ptr(), 5, &CLOUDINESS);
        debug_register_u32(b"phase".as_ptr(), 5, &CLOUD_PHASE as *const u32 as *const u8);
        debug_group_end();

        debug_group_begin(b"animation".as_ptr(), 9);
        debug_register_u8(b"rot_on".as_ptr(), 6, &ANIMATE_ROTATION);
        debug_register_f32(b"rot_rate".as_ptr(), 8, &ROTATION_RATE as *const f32 as *const u8);
        debug_register_u8(b"cloud_on".as_ptr(), 8, &ANIMATE_CLOUDS);
        debug_register_u32(
            b"cloud_rate".as_ptr(),
            10,
            &CLOUD_PHASE_RATE as *const u32 as *const u8,
        );
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

        let dt = delta_time();
        if ANIMATE_ROTATION != 0 {
            const TAU: f32 = 6.283_185_5;
            ROTATION += ROTATION_RATE * dt;
            ROTATION = ROTATION % TAU;
            if ROTATION < 0.0 {
                ROTATION += TAU;
            }
        }
        if ANIMATE_CLOUDS != 0 {
            CLOUD_PHASE = CLOUD_PHASE.wrapping_add((CLOUD_PHASE_RATE as f32 * dt) as u32);
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

        env_gradient(
            0, // base layer
            ZENITH_COLOR,
            SKY_HORIZON,
            GROUND_HORIZON,
            NADIR_COLOR,
            ROTATION,
            SHIFT,
            SUN_ELEVATION,
            SUN_DISK as u32,
            SUN_HALO as u32,
            SUN_INTENSITY as u32,
            HORIZON_HAZE as u32,
            SUN_WARMTH as u32,
            CLOUDINESS as u32,
            CLOUD_PHASE,
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

        let title = b"Env Mode 0: Gradient";
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
