//! EPU Preset Configurations (EPU 128-bit format)
//!
//! This module contains environment presets for the EPU inspector demo.
//!
//! Preset data is split across `src/presets/` to keep individual files small
//! (useful for editor navigation and AI agent token limits).

mod set_01_04;
mod set_05_08;
mod set_09_12;
mod set_13_16;
mod set_17_20;
mod set_21_24;

pub const PRESET_COUNT: usize = 24;

pub type Preset = [[u64; 2]; 8];

/// All presets array
pub static PRESETS: [Preset; PRESET_COUNT] = [
    set_01_04::PRESET_NEON_METROPOLIS,
    set_01_04::PRESET_CRIMSON_HELLSCAPE,
    set_01_04::PRESET_FROZEN_TUNDRA,
    set_01_04::PRESET_ALIEN_JUNGLE,
    set_05_08::PRESET_GOTHIC_CATHEDRAL,
    set_05_08::PRESET_OCEAN_DEPTHS,
    set_05_08::PRESET_VOID_STATION,
    set_05_08::PRESET_DESERT_MIRAGE,
    set_09_12::PRESET_NEON_ARCADE,
    set_09_12::PRESET_STORM_FRONT,
    set_09_12::PRESET_CRYSTAL_CAVERN,
    set_09_12::PRESET_WAR_ZONE,
    set_13_16::PRESET_ENCHANTED_GROVE,
    set_13_16::PRESET_ASTRAL_VOID,
    set_13_16::PRESET_TOXIC_WASTELAND,
    set_13_16::PRESET_MOONLIT_GRAVEYARD,
    set_17_20::PRESET_VOLCANIC_CORE,
    set_17_20::PRESET_DIGITAL_MATRIX,
    set_17_20::PRESET_CYBER_SHRINE,
    set_17_20::PRESET_STEAMPUNK_AIRSHIP,
    set_21_24::PRESET_STORMY_SHORES,
    set_21_24::PRESET_POLAR_AURORA,
    set_21_24::PRESET_SACRED_GEOMETRY,
    set_21_24::PRESET_RITUAL_CHAMBER,
];

/// Animation speeds per layer per preset.
/// Each value is the phase increment per frame (0 = static, 1 = slow, 2 = medium, 4 = fast).
/// Phase wraps at 256 (one full cycle = 256/speed frames).
/// Only meaningful for opcodes that read param_d as phase:
/// FLOW, LOBE, GRID, PLANE, PORTAL, BAND, DECAL.
/// For SCATTER, patching param_d changes the seed — produces shimmer/respawn, not smooth motion.
pub static ANIM_SPEEDS: [[u8; 8]; PRESET_COUNT] = [
    //                                   L0 L1 L2 L3 L4 L5 L6 L7
    [0, 0, 0, 2, 0, 0, 4, 0], //  0: Neon Metropolis  (L3=grid scroll, L6=rain flow)
    [0, 0, 0, 2, 0, 0, 4, 0], //  1: Crimson Hellscape (L3=lava flow, L6=portal pulse)
    [0, 0, 2, 0, 3, 0, 0, 0], //  2: Frozen Tundra    (L2=plane, L4=snow drift flow)
    [0, 0, 0, 0, 0, 2, 0, 0], //  3: Alien Jungle     (L5=biolum flow)
    [0, 0, 0, 0, 2, 0, 0, 0], //  4: Gothic Cathedral (L4=divine lobe pulse)
    [0, 0, 0, 3, 0, 0, 0, 2], //  5: Ocean Depths     (L3=caustic flow, L7=bio glow)
    [0, 0, 0, 2, 0, 0, 0, 4], //  6: Void Station     (L3=grid, L7=decal blink)
    [0, 0, 0, 0, 2, 2, 0, 0], //  7: Desert Mirage    (L4=shimmer flow, L5=band pulse)
    [0, 2, 3, 0, 0, 1, 2, 0], //  8: Neon Arcade      (L1=wall grid, L2=floor grid, L5=lobe, L6=flow)
    [0, 0, 3, 0, 4, 0, 0, 0], //  9: Storm Front      (L2=cloud flow, L4=rain fall)
    [0, 0, 0, 0, 0, 2, 2, 0], // 10: Crystal Cavern   (L5=lobe pulse, L6=portal spin)
    [0, 0, 0, 0, 0, 3, 0, 4], // 11: War Zone         (L5=smoke flow, L7=fire decal)
    [0, 0, 0, 0, 0, 1, 2, 0], // 12: Enchanted Grove  (L5=sunbeam pulse, L6=leaf flow)
    [0, 0, 3, 0, 0, 0, 4, 0], // 13: Astral Void      (L2=cosmic flow, L6=vortex spin)
    [0, 0, 0, 0, 2, 0, 2, 0], // 14: Toxic Wasteland  (L4=toxic puddle flow, L6=smoke rise)
    [0; 8],                    // 15: Moonlit Graveyard (stillness is the horror)
    [0, 0, 0, 0, 0, 3, 0, 0], // 16: Volcanic Core    (L5=lava flow)
    [0, 0, 4, 3, 0, 0, 0, 0], // 17: Digital Matrix   (L2=code flow, L3=secondary flow)
    [0, 0, 0, 2, 1, 0, 0, 0], // 18: Ancient Library  (L3=candle flames, L4=glow flicker)
    [0; 8],                    // 19: Steampunk Airship (static tableau)
    [0, 0, 0, 3, 0, 6, 0, 3], // 20: Stormy Shores    (L3=sea foam, L5=lightning flicker, L7=lighthouse)
    [0, 0, 3, 2, 0, 0, 2, 0], // 21: Polar Aurora     (L2=aurora band, L3=curtains, L6=shimmer flow)
    [0, 0, 0, 0, 0, 0, 2, 0], // 22: Sacred Geometry  (L6=divine light pulse)
    [0, 0, 0, 5, 2, 0, 0, 0], // 23: Ritual Chamber   (L3=pentagram, L4=portal spin)
];

/// Preset names for display
pub const PRESET_NAMES: [&str; PRESET_COUNT] = [
    "Neon Metropolis",
    "Crimson Hellscape",
    "Frozen Tundra",
    "Alien Jungle",
    "Gothic Cathedral",
    "Ocean Depths",
    "Void Station",
    "Desert Mirage",
    "Neon Arcade",
    "Storm Front",
    "Crystal Cavern",
    "War Zone",
    "Enchanted Grove",
    "Astral Void",
    "Toxic Wasteland",
    "Moonlit Graveyard",
    "Volcanic Core",
    "Digital Matrix",
    "Ancient Library",
    "Steampunk Airship",
    "Stormy Shores",
    "Polar Aurora",
    "Sacred Geometry",
    "Ritual Chamber",
];
