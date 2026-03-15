//! Preset set 17-18

#[allow(unused_imports)]
use crate::constants::*;

// -----------------------------------------------------------------------------
// Preset 17: "Toxic Wasteland" - Corrupted industrial ruin
// -----------------------------------------------------------------------------
// Goal: unmistakable ruined toxic industrial exterior with one clear skyline
// owner, a blasted exterior apron, and one toxic breach/plume event that makes
// the scene read as poisoned outdoors instead of vague interior walling.
//
// Cadence: SKY BED -> RUIN OWNER -> GROUND OWNER -> EXTERIOR FRAMING ->
// TOXIC EVENT -> HAZARD DETAIL -> FALLOUT SUPPORT
//
// L0: RAMP                   ALL         LERP      poisoned sky over dark ruined ground
// L1: SILHOUETTE/INDUSTRIAL  SKY|WALLS   LERP      ruined refinery / stack skyline owner
// L2: PLANE/STONE            FLOOR       LERP      blasted concrete apron
// L3: MASS/BANK              WALLS|FLOOR MULTIPLY  collapsed outer works / berm framing
// L4: PORTAL/RIFT            FLOOR       ADD       toxic runoff breach / waste sump
// L5: TRACE/LEAD_LINES       WALLS|FLOOR ADD       exposed hazard piping / conduit runs
// L6: MASS/PLUME             SKY|WALLS   SCREEN    poisonous exhaust cloud tied to the ruin
// L7: SCATTER/DUST           ALL         ADD       sparse toxic fallout
pub(super) const PRESET_TOXIC_WASTELAND: [[u64; 2]; 8] = [
    // L0: RAMP - start from a dirty yellow sky into a near-black yard so the scene reads as open poisoned exterior before any hazard accents.
    [
        hi(OP_RAMP, REGION_ALL, BLEND_LERP, 0, 0x7e8a33, 0x060806),
        lo(228, 0x36, 0x24, 0x14, THRESH_OPEN, DIR_UP, 15, 15),
    ],
    // L1: SILHOUETTE/INDUSTRIAL - let one broken refinery skyline and stack wall own the frame so the scene is unmistakably industrial outdoors.
    [
        hi_meta(
            OP_SILHOUETTE,
            REGION_SKY | REGION_WALLS,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            SILHOUETTE_INDUSTRIAL,
            0x050705,
            0x3f481a,
        ),
        lo(255, 196, 240, 0x48, 0, DIR_UP, 15, 13),
    ],
    // L2: PLANE/STONE - move off grating and onto a blasted concrete apron so the lower frame reads as exterior industrial ground.
    [
        hi_meta(
            OP_PLANE,
            REGION_FLOOR,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            PLANE_STONE,
            0x434733,
            0x0b0d08,
        ),
        lo(236, 112, 34, 170, 0, DIR_UP, 15, 13),
    ],
    // L3: MASS/BANK - frame the apron with collapsed outer works so the exterior gets real rubble shoulders instead of flat toxic walls.
    [
        hi_meta(
            OP_MASS,
            REGION_WALLS | REGION_FLOOR,
            BLEND_MULTIPLY,
            DOMAIN_DIRECT3D,
            MASS_BANK,
            0x3b4120,
            0x0b0e07,
        ),
        lo(172, 88, 194, 86, 0, DIR_LEFT, 11, 0),
    ],
    // L4: PORTAL/RIFT - make one toxic waste breach the clear hazard event so the ruin reads as leaking poisoned industry, not abstract ruin lighting.
    [
        hi_meta(
            OP_PORTAL,
            REGION_FLOOR,
            BLEND_ADD,
            DOMAIN_TANGENT_LOCAL,
            PORTAL_RIFT,
            0xa3db46,
            0x233807,
        ),
        lo(132, 176, 144, 190, 0, DIR_DOWN, 10, 0),
    ],
    // L5: TRACE/LEAD_LINES - exposed conduit runs and broken pipe geometry reinforce the refinery/apron read.
    [
        hi_meta(
            OP_TRACE,
            REGION_WALLS | REGION_FLOOR,
            BLEND_ADD,
            DOMAIN_TANGENT_LOCAL,
            TRACE_LEAD_LINES,
            0xd9ef7a,
            0x88ba31,
        ),
        lo(144, 118, 14, 154, 0x28, DIR_FORWARD, 10, 6),
    ],
    // L6: MASS/PLUME - hang one poisonous exhaust cloud over the skyline so the toxicity is tied to industrial stacks instead of general haze.
    [
        hi_meta(
            OP_MASS,
            REGION_SKY | REGION_WALLS,
            BLEND_SCREEN,
            DOMAIN_DIRECT3D,
            MASS_PLUME,
            0xb8d95a,
            0x435217,
        ),
        lo(124, 96, 170, 92, 0, DIR_LEFT, 8, 2),
    ],
    // L7: SCATTER/DUST - keep fallout sparse and dirty so it supports the poisoned yard without becoming the main event.
    [
        hi_meta(
            OP_SCATTER,
            REGION_ALL,
            BLEND_ADD,
            DOMAIN_DIRECT3D,
            SCATTER_DUST,
            0xb8cf63,
            0x62731f,
        ),
        lo(18, 10, 18, 0x16, 12, DIR_DOWN, 4, 1),
    ],
];

// -----------------------------------------------------------------------------
// Preset 18: "Neon Arcade" - Retro-futurist interior glow chamber
// -----------------------------------------------------------------------------
// Goal: clear entertainment interior with cabinet-row rhythm, glossy floor,
// synthwave color split, and lively scan/pulse motion distinct from the alley scene.
//
// L0: SECTOR/BOX            ALL         LERP      interior arcade bounds
// L1: GRID                  FLOOR       ADD       reflective floor scan lattice (animated)
// L2: PLANE/PAVEMENT        FLOOR       LERP      glossy arcade deck
// L3: CELL/BRICK            WALLS       LERP      clustered cabinet-row blocks
// L4: DECAL/RECT            WALLS       ADD       hero glowing cabinet / marquee planes
// L5: FLOW                  FLOOR       SCREEN    glossy floor reflection drift (animated)
// L6: LOBE                  ALL         ADD       playful neon room glow (animated)
// L7: BAND                  SKY|WALLS   ADD       marquee sweep / entertainment-space scan (animated)
pub(super) const PRESET_NEON_ARCADE: [[u64; 2]; 8] = [
    // L0: SECTOR/BOX - calmer arcade shell so the cabinet wall rhythm can lead the room.
    [
        hi_meta(
            OP_SECTOR,
            REGION_ALL,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            SECTOR_BOX,
            0x1f103d,
            0x07040d,
        ),
        lo(255, 168, 0, 0, 0, DIR_UP, 15, 15),
    ],
    // L1: GRID - keep the floor reflective, but pull the scan lattice back behind the wall rhythm.
    [
        hi(OP_GRID, REGION_FLOOR, BLEND_ADD, 0, 0x97ffff, 0x000000),
        lo(156, 36, 44, 0x18, 0, 0, 12, 0),
    ],
    // L2: PLANE/PAVEMENT - darker glossy deck so the room reads through reflections, not floor brightness.
    [
        hi_meta(
            OP_PLANE,
            REGION_FLOOR,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            PLANE_PAVEMENT,
            0x1d0d30,
            0x05030a,
        ),
        lo(255, 108, 10, 46, 0, DIR_UP, 15, 14),
    ],
    // L3: CELL/BRICK - push the cabinet-row wall rhythm harder so it clearly owns the direct background.
    [
        hi_meta(
            OP_CELL,
            REGION_WALLS,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            CELL_BRICK,
            0xae63e2,
            0x080411,
        ),
        lo(255, 22, 232, 108, 0, DIR_FORWARD, 15, 0),
    ],
    // L4: DECAL/RECT - support the cabinet row with restrained marquee planes instead of broad wall wash.
    [
        hi(OP_DECAL, REGION_WALLS, BLEND_ADD, 0, 0xfff29a, 0xff84cf),
        lo(212, 0x18, 192, 14, 0, 0x80c8, 12, 10),
    ],
    // L5: FLOW - keep a glossy floor reflection drift, but subordinate it to the wall rhythm.
    [
        hi(OP_FLOW, REGION_FLOOR, BLEND_SCREEN, 0, 0x90f8ff, 0xffa767),
        lo(76, 40, 48, 0x1d, 0, DIR_RIGHT, 7, 2),
    ],
    // L6: LOBE - keep a playful room pulse, but avoid letting it overtake the cabinet wall read.
    [
        hi(OP_LOBE, REGION_ALL, BLEND_ADD, 0, 0xffb370, 0x74dfff),
        lo(132, 176, 84, 1, 0, DIR_SUN, 10, 5),
    ],
    // L7: BAND - keep a wall sweep for entertainment energy, but tighten it behind the cabinet row.
    [
        hi(OP_BAND, REGION_WALLS, BLEND_ADD, 0, 0xff8cd9, 0x64c8ff),
        lo(112, 22, 160, 156, 0, DIR_FORWARD, 9, 4),
    ],
];
