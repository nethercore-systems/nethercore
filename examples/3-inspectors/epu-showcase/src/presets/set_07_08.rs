//! Preset set 07-08

#[allow(unused_imports)]
use crate::constants::*;

// -----------------------------------------------------------------------------
// Preset 7: "Astral Void" - Cosmic void
// -----------------------------------------------------------------------------
// Visual: BOUNDLESS infinite cosmic void - RAMP is perfect for endless space.
// Near-black with subtle purple-indigo gradient suggesting infinite depth.
// Feature layers add nebula gas, prismatic drift, stars, and celestial bodies.
pub(super) const PRESET_ASTRAL_VOID: [[u64; 2]; 8] = [
    // L0: RAMP - VERY dark void (near black, mysterious)
    [
        hi(OP_RAMP, REGION_ALL, BLEND_LERP, 0, 0x020108, 0x000001),
        // Almost pure black with barely visible purple hint
        lo(255, 0x01, 0x00, 0x02, THRESH_VAST, DIR_UP, 15, 15),
    ],
    // L1: SCATTER/STARS - dense starfield
    [
        hi_meta(
            OP_SCATTER,
            REGION_ALL,
            BLEND_ADD,
            DOMAIN_DIRECT3D,
            SCATTER_STARS,
            0xffffff, // bright white stars
            0x8090c0, // blue-tinted secondary
        ),
        lo(180, 22, 15, 0x58, 6, 0, 14, 8),
    ],
    // L2: FLOW/NOISE - very subtle dark purple nebula wisp
    [
        hi_meta(
            OP_FLOW,
            REGION_ALL,
            BLEND_ADD,
            DOMAIN_DIRECT3D,
            0,
            0x180830, // very dark purple
            0x0c0418, // near black violet
        ),
        lo(40, 180, 40, 0x38, 0, DIR_RIGHT, 6, 2),
    ],
    // L3: CELESTIAL/ECLIPSE - large eclipsed body (upper-left sky)
    [
        hi_meta(
            OP_CELESTIAL,
            REGION_SKY,
            BLEND_ADD,
            DOMAIN_DIRECT3D,
            CELESTIAL_ECLIPSE,
            0x304080, // cold blue corona
            0x010204, // near-black body
        ),
        // DIR_SUNSET = upper left area of sky
        lo(180, 60, 150, 0, 0, DIR_SUNSET, 13, 11),
    ],
    // L4: CELESTIAL/MOON - smaller moon (opposite side - lower right)
    [
        hi_meta(
            OP_CELESTIAL,
            REGION_SKY,
            BLEND_ADD,
            DOMAIN_DIRECT3D,
            CELESTIAL_MOON,
            0x202838, // dim bluish surface
            0x101018, // dark shadow
        ),
        // DIR_FORWARD = opposite side of sky from DIR_SUNSET
        lo(120, 35, 120, 60, 0, DIR_FORWARD, 10, 8),
    ],
    // L5: BAND - faint galactic plane
    [
        hi(OP_BAND, REGION_ALL, BLEND_ADD, 0, 0x100818, 0x080410),
        lo(30, 80, 60, 150, 0, DIR_FORWARD, 6, 3),
    ],
    // L6: FLOW - very subtle teal wisp (counterpoint)
    [
        hi(OP_FLOW, REGION_ALL, BLEND_ADD, 0, 0x081820, 0x040810),
        lo(25, 200, 30, 0x30, 0, DIR_LEFT, 4, 2),
    ],
    // L7: ATMOSPHERE/ABSORPTION - void darkening
    [
        hi_meta(
            OP_ATMOSPHERE,
            REGION_ALL,
            BLEND_MULTIPLY,
            DOMAIN_DIRECT3D,
            ATMO_ABSORPTION,
            0x040408, // very dark
            0x020204,
        ),
        lo(30, 120, 100, 0, 0, DIR_UP, 6, 0),
    ],
];

// -----------------------------------------------------------------------------
// Preset 8: "Hell Core" — THE HEART OF HELL
// -----------------------------------------------------------------------------
// Visual: Fractured hellscape with molten lava cracks. Dangerous but NOT seizure-
// inducing. Static shattered foundation with glowing cracks - minimal animation.
// Sparse large embers rise slowly. Ominous, not overwhelming.
pub(super) const PRESET_VOLCANIC_CORE: [[u64; 2]; 8] = [
    // L0: CELL/SHATTER - fractured reality, broken hellscape foundation
    [
        hi_meta(
            OP_CELL,
            REGION_ALL,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            CELL_SHATTER,
            0x040000, // near-black char
            0x100400, // dark ember glow at edges
        ),
        // Large shatter fragments - STATIC (alpha_a=15, alpha_b=0)
        lo(255, 50, 90, 0x30, 0, DIR_UP, 15, 0),
    ],
    // L1: TRACE/CRACKS - PRIMARY lava fissures (thick, bright)
    [
        hi_meta(
            OP_TRACE,
            REGION_ALL,
            BLEND_ADD,
            DOMAIN_DIRECT3D,
            TRACE_CRACKS,
            0xff5000, // hot orange lava core
            0xa01800, // deep red edges
        ),
        // Thick cracks, STATIC (alpha_b=0)
        lo(180, 160, 25, 120, 0x60, DIR_UP, 14, 0),
    ],
    // L2: PORTAL/RIFT - hellgate maw below (static glow)
    [
        hi_meta(
            OP_PORTAL,
            REGION_FLOOR,
            BLEND_ADD,
            DOMAIN_TANGENT_LOCAL,
            PORTAL_RIFT,
            0xa02800, // orange core (less intense)
            0x200400, // dark bloody rim
        ),
        // STATIC rift (alpha_b=0)
        lo(120, 180, 160, 180, 0, DIR_DOWN, 10, 0),
    ],
    // L3: LOBE - infernal glow from below (reduced, STATIC)
    [
        hi(OP_LOBE, REGION_ALL, BLEND_ADD, 0, 0xa01800, 0x100200),
        // STATIC glow (alpha_b=0)
        lo(100, 180, 70, 0, 0, DIR_DOWN, 10, 0),
    ],
    // L4: PLANE/STONE - dark volcanic rock floor texture
    [
        hi_meta(
            OP_PLANE,
            REGION_FLOOR,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            PLANE_STONE,
            0x100804, // dark volcanic rock
            0x040200, // charred crevices
        ),
        // STATIC (alpha_b=0)
        lo(180, 100, 60, 140, 20, DIR_UP, 14, 0),
    ],
    // L5: SCATTER/EMBERS - VERY SPARSE, COMPLETELY STATIC
    [
        hi_meta(
            OP_SCATTER,
            REGION_ALL,
            BLEND_ADD,
            DOMAIN_DIRECT3D,
            SCATTER_EMBERS,
            0xff8020, // orange-yellow sparks
            0xc04000, // orange
        ),
        // EXTREMELY sparse, STATIC (density=3, alpha_b=0)
        lo(60, 3, 180, 0x14, 4, DIR_UP, 8, 0),
    ],
    // L6: BAND - subtle heat glow at horizon (STATIC)
    [
        hi(OP_BAND, REGION_ALL, BLEND_ADD, 0, 0x500c00, 0x180400),
        // STATIC glow band (alpha_b=0)
        lo(50, 80, 100, 140, 0, DIR_DOWN, 7, 0),
    ],
    // L7: ATMOSPHERE/ABSORPTION - smoky haze
    [
        hi_meta(
            OP_ATMOSPHERE,
            REGION_ALL,
            BLEND_MULTIPLY,
            DOMAIN_DIRECT3D,
            ATMO_ABSORPTION,
            0x140804, // warm dark
            0x060200, // very dark
        ),
        // STATIC haze (alpha_b=0)
        lo(50, 90, 80, 0, 0, DIR_UP, 10, 0),
    ],
];
