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
    // L1: SECTOR/TUNNEL - tunnel enclosure bounding the scene (stronger for depth)
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
        lo(255, 100, 0, 0, 0, DIR_UP, 15, 15),
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
    // L6: FLOW/STREAKS - rain streaks (downward, TANGENT_LOCAL to avoid barrel wrap)
    [
        hi_meta(OP_FLOW, REGION_ALL, BLEND_SCREEN, DOMAIN_TANGENT_LOCAL, 0, 0x00ddff, 0x000000),
        // intensity, scale, turbulence, octaves=2 + STREAKS(1)
        lo(120, 160, 30, 0x21, 0, DIR_DOWN, 10, 0),
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
// Design: Stark contrast between dark mountains and bright ice. NOT washed out.
// L0: RAMP - dark blue sky, blue-white floor
// L1: SILHOUETTE/MOUNTAINS - dark mountain silhouettes (contrast!)
// L2: PLANE/STONE - frozen ground
// L3: SCATTER/SNOW - blizzard
// L4: FLOW - drifting snow
// L5: CELESTIAL/SUN - pale arctic sun
// L6: LOBE - sun glow
// L7: ATMOSPHERE/RAYLEIGH - thin crisp air
pub(super) const PRESET_FROZEN_TUNDRA: [[u64; 2]; 8] = [
    // L0: RAMP - DARK stormy blue sky, pale ice floor (HIGH CONTRAST)
    [
        hi(OP_RAMP, REGION_ALL, BLEND_LERP, 0, 0x101830, 0x90a8c0),
        lo(255, 0x50, 0x60, 0x70, THRESH_VAST, DIR_UP, 15, 15),
    ],
    // L1: SILHOUETTE/MOUNTAINS - VERY DARK mountain range (strong contrast)
    [
        hi_meta(
            OP_SILHOUETTE,
            REGION_WALLS,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            SILHOUETTE_MOUNTAINS,
            0x080810,
            0x182030,
        ),
        // Low height_bias for distant peaks, high roughness for jagged
        lo(255, 120, 200, 0x50, 0, DIR_UP, 15, 15),
    ],
    // L2: PLANE/STONE - pale frozen ground texture
    [
        hi_meta(
            OP_PLANE,
            REGION_FLOOR,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            PLANE_STONE,
            0x8090a8,
            0x606878,
        ),
        lo(200, 96, 0, 0, 0, DIR_UP, 15, 15),
    ],
    // L3: SCATTER/SNOW - blizzard snowfall
    [
        hi_meta(
            OP_SCATTER,
            REGION_ALL,
            BLEND_ADD,
            DOMAIN_DIRECT3D,
            SCATTER_SNOW,
            0xffffff,
            0xa0c0e0,
        ),
        lo(50, 50, 30, 0x30, 0, DIR_DOWN, 10, 0),
    ],
    // L4: FLOW - drifting snow clouds in sky only
    [
        hi(OP_FLOW, REGION_SKY, BLEND_ADD, 0, 0x607080, 0x000000),
        lo(60, 140, 40, 0x21, 80, DIR_DOWN, 8, 0),
    ],
    // L5: CELESTIAL/SUN - pale arctic sun (smaller, brighter)
    [
        hi_meta(
            OP_CELESTIAL,
            REGION_SKY,
            BLEND_ADD,
            DOMAIN_DIRECT3D,
            CELESTIAL_SUN,
            0xf0f8ff,
            0x000000,
        ),
        lo(180, 80, 160, 0, 140, DIR_SUN, 15, 10),
    ],
    // L6: LOBE - subtle sun glow halo
    [
        hi(OP_LOBE, REGION_SKY, BLEND_ADD, 0, 0x8090a0, 0x000000),
        lo(60, 120, 60, 1, 0, DIR_SUN, 10, 0),
    ],
    // L7: ATMOSPHERE/RAYLEIGH - minimal haze (preserve contrast!)
    [
        hi_meta(
            OP_ATMOSPHERE,
            REGION_ALL,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            ATMO_RAYLEIGH,
            0x304050,
            0x000000,
        ),
        lo(20, 140, 128, 0, 0, DIR_UP, 6, 0),
    ],
];

// -----------------------------------------------------------------------------
// Preset 4: "Alien Jungle" - Bioluminescent alien canopy
// -----------------------------------------------------------------------------
// Design: Dense alien forest with SILHOUETTE/FOREST as dominant feature.
// Hanging vines, glowing fungi, floating spores.
// L0: SECTOR/CAVE - dense jungle enclosure
// L1: SILHOUETTE/FOREST - prominent alien tree silhouettes (HERO)
// L2: VEIL/CURTAINS - hanging vines
// L3: SCATTER/DUST - floating spores
// L4: FLOW - bioluminescent floor glow
// L5: TRACE/ROOTS - glowing root network
// L6: LOBE - canopy light filtering down
// L7: ATMOSPHERE/ALIEN - exotic haze
pub(super) const PRESET_ALIEN_JUNGLE: [[u64; 2]; 8] = [
    // L0: SECTOR/CAVE - dense jungle enclosure as base
    [
        hi_meta(
            OP_SECTOR,
            REGION_ALL,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            SECTOR_CAVE,
            0x001010,
            0x002018,
        ),
        lo(255, 140, 0, 0, 0, DIR_UP, 15, 15),
    ],
    // L1: SILHOUETTE/FOREST - HERO: prominent alien trees (very tall, jagged)
    [
        hi_meta(
            OP_SILHOUETTE,
            REGION_WALLS,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            SILHOUETTE_FOREST,
            0x000000,
            0x002820,
        ),
        // Very low softness=10 for sharp edges, very high height_bias=250, high roughness
        lo(255, 10, 250, 0x80, 0, DIR_UP, 15, 15),
    ],
    // L2: VEIL/CURTAINS - hanging bioluminescent vines
    [
        hi_meta(
            OP_VEIL,
            REGION_WALLS,
            BLEND_ADD,
            DOMAIN_AXIS_CYL,
            VEIL_CURTAINS,
            0x00ff80,
            0x8000ff,
        ),
        lo(100, 100, 30, 120, 0, DIR_DOWN, 12, 0),
    ],
    // L3: SCATTER/DUST - floating spores (bioluminescent)
    [
        hi_meta(
            OP_SCATTER,
            REGION_ALL,
            BLEND_ADD,
            DOMAIN_DIRECT3D,
            SCATTER_DUST,
            0x00ffb0,
            0xff00ff,
        ),
        lo(40, 30, 20, 0x40, 23, DIR_UP, 10, 0),
    ],
    // L4: FLOW - rippling bioluminescence on floor
    [
        hi(OP_FLOW, REGION_FLOOR, BLEND_ADD, 0, 0x00ffa0, 0x008060),
        lo(80, 100, 50, 0x22, 0, DIR_UP, 15, 0),
    ],
    // L5: TRACE/FILAMENTS - glowing root network on floor
    [
        hi_meta(
            OP_TRACE,
            REGION_FLOOR,
            BLEND_ADD,
            DOMAIN_TANGENT_LOCAL,
            TRACE_FILAMENTS,
            0x00ff60,
            0x004020,
        ),
        lo(100, 120, 60, 80, 0, DIR_UP, 12, 0),
    ],
    // L6: LOBE - purple canopy glow from above
    [
        hi(OP_LOBE, REGION_ALL, BLEND_ADD, 0, 0x8040ff, 0x002020),
        lo(80, 160, 60, 1, 0, DIR_DOWN, 12, 6),
    ],
    // L7: ATMOSPHERE/ALIEN - exotic purple/teal haze
    [
        hi_meta(
            OP_ATMOSPHERE,
            REGION_ALL,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            ATMO_ALIEN,
            0x100818,
            0x001008,
        ),
        lo(40, 100, 128, 0, 0, DIR_UP, 10, 0),
    ],
];
