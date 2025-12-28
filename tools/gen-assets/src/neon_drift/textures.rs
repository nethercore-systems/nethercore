//! Texture definitions for NEON DRIFT
//!
//! Uses the convention-based texture system. Each asset has:
//! - `{id}.png` - Base texture
//! - `{id}_emissive.png` - Emissive glow (for neon effects)

use crate::texture::{AssetCategory, AssetTexture, TextureStyle, generate_textures_by_category};

/// All NEON DRIFT asset textures - single source of truth
pub const TEXTURES: &[AssetTexture] = &[
    // === VEHICLES ===
    // Sleek silver with cyan neon
    AssetTexture {
        id: "speedster",
        category: AssetCategory::Hero,
        base_color: [180, 190, 200, 255],
        style: TextureStyle::Metal { seed: 1 },
        size: (128, 128),
        secondary_color: None,
        emissive: Some([0, 255, 255, 255]), // Cyan glow
    },
    // Dark gunmetal with orange neon
    AssetTexture {
        id: "muscle",
        category: AssetCategory::Hero,
        base_color: [60, 60, 70, 255],
        style: TextureStyle::Metal { seed: 2 },
        size: (128, 128),
        secondary_color: None,
        emissive: Some([255, 100, 0, 255]), // Orange glow
    },
    // Clean white with magenta neon
    AssetTexture {
        id: "racer",
        category: AssetCategory::Hero,
        base_color: [240, 240, 250, 255],
        style: TextureStyle::GradientH,
        size: (128, 128),
        secondary_color: Some([220, 220, 240, 255]),
        emissive: Some([255, 0, 180, 255]), // Magenta glow
    },
    // Dark purple with violet neon
    AssetTexture {
        id: "drift",
        category: AssetCategory::Hero,
        base_color: [80, 80, 100, 255],
        style: TextureStyle::GradientV,
        size: (128, 128),
        secondary_color: Some([40, 40, 60, 255]),
        emissive: Some([180, 0, 255, 255]), // Violet glow
    },
    // Dark charcoal with toxic green neon (stealth supercar)
    AssetTexture {
        id: "phantom",
        category: AssetCategory::Hero,
        base_color: [40, 45, 50, 255],
        style: TextureStyle::Metal { seed: 5 },
        size: (128, 128),
        secondary_color: None,
        emissive: Some([0, 255, 100, 255]), // Toxic green glow
    },
    // Gunmetal silver with pure white neon (luxury GT)
    AssetTexture {
        id: "titan",
        category: AssetCategory::Hero,
        base_color: [100, 105, 115, 255],
        style: TextureStyle::Metal { seed: 6 },
        size: (128, 128),
        secondary_color: None,
        emissive: Some([255, 255, 255, 255]), // Pure white glow
    },
    // Venom red with gold neon (hypercar)
    AssetTexture {
        id: "viper",
        category: AssetCategory::Hero,
        base_color: [180, 20, 30, 255],
        style: TextureStyle::GradientV,
        size: (128, 128),
        secondary_color: Some([140, 15, 25, 255]),
        emissive: Some([255, 200, 0, 255]), // Gold glow
    },

    // === VEHICLE COLOR VARIANTS ===
    // Speedster variants
    AssetTexture {
        id: "speedster_red",
        category: AssetCategory::Hero,
        base_color: [200, 60, 60, 255],
        style: TextureStyle::Metal { seed: 11 },
        size: (128, 128),
        secondary_color: None,
        emissive: Some([255, 50, 50, 255]), // Red neon
    },
    AssetTexture {
        id: "speedster_gold",
        category: AssetCategory::Hero,
        base_color: [200, 180, 100, 255],
        style: TextureStyle::Metal { seed: 12 },
        size: (128, 128),
        secondary_color: None,
        emissive: Some([255, 200, 0, 255]), // Gold neon
    },
    // Muscle variants
    AssetTexture {
        id: "muscle_blue",
        category: AssetCategory::Hero,
        base_color: [40, 50, 90, 255],
        style: TextureStyle::Metal { seed: 21 },
        size: (128, 128),
        secondary_color: None,
        emissive: Some([0, 150, 255, 255]), // Blue neon
    },
    AssetTexture {
        id: "muscle_green",
        category: AssetCategory::Hero,
        base_color: [50, 70, 50, 255],
        style: TextureStyle::Metal { seed: 22 },
        size: (128, 128),
        secondary_color: None,
        emissive: Some([0, 255, 150, 255]), // Green neon
    },
    // Racer variants
    AssetTexture {
        id: "racer_black",
        category: AssetCategory::Hero,
        base_color: [30, 30, 35, 255],
        style: TextureStyle::GradientH,
        size: (128, 128),
        secondary_color: Some([50, 50, 60, 255]),
        emissive: Some([255, 0, 255, 255]), // Magenta neon
    },
    AssetTexture {
        id: "racer_cyan",
        category: AssetCategory::Hero,
        base_color: [200, 240, 250, 255],
        style: TextureStyle::GradientH,
        size: (128, 128),
        secondary_color: Some([150, 220, 240, 255]),
        emissive: Some([0, 255, 255, 255]), // Cyan neon
    },
    // Drift variants
    AssetTexture {
        id: "drift_orange",
        category: AssetCategory::Hero,
        base_color: [120, 60, 30, 255],
        style: TextureStyle::GradientV,
        size: (128, 128),
        secondary_color: Some([80, 40, 20, 255]),
        emissive: Some([255, 120, 0, 255]), // Orange neon
    },
    AssetTexture {
        id: "drift_pink",
        category: AssetCategory::Hero,
        base_color: [120, 60, 100, 255],
        style: TextureStyle::GradientV,
        size: (128, 128),
        secondary_color: Some([80, 40, 70, 255]),
        emissive: Some([255, 100, 200, 255]), // Pink neon
    },
    // Phantom variants
    AssetTexture {
        id: "phantom_purple",
        category: AssetCategory::Hero,
        base_color: [50, 40, 60, 255],
        style: TextureStyle::Metal { seed: 51 },
        size: (128, 128),
        secondary_color: None,
        emissive: Some([150, 0, 255, 255]), // Purple neon
    },
    AssetTexture {
        id: "phantom_ice",
        category: AssetCategory::Hero,
        base_color: [60, 70, 80, 255],
        style: TextureStyle::Metal { seed: 52 },
        size: (128, 128),
        secondary_color: None,
        emissive: Some([180, 220, 255, 255]), // Ice blue neon
    },
    // Titan variants
    AssetTexture {
        id: "titan_gold",
        category: AssetCategory::Hero,
        base_color: [150, 130, 80, 255],
        style: TextureStyle::Metal { seed: 61 },
        size: (128, 128),
        secondary_color: None,
        emissive: Some([255, 220, 100, 255]), // Warm gold glow
    },
    AssetTexture {
        id: "titan_midnight",
        category: AssetCategory::Hero,
        base_color: [30, 35, 50, 255],
        style: TextureStyle::Metal { seed: 62 },
        size: (128, 128),
        secondary_color: None,
        emissive: Some([100, 150, 255, 255]), // Blue glow
    },
    // Viper variants
    AssetTexture {
        id: "viper_green",
        category: AssetCategory::Hero,
        base_color: [30, 120, 50, 255],
        style: TextureStyle::GradientV,
        size: (128, 128),
        secondary_color: Some([20, 80, 35, 255]),
        emissive: Some([100, 255, 50, 255]), // Lime neon
    },
    AssetTexture {
        id: "viper_black",
        category: AssetCategory::Hero,
        base_color: [25, 25, 30, 255],
        style: TextureStyle::GradientV,
        size: (128, 128),
        secondary_color: Some([15, 15, 20, 255]),
        emissive: Some([255, 50, 50, 255]), // Red neon
    },

    // === TRACK SEGMENTS ===
    // Asphalt road surface
    AssetTexture {
        id: "track_straight",
        category: AssetCategory::Arena,
        base_color: [40, 40, 45, 255],
        style: TextureStyle::Stone { seed: 42 },
        size: (256, 256),
        secondary_color: None,
        emissive: None,
    },
    // Curved road surface (slight variation)
    AssetTexture {
        id: "track_curve_left",
        category: AssetCategory::Arena,
        base_color: [42, 42, 48, 255],
        style: TextureStyle::Stone { seed: 57 },
        size: (256, 256),
        secondary_color: None,
        emissive: None,
    },
    // Metallic tunnel walls
    AssetTexture {
        id: "track_tunnel",
        category: AssetCategory::Arena,
        base_color: [50, 50, 60, 255],
        style: TextureStyle::Metal { seed: 77 },
        size: (128, 128),
        secondary_color: None,
        emissive: None,
    },
    // Jump ramp with hazard stripes
    AssetTexture {
        id: "track_jump",
        category: AssetCategory::Arena,
        base_color: [255, 200, 0, 255],
        style: TextureStyle::Checker { cell_size: 8 },
        size: (64, 64),
        secondary_color: Some([30, 30, 30, 255]),
        emissive: None,
    },

    // === CRYSTAL CAVERN ===
    // Glowing crystal formations
    AssetTexture {
        id: "crystal_formation",
        category: AssetCategory::Effect,
        base_color: [80, 0, 120, 255],
        style: TextureStyle::GradientRadial,
        size: (128, 128),
        secondary_color: Some([160, 0, 255, 255]),
        emissive: Some([180, 0, 255, 255]), // Purple crystal glow
    },
    // Cavern S-curve with crystal reflections
    AssetTexture {
        id: "track_cavern_scurve",
        category: AssetCategory::Arena,
        base_color: [25, 20, 35, 255],
        style: TextureStyle::Stone { seed: 201 },
        size: (256, 256),
        secondary_color: None,
        emissive: Some([100, 0, 200, 180]), // Faint purple ambient
    },
    // Low ceiling cavern section
    AssetTexture {
        id: "track_cavern_low",
        category: AssetCategory::Arena,
        base_color: [30, 25, 40, 255],
        style: TextureStyle::Stone { seed: 202 },
        size: (256, 256),
        secondary_color: None,
        emissive: Some([0, 255, 255, 150]), // Cyan crystal vein glow
    },

    // === SOLAR HIGHWAY ===
    // Long high-speed straight with solar panels
    AssetTexture {
        id: "track_solar_straight",
        category: AssetCategory::Arena,
        base_color: [60, 55, 50, 255],
        style: TextureStyle::Metal { seed: 301 },
        size: (256, 256),
        secondary_color: None,
        emissive: Some([255, 200, 0, 150]), // Golden solar glow
    },
    // Wide sweeping solar curve
    AssetTexture {
        id: "track_solar_curve",
        category: AssetCategory::Arena,
        base_color: [55, 50, 48, 255],
        style: TextureStyle::Metal { seed: 302 },
        size: (256, 256),
        secondary_color: None,
        emissive: Some([255, 150, 0, 120]), // Orange ambient
    },
    // Dramatic solar flare jump
    AssetTexture {
        id: "track_solar_jump",
        category: AssetCategory::Arena,
        base_color: [255, 180, 50, 255],
        style: TextureStyle::GradientV,
        size: (128, 128),
        secondary_color: Some([255, 100, 0, 255]),
        emissive: Some([255, 255, 100, 255]), // Bright solar glow
    },

    // === PROPS ===
    // Concrete barrier
    AssetTexture {
        id: "prop_barrier",
        category: AssetCategory::Pickup,
        base_color: [80, 75, 70, 255],
        style: TextureStyle::Stone { seed: 123 },
        size: (64, 64),
        secondary_color: None,
        emissive: None,
    },
    // Glowing boost pad
    AssetTexture {
        id: "prop_boost_pad",
        category: AssetCategory::Pickup,
        base_color: [0, 255, 255, 255],
        style: TextureStyle::GradientRadial,
        size: (64, 64),
        secondary_color: Some([0, 150, 200, 255]),
        emissive: Some([0, 255, 255, 255]), // Cyan glow
    },
    // Billboard backing
    AssetTexture {
        id: "prop_billboard",
        category: AssetCategory::Pickup,
        base_color: [20, 20, 30, 255],
        style: TextureStyle::Solid,
        size: (128, 64),
        secondary_color: None,
        emissive: Some([255, 20, 147, 255]), // Neon pink
    },
    // City building facade
    AssetTexture {
        id: "prop_building",
        category: AssetCategory::Pickup,
        base_color: [30, 35, 50, 255],
        style: TextureStyle::GradientV,
        size: (128, 256),
        secondary_color: Some([20, 25, 40, 255]),
        emissive: None,
    },

    // === SUNSET STRIP PROPS ===
    // Palm tree bark and fronds
    AssetTexture {
        id: "prop_palm_tree",
        category: AssetCategory::Pickup,
        base_color: [110, 80, 50, 255],
        style: TextureStyle::Stone { seed: 401 },
        size: (128, 128),
        secondary_color: Some([50, 120, 40, 255]), // Green fronds
        emissive: None,
    },
    // Retro highway sign
    AssetTexture {
        id: "prop_highway_sign",
        category: AssetCategory::Pickup,
        base_color: [40, 80, 40, 255],
        style: TextureStyle::Solid,
        size: (128, 64),
        secondary_color: None,
        emissive: Some([255, 200, 100, 255]), // Warm neon border
    },

    // === NEON CITY PROPS ===
    // Holographic advertisement
    AssetTexture {
        id: "prop_hologram_ad",
        category: AssetCategory::Pickup,
        base_color: [20, 30, 60, 255],
        style: TextureStyle::GradientRadial,
        size: (128, 128),
        secondary_color: Some([50, 100, 200, 255]),
        emissive: Some([0, 200, 255, 200]), // Hologram blue glow
    },
    // Neon street lamp
    AssetTexture {
        id: "prop_street_lamp",
        category: AssetCategory::Pickup,
        base_color: [60, 60, 70, 255],
        style: TextureStyle::Metal { seed: 402 },
        size: (64, 128),
        secondary_color: None,
        emissive: Some([255, 100, 200, 255]), // Pink lamp glow
    },

    // === VOID TUNNEL PROPS ===
    // Energy pillar
    AssetTexture {
        id: "prop_energy_pillar",
        category: AssetCategory::Pickup,
        base_color: [15, 15, 30, 255],
        style: TextureStyle::GradientV,
        size: (64, 128),
        secondary_color: Some([40, 20, 80, 255]),
        emissive: Some([100, 0, 255, 255]), // Purple arcane glow
    },
    // Portal ring
    AssetTexture {
        id: "prop_portal_ring",
        category: AssetCategory::Pickup,
        base_color: [10, 10, 20, 255],
        style: TextureStyle::Metal { seed: 403 },
        size: (128, 128),
        secondary_color: None,
        emissive: Some([0, 255, 200, 255]), // Teal portal glow
    },

    // === CRYSTAL CAVERN PROPS ===
    // Glowing mushrooms
    AssetTexture {
        id: "prop_mushrooms",
        category: AssetCategory::Pickup,
        base_color: [60, 40, 80, 255],
        style: TextureStyle::GradientRadial,
        size: (64, 64),
        secondary_color: Some([100, 60, 140, 255]),
        emissive: Some([180, 100, 255, 255]), // Bioluminescent purple
    },

    // === SOLAR HIGHWAY PROPS ===
    // Heat vent
    AssetTexture {
        id: "prop_heat_vent",
        category: AssetCategory::Pickup,
        base_color: [80, 60, 40, 255],
        style: TextureStyle::Metal { seed: 404 },
        size: (64, 64),
        secondary_color: None,
        emissive: Some([255, 150, 50, 255]), // Hot orange glow
    },
    // Solar beacon
    AssetTexture {
        id: "prop_solar_beacon",
        category: AssetCategory::Pickup,
        base_color: [180, 160, 140, 255],
        style: TextureStyle::Metal { seed: 405 },
        size: (64, 128),
        secondary_color: None,
        emissive: Some([255, 255, 200, 255]), // Bright solar light
    },
];

use std::path::Path;
use proc_gen::texture::*;

/// Generate all textures for Neon Drift
pub fn generate_all(output_dir: &Path) {
    use AssetCategory::*;
    for category in [Hero, Arena, Effect, Pickup] {
        generate_textures_by_category(TEXTURES, category, output_dir);
    }
    // Additional procedural textures
    generate_road_with_markings(output_dir);
    generate_building_varieties(output_dir);
    generate_chevron_boost_pad(output_dir);
}

/// Generate the custom neon font texture
/// Layout: 16 chars per row, 6 rows = 96 characters (space through ~)
/// Each cell is 16x16 pixels, total texture size: 256x96
pub fn generate_font_texture(output_dir: &Path) {
    const CHAR_W: u32 = 16;
    const CHAR_H: u32 = 16;
    const COLS: u32 = 16;
    const ROWS: u32 = 6;
    const TEX_W: u32 = COLS * CHAR_W;  // 256
    const TEX_H: u32 = ROWS * CHAR_H;  // 96

    let mut tex = TextureBuffer::new(TEX_W, TEX_H);

    // Character definitions - simple 8x12 pixel glyphs centered in 16x16 cells
    // Format: each character is 12 rows of 8 bits
    let font_data: [(char, [u8; 12]); 95] = [
        // Row 0: space ! " # $ % & ' ( ) * + , - . /
        (' ', [0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000]),
        ('!', [0b00011000, 0b00011000, 0b00011000, 0b00011000, 0b00011000, 0b00011000, 0b00000000, 0b00011000, 0b00011000, 0b00000000, 0b00000000, 0b00000000]),
        ('"', [0b01100110, 0b01100110, 0b01100110, 0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000]),
        ('#', [0b00100100, 0b00100100, 0b11111111, 0b00100100, 0b00100100, 0b11111111, 0b00100100, 0b00100100, 0b00000000, 0b00000000, 0b00000000, 0b00000000]),
        ('$', [0b00011000, 0b01111110, 0b11011000, 0b01111110, 0b00011011, 0b01111110, 0b00011000, 0b00011000, 0b00000000, 0b00000000, 0b00000000, 0b00000000]),
        ('%', [0b01100000, 0b11100110, 0b00001100, 0b00011000, 0b00110000, 0b01100111, 0b00000110, 0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000]),
        ('&', [0b00111000, 0b01101100, 0b00111000, 0b01110000, 0b11011110, 0b11001100, 0b01110110, 0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000]),
        ('\'', [0b00011000, 0b00011000, 0b00110000, 0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000]),
        ('(', [0b00001100, 0b00011000, 0b00110000, 0b00110000, 0b00110000, 0b00011000, 0b00001100, 0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000]),
        (')', [0b00110000, 0b00011000, 0b00001100, 0b00001100, 0b00001100, 0b00011000, 0b00110000, 0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000]),
        ('*', [0b00000000, 0b01100110, 0b00111100, 0b11111111, 0b00111100, 0b01100110, 0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000]),
        ('+', [0b00000000, 0b00011000, 0b00011000, 0b01111110, 0b00011000, 0b00011000, 0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000]),
        (',', [0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00011000, 0b00011000, 0b00110000, 0b00000000, 0b00000000, 0b00000000, 0b00000000]),
        ('-', [0b00000000, 0b00000000, 0b00000000, 0b01111110, 0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000]),
        ('.', [0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00011000, 0b00011000, 0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000]),
        ('/', [0b00000110, 0b00001100, 0b00011000, 0b00110000, 0b01100000, 0b11000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000]),
        // Row 1: 0-9 : ; < = > ?
        ('0', [0b00111100, 0b01100110, 0b01101110, 0b01110110, 0b01100110, 0b01100110, 0b00111100, 0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000]),
        ('1', [0b00011000, 0b00111000, 0b00011000, 0b00011000, 0b00011000, 0b00011000, 0b01111110, 0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000]),
        ('2', [0b00111100, 0b01100110, 0b00000110, 0b00011100, 0b00110000, 0b01100110, 0b01111110, 0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000]),
        ('3', [0b00111100, 0b01100110, 0b00000110, 0b00011100, 0b00000110, 0b01100110, 0b00111100, 0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000]),
        ('4', [0b00001100, 0b00011100, 0b00111100, 0b01101100, 0b01111110, 0b00001100, 0b00001100, 0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000]),
        ('5', [0b01111110, 0b01100000, 0b01111100, 0b00000110, 0b00000110, 0b01100110, 0b00111100, 0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000]),
        ('6', [0b00011100, 0b00110000, 0b01100000, 0b01111100, 0b01100110, 0b01100110, 0b00111100, 0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000]),
        ('7', [0b01111110, 0b01100110, 0b00000110, 0b00001100, 0b00011000, 0b00011000, 0b00011000, 0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000]),
        ('8', [0b00111100, 0b01100110, 0b01100110, 0b00111100, 0b01100110, 0b01100110, 0b00111100, 0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000]),
        ('9', [0b00111100, 0b01100110, 0b01100110, 0b00111110, 0b00000110, 0b00001100, 0b00111000, 0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000]),
        (':', [0b00000000, 0b00011000, 0b00011000, 0b00000000, 0b00000000, 0b00011000, 0b00011000, 0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000]),
        (';', [0b00000000, 0b00011000, 0b00011000, 0b00000000, 0b00000000, 0b00011000, 0b00011000, 0b00110000, 0b00000000, 0b00000000, 0b00000000, 0b00000000]),
        ('<', [0b00001100, 0b00011000, 0b00110000, 0b01100000, 0b00110000, 0b00011000, 0b00001100, 0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000]),
        ('=', [0b00000000, 0b00000000, 0b01111110, 0b00000000, 0b01111110, 0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000]),
        ('>', [0b00110000, 0b00011000, 0b00001100, 0b00000110, 0b00001100, 0b00011000, 0b00110000, 0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000]),
        ('?', [0b00111100, 0b01100110, 0b00000110, 0b00001100, 0b00011000, 0b00000000, 0b00011000, 0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000]),
        // Row 2: @ A-O
        ('@', [0b00111100, 0b01100110, 0b01101110, 0b01101110, 0b01100000, 0b01100010, 0b00111100, 0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000]),
        ('A', [0b00011000, 0b00111100, 0b01100110, 0b01100110, 0b01111110, 0b01100110, 0b01100110, 0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000]),
        ('B', [0b01111100, 0b01100110, 0b01100110, 0b01111100, 0b01100110, 0b01100110, 0b01111100, 0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000]),
        ('C', [0b00111100, 0b01100110, 0b01100000, 0b01100000, 0b01100000, 0b01100110, 0b00111100, 0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000]),
        ('D', [0b01111000, 0b01101100, 0b01100110, 0b01100110, 0b01100110, 0b01101100, 0b01111000, 0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000]),
        ('E', [0b01111110, 0b01100000, 0b01100000, 0b01111100, 0b01100000, 0b01100000, 0b01111110, 0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000]),
        ('F', [0b01111110, 0b01100000, 0b01100000, 0b01111100, 0b01100000, 0b01100000, 0b01100000, 0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000]),
        ('G', [0b00111100, 0b01100110, 0b01100000, 0b01101110, 0b01100110, 0b01100110, 0b00111100, 0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000]),
        ('H', [0b01100110, 0b01100110, 0b01100110, 0b01111110, 0b01100110, 0b01100110, 0b01100110, 0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000]),
        ('I', [0b00111100, 0b00011000, 0b00011000, 0b00011000, 0b00011000, 0b00011000, 0b00111100, 0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000]),
        ('J', [0b00011110, 0b00001100, 0b00001100, 0b00001100, 0b00001100, 0b01101100, 0b00111000, 0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000]),
        ('K', [0b01100110, 0b01101100, 0b01111000, 0b01110000, 0b01111000, 0b01101100, 0b01100110, 0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000]),
        ('L', [0b01100000, 0b01100000, 0b01100000, 0b01100000, 0b01100000, 0b01100000, 0b01111110, 0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000]),
        ('M', [0b01100011, 0b01110111, 0b01111111, 0b01101011, 0b01100011, 0b01100011, 0b01100011, 0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000]),
        ('N', [0b01100110, 0b01110110, 0b01111110, 0b01111110, 0b01101110, 0b01100110, 0b01100110, 0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000]),
        ('O', [0b00111100, 0b01100110, 0b01100110, 0b01100110, 0b01100110, 0b01100110, 0b00111100, 0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000]),
        // Row 3: P-Z [ \ ] ^ _ `
        ('P', [0b01111100, 0b01100110, 0b01100110, 0b01111100, 0b01100000, 0b01100000, 0b01100000, 0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000]),
        ('Q', [0b00111100, 0b01100110, 0b01100110, 0b01100110, 0b01100110, 0b00111100, 0b00001110, 0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000]),
        ('R', [0b01111100, 0b01100110, 0b01100110, 0b01111100, 0b01111000, 0b01101100, 0b01100110, 0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000]),
        ('S', [0b00111100, 0b01100110, 0b01100000, 0b00111100, 0b00000110, 0b01100110, 0b00111100, 0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000]),
        ('T', [0b01111110, 0b00011000, 0b00011000, 0b00011000, 0b00011000, 0b00011000, 0b00011000, 0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000]),
        ('U', [0b01100110, 0b01100110, 0b01100110, 0b01100110, 0b01100110, 0b01100110, 0b00111100, 0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000]),
        ('V', [0b01100110, 0b01100110, 0b01100110, 0b01100110, 0b01100110, 0b00111100, 0b00011000, 0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000]),
        ('W', [0b01100011, 0b01100011, 0b01100011, 0b01101011, 0b01111111, 0b01110111, 0b01100011, 0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000]),
        ('X', [0b01100110, 0b01100110, 0b00111100, 0b00011000, 0b00111100, 0b01100110, 0b01100110, 0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000]),
        ('Y', [0b01100110, 0b01100110, 0b01100110, 0b00111100, 0b00011000, 0b00011000, 0b00011000, 0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000]),
        ('Z', [0b01111110, 0b00000110, 0b00001100, 0b00011000, 0b00110000, 0b01100000, 0b01111110, 0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000]),
        ('[', [0b00111100, 0b00110000, 0b00110000, 0b00110000, 0b00110000, 0b00110000, 0b00111100, 0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000]),
        ('\\', [0b11000000, 0b01100000, 0b00110000, 0b00011000, 0b00001100, 0b00000110, 0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000]),
        (']', [0b00111100, 0b00001100, 0b00001100, 0b00001100, 0b00001100, 0b00001100, 0b00111100, 0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000]),
        ('^', [0b00011000, 0b00111100, 0b01100110, 0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000]),
        ('_', [0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b01111111, 0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000]),
        // Row 4: ` a-o
        ('`', [0b00110000, 0b00011000, 0b00001100, 0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000]),
        ('a', [0b00000000, 0b00000000, 0b00111100, 0b00000110, 0b00111110, 0b01100110, 0b00111110, 0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000]),
        ('b', [0b01100000, 0b01100000, 0b01111100, 0b01100110, 0b01100110, 0b01100110, 0b01111100, 0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000]),
        ('c', [0b00000000, 0b00000000, 0b00111100, 0b01100000, 0b01100000, 0b01100000, 0b00111100, 0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000]),
        ('d', [0b00000110, 0b00000110, 0b00111110, 0b01100110, 0b01100110, 0b01100110, 0b00111110, 0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000]),
        ('e', [0b00000000, 0b00000000, 0b00111100, 0b01100110, 0b01111110, 0b01100000, 0b00111100, 0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000]),
        ('f', [0b00011100, 0b00110110, 0b00110000, 0b01111000, 0b00110000, 0b00110000, 0b00110000, 0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000]),
        ('g', [0b00000000, 0b00000000, 0b00111110, 0b01100110, 0b01100110, 0b00111110, 0b00000110, 0b01111100, 0b00000000, 0b00000000, 0b00000000, 0b00000000]),
        ('h', [0b01100000, 0b01100000, 0b01111100, 0b01100110, 0b01100110, 0b01100110, 0b01100110, 0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000]),
        ('i', [0b00011000, 0b00000000, 0b00111000, 0b00011000, 0b00011000, 0b00011000, 0b00111100, 0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000]),
        ('j', [0b00001100, 0b00000000, 0b00001100, 0b00001100, 0b00001100, 0b00001100, 0b01101100, 0b00111000, 0b00000000, 0b00000000, 0b00000000, 0b00000000]),
        ('k', [0b01100000, 0b01100000, 0b01100110, 0b01101100, 0b01111000, 0b01101100, 0b01100110, 0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000]),
        ('l', [0b00111000, 0b00011000, 0b00011000, 0b00011000, 0b00011000, 0b00011000, 0b00111100, 0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000]),
        ('m', [0b00000000, 0b00000000, 0b01100110, 0b01111111, 0b01111111, 0b01101011, 0b01100011, 0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000]),
        ('n', [0b00000000, 0b00000000, 0b01111100, 0b01100110, 0b01100110, 0b01100110, 0b01100110, 0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000]),
        ('o', [0b00000000, 0b00000000, 0b00111100, 0b01100110, 0b01100110, 0b01100110, 0b00111100, 0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000]),
        // Row 5: p-z { | } ~
        ('p', [0b00000000, 0b00000000, 0b01111100, 0b01100110, 0b01100110, 0b01111100, 0b01100000, 0b01100000, 0b00000000, 0b00000000, 0b00000000, 0b00000000]),
        ('q', [0b00000000, 0b00000000, 0b00111110, 0b01100110, 0b01100110, 0b00111110, 0b00000110, 0b00000110, 0b00000000, 0b00000000, 0b00000000, 0b00000000]),
        ('r', [0b00000000, 0b00000000, 0b01111100, 0b01100110, 0b01100000, 0b01100000, 0b01100000, 0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000]),
        ('s', [0b00000000, 0b00000000, 0b00111110, 0b01100000, 0b00111100, 0b00000110, 0b01111100, 0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000]),
        ('t', [0b00110000, 0b00110000, 0b01111100, 0b00110000, 0b00110000, 0b00110110, 0b00011100, 0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000]),
        ('u', [0b00000000, 0b00000000, 0b01100110, 0b01100110, 0b01100110, 0b01100110, 0b00111110, 0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000]),
        ('v', [0b00000000, 0b00000000, 0b01100110, 0b01100110, 0b01100110, 0b00111100, 0b00011000, 0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000]),
        ('w', [0b00000000, 0b00000000, 0b01100011, 0b01101011, 0b01111111, 0b01111111, 0b00110110, 0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000]),
        ('x', [0b00000000, 0b00000000, 0b01100110, 0b00111100, 0b00011000, 0b00111100, 0b01100110, 0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000]),
        ('y', [0b00000000, 0b00000000, 0b01100110, 0b01100110, 0b01100110, 0b00111110, 0b00000110, 0b01111100, 0b00000000, 0b00000000, 0b00000000, 0b00000000]),
        ('z', [0b00000000, 0b00000000, 0b01111110, 0b00001100, 0b00011000, 0b00110000, 0b01111110, 0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000]),
        ('{', [0b00001110, 0b00011000, 0b00011000, 0b01110000, 0b00011000, 0b00011000, 0b00001110, 0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000]),
        ('|', [0b00011000, 0b00011000, 0b00011000, 0b00000000, 0b00011000, 0b00011000, 0b00011000, 0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000]),
        ('}', [0b01110000, 0b00011000, 0b00011000, 0b00001110, 0b00011000, 0b00011000, 0b01110000, 0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000]),
        ('~', [0b00000000, 0b01110110, 0b11011100, 0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000]),
    ];

    // Neon colors - cyan primary with magenta/pink glow effect
    let core_color: [u8; 4] = [255, 255, 255, 255];   // White core
    let glow_color: [u8; 4] = [0, 255, 255, 180];     // Cyan glow (outer)
    let glow2_color: [u8; 4] = [0, 200, 255, 120];    // Softer cyan (middle)

    // Render each character
    for (char_idx, (_ch, glyph)) in font_data.iter().enumerate() {
        let col = char_idx % COLS as usize;
        let row = char_idx / COLS as usize;
        let base_x = (col * CHAR_W as usize) as u32;
        let base_y = (row * CHAR_H as usize) as u32;

        // Center the 8x12 glyph in the 16x16 cell
        let offset_x = 4u32;
        let offset_y = 2u32;

        // First pass: glow effect (larger blur)
        for gy in 0..12 {
            let byte = glyph[gy];
            for gx in 0..8 {
                if (byte >> (7 - gx)) & 1 != 0 {
                    let px = base_x + offset_x + gx as u32;
                    let py = base_y + offset_y + gy as u32;

                    // Draw glow around pixel (2-pixel radius)
                    for dy in -2i32..=2 {
                        for dx in -2i32..=2 {
                            let npx = (px as i32 + dx) as u32;
                            let npy = (py as i32 + dy) as u32;
                            if npx < TEX_W && npy < TEX_H {
                                let dist = ((dx * dx + dy * dy) as f32).sqrt();
                                if dist > 0.5 && dist <= 2.5 {
                                    let existing = tex.get_pixel(npx, npy);
                                    // Blend glow color
                                    let blend = if dist < 1.5 { glow_color } else { glow2_color };
                                    let alpha = blend[3] as u32;
                                    let r = ((existing[0] as u32 * (255 - alpha) + blend[0] as u32 * alpha) / 255) as u8;
                                    let g = ((existing[1] as u32 * (255 - alpha) + blend[1] as u32 * alpha) / 255) as u8;
                                    let b = ((existing[2] as u32 * (255 - alpha) + blend[2] as u32 * alpha) / 255) as u8;
                                    let a = existing[3].max(blend[3]);
                                    tex.set_pixel(npx, npy, [r, g, b, a]);
                                }
                            }
                        }
                    }
                }
            }
        }

        // Second pass: core pixels
        for gy in 0..12 {
            let byte = glyph[gy];
            for gx in 0..8 {
                if (byte >> (7 - gx)) & 1 != 0 {
                    let px = base_x + offset_x + gx as u32;
                    let py = base_y + offset_y + gy as u32;
                    tex.set_pixel(px, py, core_color);
                }
            }
        }
    }

    let path = output_dir.join("neon_font.png");
    write_png(&tex, &path).expect("Failed to write font texture");
    println!("    -> {} ({}x{}, 95 chars)", path.display(), TEX_W, TEX_H);
}

/// Generate road textures with centerline and edge markings
fn generate_road_with_markings(output_dir: &Path) {
    // Starting grid texture
    let mut grid_tex = TextureBuffer::new(256, 256);

    // Fill with dark asphalt
    for y in 0..256 {
        for x in 0..256 {
            grid_tex.set_pixel(x, y, [35, 35, 40, 255]);
        }
    }

    // Add starting grid boxes (2 rows of 4 boxes)
    let box_w = 50u32;
    let box_h = 100u32;
    let colors = [[255, 255, 255, 255], [30, 30, 35, 255]]; // White and dark alternating

    for row in 0..2 {
        for col in 0..4 {
            let x_start = col * box_w + 28;
            let y_start = row * box_h + 28;
            let color = colors[((row + col) % 2) as usize];

            for y in y_start..(y_start + box_h).min(256) {
                for x in x_start..(x_start + box_w).min(256) {
                    grid_tex.set_pixel(x, y, color);
                }
            }
        }
    }

    // Add "START" text area (white bar)
    for y in 0..20 {
        for x in 0..256 {
            grid_tex.set_pixel(x, y, [255, 255, 255, 255]);
        }
    }

    let path = output_dir.join("track_start_grid.png");
    write_png(&grid_tex, &path).expect("Failed to write start grid texture");
    println!("    -> {} (starting grid)", path.display());

    // Finish line texture
    let mut finish_tex = TextureBuffer::new(128, 128);

    // Fill with asphalt
    for y in 0..128 {
        for x in 0..128 {
            finish_tex.set_pixel(x, y, [35, 35, 40, 255]);
        }
    }

    // Checkerboard finish pattern
    let check_size = 16u32;
    for row in 0..8 {
        for col in 0..8 {
            let color = if (row + col) % 2 == 0 {
                [255, 255, 255, 255]
            } else {
                [20, 20, 25, 255]
            };
            let x_start = col * check_size;
            let y_start = row * check_size;
            for y in y_start..(y_start + check_size) {
                for x in x_start..(x_start + check_size) {
                    finish_tex.set_pixel(x, y, color);
                }
            }
        }
    }

    let path = output_dir.join("track_finish_line.png");
    write_png(&finish_tex, &path).expect("Failed to write finish line texture");
    println!("    -> {} (finish line)", path.display());
}

/// Generate additional building facade textures
fn generate_building_varieties(output_dir: &Path) {
    // Building 2: Taller with vertical neon strips
    let mut bldg2 = TextureBuffer::new(128, 256);
    for y in 0..256 {
        for x in 0..128 {
            // Dark base with vertical gradient
            let shade = (255 - y / 2) as u8;
            let r = 25 + shade / 20;
            let g = 30 + shade / 18;
            let b = 45 + shade / 15;
            bldg2.set_pixel(x, y as u32, [r, g, b, 255]);
        }
    }
    // Add vertical neon strips
    for y in 20..240 {
        bldg2.set_pixel(10, y, [0, 255, 255, 255]);
        bldg2.set_pixel(11, y, [0, 200, 255, 200]);
        bldg2.set_pixel(117, y, [255, 0, 255, 255]);
        bldg2.set_pixel(118, y, [200, 0, 200, 200]);
    }
    // Add window rows
    for row in 0..10 {
        let y_base = 30 + row * 22;
        for col in 0..4 {
            let x_base = 25 + col * 22;
            for dy in 0..12 {
                for dx in 0..10 {
                    let lit = (row + col) % 3 != 0;
                    let color = if lit { [255, 240, 180, 255] } else { [40, 45, 60, 255] };
                    bldg2.set_pixel(x_base + dx, y_base + dy, color);
                }
            }
        }
    }
    let path = output_dir.join("prop_building_2.png");
    write_png(&bldg2, &path).expect("Failed to write building 2");
    println!("    -> {} (building variant 2)", path.display());

    // Building 3: Wide with horizontal bands
    let mut bldg3 = TextureBuffer::new(256, 128);
    for y in 0..128 {
        for x in 0..256 {
            let r = 35;
            let g = 40;
            let b = 55;
            bldg3.set_pixel(x, y as u32, [r, g, b, 255]);
        }
    }
    // Horizontal neon bands
    for x in 0..256 {
        for band_y in [20u32, 60, 100] {
            bldg3.set_pixel(x, band_y, [255, 100, 255, 255]);
            bldg3.set_pixel(x, band_y + 1, [200, 80, 200, 180]);
        }
    }
    // Windows in grid
    for row in 0..4 {
        let y_base = 30 + row * 24;
        for col in 0..10 {
            let x_base = 15 + col * 24;
            for dy in 0..10 {
                for dx in 0..16 {
                    let lit = (row * col) % 2 == 0;
                    let color = if lit { [180, 220, 255, 255] } else { [30, 35, 50, 255] };
                    if y_base + dy < 128 && x_base + dx < 256 {
                        bldg3.set_pixel(x_base + dx, y_base + dy, color);
                    }
                }
            }
        }
    }
    let path = output_dir.join("prop_building_3.png");
    write_png(&bldg3, &path).expect("Failed to write building 3");
    println!("    -> {} (building variant 3)", path.display());
}

/// Generate boost pad texture with chevron arrows
fn generate_chevron_boost_pad(output_dir: &Path) {
    let mut tex = TextureBuffer::new(64, 64);

    // Fill with dark base
    for y in 0..64 {
        for x in 0..64 {
            tex.set_pixel(x, y, [20, 60, 80, 255]);
        }
    }

    // Draw chevron arrows (3 of them)
    let chevron_color = [0, 255, 255, 255]; // Cyan
    let glow_color = [0, 180, 200, 180];

    for chevron_idx in 0..3 {
        let base_y = 10 + chevron_idx * 18;

        // Draw chevron pointing up (>) shape rotated 90 degrees
        for i in 0..12 {
            let y_offset = base_y + i;
            let x_left = 32 - i - 4;
            let x_right = 32 + i + 3;

            if y_offset < 64 {
                // Left arm of chevron
                for dx in 0..4 {
                    if x_left + dx < 64 {
                        tex.set_pixel(x_left + dx, y_offset as u32, chevron_color);
                    }
                }
                // Right arm of chevron
                for dx in 0..4 {
                    if x_right + dx < 64 {
                        tex.set_pixel(x_right - dx, y_offset as u32, chevron_color);
                    }
                }
            }
        }
    }

    // Add glow border
    for y in 0..64 {
        tex.set_pixel(0, y, glow_color);
        tex.set_pixel(1, y, glow_color);
        tex.set_pixel(62, y, glow_color);
        tex.set_pixel(63, y, glow_color);
    }
    for x in 0..64 {
        tex.set_pixel(x, 0, glow_color);
        tex.set_pixel(x, 1, glow_color);
        tex.set_pixel(x, 62, glow_color);
        tex.set_pixel(x, 63, glow_color);
    }

    let path = output_dir.join("prop_boost_pad_chevron.png");
    write_png(&tex, &path).expect("Failed to write chevron boost pad");
    println!("    -> {} (chevron boost pad)", path.display());

    // Also generate emissive version
    let mut emissive = TextureBuffer::new(64, 64);
    for y in 0..64 {
        for x in 0..64 {
            let pixel = tex.get_pixel(x, y);
            if pixel[0] == 0 && pixel[1] == 255 && pixel[2] == 255 {
                emissive.set_pixel(x, y, [0, 255, 255, 255]);
            } else if pixel[1] > 100 {
                emissive.set_pixel(x, y, [0, pixel[1], pixel[2], 180]);
            } else {
                emissive.set_pixel(x, y, [0, 0, 0, 0]);
            }
        }
    }
    let path = output_dir.join("prop_boost_pad_chevron_emissive.png");
    write_png(&emissive, &path).expect("Failed to write chevron boost pad emissive");
    println!("    -> {} (chevron boost pad emissive)", path.display());
}
