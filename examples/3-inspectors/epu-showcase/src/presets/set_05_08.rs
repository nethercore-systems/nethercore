//! Preset set 05-08

#[allow(unused_imports)]
use crate::constants::*;

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
pub(super) const PRESET_GOTHIC_CATHEDRAL: [[u64; 2]; 8] = [
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
        hi_meta(
            OP_SCATTER,
            REGION_ALL,
            BLEND_ADD,
            DOMAIN_DIRECT3D,
            SCATTER_DUST,
            0xffcc00,
            0x000000,
        ),
        lo(140, 80, 40, 0x20, 0, DIR_DOWN, 15, 0),
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
pub(super) const PRESET_OCEAN_DEPTHS: [[u64; 2]; 8] = [
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
        hi_meta(
            OP_SCATTER,
            REGION_ALL,
            BLEND_ADD,
            DOMAIN_DIRECT3D,
            SCATTER_DUST,
            0x40a0a0,
            0x000000,
        ),
        lo(150, 100, 60, 0x20, 0, DIR_UP, 15, 0),
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
pub(super) const PRESET_VOID_STATION: [[u64; 2]; 8] = [
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
        lo(200, 180, 0, 0x10, 0, DIR_UP, 15, 0),
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
pub(super) const PRESET_DESERT_MIRAGE: [[u64; 2]; 8] = [
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
        hi_meta(
            OP_SCATTER,
            REGION_ALL,
            BLEND_ADD,
            DOMAIN_DIRECT3D,
            SCATTER_DUST,
            0xc8b080,
            0x000000,
        ),
        lo(120, 100, 60, 0x20, 0, DIR_DOWN, 12, 0),
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

