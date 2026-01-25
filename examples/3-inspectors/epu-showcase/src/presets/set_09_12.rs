//! Preset set 09-12

#[allow(unused_imports)]
use crate::constants::*;

// -----------------------------------------------------------------------------
// Preset 9: "Neon Arcade" - Retro synthwave
// -----------------------------------------------------------------------------
// L0: RAMP (sky=#000010, floor=#100020, walls=#080018)
// L1: GRID (magenta #ff00ff, floor)
// L2: GRID (cyan #00ffff, walls)
// L3: SPLIT/BANDS (neon colors, retro horizontal bands)
// L4: SCATTER (white #ffffff, starfield)
// L5: CELESTIAL/SUN (magenta #ff0088, retro sun)
// L6: BAND (cyan #00ffff, horizon glow)
// L7: FLOW (purple #8000ff, pulsing glow)
pub(super) const PRESET_NEON_ARCADE: [[u64; 2]; 8] = [
    // L0: RAMP - black sky, dark purple floor, dark blue walls
    [
        hi(OP_RAMP, REGION_ALL, BLEND_LERP, 0, 0x000010, 0x100020),
        lo(255, 0x08, 0x00, 0x18, 0, DIR_UP, 15, 15),
    ],
    // L1: GRID - magenta wireframe grid on floor (retro style)
    [
        hi(OP_GRID, REGION_FLOOR, BLEND_ADD, 0, 0xff00ff, 0x000000),
        lo(180, 48, 0, 4, 0, DIR_UP, 15, 0), // param_c=4: retro scroll
    ],
    // L2: SPLIT/BANDS - retro horizontal neon bands on walls
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
        lo(160, 128, 0, 0, 0, DIR_UP, 15, 15),
    ],
    // L3: SPLIT/BANDS - retro horizontal neon bands
    [
        hi_meta(
            OP_SPLIT,
            REGION_SKY,
            BLEND_ADD,
            DOMAIN_DIRECT3D,
            SPLIT_PRISM,
            0xff0088,
            0x00ffff,
        ),
        lo(140, 128, 0, 0, 0, DIR_UP, 15, 15),
    ],
    // L4: SCATTER - background starfield (white)
    [
        hi(OP_SCATTER, REGION_SKY, BLEND_ADD, 0, 0xffffff, 0x000000),
        lo(160, 150, 0, 0x10, 0, DIR_UP, 15, 0),
    ],
    // L5: CELESTIAL/PLANET - retro magenta planet on horizon
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
        lo(200, 128, 0, 0, 0, DIR_SUNSET, 15, 0),
    ],
    // L6: BAND - cyan horizon glow line
    [
        hi(OP_BAND, REGION_ALL, BLEND_ADD, 0, 0x00ffff, 0x000000),
        lo(180, 128, 0, 0, 0, DIR_SUNSET, 15, 0),
    ],
    // L7: FLOW - purple pulsing ambient glow
    [
        hi(OP_FLOW, REGION_ALL, BLEND_ADD, 0, 0x8000ff, 0x000000),
        lo(100, 128, 60, 0, 0, DIR_UP, 15, 0),
    ],
];

// -----------------------------------------------------------------------------
// Preset 10: "Storm Front" - Dramatic weather
// -----------------------------------------------------------------------------
// L0: RAMP (sky=#202830, floor=#181820, walls=#303840)
// L1: SPLIT/WEDGE (dark #181820 / light #404850, dramatic wedge)
// L2: FLOW (dark gray #404858, storm clouds)
// L3: TRACE/LIGHTNING (white #ffffff, sky, DOMAIN_AXIS_POLAR)
// L4: VEIL/RAIN_WALL (blue-gray #607080)
// L5: SCATTER (rain blue #8090a0, raindrops)
// L6: ATMOSPHERE/FULL (storm gray #303038)
// L7: PLANE/PAVEMENT (wet gray #282830)
pub(super) const PRESET_STORM_FRONT: [[u64; 2]; 8] = [
    // L0: RAMP - dark gray sky, wet ground, slate walls
    [
        hi(OP_RAMP, REGION_ALL, BLEND_LERP, 0, 0x202830, 0x181820),
        lo(255, 0x30, 0x38, 0x40, 0, DIR_UP, 15, 15),
    ],
    // L1: SPLIT/WEDGE - dramatic wedge-shaped sky division (storm front)
    [
        hi_meta(
            OP_SPLIT,
            REGION_SKY,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            SPLIT_CORNER,
            0x181820,
            0x404850,
        ),
        lo(160, 128, 0, 0, 0, DIR_UP, 15, 15),
    ],
    // L2: FLOW - churning storm clouds (dark gray)
    [
        hi(OP_FLOW, REGION_SKY, BLEND_ADD, 0, 0x404858, 0x000000),
        lo(140, 128, 120, 4, 0, DIR_UP, 15, 0),
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
        lo(200, 128, 0, 0, 0, DIR_UP, 15, 0),
    ],
    // L4: VEIL/RAIN_WALL - heavy rain curtains (blue-gray)
    [
        hi_meta(
            OP_VEIL,
            REGION_ALL,
            BLEND_ADD,
            DOMAIN_AXIS_CYL,
            VEIL_RAIN_WALL,
            0x607080,
            0x000000,
        ),
        lo(160, 128, 200, 0, 0, DIR_DOWN, 15, 0),
    ],
    // L5: SCATTER - raindrops (rain blue)
    [
        hi_meta(
            OP_SCATTER,
            REGION_ALL,
            BLEND_ADD,
            DOMAIN_AXIS_CYL,
            SCATTER_RAIN,
            0x8090a0,
            0x000000,
        ),
        lo(180, 140, 180, 0, 0, DIR_DOWN, 15, 0),
    ],
    // L6: ATMOSPHERE/FULL - thick storm atmosphere (gray)
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
        lo(160, 180, 0, 0, 0, DIR_UP, 15, 0),
    ],
    // L7: PLANE/PAVEMENT - rain-slicked ground (wet gray)
    [
        hi_meta(
            OP_PLANE,
            REGION_FLOOR,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            PLANE_PAVEMENT,
            0x282830,
            0x000000,
        ),
        lo(140, 128, 0, 0, 0, DIR_UP, 15, 0),
    ],
];

// -----------------------------------------------------------------------------
// Preset 11: "Crystal Cavern" - Fantasy underground
// -----------------------------------------------------------------------------
// L0: RAMP (sky=#100020, floor=#080010, walls=#180030)
// L1: CELL/VORONOI (crystal purple #400080)
// L2: PATCHES/DEBRIS (amethyst #6020a0)
// L3: TRACE/FILAMENTS (cyan #00e0ff)
// L4: SCATTER (white #ffffff, sparks)
// L5: LOBE (purple #a040ff, glow from below)
// L6: PORTAL/CIRCLE (magic circle cyan #00ffff)
// L7: ATMOSPHERE/ABSORPTION (purple mist #200040)
pub(super) const PRESET_CRYSTAL_CAVERN: [[u64; 2]; 8] = [
    // L0: RAMP - deep purple sky, dark floor, violet walls
    [
        hi(OP_RAMP, REGION_ALL, BLEND_LERP, 0, 0x100020, 0x080010),
        lo(255, 0x18, 0x00, 0x30, 0, DIR_UP, 15, 15),
    ],
    // L1: CELL/VORONOI - crystalline structure pattern (purple)
    [
        hi_meta(
            OP_CELL,
            REGION_ALL,
            BLEND_ADD,
            DOMAIN_DIRECT3D,
            CELL_VORONOI,
            0x400080,
            0x000000,
        ),
        lo(160, 128, 0, 0, 0, DIR_UP, 15, 0),
    ],
    // L2: PATCHES/DEBRIS - scattered crystal formations (amethyst)
    [
        hi_meta(
            OP_PATCHES,
            REGION_ALL,
            BLEND_ADD,
            DOMAIN_DIRECT3D,
            PATCHES_DEBRIS,
            0x6020a0,
            0x000000,
        ),
        lo(150, 128, 64, 0, 0, DIR_UP, 15, 0),
    ],
    // L3: TRACE/FILAMENTS - energy veins in crystals (cyan)
    [
        hi_meta(
            OP_TRACE,
            REGION_ALL,
            BLEND_ADD,
            DOMAIN_TANGENT_LOCAL,
            TRACE_FILAMENTS,
            0x00e0ff,
            0x000000,
        ),
        lo(180, 128, 0, 0, 0, DIR_UP, 15, 0),
    ],
    // L4: SCATTER - glinting crystal facets (white sparks)
    [
        hi(OP_SCATTER, REGION_ALL, BLEND_ADD, 0, 0xffffff, 0x000000),
        lo(170, 120, 0, 0x60, 0, DIR_UP, 15, 0),
    ],
    // L5: LOBE - ambient crystal glow from below (purple)
    [
        hi(OP_LOBE, REGION_ALL, BLEND_ADD, 0, 0xa040ff, 0x000000),
        lo(160, 128, 0, 0, 2, DIR_DOWN, 15, 0), // param_d=2: glow pulse
    ],
    // L6: PORTAL/CIRCLE - magic circle on floor (cyan)
    [
        hi_meta(
            OP_PORTAL,
            REGION_FLOOR,
            BLEND_ADD,
            DOMAIN_TANGENT_LOCAL,
            PORTAL_CIRCLE,
            0x00ffff,
            0x000000,
        ),
        lo(180, 80, 0, 0, 0, DIR_UP, 15, 0),
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
        lo(140, 160, 0, 0, 0, DIR_UP, 15, 0),
    ],
];

// -----------------------------------------------------------------------------
// Preset 12: "War Zone" - Military/apocalyptic
// -----------------------------------------------------------------------------
// L0: RAMP (sky=#383030, floor=#282020, walls=#302820)
// L1: SILHOUETTE/RUINS (black #000000)
// L2: PLANE/GRATING (dark gray #181818, industrial floor)
// L3: PATCHES/DEBRIS (brown #483828)
// L4: SCATTER (orange #ff6600, embers)
// L5: FLOW/STREAKS (gray #606060, smoke)
// L6: ATMOSPHERE/ABSORPTION (smoke #302820)
// L7: SECTOR/CAVE (BLEND_MAX, #000000/#302820)
pub(super) const PRESET_WAR_ZONE: [[u64; 2]; 8] = [
    // L0: RAMP - smoke gray sky, rubble floor, charred walls
    [
        hi(OP_RAMP, REGION_ALL, BLEND_LERP, 0, 0x383030, 0x282020),
        lo(255, 0x30, 0x28, 0x20, 0, DIR_UP, 15, 15),
    ],
    // L1: SILHOUETTE/RUINS - destroyed building silhouettes (black)
    [
        hi_meta(
            OP_SILHOUETTE,
            REGION_WALLS,
            BLEND_MULTIPLY,
            DOMAIN_DIRECT3D,
            SILHOUETTE_RUINS,
            0x000000,
            0x000000,
        ),
        lo(180, 128, 0, 0, 0, DIR_UP, 15, 0),
    ],
    // L2: PLANE/GRATING - industrial floor grating (gray metal)
    [
        hi_meta(
            OP_PLANE,
            REGION_FLOOR,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            PLANE_GRATING,
            0x484040,
            0x000000,
        ),
        lo(140, 128, 0, 0, 0, DIR_UP, 15, 0),
    ],
    // L3: PATCHES/DEBRIS - scattered rubble (brown, additive highlights)
    [
        hi_meta(
            OP_PATCHES,
            REGION_FLOOR,
            BLEND_ADD,
            DOMAIN_DIRECT3D,
            PATCHES_DEBRIS,
            0x302418,
            0x000000,
        ),
        lo(120, 128, 64, 0, 0, DIR_UP, 12, 0),
    ],
    // L4: SCATTER - floating ash and embers (orange)
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
        lo(180, 100, 80, 0x30, 0, DIR_UP, 15, 0),
    ],
    // L5: FLOW - smoke trails (gray)
    [
        hi(OP_FLOW, REGION_ALL, BLEND_ADD, 0, 0x606060, 0x000000),
        lo(120, 128, 80, 0, 0, DIR_UP, 15, 0),
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
        lo(160, 180, 0, 0, 0, DIR_UP, 15, 0),
    ],
    // L7: SECTOR/CAVE - underground bunker effect (max blend)
    [
        hi_meta(
            OP_SECTOR,
            REGION_ALL,
            BLEND_MAX,
            0,
            SECTOR_CAVE,
            0x000000,
            0x302820,
        ),
        lo(160, 128, 64, 0, 0, DIR_UP, 15, 15),
    ],
];

