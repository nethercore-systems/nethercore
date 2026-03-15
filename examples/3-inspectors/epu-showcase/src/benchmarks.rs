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
    [0, 0, 0, 0, 0, 0, 4, 2], // Front Mass
    [0, 0, 0, 0, 0, 0, 4, 0], // Frozen Bed
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
        hi(OP_RAMP, REGION_ALL, BLEND_LERP, 0, 0xc9d5dd, 0x2f3d49),
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
        hi(OP_FLOW, REGION_FLOOR, BLEND_SCREEN, 0, 0xe7eff6, 0x748594),
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
        hi(OP_LOBE, REGION_WALLS, BLEND_SCREEN, 0, 0x8ddfff, 0x264257),
        lo(164, 84, 102, 0, 0, DIR_FORWARD, 9, 0),
    ],
    [
        hi(OP_FLOW, REGION_FLOOR, BLEND_SCREEN, 0, 0xa3d7f0, 0x375163),
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
        hi(OP_RAMP, REGION_ALL, BLEND_LERP, 0, 0xd8e2e8, 0x435361),
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
            0xa5b0ba,
            0x0d141b,
        ),
        lo(0, 28, 136, 70, 114, DIR_UP, 15, 15),
    ],
    [
        hi_meta(
            OP_SILHOUETTE,
            REGION_SKY,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            SILHOUETTE_MOUNTAINS,
            0x202b33,
            0x6d7780,
        ),
        lo(180, 76, 156, 0x46, 0, DIR_UP, 12, 10),
    ],
    [
        hi_meta(
            OP_PLANE,
            REGION_FLOOR,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            PLANE_STONE,
            0x364149,
            0x0d141b,
        ),
        lo(246, 94, 24, 164, 0, DIR_UP, 15, 13),
    ],
    [
        hi_meta(
            OP_MASS,
            REGION_SKY,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            MASS_SHELF,
            0x73818d,
            0x070b10,
        ),
        lo(244, 112, 214, 80, 0, DIR_LEFT, 15, 0),
    ],
    [
        hi_meta(
            OP_MOTTLE,
            REGION_SKY,
            BLEND_MULTIPLY,
            DOMAIN_DIRECT3D,
            MOTTLE_SOFT,
            0xa4b1bc,
            0x26323b,
        ),
        lo(200, 56, 170, 58, 12, DIR_LEFT, 11, 0),
    ],
    [
        hi_meta(
            OP_MOTTLE,
            REGION_FLOOR,
            BLEND_MULTIPLY,
            DOMAIN_DIRECT3D,
            MOTTLE_GRAIN,
            0x74818b,
            0x182027,
        ),
        lo(172, 40, 180, 126, 10, DIR_RIGHT, 10, 0),
    ],
    [
        hi_meta(
            OP_ADVECT,
            REGION_SKY,
            BLEND_SCREEN,
            DOMAIN_DIRECT3D,
            ADVECT_FRONT,
            0xd7e3eb,
            0x4a5965,
        ),
        lo(176, 54, 124, 98, 0, DIR_LEFT, 10, 0),
    ],
    [hi(OP_FLOW, REGION_FLOOR, BLEND_SCREEN, 0, 0xafbfcc, 0x3b4a56), lo(84, 14, 20, 0x21, 0, DIR_RIGHT, 5, 0)],
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
            0x60707d,
            0xcfd9df,
        ),
        lo(148, 76, 142, 0x54, 0, DIR_UP, 15, 15),
    ],
    [
        hi_meta(
            OP_SPLIT,
            REGION_ALL,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            SPLIT_FACE,
            0xb1bec8,
            0x182028,
        ),
        lo(32, 92, 28, 112, 0, DIR_UP, 14, 0),
    ],
    [
        hi_meta(
            OP_PLANE,
            REGION_FLOOR,
            BLEND_LERP,
            DOMAIN_DIRECT3D,
            PLANE_STONE,
            0x54616a,
            0x131a21,
        ),
        lo(248, 104, 20, 168, 0, DIR_UP, 15, 14),
    ],
    [
        hi(
            OP_SURFACE,
            REGION_FLOOR,
            BLEND_LERP,
            SURFACE_CRUST,
            0xc0cdd4,
            0x2d3942,
        ),
        lo(196, 62, 206, 32, 10, DIR_UP, 13, 0),
    ],
    [
        hi_meta(
            OP_MOTTLE,
            REGION_FLOOR,
            BLEND_MULTIPLY,
            DOMAIN_DIRECT3D,
            MOTTLE_GRAIN,
            0xa8b6bf,
            0x29343c,
        ),
        lo(214, 46, 196, 132, 12, DIR_RIGHT, 12, 0),
    ],
    [
        hi(
            OP_SURFACE,
            REGION_FLOOR,
            BLEND_SCREEN,
            SURFACE_DUSTED,
            0xe1e7ec,
            0x8a97a3,
        ),
        lo(92, 34, 150, 58, 18, DIR_UP, 4, 0),
    ],
    [
        hi_meta(
            OP_ADVECT,
            REGION_SKY,
            BLEND_SCREEN,
            DOMAIN_DIRECT3D,
            ADVECT_SPINDRIFT,
            0xf2f7fb,
            0xb8c7d2,
        ),
        lo(164, 32, 84, 134, 0, DIR_RIGHT, 8, 0),
    ],
    [
        hi(
            OP_BAND,
            REGION_SKY | REGION_WALLS,
            BLEND_SCREEN,
            0,
            0xd7e2ea,
            0x7f93a4,
        ),
        lo(22, 82, 118, 94, 0, DIR_FORWARD, 4, 0),
    ],
];
