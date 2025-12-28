//! Type definitions for Prism Survivors
//!
//! Contains all enums, structs and their implementations.

use crate::state::MAX_POWERUPS;

// =============================================================================
// Power-up System Types
// =============================================================================

#[derive(Clone, Copy, PartialEq)]
#[repr(u8)]
pub enum PowerUpType {
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
pub enum Difficulty {
    Easy = 0,       // 0.6x enemy HP, 0.7x damage, 0.8x spawn rate
    Normal = 1,     // 1.0x all
    Hard = 2,       // 1.4x enemy HP, 1.3x damage, 1.2x spawn rate, +10% XP bonus
    Nightmare = 3,  // 2.0x enemy HP, 1.6x damage, 1.5x spawn rate, +25% XP bonus, elites wave 3
}

impl Difficulty {
    pub fn enemy_hp_mult(&self) -> f32 {
        match self { Difficulty::Easy => 0.6, Difficulty::Normal => 1.0, Difficulty::Hard => 1.4, Difficulty::Nightmare => 2.0 }
    }
    pub fn enemy_damage_mult(&self) -> f32 {
        match self { Difficulty::Easy => 0.7, Difficulty::Normal => 1.0, Difficulty::Hard => 1.3, Difficulty::Nightmare => 1.6 }
    }
    pub fn spawn_rate_mult(&self) -> f32 {
        match self { Difficulty::Easy => 0.8, Difficulty::Normal => 1.0, Difficulty::Hard => 1.2, Difficulty::Nightmare => 1.5 }
    }
    pub fn xp_mult(&self) -> f32 {
        // Reward players for taking on harder challenges (no penalties!)
        match self { Difficulty::Easy => 1.0, Difficulty::Normal => 1.0, Difficulty::Hard => 1.1, Difficulty::Nightmare => 1.25 }
    }
    pub fn elite_start_wave(&self) -> u32 {
        match self { Difficulty::Easy => 7, Difficulty::Normal => 5, Difficulty::Hard => 4, Difficulty::Nightmare => 3 }
    }
    pub fn name(&self) -> &'static [u8] {
        match self { Difficulty::Easy => b"EASY", Difficulty::Normal => b"NORMAL", Difficulty::Hard => b"HARD", Difficulty::Nightmare => b"NIGHTMARE" }
    }
    pub fn color(&self) -> u32 {
        match self { Difficulty::Easy => 0x44FF44FF, Difficulty::Normal => 0xFFFF44FF, Difficulty::Hard => 0xFF8844FF, Difficulty::Nightmare => 0xFF4444FF }
    }
}

// Synergy combinations for bonus effects (expanded)
#[derive(Clone, Copy, PartialEq)]
#[repr(u8)]
pub enum Synergy {
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
pub struct PowerUp {
    pub ptype: PowerUpType,
    pub level: u8,          // 1-5
    pub cooldown: f32,      // For weapons
    pub timer: f32,         // Current cooldown timer
}

impl PowerUp {
    pub const fn new(ptype: PowerUpType) -> Self {
        Self { ptype, level: 1, cooldown: 1.0, timer: 0.0 }
    }
}

#[derive(Clone, Copy, PartialEq)]
#[repr(u8)]
pub enum HeroClass { Knight = 0, Mage = 1, Ranger = 2, Cleric = 3, Necromancer = 4, Paladin = 5 }

#[derive(Clone, Copy, PartialEq)]
#[repr(u8)]
pub enum EnemyType {
    // Basic enemies (0-6)
    Golem = 0, Crawler = 1, Wisp = 2, Skeleton = 3, Shade = 4, Berserker = 5, ArcaneSentinel = 6,
    // Elite enemies (7-10)
    CrystalKnight = 7, VoidMage = 8, GolemTitan = 9, SpecterLord = 10,
    // Bosses (11-12)
    PrismColossus = 11, VoidDragon = 12,
}

#[derive(Clone, Copy, PartialEq)]
#[repr(u8)]
pub enum EnemyTier { Normal = 0, Elite = 1, Boss = 2 }

#[derive(Clone, Copy, PartialEq)]
#[repr(u8)]
pub enum GamePhase { Title, ClassSelect, Playing, LevelUp, Paused, GameOver, Victory }

#[derive(Clone, Copy, PartialEq)]
#[repr(u8)]
pub enum Stage { CrystalCavern = 0, EnchantedForest = 1, VolcanicDepths = 2, VoidRealm = 3 }

// =============================================================================
// Player State
// =============================================================================

#[derive(Clone, Copy)]
pub struct Player {
    pub x: f32, pub y: f32,
    pub vx: f32, pub vy: f32,
    pub health: f32, pub max_health: f32,
    pub xp: u32, pub level: u32,
    pub class: HeroClass,
    pub move_speed: f32,
    pub damage_mult: f32,
    pub attack_speed_mult: f32,
    pub pickup_range: f32,
    pub armor: f32,
    pub xp_mult: f32,
    pub facing_angle: f32,
    pub invuln_timer: f32,
    pub active: bool, pub dead: bool,
    // Revive system
    pub revive_progress: f32,  // 0.0 to 3.0 (seconds)
    pub reviving_by: i8,       // Player index reviving us, -1 if none
    pub powerups: [Option<PowerUp>; MAX_POWERUPS],
    pub powerup_count: u8,
}

impl Player {
    pub const fn new() -> Self {
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

    pub fn has_powerup(&self, ptype: PowerUpType) -> Option<u8> {
        for p in &self.powerups {
            if let Some(pu) = p {
                if pu.ptype == ptype { return Some(pu.level); }
            }
        }
        None
    }

    pub fn get_synergy(&self, ptype: PowerUpType) -> Synergy {
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

    pub fn add_powerup(&mut self, ptype: PowerUpType) {
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
pub struct Enemy {
    pub x: f32, pub y: f32, pub vx: f32, pub vy: f32,
    pub health: f32, pub max_health: f32,
    pub damage: f32, pub speed: f32,
    pub enemy_type: EnemyType,
    pub tier: EnemyTier,
    pub active: bool, pub hit_timer: f32,
    pub attack_timer: f32,  // For boss special attacks
}

impl Enemy {
    pub const fn new() -> Self {
        Self {
            x: 0.0, y: 0.0, vx: 0.0, vy: 0.0,
            health: 10.0, max_health: 10.0, damage: 10.0, speed: 2.0,
            enemy_type: EnemyType::Golem, tier: EnemyTier::Normal,
            active: false, hit_timer: 0.0, attack_timer: 0.0,
        }
    }
}

#[derive(Clone, Copy)]
pub struct XpGem { pub x: f32, pub y: f32, pub value: u32, pub active: bool, pub phase: f32 }

impl XpGem {
    pub const fn new() -> Self { Self { x: 0.0, y: 0.0, value: 1, active: false, phase: 0.0 } }
}

#[derive(Clone, Copy)]
pub struct Projectile {
    pub x: f32, pub y: f32, pub vx: f32, pub vy: f32,
    pub damage: f32, pub lifetime: f32, pub radius: f32,
    pub piercing: bool, pub owner: u8, pub active: bool,
}

impl Projectile {
    pub const fn new() -> Self {
        Self { x: 0.0, y: 0.0, vx: 0.0, vy: 0.0,
               damage: 10.0, lifetime: 2.0, radius: 0.3,
               piercing: false, owner: 0, active: false }
    }
}

// =============================================================================
// Visual Effects System
// =============================================================================

#[derive(Clone, Copy, PartialEq)]
#[repr(u8)]
pub enum VfxType {
    CleaveSlash = 0,     // Arc slash visual for melee
    HitSpark = 1,        // Small spark when enemy hit
    DamageNumber = 2,    // Floating damage number
    HolyBurst = 3,       // Radial holy nova effect
    SoulDrainBeam = 4,   // Life steal beam
    PlayerHurt = 5,      // Screen flash when player hurt
    LevelUp = 6,         // Level up celebration
    EnemyDeath = 7,      // Enemy death burst
}

#[derive(Clone, Copy)]
pub struct Vfx {
    pub active: bool,
    pub vtype: VfxType,
    pub x: f32,
    pub y: f32,
    pub angle: f32,       // Rotation/direction
    pub scale: f32,       // Size multiplier
    pub lifetime: f32,    // Current lifetime
    pub max_lifetime: f32,
    pub color: u32,       // RGBA color
    pub value: f32,       // For damage numbers: the damage value
    pub owner: u8,        // Player index for coloring
}

impl Vfx {
    pub const fn new() -> Self {
        Self {
            active: false,
            vtype: VfxType::HitSpark,
            x: 0.0, y: 0.0,
            angle: 0.0, scale: 1.0,
            lifetime: 0.0, max_lifetime: 0.5,
            color: 0xFFFFFFFF,
            value: 0.0,
            owner: 0,
        }
    }

    pub fn progress(&self) -> f32 {
        if self.max_lifetime > 0.0 { 1.0 - (self.lifetime / self.max_lifetime) } else { 1.0 }
    }
}
