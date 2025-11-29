//! Common FFI host functions
//!
//! These functions are available to all fantasy consoles.
//! Console-specific FFI functions are registered via the Console trait.

use anyhow::Result;
use wasmtime::{Caller, Linker};

use crate::wasm::GameState;

/// Register common FFI functions with the linker
pub fn register_common_ffi(linker: &mut Linker<GameState>) -> Result<()> {
    // System functions
    linker.func_wrap("env", "delta_time", delta_time)?;
    linker.func_wrap("env", "elapsed_time", elapsed_time)?;
    linker.func_wrap("env", "tick_count", tick_count)?;
    linker.func_wrap("env", "log", log_message)?;

    // Rollback functions
    linker.func_wrap("env", "random", random)?;

    // Session functions
    linker.func_wrap("env", "player_count", player_count)?;
    linker.func_wrap("env", "local_player_mask", local_player_mask)?;

    Ok(())
}

/// Get delta time since last tick (seconds)
fn delta_time(caller: Caller<'_, GameState>) -> f32 {
    caller.data().delta_time
}

/// Get elapsed time since game start (seconds)
fn elapsed_time(caller: Caller<'_, GameState>) -> f32 {
    caller.data().elapsed_time
}

/// Get current tick number
fn tick_count(caller: Caller<'_, GameState>) -> u64 {
    caller.data().tick_count
}

/// Log a message from WASM
fn log_message(caller: Caller<'_, GameState>, ptr: u32, len: u32) {
    if let Some(memory) = caller.data().memory {
        let data = memory.data(&caller);
        let ptr = ptr as usize;
        let len = len as usize;
        if ptr + len <= data.len() {
            if let Ok(msg) = std::str::from_utf8(&data[ptr..ptr + len]) {
                log::info!("[GAME] {}", msg);
            }
        }
    }
}

/// Generate deterministic random u32
fn random(mut caller: Caller<'_, GameState>) -> u32 {
    caller.data_mut().random()
}

/// Get number of players in session
fn player_count(caller: Caller<'_, GameState>) -> u32 {
    caller.data().player_count
}

/// Get bitmask of local players
fn local_player_mask(caller: Caller<'_, GameState>) -> u32 {
    caller.data().local_player_mask
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rng_determinism() {
        let mut state1 = GameState::new();
        let mut state2 = GameState::new();

        state1.seed_rng(12345);
        state2.seed_rng(12345);

        for _ in 0..100 {
            assert_eq!(state1.random(), state2.random());
        }
    }

    #[test]
    fn test_rng_different_seeds() {
        let mut state1 = GameState::new();
        let mut state2 = GameState::new();

        state1.seed_rng(12345);
        state2.seed_rng(67890);

        // Very unlikely to match with different seeds
        let mut matches = 0;
        for _ in 0..100 {
            if state1.random() == state2.random() {
                matches += 1;
            }
        }
        assert!(matches < 5); // Allow some coincidental matches
    }
}
