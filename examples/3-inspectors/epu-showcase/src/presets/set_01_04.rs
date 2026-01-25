//! Preset set 01-04

#[allow(unused_imports)]
use crate::constants::*;

// =============================================================================
// Environment Presets (EPU 128-bit format)
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
pub(super) const PRESET_NEON_METROPOLIS: [[u64; 2]; 8] = [
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
        hi_meta(
            OP_SCATTER,
            REGION_WALLS,
            BLEND_ADD,
            DOMAIN_AXIS_CYL,
            SCATTER_WINDOWS,
            0xffcc00,
            0x000000,
        ),
        lo(180, 128, 0, 0x20, 0, DIR_UP, 15, 0),
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
pub(super) const PRESET_CRIMSON_HELLSCAPE: [[u64; 2]; 8] = [
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
        hi_meta(
            OP_SCATTER,
            REGION_ALL,
            BLEND_ADD,
            DOMAIN_DIRECT3D,
            SCATTER_EMBERS,
            0xff8800,
            0x000000,
        ),
        lo(180, 100, 100, 0x30, 0, DIR_UP, 15, 0),
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
pub(super) const PRESET_FROZEN_TUNDRA: [[u64; 2]; 8] = [
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
        hi_meta(
            OP_SCATTER,
            REGION_ALL,
            BLEND_ADD,
            DOMAIN_DIRECT3D,
            SCATTER_SNOW,
            0xffffff,
            0x000000,
        ),
        lo(160, 128, 100, 0x20, 0, DIR_DOWN, 15, 0),
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
pub(super) const PRESET_ALIEN_JUNGLE: [[u64; 2]; 8] = [
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
        hi_meta(
            OP_SCATTER,
            REGION_ALL,
            BLEND_ADD,
            DOMAIN_DIRECT3D,
            SCATTER_DUST,
            0x00ffcc,
            0x000000,
        ),
        lo(170, 100, 80, 0x30, 0, DIR_UP, 15, 0),
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

