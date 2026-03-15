//! Preset set 09-10

#[allow(unused_imports)]
use crate::constants::*;

// -----------------------------------------------------------------------------
// Preset 9: "Sky Ruins" - Open-air terrace among broken colonnades
// -----------------------------------------------------------------------------
// Goal: one unmistakable ruined colonnade silhouette, one readable marble
// terrace floor in the lower frame, and layered cloud depth that keeps the
// frame feeling high and exposed instead of collapsing into a centered orb.
//
// Cadence: SKY BED -> OPEN COURT CUT -> RUIN MASONRY -> COLUMN RHYTHM ->
// FLOOR OWNER -> DISTANT CLOUD SHELF -> CLOUD DRIFT -> COOL AIR LIFT
//
// L0: RAMP                  ALL         LERP    cool sky over darker terrace base
// L1: SECTOR/BOX            ALL         LERP    open court cut so the scene reads as an exterior platform
// L2: CELL/BRICK            WALLS       LERP    broken marble wall / ruin masonry owner
// L3: VEIL/PILLARS          WALLS       SCREEN  sparse colonnade rhythm instead of a solid slab
// L4: PLANE/STONE           FLOOR       LERP    pale marble terrace owner
// L5: MASS/SHELF            SKY         LERP    broad cloud shelf behind the ruin line
// L6: ADVECT/FRONT          SKY         SCREEN  directional cloud-light drift
// L7: BAND                  SKY|WALLS   SCREEN  cool horizon lift to separate ruins from cloud depth
pub(super) const PRESET_SKY_RUINS: [[u64; 2]; 8] = [
    // L0: RAMP - hold the frame in a cool open-sky gradient with a darker terrace base so the composition starts elevated, not sun-washed.
    [
        hi(OP_RAMP, REGION_ALL, BLEND_LERP, 0, 0xf1f5fb, 0x55657c),
        lo(232, 0x28, 0x24, 0x6c, THRESH_OPEN, DIR_UP, 15, 15),
    ],
    // L1: SECTOR/BOX - force one open terrace court so the frame reads as an exterior platform with side structure, not a centered vault.
    [
        hi_meta(
            OP_SECTOR,
            REGION_ALL,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            SECTOR_BOX,
            0xd2dbe9,
            0x76859a,
        ),
        lo(196, 118, 148, 0x34, 0, DIR_FORWARD, 14, 10),
    ],
    // L2: CELL/BRICK - switch from soft silhouette to explicit broken stone courses so the ruin owner reads as architecture.
    [
        hi_meta(
            OP_CELL,
            REGION_WALLS,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            CELL_BRICK,
            0x8d96a9,
            0x1d1a20,
        ),
        lo(176, 20, 212, 28, 0, DIR_UP, 15, 11),
    ],
    // L3: VEIL/PILLARS - add a sparse colonnade rhythm so one broken ruin band reads before any cloud support.
    [
        hi_meta(
            OP_VEIL,
            REGION_WALLS,
            BLEND_SCREEN,
            DOMAIN_AXIS_CYL,
            VEIL_PILLARS,
            0xe3e8f0,
            0x7e8492,
        ),
        lo(132, 18, 26, 20, 0, DIR_UP, 10, 4),
    ],
    // L4: PLANE/STONE - give the lower frame one pale marble terrace owner so the floor plane stays readable under the ruins.
    [
        hi_meta(
            OP_PLANE,
            REGION_FLOOR,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            PLANE_STONE,
            0xf3efe6,
            0x687284,
        ),
        lo(246, 94, 18, 164, 0, DIR_UP, 15, 14),
    ],
    // L5: MASS/SHELF - hold one broad cloud shelf behind the ruin line so the background reads as layered banks, not a blank vault.
    [
        hi_meta(
            OP_MASS,
            REGION_SKY,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            MASS_SHELF,
            0xf5f7fc,
            0x7388a3,
        ),
        lo(188, 104, 146, 66, 0, DIR_LEFT, 14, 9),
    ],
    // L6: ADVECT/FRONT - carry one directional cloud-light drift across the open sky without restoring a centered glow.
    [
        hi_meta(
            OP_ADVECT,
            REGION_SKY,
            BLEND_SCREEN,
            DOMAIN_DIRECT3D,
            ADVECT_FRONT,
            0xe7eef7,
            0x8a9bb0,
        ),
        lo(76, 46, 82, 86, 0, DIR_LEFT, 8, 3),
    ],
    // L7: BAND - use one cool horizon lift to separate ruin silhouette from cloud depth instead of relying on global atmosphere.
    [
        hi(
            OP_BAND,
            REGION_SKY | REGION_WALLS,
            BLEND_SCREEN,
            0,
            0xdfe8f4,
            0x7b8ea8,
        ),
        lo(28, 92, 124, 98, 0, DIR_FORWARD, 5, 1),
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
            OP_RAMP, REGION_ALL, BLEND_LERP, 0,
            0xe6f2f8, // bright white-cyan structural shell
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
