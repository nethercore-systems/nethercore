//! Pipeline key types for caching
//!
//! Pipeline keys uniquely identify a pipeline configuration for caching purposes.

use super::super::RenderState;
use super::super::render_state::PassConfig;

/// Key for pipeline cache lookup
///
/// Pipeline keys are derived from PassConfig to enable caching by
/// depth/stencil configuration. The pass_config_hash captures the
/// unique combination of depth/stencil settings.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum PipelineKey {
    /// Regular mesh rendering pipeline
    Regular {
        render_mode: u8,
        vertex_format: u8,
        cull_mode: u8,
        /// Hash of PassConfig fields that affect pipeline state
        pass_config_hash: u64,
    },
    /// GPU-instanced quad rendering pipeline (billboards, sprites)
    Quad {
        /// Hash of PassConfig fields that affect pipeline state
        pass_config_hash: u64,
        /// True for screen-space quads (always write depth), false for billboards (use PassConfig)
        is_screen_space: bool,
    },
    /// Procedural environment rendering pipeline (always renders behind)
    Environment {
        /// Hash of PassConfig fields that affect pipeline state
        pass_config_hash: u64,
    },
}

/// Compute a hash of PassConfig fields that affect pipeline state
fn pass_config_hash(config: &PassConfig) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    // NOTE: Only hash fields that impact pipeline creation.
    // - `depth_clear` controls render pass clears (not pipeline state)
    // - `stencil_ref` is set dynamically via `render_pass.set_stencil_reference` (not pipeline state)
    config.depth_compare.hash(&mut hasher);
    config.depth_write.hash(&mut hasher);
    config.stencil_compare.hash(&mut hasher);
    config.stencil_pass.hash(&mut hasher);
    config.stencil_fail.hash(&mut hasher);
    config.stencil_depth_fail.hash(&mut hasher);
    hasher.finish()
}

impl PipelineKey {
    /// Create a new regular pipeline key from render state and pass config
    pub fn new(render_mode: u8, format: u8, state: &RenderState, pass_config: &PassConfig) -> Self {
        Self::Regular {
            render_mode,
            vertex_format: format,
            cull_mode: state.cull_mode as u8,
            pass_config_hash: pass_config_hash(pass_config),
        }
    }

    /// Create a quad pipeline key
    pub fn quad(pass_config: &PassConfig, is_screen_space: bool) -> Self {
        Self::Quad {
            pass_config_hash: pass_config_hash(pass_config),
            is_screen_space,
        }
    }

    /// Create an environment pipeline key
    pub fn environment(pass_config: &PassConfig) -> Self {
        Self::Environment {
            pass_config_hash: pass_config_hash(pass_config),
        }
    }
}
