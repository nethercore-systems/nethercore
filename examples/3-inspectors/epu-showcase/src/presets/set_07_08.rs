//! Preset set 07-08

#[allow(unused_imports)]
use crate::constants::*;

// -----------------------------------------------------------------------------
// Preset 7: "Astral Void" - Cosmic void
// -----------------------------------------------------------------------------
// Visual: an abstract, demoscene void - near-black with prismatic drift and a
// tasteful starfield. The sphere should read as a dark orb with a subtle rim/key,
// while slow color motion suggests nebula gas without turning into noisy speckle.
pub(super) const PRESET_ASTRAL_VOID: [[u64; 2]; 8] = [
    // L0: RAMP - near-black void with cold indigo walls
    [
        hi(OP_RAMP, REGION_ALL, BLEND_LERP, 0, 0x00040a, 0x01000c),
        lo(245, 0x05, 0x00, 0x1a, THRESH_VAST, DIR_UP, 15, 15),
    ],
    // L1: FLOW/NOISE - subtle nebula tint
    [
        hi_meta(
            OP_FLOW,
            REGION_ALL,
            BLEND_ADD,
            DOMAIN_DIRECT3D,
            0,
            0x240a40,
            0x000000,
        ),
        lo(18, 200, 55, 0x30, 0, DIR_RIGHT, 8, 0),
    ],
    // L2: BAND - dark galactic dust lane
    [
        hi(OP_BAND, REGION_ALL, BLEND_MULTIPLY, 0, 0x3a2a44, 0x08000e),
        lo(80, 62, 70, 175, 0, DIR_SUNSET, 12, 0),
    ],
    // L3: FLOW/NOISE - slow prismatic drift (animated)
    [
        hi_meta(
            OP_FLOW,
            REGION_ALL,
            BLEND_ADD,
            DOMAIN_DIRECT3D,
            0,
            0x2c70b8,
            0x2a0018,
        ),
        lo(32, 190, 20, 0x40, 0, DIR_RIGHT, 8, 0),
    ],
    // L4: SCATTER/STARS - tasteful starfield
    [
        hi_meta(
            OP_SCATTER,
            REGION_SKY,
            BLEND_ADD,
            DOMAIN_DIRECT3D,
            SCATTER_STARS,
            0xf4f8ff,
            0x7aa0ff,
        ),
        lo(70, 28, 6, 0x40, 7, 0, 10, 0),
    ],
    // L5: CELESTIAL/BINARY - break the "eclipse halo" grammar
    [
        hi_meta(
            OP_CELESTIAL,
            REGION_SKY,
            BLEND_ADD,
            DOMAIN_DIRECT3D,
            CELESTIAL_BINARY,
            0x0a1020,
            0xb0d8ff,
        ),
        // Smaller body, gentle phase, modest secondary ratio
        lo(80, 10, 170, 90, 110, DIR_SUNSET, 15, 10),
    ],
    // L6: LOBE - subtle rim/key so the sphere reads (animated)
    [
        hi(OP_LOBE, REGION_ALL, BLEND_ADD, 0, 0xb0d8ff, 0x2a0018),
        lo(90, 180, 70, 1, 0, DIR_UP, 12, 0),
    ],
    // L7: ATMOSPHERE/ABSORPTION - multiplicative haze
    [
        hi_meta(
            OP_ATMOSPHERE,
            REGION_ALL,
            BLEND_MULTIPLY,
            DOMAIN_DIRECT3D,
            ATMO_ABSORPTION,
            0x010008,
            0x000000,
        ),
        lo(80, 150, 128, 0, 0, DIR_UP, 12, 0),
    ],
];

// -----------------------------------------------------------------------------
// Preset 8: "Hell Core" — Infernal chamber
// -----------------------------------------------------------------------------
// Visual: an oppressive cavern lit from below by a single hellgate fissure.
// The floor has thin lava cracks and a bright rift/pool that sells motion; embers
// and smoke haze sit above it, but the big value shapes stay readable on the sphere.
pub(super) const PRESET_VOLCANIC_CORE: [[u64; 2]; 8] = [
    // L0: RAMP - base infernal gradient (keep value hierarchy)
    [
        hi(OP_RAMP, REGION_ALL, BLEND_LERP, 0, 0x030205, 0x1a0500),
        lo(210, 0x14, 0x10, 0x0e, THRESH_INTERIOR, DIR_UP, 15, 15),
    ],
    // L1: PLANE/STONE - basalt floor texture
    [
        hi_meta(
            OP_PLANE,
            REGION_FLOOR,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            PLANE_STONE,
            0x151311,
            0x070708,
        ),
        lo(180, 150, 0, 0, 0, DIR_UP, 15, 15),
    ],
    // L2: FLOW/NOISE - subtle basalt mottling (walls/ceiling)
    [
        hi_meta(
            OP_FLOW,
            REGION_SKY | REGION_WALLS,
            BLEND_MULTIPLY,
            DOMAIN_DIRECT3D,
            0,
            0x0a0606,
            0x060607,
        ),
        lo(200, 120, 90, 0x30, 0, 0, 15, 0),
    ],
    // L3: TRACE/CRACKS - lava veins
    [
        hi_meta(
            OP_TRACE,
            REGION_FLOOR,
            BLEND_ADD,
            DOMAIN_AXIS_CYL,
            TRACE_CRACKS,
            0xff7a12,
            0x2a0b00,
        ),
        // Reduce thickness; keep it readable, not a "wireframe".
        lo(190, 170, 14, 110, 0x50, DIR_UP, 15, 8),
    ],
    // L4: PORTAL/RIFT - hellgate lava pool (animated)
    [
        hi_meta(
            OP_PORTAL,
            REGION_FLOOR,
            BLEND_ADD,
            DOMAIN_TANGENT_LOCAL,
            PORTAL_RIFT,
            0x140400,
            0xff5200,
        ),
        lo(220, 150, 140, 165, 0, DIR_DOWN, 14, 10),
    ],
    // L5: LOBE - heat bounce from below (animated)
    [
        hi(OP_LOBE, REGION_ALL, BLEND_ADD, 0, 0xff3a00, 0x140400),
        lo(255, 220, 80, 1, 0, DIR_DOWN, 12, 8),
    ],
    // L6: SCATTER/EMBERS - rising sparks
    [
        hi_meta(
            OP_SCATTER,
            REGION_SKY | REGION_WALLS,
            BLEND_ADD,
            DOMAIN_DIRECT3D,
            SCATTER_EMBERS,
            0xffc477,
            0x2a0800,
        ),
        lo(34, 12, 30, 0x20, 9, DIR_UP, 10, 0),
    ],
    // L7: ATMOSPHERE/ABSORPTION - smoke absorption
    [
        hi_meta(
            OP_ATMOSPHERE,
            REGION_ALL,
            BLEND_MULTIPLY,
            DOMAIN_DIRECT3D,
            ATMO_ABSORPTION,
            0x141516,
            0x000000,
        ),
        lo(70, 120, 110, 0, 0, DIR_UP, 10, 0),
    ],
];
