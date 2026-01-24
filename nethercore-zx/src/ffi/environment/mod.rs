//! Environment Processing Unit (EPU) FFI functions
//!
//! Functions for configuring and rendering procedural environments using the
//! instruction-based EPU system. Each environment config is 128 bytes
//! containing 8 packed instruction layers.

mod draw;
mod epu;

use anyhow::Result;
use wasmtime::Linker;

use super::ZXGameContext;

// Re-export functions for registration
pub(crate) use draw::matcap_set;
pub(crate) use epu::{epu_draw, epu_set_env};

/// Register EPU FFI functions
pub fn register(linker: &mut Linker<ZXGameContext>) -> Result<()> {
    // EPU instruction-based API (config + draw request)
    linker.func_wrap("env", "epu_draw", epu_draw)?;
    linker.func_wrap("env", "epu_set_env", epu_set_env)?;

    // Matcap controls (Mode 1)
    linker.func_wrap("env", "matcap_set", matcap_set)?;

    Ok(())
}
