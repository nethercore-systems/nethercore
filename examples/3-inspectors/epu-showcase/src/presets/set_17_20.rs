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
    // L1: SECTOR/CAVE - volcanic cave enclosure (replaced hex - was too visible)
    [
        hi_meta(
            OP_SECTOR,
            REGION_ALL,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            SECTOR_CAVE,
            0x100800,
            0x080400,
        ),
        lo(200, 128, 0, 0, 0, DIR_UP, 15, 15),
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
// the iconic Matrix "digital rain" effect. Uses TANGENT_LOCAL for straight
// vertical rain without barrel distortion.
//
// L0: RAMP       ALL   LERP  sky=#000000, floor=#000400 - pure black void
// L1: VEIL       ALL   ADD   #00ff40 - vertical code rain (TANGENT_LOCAL)
// L2: FLOW       ALL   ADD   #00ff80 - streaming code motion
// L3: FLOW       SKY   ADD   #00aa40 - secondary rain layer
// L4: TRACE      ALL   ADD   #00ff60 - code circuit patterns
// L5: LOBE       FLOOR ADD   #002000 - faint floor glow
// L6: SCATTER    SKY   ADD   #00ff00 - sparse falling glyphs
// L7: ATMOSPHERE ALL   ADD   #000400 - minimal green fog
pub(super) const PRESET_DIGITAL_MATRIX: [[u64; 2]; 8] = [
    // L0: RAMP - pure black void (VAST threshold for minimal structure)
    [
        hi(OP_RAMP, REGION_ALL, BLEND_LERP, 0, 0x000000, 0x000400),
        lo(255, 0x00, 0x02, 0x00, THRESH_VAST, DIR_UP, 15, 15),
    ],
    // L1: VEIL/RAIN_WALL - vertical code rain (TANGENT_LOCAL for straight lines)
    [
        hi_meta(
            OP_VEIL,
            REGION_ALL,
            BLEND_ADD,
            DOMAIN_TANGENT_LOCAL,
            VEIL_RAIN_WALL,
            0x00ff40,
            0x004010,
        ),
        lo(200, 255, 20, 180, 60, DIR_DOWN, 12, 4),
    ],
    // L2: FLOW - primary streaming code motion
    [
        hi(OP_FLOW, REGION_ALL, BLEND_ADD, 0, 0x00ff80, 0x006020),
        lo(120, 220, 30, 0x21, 0, DIR_DOWN, 12, 0),
    ],
    // L3: FLOW - secondary rain layer (sky emphasis)
    [
        hi(OP_FLOW, REGION_SKY | REGION_WALLS, BLEND_ADD, 0, 0x00aa40, 0x004010),
        lo(80, 180, 50, 0x21, 0, DIR_DOWN, 10, 0),
    ],
    // L4: TRACE/CRACKS - code circuit patterns (vertical emphasis)
    [
        hi_meta(
            OP_TRACE,
            REGION_ALL,
            BLEND_ADD,
            DOMAIN_TANGENT_LOCAL,
            TRACE_CRACKS,
            0x00ff60,
            0x003010,
        ),
        lo(80, 180, 40, 0, 0, DIR_DOWN, 12, 0),
    ],
    // L5: LOBE - subtle floor glow (minimal)
    [
        hi(OP_LOBE, REGION_FLOOR, BLEND_ADD, 0, 0x002000, 0x000000),
        lo(30, 128, 0, 0, 0, DIR_UP, 8, 0),
    ],
    // L6: SCATTER/WINDOWS - sparse falling glyphs
    [
        hi_meta(
            OP_SCATTER,
            REGION_SKY,
            BLEND_ADD,
            DOMAIN_DIRECT3D,
            SCATTER_WINDOWS,
            0x00ff00,
            0x000000,
        ),
        lo(60, 80, 12, 0x10, 42, DIR_DOWN, 10, 0),
    ],
    // L7: ATMOSPHERE/ALIEN - minimal green fog (keep void feel)
    [
        hi_meta(
            OP_ATMOSPHERE,
            REGION_ALL,
            BLEND_ADD,
            DOMAIN_DIRECT3D,
            ATMO_ALIEN,
            0x000400,
            0x000000,
        ),
        lo(15, 60, 0, 0, 0, DIR_UP, 6, 0),
    ],
];

// -----------------------------------------------------------------------------
// Preset 19: "Ancient Library" — Mystical archive
// -----------------------------------------------------------------------------
// Design: Ancient library with floating candles and magical tomes.
// Warm amber/gold, NOT neon. Completely different from Metropolis/Arcade.
// L0: SECTOR/BOX - enclosed library chamber
// L1: PLANE/TILES - marble floor
// L2: GRID - tall bookshelves
// L3: SCATTER/EMBERS - floating candle flames
// L4: LOBE - warm candlelight glow (HERO)
// L5: TRACE/FILAMENTS - magical glyphs on books
// L6: APERTURE/RECT - arched window with moonlight
// L7: ATMOSPHERE/MIE - dusty warm haze
pub(super) const PRESET_CYBER_SHRINE: [[u64; 2]; 8] = [
    // L0: SECTOR/BOX - enclosed library chamber
    [
        hi_meta(
            OP_SECTOR,
            REGION_ALL,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            SECTOR_BOX,
            0x100808,
            0x281810,
        ),
        lo(255, 140, 0, 0, 0, DIR_UP, 15, 15),
    ],
    // L1: PLANE/TILES - polished marble floor
    [
        hi_meta(
            OP_PLANE,
            REGION_FLOOR,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            PLANE_TILES,
            0x403020,
            0x281810,
        ),
        lo(200, 100, 0, 0, 0, DIR_UP, 15, 15),
    ],
    // L2: GRID - tall wooden bookshelves (warm brown)
    [
        hi(OP_GRID, REGION_WALLS, BLEND_LERP, 0, 0x402810, 0x281008),
        lo(180, 30, 60, 0, 0, DIR_UP, 15, 12),
    ],
    // L3: SCATTER/EMBERS - floating candle flames (warm orange)
    [
        hi_meta(
            OP_SCATTER,
            REGION_ALL,
            BLEND_ADD,
            DOMAIN_DIRECT3D,
            SCATTER_EMBERS,
            0xff8040,
            0xffc080,
        ),
        lo(80, 20, 15, 0x50, 13, DIR_UP, 12, 0),
    ],
    // L4: LOBE - warm candlelight glow (HERO - amber, not neon)
    [
        hi(OP_LOBE, REGION_ALL, BLEND_ADD, 0, 0xffa040, 0x804020),
        lo(200, 160, 80, 1, 0, DIR_UP, 15, 10),
    ],
    // L5: TRACE/FILAMENTS - magical glyphs glowing on spines
    [
        hi_meta(
            OP_TRACE,
            REGION_WALLS,
            BLEND_ADD,
            DOMAIN_TANGENT_LOCAL,
            TRACE_FILAMENTS,
            0x80c0ff,
            0x4080c0,
        ),
        lo(80, 80, 40, 60, 0, DIR_UP, 10, 0),
    ],
    // L6: APERTURE/RECT - arched window with cool moonlight
    [
        hi_meta(
            OP_APERTURE,
            REGION_WALLS,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            APERTURE_RECT,
            0x201810,
            0x4060a0,
        ),
        lo(180, 40, 100, 140, 0, DIR_FORWARD, 15, 15),
    ],
    // L7: ATMOSPHERE/MIE - dusty warm library haze
    [
        hi_meta(
            OP_ATMOSPHERE,
            REGION_ALL,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            ATMO_MIE,
            0x302010,
            0x100804,
        ),
        lo(50, 100, 128, 80, 160, DIR_UP, 10, 0),
    ],
];

// -----------------------------------------------------------------------------
// Preset 20: "Steampunk Airship" — Victorian observation deck
// -----------------------------------------------------------------------------
// Design: Airship cabin with visible internal structure - brass girders,
// riveted panels, porthole windows. Key is the structural framing.
//
// L0: RAMP (amber sky, brass floor, copper walls, SEMI)
// L1: SECTOR/BOX (cabin enclosure structure)
// L2: APERTURE/MULTI (porthole windows)
// L3: GRID (brass girder framework - key structural element)
// L4: CELL/HEX (riveted hex plates on walls)
// L5: CELESTIAL/SUN (setting sun through windows)
// L6: PLANE/GRATING (brass deck plating)
// L7: ATMOSPHERE/MIE (warm amber haze)
pub(super) const PRESET_STEAMPUNK_AIRSHIP: [[u64; 2]; 8] = [
    // L0: RAMP - amber sunset sky, burnished brass floor, copper walls
    [
        hi(OP_RAMP, REGION_ALL, BLEND_LERP, 0, 0xffa040, 0x503018),
        lo(190, 0x70, 0x48, 0x28, THRESH_SEMI, DIR_UP, 15, 15),
    ],
    // L1: SECTOR/BOX - cabin enclosure (creates interior structure)
    [
        hi_meta(
            OP_SECTOR,
            REGION_ALL,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            SECTOR_BOX,
            0x604028,
            0x402818,
        ),
        lo(180, 128, 0, 0, 0, DIR_UP, 15, 15),
    ],
    // L2: APERTURE/MULTI - porthole windows
    [
        hi_meta(
            OP_APERTURE,
            REGION_WALLS,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            APERTURE_MULTI,
            0x305070,
            0x8a5a1a,
        ),
        // softness=60, visible frame
        lo(160, 60, 40, 180, 4, DIR_BACK, 15, 15),
    ],
    // L3: GRID - brass girder framework (key structural element, stronger)
    [
        hi(OP_GRID, REGION_WALLS | REGION_SKY, BLEND_ADD, 0, 0xd0a050, 0x000000),
        // Higher intensity (160), wider bars (60)
        lo(160, 60, 30, 0, 0, DIR_UP, 15, 0),
    ],
    // L4: VEIL/PILLARS - steam columns (replaced hex - was too dominant)
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
        lo(40, 60, 80, 0, 0, DIR_UP, 8, 0),
    ],
    // L5: CELESTIAL/SUN - setting sun visible through porthole
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
        lo(140, 200, 0, 0, 0, DIR_SUNSET, 15, 0),
    ],
    // L6: PLANE/GRATING - brass deck plating (floor)
    [
        hi_meta(
            OP_PLANE,
            REGION_FLOOR,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            PLANE_GRATING,
            0x6a4a24,
            0x3a2810,
        ),
        lo(160, 90, 80, 60, 0, DIR_UP, 15, 12),
    ],
    // L7: ATMOSPHERE/MIE - warm amber haze (subtle)
    [
        hi_meta(
            OP_ATMOSPHERE,
            REGION_ALL,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            ATMO_MIE,
            0x503020,
            0x000000,
        ),
        lo(50, 100, 0, 0, 0, DIR_UP, 12, 0),
    ],
];
