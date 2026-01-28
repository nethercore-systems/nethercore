//! Preset set 21-24

#[allow(unused_imports)]
use crate::constants::*;

// -----------------------------------------------------------------------------
// Preset 21: "Stormy Shores" - Coastal cliffs, crashing waves
// -----------------------------------------------------------------------------
// Goal: Dramatic coastal storm with visible lightning and lighthouse beam.
//
// L0: RAMP - dark stormy blue-gray (more contrast)
// L1: SILHOUETTE/WAVES - wave patterns on walls
// L2: PLANE/WATER - churning ocean surface
// L3: FLOW - sea foam spray (animated)
// L4: VEIL/RAIN_WALL - driving rain
// L5: TRACE/LIGHTNING - HERO: bright lightning bolt
// L6: ATMOSPHERE/FULL - coastal fog (lighter)
// L7: LOBE - HERO: lighthouse beam sweep
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
        hi(OP_FLOW, REGION_FLOOR | REGION_WALLS, BLEND_ADD, 0, 0x90a8c0, 0x405060),
        lo(80, 120, 60, 0x21, 0, DIR_FORWARD, 10, 0),
    ],
    // L4: VEIL/RAIN_WALL - driving rain (tangent-local, visible)
    [
        hi_meta(
            OP_VEIL,
            REGION_SKY | REGION_WALLS,
            BLEND_SCREEN,
            DOMAIN_TANGENT_LOCAL,
            VEIL_RAIN_WALL,
            0xb0c8e0,
            0x506070,
        ),
        lo(220, 255, 50, 160, 80, DIR_FORWARD, 15, 10),
    ],
    // L5: TRACE/LIGHTNING - bright lightning bolt (contained to sky)
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
        lo(220, 200, 180, 200, 0x2E, DIR_FORWARD, 15, 12),
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
// L0: RAMP (dark night sky, snow floor, ice walls)
// L1: CELL/RADIAL (radial ice pattern on floor, AXIS_POLAR - bound)
// L2: BAND (aurora horizon band)
// L3: VEIL/CURTAINS (aurora curtains, AXIS_POLAR)
// L4: SCATTER/STARS (night starfield)
// L5: CELESTIAL/MOON (bright arctic moon, dir=SUN)
// L6: PLANE/STONE (snow/ice ground)
// L7: ATMOSPHERE/RAYLEIGH (crisp arctic air)
pub(super) const PRESET_POLAR_AURORA: [[u64; 2]; 8] = [
    // L0: RAMP - dark arctic night sky, snow floor, ice walls
    [
        hi(OP_RAMP, REGION_ALL, BLEND_LERP, 0, 0x000008, 0x182030),
        lo(220, 0x10, 0x18, 0x28, THRESH_OPEN, DIR_UP, 15, 15),
    ],
    // L1: CELL/RADIAL - radial ice pattern on floor (bound, AXIS_POLAR)
    [
        hi_meta(
            OP_CELL,
            REGION_FLOOR,
            BLEND_LERP,
            DOMAIN_AXIS_POLAR,
            CELL_RADIAL,
            0x203050,
            0x101828,
        ),
        lo(100, 128, 200, 40, 7, DIR_UP, 10, 8),
    ],
    // L2: BAND - aurora horizon band (green/cyan, strong)
    [
        hi(OP_BAND, REGION_SKY, BLEND_ADD, 0, 0x00ff80, 0x00ffff),
        lo(200, 80, 128, 180, 0, DIR_UP, 12, 0),
    ],
    // L3: VEIL/CURTAINS - aurora curtains (vertical drapes)
    [
        hi_meta(
            OP_VEIL,
            REGION_SKY,
            BLEND_ADD,
            DOMAIN_AXIS_CYL,
            VEIL_CURTAINS,
            0x40ff80,
            0x000000,
        ),
        lo(200, 160, 80, 120, 0, DIR_UP, 12, 6),
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
    // L6: PLANE/STONE - snow and ice ground
    [
        hi_meta(
            OP_PLANE,
            REGION_FLOOR,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            PLANE_STONE,
            0x90a0b0,
            0x607080,
        ),
        lo(140, 128, 0, 0, 0, DIR_UP, 15, 15),
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
//
// L0: RAMP - deep indigo/purple base (brighter for readability)
// L1: SPLIT/PRISM - prismatic wall divisions
// L2: CELL/GRID - geometric floor tiles
// L3: GRID - gold sacred frame lines (HERO - geometric pattern)
// L4: TRACE/FILAMENTS - radial gold energy lines
// L5: APERTURE/CIRCLE - central sacred opening
// L6: LOBE - HERO: divine central light beam (bright gold)
// L7: SCATTER/DUST - golden sacred particles
pub(super) const PRESET_SACRED_GEOMETRY: [[u64; 2]; 8] = [
    // L0: RAMP - deep indigo sky, bronze floor, purple walls (BRIGHTER)
    [
        hi(OP_RAMP, REGION_ALL, BLEND_LERP, 0, 0x100820, 0x281810),
        lo(200, 0x20, 0x14, 0x30, THRESH_SEMI, DIR_UP, 15, 15),
    ],
    // L1: SPLIT/PRISM - prismatic wall divisions (more visible purple)
    [
        hi_meta(
            OP_SPLIT,
            REGION_WALLS,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            SPLIT_PRISM,
            0x180c30,
            0x281440,
        ),
        lo(180, 200, 0, 0, 0, DIR_UP, 12, 12),
    ],
    // L2: CELL/GRID - geometric floor tiles (more visible, warmer)
    [
        hi_meta(
            OP_CELL,
            REGION_FLOOR,
            BLEND_LERP,
            DOMAIN_AXIS_POLAR,
            CELL_GRID,
            0x301808,
            0x201008,
        ),
        lo(180, 80, 200, 60, 0, DIR_UP, 12, 10),
    ],
    // L3: GRID - HERO: gold sacred frame lines (brighter, more visible)
    [
        hi(OP_GRID, REGION_WALLS, BLEND_ADD, 0, 0xc08030, 0x604010),
        lo(200, 60, 30, 0, 0, DIR_UP, 15, 8),
    ],
    // L4: TRACE/FILAMENTS - radial energy lines (bright gold)
    [
        hi_meta(
            OP_TRACE,
            REGION_WALLS | REGION_FLOOR,
            BLEND_ADD,
            DOMAIN_AXIS_POLAR,
            TRACE_FILAMENTS,
            0xffa040,
            0x804020,
        ),
        lo(140, 140, 60, 120, 0, DIR_UP, 15, 8),
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
// L0: RAMP (void black sky, obsidian floor, dark stone walls)
// L1: APERTURE/CIRCLE (circular chamber opening - bound)
// L2: CELL/VORONOI (rough stone walls - bound)
// L3: DECAL (magic pentagram on floor)
// L4: PORTAL/CIRCLE (summoning portal, TANGENT_LOCAL)
// L5: TRACE/FILAMENTS (arcane energy veins, TANGENT_LOCAL)
// L6: SCATTER/EMBERS (magical sparks)
// L7: ATMOSPHERE/ALIEN (otherworldly atmosphere)
pub(super) const PRESET_RITUAL_CHAMBER: [[u64; 2]; 8] = [
    // L0: RAMP - void black sky, obsidian floor, dark stone walls
    [
        hi(OP_RAMP, REGION_ALL, BLEND_LERP, 0, 0x000004, 0x100808),
        lo(180, 0x18, 0x10, 0x18, THRESH_INTERIOR, DIR_UP, 15, 15),
    ],
    // L1: APERTURE/CIRCLE - circular chamber opening (bound)
    [
        hi_meta(
            OP_APERTURE,
            REGION_SKY,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            APERTURE_CIRCLE,
            0x080408,
            0x100810,
        ),
        lo(180, 128, 0, 0, 0, DIR_UP, 15, 15),
    ],
    // L2: CELL/VORONOI - rough stone walls (bound)
    [
        hi_meta(
            OP_CELL,
            REGION_WALLS,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            CELL_VORONOI,
            0x201020,
            0x100810,
        ),
        lo(140, 128, 0, 0, 0, DIR_UP, 15, 15),
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
        lo(220, 90, 200, 120, 0, DIR_DOWN, 15, 15),
    ],
    // L5: TRACE/LEAD_LINES - arcane sigils (tangent-local on floor)
    [
        hi_meta(
            OP_TRACE,
            REGION_FLOOR,
            BLEND_ADD,
            DOMAIN_TANGENT_LOCAL,
            TRACE_LEAD_LINES,
            0xff40ff,
            0x8020ff,
        ),
        lo(80, 200, 40, 160, 0x3A, DIR_DOWN, 15, 8),
    ],
    // L6: LOBE - portal glow from the ritual circle (reduce particle noise)
    [
        hi(OP_LOBE, REGION_ALL, BLEND_ADD, 0, 0xa040ff, 0x000000),
        lo(120, 220, 80, 1, 0, DIR_DOWN, 12, 0),
    ],
    // L7: ATMOSPHERE/ALIEN - otherworldly oppressive atmosphere (avoid flat wash)
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
        lo(60, 80, 128, 0, 0, DIR_UP, 12, 0),
    ],
];
