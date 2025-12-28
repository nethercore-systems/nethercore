//! Texture definitions for LUMINA DEPTHS
//!
//! Uses the convention-based texture system. Each asset has:
//! - `{id}.png` - Base texture
//! - Mode 3 uses Blinn-Phong (underwater lighting)
//!
//! Also includes custom bitmap font generation for the underwater UI.

use crate::texture::{AssetTexture, TextureStyle};
use proc_gen::texture::{TextureBuffer, write_png};

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
    // Reef fish - tropical orange with stripes
    AssetTexture {
        id: "reef_fish",
        base_color: [255, 140, 50, 255],
        style: TextureStyle::GradientV,
        size: (64, 64),
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
    // Coral crab - orange-red shell with texture
    AssetTexture {
        id: "coral_crab",
        base_color: [180, 80, 50, 255],
        style: TextureStyle::Stone { seed: 55 },
        size: (64, 64),
        secondary_color: None,
        emissive: None,
    },
    // Moon jelly - translucent blue bioluminescent
    AssetTexture {
        id: "moon_jelly",
        base_color: [180, 200, 255, 180],
        style: TextureStyle::GradientRadial,
        size: (64, 64),
        secondary_color: Some([120, 140, 200, 100]),
        emissive: None,
    },
    // Lanternfish - silvery with light organs
    AssetTexture {
        id: "lanternfish",
        base_color: [120, 140, 160, 255],
        style: TextureStyle::Metal { seed: 21 },
        size: (64, 64),
        secondary_color: None,
        emissive: Some([100, 200, 255, 255]),
    },
    // Siphonophore - translucent chain colony
    AssetTexture {
        id: "siphonophore",
        base_color: [200, 180, 255, 140],
        style: TextureStyle::GradientV,
        size: (64, 64),
        secondary_color: Some([150, 120, 200, 100]),
        emissive: Some([180, 150, 255, 200]),
    },
    // Giant squid - dark body with red tones
    AssetTexture {
        id: "giant_squid",
        base_color: [40, 30, 50, 255],
        style: TextureStyle::GradientV,
        size: (64, 64),
        secondary_color: Some([20, 15, 30, 255]),
        emissive: None,
    },
    // Anglerfish - deep dark body
    AssetTexture {
        id: "anglerfish",
        base_color: [15, 15, 20, 255],
        style: TextureStyle::Stone { seed: 66 },
        size: (64, 64),
        secondary_color: None,
        emissive: None,
    },
    // Gulper eel - black with bioluminescent spots
    AssetTexture {
        id: "gulper_eel",
        base_color: [10, 10, 15, 255],
        style: TextureStyle::Solid,
        size: (64, 64),
        secondary_color: None,
        emissive: Some([255, 100, 150, 255]),
    },
    // Dumbo octopus - pink translucent
    AssetTexture {
        id: "dumbo_octopus",
        base_color: [255, 180, 200, 200],
        style: TextureStyle::GradientRadial,
        size: (64, 64),
        secondary_color: Some([220, 150, 170, 180]),
        emissive: None,
    },
    // Vampire squid - deep red with webbing
    AssetTexture {
        id: "vampire_squid",
        base_color: [80, 20, 30, 255],
        style: TextureStyle::GradientRadial,
        size: (64, 64),
        secondary_color: Some([40, 10, 20, 255]),
        emissive: Some([100, 150, 255, 150]),
    },
    // Tube worms - red plumes
    AssetTexture {
        id: "tube_worms",
        base_color: [200, 40, 40, 255],
        style: TextureStyle::GradientV,
        size: (64, 64),
        secondary_color: Some([150, 20, 20, 255]),
        emissive: None,
    },
    // Vent shrimp - pale with red accents
    AssetTexture {
        id: "vent_shrimp",
        base_color: [240, 220, 200, 255],
        style: TextureStyle::Solid,
        size: (64, 64),
        secondary_color: None,
        emissive: None,
    },
    // Ghost fish - nearly transparent pale
    AssetTexture {
        id: "ghost_fish",
        base_color: [200, 210, 220, 80],
        style: TextureStyle::GradientV,
        size: (64, 64),
        secondary_color: Some([180, 190, 200, 60]),
        emissive: None,
    },
    // Vent octopus - pale white-pink
    AssetTexture {
        id: "vent_octopus",
        base_color: [220, 200, 210, 255],
        style: TextureStyle::GradientRadial,
        size: (64, 64),
        secondary_color: Some([180, 160, 170, 255]),
        emissive: None,
    },
    // Blue whale - blue-gray skin with barnacles
    AssetTexture {
        id: "blue_whale",
        base_color: [60, 70, 85, 255],
        style: TextureStyle::Stone { seed: 77 },
        size: (128, 128),
        secondary_color: None,
        emissive: None,
    },
    // Sperm whale - gray-brown scarred skin
    AssetTexture {
        id: "sperm_whale",
        base_color: [70, 65, 60, 255],
        style: TextureStyle::Stone { seed: 88 },
        size: (128, 128),
        secondary_color: None,
        emissive: None,
    },
    // Giant isopod - pale armored plates
    AssetTexture {
        id: "giant_isopod",
        base_color: [180, 170, 150, 255],
        style: TextureStyle::Stone { seed: 99 },
        size: (64, 64),
        secondary_color: None,
        emissive: None,
    },

    // === FLORA ===
    // Brain coral - pinkish tan ridged surface
    AssetTexture {
        id: "coral_brain",
        base_color: [180, 140, 120, 255],
        style: TextureStyle::Stone { seed: 33 },
        size: (64, 64),
        secondary_color: None,
        emissive: None,
    },
    // Fan coral - purple-pink delicate
    AssetTexture {
        id: "coral_fan",
        base_color: [180, 100, 150, 255],
        style: TextureStyle::GradientV,
        size: (64, 64),
        secondary_color: Some([140, 70, 120, 255]),
        emissive: None,
    },
    // Branch coral - orange branching
    AssetTexture {
        id: "coral_branch",
        base_color: [255, 160, 100, 255],
        style: TextureStyle::Solid,
        size: (64, 64),
        secondary_color: None,
        emissive: None,
    },
    // Kelp - green-brown long blades
    AssetTexture {
        id: "kelp",
        base_color: [60, 80, 40, 255],
        style: TextureStyle::GradientV,
        size: (64, 64),
        secondary_color: Some([40, 60, 30, 255]),
        emissive: None,
    },
    // Anemone - pink radial tentacles
    AssetTexture {
        id: "anemone",
        base_color: [255, 150, 180, 255],
        style: TextureStyle::GradientRadial,
        size: (64, 64),
        secondary_color: Some([200, 100, 130, 255]),
        emissive: None,
    },
    // Sea grass - soft green blades
    AssetTexture {
        id: "sea_grass",
        base_color: [80, 120, 60, 255],
        style: TextureStyle::GradientV,
        size: (64, 64),
        secondary_color: Some([60, 100, 40, 255]),
        emissive: None,
    },

    // === TERRAIN ===
    // Boulder - mossy gray rock
    AssetTexture {
        id: "rock_boulder",
        base_color: [90, 95, 85, 255],
        style: TextureStyle::Stone { seed: 111 },
        size: (64, 64),
        secondary_color: None,
        emissive: None,
    },
    // Rock pillar - dark volcanic column
    AssetTexture {
        id: "rock_pillar",
        base_color: [60, 55, 50, 255],
        style: TextureStyle::Stone { seed: 122 },
        size: (64, 64),
        secondary_color: None,
        emissive: None,
    },
    // Vent chimney - dark volcanic with mineral deposits
    AssetTexture {
        id: "vent_chimney",
        base_color: [50, 45, 40, 255],
        style: TextureStyle::GradientV,
        size: (64, 64),
        secondary_color: Some([30, 28, 25, 255]),
        emissive: None,
    },
    // Seafloor - sandy sediment
    AssetTexture {
        id: "seafloor_patch",
        base_color: [140, 130, 110, 255],
        style: TextureStyle::Stone { seed: 133 },
        size: (64, 64),
        secondary_color: None,
        emissive: None,
    },
    // Bubble cluster - translucent spheres
    AssetTexture {
        id: "bubble_cluster",
        base_color: [200, 220, 255, 100],
        style: TextureStyle::GradientRadial,
        size: (32, 32),
        secondary_color: Some([255, 255, 255, 60]),
        emissive: None,
    },
];

use std::path::Path;
use crate::texture::generate_all_textures;

pub fn generate_creature_textures(output_dir: &Path) {
    let creatures: Vec<_> = TEXTURES.iter()
        .filter(|t| [
            "reef_fish", "sea_turtle", "manta_ray", "coral_crab",
            "moon_jelly", "lanternfish", "siphonophore", "giant_squid",
            "anglerfish", "gulper_eel", "dumbo_octopus", "vampire_squid",
            "tube_worms", "vent_shrimp", "ghost_fish", "vent_octopus",
            "blue_whale", "sperm_whale", "giant_isopod"
        ].contains(&t.id))
        .cloned()
        .collect();
    generate_all_textures(&creatures, output_dir);
}

pub fn generate_flora_textures(output_dir: &Path) {
    let flora: Vec<_> = TEXTURES.iter()
        .filter(|t| [
            "coral_brain", "coral_fan", "coral_branch",
            "kelp", "anemone", "sea_grass"
        ].contains(&t.id))
        .cloned()
        .collect();
    generate_all_textures(&flora, output_dir);
}

pub fn generate_terrain_textures(output_dir: &Path) {
    let terrain: Vec<_> = TEXTURES.iter()
        .filter(|t| [
            "rock_boulder", "rock_pillar", "vent_chimney",
            "seafloor_patch", "bubble_cluster"
        ].contains(&t.id))
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

    // Deep ocean color scheme
    let glow_color: [u8; 4] = [140, 200, 255, 255];      // Bioluminescent blue-white
    let edge_color: [u8; 4] = [60, 120, 180, 200];        // Darker blue edge

    for (char_idx, glyph_data) in FONT_DATA.iter().enumerate() {
        let col = (char_idx as u32) % CHARS_PER_ROW;
        let row = (char_idx as u32) / CHARS_PER_ROW;
        let base_x = col * GLYPH_WIDTH;
        let base_y = row * GLYPH_HEIGHT;

        // Draw each row of the glyph
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

        // Add subtle glow effect by checking edges
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
    }

    let path = output_dir.join("lumina_font.png");
    write_png(&buffer, &path).expect("Failed to write font texture");
    println!("    -> lumina_font.png ({}x{}, {} chars)", tex_width, tex_height, CHAR_COUNT);
}
