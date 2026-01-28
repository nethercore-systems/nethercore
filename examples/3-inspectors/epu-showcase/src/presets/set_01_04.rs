//! Preset set 01-04

#[allow(unused_imports)]
use crate::constants::*;

// =============================================================================
// Environment Presets (EPU 128-bit format)
// =============================================================================

// -----------------------------------------------------------------------------
// Preset 1: "Neon Metropolis" - Rain-soaked cyberpunk alley
// -----------------------------------------------------------------------------
// Goal: unmistakably "city alley" (wet pavement, signage, rain) vs. Neon Arcade.
//
// L0: RAMP            ALL   LERP  base alley colors
// L1: SECTOR/TUNNEL   ALL   LERP  tight alley enclosure
// L2: SILHOUETTE/CITY WALLS LERP  skyline cutout band
// L3: PLANE/PAVEMENT  FLOOR LERP  wet pavement texture
// L4: GRID            WALLS ADD   subtle neon panel lines
// L5: DECAL           WALLS ADD   big neon sign (rect)
// L6: FLOW            ALL   SCREEN rain streaks (FLOW STREAKS)
// L7: ATMOSPHERE/FULL ALL   LERP  thin wet haze
pub(super) const PRESET_NEON_METROPOLIS: [[u64; 2]; 8] = [
    // L0: RAMP - deep purple sky, near-black floor, dark gray walls
    [
        hi(OP_RAMP, REGION_ALL, BLEND_LERP, 0, 0x1a0a2e, 0x080810),
        lo(220, 0x12, 0x12, 0x18, THRESH_INTERIOR, DIR_UP, 15, 15),
    ],
    // L1: SECTOR/TUNNEL - tunnel enclosure bounding the scene
    [
        hi_meta(
            OP_SECTOR,
            REGION_ALL,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            SECTOR_TUNNEL,
            0x120020,
            0x040408,
        ),
        lo(220, 80, 0, 0, 0, DIR_UP, 15, 15),
    ],
    // L2: SILHOUETTE/CITY - city skyline cutout on walls
    [
        hi_meta(
            OP_SILHOUETTE,
            REGION_WALLS,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            SILHOUETTE_CITY,
            0x0a0510,
            0x140a20,
        ),
        // softness, height_bias, roughness, octaves
        lo(60, 180, 180, 0x40, 0, DIR_UP, 15, 0),
    ],
    // L3: PLANE/PAVEMENT - wet asphalt underfoot
    [
        hi_meta(
            OP_PLANE,
            REGION_FLOOR,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            PLANE_PAVEMENT,
            0x0a0a10,
            0x050508,
        ),
        lo(180, 128, 0, 0, 0, DIR_UP, 15, 15),
    ],
    // L4: GRID - subtle cyan panel lines (kept restrained to avoid "arcade" feel)
    [
        hi(OP_GRID, REGION_WALLS, BLEND_ADD, 0, 0x00ffff, 0x000000),
        lo(120, 48, 0, 3, 0, DIR_UP, 10, 0),
    ],
    // L5: DECAL - one big neon sign (rect) to sell "alley"
    [
        hi(OP_DECAL, REGION_WALLS, BLEND_ADD, 0, 0xff00ff, 0x00ffff),
        // shape=RECT(2), soft=4, size=80, glow_soft=160
        lo(220, 0x24, 80, 160, 0, DIR_FORWARD, 12, 12),
    ],
    // L6: FLOW/STREAKS - rain streaks (downward)
    [
        hi(OP_FLOW, REGION_ALL, BLEND_SCREEN, 0, 0x00ddff, 0x000000),
        // intensity, scale, turbulence, octaves=2 + STREAKS(1)
        lo(160, 180, 40, 0x21, 0, DIR_DOWN, 12, 0),
    ],
    // L7: ATMOSPHERE/FULL - thin wet haze (keep horizon_y centered; avoid flat wash)
    [
        hi_meta(
            OP_ATMOSPHERE,
            REGION_ALL,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            ATMO_FULL,
            0x06060c,
            0x101020,
        ),
        lo(60, 90, 128, 0, 0, DIR_UP, 10, 0),
    ],
];

// -----------------------------------------------------------------------------
// Preset 2: "Crimson Hellscape" - Volcanic hellscape with dimensional rifts
// -----------------------------------------------------------------------------
// L0: RAMP (sky=#4a0000, floor=#0a0000, walls=#2a0808)
// L1: PATCHES/MEMBRANE (dark red #330000, organic tissue, DIRECT3D)
// L2: TRACE/CRACKS (lava veins #ff3300, floor, TANGENT_LOCAL)
// L3: FLOW (churning lava #ff4400, floor)
// L4: SCATTER/EMBERS (rising sparks #ff8800)
// L5: CELESTIAL/ECLIPSE (blood eclipse #200000/#ff0000, dir=SUN)
// L6: PORTAL/RIFT (dimensional tear #ff2200, walls, TANGENT_LOCAL)
// L7: ATMOSPHERE/ABSORPTION (blood mist #400000)
pub(super) const PRESET_CRIMSON_HELLSCAPE: [[u64; 2]; 8] = [
    // L0: RAMP - blood red sky, charred black floor, dark crimson walls
    [
        hi(OP_RAMP, REGION_ALL, BLEND_LERP, 0, 0x4a0000, 0x0a0000),
        lo(230, 0x2a, 0x08, 0x08, THRESH_BALANCED, DIR_UP, 15, 15),
    ],
    // L1: PATCHES/MEMBRANE - organic tissue texture on walls
    [
        hi_meta(
            OP_PATCHES,
            REGION_WALLS,
            BLEND_SCREEN,
            DOMAIN_DIRECT3D,
            PATCHES_MEMBRANE,
            0x330000,
            0x1a0000,
        ),
        lo(150, 96, 32, 0, 0, DIR_UP, 15, 0),
    ],
    // L2: TRACE/CRACKS - volcanic lava veins on floor
    [
        hi_meta(
            OP_TRACE,
            REGION_FLOOR,
            BLEND_ADD,
            DOMAIN_TANGENT_LOCAL,
            TRACE_CRACKS,
            0xff3300,
            0x000000,
        ),
        lo(130, 90, 64, 0, 0, DIR_UP, 15, 0),
    ],
    // L3: FLOW - churning lava glow on floor (intense, turbulent)
    [
        hi(OP_FLOW, REGION_FLOOR, BLEND_SCREEN, 0, 0xff4400, 0x000000),
        lo(200, 80, 60, 0x22, 64, DIR_UP, 15, 0),
    ],
    // L4: SCATTER/EMBERS - rising sparks and embers
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
        lo(110, 30, 20, 0x30, 0, DIR_UP, 15, 0),
    ],
    // L5: CELESTIAL/ECLIPSE - blood eclipse in the sky
    [
        hi_meta(
            OP_CELESTIAL,
            REGION_SKY,
            BLEND_ADD,
            DOMAIN_DIRECT3D,
            CELESTIAL_ECLIPSE,
            0x200000,
            0xff0000,
        ),
        lo(160, 200, 0, 0, 0, DIR_SUN, 15, 15),
    ],
    // L6: PORTAL/RIFT - dimensional tear on walls
    [
        hi_meta(
            OP_PORTAL,
            REGION_WALLS,
            BLEND_SCREEN,
            DOMAIN_TANGENT_LOCAL,
            PORTAL_RIFT,
            0xff2200,
            0x400000,
        ),
        lo(130, 90, 64, 0, 0, DIR_UP, 15, 0),
    ],
    // L7: ATMOSPHERE/ABSORPTION - thick blood mist
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
        lo(120, 100, 0, 0, 0, DIR_UP, 15, 0),
    ],
];

// -----------------------------------------------------------------------------
// Preset 3: "Frozen Tundra" - Arctic ice shelf, blizzard
// -----------------------------------------------------------------------------
// L0: RAMP (sky=#c8e0f0, floor=#f8f8ff, walls=#a0c8e0)
// L1: CELL/SHATTER (cracked ice #d0f0ff / #a0c8e0, floor)
// L2: PLANE/STONE (frozen ground #e8f4ff / #c0d8e8, floor)
// L3: SCATTER/SNOW (blizzard #ffffff, dir=DOWN)
// L4: FLOW (drifting snow clouds #ffffff, sky, dir=DOWN)
// L5: ATMOSPHERE/RAYLEIGH (crisp cold air #b0d8f0)
// L6: LOBE (pale sun glow #e0f0ff, dir=SUN)
// L7: NOP_LAYER
pub(super) const PRESET_FROZEN_TUNDRA: [[u64; 2]; 8] = [
    // L0: RAMP - cold blue everywhere (sky/wall/floor nearly same to avoid seams)
    [
        hi(OP_RAMP, REGION_ALL, BLEND_LERP, 0, 0x90a8c0, 0x90a8c0),
        lo(255, 0x90, 0xa8, 0xc0, THRESH_VAST, DIR_UP, 15, 15),
    ],
    // L1: CELL/SHATTER - cracked ice pattern on ALL regions (not just floor)
    [
        hi_meta(
            OP_CELL,
            REGION_ALL,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            CELL_SHATTER,
            0x102040,
            0x90b8d0,
        ),
        lo(220, 60, 160, 60, 0, DIR_UP, 15, 15),
    ],
    // L2: PLANE/STONE - frozen ground texture on floor
    [
        hi_meta(
            OP_PLANE,
            REGION_FLOOR,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            PLANE_STONE,
            0xc0d8e8,
            0x8098b0,
        ),
        lo(150, 96, 0, 0, 0, DIR_UP, 15, 15),
    ],
    // L3: SCATTER/SNOW - blizzard snowfall (downward)
    [
        hi_meta(
            OP_SCATTER,
            REGION_ALL,
            BLEND_ADD,
            DOMAIN_DIRECT3D,
            SCATTER_SNOW,
            0xc0d8ff,
            0x000000,
        ),
        lo(50, 60, 60, 0x20, 0, DIR_DOWN, 15, 0),
    ],
    // L4: FLOW - drifting snow clouds in sky (visible)
    [
        hi(OP_FLOW, REGION_SKY, BLEND_ADD, 0, 0xd0e0ff, 0x000000),
        lo(100, 128, 30, 0x21, 80, DIR_DOWN, 15, 0),
    ],
    // L5: ATMOSPHERE/RAYLEIGH - crisp cold arctic air
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
        lo(30, 60, 0, 0, 0, DIR_UP, 15, 0),
    ],
    // L6: LOBE - pale sun glow from above
    [
        hi(OP_LOBE, REGION_ALL, BLEND_ADD, 0, 0xe0f0ff, 0x000000),
        lo(30, 128, 0, 0, 0, DIR_SUN, 15, 0),
    ],
    // L7: (empty)
    NOP_LAYER,
];

// -----------------------------------------------------------------------------
// Preset 4: "Alien Jungle" - Bioluminescent alien canopy
// -----------------------------------------------------------------------------
// L0: RAMP (sky=#3a0050, floor=#002020, walls=#004040)
// L1: SILHOUETTE/FOREST (alien tree silhouettes #001818 / #003030, walls)
// L2: PATCHES/STREAKS (bioluminescent streaks #00ffaa / #004040, walls, AXIS_CYL)
// L3: VEIL/CURTAINS (hanging bioluminescent vines #8000ff, walls, AXIS_CYL)
// L4: SCATTER/DUST (floating spores #00ffcc)
// L5: FLOW (rippling bioluminescence #00ddcc, floor)
// L6: ATMOSPHERE/ALIEN (exotic gas #004020)
// L7: LOBE (canopy glow #3a0050, sky, dir=UP)
pub(super) const PRESET_ALIEN_JUNGLE: [[u64; 2]; 8] = [
    // L0: RAMP - jungle enclosure (keep region thresholds stable)
    [
        hi(OP_RAMP, REGION_ALL, BLEND_LERP, 0, 0x120020, 0x001008),
        lo(255, 0x00, 0x20, 0x18, THRESH_BALANCED, DIR_UP, 15, 15),
    ],
    // L1: SILHOUETTE/FOREST - alien tree silhouettes on walls
    [
        hi_meta(
            OP_SILHOUETTE,
            REGION_WALLS,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            SILHOUETTE_FOREST,
            0x000000,
            0x1a0030,
        ),
        // softness, height_bias, roughness, octaves
        lo(60, 120, 200, 0x60, 0, DIR_UP, 15, 0),
    ],
    // L2: PATCHES/BLOBS - punctual bioluminescent fungus patches (bound; avoid ADD)
    [
        hi_meta(
            OP_PATCHES,
            REGION_WALLS,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            PATCHES_BLOBS,
            0x1a0030,
            0x00cc88,
        ),
        // intensity(unused), scale, coverage, sharpness, seed
        lo(0, 120, 40, 40, 17, DIR_UP, 10, 10),
    ],
    // L3: FLOW/STREAKS - hanging purple vines (seam-free 3D flow domain)
    [
        hi(OP_FLOW, REGION_WALLS, BLEND_ADD, 0, 0x8000ff, 0x200040),
        lo(45, 80, 30, 0x21, 0, DIR_DOWN, 12, 0),
    ],
    // L4: SCATTER/DUST - floating spores (cyan/purple variation)
    [
        hi_meta(
            OP_SCATTER,
            REGION_ALL,
            BLEND_ADD,
            DOMAIN_DIRECT3D,
            SCATTER_DUST,
            0x00ffb0,
            0x8000ff,
        ),
        // Keep sparse; too much reads as "snow".
        lo(10, 16, 10, 0x00, 23, DIR_UP, 6, 0),
    ],
    // L5: FLOW/CAUSTIC - rippling bioluminescence on floor (animated)
    [
        hi(OP_FLOW, REGION_FLOOR, BLEND_ADD, 0, 0x00ff80, 0x008866),
        lo(45, 96, 40, 0x22, 0, DIR_UP, 12, 0),
    ],
    // L6: ATMOSPHERE/ALIEN - purple/teal exotic haze (keep subtle)
    [
        hi_meta(
            OP_ATMOSPHERE,
            REGION_ALL,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            ATMO_ALIEN,
            0x120020,
            0x002010,
        ),
        lo(30, 120, 128, 0, 0, DIR_UP, 10, 0),
    ],
    // L7: LOBE - canopy glow from above (purple, restrained)
    [
        hi(OP_LOBE, REGION_SKY, BLEND_ADD, 0, 0x8000ff, 0x000000),
        lo(40, 128, 0, 0, 0, DIR_DOWN, 12, 0),
    ],
];
