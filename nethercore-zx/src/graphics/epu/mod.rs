//! EPU (Environment Processing Unit) Rust API
//!
//! This module provides the Rust-side EPU types and builder API that encode
//! semantic environment configuration into the 128-byte packed format consumed
//! by the GPU compute shaders.
//!
//! # Architecture
//!
//! The EPU produces a single directional radiance signal per environment.
//! That radiance is stored in `EnvRadiance` (mip 0) and then downsampled into
//! a true mip pyramid for roughness-based reflections. Diffuse ambient uses
//! SH9 coefficients extracted from a coarse mip level.
//!
//! # Format (128-bit instructions)
//!
//! Each environment is 128 bytes (8 x 128-bit instructions). The 128-bit format
//! provides direct RGB colors, per-color alpha, and region masks for more
//! flexible compositing.
//!
//! # Example
//!
//! ```ignore
//! let mut e = epu_begin();
//! e.ramp_bounds(RampParams { ... });
//! e.sector_bounds(SectorParams { ... });
//! e.decal(DecalParams { ..Default::default() });
//! e.lobe_radiance(LobeRadianceParams { ... });
//! let config = epu_finish(e);
//! ```

// Submodules for organized runtime code
mod builder;
mod cache;
mod layer;
mod params;
mod pipelines;
pub mod runtime;
mod settings;
mod shaders;
mod types;

#[cfg(test)]
mod tests;

// Re-export runtime types
pub use cache::{ActiveEnvList, collect_active_envs};
pub use runtime::EpuRuntime;
pub use settings::{
    EPU_MAP_SIZE, EPU_MIN_MIP_SIZE, EpuRuntimeSettings, MAX_ACTIVE_ENVS, MAX_ENV_STATES,
};
pub use types::EpuSh9;

// Re-export layer types (core types, opcodes, enums, encoding utilities)
pub use layer::{
    EpuBlend, EpuConfig, EpuLayer, EpuOpcode, EpuRegion, REGION_ALL, REGION_FLOOR, REGION_NONE,
    REGION_SKY, REGION_WALLS, encode_direction_u16, pack_meta5, pack_thresholds,
};

// Re-export builder API
pub use builder::{EpuBuilder, epu_begin, epu_finish};

// Re-export parameter structs and shape/pattern enums
pub use params::{
    ApertureParams, AtmosphereParams, BandRadianceParams, CellParams, DecalParams, DecalShape,
    FlowParams, FlowPattern, GridParams, GridPattern, LobeRadianceParams, PatchesParams,
    PhaseWaveform, RampParams, ScatterParams, SectorParams, SilhouetteParams, SplitParams,
};
