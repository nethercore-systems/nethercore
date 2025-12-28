//! Texture definitions for PRISM SURVIVORS
//!
//! Uses the convention-based texture system. Each asset has:
//! - `{id}.png` - Base texture
//! - Mode 3 uses Blinn-Phong (no emissive textures needed)

use crate::texture::{AssetTexture, TextureStyle};

/// All PRISM SURVIVORS asset textures - single source of truth
pub const TEXTURES: &[AssetTexture] = &[
    // === HEROES ===
    // Knight - polished metal armor
    AssetTexture {
        id: "knight",
        base_color: [140, 140, 150, 255],
        style: TextureStyle::Metal { seed: 42 },
        size: (64, 64),
        secondary_color: None,
        emissive: None,
    },
    // Mage - mystical purple robes
    AssetTexture {
        id: "mage",
        base_color: [80, 40, 120, 255],
        style: TextureStyle::GradientV,
        size: (64, 64),
        secondary_color: Some([40, 20, 80, 255]),
        emissive: None,
    },
    // Ranger - forest green leather
    AssetTexture {
        id: "ranger",
        base_color: [60, 80, 40, 255],
        style: TextureStyle::GradientV,
        size: (64, 64),
        secondary_color: Some([40, 50, 30, 255]),
        emissive: None,
    },
    // Cleric - holy white/gold robes
    AssetTexture {
        id: "cleric",
        base_color: [240, 230, 200, 255],
        style: TextureStyle::GradientV,
        size: (64, 64),
        secondary_color: Some([200, 180, 140, 255]),
        emissive: None,
    },
    // Necromancer - dark purple/green robes with skull motif
    AssetTexture {
        id: "necromancer",
        base_color: [30, 20, 35, 255],
        style: TextureStyle::GradientV,
        size: (64, 64),
        secondary_color: Some([60, 80, 40, 255]),
        emissive: None,
    },
    // Paladin - golden holy armor
    AssetTexture {
        id: "paladin",
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
        base_color: [100, 90, 80, 255],
        style: TextureStyle::Stone { seed: 42 },
        size: (64, 64),
        secondary_color: None,
        emissive: None,
    },
    // Crawler - dark chitinous shell
    AssetTexture {
        id: "crawler",
        base_color: [40, 35, 45, 255],
        style: TextureStyle::Solid,
        size: (64, 64),
        secondary_color: None,
        emissive: None,
    },
    // Wisp - glowing ethereal
    AssetTexture {
        id: "wisp",
        base_color: [255, 200, 100, 255],
        style: TextureStyle::GradientRadial,
        size: (64, 64),
        secondary_color: Some([255, 150, 50, 200]),
        emissive: None, // Mode 3 doesn't use emissive, but the wisp glows via material
    },
    // Skeleton - bone white
    AssetTexture {
        id: "skeleton",
        base_color: [220, 210, 190, 255],
        style: TextureStyle::Solid,
        size: (64, 64),
        secondary_color: None,
        emissive: None,
    },
    // Shade - dark ethereal shadow
    AssetTexture {
        id: "shade",
        base_color: [20, 15, 30, 255],
        style: TextureStyle::GradientRadial,
        size: (64, 64),
        secondary_color: Some([40, 30, 60, 200]),
        emissive: None,
    },
    // Berserker - blood red fury
    AssetTexture {
        id: "berserker",
        base_color: [160, 40, 30, 255],
        style: TextureStyle::GradientV,
        size: (64, 64),
        secondary_color: Some([100, 30, 25, 255]),
        emissive: None,
    },
    // Arcane Sentinel - magical construct
    AssetTexture {
        id: "arcane_sentinel",
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
        base_color: [150, 200, 220, 255],
        style: TextureStyle::Metal { seed: 123 },
        size: (64, 64),
        secondary_color: Some([100, 180, 200, 255]),
        emissive: None,
    },
    // Void Mage - deep purple with void energy
    AssetTexture {
        id: "void_mage",
        base_color: [40, 20, 60, 255],
        style: TextureStyle::GradientV,
        size: (64, 64),
        secondary_color: Some([80, 40, 120, 255]),
        emissive: None,
    },
    // Golem Titan - darker, more imposing stone
    AssetTexture {
        id: "golem_titan",
        base_color: [70, 60, 55, 255],
        style: TextureStyle::Stone { seed: 789 },
        size: (64, 64),
        secondary_color: Some([50, 45, 40, 255]),
        emissive: None,
    },
    // Specter Lord - ethereal blue-white
    AssetTexture {
        id: "specter_lord",
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
        base_color: [200, 180, 220, 255],
        style: TextureStyle::Metal { seed: 456 },
        size: (128, 128),
        secondary_color: Some([180, 200, 240, 255]),
        emissive: None,
    },
    // Void Dragon - deep black with purple accents
    AssetTexture {
        id: "void_dragon",
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
        base_color: [80, 150, 255, 255],
        style: TextureStyle::GradientRadial,
        size: (32, 32),
        secondary_color: Some([40, 100, 200, 255]),
        emissive: None,
    },
    // Coin - shiny gold
    AssetTexture {
        id: "coin",
        base_color: [255, 200, 50, 255],
        style: TextureStyle::Metal { seed: 333 },
        size: (32, 32),
        secondary_color: Some([200, 150, 30, 255]),
        emissive: None,
    },
    // Powerup Orb - radiant white
    AssetTexture {
        id: "powerup_orb",
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
        base_color: [180, 220, 255, 255],
        style: TextureStyle::GradientRadial,
        size: (32, 32),
        secondary_color: Some([100, 180, 240, 255]),
        emissive: None,
    },
    // Void Orb - dark purple energy
    AssetTexture {
        id: "void_orb",
        base_color: [60, 20, 80, 255],
        style: TextureStyle::GradientRadial,
        size: (32, 32),
        secondary_color: Some([100, 40, 140, 255]),
        emissive: None,
    },
    // Lightning Bolt - electric yellow-blue
    AssetTexture {
        id: "lightning_bolt",
        base_color: [255, 255, 100, 255],
        style: TextureStyle::GradientRadial,
        size: (32, 32),
        secondary_color: Some([150, 200, 255, 255]),
        emissive: None,
    },
];

use std::path::Path;
use crate::texture::generate_all_textures;

pub fn generate_hero_textures(output_dir: &Path) {
    println!("\n  Generating hero textures...");
    let heroes: Vec<_> = TEXTURES.iter()
        .filter(|t| ["knight", "mage", "ranger", "cleric", "necromancer", "paladin"].contains(&t.id))
        .cloned()
        .collect();
    generate_all_textures(&heroes, output_dir);
}

pub fn generate_enemy_textures(output_dir: &Path) {
    println!("\n  Generating enemy textures...");
    let enemies: Vec<_> = TEXTURES.iter()
        .filter(|t| ["golem", "crawler", "wisp", "skeleton", "shade", "berserker", "arcane_sentinel"].contains(&t.id))
        .cloned()
        .collect();
    generate_all_textures(&enemies, output_dir);
}

pub fn generate_elite_textures(output_dir: &Path) {
    println!("\n  Generating elite enemy textures...");
    let elites: Vec<_> = TEXTURES.iter()
        .filter(|t| ["crystal_knight", "void_mage", "golem_titan", "specter_lord"].contains(&t.id))
        .cloned()
        .collect();
    generate_all_textures(&elites, output_dir);
}

pub fn generate_boss_textures(output_dir: &Path) {
    println!("\n  Generating boss textures...");
    let bosses: Vec<_> = TEXTURES.iter()
        .filter(|t| ["prism_colossus", "void_dragon"].contains(&t.id))
        .cloned()
        .collect();
    generate_all_textures(&bosses, output_dir);
}

pub fn generate_pickup_textures(output_dir: &Path) {
    println!("\n  Generating pickup textures...");
    let pickups: Vec<_> = TEXTURES.iter()
        .filter(|t| ["xp_gem", "coin", "powerup_orb"].contains(&t.id))
        .cloned()
        .collect();
    generate_all_textures(&pickups, output_dir);
}

pub fn generate_arena_textures(output_dir: &Path) {
    println!("\n  Generating arena textures...");
    let arena: Vec<_> = TEXTURES.iter()
        .filter(|t| ["arena_floor"].contains(&t.id))
        .cloned()
        .collect();
    generate_all_textures(&arena, output_dir);
}

pub fn generate_projectile_textures(output_dir: &Path) {
    println!("\n  Generating projectile textures...");
    let projectiles: Vec<_> = TEXTURES.iter()
        .filter(|t| ["frost_shard", "void_orb", "lightning_bolt"].contains(&t.id))
        .cloned()
        .collect();
    generate_all_textures(&projectiles, output_dir);
}
