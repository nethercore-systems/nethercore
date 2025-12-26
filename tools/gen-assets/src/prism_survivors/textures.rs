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

    // === ENEMIES ===
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
];

use std::path::Path;
use crate::texture::generate_all_textures;

pub fn generate_hero_textures(output_dir: &Path) {
    println!("\n  Generating hero textures...");
    let heroes: Vec<_> = TEXTURES.iter()
        .filter(|t| ["knight", "mage", "ranger", "cleric"].contains(&t.id))
        .cloned()
        .collect();
    generate_all_textures(&heroes, output_dir);
}

pub fn generate_enemy_textures(output_dir: &Path) {
    println!("\n  Generating enemy textures...");
    let enemies: Vec<_> = TEXTURES.iter()
        .filter(|t| ["golem", "crawler", "wisp", "skeleton"].contains(&t.id))
        .cloned()
        .collect();
    generate_all_textures(&enemies, output_dir);
}
