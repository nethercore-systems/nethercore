//! Tests for FFI functions

use super::save::persist_controller_mapped_slot_from_state;
use super::*;
use crate::test_utils::TestInput;
use crate::wasm::GameState;
use crate::wasm::WasmGameContext;
use wasmtime::{Engine, Linker, Store};

// ============================================================================
// FFI Registration Tests
// ============================================================================

#[test]
fn test_register_common_ffi() {
    let engine = Engine::default();
    let mut linker: Linker<WasmGameContext<TestInput, ()>> = Linker::new(&engine);
    let result = register_common_ffi(&mut linker);
    assert!(result.is_ok());
}

#[test]
fn test_ffi_functions_registered() {
    let engine = Engine::default();
    let mut linker: Linker<WasmGameContext<TestInput, ()>> = Linker::new(&engine);
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
    let mut linker: Linker<WasmGameContext<TestInput, ()>> = Linker::new(&engine);
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

    let mut store = Store::new(&engine, WasmGameContext::<TestInput, ()>::new());
    let result = linker.instantiate(&mut store, &module);
    assert!(result.is_ok());
}

#[test]
fn test_ffi_random_from_wasm() {
    let engine = Engine::default();
    let mut linker: Linker<WasmGameContext<TestInput, ()>> = Linker::new(&engine);
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

    let mut store = Store::new(&engine, WasmGameContext::<TestInput, ()>::new());
    store.data_mut().game.seed_rng(42);

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
    let mut linker: Linker<WasmGameContext<TestInput, ()>> = Linker::new(&engine);
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

    let mut store = Store::new(&engine, WasmGameContext::<TestInput, ()>::new());
    assert!(!store.data().game.quit_requested);

    let instance = linker.instantiate(&mut store, &module).unwrap();
    let request_quit = instance
        .get_typed_func::<(), ()>(&mut store, "request_quit")
        .unwrap();

    request_quit.call(&mut store, ()).unwrap();
    assert!(store.data().game.quit_requested);
}

// ============================================================================
// RNG Tests
// ============================================================================

#[test]
fn test_rng_determinism() {
    let mut state1 = GameState::<TestInput>::new();
    let mut state2 = GameState::<TestInput>::new();

    state1.seed_rng(12345);
    state2.seed_rng(12345);

    for _ in 0..100 {
        assert_eq!(state1.random(), state2.random());
    }
}

#[test]
fn test_rng_different_seeds() {
    let mut state1 = GameState::<TestInput>::new();
    let mut state2 = GameState::<TestInput>::new();

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
    let mut state = GameState::<TestInput>::new();
    state.seed_rng(999);

    let sequence1: Vec<u32> = (0..10).map(|_| state.random()).collect();

    // Re-seed and regenerate
    state.seed_rng(999);
    let sequence2: Vec<u32> = (0..10).map(|_| state.random()).collect();

    assert_eq!(sequence1, sequence2);
}

#[test]
fn test_rng_zero_seed() {
    let mut state = GameState::<TestInput>::new();
    state.seed_rng(0);

    // Should still produce values (zero seed is valid)
    let val1 = state.random();
    let val2 = state.random();
    assert_ne!(val1, val2);
}

#[test]
fn test_rng_max_seed() {
    let mut state = GameState::<TestInput>::new();
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
fn test_save_persists_only_local_slots() {
    let tmp = tempfile::TempDir::new().unwrap();
    let save_path = tmp.path().join("t.ncsav");

    let mut ctx = WasmGameContext::<TestInput, (), ()>::new();
    ctx.game.player_count = 4;
    ctx.game.local_player_mask = 0b0100;
    ctx.save_store = Some(crate::save_store::SaveStore::new(save_path.clone()));

    ctx.game.save_data[2] = Some(vec![7]);
    persist_controller_mapped_slot_from_state(&mut ctx, 2);

    ctx.game.save_data[1] = Some(vec![9]);
    persist_controller_mapped_slot_from_state(&mut ctx, 1);

    let store = crate::save_store::SaveStore::load_or_new(save_path).unwrap();
    assert_eq!(store.controller_slot(0).unwrap(), &[7]);
    assert!(store.controller_slot(1).is_none());
    assert!(store.controller_slot(2).is_none());
    assert!(store.controller_slot(3).is_none());
}

#[test]
fn test_delete_persists_for_local_slot() {
    let tmp = tempfile::TempDir::new().unwrap();
    let save_path = tmp.path().join("t.ncsav");

    let mut ctx = WasmGameContext::<TestInput, (), ()>::new();
    ctx.game.player_count = 4;
    ctx.game.local_player_mask = 0b0100;
    ctx.save_store = Some(crate::save_store::SaveStore::new(save_path.clone()));

    ctx.game.save_data[2] = Some(vec![7]);
    persist_controller_mapped_slot_from_state(&mut ctx, 2);
    ctx.game.save_data[2] = None;
    persist_controller_mapped_slot_from_state(&mut ctx, 2);

    let store = crate::save_store::SaveStore::load_or_new(save_path).unwrap();
    assert!(store.controller_slot(0).is_none());
}

#[test]
fn test_save_data_slots() {
    let mut state = GameState::<TestInput>::new();

    // Initially all slots are empty
    for slot in 0..crate::wasm::MAX_SAVE_SLOTS {
        assert!(state.save_data[slot].is_none());
    }

    // Save some data
    let test_data = vec![1, 2, 3, 4, 5];
    state.save_data[0] = Some(test_data.clone());
    assert_eq!(state.save_data[0], Some(test_data.clone()));

    // Other slots still empty
    for slot in 1..crate::wasm::MAX_SAVE_SLOTS {
        assert!(state.save_data[slot].is_none());
    }

    // Delete data
    state.save_data[0] = None;
    assert!(state.save_data[0].is_none());
}

#[test]
fn test_save_data_all_slots() {
    let mut state = GameState::<TestInput>::new();

    // Fill all slots
    for slot in 0..crate::wasm::MAX_SAVE_SLOTS {
        state.save_data[slot] = Some(vec![slot as u8; 10]);
    }

    // Verify all slots
    for slot in 0..crate::wasm::MAX_SAVE_SLOTS {
        let data = state.save_data[slot].as_ref().unwrap();
        assert_eq!(data.len(), 10);
        assert!(data.iter().all(|&b| b == slot as u8));
    }
}

#[test]
fn test_save_data_overwrite() {
    let mut state = GameState::<TestInput>::new();

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
    let mut state = GameState::<TestInput>::new();
    assert!(!state.quit_requested);

    state.quit_requested = true;
    assert!(state.quit_requested);
}

// ============================================================================
// Time/Counter Tests
// ============================================================================

#[test]
fn test_initial_timing_values() {
    let state = GameState::<TestInput>::new();
    assert_eq!(state.tick_count, 0);
    assert_eq!(state.elapsed_time, 0.0);
    assert_eq!(state.delta_time, 0.0);
}

#[test]
fn test_player_info_defaults() {
    let state = GameState::<TestInput>::new();
    assert_eq!(state.player_count, 1);
    assert_eq!(state.local_player_mask, 1); // Player 0 is local
}

#[test]
fn test_player_info_multiplayer() {
    let mut state = GameState::<TestInput>::new();
    state.player_count = 4;
    state.local_player_mask = 0b0101; // Players 0 and 2 are local

    assert_eq!(state.player_count, 4);
    assert!(state.local_player_mask & (1 << 0) != 0); // Player 0 local
    assert!(state.local_player_mask & (1 << 1) == 0); // Player 1 remote
    assert!(state.local_player_mask & (1 << 2) != 0); // Player 2 local
    assert!(state.local_player_mask & (1 << 3) == 0); // Player 3 remote
}

// ============================================================================
// WASM Memory Error Path Tests
// ============================================================================

#[test]
fn test_log_message_out_of_bounds() {
    let engine = Engine::default();
    let mut linker: Linker<WasmGameContext<TestInput, ()>> = Linker::new(&engine);
    register_common_ffi(&mut linker).unwrap();

    // WAT module that calls log with out-of-bounds pointer
    let wat = r#"
        (module
            (import "env" "log" (func $log (param i32 i32)))
            (memory (export "memory") 1)
            (func (export "test_oob_log")
                ;; Try to log from way past the end of memory
                ;; 1 page = 65536 bytes, so 100000 is out of bounds
                (call $log (i32.const 100000) (i32.const 10))
            )
        )
    "#;
    let wasm = wat::parse_str(wat).unwrap();
    let module = wasmtime::Module::new(&engine, wasm).unwrap();

    let mut store = Store::new(&engine, WasmGameContext::<TestInput, ()>::new());
    // Set up memory reference
    let instance = linker.instantiate(&mut store, &module).unwrap();
    if let Some(memory) = instance.get_memory(&mut store, "memory") {
        store.data_mut().game.memory = Some(memory);
    }

    let test_oob_log = instance
        .get_typed_func::<(), ()>(&mut store, "test_oob_log")
        .unwrap();

    // Should not panic - just silently fail
    let result = test_oob_log.call(&mut store, ());
    assert!(result.is_ok());
}

#[test]
fn test_log_message_wrapping_overflow() {
    let engine = Engine::default();
    let mut linker: Linker<WasmGameContext<TestInput, ()>> = Linker::new(&engine);
    register_common_ffi(&mut linker).unwrap();

    // WAT module that tries to cause ptr + len overflow
    let wat = r#"
        (module
            (import "env" "log" (func $log (param i32 i32)))
            (memory (export "memory") 1)
            (func (export "test_overflow")
                ;; ptr near max u32, len that would overflow
                (call $log (i32.const 4294967290) (i32.const 100))
            )
        )
    "#;
    let wasm = wat::parse_str(wat).unwrap();
    let module = wasmtime::Module::new(&engine, wasm).unwrap();

    let mut store = Store::new(&engine, WasmGameContext::<TestInput, ()>::new());
    let instance = linker.instantiate(&mut store, &module).unwrap();
    if let Some(memory) = instance.get_memory(&mut store, "memory") {
        store.data_mut().game.memory = Some(memory);
    }

    let test_overflow = instance
        .get_typed_func::<(), ()>(&mut store, "test_overflow")
        .unwrap();

    // Should not panic
    let result = test_overflow.call(&mut store, ());
    assert!(result.is_ok());
}

#[test]
fn test_save_invalid_slot() {
    let engine = Engine::default();
    let mut linker: Linker<WasmGameContext<TestInput, ()>> = Linker::new(&engine);
    register_common_ffi(&mut linker).unwrap();

    // WAT module that calls save with invalid slot
    let wat = r#"
        (module
            (import "env" "save" (func $save (param i32 i32 i32) (result i32)))
            (memory (export "memory") 1)
            (func (export "test_invalid_slot") (result i32)
                ;; Try to save to slot 100 (invalid, max is 7)
                (call $save (i32.const 100) (i32.const 0) (i32.const 10))
            )
        )
    "#;
    let wasm = wat::parse_str(wat).unwrap();
    let module = wasmtime::Module::new(&engine, wasm).unwrap();

    let mut store = Store::new(&engine, WasmGameContext::<TestInput, ()>::new());
    let instance = linker.instantiate(&mut store, &module).unwrap();
    if let Some(memory) = instance.get_memory(&mut store, "memory") {
        store.data_mut().game.memory = Some(memory);
    }

    let test_fn = instance
        .get_typed_func::<(), i32>(&mut store, "test_invalid_slot")
        .unwrap();

    let result = test_fn.call(&mut store, ()).unwrap();
    assert_eq!(result, 1); // 1 = invalid slot
}

#[test]
fn test_save_data_too_large() {
    let engine = Engine::default();
    let mut linker: Linker<WasmGameContext<TestInput, ()>> = Linker::new(&engine);
    register_common_ffi(&mut linker).unwrap();

    // WAT module that tries to save data larger than MAX_SAVE_SIZE
    let wat = r#"
        (module
            (import "env" "save" (func $save (param i32 i32 i32) (result i32)))
            (memory (export "memory") 1)
            (func (export "test_too_large") (result i32)
                ;; Try to save 100KB (max is 64KB)
                (call $save (i32.const 0) (i32.const 0) (i32.const 102400))
            )
        )
    "#;
    let wasm = wat::parse_str(wat).unwrap();
    let module = wasmtime::Module::new(&engine, wasm).unwrap();

    let mut store = Store::new(&engine, WasmGameContext::<TestInput, ()>::new());
    let instance = linker.instantiate(&mut store, &module).unwrap();
    if let Some(memory) = instance.get_memory(&mut store, "memory") {
        store.data_mut().game.memory = Some(memory);
    }

    let test_fn = instance
        .get_typed_func::<(), i32>(&mut store, "test_too_large")
        .unwrap();

    let result = test_fn.call(&mut store, ()).unwrap();
    assert_eq!(result, 2); // 2 = data too large
}

#[test]
fn test_save_out_of_bounds_pointer() {
    let engine = Engine::default();
    let mut linker: Linker<WasmGameContext<TestInput, ()>> = Linker::new(&engine);
    register_common_ffi(&mut linker).unwrap();

    // WAT module that tries to save from out-of-bounds memory
    let wat = r#"
        (module
            (import "env" "save" (func $save (param i32 i32 i32) (result i32)))
            (memory (export "memory") 1)
            (func (export "test_oob_ptr") (result i32)
                ;; Valid slot, but pointer + length exceeds memory
                ;; Memory is 1 page (65536 bytes), try to read from 60000 with length 10000
                (call $save (i32.const 0) (i32.const 60000) (i32.const 10000))
            )
        )
    "#;
    let wasm = wat::parse_str(wat).unwrap();
    let module = wasmtime::Module::new(&engine, wasm).unwrap();

    let mut store = Store::new(&engine, WasmGameContext::<TestInput, ()>::new());
    let instance = linker.instantiate(&mut store, &module).unwrap();
    if let Some(memory) = instance.get_memory(&mut store, "memory") {
        store.data_mut().game.memory = Some(memory);
    }

    let test_fn = instance
        .get_typed_func::<(), i32>(&mut store, "test_oob_ptr")
        .unwrap();

    let result = test_fn.call(&mut store, ()).unwrap();
    assert_eq!(result, 2); // 2 = invalid memory access (same as data too large)
}

#[test]
fn test_load_invalid_slot() {
    let engine = Engine::default();
    let mut linker: Linker<WasmGameContext<TestInput, ()>> = Linker::new(&engine);
    register_common_ffi(&mut linker).unwrap();

    // WAT module that calls load with invalid slot
    let wat = r#"
        (module
            (import "env" "load" (func $load (param i32 i32 i32) (result i32)))
            (memory (export "memory") 1)
            (func (export "test_invalid_slot") (result i32)
                ;; Try to load from slot 100 (invalid)
                (call $load (i32.const 100) (i32.const 0) (i32.const 100))
            )
        )
    "#;
    let wasm = wat::parse_str(wat).unwrap();
    let module = wasmtime::Module::new(&engine, wasm).unwrap();

    let mut store = Store::new(&engine, WasmGameContext::<TestInput, ()>::new());
    let instance = linker.instantiate(&mut store, &module).unwrap();
    if let Some(memory) = instance.get_memory(&mut store, "memory") {
        store.data_mut().game.memory = Some(memory);
    }

    let test_fn = instance
        .get_typed_func::<(), i32>(&mut store, "test_invalid_slot")
        .unwrap();

    let result = test_fn.call(&mut store, ()).unwrap();
    assert_eq!(result, 0); // 0 = no data loaded (invalid slot)
}

#[test]
fn test_load_empty_slot() {
    let engine = Engine::default();
    let mut linker: Linker<WasmGameContext<TestInput, ()>> = Linker::new(&engine);
    register_common_ffi(&mut linker).unwrap();

    // WAT module that calls load on an empty slot
    let wat = r#"
        (module
            (import "env" "load" (func $load (param i32 i32 i32) (result i32)))
            (memory (export "memory") 1)
            (func (export "test_empty_slot") (result i32)
                ;; Try to load from slot 0 (valid but empty)
                (call $load (i32.const 0) (i32.const 0) (i32.const 100))
            )
        )
    "#;
    let wasm = wat::parse_str(wat).unwrap();
    let module = wasmtime::Module::new(&engine, wasm).unwrap();

    let mut store = Store::new(&engine, WasmGameContext::<TestInput, ()>::new());
    let instance = linker.instantiate(&mut store, &module).unwrap();
    if let Some(memory) = instance.get_memory(&mut store, "memory") {
        store.data_mut().game.memory = Some(memory);
    }

    let test_fn = instance
        .get_typed_func::<(), i32>(&mut store, "test_empty_slot")
        .unwrap();

    let result = test_fn.call(&mut store, ()).unwrap();
    assert_eq!(result, 0); // 0 = no data (slot is empty)
}

#[test]
fn test_load_out_of_bounds_pointer() {
    let engine = Engine::default();
    let mut linker = Linker::new(&engine);
    register_common_ffi(&mut linker).unwrap();

    // WAT module that tries to load into out-of-bounds memory
    let wat = r#"
        (module
            (import "env" "load" (func $load (param i32 i32 i32) (result i32)))
            (memory (export "memory") 1)
            (func (export "test_oob_load") (result i32)
                ;; Try to load into out-of-bounds destination
                ;; Memory is 65536 bytes (1 page), ptr 65500 + data 100 = 65600 > 65536
                (call $load (i32.const 0) (i32.const 65500) (i32.const 1000))
            )
        )
    "#;
    let wasm = wat::parse_str(wat).unwrap();
    let module = wasmtime::Module::new(&engine, wasm).unwrap();

    let mut store = Store::new(&engine, WasmGameContext::<TestInput, ()>::new());
    let instance = linker.instantiate(&mut store, &module).unwrap();
    if let Some(memory) = instance.get_memory(&mut store, "memory") {
        store.data_mut().game.memory = Some(memory);
    }
    // Pre-populate slot 0 with 100 bytes of data
    store.data_mut().game.save_data[0] = Some(vec![0xAB; 100]);

    let test_fn = instance
        .get_typed_func::<(), i32>(&mut store, "test_oob_load")
        .unwrap();

    let result = test_fn.call(&mut store, ()).unwrap();
    assert_eq!(result, 0); // 0 = failed to load (out of bounds: 65500 + 100 > 65536)
}

#[test]
fn test_delete_invalid_slot() {
    let engine = Engine::default();
    let mut linker: Linker<WasmGameContext<TestInput, ()>> = Linker::new(&engine);
    register_common_ffi(&mut linker).unwrap();

    // WAT module that calls delete with invalid slot
    let wat = r#"
        (module
            (import "env" "delete" (func $delete (param i32) (result i32)))
            (memory (export "memory") 1)
            (func (export "test_invalid_delete") (result i32)
                ;; Try to delete slot 100 (invalid)
                (call $delete (i32.const 100))
            )
        )
    "#;
    let wasm = wat::parse_str(wat).unwrap();
    let module = wasmtime::Module::new(&engine, wasm).unwrap();

    let mut store = Store::new(&engine, WasmGameContext::<TestInput, ()>::new());
    let instance = linker.instantiate(&mut store, &module).unwrap();

    let test_fn = instance
        .get_typed_func::<(), i32>(&mut store, "test_invalid_delete")
        .unwrap();

    let result = test_fn.call(&mut store, ()).unwrap();
    assert_eq!(result, 1); // 1 = invalid slot
}

#[test]
fn test_save_load_roundtrip() {
    let engine = Engine::default();
    let mut linker: Linker<WasmGameContext<TestInput, ()>> = Linker::new(&engine);
    register_common_ffi(&mut linker).unwrap();

    // WAT module that saves and loads data, verifying it roundtrips correctly
    let wat = r#"
        (module
            (import "env" "save" (func $save (param i32 i32 i32) (result i32)))
            (import "env" "load" (func $load (param i32 i32 i32) (result i32)))
            (memory (export "memory") 1)

            ;; Write test pattern at address 0
            (data (i32.const 0) "\de\ad\be\ef\ca\fe\ba\be")

            (func (export "test_roundtrip") (result i32)
                (local $save_result i32)
                (local $load_result i32)

                ;; Save 8 bytes from address 0 to slot 0
                (local.set $save_result (call $save (i32.const 0) (i32.const 0) (i32.const 8)))

                ;; Clear the source area
                (i64.store (i32.const 0) (i64.const 0))

                ;; Load back into address 100
                (local.set $load_result (call $load (i32.const 0) (i32.const 100) (i32.const 100)))

                ;; Return load result (should be 8)
                (local.get $load_result)
            )
        )
    "#;
    let wasm = wat::parse_str(wat).unwrap();
    let module = wasmtime::Module::new(&engine, wasm).unwrap();

    let mut store = Store::new(&engine, WasmGameContext::<TestInput, ()>::new());
    let instance = linker.instantiate(&mut store, &module).unwrap();
    if let Some(memory) = instance.get_memory(&mut store, "memory") {
        store.data_mut().game.memory = Some(memory);
    }

    let test_fn = instance
        .get_typed_func::<(), i32>(&mut store, "test_roundtrip")
        .unwrap();

    let result = test_fn.call(&mut store, ()).unwrap();
    assert_eq!(result, 8); // 8 bytes loaded

    // Verify the data was loaded correctly at address 100
    let memory = instance.get_memory(&mut store, "memory").unwrap();
    let mem_data = memory.data(&store);
    assert_eq!(
        &mem_data[100..108],
        &[0xDE, 0xAD, 0xBE, 0xEF, 0xCA, 0xFE, 0xBA, 0xBE]
    );
}

#[test]
fn test_save_boundary_slot_values() {
    let engine = Engine::default();
    let mut linker: Linker<WasmGameContext<TestInput, ()>> = Linker::new(&engine);
    register_common_ffi(&mut linker).unwrap();

    // Test slot boundary values (0-7 valid, 8+ invalid)
    let wat = r#"
        (module
            (import "env" "save" (func $save (param i32 i32 i32) (result i32)))
            (memory (export "memory") 1)
            (func (export "test_slot") (param $slot i32) (result i32)
                (call $save (local.get $slot) (i32.const 0) (i32.const 1))
            )
        )
    "#;
    let wasm = wat::parse_str(wat).unwrap();
    let module = wasmtime::Module::new(&engine, wasm).unwrap();

    let mut store = Store::new(&engine, WasmGameContext::<TestInput, ()>::new());
    let instance = linker.instantiate(&mut store, &module).unwrap();
    if let Some(memory) = instance.get_memory(&mut store, "memory") {
        store.data_mut().game.memory = Some(memory);
    }

    let test_fn = instance
        .get_typed_func::<i32, i32>(&mut store, "test_slot")
        .unwrap();

    // Valid slots (0-7) should succeed
    for slot in 0..8 {
        let result = test_fn.call(&mut store, slot).unwrap();
        assert_eq!(result, 0, "Slot {} should be valid", slot);
    }

    // Invalid slot (8) should fail
    let result = test_fn.call(&mut store, 8).unwrap();
    assert_eq!(result, 1, "Slot 8 should be invalid");
}

#[test]
fn test_log_no_memory() {
    let engine = Engine::default();
    let mut linker: Linker<WasmGameContext<TestInput, ()>> = Linker::new(&engine);
    register_common_ffi(&mut linker).unwrap();

    // WAT module without exported memory
    let wat = r#"
        (module
            (import "env" "log" (func $log (param i32 i32)))
            (func (export "test_no_memory")
                (call $log (i32.const 0) (i32.const 10))
            )
        )
    "#;
    let wasm = wat::parse_str(wat).unwrap();
    let module = wasmtime::Module::new(&engine, wasm).unwrap();

    let mut store = Store::new(&engine, WasmGameContext::<TestInput, ()>::new());
    // Intentionally NOT setting memory
    let instance = linker.instantiate(&mut store, &module).unwrap();

    let test_fn = instance
        .get_typed_func::<(), ()>(&mut store, "test_no_memory")
        .unwrap();

    // Should not panic - just silently fail since memory is None
    let result = test_fn.call(&mut store, ());
    assert!(result.is_ok());
}

#[test]
fn test_save_no_memory() {
    let engine = Engine::default();
    let mut linker: Linker<WasmGameContext<TestInput, ()>> = Linker::new(&engine);
    register_common_ffi(&mut linker).unwrap();

    // WAT module without exported memory
    let wat = r#"
        (module
            (import "env" "save" (func $save (param i32 i32 i32) (result i32)))
            (func (export "test_no_memory") (result i32)
                (call $save (i32.const 0) (i32.const 0) (i32.const 10))
            )
        )
    "#;
    let wasm = wat::parse_str(wat).unwrap();
    let module = wasmtime::Module::new(&engine, wasm).unwrap();

    let mut store = Store::new(&engine, WasmGameContext::<TestInput, ()>::new());
    // Intentionally NOT setting memory
    let instance = linker.instantiate(&mut store, &module).unwrap();

    let test_fn = instance
        .get_typed_func::<(), i32>(&mut store, "test_no_memory")
        .unwrap();

    let result = test_fn.call(&mut store, ()).unwrap();
    assert_eq!(result, 2); // 2 = error (no memory)
}

#[test]
fn test_load_no_memory() {
    let engine = Engine::default();
    let mut linker: Linker<WasmGameContext<TestInput, ()>> = Linker::new(&engine);
    register_common_ffi(&mut linker).unwrap();

    // WAT module without exported memory
    let wat = r#"
        (module
            (import "env" "load" (func $load (param i32 i32 i32) (result i32)))
            (func (export "test_no_memory") (result i32)
                (call $load (i32.const 0) (i32.const 0) (i32.const 10))
            )
        )
    "#;
    let wasm = wat::parse_str(wat).unwrap();
    let module = wasmtime::Module::new(&engine, wasm).unwrap();

    let mut store = Store::new(&engine, WasmGameContext::<TestInput, ()>::new());
    // Intentionally NOT setting memory - but we need to put some data in the slot first
    store.data_mut().game.save_data[0] = Some(vec![1, 2, 3, 4]);
    let instance = linker.instantiate(&mut store, &module).unwrap();

    let test_fn = instance
        .get_typed_func::<(), i32>(&mut store, "test_no_memory")
        .unwrap();

    let result = test_fn.call(&mut store, ()).unwrap();
    assert_eq!(result, 0); // 0 = failed to load (no memory)
}
