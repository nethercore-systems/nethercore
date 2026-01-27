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
        hi(OP_RAMP, REGION_ALL, BLEND_LERP, 0, 0x100800, 0x401000),
        lo(180, 0x20, 0x10, 0x08, THRESH_INTERIOR, DIR_UP, 15, 15),
    ],
    // L1: CELL/HEX - hexagonal basalt columns (bound)
    [
        hi_meta(
            OP_CELL,
            REGION_WALLS,
            BLEND_ADD,
            DOMAIN_DIRECT3D,
            CELL_HEX,
            0xff4000,
            0x301008,
        ),
        lo(200, 128, 220, 50, 0, DIR_UP, 15, 15),
    ],
    // L2: PATCHES/DEBRIS - volcanic rubble on floor (bound)
    [
        hi_meta(
            OP_PATCHES,
            REGION_FLOOR,
            BLEND_ADD,
            DOMAIN_DIRECT3D,
            PATCHES_DEBRIS,
            0x301800,
            0x200c00,
        ),
        lo(140, 128, 64, 0, 0, DIR_UP, 15, 15),
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
    // L4: TRACE/CRACKS - lava veins glowing through floor
    [
        hi_meta(
            OP_TRACE,
            REGION_WALLS,
            BLEND_ADD,
            DOMAIN_TANGENT_LOCAL,
            TRACE_CRACKS,
            0xff4000,
            0x000000,
        ),
        lo(150, 128, 64, 0, 0, DIR_UP, 15, 0),
    ],
    // L5: FLOW - churning lava (orange-red)
    [
        hi(OP_FLOW, REGION_FLOOR, BLEND_LERP, 0, 0xff2800, 0x000000),
        lo(80, 128, 0, 0x22, 100, DIR_UP, 15, 0),
    ],
    // L6: SCATTER/EMBERS - rising sparks (bright orange)
    [
        hi_meta(
            OP_SCATTER,
            REGION_ALL,
            BLEND_ADD,
            DOMAIN_DIRECT3D,
            SCATTER_EMBERS,
            0xff8000,
            0x000000,
        ),
        lo(60, 30, 25, 0x40, 0, DIR_UP, 15, 0),
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
// L0: RAMP (sky=#000000, floor=#001000, walls=#002000, THRESH_BALANCED)
// L1: SPLIT/CROSS (data grid structure, all, ADD, bound)
// L2: CELL/GRID (data block cells, walls, LERP, bound)
// L3: GRID (green wireframe, walls, ADD)
// L4: SCATTER/RAIN (falling code rain, all, SCREEN, AXIS_CYL, DOWN)
// L5: FLOW (code streaming, all, ADD, DOWN)
// L6: DECAL (data HUD element, walls, ADD)
// L7: PORTAL/RECT (data portal, walls, ADD, TANGENT_LOCAL)
pub(super) const PRESET_DIGITAL_MATRIX: [[u64; 2]; 8] = [
    // L0: RAMP - black sky, dark green floor, matrix green walls
    [
        hi(OP_RAMP, REGION_ALL, BLEND_LERP, 0, 0x000000, 0x001000),
        lo(220, 0x00, 0x08, 0x00, THRESH_BALANCED, DIR_UP, 15, 15),
    ],
    // L1: SPLIT/CROSS - data grid structure (bound)
    [
        hi_meta(
            OP_SPLIT,
            REGION_ALL,
            BLEND_ADD,
            DOMAIN_DIRECT3D,
            SPLIT_CROSS,
            0x003000,
            0x001800,
        ),
        lo(130, 40, 0, 0, 0, DIR_UP, 15, 15),
    ],
    // L2: CELL/GRID - data block cells (bound)
    [
        hi_meta(
            OP_CELL,
            REGION_WALLS,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            CELL_GRID,
            0x003000,
            0x001000,
        ),
        lo(140, 128, 0, 0, 0, DIR_UP, 15, 15),
    ],
    // L3: GRID - green wireframe overlay
    [
        hi(OP_GRID, REGION_WALLS, BLEND_ADD, 0, 0x00ff40, 0x000000),
        lo(255, 32, 0, 3, 0, DIR_UP, 15, 0),
    ],
    // L4: SCATTER/RAIN - falling code rain (green, cylindrical, downward)
    [
        hi_meta(
            OP_SCATTER,
            REGION_ALL,
            BLEND_SCREEN,
            DOMAIN_AXIS_CYL,
            SCATTER_RAIN,
            0x00ff00,
            0x000000,
        ),
        lo(180, 40, 200, 0, 0, DIR_DOWN, 15, 0),
    ],
    // L5: FLOW - code streaming effect (green, downward)
    [
        hi(OP_FLOW, REGION_ALL, BLEND_ADD, 0, 0x00dd00, 0x000000),
        lo(60, 128, 0, 0x31, 150, DIR_DOWN, 15, 0),
    ],
    // L6: DECAL - data HUD element (cyan)
    [
        hi(OP_DECAL, REGION_WALLS, BLEND_ADD, 0, 0x00ffff, 0x000000),
        lo(120, 8, 64, 0, 0, DIR_UP, 15, 0), // shape=DISK(0), soft=8, size=64
    ],
    // L7: PORTAL/RECT - data portal (cyan/green, TANGENT_LOCAL)
    [
        hi_meta(
            OP_PORTAL,
            REGION_WALLS,
            BLEND_ADD,
            DOMAIN_TANGENT_LOCAL,
            PORTAL_RECT,
            0x00ffff,
            0x004000,
        ),
        lo(120, 128, 64, 0, 0, DIR_UP, 15, 15),
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
    // L3: SPLIT/WEDGE - venetian blind shadows (iconic noir lighting)
    [
        hi_meta(
            OP_SPLIT,
            REGION_ALL,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            SPLIT_WEDGE,
            0x504030,
            0x080404,
        ),
        lo(240, 40, 0, 0, 0, DIR_SUN, 15, 15),
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
    // L7: FLOW - rain streaking on window (low intensity)
    [
        hi(OP_FLOW, REGION_WALLS, BLEND_ADD, 0, 0x404030, 0x000000),
        lo(180, 128, 0, 0x21, 60, DIR_DOWN, 15, 0),
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
    // L1: APERTURE/ROUNDED_RECT - porthole frames (bound)
    [
        hi_meta(
            OP_APERTURE,
            REGION_WALLS,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            APERTURE_ROUNDED_RECT,
            0x402010,
            0x604030,
        ),
        lo(200, 100, 0, 0, 0, DIR_UP, 15, 15),
    ],
    // L2: CELL/HEX - riveted hexagonal plate flooring (bound)
    [
        hi_meta(
            OP_CELL,
            REGION_FLOOR,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            CELL_HEX,
            0x503020,
            0x402010,
        ),
        lo(150, 128, 0, 0, 0, DIR_UP, 15, 15),
    ],
    // L3: GRID - brass framework and girders
    [
        hi(OP_GRID, REGION_WALLS, BLEND_ADD, 0, 0xc09040, 0x000000),
        lo(180, 48, 0, 0, 0, DIR_UP, 12, 0),
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
        lo(120, 128, 60, 0, 0, DIR_UP, 10, 0),
    ],
    // L6: SCATTER/DUST - floating steam particles
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
        lo(100, 30, 50, 0x20, 0, DIR_UP, 12, 0),
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
