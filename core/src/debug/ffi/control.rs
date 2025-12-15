//! Debug control FFI functions
//!
//! Functions for grouping and querying debug state from WASM games.

use anyhow::Result;
use wasmtime::{Caller, Linker};

use crate::console::{ConsoleInput, ConsoleRollbackState};
use crate::wasm::WasmGameContext;

use super::read_string;
use super::register::HasDebugRegistry;

/// Register debug control FFI functions
pub(super) fn register<I, S, R>(linker: &mut Linker<WasmGameContext<I, S, R>>) -> Result<()>
where
    I: ConsoleInput,
    S: Send + Default + 'static,
    R: ConsoleRollbackState,
    WasmGameContext<I, S, R>: HasDebugRegistry,
{
    // Grouping functions
    linker.func_wrap("env", "debug_group_begin", debug_group_begin::<I, S, R>)?;
    linker.func_wrap("env", "debug_group_end", debug_group_end::<I, S, R>)?;

    // State query functions
    linker.func_wrap("env", "debug_is_paused", debug_is_paused::<I, S, R>)?;
    linker.func_wrap(
        "env",
        "debug_get_time_scale",
        debug_get_time_scale::<I, S, R>,
    )?;

    // Note: Change callbacks are handled via exported on_debug_change() function
    // No FFI registration needed - console looks for the export directly

    Ok(())
}

fn debug_group_begin<I, S, R>(
    mut caller: Caller<'_, WasmGameContext<I, S, R>>,
    name_ptr: u32,
    name_len: u32,
) where
    I: ConsoleInput,
    S: Send + Default + 'static,
    R: ConsoleRollbackState,
    WasmGameContext<I, S, R>: HasDebugRegistry,
{
    if let Some(name) = read_string(&caller, name_ptr, name_len) {
        caller.data_mut().debug_registry_mut().group_begin(&name);
    }
}

fn debug_group_end<I, S, R>(mut caller: Caller<'_, WasmGameContext<I, S, R>>)
where
    I: ConsoleInput,
    S: Send + Default + 'static,
    R: ConsoleRollbackState,
    WasmGameContext<I, S, R>: HasDebugRegistry,
{
    caller.data_mut().debug_registry_mut().group_end();
}

/// Query if the game is currently paused (debug mode)
///
/// Returns 1 if paused, 0 if running normally.
/// Note: This reads from frame controller state, which is stored separately.
/// For now, always returns 0 (not paused) - actual implementation requires
/// integration with the frame controller.
fn debug_is_paused<I, S, R>(_caller: Caller<'_, WasmGameContext<I, S, R>>) -> i32
where
    I: ConsoleInput,
    S: Send + Default + 'static,
    R: ConsoleRollbackState,
{
    // TODO: Read from frame controller state once integrated
    0
}

/// Get the current time scale (1.0 = normal, 0.5 = half speed, etc.)
///
/// Note: This reads from frame controller state, which is stored separately.
/// For now, always returns 1.0 - actual implementation requires
/// integration with the frame controller.
fn debug_get_time_scale<I, S, R>(_caller: Caller<'_, WasmGameContext<I, S, R>>) -> f32
where
    I: ConsoleInput,
    S: Send + Default + 'static,
    R: ConsoleRollbackState,
{
    // TODO: Read from frame controller state once integrated
    1.0
}
