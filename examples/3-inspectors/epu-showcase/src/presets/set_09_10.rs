//! Preset set 09-10

#[allow(unused_imports)]
use crate::constants::*;

// -----------------------------------------------------------------------------
// Preset 9: "Sky Ruins" - Floating colonnades among clouds
// -----------------------------------------------------------------------------
// Goal: outdoor "edge of the world" platforming vibe with dramatic clouds.
// Motion: cloud drift + sun band pulse + subtle scanning floor grid.
//
// L0: RAMP                 ALL        LERP   bright sky / cool shadows
// L1: SILHOUETTE/RUINS      SKY        LERP   broken colonnades
// L2: PLANE/STONE           FLOOR      LERP   weathered marble
// L3: GRID                  FLOOR      ADD    tile lines (animated)
// L4: FLOW (noise)          SKY        SCREEN cloud drift (animated)
// L5: VEIL/CURTAINS         SKY        SCREEN cloud banks
// L6: BAND                  SKY        ADD    warm sun break (animated)
// L7: LOBE                  ALL        ADD    high sun key (animated)
pub(super) const PRESET_SKY_RUINS: [[u64; 2]; 8] = [
    // L0: RAMP - bright sky / cool shadows
    [
        hi(OP_RAMP, REGION_ALL, BLEND_LERP, 0, 0x86c0ff, 0xe8f2ff),
        lo(170, 0x4a, 0x56, 0x62, THRESH_OPEN, DIR_UP, 15, 15),
    ],
    // L1: SILHOUETTE/RUINS - broken skyline architecture
    [
        hi_meta(
            OP_SILHOUETTE,
            REGION_ALL,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            SILHOUETTE_RUINS,
            0x1a1f28,
            0xdfeeff,
        ),
        lo(85, 120, 210, 0x50, 0, DIR_UP, 15, 0),
    ],
    // L2: PLANE/STONE - weathered marble platforms
    [
        hi_meta(
            OP_PLANE,
            REGION_FLOOR,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            PLANE_STONE,
            0xdfe8f0,
            0x4a5662,
        ),
        lo(185, 90, 16, 175, 0, DIR_UP, 15, 0),
    ],
    // L3: GRID - subtle tile lines (animated)
    [
        hi(OP_GRID, REGION_FLOOR, BLEND_ADD, 0, 0xf8fbff, 0x000000),
        lo(25, 120, 6, 0x12, 0, 0, 8, 0),
    ],
    // L4: FLOW/NOISE - cloud drift (animated)
    [
        hi_meta(
            OP_FLOW,
            REGION_SKY,
            BLEND_SCREEN,
            DOMAIN_DIRECT3D,
            0,
            0xffffff,
            0x86c0ff,
        ),
        lo(140, 140, 70, 0x30, 0, DIR_RIGHT, 10, 0),
    ],
    // L5: VEIL/CURTAINS - dramatic cloud banks
    [
        hi_meta(
            OP_VEIL,
            REGION_SKY,
            BLEND_SCREEN,
            DOMAIN_AXIS_CYL,
            VEIL_CURTAINS,
            0xffffff,
            0x2a2f3a,
        ),
        lo(30, 90, 14, 55, 0, DIR_RIGHT, 6, 2),
    ],
    // L6: BAND - warm sun break (animated)
    [
        hi(OP_BAND, REGION_SKY, BLEND_ADD, 0, 0xffd29a, 0x000000),
        lo(130, 60, 120, 200, 0, DIR_SUN, 12, 0),
    ],
    // L7: LOBE - high sun key (animated)
    [
        hi(OP_LOBE, REGION_ALL, BLEND_ADD, 0, 0xfff0c8, 0x2a2f3a),
        lo(170, 190, 95, 1, 0, DIR_SUN, 12, 0),
    ],
];

// -----------------------------------------------------------------------------
// Preset 10: "Combat Lab" - Sterile training facility
// -----------------------------------------------------------------------------
// Goal: harsh fluorescent enclosure + grid floor + holographic UI cards.
// Animation: scanning grid + pulsing HUD + shimmering hologram.
//
// Cadence: BOUNDS (SECTOR) -> BOUNDS (APERTURE) -> FEATURES (floor) -> FEATURES (HUD) -> FEATURES (motion)
//
// L0: SECTOR/BOX           ALL         LERP   sterile room enclosure
// L1: APERTURE/BARS        ALL         LERP   overhead fluorescent banks
// L2: PLANE/TILES          FLOOR       LERP   grid-lined floor
// L3: GRID                 FLOOR       ADD    scanlines (animated)
// L4: DECAL/RECT           WALLS       ADD    HUD panels (animated)
// L5: PORTAL/RECT          WALLS       ADD    hologram volume (animated)
// L6: LOBE                 ALL         ADD    fluorescent key (animated)
// L7: FLOW (noise)         ALL         ADD    subtle data shimmer (animated)
pub(super) const PRESET_COMBAT_LAB: [[u64; 2]; 8] = [
    // L0: SECTOR/BOX - sterile enclosure
    [
        hi_meta(
            OP_SECTOR,
            REGION_ALL,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            SECTOR_BOX,
            0xf8fbff,
            0x222833,
        ),
        lo(240, 150, 0, 0, 0, DIR_UP, 15, 15),
    ],
    // L1: APERTURE/BARS - overhead fluorescent banks
    [
        hi_meta(
            OP_APERTURE,
            REGION_ALL,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            APERTURE_BARS,
            0xffffff,
            0x1a1f28,
        ),
        lo(70, 92, 70, 18, 210, DIR_UP, 0, 0),
    ],
    // L2: PLANE/TILES - grid-lined floor
    [
        hi_meta(
            OP_PLANE,
            REGION_FLOOR,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            PLANE_TILES,
            0x8a919c,
            0x1a1f28,
        ),
        lo(185, 95, 14, 185, 0, DIR_UP, 15, 0),
    ],
    // L3: GRID - scanning lines (animated)
    [
        hi(
            OP_GRID,
            REGION_WALLS | REGION_FLOOR,
            BLEND_ADD,
            0,
            0xddeaff,
            0x000000,
        ),
        lo(70, 150, 10, 0x16, 0, 0, 10, 0),
    ],
    // L4: DECAL/RECT - HUD panels (animated)
    [
        hi(OP_DECAL, REGION_WALLS, BLEND_ADD, 0, 0x00d0ff, 0x46ff9a),
        // shape=RECT(2), soft=4, size, glow; param_d is phase
        lo(200, 0x24, 56, 205, 0x30, DIR_BACK, 14, 10),
    ],
    // L5: PORTAL/RECT - hologram volume (animated)
    [
        hi_meta(
            OP_PORTAL,
            REGION_WALLS,
            BLEND_ADD,
            DOMAIN_TANGENT_LOCAL,
            PORTAL_RECT,
            0x0a1220,
            0x00d0ff,
        ),
        lo(220, 140, 150, 170, 0, DIR_BACK, 12, 12),
    ],
    // L6: LOBE - fluorescent key (animated)
    [
        hi(OP_LOBE, REGION_ALL, BLEND_ADD, 0, 0xf8fbff, 0x2a2f36),
        lo(135, 210, 70, 1, 0, DIR_UP, 12, 0),
    ],
    // L7: FLOW/NOISE - subtle data shimmer (animated)
    [
        hi_meta(
            OP_FLOW,
            REGION_ALL,
            BLEND_ADD,
            DOMAIN_DIRECT3D,
            0,
            0x66f6ff,
            0xff3cff,
        ),
        lo(40, 190, 20, 0x30, 0, DIR_RIGHT, 8, 0),
    ],
];
