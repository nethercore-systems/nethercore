//! Preset set 01-04

#[allow(unused_imports)]
use crate::constants::*;

// =============================================================================
// Environment Presets (EPU 128-bit format)
// =============================================================================

// -----------------------------------------------------------------------------
// Preset 1: "Neon Metropolis" - Rain-soaked cyberpunk alley
// -----------------------------------------------------------------------------
// L0: RAMP (sky=#1a0a2e, floor=#080810, walls=#1c1c1c)
// L1: SECTOR/TUNNEL (enclosure, #1a0a2e / #0a0618)
// L2: SILHOUETTE/CITY (black skyline cutout, walls)
// L3: GRID (cyan #00ffff, walls, scroll)
// L4: SCATTER/WINDOWS (yellow #ffcc00, walls, AXIS_CYL)
// L5: VEIL/LASER_BARS (magenta #ff00ff, walls, AXIS_CYL)
// L6: FLOW (cyan #00ddff, rain, dir=DOWN)
// L7: ATMOSPHERE/MIE (gray haze #404050)
pub(super) const PRESET_NEON_METROPOLIS: [[u64; 2]; 8] = [
    // L0: RAMP - deep purple sky, near-black floor, dark gray walls
    [
        hi(OP_RAMP, REGION_ALL, BLEND_LERP, 0, 0x1a0a2e, 0x080810),
        lo(220, 0x1c, 0x1c, 0x1c, THRESH_INTERIOR, DIR_UP, 15, 15),
    ],
    // L1: SECTOR/TUNNEL - tunnel enclosure bounding the scene
    [
        hi_meta(
            OP_SECTOR,
            REGION_ALL,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            SECTOR_TUNNEL,
            0x1a0a2e,
            0x0a0618,
        ),
        lo(180, 80, 0, 0, 0, DIR_UP, 15, 15),
    ],
    // L2: SILHOUETTE/CITY - city skyline cutout on walls
    [
        hi_meta(
            OP_SILHOUETTE,
            REGION_WALLS,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            SILHOUETTE_CITY,
            0x0a0510,
            0x000000,
        ),
        lo(200, 128, 0, 0, 0, DIR_UP, 15, 0),
    ],
    // L3: GRID - cyan neon grid on walls with scroll animation
    [
        hi(OP_GRID, REGION_WALLS, BLEND_ADD, 0, 0x00ffff, 0x000000),
        lo(220, 32, 0, 3, 0, DIR_UP, 15, 0),
    ],
    // L4: SCATTER/WINDOWS - warm yellow window lights on walls
    [
        hi_meta(
            OP_SCATTER,
            REGION_WALLS,
            BLEND_ADD,
            DOMAIN_AXIS_CYL,
            SCATTER_WINDOWS,
            0xffcc00,
            0x000000,
        ),
        lo(180, 40, 0, 0x20, 0, DIR_UP, 15, 0),
    ],
    // L5: VEIL/LASER_BARS - magenta laser beams on walls
    [
        hi_meta(
            OP_VEIL,
            REGION_WALLS,
            BLEND_SCREEN,
            DOMAIN_AXIS_CYL,
            VEIL_LASER_BARS,
            0xff00ff,
            0x000000,
        ),
        lo(180, 64, 0, 0, 0, DIR_UP, 15, 0),
    ],
    // L6: FLOW - cyan rain effect (downward)
    [
        hi(OP_FLOW, REGION_ALL, BLEND_ADD, 0, 0x00ddff, 0x000000),
        lo(60, 128, 0, 0, 200, DIR_DOWN, 15, 0),
    ],
    // L7: ATMOSPHERE/MIE - gray urban haze
    [
        hi_meta(
            OP_ATMOSPHERE,
            REGION_ALL,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            ATMO_MIE,
            0x404050,
            0x000000,
        ),
        lo(60, 80, 0, 0, 0, DIR_UP, 15, 0),
    ],
];

// -----------------------------------------------------------------------------
// Preset 2: "Crimson Hellscape" - Volcanic hellscape with dimensional rifts
// -----------------------------------------------------------------------------
// L0: RAMP (sky=#4a0000, floor=#0a0000, walls=#2a0808)
// L1: PATCHES/MEMBRANE (dark red #330000, organic tissue, DIRECT3D)
// L2: TRACE/CRACKS (lava veins #ff3300, floor, TANGENT_LOCAL)
// L3: FLOW (churning lava #ff4400, floor)
// L4: SCATTER/EMBERS (rising sparks #ff8800)
// L5: CELESTIAL/ECLIPSE (blood eclipse #200000/#ff0000, dir=SUN)
// L6: PORTAL/RIFT (dimensional tear #ff2200, walls, TANGENT_LOCAL)
// L7: ATMOSPHERE/ABSORPTION (blood mist #400000)
pub(super) const PRESET_CRIMSON_HELLSCAPE: [[u64; 2]; 8] = [
    // L0: RAMP - blood red sky, charred black floor, dark crimson walls
    [
        hi(OP_RAMP, REGION_ALL, BLEND_LERP, 0, 0x4a0000, 0x0a0000),
        lo(230, 0x2a, 0x08, 0x08, THRESH_BALANCED, DIR_UP, 15, 15),
    ],
    // L1: PATCHES/MEMBRANE - organic tissue texture on walls
    [
        hi_meta(
            OP_PATCHES,
            REGION_WALLS,
            BLEND_SCREEN,
            DOMAIN_DIRECT3D,
            PATCHES_MEMBRANE,
            0x330000,
            0x1a0000,
        ),
        lo(150, 96, 32, 0, 0, DIR_UP, 15, 0),
    ],
    // L2: TRACE/CRACKS - volcanic lava veins on floor
    [
        hi_meta(
            OP_TRACE,
            REGION_FLOOR,
            BLEND_ADD,
            DOMAIN_TANGENT_LOCAL,
            TRACE_CRACKS,
            0xff3300,
            0x000000,
        ),
        lo(130, 90, 64, 0, 0, DIR_UP, 15, 0),
    ],
    // L3: FLOW - churning lava glow on floor
    [
        hi(OP_FLOW, REGION_FLOOR, BLEND_SCREEN, 0, 0xff4400, 0x000000),
        lo(120, 80, 0, 0, 64, DIR_UP, 15, 0),
    ],
    // L4: SCATTER/EMBERS - rising sparks and embers
    [
        hi_meta(
            OP_SCATTER,
            REGION_ALL,
            BLEND_ADD,
            DOMAIN_DIRECT3D,
            SCATTER_EMBERS,
            0xff8800,
            0x000000,
        ),
        lo(110, 30, 20, 0x30, 0, DIR_UP, 15, 0),
    ],
    // L5: CELESTIAL/ECLIPSE - blood eclipse in the sky
    [
        hi_meta(
            OP_CELESTIAL,
            REGION_SKY,
            BLEND_ADD,
            DOMAIN_DIRECT3D,
            CELESTIAL_ECLIPSE,
            0x200000,
            0xff0000,
        ),
        lo(160, 200, 0, 0, 0, DIR_SUN, 15, 15),
    ],
    // L6: PORTAL/RIFT - dimensional tear on walls
    [
        hi_meta(
            OP_PORTAL,
            REGION_WALLS,
            BLEND_SCREEN,
            DOMAIN_TANGENT_LOCAL,
            PORTAL_RIFT,
            0xff2200,
            0x400000,
        ),
        lo(130, 90, 64, 0, 0, DIR_UP, 15, 0),
    ],
    // L7: ATMOSPHERE/ABSORPTION - thick blood mist
    [
        hi_meta(
            OP_ATMOSPHERE,
            REGION_ALL,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            ATMO_ABSORPTION,
            0x400000,
            0x000000,
        ),
        lo(120, 100, 0, 0, 0, DIR_UP, 15, 0),
    ],
];

// -----------------------------------------------------------------------------
// Preset 3: "Frozen Tundra" - Arctic ice shelf, blizzard
// -----------------------------------------------------------------------------
// L0: RAMP (sky=#c8e0f0, floor=#f8f8ff, walls=#a0c8e0)
// L1: CELL/SHATTER (cracked ice #d0f0ff / #a0c8e0, floor)
// L2: PLANE/STONE (frozen ground #e8f4ff / #c0d8e8, floor)
// L3: SCATTER/SNOW (blizzard #ffffff, dir=DOWN)
// L4: FLOW (drifting snow clouds #ffffff, sky, dir=DOWN)
// L5: ATMOSPHERE/RAYLEIGH (crisp cold air #b0d8f0)
// L6: LOBE (pale sun glow #e0f0ff, dir=SUN)
// L7: NOP_LAYER
pub(super) const PRESET_FROZEN_TUNDRA: [[u64; 2]; 8] = [
    // L0: RAMP - pale blue sky, white floor, ice blue walls
    [
        hi(OP_RAMP, REGION_ALL, BLEND_LERP, 0, 0xa0c0e0, 0xf8f8ff),
        lo(240, 0xa0, 0xc8, 0xe0, THRESH_OPEN, DIR_UP, 15, 15),
    ],
    // L1: CELL/SHATTER - cracked ice pattern on floor
    [
        hi_meta(
            OP_CELL,
            REGION_FLOOR,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            CELL_SHATTER,
            0x80b0d0,
            0x5080a0,
        ),
        lo(140, 80, 0, 0, 0, DIR_UP, 15, 15),
    ],
    // L2: PLANE/STONE - frozen ground texture on floor
    [
        hi_meta(
            OP_PLANE,
            REGION_FLOOR,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            PLANE_STONE,
            0xe8f4ff,
            0xc0d8e8,
        ),
        lo(150, 96, 0, 0, 0, DIR_UP, 15, 15),
    ],
    // L3: SCATTER/SNOW - blizzard snowfall (downward)
    [
        hi_meta(
            OP_SCATTER,
            REGION_ALL,
            BLEND_ADD,
            DOMAIN_DIRECT3D,
            SCATTER_SNOW,
            0xc0d8ff,
            0x000000,
        ),
        lo(100, 128, 100, 0x20, 0, DIR_DOWN, 15, 0),
    ],
    // L4: FLOW - drifting snow clouds in sky
    [
        hi(OP_FLOW, REGION_SKY, BLEND_ADD, 0, 0xffffff, 0x000000),
        lo(120, 128, 0, 3, 80, DIR_DOWN, 15, 0),
    ],
    // L5: ATMOSPHERE/RAYLEIGH - crisp cold arctic air
    [
        hi_meta(
            OP_ATMOSPHERE,
            REGION_ALL,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            ATMO_RAYLEIGH,
            0xb0d8f0,
            0x000000,
        ),
        lo(100, 60, 0, 0, 0, DIR_UP, 15, 0),
    ],
    // L6: LOBE - pale sun glow from above
    [
        hi(OP_LOBE, REGION_ALL, BLEND_ADD, 0, 0xe0f0ff, 0x000000),
        lo(100, 128, 0, 0, 0, DIR_SUN, 15, 0),
    ],
    // L7: (empty)
    NOP_LAYER,
];

// -----------------------------------------------------------------------------
// Preset 4: "Alien Jungle" - Bioluminescent alien canopy
// -----------------------------------------------------------------------------
// L0: RAMP (sky=#3a0050, floor=#002020, walls=#004040)
// L1: SILHOUETTE/FOREST (alien tree silhouettes #001818 / #003030, walls)
// L2: PATCHES/STREAKS (bioluminescent streaks #00ffaa / #004040, walls, AXIS_CYL)
// L3: VEIL/CURTAINS (hanging bioluminescent vines #8000ff, walls, AXIS_CYL)
// L4: SCATTER/DUST (floating spores #00ffcc)
// L5: FLOW (rippling bioluminescence #00ddcc, floor)
// L6: ATMOSPHERE/ALIEN (exotic gas #004020)
// L7: LOBE (canopy glow #3a0050, sky, dir=UP)
pub(super) const PRESET_ALIEN_JUNGLE: [[u64; 2]; 8] = [
    // L0: RAMP - purple sky, dark teal floor, teal walls
    [
        hi(OP_RAMP, REGION_ALL, BLEND_LERP, 0, 0x3a0050, 0x002020),
        lo(230, 0x00, 0x40, 0x40, THRESH_SEMI, DIR_UP, 15, 15),
    ],
    // L1: SILHOUETTE/FOREST - alien tree silhouettes on walls
    [
        hi_meta(
            OP_SILHOUETTE,
            REGION_WALLS,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            SILHOUETTE_FOREST,
            0x000808,
            0x001010,
        ),
        lo(180, 128, 0, 0, 0, DIR_UP, 15, 15),
    ],
    // L2: PATCHES/STREAKS - bioluminescent streaks on walls
    [
        hi_meta(
            OP_PATCHES,
            REGION_WALLS,
            BLEND_ADD,
            DOMAIN_AXIS_CYL,
            PATCHES_STREAKS,
            0x00ffaa,
            0x004040,
        ),
        lo(180, 96, 64, 0, 0, DIR_UP, 15, 0),
    ],
    // L3: VEIL/CURTAINS - hanging bioluminescent vines on walls
    [
        hi_meta(
            OP_VEIL,
            REGION_WALLS,
            BLEND_SCREEN,
            DOMAIN_AXIS_CYL,
            VEIL_CURTAINS,
            0x8000ff,
            0x000000,
        ),
        lo(160, 96, 64, 0, 0, DIR_DOWN, 15, 0),
    ],
    // L4: SCATTER/DUST - floating spores (cyan)
    [
        hi_meta(
            OP_SCATTER,
            REGION_ALL,
            BLEND_ADD,
            DOMAIN_DIRECT3D,
            SCATTER_DUST,
            0x00ffcc,
            0x000000,
        ),
        lo(100, 30, 20, 0x30, 0, DIR_UP, 15, 0),
    ],
    // L5: FLOW - rippling bioluminescence on floor
    [
        hi(OP_FLOW, REGION_FLOOR, BLEND_SCREEN, 0, 0x00ddcc, 0x000000),
        lo(80, 96, 0, 0, 100, DIR_UP, 15, 0),
    ],
    // L6: ATMOSPHERE/ALIEN - exotic gas atmosphere (green tint)
    [
        hi_meta(
            OP_ATMOSPHERE,
            REGION_ALL,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            ATMO_ALIEN,
            0x004020,
            0x000000,
        ),
        lo(80, 50, 0, 0, 0, DIR_UP, 15, 0),
    ],
    // L7: LOBE - canopy glow from above (purple, downward)
    [
        hi(OP_LOBE, REGION_SKY, BLEND_ADD, 0, 0x3a0050, 0x000000),
        lo(100, 128, 0, 0, 0, DIR_DOWN, 15, 0),
    ],
];
