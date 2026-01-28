//! Preset set 13-16

#[allow(unused_imports)]
use crate::constants::*;

// -----------------------------------------------------------------------------
// Preset 13: "Enchanted Grove" - Fairy tale forest
// -----------------------------------------------------------------------------
// Design: Forest clearing with golden sunbeam. NO harsh lines.
// Use SECTOR for enclosure, not SILHOUETTE (which creates harsh horizon).
// L0: SECTOR/CAVE - forest enclosure (soft, no harsh line)
// L1: PLANE/GRASS - mossy floor
// L2: VEIL/CURTAINS - soft tree canopy edges (not harsh silhouette)
// L3: APERTURE/CIRCLE - clearing in canopy
// L4: LOBE - HERO: golden sunbeam
// L5: SCATTER/DUST - fairy motes
// L6: CELESTIAL/SUN - sun disk
// L7: ATMOSPHERE/MIE - golden haze
pub(super) const PRESET_ENCHANTED_GROVE: [[u64; 2]; 8] = [
    // L0: SECTOR/CAVE - forest enclosure (soft edges, no harsh line)
    [
        hi_meta(
            OP_SECTOR,
            REGION_ALL,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            SECTOR_CAVE,
            0x102808,
            0x305020,
        ),
        lo(220, 180, 0, 0, 0, DIR_UP, 15, 15),
    ],
    // L1: PLANE/GRASS - mossy forest floor
    [
        hi_meta(
            OP_PLANE,
            REGION_FLOOR,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            PLANE_GRASS,
            0x305030,
            0x203820,
        ),
        lo(200, 100, 0, 0, 0, DIR_UP, 15, 15),
    ],
    // L2: VEIL/CURTAINS - soft hanging foliage (not harsh silhouette)
    [
        hi_meta(
            OP_VEIL,
            REGION_WALLS,
            BLEND_LERP,
            DOMAIN_AXIS_CYL,
            VEIL_CURTAINS,
            0x183010,
            0x284820,
        ),
        lo(120, 140, 60, 100, 0, DIR_DOWN, 12, 10),
    ],
    // L3: APERTURE/CIRCLE - clearing in canopy showing golden sky
    [
        hi_meta(
            OP_APERTURE,
            REGION_SKY,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            APERTURE_CIRCLE,
            0x406028,
            0xfff8d0,
        ),
        lo(240, 120, 140, 100, 0, DIR_UP, 15, 15),
    ],
    // L4: LOBE - HERO: golden sunbeam cone
    [
        hi(OP_LOBE, REGION_ALL, BLEND_ADD, 0, 0xffd060, 0x906030),
        lo(255, 180, 80, 1, 0, DIR_SUN, 15, 12),
    ],
    // L5: SCATTER/DUST - sparse fairy motes
    [
        hi_meta(
            OP_SCATTER,
            REGION_ALL,
            BLEND_ADD,
            DOMAIN_DIRECT3D,
            SCATTER_DUST,
            0xffd080,
            0xffffff,
        ),
        lo(40, 15, 10, 0x60, 17, DIR_UP, 10, 0),
    ],
    // L6: CELESTIAL/SUN - sun disk visible through clearing
    [
        hi_meta(
            OP_CELESTIAL,
            REGION_SKY,
            BLEND_ADD,
            DOMAIN_DIRECT3D,
            CELESTIAL_SUN,
            0xfff0c0,
            0x000000,
        ),
        lo(180, 80, 200, 0, 100, DIR_SUN, 15, 10),
    ],
    // L7: ATMOSPHERE/MIE - warm golden forest haze
    [
        hi_meta(
            OP_ATMOSPHERE,
            REGION_ALL,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            ATMO_MIE,
            0x504020,
            0x102008,
        ),
        lo(60, 100, 128, 80, 160, DIR_SUN, 10, 0),
    ],
];

// -----------------------------------------------------------------------------
// Preset 14: "Astral Void" - Cosmic void
// -----------------------------------------------------------------------------
// L0: RAMP (sky=#000004, floor=#080010, walls=#100020)
// L1: PATCHES/BLOBS (purple #200840 / #100420, nebula gas clouds)
// L2: FLOW (purple #4000a0, swirling cosmic gases)
// L3: SCATTER/STARS (white #ffffff, dense starfield)
// L4: CELESTIAL/GAS_GIANT (orange #ff6040, dir=SUN)
// L5: CELESTIAL/RINGED (gold #d0c080, dir=SUNSET)
// L6: PORTAL/VORTEX (white #ffffff / purple #8040ff, TANGENT_LOCAL)
// L7: BAND (purple #4020a0, nebula horizon glow)
pub(super) const PRESET_ASTRAL_VOID: [[u64; 2]; 8] = [
    // L0: RAMP - void black sky, deep purple floor, indigo walls
    [
        hi(OP_RAMP, REGION_ALL, BLEND_LERP, 0, 0x000004, 0x080010),
        lo(200, 0x10, 0x00, 0x20, THRESH_VAST, DIR_UP, 15, 15),
    ],
    // L1: PATCHES/BLOBS - nebula gas clouds (deep purple)
    [
        hi_meta(
            OP_PATCHES,
            REGION_ALL,
            BLEND_SCREEN,
            DOMAIN_AXIS_POLAR,
            PATCHES_BLOBS,
            0x200840,
            0x100420,
        ),
        lo(120, 128, 80, 0, 0, DIR_UP, 15, 15),
    ],
    // L2: FLOW - swirling cosmic gases (purple)
    [
        hi(OP_FLOW, REGION_ALL, BLEND_SCREEN, 0, 0x4000a0, 0x000000),
        lo(100, 128, 0, 0x22, 100, DIR_UP, 15, 0),
    ],
    // L3: SCATTER/STARS - dense starfield (white)
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
        lo(140, 60, 0, 0x40, 0, DIR_UP, 15, 0),
    ],
    // L4: CELESTIAL/GAS_GIANT - massive gas giant (orange, dir=SUN)
    [
        hi_meta(
            OP_CELESTIAL,
            REGION_SKY,
            BLEND_ADD,
            DOMAIN_DIRECT3D,
            CELESTIAL_GAS_GIANT,
            0xff6040,
            0x000000,
        ),
        lo(160, 220, 0, 0, 0, DIR_SUN, 15, 0),
    ],
    // L5: CELESTIAL/RINGED - ringed planet (gold, dir=SUNSET)
    [
        hi_meta(
            OP_CELESTIAL,
            REGION_SKY,
            BLEND_ADD,
            DOMAIN_DIRECT3D,
            CELESTIAL_RINGED,
            0xd0c080,
            0x000000,
        ),
        lo(140, 128, 0, 0, 0, DIR_SUNSET, 15, 0),
    ],
    // L6: PORTAL/VORTEX - cosmic vortex on walls (white/purple, tangent local)
    [
        hi_meta(
            OP_PORTAL,
            REGION_WALLS,
            BLEND_SCREEN,
            DOMAIN_TANGENT_LOCAL,
            PORTAL_VORTEX,
            0xffffff,
            0x8040ff,
        ),
        lo(130, 100, 0, 0, 0, DIR_UP, 15, 15),
    ],
    // L7: BAND - nebula horizon glow (purple)
    [
        hi(OP_BAND, REGION_ALL, BLEND_ADD, 0, 0x4020a0, 0x000000),
        lo(110, 128, 0, 0, 0, DIR_SUNSET, 15, 0),
    ],
];

// -----------------------------------------------------------------------------
// Preset 15: "Toxic Wasteland" - Post-apocalyptic industrial
// -----------------------------------------------------------------------------
// Design: Simple dark industrial with bright green toxic glow. No complex patterns.
// L0: SECTOR/BOX - dark industrial enclosure
// L1: PLANE/TILES - concrete floor
// L2: PORTAL/VORTEX - HERO: glowing toxic pool (center)
// L3: TRACE/CRACKS - toxic veins on floor
// L4: SCATTER/EMBERS - toxic particles
// L5: LOBE - radioactive upward glow
// L6: FLOW - toxic smoke rising
// L7: ATMOSPHERE/ALIEN - green haze
pub(super) const PRESET_TOXIC_WASTELAND: [[u64; 2]; 8] = [
    // L0: SECTOR/BOX - industrial enclosure (lighter to show glow)
    [
        hi_meta(
            OP_SECTOR,
            REGION_ALL,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            SECTOR_BOX,
            0x101008,
            0x281810,
        ),
        lo(220, 140, 0, 0, 0, DIR_UP, 15, 15),
    ],
    // L1: PLANE/TILES - cracked concrete floor (brown, not green)
    [
        hi_meta(
            OP_PLANE,
            REGION_FLOOR,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            PLANE_TILES,
            0x302010,
            0x201008,
        ),
        lo(220, 100, 0, 0, 0, DIR_UP, 15, 15),
    ],
    // L2: PORTAL/VORTEX - HERO: glowing toxic waste pool (BRIGHT)
    [
        hi_meta(
            OP_PORTAL,
            REGION_FLOOR,
            BLEND_ADD,
            DOMAIN_TANGENT_LOCAL,
            PORTAL_VORTEX,
            0x40ff20,
            0x80ff40,
        ),
        lo(255, 120, 200, 140, 0, DIR_DOWN, 15, 15),
    ],
    // L3: TRACE/CRACKS - bright toxic veins radiating from pool
    [
        hi_meta(
            OP_TRACE,
            REGION_FLOOR,
            BLEND_ADD,
            DOMAIN_TANGENT_LOCAL,
            TRACE_CRACKS,
            0x60ff30,
            0x30c018,
        ),
        lo(255, 140, 100, 80, 0, DIR_UP, 15, 0),
    ],
    // L4: SCATTER/EMBERS - toxic green particles rising
    [
        hi_meta(
            OP_SCATTER,
            REGION_ALL,
            BLEND_ADD,
            DOMAIN_DIRECT3D,
            SCATTER_EMBERS,
            0x60ff40,
            0x40c020,
        ),
        lo(100, 30, 20, 0x40, 7, DIR_UP, 15, 0),
    ],
    // L5: LOBE - BRIGHT radioactive glow from pool (HERO visibility)
    [
        hi(OP_LOBE, REGION_ALL, BLEND_ADD, 0, 0x60ff30, 0x40c020),
        lo(255, 180, 120, 1, 0, DIR_UP, 15, 15),
    ],
    // L6: FLOW - toxic smoke rising (subtle, no harsh pattern)
    [
        hi(OP_FLOW, REGION_SKY | REGION_WALLS, BLEND_ADD, 0, 0x306010, 0x000000),
        lo(60, 100, 40, 0x21, 0, DIR_UP, 10, 0),
    ],
    // L7: ATMOSPHERE/ALIEN - BRIGHT green poisonous haze (HERO)
    [
        hi_meta(
            OP_ATMOSPHERE,
            REGION_ALL,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            ATMO_ALIEN,
            0x40a020,
            0x102008,
        ),
        lo(40, 100, 128, 0, 0, DIR_UP, 10, 0),
    ],
];

// -----------------------------------------------------------------------------
// Preset 16: "Moonlit Graveyard" - Gothic horror
// -----------------------------------------------------------------------------
// Design: Surrounded by gravestones with moon visible. SILHOUETTE as base.
// LOW height_bias for gravestone-height shapes, NOT blocking the moon.
// L0: SILHOUETTE/SPIRES - gravestone silhouettes as BASE (low height!)
// L1: PLANE/STONE - weathered graveyard ground
// L2: CELESTIAL/MOON - bright full moon (unobstructed)
// L3: BAND - eerie horizon glow
// L4: SCATTER/STARS - night sky stars
// L5: VEIL/CURTAINS - ground mist
// L6: LOBE - moonlight glow
// L7: ATMOSPHERE/FULL - night fog
pub(super) const PRESET_MOONLIT_GRAVEYARD: [[u64; 2]; 8] = [
    // L0: SILHOUETTE/SPIRES - gravestones as BASE (LOW height for tombstones)
    [
        hi_meta(
            OP_SILHOUETTE,
            REGION_ALL,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            SILHOUETTE_SPIRES,
            0x000004,
            0x182030,
        ),
        // LOW height_bias=60 for short gravestones, high roughness for irregular shapes
        lo(255, 60, 200, 0x50, 0, DIR_UP, 15, 15),
    ],
    // L1: PLANE/STONE - weathered graveyard ground
    [
        hi_meta(
            OP_PLANE,
            REGION_FLOOR,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            PLANE_STONE,
            0x202020,
            0x141418,
        ),
        lo(180, 128, 0, 0, 0, DIR_UP, 15, 15),
    ],
    // L2: PLANE/STONE - weathered graveyard path (gray)
    [
        hi_meta(
            OP_PLANE,
            REGION_FLOOR,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            PLANE_STONE,
            0x282828,
            0x1a1a20,
        ),
        lo(140, 128, 0, 0, 0, DIR_UP, 15, 15),
    ],
    // L3: CELESTIAL/MOON - full moon (smaller; keep contrast)
    [
        hi_meta(
            OP_CELESTIAL,
            REGION_SKY,
            BLEND_ADD,
            DOMAIN_DIRECT3D,
            CELESTIAL_MOON,
            0xe0e8f0,
            0x000000,
        ),
        lo(200, 160, 0, 0, 0, DIR_SUN, 15, 0),
    ],
    // L4: BAND - eerie blue horizon glow
    [
        hi(OP_BAND, REGION_SKY, BLEND_ADD, 0, 0x202840, 0x000000),
        lo(110, 128, 0, 0, 0, DIR_SUNSET, 15, 0),
    ],
    // L5: SCATTER/STARS - stars in sky only (not on ground)
    [
        hi_meta(
            OP_SCATTER,
            REGION_SKY,
            BLEND_ADD,
            DOMAIN_DIRECT3D,
            SCATTER_STARS,
            0xc0c8d0,
            0x000000,
        ),
        lo(60, 40, 8, 0x30, 0, DIR_UP, 10, 0),
    ],
    // L6: VEIL/CURTAINS - hanging mist (thinner)
    [
        hi_meta(
            OP_VEIL,
            REGION_WALLS,
            BLEND_ADD,
            DOMAIN_AXIS_CYL,
            VEIL_CURTAINS,
            0x404050,
            0x000000,
        ),
        lo(40, 60, 40, 100, 0, DIR_DOWN, 6, 0),
    ],
    // L7: ATMOSPHERE/FULL - light night fog (reduced to show gravestones)
    [
        hi_meta(
            OP_ATMOSPHERE,
            REGION_ALL,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            ATMO_FULL,
            0x101828,
            0x000000,
        ),
        lo(40, 100, 128, 0, 0, DIR_UP, 8, 0),
    ],
];
