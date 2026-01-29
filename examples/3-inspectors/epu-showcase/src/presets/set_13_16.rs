//! Preset set 13-16

#[allow(unused_imports)]
use crate::constants::*;

// -----------------------------------------------------------------------------
// Preset 13: "Enchanted Grove" - Fairy tale forest
// -----------------------------------------------------------------------------
// Design: Forest clearing with a strong, readable sunbeam.
// Goal: canopy/dapple motion + warm clearing + dust motes; avoid chart seams.
//
// L0: RAMP              ALL        LERP   warm base gradient (sky vs floor)
// L1: SILHOUETTE/FOREST WALLS      LERP   tree line / enclosure (doesn't cut moon/sky)
// L2: PLANE/GRASS        FLOOR     LERP   mossy ground
// L3: FLOW               SKY+WALLS SCREEN moving leaf-dapple drift
// L4: APERTURE/CIRCLE    SKY       LERP   clearing hole in canopy
// L5: LOBE               ALL       ADD    HERO: golden sunbeam
// L6: SCATTER/DUST       ALL       ADD    fairy motes (sparse)
// L7: ATMOSPHERE/MIE     ALL       LERP   warm haze to unify
pub(super) const PRESET_ENCHANTED_GROVE: [[u64; 2]; 8] = [
    // L0: RAMP - warm base gradient (readable sky vs floor)
    [
        hi(OP_RAMP, REGION_ALL, BLEND_LERP, 0, 0x496a34, 0x0b1206),
        lo(240, 0x2a, 0x18, 0x10, THRESH_SEMI, DIR_UP, 15, 15),
    ],
    // L1: SILHOUETTE/FOREST - tree line enclosure (walls only)
    [
        hi_meta(
            OP_SILHOUETTE,
            REGION_WALLS,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            SILHOUETTE_FOREST,
            0x060a03,
            0x1a2a10,
        ),
        lo(190, 150, 190, 0x40, 0, DIR_UP, 15, 0),
    ],
    // L2: PLANE/GRASS - mossy forest floor
    [
        hi_meta(
            OP_PLANE,
            REGION_FLOOR,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            PLANE_GRASS,
            0x223018,
            0x4a5a2a,
        ),
        lo(220, 105, 20, 0, 0, DIR_UP, 15, 15),
    ],
    // L3: FLOW - moving leaf-dapple drift (animation reads in screenshots)
    [
        hi_meta(
            OP_FLOW,
            REGION_SKY | REGION_WALLS,
            BLEND_SCREEN,
            DOMAIN_AXIS_CYL,
            0,
            0x88b86a,
            0x0b1206,
        ),
        lo(115, 205, 22, 0x22, 0, DIR_RIGHT, 11, 6),
    ],
    // L4: APERTURE/CIRCLE - canopy clearing (avoid polygonal irregular pinching)
    [
        hi_meta(
            OP_APERTURE,
            REGION_SKY,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            APERTURE_CIRCLE,
            0x1a2a10,
            0xffedbe,
        ),
        lo(220, 175, 110, 120, 0, DIR_UP, 15, 15),
    ],
    // L5: LOBE - HERO: golden sunbeam cone
    [
        hi(OP_LOBE, REGION_ALL, BLEND_ADD, 0, 0xffd9a6, 0x3a260a),
        lo(175, 235, 55, 1, 0, DIR_SUN, 15, 8),
    ],
    // L6: SCATTER/DUST - sparse fairy motes (no full-screen speckle)
    [
        hi_meta(
            OP_SCATTER,
            REGION_ALL,
            BLEND_ADD,
            DOMAIN_DIRECT3D,
            SCATTER_DUST,
            0xfff0c8,
            0xffc070,
        ),
        lo(55, 28, 14, 0x20, 19, DIR_UP, 10, 0),
    ],
    // L7: ATMOSPHERE/MIE - warm golden forest haze
    [
        hi_meta(
            OP_ATMOSPHERE,
            REGION_ALL,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            ATMO_MIE,
            0xa07038,
            0x0b1206,
        ),
        lo(70, 110, 128, 110, 170, DIR_SUN, 10, 0),
    ],
];

// -----------------------------------------------------------------------------
// Preset 14: "Astral Void" - Cosmic void
// -----------------------------------------------------------------------------
// Design: Immense, wrap-around deep space with a distant eclipse and subtle
// nebula drift. Avoid obvious tiling/macro blobs.
//
// L0: RAMP (sky=#000004, floor=#050010, walls=#0a0220)
// L1: PATCHES/STREAKS (wispy nebula, AXIS_CYL to avoid polar pinching)
// L2: BAND (faint dust lane)
// L3: FLOW (iridescent aurora drift, animated)
// L4: SCATTER/STARS (tasteful starfield)
// L5: CELESTIAL/ECLIPSE (cold corona)
// L6: PORTAL/RIFT (prismatic tear on walls, tangent-local)
// L7: ATMOSPHERE/ABSORPTION (subtle void haze)
pub(super) const PRESET_ASTRAL_VOID: [[u64; 2]; 8] = [
    // L0: RAMP - near-black void with cold indigo walls (immense scale)
    [
        hi(OP_RAMP, REGION_ALL, BLEND_LERP, 0, 0x00040a, 0x01000c),
        lo(245, 0x05, 0x00, 0x1a, THRESH_VAST, DIR_UP, 15, 15),
    ],
    // L1: PATCHES/STREAKS - wispy nebula (avoid chunky macro blobs)
    [
        hi_meta(
            OP_PATCHES,
            REGION_ALL,
            BLEND_SCREEN,
            DOMAIN_AXIS_CYL,
            PATCHES_STREAKS,
            0x6b2aa8,
            0x001020,
        ),
        lo(140, 84, 170, 0x28, 51, DIR_RIGHT, 12, 4),
    ],
    // L2: BAND - distant galactic dust lane
    [
        hi(OP_BAND, REGION_ALL, BLEND_SCREEN, 0, 0x6a5a44, 0x102038),
        lo(70, 80, 128, 220, 0, DIR_SUNSET, 12, 0),
    ],
    // L3: FLOW - slow prismatic drift (animated via ANIM_SPEEDS)
    [
        hi_meta(
            OP_FLOW,
            REGION_ALL,
            BLEND_SCREEN,
            DOMAIN_DIRECT3D,
            0,
            0x2c70b8,
            0x2a0018,
        ),
        lo(75, 190, 20, 0x22, 0, DIR_RIGHT, 10, 6),
    ],
    // L4: SCATTER/STARS - sparse, tasteful starfield (avoid full-screen speckle)
    [
        hi_meta(
            OP_SCATTER,
            REGION_ALL,
            BLEND_ADD,
            DOMAIN_DIRECT3D,
            SCATTER_STARS,
            0xf4f8ff,
            0x7aa0ff,
        ),
        lo(40, 28, 6, 0x14, 3, DIR_UP, 10, 0),
    ],
    // L5: CELESTIAL/ECLIPSE - cold eclipsed star (shows on the back-side too)
    [
        hi_meta(
            OP_CELESTIAL,
            REGION_ALL,
            BLEND_ADD,
            DOMAIN_DIRECT3D,
            CELESTIAL_ECLIPSE,
            0x020008,
            0xb0d8ff,
        ),
        // intensity, angular_size, limb_exp, phase, corona_extent
        lo(180, 140, 200, 0, 140, DIR_SUN, 15, 10),
    ],
    // L6: PORTAL/RIFT - prismatic tear on walls (animated via ANIM_SPEEDS; tangent-local)
    [
        hi_meta(
            OP_PORTAL,
            REGION_WALLS,
            BLEND_SCREEN,
            DOMAIN_TANGENT_LOCAL,
            PORTAL_RIFT,
            0x30e0ff,
            0xff4bb0,
        ),
        // intensity, size, edge_width, roughness, phase
        lo(110, 120, 70, 12, 0x30, DIR_UP, 12, 12),
    ],
    // L7: ATMOSPHERE/ABSORPTION - subtle void haze for depth (keep contrast)
    [
        hi_meta(
            OP_ATMOSPHERE,
            REGION_ALL,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            ATMO_ABSORPTION,
            0x0a0016,
            0x000000,
        ),
        lo(125, 150, 0, 0, 0, DIR_UP, 12, 0),
    ],
];

// -----------------------------------------------------------------------------
// Preset 15: "Toxic Wasteland" - Post-apocalyptic industrial
// -----------------------------------------------------------------------------
// Design: Industrial interior with a single toxic pool as the hero.
// Goal: readable floor + glow spill + a little smoke; avoid full-screen particle noise.
//
// L0: RAMP            ALL          LERP   sickly base palette
// L1: SECTOR/BOX      ALL          LERP   industrial enclosure
// L2: PLANE/PAVEMENT  FLOOR        LERP   concrete/ash floor
// L3: TRACE/CRACKS    FLOOR        ADD    toxic veins (localized)
// L4: PORTAL/VORTEX   FLOOR        ADD    HERO: glowing pool
// L5: LOBE            ALL          ADD    broad radioactive spill
// L6: VEIL/PILLARS    SKY+WALLS    SCREEN rising chemical smoke
// L7: ATMOSPHERE/ALIEN ALL         LERP   green haze (subtle)
pub(super) const PRESET_TOXIC_WASTELAND: [[u64; 2]; 8] = [
    // L0: RAMP - sickly base palette (keeps contrast so the pool reads)
    [
        hi(OP_RAMP, REGION_ALL, BLEND_LERP, 0, 0x141006, 0x030400),
        lo(235, 0x12, 0x08, 0x18, THRESH_INTERIOR, DIR_UP, 15, 15),
    ],
    // L1: SECTOR/BOX - industrial enclosure
    [
        hi_meta(
            OP_SECTOR,
            REGION_ALL,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            SECTOR_BOX,
            0x080807,
            0x181010,
        ),
        lo(225, 150, 0, 0, 0, DIR_UP, 15, 15),
    ],
    // L2: PLANE/PAVEMENT - cracked concrete floor
    [
        hi_meta(
            OP_PLANE,
            REGION_FLOOR,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            PLANE_PAVEMENT,
            0x1a140c,
            0x0c0906,
        ),
        lo(210, 110, 120, 0, 0, DIR_UP, 15, 15),
    ],
    // L3: TRACE/CRACKS - toxic veins radiating from the pool
    [
        hi_meta(
            OP_TRACE,
            REGION_FLOOR,
            BLEND_ADD,
            DOMAIN_TANGENT_LOCAL,
            TRACE_CRACKS,
            0x70ff40,
            0x133008,
        ),
        lo(190, 160, 70, 0x20, 0x10, DIR_UP, 15, 0),
    ],
    // L4: PORTAL/VORTEX - HERO: glowing toxic waste pool
    [
        hi_meta(
            OP_PORTAL,
            REGION_FLOOR,
            BLEND_ADD,
            DOMAIN_TANGENT_LOCAL,
            PORTAL_VORTEX,
            0x50ff2a,
            0xa0ff60,
        ),
        lo(235, 110, 200, 120, 0, DIR_DOWN, 15, 15),
    ],
    // L5: LOBE - radioactive glow spill (kept broad, not blinding)
    [
        hi(OP_LOBE, REGION_ALL, BLEND_ADD, 0, 0x66ff3a, 0x163000),
        lo(170, 220, 70, 1, 0, DIR_UP, 13, 8),
    ],
    // L6: VEIL/PILLARS - rising chemical smoke columns (adds motion without noise)
    [
        hi_meta(
            OP_VEIL,
            REGION_SKY | REGION_WALLS,
            BLEND_SCREEN,
            DOMAIN_AXIS_CYL,
            VEIL_PILLARS,
            0x4cff60,
            0x102008,
        ),
        lo(110, 170, 30, 140, 60, DIR_UP, 10, 4),
    ],
    // L7: ATMOSPHERE/ALIEN - subtle poisonous haze (keeps the pool readable)
    [
        hi_meta(
            OP_ATMOSPHERE,
            REGION_ALL,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            ATMO_ALIEN,
            0x2a6a18,
            0x000000,
        ),
        lo(80, 120, 128, 0, 0, DIR_UP, 10, 0),
    ],
];

// -----------------------------------------------------------------------------
// Preset 16: "Moonlit Graveyard" - Gothic horror
// -----------------------------------------------------------------------------
// Design: Low tombstones on the horizon, drifting ground fog, and a bright moon.
// Fixes:
// - Keep silhouettes on WALLS only (no jagged cut into the sky on the sphere)
// - Avoid AXIS_CYL veil rings (use TANGENT_LOCAL for mist)
//
// L0: RAMP              ALL        LERP   cold night base
// L1: SILHOUETTE/RUINS  WALLS      LERP   tombstones/fence line
// L2: PLANE/STONE       FLOOR      LERP   weathered ground
// L3: PATCHES/MEMBRANE  SKY+WALLS  MULTIPLY cloud ceiling
// L4: CELESTIAL/MOON    SKY        ADD    moon (unobstructed)
// L5: SCATTER/STARS     SKY        ADD    sparse stars
// L6: VEIL/CURTAINS     FLOOR+WALLS SCREEN ground mist (tangent-local)
// L7: ATMOSPHERE/MIE    ALL        LERP   thin fog to unify
pub(super) const PRESET_MOONLIT_GRAVEYARD: [[u64; 2]; 8] = [
    // L0: RAMP - cold night base
    [
        hi(OP_RAMP, REGION_ALL, BLEND_LERP, 0, 0x061022, 0x020208),
        lo(245, 0x14, 0x0c, 0x1a, THRESH_SEMI, DIR_UP, 15, 15),
    ],
    // L1: SILHOUETTE/RUINS - tombstones/fence line (walls only)
    [
        hi_meta(
            OP_SILHOUETTE,
            REGION_WALLS,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            SILHOUETTE_RUINS,
            0x000004,
            0x101828,
        ),
        lo(220, 110, 200, 0x40, 0, DIR_UP, 15, 0),
    ],
    // L2: PLANE/STONE - weathered graveyard ground
    [
        hi_meta(
            OP_PLANE,
            REGION_FLOOR,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            PLANE_STONE,
            0x1a1c22,
            0x0c0e12,
        ),
        lo(190, 120, 150, 0, 0, DIR_UP, 15, 15),
    ],
    // L3: PATCHES/MEMBRANE - low cloud ceiling (keep it subtle)
    [
        hi_meta(
            OP_PATCHES,
            REGION_SKY | REGION_WALLS,
            BLEND_MULTIPLY,
            DOMAIN_AXIS_CYL,
            PATCHES_MEMBRANE,
            0x202840,
            0x05060a,
        ),
        lo(200, 96, 160, 0x28, 43, DIR_RIGHT, 15, 0),
    ],
    // L4: CELESTIAL/MOON - full moon (unobstructed)
    [
        hi_meta(
            OP_CELESTIAL,
            REGION_SKY,
            BLEND_ADD,
            DOMAIN_DIRECT3D,
            CELESTIAL_MOON,
            0xe0e8f0,
            0x000000,
        ),
        lo(230, 190, 0, 0, 0, DIR_SUN, 15, 0),
    ],
    // L5: SCATTER/STARS - stars in sky only (not on ground)
    [
        hi_meta(
            OP_SCATTER,
            REGION_SKY,
            BLEND_ADD,
            DOMAIN_DIRECT3D,
            SCATTER_STARS,
            0xc0c8d0,
            0x000000,
        ),
        lo(55, 42, 7, 0x24, 0, DIR_UP, 10, 0),
    ],
    // L6: VEIL/CURTAINS - ground mist (tangent-local, avoids cylinder rings)
    [
        hi_meta(
            OP_VEIL,
            REGION_FLOOR | REGION_WALLS,
            BLEND_SCREEN,
            DOMAIN_TANGENT_LOCAL,
            VEIL_CURTAINS,
            0x6080a0,
            0x000000,
        ),
        lo(70, 85, 28, 120, 20, DIR_UP, 8, 0),
    ],
    // L7: ATMOSPHERE/MIE - thin fog that keeps silhouettes readable
    [
        hi_meta(
            OP_ATMOSPHERE,
            REGION_ALL,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            ATMO_MIE,
            0x141c2c,
            0x000000,
        ),
        lo(60, 95, 128, 60, 80, DIR_SUN, 8, 0),
    ],
];
