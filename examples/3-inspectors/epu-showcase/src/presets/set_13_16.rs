//! Preset set 13-16

#[allow(unused_imports)]
use crate::constants::*;

// -----------------------------------------------------------------------------
// Preset 13: "Enchanted Grove" - Fairy tale forest
// -----------------------------------------------------------------------------
// L0: RAMP (sky=#fff8d0, floor=#204020, walls=#1a3820)
// L1: SILHOUETTE/FOREST (deep green #0a2010)
// L2: PLANE/GRASS (vibrant green #308030, forest floor)
// L3: VEIL/CURTAINS (green #40a040, hanging moss)
// L4: SCATTER (gold #ffdd00, fairy dust)
// L5: PATCHES/BLOBS (soft yellow #fff080, dappled sunlight)
// L6: LOBE (golden #ffd700, sunbeam through canopy)
// L7: FLOW (green #60a060, gentle leaf movement)
pub(super) const PRESET_ENCHANTED_GROVE: [[u64; 2]; 8] = [
    // L0: RAMP - golden sky, mossy floor, forest green walls
    [
        hi(OP_RAMP, REGION_ALL, BLEND_LERP, 0, 0xfff8d0, 0x204020),
        lo(255, 0x1a, 0x38, 0x20, 0, DIR_UP, 15, 15),
    ],
    // L1: SILHOUETTE/MOUNTAINS - distant mountain backdrop (deep green)
    [
        hi_meta(
            OP_SILHOUETTE,
            REGION_WALLS,
            BLEND_MULTIPLY,
            DOMAIN_DIRECT3D,
            SILHOUETTE_MOUNTAINS,
            0x0a2010,
            0x000000,
        ),
        lo(180, 128, 0, 0, 0, DIR_UP, 15, 0),
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
            0x000000,
        ),
        lo(160, 128, 0, 0, 0, DIR_UP, 15, 0),
    ],
    // L3: VEIL/CURTAINS - hanging moss/vines (green)
    [
        hi_meta(
            OP_VEIL,
            REGION_ALL,
            BLEND_ADD,
            DOMAIN_AXIS_CYL,
            VEIL_CURTAINS,
            0x40a040,
            0x000000,
        ),
        lo(140, 128, 64, 0, 0, DIR_DOWN, 15, 0),
    ],
    // L4: SCATTER - fairy dust particles (gold)
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
        lo(170, 100, 60, 0x30, 0, DIR_UP, 15, 0),
    ],
    // L5: PATCHES/BLOBS - dappled sunlight (soft yellow)
    [
        hi_meta(
            OP_PATCHES,
            REGION_ALL,
            BLEND_ADD,
            DOMAIN_DIRECT3D,
            PATCHES_BLOBS,
            0xfff080,
            0x000000,
        ),
        lo(140, 128, 48, 0, 0, DIR_UP, 15, 0),
    ],
    // L6: LOBE - warm sunbeam through canopy (golden)
    [
        hi(OP_LOBE, REGION_ALL, BLEND_ADD, 0, 0xffd700, 0x000000),
        lo(180, 128, 0, 0, 2, DIR_SUN, 15, 0), // param_d=2: beam sway
    ],
    // L7: FLOW - gentle leaf movement (green)
    [
        hi(OP_FLOW, REGION_ALL, BLEND_ADD, 0, 0x60a060, 0x000000),
        lo(100, 128, 60, 0, 0, DIR_UP, 15, 0),
    ],
];

// -----------------------------------------------------------------------------
// Preset 14: "Astral Void" - Cosmic/abstract
// -----------------------------------------------------------------------------
// L0: RAMP (sky=#000004, floor=#080010, walls=#100020)
// L1: FLOW (nebula purple #4000a0, swirling gases)
// L2: SCATTER (white #ffffff, dense starfield)
// L3: CELESTIAL/PLANET (blue-green #4080a0, terrestrial planet)
// L4: CELESTIAL/RINGED (pale gold #d0c080)
// L5: PORTAL/VORTEX (blue #0080ff, cosmic vortex)
// L6: TRACE/FILAMENTS (white #ffffff, energy streams)
// L7: ATMOSPHERE/ALIEN (purple #200040)
pub(super) const PRESET_ASTRAL_VOID: [[u64; 2]; 8] = [
    // L0: RAMP - void black sky, deep purple floor, indigo walls
    [
        hi(OP_RAMP, REGION_ALL, BLEND_LERP, 0, 0x000004, 0x080010),
        lo(255, 0x10, 0x00, 0x20, 0, DIR_UP, 15, 15),
    ],
    // L1: FLOW - swirling cosmic gases (nebula purple)
    [
        hi(OP_FLOW, REGION_ALL, BLEND_ADD, 0, 0x4000a0, 0x000000),
        lo(140, 128, 80, 4, 0, DIR_UP, 15, 0),
    ],
    // L2: SCATTER - dense starfield (white)
    [
        hi(OP_SCATTER, REGION_ALL, BLEND_ADD, 0, 0xffffff, 0x000000),
        lo(200, 200, 0, 0x10, 0, DIR_UP, 15, 0),
    ],
    // L3: CELESTIAL/PLANET - terrestrial planet (blue-green)
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
        lo(180, 100, 0, 0, 0, DIR_SUN, 15, 0),
    ],
    // L4: CELESTIAL/RINGED - ringed planet in distance (pale gold)
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
        lo(160, 80, 0, 0, 0, DIR_SUNSET, 15, 0),
    ],
    // L5: PORTAL/VORTEX - cosmic swirling vortex (blue)
    [
        hi_meta(
            OP_PORTAL,
            REGION_WALLS,
            BLEND_ADD,
            DOMAIN_TANGENT_LOCAL,
            PORTAL_TEAR,
            0x0080ff,
            0x000000,
        ),
        lo(180, 128, 64, 0, 0, DIR_UP, 15, 0),
    ],
    // L6: PORTAL/VORTEX - cosmic swirling vortex (white energy)
    [
        hi_meta(
            OP_PORTAL,
            REGION_ALL,
            BLEND_ADD,
            DOMAIN_TANGENT_LOCAL,
            PORTAL_VORTEX,
            0xffffff,
            0x8040ff,
        ),
        lo(160, 128, 64, 0, 0, DIR_UP, 15, 15),
    ],
    // L7: NOP - void boundary (intentionally empty)
    NOP_LAYER,
];

// -----------------------------------------------------------------------------
// Preset 15: "Toxic Wasteland" - Post-apocalyptic industrial
// -----------------------------------------------------------------------------
// L0: RAMP (sky=#304010, floor=#202008, walls=#283018)
// L1: SILHOUETTE/INDUSTRIAL (black #000000)
// L2: PLANE/TILES (rust #483820)
// L3: PATCHES/STATIC (green #40a000, radioactive)
// L4: CELL/HEX (toxic yellow #a0a000, hazmat)
// L5: VEIL/PILLARS (green smoke #408020, toxic fumes)
// L6: SCATTER (yellow-green #a0c040, toxic particles)
// L7: ATMOSPHERE/ALIEN (toxic green #203008)
pub(super) const PRESET_TOXIC_WASTELAND: [[u64; 2]; 8] = [
    // L0: RAMP - sickly green sky, toxic floor, corroded walls
    [
        hi(OP_RAMP, REGION_ALL, BLEND_LERP, 0, 0x304010, 0x202008),
        lo(255, 0x28, 0x30, 0x18, 0, DIR_UP, 15, 15),
    ],
    // L1: SILHOUETTE/INDUSTRIAL - rusted factory smokestacks (black)
    [
        hi_meta(
            OP_SILHOUETTE,
            REGION_WALLS,
            BLEND_MULTIPLY,
            DOMAIN_DIRECT3D,
            SILHOUETTE_INDUSTRIAL,
            0x000000,
            0x000000,
        ),
        lo(180, 128, 0, 0, 0, DIR_UP, 15, 0),
    ],
    // L2: PLANE/TILES - cracked industrial floor tiles (rust)
    [
        hi_meta(
            OP_PLANE,
            REGION_FLOOR,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            PLANE_TILES,
            0x483820,
            0x000000,
        ),
        lo(160, 128, 0, 0, 0, DIR_UP, 15, 0),
    ],
    // L3: PATCHES/STATIC - radioactive patches (green)
    [
        hi_meta(
            OP_PATCHES,
            REGION_ALL,
            BLEND_ADD,
            DOMAIN_DIRECT3D,
            PATCHES_STATIC,
            0x40a000,
            0x000000,
        ),
        lo(150, 128, 64, 0, 0, DIR_UP, 15, 0),
    ],
    // L4: CELL/HEX - hazmat pattern (toxic yellow)
    [
        hi_meta(
            OP_CELL,
            REGION_ALL,
            BLEND_ADD,
            DOMAIN_DIRECT3D,
            CELL_HEX,
            0xa0a000,
            0x000000,
        ),
        lo(140, 128, 0, 0, 0, DIR_UP, 15, 0),
    ],
    // L5: VEIL/PILLARS - rising toxic fumes (green smoke)
    [
        hi_meta(
            OP_VEIL,
            REGION_ALL,
            BLEND_ADD,
            DOMAIN_AXIS_CYL,
            VEIL_PILLARS,
            0x408020,
            0x000000,
        ),
        lo(140, 128, 80, 0, 0, DIR_UP, 15, 0),
    ],
    // L6: SCATTER - toxic particles (yellow-green)
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
        lo(160, 100, 60, 0x20, 0, DIR_UP, 15, 0),
    ],
    // L7: ATMOSPHERE/ALIEN - poisonous atmosphere (toxic green)
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
        lo(140, 160, 0, 0, 0, DIR_UP, 15, 0),
    ],
];

// -----------------------------------------------------------------------------
// Preset 16: "Moonlit Graveyard" - Gothic horror
// -----------------------------------------------------------------------------
// L0: RAMP (sky=#0a0a1a, floor=#101010, walls=#181820)
// L1: SILHOUETTE/SPIRES (black #000000, gothic spires)
// L2: PLANE/STONE (gray #282828, weathered path)
// L3: PATCHES/MEMBRANE (dark green #0a1a0a, creeping moss)
// L4: SCATTER (pale blue #8090a0, mist particles)
// L5: CELESTIAL/MOON (pale silver #e0e8f0)
// L6: VEIL/CURTAINS (gray #404050, hanging mist)
// L7: PORTAL/CRACK (ghostly cracks #404050)
pub(super) const PRESET_MOONLIT_GRAVEYARD: [[u64; 2]; 8] = [
    // L0: RAMP - midnight blue sky, dark earth floor, slate walls
    [
        hi(OP_RAMP, REGION_ALL, BLEND_LERP, 0, 0x0a0a1a, 0x101010),
        lo(255, 0x18, 0x18, 0x20, 0, DIR_UP, 15, 15),
    ],
    // L1: SILHOUETTE/SPIRES - gothic tombstone spires (black)
    [
        hi_meta(
            OP_SILHOUETTE,
            REGION_WALLS,
            BLEND_MULTIPLY,
            DOMAIN_DIRECT3D,
            SILHOUETTE_SPIRES,
            0x000000,
            0x000000,
        ),
        lo(180, 128, 0, 0, 0, DIR_UP, 15, 0),
    ],
    // L2: PLANE/STONE - weathered stone path (gray)
    [
        hi_meta(
            OP_PLANE,
            REGION_FLOOR,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            PLANE_STONE,
            0x282828,
            0x000000,
        ),
        lo(150, 128, 0, 0, 0, DIR_UP, 15, 0),
    ],
    // L3: PATCHES/MEMBRANE - creeping moss (dark green)
    [
        hi_meta(
            OP_PATCHES,
            REGION_ALL,
            BLEND_MULTIPLY,
            DOMAIN_DIRECT3D,
            PATCHES_MEMBRANE,
            0x0a1a0a,
            0x000000,
        ),
        lo(140, 128, 48, 0, 0, DIR_UP, 15, 0),
    ],
    // L4: SCATTER - floating mist particles (pale blue)
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
        lo(160, 100, 60, 0x20, 0, DIR_UP, 15, 0),
    ],
    // L5: CELESTIAL/MOON - full moon (pale silver)
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
        lo(200, 128, 0, 0, 0, DIR_SUN, 15, 0),
    ],
    // L6: VEIL/CURTAINS - hanging mist (gray)
    [
        hi_meta(
            OP_VEIL,
            REGION_ALL,
            BLEND_ADD,
            DOMAIN_AXIS_CYL,
            VEIL_CURTAINS,
            0x404050,
            0x000000,
        ),
        lo(140, 128, 80, 0, 0, DIR_DOWN, 15, 0),
    ],
    // L7: PORTAL/CRACK - ghostly dimensional cracks (gray)
    [
        hi_meta(
            OP_PORTAL,
            REGION_ALL,
            BLEND_ADD,
            DOMAIN_TANGENT_LOCAL,
            PORTAL_CRACK,
            0x404050,
            0x000000,
        ),
        lo(140, 180, 0, 0, 0, DIR_UP, 15, 0),
    ],
];

