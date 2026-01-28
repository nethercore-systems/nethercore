//! Preset set 21-24

#[allow(unused_imports)]
use crate::constants::*;

// -----------------------------------------------------------------------------
// Preset 21: "Stormy Shores" - Coastal cliffs, crashing waves
// -----------------------------------------------------------------------------
// L0: RAMP (dark stormy sky, wet rocky shore, gray cliffs)
// L1: SILHOUETTE/WAVES (crashing waves on walls)
// L2: PLANE/STONE (wet rocky shore)
// L3: FLOW (sea foam and spray)
// L4: VEIL/RAIN_WALL (driving rain, TANGENT_LOCAL)
// L5: VEIL/SHARDS (light through storm clouds, AXIS_CYL)
// L6: ATMOSPHERE/FULL (coastal storm fog)
// L7: LOBE (lighthouse beam, dir=SUNSET)
pub(super) const PRESET_STORMY_SHORES: [[u64; 2]; 8] = [
    // L0: RAMP - dark stormy sky, wet rocky shore, gray cliff walls
    [
        hi(OP_RAMP, REGION_ALL, BLEND_LERP, 0, 0x283040, 0x303840),
        lo(240, 0x30, 0x38, 0x40, THRESH_OPEN, DIR_UP, 15, 15),
    ],
    // L1: SILHOUETTE/WAVES - crashing waves on walls (bound)
    [
        hi_meta(
            OP_SILHOUETTE,
            REGION_WALLS,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            SILHOUETTE_WAVES,
            0x081018,
            0x202830,
        ),
        lo(220, 200, 0, 0, 0, DIR_UP, 15, 15),
    ],
    // L2: PLANE/WATER - churning ocean surface
    [
        hi_meta(
            OP_PLANE,
            REGION_FLOOR,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            PLANE_WATER,
            0x204060,
            0x102030,
        ),
        lo(150, 128, 0, 0, 0, DIR_UP, 15, 15),
    ],
    // L3: (unused) - avoid large swirl artifacts
    NOP_LAYER,
    // L4: VEIL/RAIN_WALL - driving rain (tangent-local)
    [
        hi_meta(
            OP_VEIL,
            REGION_SKY | REGION_WALLS,
            BLEND_SCREEN,
            DOMAIN_TANGENT_LOCAL,
            VEIL_RAIN_WALL,
            0xa0b8d0,
            0x000000,
        ),
        lo(200, 240, 70, 140, 80, DIR_FORWARD, 14, 8),
    ],
    // L5: TRACE/LIGHTNING - very bright + thick (must be obvious)
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
        lo(255, 255, 200, 255, 0x2E, DIR_FORWARD, 15, 12),
    ],
    // L6: ATMOSPHERE/FULL - heavy coastal storm fog (keep horizon_y centered)
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
        lo(60, 90, 128, 0, 0, DIR_UP, 12, 0),
    ],
    // L7: LOBE - lighthouse beam (make it cut across the shot)
    [
        hi(OP_LOBE, REGION_ALL, BLEND_ADD, 0, 0xffffd0, 0x000000),
        lo(200, 255, 60, 1, 0, DIR_RIGHT, 12, 0),
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
// L0: RAMP (deep indigo sky, gold floor, purple walls)
// L1: SPLIT/PRISM (prismatic wall divisions - bound)
// L2: CELL/GRID (geometric floor tiles, AXIS_POLAR - bound)
// L3: GRID (geometric frame lines)
// L4: TRACE/FILAMENTS (radial energy, AXIS_POLAR)
// L5: APERTURE/CIRCLE (central opening)
// L6: LOBE (divine central light, dir=DOWN)
// L7: SCATTER/DUST (golden sacred particles)
pub(super) const PRESET_SACRED_GEOMETRY: [[u64; 2]; 8] = [
    // L0: RAMP - deep indigo sky, dark bronze floor, dark purple walls
    [
        hi(OP_RAMP, REGION_ALL, BLEND_LERP, 0, 0x060310, 0x1c1008),
        lo(200, 0x14, 0x08, 0x20, THRESH_SEMI, DIR_UP, 15, 15),
    ],
    // L1: SPLIT/PRISM - prismatic wall divisions (bound, LERP, dark purple)
    [
        hi_meta(
            OP_SPLIT,
            REGION_WALLS,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            SPLIT_PRISM,
            0x100820,
            0x180c28,
        ),
        lo(200, 200, 0, 0, 0, DIR_UP, 12, 12),
    ],
    // L2: CELL/GRID - geometric floor tiles (bound, LERP, very dark, low alpha)
    [
        hi_meta(
            OP_CELL,
            REGION_FLOOR,
            BLEND_LERP,
            DOMAIN_AXIS_POLAR,
            CELL_GRID,
            0x180c04,
            0x100804,
        ),
        lo(200, 80, 200, 50, 0, DIR_UP, 10, 10),
    ],
    // L3: GRID - geometric sacred frame lines (gold, feature ADD, reduced)
    [
        hi(OP_GRID, REGION_WALLS, BLEND_ADD, 0, 0x603010, 0x000000),
        lo(150, 64, 20, 0, 0, DIR_UP, 10, 0),
    ],
    // L4: TRACE/FILAMENTS - radial energy lines (warm gold)
    [
        hi_meta(
            OP_TRACE,
            REGION_WALLS,
            BLEND_ADD,
            DOMAIN_AXIS_POLAR,
            TRACE_FILAMENTS,
            0xff9040,
            0x000000,
        ),
        lo(80, 128, 0, 0, 0, DIR_UP, 12, 0),
    ],
    // L5: APERTURE/CIRCLE - central sacred opening (darkens edges)
    [
        hi_meta(
            OP_APERTURE,
            REGION_ALL,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            APERTURE_CIRCLE,
            0x040208,
            0x201020,
        ),
        lo(120, 128, 0, 0, 0, DIR_UP, 15, 15),
    ],
    // L6: LOBE - divine central light beam (warm gold, sine pulse)
    [
        hi(OP_LOBE, REGION_ALL, BLEND_ADD, 0, 0xffd080, 0x000000),
        lo(80, 128, 0, 1, 2, DIR_DOWN, 15, 0),
    ],
    // L7: SCATTER/DUST - golden sacred particles (subtle)
    [
        hi_meta(
            OP_SCATTER,
            REGION_ALL,
            BLEND_ADD,
            DOMAIN_DIRECT3D,
            SCATTER_DUST,
            0xffc040,
            0x000000,
        ),
        lo(60, 100, 40, 0x20, 0, DIR_UP, 12, 0),
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
