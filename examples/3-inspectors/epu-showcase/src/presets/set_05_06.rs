//! Preset set 05-06

#[allow(unused_imports)]
use crate::constants::*;

// -----------------------------------------------------------------------------
// Preset 5: "Desert Mirage" - Vast dunes under blazing sun
// -----------------------------------------------------------------------------
pub(super) const PRESET_DESERT_MIRAGE: [[u64; 2]; 8] = [
    // L0: RAMP - bleached sky / sand floor
    [
        hi(OP_RAMP, REGION_ALL, BLEND_LERP, 0, 0xf0e8d0, 0xd4b896),
        lo(165, 0xc8, 0xa8, 0x78, THRESH_VAST, DIR_UP, 15, 15),
    ],
    // L1: PLANE/SAND - grain + ripples
    [
        hi_meta(
            OP_PLANE,
            REGION_FLOOR,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            PLANE_SAND,
            0xd8c090,
            0xb08952,
        ),
        lo(195, 120, 20, 165, 28, DIR_UP, 15, 15),
    ],
    // L2: FLOW/NOISE - heat shimmer (animated)
    [
        hi_meta(
            OP_FLOW,
            REGION_ALL,
            BLEND_SCREEN,
            DOMAIN_DIRECT3D,
            0,
            0xf8f0e0,
            0xd4b896,
        ),
        lo(125, 165, 70, 0x30, 0, DIR_RIGHT, 8, 0),
    ],
    // L3: BAND - horizon heat shimmer (animated, make it obvious)
    [
        hi(OP_BAND, REGION_ALL, BLEND_ADD, 0, 0xe8c090, 0xd0a070),
        lo(160, 70, 120, 210, 0, DIR_SUNSET, 12, 0),
    ],
    // L4: LOBE - harsh sun glare (animated)
    [
        hi(OP_LOBE, REGION_ALL, BLEND_ADD, 0, 0xffffd8, 0xffc26b),
        lo(160, 240, 80, 1, 0, DIR_SUN, 12, 0),
    ],
    // L5: PORTAL/RIFT - mirage pool (animated)
    [
        hi_meta(
            OP_PORTAL,
            REGION_FLOOR,
            BLEND_ADD,
            DOMAIN_TANGENT_LOCAL,
            PORTAL_RIFT,
            0xb08952,
            0x86c0ff,
        ),
        lo(210, 140, 170, 175, 0, DIR_UP, 14, 12),
    ],
    // L6: ATMOSPHERE/MIE - desert haze
    [
        hi_meta(
            OP_ATMOSPHERE,
            REGION_ALL,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            ATMO_MIE,
            0xe8d8c0,
            0xd4b896,
        ),
        lo(34, 110, 110, 90, 160, DIR_SUN, 10, 0),
    ],
    // L7: SCATTER/DUST - blowing sand
    [
        hi_meta(
            OP_SCATTER,
            REGION_FLOOR,
            BLEND_ADD,
            DOMAIN_DIRECT3D,
            SCATTER_DUST,
            0xc8b080,
            0xb08952,
        ),
        lo(18, 18, 55, 0x10, 27, DIR_RIGHT, 6, 0),
    ],
];

// -----------------------------------------------------------------------------
// Preset 6: "Enchanted Grove" - Fairy tale forest
// -----------------------------------------------------------------------------
// Design: tree trunks + canopy gap + sun shafts. Avoid "camo" by keeping the
// moving dapple mostly on the ground and keeping leaf breakup as a BOUNDS layer.
//
// Cadence: BOUNDS (RAMP) -> BOUNDS (APERTURE) -> FEATURES (trunks/ground) -> FEATURES (light)
//
// L0: RAMP                 ALL           LERP   deep greens + warm skylight
// L1: APERTURE/IRREGULAR    ALL           LERP   canopy gap (hero light source)
// L2: VEIL/PILLARS          WALLS         LERP   tree trunks
// L3: PLANE/GRASS           FLOOR         LERP   mossy ground
// L4: FLOW (noise)          FLOOR         MULT   moving dapple shadow (animated)
// L5: VEIL/SHARDS           SKY|WALLS     SCREEN sun shafts
// L6: LOBE                  ALL           ADD    warm sun key (animated)
// L7: SCATTER/DUST          ALL           ADD    firefly motes
pub(super) const PRESET_ENCHANTED_GROVE: [[u64; 2]; 8] = [
    // L0: RAMP - deep greens + warm skylight
    [
        hi(OP_RAMP, REGION_ALL, BLEND_LERP, 0, 0xfff0c8, 0x0b1206),
        lo(170, 0x1f, 0x32, 0x18, THRESH_SEMI, DIR_UP, 15, 15),
    ],
    // L1: APERTURE/IRREGULAR - canopy gap (hero light source)
    [
        hi_meta(
            OP_APERTURE,
            REGION_ALL,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            APERTURE_IRREGULAR,
            0xfff0c8,
            0x061005,
        ),
        lo(90, 86, 66, 18, 210, DIR_UP, 0, 0),
    ],
    // L2: VEIL/PILLARS - tree trunks
    [
        hi_meta(
            OP_VEIL,
            REGION_WALLS,
            BLEND_LERP,
            DOMAIN_AXIS_CYL,
            VEIL_PILLARS,
            0x0b0a06,
            0x1a2a10,
        ),
        lo(210, 110, 60, 45, 0, DIR_UP, 15, 0),
    ],
    // L3: PLANE/GRASS - mossy forest floor
    [
        hi_meta(
            OP_PLANE,
            REGION_FLOOR,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            PLANE_GRASS,
            0x2a3a18,
            0x0d1208,
        ),
        lo(190, 130, 25, 180, 0, DIR_UP, 15, 0),
    ],
    // L4: FLOW/NOISE - moving dapple shadow on the ground (animated)
    [
        hi_meta(
            OP_FLOW,
            REGION_FLOOR,
            BLEND_MULTIPLY,
            DOMAIN_DIRECT3D,
            0,
            0x102008,
            0x2a3a18,
        ),
        lo(95, 210, 60, 0x30, 0, DIR_RIGHT, 10, 0),
    ],
    // L5: VEIL/SHARDS - sun shafts (tangent-local)
    [
        hi_meta(
            OP_VEIL,
            REGION_SKY | REGION_WALLS,
            BLEND_SCREEN,
            DOMAIN_TANGENT_LOCAL,
            VEIL_SHARDS,
            0xfff0c8,
            0x061005,
        ),
        lo(38, 70, 10, 55, 0, DIR_SUN, 6, 2),
    ],
    // L6: LOBE - HERO: warm sunbeam (animated)
    [
        hi(OP_LOBE, REGION_ALL, BLEND_ADD, 0, 0xffe1ad, 0x2b1c08),
        lo(180, 170, 150, 1, 0, DIR_SUN, 12, 0),
    ],
    // L7: SCATTER/DUST - firefly motes
    [
        hi_meta(
            OP_SCATTER,
            REGION_ALL,
            BLEND_ADD,
            DOMAIN_TANGENT_LOCAL,
            SCATTER_DUST,
            0xd6ff86,
            0x3cff9a,
        ),
        lo(18, 26, 12, 0x30, 19, DIR_SUN, 7, 0),
    ],
];
