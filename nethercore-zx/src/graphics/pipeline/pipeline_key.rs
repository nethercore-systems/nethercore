//! Pipeline key types for caching
//!
//! Pipeline keys uniquely identify a pipeline configuration for caching purposes.

use super::super::render_state::PassConfig;
use super::super::RenderState;

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
        depth_test: bool,
        cull_mode: u8,
        /// Hash of PassConfig fields that affect pipeline state
        pass_config_hash: u64,
    },
    /// GPU-instanced quad rendering pipeline (billboards, sprites)
    Quad {
        depth_test: bool,
        /// Hash of PassConfig fields that affect pipeline state
        pass_config_hash: u64,
        /// True for screen-space quads (always write depth), false for billboards (use PassConfig)
        is_screen_space: bool,
    },
    /// Procedural sky rendering pipeline (always renders behind)
    Sky {
        /// Hash of PassConfig fields that affect pipeline state
        pass_config_hash: u64,
    },
}

/// Compute a hash of PassConfig fields that affect pipeline state
fn pass_config_hash(config: &PassConfig) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    config.hash(&mut hasher);
    hasher.finish()
}

impl PipelineKey {
    /// Create a new regular pipeline key from render state and pass config
    pub fn new(render_mode: u8, format: u8, state: &RenderState, pass_config: &PassConfig) -> Self {
        Self::Regular {
            render_mode,
            vertex_format: format,
            depth_test: state.depth_test,
            cull_mode: state.cull_mode as u8,
            pass_config_hash: pass_config_hash(pass_config),
        }
    }

    /// Create a quad pipeline key
    pub fn quad(state: &RenderState, pass_config: &PassConfig, is_screen_space: bool) -> Self {
        Self::Quad {
            depth_test: state.depth_test,
            pass_config_hash: pass_config_hash(pass_config),
            is_screen_space,
        }
    }

    /// Create a sky pipeline key
    pub fn sky(pass_config: &PassConfig) -> Self {
        Self::Sky {
            pass_config_hash: pass_config_hash(pass_config),
        }
    }
}
