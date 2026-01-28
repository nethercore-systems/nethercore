//! Preset set 17-20

#[allow(unused_imports)]
use crate::constants::*;

// -----------------------------------------------------------------------------
// Preset 17: "Volcanic Core" — Inside active volcano
// -----------------------------------------------------------------------------
// L0: RAMP (sky=#100800, floor=#401000, walls=#201008, THRESH_INTERIOR)
// L1: CELL/HEX (basalt columns, walls, LERP)
// L2: PATCHES/DEBRIS (volcanic rubble, floor, ADD)
// L3: PLANE/STONE (rocky volcanic floor, LERP)
// L4: TRACE/CRACKS (lava veins, floor, ADD, TANGENT_LOCAL)
// L5: FLOW (churning lava, floor, SCREEN)
// L6: SCATTER/EMBERS (rising sparks, all, ADD)
// L7: ATMOSPHERE/ABSORPTION (volcanic gas, all, LERP)
pub(super) const PRESET_VOLCANIC_CORE: [[u64; 2]; 8] = [
    // L0: RAMP - black sky, magma floor, obsidian walls
    [
        hi(OP_RAMP, REGION_ALL, BLEND_LERP, 0, 0x080400, 0x200800),
        lo(180, 0x10, 0x08, 0x04, THRESH_INTERIOR, DIR_UP, 15, 15),
    ],
    // L1: CELL/HEX - hexagonal basalt columns (very dark, near-black stone)
    [
        hi_meta(
            OP_CELL,
            REGION_WALLS,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            CELL_HEX,
            0x0c0400,
            0x060200,
        ),
        lo(200, 128, 220, 50, 0, DIR_UP, 4, 4),
    ],
    // L2: PATCHES/DEBRIS - volcanic rubble on floor (near-black)
    [
        hi_meta(
            OP_PATCHES,
            REGION_FLOOR,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            PATCHES_DEBRIS,
            0x0c0600,
            0x080400,
        ),
        lo(140, 128, 64, 0, 0, DIR_UP, 6, 6),
    ],
    // L3: PLANE/STONE - rocky volcanic floor
    [
        hi_meta(
            OP_PLANE,
            REGION_FLOOR,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            PLANE_STONE,
            0x181008,
            0x100800,
        ),
        lo(160, 128, 0, 0, 0, DIR_UP, 15, 15),
    ],
    // L4: TRACE/CRACKS - lava veins glowing through walls and floor (bright orange)
    [
        hi_meta(
            OP_TRACE,
            REGION_ALL,
            BLEND_ADD,
            DOMAIN_TANGENT_LOCAL,
            TRACE_CRACKS,
            0xff6000,
            0x000000,
        ),
        lo(220, 128, 64, 0, 0, DIR_UP, 15, 0),
    ],
    // L5: FLOW - churning lava on floor (orange-red, vivid)
    [
        hi(OP_FLOW, REGION_FLOOR, BLEND_ADD, 0, 0xff4000, 0x000000),
        lo(150, 128, 0, 0x22, 100, DIR_UP, 15, 0),
    ],
    // L6: SCATTER/EMBERS - rising sparks (keep readable; don't fill screen)
    [
        hi_meta(
            OP_SCATTER,
            REGION_SKY | REGION_WALLS,
            BLEND_ADD,
            DOMAIN_DIRECT3D,
            SCATTER_EMBERS,
            0xff8000,
            0x000000,
        ),
        lo(45, 12, 18, 0x20, 9, DIR_UP, 10, 0),
    ],
    // L7: ATMOSPHERE/ABSORPTION - volcanic gases
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
        lo(150, 80, 0, 0, 0, DIR_UP, 15, 0),
    ],
];

// -----------------------------------------------------------------------------
// Preset 18: "Digital Matrix" — Cyber virtual reality
// -----------------------------------------------------------------------------
// Design: Total black void. Dense vertical green code rain dominates.
// No geometric grids or rings — everything is streaming downward like
// the iconic Matrix "digital rain" effect. A faint green tint in the
// void and occasional bright code streaks breaking through.
//
// L0: RAMP       ALL   LERP  sky=#000000, floor=#000800 - pure black void
// L1: VEIL       ALL   ADD   #00ff40 - vertical code rain curtains
// L2: SCATTER    ALL   SCREEN #00ff00 - falling code rain drops (CYL, DOWN)
// L3: FLOW       ALL   ADD   #00aa00 - streaming code effect (DOWN)
// L4: TRACE      WALLS ADD   #00ff60 - bright code veins on walls
// L5: LOBE       ALL   ADD   #003000 - faint green ambient glow from below
// L6: DECAL      WALLS ADD   #00ffff - single data HUD element
// L7: ATMOSPHERE ALL   ADD   #000800 - faint green digital fog
pub(super) const PRESET_DIGITAL_MATRIX: [[u64; 2]; 8] = [
    // L0: RAMP - pure black void with faint green floor
    [
        hi(OP_RAMP, REGION_ALL, BLEND_LERP, 0, 0x000000, 0x000800),
        lo(255, 0x00, 0x04, 0x00, THRESH_BALANCED, DIR_UP, 15, 15),
    ],
    // L1: VEIL/RAIN_WALL - dense code rain columns (many thin bars)
    [
        hi_meta(
            OP_VEIL,
            REGION_ALL,
            BLEND_ADD,
            DOMAIN_AXIS_CYL,
            VEIL_RAIN_WALL,
            0x00ff40,
            0x004010,
        ),
        lo(160, 220, 30, 0, 0, DIR_UP, 10, 6),
    ],
    // L2: SCATTER/WINDOWS - glyph-like rectangles (looks like "characters")
    [
        hi_meta(
            OP_SCATTER,
            REGION_ALL,
            BLEND_ADD,
            DOMAIN_AXIS_CYL,
            SCATTER_WINDOWS,
            0x00ff60,
            0x00aa20,
        ),
        lo(100, 180, 20, 0x00, 42, DIR_UP, 10, 0),
    ],
    // L3: FLOW/STREAKS - extra motion streaks (reinforces "rain")
    [
        hi(OP_FLOW, REGION_ALL, BLEND_ADD, 0, 0x00ff80, 0x008000),
        lo(90, 200, 40, 0x21, 0, DIR_DOWN, 12, 0),
    ],
    // L4: TRACE/CRACKS - bright code veins on walls (Matrix circuit patterns)
    [
        hi_meta(
            OP_TRACE,
            REGION_WALLS,
            BLEND_ADD,
            DOMAIN_TANGENT_LOCAL,
            TRACE_CRACKS,
            0x00ff60,
            0x000000,
        ),
        lo(60, 128, 64, 0, 0, DIR_DOWN, 15, 0),
    ],
    // L5: LOBE - faint green ambient glow from below
    [
        hi(OP_LOBE, REGION_ALL, BLEND_ADD, 0, 0x003000, 0x000000),
        lo(20, 128, 0, 0, 0, DIR_UP, 10, 0),
    ],
    // L6: DECAL - data HUD element (rect)
    [
        hi(OP_DECAL, REGION_WALLS, BLEND_ADD, 0, 0x00ffff, 0x000000),
        // shape=RECT(2), soft=8, size=64
        lo(80, 0x28, 64, 80, 0, DIR_FORWARD, 12, 8),
    ],
    // L7: ATMOSPHERE/ALIEN - faint green digital fog
    [
        hi_meta(
            OP_ATMOSPHERE,
            REGION_ALL,
            BLEND_ADD,
            DOMAIN_DIRECT3D,
            ATMO_ALIEN,
            0x000800,
            0x000000,
        ),
        lo(10, 80, 0, 0, 0, DIR_UP, 8, 0),
    ],
];

// -----------------------------------------------------------------------------
// Preset 19: "Noir Detective" — 1940s private eye office
// -----------------------------------------------------------------------------
// L0: RAMP (sky=#101008, floor=#302820, walls=#383428, THRESH_INTERIOR)
// L1: SECTOR/BOX (office box enclosure, all, LERP, bound)
// L2: APERTURE/RECT (window frame, walls, LERP, bound)
// L3: SPLIT/WEDGE (venetian blind shadows, walls, LERP, dir=SUN)
// L4: LOBE (desk lamp glow, floor, ADD, dir=DOWN)
// L5: SCATTER/DUST (cigarette smoke, all, ADD)
// L6: ATMOSPHERE/MIE (smoky haze, all, LERP)
// L7: FLOW (rain on window, walls, ADD, low intensity)
pub(super) const PRESET_NOIR_DETECTIVE: [[u64; 2]; 8] = [
    // L0: RAMP - dark ceiling, worn wood floor, olive walls
    [
        hi(OP_RAMP, REGION_ALL, BLEND_LERP, 0, 0x101008, 0x302820),
        lo(180, 0x38, 0x34, 0x28, THRESH_INTERIOR, DIR_UP, 15, 15),
    ],
    // L1: SECTOR/BOX - office box enclosure (bound)
    [
        hi_meta(
            OP_SECTOR,
            REGION_ALL,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            SECTOR_BOX,
            0x282418,
            0x1a1810,
        ),
        lo(180, 128, 0, 0, 0, DIR_UP, 15, 15),
    ],
    // L2: APERTURE/RECT - window frame (bound)
    [
        hi_meta(
            OP_APERTURE,
            REGION_WALLS,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            APERTURE_RECT,
            0x040404,
            0x383830,
        ),
        lo(200, 128, 0, 0, 0, DIR_UP, 15, 15),
    ],
    // L3: SPLIT/BANDS - venetian blind shadow stripes (keep subtle)
    [
        hi_meta(
            OP_SPLIT,
            REGION_WALLS | REGION_FLOOR,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            SPLIT_BANDS,
            0x302820,
            0x101008,
        ),
        // blend_width, band_count, band_offset
        lo(0, 20, 0, 200, 100, DIR_SUN, 15, 15),
    ],
    // L4: LOBE - desk lamp cone of warm light (sine flicker)
    [
        hi(OP_LOBE, REGION_FLOOR, BLEND_ADD, 0, 0xffe0a0, 0x000000),
        lo(255, 128, 0, 1, 1, DIR_DOWN, 15, 0),
    ],
    // L5: SCATTER/DUST - cigarette smoke particles
    [
        hi_meta(
            OP_SCATTER,
            REGION_ALL,
            BLEND_ADD,
            DOMAIN_DIRECT3D,
            SCATTER_DUST,
            0x504838,
            0x000000,
        ),
        lo(30, 20, 20, 0x20, 0, DIR_UP, 10, 0),
    ],
    // L6: ATMOSPHERE/MIE - smoky haze filling the room
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
        lo(60, 70, 0, 0, 0, DIR_UP, 15, 0),
    ],
    // L7: FLOW/STREAKS - rain streaking on the window
    [
        hi(OP_FLOW, REGION_WALLS, BLEND_SCREEN, 0, 0x8090a0, 0x000000),
        lo(80, 180, 40, 0x21, 0, DIR_DOWN, 10, 0),
    ],
];

// -----------------------------------------------------------------------------
// Preset 20: "Steampunk Airship" — Victorian observation deck
// -----------------------------------------------------------------------------
// L0: RAMP (sky=#ffa040, floor=#604020, walls=#805030, THRESH_SEMI)
// L1: APERTURE/ROUNDED_RECT (porthole frames, walls, LERP, bound)
// L2: CELL/HEX (riveted hex plates, floor, LERP, bound)
// L3: GRID (brass girders, walls, ADD)
// L4: CELESTIAL/SUN (setting sun, sky, ADD, dir=SUNSET)
// L5: VEIL/PILLARS (steam columns, walls, ADD, AXIS_CYL)
// L6: SCATTER/DUST (steam particles, all, ADD)
// L7: ATMOSPHERE/MIE (warm amber haze, all, LERP)
pub(super) const PRESET_STEAMPUNK_AIRSHIP: [[u64; 2]; 8] = [
    // L0: RAMP - amber sunset sky, burnished brass floor, copper walls
    [
        hi(OP_RAMP, REGION_ALL, BLEND_LERP, 0, 0xffa040, 0x604020),
        lo(190, 0x80, 0x50, 0x30, THRESH_SEMI, DIR_UP, 15, 15),
    ],
    // L1: APERTURE/MULTI - grid of porthole windows (bound)
    [
        hi_meta(
            OP_APERTURE,
            REGION_WALLS,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            APERTURE_MULTI,
            0x305070,
            0x9a6a2a,
        ),
        // softness, half_w, half_h, frame_thickness, cell_count
        lo(80, 100, 50, 200, 4, DIR_BACK, 0, 0),
    ],
    // L2: PLANE/GRATING - brass deck plating (floor only)
    [
        hi_meta(
            OP_PLANE,
            REGION_FLOOR,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            PLANE_GRATING,
            0x6a4a24,
            0x2a1808,
        ),
        lo(140, 90, 80, 60, 0, DIR_UP, 15, 0),
    ],
    // L3: GRID - brass framework and girders
    [
        hi(OP_GRID, REGION_WALLS, BLEND_ADD, 0, 0xc09040, 0x000000),
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
        lo(160, 220, 0, 0, 0, DIR_SUNSET, 15, 0),
    ],
    // L5: VEIL/PILLARS - steam columns rising from vents
    [
        hi_meta(
            OP_VEIL,
            REGION_WALLS,
            BLEND_ADD,
            DOMAIN_AXIS_CYL,
            VEIL_PILLARS,
            0xfff0d0,
            0x000000,
        ),
        lo(60, 60, 60, 0, 0, DIR_UP, 8, 0),
    ],
    // L6: SCATTER/DUST - floating steam particles (very sparse)
    [
        hi_meta(
            OP_SCATTER,
            REGION_ALL,
            BLEND_ADD,
            DOMAIN_DIRECT3D,
            SCATTER_DUST,
            0xffe8c0,
            0x000000,
        ),
        lo(20, 16, 12, 0x10, 9, DIR_UP, 6, 0),
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
        lo(70, 120, 0, 0, 0, DIR_UP, 15, 0),
    ],
];
