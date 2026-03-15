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
/// Each value is the phase increment per frame (0 = static, 1 = slow, 2 = medium, 4 = fast).
/// Phase wraps at 256 (one full cycle = 256/speed frames).
/// Only meaningful for opcodes that read param_d as phase:
/// FLOW, LOBE, GRID, PLANE, PORTAL, BAND, DECAL.
/// For SCATTER, patching param_d changes the seed — produces shimmer/respawn, not smooth motion.
/// Variant-specific note:
/// treat phase support as a property of the authored variant, not the opcode family.
/// In this showcase, reliable movers include FLOW, LOBE, GRID, BAND, DECAL,
/// PLANE/WATER, VEIL/RAIN_WALL, and PORTAL/VORTEX.
/// PORTAL/RECT stays static, and SCATTER still uses `param_d` as a seed rather than smooth phase.
pub static ANIM_SPEEDS: [[u8; 8]; PRESET_COUNT] = [
    //                                   L0 L1 L2 L3 L4 L5 L6 L7
    [0, 0, 0, 1, 2, 0, 1, 1], // 0: Neon Metropolis (rain streaks slow)
    [0, 0, 1, 1, 1, 0, 0, 0], // 1: Sakura Shrine (gentle)
    [0, 0, 3, 0, 0, 4, 0, 0], // 2: Ocean Depths (caustic drift + biolum vent need obvious motion)
    [0, 1, 0, 0, 0, 0, 0, 1], // 3: Void Station
    [0, 0, 1, 1, 1, 1, 0, 0], // 4: Desert Mirage (slow heat shimmer)
    [0, 0, 1, 1, 1, 0, 0, 1], // 5: Enchanted Grove (slow light shafts)
    [0, 0, 1, 0, 0, 1, 1, 0], // 6: Astral Void (very subtle)
    [0, 0, 0, 0, 0, 0, 0, 0], // 7: Hell Core (STATIC - no seizures!)
    [0, 0, 0, 1, 1, 1, 1, 1], // 8: Sky Ruins
    [0, 0, 4, 4, 4, 0, 2, 4], // 9: Combat Lab (floor scan + wall bay scan + projection field pulse)
    [0, 0, 3, 5, 0, 1, 1, 0], // 10: Frozen Tundra (two bounds set the ridge/face first; SURFACE glaze/crust then carry the ice bed while ADVECT stays subordinate)
    [0, 6, 1, 2, 0, 4, 8, 0], // 11: Storm Front (MASS owns the shelf; ADVECT carries subordinate internal transport)
    [0, 0, 0, 1, 1, 0, 0, 1], // 12: Crystal Cavern (veins + shard shimmer + cold lobe)
    [0, 0, 0, 0, 1, 0, 0, 1], // 13: Moonlit Graveyard (mist + haze drift)
    [0, 0, 0, 0, 1, 1, 0, 1], // 14: Alien Jungle (humid haze + spores + canopy drift)
    [0, 0, 0, 0, 0, 0, 0, 1], // 15: Gothic Cathedral (restrained incense drift only)
    [0, 0, 0, 0, 0, 1, 0, 1], // 16: Toxic Wasteland (chemical ground shimmer + hazard lobe)
    [0, 0, 0, 1, 0, 1, 1, 1], // 17: Neon Arcade (scan cadence + CRT drift + glow pulse)
    [0, 0, 0, 0, 1, 0, 0, 1], // 18: War Zone (smoke drift + flare pulse)
    [0, 0, 0, 0, 0, 1, 1, 1], // 19: Digital Matrix (code drift + partition band motion)
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
