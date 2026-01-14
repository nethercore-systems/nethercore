//! Common FFI host functions
//!
//! These functions are available to all fantasy consoles.
//! Console-specific FFI functions are registered via the Console trait.

mod random;
mod save;
mod session;
mod system;

#[cfg(test)]
mod tests;

use anyhow::Result;
use wasmtime::Linker;

use crate::console::{ConsoleInput, ConsoleRollbackState};
use crate::debug::ffi::register_debug_ffi;
use crate::wasm::WasmGameContext;

/// Register common FFI functions with the linker
pub fn register_common_ffi<
    I: ConsoleInput,
    S: Send + Default + 'static,
    R: ConsoleRollbackState,
>(
    linker: &mut Linker<WasmGameContext<I, S, R>>,
) -> Result<()> {
    // System functions
    linker.func_wrap("env", "delta_time", system::delta_time)?;
    linker.func_wrap("env", "elapsed_time", system::elapsed_time)?;
    linker.func_wrap("env", "tick_count", system::tick_count)?;
    linker.func_wrap("env", "log", system::log_message)?;
    linker.func_wrap("env", "quit", system::quit)?;

    // Rollback functions
    linker.func_wrap("env", "random", random::random)?;
    linker.func_wrap("env", "random_range", random::random_range)?;
    linker.func_wrap("env", "random_f32", random::random_f32)?;
    linker.func_wrap("env", "random_f32_range", random::random_f32_range)?;

    // Save data functions
    linker.func_wrap("env", "save", save::save)?;
    linker.func_wrap("env", "load", save::load)?;
    linker.func_wrap("env", "delete", save::delete)?;

    // Session functions
    linker.func_wrap("env", "player_count", session::player_count)?;
    linker.func_wrap("env", "local_player_mask", session::local_player_mask)?;
    linker.func_wrap("env", "player_handle", session::player_handle)?;
    linker.func_wrap("env", "is_connected", session::is_connected)?;

    // Debug inspection functions
    // These are always registered; release builds won't import them
    register_debug_ffi(linker)?;

    Ok(())
}
