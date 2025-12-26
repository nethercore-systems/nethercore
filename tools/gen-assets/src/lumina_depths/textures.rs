//! Texture definitions for LUMINA DEPTHS
//!
//! Uses the convention-based texture system. Each asset has:
//! - `{id}.png` - Base texture
//! - Mode 3 uses Blinn-Phong (underwater lighting)

use crate::texture::{AssetTexture, TextureStyle};

/// All LUMINA DEPTHS asset textures - single source of truth
pub const TEXTURES: &[AssetTexture] = &[
    // === SUBMERSIBLE ===
    AssetTexture {
        id: "submersible",
        base_color: [140, 150, 160, 255],
        style: TextureStyle::Metal { seed: 11 },
        size: (64, 64),
        secondary_color: None,
        emissive: None,
    },

    // === CREATURES ===
    // Reef fish - tropical orange
    AssetTexture {
        id: "reef_fish",
        base_color: [255, 140, 50, 255],
        style: TextureStyle::GradientV,
        size: (32, 32),
        secondary_color: Some([215, 100, 10, 255]),
        emissive: None,
    },
    // Sea turtle - mottled green shell
    AssetTexture {
        id: "sea_turtle",
        base_color: [60, 80, 50, 255],
        style: TextureStyle::Stone { seed: 42 },
        size: (64, 64),
        secondary_color: None,
        emissive: None,
    },
    // Manta ray - dark dorsal
    AssetTexture {
        id: "manta_ray",
        base_color: [30, 35, 40, 255],
        style: TextureStyle::GradientV,
        size: (64, 64),
        secondary_color: Some([20, 25, 30, 255]),
        emissive: None,
    },
    // Moon jelly - translucent blue
    AssetTexture {
        id: "moon_jelly",
        base_color: [180, 200, 255, 180],
        style: TextureStyle::GradientRadial,
        size: (64, 64),
        secondary_color: Some([120, 140, 200, 100]),
        emissive: None,
    },
    // Anglerfish - deep dark body
    AssetTexture {
        id: "anglerfish",
        base_color: [15, 15, 20, 255],
        style: TextureStyle::Solid,
        size: (32, 32),
        secondary_color: None,
        emissive: None,
    },
    // Blue whale - blue-gray skin
    AssetTexture {
        id: "blue_whale",
        base_color: [60, 70, 85, 255],
        style: TextureStyle::Stone { seed: 77 },
        size: (128, 128),
        secondary_color: None,
        emissive: None,
    },
    // Tube worms - red plumes
    AssetTexture {
        id: "tube_worms",
        base_color: [200, 40, 40, 255],
        style: TextureStyle::Solid,
        size: (32, 32),
        secondary_color: None,
        emissive: None,
    },

    // === FLORA ===
    // Brain coral - pinkish tan
    AssetTexture {
        id: "coral_brain",
        base_color: [180, 140, 120, 255],
        style: TextureStyle::Stone { seed: 33 },
        size: (64, 64),
        secondary_color: None,
        emissive: None,
    },
    // Kelp - green-brown
    AssetTexture {
        id: "kelp",
        base_color: [60, 80, 40, 255],
        style: TextureStyle::GradientV,
        size: (64, 64),
        secondary_color: Some([40, 60, 30, 255]),
        emissive: None,
    },
    // Anemone - pink radial
    AssetTexture {
        id: "anemone",
        base_color: [255, 150, 180, 255],
        style: TextureStyle::GradientRadial,
        size: (32, 32),
        secondary_color: Some([200, 100, 130, 255]),
        emissive: None,
    },

    // === TERRAIN ===
    // Vent chimney - dark volcanic rock
    AssetTexture {
        id: "vent_chimney",
        base_color: [50, 45, 40, 255],
        style: TextureStyle::GradientV,
        size: (64, 64),
        secondary_color: Some([30, 28, 25, 255]),
        emissive: None,
    },
];

use std::path::Path;
use crate::texture::generate_all_textures;

pub fn generate_creature_textures(output_dir: &Path) {
    let creatures: Vec<_> = TEXTURES.iter()
        .filter(|t| [
            "reef_fish", "sea_turtle", "manta_ray", "moon_jelly",
            "anglerfish", "blue_whale", "tube_worms"
        ].contains(&t.id))
        .cloned()
        .collect();
    generate_all_textures(&creatures, output_dir);
}

pub fn generate_flora_textures(output_dir: &Path) {
    let flora: Vec<_> = TEXTURES.iter()
        .filter(|t| ["coral_brain", "kelp", "anemone"].contains(&t.id))
        .cloned()
        .collect();
    generate_all_textures(&flora, output_dir);
}

pub fn generate_terrain_textures(output_dir: &Path) {
    let terrain: Vec<_> = TEXTURES.iter()
        .filter(|t| ["vent_chimney"].contains(&t.id))
        .cloned()
        .collect();
    generate_all_textures(&terrain, output_dir);
}

pub fn generate_submersible_textures(output_dir: &Path) {
    let sub: Vec<_> = TEXTURES.iter()
        .filter(|t| t.id == "submersible")
        .cloned()
        .collect();
    generate_all_textures(&sub, output_dir);
}
