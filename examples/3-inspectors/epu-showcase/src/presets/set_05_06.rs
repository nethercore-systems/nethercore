//! Preset set 05-06

#[allow(unused_imports)]
use crate::constants::*;

// -----------------------------------------------------------------------------
// Preset 5: "Desert Mirage" - Vast dunes under blazing sun
// -----------------------------------------------------------------------------
// Visual: BOUNDLESS vast desert with dramatic horizon, rolling dune silhouettes,
// intense heat shimmer, mirage pool illusion, and blinding sun glare.
//
// Cadence: BOUNDS (RAMP) -> FEATURES (silhouette/sand) -> FEATURES (heat/mirage/glare)
//
// L0: RAMP                 ALL           LERP   boundless desert sky-to-sand gradient
// L1: SILHOUETTE/DUNES     SKY           LERP   prominent dune horizon silhouettes
// L2: PLANE/SAND           FLOOR         LERP   textured sand with ripples
// L3: BAND                 ALL           ADD    bright horizon heat shimmer band
// L4: FLOW                 ALL           SCREEN visible heat distortion waves
// L5: PORTAL/RIFT          FLOOR         SCREEN mirage pool (false water reflection)
// L6: LOBE                 ALL           ADD    BLINDING sun glare (maximum)
// L7: SCATTER/DUST         ALL           ADD    blowing sand particles
pub(super) const PRESET_DESERT_MIRAGE: [[u64; 2]; 8] = [
    // L0: SILHOUETTE/DUNES - ONLY bounds layer, defines the horizon dunes
    [
        hi_meta(
            OP_SILHOUETTE,
            REGION_SKY,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            SILHOUETTE_DUNES,
            0x402810, // DARK brown dune silhouettes
            0x906840, // warm tan sky (NOT bright)
        ),
        lo(255, 200, 120, 180, 0, DIR_UP, 15, 15),
    ],
    // L1: PLANE/SAND - textured sand floor with warm colors
    [
        hi_meta(
            OP_PLANE,
            REGION_FLOOR,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            PLANE_SAND,
            0x806030, // warm golden sand (darker)
            0x402010, // deep brown shadows
        ),
        lo(255, 120, 70, 160, 25, DIR_SUN, 15, 14),
    ],
    // L2: BAND - heat shimmer at horizon (subtle, SCREEN not ADD)
    [
        hi(OP_BAND, REGION_WALLS, BLEND_SCREEN, 0, 0x806040, 0x503020),
        lo(50, 70, 120, 140, 0, DIR_FORWARD, 7, 2),
    ],
    // L3: FLOW - heat distortion (slow animation)
    [
        hi(OP_FLOW, REGION_WALLS, BLEND_SCREEN, 0, 0x705030, 0x403020),
        lo(40, 180, 100, 0x50, 0, DIR_UP, 6, 2),
    ],
    // L4: PORTAL/RIFT - mirage pool on floor (subtle)
    [
        hi_meta(
            OP_PORTAL,
            REGION_FLOOR,
            BLEND_SCREEN,
            DOMAIN_TANGENT_LOCAL,
            PORTAL_RIFT,
            0x507090, // sky-blue mirage (muted)
            0x304060,
        ),
        lo(80, 180, 60, 200, 0, DIR_FORWARD, 8, 3),
    ],
    // L5: LOBE - sun glare (SCREEN not ADD, moderate)
    [
        hi(OP_LOBE, REGION_ALL, BLEND_SCREEN, 0, 0xa08040, 0x604020),
        lo(100, 180, 120, 1, 0, DIR_SUN, 10, 4),
    ],
    // L6: SCATTER/DUST - blowing sand (SCREEN not ADD)
    [
        hi_meta(
            OP_SCATTER,
            REGION_ALL,
            BLEND_SCREEN,
            DOMAIN_DIRECT3D,
            SCATTER_DUST,
            0x806030, // warm sand particles (darker)
            0x503020,
        ),
        lo(40, 20, 40, 0x18, 40, DIR_RIGHT, 7, 2),
    ],
    // L7: ATMOSPHERE/ABSORPTION - warm haze for depth
    [
        hi_meta(
            OP_ATMOSPHERE,
            REGION_ALL,
            BLEND_MULTIPLY,
            DOMAIN_DIRECT3D,
            ATMO_ABSORPTION,
            0x907050, // warm haze (darker)
            0x604030,
        ),
        lo(50, 100, 80, 0, 0, DIR_UP, 8, 0),
    ],
];

// -----------------------------------------------------------------------------
// Preset 6: "Enchanted Grove" - Fairy tale forest
// -----------------------------------------------------------------------------
// Design: Magical forest with SILHOUETTE/FOREST for tree shapes, warm green
// and golden tones (not harsh black/green). Dappled sunlight, fireflies.
//
// Cadence: BOUNDS (silhouette/forest) -> FEATURES (floor/light/fireflies)
//
// L0: SILHOUETTE/FOREST    SKY        LERP   tree canopy silhouettes
// L1: PLANE/GRASS          FLOOR      LERP   mossy forest floor
// L2: FLOW                 FLOOR      SCREEN dappled light motion
// L3: VEIL/SHARDS          SKY|WALLS  ADD    golden sun shafts
// L4: LOBE                 ALL        ADD    warm golden sun glow
// L5: BAND                 SKY        ADD    canopy glow
// L6: SCATTER/EMBERS       ALL        ADD    firefly motes
// L7: ATMOSPHERE           ALL        MULT   soft forest haze
pub(super) const PRESET_ENCHANTED_GROVE: [[u64; 2]; 8] = [
    // L0: SILHOUETTE/FOREST - tree canopy silhouettes (warmer, not black)
    [
        hi_meta(
            OP_SILHOUETTE,
            REGION_SKY,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            SILHOUETTE_FOREST,
            0x283018, // Dark olive-green (warm, not black)
            0x809050, // Warm yellow-green canopy light
        ),
        lo(255, 160, 200, 0x80, 0, DIR_UP, 15, 15),
    ],
    // L1: PLANE/GRASS - warm mossy forest floor
    [
        hi_meta(
            OP_PLANE,
            REGION_FLOOR,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            PLANE_GRASS,
            0x405828, // Warm moss green
            0x202818, // Warm shadow (not black)
        ),
        lo(200, 85, 45, 130, 0, DIR_UP, 15, 13),
    ],
    // L2: VEIL/PILLARS - light shafts through tree canopy (key visual)
    [
        hi_meta(
            OP_VEIL,
            REGION_SKY | REGION_WALLS,
            BLEND_ADD,
            DOMAIN_AXIS_CYL,
            VEIL_PILLARS,
            0xa08030, // Golden light pillars
            0x503010, // Amber base
        ),
        // Slow animation (alpha_b=2)
        lo(90, 40, 50, 45, 0, DIR_SUN, 10, 2),
    ],
    // L3: FLOW - dappled golden light on floor
    [
        hi(OP_FLOW, REGION_FLOOR, BLEND_SCREEN, 0, 0x605020, 0x403010),
        // Slow animation (alpha_b=2)
        lo(60, 100, 70, 0x28, 0, DIR_RIGHT, 7, 2),
    ],
    // L4: LOBE - warm golden sun glow
    [
        hi(OP_LOBE, REGION_ALL, BLEND_ADD, 0, 0x907020, 0x402808),
        lo(100, 120, 70, 1, 0, DIR_SUN, 10, 3),
    ],
    // L5: BAND - warm canopy glow at horizon
    [
        hi(OP_BAND, REGION_SKY, BLEND_ADD, 0, 0x506028, 0x283010),
        lo(40, 100, 80, 120, 0, DIR_UP, 6, 2),
    ],
    // L6: SCATTER/EMBERS - sparse golden fireflies
    [
        hi_meta(
            OP_SCATTER,
            REGION_ALL,
            BLEND_ADD,
            DOMAIN_DIRECT3D,
            SCATTER_EMBERS,
            0xffc040, // Golden-yellow fireflies
            0x80a020, // Yellow-green glow
        ),
        // Sparse, slow (alpha_b=1)
        lo(50, 6, 30, 0x14, 12, DIR_UP, 9, 1),
    ],
    // L7: VEIL/SHARDS - additional light shards for depth
    [
        hi_meta(
            OP_VEIL,
            REGION_SKY,
            BLEND_SCREEN,
            DOMAIN_AXIS_CYL,
            VEIL_SHARDS,
            0x807030, // Warm golden shards
            0x403818,
        ),
        // Slow animation (alpha_b=2)
        lo(60, 25, 30, 40, 0, DIR_SUN, 7, 2),
    ],
];
