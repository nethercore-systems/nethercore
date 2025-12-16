//! Debug inspection FFI functions
//!
//! # WATCHDOG: Keep this file under 200 lines
//!
//! This file should ONLY contain:
//! - Module declarations
//! - Public re-exports
//! - Registration function
//! - HasDebugRegistry trait
//!
//! ❌ DO NOT add FFI function implementations here
//! ✅ DO add them to domain-specific submodules (register.rs, watch.rs, control.rs)

mod control;
mod register;
mod watch;

use anyhow::Result;
use wasmtime::{Caller, Linker};

use crate::console::{ConsoleInput, ConsoleRollbackState};
use crate::wasm::{WasmGameContext, read_string_from_memory};

// Re-export the trait
pub use self::register::HasDebugRegistry;

/// Register all debug FFI functions with the linker
///
/// These functions are always registered (even for release builds), but games
/// built in release mode won't import them, so they won't be linked.
pub fn register_debug_ffi<I, S, R>(linker: &mut Linker<WasmGameContext<I, S, R>>) -> Result<()>
where
    I: ConsoleInput,
    S: Send + Default + 'static,
    R: ConsoleRollbackState,
    WasmGameContext<I, S, R>: HasDebugRegistry,
{
    register::register(linker)?;
    watch::register(linker)?;
    control::register(linker)?;
    Ok(())
}

/// Read a length-prefixed string from WASM memory
pub(super) fn read_string<I, S, R>(
    caller: &Caller<'_, WasmGameContext<I, S, R>>,
    ptr: u32,
    len: u32,
) -> Option<String>
where
    I: ConsoleInput,
    R: ConsoleRollbackState,
{
    let memory = caller.data().game.memory?;
    read_string_from_memory(memory, caller, ptr, len)
}
