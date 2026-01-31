//! Preset set 01-02

#[allow(unused_imports)]
use crate::constants::*;

// -----------------------------------------------------------------------------
// Preset 1: "Neon Metropolis" - Rain-soaked cyberpunk alley
// -----------------------------------------------------------------------------
// Goal: unmistakably "city alley" with CLEAR VISUAL HIERARCHY.
// SINGLE bounds layer (SILHOUETTE/CITY) - no stacking!
//
// Cadence: BOUNDS (silhouette) -> FEATURES (ground/neon/windows/rain)
//
// L0: SILHOUETTE/CITY       SKY        LERP   city skyline (ONLY bounds layer)
// L1: PLANE/PAVEMENT        FLOOR      LERP   dark wet asphalt
// L2: DECAL/RECT            WALLS      ADD    hero neon sign
// L3: SCATTER/WINDOWS       WALLS      ADD    sparse window lights
// L4: FLOW                  FLOOR      SCREEN neon reflection puddles
// L5: LOBE                  ALL        ADD    neon spill glow
// L6: SCATTER/RAIN          ALL        ADD    falling rain drops
// L7: VEIL/RAIN_WALL        ALL        SCREEN rain streaks on surfaces
pub(super) const PRESET_NEON_METROPOLIS: [[u64; 2]; 8] = [
    // L0: SILHOUETTE/CITY - city skyline (ONLY bounds layer!)
    [
        hi_meta(
            OP_SILHOUETTE,
            REGION_SKY,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            SILHOUETTE_CITY,
            0x010104, // Near-black building silhouettes
            0x100818, // Dark purple polluted sky
        ),
        lo(255, 120, 200, 0x90, 0, DIR_UP, 15, 15),
    ],
    // L1: PLANE/PAVEMENT - dark wet asphalt
    [
        hi_meta(
            OP_PLANE,
            REGION_FLOOR,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            PLANE_PAVEMENT,
            0x060810, // Wet dark asphalt
            0x020304, // Cracks
        ),
        lo(200, 100, 25, 140, 0, DIR_UP, 15, 14),
    ],
    // L2: DECAL/RECT - HERO NEON SIGN (brightest element)
    [
        hi(OP_DECAL, REGION_WALLS, BLEND_ADD, 0, 0xff20ff, 0x00ffff),
        lo(255, 0x28, 180, 60, 0x40, DIR_BACK, 15, 15),
    ],
    // L3: SCATTER/WINDOWS - sparse warm window lights
    [
        hi_meta(
            OP_SCATTER,
            REGION_WALLS,
            BLEND_ADD,
            DOMAIN_DIRECT3D,
            SCATTER_WINDOWS,
            0xffd080,
            0x804020,
        ),
        lo(50, 20, 14, 0x24, 22, DIR_BACK, 9, 5),
    ],
    // L4: FLOW - neon reflection on wet ground
    [
        hi(OP_FLOW, REGION_FLOOR, BLEND_SCREEN, 0, 0x5080a0, 0xa05080),
        lo(60, 130, 55, 0x24, 0, DIR_FORWARD, 9, 5),
    ],
    // L5: LOBE - neon spill glow
    [
        hi(OP_LOBE, REGION_ALL, BLEND_ADD, 0, 0xb030a0, 0x107080),
        lo(70, 170, 55, 1, 0, DIR_BACK, 9, 5),
    ],
    // L6: VEIL/RAIN_WALL - streaking rain (primary rain effect)
    [
        hi_meta(
            OP_VEIL,
            REGION_ALL,
            BLEND_SCREEN,
            DOMAIN_AXIS_CYL,
            VEIL_RAIN_WALL,
            0x5070a0, // blue-gray rain streaks
            0x203040,
        ),
        lo(80, 60, 20, 140, 0x18, DIR_DOWN, 9, 3),
    ],
    // L7: FLOW - rain shimmer/mist in air
    [
        hi(OP_FLOW, REGION_ALL, BLEND_SCREEN, 0, 0x405060, 0x182028),
        lo(40, 120, 40, 0x20, 0, DIR_DOWN, 6, 2),
    ],
];

// -----------------------------------------------------------------------------
// Preset 2: "Sakura Shrine" - Weathered temple in perpetual bloom
// -----------------------------------------------------------------------------
// Intent: outdoor calm with obvious motion (petals + sun shimmer).
// Visual: weathered temple with SPIRE silhouettes (torii gates, pagoda tips),
// moss-covered stone paths, golden afternoon light, drifting pink petals.
//
// Cadence: BOUNDS (silhouette/spires) -> FEATURES (floor/light/petals)
//
// L0: SILHOUETTE/SPIRES     SKY        LERP   temple spires/torii silhouettes
// L1: PLANE/STONE           FLOOR      LERP   mossy path stones
// L2: LOBE                  ALL        ADD    golden afternoon sun
// L3: FLOW                  FLOOR      SCREEN subtle golden reflection
// L4: VEIL/SHARDS           SKY|WALLS  ADD    light shafts through trees
// L5: BAND                  SKY        ADD    warm horizon glow
// L6: SCATTER/DUST          ALL        ADD    drifting cherry petals
// L7: ATMOSPHERE            ALL        MULT   soft atmospheric haze
pub(super) const PRESET_SAKURA_SHRINE: [[u64; 2]; 8] = [
    // L0: SILHOUETTE/SPIRES - temple spires and torii gate silhouettes
    [
        hi_meta(
            OP_SILHOUETTE,
            REGION_SKY,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            SILHOUETTE_SPIRES,
            0x180c08, // Dark timber silhouettes
            0xb08050, // Warm amber sky
        ),
        lo(255, 140, 180, 0x70, 0, DIR_UP, 15, 15),
    ],
    // L1: PLANE/STONE - mossy green-brown path stones
    [
        hi_meta(
            OP_PLANE,
            REGION_FLOOR,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            PLANE_STONE,
            0x384028, // Mossy stone
            0x141808, // Dark moss shadow
        ),
        lo(200, 90, 55, 130, 0, DIR_UP, 15, 13),
    ],
    // L2: LOBE - warm golden afternoon sun
    [
        hi(OP_LOBE, REGION_ALL, BLEND_ADD, 0, 0xd09040, 0x402008),
        lo(130, 160, 90, 1, 0, DIR_SUNSET, 12, 6),
    ],
    // L3: FLOW - golden shimmer on stones
    [
        hi(OP_FLOW, REGION_FLOOR, BLEND_SCREEN, 0, 0x504018, 0x281008),
        lo(50, 140, 50, 0x1c, 0, DIR_RIGHT, 6, 3),
    ],
    // L4: VEIL/SHARDS - golden light shafts
    [
        hi_meta(
            OP_VEIL,
            REGION_SKY | REGION_WALLS,
            BLEND_ADD,
            DOMAIN_AXIS_CYL,
            VEIL_SHARDS,
            0xc09050, // Golden shafts
            0x402008,
        ),
        lo(70, 35, 25, 50, 0, DIR_SUNSET, 9, 4),
    ],
    // L5: BAND - warm horizon glow
    [
        hi(OP_BAND, REGION_SKY, BLEND_ADD, 0, 0x906030, 0x301008),
        lo(60, 110, 50, 70, 0, DIR_SUNSET, 7, 3),
    ],
    // L6: SCATTER/DUST - drifting cherry petals (pink accent)
    [
        hi_meta(
            OP_SCATTER,
            REGION_ALL,
            BLEND_ADD,
            DOMAIN_DIRECT3D,
            SCATTER_DUST,
            0xff90a0, // Soft pink petals
            0xc06070,
        ),
        lo(60, 16, 22, 0x24, 12, DIR_SUNSET, 9, 4),
    ],
    // L7: ATMOSPHERE - soft haze for depth
    [
        hi_meta(
            OP_ATMOSPHERE,
            REGION_ALL,
            BLEND_MULTIPLY,
            DOMAIN_DIRECT3D,
            ATMO_ABSORPTION,
            0xc0a080, // Warm haze
            0x806040,
        ),
        lo(30, 100, 70, 0, 0, DIR_UP, 6, 0),
    ],
];
