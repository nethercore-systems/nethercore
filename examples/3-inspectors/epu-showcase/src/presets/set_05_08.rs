//! Preset set 05-08

#[allow(unused_imports)]
use crate::constants::*;

// -----------------------------------------------------------------------------
// Preset 5: "Gothic Cathedral" - Candlelit stone interior
// -----------------------------------------------------------------------------
// L0: RAMP        ALL   LERP  sky=#0a0a20, floor=#1a1a1a, walls=(0x20,0x20,0x20), THRESH_INTERIOR
// L1: APERTURE/ARCH WALLS LERP  #181818 / #303028 - gothic arch frames
// L2: CELL/BRICK  WALLS LERP  #282828 / #1a1a18 - stone wall texture
// L3: TRACE/LEAD  WALLS ADD   #806040 / #000000 - stained glass leading, TANGENT_LOCAL
// L4: LOBE        ALL   ADD   #ffd700 / #000000 - divine golden light, dir=SUN
// L5: SCATTER/DUST ALL   ADD   #ffcc00 / #000000 - golden dust motes
// L6: ATMO/MIE    ALL   LERP  #302820 / #000000 - incense haze
// L7: NOP
pub(super) const PRESET_GOTHIC_CATHEDRAL: [[u64; 2]; 8] = [
    // L0: RAMP - deep blue sky, dark stone floor, gray walls
    [
        hi(OP_RAMP, REGION_ALL, BLEND_LERP, 0, 0x0a0a20, 0x1a1a1a),
        lo(180, 0x20, 0x20, 0x20, THRESH_INTERIOR, DIR_UP, 15, 15),
    ],
    // L1: APERTURE/ARCH - gothic arch window frames
    [
        hi_meta(
            OP_APERTURE,
            REGION_WALLS,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            APERTURE_ARCH,
            0x181818,
            0x303028,
        ),
        lo(180, 128, 0, 0, 0, DIR_UP, 15, 15),
    ],
    // L2: CELL/BRICK - stone wall texture
    [
        hi_meta(
            OP_CELL,
            REGION_WALLS,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            CELL_BRICK,
            0x282828,
            0x1a1a18,
        ),
        lo(140, 96, 0, 0, 0, DIR_UP, 15, 15),
    ],
    // L3: TRACE/LEAD_LINES - stained glass leading (TANGENT_LOCAL)
    [
        hi_meta(
            OP_TRACE,
            REGION_WALLS,
            BLEND_ADD,
            DOMAIN_TANGENT_LOCAL,
            TRACE_LEAD_LINES,
            0x806040,
            0x000000,
        ),
        lo(160, 64, 0, 0, 0, DIR_UP, 15, 0),
    ],
    // L4: LOBE - divine golden light from above (sine pulse)
    [
        hi(OP_LOBE, REGION_ALL, BLEND_ADD, 0, 0xffd700, 0x000000),
        lo(120, 128, 0, 1, 0, DIR_SUN, 15, 0),
    ],
    // L5: SCATTER/DUST - golden dust motes in light beam
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
        lo(100, 80, 40, 0x20, 0, DIR_DOWN, 15, 0),
    ],
    // L6: ATMOSPHERE/MIE - incense haze
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
    // L7: NOP
    NOP_LAYER,
];

// -----------------------------------------------------------------------------
// Preset 6: "Ocean Depths" - Deep sea trench
// -----------------------------------------------------------------------------
// L0: RAMP          ALL   LERP  sky=#001030, floor=#203040, walls=(0x00,0x28,0x48), THRESH_INTERIOR
// L1: SECTOR/CAVE   ALL   LERP  #001828 / #002040 - cave enclosure
// L2: PLANE/WATER   FLOOR LERP  #004080 / #002848 - caustic floor
// L3: FLOW          FLOOR ADD   #00a0c0 / #000000 - animated caustics
// L4: SCATTER/BUBBLES ALL  ADD   #40a0a0 / #000000 - floating bubbles
// L5: VEIL/SHARDS   SKY   ADD   #80c0e0 / #000000 - light shafts, AXIS_CYL
// L6: ATMO/ABSORB   ALL   LERP  #000820 / #000000 - deep water fog
// L7: DECAL         WALLS ADD   #00ffaa / #000000 - bioluminescent glow
pub(super) const PRESET_OCEAN_DEPTHS: [[u64; 2]; 8] = [
    // L0: RAMP - dark blue sky, sandy floor, deep teal walls
    [
        hi(OP_RAMP, REGION_ALL, BLEND_LERP, 0, 0x001030, 0x203040),
        lo(230, 0x00, 0x28, 0x48, THRESH_INTERIOR, DIR_UP, 15, 15),
    ],
    // L1: SECTOR/CAVE - cave enclosure
    [
        hi_meta(
            OP_SECTOR,
            REGION_ALL,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            SECTOR_CAVE,
            0x001828,
            0x002040,
        ),
        lo(180, 128, 0, 0, 0, DIR_UP, 15, 15),
    ],
    // L2: PLANE/WATER - caustic floor
    [
        hi_meta(
            OP_PLANE,
            REGION_FLOOR,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            PLANE_WATER,
            0x004080,
            0x002848,
        ),
        lo(160, 128, 80, 0, 0, DIR_UP, 15, 15),
    ],
    // L3: FLOW - animated caustic light patterns
    [
        hi(OP_FLOW, REGION_FLOOR, BLEND_ADD, 0, 0x00a0c0, 0x000000),
        lo(100, 128, 0, 0, 100, DIR_UP, 15, 0),
    ],
    // L4: SCATTER/BUBBLES - floating bubbles
    [
        hi_meta(
            OP_SCATTER,
            REGION_ALL,
            BLEND_ADD,
            DOMAIN_DIRECT3D,
            SCATTER_BUBBLES,
            0x40a0a0,
            0x000000,
        ),
        lo(90, 100, 60, 0x20, 0, DIR_UP, 15, 0),
    ],
    // L5: VEIL/SHARDS - light shafts from surface (AXIS_CYL)
    [
        hi_meta(
            OP_VEIL,
            REGION_SKY,
            BLEND_ADD,
            DOMAIN_AXIS_CYL,
            VEIL_SHARDS,
            0x80c0e0,
            0x000000,
        ),
        lo(100, 128, 0, 0, 0, DIR_DOWN, 15, 0),
    ],
    // L6: ATMOSPHERE/ABSORPTION - deep water fog
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
        lo(120, 180, 0, 0, 0, DIR_UP, 15, 0),
    ],
    // L7: DECAL - bioluminescent glow spot
    [
        hi(OP_DECAL, REGION_WALLS, BLEND_ADD, 0, 0x00ffaa, 0x000000),
        lo(120, 64, 0, 0, 3, DIR_UP, 15, 0),
    ],
];

// -----------------------------------------------------------------------------
// Preset 7: "Void Station" - Derelict space station
// -----------------------------------------------------------------------------
// L0: RAMP          ALL   LERP  sky=#000008, floor=#101018, walls=(0x18,0x18,0x20), THRESH_INTERIOR
// L1: SECTOR/BOX    ALL   LERP  #101820 / #0a0a18 - box enclosure
// L2: APERTURE/RECT WALLS LERP  #0a0a14 / #181820 - rectangular viewport
// L3: GRID          WALLS ADD   #0044aa / #000000 - blue panel lines
// L4: CELL/GRID     FLOOR LERP  #080820 / #101018 - floor grating
// L5: SCATTER/STARS SKY   ADD   #ffffff / #000000 - stars through viewport
// L6: CELESTIAL/BIN SKY   ADD   #00aa88 / #4488aa - binary star, dir=SUN
// L7: DECAL         WALLS ADD   #00ff00 / #000000 - green status indicator
pub(super) const PRESET_VOID_STATION: [[u64; 2]; 8] = [
    // L0: RAMP - near-black sky, dark metal floor, gunmetal walls
    [
        hi(OP_RAMP, REGION_ALL, BLEND_LERP, 0, 0x000008, 0x101018),
        lo(180, 0x18, 0x18, 0x20, THRESH_INTERIOR, DIR_UP, 15, 15),
    ],
    // L1: SECTOR/BOX - box enclosure
    [
        hi_meta(
            OP_SECTOR,
            REGION_ALL,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            SECTOR_BOX,
            0x101820,
            0x0a0a18,
        ),
        lo(180, 128, 0, 0, 0, DIR_UP, 15, 15),
    ],
    // L2: APERTURE/RECT - rectangular viewport frame
    [
        hi_meta(
            OP_APERTURE,
            REGION_WALLS,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            APERTURE_RECT,
            0x0a0a14,
            0x181820,
        ),
        lo(180, 128, 0, 0, 0, DIR_UP, 15, 15),
    ],
    // L3: GRID - blue panel lines on walls
    [
        hi(OP_GRID, REGION_WALLS, BLEND_ADD, 0, 0x0044aa, 0x000000),
        lo(140, 48, 0, 2, 0, DIR_UP, 15, 0),
    ],
    // L4: CELL/GRID - floor grating pattern
    [
        hi_meta(
            OP_CELL,
            REGION_FLOOR,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            CELL_GRID,
            0x080820,
            0x101018,
        ),
        lo(150, 128, 0, 0, 0, DIR_UP, 15, 15),
    ],
    // L5: SCATTER/STARS - stars visible through viewport
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
        lo(150, 180, 0, 0x10, 0, DIR_UP, 15, 0),
    ],
    // L6: CELESTIAL/BINARY - binary star system outside
    [
        hi_meta(
            OP_CELESTIAL,
            REGION_SKY,
            BLEND_ADD,
            DOMAIN_DIRECT3D,
            CELESTIAL_BINARY,
            0x00aa88,
            0x4488aa,
        ),
        lo(140, 100, 0, 0, 0, DIR_SUN, 15, 15),
    ],
    // L7: DECAL - green status indicator
    [
        hi(OP_DECAL, REGION_WALLS, BLEND_ADD, 0, 0x00ff00, 0x000000),
        lo(120, 32, 0, 0, 4, DIR_UP, 15, 0),
    ],
];

// -----------------------------------------------------------------------------
// Preset 8: "Desert Mirage" - Vast dunes under blazing sun
// -----------------------------------------------------------------------------
// L0: RAMP          ALL   LERP  sky=#f0e8d0, floor=#d4b896, walls=(0xc8,0xa8,0x78), THRESH_VAST
// L1: SILH/DUNES    WALLS LERP  #b89860 / #d0b080 - sand dune silhouettes
// L2: PLANE/SAND    FLOOR LERP  #d8c090 / #c0a870 - textured sand floor
// L3: CELESTIAL/SUN SKY   ADD   #ffffd8 / #000000 - blazing sun, dir=SUN
// L4: FLOW          WALLS ADD   #f8f0e0 / #000000 - heat shimmer, low intensity
// L5: BAND          ALL   ADD   #ffe0a0 / #000000 - warm horizon glow, dir=SUNSET
// L6: ATMO/MIE      ALL   LERP  #e8d8c0 / #000000 - desert haze
// L7: SCATTER/DUST  FLOOR ADD   #c8b080 / #000000 - blowing sand
pub(super) const PRESET_DESERT_MIRAGE: [[u64; 2]; 8] = [
    // L0: RAMP - bleached sky, sand floor, tan walls
    [
        hi(OP_RAMP, REGION_ALL, BLEND_LERP, 0, 0xf0e8d0, 0xd4b896),
        lo(240, 0xc8, 0xa8, 0x78, THRESH_VAST, DIR_UP, 15, 15),
    ],
    // L1: SILHOUETTE/DUNES - sand dune silhouettes
    [
        hi_meta(
            OP_SILHOUETTE,
            REGION_WALLS,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            SILHOUETTE_DUNES,
            0xb89860,
            0xd0b080,
        ),
        lo(160, 128, 0, 0, 0, DIR_UP, 15, 15),
    ],
    // L2: PLANE/SAND - textured sand floor
    [
        hi_meta(
            OP_PLANE,
            REGION_FLOOR,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            PLANE_SAND,
            0xd8c090,
            0xc0a870,
        ),
        lo(150, 96, 0, 0, 0, DIR_UP, 15, 15),
    ],
    // L3: CELESTIAL/SUN - blazing desert sun
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
        lo(180, 128, 0, 0, 0, DIR_SUN, 15, 0),
    ],
    // L4: FLOW - heat shimmer effect (low intensity)
    [
        hi(OP_FLOW, REGION_WALLS, BLEND_ADD, 0, 0xf8f0e0, 0x000000),
        lo(60, 128, 0, 0, 40, DIR_UP, 8, 0),
    ],
    // L5: BAND - warm horizon glow
    [
        hi(OP_BAND, REGION_ALL, BLEND_ADD, 0, 0xffe0a0, 0x000000),
        lo(100, 128, 0, 0, 0, DIR_SUNSET, 15, 0),
    ],
    // L6: ATMOSPHERE/MIE - desert haze
    [
        hi_meta(
            OP_ATMOSPHERE,
            REGION_ALL,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            ATMO_MIE,
            0xe8d8c0,
            0x000000,
        ),
        lo(100, 128, 0, 0, 0, DIR_UP, 15, 0),
    ],
    // L7: SCATTER/DUST - blowing sand particles
    [
        hi_meta(
            OP_SCATTER,
            REGION_FLOOR,
            BLEND_ADD,
            DOMAIN_DIRECT3D,
            SCATTER_DUST,
            0xc8b080,
            0x000000,
        ),
        lo(120, 100, 60, 0x20, 0, DIR_DOWN, 12, 0),
    ],
];
