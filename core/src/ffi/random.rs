//! Random number generation FFI functions

use wasmtime::Caller;

use crate::console::{ConsoleInput, ConsoleRollbackState};
use crate::wasm::WasmGameContext;

/// Generate deterministic random u32
pub(super) fn random<I: ConsoleInput, S, R: ConsoleRollbackState>(
    mut caller: Caller<'_, WasmGameContext<I, S, R>>,
) -> u32 {
    caller.data_mut().game.random()
}

/// Generate deterministic random i32 in range [min, max)
pub(super) fn random_range<I: ConsoleInput, S, R: ConsoleRollbackState>(
    mut caller: Caller<'_, WasmGameContext<I, S, R>>,
    min: i32,
    max: i32,
) -> i32 {
    if min >= max {
        return min;
    }
    let range = (max - min) as u32;
    min + (caller.data_mut().game.random() % range) as i32
}

/// Generate deterministic random f32 in range [0.0, 1.0)
pub(super) fn random_f32<I: ConsoleInput, S, R: ConsoleRollbackState>(
    mut caller: Caller<'_, WasmGameContext<I, S, R>>,
) -> f32 {
    (caller.data_mut().game.random() as f64 / (u32::MAX as f64 + 1.0)) as f32
}

/// Generate deterministic random f32 in range [min, max)
pub(super) fn random_f32_range<I: ConsoleInput, S, R: ConsoleRollbackState>(
    mut caller: Caller<'_, WasmGameContext<I, S, R>>,
    min: f32,
    max: f32,
) -> f32 {
    if min >= max {
        return min;
    }
    let t = (caller.data_mut().game.random() as f64 / (u32::MAX as f64 + 1.0)) as f32;
    min + t * (max - min)
}
