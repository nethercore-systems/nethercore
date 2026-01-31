//! Preset set 09-10

#[allow(unused_imports)]
use crate::constants::*;

// -----------------------------------------------------------------------------
// Preset 9: "Sky Ruins" - Floating colonnades among clouds
// -----------------------------------------------------------------------------
// Goal: outdoor "edge of the world" platforming vibe with dramatic clouds.
// Motion: cloud drift + sun band pulse + subtle scanning floor grid.
// Visual: crumbling stone platforms and shattered colonnades suspended among
// clouds, with warm sunlight breaking through dramatic cloud banks. The floor
// reads as weathered marble, the skyline reads as ruins silhouettes, and the sky
// layers drift to make the whole scene feel alive and windy.
//
// L0: RAMP                  ALL        LERP   blazing sunset gradient (orange to violet)
// L1: SILHOUETTE/RUINS      SKY        LERP   broken colonnades against blazing sky
// L2: PLANE/STONE           FLOOR      LERP   warm cream marble platforms
// L3: GRID                  FLOOR      ADD    subtle marble tile lines
// L4: VEIL/CURTAINS         SKY        SCREEN billowing golden clouds
// L5: FLOW (noise)          SKY        SCREEN warm cloud drift (animated)
// L6: BAND                  SKY        ADD    intense sun break band (animated)
// L7: LOBE                  ALL        ADD    blazing golden sun key (animated)
pub(super) const PRESET_SKY_RUINS: [[u64; 2]; 8] = [
    // L0: RAMP - epic sunset sky gradient (blazing orange to warm violet)
    [
        hi(OP_RAMP, REGION_ALL, BLEND_LERP, 0, 0xffa040, 0x6040a0),
        // Blazing orange sunset at horizon, warm violet depths
        lo(255, 0x60, 0x40, 0xa0, THRESH_OPEN, DIR_UP, 15, 15),
    ],
    // L1: SILHOUETTE/RUINS - broken colonnades against blazing sky
    [
        hi_meta(
            OP_SILHOUETTE,
            REGION_SKY,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            SILHOUETTE_RUINS,
            0x201820, // dark ruins silhouettes
            0xffb060, // blazing sky behind
        ),
        lo(255, 110, 200, 0x60, 0, DIR_UP, 15, 14),
    ],
    // L2: PLANE/STONE - weathered cream marble platforms
    [
        hi_meta(
            OP_PLANE,
            REGION_FLOOR,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            PLANE_STONE,
            0xf8f0e0, // warm cream marble
            0xc0a080, // golden shadow
        ),
        lo(255, 60, 25, 140, 0, DIR_UP, 15, 12),
    ],
    // L3: GRID - marble tile lines (subtle warm)
    [
        hi(OP_GRID, REGION_FLOOR, BLEND_ADD, 0, 0xffe8d0, 0x000000),
        lo(40, 90, 3, 0x10, 0, 0, 8, 0),
    ],
    // L4: VEIL/CURTAINS - billowing golden clouds
    [
        hi_meta(
            OP_VEIL,
            REGION_SKY,
            BLEND_SCREEN,
            DOMAIN_AXIS_CYL,
            VEIL_CURTAINS,
            0xffd080, // golden cloud highlights
            0xff8030, // deep orange
        ),
        lo(255, 70, 40, 45, 0, DIR_RIGHT, 15, 13),
    ],
    // L5: FLOW/NOISE - warm cloud drift (animated)
    [
        hi_meta(
            OP_FLOW,
            REGION_SKY,
            BLEND_SCREEN,
            DOMAIN_DIRECT3D,
            0,
            0xffc060, // golden drift
            0xff6020, // orange accent
        ),
        lo(200, 100, 55, 0x20, 0, DIR_RIGHT, 14, 10),
    ],
    // L6: BAND - intense sun break band across horizon
    [
        hi(OP_BAND, REGION_SKY, BLEND_ADD, 0, 0xffe080, 0xffa040),
        lo(255, 55, 160, 220, 0, DIR_SUN, 15, 13),
    ],
    // L7: LOBE - blazing golden sun key (animated)
    [
        hi(OP_LOBE, REGION_ALL, BLEND_ADD, 0, 0xffc050, 0x906020),
        lo(255, 200, 100, 1, 0, DIR_SUN, 15, 8),
    ],
];

// -----------------------------------------------------------------------------
// Preset 10: "Combat Lab" - Sterile training facility
// -----------------------------------------------------------------------------
// Goal: harsh fluorescent bounds + grid floor + holographic UI cards.
// Animation: scanning grid + pulsing HUD + shimmering hologram.
// Visual: a sterile high-tech training facility with harsh fluorescent lighting,
// glassy walls, and a grid-lined floor. Holographic panels and a rectangular
// hologram volume flicker with combat data while the room stays clean and clinical.
//
// Cadence: BOUNDS (SECTOR) -> FEATURES (floor) -> FEATURES (grids) -> FEATURES (HUD/holo) -> FEATURES (motion)
//
// L0: SECTOR/BOX           ALL         LERP   bright clinical white bounds
// L1: PLANE/TILES          FLOOR       LERP   clean white tile floor
// L2: GRID                 FLOOR       ADD    vivid cyan scanning grid (animated)
// L3: GRID                 WALLS       ADD    cyan wall scan lines (animated)
// L4: DECAL/RECT           WALLS       ADD    glowing HUD panels (animated)
// L5: PORTAL/RECT          WALLS       ADD    holographic display volume (animated)
// L6: LOBE                 ALL         ADD    harsh fluorescent overhead key
// L7: VEIL/LASER_BARS      ALL         ADD    holographic scan bars (animated)
pub(super) const PRESET_COMBAT_LAB: [[u64; 2]; 8] = [
    // L0: SECTOR/BOX - bright clinical white bounds
    [
        hi_meta(
            OP_SECTOR,
            REGION_ALL,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            SECTOR_BOX,
            0xffffff, // pure white
            0xf0f4f8, // barely tinted shadow
        ),
        lo(255, 140, 0, 0, 0, DIR_UP, 15, 15),
    ],
    // L1: PLANE/TILES - clinical floor with clean tile pattern
    [
        hi_meta(
            OP_PLANE,
            REGION_FLOOR,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            PLANE_TILES,
            0xf8fafc, // near-white floor
            0xd8e0e8, // light gray grout
        ),
        lo(255, 80, 10, 160, 0, DIR_UP, 15, 10),
    ],
    // L2: GRID - vivid cyan scanning grid on floor (animated)
    [
        hi(
            OP_GRID,
            REGION_FLOOR,
            BLEND_ADD,
            0,
            0x00ffff, // vivid cyan
            0x000000,
        ),
        lo(180, 120, 6, 0x18, 0, 0, 15, 0),
    ],
    // L3: GRID - cyan wall scan lines (animated)
    [
        hi(
            OP_GRID,
            REGION_WALLS,
            BLEND_ADD,
            0,
            0x00e0ff, // bright cyan
            0x000000,
        ),
        lo(120, 200, 4, 0x20, 0, 0, 14, 0),
    ],
    // L4: DECAL/RECT - glowing HUD panels on walls (bright cyan/green)
    [
        hi(OP_DECAL, REGION_WALLS, BLEND_ADD, 0, 0x00ffff, 0x40ffa0),
        // shape=RECT, glow params
        lo(255, 0x24, 60, 200, 0x30, DIR_BACK, 15, 15),
    ],
    // L5: PORTAL/RECT - holographic display volume (vivid)
    [
        hi_meta(
            OP_PORTAL,
            REGION_WALLS,
            BLEND_ADD,
            DOMAIN_TANGENT_LOCAL,
            PORTAL_RECT,
            0x60e0ff, // bright hologram blue
            0x00ffff, // cyan edge
        ),
        lo(255, 100, 120, 140, 0, DIR_FORWARD, 15, 15),
    ],
    // L6: LOBE - harsh fluorescent overhead key
    [
        hi(OP_LOBE, REGION_ALL, BLEND_ADD, 0, 0xffffff, 0xe8f4ff),
        lo(180, 200, 70, 1, 0, DIR_UP, 15, 10),
    ],
    // L7: VEIL/LASER_BARS - holographic scan bars (animated)
    [
        hi_meta(
            OP_VEIL,
            REGION_ALL,
            BLEND_ADD,
            DOMAIN_AXIS_CYL,
            VEIL_LASER_BARS,
            0x40f0ff, // cyan laser
            0x00c0ff, // blue edge
        ),
        lo(80, 60, 20, 30, 0, DIR_UP, 12, 8),
    ],
];
