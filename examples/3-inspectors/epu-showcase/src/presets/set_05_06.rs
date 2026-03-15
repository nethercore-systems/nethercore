//! Preset set 05-06

#[allow(unused_imports)]
use crate::constants::*;

// -----------------------------------------------------------------------------
// Preset 5: "Desert Mirage" - Vast dunes under blazing sun
// -----------------------------------------------------------------------------
// Goal: one readable dune basin and one clear horizon-mirage owner that beat
// the generic soft brown wash. Build the scene from a basin split first, then
// hang one dune silhouette on that line, then keep the mirage and glare as
// restrained support rather than as the scene owner.
//
// Cadence: SKY BED -> BASIN SPLIT -> DUNE HORIZON -> SAND FLOOR ->
// FLOOR BREAKUP -> HORIZON SHIMMER -> MIRAGE EVENT -> RESTRAINED HAZE
//
// L0: RAMP                 ALL           LERP      desert sky-to-sand bed
// L1: SPLIT/TIER           ALL           LERP      broad basin / horizon organizer
// L2: SILHOUETTE/DUNES     SKY|WALLS     LERP      owned dune horizon silhouette
// L3: PLANE/SAND           FLOOR         LERP      grounded sand floor owner
// L4: MOTTLE/RIDGE         FLOOR         MULTIPLY  dune-ripple and basin-breakup support
// L5: BAND                 WALLS         SCREEN    restrained horizon shimmer only
// L6: PORTAL/RIFT          FLOOR         SCREEN    localized mirage pool event
// L7: ATMOSPHERE/ABSORPTION ALL          MULTIPLY  warm haze restraint for depth
pub(super) const PRESET_DESERT_MIRAGE: [[u64; 2]; 8] = [
    // L0: RAMP - start from a hotter sky fading into darker sand so the scene has depth before glare layers arrive.
    [
        hi(OP_RAMP, REGION_ALL, BLEND_LERP, 0, 0xd6ae6c, 0x160b05),
        lo(236, 0x40, 0x6e, 0x52, THRESH_VAST, DIR_UP, 15, 15),
    ],
    // L1: SPLIT/TIER - create one broad dune basin so the scene gets a true horizon and floor separation instead of an all-over wash.
    [
        hi_meta(
            OP_SPLIT,
            REGION_ALL,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            SPLIT_TIER,
            0xd1ab68,
            0x352012,
        ),
        lo(18, 92, 122, 126, 0, DIR_UP, 10, 0),
    ],
    // L2: SILHOUETTE/DUNES - place one darker dune horizon on the split so the place read lands before shimmer and haze.
    [
        hi_meta(
            OP_SILHOUETTE,
            REGION_SKY | REGION_WALLS,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            SILHOUETTE_DUNES,
            0x180c04,
            0x6d4a24,
        ),
        lo(216, 152, 188, 118, 0, DIR_UP, 14, 12),
    ],
    // L3: PLANE/SAND - give the lower frame one obvious sand bed so the basin does not float as pure color.
    [
        hi_meta(
            OP_PLANE,
            REGION_FLOOR,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            PLANE_SAND,
            0x8e6a32,
            0x241208,
        ),
        lo(244, 126, 34, 170, 12, DIR_SUN, 15, 14),
    ],
    // L4: MOTTLE/RIDGE - carve ripple and basin contour into the floor so the horizon gets a stronger grounded foreground.
    [
        hi_meta(
            OP_MOTTLE,
            REGION_FLOOR,
            BLEND_MULTIPLY,
            DOMAIN_DIRECT3D,
            MOTTLE_RIDGE,
            0x7d5a2c,
            0x1d0f07,
        ),
        lo(132, 28, 184, 108, 18, DIR_RIGHT, 10, 0),
    ],
    // L5: BAND - keep one restrained heat shimmer band tied to the horizon instead of letting glare fill the frame.
    [
        hi(OP_BAND, REGION_WALLS, BLEND_SCREEN, 0, 0xc7a56a, 0x5a381b),
        lo(28, 76, 136, 126, 0, DIR_FORWARD, 5, 1),
    ],
    // L6: PORTAL/RIFT - keep one localized false-water mirage so the scene still earns the name without turning into reflective glare.
    [
        hi_meta(
            OP_PORTAL,
            REGION_FLOOR,
            BLEND_SCREEN,
            DOMAIN_TANGENT_LOCAL,
            PORTAL_RIFT,
            0x6b8aa0,
            0x2d4056,
        ),
        lo(72, 182, 58, 188, 0, DIR_FORWARD, 8, 3),
    ],
    // L7: ATMOSPHERE/ABSORPTION - keep warm distance haze, but low enough that the basin and dune line remain readable.
    [
        hi_meta(
            OP_ATMOSPHERE,
            REGION_ALL,
            BLEND_MULTIPLY,
            DOMAIN_DIRECT3D,
            ATMO_ABSORPTION,
            0x8e6d46,
            0x54331c,
        ),
        lo(34, 92, 74, 0, 0, DIR_UP, 7, 0),
    ],
];

// -----------------------------------------------------------------------------
// Preset 6: "Enchanted Grove" - Fairy tale forest
// -----------------------------------------------------------------------------
// Goal: a magical grove with one readable canopy arch over a grounded clearing,
// not a dark green graphic field. Build the scene from a clearing bowl first,
// then hang the canopy over it, then add a small believable shaft family and a
// sunpool. No haze-first wall, no mote spam, no single giant wedge.
//
// Cadence: BASE LIGHT -> BOUNDS (clearing bowl) -> CANOPY -> FLOOR ->
// LIGHT (soft shaft family + sunpool) -> SUPPORT (leaf shadow + restrained floor shimmer)
//
// L0: RAMP                 ALL         LERP      warm opening sky over deep understory
// L1: SPLIT/TIER           ALL         LERP      clearing bowl with readable sky/wall/floor separation
// L2: SILHOUETTE/FOREST    SKY|WALLS   LERP      dark canopy arch hung around the opening
// L3: PLANE/GRASS          FLOOR       LERP      moss clearing floor owner
// L4: VEIL/CURTAINS        SKY|WALLS   SCREEN    soft shaft family, localized not bar-like
// L5: LOBE                 WALLS|FLOOR ADD       warm sunpool rooted in the clearing
// L6: MOTTLE/DAPPLE        FLOOR       MULTIPLY  leaf-shadow breakup on the floor
// L7: FLOW                 FLOOR       SCREEN    restrained floor shimmer only
pub(super) const PRESET_ENCHANTED_GROVE: [[u64; 2]; 8] = [
    // L0: RAMP - start with a warm opening above a dark understory basin.
    [
        hi(OP_RAMP, REGION_ALL, BLEND_LERP, 0, 0xe9ddab, 0x060c05),
        lo(236, 0x58, 0x72, 0x4a, THRESH_OPEN, DIR_UP, 15, 15),
    ],
    // L1: SPLIT/TIER - create one broad clearing bowl so the scene has a true floor owner and framed wall mass.
    [
        hi_meta(
            OP_SPLIT,
            REGION_ALL,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            SPLIT_TIER,
            0xd9cf98, // warm opening sky at the top of the bowl
            0x13200f, // deep green-brown wall mass
        ),
        lo(34, 82, 92, 118, 0, DIR_UP, 9, 0),
    ],
    // L2: SILHOUETTE/FOREST - hang one heavier canopy arch over the clearing instead of a full-field foliage wash.
    [
        hi_meta(
            OP_SILHOUETTE,
            REGION_SKY | REGION_WALLS,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            SILHOUETTE_FOREST,
            0x081006, // dark canopy body
            0x445626, // restrained leaf-light support
        ),
        lo(255, 164, 214, 74, 0, DIR_UP, 15, 13),
    ],
    // L3: PLANE/GRASS - establish one obvious moss clearing floor under the opening.
    [
        hi_meta(
            OP_PLANE,
            REGION_FLOOR,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            PLANE_GRASS,
            0x7c9642, // lit moss clearing
            0x12160a, // dark soil-shadow edge
        ),
        lo(236, 98, 18, 152, 0, DIR_UP, 15, 14),
    ],
    // L4: VEIL/CURTAINS - use a soft localized shaft family instead of rigid pillar bars.
    [
        hi_meta(
            OP_VEIL,
            REGION_SKY | REGION_WALLS,
            BLEND_SCREEN,
            DOMAIN_TANGENT_LOCAL,
            VEIL_CURTAINS,
            0xf3e3a5, // warm shaft highlights
            0xa87b2d, // amber shaft support
        ),
        lo(132, 18, 26, 34, 0, DIR_SUN, 9, 2),
    ],
    // L5: LOBE - root the sunlight into the floor and lower canopy edge.
    [
        hi(
            OP_LOBE,
            REGION_WALLS | REGION_FLOOR,
            BLEND_ADD,
            0,
            0xe4c96a,
            0x4a3413,
        ),
        lo(120, 176, 88, 2, 0, DIR_SUN, 9, 2),
    ],
    // L6: MOTTLE/DAPPLE - break the clearing floor with leaf-shadow, not all-over haze.
    [
        hi_meta(
            OP_MOTTLE,
            REGION_FLOOR,
            BLEND_MULTIPLY,
            DOMAIN_DIRECT3D,
            MOTTLE_DAPPLE,
            0x778552,
            0x18200f,
        ),
        lo(112, 46, 150, 84, 12, DIR_SUN, 9, 0),
    ],
    // L7: FLOW - a little floor shimmer keeps the clearing alive without becoming fog.
    [
        hi(OP_FLOW, REGION_FLOOR, BLEND_SCREEN, 0, 0xd0c56d, 0x453816),
        lo(52, 44, 54, 0x16, 0, DIR_SUN, 6, 0),
    ],
];
