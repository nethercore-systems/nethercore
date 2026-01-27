//! Preset set 09-12

#[allow(unused_imports)]
use crate::constants::*;

// -----------------------------------------------------------------------------
// Preset 9: "Neon Arcade" — Retro synthwave
// -----------------------------------------------------------------------------
// L0: RAMP (sky=#000010, floor=#100020, walls=#080018)
// L1: SPLIT/BANDS (neon cyan/magenta, horizontal bands on walls)
// L2: GRID (magenta #ff00ff, wireframe on floor)
// L3: SCATTER/STARS (white #ffffff, starfield)
// L4: CELESTIAL/PLANET (magenta #ff0088, retro planet)
// L5: BAND (cyan #00ffff, horizon glow)
// L6: FLOW (purple #8000ff, pulsing glow)
// L7: NOP
pub(super) const PRESET_NEON_ARCADE: [[u64; 2]; 8] = [
    // L0: RAMP - black sky, dark purple floor, dark blue walls
    [
        hi(OP_RAMP, REGION_ALL, BLEND_LERP, 0, 0x000010, 0x100020),
        lo(220, 0x08, 0x00, 0x18, THRESH_BALANCED, DIR_UP, 15, 15),
    ],
    // L1: SPLIT/BANDS - neon horizontal bands on walls (cyan/magenta)
    [
        hi_meta(
            OP_SPLIT,
            REGION_WALLS,
            BLEND_ADD,
            DOMAIN_DIRECT3D,
            SPLIT_BANDS,
            0x00ffff,
            0xff00ff,
        ),
        lo(120, 200, 0, 0, 0, DIR_UP, 15, 15),
    ],
    // L2: GRID - magenta wireframe grid on floor (retro style)
    [
        hi(OP_GRID, REGION_FLOOR, BLEND_ADD, 0, 0xff00ff, 0x000000),
        lo(140, 48, 0, 4, 0, DIR_UP, 15, 0),
    ],
    // L3: SCATTER/STARS - background starfield (white)
    [
        hi_meta(
            OP_SCATTER,
            REGION_SKY,
            BLEND_SCREEN,
            DOMAIN_DIRECT3D,
            SCATTER_STARS,
            0xffffff,
            0x000000,
        ),
        lo(130, 150, 0, 0x10, 0, DIR_UP, 15, 0),
    ],
    // L4: CELESTIAL/PLANET - retro magenta planet on horizon
    [
        hi_meta(
            OP_CELESTIAL,
            REGION_SKY,
            BLEND_ADD,
            DOMAIN_DIRECT3D,
            CELESTIAL_PLANET,
            0xff0088,
            0x000000,
        ),
        lo(160, 128, 0, 0, 0, DIR_SUNSET, 15, 0),
    ],
    // L5: BAND - cyan horizon glow line
    [
        hi(OP_BAND, REGION_ALL, BLEND_ADD, 0, 0x00ffff, 0x000000),
        lo(200, 128, 0, 0, 0, DIR_SUNSET, 15, 0),
    ],
    // L6: FLOW - purple pulsing ambient glow
    [
        hi(OP_FLOW, REGION_ALL, BLEND_SCREEN, 0, 0x8000ff, 0x000000),
        lo(100, 128, 0, 0, 60, DIR_UP, 15, 0),
    ],
    // L7: NOP
    NOP_LAYER,
];

// -----------------------------------------------------------------------------
// Preset 10: "Storm Front" — Dramatic thunderstorm
// -----------------------------------------------------------------------------
// L0: RAMP (sky=#202830, floor=#181820, walls=#303840)
// L1: SILHOUETTE/MOUNTAINS (dark #181820 / #282830, distant mountains)
// L2: FLOW (dark gray #404858, churning storm clouds)
// L3: TRACE/LIGHTNING (white #ffffff, sky, AXIS_POLAR)
// L4: VEIL/RAIN_WALL (blue-gray #607080, AXIS_CYL)
// L5: SCATTER/RAIN (rain blue #8090a0, AXIS_CYL, dir=DOWN)
// L6: PLANE/PAVEMENT (wet gray #282830 / #202028)
// L7: ATMOSPHERE/FULL (storm gray #303038)
pub(super) const PRESET_STORM_FRONT: [[u64; 2]; 8] = [
    // L0: RAMP - dark gray sky, wet ground, slate walls
    [
        hi(OP_RAMP, REGION_ALL, BLEND_LERP, 0, 0x0c1018, 0x181820),
        lo(230, 0x30, 0x38, 0x40, THRESH_OPEN, DIR_UP, 15, 15),
    ],
    // L1: SILHOUETTE/MOUNTAINS - distant storm mountains (dark)
    [
        hi_meta(
            OP_SILHOUETTE,
            REGION_WALLS,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            SILHOUETTE_MOUNTAINS,
            0x181820,
            0x282830,
        ),
        lo(160, 128, 0, 0, 0, DIR_UP, 15, 15),
    ],
    // L2: FLOW - churning storm clouds (dark gray)
    [
        hi(OP_FLOW, REGION_SKY, BLEND_ADD, 0, 0x404858, 0x000000),
        lo(140, 128, 0, 0x22, 120, DIR_UP, 15, 0),
    ],
    // L3: TRACE/LIGHTNING - dramatic lightning bolts (white, polar domain)
    [
        hi_meta(
            OP_TRACE,
            REGION_SKY,
            BLEND_ADD,
            DOMAIN_AXIS_POLAR,
            TRACE_LIGHTNING,
            0xffffff,
            0x000000,
        ),
        lo(255, 80, 0, 0, 0, DIR_UP, 15, 0),
    ],
    // L4: VEIL/RAIN_WALL - heavy rain curtains (blue-gray, cylindrical)
    [
        hi_meta(
            OP_VEIL,
            REGION_ALL,
            BLEND_SCREEN,
            DOMAIN_AXIS_CYL,
            VEIL_RAIN_WALL,
            0x607080,
            0x000000,
        ),
        lo(100, 128, 80, 0, 0, DIR_DOWN, 15, 0),
    ],
    // L5: SCATTER/RAIN - raindrops (rain blue, cylindrical, falling)
    [
        hi_meta(
            OP_SCATTER,
            REGION_ALL,
            BLEND_SCREEN,
            DOMAIN_AXIS_CYL,
            SCATTER_RAIN,
            0x8090a0,
            0x000000,
        ),
        lo(150, 50, 180, 0, 0, DIR_DOWN, 15, 0),
    ],
    // L6: PLANE/PAVEMENT - rain-slicked wet ground
    [
        hi_meta(
            OP_PLANE,
            REGION_FLOOR,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            PLANE_PAVEMENT,
            0x384048,
            0x303840,
        ),
        lo(140, 128, 0, 0, 0, DIR_UP, 15, 15),
    ],
    // L7: ATMOSPHERE/FULL - thick storm atmosphere (gray)
    [
        hi_meta(
            OP_ATMOSPHERE,
            REGION_ALL,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            ATMO_FULL,
            0x303038,
            0x000000,
        ),
        lo(120, 180, 0, 0, 0, DIR_UP, 15, 0),
    ],
];

// -----------------------------------------------------------------------------
// Preset 11: "Crystal Cavern" — Fantasy underground geode
// -----------------------------------------------------------------------------
// L0: RAMP (sky=#100020, floor=#080010, walls=#180030)
// L1: CELL/VORONOI (crystal purple #400080 / #200040, walls)
// L2: PATCHES/DEBRIS (amethyst #6020a0 / #300060, floor)
// L3: TRACE/FILAMENTS (cyan #00e0ff, walls, TANGENT_LOCAL)
// L4: SCATTER/DUST (white #ffffff, crystal sparkles)
// L5: LOBE (purple #a040ff, glow from below)
// L6: PORTAL/CIRCLE (cyan #00ffff / #200040, floor, TANGENT_LOCAL)
// L7: ATMOSPHERE/ABSORPTION (purple mist #200040)
pub(super) const PRESET_CRYSTAL_CAVERN: [[u64; 2]; 8] = [
    // L0: RAMP - deep purple sky, dark floor, violet walls
    [
        hi(OP_RAMP, REGION_ALL, BLEND_LERP, 0, 0x100020, 0x080010),
        lo(180, 0x18, 0x00, 0x30, THRESH_INTERIOR, DIR_UP, 15, 15),
    ],
    // L1: CELL/VORONOI - crystalline structure pattern (purple)
    [
        hi_meta(
            OP_CELL,
            REGION_WALLS,
            BLEND_ADD,
            DOMAIN_DIRECT3D,
            CELL_VORONOI,
            0x4020a0,
            0x400080,
        ),
        lo(100, 128, 160, 50, 0, DIR_UP, 15, 15),
    ],
    // L2: PATCHES/DEBRIS - scattered crystal formations on floor (amethyst)
    [
        hi_meta(
            OP_PATCHES,
            REGION_FLOOR,
            BLEND_SCREEN,
            DOMAIN_DIRECT3D,
            PATCHES_DEBRIS,
            0x6020a0,
            0x300060,
        ),
        lo(60, 128, 64, 0, 0, DIR_UP, 15, 15),
    ],
    // L3: TRACE/FILAMENTS - energy veins in crystal walls (cyan)
    [
        hi_meta(
            OP_TRACE,
            REGION_WALLS,
            BLEND_ADD,
            DOMAIN_TANGENT_LOCAL,
            TRACE_FILAMENTS,
            0x00e0ff,
            0x000000,
        ),
        lo(120, 128, 0, 0, 0, DIR_UP, 15, 0),
    ],
    // L4: SCATTER/DUST - glinting crystal facets (white sparks)
    [
        hi_meta(
            OP_SCATTER,
            REGION_ALL,
            BLEND_SCREEN,
            DOMAIN_DIRECT3D,
            SCATTER_DUST,
            0xffffff,
            0x000000,
        ),
        lo(50, 30, 0, 0x60, 0, DIR_UP, 15, 0),
    ],
    // L5: LOBE - ambient crystal glow from below (purple)
    [
        hi(OP_LOBE, REGION_ALL, BLEND_ADD, 0, 0xa040ff, 0x000000),
        lo(50, 128, 0, 1, 2, DIR_DOWN, 15, 0),
    ],
    // L6: PORTAL/CIRCLE - magic circle on floor (cyan/purple)
    [
        hi_meta(
            OP_PORTAL,
            REGION_FLOOR,
            BLEND_ADD,
            DOMAIN_TANGENT_LOCAL,
            PORTAL_CIRCLE,
            0x00ffff,
            0x200040,
        ),
        lo(80, 80, 0, 0, 0, DIR_UP, 15, 15),
    ],
    // L7: ATMOSPHERE/ABSORPTION - purple cave mist
    [
        hi_meta(
            OP_ATMOSPHERE,
            REGION_ALL,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            ATMO_ABSORPTION,
            0x200040,
            0x000000,
        ),
        lo(140, 80, 0, 0, 0, DIR_UP, 15, 0),
    ],
];

// -----------------------------------------------------------------------------
// Preset 12: "War Zone" — Military/apocalyptic battlefield
// -----------------------------------------------------------------------------
// L0: RAMP (sky=#383030, floor=#282020, walls=#302820)
// L1: SILHOUETTE/RUINS (dark #201810 / #302820, ruined buildings)
// L2: APERTURE/IRREGULAR (sky, broken roof opening #201810 / #383030)
// L3: PLANE/GRATING (industrial floor #484040 / #302820)
// L4: SCATTER/EMBERS (orange #ff6600, floating ash)
// L5: FLOW (gray #606060, smoke trails)
// L6: ATMOSPHERE/ABSORPTION (war smoke #302820)
// L7: DECAL (walls, burning fire #ff4400 / #200800)
pub(super) const PRESET_WAR_ZONE: [[u64; 2]; 8] = [
    // L0: RAMP - smoke gray sky, rubble floor, charred walls
    [
        hi(OP_RAMP, REGION_ALL, BLEND_LERP, 0, 0x181010, 0x282020),
        lo(220, 0x30, 0x28, 0x20, THRESH_SEMI, DIR_UP, 15, 15),
    ],
    // L1: SILHOUETTE/RUINS - destroyed building silhouettes
    [
        hi_meta(
            OP_SILHOUETTE,
            REGION_WALLS,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            SILHOUETTE_RUINS,
            0x080400,
            0x100800,
        ),
        lo(180, 128, 0, 0, 0, DIR_UP, 15, 15),
    ],
    // L2: APERTURE/IRREGULAR - broken roof opening in sky
    [
        hi_meta(
            OP_APERTURE,
            REGION_SKY,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            APERTURE_IRREGULAR,
            0x201810,
            0x383030,
        ),
        lo(200, 128, 0, 0, 0, DIR_UP, 15, 15),
    ],
    // L3: PLANE/GRATING - industrial floor grating (gray metal)
    [
        hi_meta(
            OP_PLANE,
            REGION_FLOOR,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            PLANE_GRATING,
            0x484040,
            0x302820,
        ),
        lo(140, 128, 0, 0, 0, DIR_UP, 15, 15),
    ],
    // L4: SCATTER/EMBERS - floating ash and embers (orange)
    [
        hi_meta(
            OP_SCATTER,
            REGION_ALL,
            BLEND_ADD,
            DOMAIN_DIRECT3D,
            SCATTER_EMBERS,
            0xff6600,
            0x000000,
        ),
        lo(180, 25, 20, 0x30, 0, DIR_UP, 15, 0),
    ],
    // L5: FLOW - smoke trails (gray)
    [
        hi(OP_FLOW, REGION_SKY, BLEND_ADD, 0, 0x606060, 0x000000),
        lo(160, 128, 0, 0x11, 80, DIR_UP, 15, 0),
    ],
    // L6: ATMOSPHERE/ABSORPTION - thick war smoke (brown-gray)
    [
        hi_meta(
            OP_ATMOSPHERE,
            REGION_ALL,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            ATMO_ABSORPTION,
            0x302820,
            0x000000,
        ),
        lo(140, 90, 0, 0, 0, DIR_UP, 15, 0),
    ],
    // L7: DECAL - burning fire spot on walls
    [
        hi(OP_DECAL, REGION_WALLS, BLEND_ADD, 0, 0xff6600, 0x200800),
        lo(255, 8, 96, 0, 3, DIR_UP, 15, 15), // shape=DISK(0), soft=8, size=96
    ],
];
