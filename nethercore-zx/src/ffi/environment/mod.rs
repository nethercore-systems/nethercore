//! Environment Processing Unit (EPU) FFI functions
//!
//! Functions for configuring and rendering procedural environments using the
//! instruction-based EPU system. Each environment is a 64-byte configuration
//! containing 8 packed instruction layers.

mod draw;
mod epu;

use anyhow::Result;
use wasmtime::Linker;

use super::ZXGameContext;

// Re-export functions for registration
pub(crate) use draw::{draw_env, matcap_set};
pub(crate) use epu::{epu_draw, epu_set};

/// Register EPU FFI functions
pub fn register(linker: &mut Linker<ZXGameContext>) -> Result<()> {
    // EPU instruction-based API
    linker.func_wrap("env", "epu_set", epu_set)?;
    linker.func_wrap("env", "epu_draw", epu_draw)?;

    // Legacy functions kept for compatibility
    linker.func_wrap("env", "matcap_set", matcap_set)?;
    linker.func_wrap("env", "draw_env", draw_env)?;

    Ok(())
}
