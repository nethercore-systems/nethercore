//! Texture definitions for PRISM SURVIVORS
//!
//! Uses the convention-based texture system with category-based filtering.
//! Each asset has:
//! - `{id}.png` - Base texture
//! - Mode 3 uses Blinn-Phong (no emissive textures needed)

use crate::texture::{AssetCategory, AssetTexture, TextureStyle, generate_textures_by_category};
use std::path::Path;

/// All PRISM SURVIVORS asset textures - single source of truth
/// IMPROVED: Better styles for visual quality
pub const TEXTURES: &[AssetTexture] = &[
    // === HEROES ===
    // Knight - polished steel armor with blue tint
    AssetTexture {
        id: "knight",
        category: AssetCategory::Hero,
        base_color: [160, 165, 180, 255], // Brighter steel blue
        style: TextureStyle::Metal { seed: 42 },
        size: (128, 128), // Larger for detail
        secondary_color: Some([100, 110, 130, 255]),
        emissive: None,
    },
    // Mage - mystical purple robes with arcane shimmer
    AssetTexture {
        id: "mage",
        category: AssetCategory::Hero,
        base_color: [100, 50, 160, 255], // More saturated purple
        style: TextureStyle::Crystal { seed: 111 }, // Crystal for magical effect
        size: (128, 128),
        secondary_color: Some([60, 180, 220, 255]), // Cyan magical accents
        emissive: None,
    },
    // Ranger - forest green leather with natural texture
    AssetTexture {
        id: "ranger",
        category: AssetCategory::Hero,
        base_color: [70, 100, 50, 255], // More saturated green
        style: TextureStyle::Organic { seed: 222 }, // Organic leather look
        size: (128, 128),
        secondary_color: Some([90, 70, 50, 255]), // Brown leather accents
        emissive: None,
    },
    // Cleric - holy white/gold robes with radiance
    AssetTexture {
        id: "cleric",
        category: AssetCategory::Hero,
        base_color: [255, 250, 230, 255], // Brighter white
        style: TextureStyle::Shell { seed: 333 }, // Pearlescent holy glow
        size: (128, 128),
        secondary_color: Some([255, 220, 150, 255]), // Golden holy light
        emissive: None,
    },
    // Necromancer - dark purple/green robes with soul energy
    AssetTexture {
        id: "necromancer",
        category: AssetCategory::Hero,
        base_color: [40, 25, 50, 255], // Darker purple-black
        style: TextureStyle::Bioluminescent { seed: 444 }, // Glowing soul spots
        size: (128, 128),
        secondary_color: Some([80, 200, 100, 255]), // Sickly green soul glow
        emissive: None,
    },
    // Paladin - golden holy armor with divine radiance
    AssetTexture {
        id: "paladin",
        category: AssetCategory::Hero,
        base_color: [240, 200, 100, 255], // Brighter gold
        style: TextureStyle::Crystal { seed: 777 }, // Crystal for holy radiance
        size: (128, 128),
        secondary_color: Some([255, 255, 220, 255]), // White holy glow
        emissive: None,
    },

    // === BASIC ENEMIES ===
    // Golem - rocky stone with mossy cracks
    AssetTexture {
        id: "golem",
        category: AssetCategory::Enemy,
        base_color: [120, 105, 90, 255], // Warmer stone
        style: TextureStyle::Stone { seed: 42 },
        size: (128, 128),
        secondary_color: Some([60, 80, 50, 255]), // Mossy green in cracks
        emissive: None,
    },
    // Crawler - dark chitinous shell with scales
    AssetTexture {
        id: "crawler",
        category: AssetCategory::Enemy,
        base_color: [50, 40, 60, 255], // Dark purple-black chitin
        style: TextureStyle::Scales { seed: 55 }, // Chitin scales!
        size: (128, 128),
        secondary_color: Some([80, 60, 90, 255]), // Purple highlights
        emissive: None,
    },
    // Wisp - glowing ethereal with pulsing energy
    AssetTexture {
        id: "wisp",
        category: AssetCategory::Enemy,
        base_color: [255, 220, 120, 255], // Brighter golden core
        style: TextureStyle::Bioluminescent { seed: 66 }, // Pulsing glow
        size: (64, 64),
        secondary_color: Some([255, 180, 80, 255]), // Orange outer glow
        emissive: None,
    },
    // Skeleton - aged bone with weathering
    AssetTexture {
        id: "skeleton",
        category: AssetCategory::Enemy,
        base_color: [230, 220, 200, 255], // Aged bone white
        style: TextureStyle::Stone { seed: 77 }, // Bone texture like stone
        size: (128, 128),
        secondary_color: Some([180, 160, 130, 255]), // Darker aged spots
        emissive: None,
    },
    // Shade - dark ethereal shadow with wispy edges
    AssetTexture {
        id: "shade",
        category: AssetCategory::Enemy,
        base_color: [30, 20, 45, 255], // Darker shadow
        style: TextureStyle::Organic { seed: 88 }, // Wispy membrane look
        size: (128, 128),
        secondary_color: Some([60, 40, 100, 255]), // Purple shadow energy
        emissive: None,
    },
    // Berserker - blood red fury with rage marks
    AssetTexture {
        id: "berserker",
        category: AssetCategory::Enemy,
        base_color: [180, 50, 40, 255], // More saturated red
        style: TextureStyle::Organic { seed: 99 }, // Muscular organic texture
        size: (128, 128),
        secondary_color: Some([120, 30, 20, 255]), // Darker blood red
        emissive: None,
    },
    // Arcane Sentinel - magical construct with runes
    AssetTexture {
        id: "arcane_sentinel",
        category: AssetCategory::Enemy,
        base_color: [120, 100, 200, 255], // Brighter purple-blue
        style: TextureStyle::Crystal { seed: 555 }, // Crystalline construct
        size: (128, 128),
        secondary_color: Some([180, 150, 255, 255]), // Magical glow lines
        emissive: None,
    },

    // === ELITE ENEMIES ===
    // Crystal Knight - iridescent crystalline armor (USE CRYSTAL!)
    AssetTexture {
        id: "crystal_knight",
        category: AssetCategory::Elite,
        base_color: [180, 220, 255, 255], // Bright ice crystal
        style: TextureStyle::Crystal { seed: 123 }, // CRYSTAL style!
        size: (128, 128),
        secondary_color: Some([100, 200, 240, 255]), // Cyan crystal edges
        emissive: None,
    },
    // Void Mage - deep purple with void energy swirls
    AssetTexture {
        id: "void_mage",
        category: AssetCategory::Elite,
        base_color: [50, 25, 80, 255], // Deeper void purple
        style: TextureStyle::Crystal { seed: 234 }, // Void crystal energy
        size: (128, 128),
        secondary_color: Some([150, 80, 200, 255]), // Bright void glow
        emissive: None,
    },
    // Golem Titan - darker imposing stone with glowing cracks
    AssetTexture {
        id: "golem_titan",
        category: AssetCategory::Elite,
        base_color: [80, 70, 65, 255], // Darker ancient stone
        style: TextureStyle::Stone { seed: 789 },
        size: (128, 128),
        secondary_color: Some([160, 100, 60, 255]), // Molten orange cracks
        emissive: None,
    },
    // Specter Lord - ethereal royal ghost
    AssetTexture {
        id: "specter_lord",
        category: AssetCategory::Elite,
        base_color: [200, 220, 255, 255], // Bright ethereal blue
        style: TextureStyle::Shell { seed: 321 }, // Royal pearlescent
        size: (128, 128),
        secondary_color: Some([255, 255, 255, 255]), // Pure white crown glow
        emissive: None,
    },

    // === BOSSES (DRAMATIC!) ===
    // Prism Colossus - PRISMATIC RAINBOW CRYSTAL (the game's namesake!)
    AssetTexture {
        id: "prism_colossus",
        category: AssetCategory::Boss,
        base_color: [220, 200, 255, 255], // Bright prismatic base
        style: TextureStyle::Crystal { seed: 456 }, // CRYSTAL - dramatic facets!
        size: (256, 256), // Larger for boss detail
        secondary_color: Some([255, 150, 200, 255]), // Pink/magenta crystal edges
        emissive: None,
    },
    // Void Dragon - deep void with swirling dark energy
    AssetTexture {
        id: "void_dragon",
        category: AssetCategory::Boss,
        base_color: [40, 20, 60, 255], // Deep void purple-black
        style: TextureStyle::Crystal { seed: 567 }, // Crystal void scales
        size: (256, 256), // Larger for boss detail
        secondary_color: Some([120, 60, 160, 255]), // Bright purple void energy
        emissive: None,
    },

    // === PICKUPS ===
    // XP Gem - glowing faceted crystal (USE CRYSTAL!)
    AssetTexture {
        id: "xp_gem",
        category: AssetCategory::Pickup,
        base_color: [100, 180, 255, 255], // Brighter blue
        style: TextureStyle::Crystal { seed: 888 }, // Faceted gem!
        size: (64, 64),
        secondary_color: Some([50, 120, 255, 255]), // Deep blue edges
        emissive: None,
    },
    // Coin - shiny gold with polish
    AssetTexture {
        id: "coin",
        category: AssetCategory::Pickup,
        base_color: [255, 210, 80, 255], // Brighter gold
        style: TextureStyle::Metal { seed: 333 },
        size: (64, 64),
        secondary_color: Some([220, 170, 50, 255]),
        emissive: None,
    },
    // Powerup Orb - radiant white magical energy
    AssetTexture {
        id: "powerup_orb",
        category: AssetCategory::Pickup,
        base_color: [255, 255, 255, 255],
        style: TextureStyle::Bioluminescent { seed: 444 }, // Pulsing magical glow
        size: (64, 64),
        secondary_color: Some([220, 240, 255, 255]), // Soft blue glow
        emissive: None,
    },

    // === ARENA ===
    // Arena Floor - crystal cavern stone with glowing veins
    AssetTexture {
        id: "arena_floor",
        category: AssetCategory::Arena,
        base_color: [90, 85, 80, 255], // Warm cave stone
        style: TextureStyle::Stone { seed: 999 },
        size: (512, 512), // Larger for floor tiling
        secondary_color: Some([100, 150, 200, 255]), // Blue crystal veins
        emissive: None,
    },

    // === PROJECTILES ===
    // Frost Shard - icy crystal
    AssetTexture {
        id: "frost_shard",
        category: AssetCategory::Projectile,
        base_color: [200, 240, 255, 255], // Bright ice
        style: TextureStyle::Crystal { seed: 111 }, // Crystalline ice
        size: (32, 32),
        secondary_color: Some([120, 200, 255, 255]), // Blue ice edges
        emissive: None,
    },
    // Void Orb - dark purple energy sphere
    AssetTexture {
        id: "void_orb",
        category: AssetCategory::Projectile,
        base_color: [80, 30, 120, 255], // Deeper purple
        style: TextureStyle::Bioluminescent { seed: 222 }, // Pulsing void
        size: (32, 32),
        secondary_color: Some([150, 80, 200, 255]), // Bright void glow
        emissive: None,
    },
    // Lightning Bolt - electric energy
    AssetTexture {
        id: "lightning_bolt",
        category: AssetCategory::Projectile,
        base_color: [255, 255, 150, 255], // Bright yellow
        style: TextureStyle::Bioluminescent { seed: 333 }, // Electric sparks
        size: (32, 32),
        secondary_color: Some([180, 220, 255, 255]), // Blue electric edges
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
