//! Gameplay update logic for Prism Survivors
//!
//! Contains all update functions for game phases.

use crate::types::*;
use crate::state::*;
use crate::math::*;
use crate::powerups::*;
use crate::spawning::*;
use crate::weapons::*;

#[link(wasm_import_module = "env")]
extern "C" {
    fn delta_time() -> f32;
    fn random_f32() -> f32;
    fn player_count() -> u32;
    fn left_stick_x(player: u32) -> f32;
    fn left_stick_y(player: u32) -> f32;
    fn button_pressed(player: u32, button: u32) -> u32;
    fn play_sound(sound: u32, volume: f32, pan: f32);
}

// =============================================================================
// Game Initialization
// =============================================================================

pub fn init_run() {
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

        for i in 0..MAX_PLAYERS {
            PLAYERS[i] = Player::new();
            if i < cnt {
                PLAYERS[i].active = true;
                PLAYERS[i].x = spawns[i].0;
                PLAYERS[i].y = spawns[i].1;
                // Use selected class from class selection screen
                PLAYERS[i].class = CLASS_SELECTIONS[i];
                // Add starting weapon based on selected class
                let starter = class_starter_weapon(CLASS_SELECTIONS[i]);
                PLAYERS[i].add_powerup(starter);

                // Class base stats
                match CLASS_SELECTIONS[i] {
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

/// Get the starting weapon for a hero class
pub fn class_starter_weapon(class: HeroClass) -> PowerUpType {
    match class {
        HeroClass::Knight => PowerUpType::Cleave,
        HeroClass::Mage => PowerUpType::MagicMissile,
        HeroClass::Ranger => PowerUpType::PiercingArrow,
        HeroClass::Cleric => PowerUpType::HolyNova,
        HeroClass::Necromancer => PowerUpType::SoulDrain,
        HeroClass::Paladin => PowerUpType::DivineCrush,
    }
}

// =============================================================================
// Update Logic
// =============================================================================

pub fn update_revives() {
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

pub fn update_players() {
    unsafe {
        let dt = delta_time();

        // Handle revives first
        update_revives();

        for i in 0..MAX_PLAYERS {
            let p = &mut PLAYERS[i];
            if !p.active || p.dead { continue; }

            if p.invuln_timer > 0.0 { p.invuln_timer -= dt; }

            // Movement (negate Y for top-down: stick up = move toward top of screen)
            let sx = left_stick_x(i as u32);
            let sy = -left_stick_y(i as u32);  // Invert Y axis for natural top-down controls
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

pub fn update_enemies() {
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

                    // VFX: Screen flash and damage indicator
                    PLAYER_HURT_FLASH = 0.3;
                    spawn_hit_spark(p.x, p.y, 0xFF4444FF);
                    spawn_damage_number(p.x, p.y, dmg_taken, dmg_taken > 20.0);

                    let (kx, ky) = norm(p.x - e.x, p.y - e.y);
                    p.x += kx * 1.5; p.y += ky * 1.5;

                    if p.health <= 0.0 { p.dead = true; }
                }
            }
        }
    }
}

pub fn update_projectiles() {
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

                    // VFX: Hit spark and damage number
                    spawn_hit_spark(e.x, e.y, 0xFFAA44FF);
                    spawn_damage_number(e.x, e.y, pr.damage, pr.damage > 15.0);

                    if !pr.piercing { pr.active = false; }
                    if e.health <= 0.0 {
                        spawn_enemy_death(e.x, e.y, 0xFF8844FF);
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

pub fn update_combo() {
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

pub fn update_hazards() {
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

pub fn update_spawning() {
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

pub fn update_camera() {
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

pub fn check_game_over() {
    unsafe {
        let mut all_dead = true;
        for p in &PLAYERS { if p.active && !p.dead { all_dead = false; break; } }
        if all_dead { PHASE = GamePhase::GameOver; }
    }
}

// =============================================================================
// Phase-specific Update Functions
// =============================================================================

pub fn update_title() {
    unsafe {
        TITLE_TIMER += delta_time();
        for i in 0..MAX_PLAYERS {
            // Difficulty selection with D-pad
            if button_pressed(i as u32, BUTTON_UP) != 0 {
                if DIFFICULTY_SELECTION > 0 { DIFFICULTY_SELECTION -= 1; play_sound(SFX_SELECT, 0.3, 0.0); }
            }
            if button_pressed(i as u32, BUTTON_DOWN) != 0 {
                if DIFFICULTY_SELECTION < 3 { DIFFICULTY_SELECTION += 1; play_sound(SFX_SELECT, 0.3, 0.0); }
            }
            // Go to class selection
            if button_pressed(i as u32, BUTTON_A) != 0 || button_pressed(i as u32, BUTTON_START) != 0 {
                play_sound(SFX_SELECT, 0.5, 0.0);
                // Initialize class selection state
                for j in 0..MAX_PLAYERS {
                    CLASS_CONFIRMED[j] = false;
                    CLASS_SELECT_CURSOR[j] = j as u8 % 6;  // Different defaults per player
                    CLASS_SELECTIONS[j] = match j % 6 {
                        0 => HeroClass::Knight,
                        1 => HeroClass::Mage,
                        2 => HeroClass::Ranger,
                        3 => HeroClass::Cleric,
                        4 => HeroClass::Necromancer,
                        _ => HeroClass::Paladin,
                    };
                }
                CLASS_SELECT_COUNTDOWN = 0.0;
                ALL_PLAYERS_READY = false;
                PHASE = GamePhase::ClassSelect;
                break;
            }
        }
    }
}

pub const BUTTON_LEFT: u32 = 2;
pub const BUTTON_RIGHT: u32 = 3;

pub fn update_class_select() {
    unsafe {
        let dt = delta_time();
        let active_count = player_count().min(MAX_PLAYERS as u32) as usize;

        // Check if all active players are confirmed
        let mut all_confirmed = true;
        for i in 0..active_count {
            if !CLASS_CONFIRMED[i] { all_confirmed = false; break; }
        }

        if all_confirmed {
            if !ALL_PLAYERS_READY {
                ALL_PLAYERS_READY = true;
                CLASS_SELECT_COUNTDOWN = 3.0;  // 3 second countdown
                play_sound(SFX_LEVELUP, 0.5, 0.0);
            }

            CLASS_SELECT_COUNTDOWN -= dt;
            if CLASS_SELECT_COUNTDOWN <= 0.0 {
                init_run();
                return;
            }
        } else {
            ALL_PLAYERS_READY = false;
            CLASS_SELECT_COUNTDOWN = 0.0;
        }

        // Handle input for each player
        for i in 0..active_count {
            if CLASS_CONFIRMED[i] {
                // B to unconfirm
                if button_pressed(i as u32, BUTTON_B) != 0 {
                    CLASS_CONFIRMED[i] = false;
                    play_sound(SFX_SELECT, 0.2, 0.0);
                }
                continue;
            }

            // Left/Right to cycle classes
            if button_pressed(i as u32, BUTTON_LEFT) != 0 {
                if CLASS_SELECT_CURSOR[i] == 0 { CLASS_SELECT_CURSOR[i] = 5; }
                else { CLASS_SELECT_CURSOR[i] -= 1; }
                CLASS_SELECTIONS[i] = cursor_to_class(CLASS_SELECT_CURSOR[i]);
                play_sound(SFX_SELECT, 0.2, 0.0);
            }
            if button_pressed(i as u32, BUTTON_RIGHT) != 0 {
                CLASS_SELECT_CURSOR[i] = (CLASS_SELECT_CURSOR[i] + 1) % 6;
                CLASS_SELECTIONS[i] = cursor_to_class(CLASS_SELECT_CURSOR[i]);
                play_sound(SFX_SELECT, 0.2, 0.0);
            }

            // A to confirm
            if button_pressed(i as u32, BUTTON_A) != 0 {
                CLASS_CONFIRMED[i] = true;
                play_sound(SFX_SELECT, 0.4, 0.0);
            }

            // B to go back to title
            if button_pressed(i as u32, BUTTON_B) != 0 && i == 0 {
                PHASE = GamePhase::Title;
                return;
            }
        }
    }
}

fn cursor_to_class(cursor: u8) -> HeroClass {
    match cursor {
        0 => HeroClass::Knight,
        1 => HeroClass::Mage,
        2 => HeroClass::Ranger,
        3 => HeroClass::Cleric,
        4 => HeroClass::Necromancer,
        _ => HeroClass::Paladin,
    }
}

pub fn update_playing() {
    unsafe {
        let dt = delta_time();

        update_players();
        update_enemies();
        update_projectiles();
        update_combo();
        update_hazards();
        update_spawning();
        update_camera();
        update_vfx(dt);  // Update visual effects
        check_game_over();

        for i in 0..MAX_PLAYERS {
            if button_pressed(i as u32, BUTTON_START) != 0 { PHASE = GamePhase::Paused; break; }
        }
    }
}

pub fn update_levelup() {
    unsafe {
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
}

pub fn update_paused() {
    unsafe {
        for i in 0..MAX_PLAYERS {
            if button_pressed(i as u32, BUTTON_START) != 0 { PHASE = GamePhase::Playing; break; }
        }
    }
}

pub fn update_endgame() {
    unsafe {
        for i in 0..MAX_PLAYERS {
            if button_pressed(i as u32, BUTTON_A) != 0 || button_pressed(i as u32, BUTTON_START) != 0 {
                PHASE = GamePhase::Title; break;
            }
        }
    }
}
