//! LUMINA DEPTHS - ZX Console Showcase
//!
//! A meditative underwater exploration game demonstrating:
//! - Render Mode 3 (Specular-Shininess Blinn-Phong)
//! - Alpha transparency (jellyfish, glass canopy, depth fog)
//! - EPU environmental rendering (5 depth zones)
//! - Creature AI (flocking + curiosity)
//! - Bioluminescent materials and dynamic lighting
//!
//! Controls:
//! - Left Stick: Pitch (up/down) and Yaw (left/right)
//! - Right Stick: Camera look
//! - RT: Descend faster
//! - LT: Ascend / slow descent
//! - A Button: Headlight pulse (attracts creatures)
//! - B Button: Toggle headlight on/off
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

    // Input
    fn left_stick_x(player: u32) -> f32;
    fn left_stick_y(player: u32) -> f32;
    fn right_stick_x(player: u32) -> f32;
    fn right_stick_y(player: u32) -> f32;
    fn trigger_left(player: u32) -> f32;
    fn trigger_right(player: u32) -> f32;
    fn button_pressed(player: u32, button: u32) -> u32;

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

    // Materials (Mode 3: Specular-Shininess)
    fn material_albedo(texture_handle: u32);
    fn material_specular(color: u32);
    fn material_shininess(value: f32);
    fn material_emissive(intensity: f32);
    fn material_rim(intensity: f32, power: f32);

    // Transparency & Dithering
    fn uniform_alpha(level: u32);
    fn dither_offset(x: u32, y: u32);

    // Stencil Buffer (masked rendering)
    fn stencil_begin();
    fn stencil_end();
    fn stencil_clear();
    fn stencil_invert();

    // Billboards
    fn draw_billboard(w: f32, h: f32, mode: u32, color: u32);

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
    fn env_scatter(layer: u32, variant: u32, density: u32, size: u32, glow: u32, streak_length: u32, color_primary: u32, color_secondary: u32, parallax_rate: u32, parallax_size: u32, phase: u32);
    fn env_curtains(layer: u32, layer_count: u32, density: u32, height_min: u32, height_max: u32, width: u32, spacing: u32, waviness: u32, color_near: u32, color_far: u32, glow: u32, parallax_rate: u32, phase: u32);
    fn env_silhouette(layer: u32, jaggedness: u32, layer_count: u32, color_near: u32, color_far: u32, sky_zenith: u32, sky_horizon: u32, parallax_rate: u32, seed: u32);
    fn env_rings(layer: u32, ring_count: u32, thickness: u32, color_a: u32, color_b: u32, center_color: u32, center_falloff: u32, spiral_twist: f32, axis_x: f32, axis_y: f32, axis_z: f32, phase: u32);
    fn env_blend(mode: u32);
    fn draw_env();

    // 2D Drawing
    fn draw_rect(x: f32, y: f32, w: f32, h: f32, color: u32);
    fn draw_text(ptr: *const u8, len: u32, x: f32, y: f32, size: f32, color: u32);

    // Custom Font
    fn load_font(texture: u32, char_width: u32, char_height: u32, first_codepoint: u32, char_count: u32) -> u32;
    fn font_bind(font_handle: u32);

    // Audio
    fn play_sound(sound: u32, volume: f32, pan: f32);
    fn channel_play(channel: u32, sound: u32, volume: f32, pan: f32, looping: u32);
    fn channel_stop(channel: u32);

    // System
    fn random() -> u32;
    fn delta_time() -> f32;
}

// === Constants ===

// Button constants
const BUTTON_A: u32 = 4;
const BUTTON_B: u32 = 5;
const BUTTON_START: u32 = 11;

// Zone depth thresholds (meters)
const ZONE_SUNLIT_MAX: f32 = 200.0;
const ZONE_TWILIGHT_MAX: f32 = 1000.0;
const ZONE_MIDNIGHT_MAX: f32 = 4000.0;
const ZONE_VENTS_MAX: f32 = 5000.0;

// Submersible physics
const DESCENT_SPEED: f32 = 10.0;
const THRUST_DOWN_MULT: f32 = 2.5;
const THRUST_UP_MULT: f32 = 0.3;
const PITCH_SPEED: f32 = 30.0;
const YAW_SPEED: f32 = 45.0;

// Creature constants
const MAX_SCHOOL_FISH: usize = 40;
const MAX_LARGE_CREATURES: usize = 12;
const MAX_FLORA: usize = 20;
const SEPARATION_DIST: f32 = 1.5;
const SCATTER_DIST: f32 = 8.0;
const SCHOOL_SPEED: f32 = 4.0;

// Whale encounter (Blue Whale - Sunlit Zone)
const WHALE_TRIGGER_DEPTH: f32 = 180.0;
const WHALE_END_DEPTH: f32 = 250.0;

// Sperm Whale encounter (Midnight Zone - hunting giant squid)
const SPERM_WHALE_TRIGGER_DEPTH: f32 = 2500.0;
const SPERM_WHALE_END_DEPTH: f32 = 2700.0;

// Giant Isopod encounter (Vents Zone - taps on glass)
const ISOPOD_TRIGGER_DEPTH: f32 = 4200.0;
const ISOPOD_END_DEPTH: f32 = 4400.0;

// Colors
const COLOR_WHITE: u32 = 0xFFFFFFFF;
const COLOR_CYAN: u32 = 0x00FFFFFF;
const COLOR_DEPTH_TEXT: u32 = 0xAADDFFFF;

// God ray constants (Sunlit zone light shafts)
const GOD_RAY_COUNT: usize = 8;
const GOD_RAY_WIDTH: f32 = 4.0;
const GOD_RAY_HEIGHT: f32 = 40.0;
const GOD_RAY_SPACING: f32 = 12.0;

// Thermal shimmer constants (Vent zone)
const SHIMMER_COLUMNS: usize = 5;
const SHIMMER_WIDTH: f32 = 2.0;
const SHIMMER_HEIGHT: f32 = 15.0;

// Billboard modes
const BILLBOARD_CYLINDRICAL_Y: u32 = 2;

// === Types ===

#[derive(Clone, Copy, PartialEq)]
#[repr(u8)]
enum GameMode {
    Descending,
    Paused,
    Ending,
}

#[derive(Clone, Copy, PartialEq)]
#[repr(u8)]
enum Zone {
    Sunlit,
    Twilight,
    Midnight,
    Vents,
    Luminara,
}

#[derive(Clone, Copy, PartialEq)]
#[repr(u8)]
enum CreatureType {
    ReefFish,
    SeaTurtle,
    MantaRay,
    CoralCrab,
    MoonJelly,
    Lanternfish,
    Siphonophore,
    GiantSquid,
    Anglerfish,
    GulperEel,
    DumboOctopus,
    VampireSquid,
    TubeWorms,
    VentShrimp,
    GhostFish,
    VentOctopus,
}

#[derive(Clone, Copy, PartialEq)]
#[repr(u8)]
enum FloraType {
    CoralBrain,
    CoralFan,
    CoralBranch,
    Kelp,
    Anemone,
    SeaGrass,
}

#[derive(Clone, Copy)]
struct Submersible {
    x: f32,
    y: f32,
    z: f32,
    pitch: f32,
    yaw: f32,
    velocity_y: f32,
    headlight_on: bool,
    headlight_pulse: u32,
}

#[derive(Clone, Copy)]
struct SchoolFish {
    x: f32,
    y: f32,
    z: f32,
    vx: f32,
    vy: f32,
    vz: f32,
    variant: u8,
    active: bool,
}

#[derive(Clone, Copy)]
struct Creature {
    creature_type: CreatureType,
    x: f32,
    y: f32,
    z: f32,
    rotation_y: f32,
    velocity_x: f32,
    velocity_y: f32,
    velocity_z: f32,
    animation_phase: f32,
    active: bool,
    curiosity_target: bool,
}

#[derive(Clone, Copy)]
struct Flora {
    flora_type: FloraType,
    x: f32,
    y: f32,
    z: f32,
    rotation_y: f32,
    scale: f32,
    active: bool,
}

// Bubble particle for trail effect
const MAX_BUBBLES: usize = 30;
const BUBBLE_SPAWN_RATE: u32 = 3; // Frames between spawns
const BUBBLE_RISE_SPEED: f32 = 2.0;
const BUBBLE_LIFETIME: f32 = 4.0;

#[derive(Clone, Copy)]
struct Bubble {
    x: f32,
    y: f32,
    z: f32,
    scale: f32,
    lifetime: f32,
    active: bool,
}

impl Bubble {
    const fn new() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            z: 0.0,
            scale: 0.2,
            lifetime: 0.0,
            active: false,
        }
    }
}

impl Submersible {
    const fn new() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            z: 0.0,
            pitch: 0.0,
            yaw: 0.0,
            velocity_y: 0.0,
            headlight_on: true,
            headlight_pulse: 0,
        }
    }
}

impl SchoolFish {
    const fn new() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            z: 0.0,
            vx: 0.0,
            vy: 0.0,
            vz: 0.0,
            variant: 0,
            active: false,
        }
    }
}

impl Creature {
    const fn new() -> Self {
        Self {
            creature_type: CreatureType::SeaTurtle,
            x: 0.0,
            y: 0.0,
            z: 0.0,
            rotation_y: 0.0,
            velocity_x: 0.0,
            velocity_y: 0.0,
            velocity_z: 0.0,
            animation_phase: 0.0,
            active: false,
            curiosity_target: false,
        }
    }
}

impl Flora {
    const fn new() -> Self {
        Self {
            flora_type: FloraType::CoralBrain,
            x: 0.0,
            y: 0.0,
            z: 0.0,
            rotation_y: 0.0,
            scale: 1.0,
            active: false,
        }
    }
}

// === Static Game State ===

static mut GAME_MODE: GameMode = GameMode::Descending;
static mut CURRENT_ZONE: Zone = Zone::Sunlit;
static mut PREVIOUS_ZONE: Zone = Zone::Sunlit;
static mut DEPTH: f32 = 0.0;
static mut PHASE: u32 = 0;
static mut ELAPSED_TIME: f32 = 0.0;

static mut SUB: Submersible = Submersible::new();
static mut SCHOOL: [SchoolFish; MAX_SCHOOL_FISH] = [SchoolFish::new(); MAX_SCHOOL_FISH];
static mut CREATURES: [Creature; MAX_LARGE_CREATURES] = [Creature::new(); MAX_LARGE_CREATURES];
static mut FLORA_LIST: [Flora; MAX_FLORA] = [Flora::new(); MAX_FLORA];

// Bubble trail particles
static mut BUBBLES: [Bubble; MAX_BUBBLES] = [Bubble::new(); MAX_BUBBLES];
static mut BUBBLE_SPAWN_TIMER: u32 = 0;

// Creature discovery system
static mut DISCOVERED_CREATURES: u16 = 0; // Bitmask for 11 creature types
static mut DISCOVERY_POPUP_TEXT: [u8; 32] = [0; 32];
static mut DISCOVERY_POPUP_LEN: usize = 0;
static mut DISCOVERY_POPUP_TIMER: f32 = 0.0;
const DISCOVERY_POPUP_DURATION: f32 = 3.0;
const DISCOVERY_DISTANCE: f32 = 15.0;

// Whale encounter state (Blue Whale)
static mut WHALE_ACTIVE: bool = false;
static mut WHALE_PROGRESS: f32 = 0.0;
static mut WHALE_TRIGGERED: bool = false;

// Sperm Whale encounter state
static mut SPERM_WHALE_ACTIVE: bool = false;
static mut SPERM_WHALE_PROGRESS: f32 = 0.0;
static mut SPERM_WHALE_TRIGGERED: bool = false;

// Giant Isopod encounter state
static mut ISOPOD_ACTIVE: bool = false;
static mut ISOPOD_PROGRESS: f32 = 0.0;
static mut ISOPOD_TRIGGERED: bool = false;

// Ending state
static mut ENDING_FADE: f32 = 0.0;

// God ray state (randomized positions in Sunlit zone)
static mut GOD_RAY_OFFSETS: [(f32, f32); GOD_RAY_COUNT] = [(0.0, 0.0); GOD_RAY_COUNT];
static mut GOD_RAYS_INITIALIZED: bool = false;

// Shimmer column state (for thermal vents)
static mut SHIMMER_POSITIONS: [(f32, f32); SHIMMER_COLUMNS] = [(0.0, 0.0); SHIMMER_COLUMNS];
static mut SHIMMER_INITIALIZED: bool = false;

// Camera state - orbit camera around submersible
static mut CAM_X: f32 = 0.0;
static mut CAM_Y: f32 = 5.0;
static mut CAM_Z: f32 = -15.0;
// Orbit angles (controlled by right stick)
static mut CAM_ORBIT_YAW: f32 = 0.0;   // Horizontal orbit angle offset
static mut CAM_ORBIT_PITCH: f32 = 0.0; // Vertical orbit angle

// === Asset Handles ===

// Meshes
static mut MESH_SUBMERSIBLE: u32 = 0;
static mut MESH_REEF_FISH: u32 = 0;
static mut MESH_SEA_TURTLE: u32 = 0;
static mut MESH_MANTA_RAY: u32 = 0;
static mut MESH_MOON_JELLY: u32 = 0;
static mut MESH_LANTERNFISH: u32 = 0;
static mut MESH_SIPHONOPHORE: u32 = 0;
static mut MESH_ANGLERFISH: u32 = 0;
static mut MESH_GULPER_EEL: u32 = 0;
static mut MESH_DUMBO_OCTOPUS: u32 = 0;
static mut MESH_TUBE_WORMS: u32 = 0;
static mut MESH_VENT_SHRIMP: u32 = 0;
static mut MESH_GHOST_FISH: u32 = 0;
static mut MESH_VENT_OCTOPUS: u32 = 0;
static mut MESH_BLUE_WHALE: u32 = 0;
static mut MESH_SPERM_WHALE: u32 = 0;
static mut MESH_GIANT_ISOPOD: u32 = 0;
static mut MESH_CORAL_CRAB: u32 = 0;
static mut MESH_GIANT_SQUID: u32 = 0;
static mut MESH_VAMPIRE_SQUID: u32 = 0;
static mut MESH_CORAL_BRAIN: u32 = 0;
static mut MESH_CORAL_FAN: u32 = 0;
static mut MESH_CORAL_BRANCH: u32 = 0;
static mut MESH_KELP: u32 = 0;
static mut MESH_ANEMONE: u32 = 0;
static mut MESH_SEA_GRASS: u32 = 0;
static mut MESH_ROCK_BOULDER: u32 = 0;
static mut MESH_BUBBLE_CLUSTER: u32 = 0;

// Textures
static mut TEX_SUBMERSIBLE: u32 = 0;
static mut TEX_REEF_FISH: u32 = 0;
static mut TEX_SEA_TURTLE: u32 = 0;
static mut TEX_MOON_JELLY: u32 = 0;
static mut TEX_ANGLERFISH: u32 = 0;
static mut TEX_BLUE_WHALE: u32 = 0;
static mut TEX_CORAL_CRAB: u32 = 0;
static mut TEX_GIANT_SQUID: u32 = 0;
static mut TEX_VAMPIRE_SQUID: u32 = 0;
static mut TEX_GHOST_FISH: u32 = 0;
static mut TEX_VENT_OCTOPUS: u32 = 0;
static mut TEX_SPERM_WHALE: u32 = 0;
static mut TEX_GIANT_ISOPOD: u32 = 0;
static mut TEX_CORAL_BRAIN: u32 = 0;
static mut TEX_KELP: u32 = 0;

// Sounds
static mut SND_PROPELLER: u32 = 0;
static mut SND_SONAR: u32 = 0;
static mut SND_BUBBLES: u32 = 0;
static mut SND_WHALE: u32 = 0;
static mut SND_JELLYFISH: u32 = 0;
static mut SND_VENT: u32 = 0;

// Font
static mut TEX_FONT: u32 = 0;
static mut FONT_HANDLE: u32 = 0;

// === Helper Functions ===

fn load_mesh(id: &[u8]) -> u32 {
    unsafe { rom_mesh(id.as_ptr(), id.len() as u32) }
}

fn load_texture(id: &[u8]) -> u32 {
    unsafe { rom_texture(id.as_ptr(), id.len() as u32) }
}

fn load_sound(id: &[u8]) -> u32 {
    unsafe { rom_sound(id.as_ptr(), id.len() as u32) }
}

fn depth_to_zone(depth: f32) -> Zone {
    if depth < ZONE_SUNLIT_MAX {
        Zone::Sunlit
    } else if depth < ZONE_TWILIGHT_MAX {
        Zone::Twilight
    } else if depth < ZONE_MIDNIGHT_MAX {
        Zone::Midnight
    } else if depth < ZONE_VENTS_MAX {
        Zone::Vents
    } else {
        Zone::Luminara
    }
}

fn random_f32() -> f32 {
    unsafe { (random() as f32) / (u32::MAX as f32) }
}

fn random_range(min: f32, max: f32) -> f32 {
    min + random_f32() * (max - min)
}

fn sin_approx(x: f32) -> f32 {
    // Fast sine approximation
    let x = x % (2.0 * 3.14159);
    let x = if x > 3.14159 { x - 2.0 * 3.14159 } else { x };
    let x2 = x * x;
    x * (1.0 - x2 / 6.0 + x2 * x2 / 120.0)
}

fn cos_approx(x: f32) -> f32 {
    sin_approx(x + 1.5708)
}

fn sqrt_approx(x: f32) -> f32 {
    if x <= 0.0 { return 0.0; }
    let mut guess = x / 2.0;
    for _ in 0..5 {
        guess = (guess + x / guess) / 2.0;
    }
    guess
}

fn distance_3d(x1: f32, y1: f32, z1: f32, x2: f32, y2: f32, z2: f32) -> f32 {
    let dx = x2 - x1;
    let dy = y2 - y1;
    let dz = z2 - z1;
    sqrt_approx(dx * dx + dy * dy + dz * dz)
}

fn atan2_approx(y: f32, x: f32) -> f32 {
    // Fast atan2 approximation
    let abs_y = y.abs() + 0.0001; // Prevent division by zero
    let angle = if x >= 0.0 {
        let r = (x - abs_y) / (x + abs_y);
        0.7854 - 0.7854 * r
    } else {
        let r = (x + abs_y) / (abs_y - x);
        2.3562 - 0.7854 * r
    };
    if y < 0.0 { -angle } else { angle }
}

// === Initialization ===

#[no_mangle]
pub extern "C" fn init() {
    unsafe {
        // Configure renderer
        set_clear_color(0x001828FF);
        render_mode(3); // Mode 3: Specular-Shininess
        depth_test(1);
        set_tick_rate(2); // 60 FPS

        // Load meshes
        MESH_SUBMERSIBLE = load_mesh(b"submersible");
        MESH_REEF_FISH = load_mesh(b"reef_fish");
        MESH_SEA_TURTLE = load_mesh(b"sea_turtle");
        MESH_MANTA_RAY = load_mesh(b"manta_ray");
        MESH_MOON_JELLY = load_mesh(b"moon_jelly");
        MESH_LANTERNFISH = load_mesh(b"lanternfish");
        MESH_SIPHONOPHORE = load_mesh(b"siphonophore");
        MESH_ANGLERFISH = load_mesh(b"anglerfish");
        MESH_GULPER_EEL = load_mesh(b"gulper_eel");
        MESH_DUMBO_OCTOPUS = load_mesh(b"dumbo_octopus");
        MESH_TUBE_WORMS = load_mesh(b"tube_worms");
        MESH_VENT_SHRIMP = load_mesh(b"vent_shrimp");
        MESH_GHOST_FISH = load_mesh(b"ghost_fish");
        MESH_VENT_OCTOPUS = load_mesh(b"vent_octopus");
        MESH_BLUE_WHALE = load_mesh(b"blue_whale");
        MESH_SPERM_WHALE = load_mesh(b"sperm_whale");
        MESH_GIANT_ISOPOD = load_mesh(b"giant_isopod");
        MESH_CORAL_CRAB = load_mesh(b"coral_crab");
        MESH_GIANT_SQUID = load_mesh(b"giant_squid");
        MESH_VAMPIRE_SQUID = load_mesh(b"vampire_squid");
        MESH_CORAL_BRAIN = load_mesh(b"coral_brain");
        MESH_CORAL_FAN = load_mesh(b"coral_fan");
        MESH_CORAL_BRANCH = load_mesh(b"coral_branch");
        MESH_KELP = load_mesh(b"kelp");
        MESH_ANEMONE = load_mesh(b"anemone");
        MESH_SEA_GRASS = load_mesh(b"sea_grass");
        MESH_ROCK_BOULDER = load_mesh(b"rock_boulder");
        MESH_BUBBLE_CLUSTER = load_mesh(b"bubble_cluster");

        // Load textures
        TEX_SUBMERSIBLE = load_texture(b"submersible");
        TEX_REEF_FISH = load_texture(b"reef_fish");
        TEX_SEA_TURTLE = load_texture(b"sea_turtle");
        TEX_MOON_JELLY = load_texture(b"moon_jelly");
        TEX_ANGLERFISH = load_texture(b"anglerfish");
        TEX_BLUE_WHALE = load_texture(b"blue_whale");
        TEX_CORAL_CRAB = load_texture(b"coral_crab");
        TEX_GIANT_SQUID = load_texture(b"giant_squid");
        TEX_VAMPIRE_SQUID = load_texture(b"vampire_squid");
        TEX_GHOST_FISH = load_texture(b"ghost_fish");
        TEX_VENT_OCTOPUS = load_texture(b"vent_octopus");
        TEX_SPERM_WHALE = load_texture(b"sperm_whale");
        TEX_GIANT_ISOPOD = load_texture(b"giant_isopod");
        TEX_CORAL_BRAIN = load_texture(b"coral_brain");
        TEX_KELP = load_texture(b"kelp");

        // Load sounds
        SND_PROPELLER = load_sound(b"propeller");
        SND_SONAR = load_sound(b"sonar");
        SND_BUBBLES = load_sound(b"bubbles");
        SND_WHALE = load_sound(b"whale");
        SND_JELLYFISH = load_sound(b"jellyfish");
        SND_VENT = load_sound(b"vent");

        // Load custom underwater font (6x8 glyphs, 96 chars starting at ASCII 32)
        TEX_FONT = load_texture(b"lumina_font");
        FONT_HANDLE = load_font(TEX_FONT, 6, 8, 32, 96);

        // Initialize submersible
        SUB.velocity_y = -DESCENT_SPEED;
        SUB.headlight_on = true;

        // Initialize camera orbit (start looking slightly down at submersible)
        CAM_ORBIT_YAW = 0.0;
        CAM_ORBIT_PITCH = 15.0; // Default: slightly above and behind

        // Start ambient propeller sound
        channel_play(0, SND_PROPELLER, 0.3, 0.0, 1); // Loop propeller sound

        // Spawn initial creatures for Zone 1
        spawn_zone_creatures(Zone::Sunlit);
        spawn_zone_flora(Zone::Sunlit);
    }
}

// === Creature Spawning ===

fn spawn_zone_creatures(zone: Zone) {
    unsafe {
        // Deactivate all creatures
        for fish in SCHOOL.iter_mut() {
            fish.active = false;
        }
        for creature in CREATURES.iter_mut() {
            creature.active = false;
        }

        let sub_y = SUB.y;

        // Initialize zone-specific effects
        match zone {
            Zone::Sunlit => {
                // Initialize god ray positions (random offsets around player)
                if !GOD_RAYS_INITIALIZED {
                    for i in 0..GOD_RAY_COUNT {
                        GOD_RAY_OFFSETS[i] = (
                            random_range(-40.0, 40.0),
                            random_range(-40.0, 40.0),
                        );
                    }
                    GOD_RAYS_INITIALIZED = true;
                }
            }
            Zone::Vents => {
                // Initialize thermal shimmer column positions
                if !SHIMMER_INITIALIZED {
                    for i in 0..SHIMMER_COLUMNS {
                        SHIMMER_POSITIONS[i] = (
                            random_range(-20.0, 20.0),
                            random_range(-20.0, 20.0),
                        );
                    }
                    SHIMMER_INITIALIZED = true;
                }
            }
            _ => {}
        }

        match zone {
            Zone::Sunlit => {
                // Spawn 30 reef fish
                for i in 0..30 {
                    let fish = &mut SCHOOL[i];
                    fish.active = true;
                    fish.x = random_range(-30.0, 30.0);
                    fish.y = sub_y + random_range(-10.0, 10.0);
                    fish.z = random_range(-30.0, 30.0);
                    fish.vx = random_range(-1.0, 1.0);
                    fish.vy = random_range(-0.5, 0.5);
                    fish.vz = random_range(-1.0, 1.0);
                    fish.variant = (i % 4) as u8;
                }

                // Spawn sea turtle
                CREATURES[0].active = true;
                CREATURES[0].creature_type = CreatureType::SeaTurtle;
                CREATURES[0].x = 15.0;
                CREATURES[0].y = sub_y - 5.0;
                CREATURES[0].z = 20.0;
                CREATURES[0].rotation_y = random_range(0.0, 360.0);

                // Spawn manta ray
                CREATURES[1].active = true;
                CREATURES[1].creature_type = CreatureType::MantaRay;
                CREATURES[1].x = -20.0;
                CREATURES[1].y = sub_y - 8.0;
                CREATURES[1].z = 15.0;

                // Spawn coral crab
                CREATURES[2].active = true;
                CREATURES[2].creature_type = CreatureType::CoralCrab;
                CREATURES[2].x = 8.0;
                CREATURES[2].y = sub_y - 18.0;
                CREATURES[2].z = -10.0;
            }
            Zone::Twilight => {
                // Spawn moon jellies
                for i in 0..8 {
                    let creature = &mut CREATURES[i];
                    creature.active = true;
                    creature.creature_type = CreatureType::MoonJelly;
                    creature.x = random_range(-25.0, 25.0);
                    creature.y = sub_y + random_range(-15.0, 15.0);
                    creature.z = random_range(-25.0, 25.0);
                    creature.animation_phase = random_range(0.0, 6.28);
                }

                // Spawn siphonophore
                CREATURES[8].active = true;
                CREATURES[8].creature_type = CreatureType::Siphonophore;
                CREATURES[8].x = 20.0;
                CREATURES[8].y = sub_y;
                CREATURES[8].z = 25.0;
                CREATURES[8].animation_phase = random_range(0.0, 6.28);

                // Spawn giant squid (distant silhouette)
                CREATURES[9].active = true;
                CREATURES[9].creature_type = CreatureType::GiantSquid;
                CREATURES[9].x = 40.0;
                CREATURES[9].y = sub_y - 20.0;
                CREATURES[9].z = 35.0;
            }
            Zone::Midnight => {
                // Spawn anglerfish
                CREATURES[0].active = true;
                CREATURES[0].creature_type = CreatureType::Anglerfish;
                CREATURES[0].x = 10.0;
                CREATURES[0].y = sub_y - 20.0;
                CREATURES[0].z = 15.0;

                // Spawn gulper eel
                CREATURES[1].active = true;
                CREATURES[1].creature_type = CreatureType::GulperEel;
                CREATURES[1].x = -15.0;
                CREATURES[1].y = sub_y - 15.0;
                CREATURES[1].z = -20.0;

                // Spawn dumbo octopus
                CREATURES[2].active = true;
                CREATURES[2].creature_type = CreatureType::DumboOctopus;
                CREATURES[2].x = 5.0;
                CREATURES[2].y = sub_y - 10.0;
                CREATURES[2].z = 25.0;
                CREATURES[2].animation_phase = random_range(0.0, 6.28);

                // Spawn vampire squid
                CREATURES[3].active = true;
                CREATURES[3].creature_type = CreatureType::VampireSquid;
                CREATURES[3].x = -20.0;
                CREATURES[3].y = sub_y - 25.0;
                CREATURES[3].z = 10.0;
                CREATURES[3].animation_phase = random_range(0.0, 6.28);

                // Spawn lanternfish school
                for i in 0..15 {
                    let fish = &mut SCHOOL[i];
                    fish.active = true;
                    fish.x = random_range(-20.0, 20.0);
                    fish.y = sub_y + random_range(-10.0, 10.0);
                    fish.z = random_range(-20.0, 20.0);
                    fish.variant = 1; // Lanternfish variant
                }
            }
            Zone::Vents => {
                // Spawn tube worms clusters
                for i in 0..5 {
                    let creature = &mut CREATURES[i];
                    creature.active = true;
                    creature.creature_type = CreatureType::TubeWorms;
                    creature.x = random_range(-15.0, 15.0);
                    creature.y = sub_y - 30.0;
                    creature.z = random_range(-15.0, 15.0);
                }

                // Spawn vent shrimp
                for i in 5..8 {
                    let creature = &mut CREATURES[i];
                    creature.active = true;
                    creature.creature_type = CreatureType::VentShrimp;
                    creature.x = random_range(-10.0, 10.0);
                    creature.y = sub_y - 25.0;
                    creature.z = random_range(-10.0, 10.0);
                }

                // Spawn ghost fish (translucent, drifting)
                CREATURES[8].active = true;
                CREATURES[8].creature_type = CreatureType::GhostFish;
                CREATURES[8].x = 12.0;
                CREATURES[8].y = sub_y - 20.0;
                CREATURES[8].z = -15.0;
                CREATURES[8].animation_phase = random_range(0.0, 6.28);

                CREATURES[9].active = true;
                CREATURES[9].creature_type = CreatureType::GhostFish;
                CREATURES[9].x = -18.0;
                CREATURES[9].y = sub_y - 22.0;
                CREATURES[9].z = 10.0;
                CREATURES[9].animation_phase = random_range(0.0, 6.28);

                // Spawn vent octopus (near chimney)
                CREATURES[10].active = true;
                CREATURES[10].creature_type = CreatureType::VentOctopus;
                CREATURES[10].x = 0.0;
                CREATURES[10].y = sub_y - 28.0;
                CREATURES[10].z = 20.0;
                CREATURES[10].animation_phase = random_range(0.0, 6.28);
            }
            Zone::Luminara => {
                // All creature types converge for finale
                for i in 0..6 {
                    let creature = &mut CREATURES[i];
                    creature.active = true;
                    creature.creature_type = match i {
                        0 => CreatureType::MoonJelly,
                        1 => CreatureType::Siphonophore,
                        2 => CreatureType::DumboOctopus,
                        3 => CreatureType::MoonJelly,
                        4 => CreatureType::Siphonophore,
                        _ => CreatureType::MoonJelly,
                    };
                    creature.x = random_range(-20.0, 20.0);
                    creature.y = sub_y + random_range(-15.0, 15.0);
                    creature.z = random_range(-20.0, 20.0);
                    creature.animation_phase = random_range(0.0, 6.28);
                }
            }
        }
    }
}

fn spawn_zone_flora(zone: Zone) {
    unsafe {
        for flora in FLORA_LIST.iter_mut() {
            flora.active = false;
        }

        let sub_y = SUB.y;

        match zone {
            Zone::Sunlit => {
                // Coral reef
                for i in 0..10 {
                    let flora = &mut FLORA_LIST[i];
                    flora.active = true;
                    flora.flora_type = match i % 3 {
                        0 => FloraType::CoralBrain,
                        1 => FloraType::CoralFan,
                        _ => FloraType::CoralBranch,
                    };
                    flora.x = random_range(-30.0, 30.0);
                    flora.y = sub_y - 20.0;
                    flora.z = random_range(-30.0, 30.0);
                    flora.rotation_y = random_range(0.0, 360.0);
                    flora.scale = random_range(0.8, 1.5);
                }
            }
            Zone::Twilight => {
                // Kelp forest
                for i in 0..15 {
                    let flora = &mut FLORA_LIST[i];
                    flora.active = true;
                    flora.flora_type = if i % 3 == 0 { FloraType::Anemone } else { FloraType::Kelp };
                    flora.x = random_range(-35.0, 35.0);
                    flora.y = sub_y - 25.0;
                    flora.z = random_range(-35.0, 35.0);
                    flora.rotation_y = random_range(0.0, 360.0);
                    flora.scale = random_range(1.0, 2.0);
                }
            }
            _ => {
                // Sparse or no flora in deep zones
            }
        }
    }
}

// === Update ===

#[no_mangle]
pub extern "C" fn update() {
    unsafe {
        let dt = delta_time();
        ELAPSED_TIME += dt;
        PHASE = PHASE.wrapping_add((dt * 65535.0) as u32);

        match GAME_MODE {
            GameMode::Descending => {
                update_input(dt);
                update_submersible(dt);
                update_zone_transitions();
                update_school(dt);
                update_creatures(dt);
                update_bubbles(dt);
                update_discoveries(dt);
                update_whale_encounter(dt);
                update_sperm_whale_encounter(dt);
                update_isopod_encounter(dt);
                update_camera(dt);

                // Check for ending
                if DEPTH >= ZONE_VENTS_MAX {
                    GAME_MODE = GameMode::Ending;
                }

                // Pause
                if button_pressed(0, BUTTON_START) != 0 {
                    GAME_MODE = GameMode::Paused;
                }
            }
            GameMode::Paused => {
                if button_pressed(0, BUTTON_START) != 0 {
                    GAME_MODE = GameMode::Descending;
                }
            }
            GameMode::Ending => {
                ENDING_FADE += dt * 0.1;
                if ENDING_FADE > 1.0 {
                    ENDING_FADE = 1.0;
                }
            }
        }
    }
}

fn update_input(dt: f32) {
    unsafe {
        let steer_x = left_stick_x(0);
        let steer_y = left_stick_y(0);
        let look_x = right_stick_x(0);
        let look_y = right_stick_y(0);
        let thrust_down = trigger_right(0);
        let thrust_up = trigger_left(0);

        // Apply pitch and yaw
        SUB.pitch += steer_y * PITCH_SPEED * dt;
        SUB.yaw += steer_x * YAW_SPEED * dt;

        // Clamp pitch
        if SUB.pitch > 45.0 { SUB.pitch = 45.0; }
        if SUB.pitch < -45.0 { SUB.pitch = -45.0; }

        // Right stick controls camera orbit around submersible
        const ORBIT_SPEED: f32 = 90.0;      // Degrees per second
        const ORBIT_MAX_YAW: f32 = 180.0;   // Full rotation allowed
        const ORBIT_MAX_PITCH: f32 = 60.0;  // Look up/down limits
        const ORBIT_MIN_PITCH: f32 = -30.0; // Can't look too far down

        CAM_ORBIT_YAW += look_x * ORBIT_SPEED * dt;
        CAM_ORBIT_PITCH -= look_y * ORBIT_SPEED * dt; // Inverted for natural feel

        // Wrap yaw for full 360 rotation
        while CAM_ORBIT_YAW > 180.0 { CAM_ORBIT_YAW -= 360.0; }
        while CAM_ORBIT_YAW < -180.0 { CAM_ORBIT_YAW += 360.0; }

        // Clamp pitch
        if CAM_ORBIT_PITCH > ORBIT_MAX_PITCH { CAM_ORBIT_PITCH = ORBIT_MAX_PITCH; }
        if CAM_ORBIT_PITCH < ORBIT_MIN_PITCH { CAM_ORBIT_PITCH = ORBIT_MIN_PITCH; }

        // Gradually return orbit to behind submersible when stick released
        const ORBIT_RETURN_SPEED: f32 = 45.0;
        if look_x.abs() < 0.1 {
            if CAM_ORBIT_YAW > 0.0 {
                CAM_ORBIT_YAW -= ORBIT_RETURN_SPEED * dt;
                if CAM_ORBIT_YAW < 0.0 { CAM_ORBIT_YAW = 0.0; }
            } else if CAM_ORBIT_YAW < 0.0 {
                CAM_ORBIT_YAW += ORBIT_RETURN_SPEED * dt;
                if CAM_ORBIT_YAW > 0.0 { CAM_ORBIT_YAW = 0.0; }
            }
        }
        if look_y.abs() < 0.1 {
            // Return pitch to slightly above (15 degrees)
            const DEFAULT_PITCH: f32 = 15.0;
            if CAM_ORBIT_PITCH > DEFAULT_PITCH {
                CAM_ORBIT_PITCH -= ORBIT_RETURN_SPEED * dt;
                if CAM_ORBIT_PITCH < DEFAULT_PITCH { CAM_ORBIT_PITCH = DEFAULT_PITCH; }
            } else if CAM_ORBIT_PITCH < DEFAULT_PITCH {
                CAM_ORBIT_PITCH += ORBIT_RETURN_SPEED * dt;
                if CAM_ORBIT_PITCH > DEFAULT_PITCH { CAM_ORBIT_PITCH = DEFAULT_PITCH; }
            }
        }

        // Calculate descent speed based on triggers
        let base_speed = -DESCENT_SPEED;
        let thrust_mod = thrust_down * THRUST_DOWN_MULT - thrust_up * THRUST_UP_MULT;
        SUB.velocity_y = base_speed * (1.0 + thrust_mod);

        // Headlight controls
        if button_pressed(0, BUTTON_A) != 0 {
            SUB.headlight_pulse = 30; // 0.5 second pulse
            play_sound(SND_SONAR, 0.8, 0.0);
        }
        if button_pressed(0, BUTTON_B) != 0 {
            SUB.headlight_on = !SUB.headlight_on;
        }

        // Decrement pulse timer
        if SUB.headlight_pulse > 0 {
            SUB.headlight_pulse -= 1;
        }
    }
}

fn update_submersible(dt: f32) {
    unsafe {
        // Convert yaw to radians
        // Add 90 degrees because the submersible model faces +X, not +Z
        let yaw_rad = (SUB.yaw + 90.0) * 3.14159 / 180.0;

        // Calculate forward direction
        let forward_x = sin_approx(yaw_rad);
        let forward_z = cos_approx(yaw_rad);

        // Apply vertical movement (descent)
        SUB.y += SUB.velocity_y * dt;

        // Apply slight horizontal drift based on yaw
        SUB.x += forward_x * 2.0 * dt;
        SUB.z += forward_z * 2.0 * dt;

        // Update depth (positive value for display)
        DEPTH = -SUB.y;
    }
}

fn update_zone_transitions() {
    unsafe {
        let new_zone = depth_to_zone(DEPTH);

        if new_zone != CURRENT_ZONE {
            PREVIOUS_ZONE = CURRENT_ZONE;
            CURRENT_ZONE = new_zone;

            // Spawn new creatures for this zone
            spawn_zone_creatures(new_zone);
            spawn_zone_flora(new_zone);

            // Play zone transition sound
            play_sound(SND_BUBBLES, 0.5, 0.0);
        }
    }
}

fn update_school(dt: f32) {
    unsafe {
        for i in 0..MAX_SCHOOL_FISH {
            if !SCHOOL[i].active {
                continue;
            }

            let fish = &mut SCHOOL[i];

            // Simple flocking behavior
            let mut sep_x = 0.0;
            let mut sep_y = 0.0;
            let mut sep_z = 0.0;
            let mut _neighbors = 0;

            // Separation from other fish
            for j in 0..MAX_SCHOOL_FISH {
                if i == j || !SCHOOL[j].active {
                    continue;
                }
                let other = &SCHOOL[j];
                let dist = distance_3d(fish.x, fish.y, fish.z, other.x, other.y, other.z);
                if dist < SEPARATION_DIST && dist > 0.01 {
                    sep_x += (fish.x - other.x) / dist;
                    sep_y += (fish.y - other.y) / dist;
                    sep_z += (fish.z - other.z) / dist;
                    _neighbors += 1;
                }
            }

            // Scatter from submersible
            let dist_to_sub = distance_3d(fish.x, fish.y, fish.z, SUB.x, SUB.y, SUB.z);
            if dist_to_sub < SCATTER_DIST {
                let scatter_strength = (SCATTER_DIST - dist_to_sub) / SCATTER_DIST;
                sep_x += (fish.x - SUB.x) * scatter_strength * 2.0;
                sep_y += (fish.y - SUB.y) * scatter_strength * 2.0;
                sep_z += (fish.z - SUB.z) * scatter_strength * 2.0;
            }

            // Apply forces
            fish.vx += sep_x * 0.5 * dt;
            fish.vy += sep_y * 0.5 * dt;
            fish.vz += sep_z * 0.5 * dt;

            // Clamp speed
            let speed = sqrt_approx(fish.vx * fish.vx + fish.vy * fish.vy + fish.vz * fish.vz);
            if speed > SCHOOL_SPEED {
                let scale = SCHOOL_SPEED / speed;
                fish.vx *= scale;
                fish.vy *= scale;
                fish.vz *= scale;
            }

            // Update position
            fish.x += fish.vx * dt;
            fish.y += fish.vy * dt;
            fish.z += fish.vz * dt;

            // Despawn if too far
            if distance_3d(fish.x, fish.y, fish.z, SUB.x, SUB.y, SUB.z) > 80.0 {
                fish.active = false;
            }
        }
    }
}

fn update_creatures(dt: f32) {
    unsafe {
        for creature in CREATURES.iter_mut() {
            if !creature.active {
                continue;
            }

            creature.animation_phase += dt * 2.0;

            match creature.creature_type {
                CreatureType::MoonJelly | CreatureType::Siphonophore => {
                    // Gentle drift downward with pulse
                    creature.y -= 0.5 * dt;
                    creature.x += sin_approx(creature.animation_phase) * 0.3 * dt;
                }
                CreatureType::SeaTurtle | CreatureType::MantaRay => {
                    // Curiosity AI - approach submersible if in range
                    let dist = distance_3d(creature.x, creature.y, creature.z, SUB.x, SUB.y, SUB.z);
                    if dist > 10.0 && dist < 40.0 {
                        let approach_speed = 2.0 * dt;
                        creature.x += (SUB.x - creature.x) / dist * approach_speed;
                        creature.y += (SUB.y - creature.y) / dist * approach_speed;
                        creature.z += (SUB.z - creature.z) / dist * approach_speed;
                    } else if dist < 8.0 {
                        // Back off slightly
                        creature.x -= (SUB.x - creature.x) / dist * 1.0 * dt;
                        creature.z -= (SUB.z - creature.z) / dist * 1.0 * dt;
                    }
                }
                CreatureType::Anglerfish => {
                    // Lurk with slight drift
                    creature.x += sin_approx(creature.animation_phase * 0.3) * 0.2 * dt;
                }
                CreatureType::TubeWorms => {
                    // Stationary, gentle sway
                    creature.rotation_y = sin_approx(creature.animation_phase) * 5.0;
                }
                _ => {}
            }
        }
    }
}

fn update_bubbles(dt: f32) {
    unsafe {
        // Spawn new bubbles behind submersible
        BUBBLE_SPAWN_TIMER += 1;
        if BUBBLE_SPAWN_TIMER >= BUBBLE_SPAWN_RATE {
            BUBBLE_SPAWN_TIMER = 0;

            // Find inactive bubble slot
            for bubble in BUBBLES.iter_mut() {
                if !bubble.active {
                    // Spawn behind and below submersible
                    // Add 90 degrees because the submersible model faces +X
                    let yaw_rad = (SUB.yaw + 90.0) * 3.14159 / 180.0;
                    let offset_x = -sin_approx(yaw_rad) * 1.5;
                    let offset_z = -cos_approx(yaw_rad) * 1.5;

                    // Random spread
                    let spread_x = ((random() % 100) as f32 - 50.0) / 100.0;
                    let spread_z = ((random() % 100) as f32 - 50.0) / 100.0;

                    bubble.x = SUB.x + offset_x + spread_x;
                    bubble.y = SUB.y - 0.5;
                    bubble.z = SUB.z + offset_z + spread_z;
                    bubble.scale = 0.1 + ((random() % 20) as f32) / 100.0;
                    bubble.lifetime = BUBBLE_LIFETIME;
                    bubble.active = true;
                    break;
                }
            }
        }

        // Update active bubbles
        for bubble in BUBBLES.iter_mut() {
            if !bubble.active {
                continue;
            }

            // Rise and drift
            bubble.y += BUBBLE_RISE_SPEED * dt;
            bubble.x += sin_approx(bubble.y * 2.0) * 0.5 * dt;

            // Fade out
            bubble.lifetime -= dt;
            if bubble.lifetime <= 0.0 {
                bubble.active = false;
            }
        }
    }
}

fn creature_type_to_bit(ctype: CreatureType) -> u16 {
    1u16 << (ctype as u8)
}

fn creature_name(ctype: CreatureType) -> &'static [u8] {
    match ctype {
        CreatureType::ReefFish => b"REEF FISH",
        CreatureType::SeaTurtle => b"SEA TURTLE",
        CreatureType::MantaRay => b"MANTA RAY",
        CreatureType::CoralCrab => b"CORAL CRAB",
        CreatureType::MoonJelly => b"MOON JELLY",
        CreatureType::Lanternfish => b"LANTERNFISH",
        CreatureType::Siphonophore => b"SIPHONOPHORE",
        CreatureType::GiantSquid => b"GIANT SQUID",
        CreatureType::Anglerfish => b"ANGLERFISH",
        CreatureType::GulperEel => b"GULPER EEL",
        CreatureType::DumboOctopus => b"DUMBO OCTOPUS",
        CreatureType::VampireSquid => b"VAMPIRE SQUID",
        CreatureType::TubeWorms => b"TUBE WORMS",
        CreatureType::VentShrimp => b"VENT SHRIMP",
        CreatureType::GhostFish => b"GHOST FISH",
        CreatureType::VentOctopus => b"VENT OCTOPUS",
    }
}

fn update_discoveries(dt: f32) {
    unsafe {
        // Update popup timer
        if DISCOVERY_POPUP_TIMER > 0.0 {
            DISCOVERY_POPUP_TIMER -= dt;
        }

        // Check distance to creatures for new discoveries
        for creature in CREATURES.iter() {
            if !creature.active {
                continue;
            }

            let bit = creature_type_to_bit(creature.creature_type);
            if (DISCOVERED_CREATURES & bit) != 0 {
                continue; // Already discovered
            }

            let dist = distance_3d(creature.x, creature.y, creature.z, SUB.x, SUB.y, SUB.z);
            if dist < DISCOVERY_DISTANCE {
                // Discover this creature type!
                DISCOVERED_CREATURES |= bit;

                // Set popup text
                let name = creature_name(creature.creature_type);
                DISCOVERY_POPUP_LEN = name.len();
                for (i, &ch) in name.iter().enumerate() {
                    DISCOVERY_POPUP_TEXT[i] = ch;
                }
                DISCOVERY_POPUP_TIMER = DISCOVERY_POPUP_DURATION;

                // Play discovery sound
                play_sound(SND_SONAR, 0.5, 0.0);
            }
        }
    }
}

fn update_whale_encounter(dt: f32) {
    unsafe {
        // Trigger whale at specific depth
        if !WHALE_TRIGGERED && DEPTH > WHALE_TRIGGER_DEPTH && DEPTH < WHALE_END_DEPTH {
            WHALE_ACTIVE = true;
            WHALE_TRIGGERED = true;
            WHALE_PROGRESS = 0.0;
            play_sound(SND_WHALE, 1.0, 0.0);
        }

        if WHALE_ACTIVE {
            WHALE_PROGRESS += dt * 0.05; // ~20 second encounter
            if WHALE_PROGRESS >= 1.0 {
                WHALE_ACTIVE = false;
            }
        }
    }
}

fn update_sperm_whale_encounter(dt: f32) {
    unsafe {
        // Sperm Whale dives past in the Midnight zone, hunting giant squid
        if !SPERM_WHALE_TRIGGERED && DEPTH > SPERM_WHALE_TRIGGER_DEPTH && DEPTH < SPERM_WHALE_END_DEPTH {
            SPERM_WHALE_ACTIVE = true;
            SPERM_WHALE_TRIGGERED = true;
            SPERM_WHALE_PROGRESS = 0.0;
            play_sound(SND_WHALE, 1.0, 0.0);
        }

        if SPERM_WHALE_ACTIVE {
            SPERM_WHALE_PROGRESS += dt * 0.04; // ~25 second encounter (slower dive)
            if SPERM_WHALE_PROGRESS >= 1.0 {
                SPERM_WHALE_ACTIVE = false;
            }
        }
    }
}

fn update_isopod_encounter(dt: f32) {
    unsafe {
        // Giant Isopod appears near the submersible glass in the Vents zone
        if !ISOPOD_TRIGGERED && DEPTH > ISOPOD_TRIGGER_DEPTH && DEPTH < ISOPOD_END_DEPTH {
            ISOPOD_ACTIVE = true;
            ISOPOD_TRIGGERED = true;
            ISOPOD_PROGRESS = 0.0;
            play_sound(SND_BUBBLES, 0.6, 0.0);
        }

        if ISOPOD_ACTIVE {
            ISOPOD_PROGRESS += dt * 0.08; // ~12 second encounter (curious peek)
            if ISOPOD_PROGRESS >= 1.0 {
                ISOPOD_ACTIVE = false;
            }
        }
    }
}

fn update_camera(_dt: f32) {
    unsafe {
        // Camera orbit parameters
        const CAM_DISTANCE: f32 = 12.0;  // Distance from submersible
        const CAM_HEIGHT_OFFSET: f32 = 2.0; // Base height above submersible

        // Calculate total camera angle (submersible yaw + orbit offset)
        // Add 90 degrees because the submersible model faces +X, not +Z
        let total_yaw = SUB.yaw + CAM_ORBIT_YAW + 90.0;
        let total_yaw_rad = total_yaw * 3.14159 / 180.0;
        let pitch_rad = CAM_ORBIT_PITCH * 3.14159 / 180.0;

        // Calculate camera position in orbit around submersible
        // Camera is behind (-forward) and elevated based on pitch
        let cos_pitch = cos_approx(pitch_rad);
        let sin_pitch = sin_approx(pitch_rad);

        // Horizontal distance from submersible (shrinks as we look up/down)
        let horizontal_dist = CAM_DISTANCE * cos_pitch;

        // Camera position relative to submersible
        // -sin and -cos because we want to be BEHIND the submersible
        let offset_x = -sin_approx(total_yaw_rad) * horizontal_dist;
        let offset_y = CAM_HEIGHT_OFFSET + CAM_DISTANCE * sin_pitch;
        let offset_z = -cos_approx(total_yaw_rad) * horizontal_dist;

        let desired_x = SUB.x + offset_x;
        let desired_y = SUB.y + offset_y;
        let desired_z = SUB.z + offset_z;

        // Smooth interpolation for fluid camera movement
        let lerp = 0.12;
        CAM_X += (desired_x - CAM_X) * lerp;
        CAM_Y += (desired_y - CAM_Y) * lerp;
        CAM_Z += (desired_z - CAM_Z) * lerp;
    }
}

// === Render ===

#[no_mangle]
pub extern "C" fn render() {
    unsafe {
        // Camera target: look at the submersible with slight forward offset
        // This keeps the submersible centered in view as we orbit around it
        // Add 90 degrees because the submersible model faces +X, not +Z
        let sub_yaw_rad = (SUB.yaw + 90.0) * 3.14159 / 180.0;

        // Target is slightly ahead of submersible (where it's heading)
        let forward_offset = 2.0;
        let target_x = SUB.x + sin_approx(sub_yaw_rad) * forward_offset;
        let target_y = SUB.y + 0.5; // Look at slightly above center of submersible
        let target_z = SUB.z + cos_approx(sub_yaw_rad) * forward_offset;

        // Setup camera - looking from orbit position toward submersible
        camera_set(CAM_X, CAM_Y, CAM_Z, target_x, target_y, target_z);
        camera_fov(65.0); // Slightly narrower for better framing

        // Setup environment and lighting for current zone
        setup_zone_environment(CURRENT_ZONE);
        draw_env();
        setup_zone_lighting(CURRENT_ZONE);

        // Render volumetric effects (god rays in sunlit zone)
        render_god_rays();

        // Render world
        render_submersible();
        render_bubbles();
        render_flora();
        render_school();
        render_creatures();
        render_whale();
        render_sperm_whale();
        render_isopod();

        // Render atmospheric effects (thermal shimmer in vents)
        render_thermal_shimmer();

        // Render HUD
        render_hud();

        // Render pause/ending overlay
        if GAME_MODE == GameMode::Paused {
            render_pause_overlay();
        } else if GAME_MODE == GameMode::Ending {
            render_ending_overlay();
        }
    }
}

fn setup_zone_environment(zone: Zone) {
    unsafe {
        match zone {
            Zone::Sunlit => {
                // Bright blue gradient
                env_gradient(0, 0x87CEEBFF, 0x4A90D9FF, 0x2E6B9EFF, 0x1A4A6EFF, 0.0, 0.1);
                // Sunbeam caustics
                env_scatter(1, 0, 60, 4, 200, 12, 0xFFFFFF40, 0xFFFFFF20, 150, 80, PHASE);
                env_blend(1); // Additive
            }
            Zone::Twilight => {
                // Deep blue to indigo gradient
                env_gradient(0, 0x2E6B9EFF, 0x1A3A5EFF, 0x0F1F3AFF, 0x050A15FF, 0.0, 0.0);
                // Marine snow
                env_scatter(1, 1, 120, 2, 100, 4, 0xFFFFFF30, 0xCCCCFF20, 100, 60, PHASE);
                env_blend(0); // Alpha
                // Kelp curtains
                env_curtains(2, 3, 40, 60, 120, 2, 8, 20, 0x1A3A2AFF, 0x0F2A1AFF, 0, 100, PHASE);
            }
            Zone::Midnight => {
                // Near-black gradient
                env_gradient(0, 0x050A15FF, 0x020408FF, 0x010204FF, 0x000000FF, 0.0, 0.0);
                // Bioluminescent particles
                env_scatter(1, 0, 200, 1, 255, 0, 0x00FFFF40, 0xFF00FF30, 80, 40, PHASE);
                env_blend(1); // Additive
                // Distant seamount silhouettes
                env_silhouette(2, 200, 2, 0x00000080, 0x00000040, 0x050A15FF, 0x020408FF, 100, PHASE);
            }
            Zone::Vents => {
                // Black with volcanic hint
                env_gradient(0, 0x000000FF, 0x0A0505FF, 0x150808FF, 0x0A0202FF, 0.0, 0.2);
                // Vent smoke
                env_scatter(1, 1, 80, 3, 50, 16, 0x33221180, 0x22110060, 60, 30, PHASE);
                env_blend(0); // Alpha
            }
            Zone::Luminara => {
                // Ethereal glow
                env_gradient(0, 0x000000FF, 0x0A0A1AFF, 0x1A1A3AFF, 0x0A0A1AFF, 0.0, 0.0);
                // Dense bioluminescence
                env_scatter(1, 0, 300, 2, 255, 0, 0x00FFFFFF, 0xFF00FFFF, 120, 60, PHASE);
                env_blend(1); // Additive
                // Ethereal rings
                env_rings(2, 16, 2, 0x4400FFFF, 0x00FFFFAA, 0xFF44FFFF, 300, 8.0, 0.0, 0.0, 1.0, PHASE);
            }
        }
    }
}

fn setup_zone_lighting(zone: Zone) {
    unsafe {
        match zone {
            Zone::Sunlit => {
                // Strong sun from above
                light_set(0, 0.3, -0.9, 0.2);
                light_color(0, 0xFFF8E0FF);
                light_intensity(0, 6.0);
                light_enable(0);

                // Cyan ambient fill
                light_set(1, -0.5, -0.4, 0.3);
                light_color(1, 0x80C0FFFF);
                light_intensity(1, 0.4);
                light_enable(1);
            }
            Zone::Twilight => {
                // Fading directional
                light_set(0, 0.3, -0.9, 0.2);
                light_color(0, 0xC0D0E0FF);
                light_intensity(0, 2.0);
                light_enable(0);

                light_disable(1);
            }
            Zone::Midnight | Zone::Vents | Zone::Luminara => {
                // No directional lights
                light_disable(0);
                light_disable(1);
            }
        }

        // Headlight (light index 2)
        if SUB.headlight_on {
            let intensity = if SUB.headlight_pulse > 0 { 8.0 } else { 5.0 };
            light_set_point(2, SUB.x, SUB.y, SUB.z);
            light_color(2, 0xFFFFE0FF);
            light_intensity(2, intensity);
            light_range(2, 30.0);
            light_enable(2);
        } else {
            light_disable(2);
        }

        // Creature bioluminescence (light index 3) - for anglerfish lure
        if CURRENT_ZONE == Zone::Midnight {
            for creature in CREATURES.iter() {
                if creature.active && creature.creature_type == CreatureType::Anglerfish {
                    let pulse = (sin_approx(creature.animation_phase * 3.0) + 1.0) * 0.5;
                    light_set_point(3, creature.x, creature.y + 0.5, creature.z);
                    light_color(3, 0x44FF88FF);
                    light_intensity(3, 2.0 + pulse * 2.0);
                    light_range(3, 8.0);
                    light_enable(3);
                    break;
                }
            }
        } else {
            light_disable(3);
        }
    }
}

fn render_submersible() {
    unsafe {
        // Hull material - matte metal
        material_albedo(TEX_SUBMERSIBLE);
        material_specular(0x888888FF);
        material_shininess(0.6);
        material_emissive(0.0);
        material_rim(0.1, 6.0);
        uniform_alpha(15); // Fully opaque

        push_identity();
        push_translate(SUB.x, SUB.y, SUB.z);
        push_rotate_y(SUB.yaw);
        push_rotate_x(SUB.pitch);
        draw_mesh(MESH_SUBMERSIBLE);
    }
}

fn render_bubbles() {
    unsafe {
        // Semi-transparent white bubbles (shiny glass-like)
        material_specular(0xFFFFFFFF);
        material_shininess(0.95);
        uniform_alpha(11); // Slightly translucent (0-15 range)

        for bubble in BUBBLES.iter() {
            if !bubble.active {
                continue;
            }

            // Fade based on remaining lifetime (0-15 range)
            let alpha = ((bubble.lifetime / BUBBLE_LIFETIME) * 11.0) as u32;
            uniform_alpha(alpha.min(15));

            push_translate(bubble.x, bubble.y, bubble.z);
            push_scale(bubble.scale, bubble.scale, bubble.scale);
            draw_mesh(MESH_BUBBLE_CLUSTER);
        }

        uniform_alpha(15); // Reset to fully opaque
    }
}

fn render_flora() {
    unsafe {
        for flora in FLORA_LIST.iter() {
            if !flora.active {
                continue;
            }

            let mesh = match flora.flora_type {
                FloraType::CoralBrain => MESH_CORAL_BRAIN,
                FloraType::CoralFan => MESH_CORAL_FAN,
                FloraType::CoralBranch => MESH_CORAL_BRANCH,
                FloraType::Kelp => MESH_KELP,
                FloraType::Anemone => MESH_ANEMONE,
                FloraType::SeaGrass => MESH_SEA_GRASS,
            };

            material_albedo(TEX_CORAL_BRAIN);
            material_specular(0x445566FF);
            material_shininess(0.3);
            material_emissive(0.0);
            uniform_alpha(15);

            push_identity();
            push_translate(flora.x, flora.y, flora.z);
            push_rotate_y(flora.rotation_y);
            push_scale(flora.scale, flora.scale, flora.scale);
            draw_mesh(mesh);
        }
    }
}

fn render_school() {
    unsafe {
        let mesh = if CURRENT_ZONE == Zone::Midnight {
            MESH_LANTERNFISH
        } else {
            MESH_REEF_FISH
        };

        for fish in SCHOOL.iter() {
            if !fish.active {
                continue;
            }

            // Calculate facing direction from velocity
            let heading = if fish.vx.abs() > 0.01 || fish.vz.abs() > 0.01 {
                atan2_approx(fish.vx, fish.vz) * 180.0 / 3.14159
            } else {
                0.0
            };

            material_albedo(TEX_REEF_FISH);
            material_specular(0xFFCC88FF);
            material_shininess(0.5);

            if CURRENT_ZONE == Zone::Midnight {
                // Bioluminescent lanternfish
                material_emissive(0.4);
            } else {
                material_emissive(0.0);
            }
            material_rim(0.1, 6.0);
            uniform_alpha(15);

            push_identity();
            push_translate(fish.x, fish.y, fish.z);
            push_rotate_y(heading);
            push_scale(0.5, 0.5, 0.5);
            draw_mesh(mesh);
        }
    }
}

fn render_creatures() {
    unsafe {
        // Track creature index for dither offset cycling
        let mut creature_idx: usize = 0;

        for creature in CREATURES.iter() {
            if !creature.active {
                continue;
            }

            let mesh = match creature.creature_type {
                CreatureType::ReefFish => MESH_REEF_FISH,
                CreatureType::SeaTurtle => MESH_SEA_TURTLE,
                CreatureType::MantaRay => MESH_MANTA_RAY,
                CreatureType::CoralCrab => MESH_CORAL_CRAB,
                CreatureType::MoonJelly => MESH_MOON_JELLY,
                CreatureType::Lanternfish => MESH_LANTERNFISH,
                CreatureType::Siphonophore => MESH_SIPHONOPHORE,
                CreatureType::GiantSquid => MESH_GIANT_SQUID,
                CreatureType::Anglerfish => MESH_ANGLERFISH,
                CreatureType::GulperEel => MESH_GULPER_EEL,
                CreatureType::DumboOctopus => MESH_DUMBO_OCTOPUS,
                CreatureType::VampireSquid => MESH_VAMPIRE_SQUID,
                CreatureType::TubeWorms => MESH_TUBE_WORMS,
                CreatureType::VentShrimp => MESH_VENT_SHRIMP,
                CreatureType::GhostFish => MESH_GHOST_FISH,
                CreatureType::VentOctopus => MESH_VENT_OCTOPUS,
            };

            // Determine if creature is translucent (needs dither offset)
            let is_translucent = matches!(
                creature.creature_type,
                CreatureType::MoonJelly
                    | CreatureType::Siphonophore
                    | CreatureType::GhostFish
                    | CreatureType::VampireSquid
                    | CreatureType::GiantSquid
                    | CreatureType::DumboOctopus
                    | CreatureType::VentOctopus
            );

            // Apply unique dither offset for translucent creatures
            // This prevents pattern cancellation when they overlap
            if is_translucent {
                let dither_x = (creature_idx % 4) as u32;
                let dither_y = ((creature_idx / 4) % 4) as u32;
                dither_offset(dither_x, dither_y);
            } else {
                dither_offset(0, 0);
            }

            // Set materials based on creature type
            match creature.creature_type {
                CreatureType::MoonJelly => {
                    material_albedo(TEX_MOON_JELLY);
                    material_specular(0xAABBFFFF);
                    material_shininess(0.3);
                    material_emissive(0.2);
                    material_rim(0.3, 4.0);
                    uniform_alpha(8); // Translucent dithered
                }
                CreatureType::Siphonophore => {
                    material_specular(0xFFAAFFFF);
                    material_shininess(0.2);
                    material_emissive(0.5);
                    material_rim(0.4, 3.0);
                    uniform_alpha(6); // Very translucent dithered
                }
                CreatureType::Anglerfish => {
                    material_albedo(TEX_ANGLERFISH);
                    material_specular(0x444444FF);
                    material_shininess(0.7);
                    material_emissive(0.0);
                    uniform_alpha(15);
                }
                CreatureType::TubeWorms => {
                    material_specular(0xFF4444FF);
                    material_shininess(0.5);
                    material_emissive(0.4);
                    material_rim(0.2, 4.0);
                    uniform_alpha(15);
                }
                CreatureType::CoralCrab => {
                    material_albedo(TEX_CORAL_CRAB);
                    material_specular(0xCC8866FF);
                    material_shininess(0.8); // Hard shell
                    material_emissive(0.0);
                    material_rim(0.1, 8.0);
                    uniform_alpha(15);
                }
                CreatureType::GiantSquid => {
                    material_albedo(TEX_GIANT_SQUID);
                    material_specular(0x443355FF);
                    material_shininess(0.4);
                    material_emissive(0.15); // Slight bioluminescence
                    material_rim(0.2, 5.0);
                    uniform_alpha(11); // Dithered translucent
                }
                CreatureType::VampireSquid => {
                    material_albedo(TEX_VAMPIRE_SQUID);
                    material_specular(0x662233FF);
                    material_shininess(0.3);
                    material_emissive(0.3); // Bioluminescent webbing
                    material_rim(0.35, 4.0);
                    uniform_alpha(9); // Dithered translucent
                }
                CreatureType::GhostFish => {
                    material_albedo(TEX_GHOST_FISH);
                    material_specular(0xDDEEFFFF);
                    material_shininess(0.2);
                    material_emissive(0.1);
                    material_rim(0.5, 3.0); // Strong rim for ethereal glow
                    uniform_alpha(4); // Very translucent dithered
                }
                CreatureType::VentOctopus => {
                    material_albedo(TEX_VENT_OCTOPUS);
                    material_specular(0xEEDDEEFF);
                    material_shininess(0.5);
                    material_emissive(0.0);
                    material_rim(0.2, 5.0);
                    uniform_alpha(12); // Slightly translucent dithered
                }
                CreatureType::GulperEel => {
                    material_specular(0x222233FF);
                    material_shininess(0.6);
                    material_emissive(0.0);
                    material_rim(0.1, 8.0);
                    uniform_alpha(15);
                }
                CreatureType::DumboOctopus => {
                    material_specular(0xFFAAAAFF);
                    material_shininess(0.4);
                    material_emissive(0.1);
                    material_rim(0.25, 4.0);
                    uniform_alpha(12);
                }
                CreatureType::VentShrimp => {
                    material_specular(0xFFCCAAFF);
                    material_shininess(0.7);
                    material_emissive(0.0);
                    material_rim(0.1, 6.0);
                    uniform_alpha(15);
                }
                _ => {
                    material_albedo(TEX_SEA_TURTLE);
                    material_specular(0x667755FF);
                    material_shininess(0.6);
                    material_emissive(0.0);
                    material_rim(0.15, 6.0);
                    uniform_alpha(15);
                }
            }

            // Animation offset for jellies
            let y_offset = if creature.creature_type == CreatureType::MoonJelly {
                sin_approx(creature.animation_phase) * 0.3
            } else {
                0.0
            };

            push_identity();
            push_translate(creature.x, creature.y + y_offset, creature.z);
            push_rotate_y(creature.rotation_y);
            draw_mesh(mesh);

            // Increment creature index for dither offset cycling
            creature_idx += 1;
        }

        // Reset dither offset after all creatures
        dither_offset(0, 0);
        uniform_alpha(15);
    }
}

fn render_whale() {
    unsafe {
        if !WHALE_ACTIVE {
            return;
        }

        // Whale swims past from left to right
        let whale_x = -50.0 + WHALE_PROGRESS * 100.0;
        let whale_y = SUB.y - 5.0;
        let whale_z = SUB.z + 30.0;

        material_albedo(TEX_BLUE_WHALE);
        material_specular(0x556677FF);
        material_shininess(0.4);
        material_emissive(0.0);
        material_rim(0.15, 6.0);
        uniform_alpha(15);

        push_identity();
        push_translate(whale_x, whale_y, whale_z);
        push_rotate_y(90.0); // Facing direction of travel
        push_scale(2.0, 2.0, 2.0);
        draw_mesh(MESH_BLUE_WHALE);
    }
}

fn render_sperm_whale() {
    unsafe {
        if !SPERM_WHALE_ACTIVE {
            return;
        }

        // Sperm whale dives from above-right to below-left (hunting dive)
        let t = SPERM_WHALE_PROGRESS;
        let whale_x = 40.0 - t * 80.0;
        let whale_y = SUB.y + 20.0 - t * 50.0; // Descending dive
        let whale_z = SUB.z + 35.0 - t * 20.0;

        // Rotation: nose-down diving angle
        let dive_angle = 30.0 + t * 15.0;

        material_albedo(TEX_SPERM_WHALE);
        material_specular(0x445566FF);
        material_shininess(0.5);
        material_emissive(0.0);
        material_rim(0.12, 7.0);
        uniform_alpha(15);

        push_identity();
        push_translate(whale_x, whale_y, whale_z);
        push_rotate_y(-120.0); // Heading direction
        push_rotate_x(dive_angle); // Nose down
        push_scale(1.8, 1.8, 1.8);
        draw_mesh(MESH_SPERM_WHALE);
    }
}

fn render_isopod() {
    unsafe {
        if !ISOPOD_ACTIVE {
            return;
        }

        // Giant isopod approaches submersible glass, pauses, then retreats
        let t = ISOPOD_PROGRESS;

        // Approach (0-0.3), linger (0.3-0.7), retreat (0.7-1.0)
        let approach_dist = if t < 0.3 {
            // Approach phase: from 10 to 2 units
            10.0 - (t / 0.3) * 8.0
        } else if t < 0.7 {
            // Linger phase: stay at 2 units, slight wobble
            2.0 + sin_approx(t * 20.0) * 0.3
        } else {
            // Retreat phase: from 2 to 12 units
            2.0 + ((t - 0.7) / 0.3) * 10.0
        };

        // Position in front of submersible glass
        // Add 90 degrees because the submersible model faces +X
        let yaw_rad = (SUB.yaw + 90.0) * 3.14159 / 180.0;
        let isopod_x = SUB.x + sin_approx(yaw_rad) * approach_dist;
        let isopod_y = SUB.y - 0.5; // Slightly below eye level
        let isopod_z = SUB.z + cos_approx(yaw_rad) * approach_dist;

        // Face the submersible
        let face_angle = SUB.yaw + 180.0;

        material_albedo(TEX_GIANT_ISOPOD);
        material_specular(0xAABBCCFF);
        material_shininess(0.7); // Shiny carapace
        material_emissive(0.0);
        material_rim(0.1, 8.0);
        uniform_alpha(15);

        push_identity();
        push_translate(isopod_x, isopod_y, isopod_z);
        push_rotate_y(face_angle);
        push_scale(0.8, 0.8, 0.8);
        draw_mesh(MESH_GIANT_ISOPOD);
    }
}

/// Render volumetric god rays in Sunlit zone
/// Uses dithered billboards with offset cycling for smooth blending
fn render_god_rays() {
    unsafe {
        if CURRENT_ZONE != Zone::Sunlit {
            return;
        }

        // God rays fade based on depth (strongest at surface, gone by 180m)
        let fade = 1.0 - (DEPTH / ZONE_SUNLIT_MAX).min(1.0);
        if fade < 0.05 {
            return;
        }

        // Calculate base alpha level (0-10 range for dithering, leaving headroom)
        let base_alpha = (fade * 8.0) as u32;

        // Animated shimmer phase
        let shimmer_phase = ELAPSED_TIME * 0.5;

        for i in 0..GOD_RAY_COUNT {
            let (offset_x, offset_z) = GOD_RAY_OFFSETS[i];

            // Ray position relative to submersible (moves with player)
            let ray_x = SUB.x + offset_x;
            let ray_z = SUB.z + offset_z;

            // Ray descends from above, slightly angled
            // Position at water surface level (y=0) extending downward
            let ray_y = -DEPTH * 0.3; // Rays originate from above

            // Shimmer animation - rays sway slightly
            let sway = sin_approx(shimmer_phase + i as f32 * 1.2) * 2.0;

            // Per-ray alpha variation for natural look
            let ray_alpha = base_alpha.saturating_sub((i % 3) as u32);
            if ray_alpha == 0 {
                continue;
            }

            // Cycle dither offsets across rays for smooth blending
            // This prevents banding when multiple rays overlap
            let dither_x = (i % 4) as u32;
            let dither_y = ((i / 4) % 4) as u32;
            dither_offset(dither_x, dither_y);

            uniform_alpha(ray_alpha);

            // Golden-white light color
            let ray_color = 0xFFFAE880; // Warm sunlight with base alpha

            push_identity();
            push_translate(ray_x + sway, ray_y, ray_z);
            // Slight angle - rays come from sun direction
            push_rotate_z(5.0 + sin_approx(shimmer_phase * 0.3 + i as f32) * 3.0);

            // Draw cylindrical billboard (always faces camera on Y axis)
            draw_billboard(GOD_RAY_WIDTH, GOD_RAY_HEIGHT, BILLBOARD_CYLINDRICAL_Y, ray_color);
        }

        // Reset dither offset
        dither_offset(0, 0);
        uniform_alpha(15);
    }
}

/// Render thermal shimmer columns rising from hydrothermal vents
/// Uses stencil masking for heat distortion effect
fn render_thermal_shimmer() {
    unsafe {
        if CURRENT_ZONE != Zone::Vents {
            return;
        }

        let shimmer_phase = ELAPSED_TIME * 2.0;

        for i in 0..SHIMMER_COLUMNS {
            let (offset_x, offset_z) = SHIMMER_POSITIONS[i];

            // Shimmer rises from vent floor
            let shimmer_x = SUB.x + offset_x;
            let shimmer_z = SUB.z + offset_z;
            let shimmer_y = SUB.y - 25.0; // Near vent floor

            // Animated wobble
            let wobble_x = sin_approx(shimmer_phase + i as f32 * 2.0) * 0.5;
            let wobble_z = cos_approx(shimmer_phase * 0.7 + i as f32 * 1.5) * 0.5;

            // Dither offset cycling for each column
            let dither_x = (i % 4) as u32;
            let dither_y = ((i + 2) % 4) as u32;
            dither_offset(dither_x, dither_y);

            // Very low alpha for subtle heat shimmer
            uniform_alpha(3 + (i % 2) as u32);

            // Orange-red heat color
            let heat_color = 0xFF442240;

            push_identity();
            push_translate(shimmer_x + wobble_x, shimmer_y, shimmer_z + wobble_z);

            draw_billboard(SHIMMER_WIDTH, SHIMMER_HEIGHT, BILLBOARD_CYLINDRICAL_Y, heat_color);
        }

        // Reset dither offset
        dither_offset(0, 0);
        uniform_alpha(15);
    }
}

fn render_hud() {
    unsafe {
        // Bind custom underwater font
        font_bind(FONT_HANDLE);

        // Depth meter background
        draw_rect(10.0, 480.0, 120.0, 50.0, 0x00000088);

        // Depth text
        let depth_label = b"DEPTH";
        draw_text(depth_label.as_ptr(), depth_label.len() as u32, 20.0, 490.0, 12.0, COLOR_DEPTH_TEXT);

        // Format depth value
        let mut depth_buf = [0u8; 16];
        let depth_len = format_depth(DEPTH, &mut depth_buf);
        draw_text(depth_buf.as_ptr(), depth_len, 20.0, 505.0, 20.0, COLOR_WHITE);

        // Zone name
        let (zone_ptr, zone_len) = match CURRENT_ZONE {
            Zone::Sunlit => (b"SUNLIT WATERS".as_ptr(), 13u32),
            Zone::Twilight => (b"TWILIGHT REALM".as_ptr(), 14u32),
            Zone::Midnight => (b"MIDNIGHT ABYSS".as_ptr(), 14u32),
            Zone::Vents => (b"HYDROTHERMAL VENTS".as_ptr(), 18u32),
            Zone::Luminara => (b"LUMINARA TRENCH".as_ptr(), 15u32),
        };
        draw_text(zone_ptr, zone_len, 400.0, 20.0, 16.0, COLOR_CYAN);

        // Headlight indicator
        if SUB.headlight_on {
            let indicator = b"*";
            draw_text(indicator.as_ptr(), 1, 920.0, 20.0, 24.0, 0xFFFF88FF);
        }

        // Creature discovery popup (center of screen, fading)
        if DISCOVERY_POPUP_TIMER > 0.0 {
            let alpha = if DISCOVERY_POPUP_TIMER > 1.0 {
                255u32
            } else {
                (DISCOVERY_POPUP_TIMER * 255.0) as u32
            };
            let color = 0x88FFFF00 | alpha; // Cyan with fade

            // "NEW SPECIES" label
            let label = b"NEW SPECIES";
            draw_text(label.as_ptr(), label.len() as u32, 380.0, 200.0, 16.0, color);

            // Species name (larger)
            draw_text(
                DISCOVERY_POPUP_TEXT.as_ptr(),
                DISCOVERY_POPUP_LEN as u32,
                320.0,
                230.0,
                28.0,
                color,
            );
        }
    }
}

fn render_pause_overlay() {
    unsafe {
        // Darken screen
        draw_rect(0.0, 0.0, 960.0, 540.0, 0x00000088);

        let pause_text = b"PAUSED";
        draw_text(pause_text.as_ptr(), pause_text.len() as u32, 400.0, 250.0, 48.0, COLOR_WHITE);

        let resume_text = b"Press START to resume";
        draw_text(resume_text.as_ptr(), resume_text.len() as u32, 340.0, 320.0, 16.0, COLOR_CYAN);
    }
}

fn render_ending_overlay() {
    unsafe {
        // Fade to deep blue
        let alpha = (ENDING_FADE * 255.0) as u32;
        let fade_color = 0x0A152800 | alpha;
        draw_rect(0.0, 0.0, 960.0, 540.0, fade_color);

        if ENDING_FADE > 0.3 {
            let text_alpha = ((ENDING_FADE - 0.3) / 0.7 * 255.0) as u32;
            let title_color = 0x88DDFF00 | text_alpha;
            let text_color = 0xAADDFF00 | text_alpha;
            let sub_color = 0x6699AA00 | text_alpha;

            // Game title
            let title = b"LUMINA DEPTHS";
            draw_text(title.as_ptr(), title.len() as u32, 335.0, 100.0, 40.0, title_color);

            // Tagline
            let tagline = b"Descend into the light beneath the dark";
            draw_text(tagline.as_ptr(), tagline.len() as u32, 265.0, 160.0, 16.0, text_color);

            // Credits
            if ENDING_FADE > 0.5 {
                let credits_alpha = ((ENDING_FADE - 0.5) / 0.5 * 255.0) as u32;
                let credits_color = 0xCCEEFF00 | credits_alpha;

                let cred1 = b"A Nethercore ZX Showcase Game";
                draw_text(cred1.as_ptr(), cred1.len() as u32, 310.0, 280.0, 14.0, credits_color);

                let cred2 = b"Procedurally Generated by Claude";
                draw_text(cred2.as_ptr(), cred2.len() as u32, 295.0, 310.0, 14.0, credits_color);

                let cred3 = b"Built with the Nethercore Platform";
                draw_text(cred3.as_ptr(), cred3.len() as u32, 285.0, 340.0, 14.0, credits_color);

                // Species discovered count
                if ENDING_FADE > 0.7 {
                    let count = DISCOVERED_CREATURES.count_ones();
                    let mut discovered_buf = [0u8; 32];
                    let mut len = 0usize;

                    // "Species discovered: X/16"
                    let prefix = b"Species discovered: ";
                    for (i, &c) in prefix.iter().enumerate() {
                        discovered_buf[i] = c;
                    }
                    len = prefix.len();

                    // Add count
                    if count >= 10 {
                        discovered_buf[len] = b'1';
                        len += 1;
                        discovered_buf[len] = b'0' + ((count % 10) as u8);
                        len += 1;
                    } else {
                        discovered_buf[len] = b'0' + (count as u8);
                        len += 1;
                    }

                    let suffix = b"/16";
                    for &c in suffix.iter() {
                        discovered_buf[len] = c;
                        len += 1;
                    }

                    draw_text(discovered_buf.as_ptr(), len as u32, 355.0, 400.0, 18.0, title_color);
                }

                // Thank you
                if ENDING_FADE > 0.9 {
                    let thanks = b"Thank you for diving with us";
                    draw_text(thanks.as_ptr(), thanks.len() as u32, 300.0, 480.0, 16.0, sub_color);
                }
            }
        }
    }
}

// Format depth as "XXXXm"
fn format_depth(depth: f32, buf: &mut [u8]) -> u32 {
    let depth_int = depth as u32;
    let mut len = format_number(depth_int, buf);
    buf[len as usize] = b'm';
    len += 1;
    len
}

fn format_number(mut num: u32, buf: &mut [u8]) -> u32 {
    if num == 0 {
        buf[0] = b'0';
        return 1;
    }

    let mut temp = [0u8; 16];
    let mut temp_len = 0;

    while num > 0 {
        temp[temp_len] = b'0' + (num % 10) as u8;
        temp_len += 1;
        num /= 10;
    }

    // Reverse
    for i in 0..temp_len {
        buf[i] = temp[temp_len - 1 - i];
    }

    temp_len as u32
}
