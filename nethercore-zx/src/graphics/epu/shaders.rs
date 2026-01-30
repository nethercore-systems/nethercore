//! EPU shader source constants.
//!
//! This module contains the WGSL shader sources for EPU compute passes:
//! - Environment radiance generation (`epu_build`)
//! - Mip pyramid downsampling (`epu_downsample_mip`)
//! - SH9 irradiance extraction (`epu_extract_sh9`)

/// Common WGSL definitions (structs, helpers, blend functions).
pub(super) const EPU_COMMON: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/shaders/epu/epu_common.wgsl"
));

/// EPU bounds opcodes (modular files: RAMP + bounds ops).
pub(super) const EPU_BOUNDS: &str = concat!(
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

/// EPU feature opcodes (DECAL, GRID, SCATTER, FLOW, TRACE, VEIL, ATMOSPHERE, PLANE, CELESTIAL, PORTAL, LOBE_RADIANCE, BAND_RADIANCE) + dispatch entry.
pub(super) const EPU_FEATURES: &str = concat!(
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

/// Compute shader for environment radiance generation (mip 0).
pub(super) const EPU_COMPUTE_ENV: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/shaders/epu/epu_compute_env.wgsl"
));

/// Compute shader for mip pyramid downsampling (blur pass).
pub(super) const EPU_COMPUTE_BLUR: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/shaders/epu/epu_compute_blur.wgsl"
));

/// Compute shader for SH9 irradiance extraction.
pub(super) const EPU_COMPUTE_IRRAD: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/shaders/epu/epu_compute_irrad.wgsl"
));
