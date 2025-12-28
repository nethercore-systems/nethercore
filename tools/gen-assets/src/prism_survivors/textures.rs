//! Texture definitions for PRISM SURVIVORS
//!
//! Uses the convention-based texture system with category-based filtering.
//! Each asset has:
//! - `{id}.png` - Base texture
//! - Mode 3 uses Blinn-Phong (no emissive textures needed)

use crate::texture::{AssetCategory, AssetTexture, TextureStyle, generate_textures_by_category};
use std::path::Path;

/// All PRISM SURVIVORS asset textures - single source of truth
pub const TEXTURES: &[AssetTexture] = &[
    // === HEROES ===
    // Knight - polished metal armor
    AssetTexture {
        id: "knight",
        category: AssetCategory::Hero,
        base_color: [140, 140, 150, 255],
        style: TextureStyle::Metal { seed: 42 },
        size: (64, 64),
        secondary_color: None,
        emissive: None,
    },
    // Mage - mystical purple robes
    AssetTexture {
        id: "mage",
        category: AssetCategory::Hero,
        base_color: [80, 40, 120, 255],
        style: TextureStyle::GradientV,
        size: (64, 64),
        secondary_color: Some([40, 20, 80, 255]),
        emissive: None,
    },
    // Ranger - forest green leather
    AssetTexture {
        id: "ranger",
        category: AssetCategory::Hero,
        base_color: [60, 80, 40, 255],
        style: TextureStyle::GradientV,
        size: (64, 64),
        secondary_color: Some([40, 50, 30, 255]),
        emissive: None,
    },
    // Cleric - holy white/gold robes
    AssetTexture {
        id: "cleric",
        category: AssetCategory::Hero,
        base_color: [240, 230, 200, 255],
        style: TextureStyle::GradientV,
        size: (64, 64),
        secondary_color: Some([200, 180, 140, 255]),
        emissive: None,
    },
    // Necromancer - dark purple/green robes with skull motif
    AssetTexture {
        id: "necromancer",
        category: AssetCategory::Hero,
        base_color: [30, 20, 35, 255],
        style: TextureStyle::GradientV,
        size: (64, 64),
        secondary_color: Some([60, 80, 40, 255]),
        emissive: None,
    },
    // Paladin - golden holy armor
    AssetTexture {
        id: "paladin",
        category: AssetCategory::Hero,
        base_color: [220, 190, 100, 255],
        style: TextureStyle::Metal { seed: 777 },
        size: (64, 64),
        secondary_color: Some([180, 150, 80, 255]),
        emissive: None,
    },

    // === BASIC ENEMIES ===
    // Golem - rocky stone
    AssetTexture {
        id: "golem",
        category: AssetCategory::Enemy,
        base_color: [100, 90, 80, 255],
        style: TextureStyle::Stone { seed: 42 },
        size: (64, 64),
        secondary_color: None,
        emissive: None,
    },
    // Crawler - dark chitinous shell
    AssetTexture {
        id: "crawler",
        category: AssetCategory::Enemy,
        base_color: [40, 35, 45, 255],
        style: TextureStyle::Solid,
        size: (64, 64),
        secondary_color: None,
        emissive: None,
    },
    // Wisp - glowing ethereal
    AssetTexture {
        id: "wisp",
        category: AssetCategory::Enemy,
        base_color: [255, 200, 100, 255],
        style: TextureStyle::GradientRadial,
        size: (64, 64),
        secondary_color: Some([255, 150, 50, 200]),
        emissive: None, // Mode 3 doesn't use emissive, but the wisp glows via material
    },
    // Skeleton - bone white
    AssetTexture {
        id: "skeleton",
        category: AssetCategory::Enemy,
        base_color: [220, 210, 190, 255],
        style: TextureStyle::Solid,
        size: (64, 64),
        secondary_color: None,
        emissive: None,
    },
    // Shade - dark ethereal shadow
    AssetTexture {
        id: "shade",
        category: AssetCategory::Enemy,
        base_color: [20, 15, 30, 255],
        style: TextureStyle::GradientRadial,
        size: (64, 64),
        secondary_color: Some([40, 30, 60, 200]),
        emissive: None,
    },
    // Berserker - blood red fury
    AssetTexture {
        id: "berserker",
        category: AssetCategory::Enemy,
        base_color: [160, 40, 30, 255],
        style: TextureStyle::GradientV,
        size: (64, 64),
        secondary_color: Some([100, 30, 25, 255]),
        emissive: None,
    },
    // Arcane Sentinel - magical construct
    AssetTexture {
        id: "arcane_sentinel",
        category: AssetCategory::Enemy,
        base_color: [100, 80, 180, 255],
        style: TextureStyle::Metal { seed: 555 },
        size: (64, 64),
        secondary_color: Some([60, 50, 140, 255]),
        emissive: None,
    },

    // === ELITE ENEMIES ===
    // Crystal Knight - iridescent crystalline armor
    AssetTexture {
        id: "crystal_knight",
        category: AssetCategory::Elite,
        base_color: [150, 200, 220, 255],
        style: TextureStyle::Metal { seed: 123 },
        size: (64, 64),
        secondary_color: Some([100, 180, 200, 255]),
        emissive: None,
    },
    // Void Mage - deep purple with void energy
    AssetTexture {
        id: "void_mage",
        category: AssetCategory::Elite,
        base_color: [40, 20, 60, 255],
        style: TextureStyle::GradientV,
        size: (64, 64),
        secondary_color: Some([80, 40, 120, 255]),
        emissive: None,
    },
    // Golem Titan - darker, more imposing stone
    AssetTexture {
        id: "golem_titan",
        category: AssetCategory::Elite,
        base_color: [70, 60, 55, 255],
        style: TextureStyle::Stone { seed: 789 },
        size: (64, 64),
        secondary_color: Some([50, 45, 40, 255]),
        emissive: None,
    },
    // Specter Lord - ethereal blue-white
    AssetTexture {
        id: "specter_lord",
        category: AssetCategory::Elite,
        base_color: [180, 200, 255, 255],
        style: TextureStyle::GradientRadial,
        size: (64, 64),
        secondary_color: Some([100, 150, 255, 200]),
        emissive: None,
    },

    // === BOSSES ===
    // Prism Colossus - prismatic rainbow crystal
    AssetTexture {
        id: "prism_colossus",
        category: AssetCategory::Boss,
        base_color: [200, 180, 220, 255],
        style: TextureStyle::Metal { seed: 456 },
        size: (128, 128),
        secondary_color: Some([180, 200, 240, 255]),
        emissive: None,
    },
    // Void Dragon - deep black with purple accents
    AssetTexture {
        id: "void_dragon",
        category: AssetCategory::Boss,
        base_color: [30, 20, 40, 255],
        style: TextureStyle::GradientV,
        size: (128, 128),
        secondary_color: Some([60, 30, 80, 255]),
        emissive: None,
    },

    // === PICKUPS ===
    // XP Gem - glowing blue crystal
    AssetTexture {
        id: "xp_gem",
        category: AssetCategory::Pickup,
        base_color: [80, 150, 255, 255],
        style: TextureStyle::GradientRadial,
        size: (32, 32),
        secondary_color: Some([40, 100, 200, 255]),
        emissive: None,
    },
    // Coin - shiny gold
    AssetTexture {
        id: "coin",
        category: AssetCategory::Pickup,
        base_color: [255, 200, 50, 255],
        style: TextureStyle::Metal { seed: 333 },
        size: (32, 32),
        secondary_color: Some([200, 150, 30, 255]),
        emissive: None,
    },
    // Powerup Orb - radiant white
    AssetTexture {
        id: "powerup_orb",
        category: AssetCategory::Pickup,
        base_color: [255, 255, 255, 255],
        style: TextureStyle::GradientRadial,
        size: (32, 32),
        secondary_color: Some([200, 220, 255, 255]),
        emissive: None,
    },

    // === ARENA ===
    // Arena Floor - stone tiles
    AssetTexture {
        id: "arena_floor",
        category: AssetCategory::Arena,
        base_color: [80, 75, 70, 255],
        style: TextureStyle::Stone { seed: 999 },
        size: (256, 256),
        secondary_color: Some([60, 55, 50, 255]),
        emissive: None,
    },

    // === PROJECTILES ===
    // Frost Shard - icy blue-white
    AssetTexture {
        id: "frost_shard",
        category: AssetCategory::Projectile,
        base_color: [180, 220, 255, 255],
        style: TextureStyle::GradientRadial,
        size: (32, 32),
        secondary_color: Some([100, 180, 240, 255]),
        emissive: None,
    },
    // Void Orb - dark purple energy
    AssetTexture {
        id: "void_orb",
        category: AssetCategory::Projectile,
        base_color: [60, 20, 80, 255],
        style: TextureStyle::GradientRadial,
        size: (32, 32),
        secondary_color: Some([100, 40, 140, 255]),
        emissive: None,
    },
    // Lightning Bolt - electric yellow-blue
    AssetTexture {
        id: "lightning_bolt",
        category: AssetCategory::Projectile,
        base_color: [255, 255, 100, 255],
        style: TextureStyle::GradientRadial,
        size: (32, 32),
        secondary_color: Some([150, 200, 255, 255]),
        emissive: None,
    },
];

/// Generate all textures for Prism Survivors
pub fn generate_all(output_dir: &Path) {
    use AssetCategory::*;

    // Generate all categories using the consolidated function
    for category in [Hero, Enemy, Elite, Boss, Pickup, Arena, Projectile] {
        generate_textures_by_category(TEXTURES, category, output_dir);
    }
}
