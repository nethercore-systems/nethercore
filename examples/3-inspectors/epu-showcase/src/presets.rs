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
    set_17_20::PRESET_NOIR_DETECTIVE,
    set_17_20::PRESET_STEAMPUNK_AIRSHIP,
    set_21_24::PRESET_STORMY_SHORES,
    set_21_24::PRESET_POLAR_AURORA,
    set_21_24::PRESET_SACRED_GEOMETRY,
    set_21_24::PRESET_RITUAL_CHAMBER,
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
    "Noir Detective",
    "Steampunk Airship",
    "Stormy Shores",
    "Polar Aurora",
    "Sacred Geometry",
    "Ritual Chamber",
];
