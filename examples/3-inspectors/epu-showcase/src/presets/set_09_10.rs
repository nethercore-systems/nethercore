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
// Goal: harsh fluorescent bounds + grid floor + a world-anchored projection bay.
// Animation: scanning grid + projection-field sweep + luminous room scan motion.
// Visual: a sterile high-tech training facility with harsh fluorescent lighting,
// white structural bounds, and a grid-lined floor. A bright projection field
// and a rectangular rear test-field frame are embedded into the room shell so
// the space stays clean, clinical, and fully in-world.
//
// Cadence: BOUNDS (RAMP) -> FEATURES (floor) -> FEATURES (bay framing) -> FEATURES (projection field) -> FEATURES (motion)
//
// L0: RAMP                 ALL         LERP   cool ceiling / wall / floor enclosure
// L1: PLANE/TILES          FLOOR       LERP   sterile floor tiles with darker grout
// L2: GRID                 FLOOR       ADD    readable cyan floor grid scan (animated)
// L3: GRID                 WALLS       ADD    broad wall-bay scan lattice (animated)
// L4: DECAL/RECT           WALLS       ADD    hero animated projection field in direct view
// L5: PORTAL/RECT          WALLS       ADD    static rear test-field frame behind the projection field
// L6: LOBE                 ALL         ADD    overhead fluorescent pulse
// L7: VEIL/RAIN_WALL       WALLS       ADD    broader scanner sweep bars
pub(super) const PRESET_COMBAT_LAB: [[u64; 2]; 8] = [
    // L0: RAMP - brighter white shell so the room reads as a clean training bay rather than a gray chamber
    [
        hi(
            OP_RAMP, REGION_ALL, BLEND_LERP, 0, 0xe6f2f8, // bright white-cyan structural shell
            0x091118, // darker floor base under the tile layer
        ),
        lo(86, 0x2f, 0x38, 0x42, THRESH_INTERIOR, DIR_UP, 15, 15),
    ],
    // L1: PLANE/TILES - darker training-room deck so the cyan grid stays foreground-dominant
    [
        hi_meta(
            OP_PLANE,
            REGION_FLOOR,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            PLANE_TILES,
            0x25333c, // darker graphite tile surface
            0x091118, // darker grout for structure
        ),
        lo(242, 96, 28, 20, 0, DIR_UP, 15, 12),
    ],
    // L2: GRID - brighter floor grid with larger cells so it reads before the sphere reflection
    [
        hi(
            OP_GRID,
            REGION_FLOOR,
            BLEND_ADD,
            0,
            0x9cffff, // vivid cyan highlight
            0x000000,
        ),
        lo(220, 42, 44, 0x1a, 0, 0, 15, 0),
    ],
    // L3: GRID - restore the broader run25 wall bay so the rear chamber feels embedded in architecture again
    [
        hi(OP_GRID, REGION_WALLS, BLEND_ADD, 0, 0xd6f6ff, 0x000000),
        lo(104, 18, 28, 0x15, 0, 0, 9, 0),
    ],
    // L4: DECAL/RECT - make the projection field itself the animated hero plane anchored to the rear bay
    [
        hi(OP_DECAL, REGION_WALLS, BLEND_ADD, 0, 0xf8ffff, 0x62ecff),
        // param_a=0x24 => RECT shape with light edge softening.
        lo(240, 0x24, 184, 40, 0, 0x80c8, 15, 11),
    ],
    // L5: PORTAL/RECT - keep a static rectangular test-field frame behind the animated projection field
    [
        hi_meta(
            OP_PORTAL,
            REGION_WALLS,
            BLEND_ADD,
            DOMAIN_TANGENT_LOCAL,
            PORTAL_RECT,
            0x0d2430, // darker structural backplate
            0xd7fbff, // cool cyan frame edge
        ),
        lo(176, 198, 56, 0, 0, 0x80c8, 8, 13),
    ],
    // L6: LOBE - keep the ceiling pulse restrained so it stops washing out the wall devices
    [
        hi(OP_LOBE, REGION_ALL, BLEND_ADD, 0, 0xf2fbff, 0x28495f),
        lo(15, 176, 42, 2, 0, DIR_UP, 6, 0),
    ],
    // L7: VEIL/RAIN_WALL - fewer, broader scanner slabs so the wall scan reads as authored tech instead of rain
    [
        hi_meta(
            OP_VEIL,
            REGION_WALLS,
            BLEND_ADD,
            DOMAIN_AXIS_CYL,
            VEIL_RAIN_WALL,
            0xf4ffff, // scanner core
            0x62ecff, // scanner glow
        ),
        lo(96, 28, 184, 36, 0, DIR_UP, 11, 7),
    ],
];
