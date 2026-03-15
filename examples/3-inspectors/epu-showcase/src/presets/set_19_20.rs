//! Preset set 19-20

#[allow(unused_imports)]
use crate::constants::*;

// -----------------------------------------------------------------------------
// Preset 19: "War Zone" - Battle-torn night front
// -----------------------------------------------------------------------------
// Goal: one unmistakable ruined front / trench-line owner in direct view, with
// a grounded battle-line silhouette and one support event that reinforces the
// front instead of dissolving into brown smoke.
//
// Cadence: NIGHT BED -> REAR RUIN MASS -> TRENCH CUT -> BATTLE-LINE OWNER ->
// GROUND OWNER -> EVENT CARRIER -> FLARE SUPPORT -> GROUND BREAKUP
//
// L0: RAMP                 ALL         LERP      cold battle-night bed
// L1: SILHOUETTE/INDUSTRIAL SKY|WALLS  LERP      ruined rear-front silhouette
// L2: SPLIT/TIER           ALL         MULTIPLY  trench drop / no-man's-land cut
// L3: PATCHES/DEBRIS       WALLS|FLOOR LERP      dominant battle-line wreckage belt
// L4: PLANE/STONE          FLOOR       LERP      churned ground owner
// L5: TRACE/LEAD_LINES     WALLS       ADD       main tracer / fireline event
// L6: LOBE                 WALLS|FLOOR ADD       one-sided flare support on the line
// L7: MOTTLE/RIDGE         FLOOR       MULTIPLY  cratered ground breakup under the front
pub(super) const PRESET_WAR_ZONE: [[u64; 2]; 8] = [
    // L0: RAMP - keep the field cold and dark so the line reads as a structure, not a brown haze.
    [
        hi(OP_RAMP, REGION_ALL, BLEND_LERP, 0, 0x57504a, 0x050406),
        lo(248, 0x28, 0x1c, 0x16, THRESH_BALANCED, DIR_UP, 15, 15),
    ],
    // L1: SILHOUETTE/INDUSTRIAL - build one dark ruined rear-front so the battle line has a coherent backing mass.
    [
        hi_meta(
            OP_SILHOUETTE,
            REGION_SKY | REGION_WALLS,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            SILHOUETTE_INDUSTRIAL,
            0x050505,
            0x7c6a5b,
        ),
        lo(228, 164, 224, 92, 0, DIR_UP, 14, 12),
    ],
    // L2: SPLIT/TIER - carve one darker trench cut so the front line reads as a defended rise over no-man's-land.
    [
        hi_meta(
            OP_SPLIT,
            REGION_ALL,
            BLEND_MULTIPLY,
            DOMAIN_DIRECT3D,
            SPLIT_TIER,
            0x1b1714,
            0x050405,
        ),
        lo(176, 112, 18, 138, 0, DIR_UP, 12, 0),
    ],
    // L3: PATCHES/DEBRIS - make one continuous wreckage belt the dominant trench-line owner across wall and floor.
    [
        hi_meta(
            OP_PATCHES,
            REGION_WALLS | REGION_FLOOR,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            PATCHES_DEBRIS,
            0x8d7764,
            0x120907,
        ),
        lo(236, 74, 212, 118, 0, DIR_FORWARD, 15, 12),
    ],
    // L4: PLANE/STONE - anchor the lower frame with churned mud and broken ground under the battle-line.
    [
        hi_meta(
            OP_PLANE,
            REGION_FLOOR,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            PLANE_STONE,
            0x372b24,
            0x0d0a09,
        ),
        lo(244, 84, 28, 170, 0, DIR_UP, 15, 11),
    ],
    // L5: TRACE/LEAD_LINES - keep one main tracer rake tied to the front so motion strengthens the battle-line instead of floating freely.
    [
        hi_meta(
            OP_TRACE,
            REGION_WALLS,
            BLEND_ADD,
            DOMAIN_TANGENT_LOCAL,
            TRACE_LEAD_LINES,
            0xffe3b8,
            0xff8a38,
        ),
        lo(236, 8, 58, 182, 0x70, DIR_SUNSET, 13, 9),
    ],
    // L6: LOBE - one flare source on the line to support the event without reopening whole-frame smoke glow.
    [
        hi(
            OP_LOBE,
            REGION_WALLS | REGION_FLOOR,
            BLEND_ADD,
            0,
            0xffdd9d,
            0x8e4d24,
        ),
        lo(176, 226, 88, 1, 0, DIR_SUNSET, 12, 2),
    ],
    // L7: MOTTLE/RIDGE - break the floor into cratered ground so the front line sits on authored terrain, not a flat brown bed.
    [
        hi_meta(
            OP_MOTTLE,
            REGION_FLOOR,
            BLEND_MULTIPLY,
            DOMAIN_DIRECT3D,
            MOTTLE_RIDGE,
            0x5f5248,
            0x17110d,
        ),
        lo(188, 32, 196, 108, 14, DIR_RIGHT, 11, 0),
    ],
];

// -----------------------------------------------------------------------------
// Preset 20: "Digital Matrix" - Partitioned cyberspace chamber
// -----------------------------------------------------------------------------
// Goal: keep the recovered chamber, but hold onto one hard partition owner so
// the direct frame reads as a dark machine room rather than a washed graphic.
// The remaining blocker is still the probe-side technical shell, so this pass
// keeps the room-wall split and lets the floor/texture carriers stay subordinate
// instead of reopening the old washed-field or noisy-globe failures.
//
// L0: RAMP                  ALL         LERP      dark synthetic depth bed
// L1: SECTOR/BOX            ALL         LERP      calmer room shell instead of split wedge
// L2: DECAL                 WALLS       LERP      embedded gate slab
// L3: SILHOUETTE/INDUSTRIAL WALLS       LERP      side machine-bank body
// L4: CELL/BRICK            WALLS       LERP      restrained wall seam support
// L5: PLANE/GRATING         FLOOR       LERP      floor grid support, kept subordinate
// L6: GRID                  FLOOR       ADD       near-zero lower scan proof
// L7: MOTTLE/GRAIN          WALLS|FLOOR MULTIPLY  stronger chamber recession
pub(super) const PRESET_DIGITAL_MATRIX: [[u64; 2]; 8] = [
    // L0: RAMP - keep the chamber bed darker so the room starts from depth rather than a cyan poster field.
    [
        hi(OP_RAMP, REGION_ALL, BLEND_LERP, 0, 0x071821, 0x010206),
        lo(176, 0x18, 0x0e, 0x42, THRESH_INTERIOR, DIR_UP, 15, 15),
    ],
    // L1: SECTOR/BOX - replace the broad split wedge with a darker room shell so the chamber reads as an interior before the gate and machine banks.
    [
        hi_meta(
            OP_SECTOR,
            REGION_ALL,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            SECTOR_BOX,
            0x183547,
            0x02050a,
        ),
        lo(184, 150, 168, 0x28, 0, DIR_FORWARD, 14, 10),
    ],
    // L2: DECAL - keep the gate slab embedded in the wall shell, but dim it and shrink it slightly so it stays local hardware.
    [
        hi(
            OP_DECAL,
            REGION_WALLS,
            BLEND_LERP,
            0,
            0x55879a,
            0x07121b,
        ),
        lo(84, 0x16, 178, 20, 0, DIR_FORWARD, 11, 7),
    ],
    // L3: SILHOUETTE/INDUSTRIAL - lean a little harder on the side machine banks so the room reads less like a graphic panel.
    [
        hi_meta(
            OP_SILHOUETTE,
            REGION_WALLS,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            SILHOUETTE_INDUSTRIAL,
            0x030c14,
            0x34586d,
        ),
        lo(248, 150, 212, 78, 0, DIR_UP, 12, 8),
    ],
    // L4: CELL/BRICK - cut the seam carrier even further so it supports the wall without owning the probe.
    [
        hi_meta(
            OP_CELL,
            REGION_WALLS,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            CELL_BRICK,
            0x2c4c5d,
            0x040d15,
        ),
        lo(8, 14, 174, 18, 0, DIR_UP, 4, 2),
    ],
    // L5: PLANE/GRATING - keep a controlled lower deck/grid proof without letting it reopen the older shell ring.
    [
        hi_meta(
            OP_PLANE,
            REGION_FLOOR,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            PLANE_GRATING,
            0x081821,
            0x01050a,
        ),
        lo(192, 88, 20, 152, 8, DIR_UP, 14, 12),
    ],
    // L6: GRID - keep only a tiny floor scan proof so the chamber stays digital without rebuilding a technical ring shell.
    [
        hi(OP_GRID, REGION_FLOOR, BLEND_ADD, 0, 0x8fdde9, 0x000000),
        lo(1, 12, 20, 0x06, 0, 0, 1, 0),
    ],
    // L7: MOTTLE/GRAIN - push recession further so the room falls away around the gate and side machines instead of blooming into a poster panel.
    [
        hi_meta(
            OP_MOTTLE,
            REGION_WALLS | REGION_FLOOR,
            BLEND_MULTIPLY,
            DOMAIN_DIRECT3D,
            MOTTLE_GRAIN,
            0x29404f,
            0x09111a,
        ),
        lo(132, 18, 148, 56, 10, DIR_RIGHT, 9, 0),
    ],
];
