//! Preset set 03-04

#[allow(unused_imports)]
use crate::constants::*;

// -----------------------------------------------------------------------------
// Preset 3: "Ocean Depths" - Deep sea trench
// -----------------------------------------------------------------------------
// Visual: BOUNDLESS deep water column - RAMP creates infinite depth gradient.
// Dark overall with bright surface glow above fading to abyssal black below.
// Feature layers add brighter surface caustics, readable god-rays, and a more
// direct abyssal bioluminescent accent while keeping the seabed legible.
// L0: RAMP                 ALL            LERP   deep water column gradient (dark base)
// L1: PLANE/STONE          FLOOR          LERP   darker basalt trench floor
// L2: FLOW (caustic)       SKY            ADD    caustic shimmer drifting through upper water (animated)
// L3: VEIL/PILLARS         ALL            ADD    readable god-rays / depth shafts
// L4: LOBE                 SKY            ADD    bright surface glow above
// L5: PORTAL/VORTEX        SKY            ADD    deeper bioluminescent vent glow (animated)
// L6: SCATTER/DUST         ALL            ADD    restrained marine snow particles
// L7: SCATTER/BUBBLES      ALL            ADD    brighter rising bubbles
pub(super) const PRESET_OCEAN_DEPTHS: [[u64; 2]; 8] = [
    // L0: RAMP - deep water column gradient (boundless depth)
    [
        hi(OP_RAMP, REGION_ALL, BLEND_LERP, 0, 0x0a3040, 0x010408),
        // Dark teal surface fading to near-black abyss
        lo(255, 0x08, 0x18, 0x28, THRESH_VAST, DIR_UP, 15, 15),
    ],
    // L1: PLANE/STONE - darker basalt trench floor with bigger, more readable slabs
    [
        hi_meta(
            OP_PLANE,
            REGION_FLOOR,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            PLANE_STONE,
            0x142a33,
            0x04090d,
        ),
        lo(236, 96, 92, 196, 0, DIR_UP, 15, 14),
    ],
    // L2: FLOW - brighter surface caustics without filling the whole frame with teal fog
    [
        hi(OP_FLOW, REGION_SKY, BLEND_ADD, 0, 0x74d1d8, 0x1d4757),
        lo(136, 92, 150, 0x1c, 15, DIR_DOWN, 10, 1),
    ],
    // L3: VEIL/PILLARS - readable shafts of overhead water light to re-establish depth
    [
        hi_meta(
            OP_VEIL,
            REGION_ALL,
            BLEND_ADD,
            DOMAIN_AXIS_CYL,
            VEIL_PILLARS,
            0x2d6a74,
            0x0a1824,
        ),
        lo(96, 52, 88, 44, 5, DIR_DOWN, 10, 1),
    ],
    // L4: LOBE - stronger surface glow to hold the top-to-bottom depth gradient
    [
        hi(OP_LOBE, REGION_SKY, BLEND_ADD, 0, 0x88d7dc, 0x163847),
        lo(180, 184, 92, 0, 0, DIR_UP, 11, 0),
    ],
    // L5: PORTAL/VORTEX - brighter abyssal glow that stays underwater instead of reading as central debris
    [
        hi_meta(
            OP_PORTAL,
            REGION_SKY,
            BLEND_ADD,
            DOMAIN_TANGENT_LOCAL,
            PORTAL_VORTEX,
            0x164146,
            0x8fffe8,
        ),
        lo(148, 132, 148, 156, 8, DIR_UP, 10, 1),
    ],
    // L6: SCATTER/DUST - marine snow stays present but no longer overwhelms the trench structure
    [
        hi_meta(
            OP_SCATTER,
            REGION_ALL,
            BLEND_ADD,
            DOMAIN_DIRECT3D,
            SCATTER_DUST,
            0x86a6b3,
            0x234253,
        ),
        lo(24, 14, 12, 0x10, 22, DIR_DOWN, 5, 1),
    ],
    // L7: SCATTER/BUBBLES - fewer but brighter bubbles to read as underwater support, not static grain
    [
        hi_meta(
            OP_SCATTER,
            REGION_ALL,
            BLEND_ADD,
            DOMAIN_DIRECT3D,
            SCATTER_BUBBLES,
            0x88c8d6,
            0x1d3644,
        ),
        lo(42, 10, 12, 0x20, 18, DIR_UP, 7, 1),
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
