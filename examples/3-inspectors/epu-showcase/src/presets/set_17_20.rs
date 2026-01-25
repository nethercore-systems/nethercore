//! Preset set 17-20

#[allow(unused_imports)]
use crate::constants::*;

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
pub(super) const PRESET_VOLCANIC_CORE: [[u64; 2]; 8] = [
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
        hi_meta(
            OP_SCATTER,
            REGION_ALL,
            BLEND_ADD,
            DOMAIN_DIRECT3D,
            SCATTER_EMBERS,
            0xff8000,
            0x000000,
        ),
        lo(180, 120, 100, 0x40, 0, DIR_UP, 15, 0),
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
pub(super) const PRESET_DIGITAL_MATRIX: [[u64; 2]; 8] = [
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
        hi_meta(
            OP_SCATTER,
            REGION_ALL,
            BLEND_SCREEN,
            DOMAIN_AXIS_CYL,
            SCATTER_RAIN,
            0x00ff00,
            0x000000,
        ),
        lo(180, 150, 200, 0, 0, DIR_DOWN, 15, 0),
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
pub(super) const PRESET_NOIR_DETECTIVE: [[u64; 2]; 8] = [
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
        hi_meta(
            OP_SCATTER,
            REGION_ALL,
            BLEND_ADD,
            DOMAIN_DIRECT3D,
            SCATTER_DUST,
            0x808070,
            0x000000,
        ),
        lo(100, 80, 40, 0x20, 0, DIR_UP, 10, 0),
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
pub(super) const PRESET_STEAMPUNK_AIRSHIP: [[u64; 2]; 8] = [
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
        hi_meta(
            OP_SCATTER,
            REGION_ALL,
            BLEND_ADD,
            DOMAIN_DIRECT3D,
            SCATTER_DUST,
            0xffe8c0,
            0x000000,
        ),
        lo(140, 100, 50, 0x20, 0, DIR_UP, 12, 0),
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

