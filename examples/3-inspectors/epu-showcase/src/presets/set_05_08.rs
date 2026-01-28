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
    // L0: RAMP - dark stone interior (keep contrast; avoid beige wash)
    [
        hi(OP_RAMP, REGION_ALL, BLEND_LERP, 0, 0x050510, 0x0c0c0c),
        lo(220, 0x18, 0x18, 0x18, THRESH_INTERIOR, DIR_UP, 15, 15),
    ],
    // L1: APERTURE/ARCH - big stained-glass window (place at FORWARD so it reads)
    [
        hi_meta(
            OP_APERTURE,
            REGION_WALLS,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            APERTURE_ARCH,
            0xff60c0,
            0x080808,
        ),
        // softness, half_w, half_h, frame_thickness, rise
        lo(200, 180, 240, 80, 200, DIR_FORWARD, 0, 0),
    ],
    // L2: SCATTER/DUST - dust motes in light beams (replaced brick - was too dominant)
    [
        hi_meta(
            OP_SCATTER,
            REGION_ALL,
            BLEND_ADD,
            DOMAIN_DIRECT3D,
            SCATTER_DUST,
            0xffd080,
            0x000000,
        ),
        lo(25, 16, 12, 0x20, 5, DIR_UP, 8, 0),
    ],
    // L3: TRACE/LEAD_LINES - stained-glass leading (tangent-local, aligned to window)
    [
        hi_meta(
            OP_TRACE,
            REGION_WALLS,
            BLEND_ADD,
            DOMAIN_TANGENT_LOCAL,
            TRACE_LEAD_LINES,
            0x080808,
            0xff60c0,
        ),
        // count, thickness, jitter, seed/vertex_count
        lo(140, 200, 60, 180, 0x38, DIR_FORWARD, 15, 8),
    ],
    // L4: LOBE - warm light spill from the window (focused)
    [
        hi(OP_LOBE, REGION_ALL, BLEND_ADD, 0, 0xffd080, 0x402010),
        lo(180, 220, 80, 1, 0, DIR_FORWARD, 12, 0),
    ],
    // L5: LOBE - stained glass glow (avoid flat decal panel)
    [
        hi(OP_LOBE, REGION_ALL, BLEND_ADD, 0, 0xff60c0, 0x40a0ff),
        lo(70, 200, 80, 1, 0, DIR_FORWARD, 10, 0),
    ],
    // L6: ATMOSPHERE/MIE - light haze around the window direction
    [
        hi_meta(
            OP_ATMOSPHERE,
            REGION_ALL,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            ATMO_MIE,
            0xffd080,
            0x000000,
        ),
        // intensity, falloff, horizon_y, mie_conc, mie_exp
        lo(60, 80, 128, 120, 220, DIR_FORWARD, 10, 0),
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
    // L2: PLANE/WATER - shimmering water surface above
    [
        hi_meta(
            OP_PLANE,
            REGION_SKY,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            PLANE_WATER,
            0x004080,
            0x002848,
        ),
        lo(160, 128, 80, 0, 0, DIR_UP, 15, 15),
    ],
    // L3: FLOW - animated caustic light from above (vivid, turbulent)
    [
        hi(OP_FLOW, REGION_SKY, BLEND_ADD, 0, 0x00a0c0, 0x000000),
        lo(220, 128, 50, 0x22, 100, DIR_UP, 15, 0),
    ],
    // L4: SCATTER/BUBBLES - floating bubbles (keep sparse; bubbles easily overpower)
    [
        hi_meta(
            OP_SCATTER,
            REGION_SKY | REGION_WALLS,
            BLEND_ADD,
            DOMAIN_DIRECT3D,
            SCATTER_BUBBLES,
            0x40a0a0,
            0x000000,
        ),
        lo(50, 20, 6, 0x10, 3, DIR_UP, 10, 0),
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
        lo(180, 128, 0, 0, 0, DIR_DOWN, 15, 0),
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
        lo(120, 90, 0, 0, 0, DIR_UP, 15, 0),
    ],
    // L7: DECAL - bioluminescent glow spot
    [
        hi(OP_DECAL, REGION_WALLS, BLEND_ADD, 0, 0x00ffaa, 0x000000),
        lo(120, 8, 64, 0, 3, DIR_UP, 15, 0), // shape=DISK(0), soft=8, size=64
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
        hi(OP_RAMP, REGION_ALL, BLEND_LERP, 0, 0x000820, 0x181828),
        lo(180, 0x28, 0x28, 0x38, THRESH_INTERIOR, DIR_UP, 15, 15),
    ],
    // L1: SECTOR/BOX - box enclosure
    [
        hi_meta(
            OP_SECTOR,
            REGION_ALL,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            SECTOR_BOX,
            0x182030,
            0x0c0c20,
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
            0x0c0c1c,
            0x202830,
        ),
        lo(180, 128, 0, 0, 0, DIR_UP, 15, 15),
    ],
    // L3: GRID - blue panel lines on walls
    [
        hi(OP_GRID, REGION_WALLS, BLEND_ADD, 0, 0x0066cc, 0x000000),
        lo(255, 48, 30, 0x10, 0, DIR_UP, 15, 0),
    ],
    // L4: PLANE/GRATING - floor grating texture (replaced visible grid cells)
    [
        hi_meta(
            OP_PLANE,
            REGION_FLOOR,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            PLANE_GRATING,
            0x181830,
            0x101020,
        ),
        lo(120, 80, 60, 40, 0, DIR_UP, 12, 10),
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
        lo(255, 180, 30, 0x10, 0, DIR_UP, 15, 0),
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
        lo(255, 200, 0, 0, 0, DIR_SUN, 15, 15),
    ],
    // L7: DECAL - green status indicator
    [
        hi(OP_DECAL, REGION_WALLS, BLEND_ADD, 0, 0x00ff00, 0x000000),
        lo(200, 0x20, 32, 0, 4, DIR_UP, 15, 0), // shape=RECT(2), soft=0, size=32
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
    // L0: RAMP - bright desert, but preserve contrast/detail
    [
        hi(OP_RAMP, REGION_ALL, BLEND_LERP, 0, 0xe0d0b8, 0xb09060),
        lo(220, 0x80, 0x60, 0x30, THRESH_VAST, DIR_UP, 15, 15),
    ],
    // L1: SILHOUETTE/DUNES - sand dune silhouettes
    [
        hi_meta(
            OP_SILHOUETTE,
            REGION_WALLS,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            SILHOUETTE_DUNES,
            0x806030,
            0xb09060,
        ),
        // softness, height_bias, roughness, octaves
        lo(50, 140, 220, 0x20, 0, DIR_UP, 15, 0),
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
    // L3: CELESTIAL/SUN - bright, but not nuclear
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
        // intensity, angular_size, limb_exp, phase, corona_extent
        lo(130, 140, 200, 0, 180, DIR_SUN, 15, 10),
    ],
    // L4: FLOW - heat shimmer (very subtle, tangent-local to avoid barrel lines)
    [
        hi_meta(
            OP_FLOW,
            REGION_WALLS,
            BLEND_ADD,
            DOMAIN_TANGENT_LOCAL,
            0,
            0xf8f0e0,
            0x000000,
        ),
        lo(20, 96, 80, 0x10, 40, DIR_UP, 6, 0),
    ],
    // L5: BAND - warm horizon glow (thin band around up-axis)
    [
        hi(OP_BAND, REGION_SKY, BLEND_ADD, 0, 0xffd080, 0x000000),
        lo(80, 40, 128, 200, 0, DIR_UP, 10, 0),
    ],
    // L6: ATMOSPHERE/MIE - desert haze around the sun direction
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
        lo(40, 80, 128, 140, 180, DIR_SUN, 10, 0),
    ],
    // L7: FLOW - wind-blown sand shimmer near the ground (tangent-local to avoid barrel lines)
    [
        hi_meta(
            OP_FLOW,
            REGION_FLOOR,
            BLEND_ADD,
            DOMAIN_TANGENT_LOCAL,
            0,
            0xffe8d0,
            0xb09060,
        ),
        lo(25, 140, 60, 0x11, 0, DIR_RIGHT, 8, 0),
    ],
];
