//! Preset set 03-04

#[allow(unused_imports)]
use crate::constants::*;

// -----------------------------------------------------------------------------
// Preset 3: "Ocean Depths" - Deep sea trench
// -----------------------------------------------------------------------------
// Visual: BOUNDLESS deep water column - RAMP creates infinite depth gradient.
// Dark overall with bright surface glow above fading to abyssal black below.
// Feature layers add caustic light patterns, god-rays, and bioluminescence.
// L0: RAMP                 ALL            LERP   deep water column gradient (dark base)
// L1: PLANE/STONE          FLOOR          LERP   basalt seabed
// L2: FLOW (caustic)       SKY            ADD    caustics dancing on surface (animated)
// L3: VEIL/PILLARS         ALL            ADD    god-rays streaking down
// L4: LOBE                 SKY            ADD    bright surface glow above
// L5: PORTAL/VORTEX        SKY            ADD    biolum surface eddy (animated)
// L6: SCATTER/DUST         ALL            ADD    marine snow particles
// L7: SCATTER/BUBBLES      ALL            ADD    rising bubbles
pub(super) const PRESET_OCEAN_DEPTHS: [[u64; 2]; 8] = [
    // L0: RAMP - deep water column gradient (boundless depth)
    [
        hi(OP_RAMP, REGION_ALL, BLEND_LERP, 0, 0x0a3040, 0x010408),
        // Dark teal surface fading to near-black abyss
        lo(255, 0x08, 0x18, 0x28, THRESH_VAST, DIR_UP, 15, 15),
    ],
    // L1: PLANE/STONE - dark basalt seabed
    [
        hi_meta(
            OP_PLANE,
            REGION_FLOOR,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            PLANE_STONE,
            0x081820,
            0x020608,
        ),
        lo(200, 120, 70, 150, 15, DIR_UP, 15, 14),
    ],
    // L2: FLOW - caustic shimmer at surface (SLOW animation)
    [
        hi(OP_FLOW, REGION_SKY, BLEND_ADD, 0, 0x40a0b0, 0x183848),
        // SLOW animation (alpha_b=1)
        lo(100, 80, 140, 0x1c, 15, DIR_DOWN, 9, 1),
    ],
    // L3: VEIL/PILLARS - god-rays (SLOW)
    [
        hi_meta(
            OP_VEIL,
            REGION_ALL,
            BLEND_ADD,
            DOMAIN_AXIS_CYL,
            VEIL_PILLARS,
            0x206060, // muted cyan rays
            0x081420,
        ),
        // SLOW animation (alpha_b=1)
        lo(80, 45, 80, 55, 5, DIR_DOWN, 8, 1),
    ],
    // L4: LOBE - surface glow from above
    [
        hi(OP_LOBE, REGION_SKY, BLEND_ADD, 0, 0x308090, 0x102030),
        lo(120, 160, 80, 0, 0, DIR_UP, 10, 0),
    ],
    // L5: PORTAL/VORTEX - bioluminescent glow (SLOW)
    [
        hi_meta(
            OP_PORTAL,
            REGION_SKY,
            BLEND_ADD,
            DOMAIN_TANGENT_LOCAL,
            PORTAL_VORTEX,
            0x20a080, // cyan-green biolum
            0x103030,
        ),
        // SLOW animation (alpha_b=1)
        lo(100, 120, 130, 140, 8, DIR_UP, 9, 1),
    ],
    // L6: SCATTER/DUST - marine snow (SLOW drift)
    [
        hi_meta(
            OP_SCATTER,
            REGION_ALL,
            BLEND_ADD,
            DOMAIN_DIRECT3D,
            SCATTER_DUST,
            0x508090,
            0x203040,
        ),
        // SLOW animation (alpha_b=1)
        lo(40, 20, 15, 0x14, 25, DIR_DOWN, 6, 1),
    ],
    // L7: SCATTER/BUBBLES - rising bubbles (SLOW)
    [
        hi_meta(
            OP_SCATTER,
            REGION_ALL,
            BLEND_ADD,
            DOMAIN_DIRECT3D,
            SCATTER_BUBBLES,
            0x406070,
            0x102030,
        ),
        // SLOW animation (alpha_b=1)
        lo(30, 15, 10, 0x10, 20, DIR_UP, 5, 1),
    ],
];

// -----------------------------------------------------------------------------
// Preset 4: "Void Station" - Derelict space station
// -----------------------------------------------------------------------------
// Goal: clear interior bounds with a single viewport to deep space.
// Visual: a cold, metallic room with a single rounded viewport cut into the far
// wall. Outside the window is starfield plus a bold eclipse disk; inside, a pale
// light spill washes the floor and panels, keeping the sphere reflection readable.
//
// L0: SECTOR/BOX           ALL        LERP  hard room bounds
// L1: GRID                 WALLS|FLOOR ADD  subtle panel lines (animated)
// L2: PLANE/GRATING        FLOOR      LERP  deck grating
// L3: APERTURE/RND_RECT    ALL        LERP  viewport frame + region tag
// L4: SCATTER/STARS        SKY        ADD   stars only in the viewport
// L5: CELESTIAL/ECLIPSE    SKY        ADD   eclipse body in the viewport
// L6: DECAL/RECT           WALLS|FLOOR ADD  viewport light card spill
// L7: LOBE                 WALLS|FLOOR ADD  cool spill from viewport (animated)
pub(super) const PRESET_VOID_STATION: [[u64; 2]; 8] = [
    // L0: SECTOR/BOX - room bounds colors
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
    // L1: GRID - panel lines (slightly more visible)
    [
        hi(
            OP_GRID,
            REGION_WALLS | REGION_FLOOR,
            BLEND_ADD,
            0,
            0x3a4858, // brighter panel lines
            0x000000,
        ),
        // scale, thickness, pattern=GRID, slow scroll; phase animates via ANIM_SPEEDS
        lo(22, 110, 16, 0x14, 0, 0, 10, 0),
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
            0xffffff, // bright white stars
            0x88b8ff, // blue-tinted secondary stars
        ),
        // Higher intensity, good density for starfield
        lo(90, 24, 14, 0x50, 10, 0, 12, 6),
    ],
    // L5: CELESTIAL/ECLIPSE - hero celestial body in the viewport
    [
        hi_meta(
            OP_CELESTIAL,
            REGION_SKY,
            BLEND_ADD,
            DOMAIN_DIRECT3D,
            CELESTIAL_ECLIPSE,
            0xd0f0ff, // brighter corona/halo
            0x010206, // very dark body (eclipsed)
        ),
        // Larger size, stronger halo for dramatic eclipse
        lo(220, 85, 180, 0, 0, DIR_BACK, 15, 13),
    ],
    // L6: DECAL/RECT - viewport light card spill (helps reflection readability)
    [
        hi(
            OP_DECAL,
            REGION_WALLS | REGION_FLOOR,
            BLEND_ADD,
            0,
            0x90c8ff, // slightly brighter spill
            0x0c1828,
        ),
        lo(160, 0x28, 165, 210, 0x12, DIR_BACK, 14, 10),
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
