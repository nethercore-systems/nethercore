//! Core types for NEON DRIFT

// === Game State Enums ===

#[derive(Clone, Copy, PartialEq)]
#[repr(u8)]
pub enum GameMode {
    MainMenu,
    CarSelect,
    TrackSelect,
    CountdownReady,
    Racing,
    RaceFinished,
    Paused,
    AttractMode,
}

#[derive(Clone, Copy, PartialEq)]
#[repr(u8)]
pub enum CarType {
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
pub enum TrackId {
    SunsetStrip,
    NeonCity,
    VoidTunnel,
    CrystalCavern,
    SolarHighway,
}

// === Track Segment System ===

#[derive(Clone, Copy, PartialEq)]
#[repr(u8)]
pub enum SegmentType {
    Straight,
    CurveLeft,     // 45 degree turn left
    CurveRight,    // 45 degree turn right
    Tunnel,        // Straight segment with tunnel visual
    Jump,          // Ramp/jump segment
}

/// A track segment with position, rotation, and type
#[derive(Clone, Copy)]
pub struct TrackSegment {
    pub x: f32,
    pub z: f32,
    pub rotation: f32,  // Rotation in degrees
    pub segment_type: SegmentType,
}

impl TrackSegment {
    pub const fn new() -> Self {
        Self {
            x: 0.0,
            z: 0.0,
            rotation: 0.0,
            segment_type: SegmentType::Straight,
        }
    }
}

/// Maximum segments per track
pub const MAX_TRACK_SEGMENTS: usize = 32;

/// Waypoint for AI navigation
#[derive(Clone, Copy)]
pub struct Waypoint {
    pub x: f32,
    pub z: f32,
}

impl Waypoint {
    pub const fn new(x: f32, z: f32) -> Self {
        Self { x, z }
    }
}

pub const MAX_WAYPOINTS: usize = 32;

// === Car State ===

#[derive(Clone, Copy)]
pub struct Car {
    // Position & Orientation
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub rotation_y: f32,

    // Velocity
    pub velocity_forward: f32,
    pub velocity_lateral: f32,
    pub angular_velocity: f32,

    // Boost & Drift
    pub boost_meter: f32,
    pub is_boosting: bool,
    pub boost_timer: u32,
    pub is_drifting: bool,

    // Car type and stats
    pub car_type: CarType,
    pub max_speed: f32,
    pub acceleration: f32,
    pub handling: f32,
    pub drift_factor: f32,

    // Race state
    pub current_lap: u32,
    pub last_checkpoint: usize,
    pub race_position: u32,
    pub current_waypoint: usize,  // For AI navigation

    // Collision
    pub collision_pushback_x: f32,
    pub collision_pushback_z: f32,
}

impl Car {
    pub const fn new() -> Self {
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
            current_waypoint: 0,
            collision_pushback_x: 0.0,
            collision_pushback_z: 0.0,
        }
    }

    pub fn init_stats(&mut self) {
        match self.car_type {
            CarType::Speedster => {
                self.max_speed = 28.5;
                self.acceleration = 14.0;
                self.handling = 1.0;
                self.drift_factor = 0.85;
            }
            CarType::Muscle => {
                self.max_speed = 33.0;
                self.acceleration = 12.5;
                self.handling = 0.85;
                self.drift_factor = 0.8;
            }
            CarType::Racer => {
                self.max_speed = 28.5;
                self.acceleration = 17.0;
                self.handling = 0.95;
                self.drift_factor = 0.9;
            }
            CarType::Drift => {
                self.max_speed = 27.0;
                self.acceleration = 15.5;
                self.handling = 1.2;
                self.drift_factor = 1.0;
            }
            CarType::Phantom => {
                self.max_speed = 31.5;
                self.acceleration = 14.5;
                self.handling = 0.9;
                self.drift_factor = 0.88;
            }
            CarType::Titan => {
                self.max_speed = 25.5;
                self.acceleration = 13.5;
                self.handling = 0.75;
                self.drift_factor = 0.7;
            }
            CarType::Viper => {
                self.max_speed = 36.0;
                self.acceleration = 11.5;
                self.handling = 1.05;
                self.drift_factor = 0.95;
            }
        }
    }
}

// === Camera State ===

#[derive(Clone, Copy)]
pub struct Camera {
    pub current_pos_x: f32,
    pub current_pos_y: f32,
    pub current_pos_z: f32,
    pub current_target_x: f32,
    pub current_target_y: f32,
    pub current_target_z: f32,
    #[allow(dead_code)] // Reserved for look-back feature
    pub is_looking_back: bool,
    pub shake_intensity: f32,
    pub shake_decay: f32,
    pub shake_offset_x: f32,
    pub shake_offset_y: f32,
}

impl Camera {
    pub const fn new() -> Self {
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

    pub fn add_shake(&mut self, intensity: f32) {
        self.shake_intensity = (self.shake_intensity + intensity).min(1.0);
    }

    pub fn update_shake(&mut self, rand_val: u32) {
        if self.shake_intensity > 0.01 {
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

// === Particle System ===

pub const MAX_PARTICLES: usize = 64;

#[derive(Clone, Copy, PartialEq)]
#[repr(u8)]
#[allow(dead_code)] // All variants may not be used yet
pub enum ParticleType {
    BoostFlame,
    DriftSmoke,
    Spark,
    SpeedLine,
}

#[derive(Clone, Copy)]
pub struct Particle {
    pub active: bool,
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub vel_x: f32,
    pub vel_y: f32,
    pub vel_z: f32,
    pub life: f32,
    pub max_life: f32,
    pub size: f32,
    pub color: u32,
    pub particle_type: ParticleType,
}

impl Particle {
    pub const fn new() -> Self {
        Self {
            active: false,
            x: 0.0,
            y: 0.0,
            z: 0.0,
            vel_x: 0.0,
            vel_y: 0.0,
            vel_z: 0.0,
            life: 0.0,
            max_life: 1.0,
            size: 0.1,
            color: 0xFFFFFFFF,
            particle_type: ParticleType::Spark,
        }
    }
}
