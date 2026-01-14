//! System and timing FFI functions

use wasmtime::Caller;

use crate::console::{ConsoleInput, ConsoleRollbackState};
use crate::wasm::{WasmGameContext, read_bytes_from_memory};

/// Get delta time since last tick (seconds)
pub(super) fn delta_time<I: ConsoleInput, S, R: ConsoleRollbackState>(
    caller: Caller<'_, WasmGameContext<I, S, R>>,
) -> f32 {
    caller.data().game.delta_time
}

/// Get elapsed time since game start (seconds)
pub(super) fn elapsed_time<I: ConsoleInput, S, R: ConsoleRollbackState>(
    caller: Caller<'_, WasmGameContext<I, S, R>>,
) -> f32 {
    caller.data().game.elapsed_time
}

/// Get current tick number
pub(super) fn tick_count<I: ConsoleInput, S, R: ConsoleRollbackState>(
    caller: Caller<'_, WasmGameContext<I, S, R>>,
) -> u64 {
    caller.data().game.tick_count
}

/// Log a message from WASM
pub(super) fn log_message<I: ConsoleInput, S, R: ConsoleRollbackState>(
    caller: Caller<'_, WasmGameContext<I, S, R>>,
    ptr: u32,
    len: u32,
) {
    if let Some(memory) = caller.data().game.memory
        && let Ok(bytes) = read_bytes_from_memory(memory, &caller, ptr, len)
        && let Ok(msg) = std::str::from_utf8(&bytes)
    {
        tracing::info!("[GAME] {}", msg);
    }
}

/// Request to quit to the library
pub(super) fn quit<I: ConsoleInput, S, R: ConsoleRollbackState>(
    mut caller: Caller<'_, WasmGameContext<I, S, R>>,
) {
    caller.data_mut().game.quit_requested = true;
}
