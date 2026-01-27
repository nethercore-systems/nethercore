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
// L4: SCATTER/RAIN (storm spray, dir=DOWN)
// L5: VEIL/SHARDS (light through storm clouds, AXIS_CYL)
// L6: ATMOSPHERE/FULL (coastal storm fog)
// L7: LOBE (lighthouse beam, dir=SUNSET)
pub(super) const PRESET_STORMY_SHORES: [[u64; 2]; 8] = [
    // L0: RAMP - dark stormy sky, wet rocky shore, gray cliff walls
    [
        hi(OP_RAMP, REGION_ALL, BLEND_LERP, 0, 0x1a2028, 0x202830),
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
            0x101820,
            0x1a2028,
        ),
        lo(200, 128, 0, 0, 0, DIR_UP, 15, 15),
    ],
    // L2: PLANE/STONE - wet rocky shoreline
    [
        hi_meta(
            OP_PLANE,
            REGION_FLOOR,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            PLANE_STONE,
            0x303840,
            0x202830,
        ),
        lo(150, 128, 0, 0, 0, DIR_UP, 15, 15),
    ],
    // L3: FLOW - sea foam and spray
    [
        hi(OP_FLOW, REGION_WALLS, BLEND_ADD, 0, 0x607080, 0x000000),
        lo(180, 128, 0, 3, 100, DIR_UP, 12, 0),
    ],
    // L4: SCATTER/RAIN - storm spray particles, falling downward
    [
        hi_meta(
            OP_SCATTER,
            REGION_ALL,
            BLEND_ADD,
            DOMAIN_DIRECT3D,
            SCATTER_RAIN,
            0x90a0b0,
            0x000000,
        ),
        lo(100, 40, 80, 0x30, 0, DIR_DOWN, 12, 0),
    ],
    // L5: VEIL/SHARDS - light breaking through storm clouds (AXIS_CYL)
    [
        hi_meta(
            OP_VEIL,
            REGION_SKY,
            BLEND_ADD,
            DOMAIN_AXIS_CYL,
            VEIL_SHARDS,
            0x8090a0,
            0x000000,
        ),
        lo(100, 128, 0, 0, 0, DIR_SUN, 10, 0),
    ],
    // L6: ATMOSPHERE/FULL - heavy coastal storm fog
    [
        hi_meta(
            OP_ATMOSPHERE,
            REGION_ALL,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            ATMO_FULL,
            0x283038,
            0x000000,
        ),
        lo(120, 90, 0, 0, 0, DIR_UP, 15, 0),
    ],
    // L7: LOBE - lighthouse beam sweeping through fog (sine)
    [
        hi(OP_LOBE, REGION_ALL, BLEND_ADD, 0, 0xffffd0, 0x000000),
        lo(230, 128, 0, 1, 3, DIR_SUNSET, 15, 0),
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
        hi(OP_RAMP, REGION_ALL, BLEND_LERP, 0, 0x080818, 0xd0e0f0),
        lo(240, 0x40, 0x50, 0x60, THRESH_OPEN, DIR_UP, 15, 15),
    ],
    // L1: CELL/RADIAL - radial ice pattern on floor (bound, AXIS_POLAR)
    [
        hi_meta(
            OP_CELL,
            REGION_FLOOR,
            BLEND_ADD,
            DOMAIN_AXIS_POLAR,
            CELL_RADIAL,
            0x406080,
            0x203040,
        ),
        lo(130, 128, 0, 0, 0, DIR_UP, 12, 12),
    ],
    // L2: BAND - aurora horizon band (green/cyan)
    [
        hi(OP_BAND, REGION_SKY, BLEND_ADD, 0, 0x00ff80, 0x00ffff),
        lo(220, 128, 0, 0, 0, DIR_UP, 15, 15),
    ],
    // L3: VEIL/CURTAINS - aurora curtains (AXIS_POLAR)
    [
        hi_meta(
            OP_VEIL,
            REGION_SKY,
            BLEND_ADD,
            DOMAIN_AXIS_POLAR,
            VEIL_CURTAINS,
            0x40ff80,
            0x000000,
        ),
        lo(200, 128, 80, 0, 0, DIR_UP, 12, 0),
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
            0xe0f0ff,
            0xc0d0e0,
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
        lo(80, 100, 0, 0, 0, DIR_UP, 15, 0),
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
    // L0: RAMP - deep indigo sky, gold floor, purple walls
    [
        hi(OP_RAMP, REGION_ALL, BLEND_LERP, 0, 0x100828, 0xc0a040),
        lo(200, 0x40, 0x20, 0x50, THRESH_SEMI, DIR_UP, 15, 15),
    ],
    // L1: SPLIT/PRISM - prismatic wall divisions (bound)
    [
        hi_meta(
            OP_SPLIT,
            REGION_WALLS,
            BLEND_ADD,
            DOMAIN_DIRECT3D,
            SPLIT_PRISM,
            0x402060,
            0x604080,
        ),
        lo(130, 200, 0, 0, 0, DIR_UP, 12, 12),
    ],
    // L2: CELL/GRID - geometric floor tiles (bound, AXIS_POLAR)
    [
        hi_meta(
            OP_CELL,
            REGION_FLOOR,
            BLEND_ADD,
            DOMAIN_AXIS_POLAR,
            CELL_GRID,
            0xffd080,
            0xa08040,
        ),
        lo(130, 128, 0, 0, 0, DIR_UP, 12, 12),
    ],
    // L3: GRID - geometric sacred frame lines
    [
        hi(OP_GRID, REGION_WALLS, BLEND_ADD, 0, 0x806040, 0x000000),
        lo(200, 64, 0, 0, 0, DIR_UP, 10, 0),
    ],
    // L4: TRACE/FILAMENTS - radial energy lines (AXIS_POLAR)
    [
        hi_meta(
            OP_TRACE,
            REGION_WALLS,
            BLEND_ADD,
            DOMAIN_AXIS_POLAR,
            TRACE_FILAMENTS,
            0xffffff,
            0x000000,
        ),
        lo(200, 128, 0, 0, 0, DIR_UP, 12, 0),
    ],
    // L5: APERTURE/CIRCLE - central sacred opening
    [
        hi_meta(
            OP_APERTURE,
            REGION_ALL,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            APERTURE_CIRCLE,
            0x200810,
            0x402040,
        ),
        lo(180, 128, 0, 0, 0, DIR_UP, 15, 15),
    ],
    // L6: LOBE - divine central light beam (sine pulse)
    [
        hi(OP_LOBE, REGION_ALL, BLEND_ADD, 0, 0xfff0c0, 0x000000),
        lo(140, 128, 0, 1, 2, DIR_DOWN, 15, 0),
    ],
    // L7: SCATTER/DUST - golden sacred particles
    [
        hi_meta(
            OP_SCATTER,
            REGION_ALL,
            BLEND_SCREEN,
            DOMAIN_DIRECT3D,
            SCATTER_DUST,
            0xffd040,
            0x000000,
        ),
        lo(100, 100, 40, 0x20, 0, DIR_UP, 12, 0),
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
    // L3: DECAL - magic pentagram on floor (crimson)
    [
        hi(OP_DECAL, REGION_FLOOR, BLEND_ADD, 0, 0xff2000, 0x400000),
        lo(255, 8, 100, 0, 3, DIR_UP, 15, 15), // shape=DISK(0), soft=8, size=100, param_d=3: ritual pulse
    ],
    // L4: PORTAL/CIRCLE - summoning portal (dark purple, TANGENT_LOCAL)
    [
        hi_meta(
            OP_PORTAL,
            REGION_FLOOR,
            BLEND_ADD,
            DOMAIN_TANGENT_LOCAL,
            PORTAL_CIRCLE,
            0x8020ff,
            0x200040,
        ),
        lo(255, 128, 0, 0, 0, DIR_UP, 15, 15),
    ],
    // L5: TRACE/FILAMENTS - arcane energy veins (purple, TANGENT_LOCAL)
    [
        hi_meta(
            OP_TRACE,
            REGION_WALLS,
            BLEND_ADD,
            DOMAIN_TANGENT_LOCAL,
            TRACE_FILAMENTS,
            0xa040ff,
            0x000000,
        ),
        lo(220, 128, 0, 0, 0, DIR_UP, 12, 0),
    ],
    // L6: SCATTER/EMBERS - magical sparks and embers
    [
        hi_meta(
            OP_SCATTER,
            REGION_ALL,
            BLEND_SCREEN,
            DOMAIN_DIRECT3D,
            SCATTER_EMBERS,
            0xff8040,
            0x000000,
        ),
        lo(110, 20, 60, 0x30, 0, DIR_UP, 15, 0),
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
        lo(100, 60, 0, 0, 0, DIR_UP, 15, 0),
    ],
];
