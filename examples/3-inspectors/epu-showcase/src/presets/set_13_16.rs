//! Preset set 13-16

#[allow(unused_imports)]
use crate::constants::*;

// -----------------------------------------------------------------------------
// Preset 13: "Enchanted Grove" - Fairy tale forest
// -----------------------------------------------------------------------------
// L0: RAMP (sky=#fff8d0, floor=#204020, walls=#1a3820)
// L1: SILHOUETTE/FOREST (dark green #0a2010 / #1a3820, forest silhouette)
// L2: PLANE/GRASS (green #308030 / #204020, lush floor)
// L3: VEIL/CURTAINS (green #40a040, hanging moss, AXIS_CYL)
// L4: SCATTER/DUST (gold #ffdd00, fairy dust)
// L5: PATCHES/BLOBS (yellow #fff080, dappled sunlight)
// L6: LOBE (gold #ffd700, sunbeam, dir=SUN)
// L7: FLOW (green #60a060, gentle leaf movement)
pub(super) const PRESET_ENCHANTED_GROVE: [[u64; 2]; 8] = [
    // L0: RAMP - golden sky, mossy floor, forest green walls
    [
        hi(OP_RAMP, REGION_ALL, BLEND_LERP, 0, 0xe0d8a0, 0x204020),
        lo(220, 0x1a, 0x38, 0x20, THRESH_OPEN, DIR_UP, 15, 15),
    ],
    // L1: SILHOUETTE/FOREST - tree silhouettes on walls (deep green)
    [
        hi_meta(
            OP_SILHOUETTE,
            REGION_WALLS,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            SILHOUETTE_FOREST,
            0x000800,
            0x0a1808,
        ),
        lo(160, 128, 0, 0, 0, DIR_UP, 15, 15),
    ],
    // L2: PLANE/GRASS - lush forest floor (vibrant green)
    [
        hi_meta(
            OP_PLANE,
            REGION_FLOOR,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            PLANE_GRASS,
            0x308030,
            0x204020,
        ),
        lo(140, 128, 0, 0, 0, DIR_UP, 15, 15),
    ],
    // L3: VEIL/CURTAINS - hanging moss/vines (green, cylindrical domain)
    [
        hi_meta(
            OP_VEIL,
            REGION_WALLS,
            BLEND_ADD,
            DOMAIN_AXIS_CYL,
            VEIL_CURTAINS,
            0x40a040,
            0x000000,
        ),
        lo(100, 128, 160, 0, 0, DIR_DOWN, 15, 0),
    ],
    // L4: SCATTER/DUST - fairy dust particles (gold)
    [
        hi_meta(
            OP_SCATTER,
            REGION_ALL,
            BLEND_ADD,
            DOMAIN_DIRECT3D,
            SCATTER_DUST,
            0xffdd00,
            0x000000,
        ),
        lo(110, 25, 60, 0x20, 0, DIR_UP, 15, 0),
    ],
    // L5: PATCHES/BLOBS - dappled sunlight filtering through canopy (yellow)
    [
        hi_meta(
            OP_PATCHES,
            REGION_SKY,
            BLEND_SCREEN,
            DOMAIN_DIRECT3D,
            PATCHES_BLOBS,
            0xfff080,
            0x000000,
        ),
        lo(120, 128, 80, 0, 0, DIR_UP, 15, 0),
    ],
    // L6: LOBE - golden sunbeam through canopy (sine pulse, dir=SUN)
    [
        hi(OP_LOBE, REGION_ALL, BLEND_ADD, 0, 0xffd700, 0x000000),
        lo(220, 128, 0, 1, 0, DIR_SUN, 15, 0),
    ],
    // L7: FLOW - gentle leaf movement on forest floor (green)
    [
        hi(OP_FLOW, REGION_FLOOR, BLEND_ADD, 0, 0x60a060, 0x000000),
        lo(80, 128, 0, 0, 60, DIR_UP, 15, 0),
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
        lo(100, 128, 0, 4, 100, DIR_UP, 15, 0),
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
// L0: RAMP (sky=#304010, floor=#202008, walls=#283018)
// L1: SILHOUETTE/INDUSTRIAL (dark #181808 / #283018, factory smokestacks)
// L2: PATCHES/ISLANDS (green #40a000 / #204000, radioactive puddles)
// L3: PLANE/TILES (brown #483820 / #302810, cracked industrial floor)
// L4: CELL/HEX (yellow-green #a0a000, hazmat pattern)
// L5: VEIL/PILLARS (green #408020, toxic fume columns, AXIS_CYL)
// L6: SCATTER/DUST (yellow-green #a0c040, toxic particles)
// L7: ATMOSPHERE/ALIEN (green #203008, poisonous air)
pub(super) const PRESET_TOXIC_WASTELAND: [[u64; 2]; 8] = [
    // L0: RAMP - sickly green sky, toxic floor, corroded walls
    [
        hi(OP_RAMP, REGION_ALL, BLEND_LERP, 0, 0x304010, 0x202008),
        lo(220, 0x28, 0x30, 0x18, THRESH_BALANCED, DIR_UP, 15, 15),
    ],
    // L1: SILHOUETTE/INDUSTRIAL - factory smokestacks (dark olive)
    [
        hi_meta(
            OP_SILHOUETTE,
            REGION_WALLS,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            SILHOUETTE_INDUSTRIAL,
            0x080400,
            0x141008,
        ),
        lo(160, 128, 0, 0, 0, DIR_UP, 15, 15),
    ],
    // L2: PATCHES/ISLANDS - radioactive puddles on floor (toxic green)
    [
        hi_meta(
            OP_PATCHES,
            REGION_FLOOR,
            BLEND_ADD,
            DOMAIN_DIRECT3D,
            PATCHES_ISLANDS,
            0x40a000,
            0x204000,
        ),
        lo(110, 128, 64, 0, 0, DIR_UP, 15, 15),
    ],
    // L3: PLANE/TILES - cracked industrial floor (brown)
    [
        hi_meta(
            OP_PLANE,
            REGION_FLOOR,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            PLANE_TILES,
            0x483820,
            0x302810,
        ),
        lo(140, 128, 0, 0, 0, DIR_UP, 15, 15),
    ],
    // L4: CELL/HEX - hazmat hex pattern on walls (yellow-green)
    [
        hi_meta(
            OP_CELL,
            REGION_WALLS,
            BLEND_SCREEN,
            DOMAIN_DIRECT3D,
            CELL_HEX,
            0xe0e000,
            0x000000,
        ),
        lo(200, 128, 0, 0, 0, DIR_UP, 15, 0),
    ],
    // L5: VEIL/PILLARS - toxic fume columns (green, cylindrical domain)
    [
        hi_meta(
            OP_VEIL,
            REGION_WALLS,
            BLEND_ADD,
            DOMAIN_AXIS_CYL,
            VEIL_PILLARS,
            0x408020,
            0x000000,
        ),
        lo(100, 128, 140, 0, 0, DIR_UP, 15, 0),
    ],
    // L6: SCATTER/DUST - toxic particles (yellow-green)
    [
        hi_meta(
            OP_SCATTER,
            REGION_ALL,
            BLEND_ADD,
            DOMAIN_DIRECT3D,
            SCATTER_DUST,
            0xa0c040,
            0x000000,
        ),
        lo(100, 30, 60, 0x18, 0, DIR_UP, 15, 0),
    ],
    // L7: ATMOSPHERE/ALIEN - poisonous atmosphere (dark green)
    [
        hi_meta(
            OP_ATMOSPHERE,
            REGION_ALL,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            ATMO_ALIEN,
            0x203008,
            0x000000,
        ),
        lo(130, 80, 0, 0, 0, DIR_UP, 15, 0),
    ],
];

// -----------------------------------------------------------------------------
// Preset 16: "Moonlit Graveyard" - Gothic horror
// -----------------------------------------------------------------------------
// L0: RAMP (sky=#0a0a1a, floor=#101010, walls=#181820)
// L1: SILHOUETTE/SPIRES (dark #0a0810 / #141420, gothic tombstones)
// L2: PLANE/STONE (gray #282828 / #1a1a20, weathered path)
// L3: CELESTIAL/MOON (white #e0e8f0, full moon, dir=SUN)
// L4: BAND (blue #202840, eerie horizon glow)
// L5: SCATTER/DUST (blue-gray #8090a0, mist particles)
// L6: VEIL/CURTAINS (gray #404050, hanging mist, AXIS_CYL)
// L7: ATMOSPHERE/FULL (dark blue #101020, heavy night fog)
pub(super) const PRESET_MOONLIT_GRAVEYARD: [[u64; 2]; 8] = [
    // L0: RAMP - midnight blue sky, dark earth floor, slate walls
    [
        hi(OP_RAMP, REGION_ALL, BLEND_LERP, 0, 0x0a0a1a, 0x101010),
        lo(200, 0x18, 0x18, 0x20, THRESH_OPEN, DIR_UP, 15, 15),
    ],
    // L1: SILHOUETTE/SPIRES - gothic tombstone silhouettes (dark blue)
    [
        hi_meta(
            OP_SILHOUETTE,
            REGION_WALLS,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            SILHOUETTE_SPIRES,
            0x0a0810,
            0x141420,
        ),
        lo(160, 128, 0, 0, 0, DIR_UP, 15, 15),
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
    // L3: CELESTIAL/MOON - full moon (pale silver, dir=SUN)
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
        lo(255, 220, 0, 0, 0, DIR_SUN, 15, 0),
    ],
    // L4: BAND - eerie blue horizon glow
    [
        hi(OP_BAND, REGION_SKY, BLEND_ADD, 0, 0x202840, 0x000000),
        lo(110, 128, 0, 0, 0, DIR_SUNSET, 15, 0),
    ],
    // L5: SCATTER/DUST - mist particles (blue-gray)
    [
        hi_meta(
            OP_SCATTER,
            REGION_ALL,
            BLEND_ADD,
            DOMAIN_DIRECT3D,
            SCATTER_DUST,
            0x8090a0,
            0x000000,
        ),
        lo(100, 25, 40, 0x18, 0, DIR_UP, 15, 0),
    ],
    // L6: VEIL/CURTAINS - hanging mist (gray, cylindrical domain)
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
        lo(100, 128, 140, 0, 0, DIR_DOWN, 15, 0),
    ],
    // L7: ATMOSPHERE/FULL - heavy night fog (dark blue)
    [
        hi_meta(
            OP_ATMOSPHERE,
            REGION_ALL,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            ATMO_FULL,
            0x101020,
            0x000000,
        ),
        lo(140, 80, 0, 0, 0, DIR_UP, 15, 0),
    ],
];
