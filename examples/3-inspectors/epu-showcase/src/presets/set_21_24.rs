//! Preset set 21-24

#[allow(unused_imports)]
use crate::constants::*;

// -----------------------------------------------------------------------------
// Preset 21: "Stormy Shores" - Coastal cliffs, crashing waves
// -----------------------------------------------------------------------------
// Goal: Dramatic coastal storm with visible lightning and lighthouse beam.
// Rain should fall evenly across the scene using DIRECT3D domain.
// Lightning needs animation speed in ANIM_SPEEDS[20] for L5.
//
// L0: RAMP - dark stormy blue-gray
// L1: SILHOUETTE/WAVES - wave patterns on walls
// L2: PLANE/WATER - churning ocean surface
// L3: FLOW - sea foam spray (animated)
// L4: VEIL/RAIN_WALL - driving rain (DIRECT3D for even coverage)
// L5: DECAL - lightning flash (uses param_d for flicker animation)
// L6: ATMOSPHERE/FULL - coastal fog
// L7: LOBE - lighthouse beam sweep
pub(super) const PRESET_STORMY_SHORES: [[u64; 2]; 8] = [
    // L0: RAMP - dark stormy blue-gray with visible differentiation
    [
        hi(OP_RAMP, REGION_ALL, BLEND_LERP, 0, 0x1a2028, 0x283848),
        lo(200, 0x38, 0x40, 0x50, THRESH_OPEN, DIR_UP, 15, 15),
    ],
    // L1: SILHOUETTE/WAVES - crashing waves on walls (darker for contrast)
    [
        hi_meta(
            OP_SILHOUETTE,
            REGION_WALLS,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            SILHOUETTE_WAVES,
            0x081018,
            0x182028,
        ),
        lo(200, 180, 200, 0x30, 0, DIR_UP, 15, 15),
    ],
    // L2: PLANE/WATER - churning ocean surface (more blue)
    [
        hi_meta(
            OP_PLANE,
            REGION_FLOOR,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            PLANE_WATER,
            0x305070,
            0x183050,
        ),
        lo(160, 128, 80, 0, 0, DIR_UP, 15, 15),
    ],
    // L3: FLOW - sea foam and spray (white-ish, visible)
    [
        hi(
            OP_FLOW,
            REGION_FLOOR | REGION_WALLS,
            BLEND_ADD,
            0,
            0x90a8c0,
            0x405060,
        ),
        lo(80, 120, 60, 0x21, 0, DIR_FORWARD, 10, 0),
    ],
    // L4: VEIL/RAIN_WALL - driving rain (DIRECT3D for even coverage, DIR_DOWN)
    [
        hi_meta(
            OP_VEIL,
            REGION_ALL,
            BLEND_SCREEN,
            DOMAIN_DIRECT3D,
            VEIL_RAIN_WALL,
            0xa0b8d0,
            0x405060,
        ),
        lo(180, 200, 40, 140, 60, DIR_DOWN, 12, 8),
    ],
    // L5: DECAL - lightning flash (param_d animates for flicker effect)
    [
        hi(OP_DECAL, REGION_SKY, BLEND_ADD, 0, 0xffffff, 0x80c0ff),
        // shape=0 (point flash), large soft spread, param_d for animation
        lo(255, 0x00, 200, 180, 0, DIR_UP, 15, 12),
    ],
    // L6: ATMOSPHERE/FULL - coastal storm fog (moderate, preserve detail)
    [
        hi_meta(
            OP_ATMOSPHERE,
            REGION_ALL,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            ATMO_FULL,
            0x384858,
            0x202838,
        ),
        lo(50, 100, 128, 0, 0, DIR_UP, 10, 0),
    ],
    // L7: LOBE - lighthouse beam (focused, not overpowering)
    [
        hi(OP_LOBE, REGION_ALL, BLEND_ADD, 0, 0xffffd0, 0x000000),
        // Moderate intensity (180), tight beam (240 param_a), sharp (30 param_b)
        lo(180, 240, 30, 1, 0, DIR_SUNSET, 12, 0),
    ],
];

// -----------------------------------------------------------------------------
// Preset 22: "Polar Aurora" - Arctic night with northern lights
// -----------------------------------------------------------------------------
// Design: Arctic night with vivid aurora borealis. No grid pattern on floor,
// use natural ice/snow textures instead. Aurora should animate via ANIM_SPEEDS.
//
// L0: RAMP (dark night sky, snow floor, ice walls)
// L1: PLANE/STONE (snow/ice ground - natural texture, not grid)
// L2: BAND (aurora horizon band - animated)
// L3: VEIL/CURTAINS (aurora curtains - animated)
// L4: SCATTER/STARS (night starfield)
// L5: CELESTIAL/MOON (bright arctic moon)
// L6: FLOW (subtle aurora shimmer)
// L7: ATMOSPHERE/RAYLEIGH (crisp arctic air)
pub(super) const PRESET_POLAR_AURORA: [[u64; 2]; 8] = [
    // L0: RAMP - dark arctic night sky, snow floor, ice walls
    [
        hi(OP_RAMP, REGION_ALL, BLEND_LERP, 0, 0x000010, 0x203040),
        lo(220, 0x18, 0x20, 0x30, THRESH_OPEN, DIR_UP, 15, 15),
    ],
    // L1: PLANE/STONE - snow and ice ground (natural texture, no grid)
    [
        hi_meta(
            OP_PLANE,
            REGION_FLOOR,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            PLANE_STONE,
            0xa0b0c0,
            0x708090,
        ),
        lo(180, 128, 0, 0, 0, DIR_UP, 15, 15),
    ],
    // L2: BAND - aurora horizon band (green/cyan, animated via ANIM_SPEEDS)
    [
        hi(OP_BAND, REGION_SKY, BLEND_ADD, 0, 0x00ff80, 0x00ffff),
        lo(200, 80, 128, 180, 0, DIR_UP, 12, 0),
    ],
    // L3: VEIL/CURTAINS - aurora curtains (animated via ANIM_SPEEDS)
    [
        hi_meta(
            OP_VEIL,
            REGION_SKY,
            BLEND_ADD,
            DOMAIN_AXIS_CYL,
            VEIL_CURTAINS,
            0x40ff80,
            0x20ff60,
        ),
        lo(180, 160, 80, 120, 0, DIR_UP, 12, 8),
    ],
    // L4: SCATTER/STARS - night starfield
    [
        hi_meta(
            OP_SCATTER,
            REGION_SKY,
            BLEND_ADD,
            DOMAIN_DIRECT3D,
            SCATTER_STARS,
            0xffffff,
            0x000000,
        ),
        lo(150, 60, 0, 0x10, 0, DIR_UP, 15, 0),
    ],
    // L5: CELESTIAL/MOON - bright arctic moon
    [
        hi_meta(
            OP_CELESTIAL,
            REGION_SKY,
            BLEND_ADD,
            DOMAIN_DIRECT3D,
            CELESTIAL_MOON,
            0xf0f8ff,
            0x000000,
        ),
        lo(160, 128, 0, 0, 0, DIR_SUN, 15, 0),
    ],
    // L6: FLOW - subtle aurora shimmer on sky (adds motion)
    [
        hi(OP_FLOW, REGION_SKY, BLEND_ADD, 0, 0x20ff60, 0x000000),
        lo(40, 100, 60, 0x21, 0, DIR_UP, 8, 0),
    ],
    // L7: ATMOSPHERE/RAYLEIGH - crisp cold arctic air
    [
        hi_meta(
            OP_ATMOSPHERE,
            REGION_ALL,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            ATMO_RAYLEIGH,
            0x102030,
            0x000000,
        ),
        lo(30, 100, 0, 0, 0, DIR_UP, 15, 0),
    ],
];

// -----------------------------------------------------------------------------
// Preset 23: "Sacred Geometry" - Abstract mathematical temple
// -----------------------------------------------------------------------------
// Goal: Purple/gold temple with visible geometric patterns and divine light.
// Floor uses DIRECT3D domain to avoid spherical distortion.
//
// L0: RAMP - deep indigo/purple base
// L1: SPLIT/PRISM - prismatic wall divisions
// L2: CELL/GRID - geometric floor tiles (DIRECT3D - flat, not spherical)
// L3: GRID - gold sacred frame lines (HERO)
// L4: TRACE/FILAMENTS - radial gold energy lines
// L5: APERTURE/CIRCLE - central sacred opening
// L6: LOBE - HERO: divine central light beam
// L7: SCATTER/DUST - golden sacred particles
pub(super) const PRESET_SACRED_GEOMETRY: [[u64; 2]; 8] = [
    // L0: RAMP - purple/gold temple base
    [
        hi(OP_RAMP, REGION_ALL, BLEND_LERP, 0, 0x281040, 0x604020),
        lo(160, 0x50, 0x30, 0x60, THRESH_SEMI, DIR_UP, 15, 15),
    ],
    // L1: SECTOR/BOX - temple enclosure structure
    [
        hi_meta(
            OP_SECTOR,
            REGION_ALL,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            SECTOR_BOX,
            0x402050,
            0x281030,
        ),
        lo(180, 128, 0, 0, 0, DIR_UP, 15, 15),
    ],
    // L2: PLANE/TILES - bronze floor tiles
    [
        hi_meta(
            OP_PLANE,
            REGION_FLOOR,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            PLANE_TILES,
            0x705030,
            0x503018,
        ),
        lo(200, 100, 0, 0, 0, DIR_UP, 15, 15),
    ],
    // L3: GRID - HERO: bright gold sacred frame (very visible)
    [
        hi(OP_GRID, REGION_ALL, BLEND_ADD, 0, 0xffc060, 0xffa040),
        lo(255, 40, 50, 0, 0, DIR_UP, 15, 15),
    ],
    // L4: TRACE/FILAMENTS - radial energy lines (bright gold, walls only)
    [
        hi_meta(
            OP_TRACE,
            REGION_WALLS,
            BLEND_ADD,
            DOMAIN_TANGENT_LOCAL,
            TRACE_FILAMENTS,
            0xffa040,
            0x804020,
        ),
        lo(120, 140, 60, 120, 0, DIR_UP, 15, 8),
    ],
    // L5: APERTURE/CIRCLE - central sacred opening (visible vignette)
    [
        hi_meta(
            OP_APERTURE,
            REGION_ALL,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            APERTURE_CIRCLE,
            0x080410,
            0x301830,
        ),
        lo(140, 128, 0, 0, 0, DIR_UP, 15, 15),
    ],
    // L6: LOBE - HERO: divine central light beam (bright gold, strong)
    [
        hi(OP_LOBE, REGION_ALL, BLEND_ADD, 0, 0xffd080, 0x804020),
        lo(180, 200, 80, 1, 2, DIR_DOWN, 15, 8),
    ],
    // L7: SCATTER/DUST - golden sacred particles (more visible)
    [
        hi_meta(
            OP_SCATTER,
            REGION_ALL,
            BLEND_ADD,
            DOMAIN_DIRECT3D,
            SCATTER_DUST,
            0xffc060,
            0xffa040,
        ),
        lo(40, 60, 30, 0x30, 0, DIR_UP, 12, 0),
    ],
];

// -----------------------------------------------------------------------------
// Preset 24: "Ritual Chamber" - Dark magic summoning room
// -----------------------------------------------------------------------------
// Design: Dark chamber with visible wall stone texture and focused floor ritual.
// Walls need more detail (CELL/VORONOI stronger), floor detail simplified.
//
// L0: RAMP (void black sky, obsidian floor, dark stone walls)
// L1: CELL/VORONOI (rough stone walls - MORE visible, key detail)
// L2: TRACE/CRACKS (arcane veins on walls - adds wall interest)
// L3: DECAL (magic pentagram on floor)
// L4: PORTAL/VORTEX (summoning portal)
// L5: APERTURE/CIRCLE (circular chamber opening)
// L6: LOBE (portal glow)
// L7: ATMOSPHERE/ALIEN (otherworldly atmosphere)
pub(super) const PRESET_RITUAL_CHAMBER: [[u64; 2]; 8] = [
    // L0: RAMP - void black sky, obsidian floor, dark stone walls
    [
        hi(OP_RAMP, REGION_ALL, BLEND_LERP, 0, 0x000004, 0x100808),
        lo(180, 0x20, 0x14, 0x20, THRESH_INTERIOR, DIR_UP, 15, 15),
    ],
    // L1: CELL/VORONOI - rough stone walls (moderate intensity, larger cells)
    [
        hi_meta(
            OP_CELL,
            REGION_WALLS,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            CELL_VORONOI,
            0x281420,
            0x140a10,
        ),
        // Moderate intensity, large param_a for bigger stones, higher gap
        lo(100, 120, 180, 80, 0, DIR_UP, 12, 10),
    ],
    // L2: TRACE/CRACKS - arcane veins on walls (adds visual interest to walls)
    [
        hi_meta(
            OP_TRACE,
            REGION_WALLS,
            BLEND_ADD,
            DOMAIN_TANGENT_LOCAL,
            TRACE_CRACKS,
            0x8020ff,
            0x400080,
        ),
        lo(80, 120, 50, 80, 0, DIR_UP, 12, 6),
    ],
    // L3: DECAL - ritual ring on floor (direction must be DOWN)
    [
        hi(OP_DECAL, REGION_FLOOR, BLEND_ADD, 0, 0xff2000, 0x8020ff),
        // shape=RING(1), soft=6, size=70, glow_soft=140
        lo(200, 0x16, 70, 140, 0, DIR_DOWN, 12, 12),
    ],
    // L4: PORTAL/VORTEX - summoning portal (direction must be DOWN)
    [
        hi_meta(
            OP_PORTAL,
            REGION_FLOOR,
            BLEND_ADD,
            DOMAIN_TANGENT_LOCAL,
            PORTAL_VORTEX,
            0x000000,
            0xa040ff,
        ),
        lo(200, 90, 180, 120, 0, DIR_DOWN, 15, 15),
    ],
    // L5: APERTURE/CIRCLE - circular chamber opening (bound)
    [
        hi_meta(
            OP_APERTURE,
            REGION_SKY,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            APERTURE_CIRCLE,
            0x080408,
            0x180818,
        ),
        lo(160, 128, 0, 0, 0, DIR_UP, 15, 15),
    ],
    // L6: LOBE - portal glow from the ritual circle
    [
        hi(OP_LOBE, REGION_ALL, BLEND_ADD, 0, 0xa040ff, 0x000000),
        lo(140, 200, 60, 1, 0, DIR_DOWN, 12, 0),
    ],
    // L7: ATMOSPHERE/ALIEN - otherworldly oppressive atmosphere
    [
        hi_meta(
            OP_ATMOSPHERE,
            REGION_ALL,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            ATMO_ALIEN,
            0x100010,
            0x000000,
        ),
        lo(50, 80, 128, 0, 0, DIR_UP, 10, 0),
    ],
];
