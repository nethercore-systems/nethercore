//! NEON DRIFT - ZX Console Showcase
//!
//! A complete arcade racing game demonstrating:
//! - Render Mode 2 (Metallic-Roughness PBR)
//! - 1-4 player split-screen racing
//! - Boost & drift mechanics
//! - AI opponents with rubber-banding
//! - Procedural EPU environments (5 tracks)
//! - Rollback netcode multiplayer
//!
//! Controls:
//! - RT: Accelerate (analog 0.0-1.0)
//! - LT: Brake (analog 0.0-1.0)
//! - Left Stick: Steering
//! - A Button: Boost (when meter >= 50%)
//! - B Button: Brake/Drift
//! - X Button: Look back
//! - Start: Pause

#![no_std]
#![no_main]

use core::panic::PanicInfo;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

// Game modules
mod ffi;
mod types;
mod state;
mod particles;
mod physics;
mod menus;
mod racing;
mod rendering;
mod hud;

use ffi::*;

use types::*;
use state::*;
use rendering::*;
use menus::*;
use racing::*;

// === Initialization ===

#[no_mangle]
pub extern "C" fn init() {
    unsafe {
        // Configure renderer
        set_clear_color(0x000000FF);
        render_mode(2); // Mode 2: Metallic-Roughness PBR
        depth_test(1);
        set_tick_rate(2); // 60 FPS

        // Load car meshes from ROM
        MESH_SPEEDSTER = load_rom_mesh(b"speedster");
        MESH_MUSCLE = load_rom_mesh(b"muscle");
        MESH_RACER = load_rom_mesh(b"racer");
        MESH_DRIFT = load_rom_mesh(b"drift");
        MESH_PHANTOM = load_rom_mesh(b"phantom");
        MESH_TITAN = load_rom_mesh(b"titan");
        MESH_VIPER = load_rom_mesh(b"viper");

        // Load track meshes from ROM
        MESH_TRACK_STRAIGHT = load_rom_mesh(b"track_straight");
        MESH_TRACK_CURVE_LEFT = load_rom_mesh(b"track_curve_left");
        MESH_TRACK_TUNNEL = load_rom_mesh(b"track_tunnel");
        MESH_TRACK_JUMP = load_rom_mesh(b"track_jump");

        // Load prop meshes from ROM
        MESH_PROP_BARRIER = load_rom_mesh(b"prop_barrier");
        MESH_PROP_BOOST_PAD = load_rom_mesh(b"prop_boost_pad");
        MESH_PROP_BILLBOARD = load_rom_mesh(b"prop_billboard");
        MESH_PROP_BUILDING = load_rom_mesh(b"prop_building");

        // Load car textures (albedo)
        TEX_SPEEDSTER = load_rom_texture(b"speedster");
        TEX_MUSCLE = load_rom_texture(b"muscle");
        TEX_RACER = load_rom_texture(b"racer");
        TEX_DRIFT = load_rom_texture(b"drift");
        TEX_PHANTOM = load_rom_texture(b"phantom");
        TEX_TITAN = load_rom_texture(b"titan");
        TEX_VIPER = load_rom_texture(b"viper");

        // Load car textures (emissive)
        TEX_SPEEDSTER_EMISSIVE = load_rom_texture(b"speedster_emissive");
        TEX_MUSCLE_EMISSIVE = load_rom_texture(b"muscle_emissive");
        TEX_RACER_EMISSIVE = load_rom_texture(b"racer_emissive");
        TEX_DRIFT_EMISSIVE = load_rom_texture(b"drift_emissive");
        TEX_PHANTOM_EMISSIVE = load_rom_texture(b"phantom_emissive");
        TEX_TITAN_EMISSIVE = load_rom_texture(b"titan_emissive");
        TEX_VIPER_EMISSIVE = load_rom_texture(b"viper_emissive");

        // Load track textures
        TEX_TRACK_STRAIGHT = load_rom_texture(b"track_straight");
        TEX_TRACK_CURVE = load_rom_texture(b"track_curve_left");

        // Load prop textures
        TEX_PROP_BUILDING = load_rom_texture(b"prop_building");
        TEX_PROP_BARRIER = load_rom_texture(b"prop_barrier");
        TEX_PROP_BILLBOARD = load_rom_texture(b"prop_billboard");
        TEX_PROP_BOOST_PAD = load_rom_texture(b"prop_boost_pad");

        // Load sounds
        SND_BOOST = load_rom_sound(b"boost");
        SND_DRIFT = load_rom_sound(b"drift");
        SND_WALL = load_rom_sound(b"wall");
        SND_CHECKPOINT = load_rom_sound(b"checkpoint");
        SND_FINISH = load_rom_sound(b"finish");

        // Load music tracks (XM tracker format with FM synthwave sounds)
        MUSIC_SUNSET_STRIP = rom_tracker_str("sunset_strip");
        MUSIC_NEON_CITY = rom_tracker_str("neon_city");
        MUSIC_VOID_TUNNEL = rom_tracker_str("void_tunnel");
        MUSIC_CRYSTAL_CAVERN = rom_tracker_str("crystal_cavern");
        MUSIC_SOLAR_HIGHWAY = rom_tracker_str("solar_highway");

        // Load custom font (16x16 cells, 95 characters starting from space (32))
        TEX_NEON_FONT = load_rom_texture(b"neon_font");
        NEON_FONT = load_font(TEX_NEON_FONT, 16, 16, 32, 95);

        // Initialize title animation
        TITLE_ANIM_TIME = 0.0;
        IDLE_TIMER = 0.0;

        // Initialize game state
        GAME_MODE = GameMode::MainMenu;
        ACTIVE_PLAYER_COUNT = 1;

        // Initialize cars
        for i in 0..4 {
            CARS[i] = Car::new();
            CARS[i].car_type = CarType::Speedster;
            CARS[i].init_stats();
        }
    }
}

// === Update ===

#[no_mangle]
pub extern "C" fn update() {
    unsafe {
        let dt = delta_time();
        ELAPSED_TIME += dt;

        // Update animation phases
        GRID_PHASE = GRID_PHASE.wrapping_add((dt * 3.0 * 65535.0) as u32);
        RING_PHASE = RING_PHASE.wrapping_add((dt * 4.0 * 65535.0) as u32);
        SPEED_PHASE = SPEED_PHASE.wrapping_add((dt * 8.0 * 65535.0) as u32);
        WINDOW_PHASE = WINDOW_PHASE.wrapping_add((dt * 0.5 * 65535.0) as u32);

        // Update title animation time
        TITLE_ANIM_TIME += dt;

        match GAME_MODE {
            GameMode::MainMenu => update_main_menu(dt),
            GameMode::CarSelect => update_car_select(),
            GameMode::TrackSelect => update_track_select(),
            GameMode::CountdownReady => update_countdown(dt),
            GameMode::Racing => update_racing(dt),
            GameMode::RaceFinished => update_results(),
            GameMode::Paused => update_paused(),
            GameMode::AttractMode => update_attract_mode(dt),
        }
    }
}

// === Render ===

#[no_mangle]
pub extern "C" fn render() {
    unsafe {
        match GAME_MODE {
            GameMode::MainMenu => render_main_menu(),
            GameMode::CarSelect => render_car_select(),
            GameMode::TrackSelect => render_track_select(),
            GameMode::CountdownReady => render_countdown(),
            GameMode::Racing => render_racing(),
            GameMode::RaceFinished => render_results(),
            GameMode::Paused => render_paused(),
            GameMode::AttractMode => render_attract_mode(),
        }
    }
}
