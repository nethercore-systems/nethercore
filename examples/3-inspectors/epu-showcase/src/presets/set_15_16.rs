//! Preset set 15-16

#[allow(unused_imports)]
use crate::constants::*;

// -----------------------------------------------------------------------------
// Preset 15: "Alien Jungle" - humid extraterrestrial canopy
// -----------------------------------------------------------------------------
// Goal: a dense non-Earth jungle with a readable alien canopy silhouette,
// toxic humid atmosphere, glowing spores, and strange organic color separation
// that does not collapse into Enchanted Grove recolor territory.
//
// L0: RAMP                 ALL         LERP      toxic olive-to-indigo canopy dusk
// L1: SILHOUETTE/FOREST    SKY|WALLS   LERP      dense alien vine-canopy envelope
// L2: PLANE/GRASS          FLOOR       LERP      humid bio-floor with violet separation
// L3: ATMOSPHERE/ALIEN     SKY|WALLS   SCREEN    restrained toxic humidity
// L4: VEIL/CURTAINS        WALLS       SCREEN    hanging vine-frond curtains
// L5: FLOW                 ALL         SCREEN    humid canopy shimmer / fog drift
// L6: SCATTER/DUST         ALL         ADD       bioluminescent spores
// L7: LOBE                 WALLS|FLOOR ADD       under-canopy alien glow
pub(super) const PRESET_ALIEN_JUNGLE: [[u64; 2]; 8] = [
    // L0: RAMP - vine-canopy candidate dusk, shifted toward toxic olive over deep indigo.
    [
        hi(OP_RAMP, REGION_ALL, BLEND_LERP, 0, 0x9ca83e, 0x0c0a22),
        lo(228, 0x58, 0x92, 0x58, THRESH_OPEN, DIR_UP, 15, 15),
    ],
    // L1: SILHOUETTE/FOREST - broaden the alien canopy wall into sky and upper walls.
    [
        hi_meta(
            OP_SILHOUETTE,
            REGION_SKY | REGION_WALLS,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            SILHOUETTE_FOREST,
            0x020a0c,
            0x36563e,
        ),
        lo(255, 196, 236, 98, 0, DIR_UP, 15, 14),
    ],
    // L2: PLANE/GRASS - humid floor bias with stronger violet separation than the Grove lane.
    [
        hi_meta(
            OP_PLANE,
            REGION_FLOOR,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            PLANE_GRASS,
            0x122a26,
            0x601276,
        ),
        lo(204, 84, 28, 140, 0, DIR_UP, 15, 11),
    ],
    // L3: ATMOSPHERE/ALIEN - toxic humidity stays present but not so strong that it buries foliage shape.
    [
        hi_meta(
            OP_ATMOSPHERE,
            REGION_SKY | REGION_WALLS,
            BLEND_SCREEN,
            DOMAIN_DIRECT3D,
            ATMO_ALIEN,
            0x9aff5e,
            0x1a7086,
        ),
        lo(92, 112, 164, 0, 0, DIR_UP, 9, 0),
    ],
    // L4: VEIL/CURTAINS - hanging alien vine-fronds instead of shard-chamber geometry.
    [
        hi_meta(
            OP_VEIL,
            REGION_WALLS,
            BLEND_SCREEN,
            DOMAIN_AXIS_CYL,
            VEIL_CURTAINS,
            0x4cffbc,
            0x125662,
        ),
        lo(124, 28, 44, 108, 176, DIR_DOWN, 11, 7),
    ],
    // L5: FLOW - humid shimmer and fog drift through the jungle volume.
    [
        hi_meta(
            OP_FLOW,
            REGION_ALL,
            BLEND_SCREEN,
            DOMAIN_DIRECT3D,
            0,
            0x34ecac,
            0x561076,
        ),
        lo(184, 108, 72, 0x2a, 200, DIR_RIGHT, 11, 6),
    ],
    // L6: SCATTER/DUST - glowing spores, brighter and stranger than fireflies.
    [
        hi_meta(
            OP_SCATTER,
            REGION_ALL,
            BLEND_ADD,
            DOMAIN_DIRECT3D,
            SCATTER_DUST,
            0x7effee,
            0xb4ff5e,
        ),
        lo(72, 18, 164, 0x24, 0, 0, 9, 4),
    ],
    // L7: LOBE - canopy-biased glow rooted under the foliage instead of filling the whole dome.
    [
        hi(
            OP_LOBE,
            REGION_WALLS | REGION_FLOOR,
            BLEND_ADD,
            0,
            0x58ffb0,
            0x127658,
        ),
        lo(72, 164, 88, 2, 200, DIR_FORWARD, 8, 3),
    ],
];

// -----------------------------------------------------------------------------
// Preset 16: "Gothic Cathedral" - towering sacred interior
// -----------------------------------------------------------------------------
// Goal: one unmistakable cathedral nave with arch ownership and one readable
// stained-light support event, without leaning on the old bright-bar / tracery
// rail family.
//
// Cadence: SHELL -> NAVE MASS -> ARCH LANGUAGE -> FLOOR -> STAINED WINDOW ->
// STAINED SPILL -> STONE BREAKUP
//
// L0: RAMP                 ALL         LERP      cool stone enclosure
// L1: SECTOR/TUNNEL        ALL         LERP      long nave shell
// L2: DECAL/RECT           WALLS|FLOOR LERP      lower nave / altar mass carrier
// L3: APERTURE/ARCH        WALLS       LERP      high arch openings
// L4: PLANE/STONE          FLOOR       LERP      stone nave floor
// L5: PORTAL/RECT          WALLS       ADD       single stained window / clerestory event
// L6: LOBE                 WALLS|FLOOR ADD       stained spill across nave
// L7: MOTTLE/RIDGE         WALLS|FLOOR MULTIPLY  worn stone breakup
pub(super) const PRESET_GOTHIC_CATHEDRAL: [[u64; 2]; 8] = [
    // L0: RAMP - hold a cool clerestory-to-nave gradient so the shell reads as stone before the sacred accents land.
    [
        hi(OP_RAMP, REGION_ALL, BLEND_LERP, 0, 0xb1b7c5, 0x130e14),
        lo(248, 0x34, 0x30, 0x46, THRESH_INTERIOR, DIR_UP, 15, 15),
    ],
    // L1: SECTOR/TUNNEL - make the room read as a long nave with vaulted recession, not a flat chamber.
    [
        hi_meta(
            OP_SECTOR,
            REGION_ALL,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            SECTOR_TUNNEL,
            0x7f8797,
            0x171018,
        ),
        lo(224, 92, 184, 0x44, 0, DIR_FORWARD, 15, 12),
    ],
    // L2: DECAL/RECT - add one lower nave / altar mass so the direct frame owns a believable interior body before stained light.
    [
        hi(OP_DECAL, REGION_WALLS | REGION_FLOOR, BLEND_LERP, 0, 0x8f9099, 0x171018),
        lo(196, 0x24, 214, 86, 0x18, 0x8088, 11, 13),
    ],
    // L3: APERTURE/ARCH - keep the arch language high and structural instead of close-up tracery.
    [
        hi_meta(
            OP_APERTURE,
            REGION_WALLS,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            APERTURE_ARCH,
            0x5b6273,
            0x15101a,
        ),
        lo(188, 116, 150, 46, 0, DIR_UP, 11, 0),
    ],
    // L4: PLANE/STONE - give the lower nave one readable stone floor owner under the altar mass.
    [
        hi_meta(
            OP_PLANE,
            REGION_FLOOR,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            PLANE_STONE,
            0x66656d,
            0x1c171c,
        ),
        lo(232, 80, 24, 150, 0, DIR_UP, 15, 12),
    ],
    // L5: PORTAL/RECT - use one stained clerestory/window event instead of a family of bright rails.
    [
        hi_meta(
            OP_PORTAL,
            REGION_WALLS,
            BLEND_ADD,
            DOMAIN_TANGENT_LOCAL,
            PORTAL_RECT,
            0x3a4768,
            0xc36c8a,
        ),
        lo(96, 156, 52, 0, 0, 0x80c8, 6, 6),
    ],
    // L6: LOBE - throw one colored sacred spill from that window across the walls and floor.
    [
        hi(
            OP_LOBE,
            REGION_WALLS | REGION_FLOOR,
            BLEND_ADD,
            0,
            0xe8d8c5,
            0x7b5a84,
        ),
        lo(104, 206, 74, 1, 0, DIR_UP, 9, 3),
    ],
    // L7: MOTTLE/RIDGE - roughen the stone so the nave stays architectural instead of collapsing into smooth guides.
    [
        hi_meta(
            OP_MOTTLE,
            REGION_WALLS | REGION_FLOOR,
            BLEND_MULTIPLY,
            DOMAIN_DIRECT3D,
            MOTTLE_RIDGE,
            0x726f76,
            0x19141a,
        ),
        lo(120, 32, 164, 108, 10, DIR_RIGHT, 8, 0),
    ],
];
