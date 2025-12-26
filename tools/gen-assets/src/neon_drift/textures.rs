//! Texture definitions for NEON DRIFT
//!
//! Uses the convention-based texture system. Each asset has:
//! - `{id}.png` - Base texture
//! - `{id}_emissive.png` - Emissive glow (for neon effects)

use crate::texture::{AssetTexture, TextureStyle};

/// All NEON DRIFT asset textures - single source of truth
pub const TEXTURES: &[AssetTexture] = &[
    // === VEHICLES ===
    // Sleek silver with cyan neon
    AssetTexture {
        id: "speedster",
        base_color: [180, 190, 200, 255],
        style: TextureStyle::Metal { seed: 1 },
        size: (128, 128),
        secondary_color: None,
        emissive: Some([0, 255, 255, 255]), // Cyan glow
    },
    // Dark gunmetal with orange neon
    AssetTexture {
        id: "muscle",
        base_color: [60, 60, 70, 255],
        style: TextureStyle::Metal { seed: 2 },
        size: (128, 128),
        secondary_color: None,
        emissive: Some([255, 100, 0, 255]), // Orange glow
    },
    // Clean white with magenta neon
    AssetTexture {
        id: "racer",
        base_color: [240, 240, 250, 255],
        style: TextureStyle::GradientH,
        size: (128, 128),
        secondary_color: Some([220, 220, 240, 255]),
        emissive: Some([255, 0, 180, 255]), // Magenta glow
    },
    // Dark purple with violet neon
    AssetTexture {
        id: "drift",
        base_color: [80, 80, 100, 255],
        style: TextureStyle::GradientV,
        size: (128, 128),
        secondary_color: Some([40, 40, 60, 255]),
        emissive: Some([180, 0, 255, 255]), // Violet glow
    },

    // === TRACK SEGMENTS ===
    // Asphalt road surface
    AssetTexture {
        id: "track_straight",
        base_color: [40, 40, 45, 255],
        style: TextureStyle::Stone { seed: 42 },
        size: (256, 256),
        secondary_color: None,
        emissive: None,
    },
    // Curved road surface (slight variation)
    AssetTexture {
        id: "track_curve_left",
        base_color: [42, 42, 48, 255],
        style: TextureStyle::Stone { seed: 57 },
        size: (256, 256),
        secondary_color: None,
        emissive: None,
    },
    // Metallic tunnel walls
    AssetTexture {
        id: "track_tunnel",
        base_color: [50, 50, 60, 255],
        style: TextureStyle::Metal { seed: 77 },
        size: (128, 128),
        secondary_color: None,
        emissive: None,
    },
    // Jump ramp with hazard stripes
    AssetTexture {
        id: "track_jump",
        base_color: [255, 200, 0, 255],
        style: TextureStyle::Checker { cell_size: 8 },
        size: (64, 64),
        secondary_color: Some([30, 30, 30, 255]),
        emissive: None,
    },

    // === PROPS ===
    // Concrete barrier
    AssetTexture {
        id: "prop_barrier",
        base_color: [80, 75, 70, 255],
        style: TextureStyle::Stone { seed: 123 },
        size: (64, 64),
        secondary_color: None,
        emissive: None,
    },
    // Glowing boost pad
    AssetTexture {
        id: "prop_boost_pad",
        base_color: [0, 255, 255, 255],
        style: TextureStyle::GradientRadial,
        size: (64, 64),
        secondary_color: Some([0, 150, 200, 255]),
        emissive: Some([0, 255, 255, 255]), // Cyan glow
    },
    // Billboard backing
    AssetTexture {
        id: "prop_billboard",
        base_color: [20, 20, 30, 255],
        style: TextureStyle::Solid,
        size: (128, 64),
        secondary_color: None,
        emissive: Some([255, 20, 147, 255]), // Neon pink
    },
    // City building facade
    AssetTexture {
        id: "prop_building",
        base_color: [30, 35, 50, 255],
        style: TextureStyle::GradientV,
        size: (128, 256),
        secondary_color: Some([20, 25, 40, 255]),
        emissive: None,
    },
];

use std::path::Path;
use crate::texture::generate_all_textures;

pub fn generate_vehicle_textures(output_dir: &Path) {
    let vehicles: Vec<_> = TEXTURES.iter()
        .filter(|t| ["speedster", "muscle", "racer", "drift"].contains(&t.id))
        .cloned()
        .collect();
    generate_all_textures(&vehicles, output_dir);
}

pub fn generate_track_textures(output_dir: &Path) {
    let tracks: Vec<_> = TEXTURES.iter()
        .filter(|t| t.id.starts_with("track_"))
        .cloned()
        .collect();
    generate_all_textures(&tracks, output_dir);
}

pub fn generate_prop_textures(output_dir: &Path) {
    let props: Vec<_> = TEXTURES.iter()
        .filter(|t| t.id.starts_with("prop_"))
        .cloned()
        .collect();
    generate_all_textures(&props, output_dir);
}
