//! PRISM SURVIVORS - 4-Player Co-op Roguelite Survival
//!
//! A run-based survival roguelite for Nethercore ZX featuring:
//!
//! ## Core Features
//! - 6 unique hero classes with distinct starting weapons
//! - 18 power-ups (10 weapons + 8 passives) with roguelite level-up choices
//! - 10 synergy combinations for emergent build variety
//! - Wave-based enemy spawning with 7 basic, 4 elite, and 2 boss types
//! - 4 distinct stage environments with unique hazards
//! - Full 4-player co-op with rollback netcode and revive system
//!
//! ## Difficulty System
//! - Easy: 0.6x HP, 0.7x damage, 0.8x spawns, elites wave 7+
//! - Normal: Balanced gameplay, elites wave 5+
//! - Hard: 1.4x HP, 1.3x damage, 1.2x spawns, +10% XP bonus, elites wave 4+
//! - Nightmare: 2.0x HP, 1.6x damage, 1.5x spawns, +25% XP bonus, elites wave 3+
//!
//! ## Player Scaling
//! Enemy HP and spawn rates scale with player count (1-4 players)
//! for balanced challenge regardless of party size.
//!
//! ## Combo System
//! Chain kills to build combo multiplier (up to 2.5x XP bonus).
//! Combo tiers: 5+ (green), 10+ (orange), 25+ (gold), 50+ (purple).
//!
//! ## Synergies
//! - Steam Explosion: Fireball + Ice Shards (extra shards + AoE slow)
//! - Chain Lightning: Magic Missile + Lightning (chain damage)
//! - Whirlwind: Cleave + Shadow Orb (spinning blade attack)
//! - Holy Fire: Holy Nova + Fireball (ignite enemies)
//! - Frost Arrow: Piercing Arrow + Ice Shards (slow + pierce)
//! - Void Drain: Soul Drain + Shadow Orb (orbs heal)
//! - Divine Bolt: Divine Crush + Lightning (stun + chain)
//! - Berserk Fury: Cleave + Fury (damage scales with missing HP)
//! - Vampire Touch: Soul Drain + Vitality (double lifesteal)
//! - Speed Demon: Swiftness + Haste (+30% proj speed, -20% cooldown)
//!
//! ## Environmental Hazards
//! Each stage has unique hazards that spawn periodically:
//! - Crystal Cavern: Crystal shards
//! - Enchanted Forest: Poison clouds
//! - Volcanic Depths: Lava pools
//! - Void Realm: Void rifts
//!
//! Render Mode 3 (Specular-Shininess Blinn-Phong)

#![no_std]
#![no_main]

use core::panic::PanicInfo;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    core::arch::wasm32::unreachable()
}

// =============================================================================
// FFI Declarations
// =============================================================================

#[link(wasm_import_module = "env")]
extern "C" {
    // Configuration (init-only)
    fn set_clear_color(color: u32);
    fn render_mode(mode: u32);
    fn set_tick_rate(rate: u32);

    // System
    fn delta_time() -> f32;
    fn elapsed_time() -> f32;
    fn random() -> u32;
    fn random_range(min: i32, max: i32) -> i32;
    fn random_f32() -> f32;
    fn player_count() -> u32;

    // Input
    fn left_stick_x(player: u32) -> f32;
    fn left_stick_y(player: u32) -> f32;
    fn button_pressed(player: u32, button: u32) -> u32;
    fn button_held(player: u32, button: u32) -> u32;

    // Camera
    fn camera_set(x: f32, y: f32, z: f32, target_x: f32, target_y: f32, target_z: f32);
    fn camera_fov(fov_degrees: f32);

    // ROM Asset Loading (init-only)
    fn rom_texture(id_ptr: *const u8, id_len: u32) -> u32;
    fn rom_mesh(id_ptr: *const u8, id_len: u32) -> u32;
    fn rom_sound(id_ptr: *const u8, id_len: u32) -> u32;

    // Procedural Meshes (init-only)
    fn sphere(radius: f32, segments: u32, rings: u32) -> u32;
    fn plane(size_x: f32, size_z: f32, subdiv_x: u32, subdiv_z: u32) -> u32;

    // Transforms
    fn push_identity();
    fn push_translate(x: f32, y: f32, z: f32);
    fn push_rotate_y(angle_deg: f32);
    fn push_scale_uniform(s: f32);

    // Textures
    fn texture_bind(handle: u32);
    fn texture_filter(filter: u32);

    // Mesh Drawing
    fn draw_mesh(handle: u32);

    // Render State
    fn set_color(color: u32);
    fn depth_test(enabled: u32);
    fn cull_mode(mode: u32);

    // Materials (Mode 3)
    fn material_shininess(value: f32);
    fn material_specular(color: u32);

    // Lighting
    fn light_set(index: u32, x: f32, y: f32, z: f32);
    fn light_color(index: u32, color: u32);
    fn light_intensity(index: u32, intensity: f32);
    fn light_enable(index: u32);

    // Environment
    fn env_gradient(layer: u32, zenith: u32, sky_h: u32, ground_h: u32, nadir: u32, rot: f32, shift: f32);
    fn env_scatter(layer: u32, variant: u32, density: u32, size: u32, glow: u32, streak: u32, c1: u32, c2: u32, parallax: u32, psize: u32, phase: u32);
    fn draw_env();

    // 2D Drawing
    fn draw_text(ptr: *const u8, len: u32, x: f32, y: f32, size: f32, color: u32);
    fn draw_rect(x: f32, y: f32, w: f32, h: f32, color: u32);
    fn layer(n: u32);

    // Font
    fn load_font(texture: u32, char_width: u32, char_height: u32, first_codepoint: u32, char_count: u32) -> u32;
    fn font_bind(handle: u32);

    // Audio
    fn play_sound(sound: u32, volume: f32, pan: f32);
}

// =============================================================================
// Constants
// =============================================================================

const SCREEN_WIDTH: f32 = 960.0;
const SCREEN_HEIGHT: f32 = 540.0;

// Buttons
const BUTTON_UP: u32 = 0;
const BUTTON_DOWN: u32 = 1;
const BUTTON_A: u32 = 4;
const BUTTON_B: u32 = 5;
const BUTTON_START: u32 = 12;

// Game limits
const MAX_PLAYERS: usize = 4;
const MAX_ENEMIES: usize = 100;
const MAX_XP_GEMS: usize = 200;
const MAX_PROJECTILES: usize = 80;
const MAX_POWERUPS: usize = 8;

// Arena
const ARENA_SIZE: f32 = 20.0;

// Progression
const XP_PER_LEVEL: [u32; 15] = [10, 20, 35, 55, 80, 110, 150, 200, 260, 330, 410, 500, 600, 720, 850];
const STAGE_DURATION: f32 = 120.0; // 2 minutes per stage

// Calculate XP required for a given level (1-indexed)
// Uses table for levels 1-15, exponential scaling after
fn xp_for_level(level: u32) -> u32 {
    if level <= 15 {
        XP_PER_LEVEL[(level - 1) as usize]
    } else {
        // Exponential scaling: base = 850, growth = 1.15x per level
        let base = XP_PER_LEVEL[14] as f32;  // 850
        let extra_levels = (level - 15) as f32;
        let mult = 1.15f32;
        // Using simple power approximation: base * mult^extra_levels
        let mut result = base;
        for _ in 0..(extra_levels as u32) {
            result *= mult;
        }
        result as u32
    }
}

// =============================================================================
// Types - Roguelite Power-up System
// =============================================================================

#[derive(Clone, Copy, PartialEq)]
#[repr(u8)]
enum PowerUpType {
    // Weapons (10 types)
    Cleave = 0,         // Knight starting - melee sweep
    MagicMissile = 1,   // Mage starting - homing projectiles
    PiercingArrow = 2,  // Ranger starting - piercing shots
    HolyNova = 3,       // Cleric starting - radial burst
    SoulDrain = 4,      // Necromancer starting - life steal beam
    DivineCrush = 5,    // Paladin starting - hammer smash
    Fireball = 6,       // AoE explosion
    IceShards = 7,      // Multi-projectile spread
    LightningBolt = 8,  // Chain damage
    ShadowOrb = 9,      // Orbiting damage

    // Passives (8 types)
    Might = 10,         // +20% damage
    Swiftness = 11,     // +15% move speed
    Vitality = 12,      // +25 max HP, heal
    Haste = 13,         // +20% attack speed
    Magnetism = 14,     // +50% pickup range
    Armor = 15,         // -15% damage taken
    Luck = 16,          // +25% XP gain
    Fury = 17,          // +5% damage per missing 10% HP
}

// Difficulty modes with multipliers
#[derive(Clone, Copy, PartialEq)]
#[repr(u8)]
enum Difficulty {
    Easy = 0,       // 0.6x enemy HP, 0.7x damage, 0.8x spawn rate
    Normal = 1,     // 1.0x all
    Hard = 2,       // 1.4x enemy HP, 1.3x damage, 1.2x spawn rate, +10% XP bonus
    Nightmare = 3,  // 2.0x enemy HP, 1.6x damage, 1.5x spawn rate, +25% XP bonus, elites wave 3
}

impl Difficulty {
    fn enemy_hp_mult(&self) -> f32 {
        match self { Difficulty::Easy => 0.6, Difficulty::Normal => 1.0, Difficulty::Hard => 1.4, Difficulty::Nightmare => 2.0 }
    }
    fn enemy_damage_mult(&self) -> f32 {
        match self { Difficulty::Easy => 0.7, Difficulty::Normal => 1.0, Difficulty::Hard => 1.3, Difficulty::Nightmare => 1.6 }
    }
    fn spawn_rate_mult(&self) -> f32 {
        match self { Difficulty::Easy => 0.8, Difficulty::Normal => 1.0, Difficulty::Hard => 1.2, Difficulty::Nightmare => 1.5 }
    }
    fn xp_mult(&self) -> f32 {
        // Reward players for taking on harder challenges (no penalties!)
        match self { Difficulty::Easy => 1.0, Difficulty::Normal => 1.0, Difficulty::Hard => 1.1, Difficulty::Nightmare => 1.25 }
    }
    fn elite_start_wave(&self) -> u32 {
        match self { Difficulty::Easy => 7, Difficulty::Normal => 5, Difficulty::Hard => 4, Difficulty::Nightmare => 3 }
    }
    fn name(&self) -> &'static [u8] {
        match self { Difficulty::Easy => b"EASY", Difficulty::Normal => b"NORMAL", Difficulty::Hard => b"HARD", Difficulty::Nightmare => b"NIGHTMARE" }
    }
    fn color(&self) -> u32 {
        match self { Difficulty::Easy => 0x44FF44FF, Difficulty::Normal => 0xFFFF44FF, Difficulty::Hard => 0xFF8844FF, Difficulty::Nightmare => 0xFF4444FF }
    }
}

// Synergy combinations for bonus effects (expanded)
#[derive(Clone, Copy, PartialEq)]
#[repr(u8)]
enum Synergy {
    None = 0,
    // Weapon + Weapon synergies
    SteamExplosion = 1,   // Fireball + IceShards: 5 extra shards + AoE slow
    ChainLightning = 2,   // MagicMissile + LightningBolt: +50% damage + chain to 2nd enemy
    Whirlwind = 3,        // Cleave + ShadowOrb: +75% range + 3 orbs
    HolyFire = 4,         // HolyNova + Fireball: Nova ignites enemies (DoT)
    FrostArrow = 5,       // IceShards + PiercingArrow: Arrows slow + pierce extra targets
    VoidDrain = 6,        // SoulDrain + ShadowOrb: Orbs heal on hit
    DivineBolt = 7,       // DivineCrush + LightningBolt: Stun + chain smash
    // Weapon + Passive synergies
    BerserkFury = 8,      // Cleave + Fury: Cleave grows with missing HP
    VampireTouch = 9,     // SoulDrain + Vitality: Double lifesteal
    SpeedDemon = 10,      // Any weapon + Swiftness + Haste: +30% proj speed, -20% cooldown
}

#[derive(Clone, Copy)]
struct PowerUp {
    ptype: PowerUpType,
    level: u8,          // 1-5
    cooldown: f32,      // For weapons
    timer: f32,         // Current cooldown timer
}

impl PowerUp {
    const fn new(ptype: PowerUpType) -> Self {
        Self { ptype, level: 1, cooldown: 1.0, timer: 0.0 }
    }
}

#[derive(Clone, Copy, PartialEq)]
#[repr(u8)]
enum HeroClass { Knight = 0, Mage = 1, Ranger = 2, Cleric = 3, Necromancer = 4, Paladin = 5 }

#[derive(Clone, Copy, PartialEq)]
#[repr(u8)]
enum EnemyType {
    // Basic enemies (0-6)
    Golem = 0, Crawler = 1, Wisp = 2, Skeleton = 3, Shade = 4, Berserker = 5, ArcaneSentinel = 6,
    // Elite enemies (7-10)
    CrystalKnight = 7, VoidMage = 8, GolemTitan = 9, SpecterLord = 10,
    // Bosses (11-12)
    PrismColossus = 11, VoidDragon = 12,
}

#[derive(Clone, Copy, PartialEq)]
#[repr(u8)]
enum EnemyTier { Normal = 0, Elite = 1, Boss = 2 }

#[derive(Clone, Copy, PartialEq)]
#[repr(u8)]
enum GamePhase { Title, ClassSelect, Playing, LevelUp, Paused, GameOver, Victory }

#[derive(Clone, Copy, PartialEq)]
#[repr(u8)]
enum Stage { CrystalCavern = 0, EnchantedForest = 1, VolcanicDepths = 2, VoidRealm = 3 }

// =============================================================================
// Player State
// =============================================================================

#[derive(Clone, Copy)]
struct Player {
    x: f32, y: f32,
    vx: f32, vy: f32,
    health: f32, max_health: f32,
    xp: u32, level: u32,
    class: HeroClass,
    move_speed: f32,
    damage_mult: f32,
    attack_speed_mult: f32,
    pickup_range: f32,
    armor: f32,
    xp_mult: f32,
    facing_angle: f32,
    invuln_timer: f32,
    active: bool, dead: bool,
    // Revive system
    revive_progress: f32,  // 0.0 to 3.0 (seconds)
    reviving_by: i8,       // Player index reviving us, -1 if none
    powerups: [Option<PowerUp>; MAX_POWERUPS],
    powerup_count: u8,
}

impl Player {
    const fn new() -> Self {
        Self {
            x: 0.0, y: 0.0, vx: 0.0, vy: 0.0,
            health: 100.0, max_health: 100.0,
            xp: 0, level: 1, class: HeroClass::Knight,
            move_speed: 4.0, damage_mult: 1.0, attack_speed_mult: 1.0,
            pickup_range: 1.5, armor: 0.0, xp_mult: 1.0,
            facing_angle: 0.0, invuln_timer: 0.0,
            active: false, dead: false,
            revive_progress: 0.0, reviving_by: -1,
            powerups: [None; MAX_POWERUPS],
            powerup_count: 0,
        }
    }

    fn has_powerup(&self, ptype: PowerUpType) -> Option<u8> {
        for p in &self.powerups {
            if let Some(pu) = p {
                if pu.ptype == ptype { return Some(pu.level); }
            }
        }
        None
    }

    fn get_synergy(&self, ptype: PowerUpType) -> Synergy {
        // Check all possible synergies for this power-up type
        match ptype {
            // Steam Explosion: Fireball + IceShards
            PowerUpType::Fireball | PowerUpType::IceShards => {
                if self.has_powerup(PowerUpType::Fireball).is_some() &&
                   self.has_powerup(PowerUpType::IceShards).is_some() {
                    return Synergy::SteamExplosion;
                }
            }
            // Chain Lightning: MagicMissile + LightningBolt
            PowerUpType::MagicMissile | PowerUpType::LightningBolt => {
                if self.has_powerup(PowerUpType::MagicMissile).is_some() &&
                   self.has_powerup(PowerUpType::LightningBolt).is_some() {
                    return Synergy::ChainLightning;
                }
            }
            // Whirlwind: Cleave + ShadowOrb
            PowerUpType::Cleave => {
                if self.has_powerup(PowerUpType::ShadowOrb).is_some() {
                    return Synergy::Whirlwind;
                }
                // Also check BerserkFury: Cleave + Fury
                if self.has_powerup(PowerUpType::Fury).is_some() {
                    return Synergy::BerserkFury;
                }
            }
            PowerUpType::ShadowOrb => {
                if self.has_powerup(PowerUpType::Cleave).is_some() {
                    return Synergy::Whirlwind;
                }
                // VoidDrain: SoulDrain + ShadowOrb
                if self.has_powerup(PowerUpType::SoulDrain).is_some() {
                    return Synergy::VoidDrain;
                }
            }
            // Holy Fire: HolyNova + Fireball
            PowerUpType::HolyNova => {
                if self.has_powerup(PowerUpType::Fireball).is_some() {
                    return Synergy::HolyFire;
                }
            }
            // Frost Arrow: IceShards + PiercingArrow
            PowerUpType::PiercingArrow => {
                if self.has_powerup(PowerUpType::IceShards).is_some() {
                    return Synergy::FrostArrow;
                }
            }
            // Divine Bolt: DivineCrush + LightningBolt
            PowerUpType::DivineCrush => {
                if self.has_powerup(PowerUpType::LightningBolt).is_some() {
                    return Synergy::DivineBolt;
                }
            }
            // Vampire Touch: SoulDrain + Vitality
            PowerUpType::SoulDrain => {
                if self.has_powerup(PowerUpType::Vitality).is_some() {
                    return Synergy::VampireTouch;
                }
                if self.has_powerup(PowerUpType::ShadowOrb).is_some() {
                    return Synergy::VoidDrain;
                }
            }
            // Speed Demon: Any weapon + Swiftness + Haste
            PowerUpType::Swiftness | PowerUpType::Haste => {
                if self.has_powerup(PowerUpType::Swiftness).is_some() &&
                   self.has_powerup(PowerUpType::Haste).is_some() {
                    return Synergy::SpeedDemon;
                }
            }
            _ => {}
        }
        Synergy::None
    }

    fn add_powerup(&mut self, ptype: PowerUpType) {
        // Check if already have it (upgrade)
        for p in &mut self.powerups {
            if let Some(pu) = p {
                if pu.ptype == ptype && pu.level < 5 {
                    pu.level += 1;
                    return;
                }
            }
        }
        // Add new
        if (self.powerup_count as usize) < MAX_POWERUPS {
            let cooldown = match ptype {
                PowerUpType::Cleave => 0.8,
                PowerUpType::MagicMissile => 0.6,
                PowerUpType::PiercingArrow => 0.4,
                PowerUpType::HolyNova => 1.5,
                PowerUpType::SoulDrain => 1.0,      // Necromancer - life steal beam
                PowerUpType::DivineCrush => 1.2,    // Paladin - hammer smash
                PowerUpType::Fireball => 2.0,
                PowerUpType::IceShards => 0.7,
                PowerUpType::LightningBolt => 1.2,
                PowerUpType::ShadowOrb => 3.0,
                _ => 0.0,
            };
            self.powerups[self.powerup_count as usize] = Some(PowerUp {
                ptype, level: 1, cooldown, timer: 0.0
            });
            self.powerup_count += 1;
        }
    }
}

// =============================================================================
// Other Entities
// =============================================================================

#[derive(Clone, Copy)]
struct Enemy {
    x: f32, y: f32, vx: f32, vy: f32,
    health: f32, max_health: f32,
    damage: f32, speed: f32,
    enemy_type: EnemyType,
    tier: EnemyTier,
    active: bool, hit_timer: f32,
    attack_timer: f32,  // For boss special attacks
}

impl Enemy {
    const fn new() -> Self {
        Self {
            x: 0.0, y: 0.0, vx: 0.0, vy: 0.0,
            health: 10.0, max_health: 10.0, damage: 10.0, speed: 2.0,
            enemy_type: EnemyType::Golem, tier: EnemyTier::Normal,
            active: false, hit_timer: 0.0, attack_timer: 0.0,
        }
    }
}

#[derive(Clone, Copy)]
struct XpGem { x: f32, y: f32, value: u32, active: bool, phase: f32 }

impl XpGem {
    const fn new() -> Self { Self { x: 0.0, y: 0.0, value: 1, active: false, phase: 0.0 } }
}

#[derive(Clone, Copy)]
struct Projectile {
    x: f32, y: f32, vx: f32, vy: f32,
    damage: f32, lifetime: f32, radius: f32,
    piercing: bool, owner: u8, active: bool,
}

impl Projectile {
    const fn new() -> Self {
        Self { x: 0.0, y: 0.0, vx: 0.0, vy: 0.0,
               damage: 10.0, lifetime: 2.0, radius: 0.3,
               piercing: false, owner: 0, active: false }
    }
}

// =============================================================================
// Game State (static for rollback safety)
// =============================================================================

static mut PHASE: GamePhase = GamePhase::Title;
static mut PLAYERS: [Player; MAX_PLAYERS] = [Player::new(); MAX_PLAYERS];
static mut ENEMIES: [Enemy; MAX_ENEMIES] = [Enemy::new(); MAX_ENEMIES];
static mut XP_GEMS: [XpGem; MAX_XP_GEMS] = [XpGem::new(); MAX_XP_GEMS];
static mut PROJECTILES: [Projectile; MAX_PROJECTILES] = [Projectile::new(); MAX_PROJECTILES];

// Run stats
static mut WAVE: u32 = 1;
static mut STAGE: Stage = Stage::CrystalCavern;
static mut GAME_TIME: f32 = 0.0;
static mut STAGE_TIME: f32 = 0.0;
static mut SPAWN_TIMER: f32 = 0.0;
static mut ELITE_SPAWN_TIMER: f32 = 0.0;

// Difficulty and scaling
static mut DIFFICULTY: Difficulty = Difficulty::Normal;
static mut DIFFICULTY_SELECTION: u8 = 1;  // Menu cursor (0-3)
static mut ACTIVE_PLAYERS: u32 = 1;       // Cached player count for scaling

// Combo system (emergent gameplay)
static mut COMBO_COUNT: u32 = 0;          // Current kill streak
static mut COMBO_TIMER: f32 = 0.0;        // Time until combo expires
static mut COMBO_MULT: f32 = 1.0;         // Current combo multiplier
static mut MAX_COMBO: u32 = 0;            // Best combo this run
const COMBO_WINDOW: f32 = 2.0;            // Seconds to maintain combo
const COMBO_DECAY: f32 = 0.5;             // Extra time per kill

// Environmental hazard timers
static mut HAZARD_TIMER: f32 = 0.0;
static mut HAZARD_ACTIVE: bool = false;
static mut HAZARD_X: f32 = 0.0;
static mut HAZARD_Y: f32 = 0.0;
static mut HAZARD_RADIUS: f32 = 0.0;
static mut BOSS_SPAWNED_THIS_WAVE: bool = false;
static mut KILLS: u32 = 0;

// Level-up UI
static mut LEVELUP_PLAYER: usize = 0;
static mut LEVELUP_CHOICES: [PowerUpType; 3] = [PowerUpType::Might; 3];
static mut LEVELUP_SELECTION: usize = 0;

// Camera
static mut CAM_X: f32 = 0.0;
static mut CAM_Y: f32 = 0.0;

// Assets - Heroes
static mut TEX_KNIGHT: u32 = 0;
static mut TEX_MAGE: u32 = 0;
static mut TEX_RANGER: u32 = 0;
static mut TEX_CLERIC: u32 = 0;
static mut TEX_NECROMANCER: u32 = 0;
static mut TEX_PALADIN: u32 = 0;
// Assets - Enemies
static mut TEX_GOLEM: u32 = 0;
static mut TEX_CRAWLER: u32 = 0;
static mut TEX_WISP: u32 = 0;
static mut TEX_SKELETON: u32 = 0;
static mut TEX_SHADE: u32 = 0;
static mut TEX_BERSERKER: u32 = 0;
static mut TEX_ARCANE_SENTINEL: u32 = 0;
// Assets - Elites
static mut TEX_CRYSTAL_KNIGHT: u32 = 0;
static mut TEX_VOID_MAGE: u32 = 0;
static mut TEX_GOLEM_TITAN: u32 = 0;
static mut TEX_SPECTER_LORD: u32 = 0;
// Assets - Bosses
static mut TEX_PRISM_COLOSSUS: u32 = 0;
static mut TEX_VOID_DRAGON: u32 = 0;
// Assets - Other
static mut TEX_XP: u32 = 0;
static mut TEX_ARENA: u32 = 0;

static mut MESH_KNIGHT: u32 = 0;
static mut MESH_MAGE: u32 = 0;
static mut MESH_RANGER: u32 = 0;
static mut MESH_CLERIC: u32 = 0;
static mut MESH_NECROMANCER: u32 = 0;
static mut MESH_PALADIN: u32 = 0;
static mut MESH_GOLEM: u32 = 0;
static mut MESH_CRAWLER: u32 = 0;
static mut MESH_WISP: u32 = 0;
static mut MESH_SKELETON: u32 = 0;
static mut MESH_SHADE: u32 = 0;
static mut MESH_BERSERKER: u32 = 0;
static mut MESH_ARCANE_SENTINEL: u32 = 0;
// Meshes - Elites
static mut MESH_CRYSTAL_KNIGHT: u32 = 0;
static mut MESH_VOID_MAGE: u32 = 0;
static mut MESH_GOLEM_TITAN: u32 = 0;
static mut MESH_SPECTER_LORD: u32 = 0;
// Meshes - Bosses
static mut MESH_PRISM_COLOSSUS: u32 = 0;
static mut MESH_VOID_DRAGON: u32 = 0;
// Meshes - Other
static mut MESH_XP: u32 = 0;
static mut MESH_ARENA: u32 = 0;
static mut MESH_PROJ: u32 = 0;

// Boss tracking
static mut ACTIVE_BOSS: Option<usize> = None;

static mut SFX_SHOOT: u32 = 0;
static mut SFX_HIT: u32 = 0;
static mut SFX_DEATH: u32 = 0;
static mut SFX_XP: u32 = 0;
static mut SFX_LEVELUP: u32 = 0;
static mut SFX_HURT: u32 = 0;
static mut SFX_SELECT: u32 = 0;

// Font
static mut TEX_FONT: u32 = 0;
static mut FONT_PRISM: u32 = 0;

// Attract mode / title animation
static mut TITLE_TIMER: f32 = 0.0;

// =============================================================================
// Math Helpers
// =============================================================================

fn sqrt(x: f32) -> f32 { libm::sqrtf(x) }
fn sin(x: f32) -> f32 { libm::sinf(x) }
fn cos(x: f32) -> f32 { libm::cosf(x) }
fn atan2(y: f32, x: f32) -> f32 { libm::atan2f(y, x) }
fn abs(x: f32) -> f32 { if x < 0.0 { -x } else { x } }
fn clamp(v: f32, min: f32, max: f32) -> f32 { if v < min { min } else if v > max { max } else { v } }
fn dist(x1: f32, y1: f32, x2: f32, y2: f32) -> f32 { let dx = x2-x1; let dy = y2-y1; sqrt(dx*dx+dy*dy) }
fn norm(x: f32, y: f32) -> (f32, f32) { let l = sqrt(x*x+y*y); if l > 0.0001 { (x/l, y/l) } else { (0.0, 0.0) } }

fn draw_str(s: &[u8], x: f32, y: f32, size: f32, color: u32) {
    unsafe { draw_text(s.as_ptr(), s.len() as u32, x, y, size, color); }
}

fn fmt_num(n: u32, buf: &mut [u8]) -> usize {
    if n == 0 { buf[0] = b'0'; return 1; }
    let mut v = n; let mut i = 0;
    while v > 0 && i < buf.len() { buf[i] = b'0' + (v % 10) as u8; v /= 10; i += 1; }
    for j in 0..i/2 { let t = buf[j]; buf[j] = buf[i-1-j]; buf[i-1-j] = t; }
    i
}

// =============================================================================
// Roguelite: Power-up Choice Generation
// =============================================================================

fn generate_levelup_choices(player_idx: usize) {
    unsafe {
        let p = &PLAYERS[player_idx];
        let mut available: [PowerUpType; 16] = [PowerUpType::Might; 16];
        let mut count = 0usize;

        // Add weapons not at max level
        let weapons = [
            PowerUpType::Cleave, PowerUpType::MagicMissile, PowerUpType::PiercingArrow,
            PowerUpType::HolyNova, PowerUpType::Fireball, PowerUpType::IceShards,
            PowerUpType::LightningBolt, PowerUpType::ShadowOrb
        ];
        for &w in &weapons {
            if let Some(lv) = p.has_powerup(w) {
                if lv < 5 { available[count] = w; count += 1; }
            } else if p.powerup_count < MAX_POWERUPS as u8 {
                available[count] = w; count += 1;
            }
        }

        // Add passives not at max level
        let passives = [
            PowerUpType::Might, PowerUpType::Swiftness, PowerUpType::Vitality,
            PowerUpType::Haste, PowerUpType::Magnetism, PowerUpType::Armor,
            PowerUpType::Luck, PowerUpType::Fury
        ];
        for &pa in &passives {
            if let Some(lv) = p.has_powerup(pa) {
                if lv < 5 { available[count] = pa; count += 1; }
            } else if p.powerup_count < MAX_POWERUPS as u8 {
                available[count] = pa; count += 1;
            }
        }

        // Pick 3 random choices
        if count >= 3 {
            for i in 0..3 {
                let idx = random_range(i as i32, count as i32) as usize;
                let tmp = available[i]; available[i] = available[idx]; available[idx] = tmp;
            }
            LEVELUP_CHOICES[0] = available[0];
            LEVELUP_CHOICES[1] = available[1];
            LEVELUP_CHOICES[2] = available[2];
        } else {
            // Fallback
            LEVELUP_CHOICES = [PowerUpType::Might, PowerUpType::Swiftness, PowerUpType::Vitality];
        }

        LEVELUP_SELECTION = 0;
    }
}

fn apply_powerup(player_idx: usize, ptype: PowerUpType) {
    unsafe {
        let p = &mut PLAYERS[player_idx];
        p.add_powerup(ptype);

        // Apply passive stat boosts
        match ptype {
            PowerUpType::Might => { p.damage_mult += 0.20; }
            PowerUpType::Swiftness => { p.move_speed += 0.6; }
            PowerUpType::Vitality => { p.max_health += 25.0; p.health = p.max_health; }
            PowerUpType::Haste => { p.attack_speed_mult += 0.20; }
            PowerUpType::Magnetism => { p.pickup_range += 0.75; }
            PowerUpType::Armor => { p.armor += 0.15; }
            PowerUpType::Luck => { p.xp_mult += 0.25; }
            PowerUpType::Fury => { p.damage_mult += 0.05; } // Base fury bonus
            _ => {}
        }
    }
}

// =============================================================================
// Game Initialization
// =============================================================================

// Player count scaling - more players = harder enemies
fn player_scale_hp() -> f32 {
    unsafe {
        match ACTIVE_PLAYERS {
            1 => 1.0,
            2 => 1.5,   // +50% HP for 2 players
            3 => 2.0,   // +100% HP for 3 players
            _ => 2.5,   // +150% HP for 4 players
        }
    }
}

fn player_scale_spawn() -> f32 {
    unsafe {
        match ACTIVE_PLAYERS {
            1 => 1.0,
            2 => 1.3,   // +30% spawn rate for 2 players
            3 => 1.6,   // +60% spawn rate for 3 players
            _ => 2.0,   // +100% spawn rate for 4 players
        }
    }
}

fn init_run() {
    unsafe {
        PHASE = GamePhase::Playing;
        WAVE = 1; STAGE = Stage::CrystalCavern;
        GAME_TIME = 0.0; STAGE_TIME = 0.0; SPAWN_TIMER = 0.0; KILLS = 0;
        ELITE_SPAWN_TIMER = 0.0; BOSS_SPAWNED_THIS_WAVE = false;
        ACTIVE_BOSS = None;
        CAM_X = 0.0; CAM_Y = 0.0;

        // Cache player count and reset combo
        ACTIVE_PLAYERS = player_count().min(MAX_PLAYERS as u32);
        COMBO_COUNT = 0; COMBO_TIMER = 0.0; COMBO_MULT = 1.0; MAX_COMBO = 0;
        HAZARD_TIMER = 0.0; HAZARD_ACTIVE = false;

        // Apply selected difficulty
        DIFFICULTY = match DIFFICULTY_SELECTION {
            0 => Difficulty::Easy,
            1 => Difficulty::Normal,
            2 => Difficulty::Hard,
            _ => Difficulty::Nightmare,
        };

        for e in &mut ENEMIES { e.active = false; }
        for g in &mut XP_GEMS { g.active = false; }
        for pr in &mut PROJECTILES { pr.active = false; }

        let cnt = ACTIVE_PLAYERS as usize;
        let spawns = [(-2.0, -2.0), (2.0, -2.0), (-2.0, 2.0), (2.0, 2.0)];
        let classes = [HeroClass::Knight, HeroClass::Mage, HeroClass::Ranger, HeroClass::Cleric];
        let starters = [PowerUpType::Cleave, PowerUpType::MagicMissile, PowerUpType::PiercingArrow, PowerUpType::HolyNova];

        for i in 0..MAX_PLAYERS {
            PLAYERS[i] = Player::new();
            if i < cnt {
                PLAYERS[i].active = true;
                PLAYERS[i].x = spawns[i].0;
                PLAYERS[i].y = spawns[i].1;
                PLAYERS[i].class = classes[i];
                PLAYERS[i].add_powerup(starters[i]);

                // Class base stats
                match classes[i] {
                    HeroClass::Knight => {
                        PLAYERS[i].max_health = 150.0; PLAYERS[i].health = 150.0;
                        PLAYERS[i].armor = 0.1; PLAYERS[i].move_speed = 3.5;
                    }
                    HeroClass::Mage => {
                        PLAYERS[i].max_health = 80.0; PLAYERS[i].health = 80.0;
                        PLAYERS[i].damage_mult = 1.3; PLAYERS[i].move_speed = 4.0;
                    }
                    HeroClass::Ranger => {
                        PLAYERS[i].max_health = 100.0; PLAYERS[i].health = 100.0;
                        PLAYERS[i].attack_speed_mult = 1.3; PLAYERS[i].move_speed = 5.0;
                    }
                    HeroClass::Cleric => {
                        PLAYERS[i].max_health = 120.0; PLAYERS[i].health = 120.0;
                        PLAYERS[i].pickup_range = 2.5; PLAYERS[i].move_speed = 3.8;
                    }
                    HeroClass::Necromancer => {
                        PLAYERS[i].max_health = 90.0; PLAYERS[i].health = 90.0;
                        PLAYERS[i].damage_mult = 1.2; PLAYERS[i].move_speed = 3.6;
                    }
                    HeroClass::Paladin => {
                        PLAYERS[i].max_health = 180.0; PLAYERS[i].health = 180.0;
                        PLAYERS[i].armor = 0.2; PLAYERS[i].move_speed = 3.0;
                    }
                }
            }
        }
    }
}

// =============================================================================
// Spawning
// =============================================================================

fn spawn_enemy() {
    unsafe {
        let mut slot = None;
        for (i, e) in ENEMIES.iter().enumerate() { if !e.active { slot = Some(i); break; } }
        if let Some(i) = slot {
            let angle = random_f32() * 6.28318;
            let d = ARENA_SIZE + 3.0;

            ENEMIES[i] = Enemy::new();
            ENEMIES[i].active = true;
            ENEMIES[i].x = cos(angle) * d;
            ENEMIES[i].y = sin(angle) * d;

            // Type based on wave/stage (higher waves unlock more enemy types)
            let roll = random_range(0, 100);
            let wave_tier = WAVE.min(10);
            ENEMIES[i].enemy_type = if roll < (50 - wave_tier * 4) as i32 { EnemyType::Crawler }
                else if roll < (70 - wave_tier * 3) as i32 { EnemyType::Skeleton }
                else if roll < (80 - wave_tier * 2) as i32 { EnemyType::Wisp }
                else if WAVE >= 3 && roll < 85 { EnemyType::Shade }      // Fast, unlocks wave 3
                else if WAVE >= 5 && roll < 90 { EnemyType::Berserker }  // Strong, unlocks wave 5
                else if WAVE >= 7 && roll < 95 { EnemyType::ArcaneSentinel } // Magic, unlocks wave 7
                else { EnemyType::Golem };

            // Stats scale with wave, difficulty, and player count
            let wm = 1.0 + (WAVE as f32 - 1.0) * 0.12;
            let hp_scale = wm * DIFFICULTY.enemy_hp_mult() * player_scale_hp();
            let dmg_scale = wm * DIFFICULTY.enemy_damage_mult();
            match ENEMIES[i].enemy_type {
                EnemyType::Crawler => { ENEMIES[i].health = 8.0*hp_scale; ENEMIES[i].damage = 5.0*dmg_scale; ENEMIES[i].speed = 3.2; }
                EnemyType::Skeleton => { ENEMIES[i].health = 15.0*hp_scale; ENEMIES[i].damage = 8.0*dmg_scale; ENEMIES[i].speed = 2.0; }
                EnemyType::Wisp => { ENEMIES[i].health = 6.0*hp_scale; ENEMIES[i].damage = 12.0*dmg_scale; ENEMIES[i].speed = 2.4; }
                EnemyType::Golem => { ENEMIES[i].health = 45.0*hp_scale; ENEMIES[i].damage = 15.0*dmg_scale; ENEMIES[i].speed = 1.0; }
                EnemyType::Shade => { ENEMIES[i].health = 10.0*hp_scale; ENEMIES[i].damage = 8.0*dmg_scale; ENEMIES[i].speed = 4.0; }
                EnemyType::Berserker => { ENEMIES[i].health = 35.0*hp_scale; ENEMIES[i].damage = 20.0*dmg_scale; ENEMIES[i].speed = 2.5; }
                EnemyType::ArcaneSentinel => { ENEMIES[i].health = 25.0*hp_scale; ENEMIES[i].damage = 15.0*dmg_scale; ENEMIES[i].speed = 1.5; }
                _ => {} // Elites and bosses use dedicated spawn functions
            }
            ENEMIES[i].max_health = ENEMIES[i].health;
        }
    }
}

fn spawn_elite() {
    unsafe {
        let mut slot = None;
        for (i, e) in ENEMIES.iter().enumerate() { if !e.active { slot = Some(i); break; } }
        if let Some(i) = slot {
            let angle = random_f32() * 6.28318;
            let d = ARENA_SIZE + 5.0;

            ENEMIES[i] = Enemy::new();
            ENEMIES[i].active = true;
            ENEMIES[i].tier = EnemyTier::Elite;
            ENEMIES[i].x = cos(angle) * d;
            ENEMIES[i].y = sin(angle) * d;

            // Pick random elite type
            let roll = random_range(0, 4);
            ENEMIES[i].enemy_type = match roll {
                0 => EnemyType::CrystalKnight,
                1 => EnemyType::VoidMage,
                2 => EnemyType::GolemTitan,
                _ => EnemyType::SpecterLord,
            };

            // Elite stats: 3x health, 2x damage, 0.8x speed (scaled by difficulty + players)
            let wm = 1.0 + (WAVE as f32 - 1.0) * 0.15;
            let hp_scale = wm * DIFFICULTY.enemy_hp_mult() * player_scale_hp();
            let dmg_scale = wm * DIFFICULTY.enemy_damage_mult();
            match ENEMIES[i].enemy_type {
                EnemyType::CrystalKnight => {
                    ENEMIES[i].health = 120.0 * hp_scale;
                    ENEMIES[i].damage = 25.0 * dmg_scale;
                    ENEMIES[i].speed = 1.8;
                }
                EnemyType::VoidMage => {
                    ENEMIES[i].health = 80.0 * hp_scale;
                    ENEMIES[i].damage = 35.0 * dmg_scale;
                    ENEMIES[i].speed = 1.5;
                }
                EnemyType::GolemTitan => {
                    ENEMIES[i].health = 200.0 * hp_scale;
                    ENEMIES[i].damage = 30.0 * dmg_scale;
                    ENEMIES[i].speed = 0.8;
                }
                EnemyType::SpecterLord => {
                    ENEMIES[i].health = 100.0 * hp_scale;
                    ENEMIES[i].damage = 20.0 * dmg_scale;
                    ENEMIES[i].speed = 2.5;
                }
                _ => {}
            }
            ENEMIES[i].max_health = ENEMIES[i].health;
        }
    }
}

fn spawn_boss(boss_type: EnemyType) {
    unsafe {
        // Only one boss at a time
        if ACTIVE_BOSS.is_some() { return; }

        let mut slot = None;
        for (i, e) in ENEMIES.iter().enumerate() { if !e.active { slot = Some(i); break; } }
        if let Some(i) = slot {
            ENEMIES[i] = Enemy::new();
            ENEMIES[i].active = true;
            ENEMIES[i].tier = EnemyTier::Boss;
            ENEMIES[i].enemy_type = boss_type;

            // Spawn opposite to average player position
            let mut px = 0.0f32; let mut py = 0.0f32; let mut cnt = 0;
            for p in &PLAYERS { if p.active && !p.dead { px += p.x; py += p.y; cnt += 1; } }
            if cnt > 0 { px /= cnt as f32; py /= cnt as f32; }
            let angle = atan2(-py, -px);
            ENEMIES[i].x = cos(angle) * (ARENA_SIZE + 8.0);
            ENEMIES[i].y = sin(angle) * (ARENA_SIZE + 8.0);

            // Boss stats: massive health, high damage, slow (scaled by difficulty + players)
            let wm = 1.0 + (WAVE as f32 - 1.0) * 0.1;
            let hp_scale = wm * DIFFICULTY.enemy_hp_mult() * player_scale_hp();
            let dmg_scale = wm * DIFFICULTY.enemy_damage_mult();
            match boss_type {
                EnemyType::PrismColossus => {
                    ENEMIES[i].health = 800.0 * hp_scale;
                    ENEMIES[i].damage = 40.0 * dmg_scale;
                    ENEMIES[i].speed = 1.0;
                }
                EnemyType::VoidDragon => {
                    ENEMIES[i].health = 600.0 * hp_scale;
                    ENEMIES[i].damage = 50.0 * dmg_scale;
                    ENEMIES[i].speed = 1.5;
                }
                _ => {}
            }
            ENEMIES[i].max_health = ENEMIES[i].health;
            ACTIVE_BOSS = Some(i);
        }
    }
}

fn spawn_xp(x: f32, y: f32, val: u32) {
    unsafe {
        for g in &mut XP_GEMS {
            if !g.active { g.active = true; g.x = x; g.y = y; g.value = val; g.phase = random_f32() * 6.28; break; }
        }
    }
}

fn spawn_proj(x: f32, y: f32, vx: f32, vy: f32, dmg: f32, pierce: bool, owner: u8) {
    unsafe {
        for pr in &mut PROJECTILES {
            if !pr.active {
                pr.active = true; pr.x = x; pr.y = y; pr.vx = vx; pr.vy = vy;
                pr.damage = dmg; pr.lifetime = 2.5; pr.piercing = pierce; pr.owner = owner;
                break;
            }
        }
    }
}

fn nearest_enemy(x: f32, y: f32) -> Option<usize> {
    unsafe {
        let mut best: Option<usize> = None; let mut bd = f32::MAX;
        for (i, e) in ENEMIES.iter().enumerate() {
            if e.active { let d = dist(x, y, e.x, e.y); if d < bd { bd = d; best = Some(i); } }
        }
        best
    }
}

// =============================================================================
// Weapon Firing
// =============================================================================

fn fire_weapons(player_idx: usize) {
    unsafe {
        let p = &mut PLAYERS[player_idx];
        if !p.active || p.dead { return; }

        let dt = delta_time();

        // Pre-calculate fury bonus before mutable iteration
        let has_fury = p.has_powerup(PowerUpType::Fury).is_some();
        let fury_bonus = if has_fury {
            let missing = 1.0 - (p.health / p.max_health);
            missing * 0.5
        } else { 0.0 };

        // Pre-calculate synergies
        let has_steam = p.has_powerup(PowerUpType::Fireball).is_some() &&
                        p.has_powerup(PowerUpType::IceShards).is_some();
        let has_chain = p.has_powerup(PowerUpType::MagicMissile).is_some() &&
                        p.has_powerup(PowerUpType::LightningBolt).is_some();
        let has_whirlwind = p.has_powerup(PowerUpType::Cleave).is_some() &&
                            p.has_powerup(PowerUpType::ShadowOrb).is_some();

        for pu_opt in &mut p.powerups {
            if let Some(pu) = pu_opt {
                pu.timer -= dt * p.attack_speed_mult;
                if pu.timer > 0.0 { continue; }
                pu.timer = pu.cooldown;

                let base_dmg = 10.0 * p.damage_mult * (1.0 + pu.level as f32 * 0.15);
                let dmg = base_dmg * (1.0 + fury_bonus);

                match pu.ptype {
                    PowerUpType::Cleave => {
                        // Melee arc - damage all nearby enemies in front
                        // Whirlwind synergy: larger range, spinning attack
                        let range = if has_whirlwind {
                            3.5 + pu.level as f32 * 0.4  // +50% range
                        } else {
                            2.0 + pu.level as f32 * 0.3
                        };
                        let synergy_dmg = if has_whirlwind { dmg * 1.3 } else { dmg };

                        for e in &mut ENEMIES {
                            if e.active && dist(p.x, p.y, e.x, e.y) < range {
                                e.health -= synergy_dmg; e.hit_timer = 0.1;
                                play_sound(SFX_HIT, 0.2, 0.0);
                                if e.health <= 0.0 { e.active = false; KILLS += 1; spawn_xp(e.x, e.y, 2); }
                            }
                        }
                    }
                    PowerUpType::MagicMissile | PowerUpType::PiercingArrow | PowerUpType::Fireball | PowerUpType::IceShards | PowerUpType::LightningBolt => {
                        if let Some(ei) = nearest_enemy(p.x, p.y) {
                            let e = &ENEMIES[ei];
                            let (nx, ny) = norm(e.x - p.x, e.y - p.y);
                            p.facing_angle = atan2(ny, nx) * 57.3;
                            let speed = 14.0;
                            let pierce = pu.ptype == PowerUpType::PiercingArrow;

                            // Chain Lightning synergy: extra damage and chain to nearby enemies
                            let synergy_dmg = if has_chain && (pu.ptype == PowerUpType::MagicMissile || pu.ptype == PowerUpType::LightningBolt) {
                                dmg * 1.5  // +50% damage
                            } else {
                                dmg
                            };

                            spawn_proj(p.x + nx * 0.5, p.y + ny * 0.5, nx * speed, ny * speed, synergy_dmg, pierce, player_idx as u8);
                            play_sound(SFX_SHOOT, 0.25, 0.0);

                            // Chain Lightning synergy: spawn extra projectile to second nearest enemy
                            if has_chain && (pu.ptype == PowerUpType::MagicMissile || pu.ptype == PowerUpType::LightningBolt) {
                                // Find second nearest enemy
                                let mut second_best: Option<usize> = None;
                                let mut second_dist = f32::MAX;
                                for (j, e2) in ENEMIES.iter().enumerate() {
                                    if e2.active && j != ei {
                                        let d = dist(p.x, p.y, e2.x, e2.y);
                                        if d < second_dist && d < 15.0 {
                                            second_dist = d;
                                            second_best = Some(j);
                                        }
                                    }
                                }
                                if let Some(j) = second_best {
                                    let e2 = &ENEMIES[j];
                                    let (nx2, ny2) = norm(e2.x - p.x, e2.y - p.y);
                                    spawn_proj(p.x, p.y, nx2 * speed, ny2 * speed, synergy_dmg * 0.7, false, player_idx as u8);
                                }
                            }

                            // Ice shards: multi-shot
                            if pu.ptype == PowerUpType::IceShards {
                                let angle = atan2(ny, nx);
                                // Steam Explosion synergy: 5 shards instead of 3, larger AoE damage
                                let offsets: &[f32] = if has_steam {
                                    &[-0.5, -0.25, 0.0, 0.25, 0.5]
                                } else {
                                    &[-0.3, 0.0, 0.3]
                                };
                                for off in offsets {
                                    let a2 = angle + off;
                                    let shard_dmg = if has_steam { dmg * 0.8 } else { dmg * 0.6 };
                                    spawn_proj(p.x, p.y, cos(a2) * speed, sin(a2) * speed, shard_dmg, false, player_idx as u8);
                                }
                            }

                            // Steam Explosion synergy for Fireball: larger AoE
                            if pu.ptype == PowerUpType::Fireball && has_steam {
                                // Extra damage to nearby enemies (steam explosion)
                                let ex = ENEMIES[ei].x;
                                let ey = ENEMIES[ei].y;
                                for e2 in &mut ENEMIES {
                                    if e2.active && dist(ex, ey, e2.x, e2.y) < 3.0 {
                                        e2.health -= dmg * 0.4; e2.hit_timer = 0.1;
                                        if e2.health <= 0.0 { e2.active = false; KILLS += 1; spawn_xp(e2.x, e2.y, 2); }
                                    }
                                }
                            }
                        }
                    }
                    PowerUpType::HolyNova => {
                        // Radial burst
                        for e in &mut ENEMIES {
                            if e.active && dist(p.x, p.y, e.x, e.y) < 3.5 + pu.level as f32 * 0.5 {
                                e.health -= dmg * 0.7; e.hit_timer = 0.1;
                                if e.health <= 0.0 { e.active = false; KILLS += 1; spawn_xp(e.x, e.y, 2); }
                            }
                        }
                        play_sound(SFX_SHOOT, 0.3, 0.0);
                    }
                    PowerUpType::ShadowOrb => {
                        // Damage enemies in orbit
                        // Whirlwind synergy: multiple orbs
                        let orbit_r = 2.5;
                        let orb_angle = GAME_TIME * 3.0;  // Use GAME_TIME for rollback safety
                        let orb_count = if has_whirlwind { 3 } else { 1 };
                        let synergy_dmg = if has_whirlwind { dmg * 0.4 } else { dmg * 0.5 };

                        for orb_i in 0..orb_count {
                            let angle_offset = (orb_i as f32) * (6.28 / orb_count as f32);
                            let ox = p.x + cos(orb_angle + angle_offset) * orbit_r;
                            let oy = p.y + sin(orb_angle + angle_offset) * orbit_r;
                            for e in &mut ENEMIES {
                                if e.active && dist(ox, oy, e.x, e.y) < 1.0 {
                                    e.health -= synergy_dmg; e.hit_timer = 0.05;
                                    if e.health <= 0.0 { e.active = false; KILLS += 1; spawn_xp(e.x, e.y, 2); }
                                }
                            }
                        }
                    }
                    _ => {} // Passives don't fire
                }
            }
        }
    }
}

// =============================================================================
// Update Logic
// =============================================================================

const REVIVE_TIME: f32 = 3.0;   // Seconds to revive
const REVIVE_RANGE: f32 = 2.0;  // Units distance
const REVIVE_HEALTH: f32 = 0.5; // Percentage of max HP
const REVIVE_INVULN: f32 = 1.0; // Seconds of invincibility

fn update_revives() {
    unsafe {
        let dt = delta_time();

        // For each dead player, check if anyone is reviving them
        for di in 0..MAX_PLAYERS {
            if !PLAYERS[di].active || !PLAYERS[di].dead { continue; }

            let dx = PLAYERS[di].x;
            let dy = PLAYERS[di].y;

            // Check if current reviver is still valid
            let current_reviver = PLAYERS[di].reviving_by;
            if current_reviver >= 0 {
                let ri = current_reviver as usize;
                if ri >= MAX_PLAYERS || !PLAYERS[ri].active || PLAYERS[ri].dead
                    || dist(PLAYERS[ri].x, PLAYERS[ri].y, dx, dy) > REVIVE_RANGE {
                    // Reviver left or died, reset progress
                    PLAYERS[di].reviving_by = -1;
                    PLAYERS[di].revive_progress = 0.0;
                }
            }

            // Find closest living player in range
            if PLAYERS[di].reviving_by < 0 {
                let mut closest: Option<usize> = None;
                let mut closest_dist = REVIVE_RANGE + 1.0;

                for ri in 0..MAX_PLAYERS {
                    if ri == di { continue; }
                    if !PLAYERS[ri].active || PLAYERS[ri].dead { continue; }
                    let d = dist(PLAYERS[ri].x, PLAYERS[ri].y, dx, dy);
                    if d <= REVIVE_RANGE && d < closest_dist {
                        closest = Some(ri);
                        closest_dist = d;
                    }
                }

                if let Some(ri) = closest {
                    PLAYERS[di].reviving_by = ri as i8;
                }
            }

            // Progress revive
            if PLAYERS[di].reviving_by >= 0 {
                PLAYERS[di].revive_progress += dt;

                if PLAYERS[di].revive_progress >= REVIVE_TIME {
                    // Revive successful!
                    PLAYERS[di].dead = false;
                    PLAYERS[di].health = PLAYERS[di].max_health * REVIVE_HEALTH;
                    PLAYERS[di].invuln_timer = REVIVE_INVULN;
                    PLAYERS[di].revive_progress = 0.0;
                    PLAYERS[di].reviving_by = -1;
                    play_sound(SFX_LEVELUP, 0.4, 0.0);  // Use levelup sound for revive
                }
            }
        }
    }
}

fn update_players() {
    unsafe {
        let dt = delta_time();

        // Handle revives first
        update_revives();

        for i in 0..MAX_PLAYERS {
            let p = &mut PLAYERS[i];
            if !p.active || p.dead { continue; }

            if p.invuln_timer > 0.0 { p.invuln_timer -= dt; }

            // Movement
            let sx = left_stick_x(i as u32);
            let sy = left_stick_y(i as u32);
            let len = sqrt(sx*sx + sy*sy);
            if len > 0.1 {
                let (nx, ny) = norm(sx, sy);
                p.vx = nx * p.move_speed;
                p.vy = ny * p.move_speed;
            } else {
                p.vx *= 0.8; p.vy *= 0.8;
            }
            p.x += p.vx * dt; p.y += p.vy * dt;
            p.x = clamp(p.x, -ARENA_SIZE, ARENA_SIZE);
            p.y = clamp(p.y, -ARENA_SIZE, ARENA_SIZE);

            // Weapons
            fire_weapons(i);

            // XP collection (difficulty and combo multiplier)
            for g in &mut XP_GEMS {
                if g.active && dist(p.x, p.y, g.x, g.y) < p.pickup_range {
                    g.active = false;
                    let gained = (g.value as f32 * p.xp_mult * DIFFICULTY.xp_mult() * COMBO_MULT) as u32;
                    p.xp += gained.max(1);
                    play_sound(SFX_XP, 0.15, 0.0);

                    // Level up check (now supports infinite levels with exponential scaling)
                    let xp_needed = xp_for_level(p.level);
                    if p.xp >= xp_needed {
                        p.xp -= xp_needed;
                        p.level += 1;
                        PHASE = GamePhase::LevelUp;
                        LEVELUP_PLAYER = i;
                        generate_levelup_choices(i);
                        play_sound(SFX_LEVELUP, 0.5, 0.0);
                    }
                }
            }
        }
    }
}

fn update_enemies() {
    unsafe {
        let dt = delta_time();

        for e in &mut ENEMIES {
            if !e.active { continue; }
            if e.hit_timer > 0.0 { e.hit_timer -= dt; }

            // Chase nearest player
            let mut tx = 0.0f32; let mut ty = 0.0f32; let mut bd = f32::MAX;
            for p in &PLAYERS {
                if p.active && !p.dead {
                    let d = dist(e.x, e.y, p.x, p.y);
                    if d < bd { bd = d; tx = p.x; ty = p.y; }
                }
            }

            if bd < 100.0 {
                let (nx, ny) = norm(tx - e.x, ty - e.y);
                e.vx = nx * e.speed; e.vy = ny * e.speed;
                e.x += e.vx * dt; e.y += e.vy * dt;
            }

            // Damage players
            for p in &mut PLAYERS {
                if p.active && !p.dead && p.invuln_timer <= 0.0 && dist(e.x, e.y, p.x, p.y) < 0.8 {
                    let dmg_taken = e.damage * (1.0 - p.armor);
                    p.health -= dmg_taken;
                    p.invuln_timer = 0.5;
                    play_sound(SFX_HURT, 0.4, 0.0);

                    let (kx, ky) = norm(p.x - e.x, p.y - e.y);
                    p.x += kx * 1.5; p.y += ky * 1.5;

                    if p.health <= 0.0 { p.dead = true; }
                }
            }
        }
    }
}

fn update_projectiles() {
    unsafe {
        let dt = delta_time();

        for pr in &mut PROJECTILES {
            if !pr.active { continue; }
            pr.lifetime -= dt;
            if pr.lifetime <= 0.0 { pr.active = false; continue; }

            pr.x += pr.vx * dt; pr.y += pr.vy * dt;

            for e in &mut ENEMIES {
                if e.active && dist(pr.x, pr.y, e.x, e.y) < 0.7 {
                    e.health -= pr.damage; e.hit_timer = 0.1;
                    play_sound(SFX_HIT, 0.2, 0.0);
                    if !pr.piercing { pr.active = false; }
                    if e.health <= 0.0 {
                        e.active = false; KILLS += 1;

                        // Combo system - extend timer and increase count
                        COMBO_COUNT += 1;
                        COMBO_TIMER = COMBO_WINDOW + (COMBO_COUNT as f32 * COMBO_DECAY).min(3.0);
                        if COMBO_COUNT > MAX_COMBO { MAX_COMBO = COMBO_COUNT; }
                        // Combo multiplier: 1.0 base, +10% per 5 kills, capped at 2.5x
                        COMBO_MULT = (1.0 + (COMBO_COUNT / 5) as f32 * 0.1).min(2.5);

                        let xp_val = match e.enemy_type {
                            // Basic enemies
                            EnemyType::Crawler => 1, EnemyType::Skeleton => 2,
                            EnemyType::Wisp => 2, EnemyType::Golem => 5,
                            EnemyType::Shade => 2, EnemyType::Berserker => 4,
                            EnemyType::ArcaneSentinel => 3,
                            // Elites (more XP)
                            EnemyType::CrystalKnight => 15, EnemyType::VoidMage => 12,
                            EnemyType::GolemTitan => 20, EnemyType::SpecterLord => 15,
                            // Bosses (lots of XP)
                            EnemyType::PrismColossus => 100, EnemyType::VoidDragon => 100,
                        };
                        // Spawn multiple XP gems for elites/bosses
                        if e.tier == EnemyTier::Boss {
                            for j in 0..10 {
                                let angle = (j as f32) * 0.628;
                                spawn_xp(e.x + cos(angle) * 0.5, e.y + sin(angle) * 0.5, xp_val / 10);
                            }
                        } else if e.tier == EnemyTier::Elite {
                            for j in 0..4 {
                                let angle = (j as f32) * 1.57;
                                spawn_xp(e.x + cos(angle) * 0.3, e.y + sin(angle) * 0.3, xp_val / 4);
                            }
                        } else {
                            spawn_xp(e.x, e.y, xp_val);
                        }
                        play_sound(SFX_DEATH, 0.25, 0.0);
                    }
                    if !pr.piercing { break; }
                }
            }
        }
    }
}

fn update_combo() {
    unsafe {
        if COMBO_COUNT > 0 {
            COMBO_TIMER -= delta_time();
            if COMBO_TIMER <= 0.0 {
                COMBO_COUNT = 0;
                COMBO_MULT = 1.0;
            }
        }
    }
}

fn update_hazards() {
    unsafe {
        let dt = delta_time();
        HAZARD_TIMER -= dt;

        // Spawn new hazard based on stage
        if !HAZARD_ACTIVE && HAZARD_TIMER <= 0.0 && WAVE >= 3 {
            HAZARD_ACTIVE = true;
            // Hazard spawns in random location within arena
            let angle = random_f32() * 6.28318;
            let r = random_f32() * (ARENA_SIZE - 3.0);
            HAZARD_X = cos(angle) * r;
            HAZARD_Y = sin(angle) * r;
            HAZARD_RADIUS = match STAGE {
                Stage::CrystalCavern => 2.5,    // Crystal shards
                Stage::EnchantedForest => 3.0,  // Poison cloud
                Stage::VolcanicDepths => 3.5,   // Lava pool
                Stage::VoidRealm => 4.0,        // Void rift
            };
            HAZARD_TIMER = 5.0;  // Hazard lasts 5 seconds
        }

        // Hazard damages players and enemies
        if HAZARD_ACTIVE {
            let dmg = match STAGE {
                Stage::CrystalCavern => 5.0,
                Stage::EnchantedForest => 8.0,
                Stage::VolcanicDepths => 12.0,
                Stage::VoidRealm => 15.0,
            } * dt;

            // Damage players in hazard
            for p in &mut PLAYERS {
                if p.active && !p.dead && p.invuln_timer <= 0.0 {
                    if dist(p.x, p.y, HAZARD_X, HAZARD_Y) < HAZARD_RADIUS {
                        p.health -= dmg * (1.0 - p.armor);
                        if p.health <= 0.0 { p.dead = true; }
                    }
                }
            }

            // Damage enemies in hazard (player benefit!)
            for e in &mut ENEMIES {
                if e.active && dist(e.x, e.y, HAZARD_X, HAZARD_Y) < HAZARD_RADIUS {
                    e.health -= dmg * 2.0;  // Enemies take more damage
                    if e.health <= 0.0 { e.active = false; KILLS += 1; }
                }
            }

            // Deactivate after timer
            if HAZARD_TIMER <= 0.0 {
                HAZARD_ACTIVE = false;
                HAZARD_TIMER = 15.0 + random_f32() * 10.0;  // 15-25s until next hazard
            }
        }
    }
}

fn update_spawning() {
    unsafe {
        let dt = delta_time();
        GAME_TIME += dt;
        STAGE_TIME += dt;
        SPAWN_TIMER -= dt;
        ELITE_SPAWN_TIMER -= dt;

        // Track previous wave for boss spawn detection
        let prev_wave = WAVE;

        // Wave progression
        if STAGE_TIME > 30.0 * WAVE as f32 && WAVE < 10 {
            WAVE += 1;
            BOSS_SPAWNED_THIS_WAVE = false;  // Reset boss flag for new wave
        }

        // Stage progression
        if STAGE_TIME > STAGE_DURATION {
            STAGE_TIME = 0.0;
            STAGE = match STAGE {
                Stage::CrystalCavern => Stage::EnchantedForest,
                Stage::EnchantedForest => Stage::VolcanicDepths,
                Stage::VolcanicDepths => Stage::VoidRealm,
                Stage::VoidRealm => { PHASE = GamePhase::Victory; return; }
            };
            WAVE = 1;
            BOSS_SPAWNED_THIS_WAVE = false;
        }

        // Spawn regular enemies (scaled by difficulty and player count)
        let base_interval = (2.0 - (WAVE as f32 - 1.0) * 0.12).max(0.25);
        let interval = base_interval / (DIFFICULTY.spawn_rate_mult() * player_scale_spawn());
        if SPAWN_TIMER <= 0.0 {
            SPAWN_TIMER = interval;
            let count = 1 + WAVE.min(3);
            for _ in 0..count { spawn_enemy(); }
        }

        // Spawn elites (difficulty determines start wave, +1 per 5 waves)
        let elite_wave = DIFFICULTY.elite_start_wave();
        if WAVE >= elite_wave && ELITE_SPAWN_TIMER <= 0.0 {
            ELITE_SPAWN_TIMER = 8.0 / DIFFICULTY.spawn_rate_mult();  // Scale with difficulty
            let elite_count = 1 + (WAVE - elite_wave) / 5;
            for _ in 0..elite_count.min(3) {
                spawn_elite();
            }
        }

        // Spawn boss every 10 waves (alternating types)
        if WAVE % 10 == 0 && !BOSS_SPAWNED_THIS_WAVE && WAVE > prev_wave {
            BOSS_SPAWNED_THIS_WAVE = true;
            // Alternate between Prism Colossus and Void Dragon
            let boss_type = if (WAVE / 10) % 2 == 1 {
                EnemyType::PrismColossus
            } else {
                EnemyType::VoidDragon
            };
            spawn_boss(boss_type);
        }

        // Clear boss tracker when boss dies
        if let Some(i) = ACTIVE_BOSS {
            if !ENEMIES[i].active {
                ACTIVE_BOSS = None;
            }
        }
    }
}

fn update_camera() {
    unsafe {
        let mut sx = 0.0f32; let mut sy = 0.0f32; let mut cnt = 0;
        for p in &PLAYERS { if p.active && !p.dead { sx += p.x; sy += p.y; cnt += 1; } }
        if cnt > 0 {
            let tx = sx / cnt as f32; let ty = sy / cnt as f32;
            CAM_X += (tx - CAM_X) * 5.0 * delta_time();
            CAM_Y += (ty - CAM_Y) * 5.0 * delta_time();
        }
    }
}

fn check_game_over() {
    unsafe {
        let mut all_dead = true;
        for p in &PLAYERS { if p.active && !p.dead { all_dead = false; break; } }
        if all_dead { PHASE = GamePhase::GameOver; }
    }
}

// =============================================================================
// Entry Points
// =============================================================================

#[no_mangle]
pub extern "C" fn init() {
    unsafe {
        render_mode(3);
        set_tick_rate(2);
        set_clear_color(0x1a1a2eFF);
        depth_test(1);
        cull_mode(1);
        texture_filter(0);

        // Load textures - Heroes
        TEX_KNIGHT = rom_texture(b"knight".as_ptr(), 6);
        TEX_MAGE = rom_texture(b"mage".as_ptr(), 4);
        TEX_RANGER = rom_texture(b"ranger".as_ptr(), 6);
        TEX_CLERIC = rom_texture(b"cleric".as_ptr(), 6);
        TEX_NECROMANCER = rom_texture(b"necromancer".as_ptr(), 11);
        TEX_PALADIN = rom_texture(b"paladin".as_ptr(), 7);
        // Load textures - Enemies
        TEX_GOLEM = rom_texture(b"golem".as_ptr(), 5);
        TEX_CRAWLER = rom_texture(b"crawler".as_ptr(), 7);
        TEX_WISP = rom_texture(b"wisp".as_ptr(), 4);
        TEX_SKELETON = rom_texture(b"skeleton".as_ptr(), 8);
        TEX_SHADE = rom_texture(b"shade".as_ptr(), 5);
        TEX_BERSERKER = rom_texture(b"berserker".as_ptr(), 9);
        TEX_ARCANE_SENTINEL = rom_texture(b"arcane_sentinel".as_ptr(), 15);
        // Load textures - Elites
        TEX_CRYSTAL_KNIGHT = rom_texture(b"crystal_knight".as_ptr(), 14);
        TEX_VOID_MAGE = rom_texture(b"void_mage".as_ptr(), 9);
        TEX_GOLEM_TITAN = rom_texture(b"golem_titan".as_ptr(), 11);
        TEX_SPECTER_LORD = rom_texture(b"specter_lord".as_ptr(), 12);
        // Load textures - Bosses
        TEX_PRISM_COLOSSUS = rom_texture(b"prism_colossus".as_ptr(), 14);
        TEX_VOID_DRAGON = rom_texture(b"void_dragon".as_ptr(), 11);
        // Load textures - Other
        TEX_XP = rom_texture(b"xp_gem".as_ptr(), 6);
        TEX_ARENA = rom_texture(b"arena_floor".as_ptr(), 11);

        // Load meshes - Heroes
        MESH_KNIGHT = rom_mesh(b"knight".as_ptr(), 6);
        MESH_MAGE = rom_mesh(b"mage".as_ptr(), 4);
        MESH_RANGER = rom_mesh(b"ranger".as_ptr(), 6);
        MESH_CLERIC = rom_mesh(b"cleric".as_ptr(), 6);
        MESH_NECROMANCER = rom_mesh(b"necromancer".as_ptr(), 11);
        MESH_PALADIN = rom_mesh(b"paladin".as_ptr(), 7);
        // Load meshes - Enemies
        MESH_GOLEM = rom_mesh(b"golem".as_ptr(), 5);
        MESH_CRAWLER = rom_mesh(b"crawler".as_ptr(), 7);
        MESH_WISP = rom_mesh(b"wisp".as_ptr(), 4);
        MESH_SKELETON = rom_mesh(b"skeleton".as_ptr(), 8);
        MESH_SHADE = rom_mesh(b"shade".as_ptr(), 5);
        MESH_BERSERKER = rom_mesh(b"berserker".as_ptr(), 9);
        MESH_ARCANE_SENTINEL = rom_mesh(b"arcane_sentinel".as_ptr(), 15);
        // Load meshes - Elites
        MESH_CRYSTAL_KNIGHT = rom_mesh(b"crystal_knight".as_ptr(), 14);
        MESH_VOID_MAGE = rom_mesh(b"void_mage".as_ptr(), 9);
        MESH_GOLEM_TITAN = rom_mesh(b"golem_titan".as_ptr(), 11);
        MESH_SPECTER_LORD = rom_mesh(b"specter_lord".as_ptr(), 12);
        // Load meshes - Bosses
        MESH_PRISM_COLOSSUS = rom_mesh(b"prism_colossus".as_ptr(), 14);
        MESH_VOID_DRAGON = rom_mesh(b"void_dragon".as_ptr(), 11);
        // Load meshes - Other
        MESH_XP = rom_mesh(b"xp_gem".as_ptr(), 6);
        MESH_ARENA = rom_mesh(b"arena_floor".as_ptr(), 11);
        MESH_PROJ = sphere(0.12, 6, 4);

        // Load sounds
        SFX_SHOOT = rom_sound(b"shoot".as_ptr(), 5);
        SFX_HIT = rom_sound(b"hit".as_ptr(), 3);
        SFX_DEATH = rom_sound(b"death".as_ptr(), 5);
        SFX_XP = rom_sound(b"xp".as_ptr(), 2);
        SFX_LEVELUP = rom_sound(b"level_up".as_ptr(), 8);
        SFX_HURT = rom_sound(b"hurt".as_ptr(), 4);
        SFX_SELECT = rom_sound(b"select".as_ptr(), 6);

        // Load font
        TEX_FONT = rom_texture(b"prism_font".as_ptr(), 10);
        FONT_PRISM = load_font(TEX_FONT, 8, 12, 32, 96);

        // Lighting
        light_set(0, -0.5, -1.0, -0.3);
        light_color(0, 0xFFFFFFFF);
        light_intensity(0, 1.5);
        light_enable(0);

        PHASE = GamePhase::Title;
    }
}

#[no_mangle]
pub extern "C" fn update() {
    unsafe {
        match PHASE {
            GamePhase::Title => {
                TITLE_TIMER += delta_time();
                for i in 0..MAX_PLAYERS {
                    // Difficulty selection with D-pad
                    if button_pressed(i as u32, BUTTON_UP) != 0 {
                        if DIFFICULTY_SELECTION > 0 { DIFFICULTY_SELECTION -= 1; play_sound(SFX_SELECT, 0.3, 0.0); }
                    }
                    if button_pressed(i as u32, BUTTON_DOWN) != 0 {
                        if DIFFICULTY_SELECTION < 3 { DIFFICULTY_SELECTION += 1; play_sound(SFX_SELECT, 0.3, 0.0); }
                    }
                    // Start game
                    if button_pressed(i as u32, BUTTON_A) != 0 || button_pressed(i as u32, BUTTON_START) != 0 {
                        play_sound(SFX_SELECT, 0.5, 0.0);
                        init_run();
                        break;
                    }
                }
            }
            GamePhase::Playing => {
                update_players();
                update_enemies();
                update_projectiles();
                update_combo();
                update_hazards();
                update_spawning();
                update_camera();
                check_game_over();

                for i in 0..MAX_PLAYERS {
                    if button_pressed(i as u32, BUTTON_START) != 0 { PHASE = GamePhase::Paused; break; }
                }
            }
            GamePhase::LevelUp => {
                let pi = LEVELUP_PLAYER as u32;
                if button_pressed(pi, BUTTON_UP) != 0 && LEVELUP_SELECTION > 0 {
                    LEVELUP_SELECTION -= 1; play_sound(SFX_SELECT, 0.3, 0.0);
                }
                if button_pressed(pi, BUTTON_DOWN) != 0 && LEVELUP_SELECTION < 2 {
                    LEVELUP_SELECTION += 1; play_sound(SFX_SELECT, 0.3, 0.0);
                }
                if button_pressed(pi, BUTTON_A) != 0 {
                    apply_powerup(LEVELUP_PLAYER, LEVELUP_CHOICES[LEVELUP_SELECTION]);
                    PHASE = GamePhase::Playing;
                    play_sound(SFX_LEVELUP, 0.4, 0.0);
                }
            }
            GamePhase::Paused => {
                for i in 0..MAX_PLAYERS {
                    if button_pressed(i as u32, BUTTON_START) != 0 { PHASE = GamePhase::Playing; break; }
                }
            }
            GamePhase::GameOver | GamePhase::Victory => {
                for i in 0..MAX_PLAYERS {
                    if button_pressed(i as u32, BUTTON_A) != 0 || button_pressed(i as u32, BUTTON_START) != 0 {
                        PHASE = GamePhase::Title; break;
                    }
                }
            }
            _ => {}
        }
    }
}

#[no_mangle]
pub extern "C" fn render() {
    unsafe {
        // Stage-specific environment (use GAME_TIME for visual sync across clients)
        match STAGE {
            Stage::CrystalCavern => {
                env_gradient(0, 0x1a2a4aFF, 0x2a3a5aFF, 0x101828FF, 0x080810FF, 0.0, 0.0);
                env_scatter(1, 0, 30, 3, 200, 0, 0x8080FFFF, 0x4040AAFF, 20, 50, (GAME_TIME * 20.0) as u32);
            }
            Stage::EnchantedForest => {
                env_gradient(0, 0x102810FF, 0x1a3a1aFF, 0x081808FF, 0x040804FF, 0.0, 0.0);
                env_scatter(1, 0, 40, 2, 150, 0, 0x40FF40FF, 0x208020FF, 15, 40, (GAME_TIME * 15.0) as u32);
            }
            Stage::VolcanicDepths => {
                env_gradient(0, 0x2a1010FF, 0x4a2020FF, 0x200808FF, 0x100404FF, 0.0, 0.0);
                env_scatter(1, 0, 25, 4, 255, 10, 0xFF8040FF, 0xFF4020FF, 30, 60, (GAME_TIME * 25.0) as u32);
            }
            Stage::VoidRealm => {
                env_gradient(0, 0x0a0a1aFF, 0x1a1a3aFF, 0x050510FF, 0x000008FF, 0.0, 0.0);
                env_scatter(1, 0, 50, 2, 180, 5, 0x8040FFFF, 0x4020AAFF, 40, 80, (GAME_TIME * 30.0) as u32);
            }
        }
        draw_env();

        match PHASE {
            GamePhase::Title => render_title(),
            GamePhase::Playing | GamePhase::Paused | GamePhase::LevelUp => {
                render_game();
                render_hud();
                if PHASE == GamePhase::Paused { render_pause(); }
                if PHASE == GamePhase::LevelUp { render_levelup(); }
            }
            GamePhase::GameOver => { render_game(); render_gameover(); }
            GamePhase::Victory => { render_game(); render_victory(); }
            _ => {}
        }
    }
}

// =============================================================================
// Render Helpers
// =============================================================================

fn render_title() {
    unsafe {
        let t = TITLE_TIMER;

        // Animated background gradient
        let phase = (t * 0.3) % 6.28;
        let bg_shift = (sin(phase) * 0.2 + 0.5) as f32;
        env_gradient(0, 0x0a0a18FF, 0x1a1a3eFF, 0x0e0e20FF, 0x000000FF, 0.0, bg_shift);
        env_scatter(0, 1, 40, 2, 100, 0, 0x00FFFFFF, 0xB060FFFF, 30, 3, (t * 10.0) as u32);
        draw_env();

        // Showcase rotating heroes in background
        camera_set(0.0, 3.0, 8.0, 0.0, 0.5, 0.0);
        camera_fov(45.0);

        // Draw rotating showcase of all heroes
        let hero_angle = t * 30.0;
        let radius = 2.5;

        // Knight
        push_identity();
        push_translate(cos(hero_angle * 0.0175) * radius, 0.0, sin(hero_angle * 0.0175) * radius);
        push_rotate_y(hero_angle + 180.0);
        push_scale_uniform(0.9);
        texture_bind(TEX_KNIGHT);
        material_shininess(0.6);
        material_specular(0x8080C0FF);
        draw_mesh(MESH_KNIGHT);

        // Mage (offset 60 degrees)
        push_identity();
        push_translate(cos((hero_angle + 60.0) * 0.0175) * radius, 0.0, sin((hero_angle + 60.0) * 0.0175) * radius);
        push_rotate_y(hero_angle + 60.0 + 180.0);
        push_scale_uniform(0.9);
        texture_bind(TEX_MAGE);
        draw_mesh(MESH_MAGE);

        // Ranger (offset 120 degrees)
        push_identity();
        push_translate(cos((hero_angle + 120.0) * 0.0175) * radius, 0.0, sin((hero_angle + 120.0) * 0.0175) * radius);
        push_rotate_y(hero_angle + 120.0 + 180.0);
        push_scale_uniform(0.9);
        texture_bind(TEX_RANGER);
        draw_mesh(MESH_RANGER);

        // Cleric (offset 180 degrees)
        push_identity();
        push_translate(cos((hero_angle + 180.0) * 0.0175) * radius, 0.0, sin((hero_angle + 180.0) * 0.0175) * radius);
        push_rotate_y(hero_angle + 180.0 + 180.0);
        push_scale_uniform(0.9);
        texture_bind(TEX_CLERIC);
        draw_mesh(MESH_CLERIC);

        // Necromancer (offset 240 degrees)
        push_identity();
        push_translate(cos((hero_angle + 240.0) * 0.0175) * radius, 0.0, sin((hero_angle + 240.0) * 0.0175) * radius);
        push_rotate_y(hero_angle + 240.0 + 180.0);
        push_scale_uniform(0.9);
        texture_bind(TEX_NECROMANCER);
        draw_mesh(MESH_NECROMANCER);

        // Paladin (offset 300 degrees)
        push_identity();
        push_translate(cos((hero_angle + 300.0) * 0.0175) * radius, 0.0, sin((hero_angle + 300.0) * 0.0175) * radius);
        push_rotate_y(hero_angle + 300.0 + 180.0);
        push_scale_uniform(0.9);
        texture_bind(TEX_PALADIN);
        draw_mesh(MESH_PALADIN);

        // HUD layer
        layer(10);

        // Dark overlay for text readability
        draw_rect(0.0, 0.0, SCREEN_WIDTH, 160.0, 0x000000A0);
        draw_rect(0.0, 380.0, SCREEN_WIDTH, 160.0, 0x000000A0);

        // Use custom font
        font_bind(FONT_PRISM);

        // Animated title with wave effect
        let title_y = 50.0 + sin(t * 2.0) * 5.0;
        let title = b"PRISM SURVIVORS";

        // Draw title letter by letter with wave
        for (i, &c) in title.iter().enumerate() {
            let wave = sin(t * 4.0 + i as f32 * 0.4) * 4.0;
            let x = 200.0 + i as f32 * 38.0;
            let y = title_y + wave;
            // Rainbow color per letter
            let hue = (t * 60.0 + i as f32 * 25.0) % 360.0;
            let color = hsv_to_rgb(hue);
            draw_text(&c as *const u8, 1, x, y, 56.0, color);
        }

        // Subtitle
        draw_str(b"ROGUELITE CO-OP SURVIVAL", 280.0, 120.0, 22.0, 0x00FFFFFF);

        // Hero classes info (fade in after 1 second)
        let info_alpha = clamp((t - 1.0) * 2.0, 0.0, 1.0);
        if info_alpha > 0.0 {
            let a = (info_alpha * 255.0) as u32;
            draw_str(b"6 UNIQUE HEROES", 120.0, 390.0, 16.0, 0x00FFFF00 | a);
            draw_str(b"ROGUELITE UPGRADES", 350.0, 390.0, 16.0, 0xB060FF00 | a);
            draw_str(b"4-PLAYER CO-OP", 620.0, 390.0, 16.0, 0xFFFF4000 | a);
        }

        // Difficulty selection
        draw_str(b"DIFFICULTY:", 380.0, 420.0, 16.0, 0xAAAAAAFF);
        let difficulties = [
            (b"EASY" as &[u8], 0x44FF44FF),
            (b"NORMAL" as &[u8], 0xFFFF44FF),
            (b"HARD" as &[u8], 0xFF8844FF),
            (b"NIGHTMARE" as &[u8], 0xFF4444FF),
        ];

        let base_x = 280.0;
        for (i, (name, color)) in difficulties.iter().enumerate() {
            let x = base_x + i as f32 * 110.0;
            let y = 440.0;
            let selected = i as u8 == DIFFICULTY_SELECTION;

            // Selection indicator
            if selected {
                let pulse = (sin(t * 6.0) + 1.0) * 0.3 + 0.7;
                let c = (*color & 0xFFFFFF00) | ((pulse * 255.0) as u32);
                draw_str(b">", x - 12.0, y, 20.0, c);
                draw_text(name.as_ptr(), name.len() as u32, x, y, 20.0, c);
            } else {
                draw_text(name.as_ptr(), name.len() as u32, x, y, 16.0, (*color & 0xFFFFFF00) | 0x88);
            }
        }

        // Instructions
        draw_str(b"UP/DOWN: SELECT DIFFICULTY", 320.0, 470.0, 14.0, 0x888888FF);

        // Blinking "Press Start" text
        let blink = (sin(t * 4.0) + 1.0) * 0.5;
        let alpha = ((0.3 + blink * 0.7) * 255.0) as u32;
        draw_str(b"PRESS A OR START", 340.0, 495.0, 24.0, 0xFFFFFF00 | alpha);

        // Player count
        let mut buf = [0u8; 16];
        buf[0..9].copy_from_slice(b"PLAYERS: ");
        buf[9] = b'0' + player_count() as u8;
        draw_text(buf.as_ptr(), 10, 780.0, 500.0, 16.0, 0x88FF88FF);

        // Reset font
        font_bind(0);
    }
}

// HSV to RGB for rainbow effect (H in degrees, returns RGBA u32)
fn hsv_to_rgb(h: f32) -> u32 {
    let h = h % 360.0;
    let s = 1.0f32;
    let v = 1.0f32;
    let c = v * s;
    let x = c * (1.0 - abs((h / 60.0) % 2.0 - 1.0));
    let m = v - c;
    let (r, g, b) = if h < 60.0 { (c, x, 0.0) }
        else if h < 120.0 { (x, c, 0.0) }
        else if h < 180.0 { (0.0, c, x) }
        else if h < 240.0 { (0.0, x, c) }
        else if h < 300.0 { (x, 0.0, c) }
        else { (c, 0.0, x) };
    let r = ((r + m) * 255.0) as u32;
    let g = ((g + m) * 255.0) as u32;
    let b = ((b + m) * 255.0) as u32;
    (r << 24) | (g << 16) | (b << 8) | 0xFF
}

fn render_game() {
    unsafe {
        camera_set(CAM_X, CAM_Y + 25.0, CAM_Y + 15.0, CAM_X, 0.0, CAM_Y);
        camera_fov(45.0);

        // Arena floor
        push_identity(); push_translate(0.0, -0.1, 0.0);
        texture_bind(TEX_ARENA);
        material_shininess(0.1); material_specular(0x404040FF);
        set_color(0x808080FF);
        draw_mesh(MESH_ARENA);

        // Environmental hazard
        if HAZARD_ACTIVE {
            let hazard_color = match STAGE {
                Stage::CrystalCavern => 0x8080FFFF,   // Blue crystal
                Stage::EnchantedForest => 0x40FF40FF, // Green poison
                Stage::VolcanicDepths => 0xFF4000FF,  // Orange lava
                Stage::VoidRealm => 0x8000FFFF,       // Purple void
            };
            let pulse = 0.8 + (sin(GAME_TIME * 4.0) + 1.0) * 0.2;
            let alpha = ((HAZARD_TIMER / 5.0).min(1.0) * pulse * 200.0) as u32;

            // Draw hazard circle on ground
            push_identity();
            push_translate(HAZARD_X, 0.05, HAZARD_Y);
            push_scale_uniform(HAZARD_RADIUS * 2.0);
            set_color((hazard_color & 0xFFFFFF00) | alpha);
            material_shininess(0.8);
            material_specular(hazard_color);
            draw_mesh(MESH_ARENA);  // Reuse arena mesh as circle indicator
        }

        // XP gems
        for g in &XP_GEMS {
            if !g.active { continue; }
            let bob = sin(GAME_TIME * 4.0 + g.phase) * 0.1;
            push_identity(); push_translate(g.x, 0.3 + bob, g.y);
            push_rotate_y(GAME_TIME * 90.0); push_scale_uniform(0.3);
            texture_bind(TEX_XP);
            material_shininess(0.8); material_specular(0x8080FFFF);
            set_color(0x4080FFFF);
            draw_mesh(MESH_XP);
        }

        // Enemies
        for e in &ENEMIES {
            if !e.active { continue; }
            push_identity(); push_translate(e.x, 0.0, e.y);
            let angle = atan2(-e.y, -e.x) * 57.3;
            push_rotate_y(angle);
            let scale = match e.enemy_type {
                // Basic enemies
                EnemyType::Crawler => 0.6, EnemyType::Skeleton => 0.8, EnemyType::Wisp => 0.5,
                EnemyType::Golem => 1.2, EnemyType::Shade => 0.7, EnemyType::Berserker => 1.0,
                EnemyType::ArcaneSentinel => 0.9,
                // Elites (larger)
                EnemyType::CrystalKnight => 1.3, EnemyType::VoidMage => 1.1,
                EnemyType::GolemTitan => 1.6, EnemyType::SpecterLord => 1.2,
                // Bosses (much larger)
                EnemyType::PrismColossus => 2.5, EnemyType::VoidDragon => 2.2,
            };
            push_scale_uniform(scale);
            let (tex, mesh) = match e.enemy_type {
                // Basic enemies
                EnemyType::Golem => (TEX_GOLEM, MESH_GOLEM),
                EnemyType::Crawler => (TEX_CRAWLER, MESH_CRAWLER),
                EnemyType::Wisp => (TEX_WISP, MESH_WISP),
                EnemyType::Skeleton => (TEX_SKELETON, MESH_SKELETON),
                EnemyType::Shade => (TEX_SHADE, MESH_SHADE),
                EnemyType::Berserker => (TEX_BERSERKER, MESH_BERSERKER),
                EnemyType::ArcaneSentinel => (TEX_ARCANE_SENTINEL, MESH_ARCANE_SENTINEL),
                // Elites
                EnemyType::CrystalKnight => (TEX_CRYSTAL_KNIGHT, MESH_CRYSTAL_KNIGHT),
                EnemyType::VoidMage => (TEX_VOID_MAGE, MESH_VOID_MAGE),
                EnemyType::GolemTitan => (TEX_GOLEM_TITAN, MESH_GOLEM_TITAN),
                EnemyType::SpecterLord => (TEX_SPECTER_LORD, MESH_SPECTER_LORD),
                // Bosses
                EnemyType::PrismColossus => (TEX_PRISM_COLOSSUS, MESH_PRISM_COLOSSUS),
                EnemyType::VoidDragon => (TEX_VOID_DRAGON, MESH_VOID_DRAGON),
            };
            texture_bind(tex);
            // Elites glow purple, bosses glow red
            let (shin, spec) = match e.tier {
                EnemyTier::Normal => (0.3, 0x606060FF),
                EnemyTier::Elite => (0.6, 0x8040FFFF),
                EnemyTier::Boss => (0.8, 0xFF4040FF),
            };
            material_shininess(shin); material_specular(spec);
            set_color(if e.hit_timer > 0.0 { 0xFF8080FF } else { 0xFFFFFFFF });
            draw_mesh(mesh);
        }

        // Players (living and dead)
        for (_i, p) in PLAYERS.iter().enumerate() {
            if !p.active { continue; }

            let (tex, mesh) = match p.class {
                HeroClass::Knight => (TEX_KNIGHT, MESH_KNIGHT), HeroClass::Mage => (TEX_MAGE, MESH_MAGE),
                HeroClass::Ranger => (TEX_RANGER, MESH_RANGER), HeroClass::Cleric => (TEX_CLERIC, MESH_CLERIC),
                HeroClass::Necromancer => (TEX_NECROMANCER, MESH_NECROMANCER), HeroClass::Paladin => (TEX_PALADIN, MESH_PALADIN),
            };

            if p.dead {
                // Draw downed player (lying down, semi-transparent)
                push_identity(); push_translate(p.x, 0.1, p.y);
                push_rotate_y(p.facing_angle - 90.0);
                // Tilt forward to show they're downed
                push_scale_uniform(0.8);
                texture_bind(tex);
                material_shininess(0.2); material_specular(0x404040FF);
                set_color(0x808080A0);  // Gray and transparent
                draw_mesh(mesh);

                // Revive progress indicator (ring around player)
                if p.revive_progress > 0.0 {
                    let progress = p.revive_progress / REVIVE_TIME;
                    // Draw progress using a spinning effect via repeated positioning
                    let angle = GAME_TIME * 180.0;
                    push_identity(); push_translate(p.x, 0.5, p.y);
                    push_rotate_y(angle);
                    push_scale_uniform(0.15);
                    material_shininess(1.0); material_specular(0x40FF40FF);
                    let color = 0x40FF4000 | ((progress * 255.0) as u32);
                    set_color(color);
                    draw_mesh(MESH_PROJ);  // Simple sphere as indicator
                }
            } else {
                // Draw living player
                push_identity(); push_translate(p.x, 0.0, p.y);
                push_rotate_y(p.facing_angle - 90.0); push_scale_uniform(0.8);
                texture_bind(tex);
                material_shininess(0.5); material_specular(0x808080FF);
                let flash = p.invuln_timer > 0.0 && (GAME_TIME * 10.0) as i32 % 2 == 0;
                set_color(if flash { 0xFFFFFF80 } else { 0xFFFFFFFF });
                draw_mesh(mesh);
            }
        }

        // Projectiles
        for pr in &PROJECTILES {
            if !pr.active { continue; }
            push_identity(); push_translate(pr.x, 0.4, pr.y); push_scale_uniform(1.2);
            material_shininess(1.0); material_specular(0xFFFF80FF);
            set_color(0xFFFF40FF);
            draw_mesh(MESH_PROJ);
        }
    }
}

fn render_hud() {
    unsafe {
        layer(10);

        // Stage & Wave
        draw_rect(10.0, 10.0, 200.0, 55.0, 0x00000088);
        let stage_name: &[u8] = match STAGE {
            Stage::CrystalCavern => b"Crystal Cavern",
            Stage::EnchantedForest => b"Enchanted Forest",
            Stage::VolcanicDepths => b"Volcanic Depths",
            Stage::VoidRealm => b"Void Realm",
        };
        draw_text(stage_name.as_ptr(), stage_name.len() as u32, 20.0, 22.0, 16.0, 0xFFFFFFFF);

        let mut buf = [0u8; 16];
        buf[0..6].copy_from_slice(b"Wave: ");
        let len = fmt_num(WAVE, &mut buf[6..]);
        draw_text(buf.as_ptr(), 6 + len as u32, 20.0, 44.0, 14.0, 0xCCCCCCFF);

        // Time
        let ts = GAME_TIME as u32;
        buf[0..5].copy_from_slice(b"Time ");
        buf[5] = b'0' + ((ts / 60) / 10) as u8; buf[6] = b'0' + ((ts / 60) % 10) as u8;
        buf[7] = b':'; buf[8] = b'0' + ((ts % 60) / 10) as u8; buf[9] = b'0' + ((ts % 60) % 10) as u8;
        draw_text(buf.as_ptr(), 10, 130.0, 44.0, 14.0, 0xAAAAAAFF);

        // Player health/level
        let mut y = 75.0;
        for (_i, p) in PLAYERS.iter().enumerate() {
            if !p.active { continue; }
            let color = match p.class { HeroClass::Knight => 0x4080FFFF, HeroClass::Mage => 0x8040FFFF, HeroClass::Ranger => 0x40FF40FF, HeroClass::Cleric => 0xFFFF40FF, HeroClass::Necromancer => 0x6040A0FF, HeroClass::Paladin => 0xDDCC66FF };
            draw_rect(10.0, y, 150.0, 35.0, 0x00000088);
            let name: &[u8] = match p.class { HeroClass::Knight => b"Knight", HeroClass::Mage => b"Mage", HeroClass::Ranger => b"Ranger", HeroClass::Cleric => b"Cleric", HeroClass::Necromancer => b"Necromancer", HeroClass::Paladin => b"Paladin" };

            if p.dead {
                // Show "DOWNED" status
                draw_text(name.as_ptr(), name.len() as u32, 15.0, y + 5.0, 12.0, 0x808080FF);
                draw_str(b"DOWNED", 90.0, y + 5.0, 12.0, 0xFF4040FF);

                // Revive progress bar (green)
                if p.revive_progress > 0.0 {
                    let progress = p.revive_progress / REVIVE_TIME;
                    draw_rect(15.0, y + 20.0, 130.0, 8.0, 0x004000FF);
                    draw_rect(15.0, y + 20.0, 130.0 * progress, 8.0, 0x40FF40FF);
                } else {
                    draw_rect(15.0, y + 20.0, 130.0, 8.0, 0x400000FF);
                    draw_str(b"Get close!", 30.0, y + 19.0, 8.0, 0xCCCCCCFF);
                }
            } else {
                draw_text(name.as_ptr(), name.len() as u32, 15.0, y + 5.0, 12.0, color);
                buf[0..3].copy_from_slice(b"Lv "); let len = fmt_num(p.level, &mut buf[3..]); draw_text(buf.as_ptr(), 3 + len as u32, 90.0, y + 5.0, 12.0, 0xFFFFFFFF);
                let hp = (p.health / p.max_health).max(0.0);
                draw_rect(15.0, y + 20.0, 130.0, 8.0, 0x400000FF);
                draw_rect(15.0, y + 20.0, 130.0 * hp, 8.0, 0xFF4040FF);
            }
            y += 40.0;
        }

        // Kill counter
        draw_rect(SCREEN_WIDTH - 120.0, 10.0, 110.0, 30.0, 0x00000088);
        buf[0..7].copy_from_slice(b"Kills: ");
        let len = fmt_num(KILLS, &mut buf[7..]);
        draw_text(buf.as_ptr(), 7 + len as u32, SCREEN_WIDTH - 110.0, 22.0, 16.0, 0xFF8080FF);

        // Combo display (when active)
        if COMBO_COUNT >= 5 {
            let combo_y = 50.0;
            draw_rect(SCREEN_WIDTH - 140.0, combo_y, 130.0, 45.0, 0x00000088);

            // Combo count with pulsing effect
            let pulse = 1.0 + (sin(GAME_TIME * 8.0) + 1.0) * 0.15;
            let size = 20.0 * pulse;
            buf[0..6].copy_from_slice(b"COMBO ");
            let len = fmt_num(COMBO_COUNT, &mut buf[6..]);
            let color = if COMBO_COUNT >= 50 { 0xFF00FFFF }  // Legendary (purple)
                else if COMBO_COUNT >= 25 { 0xFFFF00FF }     // Epic (gold)
                else if COMBO_COUNT >= 10 { 0xFF8000FF }     // Rare (orange)
                else { 0x00FF00FF };                          // Common (green)
            draw_text(buf.as_ptr(), 6 + len as u32, SCREEN_WIDTH - 130.0, combo_y + 8.0, size, color);

            // Multiplier
            buf[0] = b'x';
            let mult_int = (COMBO_MULT * 10.0) as u32;
            buf[1] = b'0' + (mult_int / 10) as u8;
            buf[2] = b'.';
            buf[3] = b'0' + (mult_int % 10) as u8;
            draw_text(buf.as_ptr(), 4, SCREEN_WIDTH - 90.0, combo_y + 30.0, 14.0, 0xFFFF88FF);

            // Timer bar
            let timer_pct = (COMBO_TIMER / (COMBO_WINDOW + 3.0)).max(0.0);
            draw_rect(SCREEN_WIDTH - 130.0, combo_y + 40.0, 110.0, 4.0, 0x404040FF);
            draw_rect(SCREEN_WIDTH - 130.0, combo_y + 40.0, 110.0 * timer_pct, 4.0, color);
        }

        // Difficulty indicator (top center)
        draw_rect(SCREEN_WIDTH / 2.0 - 50.0, 10.0, 100.0, 20.0, 0x00000088);
        let diff_name = DIFFICULTY.name();
        let diff_color = DIFFICULTY.color();
        draw_text(diff_name.as_ptr(), diff_name.len() as u32, SCREEN_WIDTH / 2.0 - 40.0, 14.0, 14.0, diff_color);

        // Hazard warning (when active)
        if HAZARD_ACTIVE {
            let warn_pulse = (sin(GAME_TIME * 6.0) + 1.0) * 0.5;
            let warn_alpha = ((0.5 + warn_pulse * 0.5) * 255.0) as u32;
            let hazard_name: &[u8] = match STAGE {
                Stage::CrystalCavern => b"CRYSTAL SHARDS!",
                Stage::EnchantedForest => b"POISON CLOUD!",
                Stage::VolcanicDepths => b"LAVA POOL!",
                Stage::VoidRealm => b"VOID RIFT!",
            };
            draw_rect(SCREEN_WIDTH / 2.0 - 80.0, 35.0, 160.0, 22.0, 0xFF000044 | (warn_alpha / 2));
            draw_text(hazard_name.as_ptr(), hazard_name.len() as u32, SCREEN_WIDTH / 2.0 - 65.0, 40.0, 14.0, 0xFF404000 | warn_alpha);
        }

        // Boss health bar (bottom center of screen)
        if let Some(bi) = ACTIVE_BOSS {
            let boss = &ENEMIES[bi];
            if boss.active {
                let bar_w = 400.0;
                let bar_h = 20.0;
                let bar_x = (SCREEN_WIDTH - bar_w) / 2.0;
                let bar_y = SCREEN_HEIGHT - 50.0;

                // Background
                draw_rect(bar_x - 5.0, bar_y - 5.0, bar_w + 10.0, bar_h + 30.0, 0x00000088);

                // Boss name
                let boss_name: &[u8] = match boss.enemy_type {
                    EnemyType::PrismColossus => b"PRISM COLOSSUS",
                    EnemyType::VoidDragon => b"VOID DRAGON",
                    _ => b"BOSS",
                };
                draw_text(boss_name.as_ptr(), boss_name.len() as u32, bar_x + (bar_w - (boss_name.len() as f32 * 12.0)) / 2.0, bar_y - 2.0, 16.0, 0xFF4040FF);

                // Health bar
                let hp = (boss.health / boss.max_health).max(0.0);
                draw_rect(bar_x, bar_y + 15.0, bar_w, bar_h, 0x400000FF);
                draw_rect(bar_x, bar_y + 15.0, bar_w * hp, bar_h, 0xFF2020FF);

                // Health percentage
                let pct = (hp * 100.0) as u32;
                buf[0..3].copy_from_slice(b"HP ");
                let len = fmt_num(pct, &mut buf[3..]);
                buf[3 + len] = b'%';
                draw_text(buf.as_ptr(), 4 + len as u32, bar_x + bar_w / 2.0 - 25.0, bar_y + 17.0, 14.0, 0xFFFFFFFF);
            }
        }
    }
}

fn render_levelup() {
    unsafe {
        draw_rect(250.0, 150.0, 460.0, 280.0, 0x000000DD);
        draw_str(b"LEVEL UP!", 380.0, 170.0, 32.0, 0xFFFF40FF);
        draw_str(b"Choose a power-up:", 340.0, 210.0, 18.0, 0xCCCCCCFF);

        for i in 0..3 {
            let y = 250.0 + i as f32 * 60.0;
            let selected = i == LEVELUP_SELECTION;
            let bg = if selected { 0x404080FF } else { 0x202040FF };
            draw_rect(270.0, y, 420.0, 50.0, bg);

            let name: &[u8] = match LEVELUP_CHOICES[i] {
                PowerUpType::Cleave => b"Cleave - Melee sweep attack",
                PowerUpType::MagicMissile => b"Magic Missile - Homing shots",
                PowerUpType::PiercingArrow => b"Piercing Arrow - Pass through",
                PowerUpType::HolyNova => b"Holy Nova - Radial burst",
                PowerUpType::SoulDrain => b"Soul Drain - Life steal beam",
                PowerUpType::DivineCrush => b"Divine Crush - Holy smash",
                PowerUpType::Fireball => b"Fireball - AoE explosion",
                PowerUpType::IceShards => b"Ice Shards - Multi-shot spread",
                PowerUpType::LightningBolt => b"Lightning - Chain damage",
                PowerUpType::ShadowOrb => b"Shadow Orb - Orbiting damage",
                PowerUpType::Might => b"Might - +20% Damage",
                PowerUpType::Swiftness => b"Swiftness - +15% Speed",
                PowerUpType::Vitality => b"Vitality - +25 Max HP",
                PowerUpType::Haste => b"Haste - +20% Attack Speed",
                PowerUpType::Magnetism => b"Magnetism - +50% Pickup Range",
                PowerUpType::Armor => b"Armor - -15% Damage Taken",
                PowerUpType::Luck => b"Luck - +25% XP Gain",
                PowerUpType::Fury => b"Fury - Damage when low HP",
            };
            let color = if selected { 0xFFFFFFFF } else { 0xAAAAAAFF };
            draw_text(name.as_ptr(), name.len() as u32, 280.0, y + 18.0, 16.0, color);
        }

        draw_str(b"[Up/Down] Select  [A] Confirm", 310.0, 440.0, 14.0, 0x888888FF);
    }
}

fn render_pause() {
    unsafe {
        draw_rect(0.0, 0.0, SCREEN_WIDTH, SCREEN_HEIGHT, 0x00000088);
        draw_str(b"PAUSED", 400.0, 250.0, 48.0, 0xFFFFFFFF);
        draw_str(b"Press START to continue", 340.0, 320.0, 20.0, 0xAAAAAAFF);
    }
}

fn render_gameover() {
    unsafe {
        draw_rect(0.0, 0.0, SCREEN_WIDTH, SCREEN_HEIGHT, 0x000000CC);
        draw_str(b"GAME OVER", 350.0, 180.0, 48.0, 0xFF4040FF);

        let mut buf = [0u8; 32];
        buf[0..7].copy_from_slice(b"Wave: "); let len = fmt_num(WAVE, &mut buf[7..]); draw_text(buf.as_ptr(), 7 + len as u32, 400.0, 260.0, 22.0, 0xFFFFFFFF);
        buf[0..7].copy_from_slice(b"Kills: "); let len = fmt_num(KILLS, &mut buf[7..]); draw_text(buf.as_ptr(), 7 + len as u32, 400.0, 295.0, 22.0, 0xFFFFFFFF);

        let ts = GAME_TIME as u32;
        buf[0..6].copy_from_slice(b"Time: ");
        buf[6] = b'0' + ((ts / 60) / 10) as u8; buf[7] = b'0' + ((ts / 60) % 10) as u8; buf[8] = b':';
        buf[9] = b'0' + ((ts % 60) / 10) as u8; buf[10] = b'0' + ((ts % 60) % 10) as u8;
        draw_text(buf.as_ptr(), 11, 400.0, 330.0, 22.0, 0xFFFFFFFF);

        draw_str(b"Press A to restart", 370.0, 420.0, 20.0, 0xAAAAAAFF);
    }
}

fn render_victory() {
    unsafe {
        draw_rect(0.0, 0.0, SCREEN_WIDTH, SCREEN_HEIGHT, 0x000000CC);
        draw_str(b"VICTORY!", 380.0, 180.0, 48.0, 0x40FF40FF);
        draw_str(b"You survived the Void Realm!", 300.0, 250.0, 22.0, 0xFFFFFFFF);

        let mut buf = [0u8; 32];
        buf[0..7].copy_from_slice(b"Kills: "); let len = fmt_num(KILLS, &mut buf[7..]); draw_text(buf.as_ptr(), 7 + len as u32, 400.0, 300.0, 22.0, 0xFFFFFFFF);

        let ts = GAME_TIME as u32;
        buf[0..6].copy_from_slice(b"Time: ");
        buf[6] = b'0' + ((ts / 60) / 10) as u8; buf[7] = b'0' + ((ts / 60) % 10) as u8; buf[8] = b':';
        buf[9] = b'0' + ((ts % 60) / 10) as u8; buf[10] = b'0' + ((ts % 60) % 10) as u8;
        draw_text(buf.as_ptr(), 11, 400.0, 335.0, 22.0, 0xFFFFFFFF);

        draw_str(b"Press A to play again", 360.0, 420.0, 20.0, 0xAAAAAAFF);
    }
}
