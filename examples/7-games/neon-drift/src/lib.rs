//! NEON DRIFT - ZX Console Showcase
//!
//! A complete arcade racing game demonstrating:
//! - Render Mode 2 (Metallic-Roughness PBR)
//! - 1-4 player split-screen racing
//! - Boost & drift mechanics
//! - AI opponents with rubber-banding
//! - Procedural EPU environments (3 tracks)
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
    core::arch::wasm32::unreachable()
}

// === FFI Imports ===

#[link(wasm_import_module = "env")]
extern "C" {
    // Configuration
    fn set_clear_color(color: u32);
    fn render_mode(mode: u32);
    fn depth_test(enabled: u32);
    fn set_tick_rate(rate: u32);

    // Camera
    fn camera_set(x: f32, y: f32, z: f32, target_x: f32, target_y: f32, target_z: f32);
    fn camera_fov(fov_degrees: f32);

    // Viewport (split-screen)
    fn viewport(x: u32, y: u32, width: u32, height: u32);
    fn viewport_clear();

    // Input
    fn left_stick_x(player: u32) -> f32;
    fn left_stick_y(player: u32) -> f32;
    fn right_stick_x(player: u32) -> f32;
    fn right_stick_y(player: u32) -> f32;
    fn trigger_left(player: u32) -> f32;
    fn trigger_right(player: u32) -> f32;
    fn button_pressed(player: u32, button: u32) -> u32;
    fn button_held(player: u32, button: u32) -> u32;
    fn player_count() -> u32;

    // ROM Assets
    fn rom_mesh(id_ptr: *const u8, id_len: u32) -> u32;
    fn rom_texture(id_ptr: *const u8, id_len: u32) -> u32;
    fn rom_sound(id_ptr: *const u8, id_len: u32) -> u32;

    // Mesh rendering
    fn draw_mesh(handle: u32);

    // Transform
    fn push_identity();
    fn push_translate(x: f32, y: f32, z: f32);
    fn push_rotate_y(angle_deg: f32);
    fn push_rotate_x(angle_deg: f32);
    fn push_rotate_z(angle_deg: f32);
    fn push_scale(x: f32, y: f32, z: f32);

    // Materials (Mode 2: Metallic-Roughness PBR)
    fn material_albedo(texture_handle: u32);
    fn material_mre(texture_handle: u32);
    fn material_emissive_texture(texture_handle: u32);
    fn material_metallic(value: f32);
    fn material_roughness(value: f32);
    fn material_emissive(intensity: f32);

    // Lighting
    fn light_set(index: u32, dir_x: f32, dir_y: f32, dir_z: f32);
    fn light_set_point(index: u32, pos_x: f32, pos_y: f32, pos_z: f32);
    fn light_color(index: u32, color: u32);
    fn light_intensity(index: u32, value: f32);
    fn light_range(index: u32, range: f32);
    fn light_enable(index: u32);
    fn light_disable(index: u32);

    // Environment (EPU)
    fn env_gradient(layer: u32, zenith: u32, sky_horizon: u32, ground_horizon: u32, nadir: u32, rotation: f32, shift: f32);
    fn env_lines(layer: u32, variant: u32, line_type: u32, thickness: u32, spacing: f32, fade: f32, color_primary: u32, color_accent: u32, accent_every: u32, phase: u32);
    fn env_rectangles(layer: u32, variant: u32, density: u32, lit_ratio: u32, size_min: u32, size_max: u32, aspect: u32, color_primary: u32, color_variation: u32, parallax_rate: u32, phase: u32);
    fn env_rings(layer: u32, ring_count: u32, thickness: u32, color_a: u32, color_b: u32, center_color: u32, center_falloff: u32, spiral_twist: f32, axis_x: f32, axis_y: f32, axis_z: f32, phase: u32);
    fn env_scatter(layer: u32, variant: u32, density: u32, size: u32, glow: u32, streak_length: u32, color_primary: u32, color_secondary: u32, parallax_rate: u32, parallax_size: u32, phase: u32);
    fn env_blend(mode: u32);
    fn draw_env();

    // Transparency
    fn uniform_alpha(level: u32);

    // 2D Drawing
    fn draw_rect(x: f32, y: f32, w: f32, h: f32, color: u32);
    fn draw_sprite(x: f32, y: f32, w: f32, h: f32, color: u32);
    fn draw_text(ptr: *const u8, len: u32, x: f32, y: f32, size: f32, color: u32);
    fn texture_bind(handle: u32);
    fn set_color(color: u32);

    // Billboard rendering (for particles)
    fn draw_billboard(width: f32, height: f32, mode: u32, color: u32);

    // Audio
    fn play_sound(sound: u32, volume: f32, pan: f32);
    fn channel_play(channel: u32, sound: u32, volume: f32, pan: f32);
    fn channel_stop(channel: u32);

    // System
    fn random() -> u32;
    fn delta_time() -> f32;
    fn tick_count() -> u64;
}

// === Constants ===

// Screen dimensions
const SCREEN_WIDTH: u32 = 960;
const SCREEN_HEIGHT: u32 = 540;

// Button constants
const BUTTON_A: u32 = 4;
const BUTTON_B: u32 = 5;
const BUTTON_X: u32 = 6;
const BUTTON_Y: u32 = 7;
const BUTTON_START: u32 = 11;
const BUTTON_SELECT: u32 = 10;

// Colors
const COLOR_WHITE: u32 = 0xFFFFFFFF;
const COLOR_CYAN: u32 = 0x00FFFFFF;
const COLOR_MAGENTA: u32 = 0xFF00FFFF;
const COLOR_ORANGE: u32 = 0xFF6600FF;
const COLOR_PURPLE: u32 = 0x9933FFFF;

// Physics constants
const DRIFT_THRESHOLD: f32 = 0.3;
const BOOST_COST: f32 = 0.5;
const BOOST_MULTIPLIER: f32 = 1.5;
const BOOST_DURATION: u32 = 120; // 2 seconds at 60fps

// === Game State ===

#[derive(Clone, Copy, PartialEq)]
#[repr(u8)]
enum GameMode {
    MainMenu,
    CarSelect,
    TrackSelect,
    CountdownReady,
    Racing,
    RaceFinished,
    Paused,
}

#[derive(Clone, Copy, PartialEq)]
#[repr(u8)]
enum CarType {
    Speedster,
    Muscle,
    Racer,
    Drift,
}

#[derive(Clone, Copy, PartialEq)]
#[repr(u8)]
enum TrackId {
    SunsetStrip,
    NeonCity,
    VoidTunnel,
}

#[derive(Clone, Copy)]
struct Car {
    // Position & Orientation
    x: f32,
    y: f32,
    z: f32,
    rotation_y: f32, // Yaw angle in radians

    // Velocity
    velocity_forward: f32,
    velocity_lateral: f32,
    angular_velocity: f32,

    // Boost & Drift
    boost_meter: f32,        // 0.0-1.0
    is_boosting: bool,
    boost_timer: u32,
    is_drifting: bool,

    // Car type and stats
    car_type: CarType,
    max_speed: f32,
    acceleration: f32,
    handling: f32,
    drift_factor: f32,

    // Race state
    current_lap: u32,
    last_checkpoint: usize,
    race_position: u32,

    // Collision
    collision_pushback_x: f32,
    collision_pushback_z: f32,
}

#[derive(Clone, Copy)]
struct Camera {
    current_pos_x: f32,
    current_pos_y: f32,
    current_pos_z: f32,
    current_target_x: f32,
    current_target_y: f32,
    current_target_z: f32,
    is_looking_back: bool,
}

// === Static Game State (Rollback-safe) ===

static mut GAME_MODE: GameMode = GameMode::MainMenu;
static mut SELECTED_TRACK: TrackId = TrackId::SunsetStrip;
static mut CARS: [Car; 4] = [Car::new(); 4];
static mut CAMERAS: [Camera; 4] = [Camera::new(); 4];
static mut ACTIVE_PLAYER_COUNT: u32 = 1;

// Animation state
static mut GRID_PHASE: u32 = 0;
static mut RING_PHASE: u32 = 0;
static mut SPEED_PHASE: u32 = 0;
static mut WINDOW_PHASE: u32 = 0;
static mut ELAPSED_TIME: f32 = 0.0;

// === Asset Handles ===

// Car meshes
static mut MESH_SPEEDSTER: u32 = 0;
static mut MESH_MUSCLE: u32 = 0;
static mut MESH_RACER: u32 = 0;
static mut MESH_DRIFT: u32 = 0;

// Track segment meshes
static mut MESH_TRACK_STRAIGHT: u32 = 0;
static mut MESH_TRACK_CURVE_LEFT: u32 = 0;
static mut MESH_TRACK_TUNNEL: u32 = 0;
static mut MESH_TRACK_JUMP: u32 = 0;

// Prop meshes
static mut MESH_PROP_BARRIER: u32 = 0;
static mut MESH_PROP_BOOST_PAD: u32 = 0;
static mut MESH_PROP_BILLBOARD: u32 = 0;
static mut MESH_PROP_BUILDING: u32 = 0;

// Textures (albedo)
static mut TEX_SPEEDSTER: u32 = 0;
static mut TEX_MUSCLE: u32 = 0;
static mut TEX_RACER: u32 = 0;
static mut TEX_DRIFT: u32 = 0;

// Textures (emissive)
static mut TEX_SPEEDSTER_EMISSIVE: u32 = 0;
static mut TEX_MUSCLE_EMISSIVE: u32 = 0;
static mut TEX_RACER_EMISSIVE: u32 = 0;
static mut TEX_DRIFT_EMISSIVE: u32 = 0;

// Sounds
static mut SND_BOOST: u32 = 0;
static mut SND_DRIFT: u32 = 0;
static mut SND_WALL: u32 = 0;
static mut SND_CHECKPOINT: u32 = 0;
static mut SND_FINISH: u32 = 0;

// === Helper Functions ===

impl Car {
    const fn new() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            z: 0.0,
            rotation_y: 0.0,
            velocity_forward: 0.0,
            velocity_lateral: 0.0,
            angular_velocity: 0.0,
            boost_meter: 0.0,
            is_boosting: false,
            boost_timer: 0,
            is_drifting: false,
            car_type: CarType::Speedster,
            max_speed: 25.0,
            acceleration: 15.0,
            handling: 0.9,
            drift_factor: 0.9,
            current_lap: 0,
            last_checkpoint: 0,
            race_position: 1,
            collision_pushback_x: 0.0,
            collision_pushback_z: 0.0,
        }
    }

    fn init_stats(&mut self) {
        match self.car_type {
            CarType::Speedster => {
                self.max_speed = 28.5;   // 95% normalized
                self.acceleration = 14.0; // 90%
                self.handling = 1.0;      // 100%
                self.drift_factor = 0.85;
            }
            CarType::Muscle => {
                self.max_speed = 33.0;    // 110%
                self.acceleration = 12.5; // 80%
                self.handling = 0.85;     // 85%
                self.drift_factor = 0.8;
            }
            CarType::Racer => {
                self.max_speed = 28.5;    // 95%
                self.acceleration = 17.0; // 110%
                self.handling = 0.95;     // 95%
                self.drift_factor = 0.9;
            }
            CarType::Drift => {
                self.max_speed = 27.0;    // 90%
                self.acceleration = 15.5; // 100%
                self.handling = 1.2;      // 120%
                self.drift_factor = 1.0;
            }
        }
    }
}

impl Camera {
    const fn new() -> Self {
        Self {
            current_pos_x: 0.0,
            current_pos_y: 5.0,
            current_pos_z: -10.0,
            current_target_x: 0.0,
            current_target_y: 0.0,
            current_target_z: 0.0,
            is_looking_back: false,
        }
    }
}

fn load_rom_mesh(id: &[u8]) -> u32 {
    unsafe { rom_mesh(id.as_ptr(), id.len() as u32) }
}

fn load_rom_texture(id: &[u8]) -> u32 {
    unsafe { rom_texture(id.as_ptr(), id.len() as u32) }
}

fn load_rom_sound(id: &[u8]) -> u32 {
    unsafe { rom_sound(id.as_ptr(), id.len() as u32) }
}

// === Initialization ===

#[no_mangle]
pub extern "C" fn init() {
    unsafe {
        // Configure renderer
        set_clear_color(0x000000FF);
        render_mode(2); // Mode 2: Metallic-Roughness PBR
        depth_test(1);
        set_tick_rate(2); // 60 FPS

        // Load car meshes
        MESH_SPEEDSTER = load_rom_mesh(b"speedster");
        MESH_MUSCLE = load_rom_mesh(b"muscle");
        MESH_RACER = load_rom_mesh(b"racer");
        MESH_DRIFT = load_rom_mesh(b"drift");

        // Load track segment meshes
        MESH_TRACK_STRAIGHT = load_rom_mesh(b"track_straight");
        MESH_TRACK_CURVE_LEFT = load_rom_mesh(b"track_curve_left");
        MESH_TRACK_TUNNEL = load_rom_mesh(b"track_tunnel");
        MESH_TRACK_JUMP = load_rom_mesh(b"track_jump");

        // Load prop meshes
        MESH_PROP_BARRIER = load_rom_mesh(b"prop_barrier");
        MESH_PROP_BOOST_PAD = load_rom_mesh(b"prop_boost_pad");
        MESH_PROP_BILLBOARD = load_rom_mesh(b"prop_billboard");
        MESH_PROP_BUILDING = load_rom_mesh(b"prop_building");

        // Load car textures (albedo)
        TEX_SPEEDSTER = load_rom_texture(b"speedster");
        TEX_MUSCLE = load_rom_texture(b"muscle");
        TEX_RACER = load_rom_texture(b"racer");
        TEX_DRIFT = load_rom_texture(b"drift");

        // Load car textures (emissive)
        TEX_SPEEDSTER_EMISSIVE = load_rom_texture(b"speedster_emissive");
        TEX_MUSCLE_EMISSIVE = load_rom_texture(b"muscle_emissive");
        TEX_RACER_EMISSIVE = load_rom_texture(b"racer_emissive");
        TEX_DRIFT_EMISSIVE = load_rom_texture(b"drift_emissive");

        // Load sounds
        SND_BOOST = load_rom_sound(b"boost");
        SND_DRIFT = load_rom_sound(b"drift");
        SND_WALL = load_rom_sound(b"wall");
        SND_CHECKPOINT = load_rom_sound(b"checkpoint");
        SND_FINISH = load_rom_sound(b"finish");

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

        match GAME_MODE {
            GameMode::MainMenu => {
                // TODO: Menu input handling
                if button_pressed(0, BUTTON_A) != 0 {
                    GAME_MODE = GameMode::Racing; // Temporary: skip to racing
                    init_race();
                }
            }
            GameMode::Racing => {
                update_racing(dt);
            }
            _ => {
                // TODO: Other game modes
            }
        }
    }
}

fn init_race() {
    unsafe {
        // Initialize car positions
        for i in 0..4 {
            CARS[i].x = (i as f32) * 4.0 - 6.0; // Spread across start line
            CARS[i].y = 0.0;
            CARS[i].z = 0.0;
            CARS[i].rotation_y = 0.0;
            CARS[i].velocity_forward = 0.0;
            CARS[i].velocity_lateral = 0.0;
            CARS[i].boost_meter = 1.0; // Start with full boost for testing
            CARS[i].current_lap = 1;
        }
    }
}

fn update_racing(dt: f32) {
    unsafe {
        let active_count = player_count();
        ACTIVE_PLAYER_COUNT = active_count;

        // Update all active cars
        for i in 0..active_count as usize {
            update_car_physics(&mut CARS[i], i as u32, dt);
            update_camera(&mut CAMERAS[i], &CARS[i], dt);
        }
    }
}

fn update_car_physics(car: &mut Car, player_idx: u32, dt: f32) {
    unsafe {
        // Read inputs
        let gas = trigger_right(player_idx);
        let brake = trigger_left(player_idx);
        let steer_x = left_stick_x(player_idx);
        let boost_pressed = button_pressed(player_idx, BUTTON_A);

        // Acceleration/braking
        let accel_input = gas - (brake * 0.7);

        if accel_input > 0.01 {
            car.velocity_forward += car.acceleration * accel_input * dt;
        } else if accel_input < -0.01 {
            car.velocity_forward += car.acceleration * accel_input * 2.0 * dt;
        } else {
            car.velocity_forward *= 0.98;
        }

        // Boost activation
        if boost_pressed != 0 && car.boost_meter >= BOOST_COST && !car.is_boosting {
            car.is_boosting = true;
            car.boost_timer = BOOST_DURATION;
            car.boost_meter -= BOOST_COST;
            play_sound(SND_BOOST, 1.0, 0.0);
        }

        // Update boost timer
        if car.boost_timer > 0 {
            car.boost_timer -= 1;
            if car.boost_timer == 0 {
                car.is_boosting = false;
            }
        }

        // Apply boost multiplier
        let boost_mult = if car.is_boosting { BOOST_MULTIPLIER } else { 1.0 };
        let max_speed = car.max_speed * boost_mult;
        car.velocity_forward = car.velocity_forward.clamp(-max_speed * 0.5, max_speed);

        // Drift detection and physics
        let speed_factor = (car.velocity_forward.abs() / car.max_speed).min(1.0);

        if brake > DRIFT_THRESHOLD && steer_x.abs() > DRIFT_THRESHOLD && speed_factor > 0.4 {
            if !car.is_drifting {
                car.is_drifting = true;
                play_sound(SND_DRIFT, 0.7, 0.0);
            }

            // Drift physics
            let drift_power = steer_x * car.drift_factor;
            car.velocity_lateral += drift_power * 15.0 * dt;
            car.angular_velocity = drift_power * 120.0;
            car.velocity_forward *= 0.97;

            // Fill boost meter while drifting
            car.boost_meter = (car.boost_meter + 0.015).min(1.0);
        } else {
            car.is_drifting = false;

            // Normal steering
            let turn_speed = car.handling * 90.0 * speed_factor;
            car.angular_velocity = steer_x * turn_speed;
            car.velocity_lateral *= 0.85;
        }

        // Update rotation
        car.rotation_y += car.angular_velocity * dt;

        // Update position (convert local velocity to world space)
        let sin_rot = libm::sinf(car.rotation_y * 3.14159 / 180.0);
        let cos_rot = libm::cosf(car.rotation_y * 3.14159 / 180.0);
        let forward_x = sin_rot;
        let forward_z = cos_rot;
        let right_x = cos_rot;
        let right_z = -sin_rot;

        car.x += (forward_x * car.velocity_forward + right_x * car.velocity_lateral) * dt;
        car.z += (forward_z * car.velocity_forward + right_z * car.velocity_lateral) * dt;

        // Apply collision pushback
        car.x += car.collision_pushback_x;
        car.z += car.collision_pushback_z;
        car.collision_pushback_x *= 0.5;
        car.collision_pushback_z *= 0.5;
    }
}

fn update_camera(camera: &mut Camera, car: &Car, dt: f32) {
    unsafe {
        let sin_rot = libm::sinf(car.rotation_y * 3.14159 / 180.0);
        let cos_rot = libm::cosf(car.rotation_y * 3.14159 / 180.0);

        // Camera offset (8 units behind, 3 units up)
        let offset_distance = 8.0;
        let offset_height = 3.0;

        let desired_pos_x = car.x - sin_rot * offset_distance;
        let desired_pos_y = car.y + offset_height;
        let desired_pos_z = car.z - cos_rot * offset_distance;

        let desired_target_x = car.x + sin_rot * 5.0;
        let desired_target_y = car.y + 1.0;
        let desired_target_z = car.z + cos_rot * 5.0;

        // Smooth interpolation
        let lerp = 0.1;
        camera.current_pos_x += (desired_pos_x - camera.current_pos_x) * lerp;
        camera.current_pos_y += (desired_pos_y - camera.current_pos_y) * lerp;
        camera.current_pos_z += (desired_pos_z - camera.current_pos_z) * lerp;

        camera.current_target_x += (desired_target_x - camera.current_target_x) * lerp;
        camera.current_target_y += (desired_target_y - camera.current_target_y) * lerp;
        camera.current_target_z += (desired_target_z - camera.current_target_z) * lerp;
    }
}

// === Render ===

#[no_mangle]
pub extern "C" fn render() {
    unsafe {
        match GAME_MODE {
            GameMode::MainMenu => {
                render_main_menu();
            }
            GameMode::Racing => {
                render_racing();
            }
            _ => {
                // TODO: Other game modes
            }
        }
    }
}

fn render_main_menu() {
    unsafe {
        // Simple title screen
        let title = b"NEON DRIFT";
        draw_text(title.as_ptr(), title.len() as u32,
                  SCREEN_WIDTH as f32 / 2.0 - 150.0, 100.0, 48.0, COLOR_CYAN);

        let prompt = b"Press A to Start";
        draw_text(prompt.as_ptr(), prompt.len() as u32,
                  SCREEN_WIDTH as f32 / 2.0 - 100.0, 300.0, 24.0, COLOR_WHITE);
    }
}

fn render_racing() {
    unsafe {
        let viewports = get_viewport_layout(ACTIVE_PLAYER_COUNT);

        for player_id in 0..ACTIVE_PLAYER_COUNT as usize {
            let (vp_x, vp_y, vp_w, vp_h) = viewports[player_id];

            // Set viewport
            viewport(vp_x, vp_y, vp_w, vp_h);

            // Setup camera
            let camera = &CAMERAS[player_id];
            camera_set(
                camera.current_pos_x, camera.current_pos_y, camera.current_pos_z,
                camera.current_target_x, camera.current_target_y, camera.current_target_z
            );
            camera_fov(75.0);

            // Setup environment
            setup_environment(SELECTED_TRACK);
            draw_env();

            // Render cars
            render_all_cars();

            // Render HUD
            render_hud(player_id as u32, vp_w, vp_h);
        }

        // Reset viewport for shared UI
        viewport_clear();
    }
}

fn get_viewport_layout(player_count: u32) -> [(u32, u32, u32, u32); 4] {
    match player_count {
        1 => [
            (0, 0, SCREEN_WIDTH, SCREEN_HEIGHT),
            (0, 0, 0, 0),
            (0, 0, 0, 0),
            (0, 0, 0, 0),
        ],
        2 => [
            (0, 0, SCREEN_WIDTH / 2, SCREEN_HEIGHT),
            (SCREEN_WIDTH / 2, 0, SCREEN_WIDTH / 2, SCREEN_HEIGHT),
            (0, 0, 0, 0),
            (0, 0, 0, 0),
        ],
        3 => [
            (0, 0, SCREEN_WIDTH / 2, SCREEN_HEIGHT / 2),
            (SCREEN_WIDTH / 2, 0, SCREEN_WIDTH / 2, SCREEN_HEIGHT / 2),
            (0, SCREEN_HEIGHT / 2, SCREEN_WIDTH, SCREEN_HEIGHT / 2),
            (0, 0, 0, 0),
        ],
        _ => [
            (0, 0, SCREEN_WIDTH / 2, SCREEN_HEIGHT / 2),
            (SCREEN_WIDTH / 2, 0, SCREEN_WIDTH / 2, SCREEN_HEIGHT / 2),
            (0, SCREEN_HEIGHT / 2, SCREEN_WIDTH / 2, SCREEN_HEIGHT / 2),
            (SCREEN_WIDTH / 2, SCREEN_HEIGHT / 2, SCREEN_WIDTH / 2, SCREEN_HEIGHT / 2),
        ],
    }
}

fn setup_environment(track: TrackId) {
    unsafe {
        match track {
            TrackId::SunsetStrip => {
                // Sunset gradient
                env_gradient(0, 0xFF6B35FF, 0xF72585FF, 0x7209B7FF, 0x1A0533FF, 0.0, 0.2);
                // Synthwave grid
                env_lines(1, 0, 2, 3, 2.5, 100.0, 0xFF00FFFF, 0x00FFFFFF, 5, GRID_PHASE);
                env_blend(0); // Alpha blend
            }
            TrackId::NeonCity => {
                // Dark sky
                env_gradient(0, 0x0D0221FF, 0x0D0221FF, 0x190A3DFF, 0x000000FF, 0.0, 0.0);
                // City buildings
                env_rectangles(1, 1, 200, 180, 10, 28, 3, 0xFF00FFFF, 0x00FFFFFF, 120, WINDOW_PHASE);
                // Grid floor
                env_lines(2, 0, 2, 2, 2.0, 80.0, 0x00FFFFFF, 0xFF00FFFF, 6, GRID_PHASE);
                env_blend(1); // Additive
            }
            TrackId::VoidTunnel => {
                // Black void
                env_gradient(0, 0x000000FF, 0x000000FF, 0x000000FF, 0x000000FF, 0.0, 0.0);
                // Tunnel rings
                env_rings(1, 50, 4, 0xFF00FFFF, 0x00FFFFFF, 0xFFFFFFFF, 220, 10.0, 0.0, 0.0, 1.0, RING_PHASE);
                // Speed lines
                env_scatter(2, 3, 150, 3, 128, 20, 0x00FFFFFF, 0xFF00FFFF, 200, 150, SPEED_PHASE);
                env_blend(1); // Additive
            }
        }
    }
}

fn render_all_cars() {
    unsafe {
        for i in 0..ACTIVE_PLAYER_COUNT as usize {
            let car = &CARS[i];

            // Select mesh and textures based on car type
            let (mesh, tex_albedo, tex_emissive) = match car.car_type {
                CarType::Speedster => (MESH_SPEEDSTER, TEX_SPEEDSTER, TEX_SPEEDSTER_EMISSIVE),
                CarType::Muscle => (MESH_MUSCLE, TEX_MUSCLE, TEX_MUSCLE_EMISSIVE),
                CarType::Racer => (MESH_RACER, TEX_RACER, TEX_RACER_EMISSIVE),
                CarType::Drift => (MESH_DRIFT, TEX_DRIFT, TEX_DRIFT_EMISSIVE),
            };

            // Setup materials
            material_albedo(tex_albedo);
            material_emissive_texture(tex_emissive);
            material_metallic(0.9);
            material_roughness(0.1);
            material_emissive(2.0);

            // Render car
            push_identity();
            push_translate(car.x, car.y, car.z);
            push_rotate_y(car.rotation_y);
            draw_mesh(mesh);
        }
    }
}

fn render_hud(player_id: u32, vp_width: u32, vp_height: u32) {
    unsafe {
        let car = &CARS[player_id as usize];

        // Speed (bottom-left)
        let speed = (car.velocity_forward.abs() * 10.0) as u32;
        let mut speed_str = [0u8; 16];
        let speed_len = format_number(speed, &mut speed_str);
        draw_text(speed_str.as_ptr(), speed_len, 10.0, vp_height as f32 - 40.0, 24.0, COLOR_WHITE);

        // Boost meter (bottom-left)
        let meter_w = 100.0;
        let meter_h = 10.0;
        let meter_x = 10.0;
        let meter_y = vp_height as f32 - 60.0;

        // Background
        draw_rect(meter_x, meter_y, meter_w, meter_h, 0x333333FF);

        // Fill
        let fill_w = meter_w * car.boost_meter;
        let boost_color = if car.boost_meter > 0.8 { COLOR_CYAN } else { 0x0088FFFF };
        draw_rect(meter_x, meter_y, fill_w, meter_h, boost_color);
    }
}

// Helper function to format numbers without std
fn format_number(mut num: u32, buf: &mut [u8]) -> u32 {
    if num == 0 {
        buf[0] = b'0';
        return 1;
    }

    let mut len = 0;
    let mut temp = [0u8; 16];
    let mut temp_len = 0;

    while num > 0 {
        temp[temp_len] = b'0' + (num % 10) as u8;
        temp_len += 1;
        num /= 10;
    }

    // Reverse
    for i in 0..temp_len {
        buf[len] = temp[temp_len - 1 - i];
        len += 1;
    }

    len as u32
}
