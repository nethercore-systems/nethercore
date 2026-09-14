//! EPU Preset Configurations (EPU 128-bit format)
//!
//! This module contains environment presets for the EPU inspector demo.
//!
//! Preset data is split across `src/presets/` to keep individual files small
//! (useful for editor navigation and AI agent token limits).

mod set_01_02;
mod set_03_04;
mod set_05_06;
mod set_07_08;
mod set_09_10;
mod set_11_12;
mod set_13_14;
mod set_15_16;
mod set_17_18;
mod set_19_20;

// Keep this showcase focused: a small set of "hero" presets with strong
// genre/mood variety and broad opcode coverage.
pub const PRESET_COUNT: usize = 20;

pub type Preset = [[u64; 2]; 8];

/// All presets array
pub static PRESETS: [Preset; PRESET_COUNT] = [
    set_01_02::PRESET_NEON_METROPOLIS,
    set_01_02::PRESET_SAKURA_SHRINE,
    set_03_04::PRESET_OCEAN_DEPTHS,
    set_03_04::PRESET_VOID_STATION,
    set_05_06::PRESET_DESERT_MIRAGE,
    set_05_06::PRESET_ENCHANTED_GROVE,
    set_07_08::PRESET_ASTRAL_VOID,
    set_07_08::PRESET_VOLCANIC_CORE,
    set_09_10::PRESET_SKY_RUINS,
    set_09_10::PRESET_COMBAT_LAB,
    set_11_12::PRESET_FROZEN_TUNDRA,
    set_11_12::PRESET_STORM_FRONT,
    set_13_14::PRESET_CRYSTAL_CAVERN,
    set_13_14::PRESET_MOONLIT_GRAVEYARD,
    set_15_16::PRESET_ALIEN_JUNGLE,
    set_15_16::PRESET_GOTHIC_CATHEDRAL,
    set_17_18::PRESET_TOXIC_WASTELAND,
    set_17_18::PRESET_NEON_ARCADE,
    set_19_20::PRESET_WAR_ZONE,
    set_19_20::PRESET_DIGITAL_MATRIX,
];

/// Animation speeds per layer per preset.
/// Each value is the parameter-byte increment per simulation update (0 = static).
/// The byte wraps modulo 256; its sequence repeats after 256/gcd(256, speed)
/// updates for nonzero speed. This does not guarantee a visually seamless loop.
/// `animate_phases` adds each increment to the authored starting phase, using
/// param_c for SCATTER_PHASED and param_d for supported phase-driven variants.
/// Seeds, bounds, material patterns and static variants are preserved. Legacy
/// SCATTER stays static; use SCATTER_PHASED for fixed-point per-point modulation.
/// These are editable guest recipes, not engine-owned animation restrictions.
pub static ANIM_SPEEDS: [[u8; 8]; PRESET_COUNT] = [
    //                                   L0 L1 L2 L3 L4 L5 L6 L7
    [0, 0, 0, 1, 0, 0, 1, 1], // 0: Neon Metropolis
    [0, 0, 0, 0, 1, 0, 0, 0], // 1: Sakura Shrine
    [0, 0, 0, 0, 0, 4, 0, 0], // 2: Ocean Depths
    [0, 0, 0, 0, 0, 0, 0, 1], // 3: Void Station
    [0, 0, 0, 0, 1, 1, 0, 0], // 4: Desert Mirage
    [0, 0, 0, 0, 0, 0, 0, 1], // 5: Enchanted Grove
    [0, 0, 1, 0, 0, 1, 1, 0], // 6: Astral Void
    [0, 0, 0, 0, 0, 0, 0, 0], // 7: Hell Core
    [0, 0, 0, 0, 0, 1, 1, 1], // 8: Sky Ruins
    [0, 0, 4, 4, 4, 0, 2, 4], // 9: Combat Lab
    [0, 0, 3, 5, 0, 1, 1, 0], // 10: Frozen Tundra
    [0, 6, 1, 2, 0, 4, 8, 0], // 11: Storm Front
    [0, 0, 0, 1, 0, 0, 0, 1], // 12: Crystal Cavern
    [0, 0, 0, 0, 0, 0, 0, 1], // 13: Moonlit Graveyard
    [0, 0, 0, 0, 0, 1, 0, 1], // 14: Alien Jungle
    [0, 0, 0, 0, 0, 0, 0, 1], // 15: Gothic Cathedral
    [0, 0, 0, 0, 0, 0, 0, 1], // 16: Toxic Wasteland
    [0, 0, 0, 0, 0, 1, 1, 1], // 17: Neon Arcade
    [0, 0, 0, 0, 0, 0, 0, 1], // 18: War Zone
    [0, 0, 0, 0, 0, 0, 1, 1], // 19: Digital Matrix
];

/// Preset names for display
pub const PRESET_NAMES: [&str; PRESET_COUNT] = [
    "Neon Metropolis",
    "Sakura Shrine",
    "Ocean Depths",
    "Void Station",
    "Desert Mirage",
    "Enchanted Grove",
    "Astral Void",
    "Hell Core",
    "Sky Ruins",
    "Combat Lab",
    "Frozen Tundra",
    "Storm Front",
    "Crystal Cavern",
    "Moonlit Graveyard",
    "Alien Jungle",
    "Gothic Cathedral",
    "Toxic Wasteland",
    "Neon Arcade",
    "War Zone",
    "Digital Matrix",
];
