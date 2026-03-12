//! EPU benchmark scenes.
//!
//! These are small capability probes, not showcase presets. They exist to
//! answer "can the current EPU surface do this at all?" before agents burn
//! full-roster passes trying to infer system truth from final scenes.

#[allow(unused_imports)]
use crate::constants::*;
use crate::SceneRig;

pub type Benchmark = [[u64; 2]; 8];

pub const BENCHMARK_COUNT: usize = 6;

pub static BENCHMARKS: [Benchmark; BENCHMARK_COUNT] = [
    BENCHMARK_OPEN_HORIZON,
    BENCHMARK_REGION_ISOLATION,
    BENCHMARK_PROJECTION_BAY,
    BENCHMARK_TRANSPORT_SWEEP,
    BENCHMARK_FRONT_MASS,
    BENCHMARK_FROZEN_BED,
];

pub static BENCHMARK_ANIM_SPEEDS: [[u8; 8]; BENCHMARK_COUNT] = [
    [0, 0, 0, 2, 0, 1, 0, 0], // Open Horizon
    [0, 0, 0, 3, 0, 0, 0, 0], // Region Isolation
    [0, 0, 4, 3, 4, 2, 0, 0], // Projection Bay
    [0, 0, 4, 0, 0, 0, 0, 0], // Transport Sweep
    [0, 0, 2, 0, 4, 0, 0, 0], // Front Mass
    [0, 0, 0, 1, 4, 0, 0, 0], // Frozen Bed
];

pub static BENCHMARK_RIGS: [SceneRig; BENCHMARK_COUNT] = [
    SceneRig::new(6.0, 11.0, 58.0, 0.86), // Open Horizon
    SceneRig::new(5.6, 14.0, 58.0, 0.92), // Region Isolation
    SceneRig::new(5.0, 15.0, 60.0, 1.0),  // Projection Bay
    SceneRig::new(6.0, 11.0, 58.0, 0.86), // Transport Sweep
    SceneRig::new(6.2, 10.0, 56.0, 0.82), // Front Mass
    SceneRig::new(6.2, 10.0, 56.0, 0.82), // Frozen Bed
];

pub const BENCHMARK_NAMES: [&str; BENCHMARK_COUNT] = [
    "Benchmark: Open Horizon",
    "Benchmark: Region Isolation",
    "Benchmark: Projection Bay",
    "Benchmark: Transport Sweep",
    "Benchmark: Front Mass",
    "Benchmark: Frozen Bed",
];

// -----------------------------------------------------------------------------
// 1. Open Horizon
// -----------------------------------------------------------------------------
pub(super) const BENCHMARK_OPEN_HORIZON: Benchmark = [
    [
        hi_meta(
            OP_SILHOUETTE,
            REGION_SKY,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            SILHOUETTE_MOUNTAINS,
            0x6d7d8a,
            0xd8e0e6,
        ),
        lo(120, 72, 148, 0x60, 0, DIR_UP, 15, 15),
    ],
    [
        hi_meta(
            OP_SPLIT,
            REGION_ALL,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            SPLIT_TIER,
            0xbdc8d1,
            0x35424f,
        ),
        lo(24, 84, 30, 96, 0, DIR_UP, 7, 0),
    ],
    [
        hi(
            OP_SURFACE,
            REGION_FLOOR,
            BLEND_LERP,
            SURFACE_GLAZE,
            0xd5dde2,
            0x536676,
        ),
        lo(132, 62, 120, 34, 0, DIR_UP, 10, 0),
    ],
    [
        hi_meta(
            OP_ADVECT,
            REGION_SKY,
            BLEND_SCREEN,
            DOMAIN_DIRECT3D,
            ADVECT_MIST,
            0xf2f7fb,
            0xb8c8d4,
        ),
        lo(120, 36, 96, 118, 0, DIR_RIGHT, 7, 0),
    ],
    [
        hi_meta(
            OP_ATMOSPHERE,
            REGION_SKY | REGION_WALLS,
            BLEND_SCREEN,
            DOMAIN_DIRECT3D,
            ATMO_RAYLEIGH,
            0x6b7f93,
            0xd1dce4,
        ),
        lo(8, 70, 88, 0, 0, DIR_UP, 2, 0),
    ],
    [
        hi_meta(
            OP_MOTTLE,
            REGION_SKY,
            BLEND_MULTIPLY,
            DOMAIN_DIRECT3D,
            MOTTLE_SOFT,
            0xbac7d0,
            0x536372,
        ),
        lo(160, 54, 144, 56, 10, DIR_LEFT, 10, 0),
    ],
    [
        hi_meta(
            OP_MOTTLE,
            REGION_FLOOR,
            BLEND_MULTIPLY,
            DOMAIN_DIRECT3D,
            MOTTLE_GRAIN,
            0xb7c3cc,
            0x4a5968,
        ),
        lo(148, 42, 176, 120, 8, DIR_RIGHT, 9, 0),
    ],
    [0, 0],
];

// -----------------------------------------------------------------------------
// 2. Region Isolation
// -----------------------------------------------------------------------------
pub(super) const BENCHMARK_REGION_ISOLATION: Benchmark = [
    [
        hi(
            OP_RAMP,
            REGION_ALL,
            BLEND_LERP,
            0,
            0xc9d5dd,
            0x2f3d49,
        ),
        lo(0, 0xB0, 0x38, 0x74, 0x10, DIR_UP, 8, 15),
    ],
    [
        hi_meta(
            OP_SPLIT,
            REGION_ALL,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            SPLIT_FACE,
            0xa9bac6,
            0x25323c,
        ),
        lo(20, 96, 26, 96, 0, DIR_LEFT, 8, 0),
    ],
    [
        hi_meta(
            OP_PLANE,
            REGION_FLOOR,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            PLANE_PAVEMENT,
            0xa6b4be,
            0x50606e,
        ),
        lo(176, 54, 0, 0, 0, DIR_UP, 12, 0),
    ],
    [
        hi(
            OP_FLOW,
            REGION_FLOOR,
            BLEND_SCREEN,
            0,
            0xe7eff6,
            0x748594,
        ),
        lo(118, 18, 22, 0x21, 0, DIR_RIGHT, 7, 0),
    ],
    [
        hi_meta(
            OP_MOTTLE,
            REGION_WALLS,
            BLEND_OVERLAY,
            DOMAIN_DIRECT3D,
            MOTTLE_RIDGE,
            0xa7b5c0,
            0x22303a,
        ),
        lo(78, 44, 170, 96, 16, DIR_LEFT, 8, 0),
    ],
    [0, 0],
    [0, 0],
    [0, 0],
];

// -----------------------------------------------------------------------------
// 3. Projection Bay
// -----------------------------------------------------------------------------
pub(super) const BENCHMARK_PROJECTION_BAY: Benchmark = [
    [
        hi_meta(
            OP_SPLIT,
            REGION_ALL,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            SPLIT_FACE,
            0x77909b,
            0x141c24,
        ),
        lo(0, 34, 132, 74, 112, DIR_UP, 15, 15),
    ],
    [
        hi_meta(
            OP_PORTAL,
            REGION_WALLS,
            BLEND_SCREEN,
            DOMAIN_TANGENT_LOCAL,
            PORTAL_RECT,
            0xb9f3ff,
            0x23485b,
        ),
        lo(232, 58, 72, 96, 0, DIR_FORWARD, 12, 0),
    ],
    [
        hi_meta(
            OP_GRID,
            REGION_WALLS,
            BLEND_SCREEN,
            DOMAIN_TANGENT_LOCAL,
            1,
            0xdaf7ff,
            0x5a7588,
        ),
        lo(196, 48, 108, 0, 0, DIR_FORWARD, 8, 0),
    ],
    [
        hi_meta(
            OP_VEIL,
            REGION_WALLS,
            BLEND_SCREEN,
            DOMAIN_AXIS_CYL,
            VEIL_RAIN_WALL,
            0xcdf6ff,
            0x5d7d8f,
        ),
        lo(184, 20, 144, 68, 0, DIR_DOWN, 7, 0),
    ],
    [
        hi(
            OP_LOBE,
            REGION_WALLS,
            BLEND_SCREEN,
            0,
            0x8ddfff,
            0x264257,
        ),
        lo(164, 84, 102, 0, 0, DIR_FORWARD, 9, 0),
    ],
    [
        hi(
            OP_FLOW,
            REGION_FLOOR,
            BLEND_SCREEN,
            0,
            0xa3d7f0,
            0x375163,
        ),
        lo(92, 18, 20, 0x21, 0, DIR_RIGHT, 6, 0),
    ],
    [0, 0],
    [0, 0],
];

// -----------------------------------------------------------------------------
// 4. Transport Sweep
// -----------------------------------------------------------------------------
pub(super) const BENCHMARK_TRANSPORT_SWEEP: Benchmark = [
    [
        hi(
            OP_RAMP,
            REGION_ALL,
            BLEND_LERP,
            0,
            0xd8e2e8,
            0x435361,
        ),
        lo(0, 0xB0, 0x46, 0x86, 0x10, DIR_UP, 8, 15),
    ],
    [
        hi_meta(
            OP_ADVECT,
            REGION_WALLS,
            BLEND_SCREEN,
            DOMAIN_DIRECT3D,
            ADVECT_SHEET,
            0xf5fbff,
            0x7890a0,
        ),
        lo(212, 42, 132, 128, 0, DIR_RIGHT, 11, 0),
    ],
    [
        hi_meta(
            OP_ADVECT,
            REGION_WALLS,
            BLEND_SCREEN,
            DOMAIN_DIRECT3D,
            ADVECT_SPINDRIFT,
            0xffffff,
            0xbcd0da,
        ),
        lo(188, 30, 104, 164, 0, DIR_RIGHT, 10, 0),
    ],
    [0, 0],
    [0, 0],
    [0, 0],
    [0, 0],
    [0, 0],
];

// -----------------------------------------------------------------------------
// 5. Front Mass
// -----------------------------------------------------------------------------
pub(super) const BENCHMARK_FRONT_MASS: Benchmark = [
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
        lo(0, 26, 132, 62, 116, DIR_UP, 6, 15),
    ],
    [
        hi_meta(
            OP_PLANE,
            REGION_FLOOR,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            PLANE_WATER,
            0x324755,
            0x081017,
        ),
        lo(255, 54, 0, 0, 0, DIR_UP, 15, 0),
    ],
    [
        hi_meta(
            OP_ADVECT,
            REGION_WALLS,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            ADVECT_FRONT,
            0x9dadb8,
            0x384753,
        ),
        lo(176, 56, 132, 96, 0, DIR_LEFT, 10, 0),
    ],
    [
        hi_meta(
            OP_MASS,
            REGION_SKY | REGION_WALLS,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            MASS_BANK,
            0x667783,
            0x060a0f,
        ),
        lo(248, 104, 212, 82, 0, DIR_LEFT, 15, 0),
    ],
    [
        hi_meta(
            OP_MOTTLE,
            REGION_WALLS,
            BLEND_MULTIPLY,
            DOMAIN_DIRECT3D,
            MOTTLE_RIDGE,
            0x74838e,
            0x121b23,
        ),
        lo(72, 54, 172, 96, 18, DIR_LEFT, 5, 0),
    ],
    [0, 0],
    [0, 0],
    [0, 0],
];

// -----------------------------------------------------------------------------
// 6. Frozen Bed
// -----------------------------------------------------------------------------
pub(super) const BENCHMARK_FROZEN_BED: Benchmark = [
    [
        hi_meta(
            OP_SILHOUETTE,
            REGION_SKY,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            SILHOUETTE_MOUNTAINS,
            0x6e8290,
            0xd9e2e8,
        ),
        lo(116, 70, 150, 0x62, 0, DIR_UP, 15, 15),
    ],
    [
        hi_meta(
            OP_SPLIT,
            REGION_ALL,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            SPLIT_TIER,
            0xc1ccd4,
            0x33424f,
        ),
        lo(18, 78, 26, 96, 0, DIR_UP, 6, 0),
    ],
    [
        hi_meta(
            OP_MASS,
            REGION_WALLS,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            MASS_BANK,
            0xd6e0e6,
            0x425563,
        ),
        lo(232, 84, 188, 72, 0, DIR_UP, 13, 0),
    ],
    [
        hi(
            OP_SURFACE,
            REGION_FLOOR,
            BLEND_LERP,
            SURFACE_GLAZE,
            0xd7e2e8,
            0x4f6474,
        ),
        lo(176, 66, 152, 40, 0, DIR_UP, 12, 0),
    ],
    [
        hi(
            OP_SURFACE,
            REGION_FLOOR,
            BLEND_MULTIPLY,
            SURFACE_CRUST,
            0xc3d0d8,
            0x31404c,
        ),
        lo(152, 46, 220, 14, 18, DIR_UP, 10, 0),
    ],
    [
        hi_meta(
            OP_ADVECT,
            REGION_SKY,
            BLEND_SCREEN,
            DOMAIN_DIRECT3D,
            ADVECT_SPINDRIFT,
            0xf6fbff,
            0xc0d0da,
        ),
        lo(180, 34, 86, 144, 0, DIR_RIGHT, 10, 0),
    ],
    [0, 0],
    [0, 0],
];
