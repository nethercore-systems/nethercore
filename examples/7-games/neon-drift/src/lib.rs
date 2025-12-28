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
    fn input_buttons(player: u32) -> u32;  // Raw button state

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
    // material_emissive_texture removed - use material_emissive(f32) instead
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

    // Custom Fonts
    fn load_font(texture: u32, char_width: u32, char_height: u32, first_codepoint: u32, char_count: u32) -> u32;
    fn font_bind(font_handle: u32);

    // Text measurement
    fn text_width(ptr: *const u8, len: u32, size: f32) -> f32;

    // Line drawing (for title effects)
    fn draw_line(x1: f32, y1: f32, x2: f32, y2: f32, thickness: f32, color: u32);
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
    AttractMode,  // Auto-demo when idle
}

#[derive(Clone, Copy, PartialEq)]
#[repr(u8)]
enum CarType {
    Speedster,
    Muscle,
    Racer,
    Drift,
    Phantom,
    Titan,
    Viper,
}

#[derive(Clone, Copy, PartialEq)]
#[repr(u8)]
enum TrackId {
    SunsetStrip,
    NeonCity,
    VoidTunnel,
    CrystalCavern,
    SolarHighway,
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
    // Screen shake
    shake_intensity: f32,
    shake_decay: f32,
    shake_offset_x: f32,
    shake_offset_y: f32,
}

// === Particle System ===
const MAX_PARTICLES: usize = 64;

#[derive(Clone, Copy)]
struct Particle {
    active: bool,
    x: f32,
    y: f32,
    z: f32,
    vel_x: f32,
    vel_y: f32,
    vel_z: f32,
    life: f32,
    max_life: f32,
    size: f32,
    color: u32,
    particle_type: ParticleType,
}

#[derive(Clone, Copy, PartialEq)]
#[repr(u8)]
enum ParticleType {
    BoostFlame,
    DriftSmoke,
    Spark,
    SpeedLine,
}

impl Particle {
    const fn new() -> Self {
        Self {
            active: false,
            x: 0.0, y: 0.0, z: 0.0,
            vel_x: 0.0, vel_y: 0.0, vel_z: 0.0,
            life: 0.0, max_life: 1.0,
            size: 0.1,
            color: 0xFFFFFFFF,
            particle_type: ParticleType::Spark,
        }
    }
}

// === Static Game State (Rollback-safe) ===

static mut GAME_MODE: GameMode = GameMode::MainMenu;
static mut SELECTED_TRACK: TrackId = TrackId::SunsetStrip;
static mut CARS: [Car; 4] = [Car::new(); 4];
static mut CAMERAS: [Camera; 4] = [Camera::new(); 4];
static mut ACTIVE_PLAYER_COUNT: u32 = 1;

// Menu selection state
static mut MENU_SELECTION: u32 = 0;  // Current menu item
static mut MENU_TIME: f32 = 0.0;     // Animation timer for menu effects
static mut CAR_SELECTIONS: [u32; 4] = [0; 4];  // Car type per player (0-3)
static mut PLAYER_CONFIRMED: [bool; 4] = [false; 4];  // Player has locked in car

// Race state
static mut COUNTDOWN_TIMER: u32 = 0;  // Frames until race starts
static mut RACE_TIME: f32 = 0.0;  // Total race elapsed time
static mut RACE_FINISHED: bool = false;

// Track layout (simplified: just Z positions for checkpoints)
static mut TRACK_LENGTH: f32 = 200.0;  // Total track length
const NUM_CHECKPOINTS: usize = 4;
static mut CHECKPOINT_Z: [f32; NUM_CHECKPOINTS] = [0.0, 50.0, 100.0, 150.0];

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
static mut MESH_PHANTOM: u32 = 0;
static mut MESH_TITAN: u32 = 0;
static mut MESH_VIPER: u32 = 0;

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
static mut TEX_PHANTOM: u32 = 0;
static mut TEX_TITAN: u32 = 0;
static mut TEX_VIPER: u32 = 0;

// Textures (emissive)
static mut TEX_SPEEDSTER_EMISSIVE: u32 = 0;
static mut TEX_MUSCLE_EMISSIVE: u32 = 0;
static mut TEX_RACER_EMISSIVE: u32 = 0;
static mut TEX_DRIFT_EMISSIVE: u32 = 0;
static mut TEX_PHANTOM_EMISSIVE: u32 = 0;
static mut TEX_TITAN_EMISSIVE: u32 = 0;
static mut TEX_VIPER_EMISSIVE: u32 = 0;

// Sounds
static mut SND_BOOST: u32 = 0;
static mut SND_DRIFT: u32 = 0;
static mut SND_WALL: u32 = 0;
static mut SND_CHECKPOINT: u32 = 0;
static mut SND_FINISH: u32 = 0;

// Font
static mut TEX_NEON_FONT: u32 = 0;
static mut NEON_FONT: u32 = 0;

// Title screen & attract mode
static mut TITLE_ANIM_TIME: f32 = 0.0;
static mut IDLE_TIMER: f32 = 0.0;
const ATTRACT_MODE_DELAY: f32 = 15.0;  // Start attract mode after 15 seconds idle

// Particle system
static mut PARTICLES: [Particle; MAX_PARTICLES] = [Particle::new(); MAX_PARTICLES];
static mut NEXT_PARTICLE: usize = 0;

// Visual effect state per player
static mut SPEED_LINE_INTENSITY: [f32; 4] = [0.0; 4];  // 0.0-1.0 based on speed
static mut BOOST_GLOW_INTENSITY: [f32; 4] = [0.0; 4];  // Pulsing boost effect

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
            CarType::Phantom => {
                self.max_speed = 31.5;    // 105%
                self.acceleration = 14.5; // 95%
                self.handling = 0.9;      // 90%
                self.drift_factor = 0.88;
            }
            CarType::Titan => {
                self.max_speed = 25.5;    // 85%
                self.acceleration = 13.5; // 85%
                self.handling = 0.75;     // 75%
                self.drift_factor = 0.7;
            }
            CarType::Viper => {
                self.max_speed = 36.0;    // 120%
                self.acceleration = 11.5; // 75%
                self.handling = 1.05;     // 105%
                self.drift_factor = 0.95;
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
            shake_intensity: 0.0,
            shake_decay: 0.9,
            shake_offset_x: 0.0,
            shake_offset_y: 0.0,
        }
    }

    fn add_shake(&mut self, intensity: f32) {
        self.shake_intensity = (self.shake_intensity + intensity).min(1.0);
    }

    fn update_shake(&mut self, rand_val: u32) {
        if self.shake_intensity > 0.01 {
            // Generate pseudo-random offset from random value
            let rx = ((rand_val & 0xFF) as f32 / 128.0) - 1.0;
            let ry = (((rand_val >> 8) & 0xFF) as f32 / 128.0) - 1.0;
            self.shake_offset_x = rx * self.shake_intensity * 0.3;
            self.shake_offset_y = ry * self.shake_intensity * 0.2;
            self.shake_intensity *= self.shake_decay;
        } else {
            self.shake_intensity = 0.0;
            self.shake_offset_x = 0.0;
            self.shake_offset_y = 0.0;
        }
    }
}

// === Particle System Functions ===

fn spawn_particle(x: f32, y: f32, z: f32, vel_x: f32, vel_y: f32, vel_z: f32,
                  life: f32, size: f32, color: u32, ptype: ParticleType) {
    unsafe {
        let p = &mut PARTICLES[NEXT_PARTICLE];
        p.active = true;
        p.x = x;
        p.y = y;
        p.z = z;
        p.vel_x = vel_x;
        p.vel_y = vel_y;
        p.vel_z = vel_z;
        p.life = life;
        p.max_life = life;
        p.size = size;
        p.color = color;
        p.particle_type = ptype;
        NEXT_PARTICLE = (NEXT_PARTICLE + 1) % MAX_PARTICLES;
    }
}

fn spawn_boost_flames(car: &Car) {
    // Spawn orange/cyan flames behind car when boosting
    let angle = car.rotation_y;
    let sin_a = libm::sinf(angle);
    let cos_a = libm::cosf(angle);

    // Exhaust position (behind car)
    let exhaust_x = car.x - sin_a * 0.6;
    let exhaust_z = car.z - cos_a * 0.6;

    unsafe {
        let rand = random();
        let spread = ((rand & 0xFF) as f32 / 255.0 - 0.5) * 0.2;
        let vel_back = -car.velocity_forward * 0.3;

        // Cyan core flame
        spawn_particle(
            exhaust_x + spread * cos_a,
            0.15,
            exhaust_z + spread * sin_a,
            -sin_a * vel_back + spread,
            0.5 + (rand >> 16) as f32 / 65536.0 * 0.3,
            -cos_a * vel_back,
            0.3, 0.15, 0x00FFFFFF, ParticleType::BoostFlame
        );

        // Orange outer flame
        spawn_particle(
            exhaust_x + spread * cos_a * 1.2,
            0.12,
            exhaust_z + spread * sin_a * 1.2,
            -sin_a * vel_back * 0.8,
            0.3,
            -cos_a * vel_back * 0.8,
            0.4, 0.2, 0xFF6600FF, ParticleType::BoostFlame
        );
    }
}

fn spawn_drift_smoke(car: &Car) {
    // Spawn gray smoke from rear wheels when drifting
    let angle = car.rotation_y;
    let sin_a = libm::sinf(angle);
    let cos_a = libm::cosf(angle);

    unsafe {
        let rand = random();

        // Left wheel smoke
        let left_x = car.x - sin_a * 0.3 + cos_a * 0.3;
        let left_z = car.z - cos_a * 0.3 - sin_a * 0.3;
        spawn_particle(
            left_x, 0.05, left_z,
            ((rand & 0xFF) as f32 / 255.0 - 0.5) * 0.5,
            0.2,
            ((rand >> 8 & 0xFF) as f32 / 255.0 - 0.5) * 0.5,
            0.6, 0.25, 0x888888AA, ParticleType::DriftSmoke
        );

        // Right wheel smoke
        let right_x = car.x - sin_a * 0.3 - cos_a * 0.3;
        let right_z = car.z - cos_a * 0.3 + sin_a * 0.3;
        spawn_particle(
            right_x, 0.05, right_z,
            ((rand >> 16 & 0xFF) as f32 / 255.0 - 0.5) * 0.5,
            0.2,
            ((rand >> 24 & 0xFF) as f32 / 255.0 - 0.5) * 0.5,
            0.6, 0.25, 0x888888AA, ParticleType::DriftSmoke
        );
    }
}

fn spawn_collision_sparks(x: f32, z: f32) {
    unsafe {
        for _ in 0..8 {
            let rand = random();
            let vx = ((rand & 0xFF) as f32 / 128.0 - 1.0) * 3.0;
            let vy = ((rand >> 8 & 0xFF) as f32 / 255.0) * 2.0 + 1.0;
            let vz = ((rand >> 16 & 0xFF) as f32 / 128.0 - 1.0) * 3.0;

            spawn_particle(
                x, 0.2, z,
                vx, vy, vz,
                0.3, 0.05, 0xFFFF00FF, ParticleType::Spark
            );
        }
    }
}

fn update_particles(dt: f32) {
    unsafe {
        for p in PARTICLES.iter_mut() {
            if !p.active { continue; }

            // Update position
            p.x += p.vel_x * dt;
            p.y += p.vel_y * dt;
            p.z += p.vel_z * dt;

            // Apply gravity/drag based on type
            match p.particle_type {
                ParticleType::BoostFlame => {
                    p.vel_y -= 2.0 * dt; // Light upward then fall
                    p.size *= 0.95; // Shrink
                }
                ParticleType::DriftSmoke => {
                    p.vel_y += 0.5 * dt; // Rise
                    p.size *= 1.02; // Expand
                    // Fade velocity
                    p.vel_x *= 0.95;
                    p.vel_z *= 0.95;
                }
                ParticleType::Spark => {
                    p.vel_y -= 15.0 * dt; // Heavy gravity
                }
                ParticleType::SpeedLine => {
                    // Just streak backward
                }
            }

            // Update life
            p.life -= dt;
            if p.life <= 0.0 || p.y < -0.5 {
                p.active = false;
            }
        }
    }
}

fn render_particles() {
    unsafe {
        for p in PARTICLES.iter() {
            if !p.active { continue; }

            let alpha = ((p.life / p.max_life) * 255.0) as u32;
            let color = (p.color & 0xFFFFFF00) | alpha;

            push_identity();
            push_translate(p.x, p.y, p.z);
            draw_billboard(p.size, p.size, 0, color);
        }
    }
}

fn render_speed_lines(player_id: usize, vp_width: u32, vp_height: u32) {
    unsafe {
        let intensity = SPEED_LINE_INTENSITY[player_id];
        if intensity < 0.1 { return; }

        // Draw radial speed lines from center
        let cx = vp_width as f32 / 2.0;
        let cy = vp_height as f32 / 2.0;
        let alpha = (intensity * 180.0) as u32;
        let color = 0xFFFFFF00 | alpha;

        // Draw lines radiating outward
        for i in 0..12 {
            let angle = (i as f32 / 12.0) * 6.28318;
            let sin_a = libm::sinf(angle);
            let cos_a = libm::cosf(angle);

            let inner_r = 50.0 + (1.0 - intensity) * 100.0;
            let outer_r = inner_r + intensity * 200.0;

            let x1 = cx + cos_a * inner_r;
            let y1 = cy + sin_a * inner_r;
            let x2 = cx + cos_a * outer_r;
            let y2 = cy + sin_a * outer_r;

            draw_line(x1, y1, x2, y2, 2.0 + intensity * 3.0, color);
        }
    }
}

fn render_vignette(intensity: f32) {
    unsafe {
        if intensity < 0.01 { return; }

        // Draw dark corners for vignette effect
        let alpha = (intensity * 100.0) as u32;
        let color = alpha; // Black with alpha

        // Top corners
        draw_rect(0.0, 0.0, 200.0, 100.0, color);
        draw_rect(SCREEN_WIDTH as f32 - 200.0, 0.0, 200.0, 100.0, color);

        // Bottom corners
        draw_rect(0.0, SCREEN_HEIGHT as f32 - 100.0, 200.0, 100.0, color);
        draw_rect(SCREEN_WIDTH as f32 - 200.0, SCREEN_HEIGHT as f32 - 100.0, 200.0, 100.0, color);
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
        MESH_PHANTOM = load_rom_mesh(b"phantom");
        MESH_TITAN = load_rom_mesh(b"titan");
        MESH_VIPER = load_rom_mesh(b"viper");

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

        // Load sounds
        SND_BOOST = load_rom_sound(b"boost");
        SND_DRIFT = load_rom_sound(b"drift");
        SND_WALL = load_rom_sound(b"wall");
        SND_CHECKPOINT = load_rom_sound(b"checkpoint");
        SND_FINISH = load_rom_sound(b"finish");

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
            GameMode::MainMenu => {
                update_main_menu(dt);
            }
            GameMode::CarSelect => {
                update_car_select();
            }
            GameMode::TrackSelect => {
                update_track_select();
            }
            GameMode::CountdownReady => {
                update_countdown(dt);
            }
            GameMode::Racing => {
                update_racing(dt);
            }
            GameMode::RaceFinished => {
                update_results();
            }
            GameMode::Paused => {
                update_paused();
            }
            GameMode::AttractMode => {
                update_attract_mode(dt);
            }
        }
    }
}

fn update_main_menu(dt: f32) {
    unsafe {
        let any_input = input_buttons(0) != 0;

        // Any input resets idle timer
        if any_input {
            IDLE_TIMER = 0.0;
        } else {
            IDLE_TIMER += dt;
        }

        // Start attract mode after idle
        if IDLE_TIMER >= ATTRACT_MODE_DELAY {
            start_attract_mode();
            return;
        }

        // D-pad up/down to change selection
        if button_pressed(0, 0) != 0 { // D-pad up
            if MENU_SELECTION > 0 { MENU_SELECTION -= 1; }
        }
        if button_pressed(0, 1) != 0 { // D-pad down
            if MENU_SELECTION < 2 { MENU_SELECTION += 1; }
        }

        // A button to select
        if button_pressed(0, BUTTON_A) != 0 {
            match MENU_SELECTION {
                0 => { // Single Race
                    GAME_MODE = GameMode::CarSelect;
                    for i in 0..4 { PLAYER_CONFIRMED[i] = false; }
                }
                1 => { // Quick Race (skip to racing)
                    GAME_MODE = GameMode::CountdownReady;
                    COUNTDOWN_TIMER = 240; // 4 seconds at 60fps
                    init_race();
                }
                _ => {}
            }
        }
    }
}

fn update_car_select() {
    unsafe {
        let pcount = player_count();
        ACTIVE_PLAYER_COUNT = pcount;

        // Update animation timer
        MENU_TIME += 1.0 / 60.0;

        // Each player can select their car
        for p in 0..pcount as usize {
            if PLAYER_CONFIRMED[p] { continue; }

            // Left/right to change car
            if button_pressed(p as u32, 2) != 0 { // D-pad left
                if CAR_SELECTIONS[p] > 0 { CAR_SELECTIONS[p] -= 1; }
            }
            if button_pressed(p as u32, 3) != 0 { // D-pad right
                if CAR_SELECTIONS[p] < 6 { CAR_SELECTIONS[p] += 1; }
            }

            // A to confirm
            if button_pressed(p as u32, BUTTON_A) != 0 {
                PLAYER_CONFIRMED[p] = true;
                // Set the car type
                CARS[p].car_type = match CAR_SELECTIONS[p] {
                    0 => CarType::Speedster,
                    1 => CarType::Muscle,
                    2 => CarType::Racer,
                    3 => CarType::Drift,
                    4 => CarType::Phantom,
                    5 => CarType::Titan,
                    _ => CarType::Viper,
                };
                CARS[p].init_stats();
            }
        }

        // Check if all players confirmed
        let mut all_confirmed = true;
        for p in 0..pcount as usize {
            if !PLAYER_CONFIRMED[p] { all_confirmed = false; break; }
        }

        if all_confirmed {
            GAME_MODE = GameMode::TrackSelect;
            MENU_SELECTION = 0;
        }

        // B to go back
        if button_pressed(0, BUTTON_B) != 0 {
            GAME_MODE = GameMode::MainMenu;
        }
    }
}

fn update_track_select() {
    unsafe {
        // Update animation timer
        MENU_TIME += 1.0 / 60.0;

        // Only P1 selects track (up/down to navigate)
        if button_pressed(0, 0) != 0 { // D-pad up
            if MENU_SELECTION > 0 { MENU_SELECTION -= 1; }
        }
        if button_pressed(0, 1) != 0 { // D-pad down
            if MENU_SELECTION < 4 { MENU_SELECTION += 1; }
        }

        // A to confirm
        if button_pressed(0, BUTTON_A) != 0 {
            SELECTED_TRACK = match MENU_SELECTION {
                0 => TrackId::SunsetStrip,
                1 => TrackId::NeonCity,
                2 => TrackId::VoidTunnel,
                3 => TrackId::CrystalCavern,
                _ => TrackId::SolarHighway,
            };
            GAME_MODE = GameMode::CountdownReady;
            COUNTDOWN_TIMER = 240; // 4 seconds
            init_race();
        }

        // B to go back
        if button_pressed(0, BUTTON_B) != 0 {
            GAME_MODE = GameMode::CarSelect;
            for p in 0..4 { PLAYER_CONFIRMED[p] = false; }
        }
    }
}

fn update_countdown(dt: f32) {
    unsafe {
        if COUNTDOWN_TIMER > 0 {
            COUNTDOWN_TIMER -= 1;
        } else {
            GAME_MODE = GameMode::Racing;
            RACE_TIME = 0.0;
            RACE_FINISHED = false;
        }

        // Update cameras during countdown
        for i in 0..ACTIVE_PLAYER_COUNT as usize {
            update_camera(&mut CAMERAS[i], &CARS[i], dt);
        }
    }
}

fn update_results() {
    unsafe {
        // A to return to main menu
        if button_pressed(0, BUTTON_A) != 0 {
            GAME_MODE = GameMode::MainMenu;
            MENU_SELECTION = 0;
        }
    }
}

fn update_paused() {
    unsafe {
        // Start to unpause
        if button_pressed(0, BUTTON_START) != 0 {
            GAME_MODE = GameMode::Racing;
        }
        // Select to quit to menu
        if button_pressed(0, BUTTON_SELECT) != 0 {
            GAME_MODE = GameMode::MainMenu;
        }
    }
}

// === Attract Mode Functions ===

fn start_attract_mode() {
    unsafe {
        GAME_MODE = GameMode::AttractMode;
        IDLE_TIMER = 0.0;

        // Set up demo race with all AI cars
        init_race();

        // Position camera for spectator view
        for i in 0..4 {
            // Give AI cars some variety in starting boost
            CARS[i].boost_meter = 0.3 + (random() % 40) as f32 / 100.0;
        }
    }
}

fn update_attract_mode(dt: f32) {
    unsafe {
        // Any input exits attract mode
        if input_buttons(0) != 0 || input_buttons(1) != 0 {
            GAME_MODE = GameMode::MainMenu;
            IDLE_TIMER = 0.0;
            TITLE_ANIM_TIME = 0.0;
            return;
        }

        // Run the race simulation with all AI
        RACE_TIME += dt;

        // Update all cars as AI
        for i in 0..4 {
            update_ai_car(&mut CARS[i], dt);
            check_track_collision(&mut CARS[i]);
            check_checkpoints(&mut CARS[i], i);
        }

        // Cycle camera between cars for spectator view
        let camera_car_idx = ((RACE_TIME * 0.2) as usize) % 4;
        update_camera(&mut CAMERAS[0], &CARS[camera_car_idx], dt);

        // After 30 seconds or if race ends, return to menu
        if RACE_TIME > 30.0 {
            GAME_MODE = GameMode::MainMenu;
            IDLE_TIMER = 0.0;
            TITLE_ANIM_TIME = 0.0;
        }

        // Check for race finish (any car completes 3 laps)
        for i in 0..4 {
            if CARS[i].current_lap > 3 {
                GAME_MODE = GameMode::MainMenu;
                IDLE_TIMER = 0.0;
                TITLE_ANIM_TIME = 0.0;
                break;
            }
        }
    }
}

fn init_race() {
    unsafe {
        // Initialize car positions on starting grid
        for i in 0..4 {
            CARS[i].x = ((i % 2) as f32) * 3.0 - 1.5; // 2 columns
            CARS[i].y = 0.0;
            CARS[i].z = -((i / 2) as f32) * 4.0 - 5.0; // 2 rows, behind start
            CARS[i].rotation_y = 0.0;
            CARS[i].velocity_forward = 0.0;
            CARS[i].velocity_lateral = 0.0;
            CARS[i].boost_meter = 0.5; // Start with half boost
            CARS[i].current_lap = 1;
            CARS[i].last_checkpoint = 0;
            CARS[i].race_position = (i + 1) as u32;
        }

        RACE_TIME = 0.0;
        RACE_FINISHED = false;
    }
}

fn update_racing(dt: f32) {
    unsafe {
        if RACE_FINISHED { return; }

        let active_count = player_count();
        ACTIVE_PLAYER_COUNT = active_count;
        RACE_TIME += dt;

        // Check for pause
        if button_pressed(0, BUTTON_START) != 0 {
            GAME_MODE = GameMode::Paused;
            return;
        }

        // Update all player cars
        for i in 0..active_count as usize {
            let was_boosting = CARS[i].is_boosting;
            update_car_physics(&mut CARS[i], i as u32, dt);
            check_track_collision_with_effects(&mut CARS[i], i);
            check_checkpoints(&mut CARS[i], i);
            update_camera(&mut CAMERAS[i], &CARS[i], dt);

            // Screen shake on boost start
            if CARS[i].is_boosting && !was_boosting {
                CAMERAS[i].add_shake(0.3);
            }

            // Spawn visual effects
            if CARS[i].is_boosting {
                spawn_boost_flames(&CARS[i]);
                BOOST_GLOW_INTENSITY[i] = 1.0;
            } else {
                BOOST_GLOW_INTENSITY[i] *= 0.9;
            }

            if CARS[i].is_drifting {
                spawn_drift_smoke(&CARS[i]);
            }

            // Update speed line intensity based on velocity
            let speed_ratio = CARS[i].velocity_forward / CARS[i].max_speed;
            let target_intensity = if speed_ratio > 0.8 { (speed_ratio - 0.8) * 5.0 } else { 0.0 };
            SPEED_LINE_INTENSITY[i] = SPEED_LINE_INTENSITY[i] * 0.9 + target_intensity * 0.1;

            // Update screen shake
            let rand = random();
            CAMERAS[i].update_shake(rand);
        }

        // Update AI cars (fill remaining slots)
        for i in active_count as usize..4 {
            update_ai_car(&mut CARS[i], dt);
            check_track_collision(&mut CARS[i]);
            check_checkpoints(&mut CARS[i], i);
        }

        // Update particle system
        update_particles(dt);

        // Calculate positions
        calculate_positions();

        // Check for race finish
        for i in 0..4 {
            if CARS[i].current_lap > 3 {
                RACE_FINISHED = true;
                GAME_MODE = GameMode::RaceFinished;
                play_sound(SND_FINISH, 1.0, 0.0);

                // Victory screen shake!
                for j in 0..4 {
                    CAMERAS[j].add_shake(0.5);
                }
                break;
            }
        }
    }
}

fn check_track_collision_with_effects(car: &mut Car, player_idx: usize) {
    unsafe {
        // Simple track boundary: road is 10 units wide (-5 to +5 in X)
        let track_half_width = 5.0;

        if car.x > track_half_width {
            car.x = track_half_width;
            car.velocity_forward *= 0.7; // Slow down on wall hit
            car.velocity_lateral = -car.velocity_lateral.abs() * 0.3;
            car.boost_meter = (car.boost_meter - 0.1).max(0.0);
            play_sound(SND_WALL, 0.5, (car.x / track_half_width).clamp(-1.0, 1.0));

            // Collision effects!
            CAMERAS[player_idx].add_shake(0.4);
            spawn_collision_sparks(car.x, car.z);
        }
        if car.x < -track_half_width {
            car.x = -track_half_width;
            car.velocity_forward *= 0.7;
            car.velocity_lateral = car.velocity_lateral.abs() * 0.3;
            car.boost_meter = (car.boost_meter - 0.1).max(0.0);
            play_sound(SND_WALL, 0.5, (car.x / track_half_width).clamp(-1.0, 1.0));

            // Collision effects!
            CAMERAS[player_idx].add_shake(0.4);
            spawn_collision_sparks(car.x, car.z);
        }

        // Check boost pad pickups (5 pads at set positions)
        for i in 0..5 {
            let pad_z = (i as f32) * 40.0 + 30.0;
            let pad_x = if i % 2 == 0 { -2.0 } else { 2.0 };

            // Simple AABB collision (pad is roughly 2x2 units)
            let dx = (car.x - pad_x).abs();
            let dz = (car.z % TRACK_LENGTH - pad_z).abs();

            if dx < 1.5 && dz < 1.5 {
                // Give boost!
                if car.boost_meter < 0.95 {
                    car.boost_meter = (car.boost_meter + 0.3).min(1.0);
                    play_sound(SND_BOOST, 0.6, 0.0);
                    CAMERAS[player_idx].add_shake(0.15); // Small shake on boost pickup
                }
            }
        }
    }
}

fn check_track_collision(car: &mut Car) {
    unsafe {
        // Simple track boundary: road is 10 units wide (-5 to +5 in X)
        let track_half_width = 5.0;

        if car.x > track_half_width {
            car.x = track_half_width;
            car.velocity_forward *= 0.7; // Slow down on wall hit
            car.velocity_lateral = -car.velocity_lateral.abs() * 0.3;
            car.boost_meter = (car.boost_meter - 0.1).max(0.0);
            play_sound(SND_WALL, 0.5, (car.x / track_half_width).clamp(-1.0, 1.0));
        }
        if car.x < -track_half_width {
            car.x = -track_half_width;
            car.velocity_forward *= 0.7;
            car.velocity_lateral = car.velocity_lateral.abs() * 0.3;
            car.boost_meter = (car.boost_meter - 0.1).max(0.0);
            play_sound(SND_WALL, 0.5, (car.x / track_half_width).clamp(-1.0, 1.0));
        }

        // Check boost pad pickups (5 pads at set positions)
        for i in 0..5 {
            let pad_z = (i as f32) * 40.0 + 30.0;
            let pad_x = if i % 2 == 0 { -2.0 } else { 2.0 };

            // Simple AABB collision (pad is roughly 2x2 units)
            let dx = (car.x - pad_x).abs();
            let dz = (car.z % TRACK_LENGTH - pad_z).abs();

            if dx < 1.5 && dz < 1.5 {
                // Give boost!
                if car.boost_meter < 0.95 {
                    car.boost_meter = (car.boost_meter + 0.3).min(1.0);
                    play_sound(SND_BOOST, 0.6, 0.0);
                }
            }
        }
    }
}

fn check_checkpoints(car: &mut Car, _car_idx: usize) {
    unsafe {
        // Simple checkpoint system based on Z position
        let next_cp = (car.last_checkpoint + 1) % NUM_CHECKPOINTS;
        let cp_z = CHECKPOINT_Z[next_cp];

        // Check if car crossed this checkpoint (moving forward)
        if car.z > cp_z && car.velocity_forward > 0.0 {
            car.last_checkpoint = next_cp;

            // If we completed all checkpoints and crossed start/finish
            if next_cp == 0 {
                car.current_lap += 1;
                play_sound(SND_CHECKPOINT, 0.8, 0.0);
            }
        }

        // Wrap track (after Z > TRACK_LENGTH, reset to beginning)
        if car.z > TRACK_LENGTH {
            car.z -= TRACK_LENGTH;
        }
        if car.z < 0.0 {
            car.z += TRACK_LENGTH;
        }
    }
}

fn update_ai_car(car: &mut Car, dt: f32) {
    unsafe {
        // Simple AI: accelerate forward, steer toward center
        let center_error = -car.x; // Target X = 0
        let steer = (center_error * 0.2).clamp(-1.0, 1.0);

        // Accelerate
        car.velocity_forward += car.acceleration * 0.8 * dt;

        // Apply steering
        let speed_factor = (car.velocity_forward / car.max_speed).min(1.0);
        car.angular_velocity = steer * car.handling * 60.0 * speed_factor;
        car.rotation_y += car.angular_velocity * dt;

        // Clamp speed
        car.velocity_forward = car.velocity_forward.clamp(0.0, car.max_speed * 0.9);

        // Random boost usage
        if car.boost_meter > 0.7 && (random() % 200) == 0 {
            car.is_boosting = true;
            car.boost_timer = BOOST_DURATION / 2;
            car.boost_meter -= BOOST_COST;
        }

        // Update boost
        if car.boost_timer > 0 {
            car.boost_timer -= 1;
            if car.boost_timer == 0 { car.is_boosting = false; }
        }

        let boost_mult = if car.is_boosting { BOOST_MULTIPLIER } else { 1.0 };
        let max_spd = car.max_speed * boost_mult;
        car.velocity_forward = car.velocity_forward.clamp(0.0, max_spd);

        // Update position
        let sin_rot = libm::sinf(car.rotation_y * 3.14159 / 180.0);
        let cos_rot = libm::cosf(car.rotation_y * 3.14159 / 180.0);
        car.x += sin_rot * car.velocity_forward * dt;
        car.z += cos_rot * car.velocity_forward * dt;
    }
}

fn calculate_positions() {
    unsafe {
        // Calculate race progress for each car
        let mut progress: [(f32, usize); 4] = [(0.0, 0); 4];

        for i in 0..4 {
            let car = &CARS[i];
            // Progress = (laps * track_length) + current_z + (checkpoint_progress)
            let cp_progress = (car.last_checkpoint as f32) * (TRACK_LENGTH / NUM_CHECKPOINTS as f32);
            progress[i] = (
                (car.current_lap as f32) * TRACK_LENGTH + cp_progress + car.z,
                i
            );
        }

        // Sort by progress (descending)
        for i in 0..4 {
            for j in i+1..4 {
                if progress[j].0 > progress[i].0 {
                    let tmp = progress[i];
                    progress[i] = progress[j];
                    progress[j] = tmp;
                }
            }
        }

        // Assign positions
        for pos in 0..4 {
            CARS[progress[pos].1].race_position = (pos + 1) as u32;
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

fn update_camera(camera: &mut Camera, car: &Car, _dt: f32) {
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

// === Render ===

#[no_mangle]
pub extern "C" fn render() {
    unsafe {
        match GAME_MODE {
            GameMode::MainMenu => {
                render_main_menu();
            }
            GameMode::CarSelect => {
                render_car_select();
            }
            GameMode::TrackSelect => {
                render_track_select();
            }
            GameMode::CountdownReady => {
                render_countdown();
            }
            GameMode::Racing => {
                render_racing();
            }
            GameMode::RaceFinished => {
                render_results();
            }
            GameMode::Paused => {
                render_paused();
            }
            GameMode::AttractMode => {
                render_attract_mode();
            }
        }
    }
}

fn render_attract_mode() {
    unsafe {
        // Render the demo race (same as racing but with spectator camera)
        render_racing();

        // Overlay "DEMO" text and prompt
        font_bind(NEON_FONT);
        depth_test(0);

        // Demo banner at top
        let t = TITLE_ANIM_TIME;
        let demo_pulse = (libm::sinf(t * 3.0) * 0.2 + 0.8) as f32;
        let demo_alpha = (demo_pulse * 255.0) as u32;
        let demo_color = 0xFF00FF00 | demo_alpha;

        let demo_text = b"DEMO MODE";
        draw_text(demo_text.as_ptr(), demo_text.len() as u32, 400.0, 20.0, 32.0, demo_color);

        // Press any button prompt
        let prompt = b"PRESS ANY BUTTON";
        let blink = if (t * 2.0) as u32 % 2 == 0 { 0xFFFFFFFF } else { 0x666666FF };
        draw_text(prompt.as_ptr(), prompt.len() as u32, 360.0, 500.0, 20.0, blink);

        depth_test(1);
    }
}

fn render_main_menu() {
    unsafe {
        // Use custom neon font
        font_bind(NEON_FONT);

        let t = TITLE_ANIM_TIME;

        // === Animated background lines ===
        let line_color = 0x00FFFF40u32; // Cyan with transparency
        for i in 0..8 {
            let offset = libm::sinf(t * 0.3 + i as f32 * 0.5) * 20.0;
            let y = 50.0 + i as f32 * 70.0;
            draw_line(0.0, y + offset, 960.0, y - offset + 10.0, 1.0, line_color);
        }

        // === Glowing title with pulsing effect ===
        let title = b"NEON DRIFT";
        let title_scale = 64.0;
        let title_x = 480.0 - (title.len() as f32 * title_scale * 0.4); // Center
        let title_y = 80.0;

        // Glow effect (draw behind at larger size)
        let glow_pulse = (libm::sinf(t * 2.0) * 0.3 + 0.7) as f32;
        let glow_alpha = (glow_pulse * 100.0) as u32;
        let glow_color = 0x00FFFF00 | glow_alpha;
        draw_text(title.as_ptr(), title.len() as u32,
                  title_x - 2.0, title_y - 2.0, title_scale + 4.0, glow_color);

        // Main title with color cycling
        let hue_shift = (libm::sinf(t * 0.5) * 0.5 + 0.5) as f32;
        let r = (255.0 * (1.0 - hue_shift * 0.5)) as u32;
        let g = 255u32;
        let b = (255.0 * (0.5 + hue_shift * 0.5)) as u32;
        let title_color = (r << 24) | (g << 16) | (b << 8) | 0xFF;
        draw_text(title.as_ptr(), title.len() as u32,
                  title_x, title_y, title_scale, title_color);

        // === Subtitle ===
        let subtitle = b"ARCADE RACING";
        let sub_x = 480.0 - (subtitle.len() as f32 * 16.0 * 0.4);
        let sub_alpha = (libm::sinf(t * 1.5) * 0.3 + 0.7) * 255.0;
        let sub_color = 0xFF00FF00 | (sub_alpha as u32);
        draw_text(subtitle.as_ptr(), subtitle.len() as u32, sub_x, 160.0, 24.0, sub_color);

        // === Menu options with animated selection ===
        let options = [b"SINGLE RACE" as &[u8], b"QUICK RACE", b"TIME TRIAL"];
        for (i, opt) in options.iter().enumerate() {
            let y = 240.0 + (i as f32 * 50.0);
            let is_selected = MENU_SELECTION == i as u32;

            // Selection highlight box
            if is_selected {
                let box_pulse = (libm::sinf(t * 4.0) * 0.1 + 0.9) as f32;
                let box_width = 300.0 * box_pulse;
                let box_x = 480.0 - box_width / 2.0;
                draw_rect(box_x, y - 5.0, box_width, 35.0, 0x00FFFF30);

                // Animated arrows
                let arrow_offset = (libm::sinf(t * 6.0) * 5.0) as f32;
                draw_text(b">".as_ptr(), 1, 330.0 - arrow_offset, y, 24.0, COLOR_CYAN);
                draw_text(b"<".as_ptr(), 1, 620.0 + arrow_offset, y, 24.0, COLOR_CYAN);
            }

            let color = if is_selected { COLOR_CYAN } else { 0xAAAAAAFF };
            let opt_x = 480.0 - (opt.len() as f32 * 24.0 * 0.4);
            draw_text(opt.as_ptr(), opt.len() as u32, opt_x, y, 24.0, color);
        }

        // === Prompt ===
        let prompt = b"PRESS A TO SELECT";
        let prompt_x = 480.0 - (prompt.len() as f32 * 14.0 * 0.4);
        let blink = if (t * 2.0) as u32 % 2 == 0 { 0xFFFFFFFF } else { 0x888888FF };
        draw_text(prompt.as_ptr(), prompt.len() as u32, prompt_x, 420.0, 14.0, blink);

        // === Footer ===
        let footer = b"(C) 2024 NETHERCORE";
        let footer_x = 480.0 - (footer.len() as f32 * 12.0 * 0.4);
        draw_text(footer.as_ptr(), footer.len() as u32, footer_x, 500.0, 12.0, 0x444444FF);

        // === Idle timer indicator (subtle) ===
        if IDLE_TIMER > ATTRACT_MODE_DELAY - 5.0 {
            let countdown = (ATTRACT_MODE_DELAY - IDLE_TIMER) as u32;
            let demo_text = b"DEMO IN ";
            let demo_x = 800.0;
            draw_text(demo_text.as_ptr(), demo_text.len() as u32, demo_x, 520.0, 10.0, 0x666666FF);
            let digit = [b'0' + countdown as u8];
            draw_text(digit.as_ptr(), 1, demo_x + 65.0, 520.0, 10.0, 0x666666FF);
        }
    }
}

/// Get car mesh and textures for car selection index
fn get_car_assets(sel: u32) -> (u32, u32, u32) {
    unsafe {
        match sel {
            0 => (MESH_SPEEDSTER, TEX_SPEEDSTER, TEX_SPEEDSTER_EMISSIVE),
            1 => (MESH_MUSCLE, TEX_MUSCLE, TEX_MUSCLE_EMISSIVE),
            2 => (MESH_RACER, TEX_RACER, TEX_RACER_EMISSIVE),
            3 => (MESH_DRIFT, TEX_DRIFT, TEX_DRIFT_EMISSIVE),
            4 => (MESH_PHANTOM, TEX_PHANTOM, TEX_PHANTOM_EMISSIVE),
            5 => (MESH_TITAN, TEX_TITAN, TEX_TITAN_EMISSIVE),
            _ => (MESH_VIPER, TEX_VIPER, TEX_VIPER_EMISSIVE),
        }
    }
}

/// Draw an animated stat bar
fn draw_stat_bar(x: f32, y: f32, width: f32, height: f32, value: f32, color: u32, anim_phase: f32) {
    unsafe {
        // Background bar
        draw_rect(x, y, width, height, 0x222233FF);

        // Animated fill (slight pulse)
        let pulse = 1.0 + 0.05 * libm::sinf(anim_phase * 3.0);
        let fill_width = (width - 4.0) * value * pulse;
        let fill_width = if fill_width > width - 4.0 { width - 4.0 } else { fill_width };

        draw_rect(x + 2.0, y + 2.0, fill_width, height - 4.0, color);

        // Glow effect on high values
        if value > 0.9 {
            let glow_alpha = ((libm::sinf(anim_phase * 5.0) * 0.5 + 0.5) * 128.0) as u32;
            let glow_color = (color & 0xFFFFFF00) | glow_alpha;
            draw_rect(x, y - 1.0, fill_width + 4.0, height + 2.0, glow_color);
        }
    }
}

/// Render 3D car preview
fn render_car_preview_3d(mesh: u32, tex: u32, _tex_emissive: u32, x: f32, y: f32, size: f32, rotation: f32, pulse: f32) {
    unsafe {
        // Setup isolated viewport for this preview
        let preview_x = (x - size / 2.0) as u32;
        let preview_y = (y - size / 2.0) as u32;
        let preview_size = size as u32;

        viewport(preview_x, preview_y, preview_size, preview_size);

        // Draw a dark background rect for the preview area
        draw_rect(0.0, 0.0, size, size, 0x0D0D15FF);

        // Setup camera looking at car from front-right angle
        let cam_dist = 1.8 + pulse * 0.1;
        let cam_angle = rotation;
        let cam_x = cam_dist * libm::cosf(cam_angle);
        let cam_z = cam_dist * libm::sinf(cam_angle);
        camera_set(cam_x, 0.6, cam_z, 0.0, 0.1, 0.0);

        // Setup projection via camera FOV
        camera_fov(45.0);

        // Setup lighting for preview
        light_set(0, -0.5, -1.0, 0.3);  // Directional from above-front
        light_color(0, 0xCCCCFFFF);      // Slightly blue-white
        light_intensity(0, 1.0);
        light_enable(0);

        // Draw car with material
        material_albedo(tex);
        material_metallic(0.7);
        material_roughness(0.3);
        material_emissive(0.2);  // Subtle emissive glow

        push_identity();
        push_rotate_y(-90.0);  // Face the camera
        draw_mesh(mesh);

        // Reset viewport will be done by caller
    }
}

fn render_car_select() {
    unsafe {
        // Car data: (name, speed, accel, handling, difficulty_stars)
        let car_data: [(&[u8], f32, f32, f32, &[u8]); 7] = [
            (b"SPEEDSTER", 0.95, 0.90, 1.00, b"**"),   // 2 stars
            (b"MUSCLE", 1.10, 0.80, 0.85, b"**"),      // 2 stars
            (b"RACER", 0.95, 1.10, 0.95, b"***"),       // 3 stars
            (b"DRIFT", 0.90, 1.00, 1.20, b"**"),      // 2 stars
            (b"PHANTOM", 1.05, 0.95, 0.90, b"**"),     // 2 stars
            (b"TITAN", 0.85, 0.85, 0.75, b"*"),         // 1 star (beginner)
            (b"VIPER", 1.20, 0.75, 1.05, b"***"),     // 3 stars (expert)
        ];
        let pcount = ACTIVE_PLAYER_COUNT;

        // Clear main viewport
        viewport_clear();

        // Animated background
        let bg_pulse = libm::sinf(MENU_TIME * 0.5) * 0.02;
        draw_rect(0.0, 0.0, SCREEN_WIDTH as f32, SCREEN_HEIGHT as f32, 0x0D0D1500);

        // Title with pulse effect
        let title_pulse = 1.0 + 0.05 * libm::sinf(MENU_TIME * 2.0);
        let title = b"SELECT CAR";
        let title_size = 32.0 * title_pulse;
        draw_text(title.as_ptr(), title.len() as u32, 380.0, 40.0, title_size, COLOR_CYAN);

        // Calculate layout based on player count
        let preview_size = if pcount == 1 { 200.0 } else if pcount == 2 { 160.0 } else { 120.0 };

        for p in 0..pcount as usize {
            let sel = CAR_SELECTIONS[p] as usize;
            let (name, speed, accel, handling, diff) = car_data[sel];
            let (mesh, tex, tex_emissive) = get_car_assets(CAR_SELECTIONS[p]);

            // Layout depends on player count
            let (panel_x, panel_y, panel_w, panel_h) = if pcount == 1 {
                (100.0, 100.0, 760.0, 350.0)
            } else if pcount == 2 {
                (100.0, 80.0 + (p as f32 * 220.0), 760.0, 200.0)
            } else {
                (100.0, 80.0 + (p as f32 * 130.0), 760.0, 120.0)
            };

            // Selection panel background with pulse if not confirmed
            let panel_color = if PLAYER_CONFIRMED[p] {
                0x003300AA  // Green tint when confirmed
            } else {
                let pulse = ((libm::sinf(MENU_TIME * 4.0) * 0.5 + 0.5) * 40.0) as u32;
                0x222244AA + (pulse << 16) + pulse
            };
            draw_rect(panel_x, panel_y, panel_w, panel_h, panel_color);

            // Player label with glow
            let plabel = [b'P', b'1' + p as u8];
            let label_color = if PLAYER_CONFIRMED[p] { 0x00FF00FF } else { COLOR_WHITE };
            draw_text(plabel.as_ptr(), 2, panel_x + 10.0, panel_y + 10.0, 28.0, label_color);

            // Car name with selection arrows
            let name_x = panel_x + 70.0;
            let name_y = panel_y + 10.0;
            let name_color = if PLAYER_CONFIRMED[p] { 0x00FF00FF } else { COLOR_CYAN };
            draw_text(name.as_ptr(), name.len() as u32, name_x, name_y, 26.0, name_color);

            // Arrows with pulse when not confirmed
            if !PLAYER_CONFIRMED[p] {
                let arrow_pulse = libm::sinf(MENU_TIME * 6.0);
                let arrow_offset = arrow_pulse * 3.0;
                let left = b"<";
                let right = b">";
                draw_text(left.as_ptr(), 1, name_x - 25.0 - arrow_offset, name_y, 26.0, COLOR_WHITE);
                draw_text(right.as_ptr(), 1, name_x + 130.0 + arrow_offset, name_y, 26.0, COLOR_WHITE);
            } else {
                let ready = b"READY!";
                let ready_pulse = libm::sinf(MENU_TIME * 8.0) * 0.15 + 1.0;
                draw_text(ready.as_ptr(), ready.len() as u32, name_x + 180.0, name_y, 24.0 * ready_pulse, 0x00FF00FF);
            }

            // Difficulty stars
            draw_text(diff.as_ptr(), diff.len() as u32, panel_x + panel_w - 80.0, panel_y + 12.0, 22.0, 0xFFD700FF);

            // Stat bars (animated)
            let bar_x = panel_x + 70.0;
            let bar_y = panel_y + 45.0;
            let bar_w = 180.0;
            let bar_h = 12.0;
            let bar_spacing = 18.0;

            // SPD label and bar
            let spd_label = b"SPD";
            draw_text(spd_label.as_ptr(), 3, bar_x, bar_y, 12.0, 0xAAAAAAFF);
            draw_stat_bar(bar_x + 35.0, bar_y, bar_w, bar_h, speed / 1.2, 0xFF4444FF, MENU_TIME + p as f32);

            // ACC label and bar
            let acc_label = b"ACC";
            draw_text(acc_label.as_ptr(), 3, bar_x, bar_y + bar_spacing, 12.0, 0xAAAAAAFF);
            draw_stat_bar(bar_x + 35.0, bar_y + bar_spacing, bar_w, bar_h, accel / 1.2, 0x44FF44FF, MENU_TIME + p as f32 + 0.5);

            // HND label and bar
            let hnd_label = b"HND";
            draw_text(hnd_label.as_ptr(), 3, bar_x, bar_y + bar_spacing * 2.0, 12.0, 0xAAAAAAFF);
            draw_stat_bar(bar_x + 35.0, bar_y + bar_spacing * 2.0, bar_w, bar_h, handling / 1.2, 0x4444FFFF, MENU_TIME + p as f32 + 1.0);

            // 3D car preview
            let preview_x = panel_x + panel_w - preview_size - 30.0;
            let preview_y = panel_y + panel_h / 2.0;
            let rotation = MENU_TIME * 0.8 + (p as f32 * 0.5);  // Slow rotation
            let preview_pulse = if PLAYER_CONFIRMED[p] { 0.0 } else { libm::sinf(MENU_TIME * 3.0) * 0.1 };

            render_car_preview_3d(mesh, tex, tex_emissive, preview_x + preview_size / 2.0, preview_y, preview_size, rotation, preview_pulse);
        }

        // Reset viewport for UI
        viewport_clear();

        // Bottom prompt
        let prompt = b"A:Confirm  B:Back  (</>:Select)";
        draw_text(prompt.as_ptr(), prompt.len() as u32, 310.0, 510.0, 16.0, 0x888888FF);
    }
}

fn render_track_select() {
    unsafe {
        let title = b"SELECT TRACK";
        draw_text(title.as_ptr(), title.len() as u32, 360.0, 60.0, 32.0, COLOR_CYAN);

        // Track data: (name, description, difficulty_stars)
        let tracks: [(&[u8], &[u8], &[u8]); 5] = [
            (b"SUNSET STRIP", b"Wide roads, gentle curves", b"*"),
            (b"NEON CITY", b"Moderate curves, good visibility", b"**"),
            (b"VOID TUNNEL", b"Disorienting visuals, tight spaces", b"***"),
            (b"CRYSTAL CAVERN", b"Low visibility, technical sections", b"***"),
            (b"SOLAR HIGHWAY", b"High speed, throttle control needed", b"****"),
        ];

        for (i, (name, desc, diff)) in tracks.iter().enumerate() {
            let y = 110.0 + (i as f32 * 75.0);
            let color = if MENU_SELECTION == i as u32 { COLOR_CYAN } else { COLOR_WHITE };

            if MENU_SELECTION == i as u32 {
                draw_rect(180.0, y - 10.0, 600.0, 65.0, 0x222244FF);
            }

            // Track name
            draw_text(name.as_ptr(), name.len() as u32, 200.0, y, 24.0, color);

            // Difficulty stars (yellow)
            draw_text(diff.as_ptr(), diff.len() as u32, 450.0, y, 20.0, 0xFFD700FF);

            // Description
            draw_text(desc.as_ptr(), desc.len() as u32, 200.0, y + 28.0, 14.0, 0x888888FF);
        }

        let prompt = b"A:Select  B:Back  (^/v:Choose)";
        draw_text(prompt.as_ptr(), prompt.len() as u32, 340.0, 500.0, 16.0, 0x666666FF);
    }
}

fn render_countdown() {
    unsafe {
        // First render the racing view
        render_racing_view();

        // Then overlay the countdown
        viewport_clear();

        let number = (COUNTDOWN_TIMER / 60) + 1;
        let text: &[u8] = match number {
            4 => b"3",
            3 => b"3",
            2 => b"2",
            1 => b"1",
            _ => b"GO!",
        };

        let size = if number > 0 { 96.0 } else { 72.0 };
        let color = if number > 0 { COLOR_WHITE } else { 0x00FF00FF };

        draw_text(text.as_ptr(), text.len() as u32,
                  SCREEN_WIDTH as f32 / 2.0 - 40.0, SCREEN_HEIGHT as f32 / 2.0 - 50.0,
                  size, color);
    }
}

fn render_results() {
    unsafe {
        // Background
        draw_rect(0.0, 0.0, SCREEN_WIDTH as f32, SCREEN_HEIGHT as f32, 0x111122DD);

        let title = b"RACE COMPLETE";
        draw_text(title.as_ptr(), title.len() as u32, 320.0, 60.0, 36.0, COLOR_CYAN);

        // Show final positions
        let pos_text = [b"1ST" as &[u8], b"2ND", b"3RD", b"4TH"];
        let pos_colors = [0xFFD700FF, 0xC0C0C0FF, 0xCD7F32FF, COLOR_WHITE];

        for i in 0..4 {
            let car_idx = (0..4).find(|&c| CARS[c].race_position == (i + 1) as u32).unwrap_or(i);
            let y = 140.0 + (i as f32 * 60.0);

            draw_text(pos_text[i].as_ptr(), pos_text[i].len() as u32, 300.0, y, 28.0, pos_colors[i]);

            let car_name = match CARS[car_idx].car_type {
                CarType::Speedster => b"SPEEDSTER" as &[u8],
                CarType::Muscle => b"MUSCLE",
                CarType::Racer => b"RACER",
                CarType::Drift => b"DRIFT",
                CarType::Phantom => b"PHANTOM",
                CarType::Titan => b"TITAN",
                CarType::Viper => b"VIPER",
            };
            draw_text(car_name.as_ptr(), car_name.len() as u32, 400.0, y, 24.0, COLOR_WHITE);

            // Player or AI label
            if car_idx < ACTIVE_PLAYER_COUNT as usize {
                let plabel = [b'P', b'1' + car_idx as u8];
                draw_text(plabel.as_ptr(), 2, 550.0, y, 20.0, COLOR_CYAN);
            } else {
                let cpu = b"CPU";
                draw_text(cpu.as_ptr(), 3, 550.0, y, 20.0, 0x888888FF);
            }
        }

        // Race time
        let time_label = b"TIME:";
        draw_text(time_label.as_ptr(), 5, 360.0, 400.0, 20.0, COLOR_WHITE);

        let mins = (RACE_TIME / 60.0) as u32;
        let secs = (RACE_TIME % 60.0) as u32;
        let mut time_str = [b'0', b'0', b':', b'0', b'0'];
        time_str[0] = b'0' + ((mins / 10) % 10) as u8;
        time_str[1] = b'0' + (mins % 10) as u8;
        time_str[3] = b'0' + ((secs / 10) % 10) as u8;
        time_str[4] = b'0' + (secs % 10) as u8;
        draw_text(time_str.as_ptr(), 5, 430.0, 400.0, 20.0, COLOR_CYAN);

        let prompt = b"Press A to Continue";
        draw_text(prompt.as_ptr(), prompt.len() as u32, 350.0, 480.0, 18.0, 0x888888FF);
    }
}

fn render_paused() {
    unsafe {
        // First render the racing view
        render_racing_view();

        // Then overlay pause menu
        viewport_clear();
        draw_rect(300.0, 180.0, 360.0, 180.0, 0x111122EE);

        let title = b"PAUSED";
        draw_text(title.as_ptr(), title.len() as u32, 420.0, 200.0, 32.0, COLOR_CYAN);

        let resume = b"START: Resume";
        draw_text(resume.as_ptr(), resume.len() as u32, 360.0, 280.0, 18.0, COLOR_WHITE);

        let quit = b"SELECT: Quit";
        draw_text(quit.as_ptr(), quit.len() as u32, 360.0, 320.0, 18.0, COLOR_WHITE);
    }
}

fn render_racing_view() {
    // Renders the 3D world without HUD - used by countdown and paused overlays
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

            // Render track
            render_track();

            // Render cars
            render_all_cars();
        }

        // Reset viewport
        viewport_clear();
    }
}

fn render_track() {
    unsafe {
        // Load track textures (we should load these in init, but for now use IDs)
        let tex_straight = load_rom_texture(b"track_straight");
        let tex_curve = load_rom_texture(b"track_curve_left");

        // Track is a series of straight segments along Z axis
        // Each segment is 10 units long
        let segment_length = 10.0;
        let num_segments = (TRACK_LENGTH / segment_length) as i32;

        // Use metallic material for track
        material_metallic(0.3);
        material_roughness(0.7);
        material_emissive(0.5);

        for i in 0..num_segments {
            let z_pos = (i as f32) * segment_length;

            // Render track segment
            push_identity();
            push_translate(0.0, 0.0, z_pos);

            // Alternate between straight and slight curves for variety
            if i % 4 == 2 {
                material_albedo(tex_curve);
                draw_mesh(MESH_TRACK_CURVE_LEFT);
            } else {
                material_albedo(tex_straight);
                draw_mesh(MESH_TRACK_STRAIGHT);
            }
        }

        // Render props along the track
        render_track_props();
    }
}

fn render_track_props() {
    unsafe {
        let tex_building = load_rom_texture(b"prop_building");
        let tex_barrier = load_rom_texture(b"prop_barrier");
        let tex_billboard = load_rom_texture(b"prop_billboard");
        let tex_boost = load_rom_texture(b"prop_boost_pad");

        material_metallic(0.5);
        material_roughness(0.5);

        // Buildings along the sides
        let num_buildings = 10;
        for i in 0..num_buildings {
            let z_pos = (i as f32) * 20.0;

            // Left side buildings
            material_albedo(tex_building);
            material_emissive(1.5);
            push_identity();
            push_translate(-15.0, 0.0, z_pos);
            push_scale(1.5, 2.0 + (i % 3) as f32, 1.5);
            draw_mesh(MESH_PROP_BUILDING);

            // Right side buildings
            push_identity();
            push_translate(15.0, 0.0, z_pos + 10.0);
            push_scale(1.2, 1.5 + (i % 2) as f32, 1.2);
            draw_mesh(MESH_PROP_BUILDING);
        }

        // Barriers along track edges
        material_albedo(tex_barrier);
        material_emissive(2.0);
        for i in 0..20 {
            let z_pos = (i as f32) * 10.0;

            // Left barrier
            push_identity();
            push_translate(-5.5, 0.0, z_pos);
            draw_mesh(MESH_PROP_BARRIER);

            // Right barrier
            push_identity();
            push_translate(5.5, 0.0, z_pos);
            push_rotate_y(180.0);
            draw_mesh(MESH_PROP_BARRIER);
        }

        // Billboards at intervals
        material_albedo(tex_billboard);
        material_emissive(3.0);
        for i in 0..4 {
            let z_pos = (i as f32) * 50.0 + 25.0;

            push_identity();
            push_translate(-12.0, 8.0, z_pos);
            push_rotate_y(-15.0);
            draw_mesh(MESH_PROP_BILLBOARD);
        }

        // Boost pads on track
        material_albedo(tex_boost);
        material_emissive(4.0);
        for i in 0..5 {
            let z_pos = (i as f32) * 40.0 + 30.0;
            let x_pos = if i % 2 == 0 { -2.0 } else { 2.0 };

            push_identity();
            push_translate(x_pos, 0.1, z_pos);
            draw_mesh(MESH_PROP_BOOST_PAD);
        }
    }
}

fn render_racing() {
    unsafe {
        let viewports = get_viewport_layout(ACTIVE_PLAYER_COUNT);

        for player_id in 0..ACTIVE_PLAYER_COUNT as usize {
            let (vp_x, vp_y, vp_w, vp_h) = viewports[player_id];

            // Set viewport
            viewport(vp_x, vp_y, vp_w, vp_h);

            // Setup camera with screen shake applied
            let camera = &CAMERAS[player_id];
            let shake_x = camera.shake_offset_x;
            let shake_y = camera.shake_offset_y;
            camera_set(
                camera.current_pos_x + shake_x,
                camera.current_pos_y + shake_y,
                camera.current_pos_z,
                camera.current_target_x + shake_x,
                camera.current_target_y + shake_y,
                camera.current_target_z
            );
            camera_fov(75.0);

            // Setup environment
            setup_environment(SELECTED_TRACK);
            draw_env();

            // Render track
            render_track();

            // Render cars
            render_all_cars();

            // Render particles (3D billboard effects)
            render_particles();

            // 2D overlays (speed lines when going fast)
            render_speed_lines(player_id, vp_w, vp_h);

            // Vignette overlay when boosting
            if BOOST_GLOW_INTENSITY[player_id] > 0.1 {
                render_vignette(BOOST_GLOW_INTENSITY[player_id]);
            }

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
            TrackId::CrystalCavern => {
                // Deep underground purple gradient
                env_gradient(0, 0x1A0533FF, 0x2D1B4EFF, 0x0D0221FF, 0x000000FF, 0.0, 0.0);
                // Crystal scatter (diamond shapes, high parallax)
                env_scatter(1, 2, 180, 8, 200, 5, 0x00FFFFFF, 0xFF00FFFF, 250, 180, SPEED_PHASE);
                // Stalactite lines (vertical, dotted)
                env_lines(2, 1, 1, 2, 3.0, 120.0, 0x8B5CF6FF, 0x00FFFFFF, 4, GRID_PHASE);
                env_blend(1); // Additive
            }
            TrackId::SolarHighway => {
                // Hot solar gradient (tilted sun position)
                env_gradient(0, 0xFFFFFFFF, 0xFFAA00FF, 0xFF4400FF, 0x330000FF, 0.3, 0.0);
                // Solar flare scatter (round particles with motion blur)
                env_scatter(1, 0, 100, 12, 255, 30, 0xFFFF00FF, 0xFF8800FF, 180, 200, SPEED_PHASE);
                // Corona rings (centered with spiral)
                env_rings(2, 30, 5, 0xFFAA00FF, 0xFFFFAAFF, 0xFFFFFFFF, 180, 5.0, 0.0, 0.2, 1.0, RING_PHASE);
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
                CarType::Phantom => (MESH_PHANTOM, TEX_PHANTOM, TEX_PHANTOM_EMISSIVE),
                CarType::Titan => (MESH_TITAN, TEX_TITAN, TEX_TITAN_EMISSIVE),
                CarType::Viper => (MESH_VIPER, TEX_VIPER, TEX_VIPER_EMISSIVE),
            };

            // Setup materials (Mode 2: Metallic-Roughness PBR)
            material_albedo(tex_albedo);
            // Note: emissive texture stored in tex_emissive but using intensity only for now
            let _ = tex_emissive; // Suppress unused warning
            material_metallic(0.9);
            material_roughness(0.1);
            material_emissive(2.0); // Neon glow intensity

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

        // === TOP-LEFT: Position ===
        let pos_text = match car.race_position {
            1 => b"1ST" as &[u8],
            2 => b"2ND",
            3 => b"3RD",
            _ => b"4TH",
        };
        let pos_colors = [0xFFD700FF, 0xC0C0C0FF, 0xCD7F32FF, COLOR_WHITE];
        let pos_color = pos_colors[(car.race_position - 1).min(3) as usize];
        draw_text(pos_text.as_ptr(), 3, 10.0, 10.0, 28.0, pos_color);

        // === TOP-RIGHT: Lap counter ===
        let lap_label = b"LAP";
        let lap_x = vp_width as f32 - 100.0;
        draw_text(lap_label.as_ptr(), 3, lap_x, 10.0, 16.0, 0x888888FF);

        // Lap numbers
        let current_lap = car.current_lap.min(3);
        let lap_str = [b'0' + current_lap as u8, b'/', b'3'];
        draw_text(lap_str.as_ptr(), 3, lap_x, 30.0, 24.0, COLOR_CYAN);

        // === TOP-CENTER: Race Time ===
        let mins = (RACE_TIME / 60.0) as u32;
        let secs = (RACE_TIME % 60.0) as u32;
        let centis = ((RACE_TIME * 100.0) as u32) % 100;

        let center_x = (vp_width as f32 / 2.0) - 40.0;
        let time_str = [
            b'0' + ((mins / 10) % 10) as u8,
            b'0' + (mins % 10) as u8,
            b':',
            b'0' + ((secs / 10) % 10) as u8,
            b'0' + (secs % 10) as u8,
            b'.',
            b'0' + ((centis / 10) % 10) as u8,
            b'0' + (centis % 10) as u8,
        ];
        draw_text(time_str.as_ptr(), 8, center_x, 10.0, 20.0, COLOR_WHITE);

        // === BOTTOM-LEFT: Speed ===
        let speed = (car.velocity_forward.abs() * 10.0) as u32;
        let mut speed_str = [0u8; 16];
        let speed_len = format_number(speed, &mut speed_str);
        draw_text(speed_str.as_ptr(), speed_len, 10.0, vp_height as f32 - 40.0, 32.0, COLOR_WHITE);

        let kmh = b"KM/H";
        draw_text(kmh.as_ptr(), 4, 10.0 + (speed_len as f32) * 18.0 + 5.0, vp_height as f32 - 35.0, 14.0, 0x888888FF);

        // === BOTTOM-LEFT: Boost meter ===
        let meter_w = 120.0;
        let meter_h = 12.0;
        let meter_x = 10.0;
        let meter_y = vp_height as f32 - 70.0;

        // Background
        draw_rect(meter_x, meter_y, meter_w, meter_h, 0x222222FF);

        // Fill
        let fill_w = meter_w * car.boost_meter;
        let boost_color = if car.boost_meter >= BOOST_COST {
            if car.is_boosting { COLOR_CYAN } else { 0x00AAFFFF }
        } else {
            0x0066AAFF
        };
        draw_rect(meter_x, meter_y, fill_w, meter_h, boost_color);

        // Border
        draw_rect(meter_x, meter_y, meter_w, 2.0, 0x444444FF);
        draw_rect(meter_x, meter_y + meter_h - 2.0, meter_w, 2.0, 0x444444FF);

        // Boost label
        let boost_label = b"BOOST";
        draw_text(boost_label.as_ptr(), 5, meter_x + meter_w + 8.0, meter_y - 1.0, 12.0, 0x888888FF);

        // Boosting indicator
        if car.is_boosting {
            let boosting = b"BOOSTING!";
            draw_text(boosting.as_ptr(), 9, meter_x, meter_y - 20.0, 16.0, COLOR_CYAN);
        }

        // Drift indicator
        if car.is_drifting {
            let drifting = b"DRIFT!";
            draw_text(drifting.as_ptr(), 6, vp_width as f32 - 80.0, vp_height as f32 - 40.0, 18.0, COLOR_MAGENTA);
        }
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
