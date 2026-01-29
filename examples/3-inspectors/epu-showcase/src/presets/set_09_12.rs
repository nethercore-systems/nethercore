//! Preset set 09-12

#[allow(unused_imports)]
use crate::constants::*;

// -----------------------------------------------------------------------------
// Preset 9: "Neon Arcade" — Gamer room / RGB den
// -----------------------------------------------------------------------------
// Goal: unmistakably an interior: dark room shell, a big front monitor, RGB strips,
// and a couple of tiny practical LEDs. Keep domains mostly DIRECT3D to avoid seams.
//
// L0: RAMP                 base room palette (interior thresholds)
// L1: SECTOR/BOX           room enclosure
// L2: PLANE/GRATING        desk/floor surface
// L3: APERTURE/ROUNDED     main monitor glow + bezel
// L4: VEIL/LASER_BARS      RGB strip lights (animated)
// L5: DECAL                neon sign / accent (animated blink)
// L6: LOBE                 monitor spill (directional)
// L7: SCATTER/DUST         faint dust motes
pub(super) const PRESET_NEON_ARCADE: [[u64; 2]; 8] = [
    // L0: RAMP - dark room base (keep walls dominant)
    [
        hi(OP_RAMP, REGION_ALL, BLEND_LERP, 0, 0x06060a, 0x120018),
        lo(228, 0x18, 0x10, 0x08, THRESH_INTERIOR, DIR_UP, 15, 15),
    ],
    // L1: SECTOR/BOX - room enclosure (instant "interior" read)
    [
        hi_meta(
            OP_SECTOR,
            REGION_ALL,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            SECTOR_BOX,
            0x1a1022,
            0x05040a,
        ),
        // Slightly softer enclosure to avoid band/ring read on the sphere.
        lo(200, 118, 0, 0, 0, DIR_UP, 15, 15),
    ],
    // L2: PLANE/GRATING - desk/floor texture (subtle, non-grid-cyber)
    [
        hi_meta(
            OP_PLANE,
            REGION_FLOOR,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            PLANE_GRATING,
            0x0b0b10,
            0x181622,
        ),
        // Give the grating a slow scroll so it reads like a real surface.
        lo(200, 120, 160, 15, 0x40, DIR_UP, 15, 15),
    ],
    // L3: APERTURE/ROUNDED_RECT - main monitor (bezel + glow)
    [
        hi_meta(
            OP_APERTURE,
            REGION_WALLS,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            APERTURE_ROUNDED_RECT,
            0x020205,
            0x6cf6ff,
        ),
        // intensity, softness, half_w, half_h, frame_thickness
        // Widescreen proportions to avoid the "eye"/ring read.
        lo(240, 55, 170, 108, 26, DIR_FORWARD, 15, 15),
    ],
    // L4: VEIL/LASER_BARS - RGB strip lights along walls/ceiling (slow sweep)
    [
        hi_meta(
            OP_VEIL,
            REGION_WALLS | REGION_SKY,
            BLEND_ADD,
            // Localize the strips to kill wrap-around ring artifacts.
            DOMAIN_TANGENT_LOCAL,
            VEIL_LASER_BARS,
            0xff2bd6,
            0x00e5ff,
        ),
        lo(120, 155, 38, 165, 0, DIR_UP, 12, 8),
    ],
    // L5: DECAL - neon sign/accent (blink)
    [
        hi(OP_DECAL, REGION_WALLS, BLEND_ADD, 0, 0xff2bd6, 0x00e5ff),
        // shape=RECT(2), soft=4, size=64, glow_soft=170; param_d is phase
        lo(205, 0x24, 64, 170, 0, DIR_LEFT, 14, 10),
    ],
    // L6: LOBE - monitor spill (sells the room lighting)
    [
        hi(OP_LOBE, REGION_ALL, BLEND_ADD, 0, 0x6cf6ff, 0xff2bd6),
        // Slightly stronger + a gentle "breath" via phase.
        lo(130, 215, 55, 1, 0x30, DIR_FORWARD, 13, 9),
    ],
    // L7: SCATTER/DUST - faint dust motes (depth, not glitter)
    [
        hi_meta(
            OP_SCATTER,
            REGION_ALL,
            BLEND_ADD,
            DOMAIN_DIRECT3D,
            SCATTER_DUST,
            0xffffff,
            0x00e5ff,
        ),
        // Much sparser: avoid "outer space" starfield read.
        lo(10, 14, 9, 0x24, 17, DIR_FORWARD, 8, 0),
    ],
];

// -----------------------------------------------------------------------------
// Preset 10: "Storm Front" — Dramatic thunderstorm
// -----------------------------------------------------------------------------
// Goal: heavy low cloud ceiling, visible rain motion, and lightning that
// actually lights the "wet" ground.
//
// L0: RAMP              base cold storm palette (low ceiling)
// L1: PATCHES/MEMBRANE  big cloud masses (DIRECT3D; avoid seams)
// L2: FLOW              wind shear / cloud crawl (DIRECT3D)
// L3: VEIL/RAIN_WALL     driving rain (DIRECT3D; downward motion)
// L4: PLANE/PAVEMENT     wet ground
// L5: TRACE/LIGHTNING    branching lightning channels (AXIS_POLAR)
// L6: DECAL              broad lightning flash (flicker)
// L7: ATMOSPHERE/MIE     dense storm haze
pub(super) const PRESET_STORM_FRONT: [[u64; 2]; 8] = [
    // L0: RAMP - cold storm base (deep sky, near-black wet ground)
    [
        hi(OP_RAMP, REGION_ALL, BLEND_LERP, 0, 0x0b1018, 0x050608),
        lo(240, 0x20, 0x14, 0x26, THRESH_SEMI, DIR_UP, 15, 15),
    ],
    // L1: PATCHES/MEMBRANE - heavy low cloud ceiling (big shapes)
    [
        hi_meta(
            OP_PATCHES,
            REGION_SKY | REGION_WALLS,
            BLEND_MULTIPLY,
            DOMAIN_DIRECT3D,
            PATCHES_MEMBRANE,
            0x3b4652,
            0x0a0c10,
        ),
        lo(235, 105, 175, 0x40, 27, DIR_UP, 15, 0),
    ],
    // L2: FLOW - wind shear / cloud crawl (DIRECT3D, subtle)
    [
        hi(
            OP_FLOW,
            REGION_SKY | REGION_WALLS,
            BLEND_SCREEN,
            0,
            0x53667a,
            0x0a0c10,
        ),
        // Animate slowly (param_d is phase).
        lo(85, 160, 28, 0x22, 0x10, DIR_RIGHT, 11, 0),
    ],
    // L3: VEIL/RAIN_WALL - driving rain (even coverage, physically downward)
    [
        hi_meta(
            OP_VEIL,
            REGION_ALL,
            BLEND_SCREEN,
            DOMAIN_DIRECT3D,
            VEIL_RAIN_WALL,
            0xe6f1ff,
            0x263544,
        ),
        // Reduce streak thickness + avoid chunky "bars".
        lo(150, 205, 26, 150, 0x20, DIR_DOWN, 12, 8),
    ],
    // L4: PLANE/PAVEMENT - wet ground to catch flashes
    [
        hi_meta(
            OP_PLANE,
            REGION_FLOOR,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            PLANE_PAVEMENT,
            0x0b0d10,
            0x1a2430,
        ),
        // Subtle moving sheen so flashes feel like they travel across water.
        lo(210, 120, 160, 25, 0x30, DIR_UP, 15, 15),
    ],
    // L5: TRACE/LIGHTNING - faint branching channels (no obvious seams)
    [
        hi_meta(
            OP_TRACE,
            REGION_SKY,
            BLEND_ADD,
            // Tangent-local avoids polar "ring" segmentation.
            DOMAIN_TANGENT_LOCAL,
            TRACE_LIGHTNING,
            0xffffff,
            0x9ad7ff,
        ),
        lo(190, 210, 86, 200, 0x52, DIR_UP, 15, 12),
    ],
    // L6: LOBE - broad lightning flash (actually lights the sphere)
    [
        hi(OP_LOBE, REGION_ALL, BLEND_ADD, 0, 0xffffff, 0x8fd0ff),
        // Strong, wide, slightly colored flash; param_d is phase (flicker via ANIM_SPEEDS).
        lo(235, 205, 38, 1, 0x00, DIR_UP, 15, 12),
    ],
    // L7: ATMOSPHERE/MIE - dense storm haze
    [
        hi_meta(
            OP_ATMOSPHERE,
            REGION_ALL,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            ATMO_MIE,
            0x28323e,
            0x06080b,
        ),
        // Back off a touch so rain + flashes stay readable.
        lo(80, 110, 128, 175, 140, DIR_UP, 10, 0),
    ],
];

// -----------------------------------------------------------------------------
// Preset 11: "Crystal Cavern" — Fantasy underground geode
// -----------------------------------------------------------------------------
// Goal: big readable crystal facets + glowing veins + a floor rift. Avoid obvious
// cylindrical banding by keeping the "veins" in DIRECT3D/TANGENT_LOCAL at low gain.
//
// L0: RAMP               base cavern palette (interior thresholds)
// L1: SECTOR/CAVE        enclosure
// L2: CELL/SHATTER       faceted crystal walls
// L3: TRACE/FILAMENTS    glowing veins (subtle)
// L4: SCATTER/DUST       sparkle glints
// L5: PORTAL/VORTEX      floor rift
// L6: LOBE               underglow / bounce
// L7: ATMOSPHERE/RAYLEIGH colored mist
pub(super) const PRESET_CRYSTAL_CAVERN: [[u64; 2]; 8] = [
    // L0: RAMP - base bounce light (no pure-black void; reflections always have something)
    [
        hi(OP_RAMP, REGION_ALL, BLEND_LERP, 0, 0x14082a, 0x05102a),
        lo(238, 0x18, 0x08, 0x24, THRESH_INTERIOR, DIR_UP, 15, 15),
    ],
    // L1: SECTOR/CAVE - geode chamber enclosure (organic, not boxy)
    [
        hi_meta(
            OP_SECTOR,
            REGION_ALL,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            SECTOR_CAVE,
            0x1b0a3a,
            0x060616,
        ),
        lo(228, 122, 12, 0, 0, DIR_UP, 15, 15),
    ],
    // L2: CELL/SHATTER - big faceted crystal plates on walls (large forms, low visual noise)
    [
        hi_meta(
            OP_CELL,
            REGION_WALLS,
            BLEND_OVERLAY,
            DOMAIN_DIRECT3D,
            CELL_SHATTER,
            0x4a1b8a,
            0x080220,
        ),
        lo(190, 70, 228, 16, 9, DIR_UP, 12, 10),
    ],
    // L3: TRACE/FILAMENTS - glowing veins (keep subtle to avoid chart seams)
    [
        hi_meta(
            OP_TRACE,
            REGION_WALLS,
            BLEND_ADD,
            DOMAIN_TANGENT_LOCAL,
            TRACE_FILAMENTS,
            0x6ffff7,
            0xff7cf6,
        ),
        lo(120, 178, 74, 150, 0x30, DIR_UP, 12, 6),
    ],
    // L4: SCATTER/DUST - sparse sparkle highlights (glints, not snow)
    [
        hi_meta(
            OP_SCATTER,
            REGION_WALLS | REGION_FLOOR,
            BLEND_ADD,
            DOMAIN_DIRECT3D,
            SCATTER_DUST,
            0xffffff,
            0xaad5ff,
        ),
        lo(12, 12, 8, 0x20, 23, DIR_UP, 4, 0),
    ],
    // L5: PORTAL/RIFT - jagged floor tear (avoid perfect ring)
    [
        hi_meta(
            OP_PORTAL,
            REGION_FLOOR,
            BLEND_ADD,
            DOMAIN_TANGENT_LOCAL,
            PORTAL_RIFT,
            0x7a3dff,
            0x55f3ff,
        ),
        // intensity, size, edge_width, roughness, phase
        lo(225, 104, 140, 48, 0x20, DIR_DOWN, 0, 15),
    ],
    // L6: LOBE - soft underglow to sell depth + reflections (slow magical breath)
    [
        hi(OP_LOBE, REGION_ALL, BLEND_ADD, 0, 0x55e9ff, 0x2b0b58),
        lo(98, 228, 65, 1, 0x10, DIR_DOWN, 12, 0),
    ],
    // L7: ATMOSPHERE/RAYLEIGH - soft colored haze for bounce + depth
    [
        hi_meta(
            OP_ATMOSPHERE,
            REGION_ALL,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            ATMO_RAYLEIGH,
            0x1a0b40,
            0x041226,
        ),
        lo(75, 95, 150, 120, 0, DIR_UP, 10, 0),
    ],
];

// -----------------------------------------------------------------------------
// Preset 12: "War Zone" — Military/apocalyptic battlefield
// -----------------------------------------------------------------------------
// Design: blasted street canyon: concrete walls, wet/asphalt ground, smoke layers,
// active firelight, and sparse embers rising. Keep particles controlled so the
// reflection sphere reads "place" instead of "orange confetti".
//
// L0: RAMP              dusty smoke palette
// L1: SECTOR/TUNNEL      street canyon enclosure
// L2: PLANE/PAVEMENT     broken road surface
// L3: PATCHES/DEBRIS     smoke plumes + soot clouds
// L4: VEIL/SHARDS        tracer/shrapnel streaks (subtle motion)
// L5: DECAL              localized fires (flicker)
// L6: SCATTER/EMBERS      sparse rising embers
// L7: ATMOSPHERE/ABSORPTION thick brown-gray smoke
pub(super) const PRESET_WAR_ZONE: [[u64; 2]; 8] = [
    // L0: RAMP - dusty smoke base (keep contrast; no flat gray wash)
    [
        hi(OP_RAMP, REGION_ALL, BLEND_LERP, 0, 0x201810, 0x060608),
        lo(230, 0x24, 0x18, 0x10, THRESH_SEMI, DIR_UP, 15, 15),
    ],
    // L1: SECTOR/TUNNEL - street canyon enclosure (walls read as concrete)
    [
        hi_meta(
            OP_SECTOR,
            REGION_ALL,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            SECTOR_TUNNEL,
            0x2a2a28,
            0x0b0a0a,
        ),
        // Reduce hard banding from the tunnel gradient.
        lo(195, 102, 0, 0, 0, DIR_UP, 15, 15),
    ],
    // L2: PLANE/PAVEMENT - broken road / wet ash
    [
        hi_meta(
            OP_PLANE,
            REGION_FLOOR,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            PLANE_PAVEMENT,
            0x121316,
            0x2a2b2e,
        ),
        lo(205, 110, 170, 20, 0, DIR_UP, 15, 15),
    ],
    // L3: FLOW - smoke drift (slow, physical billow)
    [
        hi(
            OP_FLOW,
            REGION_SKY | REGION_WALLS,
            BLEND_MULTIPLY,
            0,
            0x3a342e,
            0x0a0a0b,
        ),
        lo(130, 150, 40, 0x34, 0x10, DIR_UP, 15, 0),
    ],
    // L4: TRACE/LEAD_LINES - tracer streaks (avoid big veil slabs)
    [
        hi_meta(
            OP_TRACE,
            REGION_SKY | REGION_WALLS,
            BLEND_ADD,
            DOMAIN_AXIS_CYL,
            TRACE_LEAD_LINES,
            0xffd39a,
            0x2a2b2e,
        ),
        lo(115, 190, 92, 170, 0x28, DIR_RIGHT, 12, 8),
    ],
    // L5: DECAL - localized fires (flicker)
    [
        hi(
            OP_DECAL,
            REGION_WALLS | REGION_FLOOR,
            BLEND_ADD,
            0,
            0xff5a12,
            0x2a0600,
        ),
        // shape=DISK(0), soft=10, size=110, glow_soft=190; param_d phase
        lo(235, 0x0A, 104, 170, 0x20, DIR_LEFT, 15, 12),
    ],
    // L6: SCATTER/EMBERS - sparse rising embers (upward drift)
    [
        hi_meta(
            OP_SCATTER,
            REGION_ALL,
            BLEND_ADD,
            DOMAIN_DIRECT3D,
            SCATTER_EMBERS,
            0xff8a22,
            0x000000,
        ),
        // Fewer, larger embers so it reads like ash in air, not confetti.
        lo(35, 10, 14, 0x20, 0x18, DIR_UP, 8, 0),
    ],
    // L7: ATMOSPHERE/ABSORPTION - thick war smoke (brown-gray)
    [
        hi_meta(
            OP_ATMOSPHERE,
            REGION_ALL,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            ATMO_ABSORPTION,
            0x3a2f26,
            0x000000,
        ),
        lo(155, 95, 0, 0, 0, DIR_UP, 15, 0),
    ],
];
