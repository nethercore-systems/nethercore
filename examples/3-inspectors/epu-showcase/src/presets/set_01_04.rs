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
// L2: SILHOUETTE/CITY SKY   LERP  skyline cutout band
// L3: PLANE/PAVEMENT  FLOOR LERP  wet pavement shimmer
// L4: GRID            WALLS ADD   subtle neon panel lines
// L5: APERTURE/RECT   WALLS SCREEN hero neon sign
// L6: FLOW            ALL   SCREEN rain streaks
// L7: ATMOSPHERE/FULL ALL   LERP  thin wet haze
pub(super) const PRESET_NEON_METROPOLIS: [[u64; 2]; 8] = [
    // L0: RAMP - deep navy sky, near-black floor, cool concrete walls
    [
        hi(OP_RAMP, REGION_ALL, BLEND_LERP, 0, 0x060a16, 0x010102),
        lo(235, 0x10, 0x14, 0x22, THRESH_SEMI, DIR_UP, 15, 15),
    ],
    // L1: SECTOR/BOX - tighter alley enclosure (avoid obvious "tube" read)
    [
        hi_meta(
            OP_SECTOR,
            REGION_ALL,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            SECTOR_TUNNEL,
            0x0b0f18,
            0x030406,
        ),
        lo(235, 115, 0, 0, 0, DIR_UP, 15, 15),
    ],
    // L2: CELL/BRICK - concrete/brick wall breakup (grounds the scene as an alley)
    [
        hi_meta(
            OP_SILHOUETTE,
            REGION_SKY,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            SILHOUETTE_CITY,
            0x020305,
            0x101a28,
        ),
        lo(170, 120, 165, 0x40, 0, DIR_UP, 15, 0),
    ],
    // L3: PLANE/PAVEMENT - wet asphalt underfoot
    [
        hi_meta(
            OP_PLANE,
            REGION_FLOOR,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            PLANE_PAVEMENT,
            0x06080b,
            0x0e131c,
        ),
        // param_d is phase (animated via ANIM_SPEEDS)
        lo(220, 90, 0, 0, 1, DIR_UP, 15, 15),
    ],
    // L4: GRID - subtle neon panel lines (DIRECT3D to avoid axial seams)
    [
        hi(OP_GRID, REGION_WALLS, BLEND_ADD, 0, 0x2a0f2a, 0x00b7ff),
        // cell_w, cell_h, phase (animated via ANIM_SPEEDS)
        lo(78, 22, 64, 0, 0, DIR_UP, 12, 6),
    ],
    // L5: APERTURE/RECT - hero neon sign (DIRECT3D: avoids tangent seams/rings)
    [
        hi_meta(
            OP_APERTURE,
            REGION_WALLS,
            BLEND_SCREEN,
            DOMAIN_DIRECT3D,
            APERTURE_RECT,
            0x061018,
            0x00d7ff,
        ),
        // intensity, softness, half_w, half_h, frame_thickness
        lo(155, 58, 56, 92, 10, DIR_LEFT, 15, 10),
    ],
    // L6: FLOW - rain streaks (DIRECT3D: no axial rings/seams; animated via ANIM_SPEEDS)
    [
        hi(OP_FLOW, REGION_ALL, BLEND_SCREEN, 0, 0xe7f2ff, 0x0e1520),
        lo(138, 232, 12, 0x21, 0, DIR_DOWN, 12, 4),
    ],
    // L7: ATMOSPHERE/MIE - thin wet haze (keeps contrast; adds depth)
    [
        hi_meta(
            OP_ATMOSPHERE,
            REGION_ALL,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            ATMO_MIE,
            0x0b1018,
            0x020306,
        ),
        lo(60, 85, 128, 140, 210, DIR_FORWARD, 10, 0),
    ],
];

// -----------------------------------------------------------------------------
// Preset 2: "Crimson Hellscape" - Volcanic hellscape with dimensional rifts
// -----------------------------------------------------------------------------
// Design: A breached caldera under a blood eclipse.
// The floor is a churning lava lake; a single wall-rift is the "hero" light source.
// Keep large shapes readable for the reflection sphere (avoid "confetti" noise).
//
// L0: RAMP (dark maroon sky, charred floor, obsidian-crimson walls; THRESH_INTERIOR)
// L1: SECTOR/CAVE (caldera enclosure; DIRECT3D)
// L2: LOBE (hot bounce light + eclipse rim; dir=SUNSET)
// L3: FLOW (lava lake motion; floor)
// L4: SCATTER/EMBERS (sparse ash/embers; sky+walls; static seed)
// L5: CELESTIAL/ECLIPSE (blood eclipse; dir=SUNSET)
// L6: PORTAL/RIFT (demonic breach in wall; TANGENT_LOCAL; animated phase)
// L7: ATMOSPHERE/ABSORPTION (sooty crimson haze)
pub(super) const PRESET_CRIMSON_HELLSCAPE: [[u64; 2]; 8] = [
    // L0: RAMP - enclosed caldera palette (keep sky narrow, walls dominant)
    [
        hi(OP_RAMP, REGION_ALL, BLEND_LERP, 0, 0x240000, 0x040000),
        lo(210, 0x18, 0x06, 0x04, THRESH_INTERIOR, DIR_UP, 15, 15),
    ],
    // L1: SECTOR/CAVE - crater enclosure (gives the reflection sphere big shapes)
    [
        hi_meta(
            OP_SECTOR,
            REGION_ALL,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            SECTOR_CAVE,
            0x120000,
            0x040000,
        ),
        lo(220, 140, 0, 0, 0, DIR_UP, 15, 15),
    ],
    // L2: PATCHES/MEMBRANE - sooty smoke sheets (big readable masses; seam-free)
    [
        hi_meta(
            OP_PATCHES,
            REGION_SKY | REGION_WALLS,
            BLEND_MULTIPLY,
            DOMAIN_DIRECT3D,
            PATCHES_MEMBRANE,
            0x2a0000,
            0x050000,
        ),
        lo(145, 108, 70, 0x10, 31, DIR_UP, 15, 0),
    ],
    // L3: FLOW - lava lake motion (directional so the highlight feels plausible)
    [
        hi_meta(
            OP_FLOW,
            REGION_FLOOR,
            BLEND_ADD,
            DOMAIN_TANGENT_LOCAL,
            0,
            0xff4a00,
            0x120000,
        ),
        lo(220, 150, 40, 0x18, 0, DIR_LEFT, 15, 0),
    ],
    // L4: SCATTER/EMBERS - sparse ash/embers (avoid full-screen confetti)
    [
        hi_meta(
            OP_SCATTER,
            REGION_SKY | REGION_WALLS,
            BLEND_ADD,
            DOMAIN_DIRECT3D,
            SCATTER_EMBERS,
            0xff7000,
            0x802000,
        ),
        lo(14, 9, 8, 0x18, 13, DIR_UP, 10, 0),
    ],
    // L5: CELESTIAL/ECLIPSE - a readable, directional "blood eclipse" key light
    [
        hi_meta(
            OP_CELESTIAL,
            REGION_SKY,
            BLEND_ADD,
            DOMAIN_DIRECT3D,
            CELESTIAL_ECLIPSE,
            0x080000,
            0xb00000,
        ),
        lo(130, 102, 165, 0, 110, DIR_UP, 15, 10),
    ],
    // L6: PORTAL/RIFT - the breach (anchored to the right wall for clean reflections)
    [
        hi_meta(
            OP_PORTAL,
            REGION_WALLS,
            BLEND_SCREEN,
            DOMAIN_TANGENT_LOCAL,
            PORTAL_RIFT,
            0xff9a20,
            0x200000,
        ),
        lo(165, 72, 110, 0x18, 0x20, DIR_RIGHT, 12, 9),
    ],
    // L7: ATMOSPHERE/ABSORPTION - sooty crimson haze (depth without killing contrast)
    [
        hi_meta(
            OP_ATMOSPHERE,
            REGION_ALL,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            ATMO_ABSORPTION,
            0x1a0000,
            0x000000,
        ),
        lo(95, 110, 0, 0, 0, DIR_UP, 15, 0),
    ],
];

// -----------------------------------------------------------------------------
// Preset 3: "Frozen Tundra" - Arctic ice shelf, blizzard
// -----------------------------------------------------------------------------
// Design: Stark contrast between dark mountains and bright ice. NOT washed out.
// L0: RAMP - dark blue sky, blue-white floor
// L1: SILHOUETTE/MOUNTAINS - dark mountain silhouettes (contrast!)
// L2: PLANE/STONE - frozen ground
// L3: FLOW - snowfall streaks
// L4: FLOW - spindrift haze
// L5: CELESTIAL/SUN - pale arctic sun
// L6: LOBE - sun glow
// L7: ATMOSPHERE/RAYLEIGH - thin crisp air
pub(super) const PRESET_FROZEN_TUNDRA: [[u64; 2]; 8] = [
    // L0: RAMP - deep arctic night sky, bright ice shelf (keep contrast)
    [
        hi(OP_RAMP, REGION_ALL, BLEND_LERP, 0, 0x050a18, 0x9ebddb),
        lo(195, 0x18, 0x24, 0x36, THRESH_VAST, DIR_UP, 15, 15),
    ],
    // L1: SILHOUETTE/MOUNTAINS - distant mountain rim (big, readable shapes)
    [
        hi_meta(
            OP_SILHOUETTE,
            REGION_SKY,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            SILHOUETTE_MOUNTAINS,
            0x010206,
            0x071428,
        ),
        // height_bias down for distance; roughness up for jagged peaks
        lo(105, 90, 160, 0x70, 0, DIR_UP, 15, 0),
    ],
    // L2: PLANE/STONE - ice shelf texture (keeps ground from reading as a flat gradient)
    [
        hi_meta(
            OP_PLANE,
            REGION_FLOOR,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            PLANE_STONE,
            0xd6efff,
            0x7a9ab8,
        ),
        // param_d is phase (animated via ANIM_SPEEDS)
        lo(210, 150, 115, 0, 1, DIR_UP, 15, 15),
    ],
    // L3: FLOW - falling snow (smooth motion; DIRECT3D avoids axial banding)
    [
        hi(OP_FLOW, REGION_ALL, BLEND_SCREEN, 0, 0xffffff, 0x0a1324),
        lo(120, 226, 10, 0x21, 0, DIR_DOWN, 12, 6),
    ],
    // L4: FLOW - spindrift haze (wind-driven, seam-free 3D noise)
    [
        hi_meta(
            OP_FLOW,
            REGION_SKY | REGION_WALLS,
            BLEND_SCREEN,
            DOMAIN_DIRECT3D,
            0,
            0xd6f0ff,
            0x0a1020,
        ),
        lo(70, 150, 26, 0x22, 0, DIR_LEFT, 10, 0),
    ],
    // L5: CELESTIAL/MOON - cold crescent (anchors reflections)
    [
        hi_meta(
            OP_CELESTIAL,
            REGION_SKY,
            BLEND_ADD,
            DOMAIN_DIRECT3D,
            CELESTIAL_MOON,
            0xf6fbff,
            0xa7c8ff,
        ),
        lo(120, 58, 165, 120, 0, DIR_UP, 12, 9),
    ],
    // L6: BAND - auroral arc (subtle; animated via ANIM_SPEEDS)
    [
        hi(OP_BAND, REGION_SKY, BLEND_ADD, 0, 0x6dffb0, 0x3f74d6),
        // width, vertical offset, edge softness, phase (animated)
        lo(78, 72, 180, 140, 0, DIR_UP, 7, 0),
    ],
    // L7: ATMOSPHERE/RAYLEIGH - low horizon haze (keeps value structure)
    [
        hi_meta(
            OP_ATMOSPHERE,
            REGION_ALL,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            ATMO_RAYLEIGH,
            0x08142c,
            0x1d3550,
        ),
        lo(55, 150, 90, 0, 0, DIR_UP, 6, 0),
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
// L4: PLANE/GRASS - jungle floor base
// L5: FLOW - biolum veins
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
            0x001612,
            0x002a1e,
        ),
        lo(235, 155, 0, 0, 0, DIR_UP, 15, 15),
    ],
    // L1: SILHOUETTE/FOREST - HERO: prominent alien trees (very tall, jagged)
    [
        hi_meta(
            OP_SILHOUETTE,
            REGION_WALLS | REGION_SKY,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            SILHOUETTE_FOREST,
            0x000000,
            0x001a12,
        ),
        // Very low softness=10 for sharp edges, very high height_bias=250, high roughness
        lo(210, 60, 220, 0x60, 0, DIR_UP, 15, 0),
    ],
    // L2: VEIL/CURTAINS - hanging bioluminescent vines
    [
        hi_meta(
            OP_VEIL,
            REGION_WALLS,
            BLEND_LERP,
            DOMAIN_TANGENT_LOCAL,
            VEIL_CURTAINS,
            0x0c2a18,
            0x00ff98,
        ),
        lo(80, 130, 55, 120, 0, DIR_DOWN, 12, 10),
    ],
    // L3: SCATTER/DUST - floating spores (bioluminescent)
    [
        hi_meta(
            OP_SCATTER,
            REGION_ALL,
            BLEND_ADD,
            DOMAIN_DIRECT3D,
            SCATTER_DUST,
            0xc6ffe4,
            0x2cff9a,
        ),
        lo(12, 16, 9, 0x28, 23, DIR_UP, 10, 0),
    ],
    // L4: PLANE/GRASS - organic jungle floor base (subtle animated sheen)
    [
        hi_meta(
            OP_PLANE,
            REGION_FLOOR,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            PLANE_GRASS,
            0x062016,
            0x0f3b28,
        ),
        // param_d is phase (animated via ANIM_SPEEDS)
        lo(220, 115, 90, 0, 1, DIR_UP, 15, 15),
    ],
    // L5: FLOW - biolum fungal veins (smooth, avoids TRACE scanline artifacts)
    [
        hi(OP_FLOW, REGION_FLOOR, BLEND_ADD, 0, 0x00ff82, 0x00160c),
        lo(95, 140, 26, 0x22, 0, DIR_FORWARD, 12, 0),
    ],
    // L6: LOBE - purple canopy glow from above
    [
        hi(OP_LOBE, REGION_ALL, BLEND_ADD, 0, 0xa0ff80, 0x001010),
        lo(70, 200, 70, 1, 0, DIR_DOWN, 12, 6),
    ],
    // L7: ATMOSPHERE/ALIEN - exotic purple/teal haze
    [
        hi_meta(
            OP_ATMOSPHERE,
            REGION_ALL,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            ATMO_ALIEN,
            0x0a1a10,
            0x001008,
        ),
        lo(55, 120, 128, 0, 0, DIR_UP, 10, 0),
    ],
];
