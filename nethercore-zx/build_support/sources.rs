//! WGSL sources embedded at build time.

// The shared common WGSL sources are split into multiple files to keep shader code manageable.
// Order is significant; the concatenation must preserve marker strings used by
// `extract_common_*()` (e.g. "// Data Unpacking Utilities").
pub(crate) const COMMON: &str = concat!(
    include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/shaders/common/00_bindings.wgsl"
    )),
    include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/shaders/common/10_unpacking.wgsl"
    )),
    // EPU evaluation (procedural radiance for sky + specular residual)
    // Common utilities, constants, structs
    include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/shaders/epu/epu_common.wgsl"
    )),
    // Bounds opcodes (enclosure layers)
    include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/shaders/epu/bounds/00_ramp.wgsl"
    )),
    include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/shaders/epu/bounds/01_sector.wgsl"
    )),
    include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/shaders/epu/bounds/02_silhouette.wgsl"
    )),
    include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/shaders/epu/bounds/03_split.wgsl"
    )),
    include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/shaders/epu/bounds/04_cell.wgsl"
    )),
    include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/shaders/epu/bounds/05_patches.wgsl"
    )),
    include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/shaders/epu/bounds/06_aperture.wgsl"
    )),
    // Feature opcodes (radiance motifs)
    include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/shaders/epu/features/00_decal.wgsl"
    )),
    include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/shaders/epu/features/01_grid.wgsl"
    )),
    include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/shaders/epu/features/02_scatter.wgsl"
    )),
    include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/shaders/epu/features/03_flow.wgsl"
    )),
    include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/shaders/epu/features/04_trace.wgsl"
    )),
    include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/shaders/epu/features/05_veil.wgsl"
    )),
    include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/shaders/epu/features/06_atmosphere.wgsl"
    )),
    include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/shaders/epu/features/07_plane.wgsl"
    )),
    include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/shaders/epu/features/08_celestial.wgsl"
    )),
    include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/shaders/epu/features/09_portal.wgsl"
    )),
    include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/shaders/epu/features/10_lobe_radiance.wgsl"
    )),
    include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/shaders/epu/features/11_band_radiance.wgsl"
    )),
    // Layer dispatch
    include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/shaders/epu/epu_dispatch.wgsl"
    )),
    include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/shaders/common/20_environment/90_sampling.wgsl"
    )),
    include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/shaders/common/30_lighting.wgsl"
    )),
    include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/shaders/common/90_vertex_io.wgsl"
    )),
);

pub(crate) const BLINNPHONG_COMMON: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/shaders/blinnphong_common.wgsl"
));
pub(crate) const TEMPLATE_MODE0: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/shaders/mode0_lambert.wgsl"
));
pub(crate) const TEMPLATE_MODE1: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/shaders/mode1_matcap.wgsl"
));
pub(crate) const ENV_TEMPLATE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/shaders/env_template.wgsl"
));
pub(crate) const QUAD_TEMPLATE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/shaders/quad_template.wgsl"
));

// ============================================================================
// EPU (Environment Processing Unit) Shaders
// ============================================================================
// Note: These constants are declared for use in subsequent EPU pipeline tasks.
// Allow dead_code until the runtime wiring is complete.

/// EPU common types, decoding, and helpers (octahedral encode/decode, instruction
/// field extraction, region weights, blend logic, palette lookup).
#[allow(dead_code)]
pub(crate) const EPU_COMMON: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/shaders/epu/epu_common.wgsl"
));

/// EPU bounds opcodes (modular): RAMP, SECTOR, SILHOUETTE, SPLIT, CELL, PATCHES, APERTURE.
#[allow(dead_code)]
pub(crate) const EPU_BOUNDS: &str = concat!(
    include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/shaders/epu/bounds/00_ramp.wgsl"
    )),
    include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/shaders/epu/bounds/01_sector.wgsl"
    )),
    include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/shaders/epu/bounds/02_silhouette.wgsl"
    )),
    include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/shaders/epu/bounds/03_split.wgsl"
    )),
    include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/shaders/epu/bounds/04_cell.wgsl"
    )),
    include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/shaders/epu/bounds/05_patches.wgsl"
    )),
    include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/shaders/epu/bounds/06_aperture.wgsl"
    )),
);

/// EPU feature opcodes (modular): DECAL, GRID, SCATTER, FLOW, TRACE, VEIL, ATMOSPHERE, PLANE, CELESTIAL, PORTAL, LOBE_RADIANCE, BAND_RADIANCE, plus layer dispatch.
#[allow(dead_code)]
pub(crate) const EPU_FEATURES: &str = concat!(
    include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/shaders/epu/features/00_decal.wgsl"
    )),
    include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/shaders/epu/features/01_grid.wgsl"
    )),
    include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/shaders/epu/features/02_scatter.wgsl"
    )),
    include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/shaders/epu/features/03_flow.wgsl"
    )),
    include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/shaders/epu/features/04_trace.wgsl"
    )),
    include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/shaders/epu/features/05_veil.wgsl"
    )),
    include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/shaders/epu/features/06_atmosphere.wgsl"
    )),
    include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/shaders/epu/features/07_plane.wgsl"
    )),
    include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/shaders/epu/features/08_celestial.wgsl"
    )),
    include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/shaders/epu/features/09_portal.wgsl"
    )),
    include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/shaders/epu/features/10_lobe_radiance.wgsl"
    )),
    include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/shaders/epu/features/11_band_radiance.wgsl"
    )),
    include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/shaders/epu/epu_dispatch.wgsl"
    )),
);

/// EPU compute shader: environment evaluation (builds EnvSharp + EnvLight0).
#[allow(dead_code)]
pub(crate) const EPU_COMPUTE_ENV: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/shaders/epu/epu_compute_env.wgsl"
));

/// EPU compute shader: Kawase blur pyramid generation.
#[allow(dead_code)]
pub(crate) const EPU_COMPUTE_BLUR: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/shaders/epu/epu_compute_blur.wgsl"
));

/// EPU compute shader: 6-direction ambient cube extraction.
#[allow(dead_code)]
pub(crate) const EPU_COMPUTE_IRRAD: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/shaders/epu/epu_compute_irrad.wgsl"
));

/// EPU sampling functions for render pipelines (background, reflection, ambient).
#[allow(dead_code)]
pub(crate) const EPU_SAMPLE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/shaders/epu/epu_sample.wgsl"
));

const RERUN_IF_CHANGED_FILES: &[&str] = &[
    "shaders/common/00_bindings.wgsl",
    "shaders/common/10_unpacking.wgsl",
    "shaders/common/20_environment/90_sampling.wgsl",
    "shaders/common/30_lighting.wgsl",
    "shaders/common/90_vertex_io.wgsl",
    "shaders/blinnphong_common.wgsl",
    "shaders/mode0_lambert.wgsl",
    "shaders/mode1_matcap.wgsl",
    "shaders/env_template.wgsl",
    "shaders/quad_template.wgsl",
    // EPU shaders - common
    "shaders/epu/epu_common.wgsl",
    "shaders/epu/epu_dispatch.wgsl",
    // EPU bounds opcodes
    "shaders/epu/bounds/00_ramp.wgsl",
    "shaders/epu/bounds/01_sector.wgsl",
    "shaders/epu/bounds/02_silhouette.wgsl",
    "shaders/epu/bounds/03_split.wgsl",
    "shaders/epu/bounds/04_cell.wgsl",
    "shaders/epu/bounds/05_patches.wgsl",
    "shaders/epu/bounds/06_aperture.wgsl",
    // EPU feature opcodes
    "shaders/epu/features/00_decal.wgsl",
    "shaders/epu/features/01_grid.wgsl",
    "shaders/epu/features/02_scatter.wgsl",
    "shaders/epu/features/03_flow.wgsl",
    "shaders/epu/features/04_trace.wgsl",
    "shaders/epu/features/05_veil.wgsl",
    "shaders/epu/features/06_atmosphere.wgsl",
    "shaders/epu/features/07_plane.wgsl",
    "shaders/epu/features/08_celestial.wgsl",
    "shaders/epu/features/09_portal.wgsl",
    "shaders/epu/features/10_lobe_radiance.wgsl",
    "shaders/epu/features/11_band_radiance.wgsl",
    // EPU compute shaders
    "shaders/epu/epu_compute_env.wgsl",
    "shaders/epu/epu_compute_blur.wgsl",
    "shaders/epu/epu_compute_irrad.wgsl",
    "shaders/epu/epu_sample.wgsl",
];

pub(crate) fn emit_rerun_if_changed() {
    for file in RERUN_IF_CHANGED_FILES {
        println!("cargo:rerun-if-changed={file}");
    }
}
