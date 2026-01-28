//! Preset set 09-12

#[allow(unused_imports)]
use crate::constants::*;

// -----------------------------------------------------------------------------
// Preset 9: "Neon Arcade" — Gamer room / RGB den
// -----------------------------------------------------------------------------
// Goal: clearly "interior gamer room" (monitor glow + RGB lighting), not a city.
//
// L0: RAMP            ALL   LERP  dark room base
// L1: SECTOR/BOX      ALL   LERP  room enclosure
// L2: APERTURE/RECT   WALLS LERP  big monitor/window (front)
// L3: PLANE/TILES     FLOOR LERP  dark floor
// L4: GRID            WALLS ADD   subtle wall seams
// L5: DECAL           WALLS ADD   RGB ring light (side)
// L6: LOBE            ALL   ADD   monitor light spill (directional)
// L7: SCATTER/DUST    ALL   ADD   faint room dust motes
pub(super) const PRESET_NEON_ARCADE: [[u64; 2]; 8] = [
    // L0: RAMP - dark room base (neutral/blue)
    [
        hi(OP_RAMP, REGION_ALL, BLEND_LERP, 0, 0x040408, 0x060608),
        lo(220, 0x0c, 0x0c, 0x10, THRESH_INTERIOR, DIR_UP, 15, 15),
    ],
    // L1: SECTOR/BOX - room enclosure (stronger to feel like walls/floor/ceiling)
    [
        hi_meta(
            OP_SECTOR,
            REGION_ALL,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            SECTOR_BOX,
            0x101820,
            0x040608,
        ),
        lo(255, 100, 0, 0, 0, DIR_UP, 15, 15),
    ],
    // L2: APERTURE/RECT - main monitor/window in front (smaller, like a TV screen)
    [
        hi_meta(
            OP_APERTURE,
            REGION_WALLS,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            APERTURE_RECT,
            0x00c0ff,
            0x101018,
        ),
        // softness, half_w, half_h, frame_thickness (reduced from 140x100 to 60x40)
        lo(160, 60, 40, 30, 0, DIR_FORWARD, 0, 0),
    ],
    // L3: PLANE/TILES - dark floor
    [
        hi_meta(
            OP_PLANE,
            REGION_FLOOR,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            PLANE_TILES,
            0x0a0a0c,
            0x040406,
        ),
        lo(160, 128, 0, 0, 0, DIR_UP, 15, 15),
    ],
    // L4: GRID - subtle wall panel seams
    [
        hi(OP_GRID, REGION_WALLS, BLEND_ADD, 0, 0x202030, 0x000000),
        lo(80, 64, 20, 0x10, 0, DIR_UP, 10, 0),
    ],
    // L5: DECAL - RGB ring light (side)
    [
        hi(OP_DECAL, REGION_WALLS, BLEND_ADD, 0, 0xff40ff, 0x00c0ff),
        // shape=RING(1), soft=6, size=60, glow_soft=140
        lo(180, 0x16, 60, 140, 0, DIR_RIGHT, 12, 12),
    ],
    // L6: LOBE - monitor glow spill (sharp, directional)
    [
        hi(OP_LOBE, REGION_ALL, BLEND_ADD, 0, 0x00c0ff, 0x202040),
        lo(140, 220, 80, 0, 0, DIR_FORWARD, 10, 0),
    ],
    // L7: SCATTER/DUST - faint room dust motes
    [
        hi_meta(
            OP_SCATTER,
            REGION_ALL,
            BLEND_ADD,
            DOMAIN_DIRECT3D,
            SCATTER_DUST,
            0x80a0ff,
            0x000000,
        ),
        lo(12, 12, 10, 0x10, 9, DIR_UP, 6, 0),
    ],
];

// -----------------------------------------------------------------------------
// Preset 10: "Storm Front" — Dramatic thunderstorm
// -----------------------------------------------------------------------------
// L0: RAMP (sky=#202830, floor=#181820, walls=#303840)
// L1: SILHOUETTE/WAVES (dark #181820 / #282830, distant storm clouds)
// L2: PATCHES/MEMBRANE (storm cloud masses)
// L3: TRACE/LIGHTNING (white #ffffff, sky - HERO ELEMENT)
// L4: VEIL/RAIN_WALL (blue-gray rain streaks)
// L5: LOBE (lightning flash fill)
// L6: PLANE/PAVEMENT (wet gray ground)
// L7: ATMOSPHERE/FULL (storm gray)
pub(super) const PRESET_STORM_FRONT: [[u64; 2]; 8] = [
    // L0: RAMP - dark stormy gray with visible differentiation
    [
        hi(OP_RAMP, REGION_ALL, BLEND_LERP, 0, 0x182028, 0x283038),
        lo(220, 0x38, 0x40, 0x48, THRESH_OPEN, DIR_UP, 15, 15),
    ],
    // L1: SILHOUETTE/WAVES - rolling storm cloud banks (darker for contrast)
    [
        hi_meta(
            OP_SILHOUETTE,
            REGION_WALLS,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            SILHOUETTE_WAVES,
            0x101418,
            0x282830,
        ),
        lo(180, 140, 180, 0x30, 0, DIR_UP, 15, 15),
    ],
    // L2: PATCHES/MEMBRANE - storm cloud masses (more visible)
    [
        hi_meta(
            OP_PATCHES,
            REGION_SKY,
            BLEND_SCREEN,
            DOMAIN_DIRECT3D,
            PATCHES_MEMBRANE,
            0x202838,
            0x384858,
        ),
        lo(120, 140, 100, 0, 0, DIR_UP, 12, 12),
    ],
    // L3: TRACE/LIGHTNING - bright lightning bolt (contained to sky region)
    [
        hi_meta(
            OP_TRACE,
            REGION_SKY,
            BLEND_ADD,
            DOMAIN_TANGENT_LOCAL,
            TRACE_LIGHTNING,
            0xffffff,
            0x80c0ff,
        ),
        // High intensity but contained
        lo(220, 200, 160, 200, 0x2E, DIR_FORWARD, 15, 12),
    ],
    // L4: VEIL/RAIN_WALL - rain streaks (tangent-local, visible)
    [
        hi_meta(
            OP_VEIL,
            REGION_SKY | REGION_WALLS,
            BLEND_SCREEN,
            DOMAIN_TANGENT_LOCAL,
            VEIL_RAIN_WALL,
            0xa0b8d0,
            0x506070,
        ),
        lo(200, 240, 60, 140, 64, DIR_FORWARD, 14, 8),
    ],
    // L5: LOBE - subtle lightning flash fill (add to sky only, not full wash)
    [
        hi(OP_LOBE, REGION_SKY, BLEND_ADD, 0, 0xa0b0c0, 0x000000),
        lo(40, 200, 100, 1, 0, DIR_FORWARD, 10, 0),
    ],
    // L6: PLANE/PAVEMENT - rain-slicked wet ground (more visible)
    [
        hi_meta(
            OP_PLANE,
            REGION_FLOOR,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            PLANE_PAVEMENT,
            0x404850,
            0x383840,
        ),
        lo(160, 128, 0, 0, 0, DIR_UP, 15, 15),
    ],
    // L7: ATMOSPHERE/FULL - thinner fog (preserve lightning visibility)
    [
        hi_meta(
            OP_ATMOSPHERE,
            REGION_ALL,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            ATMO_FULL,
            0x283038,
            0x101820,
        ),
        lo(40, 90, 128, 0, 0, DIR_UP, 10, 0),
    ],
];

// -----------------------------------------------------------------------------
// Preset 11: "Crystal Cavern" — Fantasy underground geode
// -----------------------------------------------------------------------------
// L0: RAMP (sky=#100020, floor=#080010, walls=#180030)
// L1: SECTOR/BOX - cave enclosure (provides structure)
// L2: CELL/VORONOI (crystal purple #400080 / #200040, walls)
// L3: TRACE/FILAMENTS (cyan #00e0ff, walls, TANGENT_LOCAL)
// L4: SCATTER/DUST (white #ffffff, crystal sparkles)
// L5: LOBE (purple #a040ff, glow from below)
// L6: PORTAL/CIRCLE (cyan #00ffff / #200040, floor, TANGENT_LOCAL)
// L7: ATMOSPHERE/ABSORPTION (purple mist #200040)
pub(super) const PRESET_CRYSTAL_CAVERN: [[u64; 2]; 8] = [
    // L0: RAMP - near-black cave everywhere (features must pop from darkness)
    [
        hi(OP_RAMP, REGION_ALL, BLEND_LERP, 0, 0x020008, 0x020004),
        lo(255, 0x04, 0x00, 0x08, THRESH_INTERIOR, DIR_UP, 15, 15),
    ],
    // L1: SECTOR/BOX - cave enclosure (strong walls/ceiling/floor)
    [
        hi_meta(
            OP_SECTOR,
            REGION_ALL,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            SECTOR_BOX,
            0x100828,
            0x040210,
        ),
        lo(220, 90, 0, 0, 0, DIR_UP, 15, 15),
    ],
    // L2: CELL/VORONOI - crystalline facets on walls (keep dark; cyan features pop)
    [
        hi_meta(
            OP_CELL,
            REGION_WALLS,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            CELL_VORONOI,
            0x080420,
            0x180848,
        ),
        lo(120, 128, 200, 30, 7, DIR_UP, 10, 8),
    ],
    // L3: TRACE/FILAMENTS - glowing energy veins in crystal walls (cyan, controlled)
    [
        hi_meta(
            OP_TRACE,
            REGION_WALLS,
            BLEND_ADD,
            DOMAIN_AXIS_CYL,
            TRACE_FILAMENTS,
            0x00ffff,
            0x00a0ff,
        ),
        lo(180, 120, 50, 160, 0x7A, DIR_UP, 15, 10),
    ],
    // L4: SCATTER/DUST - glinting crystal facets (very sparse; avoid "snow")
    [
        hi_meta(
            OP_SCATTER,
            REGION_ALL,
            BLEND_ADD,
            DOMAIN_DIRECT3D,
            SCATTER_DUST,
            0xa0c0ff,
            0xffffff,
        ),
        lo(14, 16, 10, 0x10, 11, DIR_UP, 5, 0),
    ],
    // L5: LOBE - ambient crystal glow from below (subtle)
    [
        hi(OP_LOBE, REGION_ALL, BLEND_ADD, 0, 0x401080, 0x000000),
        lo(20, 140, 0, 1, 2, DIR_DOWN, 12, 0),
    ],
    // L6: PORTAL/CIRCLE - magic circle on floor (cyan ring - hero element)
    [
        hi_meta(
            OP_PORTAL,
            REGION_FLOOR,
            BLEND_ADD,
            DOMAIN_TANGENT_LOCAL,
            PORTAL_CIRCLE,
            0x000008,
            0x00ffff,
        ),
        // intensity, size, edge_width, roughness, phase
        lo(200, 100, 140, 0, 0, DIR_DOWN, 0, 15),
    ],
    // L7: ATMOSPHERE/ABSORPTION - purple cave mist (adds depth)
    [
        hi_meta(
            OP_ATMOSPHERE,
            REGION_ALL,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            ATMO_ABSORPTION,
            0x100420,
            0x000000,
        ),
        lo(80, 110, 0, 0, 0, DIR_UP, 12, 0),
    ],
];

// -----------------------------------------------------------------------------
// Preset 12: "War Zone" — Military/apocalyptic battlefield
// -----------------------------------------------------------------------------
// Design: Ruined urban battlefield with fires, smoke. SECTOR for structure.
// NO harsh silhouette line - use SECTOR/BOX for enclosed bombed-out building.
// L0: SECTOR/BOX - ruined building enclosure (structure!)
// L1: PLANE/TILES - rubble floor
// L2: TRACE/CRACKS - structural damage cracks
// L3: APERTURE/IRREGULAR - bombed-out roof/wall holes
// L4: SCATTER/EMBERS - floating ash and sparks
// L5: DECAL - burning fires
// L6: FLOW - smoke trails
// L7: ATMOSPHERE/ABSORPTION - war smoke
pub(super) const PRESET_WAR_ZONE: [[u64; 2]; 8] = [
    // L0: SECTOR/BOX - ruined building enclosure as BASE (structure!)
    [
        hi_meta(
            OP_SECTOR,
            REGION_ALL,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            SECTOR_BOX,
            0x181410,
            0x302820,
        ),
        lo(255, 140, 0, 0, 0, DIR_UP, 15, 15),
    ],
    // L1: PLANE/TILES - rubble/debris floor
    [
        hi_meta(
            OP_PLANE,
            REGION_FLOOR,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            PLANE_TILES,
            0x383028,
            0x282018,
        ),
        lo(200, 100, 0, 0, 0, DIR_UP, 15, 15),
    ],
    // L2: TRACE/CRACKS - structural damage in walls
    [
        hi_meta(
            OP_TRACE,
            REGION_WALLS,
            BLEND_LERP,
            DOMAIN_TANGENT_LOCAL,
            TRACE_CRACKS,
            0x101008,
            0x201810,
        ),
        lo(150, 100, 80, 60, 0, DIR_UP, 12, 10),
    ],
    // L3: APERTURE/IRREGULAR - bombed-out holes showing smoky sky
    [
        hi_meta(
            OP_APERTURE,
            REGION_SKY,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            APERTURE_IRREGULAR,
            0x282018,
            0x504840,
        ),
        lo(200, 100, 80, 60, 0, DIR_UP, 15, 15),
    ],
    // L4: SCATTER/EMBERS - floating ash and embers (keep readable; avoid full-screen bokeh)
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
        lo(120, 12, 18, 0x20, 9, DIR_UP, 10, 0),
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
