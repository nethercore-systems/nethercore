//! Preset set 01-02

#[allow(unused_imports)]
use crate::constants::*;

// -----------------------------------------------------------------------------
// Preset 1: "Neon Metropolis" - Rain-soaked cyberpunk alley
// -----------------------------------------------------------------------------
// Goal: unmistakably "city alley" (wet pavement, signage, rain), with a clean
// enclosure read on the reflection sphere (big shapes first; low noise).
//
// L0: RAMP                 ALL        LERP   base palette + region thresholds
// L1: VEIL/LASER_BARS       WALLS      ADD    vertical neon signage (animated drift)
// L2: SILHOUETTE/CITY       SKY        LERP   skyline cutout band
// L3: PLANE/PAVEMENT        FLOOR      LERP   wet asphalt grounding
// L4: FLOW (caustic)        FLOOR      ADD    neon reflection sheen
// L5: SCATTER/WINDOWS       WALLS      ADD    distant window lights
// L6: DECAL/RECT            WALLS      ADD    hero neon sign (front wall)
// L7: VEIL/RAIN_WALL        SKY|WALLS  SCREEN rain streaks
pub(super) const PRESET_NEON_METROPOLIS: [[u64; 2]; 8] = [
    // L0: RAMP - base palette (ceil_y=+0.60, floor_y=-0.20)
    [
        hi(OP_RAMP, REGION_ALL, BLEND_LERP, 0, 0x050914, 0x020306),
        // softness, wall_r, wall_g, wall_b, thresholds
        lo(210, 0x0b, 0x11, 0x1a, THRESH_BALANCED, DIR_UP, 15, 15),
    ],
    // L1: VEIL/LASER_BARS - vertical neon signage
    [
        hi_meta(
            OP_VEIL,
            REGION_WALLS,
            BLEND_ADD,
            DOMAIN_AXIS_CYL,
            VEIL_LASER_BARS,
            0xff00ff,
            0x00d0ff,
        ),
        lo(70, 140, 18, 90, 0, DIR_DOWN, 10, 4),
    ],
    // L2: SILHOUETTE/CITY - skyline cutout band
    [
        hi_meta(
            OP_SILHOUETTE,
            REGION_SKY,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            SILHOUETTE_CITY,
            0x02030a,
            0x0b1730,
        ),
        lo(85, 120, 170, 0x30, 0, DIR_UP, 15, 0),
    ],
    // L3: PLANE/PAVEMENT - wet asphalt underfoot (reflection-friendly)
    [
        hi_meta(
            OP_PLANE,
            REGION_FLOOR,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            PLANE_PAVEMENT,
            0x0b0f1a,
            0x020205,
        ),
        // scale, (unused), (unused), phase animates via ANIM_SPEEDS
        lo(190, 85, 0, 0, 0, DIR_UP, 15, 0),
    ],
    // L4: FLOW/CAUSTIC - neon sheen on the floor (reflection-friendly)
    [
        hi(OP_FLOW, REGION_FLOOR, BLEND_ADD, 0, 0x1aa0b6, 0x701a93),
        lo(55, 120, 110, 0x22, 0, DIR_FORWARD, 10, 0),
    ],
    // L5: SCATTER/WINDOWS - distant window lights (localized)
    [
        hi_meta(
            OP_SCATTER,
            REGION_WALLS,
            BLEND_ADD,
            DOMAIN_TANGENT_LOCAL,
            SCATTER_WINDOWS,
            0xffd6a0,
            0x3a1c12,
        ),
        // Keep windows subtle; avoid a dense lattice in the sphere reflection.
        lo(6, 8, 12, 0x10, 19, DIR_BACK, 8, 0),
    ],
    // L6: DECAL/RECT - hero neon sign
    [
        hi(OP_DECAL, REGION_WALLS, BLEND_ADD, 0, 0xe81fbf, 0x00d0ff),
        // shape=RECT(2), soft=4, size=110, glow_soft=190; param_d is phase
        // Keep the sign as a hero read, but avoid overpowering the sphere.
        lo(95, 0x24, 34, 90, 0x28, DIR_BACK, 15, 15),
    ],
    // L7: VEIL/RAIN_WALL - rain streaks (keep off the ground; fix direction)
    [
        hi_meta(
            OP_VEIL,
            REGION_SKY | REGION_WALLS,
            BLEND_SCREEN,
            DOMAIN_DIRECT3D,
            VEIL_RAIN_WALL,
            0xddeaff,
            0x0a0f16,
        ),
        lo(40, 96, 10, 150, 0x20, DIR_UP, 9, 2),
    ],
];

// -----------------------------------------------------------------------------
// Preset 2: "Sakura Shrine" - Weathered temple in perpetual bloom
// -----------------------------------------------------------------------------
// Intent: outdoor calm with obvious motion (petals + sun shimmer). Avoid stacking
// multiple bounds; use a blossom canopy bound and then exploit it.
//
// Cadence: BOUNDS (RAMP) -> BOUNDS (PATCHES) -> FEATURES (ground) -> FEATURES (light) -> FEATURES (petals)
//
// L0: RAMP                 ALL            LERP   warm sky / rain-dark shadows
// L1: PATCHES/BLOBS         SKY|WALLS      SCREEN blossom canopy (big shapes)
// L2: PLANE/STONE           FLOOR          LERP   mossy path stones
// L3: FLOW (caustic)        FLOOR          SCREEN wet sheen / dapple (animated)
// L4: LOBE                  ALL            ADD    golden afternoon key (animated)
// L5: VEIL/CURTAINS         SKY|WALLS      SCREEN soft branch/vine curtains
// L6: DECAL/RING            WALLS|FLOOR    ADD    stone lantern glow
// L7: SCATTER/DUST          ALL            ADD    drifting petals
pub(super) const PRESET_SAKURA_SHRINE: [[u64; 2]; 8] = [
    // L0: RAMP - warm sky / rain-dark timber shadows
    [
        hi(OP_RAMP, REGION_ALL, BLEND_LERP, 0, 0xf6c7dd, 0x121a0a),
        lo(180, 0x5a, 0x3a, 0x20, THRESH_OPEN, DIR_UP, 15, 15),
    ],
    // L1: PATCHES/BLOBS - blossom canopy (big, soft)
    [
        hi_meta(
            OP_PATCHES,
            REGION_SKY | REGION_WALLS,
            BLEND_SCREEN,
            DOMAIN_DIRECT3D,
            PATCHES_BLOBS,
            0xffc4e6,
            0x3a5a2a,
        ),
        lo(55, 160, 110, 120, 42, DIR_UP, 12, 12),
    ],
    // L2: PLANE/STONE - mossy path stones
    [
        hi_meta(
            OP_PLANE,
            REGION_FLOOR,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            PLANE_STONE,
            0x5e6b3a,
            0x14180c,
        ),
        lo(185, 85, 18, 175, 0, DIR_UP, 15, 0),
    ],
    // L3: FLOW/CAUSTIC - wet sheen (animated)
    [
        hi(OP_FLOW, REGION_FLOOR, BLEND_SCREEN, 0, 0xfff0c8, 0xc06090),
        lo(85, 120, 120, 0x22, 0, DIR_RIGHT, 10, 0),
    ],
    // L4: LOBE - golden afternoon key (animated)
    [
        hi(OP_LOBE, REGION_ALL, BLEND_ADD, 0, 0xffe1ad, 0x2b1c08),
        lo(180, 190, 125, 1, 0, DIR_SUN, 12, 0),
    ],
    // L5: VEIL/CURTAINS - soft branch curtains
    [
        hi_meta(
            OP_VEIL,
            REGION_SKY | REGION_WALLS,
            BLEND_SCREEN,
            DOMAIN_AXIS_CYL,
            VEIL_CURTAINS,
            0x3a5a2a,
            0x121a0a,
        ),
        lo(22, 100, 14, 55, 0, DIR_RIGHT, 6, 2),
    ],
    // L6: DECAL/RING - stone lantern glow
    [
        hi(
            OP_DECAL,
            REGION_WALLS | REGION_FLOOR,
            BLEND_ADD,
            0,
            0xffd29a,
            0xff7fb8,
        ),
        // shape=RING(1), soft=4
        lo(160, 0x14, 52, 200, 0x10, DIR_SUN, 14, 10),
    ],
    // L7: SCATTER/DUST - drifting petals
    [
        hi_meta(
            OP_SCATTER,
            REGION_ALL,
            BLEND_ADD,
            DOMAIN_TANGENT_LOCAL,
            SCATTER_DUST,
            0xffd6f0,
            0xff7fb8,
        ),
        lo(26, 26, 14, 0x30, 19, DIR_SUN, 7, 0),
    ],
];
