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
    // L0: RAMP - readable forest base (not pitch black)
    [
        hi(OP_RAMP, REGION_ALL, BLEND_LERP, 0, 0x304020, 0x182818),
        lo(210, 0x20, 0x40, 0x28, THRESH_OPEN, DIR_UP, 15, 15),
    ],
    // L1: SILHOUETTE/FOREST - strong treeline shapes
    [
        hi_meta(
            OP_SILHOUETTE,
            REGION_WALLS,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            SILHOUETTE_FOREST,
            0x000000,
            0x183020,
        ),
        lo(40, 200, 220, 0x40, 0, DIR_UP, 15, 0),
    ],
    // L2: PLANE/GRASS - lush ground
    [
        hi_meta(
            OP_PLANE,
            REGION_FLOOR,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            PLANE_GRASS,
            0x1a3a1a,
            0x102010,
        ),
        lo(160, 128, 0, 0, 0, DIR_UP, 15, 15),
    ],
    // L3: VEIL/SHARDS - volumetric god rays (many thin rays; avoid searchlight wedges)
    [
        hi_meta(
            OP_VEIL,
            REGION_SKY | REGION_WALLS,
            BLEND_ADD,
            DOMAIN_AXIS_CYL,
            VEIL_SHARDS,
            0xffd080,
            0xfff0c0,
        ),
        lo(45, 200, 30, 120, 0, DIR_UP, 12, 8),
    ],
    // L4: SCATTER/DUST - fairy motes (tiny + sparse)
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
        lo(8, 6, 6, 0x40, 17, DIR_UP, 8, 0),
    ],
    // L5: FLOW/CAUSTIC - dappled light through canopy
    [
        hi(
            OP_FLOW,
            REGION_SKY | REGION_WALLS,
            BLEND_SCREEN,
            0,
            0xffd080,
            0x204020,
        ),
        lo(120, 120, 100, 0x22, 0, DIR_SUN, 10, 0),
    ],
    // L6: LOBE - primary sunbeam (hero)
    [
        hi(OP_LOBE, REGION_ALL, BLEND_ADD, 0, 0xffd080, 0x402010),
        lo(170, 220, 80, 1, 0, DIR_SUN, 12, 0),
    ],
    // L7: ATMOSPHERE/MIE - warm volume around sun direction
    [
        hi_meta(
            OP_ATMOSPHERE,
            REGION_ALL,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            ATMO_MIE,
            0xffd080,
            0x102010,
        ),
        lo(60, 80, 128, 120, 200, DIR_SUN, 10, 0),
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
// Goal: Clearly "toxic/radioactive" with visible green glow as hero element.
//
// L0: RAMP - murky olive/brown base (brighter for readability)
// L1: SILHOUETTE/INDUSTRIAL - factory smokestacks
// L2: PLANE/TILES - cracked industrial floor
// L3: CELL/HEX - hazmat hex pattern on walls
// L4: FLOW - toxic puddle glow on floor (HERO - radioactive pools)
// L5: VEIL/PILLARS - toxic fume columns
// L6: LOBE - radioactive floor glow (green, upward)
// L7: ATMOSPHERE/ALIEN - poisonous green haze
pub(super) const PRESET_TOXIC_WASTELAND: [[u64; 2]; 8] = [
    // L0: RAMP - murky olive sky, brown floor, corroded walls (BRIGHTER base)
    [
        hi(OP_RAMP, REGION_ALL, BLEND_LERP, 0, 0x181808, 0x201810),
        lo(200, 0x28, 0x30, 0x18, THRESH_BALANCED, DIR_UP, 15, 15),
    ],
    // L1: SILHOUETTE/INDUSTRIAL - factory smokestacks (more visible)
    [
        hi_meta(
            OP_SILHOUETTE,
            REGION_WALLS,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            SILHOUETTE_INDUSTRIAL,
            0x080400,
            0x181008,
        ),
        lo(180, 160, 200, 0x30, 0, DIR_UP, 12, 10),
    ],
    // L2: PLANE/TILES - cracked industrial floor (more visible brown)
    [
        hi_meta(
            OP_PLANE,
            REGION_FLOOR,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            PLANE_TILES,
            0x302010,
            0x201408,
        ),
        lo(160, 128, 0, 0, 0, DIR_UP, 15, 15),
    ],
    // L3: CELL/HEX - hazmat hex outlines on walls (visible green)
    [
        hi_meta(
            OP_CELL,
            REGION_WALLS,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            CELL_HEX,
            0x182008,
            0x101808,
        ),
        lo(180, 100, 200, 50, 0, DIR_UP, 10, 8),
    ],
    // L4: FLOW - HERO: toxic radioactive puddle glow on floor (bright green)
    [
        hi(OP_FLOW, REGION_FLOOR, BLEND_ADD, 0, 0x40ff20, 0x208010),
        lo(100, 80, 60, 0x22, 0, DIR_UP, 15, 0),
    ],
    // L5: VEIL/PILLARS - toxic fume columns (visible green glow)
    [
        hi_meta(
            OP_VEIL,
            REGION_WALLS | REGION_SKY,
            BLEND_ADD,
            DOMAIN_AXIS_CYL,
            VEIL_PILLARS,
            0x306010,
            0x000000,
        ),
        lo(60, 80, 100, 0, 0, DIR_UP, 12, 0),
    ],
    // L6: LOBE - radioactive floor glow (green, upward, stronger)
    [
        hi(OP_LOBE, REGION_ALL, BLEND_ADD, 0, 0x308010, 0x000000),
        lo(80, 180, 60, 1, 0, DIR_UP, 12, 0),
    ],
    // L7: ATMOSPHERE/ALIEN - green poisonous haze (more visible)
    [
        hi_meta(
            OP_ATMOSPHERE,
            REGION_ALL,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            ATMO_ALIEN,
            0x182008,
            0x080804,
        ),
        lo(40, 100, 128, 0, 0, DIR_UP, 10, 0),
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
        // softness, height_bias, roughness, octaves
        lo(40, 180, 220, 0x40, 0, DIR_UP, 15, 0),
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
    // L5: SCATTER/DUST - mist particles (reduced; avoid full-screen bokeh)
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
        lo(20, 8, 12, 0x10, 7, DIR_UP, 8, 0),
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
    // L7: ATMOSPHERE/FULL - heavy night fog (avoid horizon_y=-1 flat wash)
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
        lo(100, 80, 128, 0, 0, DIR_UP, 12, 0),
    ],
];
