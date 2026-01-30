//! Preset set 03-04

#[allow(unused_imports)]
use crate::constants::*;

// -----------------------------------------------------------------------------
// Preset 3: "Ocean Depths" - Deep sea trench
// -----------------------------------------------------------------------------
// Visual: a deep-water column with a darker seabed and a bright, soft surface
// above. Caustics dance on the "roof" (the underside of the surface), faint rays
// streak downward, and a bioluminescent eddy glows overhead to anchor the motion.
// L0: RAMP                 ALL            LERP   base water column
// L1: PLANE/STONE          FLOOR          LERP   basalt seabed
// L2: FLOW (caustic)       SKY            SCREEN caustics on the "roof" (animated)
// L3: VEIL/PILLARS         SKY            SCREEN god-rays from the surface
// L4: LOBE                 ALL            ADD    soft top light (helps reflections)
// L5: SCATTER/DUST         ALL            ADD    marine snow
// L6: ATMOSPHERE/ABSORB    ALL            MULT   deep-water absorption
// L7: PORTAL/VORTEX        SKY            ADD    biolum surface eddy (animated)
pub(super) const PRESET_OCEAN_DEPTHS: [[u64; 2]; 8] = [
    // L0: RAMP - base water column
    [
        hi(OP_RAMP, REGION_ALL, BLEND_LERP, 0, 0x55f6ff, 0x001018),
        lo(165, 0x00, 0x10, 0x18, THRESH_SEMI, DIR_UP, 15, 15),
    ],
    // L1: PLANE/STONE - dark basalt seabed
    [
        hi_meta(
            OP_PLANE,
            REGION_FLOOR,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            PLANE_STONE,
            0x0b2430,
            0x020608,
        ),
        lo(205, 140, 90, 150, 20, DIR_UP, 15, 12),
    ],
    // L2: FLOW - caustics on the roof/surface (animated)
    [
        hi_meta(
            OP_FLOW,
            REGION_SKY,
            BLEND_SCREEN,
            DOMAIN_DIRECT3D,
            0,
            0x9ffcff,
            0x001018,
        ),
        lo(95, 120, 170, 0x22, 0, DIR_RIGHT, 11, 0),
    ],
    // L3: VEIL/PILLARS - soft god-rays from the surface
    [
        hi_meta(
            OP_VEIL,
            REGION_SKY,
            BLEND_SCREEN,
            DOMAIN_DIRECT3D,
            VEIL_PILLARS,
            0xcafcff,
            0x001018,
        ),
        lo(95, 42, 70, 110, 0, DIR_UP, 10, 6),
    ],
    // L4: LOBE - soft top light to give the sphere a readable spec/rim
    [
        hi(OP_LOBE, REGION_ALL, BLEND_ADD, 0, 0x7cf6ff, 0x001018),
        lo(90, 150, 110, 0, 0, DIR_UP, 12, 0),
    ],
    // L5: SCATTER/DUST - sparse marine snow points
    [
        hi_meta(
            OP_SCATTER,
            REGION_ALL,
            BLEND_ADD,
            DOMAIN_DIRECT3D,
            SCATTER_BUBBLES,
            0xcafcff,
            0x001018,
        ),
        // Lower density + slightly larger points to avoid a "dot grid" read.
        lo(14, 22, 16, 0x20, 33, 0, 6, 0),
    ],
    // L6: ATMOSPHERE/ABSORPTION - deep-water light falloff
    [
        hi_meta(
            OP_ATMOSPHERE,
            REGION_ALL,
            BLEND_MULTIPLY,
            DOMAIN_DIRECT3D,
            ATMO_ABSORPTION,
            0x002030,
            0x000000,
        ),
        lo(115, 155, 110, 0, 0, DIR_UP, 12, 0),
    ],
    // L7: PORTAL/VORTEX - faint biolum surface eddy (animated)
    [
        hi_meta(
            OP_PORTAL,
            REGION_SKY,
            BLEND_ADD,
            DOMAIN_TANGENT_LOCAL,
            PORTAL_VORTEX,
            0x001018,
            0x20ffd8,
        ),
        lo(200, 140, 170, 170, 0, DIR_UP, 14, 10),
    ],
];

// -----------------------------------------------------------------------------
// Preset 4: "Void Station" - Derelict space station
// -----------------------------------------------------------------------------
// Goal: clear interior enclosure with a single viewport to deep space.
// Visual: a cold, metallic room with a single rounded viewport cut into the far
// wall. Outside the window is starfield plus a bold eclipse disk; inside, a pale
// light spill washes the floor and panels, keeping the sphere reflection readable.
//
// L0: SECTOR/BOX           ALL        LERP  hard room enclosure
// L1: GRID                 WALLS|FLOOR ADD  subtle panel lines (animated)
// L2: PLANE/GRATING        FLOOR      LERP  deck grating
// L3: APERTURE/RND_RECT    ALL        LERP  viewport frame + region tag
// L4: SCATTER/STARS        SKY        ADD   stars only in the viewport
// L5: CELESTIAL/ECLIPSE    SKY        ADD   eclipse body in the viewport
// L6: DECAL/RECT           WALLS|FLOOR ADD  viewport light card spill
// L7: LOBE                 WALLS|FLOOR ADD  cool spill from viewport (animated)
pub(super) const PRESET_VOID_STATION: [[u64; 2]; 8] = [
    // L0: SECTOR/BOX - room enclosure colors
    [
        hi_meta(
            OP_SECTOR,
            REGION_ALL,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            SECTOR_BOX,
            0x0e1622,
            0x0a0c12,
        ),
        lo(230, 145, 0, 0, 0, DIR_UP, 15, 15),
    ],
    // L1: GRID - subtle panel lines (animated)
    [
        hi(
            OP_GRID,
            REGION_WALLS | REGION_FLOOR,
            BLEND_ADD,
            0,
            0x2e3948,
            0x000000,
        ),
        // scale, thickness, pattern=GRID, slow scroll; phase animates via ANIM_SPEEDS
        lo(18, 120, 14, 0x12, 0, 0, 8, 0),
    ],
    // L2: PLANE/GRATING - deck plating
    [
        hi_meta(
            OP_PLANE,
            REGION_FLOOR,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            PLANE_GRATING,
            0x111821,
            0x070a10,
        ),
        lo(170, 100, 85, 55, 20, DIR_UP, 15, 12),
    ],
    // L3: APERTURE/ROUNDED_RECT - viewport frame + region tag
    [
        hi_meta(
            OP_APERTURE,
            REGION_ALL,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            APERTURE_ROUNDED_RECT,
            // interior (space) / exterior (wall)
            0x050b1f,
            0x080b14,
        ),
        lo(100, 78, 56, 30, 96, DIR_BACK, 0, 0),
    ],
    // L4: SCATTER/STARS - stars only in the viewport opening (SKY)
    [
        hi_meta(
            OP_SCATTER,
            REGION_SKY,
            BLEND_ADD,
            DOMAIN_DIRECT3D,
            SCATTER_STARS,
            0xf8fbff,
            0x6aa6ff,
        ),
        lo(60, 18, 10, 0x40, 7, 0, 10, 0),
    ],
    // L5: CELESTIAL/ECLIPSE - hero celestial body in the viewport
    [
        hi_meta(
            OP_CELESTIAL,
            REGION_SKY,
            BLEND_ADD,
            DOMAIN_DIRECT3D,
            CELESTIAL_ECLIPSE,
            0xb0d8ff,
            0x02030a,
        ),
        // size, offset, halo
        lo(175, 70, 150, 0, 0, DIR_BACK, 15, 10),
    ],
    // L6: DECAL/RECT - viewport light card spill (helps reflection readability)
    [
        hi(
            OP_DECAL,
            REGION_WALLS | REGION_FLOOR,
            BLEND_ADD,
            0,
            0x86c0ff,
            0x0a1220,
        ),
        lo(150, 0x24, 160, 200, 0x10, DIR_BACK, 14, 10),
    ],
    // L7: LOBE - cold spill from the viewport onto the room (animated)
    [
        hi(
            OP_LOBE,
            REGION_WALLS | REGION_FLOOR,
            BLEND_ADD,
            0,
            0x86c0ff,
            0x0a1220,
        ),
        // waveform=1 (sine), phase animated via ANIM_SPEEDS
        lo(220, 130, 80, 1, 0, DIR_BACK, 12, 0),
    ],
];
