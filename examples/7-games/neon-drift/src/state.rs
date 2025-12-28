//! Global game state for NEON DRIFT
//!
//! All static mutable state is centralized here for rollback netcode compatibility.

use crate::types::{
    Car, Camera, Particle, MAX_PARTICLES, GameMode, TrackId,
    TrackSegment, MAX_TRACK_SEGMENTS, Waypoint, MAX_WAYPOINTS,
};

// === Constants ===

pub const SCREEN_WIDTH: u32 = 960;
pub const SCREEN_HEIGHT: u32 = 540;

// Colors
pub const COLOR_WHITE: u32 = 0xFFFFFFFF;
pub const COLOR_CYAN: u32 = 0x00FFFFFF;
pub const COLOR_MAGENTA: u32 = 0xFF00FFFF;

// Physics constants
pub const DRIFT_THRESHOLD: f32 = 0.3;
pub const BOOST_COST: f32 = 0.5;
pub const BOOST_MULTIPLIER: f32 = 1.5;
pub const BOOST_DURATION: u32 = 120;

// Track constants
pub const NUM_CHECKPOINTS: usize = 4;
pub const ATTRACT_MODE_DELAY: f32 = 15.0;

// === Global State ===

pub static mut GAME_MODE: GameMode = GameMode::MainMenu;
pub static mut SELECTED_TRACK: TrackId = TrackId::SunsetStrip;
pub static mut CARS: [Car; 4] = [Car::new(); 4];
pub static mut CAMERAS: [Camera; 4] = [Camera::new(); 4];
pub static mut ACTIVE_PLAYER_COUNT: u32 = 1;

// Menu selection state
pub static mut MENU_SELECTION: u32 = 0;
pub static mut MENU_TIME: f32 = 0.0;
pub static mut CAR_SELECTIONS: [u32; 4] = [0; 4];
pub static mut PLAYER_CONFIRMED: [bool; 4] = [false; 4];

// Race state
pub static mut COUNTDOWN_TIMER: u32 = 0;
pub static mut RACE_TIME: f32 = 0.0;
pub static mut RACE_FINISHED: bool = false;
pub static mut TRACK_LENGTH: f32 = 200.0;
pub static mut CHECKPOINT_Z: [f32; NUM_CHECKPOINTS] = [0.0, 50.0, 100.0, 150.0];

// Track layout data
pub static mut TRACK_SEGMENTS: [TrackSegment; MAX_TRACK_SEGMENTS] = [TrackSegment::new(); MAX_TRACK_SEGMENTS];
pub static mut TRACK_SEGMENT_COUNT: usize = 0;

// AI Waypoints (3D)
pub static mut WAYPOINTS: [Waypoint; MAX_WAYPOINTS] = [Waypoint::new(0.0, 0.0, 0.0); MAX_WAYPOINTS];
pub static mut WAYPOINT_COUNT: usize = 0;

// Animation state
pub static mut GRID_PHASE: u32 = 0;
pub static mut RING_PHASE: u32 = 0;
pub static mut SPEED_PHASE: u32 = 0;
pub static mut WINDOW_PHASE: u32 = 0;
pub static mut ELAPSED_TIME: f32 = 0.0;

// Title screen & attract mode
pub static mut TITLE_ANIM_TIME: f32 = 0.0;
pub static mut IDLE_TIMER: f32 = 0.0;

// Particle system
pub static mut PARTICLES: [Particle; MAX_PARTICLES] = [Particle::new(); MAX_PARTICLES];
pub static mut NEXT_PARTICLE: usize = 0;

// Visual effect state per player
pub static mut SPEED_LINE_INTENSITY: [f32; 4] = [0.0; 4];
pub static mut BOOST_GLOW_INTENSITY: [f32; 4] = [0.0; 4];

// === Asset Handles ===

// Car meshes
pub static mut MESH_SPEEDSTER: u32 = 0;
pub static mut MESH_MUSCLE: u32 = 0;
pub static mut MESH_RACER: u32 = 0;
pub static mut MESH_DRIFT: u32 = 0;
pub static mut MESH_PHANTOM: u32 = 0;
pub static mut MESH_TITAN: u32 = 0;
pub static mut MESH_VIPER: u32 = 0;

// Track segment meshes
pub static mut MESH_TRACK_STRAIGHT: u32 = 0;
pub static mut MESH_TRACK_CURVE_LEFT: u32 = 0;
pub static mut MESH_TRACK_TUNNEL: u32 = 0;
pub static mut MESH_TRACK_JUMP: u32 = 0;

// Prop meshes
pub static mut MESH_PROP_BARRIER: u32 = 0;
pub static mut MESH_PROP_BOOST_PAD: u32 = 0;
pub static mut MESH_PROP_BILLBOARD: u32 = 0;
pub static mut MESH_PROP_BUILDING: u32 = 0;

// Textures (albedo)
pub static mut TEX_SPEEDSTER: u32 = 0;
pub static mut TEX_MUSCLE: u32 = 0;
pub static mut TEX_RACER: u32 = 0;
pub static mut TEX_DRIFT: u32 = 0;
pub static mut TEX_PHANTOM: u32 = 0;
pub static mut TEX_TITAN: u32 = 0;
pub static mut TEX_VIPER: u32 = 0;

// Textures (emissive)
pub static mut TEX_SPEEDSTER_EMISSIVE: u32 = 0;
pub static mut TEX_MUSCLE_EMISSIVE: u32 = 0;
pub static mut TEX_RACER_EMISSIVE: u32 = 0;
pub static mut TEX_DRIFT_EMISSIVE: u32 = 0;
pub static mut TEX_PHANTOM_EMISSIVE: u32 = 0;
pub static mut TEX_TITAN_EMISSIVE: u32 = 0;
pub static mut TEX_VIPER_EMISSIVE: u32 = 0;

// Sounds
pub static mut SND_BOOST: u32 = 0;
pub static mut SND_DRIFT: u32 = 0;
pub static mut SND_WALL: u32 = 0;
pub static mut SND_CHECKPOINT: u32 = 0;
pub static mut SND_FINISH: u32 = 0;

// Music Tracks (XM Tracker handles)
pub static mut MUSIC_SUNSET_STRIP: u32 = 0;
pub static mut MUSIC_NEON_CITY: u32 = 0;
pub static mut MUSIC_VOID_TUNNEL: u32 = 0;
pub static mut MUSIC_CRYSTAL_CAVERN: u32 = 0;
pub static mut MUSIC_SOLAR_HIGHWAY: u32 = 0;

// Track textures
pub static mut TEX_TRACK_STRAIGHT: u32 = 0;
pub static mut TEX_TRACK_CURVE: u32 = 0;

// Prop textures
pub static mut TEX_PROP_BUILDING: u32 = 0;
pub static mut TEX_PROP_BARRIER: u32 = 0;
pub static mut TEX_PROP_BILLBOARD: u32 = 0;
pub static mut TEX_PROP_BOOST_PAD: u32 = 0;

// Font
pub static mut TEX_NEON_FONT: u32 = 0;
pub static mut NEON_FONT: u32 = 0;
