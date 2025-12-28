//! Power-up system for Prism Survivors
//!
//! Handles level-up choice generation and power-up application.

use crate::types::*;
use crate::state::*;

#[link(wasm_import_module = "env")]
extern "C" {
    fn random_range(min: i32, max: i32) -> i32;
}

// =============================================================================
// Roguelite: Power-up Choice Generation
// =============================================================================

pub fn generate_levelup_choices(player_idx: usize) {
    unsafe {
        let p = &PLAYERS[player_idx];
        // Increased to 32 to handle all powerup types safely
        let mut available: [PowerUpType; 32] = [PowerUpType::Might; 32];
        let mut count = 0usize;

        // Add weapons not at max level
        let weapons = [
            PowerUpType::Cleave, PowerUpType::MagicMissile, PowerUpType::PiercingArrow,
            PowerUpType::HolyNova, PowerUpType::SoulDrain, PowerUpType::DivineCrush,
            PowerUpType::Fireball, PowerUpType::IceShards,
            PowerUpType::LightningBolt, PowerUpType::ShadowOrb
        ];
        for &w in &weapons {
            if count >= 31 { break; } // Bounds check
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
            if count >= 31 { break; } // Bounds check
            if let Some(lv) = p.has_powerup(pa) {
                if lv < 5 { available[count] = pa; count += 1; }
            } else if p.powerup_count < MAX_POWERUPS as u8 {
                available[count] = pa; count += 1;
            }
        }

        // Ensure count stays in bounds
        count = if count > 31 { 31 } else { count };

        // Pick 3 random choices
        if count >= 3 {
            for i in 0..3 {
                let idx = (random_range(i as i32, count as i32) as usize).min(31);
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

pub fn apply_powerup(player_idx: usize, ptype: PowerUpType) {
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
            PowerUpType::Armor => { p.armor = if p.armor + 0.15 > 0.80 { 0.80 } else { p.armor + 0.15 }; }
            PowerUpType::Luck => { p.xp_mult += 0.25; }
            PowerUpType::Fury => { p.damage_mult += 0.05; } // Base fury bonus
            _ => {}
        }
    }
}
