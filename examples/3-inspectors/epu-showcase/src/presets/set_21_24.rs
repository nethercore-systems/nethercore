//! Preset set 21-24

#[allow(unused_imports)]
use crate::constants::*;

// -----------------------------------------------------------------------------
// Preset 21: "Stormy Shores" - Coastal storm, crashing waves
// -----------------------------------------------------------------------------
// L0: RAMP (dark stormy sky, wet rocky shore, gray cliffs)
// L1: SILHOUETTE/WAVES (crashing waves on horizon)
// L2: PLANE/STONE (rocky shoreline)
// L3: FLOW (churning sea foam and spray)
// L4: SCATTER (sea spray particles)
// L5: VEIL/SHARDS (light breaking through storm clouds)
// L6: ATMOSPHERE/FULL (heavy storm fog)
// L7: LOBE (lighthouse beam cutting through fog)
pub(super) const PRESET_STORMY_SHORES: [[u64; 2]; 8] = [
    // L0: RAMP - dark stormy sky, wet rocky shore, gray cliff walls
    [
        hi(OP_RAMP, REGION_ALL, BLEND_LERP, 0, 0x1a2028, 0x202830),
        lo(255, 0x30, 0x38, 0x40, 0, DIR_UP, 15, 15),
    ],
    // L1: SILHOUETTE/WAVES - crashing ocean waves on horizon
    [
        hi_meta(
            OP_SILHOUETTE,
            REGION_WALLS,
            BLEND_MULTIPLY,
            DOMAIN_DIRECT3D,
            SILHOUETTE_WAVES,
            0x101820,
            0x000000,
        ),
        lo(200, 128, 0, 0, 0, DIR_UP, 15, 0),
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
            0x000000,
        ),
        lo(150, 128, 0, 0, 0, DIR_UP, 15, 0),
    ],
    // L3: FLOW - churning sea foam and spray
    [
        hi(OP_FLOW, REGION_ALL, BLEND_ADD, 0, 0x607080, 0x000000),
        lo(140, 128, 100, 3, 0, DIR_UP, 12, 0),
    ],
    // L4: SCATTER - sea spray particles
    [
        hi_meta(
            OP_SCATTER,
            REGION_ALL,
            BLEND_ADD,
            DOMAIN_DIRECT3D,
            SCATTER_DUST,
            0x90a0b0,
            0x000000,
        ),
        lo(160, 120, 80, 0x30, 0, DIR_UP, 12, 0),
    ],
    // L5: VEIL/SHARDS - light breaking through storm clouds
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
        lo(180, 200, 0, 0, 0, DIR_UP, 15, 0),
    ],
    // L7: LOBE - lighthouse beam cutting through fog
    [
        hi(OP_LOBE, REGION_ALL, BLEND_ADD, 0, 0xffffd0, 0x000000),
        lo(160, 128, 0, 0, 3, DIR_SUNSET, 15, 0), // param_d=3: beam sweep
    ],
];

// -----------------------------------------------------------------------------
// Preset 22: "Polar Aurora" - Arctic night with northern lights
// -----------------------------------------------------------------------------
// L0: RAMP (dark night sky, snow floor, ice walls)
// L1: BAND (aurora band on horizon - key showcase)
// L2: VEIL/CURTAINS (aurora curtains with AXIS_POLAR for radial spread)
// L3: FLOW (swirling aurora patterns)
// L4: SCATTER (stars in night sky)
// L5: CELESTIAL/MOON (arctic moon)
// L6: PLANE/STONE (snow/ice ground)
// L7: ATMOSPHERE/RAYLEIGH (crisp arctic air)
pub(super) const PRESET_POLAR_AURORA: [[u64; 2]; 8] = [
    // L0: RAMP - dark arctic night sky, snow floor, ice walls
    [
        hi(OP_RAMP, REGION_ALL, BLEND_LERP, 0, 0x080818, 0xd0e0f0),
        lo(255, 0x40, 0x50, 0x60, 0, DIR_UP, 15, 15),
    ],
    // L1: BAND - aurora band on horizon (green/cyan)
    [
        hi(OP_BAND, REGION_SKY, BLEND_ADD, 0, 0x00ff80, 0x00ffff),
        lo(180, 128, 0, 0, 0, DIR_UP, 15, 15),
    ],
    // L2: VEIL/CURTAINS - aurora curtains with AXIS_POLAR (radial spread)
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
        lo(160, 128, 80, 0, 0, DIR_UP, 12, 0),
    ],
    // L3: FLOW - swirling aurora patterns (green/purple)
    [
        hi(OP_FLOW, REGION_SKY, BLEND_ADD, 0, 0x8040ff, 0x000000),
        lo(120, 128, 60, 2, 0, DIR_UP, 10, 0),
    ],
    // L4: SCATTER - stars in night sky
    [
        hi(OP_SCATTER, REGION_SKY, BLEND_ADD, 0, 0xffffff, 0x000000),
        lo(200, 200, 0, 0x10, 0, DIR_UP, 15, 0),
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
        lo(220, 128, 0, 0, 0, DIR_SUN, 15, 0),
    ],
    // L6: PLANE/STONE - snow and ice ground (bright white)
    [
        hi_meta(
            OP_PLANE,
            REGION_FLOOR,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            PLANE_STONE,
            0xe0f0ff,
            0x000000,
        ),
        lo(140, 128, 0, 0, 0, DIR_UP, 15, 0),
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
// Preset 23: "Sacred Geometry" - Mandala chamber, radial patterns
// -----------------------------------------------------------------------------
// L0: RAMP (deep indigo, gold floor, purple walls)
// L1: CELL/RADIAL (radial mandala pattern - AXIS_POLAR)
// L2: TRACE/FILAMENTS (radial energy lines - AXIS_POLAR)
// L3: GRID (geometric frame)
// L4: APERTURE/CIRCLE (central sacred opening)
// L5: LOBE (divine central light)
// L6: SCATTER (golden particles)
// L7: DECAL (sacred symbol on floor)
pub(super) const PRESET_SACRED_GEOMETRY: [[u64; 2]; 8] = [
    // L0: RAMP - deep indigo sky, gold floor, purple walls
    [
        hi(OP_RAMP, REGION_ALL, BLEND_LERP, 0, 0x100828, 0xc0a040),
        lo(255, 0x40, 0x20, 0x50, 0, DIR_UP, 15, 15),
    ],
    // L1: CELL/RADIAL - radial mandala pattern (AXIS_POLAR domain)
    [
        hi_meta(
            OP_CELL,
            REGION_FLOOR,
            BLEND_ADD,
            DOMAIN_AXIS_POLAR,
            CELL_RADIAL,
            0xffd080,
            0x000000,
        ),
        lo(160, 128, 0, 0, 0, DIR_UP, 12, 0),
    ],
    // L2: TRACE/FILAMENTS - radial energy lines (AXIS_POLAR)
    [
        hi_meta(
            OP_TRACE,
            REGION_ALL,
            BLEND_ADD,
            DOMAIN_AXIS_POLAR,
            TRACE_FILAMENTS,
            0xffffff,
            0x000000,
        ),
        lo(140, 128, 0, 0, 0, DIR_UP, 12, 0),
    ],
    // L3: GRID - geometric sacred frame
    [
        hi(OP_GRID, REGION_ALL, BLEND_ADD, 0, 0x806040, 0x000000),
        lo(100, 64, 0, 0, 0, DIR_UP, 10, 0),
    ],
    // L4: APERTURE/CIRCLE - central sacred circular opening
    [
        hi_meta(
            OP_APERTURE,
            REGION_ALL,
            BLEND_MULTIPLY,
            DOMAIN_DIRECT3D,
            APERTURE_CIRCLE,
            0x200810,
            0x000000,
        ),
        lo(180, 128, 0, 0, 0, DIR_UP, 15, 0),
    ],
    // L5: LOBE - divine central light beam
    [
        hi(OP_LOBE, REGION_ALL, BLEND_ADD, 0, 0xfff0c0, 0x000000),
        lo(200, 128, 0, 0, 2, DIR_DOWN, 15, 0), // param_d=2: divine pulse
    ],
    // L6: SCATTER - golden sacred particles
    [
        hi_meta(
            OP_SCATTER,
            REGION_ALL,
            BLEND_ADD,
            DOMAIN_DIRECT3D,
            SCATTER_DUST,
            0xffd040,
            0x000000,
        ),
        lo(150, 100, 40, 0x20, 0, DIR_UP, 12, 0),
    ],
    // L7: DECAL - sacred symbol on floor
    [
        hi(OP_DECAL, REGION_FLOOR, BLEND_ADD, 0, 0xffe080, 0x000000),
        lo(180, 80, 0, 0, 2, DIR_UP, 15, 0), // param_d=2: glow pulse
    ],
];

// -----------------------------------------------------------------------------
// Preset 24: "Ritual Chamber" - Dark magic summoning room
// -----------------------------------------------------------------------------
// L0: RAMP (void black, obsidian floor, dark stone walls)
// L1: DECAL (magic circle/pentagram on floor - key showcase)
// L2: PORTAL/CIRCLE (summoning portal)
// L3: TRACE/FILAMENTS (arcane energy veins)
// L4: VEIL/PILLARS (energy pillars at cardinal points)
// L5: FLOW (swirling dark energy)
// L6: SCATTER (magical sparks)
// L7: ATMOSPHERE/ALIEN (otherworldly atmosphere)
pub(super) const PRESET_RITUAL_CHAMBER: [[u64; 2]; 8] = [
    // L0: RAMP - void black sky, obsidian floor, dark stone walls
    [
        hi(OP_RAMP, REGION_ALL, BLEND_LERP, 0, 0x000004, 0x100808),
        lo(255, 0x18, 0x10, 0x18, 0, DIR_UP, 15, 15),
    ],
    // L1: DECAL - magic circle/pentagram on floor (crimson)
    [
        hi(OP_DECAL, REGION_FLOOR, BLEND_ADD, 0, 0xff2000, 0x400000),
        lo(220, 100, 0, 0, 3, DIR_UP, 15, 15), // param_d=3: ritual pulse
    ],
    // L2: PORTAL/CIRCLE - summoning portal (dark purple)
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
        lo(180, 64, 0, 0, 0, DIR_UP, 15, 15),
    ],
    // L3: TRACE/FILAMENTS - arcane energy veins (purple)
    [
        hi_meta(
            OP_TRACE,
            REGION_ALL,
            BLEND_ADD,
            DOMAIN_TANGENT_LOCAL,
            TRACE_FILAMENTS,
            0xa040ff,
            0x000000,
        ),
        lo(160, 128, 0, 0, 0, DIR_UP, 12, 0),
    ],
    // L4: VEIL/PILLARS - energy pillars at ritual points
    [
        hi_meta(
            OP_VEIL,
            REGION_ALL,
            BLEND_ADD,
            DOMAIN_AXIS_CYL,
            VEIL_PILLARS,
            0xff4080,
            0x000000,
        ),
        lo(140, 128, 0, 0, 0, DIR_UP, 12, 0),
    ],
    // L5: FLOW - swirling dark magical energy
    [
        hi(OP_FLOW, REGION_ALL, BLEND_ADD, 0, 0x400060, 0x000000),
        lo(120, 128, 80, 3, 0, DIR_UP, 10, 0),
    ],
    // L6: SCATTER - magical sparks and embers
    [
        hi_meta(
            OP_SCATTER,
            REGION_ALL,
            BLEND_ADD,
            DOMAIN_DIRECT3D,
            SCATTER_EMBERS,
            0xff8040,
            0x000000,
        ),
        lo(180, 120, 60, 0x30, 0, DIR_UP, 15, 0),
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
        lo(140, 150, 0, 0, 0, DIR_UP, 15, 0),
    ],
];

