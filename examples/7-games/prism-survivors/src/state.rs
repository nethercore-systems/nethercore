//! Game state module for Prism Survivors
//!
//! Contains all constants and static mut game state variables.

use crate::types::*;

// =============================================================================
// Constants
// =============================================================================

pub const SCREEN_WIDTH: f32 = 960.0;
pub const SCREEN_HEIGHT: f32 = 540.0;

// Buttons
pub const BUTTON_UP: u32 = 0;
pub const BUTTON_DOWN: u32 = 1;
pub const BUTTON_A: u32 = 4;
pub const BUTTON_B: u32 = 5;
pub const BUTTON_START: u32 = 12;

// Game limits
pub const MAX_PLAYERS: usize = 4;
pub const MAX_ENEMIES: usize = 100;
pub const MAX_XP_GEMS: usize = 200;
pub const MAX_PROJECTILES: usize = 80;
pub const MAX_POWERUPS: usize = 8;
pub const MAX_VFX: usize = 100;

// Arena
pub const ARENA_SIZE: f32 = 20.0;

// Progression
pub const XP_PER_LEVEL: [u32; 15] = [10, 20, 35, 55, 80, 110, 150, 200, 260, 330, 410, 500, 600, 720, 850];
pub const STAGE_DURATION: f32 = 120.0; // 2 minutes per stage

// Revive system constants
pub const REVIVE_TIME: f32 = 3.0;   // Seconds to revive
pub const REVIVE_RANGE: f32 = 2.0;  // Units distance
pub const REVIVE_HEALTH: f32 = 0.5; // Percentage of max HP
pub const REVIVE_INVULN: f32 = 1.0; // Seconds of invincibility

// Combo system constants
pub const COMBO_WINDOW: f32 = 2.0;  // Seconds to maintain combo
pub const COMBO_DECAY: f32 = 0.5;   // Extra time per kill

// =============================================================================
// Game State (static for rollback safety)
// =============================================================================

pub static mut PHASE: GamePhase = GamePhase::Title;
pub static mut PLAYERS: [Player; MAX_PLAYERS] = [Player::new(); MAX_PLAYERS];
pub static mut ENEMIES: [Enemy; MAX_ENEMIES] = [Enemy::new(); MAX_ENEMIES];
pub static mut XP_GEMS: [XpGem; MAX_XP_GEMS] = [XpGem::new(); MAX_XP_GEMS];
pub static mut PROJECTILES: [Projectile; MAX_PROJECTILES] = [Projectile::new(); MAX_PROJECTILES];
pub static mut VFX_POOL: [Vfx; MAX_VFX] = [Vfx::new(); MAX_VFX];
pub static mut PLAYER_HURT_FLASH: f32 = 0.0;  // Screen flash timer when any player is hurt

// Run stats
pub static mut WAVE: u32 = 1;
pub static mut STAGE: Stage = Stage::CrystalCavern;
pub static mut GAME_TIME: f32 = 0.0;
pub static mut STAGE_TIME: f32 = 0.0;
pub static mut SPAWN_TIMER: f32 = 0.0;
pub static mut ELITE_SPAWN_TIMER: f32 = 0.0;

// Difficulty and scaling
pub static mut DIFFICULTY: Difficulty = Difficulty::Normal;
pub static mut DIFFICULTY_SELECTION: u8 = 1;  // Menu cursor (0-3)
pub static mut ACTIVE_PLAYERS: u32 = 1;       // Cached player count for scaling

// Combo system (emergent gameplay)
pub static mut COMBO_COUNT: u32 = 0;          // Current kill streak
pub static mut COMBO_TIMER: f32 = 0.0;        // Time until combo expires
pub static mut COMBO_MULT: f32 = 1.0;         // Current combo multiplier
pub static mut MAX_COMBO: u32 = 0;            // Best combo this run

// Environmental hazard timers
pub static mut HAZARD_TIMER: f32 = 0.0;
pub static mut HAZARD_ACTIVE: bool = false;
pub static mut HAZARD_X: f32 = 0.0;
pub static mut HAZARD_Y: f32 = 0.0;
pub static mut HAZARD_RADIUS: f32 = 0.0;
pub static mut BOSS_SPAWNED_THIS_WAVE: bool = false;
pub static mut KILLS: u32 = 0;

// Level-up UI
pub static mut LEVELUP_PLAYER: usize = 0;
pub static mut LEVELUP_CHOICES: [PowerUpType; 3] = [PowerUpType::Might; 3];
pub static mut LEVELUP_SELECTION: usize = 0;

// Camera
pub static mut CAM_X: f32 = 0.0;
pub static mut CAM_Y: f32 = 0.0;

// Assets - Heroes
pub static mut TEX_KNIGHT: u32 = 0;
pub static mut TEX_MAGE: u32 = 0;
pub static mut TEX_RANGER: u32 = 0;
pub static mut TEX_CLERIC: u32 = 0;
pub static mut TEX_NECROMANCER: u32 = 0;
pub static mut TEX_PALADIN: u32 = 0;
// Assets - Enemies
pub static mut TEX_GOLEM: u32 = 0;
pub static mut TEX_CRAWLER: u32 = 0;
pub static mut TEX_WISP: u32 = 0;
pub static mut TEX_SKELETON: u32 = 0;
pub static mut TEX_SHADE: u32 = 0;
pub static mut TEX_BERSERKER: u32 = 0;
pub static mut TEX_ARCANE_SENTINEL: u32 = 0;
// Assets - Elites
pub static mut TEX_CRYSTAL_KNIGHT: u32 = 0;
pub static mut TEX_VOID_MAGE: u32 = 0;
pub static mut TEX_GOLEM_TITAN: u32 = 0;
pub static mut TEX_SPECTER_LORD: u32 = 0;
// Assets - Bosses
pub static mut TEX_PRISM_COLOSSUS: u32 = 0;
pub static mut TEX_VOID_DRAGON: u32 = 0;
// Assets - Other
pub static mut TEX_XP: u32 = 0;
pub static mut TEX_ARENA: u32 = 0;

pub static mut MESH_KNIGHT: u32 = 0;
pub static mut MESH_MAGE: u32 = 0;
pub static mut MESH_RANGER: u32 = 0;
pub static mut MESH_CLERIC: u32 = 0;
pub static mut MESH_NECROMANCER: u32 = 0;
pub static mut MESH_PALADIN: u32 = 0;
pub static mut MESH_GOLEM: u32 = 0;
pub static mut MESH_CRAWLER: u32 = 0;
pub static mut MESH_WISP: u32 = 0;
pub static mut MESH_SKELETON: u32 = 0;
pub static mut MESH_SHADE: u32 = 0;
pub static mut MESH_BERSERKER: u32 = 0;
pub static mut MESH_ARCANE_SENTINEL: u32 = 0;
// Meshes - Elites
pub static mut MESH_CRYSTAL_KNIGHT: u32 = 0;
pub static mut MESH_VOID_MAGE: u32 = 0;
pub static mut MESH_GOLEM_TITAN: u32 = 0;
pub static mut MESH_SPECTER_LORD: u32 = 0;
// Meshes - Bosses
pub static mut MESH_PRISM_COLOSSUS: u32 = 0;
pub static mut MESH_VOID_DRAGON: u32 = 0;
// Meshes - Other
pub static mut MESH_XP: u32 = 0;
pub static mut MESH_ARENA: u32 = 0;
pub static mut MESH_PROJ: u32 = 0;

// Boss tracking
pub static mut ACTIVE_BOSS: Option<usize> = None;

pub static mut SFX_SHOOT: u32 = 0;
pub static mut SFX_HIT: u32 = 0;
pub static mut SFX_DEATH: u32 = 0;
pub static mut SFX_XP: u32 = 0;
pub static mut SFX_LEVELUP: u32 = 0;
pub static mut SFX_HURT: u32 = 0;
pub static mut SFX_SELECT: u32 = 0;

// Font
pub static mut TEX_FONT: u32 = 0;
pub static mut FONT_PRISM: u32 = 0;

// Attract mode / title animation
pub static mut TITLE_TIMER: f32 = 0.0;

// =============================================================================
// Class Selection State
// =============================================================================

pub static mut CLASS_SELECTIONS: [HeroClass; MAX_PLAYERS] = [HeroClass::Knight; MAX_PLAYERS];
pub static mut CLASS_CONFIRMED: [bool; MAX_PLAYERS] = [false; MAX_PLAYERS];
pub static mut CLASS_SELECT_CURSOR: [u8; MAX_PLAYERS] = [0; MAX_PLAYERS];  // 0-5 for each class
pub static mut CLASS_SELECT_COUNTDOWN: f32 = 0.0;  // Countdown after all ready
pub static mut ALL_PLAYERS_READY: bool = false;
