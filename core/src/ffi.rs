//! Common FFI host functions
//!
//! These functions are available to all fantasy consoles.
//! Console-specific FFI functions are registered via the Console trait.

use anyhow::Result;
use wasmtime::{Caller, Linker};

use crate::wasm::{GameState, MAX_SAVE_SIZE, MAX_SAVE_SLOTS};

/// Register common FFI functions with the linker
pub fn register_common_ffi(linker: &mut Linker<GameState>) -> Result<()> {
    // System functions
    linker.func_wrap("env", "delta_time", delta_time)?;
    linker.func_wrap("env", "elapsed_time", elapsed_time)?;
    linker.func_wrap("env", "tick_count", tick_count)?;
    linker.func_wrap("env", "log", log_message)?;
    linker.func_wrap("env", "quit", quit)?;

    // Rollback functions
    linker.func_wrap("env", "random", random)?;

    // Save data functions
    linker.func_wrap("env", "save", save)?;
    linker.func_wrap("env", "load", load)?;
    linker.func_wrap("env", "delete", delete)?;

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

/// Request to quit to the library
fn quit(mut caller: Caller<'_, GameState>) {
    caller.data_mut().quit_requested = true;
}

/// Save data to a slot (0-7)
///
/// Returns: 0 = success, 1 = invalid slot, 2 = data too large
fn save(mut caller: Caller<'_, GameState>, slot: u32, data_ptr: u32, data_len: u32) -> u32 {
    let slot = slot as usize;
    let data_len = data_len as usize;

    // Validate slot
    if slot >= MAX_SAVE_SLOTS {
        return 1;
    }

    // Validate data size
    if data_len > MAX_SAVE_SIZE {
        return 2;
    }

    // Read data from WASM memory
    let data = if let Some(memory) = caller.data().memory {
        let mem_data = memory.data(&caller);
        let ptr = data_ptr as usize;
        if ptr + data_len <= mem_data.len() {
            mem_data[ptr..ptr + data_len].to_vec()
        } else {
            return 2; // Invalid memory access
        }
    } else {
        return 2;
    };

    // Store the data
    caller.data_mut().save_data[slot] = Some(data);
    0
}

/// Load data from a slot (0-7)
///
/// Returns: bytes read, or 0 if slot is empty/invalid
fn load(mut caller: Caller<'_, GameState>, slot: u32, data_ptr: u32, max_len: u32) -> u32 {
    let slot = slot as usize;
    let max_len = max_len as usize;

    // Validate slot
    if slot >= MAX_SAVE_SLOTS {
        return 0;
    }

    // Get saved data (clone to avoid borrow issues)
    let data = match &caller.data().save_data[slot] {
        Some(d) => d.clone(),
        None => return 0,
    };

    // Calculate actual length to copy
    let copy_len = data.len().min(max_len);

    // Write data to WASM memory
    if let Some(memory) = caller.data().memory {
        let mem_data = memory.data_mut(&mut caller);
        let ptr = data_ptr as usize;
        if ptr + copy_len <= mem_data.len() {
            mem_data[ptr..ptr + copy_len].copy_from_slice(&data[..copy_len]);
            copy_len as u32
        } else {
            0
        }
    } else {
        0
    }
}

/// Delete data in a slot (0-7)
///
/// Returns: 0 = success, 1 = invalid slot
fn delete(mut caller: Caller<'_, GameState>, slot: u32) -> u32 {
    let slot = slot as usize;

    // Validate slot
    if slot >= MAX_SAVE_SLOTS {
        return 1;
    }

    caller.data_mut().save_data[slot] = None;
    0
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
    use wasmtime::{Engine, Store};

    // ============================================================================
    // FFI Registration Tests
    // ============================================================================

    #[test]
    fn test_register_common_ffi() {
        let engine = Engine::default();
        let mut linker = Linker::new(&engine);
        let result = register_common_ffi(&mut linker);
        assert!(result.is_ok());
    }

    #[test]
    fn test_ffi_functions_registered() {
        let engine = Engine::default();
        let mut linker = Linker::new(&engine);
        register_common_ffi(&mut linker).unwrap();

        // Verify key functions are registered by checking module "env"
        // We can't easily check if specific functions are in the linker without
        // trying to instantiate a module that imports them, but registration
        // should succeed without errors.
    }

    #[test]
    fn test_ffi_with_wasm_module() {
        // Create a minimal WASM module that imports common FFI functions
        let engine = Engine::default();
        let mut linker = Linker::new(&engine);
        register_common_ffi(&mut linker).unwrap();

        // WAT module that imports and calls delta_time
        let wat = r#"
            (module
                (import "env" "delta_time" (func $delta_time (result f32)))
                (import "env" "elapsed_time" (func $elapsed_time (result f32)))
                (import "env" "tick_count" (func $tick_count (result i64)))
                (import "env" "player_count" (func $player_count (result i32)))
                (import "env" "local_player_mask" (func $local_player_mask (result i32)))
                (memory (export "memory") 1)
            )
        "#;
        let wasm = wat::parse_str(wat).unwrap();
        let module = wasmtime::Module::new(&engine, wasm).unwrap();

        let mut store = Store::new(&engine, GameState::new());
        let result = linker.instantiate(&mut store, &module);
        assert!(result.is_ok());
    }

    #[test]
    fn test_ffi_random_from_wasm() {
        let engine = Engine::default();
        let mut linker = Linker::new(&engine);
        register_common_ffi(&mut linker).unwrap();

        // WAT module that imports random
        let wat = r#"
            (module
                (import "env" "random" (func $random (result i32)))
                (memory (export "memory") 1)
                (func (export "get_random") (result i32)
                    call $random
                )
            )
        "#;
        let wasm = wat::parse_str(wat).unwrap();
        let module = wasmtime::Module::new(&engine, wasm).unwrap();

        let mut store = Store::new(&engine, GameState::new());
        store.data_mut().seed_rng(42);

        let instance = linker.instantiate(&mut store, &module).unwrap();
        let get_random = instance
            .get_typed_func::<(), i32>(&mut store, "get_random")
            .unwrap();

        // Call random and verify it returns a value
        let val1 = get_random.call(&mut store, ()).unwrap();
        let val2 = get_random.call(&mut store, ()).unwrap();

        // Values should be different (very unlikely to be the same)
        assert_ne!(val1, val2);
    }

    #[test]
    fn test_ffi_quit_from_wasm() {
        let engine = Engine::default();
        let mut linker = Linker::new(&engine);
        register_common_ffi(&mut linker).unwrap();

        // WAT module that imports quit
        let wat = r#"
            (module
                (import "env" "quit" (func $quit))
                (memory (export "memory") 1)
                (func (export "request_quit")
                    call $quit
                )
            )
        "#;
        let wasm = wat::parse_str(wat).unwrap();
        let module = wasmtime::Module::new(&engine, wasm).unwrap();

        let mut store = Store::new(&engine, GameState::new());
        assert!(!store.data().quit_requested);

        let instance = linker.instantiate(&mut store, &module).unwrap();
        let request_quit = instance
            .get_typed_func::<(), ()>(&mut store, "request_quit")
            .unwrap();

        request_quit.call(&mut store, ()).unwrap();
        assert!(store.data().quit_requested);
    }

    // ============================================================================
    // RNG Tests
    // ============================================================================

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

    #[test]
    fn test_rng_sequence_reproducible() {
        let mut state = GameState::new();
        state.seed_rng(999);

        let sequence1: Vec<u32> = (0..10).map(|_| state.random()).collect();

        // Re-seed and regenerate
        state.seed_rng(999);
        let sequence2: Vec<u32> = (0..10).map(|_| state.random()).collect();

        assert_eq!(sequence1, sequence2);
    }

    #[test]
    fn test_rng_zero_seed() {
        let mut state = GameState::new();
        state.seed_rng(0);

        // Should still produce values (zero seed is valid)
        let val1 = state.random();
        let val2 = state.random();
        assert_ne!(val1, val2);
    }

    #[test]
    fn test_rng_max_seed() {
        let mut state = GameState::new();
        state.seed_rng(u64::MAX);

        // Should work with maximum seed value
        let val1 = state.random();
        let val2 = state.random();
        assert_ne!(val1, val2);
    }

    // ============================================================================
    // Save Data Tests
    // ============================================================================

    #[test]
    fn test_save_data_slots() {
        let mut state = GameState::new();

        // Initially all slots are empty
        for slot in 0..MAX_SAVE_SLOTS {
            assert!(state.save_data[slot].is_none());
        }

        // Save some data
        let test_data = vec![1, 2, 3, 4, 5];
        state.save_data[0] = Some(test_data.clone());
        assert_eq!(state.save_data[0], Some(test_data.clone()));

        // Other slots still empty
        for slot in 1..MAX_SAVE_SLOTS {
            assert!(state.save_data[slot].is_none());
        }

        // Delete data
        state.save_data[0] = None;
        assert!(state.save_data[0].is_none());
    }

    #[test]
    fn test_save_data_all_slots() {
        let mut state = GameState::new();

        // Fill all slots
        for slot in 0..MAX_SAVE_SLOTS {
            state.save_data[slot] = Some(vec![slot as u8; 10]);
        }

        // Verify all slots
        for slot in 0..MAX_SAVE_SLOTS {
            let data = state.save_data[slot].as_ref().unwrap();
            assert_eq!(data.len(), 10);
            assert!(data.iter().all(|&b| b == slot as u8));
        }
    }

    #[test]
    fn test_save_data_overwrite() {
        let mut state = GameState::new();

        state.save_data[0] = Some(vec![1, 2, 3]);
        assert_eq!(state.save_data[0], Some(vec![1, 2, 3]));

        // Overwrite
        state.save_data[0] = Some(vec![4, 5, 6, 7]);
        assert_eq!(state.save_data[0], Some(vec![4, 5, 6, 7]));
    }

    // ============================================================================
    // Quit Tests
    // ============================================================================

    #[test]
    fn test_quit_requested() {
        let mut state = GameState::new();
        assert!(!state.quit_requested);

        state.quit_requested = true;
        assert!(state.quit_requested);
    }

    // ============================================================================
    // Time/Counter Tests
    // ============================================================================

    #[test]
    fn test_initial_timing_values() {
        let state = GameState::new();
        assert_eq!(state.tick_count, 0);
        assert_eq!(state.elapsed_time, 0.0);
        assert_eq!(state.delta_time, 0.0);
    }

    #[test]
    fn test_player_info_defaults() {
        let state = GameState::new();
        assert_eq!(state.player_count, 1);
        assert_eq!(state.local_player_mask, 1); // Player 0 is local
    }

    #[test]
    fn test_player_info_multiplayer() {
        let mut state = GameState::new();
        state.player_count = 4;
        state.local_player_mask = 0b0101; // Players 0 and 2 are local

        assert_eq!(state.player_count, 4);
        assert!(state.local_player_mask & (1 << 0) != 0); // Player 0 local
        assert!(state.local_player_mask & (1 << 1) == 0); // Player 1 remote
        assert!(state.local_player_mask & (1 << 2) != 0); // Player 2 local
        assert!(state.local_player_mask & (1 << 3) == 0); // Player 3 remote
    }
}
