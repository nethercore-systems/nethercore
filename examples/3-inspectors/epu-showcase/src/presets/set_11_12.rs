//! Preset set 11-12

#[allow(unused_imports)]
use crate::constants::*;

// -----------------------------------------------------------------------------
// Preset 11: "Frozen Tundra" - Wind-cut arctic expanse
// -----------------------------------------------------------------------------
// Goal: exposed arctic survival scene with a hard glacier ridge, pale sky,
// a planar polished-ice floor, low polar light, one obvious spindrift mover,
// and restrained snow support.
//
// Cadence: BOUNDS (ridge + narrow far-field seam) -> FLOOR (ice) -> WEATHER (spindrift mover) ->
// AIR (cold depth) -> ACCENTS (sheen + sky breakup + grounded breakup)
//
// L0: SILHOUETTE/MOUNTAINS SKY        LERP      one clear glacier ridge instead of a flat shelf
// L1: SPLIT/TIER          ALL         LERP      a narrow dark far-field seam so the scene stays open instead of collapsing into a full chamber band
// L2: SURFACE/GLAZE       FLOOR       LERP      broad frozen sheen so the floor stops reading as water
// L3: ADVECT/SPINDRIFT    SKY         SCREEN    one obvious moving spindrift field in direct view without turning the side walls into a chamber
// L4: ATMOSPHERE/RAYLEIGH SKY|WALLS   SCREEN    crisp cold aerial perspective
// L5: MOTTLE/SOFT         SKY         MULTIPLY  broad cold cloud breakup so the sky is not a flat pale shelf
// L6: SURFACE/CRUST       FLOOR       MULTIPLY  broken frost plates over the glazed bed
// L7: MOTTLE/GRAIN        FLOOR       MULTIPLY  subordinate grounded breakup so the lower scene stops reading as one pale flat slab
pub(super) const PRESET_FROZEN_TUNDRA: [[u64; 2]; 8] = [
    // L0: SILHOUETTE/MOUNTAINS - drop the ridge slightly and sharpen it so the scene keeps a real horizon under open sky
    [
        hi_meta(
            OP_SILHOUETTE,
            REGION_SKY,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            SILHOUETTE_MOUNTAINS,
            0x728695,
            0xd6e0e6,
        ),
        lo(124, 74, 156, 0x60, 0, DIR_UP, 15, 15),
    ],
    // L1: SPLIT/TIER - narrow the far-field seam and drop it lower so the scene keeps a large open floor instead of a giant mid-band
    [
        hi_meta(
            OP_SPLIT,
            REGION_ALL,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            SPLIT_TIER,
            0xb9c8d2,
            0x32414e,
        ),
        lo(20, 78, 28, 96, 0, DIR_UP, 15, 0),
    ],
    // L2: SURFACE/GLAZE - deepen the exposed ice bed now that the far-field seam is sharper, so the lower scene holds a real ground read
    [
        hi(
            OP_SURFACE,
            REGION_FLOOR,
            BLEND_LERP,
            SURFACE_GLAZE,
            0xcbd7df,
            0x3d4f5f,
        ),
        lo(168, 68, 146, 42, 0, DIR_UP, 13, 0),
    ],
    // L3: ADVECT/SPINDRIFT - keep the mover in the sky belt so the scene gets motion without painting the side walls into an enclosed room
    [
        hi_meta(
            OP_ADVECT,
            REGION_SKY,
            BLEND_SCREEN,
            DOMAIN_DIRECT3D,
            ADVECT_SPINDRIFT,
            0xf5fbff,
            0xb7c9d6,
        ),
        lo(196, 44, 88, 144, 0, DIR_RIGHT, 12, 0),
    ],
    // L4: ATMOSPHERE/RAYLEIGH - keep only a trace of cold depth so the structural/floor read stays intact
    [
        hi_meta(
            OP_ATMOSPHERE,
            REGION_SKY | REGION_WALLS,
            BLEND_SCREEN,
            DOMAIN_DIRECT3D,
            ATMO_RAYLEIGH,
            0x617a94,
            0xcfdce4,
        ),
        lo(4, 64, 88, 0, 0, DIR_UP, 2, 0),
    ],
    // L5: MOTTLE/SOFT - keep more visible cold cloud breakup so the sky stops reading as a blank white roof
    [
        hi_meta(
            OP_MOTTLE,
            REGION_SKY,
            BLEND_MULTIPLY,
            DOMAIN_DIRECT3D,
            MOTTLE_SOFT,
            0xb6c6d1,
            0x485b6d,
        ),
        lo(208, 54, 168, 54, 14, DIR_LEFT, 13, 0),
    ],
    // L6: SURFACE/CRUST - push the crust darker and slightly stronger so the floor does not fall back to one uniform pale sheet
    [
        hi(
            OP_SURFACE,
            REGION_FLOOR,
            BLEND_MULTIPLY,
            SURFACE_CRUST,
            0xb4c3cd,
            0x31404c,
        ),
        lo(136, 46, 220, 14, 18, DIR_UP, 10, 0),
    ],
    // L7: MOTTLE/GRAIN - move the breakup fully onto the floor so the ground stays textured without surrounding the scene in hard wall facets
    [
        hi_meta(
            OP_MOTTLE,
            REGION_FLOOR,
            BLEND_MULTIPLY,
            DOMAIN_DIRECT3D,
            MOTTLE_GRAIN,
            0xbac8d2,
            0x42515f,
        ),
        lo(188, 42, 184, 128, 10, DIR_RIGHT, 11, 0),
    ],
];

// -----------------------------------------------------------------------------
// Preset 12: "Storm Front" - Squall line over black water
// -----------------------------------------------------------------------------
// Goal: violent weather over black water with a readable sea horizon, a dark
// storm shelf, a moving squall curtain, and lightning that reads in the world.
//
// Cadence: BOUNDS (sea / shelf) -> FLOOR (water) -> LIGHT (flash / shelf) ->
// WEATHER (lightning / squall / haze)
//
// L0: SPLIT/TIER           ALL        LERP   clear shelf + horizon organizer before weather layers
// L1: PLANE/WATER          FLOOR      LERP   main moving black-water read
// L2: MASS/SHELF           SKY|WALLS  MULTIPLY main dominant storm shelf/body in direct view
// L3: MOTTLE/RIDGE         WALLS      OVERLAY darker internal storm ridges so the body does not collapse into one flat slab
// L4: TRACE/LIGHTNING      SKY|WALLS  ADD    static strike silhouette in a planar world chart
// L5: FLOW                 FLOOR      SCREEN moving water breakup under the storm shelf
// L6: ADVECT/FRONT         SKY|WALLS  SCREEN subordinate moving transport inside the main storm body
// L7: ATMOSPHERE/FULL      SKY|WALLS  SCREEN minimal cold storm haze
pub(super) const PRESET_STORM_FRONT: [[u64; 2]; 8] = [
    // L0: SPLIT/TIER - stepped shelf split creates a dark storm face over a separate water field
    [
        hi_meta(
            OP_SPLIT,
            REGION_ALL,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            SPLIT_TIER,
            0x8a96a3,
            0x18222c,
        ),
        lo(0, 24, 132, 62, 116, DIR_UP, 15, 15),
    ],
    // L1: PLANE/WATER - black water is the primary floor read; the shader's specular term carries the silver motion
    [
        hi_meta(
            OP_PLANE,
            REGION_FLOOR,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            PLANE_WATER,
            0x2a3d4b,
            0x091017,
        ),
        lo(255, 54, 0, 0, 0, DIR_UP, 15, 0),
    ],
    // L2: MASS/SHELF - let one explicit body carrier own the storm shelf instead of asking texture breakup to fake the front
    [
        hi_meta(
            OP_MASS,
            REGION_SKY | REGION_WALLS,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            MASS_SHELF,
            0x8798a4,
            0x080d12,
        ),
        lo(248, 104, 212, 82, 0, DIR_LEFT, 15, 0),
    ],
    // L3: MOTTLE/RIDGE - keep a little internal structure so the mass reads as weather volume, not a perfectly smooth wall
    [
        hi_meta(
            OP_MOTTLE,
            REGION_WALLS,
            BLEND_OVERLAY,
            DOMAIN_DIRECT3D,
            MOTTLE_RIDGE,
            0x9eb0bc,
            0x1c2933,
        ),
        lo(60, 54, 172, 88, 18, DIR_LEFT, 5, 0),
    ],
    // L4: TRACE/LIGHTNING - keep the strike as a static event silhouette, but localize it instead of painting giant dome arcs
    [
        hi_meta(
            OP_TRACE,
            REGION_SKY | REGION_WALLS,
            BLEND_ADD,
            DOMAIN_TANGENT_LOCAL,
            TRACE_LIGHTNING,
            0xffffff,
            0xa9d2ff,
        ),
        lo(236, 16, 28, 144, 0x74, DIR_FORWARD, 15, 10),
    ],
    // L5: FLOW - give the water bed direct-view variation and motion so it does not read as a single dark slab
    [
        hi(
            OP_FLOW,
            REGION_FLOOR,
            BLEND_SCREEN,
            0,
            0xc3d2de,
            0x4b5a68,
        ),
        lo(88, 18, 20, 0x21, 0, DIR_RIGHT, 6, 0),
    ],
    // L6: ADVECT/FRONT - keep transport subordinate under MASS so the body moves without losing the recovered shelf read
    [
        hi_meta(
            OP_ADVECT,
            REGION_SKY | REGION_WALLS,
            BLEND_SCREEN,
            DOMAIN_DIRECT3D,
            ADVECT_FRONT,
            0xd2dee6,
            0x4b5c68,
        ),
        lo(176, 56, 132, 96, 0, DIR_LEFT, 10, 0),
    ],
    // L7: ATMOSPHERE/FULL - keep the haze in the sky belt so it stops washing the wall-attached storm body
    [
        hi_meta(
            OP_ATMOSPHERE,
            REGION_SKY,
            BLEND_SCREEN,
            DOMAIN_DIRECT3D,
            ATMO_FULL,
            0x495562,
            0x101820,
        ),
        lo(10, 62, 116, 18, 88, DIR_FORWARD, 3, 0),
    ],
];
