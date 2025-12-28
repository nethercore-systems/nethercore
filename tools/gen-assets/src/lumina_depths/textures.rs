//! Texture definitions for LUMINA DEPTHS
//!
//! Uses the convention-based texture system. Each asset has:
//! - `{id}.png` - Base texture
//! - Mode 3 uses Blinn-Phong (underwater lighting)
//!
//! Color palette designed for coherent underwater aesthetic:
//! - Sunlit zone: Warm corals, oranges, yellows, bright greens
//! - Twilight zone: Cool blues, purples, silvers
//! - Midnight zone: Deep blacks, bioluminescent accents
//! - Vent zone: Reds, oranges, pale whites
//!
//! Also includes custom bitmap font generation for the underwater UI.

use crate::texture::{AssetCategory, AssetTexture, TextureStyle, generate_textures_by_category};
use proc_gen::texture::{TextureBuffer, write_png};

/// All LUMINA DEPTHS asset textures - single source of truth
pub const TEXTURES: &[AssetTexture] = &[
    // === SUBMERSIBLE ===
    // Deep sea research vessel - weathered metal with barnacle accents
    AssetTexture {
        id: "submersible",
        category: AssetCategory::Hero,
        base_color: [120, 140, 160, 255],  // Cool steel blue
        style: TextureStyle::Barnacles { seed: 11 },
        size: (128, 128),
        secondary_color: Some([80, 100, 120, 255]),  // Darker weathered areas
        emissive: None,
    },

    // === SUNLIT ZONE CREATURES (0-200m) ===
    // Reef fish - vibrant tropical scales
    AssetTexture {
        id: "reef_fish",
        category: AssetCategory::Enemy,
        base_color: [255, 160, 60, 255],   // Bright tropical orange
        style: TextureStyle::Scales { seed: 101 },
        size: (64, 64),
        secondary_color: Some([255, 200, 100, 255]),  // Golden highlights
        emissive: None,
    },
    // Sea turtle - mottled shell pattern
    AssetTexture {
        id: "sea_turtle",
        category: AssetCategory::Enemy,
        base_color: [70, 90, 50, 255],     // Olive green
        style: TextureStyle::Shell { seed: 102 },
        size: (64, 64),
        secondary_color: Some([90, 110, 70, 255]),  // Lighter shell bands
        emissive: None,
    },
    // Manta ray - smooth gradient with subtle pattern
    AssetTexture {
        id: "manta_ray",
        category: AssetCategory::Enemy,
        base_color: [30, 35, 45, 255],     // Dark slate
        style: TextureStyle::Organic { seed: 103 },
        size: (64, 64),
        secondary_color: Some([50, 55, 65, 255]),  // Subtle markings
        emissive: None,
    },
    // Coral crab - encrusted shell
    AssetTexture {
        id: "coral_crab",
        category: AssetCategory::Enemy,
        base_color: [180, 90, 60, 255],    // Rust orange
        style: TextureStyle::Barnacles { seed: 104 },
        size: (64, 64),
        secondary_color: Some([140, 70, 50, 255]),  // Darker shell
        emissive: None,
    },

    // === TWILIGHT ZONE CREATURES (200-1000m) ===
    // Moon jelly - translucent organic membrane
    AssetTexture {
        id: "moon_jelly",
        category: AssetCategory::Enemy,
        base_color: [180, 200, 240, 140],  // Translucent blue-white
        style: TextureStyle::Organic { seed: 201 },
        size: (64, 64),
        secondary_color: Some([140, 160, 200, 100]),  // Vein structure
        emissive: Some([160, 200, 255, 180]),  // Soft glow
    },
    // Lanternfish - silvery scales with light organs
    AssetTexture {
        id: "lanternfish",
        category: AssetCategory::Enemy,
        base_color: [150, 160, 180, 255],  // Silver
        style: TextureStyle::Scales { seed: 202 },
        size: (64, 64),
        secondary_color: Some([180, 190, 210, 255]),  // Bright silver
        emissive: Some([100, 200, 255, 255]),  // Blue photophores
    },
    // Siphonophore - bioluminescent chain colony
    AssetTexture {
        id: "siphonophore",
        category: AssetCategory::Enemy,
        base_color: [200, 160, 220, 100],  // Translucent purple
        style: TextureStyle::Organic { seed: 203 },
        size: (64, 64),
        secondary_color: Some([160, 120, 180, 80]),  // Darker segments
        emissive: Some([220, 180, 255, 200]),  // Purple glow
    },
    // Giant squid - deep coloration with chromatophores
    AssetTexture {
        id: "giant_squid",
        category: AssetCategory::Enemy,
        base_color: [60, 40, 70, 255],     // Deep purple-black
        style: TextureStyle::Organic { seed: 204 },
        size: (64, 64),
        secondary_color: Some([40, 25, 50, 255]),  // Darker patches
        emissive: None,
    },

    // === MIDNIGHT ZONE CREATURES (1000-4000m) ===
    // Anglerfish - dark rough skin
    AssetTexture {
        id: "anglerfish",
        category: AssetCategory::Enemy,
        base_color: [20, 18, 25, 255],     // Near black
        style: TextureStyle::Barnacles { seed: 301 },
        size: (64, 64),
        secondary_color: Some([35, 30, 40, 255]),  // Slightly lighter bumps
        emissive: Some([255, 240, 150, 255]),  // Lure glow
    },
    // Gulper eel - smooth black with bioluminescent spots
    AssetTexture {
        id: "gulper_eel",
        category: AssetCategory::Enemy,
        base_color: [12, 10, 18, 255],     // Deep black
        style: TextureStyle::Bioluminescent { seed: 302 },
        size: (64, 64),
        secondary_color: Some([255, 120, 180, 255]),  // Pink spots
        emissive: Some([255, 100, 150, 255]),
    },
    // Dumbo octopus - soft translucent pink
    AssetTexture {
        id: "dumbo_octopus",
        category: AssetCategory::Enemy,
        base_color: [255, 200, 210, 180],  // Translucent pink
        style: TextureStyle::Organic { seed: 303 },
        size: (64, 64),
        secondary_color: Some([240, 180, 190, 150]),  // Vein patterns
        emissive: None,
    },
    // Vampire squid - deep red webbing with bioluminescence
    AssetTexture {
        id: "vampire_squid",
        category: AssetCategory::Enemy,
        base_color: [80, 25, 35, 255],     // Deep crimson
        style: TextureStyle::Organic { seed: 304 },
        size: (64, 64),
        secondary_color: Some([50, 15, 25, 255]),  // Darker areas
        emissive: Some([100, 180, 255, 150]),  // Blue photophore tips
    },

    // === VENT ZONE CREATURES (hydrothermal) ===
    // Tube worms - bright red plumes
    AssetTexture {
        id: "tube_worms",
        category: AssetCategory::Enemy,
        base_color: [220, 50, 40, 255],    // Vivid red
        style: TextureStyle::Seaweed { seed: 401 },
        size: (64, 64),
        secondary_color: Some([180, 30, 25, 255]),  // Darker ridges
        emissive: None,
    },
    // Vent shrimp - pale with bacterial coating
    AssetTexture {
        id: "vent_shrimp",
        category: AssetCategory::Enemy,
        base_color: [245, 235, 220, 255],  // Pale cream
        style: TextureStyle::Shell { seed: 402 },
        size: (64, 64),
        secondary_color: Some([230, 220, 200, 255]),  // Shell segments
        emissive: None,
    },
    // Ghost fish - nearly transparent pale
    AssetTexture {
        id: "ghost_fish",
        category: AssetCategory::Enemy,
        base_color: [220, 225, 235, 60],   // Very translucent
        style: TextureStyle::Scales { seed: 403 },
        size: (64, 64),
        secondary_color: Some([200, 205, 215, 40]),  // Faint scale pattern
        emissive: None,
    },
    // Vent octopus - pale white-pink
    AssetTexture {
        id: "vent_octopus",
        category: AssetCategory::Enemy,
        base_color: [235, 220, 225, 255],  // Pale pink-white
        style: TextureStyle::Organic { seed: 404 },
        size: (64, 64),
        secondary_color: Some([210, 195, 200, 255]),  // Vein patterns
        emissive: None,
    },

    // === MEGAFAUNA (zone-spanning) ===
    // Blue whale - textured skin with barnacle patches
    AssetTexture {
        id: "blue_whale",
        category: AssetCategory::Enemy,
        base_color: [65, 75, 90, 255],     // Blue-gray
        style: TextureStyle::Barnacles { seed: 501 },
        size: (128, 128),
        secondary_color: Some([85, 95, 110, 255]),  // Lighter mottling
        emissive: None,
    },
    // Sperm whale - scarred gray-brown
    AssetTexture {
        id: "sperm_whale",
        category: AssetCategory::Enemy,
        base_color: [75, 70, 65, 255],     // Warm gray
        style: TextureStyle::Barnacles { seed: 502 },
        size: (128, 128),
        secondary_color: Some([95, 90, 85, 255]),  // Scars and marks
        emissive: None,
    },
    // Giant isopod - armored plates
    AssetTexture {
        id: "giant_isopod",
        category: AssetCategory::Enemy,
        base_color: [170, 160, 145, 255],  // Pale tan
        style: TextureStyle::Shell { seed: 503 },
        size: (64, 64),
        secondary_color: Some([190, 180, 165, 255]),  // Plate segments
        emissive: None,
    },

    // === FLORA ===
    // Brain coral - ridged organic surface
    AssetTexture {
        id: "coral_brain",
        category: AssetCategory::Pickup,
        base_color: [190, 150, 130, 255],  // Pinkish tan
        style: TextureStyle::Coral { seed: 601 },
        size: (64, 64),
        secondary_color: Some([160, 120, 100, 255]),  // Ridge shadows
        emissive: None,
    },
    // Fan coral - delicate purple-pink
    AssetTexture {
        id: "coral_fan",
        category: AssetCategory::Pickup,
        base_color: [190, 110, 160, 255],  // Pink-purple
        style: TextureStyle::Seaweed { seed: 602 },
        size: (64, 64),
        secondary_color: Some([150, 80, 130, 255]),  // Darker veins
        emissive: None,
    },
    // Branch coral - vibrant orange
    AssetTexture {
        id: "coral_branch",
        category: AssetCategory::Pickup,
        base_color: [255, 170, 110, 255],  // Coral orange
        style: TextureStyle::Coral { seed: 603 },
        size: (64, 64),
        secondary_color: Some([230, 140, 80, 255]),  // Texture detail
        emissive: None,
    },
    // Kelp - organic green-brown
    AssetTexture {
        id: "kelp",
        category: AssetCategory::Pickup,
        base_color: [70, 95, 50, 255],     // Kelp green
        style: TextureStyle::Seaweed { seed: 604 },
        size: (64, 64),
        secondary_color: Some([50, 70, 35, 255]),  // Darker edges
        emissive: None,
    },
    // Anemone - radial organic pattern
    AssetTexture {
        id: "anemone",
        category: AssetCategory::Pickup,
        base_color: [255, 140, 170, 255],  // Pink
        style: TextureStyle::Organic { seed: 605 },
        size: (64, 64),
        secondary_color: Some([220, 100, 130, 255]),  // Center darker
        emissive: None,
    },
    // Sea grass - soft vertical stripes
    AssetTexture {
        id: "sea_grass",
        category: AssetCategory::Pickup,
        base_color: [90, 130, 70, 255],    // Bright green
        style: TextureStyle::Seaweed { seed: 606 },
        size: (64, 64),
        secondary_color: Some([70, 110, 50, 255]),  // Darker veins
        emissive: None,
    },

    // === TERRAIN ===
    // Boulder - mossy encrusted rock
    AssetTexture {
        id: "rock_boulder",
        category: AssetCategory::Arena,
        base_color: [85, 90, 80, 255],     // Gray-green rock
        style: TextureStyle::Barnacles { seed: 701 },
        size: (64, 64),
        secondary_color: Some([65, 80, 60, 255]),  // Moss patches
        emissive: None,
    },
    // Rock pillar - volcanic column
    AssetTexture {
        id: "rock_pillar",
        category: AssetCategory::Arena,
        base_color: [55, 50, 48, 255],     // Dark volcanic
        style: TextureStyle::Stone { seed: 702 },
        size: (64, 64),
        secondary_color: None,
        emissive: None,
    },
    // Vent chimney - mineral-encrusted smoker
    AssetTexture {
        id: "vent_chimney",
        category: AssetCategory::Arena,
        base_color: [45, 40, 38, 255],     // Dark volcanic
        style: TextureStyle::Barnacles { seed: 703 },
        size: (64, 64),
        secondary_color: Some([70, 60, 50, 255]),  // Mineral deposits
        emissive: Some([255, 180, 100, 100]),  // Faint heat glow
    },
    // Seafloor - layered sediment
    AssetTexture {
        id: "seafloor_patch",
        category: AssetCategory::Arena,
        base_color: [130, 120, 100, 255],  // Sandy tan
        style: TextureStyle::Sediment { seed: 704 },
        size: (64, 64),
        secondary_color: Some([150, 140, 120, 255]),  // Layer variation
        emissive: None,
    },
    // Bubble cluster - translucent spheres
    AssetTexture {
        id: "bubble_cluster",
        category: AssetCategory::Arena,
        base_color: [200, 220, 255, 80],   // Very translucent
        style: TextureStyle::GradientRadial,
        size: (32, 32),
        secondary_color: Some([255, 255, 255, 40]),  // Bright center
        emissive: None,
    },
];

use std::path::Path;

/// Generate all textures for Lumina Depths
pub fn generate_all(output_dir: &Path) {
    use AssetCategory::*;
    for category in [Hero, Enemy, Pickup, Arena] {
        generate_textures_by_category(TEXTURES, category, output_dir);
    }
}

// =============================================================================
// CUSTOM BITMAP FONT GENERATION
// =============================================================================

/// Font configuration for LUMINA DEPTHS underwater theme
const GLYPH_WIDTH: u32 = 6;  // 5 pixels + 1 spacing
const GLYPH_HEIGHT: u32 = 8; // 7 pixels + 1 spacing
const CHARS_PER_ROW: u32 = 16;
const CHAR_COUNT: u32 = 96;  // ASCII 32-127
const FONT_ROWS: u32 = (CHAR_COUNT + CHARS_PER_ROW - 1) / CHARS_PER_ROW;

/// 5x7 pixel font bitmaps (7 bytes per char, 5 bits per row, LSB = left)
/// Characters 32 (space) through 127 (DEL)
#[rustfmt::skip]
const FONT_DATA: [[u8; 7]; 96] = [
    // 32 ' '  space
    [0b00000, 0b00000, 0b00000, 0b00000, 0b00000, 0b00000, 0b00000],
    // 33 '!'
    [0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b00000, 0b00100],
    // 34 '"'
    [0b01010, 0b01010, 0b00000, 0b00000, 0b00000, 0b00000, 0b00000],
    // 35 '#'
    [0b01010, 0b11111, 0b01010, 0b01010, 0b11111, 0b01010, 0b00000],
    // 36 '$'
    [0b00100, 0b01111, 0b10100, 0b01110, 0b00101, 0b11110, 0b00100],
    // 37 '%'
    [0b11001, 0b11010, 0b00100, 0b00100, 0b01011, 0b10011, 0b00000],
    // 38 '&'
    [0b01100, 0b10010, 0b01100, 0b01101, 0b10010, 0b01101, 0b00000],
    // 39 '\''
    [0b00100, 0b00100, 0b00000, 0b00000, 0b00000, 0b00000, 0b00000],
    // 40 '('
    [0b00010, 0b00100, 0b01000, 0b01000, 0b01000, 0b00100, 0b00010],
    // 41 ')'
    [0b01000, 0b00100, 0b00010, 0b00010, 0b00010, 0b00100, 0b01000],
    // 42 '*'
    [0b00000, 0b00100, 0b10101, 0b01110, 0b10101, 0b00100, 0b00000],
    // 43 '+'
    [0b00000, 0b00100, 0b00100, 0b11111, 0b00100, 0b00100, 0b00000],
    // 44 ','
    [0b00000, 0b00000, 0b00000, 0b00000, 0b00000, 0b00100, 0b01000],
    // 45 '-'
    [0b00000, 0b00000, 0b00000, 0b11111, 0b00000, 0b00000, 0b00000],
    // 46 '.'
    [0b00000, 0b00000, 0b00000, 0b00000, 0b00000, 0b00000, 0b00100],
    // 47 '/'
    [0b00001, 0b00010, 0b00010, 0b00100, 0b01000, 0b01000, 0b10000],
    // 48 '0'
    [0b01110, 0b10001, 0b10011, 0b10101, 0b11001, 0b10001, 0b01110],
    // 49 '1'
    [0b00100, 0b01100, 0b00100, 0b00100, 0b00100, 0b00100, 0b01110],
    // 50 '2'
    [0b01110, 0b10001, 0b00001, 0b00110, 0b01000, 0b10000, 0b11111],
    // 51 '3'
    [0b01110, 0b10001, 0b00001, 0b00110, 0b00001, 0b10001, 0b01110],
    // 52 '4'
    [0b00010, 0b00110, 0b01010, 0b10010, 0b11111, 0b00010, 0b00010],
    // 53 '5'
    [0b11111, 0b10000, 0b11110, 0b00001, 0b00001, 0b10001, 0b01110],
    // 54 '6'
    [0b00110, 0b01000, 0b10000, 0b11110, 0b10001, 0b10001, 0b01110],
    // 55 '7'
    [0b11111, 0b00001, 0b00010, 0b00100, 0b01000, 0b01000, 0b01000],
    // 56 '8'
    [0b01110, 0b10001, 0b10001, 0b01110, 0b10001, 0b10001, 0b01110],
    // 57 '9'
    [0b01110, 0b10001, 0b10001, 0b01111, 0b00001, 0b00010, 0b01100],
    // 58 ':'
    [0b00000, 0b00000, 0b00100, 0b00000, 0b00000, 0b00100, 0b00000],
    // 59 ';'
    [0b00000, 0b00000, 0b00100, 0b00000, 0b00000, 0b00100, 0b01000],
    // 60 '<'
    [0b00010, 0b00100, 0b01000, 0b10000, 0b01000, 0b00100, 0b00010],
    // 61 '='
    [0b00000, 0b00000, 0b11111, 0b00000, 0b11111, 0b00000, 0b00000],
    // 62 '>'
    [0b01000, 0b00100, 0b00010, 0b00001, 0b00010, 0b00100, 0b01000],
    // 63 '?'
    [0b01110, 0b10001, 0b00001, 0b00110, 0b00100, 0b00000, 0b00100],
    // 64 '@'
    [0b01110, 0b10001, 0b10111, 0b10101, 0b10111, 0b10000, 0b01110],
    // 65 'A'
    [0b01110, 0b10001, 0b10001, 0b11111, 0b10001, 0b10001, 0b10001],
    // 66 'B'
    [0b11110, 0b10001, 0b10001, 0b11110, 0b10001, 0b10001, 0b11110],
    // 67 'C'
    [0b01110, 0b10001, 0b10000, 0b10000, 0b10000, 0b10001, 0b01110],
    // 68 'D'
    [0b11110, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b11110],
    // 69 'E'
    [0b11111, 0b10000, 0b10000, 0b11110, 0b10000, 0b10000, 0b11111],
    // 70 'F'
    [0b11111, 0b10000, 0b10000, 0b11110, 0b10000, 0b10000, 0b10000],
    // 71 'G'
    [0b01110, 0b10001, 0b10000, 0b10111, 0b10001, 0b10001, 0b01111],
    // 72 'H'
    [0b10001, 0b10001, 0b10001, 0b11111, 0b10001, 0b10001, 0b10001],
    // 73 'I'
    [0b01110, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b01110],
    // 74 'J'
    [0b00111, 0b00010, 0b00010, 0b00010, 0b00010, 0b10010, 0b01100],
    // 75 'K'
    [0b10001, 0b10010, 0b10100, 0b11000, 0b10100, 0b10010, 0b10001],
    // 76 'L'
    [0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b11111],
    // 77 'M'
    [0b10001, 0b11011, 0b10101, 0b10101, 0b10001, 0b10001, 0b10001],
    // 78 'N'
    [0b10001, 0b11001, 0b10101, 0b10011, 0b10001, 0b10001, 0b10001],
    // 79 'O'
    [0b01110, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01110],
    // 80 'P'
    [0b11110, 0b10001, 0b10001, 0b11110, 0b10000, 0b10000, 0b10000],
    // 81 'Q'
    [0b01110, 0b10001, 0b10001, 0b10001, 0b10101, 0b10010, 0b01101],
    // 82 'R'
    [0b11110, 0b10001, 0b10001, 0b11110, 0b10100, 0b10010, 0b10001],
    // 83 'S'
    [0b01110, 0b10001, 0b10000, 0b01110, 0b00001, 0b10001, 0b01110],
    // 84 'T'
    [0b11111, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100],
    // 85 'U'
    [0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01110],
    // 86 'V'
    [0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01010, 0b00100],
    // 87 'W'
    [0b10001, 0b10001, 0b10001, 0b10101, 0b10101, 0b11011, 0b10001],
    // 88 'X'
    [0b10001, 0b10001, 0b01010, 0b00100, 0b01010, 0b10001, 0b10001],
    // 89 'Y'
    [0b10001, 0b10001, 0b01010, 0b00100, 0b00100, 0b00100, 0b00100],
    // 90 'Z'
    [0b11111, 0b00001, 0b00010, 0b00100, 0b01000, 0b10000, 0b11111],
    // 91 '['
    [0b01110, 0b01000, 0b01000, 0b01000, 0b01000, 0b01000, 0b01110],
    // 92 '\\'
    [0b10000, 0b01000, 0b01000, 0b00100, 0b00010, 0b00010, 0b00001],
    // 93 ']'
    [0b01110, 0b00010, 0b00010, 0b00010, 0b00010, 0b00010, 0b01110],
    // 94 '^'
    [0b00100, 0b01010, 0b10001, 0b00000, 0b00000, 0b00000, 0b00000],
    // 95 '_'
    [0b00000, 0b00000, 0b00000, 0b00000, 0b00000, 0b00000, 0b11111],
    // 96 '`'
    [0b01000, 0b00100, 0b00000, 0b00000, 0b00000, 0b00000, 0b00000],
    // 97 'a'
    [0b00000, 0b00000, 0b01110, 0b00001, 0b01111, 0b10001, 0b01111],
    // 98 'b'
    [0b10000, 0b10000, 0b11110, 0b10001, 0b10001, 0b10001, 0b11110],
    // 99 'c'
    [0b00000, 0b00000, 0b01110, 0b10000, 0b10000, 0b10000, 0b01110],
    // 100 'd'
    [0b00001, 0b00001, 0b01111, 0b10001, 0b10001, 0b10001, 0b01111],
    // 101 'e'
    [0b00000, 0b00000, 0b01110, 0b10001, 0b11111, 0b10000, 0b01110],
    // 102 'f'
    [0b00110, 0b01001, 0b01000, 0b11110, 0b01000, 0b01000, 0b01000],
    // 103 'g'
    [0b00000, 0b00000, 0b01111, 0b10001, 0b01111, 0b00001, 0b01110],
    // 104 'h'
    [0b10000, 0b10000, 0b11110, 0b10001, 0b10001, 0b10001, 0b10001],
    // 105 'i'
    [0b00100, 0b00000, 0b01100, 0b00100, 0b00100, 0b00100, 0b01110],
    // 106 'j'
    [0b00010, 0b00000, 0b00110, 0b00010, 0b00010, 0b10010, 0b01100],
    // 107 'k'
    [0b10000, 0b10000, 0b10010, 0b10100, 0b11000, 0b10100, 0b10010],
    // 108 'l'
    [0b01100, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b01110],
    // 109 'm'
    [0b00000, 0b00000, 0b11010, 0b10101, 0b10101, 0b10101, 0b10001],
    // 110 'n'
    [0b00000, 0b00000, 0b11110, 0b10001, 0b10001, 0b10001, 0b10001],
    // 111 'o'
    [0b00000, 0b00000, 0b01110, 0b10001, 0b10001, 0b10001, 0b01110],
    // 112 'p'
    [0b00000, 0b00000, 0b11110, 0b10001, 0b11110, 0b10000, 0b10000],
    // 113 'q'
    [0b00000, 0b00000, 0b01111, 0b10001, 0b01111, 0b00001, 0b00001],
    // 114 'r'
    [0b00000, 0b00000, 0b10110, 0b11001, 0b10000, 0b10000, 0b10000],
    // 115 's'
    [0b00000, 0b00000, 0b01111, 0b10000, 0b01110, 0b00001, 0b11110],
    // 116 't'
    [0b00100, 0b00100, 0b01110, 0b00100, 0b00100, 0b00100, 0b00011],
    // 117 'u'
    [0b00000, 0b00000, 0b10001, 0b10001, 0b10001, 0b10001, 0b01111],
    // 118 'v'
    [0b00000, 0b00000, 0b10001, 0b10001, 0b10001, 0b01010, 0b00100],
    // 119 'w'
    [0b00000, 0b00000, 0b10001, 0b10101, 0b10101, 0b10101, 0b01010],
    // 120 'x'
    [0b00000, 0b00000, 0b10001, 0b01010, 0b00100, 0b01010, 0b10001],
    // 121 'y'
    [0b00000, 0b00000, 0b10001, 0b10001, 0b01111, 0b00001, 0b01110],
    // 122 'z'
    [0b00000, 0b00000, 0b11111, 0b00010, 0b00100, 0b01000, 0b11111],
    // 123 '{'
    [0b00011, 0b00100, 0b00100, 0b01000, 0b00100, 0b00100, 0b00011],
    // 124 '|'
    [0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100],
    // 125 '}'
    [0b11000, 0b00100, 0b00100, 0b00010, 0b00100, 0b00100, 0b11000],
    // 126 '~'
    [0b00000, 0b00000, 0b01000, 0b10101, 0b00010, 0b00000, 0b00000],
    // 127 DEL (empty)
    [0b11111, 0b11111, 0b11111, 0b11111, 0b11111, 0b11111, 0b11111],
];

/// Generate the LUMINA DEPTHS custom underwater font
pub fn generate_font(output_dir: &Path) {
    let tex_width = CHARS_PER_ROW * GLYPH_WIDTH;
    let tex_height = FONT_ROWS * GLYPH_HEIGHT;

    let mut buffer = TextureBuffer::new(tex_width, tex_height);

    // Deep ocean bioluminescent color scheme
    let glow_color: [u8; 4] = [140, 220, 255, 255];      // Bright cyan-white
    let edge_color: [u8; 4] = [60, 140, 200, 200];       // Medium blue edge
    let outer_glow: [u8; 4] = [30, 80, 120, 100];        // Faint outer glow

    for (char_idx, glyph_data) in FONT_DATA.iter().enumerate() {
        let col = (char_idx as u32) % CHARS_PER_ROW;
        let row = (char_idx as u32) / CHARS_PER_ROW;
        let base_x = col * GLYPH_WIDTH;
        let base_y = row * GLYPH_HEIGHT;

        // First pass: outer glow (2-pixel radius)
        for (py, &row_bits) in glyph_data.iter().enumerate() {
            for px in 0..5 {
                let bit = (row_bits >> (4 - px)) & 1;
                if bit == 1 {
                    // Add faint outer glow
                    for dy in -2i32..=2 {
                        for dx in -2i32..=2 {
                            if dx.abs() + dy.abs() > 2 { continue; }
                            let gx = base_x as i32 + px as i32 + dx;
                            let gy = base_y as i32 + py as i32 + dy;
                            if gx >= 0 && gx < tex_width as i32 && gy >= 0 && gy < tex_height as i32 {
                                let existing = buffer.get_pixel(gx as u32, gy as u32);
                                if existing[3] < outer_glow[3] {
                                    buffer.set_pixel(gx as u32, gy as u32, outer_glow);
                                }
                            }
                        }
                    }
                }
            }
        }

        // Second pass: edge glow (1-pixel radius)
        for (py, &row_bits) in glyph_data.iter().enumerate() {
            for px in 0..5 {
                let bit = (row_bits >> (4 - px)) & 1;
                if bit == 0 {
                    // Check if adjacent to a lit pixel
                    let mut adjacent = false;
                    for dy in -1i32..=1 {
                        for dx in -1i32..=1 {
                            if dx == 0 && dy == 0 { continue; }
                            let npx = (px as i32) + dx;
                            let npy = (py as i32) + dy;
                            if npx >= 0 && npx < 5 && npy >= 0 && npy < 7 {
                                let nrow = glyph_data[npy as usize];
                                let nbit = (nrow >> (4 - npx)) & 1;
                                if nbit == 1 {
                                    adjacent = true;
                                    break;
                                }
                            }
                        }
                        if adjacent { break; }
                    }
                    if adjacent {
                        let x = base_x + px;
                        let y = base_y + py as u32;
                        buffer.set_pixel(x, y, edge_color);
                    }
                }
            }
        }

        // Third pass: main glyph pixels
        for (py, &row_bits) in glyph_data.iter().enumerate() {
            for px in 0..5 {
                let bit = (row_bits >> (4 - px)) & 1;
                if bit == 1 {
                    let x = base_x + px;
                    let y = base_y + py as u32;
                    buffer.set_pixel(x, y, glow_color);
                }
            }
        }
    }

    let path = output_dir.join("lumina_font.png");
    write_png(&buffer, &path).expect("Failed to write font texture");
    println!("    -> lumina_font.png ({}x{}, {} chars)", tex_width, tex_height, CHAR_COUNT);
}
