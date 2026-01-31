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

// Keep this showcase focused: a small set of "hero" presets with strong
// genre/mood variety and broad opcode coverage.
pub const PRESET_COUNT: usize = 10;

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
];

/// Animation speeds per layer per preset.
/// Each value is the phase increment per frame (0 = static, 1 = slow, 2 = medium, 4 = fast).
/// Phase wraps at 256 (one full cycle = 256/speed frames).
/// Only meaningful for opcodes that read param_d as phase:
/// FLOW, LOBE, GRID, PLANE, PORTAL, BAND, DECAL.
/// For SCATTER, patching param_d changes the seed — produces shimmer/respawn, not smooth motion.
pub static ANIM_SPEEDS: [[u8; 8]; PRESET_COUNT] = [
    //                                   L0 L1 L2 L3 L4 L5 L6 L7
    [0, 0, 0, 1, 2, 0, 1, 1], // 0: Neon Metropolis (rain streaks slow)
    [0, 0, 1, 1, 1, 0, 0, 0], // 1: Sakura Shrine (gentle)
    [0, 0, 1, 1, 0, 1, 0, 0], // 2: Ocean Depths (very slow - underwater)
    [0, 1, 0, 0, 0, 0, 0, 1], // 3: Void Station
    [0, 0, 1, 1, 1, 1, 0, 0], // 4: Desert Mirage (slow heat shimmer)
    [0, 0, 1, 1, 1, 0, 0, 1], // 5: Enchanted Grove (slow light shafts)
    [0, 0, 1, 0, 0, 1, 1, 0], // 6: Astral Void (very subtle)
    [0, 0, 0, 0, 0, 0, 0, 0], // 7: Hell Core (STATIC - no seizures!)
    [0, 0, 0, 1, 1, 1, 1, 1], // 8: Sky Ruins
    [0, 0, 1, 1, 1, 1, 1, 1], // 9: Combat Lab
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
];
