//! Preset set 03-04

#[allow(unused_imports)]
use crate::constants::*;

// -----------------------------------------------------------------------------
// Preset 3: "Ocean Depths" - Deep sea trench
// -----------------------------------------------------------------------------
// Goal: one dark trench basin and seabed owner that beats the pale water-column
// read, with a grounded lower frame and one clear abyssal focal that gives the
// depth somewhere to fall toward.
//
// Cadence: WATER BED -> BASIN SHELL -> SEABED OWNER -> CONTOUR SUPPORT ->
// FOCAL STRUCTURE -> BIOLUM FOCAL -> MOTION SUPPORT -> SPARSE SNOW
//
// L0: RAMP                 ALL            LERP   dark water-bed gradient
// L1: SECTOR/CAVE          ALL            LERP   enclosing trench basin shell
// L2: PLANE/STONE          FLOOR          LERP   basalt seabed owner
// L3: MOTTLE/RIDGE         FLOOR          MULTIPLY floor contour breakup
// L4: SILHOUETTE/SPIRES    WALLS          LERP   vent-chimney / trench structure focal
// L5: PORTAL/VORTEX        FLOOR          ADD    biolum vent at the basin floor
// L6: FLOW                 WALLS|FLOOR    SCREEN restrained trench-current drift
// L7: SCATTER/DUST         SKY|WALLS      ADD    sparse marine snow kept off the floor
pub(super) const PRESET_OCEAN_DEPTHS: [[u64; 2]; 8] = [
    // L0: RAMP - keep only a dim upper-water lift so the scene starts from dark depth instead of a pale cap.
    [
        hi(OP_RAMP, REGION_ALL, BLEND_LERP, 0, 0x1d5562, 0x010205),
        lo(244, 0x10, 0x28, 0x54, THRESH_VAST, DIR_UP, 15, 15),
    ],
    // L1: SECTOR/CAVE - make the whole space read as an enclosing trench basin rather than a flat water column.
    [
        hi_meta(
            OP_SECTOR,
            REGION_ALL,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            SECTOR_CAVE,
            0x08141c,
            0x010205,
        ),
        lo(224, 82, 176, 0x44, 0, DIR_FORWARD, 15, 10),
    ],
    // L2: PLANE/STONE - give the lower frame one dark basalt floor owner so the seabed beats the water-column read.
    [
        hi_meta(
            OP_PLANE,
            REGION_FLOOR,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            PLANE_STONE,
            0x061016,
            0x000102,
        ),
        lo(255, 132, 36, 202, 0, DIR_UP, 15, 14),
    ],
    // L3: MOTTLE/RIDGE - carve floor contours and side-basin breakup so the trench has a darker basin shape around the focal.
    [
        hi_meta(
            OP_MOTTLE,
            REGION_FLOOR,
            BLEND_MULTIPLY,
            DOMAIN_DIRECT3D,
            MOTTLE_RIDGE,
            0x1c2a30,
            0x041017,
        ),
        lo(192, 34, 188, 112, 16, DIR_RIGHT, 13, 0),
    ],
    // L4: SILHOUETTE/SPIRES - add one vent-chimney family so the basin has a readable structure dropping into depth.
    [
        hi_meta(
            OP_SILHOUETTE,
            REGION_WALLS,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            SILHOUETTE_SPIRES,
            0x02080c,
            0x1d4c57,
        ),
        lo(164, 46, 168, 92, 0, DIR_UP, 11, 7),
    ],
    // L5: PORTAL/VORTEX - keep one bright abyssal vent at the basin floor as the depth focal.
    [
        hi_meta(
            OP_PORTAL,
            REGION_FLOOR,
            BLEND_ADD,
            DOMAIN_TANGENT_LOCAL,
            PORTAL_VORTEX,
            0x031216,
            0xb8fff2,
        ),
        lo(228, 220, 156, 214, 8, 0x80bc, 14, 8),
    ],
    // L6: FLOW - keep motion low in the trench body so the lower scene moves without reintroducing a pale full-column shimmer.
    [
        hi(
            OP_FLOW,
            REGION_WALLS | REGION_FLOOR,
            BLEND_SCREEN,
            0,
            0x4ba6b0,
            0x08202a,
        ),
        lo(144, 76, 148, 0x18, 14, DIR_RIGHT, 9, 1),
    ],
    // L7: SCATTER/DUST - keep only sparse marine snow in the upper water and walls so the floor stays owned.
    [
        hi_meta(
            OP_SCATTER,
            REGION_SKY | REGION_WALLS,
            BLEND_ADD,
            DOMAIN_DIRECT3D,
            SCATTER_DUST,
            0x7698a3,
            0x15313e,
        ),
        lo(10, 10, 10, 0x0d, 18, DIR_DOWN, 3, 1),
    ],
];

// -----------------------------------------------------------------------------
// Preset 4: "Void Station" - Derelict space station
// -----------------------------------------------------------------------------
// Goal: keep the recovered maintenance-bay direction, but localize the hatch
// and bulkhead further so the room stops reading as one bright shell wrapped
// around a cap. Let side machinery and the lower deck carry more of the bay.
// Avoid reopening the dark-dome, speckle, or washed-shell failures.
//
// Cadence: HULL BED -> BULKHEAD CUT -> REAR HATCH -> DECK OWNER ->
// SIDE MACHINES -> WALL PANELS -> DECK RAILS -> SIDE/FLOOR RECESSION
//
// L0: RAMP                  ALL         LERP      darker hull bed and bay shell
// L1: SPLIT/FACE            WALLS       LERP      narrower rear bulkhead cut
// L2: DECAL/RECT            WALLS       LERP      dimmer inset rear hatch owner
// L3: PLANE/GRATING         FLOOR       LERP      darker grounded maintenance deck wedge
// L4: SILHOUETTE/INDUSTRIAL WALLS       LERP      stronger side machine-bank framing
// L5: CELL/BRICK            WALLS       LERP      minimal wall panel breakup
// L6: GRID                  FLOOR       ADD       almost-zero deck rails
// L7: MOTTLE/GRAIN          WALLS|FLOOR MULTIPLY  stronger shell recession around hatch
pub(super) const PRESET_VOID_STATION: [[u64; 2]; 8] = [
    // L0: RAMP - darken the hull bed so the shell stops blooming across the whole bay.
    [
        hi(OP_RAMP, REGION_ALL, BLEND_LERP, 0, 0x434f5d, 0x010205),
        lo(104, 0x18, 0x14, 0x30, THRESH_INTERIOR, DIR_UP, 15, 15),
    ],
    // L1: SPLIT/FACE - tighten the rear bulkhead cut so it reads as a local wall insert, not a shell cap.
    [
        hi_meta(
            OP_SPLIT,
            REGION_WALLS,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            SPLIT_FACE,
            0x4b5762,
            0x05080d,
        ),
        lo(136, 170, 18, 126, 0, DIR_FORWARD, 14, 8),
    ],
    // L2: DECAL/RECT - keep the hatch readable, but dimmer and smaller so it stays local hardware.
    [
        hi(
            OP_DECAL,
            REGION_WALLS,
            BLEND_LERP,
            0,
            0x6a7787,
            0x0f161d,
        ),
        lo(104, 0x18, 176, 18, 0, DIR_FORWARD, 12, 8),
    ],
    // L3: PLANE/GRATING - keep the lower deck grounded, but darker so it supports rather than flares.
    [
        hi_meta(
            OP_PLANE,
            REGION_FLOOR,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            PLANE_GRATING,
            0x313946,
            0x04070b,
        ),
        lo(208, 96, 20, 166, 0, DIR_UP, 15, 14),
    ],
    // L4: SILHOUETTE/INDUSTRIAL - push the side machine banks harder so the room reads as machinery around a hatch.
    [
        hi_meta(
            OP_SILHOUETTE,
            REGION_WALLS,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            SILHOUETTE_INDUSTRIAL,
            0x070b10,
            0x698093,
        ),
        lo(248, 154, 210, 84, 0, DIR_UP, 12, 7),
    ],
    // L5: CELL/BRICK - keep only a whisper of wall panel breakup so the probe inherits fewer technical rings.
    [
        hi_meta(
            OP_CELL,
            REGION_WALLS,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            CELL_BRICK,
            0x465361,
            0x091017,
        ),
        lo(8, 16, 184, 20, 0, DIR_UP, 6, 2),
    ],
    // L6: GRID - almost-zero deck rails, only enough to hint at service flooring.
    [
        hi(OP_GRID, REGION_FLOOR, BLEND_ADD, 0, 0xbfd8ee, 0x000000),
        lo(2, 20, 24, 0x0a, 0, 0, 2, 0),
    ],
    // L7: MOTTLE/GRAIN - deepen side and floor recession around the hatch so the shell falls back behind the bay.
    [
        hi_meta(
            OP_MOTTLE,
            REGION_WALLS | REGION_FLOOR,
            BLEND_MULTIPLY,
            DOMAIN_DIRECT3D,
            MOTTLE_GRAIN,
            0x333d48,
            0x10161d,
        ),
        lo(112, 20, 148, 64, 10, DIR_RIGHT, 8, 0),
    ],
];
