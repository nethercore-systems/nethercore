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
    include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/shaders/common/20_environment/00_utils.wgsl"
    )),
    include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/shaders/common/20_environment/10_mode0_gradient.wgsl"
    )),
    include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/shaders/common/20_environment/20_mode1_cells.wgsl"
    )),
    include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/shaders/common/20_environment/30_mode2_lines.wgsl"
    )),
    include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/shaders/common/20_environment/40_mode3_silhouette.wgsl"
    )),
    include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/shaders/common/20_environment/50_mode4_nebula.wgsl"
    )),
    include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/shaders/common/20_environment/60_mode5_room.wgsl"
    )),
    include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/shaders/common/20_environment/70_mode6_veil.wgsl"
    )),
    include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/shaders/common/20_environment/80_mode7_rings.wgsl"
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

/// EPU bounds opcodes: RAMP, LOBE, BAND, FOG.
#[allow(dead_code)]
pub(crate) const EPU_BOUNDS: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/shaders/epu/epu_bounds.wgsl"
));

/// EPU feature opcodes: DECAL, GRID, SCATTER, FLOW, plus layer dispatch.
#[allow(dead_code)]
pub(crate) const EPU_FEATURES: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/shaders/epu/epu_features.wgsl"
));

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
    "shaders/common/20_environment/00_utils.wgsl",
    "shaders/common/20_environment/10_mode0_gradient.wgsl",
    "shaders/common/20_environment/20_mode1_cells.wgsl",
    "shaders/common/20_environment/30_mode2_lines.wgsl",
    "shaders/common/20_environment/40_mode3_silhouette.wgsl",
    "shaders/common/20_environment/50_mode4_nebula.wgsl",
    "shaders/common/20_environment/60_mode5_room.wgsl",
    "shaders/common/20_environment/70_mode6_veil.wgsl",
    "shaders/common/20_environment/80_mode7_rings.wgsl",
    "shaders/common/20_environment/90_sampling.wgsl",
    "shaders/common/30_lighting.wgsl",
    "shaders/common/90_vertex_io.wgsl",
    "shaders/blinnphong_common.wgsl",
    "shaders/mode0_lambert.wgsl",
    "shaders/mode1_matcap.wgsl",
    "shaders/env_template.wgsl",
    "shaders/quad_template.wgsl",
    // EPU shaders
    "shaders/epu/epu_common.wgsl",
    "shaders/epu/epu_bounds.wgsl",
    "shaders/epu/epu_features.wgsl",
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
