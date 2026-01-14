//! Environment system FFI functions (Multi-Environment v3)
//!
//! Functions for setting procedural environment rendering parameters.
//! Environments support 8 modes with layering (base + overlay) and blend modes.

mod config;
mod draw;

use anyhow::Result;
use wasmtime::Linker;

use super::ZXGameContext;

// Re-export functions for registration
pub(crate) use config::{
    env_curtains, env_gradient, env_lines, env_rectangles, env_rings, env_room, env_scatter,
    env_silhouette,
};
pub(crate) use draw::{draw_env, env_blend, matcap_set};

/// Register environment system FFI functions
pub fn register(linker: &mut Linker<ZXGameContext>) -> Result<()> {
    linker.func_wrap("env", "env_gradient", env_gradient)?;
    linker.func_wrap("env", "env_scatter", env_scatter)?;
    linker.func_wrap("env", "env_lines", env_lines)?;
    linker.func_wrap("env", "env_silhouette", env_silhouette)?;
    linker.func_wrap("env", "env_rectangles", env_rectangles)?;
    linker.func_wrap("env", "env_room", env_room)?;
    linker.func_wrap("env", "env_curtains", env_curtains)?;
    linker.func_wrap("env", "env_rings", env_rings)?;
    linker.func_wrap("env", "env_blend", env_blend)?;
    linker.func_wrap("env", "matcap_set", matcap_set)?;
    linker.func_wrap("env", "draw_env", draw_env)?;
    Ok(())
}
