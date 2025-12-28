//! Weapon firing system for Prism Survivors
//!
//! Handles all weapon types and their firing logic.

use crate::types::*;
use crate::state::*;
use crate::math::*;
use crate::spawning::*;

#[link(wasm_import_module = "env")]
extern "C" {
    fn delta_time() -> f32;
    fn play_sound(sound: u32, volume: f32, pan: f32);
}

// =============================================================================
// Weapon Firing
// =============================================================================

pub fn fire_weapons(player_idx: usize) {
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
        let has_vampire = p.has_powerup(PowerUpType::SoulDrain).is_some() &&
                          p.has_powerup(PowerUpType::Vitality).is_some();
        let has_divine_bolt = p.has_powerup(PowerUpType::DivineCrush).is_some() &&
                              p.has_powerup(PowerUpType::LightningBolt).is_some();

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

                        // Spawn cleave visual effect
                        spawn_cleave_vfx(p.x, p.y, p.facing_angle, range, player_idx as u8);

                        for e in &mut ENEMIES {
                            if e.active && dist(p.x, p.y, e.x, e.y) < range {
                                e.health -= synergy_dmg; e.hit_timer = 0.1;
                                play_sound(SFX_HIT, 0.2, 0.0);

                                // Spawn hit effects
                                spawn_hit_spark(e.x, e.y, 0xFFAA44FF);
                                spawn_damage_number(e.x, e.y, synergy_dmg, false);

                                if e.health <= 0.0 {
                                    spawn_enemy_death(e.x, e.y, 0xFF6644FF);
                                    e.active = false; KILLS += 1; spawn_xp(e.x, e.y, 2);
                                }
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
                        let nova_range = 3.5 + pu.level as f32 * 0.5;

                        // Spawn holy burst VFX
                        spawn_holy_burst(p.x, p.y, nova_range, player_idx as u8);

                        for e in &mut ENEMIES {
                            if e.active && dist(p.x, p.y, e.x, e.y) < nova_range {
                                e.health -= dmg * 0.7; e.hit_timer = 0.1;

                                // Spawn hit spark and damage number
                                spawn_hit_spark(e.x, e.y, 0xFFFF88FF);
                                spawn_damage_number(e.x, e.y, dmg * 0.7, false);

                                if e.health <= 0.0 {
                                    spawn_enemy_death(e.x, e.y, 0xFFFF44FF);
                                    e.active = false; KILLS += 1; spawn_xp(e.x, e.y, 2);
                                }
                            }
                        }
                        play_sound(SFX_SHOOT, 0.3, 0.0);
                    }
                    PowerUpType::SoulDrain => {
                        // Necromancer: Life steal beam - damages enemies in cone, heals player
                        // VampireTouch synergy: double lifesteal
                        let range = 4.0 + pu.level as f32 * 0.5;
                        let lifesteal_mult = if has_vampire { 0.4 } else { 0.2 };

                        for e in &mut ENEMIES {
                            if e.active && dist(p.x, p.y, e.x, e.y) < range {
                                let drain_dmg = dmg * 0.6;
                                e.health -= drain_dmg; e.hit_timer = 0.1;
                                // Heal player based on damage dealt
                                p.health = (p.health + drain_dmg * lifesteal_mult).min(p.max_health);
                                play_sound(SFX_HIT, 0.15, 0.0);
                                if e.health <= 0.0 { e.active = false; KILLS += 1; spawn_xp(e.x, e.y, 2); }
                            }
                        }
                    }
                    PowerUpType::DivineCrush => {
                        // Paladin: Hammer smash - AoE ground slam with stun effect
                        // DivineBolt synergy: chain to nearby enemies
                        let range = 2.5 + pu.level as f32 * 0.3;

                        // Primary smash
                        for e in &mut ENEMIES {
                            if e.active && dist(p.x, p.y, e.x, e.y) < range {
                                e.health -= dmg * 1.2; e.hit_timer = 0.3; // Longer stun
                                play_sound(SFX_HIT, 0.3, 0.0);
                                if e.health <= 0.0 { e.active = false; KILLS += 1; spawn_xp(e.x, e.y, 3); }
                            }
                        }

                        // DivineBolt synergy: chain lightning to nearby enemies
                        if has_divine_bolt {
                            for e in &mut ENEMIES {
                                if e.active && dist(p.x, p.y, e.x, e.y) < range + 3.0 && dist(p.x, p.y, e.x, e.y) >= range {
                                    e.health -= dmg * 0.5; e.hit_timer = 0.1;
                                    if e.health <= 0.0 { e.active = false; KILLS += 1; spawn_xp(e.x, e.y, 2); }
                                }
                            }
                        }
                        play_sound(SFX_SHOOT, 0.4, 0.0);
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
