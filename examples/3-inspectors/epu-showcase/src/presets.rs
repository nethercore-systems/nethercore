//! EPU Preset Configurations (EPU v1 128-bit format)
//!
//! This module contains environment presets for the EPU inspector demo.
//!
//! Each 128-bit layer is stored as [hi, lo] u64 pair:
//!
//! u64 hi [bits 127..64]:
//!   bits 63..59: opcode     (5)   - NOP=0, bounds=1..7 (RAMP=1, enclosure=2..7), features=8.. (DECAL=8, GRID=9, SCATTER=10, FLOW=11), radiance=12.. (TRACE, VEIL, etc.)
//!   bits 58..56: region     (3)   - Bitfield: SKY=0b100, WALLS=0b010, FLOOR=0b001, ALL=0b111
//!   bits 55..53: blend      (3)   - ADD=0, MULTIPLY=1, MAX=2, LERP=3, SCREEN=4, HSV_MOD=5, MIN=6, OVERLAY=7
//!   bits 52..49: meta_hi    (4)   - domain/variant (high bits)
//!   bit  48:     meta_lo    (1)   - domain/variant (low bit)
//!   bits 47..24: color_a    (24)  - RGB24 primary color
//!   bits 23..0:  color_b    (24)  - RGB24 secondary color
//!
//! u64 lo [bits 63..0]:
//!   bits 63..56: intensity  (8)
//!   bits 55..48: param_a    (8)
//!   bits 47..40: param_b    (8)
//!   bits 39..32: param_c    (8)
//!   bits 31..24: param_d    (8)
//!   bits 23..8:  direction  (16)  - Octahedral encoded
//!   bits 7..4:   alpha_a    (4)   - color_a alpha (0-15)
//!   bits 3..0:   alpha_b    (4)   - color_b alpha (0-15)

#[allow(unused_imports)]
use crate::constants::{
    hi,
    hi_meta,
    lo,
    // Variant IDs
    APERTURE_ARCH,
    APERTURE_BARS,
    APERTURE_CIRCLE,
    APERTURE_IRREGULAR,
    APERTURE_MULTI,
    APERTURE_RECT,
    APERTURE_ROUNDED_RECT,
    ATMO_ABSORPTION,
    ATMO_ALIEN,
    ATMO_FULL,
    ATMO_MIE,
    ATMO_RAYLEIGH,
    // Blend modes
    BLEND_ADD,
    BLEND_HSV_MOD,
    BLEND_LERP,
    BLEND_MAX,
    BLEND_MIN,
    BLEND_MULTIPLY,
    BLEND_OVERLAY,
    BLEND_SCREEN,
    CELESTIAL_BINARY,
    CELESTIAL_ECLIPSE,
    CELESTIAL_GAS_GIANT,
    CELESTIAL_MOON,
    CELESTIAL_PLANET,
    CELESTIAL_RINGED,
    CELESTIAL_SUN,
    CELL_BRICK,
    CELL_GRID,
    CELL_HEX,
    CELL_RADIAL,
    CELL_SHATTER,
    CELL_VORONOI,
    // Directions
    DIR_DOWN,
    DIR_SUN,
    DIR_SUNSET,
    DIR_UP,
    // Domain IDs
    DOMAIN_AXIS_CYL,
    DOMAIN_AXIS_POLAR,
    DOMAIN_DIRECT3D,
    DOMAIN_TANGENT_LOCAL,
    // NOP layer constant
    NOP_LAYER,
    // Enclosure opcodes
    OP_APERTURE,
    // Radiance opcodes
    OP_ATMOSPHERE,
    OP_BAND,
    OP_CELESTIAL,
    OP_CELL,
    OP_DECAL,
    OP_FLOW,
    OP_GRID,
    OP_LOBE,
    OP_PATCHES,
    OP_PLANE,
    OP_PORTAL,
    OP_RAMP,
    OP_SCATTER,
    OP_SECTOR,
    OP_SILHOUETTE,
    OP_SPLIT,
    OP_TRACE,
    OP_VEIL,
    PATCHES_BLOBS,
    PATCHES_DEBRIS,
    PATCHES_ISLANDS,
    PATCHES_MEMBRANE,
    PATCHES_STATIC,
    PATCHES_STREAKS,
    PLANE_GRASS,
    PLANE_GRATING,
    PLANE_HEX,
    PLANE_PAVEMENT,
    PLANE_SAND,
    PLANE_STONE,
    PLANE_TILES,
    PLANE_WATER,
    PORTAL_CIRCLE,
    PORTAL_CRACK,
    PORTAL_RECT,
    PORTAL_RIFT,
    PORTAL_TEAR,
    PORTAL_VORTEX,
    // Regions
    REGION_ALL,
    REGION_FLOOR,
    REGION_SKY,
    REGION_WALLS,
    SECTOR_BOX,
    SECTOR_CAVE,
    SECTOR_TUNNEL,
    SILHOUETTE_CITY,
    SILHOUETTE_DUNES,
    SILHOUETTE_FOREST,
    SILHOUETTE_INDUSTRIAL,
    SILHOUETTE_MOUNTAINS,
    SILHOUETTE_RUINS,
    SILHOUETTE_SPIRES,
    SILHOUETTE_WAVES,
    SPLIT_BANDS,
    SPLIT_CORNER,
    SPLIT_CROSS,
    SPLIT_HALF,
    SPLIT_PRISM,
    SPLIT_WEDGE,
    TRACE_CRACKS,
    TRACE_FILAMENTS,
    TRACE_LEAD_LINES,
    TRACE_LIGHTNING,
    VEIL_CURTAINS,
    VEIL_LASER_BARS,
    VEIL_PILLARS,
    VEIL_RAIN_WALL,
    VEIL_SHARDS,
};

// =============================================================================
// Environment Presets (EPU v1 128-bit format)
// =============================================================================

// -----------------------------------------------------------------------------
// Preset 1: "Neon Metropolis" - Cyberpunk urban
// -----------------------------------------------------------------------------
// L0: RAMP (sky=#1a0a2e, floor=#000000, walls RGB=#1c1c1c via param_a/b/c)
// L1: SILHOUETTE/CITY (black #000000)
// L2: GRID (cyan #00ffff, walls only)
// L3: SCATTER (warm yellow #ffcc00, walls, windows-like)
// L4: VEIL/LASER_BARS (magenta #ff00ff)
// L5: ATMOSPHERE/MIE (gray #404050)
// L6: FLOW (cyan #00ddff, rain effect)
// L7: SECTOR/TUNNEL (BLEND_OVERLAY, #000000/#1a0a2e)
const PRESET_NEON_METROPOLIS: [[u64; 2]; 8] = [
    // L0: RAMP - deep purple sky, black floor, dark gray walls
    // hi: opcode=RAMP, region=ALL, blend=LERP, meta5=0, color_a=sky, color_b=floor
    // lo: intensity=255, param_a/b/c=wall RGB (0x1c each), param_d=0, dir=UP, alpha=15/15
    [
        hi(OP_RAMP, REGION_ALL, BLEND_LERP, 0, 0x1a0a2e, 0x000000),
        lo(255, 0x1c, 0x1c, 0x1c, 0, DIR_UP, 15, 15),
    ],
    // L1: SILHOUETTE/CITY - black city skyline cutout on walls
    [
        hi_meta(
            OP_SILHOUETTE,
            REGION_WALLS,
            BLEND_MULTIPLY,
            DOMAIN_DIRECT3D,
            SILHOUETTE_CITY,
            0x000000,
            0x000000,
        ),
        lo(200, 128, 0, 0, 0, DIR_UP, 15, 15),
    ],
    // L2: GRID - cyan grid on walls (vertical bars)
    [
        hi(OP_GRID, REGION_WALLS, BLEND_ADD, 0, 0x00ffff, 0x000000),
        lo(160, 32, 0, 3, 0, DIR_UP, 15, 0), // param_c=3: slow scroll animation
    ],
    // L3: SCATTER - warm yellow windows/lights on walls
    [
        hi(OP_SCATTER, REGION_WALLS, BLEND_ADD, 0, 0xffcc00, 0x000000),
        lo(180, 128, 0, 2, 0, DIR_UP, 15, 0), // param_c=2: twinkle animation
    ],
    // L4: VEIL/LASER_BARS - magenta vertical laser beams
    [
        hi_meta(
            OP_VEIL,
            REGION_ALL,
            BLEND_ADD,
            DOMAIN_AXIS_CYL,
            VEIL_LASER_BARS,
            0xff00ff,
            0x000000,
        ),
        lo(180, 64, 0, 0, 0, DIR_UP, 15, 0),
    ],
    // L5: ATMOSPHERE/MIE - subtle gray haze
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
        lo(60, 128, 0, 0, 0, DIR_UP, 15, 0),
    ],
    // L6: FLOW - cyan rain effect (downward)
    [
        hi(OP_FLOW, REGION_ALL, BLEND_ADD, 0, 0x00ddff, 0x000000),
        lo(140, 128, 200, 0, 0, DIR_DOWN, 15, 0),
    ],
    // L7: SECTOR/TUNNEL - tunnel enclosure effect with overlay blend
    [
        hi_meta(
            OP_SECTOR,
            REGION_ALL,
            BLEND_OVERLAY,
            0,
            SECTOR_TUNNEL,
            0x000000,
            0x1a0a2e,
        ),
        lo(140, 128, 0, 0, 0, DIR_UP, 15, 15),
    ],
];

// -----------------------------------------------------------------------------
// Preset 2: "Crimson Hellscape" - Horror/demonic
// -----------------------------------------------------------------------------
// L0: RAMP (sky=#4a0000, floor=#0a0000, walls=#2a0808)
// L1: TRACE/CRACKS (orange-red #ff3300)
// L2: PATCHES/MEMBRANE (dark red #330000)
// L3: FLOW (ember orange #ff4400)
// L4: SCATTER (bright orange #ff8800, embers)
// L5: ATMOSPHERE/ABSORPTION (blood mist #400000)
// L6: CELESTIAL/ECLIPSE (black #000000 with red #ff0000)
// L7: PORTAL/RIFT (hellfire red #ff2200)
const PRESET_CRIMSON_HELLSCAPE: [[u64; 2]; 8] = [
    // L0: RAMP - blood red sky, charred black floor, dark crimson walls
    [
        hi(OP_RAMP, REGION_ALL, BLEND_LERP, 0, 0x4a0000, 0x0a0000),
        lo(255, 0x2a, 0x08, 0x08, 0, DIR_UP, 15, 15),
    ],
    // L1: TRACE/CRACKS - volcanic fissures on floor/walls
    [
        hi_meta(
            OP_TRACE,
            REGION_ALL,
            BLEND_ADD,
            DOMAIN_TANGENT_LOCAL,
            TRACE_CRACKS,
            0xff3300,
            0x000000,
        ),
        lo(180, 128, 64, 0, 0, DIR_UP, 15, 0),
    ],
    // L2: PATCHES/MEMBRANE - organic tissue texture
    [
        hi_meta(
            OP_PATCHES,
            REGION_ALL,
            BLEND_MULTIPLY,
            DOMAIN_DIRECT3D,
            PATCHES_MEMBRANE,
            0x330000,
            0x000000,
        ),
        lo(150, 128, 32, 0, 0, DIR_UP, 15, 0),
    ],
    // L3: FLOW - ember orange, slowly churning lava glow
    [
        hi(OP_FLOW, REGION_FLOOR, BLEND_ADD, 0, 0xff4400, 0x000000),
        lo(160, 128, 64, 0, 0, DIR_UP, 15, 0),
    ],
    // L4: SCATTER - rising embers/sparks
    [
        hi(OP_SCATTER, REGION_ALL, BLEND_ADD, 0, 0xff8800, 0x000000),
        lo(180, 100, 100, 3, 0, DIR_UP, 15, 0), // param_c=3: ember flicker
    ],
    // L5: ATMOSPHERE/ABSORPTION - thick blood mist
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
        lo(180, 200, 0, 0, 0, DIR_UP, 15, 0),
    ],
    // L6: CELESTIAL/ECLIPSE - black sun with red corona
    [
        hi_meta(
            OP_CELESTIAL,
            REGION_SKY,
            BLEND_ADD,
            DOMAIN_DIRECT3D,
            CELESTIAL_ECLIPSE,
            0x000000,
            0xff0000,
        ),
        lo(200, 128, 0, 0, 0, DIR_SUN, 15, 15),
    ],
    // L7: PORTAL/RIFT - hellfire dimensional tear on wall
    [
        hi_meta(
            OP_PORTAL,
            REGION_WALLS,
            BLEND_ADD,
            DOMAIN_TANGENT_LOCAL,
            PORTAL_RIFT,
            0xff2200,
            0x000000,
        ),
        lo(180, 128, 64, 0, 0, DIR_UP, 15, 0),
    ],
];

// -----------------------------------------------------------------------------
// Preset 3: "Frozen Tundra" - Arctic survival
// -----------------------------------------------------------------------------
// L0: RAMP (sky=#c8e0f0, floor=#f8f8ff, walls=#a0c8e0)
// L1: PLANE/STONE (ice white #e8f4ff)
// L2: CELL/SHATTER (pale cyan #d0f0ff)
// L3: FLOW (white #ffffff, snow)
// L4: SCATTER (white #ffffff, flakes)
// L5: ATMOSPHERE/RAYLEIGH (arctic blue #b0d8f0)
// L6: NOP_LAYER (disabled - reserved for aurora effect)
// L7: APERTURE/CIRCLE (BLEND_MIN, icy vignette #d0f0ff)
const PRESET_FROZEN_TUNDRA: [[u64; 2]; 8] = [
    // L0: RAMP - pale blue sky, white floor, ice blue walls
    [
        hi(OP_RAMP, REGION_ALL, BLEND_LERP, 0, 0xc8e0f0, 0xf8f8ff),
        lo(255, 0xa0, 0xc8, 0xe0, 0, DIR_UP, 15, 15),
    ],
    // L1: PLANE/STONE - frozen ground texture (ice white)
    [
        hi_meta(
            OP_PLANE,
            REGION_FLOOR,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            PLANE_STONE,
            0xe8f4ff,
            0x000000,
        ),
        lo(150, 128, 0, 0, 0, DIR_UP, 15, 0),
    ],
    // L2: CELL/SHATTER - cracked ice pattern
    [
        hi_meta(
            OP_CELL,
            REGION_FLOOR,
            BLEND_MULTIPLY,
            DOMAIN_DIRECT3D,
            CELL_SHATTER,
            0xd0f0ff,
            0x000000,
        ),
        lo(140, 128, 0, 0, 0, DIR_UP, 15, 0),
    ],
    // L3: FLOW - slow drifting snow
    [
        hi(OP_FLOW, REGION_ALL, BLEND_ADD, 0, 0xffffff, 0x000000),
        lo(120, 128, 80, 3, 0, DIR_DOWN, 15, 0),
    ],
    // L4: SCATTER - snowfall with downward drift
    [
        hi(OP_SCATTER, REGION_ALL, BLEND_ADD, 0, 0xffffff, 0x000000),
        lo(160, 128, 100, 2, 0, DIR_DOWN, 15, 0), // param_c=2: gentle swirl
    ],
    // L5: ATMOSPHERE/RAYLEIGH - crisp cold air (arctic blue)
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
        lo(100, 128, 0, 0, 0, DIR_UP, 15, 0),
    ],
    // L6: APERTURE/BARS - icy prison bars effect
    [
        hi_meta(
            OP_APERTURE,
            REGION_ALL,
            BLEND_MULTIPLY,
            0,
            APERTURE_BARS,
            0xe0f0ff,
            0x000000,
        ),
        lo(80, 64, 0, 0, 0, DIR_UP, 15, 0),
    ],
    // L7: APERTURE/CIRCLE - icy vignette effect (pale cyan with min blend)
    [
        hi_meta(
            OP_APERTURE,
            REGION_ALL,
            BLEND_MIN,
            0,
            APERTURE_CIRCLE,
            0xd0f0ff,
            0x000000,
        ),
        lo(140, 128, 0, 0, 0, DIR_UP, 15, 0),
    ],
];

// -----------------------------------------------------------------------------
// Preset 4: "Alien Jungle" - Sci-fi nature
// -----------------------------------------------------------------------------
// L0: RAMP (sky=#3a0050, floor=#002020, walls=#004040)
// L1: SILHOUETTE/FOREST (dark teal #001818)
// L2: PATCHES/BLOBS (cyan #00ffaa)
// L3: CELL/RADIAL (deep purple #200040)
// L4: SCATTER (cyan #00ffcc, spores)
// L5: VEIL/CURTAINS (purple #8000ff)
// L6: ATMOSPHERE/ALIEN (green #004020)
// L7: FLOW/CAUSTIC (cyan #00ddcc)
const PRESET_ALIEN_JUNGLE: [[u64; 2]; 8] = [
    // L0: RAMP - purple sky, bioluminescent floor, teal walls
    [
        hi(OP_RAMP, REGION_ALL, BLEND_LERP, 0, 0x3a0050, 0x002020),
        lo(255, 0x00, 0x40, 0x40, 0, DIR_UP, 15, 15),
    ],
    // L1: SILHOUETTE/FOREST - alien tree silhouettes (dark teal)
    [
        hi_meta(
            OP_SILHOUETTE,
            REGION_WALLS,
            BLEND_MULTIPLY,
            DOMAIN_DIRECT3D,
            SILHOUETTE_FOREST,
            0x001818,
            0x000000,
        ),
        lo(180, 128, 0, 0, 0, DIR_UP, 15, 0),
    ],
    // L2: PATCHES/BLOBS - glowing fungal patches (bioluminescent cyan)
    [
        hi_meta(
            OP_PATCHES,
            REGION_ALL,
            BLEND_ADD,
            DOMAIN_AXIS_CYL,
            PATCHES_STREAKS,
            0x00ffaa,
            0x000000,
        ),
        lo(160, 128, 64, 0, 0, DIR_UP, 15, 0),
    ],
    // L3: CELL/RADIAL - organic radial cell structure (deep purple)
    [
        hi_meta(
            OP_CELL,
            REGION_ALL,
            BLEND_MULTIPLY,
            DOMAIN_DIRECT3D,
            CELL_RADIAL,
            0x200040,
            0x000000,
        ),
        lo(140, 128, 0, 0, 0, DIR_UP, 15, 0),
    ],
    // L4: SCATTER - floating spores (cyan)
    [
        hi(OP_SCATTER, REGION_ALL, BLEND_ADD, 0, 0x00ffcc, 0x000000),
        lo(170, 100, 80, 3, 0, DIR_UP, 15, 0), // param_c=3: spore drift
    ],
    // L5: VEIL/CURTAINS - bioluminescent hanging vines (purple)
    [
        hi_meta(
            OP_VEIL,
            REGION_ALL,
            BLEND_ADD,
            DOMAIN_AXIS_CYL,
            VEIL_CURTAINS,
            0x8000ff,
            0x000000,
        ),
        lo(150, 128, 64, 0, 0, DIR_DOWN, 15, 0),
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
        lo(80, 128, 0, 0, 0, DIR_UP, 15, 0),
    ],
    // L7: FLOW - rippling bioluminescence (cyan caustics)
    [
        hi(OP_FLOW, REGION_FLOOR, BLEND_ADD, 0, 0x00ddcc, 0x000000),
        lo(140, 128, 100, 0, 0, DIR_UP, 15, 0),
    ],
];

// -----------------------------------------------------------------------------
// Preset 5: "Gothic Cathedral" - Dark fantasy
// -----------------------------------------------------------------------------
// L0: RAMP (sky=#0a0a20, floor=#1a1a1a, walls=#202020)
// L1: APERTURE/ARCH (gothic arch window frame)
// L2: GRID (dark stone #303030, walls)
// L3: CELL/BRICK (gray #282828)
// L4: TRACE/LEAD_LINES (black #000000)
// L5: LOBE (golden #ffd700)
// L6: SCATTER (gold #ffcc00, dust)
// L7: ATMOSPHERE/MIE (incense #302820)
const PRESET_GOTHIC_CATHEDRAL: [[u64; 2]; 8] = [
    // L0: RAMP - deep blue sky, stone floor, dark gray walls
    [
        hi(OP_RAMP, REGION_ALL, BLEND_LERP, 0, 0x0a0a20, 0x1a1a1a),
        lo(255, 0x20, 0x20, 0x20, 0, DIR_UP, 15, 15),
    ],
    // L1: APERTURE/ARCH - gothic arch window frame
    [
        hi_meta(
            OP_APERTURE,
            REGION_WALLS,
            BLEND_MULTIPLY,
            DOMAIN_DIRECT3D,
            APERTURE_ARCH,
            0x000000,
            0x000000,
        ),
        lo(180, 128, 0, 0, 0, DIR_UP, 15, 0),
    ],
    // L2: GRID - gothic window frames (dark stone)
    [
        hi(OP_GRID, REGION_WALLS, BLEND_MULTIPLY, 0, 0x303030, 0x000000),
        lo(150, 64, 0, 0, 0, DIR_UP, 15, 0),
    ],
    // L3: CELL/BRICK - stone wall texture
    [
        hi_meta(
            OP_CELL,
            REGION_WALLS,
            BLEND_MULTIPLY,
            DOMAIN_DIRECT3D,
            CELL_BRICK,
            0x282828,
            0x000000,
        ),
        lo(140, 128, 0, 0, 0, DIR_UP, 15, 0),
    ],
    // L4: TRACE/LEAD_LINES - stained glass leading (black)
    [
        hi_meta(
            OP_TRACE,
            REGION_WALLS,
            BLEND_MULTIPLY,
            DOMAIN_TANGENT_LOCAL,
            TRACE_LEAD_LINES,
            0x000000,
            0x000000,
        ),
        lo(160, 128, 0, 0, 0, DIR_UP, 15, 0),
    ],
    // L5: LOBE - shaft of divine golden light from above
    [
        hi(OP_LOBE, REGION_ALL, BLEND_ADD, 0, 0xffd700, 0x000000),
        lo(180, 128, 0, 0, 2, DIR_SUN, 15, 0), // param_d=2: subtle pulse
    ],
    // L6: SCATTER - golden dust motes in light beam
    [
        hi(OP_SCATTER, REGION_ALL, BLEND_ADD, 0, 0xffcc00, 0x000000),
        lo(140, 80, 40, 2, 0, DIR_DOWN, 15, 0), // param_c=2: gentle drift
    ],
    // L7: ATMOSPHERE/MIE - incense haze (smoky interior)
    [
        hi_meta(
            OP_ATMOSPHERE,
            REGION_ALL,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            ATMO_MIE,
            0x302820,
            0x000000,
        ),
        lo(100, 150, 0, 0, 0, DIR_UP, 15, 0),
    ],
];

// -----------------------------------------------------------------------------
// Preset 6: "Ocean Depths" - Underwater
// -----------------------------------------------------------------------------
// L0: RAMP (sky=#001030, floor=#203040, walls=#002848)
// L1: PLANE/WATER (blue #004080)
// L2: FLOW/CAUSTIC (cyan #00a0c0)
// L3: SCATTER (blue-green #40a0a0, particles)
// L4: VEIL/SHARDS (pale blue #80c0e0)
// L5: PATCHES/ISLANDS (teal #004050, reef formations)
// L6: ATMOSPHERE/ABSORPTION (deep blue #000820)
// L7: DECAL (circle, bioluminescent #00ffaa)
const PRESET_OCEAN_DEPTHS: [[u64; 2]; 8] = [
    // L0: RAMP - dark blue sky, sandy floor, deep teal walls
    [
        hi(OP_RAMP, REGION_ALL, BLEND_LERP, 0, 0x001030, 0x203040),
        lo(255, 0x00, 0x28, 0x48, 0, DIR_UP, 15, 15),
    ],
    // L1: PLANE/WATER - rippling caustic floor (blue)
    [
        hi_meta(
            OP_PLANE,
            REGION_FLOOR,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            PLANE_WATER,
            0x004080,
            0x000000,
        ),
        lo(160, 128, 80, 0, 0, DIR_UP, 15, 0),
    ],
    // L2: FLOW - animated caustic light patterns (cyan)
    [
        hi(OP_FLOW, REGION_ALL, BLEND_ADD, 0, 0x00a0c0, 0x000000),
        lo(140, 128, 100, 0, 0, DIR_UP, 15, 0),
    ],
    // L3: SCATTER - floating particles/plankton (blue-green)
    [
        hi(OP_SCATTER, REGION_ALL, BLEND_ADD, 0, 0x40a0a0, 0x000000),
        lo(150, 100, 60, 2, 0, DIR_UP, 15, 0), // param_c=2: plankton drift
    ],
    // L4: VEIL/SHARDS - light shafts from surface (pale blue)
    [
        hi_meta(
            OP_VEIL,
            REGION_ALL,
            BLEND_ADD,
            DOMAIN_AXIS_CYL,
            VEIL_SHARDS,
            0x80c0e0,
            0x000000,
        ),
        lo(160, 128, 0, 0, 0, DIR_DOWN, 15, 0),
    ],
    // L5: PATCHES/ISLANDS - reef/island formations (teal)
    [
        hi_meta(
            OP_PATCHES,
            REGION_FLOOR,
            BLEND_MULTIPLY,
            DOMAIN_DIRECT3D,
            PATCHES_ISLANDS,
            0x004050,
            0x000000,
        ),
        lo(140, 128, 48, 0, 0, DIR_UP, 15, 0),
    ],
    // L6: ATMOSPHERE/ABSORPTION - water depth fog (deep blue)
    [
        hi_meta(
            OP_ATMOSPHERE,
            REGION_ALL,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            ATMO_ABSORPTION,
            0x000820,
            0x000000,
        ),
        lo(180, 180, 0, 0, 0, DIR_UP, 15, 0),
    ],
    // L7: DECAL - bioluminescent creature (circle, cyan-green)
    [
        hi(OP_DECAL, REGION_WALLS, BLEND_ADD, 0, 0x00ffaa, 0x000000),
        lo(160, 64, 0, 0, 3, DIR_UP, 15, 0), // param_d=3: pulse glow
    ],
];

// -----------------------------------------------------------------------------
// Preset 7: "Void Station" - Sci-fi space station
// -----------------------------------------------------------------------------
// L0: RAMP (sky=#000008, floor=#101018, walls=#181820)
// L1: SPLIT/HALF (blue #002040 / gray #202028)
// L2: GRID (blue #0044aa, walls)
// L3: CELL/GRID (dark blue #080820)
// L4: SCATTER (white #ffffff, stars through viewport)
// L5: DECAL (rect, green #00ff00 status indicator)
// L6: APERTURE/IRREGULAR (damaged viewport frame)
// L7: CELESTIAL/BINARY (binary star system #00aa88)
const PRESET_VOID_STATION: [[u64; 2]; 8] = [
    // L0: RAMP - near-black sky, dark metal floor, gunmetal walls
    [
        hi(OP_RAMP, REGION_ALL, BLEND_LERP, 0, 0x000008, 0x101018),
        lo(255, 0x18, 0x18, 0x20, 0, DIR_UP, 15, 15),
    ],
    // L1: SPLIT/HALF - two-tone walls (blue / gray division)
    [
        hi_meta(
            OP_SPLIT,
            REGION_WALLS,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            SPLIT_HALF,
            0x002040,
            0x202028,
        ),
        lo(180, 128, 0, 0, 0, DIR_UP, 15, 15),
    ],
    // L2: GRID - technical blue panel lines on walls
    [
        hi(OP_GRID, REGION_WALLS, BLEND_ADD, 0, 0x0044aa, 0x000000),
        lo(140, 48, 0, 2, 0, DIR_UP, 15, 0), // param_c=2: scan line effect
    ],
    // L3: CELL/GRID - floor grating pattern (dark blue)
    [
        hi_meta(
            OP_CELL,
            REGION_FLOOR,
            BLEND_MULTIPLY,
            DOMAIN_DIRECT3D,
            CELL_GRID,
            0x080820,
            0x000000,
        ),
        lo(150, 128, 0, 0, 0, DIR_UP, 15, 0),
    ],
    // L4: SCATTER - distant stars visible through viewport (white)
    [
        hi(OP_SCATTER, REGION_SKY, BLEND_ADD, 0, 0xffffff, 0x000000),
        lo(200, 180, 0, 1, 0, DIR_UP, 15, 0), // param_c=1: subtle twinkle
    ],
    // L5: DECAL - green status indicator rectangle
    [
        hi(OP_DECAL, REGION_WALLS, BLEND_ADD, 0, 0x00ff00, 0x000000),
        lo(180, 32, 0, 0, 4, DIR_UP, 15, 0), // param_d=4: blink animation
    ],
    // L6: APERTURE/IRREGULAR - damaged/irregular viewport frame
    [
        hi_meta(
            OP_APERTURE,
            REGION_SKY,
            BLEND_MULTIPLY,
            DOMAIN_DIRECT3D,
            APERTURE_IRREGULAR,
            0x000000,
            0x000000,
        ),
        lo(200, 128, 0, 0, 0, DIR_UP, 15, 0),
    ],
    // L7: CELESTIAL/BINARY - binary star system visible outside
    [
        hi_meta(
            OP_CELESTIAL,
            REGION_SKY,
            BLEND_ADD,
            DOMAIN_DIRECT3D,
            CELESTIAL_BINARY,
            0x00aa88,
            0x000000,
        ),
        lo(180, 100, 0, 0, 0, DIR_SUN, 15, 0),
    ],
];

// -----------------------------------------------------------------------------
// Preset 8: "Desert Mirage" - Middle Eastern fantasy
// -----------------------------------------------------------------------------
// L0: RAMP (sky=#f0e8d0, floor=#d4b896, walls=#c8a878)
// L1: SILHOUETTE/DUNES (golden #b89860, sand dunes)
// L2: PLANE/SAND (warm sand #d8c090)
// L3: FLOW (heat shimmer #f8f0e0, low intensity)
// L4: SCATTER (sand #c8b080, dust)
// L5: CELESTIAL/SUN (blazing white #ffffd8)
// L6: ATMOSPHERE/RAYLEIGH (haze #e8d8c0)
// L7: SECTOR/BOX (BLEND_HSV_MOD, warm golden #e8c090 / #d0a070)
const PRESET_DESERT_MIRAGE: [[u64; 2]; 8] = [
    // L0: RAMP - bleached sky, sand floor, tan walls
    [
        hi(OP_RAMP, REGION_ALL, BLEND_LERP, 0, 0xf0e8d0, 0xd4b896),
        lo(255, 0xc8, 0xa8, 0x78, 0, DIR_UP, 15, 15),
    ],
    // L1: SILHOUETTE/DUNES - rolling desert sand dunes (golden)
    [
        hi_meta(
            OP_SILHOUETTE,
            REGION_WALLS,
            BLEND_MULTIPLY,
            DOMAIN_DIRECT3D,
            SILHOUETTE_DUNES,
            0xb89860,
            0x000000,
        ),
        lo(160, 128, 0, 0, 0, DIR_UP, 15, 0),
    ],
    // L2: PLANE/SAND - textured desert floor (warm sand)
    [
        hi_meta(
            OP_PLANE,
            REGION_FLOOR,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            PLANE_SAND,
            0xd8c090,
            0x000000,
        ),
        lo(150, 128, 0, 0, 0, DIR_UP, 15, 0),
    ],
    // L3: FLOW - heat shimmer effect (subtle wavering)
    [
        hi(OP_FLOW, REGION_ALL, BLEND_ADD, 0, 0xf8f0e0, 0x000000),
        lo(60, 128, 40, 0, 0, DIR_UP, 8, 0),
    ],
    // L4: SCATTER - blowing dust particles (sand color)
    [
        hi(OP_SCATTER, REGION_ALL, BLEND_ADD, 0, 0xc8b080, 0x000000),
        lo(120, 100, 60, 2, 0, DIR_DOWN, 12, 0), // param_c=2: dust swirl
    ],
    // L5: CELESTIAL/SUN - blazing desert sun (intense white-yellow)
    [
        hi_meta(
            OP_CELESTIAL,
            REGION_SKY,
            BLEND_ADD,
            DOMAIN_DIRECT3D,
            CELESTIAL_SUN,
            0xffffd8,
            0x000000,
        ),
        lo(220, 128, 0, 0, 0, DIR_SUN, 15, 0),
    ],
    // L6: ATMOSPHERE/RAYLEIGH - heat haze (warm tan)
    [
        hi_meta(
            OP_ATMOSPHERE,
            REGION_ALL,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            ATMO_RAYLEIGH,
            0xe8d8c0,
            0x000000,
        ),
        lo(100, 128, 0, 0, 0, DIR_UP, 15, 0),
    ],
    // L7: SECTOR/BOX - warm golden box sector with HSV modulation
    [
        hi_meta(
            OP_SECTOR,
            REGION_ALL,
            BLEND_HSV_MOD,
            0,
            SECTOR_BOX,
            0xe8c090,
            0xd0a070,
        ),
        lo(80, 128, 0, 0, 0, DIR_UP, 15, 15),
    ],
];

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
const PRESET_NEON_ARCADE: [[u64; 2]; 8] = [
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
        lo(160, 150, 0, 1, 0, DIR_UP, 15, 0), // param_c=1: star twinkle
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
const PRESET_STORM_FRONT: [[u64; 2]; 8] = [
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
        hi(OP_SCATTER, REGION_ALL, BLEND_ADD, 0, 0x8090a0, 0x000000),
        lo(180, 140, 180, 3, 0, DIR_DOWN, 15, 0), // param_c=3: rain streaks
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
const PRESET_CRYSTAL_CAVERN: [[u64; 2]; 8] = [
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
        lo(170, 120, 0, 2, 0, DIR_UP, 15, 0), // param_c=2: crystal sparkle
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
const PRESET_WAR_ZONE: [[u64; 2]; 8] = [
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
        hi(OP_SCATTER, REGION_ALL, BLEND_ADD, 0, 0xff6600, 0x000000),
        lo(180, 100, 80, 3, 0, DIR_UP, 15, 0), // param_c=3: ember rise
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

// -----------------------------------------------------------------------------
// Preset 13: "Enchanted Grove" - Fairy tale forest
// -----------------------------------------------------------------------------
// L0: RAMP (sky=#fff8d0, floor=#204020, walls=#1a3820)
// L1: SILHOUETTE/FOREST (deep green #0a2010)
// L2: PLANE/GRASS (vibrant green #308030, forest floor)
// L3: VEIL/CURTAINS (green #40a040, hanging moss)
// L4: SCATTER (gold #ffdd00, fairy dust)
// L5: PATCHES/BLOBS (soft yellow #fff080, dappled sunlight)
// L6: LOBE (golden #ffd700, sunbeam through canopy)
// L7: FLOW (green #60a060, gentle leaf movement)
const PRESET_ENCHANTED_GROVE: [[u64; 2]; 8] = [
    // L0: RAMP - golden sky, mossy floor, forest green walls
    [
        hi(OP_RAMP, REGION_ALL, BLEND_LERP, 0, 0xfff8d0, 0x204020),
        lo(255, 0x1a, 0x38, 0x20, 0, DIR_UP, 15, 15),
    ],
    // L1: SILHOUETTE/MOUNTAINS - distant mountain backdrop (deep green)
    [
        hi_meta(
            OP_SILHOUETTE,
            REGION_WALLS,
            BLEND_MULTIPLY,
            DOMAIN_DIRECT3D,
            SILHOUETTE_MOUNTAINS,
            0x0a2010,
            0x000000,
        ),
        lo(180, 128, 0, 0, 0, DIR_UP, 15, 0),
    ],
    // L2: PLANE/GRASS - lush forest floor (vibrant green)
    [
        hi_meta(
            OP_PLANE,
            REGION_FLOOR,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            PLANE_GRASS,
            0x308030,
            0x000000,
        ),
        lo(160, 128, 0, 0, 0, DIR_UP, 15, 0),
    ],
    // L3: VEIL/CURTAINS - hanging moss/vines (green)
    [
        hi_meta(
            OP_VEIL,
            REGION_ALL,
            BLEND_ADD,
            DOMAIN_AXIS_CYL,
            VEIL_CURTAINS,
            0x40a040,
            0x000000,
        ),
        lo(140, 128, 64, 0, 0, DIR_DOWN, 15, 0),
    ],
    // L4: SCATTER - fairy dust particles (gold)
    [
        hi(OP_SCATTER, REGION_ALL, BLEND_ADD, 0, 0xffdd00, 0x000000),
        lo(170, 100, 60, 3, 0, DIR_UP, 15, 0), // param_c=3: fairy sparkle
    ],
    // L5: PATCHES/BLOBS - dappled sunlight (soft yellow)
    [
        hi_meta(
            OP_PATCHES,
            REGION_ALL,
            BLEND_ADD,
            DOMAIN_DIRECT3D,
            PATCHES_BLOBS,
            0xfff080,
            0x000000,
        ),
        lo(140, 128, 48, 0, 0, DIR_UP, 15, 0),
    ],
    // L6: LOBE - warm sunbeam through canopy (golden)
    [
        hi(OP_LOBE, REGION_ALL, BLEND_ADD, 0, 0xffd700, 0x000000),
        lo(180, 128, 0, 0, 2, DIR_SUN, 15, 0), // param_d=2: beam sway
    ],
    // L7: FLOW - gentle leaf movement (green)
    [
        hi(OP_FLOW, REGION_ALL, BLEND_ADD, 0, 0x60a060, 0x000000),
        lo(100, 128, 60, 0, 0, DIR_UP, 15, 0),
    ],
];

// -----------------------------------------------------------------------------
// Preset 14: "Astral Void" - Cosmic/abstract
// -----------------------------------------------------------------------------
// L0: RAMP (sky=#000004, floor=#080010, walls=#100020)
// L1: FLOW (nebula purple #4000a0, swirling gases)
// L2: SCATTER (white #ffffff, dense starfield)
// L3: CELESTIAL/PLANET (blue-green #4080a0, terrestrial planet)
// L4: CELESTIAL/RINGED (pale gold #d0c080)
// L5: PORTAL/VORTEX (blue #0080ff, cosmic vortex)
// L6: TRACE/FILAMENTS (white #ffffff, energy streams)
// L7: ATMOSPHERE/ALIEN (purple #200040)
const PRESET_ASTRAL_VOID: [[u64; 2]; 8] = [
    // L0: RAMP - void black sky, deep purple floor, indigo walls
    [
        hi(OP_RAMP, REGION_ALL, BLEND_LERP, 0, 0x000004, 0x080010),
        lo(255, 0x10, 0x00, 0x20, 0, DIR_UP, 15, 15),
    ],
    // L1: FLOW - swirling cosmic gases (nebula purple)
    [
        hi(OP_FLOW, REGION_ALL, BLEND_ADD, 0, 0x4000a0, 0x000000),
        lo(140, 128, 80, 4, 0, DIR_UP, 15, 0),
    ],
    // L2: SCATTER - dense starfield (white)
    [
        hi(OP_SCATTER, REGION_ALL, BLEND_ADD, 0, 0xffffff, 0x000000),
        lo(200, 200, 0, 1, 0, DIR_UP, 15, 0), // param_c=1: star shimmer
    ],
    // L3: CELESTIAL/PLANET - terrestrial planet (blue-green)
    [
        hi_meta(
            OP_CELESTIAL,
            REGION_SKY,
            BLEND_ADD,
            DOMAIN_DIRECT3D,
            CELESTIAL_GAS_GIANT,
            0xff6040,
            0x000000,
        ),
        lo(180, 100, 0, 0, 0, DIR_SUN, 15, 0),
    ],
    // L4: CELESTIAL/RINGED - ringed planet in distance (pale gold)
    [
        hi_meta(
            OP_CELESTIAL,
            REGION_SKY,
            BLEND_ADD,
            DOMAIN_DIRECT3D,
            CELESTIAL_RINGED,
            0xd0c080,
            0x000000,
        ),
        lo(160, 80, 0, 0, 0, DIR_SUNSET, 15, 0),
    ],
    // L5: PORTAL/VORTEX - cosmic swirling vortex (blue)
    [
        hi_meta(
            OP_PORTAL,
            REGION_WALLS,
            BLEND_ADD,
            DOMAIN_TANGENT_LOCAL,
            PORTAL_TEAR,
            0x0080ff,
            0x000000,
        ),
        lo(180, 128, 64, 0, 0, DIR_UP, 15, 0),
    ],
    // L6: PORTAL/VORTEX - cosmic swirling vortex (white energy)
    [
        hi_meta(
            OP_PORTAL,
            REGION_ALL,
            BLEND_ADD,
            DOMAIN_TANGENT_LOCAL,
            PORTAL_VORTEX,
            0xffffff,
            0x8040ff,
        ),
        lo(160, 128, 64, 0, 0, DIR_UP, 15, 15),
    ],
    // L7: NOP - void boundary (intentionally empty)
    NOP_LAYER,
];

// -----------------------------------------------------------------------------
// Preset 15: "Toxic Wasteland" - Post-apocalyptic industrial
// -----------------------------------------------------------------------------
// L0: RAMP (sky=#304010, floor=#202008, walls=#283018)
// L1: SILHOUETTE/INDUSTRIAL (black #000000)
// L2: PLANE/TILES (rust #483820)
// L3: PATCHES/STATIC (green #40a000, radioactive)
// L4: CELL/HEX (toxic yellow #a0a000, hazmat)
// L5: VEIL/PILLARS (green smoke #408020, toxic fumes)
// L6: SCATTER (yellow-green #a0c040, toxic particles)
// L7: ATMOSPHERE/ALIEN (toxic green #203008)
const PRESET_TOXIC_WASTELAND: [[u64; 2]; 8] = [
    // L0: RAMP - sickly green sky, toxic floor, corroded walls
    [
        hi(OP_RAMP, REGION_ALL, BLEND_LERP, 0, 0x304010, 0x202008),
        lo(255, 0x28, 0x30, 0x18, 0, DIR_UP, 15, 15),
    ],
    // L1: SILHOUETTE/INDUSTRIAL - rusted factory smokestacks (black)
    [
        hi_meta(
            OP_SILHOUETTE,
            REGION_WALLS,
            BLEND_MULTIPLY,
            DOMAIN_DIRECT3D,
            SILHOUETTE_INDUSTRIAL,
            0x000000,
            0x000000,
        ),
        lo(180, 128, 0, 0, 0, DIR_UP, 15, 0),
    ],
    // L2: PLANE/TILES - cracked industrial floor tiles (rust)
    [
        hi_meta(
            OP_PLANE,
            REGION_FLOOR,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            PLANE_TILES,
            0x483820,
            0x000000,
        ),
        lo(160, 128, 0, 0, 0, DIR_UP, 15, 0),
    ],
    // L3: PATCHES/STATIC - radioactive patches (green)
    [
        hi_meta(
            OP_PATCHES,
            REGION_ALL,
            BLEND_ADD,
            DOMAIN_DIRECT3D,
            PATCHES_STATIC,
            0x40a000,
            0x000000,
        ),
        lo(150, 128, 64, 0, 0, DIR_UP, 15, 0),
    ],
    // L4: CELL/HEX - hazmat pattern (toxic yellow)
    [
        hi_meta(
            OP_CELL,
            REGION_ALL,
            BLEND_ADD,
            DOMAIN_DIRECT3D,
            CELL_HEX,
            0xa0a000,
            0x000000,
        ),
        lo(140, 128, 0, 0, 0, DIR_UP, 15, 0),
    ],
    // L5: VEIL/PILLARS - rising toxic fumes (green smoke)
    [
        hi_meta(
            OP_VEIL,
            REGION_ALL,
            BLEND_ADD,
            DOMAIN_AXIS_CYL,
            VEIL_PILLARS,
            0x408020,
            0x000000,
        ),
        lo(140, 128, 80, 0, 0, DIR_UP, 15, 0),
    ],
    // L6: SCATTER - toxic particles (yellow-green)
    [
        hi(OP_SCATTER, REGION_ALL, BLEND_ADD, 0, 0xa0c040, 0x000000),
        lo(160, 100, 60, 2, 0, DIR_UP, 15, 0), // param_c=2: toxic swirl
    ],
    // L7: ATMOSPHERE/ALIEN - poisonous atmosphere (toxic green)
    [
        hi_meta(
            OP_ATMOSPHERE,
            REGION_ALL,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            ATMO_ALIEN,
            0x203008,
            0x000000,
        ),
        lo(140, 160, 0, 0, 0, DIR_UP, 15, 0),
    ],
];

// -----------------------------------------------------------------------------
// Preset 16: "Moonlit Graveyard" - Gothic horror
// -----------------------------------------------------------------------------
// L0: RAMP (sky=#0a0a1a, floor=#101010, walls=#181820)
// L1: SILHOUETTE/SPIRES (black #000000, gothic spires)
// L2: PLANE/STONE (gray #282828, weathered path)
// L3: PATCHES/MEMBRANE (dark green #0a1a0a, creeping moss)
// L4: SCATTER (pale blue #8090a0, mist particles)
// L5: CELESTIAL/MOON (pale silver #e0e8f0)
// L6: VEIL/CURTAINS (gray #404050, hanging mist)
// L7: PORTAL/CRACK (ghostly cracks #404050)
const PRESET_MOONLIT_GRAVEYARD: [[u64; 2]; 8] = [
    // L0: RAMP - midnight blue sky, dark earth floor, slate walls
    [
        hi(OP_RAMP, REGION_ALL, BLEND_LERP, 0, 0x0a0a1a, 0x101010),
        lo(255, 0x18, 0x18, 0x20, 0, DIR_UP, 15, 15),
    ],
    // L1: SILHOUETTE/SPIRES - gothic tombstone spires (black)
    [
        hi_meta(
            OP_SILHOUETTE,
            REGION_WALLS,
            BLEND_MULTIPLY,
            DOMAIN_DIRECT3D,
            SILHOUETTE_SPIRES,
            0x000000,
            0x000000,
        ),
        lo(180, 128, 0, 0, 0, DIR_UP, 15, 0),
    ],
    // L2: PLANE/STONE - weathered stone path (gray)
    [
        hi_meta(
            OP_PLANE,
            REGION_FLOOR,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            PLANE_STONE,
            0x282828,
            0x000000,
        ),
        lo(150, 128, 0, 0, 0, DIR_UP, 15, 0),
    ],
    // L3: PATCHES/MEMBRANE - creeping moss (dark green)
    [
        hi_meta(
            OP_PATCHES,
            REGION_ALL,
            BLEND_MULTIPLY,
            DOMAIN_DIRECT3D,
            PATCHES_MEMBRANE,
            0x0a1a0a,
            0x000000,
        ),
        lo(140, 128, 48, 0, 0, DIR_UP, 15, 0),
    ],
    // L4: SCATTER - floating mist particles (pale blue)
    [
        hi(OP_SCATTER, REGION_ALL, BLEND_ADD, 0, 0x8090a0, 0x000000),
        lo(160, 100, 60, 2, 0, DIR_UP, 15, 0), // param_c=2: mist drift
    ],
    // L5: CELESTIAL/MOON - full moon (pale silver)
    [
        hi_meta(
            OP_CELESTIAL,
            REGION_SKY,
            BLEND_ADD,
            DOMAIN_DIRECT3D,
            CELESTIAL_MOON,
            0xe0e8f0,
            0x000000,
        ),
        lo(200, 128, 0, 0, 0, DIR_SUN, 15, 0),
    ],
    // L6: VEIL/CURTAINS - hanging mist (gray)
    [
        hi_meta(
            OP_VEIL,
            REGION_ALL,
            BLEND_ADD,
            DOMAIN_AXIS_CYL,
            VEIL_CURTAINS,
            0x404050,
            0x000000,
        ),
        lo(140, 128, 80, 0, 0, DIR_DOWN, 15, 0),
    ],
    // L7: PORTAL/CRACK - ghostly dimensional cracks (gray)
    [
        hi_meta(
            OP_PORTAL,
            REGION_ALL,
            BLEND_ADD,
            DOMAIN_TANGENT_LOCAL,
            PORTAL_CRACK,
            0x404050,
            0x000000,
        ),
        lo(140, 180, 0, 0, 0, DIR_UP, 15, 0),
    ],
];

// -----------------------------------------------------------------------------
// Preset 17: "Volcanic Core" - Primordial/elemental
// -----------------------------------------------------------------------------
// L0: RAMP (sky=#100800, floor=#401000, walls=#201008)
// L1: PLANE/STONE (volcanic black #181008)
// L2: TRACE/CRACKS (orange #ff4000, lava veins)
// L3: SPLIT/CROSS (dark red #300800 / bright orange #ff4000)
// L4: FLOW (orange-red #ff2800, churning lava)
// L5: SCATTER (bright orange #ff8000, rising sparks)
// L6: LOBE (deep red #ff2000, heat glow from below)
// L7: ATMOSPHERE/ABSORPTION (smoke black #100800)
const PRESET_VOLCANIC_CORE: [[u64; 2]; 8] = [
    // L0: RAMP - black sky, magma floor, obsidian walls
    [
        hi(OP_RAMP, REGION_ALL, BLEND_LERP, 0, 0x100800, 0x401000),
        lo(255, 0x20, 0x10, 0x08, 0, DIR_UP, 15, 15),
    ],
    // L1: PLANE/HEX - hexagonal basalt columns (volcanic black)
    [
        hi_meta(
            OP_PLANE,
            REGION_FLOOR,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            PLANE_HEX,
            0x181008,
            0x000000,
        ),
        lo(160, 128, 0, 0, 0, DIR_UP, 15, 0),
    ],
    // L2: TRACE/CRACKS - lava veins (orange)
    [
        hi_meta(
            OP_TRACE,
            REGION_ALL,
            BLEND_ADD,
            DOMAIN_TANGENT_LOCAL,
            TRACE_CRACKS,
            0xff4000,
            0x000000,
        ),
        lo(200, 128, 64, 0, 0, DIR_UP, 15, 0),
    ],
    // L3: SPLIT/CROSS - volcanic cross pattern (dark red / bright orange)
    [
        hi_meta(
            OP_SPLIT,
            REGION_ALL,
            BLEND_ADD,
            0,
            SPLIT_CROSS,
            0x300800,
            0xff4000,
        ),
        lo(150, 128, 0, 0, 0, DIR_UP, 15, 15),
    ],
    // L4: FLOW - churning lava (orange-red)
    [
        hi(OP_FLOW, REGION_FLOOR, BLEND_ADD, 0, 0xff2800, 0x000000),
        lo(180, 128, 100, 0, 0, DIR_UP, 15, 0),
    ],
    // L5: SCATTER - rising sparks (bright orange)
    [
        hi(OP_SCATTER, REGION_ALL, BLEND_ADD, 0, 0xff8000, 0x000000),
        lo(180, 120, 100, 4, 0, DIR_UP, 15, 0), // param_c=4: spark flicker
    ],
    // L6: LOBE - intense heat glow from below (deep red)
    [
        hi(OP_LOBE, REGION_ALL, BLEND_ADD, 0, 0xff2000, 0x000000),
        lo(200, 128, 0, 0, 3, DIR_DOWN, 15, 0), // param_d=3: heat pulse
    ],
    // L7: ATMOSPHERE/ABSORPTION - volcanic gases (smoke black)
    [
        hi_meta(
            OP_ATMOSPHERE,
            REGION_ALL,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            ATMO_ABSORPTION,
            0x100800,
            0x000000,
        ),
        lo(160, 180, 0, 0, 0, DIR_UP, 15, 0),
    ],
];

// -----------------------------------------------------------------------------
// Preset 18: "Digital Matrix" - Cyber/virtual reality
// -----------------------------------------------------------------------------
// L0: RAMP (sky=#000000, floor=#001000, walls=#002000)
// L1: GRID (bright green #00ff00, all regions)
// L2: SCATTER (green #00ff00, falling code rain, BLEND_SCREEN for glow)
// L3: CELL/GRID (dark green #003000, data blocks)
// L4: TRACE/FILAMENTS (cyan #00ffff, data streams)
// L5: APERTURE/RECT (green #00ff00, rectangular data viewport)
// L6: APERTURE/ROUNDED_RECT (green #00aa00, rounded terminal frame)
// L7: FLOW (green #00dd00, code streaming)
const PRESET_DIGITAL_MATRIX: [[u64; 2]; 8] = [
    // L0: RAMP - black sky, dark green floor, matrix green walls
    [
        hi(OP_RAMP, REGION_ALL, BLEND_LERP, 0, 0x000000, 0x001000),
        lo(255, 0x00, 0x20, 0x00, 0, DIR_UP, 15, 15),
    ],
    // L1: GRID - digital grid (bright green, all regions)
    [
        hi(OP_GRID, REGION_ALL, BLEND_ADD, 0, 0x00ff00, 0x000000),
        lo(160, 32, 0, 3, 0, DIR_UP, 15, 0), // param_c=3: matrix scroll
    ],
    // L2: SCATTER - falling code rain with BLEND_SCREEN for bright glow
    [
        hi(OP_SCATTER, REGION_ALL, BLEND_SCREEN, 0, 0x00ff00, 0x000000),
        lo(180, 150, 200, 4, 0, DIR_DOWN, 15, 0), // param_c=4: code rain
    ],
    // L3: CELL/GRID - data block structure (dark green)
    [
        hi_meta(
            OP_CELL,
            REGION_ALL,
            BLEND_MULTIPLY,
            DOMAIN_DIRECT3D,
            CELL_GRID,
            0x003000,
            0x000000,
        ),
        lo(140, 128, 0, 0, 0, DIR_UP, 15, 0),
    ],
    // L4: PORTAL/RECT - rectangular data portal (cyan)
    [
        hi_meta(
            OP_PORTAL,
            REGION_WALLS,
            BLEND_ADD,
            DOMAIN_TANGENT_LOCAL,
            PORTAL_RECT,
            0x00ffff,
            0x000000,
        ),
        lo(170, 128, 64, 0, 0, DIR_UP, 15, 0),
    ],
    // L5: APERTURE/RECT - rectangular data viewport (green)
    [
        hi_meta(
            OP_APERTURE,
            REGION_WALLS,
            BLEND_ADD,
            DOMAIN_DIRECT3D,
            APERTURE_RECT,
            0x00ff00,
            0x000000,
        ),
        lo(160, 128, 0, 0, 0, DIR_UP, 15, 0),
    ],
    // L6: APERTURE/ROUNDED_RECT - rounded terminal frame (green)
    [
        hi_meta(
            OP_APERTURE,
            REGION_ALL,
            BLEND_MULTIPLY,
            DOMAIN_DIRECT3D,
            APERTURE_ROUNDED_RECT,
            0x00aa00,
            0x000000,
        ),
        lo(140, 128, 0, 0, 0, DIR_UP, 15, 0),
    ],
    // L7: FLOW - code streaming effect (green)
    [
        hi(OP_FLOW, REGION_ALL, BLEND_ADD, 0, 0x00dd00, 0x000000),
        lo(120, 128, 150, 0, 0, DIR_DOWN, 15, 0),
    ],
];

// -----------------------------------------------------------------------------
// Preset 19: "Noir Detective" - 1940s private eye office
// -----------------------------------------------------------------------------
// L0: RAMP (dark ceiling, worn wood floor, olive/brown walls)
// L1: SPLIT/WEDGE (venetian blind shadow stripes - iconic noir lighting)
// L2: GRID (window frame grid)
// L3: LOBE (desk lamp cone of light)
// L4: SCATTER (cigarette smoke particles)
// L5: ATMOSPHERE/MIE (smoky haze)
// L6: CELL/BRICK (subtle wall texture)
// L7: APERTURE/RECT (window frame vignette)
const PRESET_NOIR_DETECTIVE: [[u64; 2]; 8] = [
    // L0: RAMP - dark ceiling, worn wood floor, olive walls
    [
        hi(OP_RAMP, REGION_ALL, BLEND_LERP, 0, 0x101008, 0x302820),
        lo(255, 0x38, 0x34, 0x28, 0, DIR_UP, 15, 15),
    ],
    // L1: SPLIT/WEDGE - venetian blind shadows (the defining noir look)
    [
        hi_meta(
            OP_SPLIT,
            REGION_ALL,
            BLEND_MULTIPLY,
            DOMAIN_DIRECT3D,
            SPLIT_WEDGE,
            0x000000,
            0x404030,
        ),
        lo(180, 128, 0, 0, 0, DIR_SUN, 15, 15),
    ],
    // L2: GRID - window frame structure
    [
        hi(OP_GRID, REGION_WALLS, BLEND_MULTIPLY, 0, 0x202018, 0x000000),
        lo(120, 64, 0, 0, 0, DIR_UP, 15, 0),
    ],
    // L3: LOBE - desk lamp cone of warm light
    [
        hi(OP_LOBE, REGION_ALL, BLEND_ADD, 0, 0xffe0a0, 0x000000),
        lo(160, 128, 0, 0, 1, DIR_DOWN, 15, 0), // param_d=1: subtle flicker
    ],
    // L4: SCATTER - cigarette smoke particles
    [
        hi(OP_SCATTER, REGION_ALL, BLEND_ADD, 0, 0x808070, 0x000000),
        lo(100, 80, 40, 2, 0, DIR_UP, 10, 0), // param_c=2: smoke rise
    ],
    // L5: ATMOSPHERE/MIE - smoky haze filling the room
    [
        hi_meta(
            OP_ATMOSPHERE,
            REGION_ALL,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            ATMO_MIE,
            0x302820,
            0x000000,
        ),
        lo(120, 150, 0, 0, 0, DIR_UP, 15, 0),
    ],
    // L6: CELL/BRICK - subtle worn wall texture
    [
        hi_meta(
            OP_CELL,
            REGION_WALLS,
            BLEND_MULTIPLY,
            DOMAIN_DIRECT3D,
            CELL_BRICK,
            0x383028,
            0x000000,
        ),
        lo(80, 128, 0, 0, 0, DIR_UP, 15, 0),
    ],
    // L7: APERTURE/RECT - window frame vignette
    [
        hi_meta(
            OP_APERTURE,
            REGION_ALL,
            BLEND_MULTIPLY,
            DOMAIN_DIRECT3D,
            APERTURE_RECT,
            0x000000,
            0x000000,
        ),
        lo(140, 160, 0, 0, 0, DIR_UP, 15, 0),
    ],
];

// -----------------------------------------------------------------------------
// Preset 20: "Steampunk Airship" - Victorian observation deck
// -----------------------------------------------------------------------------
// L0: RAMP (amber sky, burnished brass floor, copper walls)
// L1: APERTURE/MULTI (multiple circular portholes)
// L2: CELL/HEX (riveted hex plate flooring)
// L3: GRID (brass framework girders)
// L4: CELESTIAL/SUN (setting sun through porthole)
// L5: VEIL/PILLARS (steam columns rising)
// L6: SCATTER (floating steam particles)
// L7: ATMOSPHERE/MIE (warm amber haze)
const PRESET_STEAMPUNK_AIRSHIP: [[u64; 2]; 8] = [
    // L0: RAMP - amber sunset sky, burnished brass floor, copper walls
    [
        hi(OP_RAMP, REGION_ALL, BLEND_LERP, 0, 0xffa040, 0x604020),
        lo(255, 0x80, 0x50, 0x30, 0, DIR_UP, 15, 15),
    ],
    // L1: APERTURE/MULTI - multiple circular observation portholes
    [
        hi_meta(
            OP_APERTURE,
            REGION_WALLS,
            BLEND_MULTIPLY,
            DOMAIN_DIRECT3D,
            APERTURE_MULTI,
            0x402010,
            0x000000,
        ),
        lo(200, 100, 0, 0, 0, DIR_UP, 15, 0),
    ],
    // L2: CELL/HEX - riveted hexagonal plate flooring
    [
        hi_meta(
            OP_CELL,
            REGION_FLOOR,
            BLEND_MULTIPLY,
            DOMAIN_DIRECT3D,
            CELL_HEX,
            0x503020,
            0x000000,
        ),
        lo(150, 128, 0, 0, 0, DIR_UP, 15, 0),
    ],
    // L3: GRID - brass framework and girders
    [
        hi(OP_GRID, REGION_ALL, BLEND_ADD, 0, 0xc09040, 0x000000),
        lo(100, 48, 0, 0, 0, DIR_UP, 12, 0),
    ],
    // L4: CELESTIAL/SUN - setting sun visible through porthole
    [
        hi_meta(
            OP_CELESTIAL,
            REGION_SKY,
            BLEND_ADD,
            DOMAIN_DIRECT3D,
            CELESTIAL_SUN,
            0xffc060,
            0x000000,
        ),
        lo(200, 128, 0, 0, 0, DIR_SUNSET, 15, 0),
    ],
    // L5: VEIL/PILLARS - steam columns rising from vents
    [
        hi_meta(
            OP_VEIL,
            REGION_ALL,
            BLEND_ADD,
            DOMAIN_AXIS_CYL,
            VEIL_PILLARS,
            0xfff0d0,
            0x000000,
        ),
        lo(120, 128, 60, 0, 0, DIR_UP, 10, 0),
    ],
    // L6: SCATTER - floating steam and dust particles
    [
        hi(OP_SCATTER, REGION_ALL, BLEND_ADD, 0, 0xffe8c0, 0x000000),
        lo(140, 100, 50, 2, 0, DIR_UP, 12, 0), // param_c=2: steam rise
    ],
    // L7: ATMOSPHERE/MIE - warm amber engine room haze
    [
        hi_meta(
            OP_ATMOSPHERE,
            REGION_ALL,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            ATMO_MIE,
            0x604020,
            0x000000,
        ),
        lo(100, 120, 0, 0, 0, DIR_UP, 15, 0),
    ],
];

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
const PRESET_STORMY_SHORES: [[u64; 2]; 8] = [
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
        hi(OP_SCATTER, REGION_ALL, BLEND_ADD, 0, 0x90a0b0, 0x000000),
        lo(160, 120, 80, 3, 0, DIR_UP, 12, 0), // param_c=3: spray churn
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
const PRESET_POLAR_AURORA: [[u64; 2]; 8] = [
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
        lo(200, 200, 0, 1, 0, DIR_UP, 15, 0), // param_c=1: star twinkle
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
const PRESET_SACRED_GEOMETRY: [[u64; 2]; 8] = [
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
        hi(OP_SCATTER, REGION_ALL, BLEND_ADD, 0, 0xffd040, 0x000000),
        lo(150, 100, 40, 2, 0, DIR_UP, 12, 0), // param_c=2: sacred swirl
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
const PRESET_RITUAL_CHAMBER: [[u64; 2]; 8] = [
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
        hi(OP_SCATTER, REGION_ALL, BLEND_ADD, 0, 0xff8040, 0x000000),
        lo(180, 120, 60, 3, 0, DIR_UP, 15, 0), // param_c=3: spark dance
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

// =============================================================================
// Preset Arrays
// =============================================================================

/// All presets array
pub static PRESETS: [[[u64; 2]; 8]; PRESET_COUNT] = [
    PRESET_NEON_METROPOLIS,
    PRESET_CRIMSON_HELLSCAPE,
    PRESET_FROZEN_TUNDRA,
    PRESET_ALIEN_JUNGLE,
    PRESET_GOTHIC_CATHEDRAL,
    PRESET_OCEAN_DEPTHS,
    PRESET_VOID_STATION,
    PRESET_DESERT_MIRAGE,
    PRESET_NEON_ARCADE,
    PRESET_STORM_FRONT,
    PRESET_CRYSTAL_CAVERN,
    PRESET_WAR_ZONE,
    PRESET_ENCHANTED_GROVE,
    PRESET_ASTRAL_VOID,
    PRESET_TOXIC_WASTELAND,
    PRESET_MOONLIT_GRAVEYARD,
    PRESET_VOLCANIC_CORE,
    PRESET_DIGITAL_MATRIX,
    PRESET_NOIR_DETECTIVE,
    PRESET_STEAMPUNK_AIRSHIP,
    PRESET_STORMY_SHORES,
    PRESET_POLAR_AURORA,
    PRESET_SACRED_GEOMETRY,
    PRESET_RITUAL_CHAMBER,
];

/// Preset names for display
pub const PRESET_NAMES: [&str; PRESET_COUNT] = [
    "Neon Metropolis",
    "Crimson Hellscape",
    "Frozen Tundra",
    "Alien Jungle",
    "Gothic Cathedral",
    "Ocean Depths",
    "Void Station",
    "Desert Mirage",
    "Neon Arcade",
    "Storm Front",
    "Crystal Cavern",
    "War Zone",
    "Enchanted Grove",
    "Astral Void",
    "Toxic Wasteland",
    "Moonlit Graveyard",
    "Volcanic Core",
    "Digital Matrix",
    "Noir Detective",
    "Steampunk Airship",
    "Stormy Shores",
    "Polar Aurora",
    "Sacred Geometry",
    "Ritual Chamber",
];

pub const PRESET_COUNT: usize = 24;
