//! Preset set 13-14

#[allow(unused_imports)]
use crate::constants::*;

// -----------------------------------------------------------------------------
// Preset 13: "Crystal Cavern" - Enclosed basin of owned crystal growth
// -----------------------------------------------------------------------------
// Goal: unmistakable cave enclosure with a dark basin and ceiling lip first,
// then crystal planes claiming the surviving walls/floor, plus narrow mineral
// support layers that read as crystal structure instead of fog or generic glow.
//
// Cadence: SHELL (ceiling + cave body + basin organizer) -> CRYSTAL OWNER
// (facets + veins) -> STONE SUPPORT (rough wall backing) -> LIGHT (one cold
// reflected pulse)
//
// L0: RAMP                 ALL         LERP      cold chamber ceiling into deep basin floor
// L1: SECTOR/CAVE          ALL         LERP      enclosing cavern shell / forward recess
// L2: SPLIT/TIER           ALL         MULTIPLY  ceiling lip and lower-basin organizer
// L3: MASS/BANK            WALLS|FLOOR MULTIPLY stone shoulders that keep the basin enclosed
// L4: CELL/SHATTER         WALLS|FLOOR LERP      scene-owning faceted crystal faces
// L5: TRACE/CRACKS         WALLS|FLOOR ADD       localized mineral seams reinforcing crystal identity
// L6: MOTTLE/RIDGE         WALLS       MULTIPLY  rough cave backing behind the crystal planes
// L7: LOBE                 WALLS|FLOOR ADD       restrained cold bounce pulse tied to the crystal basin
pub(super) const PRESET_CRYSTAL_CAVERN: [[u64; 2]; 8] = [
    // L0: RAMP - start with a cold roof and a much darker floor so the frame reads as an interior basin before any crystal structure shows up.
    [
        hi(OP_RAMP, REGION_ALL, BLEND_LERP, 0, 0x6f87a0, 0x010205),
        lo(212, 0x30, 0x24, 0x5c, THRESH_INTERIOR, DIR_UP, 15, 15),
    ],
    // L1: SECTOR/CAVE - push the walls and ceiling inward hard enough that the view reads as a cave pocket, not an open shard hall.
    [
        hi_meta(
            OP_SECTOR,
            REGION_ALL,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            SECTOR_CAVE,
            0x3f5062,
            0x02050a,
        ),
        lo(255, 176, 228, 0x48, 0, DIR_FORWARD, 15, 13),
    ],
    // L2: SPLIT/TIER - carve a darker lower basin and a readable upper lip so the chamber has a floor pocket and ceiling closure.
    [
        hi_meta(
            OP_SPLIT,
            REGION_ALL,
            BLEND_MULTIPLY,
            DOMAIN_DIRECT3D,
            SPLIT_TIER,
            0x11202c,
            0x010203,
        ),
        lo(188, 116, 18, 146, 0, DIR_UP, 11, 0),
    ],
    // L3: MASS/BANK - add one heavy stone body around the side walls and basin floor so crystal planes sit inside carved rock instead of repainting the room.
    [
        hi_meta(
            OP_MASS,
            REGION_WALLS | REGION_FLOOR,
            BLEND_MULTIPLY,
            DOMAIN_DIRECT3D,
            MASS_BANK,
            0x284150,
            0x03070c,
        ),
        lo(176, 92, 204, 84, 0, DIR_LEFT, 10, 0),
    ],
    // L4: CELL/SHATTER - let brighter crystal planes own the walls and basin edges, but keep the recess color deep enough that the cave shell still wins.
    [
        hi_meta(
            OP_CELL,
            REGION_WALLS | REGION_FLOOR,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            CELL_SHATTER,
            0xd7f6ff,
            0x0b1320,
        ),
        lo(216, 20, 226, 0x34, 0, DIR_FORWARD, 14, 4),
    ],
    // L5: TRACE/CRACKS - use narrow luminous seams as the crystal support layer so identity comes from mineral structure, not broad haze.
    [
        hi_meta(
            OP_TRACE,
            REGION_WALLS | REGION_FLOOR,
            BLEND_ADD,
            DOMAIN_TANGENT_LOCAL,
            TRACE_CRACKS,
            0xd3fbff,
            0x69d7ff,
        ),
        lo(156, 92, 34, 168, 0x58, DIR_FORWARD, 10, 6),
    ],
    // L6: MOTTLE/RIDGE - roughen the wall backing so the cave shell stays readable behind the crystal planes and ceiling closure.
    [
        hi_meta(
            OP_MOTTLE,
            REGION_WALLS,
            BLEND_MULTIPLY,
            DOMAIN_DIRECT3D,
            MOTTLE_RIDGE,
            0x486676,
            0x081018,
        ),
        lo(204, 40, 182, 92, 14, DIR_LEFT, 11, 0),
    ],
    // L7: LOBE - keep one narrow reflected-light pulse so the animation reads as crystal bounce inside the basin, not an all-room fog wash.
    [
        hi(
            OP_LOBE,
            REGION_WALLS | REGION_FLOOR,
            BLEND_ADD,
            0,
            0xc8efff,
            0x123044,
        ),
        lo(116, 188, 84, 1, 0, DIR_SUNSET, 8, 2),
    ],
];

// -----------------------------------------------------------------------------
// Preset 14: "Moonlit Graveyard" - Cold grave markers and spectral mist
// -----------------------------------------------------------------------------
// Goal: unmistakable moonlit cemetery with one strong grave-marker owner, a
// readable path/earth recession through the lower frame, and one restrained
// spectral rupture that stays secondary to the graves.
//
// Cadence: SKY BED -> CEMETERY OWNER -> GROUND/PATH OWNER -> MARKER RHYTHM ->
// MOONLIGHT -> SPECTRAL EVENT -> LOW MIST -> GROUND BREAKUP
//
// L0: RAMP                 ALL         LERP      silver-blue night sky into dark cemetery earth
// L1: SILHOUETTE/SPIRES    SKY|WALLS   LERP      crooked grave-marker skyline and cemetery envelope
// L2: PLANE/STONE          FLOOR       LERP      central path / graveyard ground owner
// L3: CELL/BRICK           WALLS       LERP      grave-row rhythm and marker density
// L4: CELESTIAL/MOON       SKY         ADD       cold moon anchor
// L5: PORTAL/RIFT          WALLS       ADD       localized spectral rupture
// L6: ADVECT/MIST          FLOOR|WALLS SCREEN    restrained low mist support (animated)
// L7: MOTTLE/GRAIN         FLOOR       MULTIPLY  earthy breakup strengthening path recession
pub(super) const PRESET_MOONLIT_GRAVEYARD: [[u64; 2]; 8] = [
    // L0: RAMP - start from a colder moonlit sky over much darker soil so the graveyard reads as exterior ground first, not interior blue fog.
    [
        hi(OP_RAMP, REGION_ALL, BLEND_LERP, 0, 0xb3bdd0, 0x090b11),
        lo(232, 0x34, 0x20, 0x4c, THRESH_OPEN, DIR_UP, 15, 15),
    ],
    // L1: SILHOUETTE/SPIRES - make one crooked grave-marker skyline own the view so the composition reads as cemetery before any spectral layer appears.
    [
        hi_meta(
            OP_SILHOUETTE,
            REGION_SKY | REGION_WALLS,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            SILHOUETTE_SPIRES,
            0x11161d,
            0x8793ab,
        ),
        lo(244, 104, 206, 0x42, 0, DIR_UP, 15, 13),
    ],
    // L2: PLANE/STONE - establish one darker path and earth field so the lower frame reads as a cemetery approach instead of flat blue-gray ground.
    [
        hi_meta(
            OP_PLANE,
            REGION_FLOOR,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            PLANE_STONE,
            0x4a515d,
            0x12161c,
        ),
        lo(232, 76, 30, 150, 0, DIR_UP, 15, 12),
    ],
    // L3: CELL/BRICK - use dark marker-row rhythm on the walls to keep the cemetery dense and readable without abstract framing.
    [
        hi_meta(
            OP_CELL,
            REGION_WALLS,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            CELL_BRICK,
            0x5a6472,
            0x0b0e14,
        ),
        lo(176, 18, 214, 104, 0, DIR_FORWARD, 12, 0),
    ],
    // L4: CELESTIAL/MOON - keep one clear moon anchor so the scene reads as moonlit cemetery rather than generic cold night.
    [
        hi_meta(
            OP_CELESTIAL,
            REGION_SKY,
            BLEND_ADD,
            DOMAIN_DIRECT3D,
            CELESTIAL_MOON,
            0xf2f6ff,
            0x8ca0c8,
        ),
        lo(172, 44, 168, 58, 0, DIR_SUNSET, 14, 9),
    ],
    // L5: PORTAL/RIFT - localize the ghostly rupture so it reads as one eerie incident inside the graveyard, not a scene-wide fantasy effect.
    [
        hi_meta(
            OP_PORTAL,
            REGION_WALLS,
            BLEND_ADD,
            DOMAIN_TANGENT_LOCAL,
            PORTAL_RIFT,
            0xadd0f8,
            0x48678f,
        ),
        lo(108, 146, 120, 122, 0, 0x808e, 7, 2),
    ],
    // L6: ADVECT/MIST - keep the mist low and weak so it supports the ground recession and spectral event without erasing the markers.
    [
        hi_meta(
            OP_ADVECT,
            REGION_WALLS | REGION_FLOOR,
            BLEND_SCREEN,
            DOMAIN_DIRECT3D,
            ADVECT_MIST,
            0xe4edf9,
            0x67758d,
        ),
        lo(88, 36, 132, 102, 0, DIR_RIGHT, 7, 1),
    ],
    // L7: MOTTLE/GRAIN - put darker soil breakup onto the floor so the path recession survives under the low mist.
    [
        hi_meta(
            OP_MOTTLE,
            REGION_FLOOR,
            BLEND_MULTIPLY,
            DOMAIN_DIRECT3D,
            MOTTLE_GRAIN,
            0xb2beca,
            0x404955,
        ),
        lo(176, 40, 182, 124, 10, DIR_RIGHT, 10, 0),
    ],
];
