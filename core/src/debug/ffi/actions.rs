//! Debug action registration FFI functions
//!
//! Functions for registering callable actions (buttons) from WASM games.

use anyhow::Result;
use wasmtime::{Caller, Linker};

use crate::console::{ConsoleInput, ConsoleRollbackState};
use crate::wasm::WasmGameContext;

use super::read_string;
use super::register::HasDebugRegistry;

/// Register debug action FFI functions
pub(super) fn register<I, S, R>(linker: &mut Linker<WasmGameContext<I, S, R>>) -> Result<()>
where
    I: ConsoleInput,
    S: Send + Default + 'static,
    R: ConsoleRollbackState,
    WasmGameContext<I, S, R>: HasDebugRegistry,
{
    // Simple action registration (no parameters)
    linker.func_wrap(
        "env",
        "debug_register_action",
        debug_register_action::<I, S, R>,
    )?;

    // Builder pattern for actions with parameters
    linker.func_wrap("env", "debug_action_begin", debug_action_begin::<I, S, R>)?;
    linker.func_wrap(
        "env",
        "debug_action_param_i32",
        debug_action_param_i32::<I, S, R>,
    )?;
    linker.func_wrap(
        "env",
        "debug_action_param_f32",
        debug_action_param_f32::<I, S, R>,
    )?;
    linker.func_wrap("env", "debug_action_end", debug_action_end::<I, S, R>)?;

    Ok(())
}

// =============================================================================
// Action Registration Functions
// =============================================================================

/// Register a simple action with no parameters
fn debug_register_action<I, S, R>(
    mut caller: Caller<'_, WasmGameContext<I, S, R>>,
    name_ptr: u32,
    name_len: u32,
    func_name_ptr: u32,
    func_name_len: u32,
) where
    I: ConsoleInput,
    S: Send + Default + 'static,
    R: ConsoleRollbackState,
    WasmGameContext<I, S, R>: HasDebugRegistry,
{
    let name = match read_string(&caller, name_ptr, name_len) {
        Some(n) => n,
        None => return,
    };
    let func_name = match read_string(&caller, func_name_ptr, func_name_len) {
        Some(n) => n,
        None => return,
    };

    caller
        .data_mut()
        .debug_registry_mut()
        .register_action(&name, &func_name);
}

/// Begin building an action with parameters
fn debug_action_begin<I, S, R>(
    mut caller: Caller<'_, WasmGameContext<I, S, R>>,
    name_ptr: u32,
    name_len: u32,
    func_name_ptr: u32,
    func_name_len: u32,
) where
    I: ConsoleInput,
    S: Send + Default + 'static,
    R: ConsoleRollbackState,
    WasmGameContext<I, S, R>: HasDebugRegistry,
{
    let name = match read_string(&caller, name_ptr, name_len) {
        Some(n) => n,
        None => return,
    };
    let func_name = match read_string(&caller, func_name_ptr, func_name_len) {
        Some(n) => n,
        None => return,
    };

    caller
        .data_mut()
        .debug_registry_mut()
        .action_begin(&name, &func_name);
}

/// Add an i32 parameter to the pending action
fn debug_action_param_i32<I, S, R>(
    mut caller: Caller<'_, WasmGameContext<I, S, R>>,
    name_ptr: u32,
    name_len: u32,
    default_value: i32,
) where
    I: ConsoleInput,
    S: Send + Default + 'static,
    R: ConsoleRollbackState,
    WasmGameContext<I, S, R>: HasDebugRegistry,
{
    let name = match read_string(&caller, name_ptr, name_len) {
        Some(n) => n,
        None => return,
    };

    caller
        .data_mut()
        .debug_registry_mut()
        .action_param_i32(&name, default_value);
}

/// Add an f32 parameter to the pending action
fn debug_action_param_f32<I, S, R>(
    mut caller: Caller<'_, WasmGameContext<I, S, R>>,
    name_ptr: u32,
    name_len: u32,
    default_value: f32,
) where
    I: ConsoleInput,
    S: Send + Default + 'static,
    R: ConsoleRollbackState,
    WasmGameContext<I, S, R>: HasDebugRegistry,
{
    let name = match read_string(&caller, name_ptr, name_len) {
        Some(n) => n,
        None => return,
    };

    caller
        .data_mut()
        .debug_registry_mut()
        .action_param_f32(&name, default_value);
}

/// Finish building the pending action
fn debug_action_end<I, S, R>(mut caller: Caller<'_, WasmGameContext<I, S, R>>)
where
    I: ConsoleInput,
    S: Send + Default + 'static,
    R: ConsoleRollbackState,
    WasmGameContext<I, S, R>: HasDebugRegistry,
{
    caller.data_mut().debug_registry_mut().action_end();
}
