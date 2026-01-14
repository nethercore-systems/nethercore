//! Session and multiplayer FFI functions

use wasmtime::Caller;

use crate::console::{ConsoleInput, ConsoleRollbackState};
use crate::wasm::WasmGameContext;

/// Get number of players in session
pub(super) fn player_count<I: ConsoleInput, S, R: ConsoleRollbackState>(
    caller: Caller<'_, WasmGameContext<I, S, R>>,
) -> u32 {
    caller.data().game.player_count
}

/// Get bitmask of local players
pub(super) fn local_player_mask<I: ConsoleInput, S, R: ConsoleRollbackState>(
    caller: Caller<'_, WasmGameContext<I, S, R>>,
) -> u32 {
    caller.data().game.local_player_mask
}

/// Get local player handle for netplay (0-3)
///
/// Returns 0xFF if not connected (single player or pre-handshake).
/// This is only valid after NCHS handshake completes and before post_connect() is called.
pub(super) fn player_handle<I: ConsoleInput, S, R: ConsoleRollbackState>(
    caller: Caller<'_, WasmGameContext<I, S, R>>,
) -> u32 {
    caller
        .data()
        .game
        .local_player_handle
        .map(|h| h as u32)
        .unwrap_or(0xFF)
}

/// Check if connected to a netplay session
///
/// Returns 1 if connected (NCHS handshake complete), 0 otherwise.
pub(super) fn is_connected<I: ConsoleInput, S, R: ConsoleRollbackState>(
    caller: Caller<'_, WasmGameContext<I, S, R>>,
) -> u32 {
    if caller.data().game.local_player_handle.is_some() {
        1
    } else {
        0
    }
}
