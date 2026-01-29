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
    // L0: RAMP - cold nave shadows + warm floor bounce
    [
        hi(OP_RAMP, REGION_ALL, BLEND_LERP, 0, 0x03030c, 0x0c0705),
        lo(210, 0x24, 0x1d, 0x18, THRESH_INTERIOR, DIR_UP, 15, 15),
    ],
    // L1: SECTOR/TUNNEL - long hall read (keeps it from looking like an outdoor skybox)
    [
        hi_meta(
            OP_SECTOR,
            REGION_ALL,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            SECTOR_TUNNEL,
            0x060612,
            0x16110f,
        ),
        lo(200, 120, 82, 0, 0, DIR_UP, 15, 15),
    ],
    // L2: APERTURE/ARCH - stained-glass window (opening is vivid; frame stays stone-dark)
    [
        hi_meta(
            OP_APERTURE,
            REGION_ALL,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            APERTURE_ARCH,
            0x2b63ff,
            0x09080b,
        ),
        // half_w, half_h, frame_thickness, rise
        lo(185, 175, 235, 55, 245, DIR_FORWARD, 0, 0),
    ],
    // L3: CELL/BRICK - stone block breakup (low contrast, large scale; avoid full-screen confetti)
    [
        hi_meta(
            OP_CELL,
            REGION_WALLS,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            CELL_BRICK,
            0x2c2a2a,
            0x141212,
        ),
        lo(70, 90, 210, 70, 9, DIR_UP, 12, 10),
    ],
    // L4: TRACE/LEAD_LINES - dark leading inside the window (multiplies only where the lines are)
    [
        hi_meta(
            OP_TRACE,
            REGION_SKY,
            BLEND_MULTIPLY,
            DOMAIN_TANGENT_LOCAL,
            TRACE_LEAD_LINES,
            0x0b0b10,
            0x202028,
        ),
        lo(120, 115, 55, 150, 0x82, DIR_FORWARD, 12, 0),
    ],
    // L5: FLOW - subtle animated shaft shimmer (keeps motion physical; avoid VEIL seam risk)
    [
        hi_meta(
            OP_FLOW,
            REGION_SKY | REGION_WALLS,
            BLEND_SCREEN,
            DOMAIN_DIRECT3D,
            0,
            0xfff1c8,
            0x110c0c,
        ),
        lo(95, 145, 24, 0x18, 0, DIR_FORWARD, 12, 0),
    ],
    // L6: SCATTER/DUST - incense motes catching the shafts
    [
        hi_meta(
            OP_SCATTER,
            REGION_ALL,
            BLEND_ADD,
            DOMAIN_DIRECT3D,
            SCATTER_DUST,
            0xffe6b8,
            0x806040,
        ),
        lo(22, 24, 52, 0x18, 33, 0, 8, 0),
    ],
    // L7: ATMOSPHERE/MIE - warm incense haze (LERP so it doesn't bleach the whole scene)
    [
        hi_meta(
            OP_ATMOSPHERE,
            REGION_ALL,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            ATMO_MIE,
            0x241a14,
            0x040306,
        ),
        lo(55, 105, 128, 110, 200, DIR_FORWARD, 10, 0),
    ],
];

// -----------------------------------------------------------------------------
// Preset 6: "Ocean Depths" - Deep sea trench
// -----------------------------------------------------------------------------
// L0: RAMP               ALL   LERP  water column gradient, THRESH_SEMI
// L1: SILHOUETTE/SPIRES   WALLS LERP  jagged trench rim silhouettes
// L2: PLANE/STONE         FLOOR LERP  dark rock seabed texture
// L3: FLOW                WALLS|FLOOR ADD  animated caustics (TANGENT_LOCAL to avoid seams)
// L4: VEIL/SHARDS         SKY|WALLS  SCREEN soft god-rays from the surface (AXIS_CYL)
// L5: SCATTER/DUST        ALL   ADD  "marine snow" particulates (static seed)
// L6: ATMO/ABSORPTION     ALL   LERP  deep-water absorption haze
// L7: PORTAL/RIFT         FLOOR|WALLS SCREEN faint bioluminescent trench mouth
pub(super) const PRESET_OCEAN_DEPTHS: [[u64; 2]; 8] = [
    // L0: RAMP - surface cyan spill -> deep abyss
    [
        hi(OP_RAMP, REGION_ALL, BLEND_LERP, 0, 0x14aeb4, 0x000006),
        lo(235, 0x04, 0x1a, 0x24, THRESH_SEMI, DIR_UP, 15, 15),
    ],
    // L1: SECTOR/CAVE - trench bite-out (big shapes for believable reflections)
    [
        hi_meta(
            OP_SECTOR,
            REGION_ALL,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            SECTOR_CAVE,
            0x033646,
            0x000a10,
        ),
        lo(205, 128, 110, 0, 0, DIR_UP, 15, 15),
    ],
    // L2: PLANE/STONE - dark basalt seabed
    [
        hi_meta(
            OP_PLANE,
            REGION_FLOOR,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            PLANE_STONE,
            0x06131b,
            0x020608,
        ),
        lo(165, 140, 90, 130, 20, DIR_UP, 15, 12),
    ],
    // L3: FLOW - caustics (seam-free DIRECT3D; animated)
    [
        hi_meta(
            OP_FLOW,
            REGION_FLOOR | REGION_WALLS,
            BLEND_SCREEN,
            DOMAIN_DIRECT3D,
            0,
            0x9ffcff,
            0x001018,
        ),
        lo(95, 165, 55, 0x22, 0, DIR_DOWN, 11, 0),
    ],
    // L4: VEIL/PILLARS - soft god-rays from the surface (DIRECT3D to avoid axial seams)
    [
        hi_meta(
            OP_VEIL,
            REGION_SKY | REGION_WALLS,
            BLEND_SCREEN,
            DOMAIN_DIRECT3D,
            VEIL_PILLARS,
            0xcafcff,
            0x001018,
        ),
        lo(95, 85, 70, 120, 0, DIR_UP, 10, 6),
    ],
    // L5: FLOW - slow "marine snow" drift (animated, but continuous - avoids SCATTER shimmer)
    [
        hi_meta(
            OP_FLOW,
            REGION_ALL,
            BLEND_SCREEN,
            DOMAIN_DIRECT3D,
            0,
            0xd8ffff,
            0x001018,
        ),
        lo(35, 210, 12, 0x21, 0, DIR_DOWN, 8, 0),
    ],
    // L6: ATMOSPHERE/ABSORPTION - deep-water light falloff (multiply reads physical)
    [
        hi_meta(
            OP_ATMOSPHERE,
            REGION_ALL,
            BLEND_MULTIPLY,
            DOMAIN_DIRECT3D,
            ATMO_ABSORPTION,
            0x002030,
            0x000000,
        ),
        lo(195, 155, 110, 0, 0, DIR_UP, 12, 0),
    ],
    // L7: PORTAL/RIFT - faint biolum trench mouth (animated pulse)
    [
        hi_meta(
            OP_PORTAL,
            REGION_WALLS | REGION_FLOOR,
            BLEND_ADD,
            DOMAIN_TANGENT_LOCAL,
            PORTAL_RIFT,
            0x001018,
            0x00ffd0,
        ),
        lo(145, 115, 70, 170, 0, DIR_FORWARD, 14, 10),
    ],
];

// -----------------------------------------------------------------------------
// Preset 7: "Void Station" - Derelict space station
// -----------------------------------------------------------------------------
// Goal: clear interior enclosure with a single viewport to deep space.
// Big shapes first (box + ramp), then sparse detail (panels + one blinking LED).
//
// L0: RAMP               ALL        LERP  cold interior gradient, THRESH_INTERIOR
// L1: SECTOR/BOX         ALL        LERP  hard room enclosure (gives reflections a "room" read)
// L2: CELL/HEX           WALLS|SKY  LERP  subtle metal ceiling/wall panels (low contrast)
// L3: PLANE/GRATING      FLOOR      LERP  dark deck grating (grounding cue)
// L4: APERTURE/RND_RECT  WALLS      LERP  viewport frame (DIR_FORWARD)
// L5: SCATTER/STARS      WALLS      ADD   tiny distant stars localized to the viewport (TANGENT_LOCAL)
// L6: LOBE               ALL        ADD   cool window spill light (DIR_FORWARD)
// L7: DECAL              WALLS      ADD   small blinking status LED (DIR_RIGHT)
pub(super) const PRESET_VOID_STATION: [[u64; 2]; 8] = [
    // L0: RAMP - cold station interior + near-black space
    [
        hi(OP_RAMP, REGION_ALL, BLEND_LERP, 0, 0x000006, 0x0f141d),
        lo(225, 0x14, 0x18, 0x22, THRESH_INTERIOR, DIR_UP, 15, 15),
    ],
    // L1: SECTOR/BOX - hard enclosure read (big shapes for reflections)
    [
        hi_meta(
            OP_SECTOR,
            REGION_ALL,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            SECTOR_BOX,
            0x131c26,
            0x04050a,
        ),
        lo(230, 145, 0, 0, 0, DIR_UP, 15, 15),
    ],
    // L2: PLANE/GRATING - deck plating
    [
        hi_meta(
            OP_PLANE,
            REGION_FLOOR,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            PLANE_GRATING,
            0x111821,
            0x070a10,
        ),
        lo(170, 100, 85, 55, 20, DIR_UP, 15, 12),
    ],
    // L3: CELL/HEX - subtle panel plates (softer than GRID; avoids obvious line lattice)
    [
        hi_meta(
            OP_CELL,
            REGION_WALLS | REGION_SKY,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            CELL_HEX,
            0x141c26,
            0x080b12,
        ),
        lo(80, 85, 165, 55, 9, DIR_UP, 12, 10),
    ],
    // L4: APERTURE/ROUNDED_RECT - single viewport on one wall (local chart, no wrap)
    [
        hi_meta(
            OP_APERTURE,
            REGION_WALLS,
            BLEND_LERP,
            DOMAIN_TANGENT_LOCAL,
            APERTURE_ROUNDED_RECT,
            0x000004,
            0x2a3a4c,
        ),
        // intensity, softness, half_w, half_h, frame_thickness
        lo(220, 55, 120, 185, 18, DIR_FORWARD, 15, 15),
    ],
    // L5: SCATTER/STARS - localized to the viewport wall patch (no full-ceiling glitter)
    [
        hi_meta(
            OP_SCATTER,
            REGION_WALLS,
            BLEND_ADD,
            DOMAIN_TANGENT_LOCAL,
            SCATTER_STARS,
            0xf8fbff,
            0x6aa6ff,
        ),
        lo(120, 26, 18, 0x60, 19, DIR_FORWARD, 12, 0),
    ],
    // L6: LOBE - cold spill from the viewport onto the room
    [
        hi(OP_LOBE, REGION_ALL, BLEND_ADD, 0, 0x86c0ff, 0x0a1220),
        lo(80, 215, 85, 1, 0, DIR_FORWARD, 12, 0),
    ],
    // L7: DECAL - tiny status LED (blink via phase)
    [
        hi(OP_DECAL, REGION_WALLS, BLEND_ADD, 0, 0xff6a3a, 0x2a0c08),
        // shape=RECT(2), soft=4, size=18, glow_soft=90
        lo(190, 0x24, 18, 90, 0, DIR_RIGHT, 15, 10),
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
    // L0: RAMP - hotter sky, deeper sand (fixes the washed-out, monotone read)
    [
        hi(OP_RAMP, REGION_ALL, BLEND_LERP, 0, 0xf2ead6, 0x9a6f3a),
        lo(215, 0xc8, 0xa4, 0x6c, THRESH_VAST, DIR_UP, 15, 15),
    ],
    // L1: SILHOUETTE/DUNES - higher contrast dune line so the scene reads instantly
    [
        hi_meta(
            OP_SILHOUETTE,
            REGION_SKY,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            SILHOUETTE_DUNES,
            0x7a4f24,
            0xfff6e4,
        ),
        lo(70, 140, 210, 0x40, 0, DIR_UP, 15, 0),
    ],
    // L2: PLANE/SAND - grain + ripples (push contrast a touch)
    [
        hi_meta(
            OP_PLANE,
            REGION_FLOOR,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            PLANE_SAND,
            0xd6ba86,
            0x9a6f3a,
        ),
        lo(190, 130, 15, 160, 30, DIR_UP, 15, 15),
    ],
    // L3: CELESTIAL/SUN - ensure it's actually visible in the default camera (DIR_UP)
    [
        hi_meta(
            OP_CELESTIAL,
            REGION_SKY,
            BLEND_ADD,
            DOMAIN_DIRECT3D,
            CELESTIAL_SUN,
            0xfff3c8,
            0xffd28a,
        ),
        // intensity, angular_size, limb_exp, phase, corona_extent
        lo(255, 95, 220, 0, 235, DIR_SUN, 15, 12),
    ],
    // L4: FLOW - heat shimmer (noise; subtle; seam-free)
    [
        hi_meta(
            OP_FLOW,
            REGION_ALL,
            BLEND_SCREEN,
            DOMAIN_DIRECT3D,
            0,
            0xfff2d8,
            0x9a6f3a,
        ),
        lo(35, 135, 95, 0x20, 60, DIR_RIGHT, 6, 0),
    ],
    // L5: BAND - warm horizon glow
    [
        hi(OP_BAND, REGION_SKY, BLEND_ADD, 0, 0xffcf82, 0x000000),
        lo(110, 60, 128, 220, 0, DIR_UP, 10, 0),
    ],
    // L6: ATMOSPHERE/MIE - bright haze around the sun direction
    [
        hi_meta(
            OP_ATMOSPHERE,
            REGION_ALL,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            ATMO_MIE,
            0xf6ddba,
            0x9a6f3a,
        ),
        lo(55, 120, 128, 150, 220, DIR_SUN, 10, 0),
    ],
    // L7: SCATTER/DUST - blowing sand near the ground
    [
        hi_meta(
            OP_SCATTER,
            REGION_FLOOR,
            BLEND_ADD,
            DOMAIN_DIRECT3D,
            SCATTER_DUST,
            0xffe0b8,
            0xcaa46c,
        ),
        lo(55, 34, 110, 0x18, 27, DIR_RIGHT, 8, 0),
    ],
];
