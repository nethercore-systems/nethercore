//! Preset set 01-02

#[allow(unused_imports)]
use crate::constants::*;

// -----------------------------------------------------------------------------
// Preset 1: "Neon Metropolis" - Dense avenue under neon towers
// -----------------------------------------------------------------------------
// Goal: unmistakable neon skyline and avenue depth with one dominant city
// structure in direct view. The frame should read as a built urban canyon with
// a hero sign/tower face, dense windows, and a wet street pulling inward, not
// a diffuse magenta/teal wash.
//
// Cadence: SHELL (smog + avenue canyon + skyline slab) -> STREET (pavement) ->
// CITY LIGHT (hero sign + windows) -> MOTION (street reflections + rain depth)
//
// L0: RAMP                 ALL         LERP      polluted night shell from sodium-violet haze to black street
// L1: SECTOR/TUNNEL        ALL         LERP      long avenue recession / urban canyon depth
// L2: SILHOUETTE/CITY      SKY|WALLS   LERP      dense skyline slab and dominant tower mass
// L3: DECAL/RECT           WALLS       SCREEN    one hero vertical sign / tower-face anchor
// L4: PLANE/PAVEMENT       FLOOR       LERP      wet avenue owner
// L5: SCATTER/WINDOWS      WALLS       ADD       dense window rhythm on the side blocks
// L6: FLOW                 FLOOR       SCREEN    neon street reflections carrying the avenue
// L7: VEIL/RAIN_WALL       ALL         SCREEN    rain depth tying the skyline and avenue together
pub(super) const PRESET_NEON_METROPOLIS: [[u64; 2]; 8] = [
    // L0: RAMP - start with a dirtier urban night shell so the scene reads as city atmosphere over dark street, not pure color wash.
    [
        hi(OP_RAMP, REGION_ALL, BLEND_LERP, 0, 0x7c4a86, 0x04060b),
        lo(224, 0x42, 0x38, 0x66, THRESH_INTERIOR, DIR_UP, 15, 15),
    ],
    // L1: SECTOR/TUNNEL - push the scene into an avenue canyon so the frame gets a clear urban depth cue instead of flat side washes.
    [
        hi_meta(
            OP_SECTOR,
            REGION_ALL,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            SECTOR_TUNNEL,
            0x4f405f,
            0x090b12,
        ),
        lo(212, 92, 170, 0x40, 0, DIR_FORWARD, 15, 11),
    ],
    // L2: SILHOUETTE/CITY - let one skyline slab and tower wall own the direct frame so the scene reads as metropolis first, alley second.
    [
        hi_meta(
            OP_SILHOUETTE,
            REGION_SKY | REGION_WALLS,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            SILHOUETTE_CITY,
            0x020305,
            0x17101f,
        ),
        lo(255, 188, 238, 0x82, 0, DIR_UP, 15, 14),
    ],
    // L3: DECAL/RECT - anchor the composition with one readable vertical neon sign embedded in the tower face.
    [
        hi(OP_DECAL, REGION_WALLS, BLEND_SCREEN, 0, 0xff4aa8, 0x34e6ff),
        lo(208, 0x24, 226, 56, 0x30, DIR_BACK, 12, 14),
    ],
    // L4: PLANE/PAVEMENT - make the lower frame unmistakably avenue pavement instead of generic dark floor.
    [
        hi_meta(
            OP_PLANE,
            REGION_FLOOR,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            PLANE_PAVEMENT,
            0x141922,
            0x06080d,
        ),
        lo(236, 118, 22, 152, 0, DIR_UP, 15, 14),
    ],
    // L5: SCATTER/WINDOWS - increase window density so the side blocks read as inhabited city walls instead of blank neon haze.
    [
        hi_meta(
            OP_SCATTER,
            REGION_WALLS,
            BLEND_ADD,
            DOMAIN_DIRECT3D,
            SCATTER_WINDOWS,
            0xffd58c,
            0x7e5a2a,
        ),
        lo(96, 16, 14, 0x24, 18, DIR_BACK, 11, 5),
    ],
    // L6: FLOW - keep the moving color on the avenue floor so motion reinforces street depth instead of fogging the whole frame.
    [
        hi(OP_FLOW, REGION_FLOOR, BLEND_SCREEN, 0, 0x3ed6ff, 0xd43fb2),
        lo(112, 148, 58, 0x22, 0, DIR_FORWARD, 10, 4),
    ],
    // L7: VEIL/RAIN_WALL - let rain tie the skyline and avenue planes together, but keep it subordinate to structure and street.
    [
        hi_meta(
            OP_VEIL,
            REGION_ALL,
            BLEND_SCREEN,
            DOMAIN_AXIS_CYL,
            VEIL_RAIN_WALL,
            0x5f7ea8,
            0x1d2936,
        ),
        lo(88, 52, 20, 142, 0x18, DIR_DOWN, 9, 3),
    ],
];

// -----------------------------------------------------------------------------
// Preset 2: "Sakura Shrine" - Weathered temple in perpetual bloom
// -----------------------------------------------------------------------------
// Goal: one unmistakable shrine / torii silhouette and one readable stone
// approach path, with blossom kept as a supporting accent instead of a diffuse
// pink field.
//
// Cadence: SKY BED -> SHRINE OWNER -> SHRINE CUTOUTS -> PATH OWNER ->
// PATH LIGHT -> BRANCH SHADOW -> BLOSSOM ACCENT -> AIR DEPTH
//
// L0: RAMP                 ALL         LERP      dusk garden bed
// L1: SILHOUETTE/SPIRES    SKY|WALLS   LERP      shrine / torii owner
// L2: APERTURE/ARCH        WALLS       LERP      gate-window cutouts in the shrine mass
// L3: PLANE/STONE          FLOOR       LERP      stone approach path
// L4: LOBE                 WALLS|FLOOR ADD       warm path / shrine light
// L5: MOTTLE/DAPPLE        WALLS|FLOOR MULTIPLY  branch-shadow breakup
// L6: SCATTER/DUST         ALL         ADD       restrained blossom petals
// L7: ATMOSPHERE           ALL         MULTIPLY  restrained garden depth
pub(super) const PRESET_SAKURA_SHRINE: [[u64; 2]; 8] = [
    // L0: RAMP - establish a warm dusk canopy over a darker garden floor so the shrine and path can read against a stable bed.
    [
        hi(OP_RAMP, REGION_ALL, BLEND_LERP, 0, 0xdab889, 0x100d09),
        lo(232, 0x3e, 0x46, 0x4c, THRESH_OPEN, DIR_UP, 15, 15),
    ],
    // L1: SILHOUETTE/SPIRES - make one dark shrine / torii owner that clearly beats the blossom support.
    [
        hi_meta(
            OP_SILHOUETTE,
            REGION_SKY | REGION_WALLS,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            SILHOUETTE_SPIRES,
            0x050201,
            0x5f3118,
        ),
        lo(236, 92, 184, 56, 0, DIR_UP, 14, 12),
    ],
    // L2: APERTURE/ARCH - carve shrine/gate openings into the wall mass so the silhouette reads as architecture, not just pointed trees.
    [
        hi_meta(
            OP_APERTURE,
            REGION_WALLS,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            APERTURE_ARCH,
            0xe8d7bf,
            0x130905,
        ),
        lo(168, 108, 132, 42, 148, DIR_UP, 10, 0),
    ],
    // L3: PLANE/STONE - give the lower frame one clear stone approach path instead of a generalized moss floor.
    [
        hi_meta(
            OP_PLANE,
            REGION_FLOOR,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            PLANE_STONE,
            0x6f7061,
            0x1b1a14,
        ),
        lo(238, 96, 24, 158, 0, DIR_UP, 15, 13),
    ],
    // L4: LOBE - root the warm accent on the shrine frontage and path so the approach cue wins before petals.
    [
        hi(OP_LOBE, REGION_WALLS | REGION_FLOOR, BLEND_ADD, 0, 0xf0c989, 0x5d3414),
        lo(120, 182, 84, 1, 0, DIR_SUNSET, 10, 2),
    ],
    // L5: MOTTLE/DAPPLE - add branch-shadow breakup so the scene stays shrine-garden specific instead of broad warm wash.
    [
        hi_meta(
            OP_MOTTLE,
            REGION_WALLS | REGION_FLOOR,
            BLEND_MULTIPLY,
            DOMAIN_DIRECT3D,
            MOTTLE_DAPPLE,
            0x6f5c47,
            0x20150d,
        ),
        lo(112, 42, 148, 78, 12, DIR_SUNSET, 8, 0),
    ],
    // L6: SCATTER/DUST - keep blossoms present but subordinate so they accent the shrine instead of replacing it.
    [
        hi_meta(
            OP_SCATTER,
            REGION_ALL,
            BLEND_ADD,
            DOMAIN_DIRECT3D,
            SCATTER_DUST,
            0xffa3b4,
            0xc86c7e,
        ),
        lo(18, 10, 14, 0x1c, 10, DIR_SUNSET, 6, 2),
    ],
    // L7: ATMOSPHERE - keep only restrained depth absorption so the frame does not collapse back into haze.
    [
        hi_meta(
            OP_ATMOSPHERE,
            REGION_ALL,
            BLEND_MULTIPLY,
            DOMAIN_DIRECT3D,
            ATMO_ABSORPTION,
            0xb59679,
            0x6a4d38,
        ),
        lo(20, 86, 62, 0, 0, DIR_UP, 5, 0),
    ],
];
