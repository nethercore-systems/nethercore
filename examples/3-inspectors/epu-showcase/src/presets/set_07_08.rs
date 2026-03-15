//! Preset set 07-08

#[allow(unused_imports)]
use crate::constants::*;

// -----------------------------------------------------------------------------
// Preset 7: "Astral Void" - composed cosmic tableau
// -----------------------------------------------------------------------------
// Goal: one readable structured void field in direct view: a dark depth cut,
// broad void pockets, one soft orienting cosmic lane, a small subordinate moon,
// and restrained distant drift. This should recover the field read without
// collapsing into either a single glossy hero orb or a bright cellular shell.
pub(super) const PRESET_ASTRAL_VOID: [[u64; 2]; 8] = [
    // L0: RAMP - keep the bed deep and cold so the scene starts from open void, not from a pale cosmic shell.
    [
        hi(OP_RAMP, REGION_ALL, BLEND_LERP, 0, 0x140f32, 0x000001),
        lo(232, 0x06, 0x08, 0x18, THRESH_VAST, DIR_UP, 15, 15),
    ],
    // L1: SPLIT/FACE - cut one broad dark depth partition through the field so the composition starts from negative space instead of from a central body.
    [
        hi_meta(
            OP_SPLIT,
            REGION_ALL,
            BLEND_MULTIPLY,
            DOMAIN_DIRECT3D,
            SPLIT_FACE,
            0x18193f,
            0x010103,
        ),
        lo(184, 104, 18, 144, 0, DIR_UP, 12, 0),
    ],
    // L2: MASS/VEIL - broaden the main void pocket so the field reads as layered depth with open negative space instead of a single sphere.
    [
        hi_meta(
            OP_MASS,
            REGION_SKY | REGION_WALLS,
            BLEND_MULTIPLY,
            DOMAIN_DIRECT3D,
            MASS_VEIL,
            0x14173a,
            0x010204,
        ),
        lo(220, 86, 182, 66, 0, DIR_LEFT, 12, 0),
    ],
    // L3: BAND - keep one soft lane as orientation only; it should live in the field, not become a bright shaft or ring edge.
    [
        hi(OP_BAND, REGION_SKY, BLEND_SCREEN, 0, 0x626fd0, 0x171f46),
        lo(68, 82, 42, 132, 0, DIR_SUNSET, 6, 1),
    ],
    // L4: CELESTIAL/MOON - keep one smaller moon anchor as a subordinate focal embedded in the field instead of as a hero orb or hard eclipse ring.
    [
        hi_meta(
            OP_CELESTIAL,
            REGION_SKY,
            BLEND_ADD,
            DOMAIN_DIRECT3D,
            CELESTIAL_MOON,
            0xa5b6ff,
            0x0a1021,
        ),
        lo(40, 24, 74, 18, 0, DIR_RIGHT, 6, 2),
    ],
    // L5: FLOW - keep only faint distant drift so motion sits in the field and does not wrap a body.
    [
        hi_meta(
            OP_FLOW,
            REGION_SKY,
            BLEND_SCREEN,
            DOMAIN_DIRECT3D,
            0,
            0x6b83ef,
            0x1a2755,
        ),
        lo(44, 96, 38, 0x14, 8, DIR_RIGHT, 6, 1),
    ],
    // L6: MOTTLE/SOFT - deepen the soft recession layer so the field breaks into broad pockets instead of reading as one shell.
    [
        hi_meta(
            OP_MOTTLE,
            REGION_SKY | REGION_WALLS,
            BLEND_MULTIPLY,
            DOMAIN_DIRECT3D,
            MOTTLE_SOFT,
            0x25305d,
            0x05080f,
        ),
        lo(132, 20, 168, 92, 10, DIR_RIGHT, 9, 0),
    ],
    // L7: SCATTER/STARS - let stars confirm scale more clearly again, but keep them distant so they do not become the main event.
    [
        hi_meta(
            OP_SCATTER,
            REGION_SKY,
            BLEND_ADD,
            DOMAIN_DIRECT3D,
            SCATTER_STARS,
            0xf7fbff,
            0x98a8dd,
        ),
        lo(64, 6, 8, 0x24, 4, 0, 4, 1),
    ],
];

// -----------------------------------------------------------------------------
// Preset 8: "Hell Core" - cracked volcanic heart
// -----------------------------------------------------------------------------
// Goal: make the frame read as shattered volcanic ground first. The lava
// fissures and lower rift should dominate while the rest of the scene supports
// them with charred rock, infernal underglow, and sparse embers.
pub(super) const PRESET_VOLCANIC_CORE: [[u64; 2]; 8] = [
    // L0: RAMP - keep infernal pressure but trim the upper hot band one more small step so the floor fissures lead.
    [
        hi(OP_RAMP, REGION_ALL, BLEND_LERP, 0, 0x010000, 0x010000),
        lo(255, 0x01, 0x02, 0x00, THRESH_OPEN, DIR_UP, 15, 15),
    ],
    // L1: PLANE/STONE - darker volcanic deck under the fissures.
    [
        hi_meta(
            OP_PLANE,
            REGION_FLOOR,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            PLANE_STONE,
            0x0d0703,
            0x030100,
        ),
        lo(255, 120, 102, 178, 0, DIR_UP, 15, 7),
    ],
    // L2: CELL/SHATTER - add a little more crack-linked floor segmentation so the infernal fracture field overtakes the last smooth plates.
    [
        hi_meta(
            OP_CELL,
            REGION_FLOOR,
            BLEND_MULTIPLY,
            DOMAIN_DIRECT3D,
            CELL_SHATTER,
            0x090402,
            0x060200,
        ),
        lo(255, 12, 220, 0x0d, 0, DIR_UP, 15, 0),
    ],
    // L3: TRACE/CRACKS - give the side-floor fissures one more small width/contrast push so they dominate over the remaining smooth chamber planes.
    [
        hi_meta(
            OP_TRACE,
            REGION_FLOOR,
            BLEND_ADD,
            DOMAIN_DIRECT3D,
            TRACE_CRACKS,
            0xffefaf,
            0xff3a00,
        ),
        lo(255, 255, 8, 250, 0x60, DIR_UP, 15, 13),
    ],
    // L4: PORTAL/RIFT - keep the lower hellgate present, but trim its central vertical emphasis one more notch behind the floor fissure web.
    [
        hi_meta(
            OP_PORTAL,
            REGION_FLOOR,
            BLEND_ADD,
            DOMAIN_TANGENT_LOCAL,
            PORTAL_RIFT,
            0xe26000,
            0x2e0500,
        ),
        lo(132, 182, 148, 188, 0, DIR_DOWN, 9, 0),
    ],
    // L5: LOBE - keep the underglow tight and subordinate to the cracks plus hellgate.
    [
        hi(OP_LOBE, REGION_FLOOR, BLEND_ADD, 0, 0xac2400, 0x120200),
        lo(56, 180, 74, 0, 0, DIR_DOWN, 7, 0),
    ],
    // L6: SCATTER/EMBERS - keep embers floor-biased and very sparse so the chamber does not glow back up.
    [
        hi_meta(
            OP_SCATTER,
            REGION_FLOOR,
            BLEND_ADD,
            DOMAIN_DIRECT3D,
            SCATTER_EMBERS,
            0xff962c,
            0xc84200,
        ),
        lo(8, 2, 148, 0x10, 4, DIR_UP, 3, 0),
    ],
    // L7: ATMOSPHERE/ABSORPTION - darken walls harder so broad amber facets recede behind the floor event.
    [
        hi_meta(
            OP_ATMOSPHERE,
            REGION_WALLS,
            BLEND_MULTIPLY,
            DOMAIN_DIRECT3D,
            ATMO_ABSORPTION,
            0x0c0201,
            0x040100,
        ),
        lo(148, 128, 112, 0, 0, DIR_UP, 15, 0),
    ],
];
