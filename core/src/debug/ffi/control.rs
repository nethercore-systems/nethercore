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

    // Error testing function (for developers to test error recovery)
    linker.func_wrap("env", "debug_trigger_error", debug_trigger_error::<I, S, R>)?;

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

/// Trigger a test error for testing error recovery
///
/// This function intentionally causes a WASM trap to test the error
/// handling and recovery system. Game developers can use this to verify
/// their games handle errors gracefully.
///
/// # Arguments
/// * `error_type` - Type of error to trigger:
///   - 0: Panic (unreachable trap)
///   - 1: Out of bounds memory access (simulated)
///   - 2: Stack overflow (simulated)
/// * `message_ptr` - Pointer to optional custom message string
/// * `message_len` - Length of the message (0 for default message)
fn debug_trigger_error<I, S, R>(
    caller: Caller<'_, WasmGameContext<I, S, R>>,
    error_type: u32,
    message_ptr: u32,
    message_len: u32,
) where
    I: ConsoleInput,
    S: Send + Default + 'static,
    R: ConsoleRollbackState,
    WasmGameContext<I, S, R>: HasDebugRegistry,
{
    // Read custom message if provided
    let custom_msg = if message_len > 0 {
        read_string(&caller, message_ptr, message_len).unwrap_or_else(|| "Test error".to_string())
    } else {
        "Test error triggered by debug_trigger_error()".to_string()
    };

    // Log that this was intentional
    tracing::warn!(
        "debug_trigger_error called (type={}): {}",
        error_type,
        custom_msg
    );

    // Trigger the appropriate error type
    match error_type {
        0 => {
            // Panic - causes WASM unreachable trap
            panic!("{}", custom_msg);
        }
        1 => {
            // Simulated out of bounds - also panics with descriptive message
            panic!(
                "Simulated out of bounds memory access at offset 0xDEADBEEF: {}",
                custom_msg
            );
        }
        2 => {
            // Simulated stack overflow
            panic!("Simulated stack overflow: {}", custom_msg);
        }
        _ => {
            // Unknown error type - still trigger an error
            panic!("Unknown error type {}: {}", error_type, custom_msg);
        }
    }
}
