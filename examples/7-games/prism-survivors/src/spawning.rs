//! Enemy and entity spawning for Prism Survivors
//!
//! Handles spawning of enemies, XP gems, and projectiles.

use crate::types::*;
use crate::state::*;
use crate::math::*;

#[link(wasm_import_module = "env")]
extern "C" {
    fn random_f32() -> f32;
    fn random_range(min: i32, max: i32) -> i32;
}

// =============================================================================
// Player Scaling
// =============================================================================

// Player count scaling - more players = harder enemies
pub fn player_scale_hp() -> f32 {
    unsafe {
        match ACTIVE_PLAYERS {
            1 => 1.0,
            2 => 1.5,   // +50% HP for 2 players
            3 => 2.0,   // +100% HP for 3 players
            _ => 2.5,   // +150% HP for 4 players
        }
    }
}

pub fn player_scale_spawn() -> f32 {
    unsafe {
        match ACTIVE_PLAYERS {
            1 => 1.0,
            2 => 1.3,   // +30% spawn rate for 2 players
            3 => 1.6,   // +60% spawn rate for 3 players
            _ => 2.0,   // +100% spawn rate for 4 players
        }
    }
}

// =============================================================================
// Spawning Functions
// =============================================================================

pub fn spawn_enemy() {
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

pub fn spawn_elite() {
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

pub fn spawn_boss(boss_type: EnemyType) {
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

pub fn spawn_xp(x: f32, y: f32, val: u32) {
    unsafe {
        for g in &mut XP_GEMS {
            if !g.active { g.active = true; g.x = x; g.y = y; g.value = val; g.phase = random_f32() * 6.28; break; }
        }
    }
}

pub fn spawn_proj(x: f32, y: f32, vx: f32, vy: f32, dmg: f32, pierce: bool, owner: u8) {
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

pub fn nearest_enemy(x: f32, y: f32) -> Option<usize> {
    unsafe {
        let mut best: Option<usize> = None; let mut bd = f32::MAX;
        for (i, e) in ENEMIES.iter().enumerate() {
            if e.active { let d = dist(x, y, e.x, e.y); if d < bd { bd = d; best = Some(i); } }
        }
        best
    }
}

// =============================================================================
// VFX Spawning Functions
// =============================================================================

pub fn spawn_vfx(vtype: VfxType, x: f32, y: f32, angle: f32, scale: f32, lifetime: f32, color: u32, value: f32, owner: u8) {
    unsafe {
        for v in &mut VFX_POOL {
            if !v.active {
                v.active = true;
                v.vtype = vtype;
                v.x = x;
                v.y = y;
                v.angle = angle;
                v.scale = scale;
                v.lifetime = lifetime;
                v.max_lifetime = lifetime;
                v.color = color;
                v.value = value;
                v.owner = owner;
                break;
            }
        }
    }
}

// Convenience functions for common VFX
pub fn spawn_cleave_vfx(x: f32, y: f32, angle: f32, range: f32, owner: u8) {
    // Player colors for cleave arc
    let colors: [u32; 4] = [0xFF6666FF, 0x66FF66FF, 0x6666FFFF, 0xFFFF66FF];
    let color = colors[owner as usize % 4];
    spawn_vfx(VfxType::CleaveSlash, x, y, angle, range, 0.3, color, 0.0, owner);
}

pub fn spawn_hit_spark(x: f32, y: f32, color: u32) {
    unsafe {
        spawn_vfx(VfxType::HitSpark, x, y, random_f32() * 360.0, 0.3, 0.15, color, 0.0, 0);
    }
}

pub fn spawn_damage_number(x: f32, y: f32, damage: f32, is_crit: bool) {
    unsafe {
        let color = if is_crit { 0xFFFF00FF } else { 0xFFFFFFFF };
        let scale = if is_crit { 1.5 } else { 1.0 };
        // Add slight random offset so numbers don't stack
        let ox = (random_f32() - 0.5) * 0.5;
        let oy = (random_f32() - 0.5) * 0.3;
        spawn_vfx(VfxType::DamageNumber, x + ox, y + oy, 0.0, scale, 0.8, color, damage, 0);
    }
}

pub fn spawn_holy_burst(x: f32, y: f32, radius: f32, owner: u8) {
    spawn_vfx(VfxType::HolyBurst, x, y, 0.0, radius, 0.4, 0xFFFF88FF, 0.0, owner);
}

pub fn spawn_enemy_death(x: f32, y: f32, color: u32) {
    spawn_vfx(VfxType::EnemyDeath, x, y, 0.0, 1.0, 0.3, color, 0.0, 0);
}

pub fn spawn_level_up_vfx(x: f32, y: f32, owner: u8) {
    let colors: [u32; 4] = [0xFFD700FF, 0xFFD700FF, 0xFFD700FF, 0xFFD700FF];
    spawn_vfx(VfxType::LevelUp, x, y, 0.0, 2.0, 1.0, colors[owner as usize % 4], 0.0, owner);
}

pub fn update_vfx(dt: f32) {
    unsafe {
        // Update player hurt flash
        if PLAYER_HURT_FLASH > 0.0 {
            PLAYER_HURT_FLASH -= dt;
        }

        // Update all VFX
        for v in &mut VFX_POOL {
            if v.active {
                v.lifetime -= dt;
                if v.lifetime <= 0.0 {
                    v.active = false;
                }
            }
        }
    }
}
