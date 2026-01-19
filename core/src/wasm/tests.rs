//! Tests for WASM engine and game instance

use super::*;
use glam::{Mat4, Vec3};
use std::f32::consts::PI;

use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(
    Debug, Clone, Copy, Default, PartialEq, Eq, Pod, Zeroable, serde::Serialize, serde::Deserialize,
)]
struct TestInput {
    buttons: u16,
}
impl crate::console::ConsoleInput for TestInput {}

// ============================================================================
// WasmEngine Tests
// ============================================================================

#[test]
fn test_wasm_engine_creation() {
    let engine = WasmEngine::new();
    assert!(engine.is_ok());
}

// NOTE: WasmEngine does not implement Default because engine initialization
// is fallible. Use WasmEngine::new() which returns Result<Self>.

#[test]
fn test_wasm_engine_load_invalid_module() {
    let engine = WasmEngine::new().unwrap();
    let result = engine.load_module(b"not valid wasm");
    assert!(result.is_err());
}

#[test]
fn test_wasm_engine_load_valid_module() {
    let engine = WasmEngine::new().unwrap();
    // Minimal valid WASM module (empty module)
    let wasm = wat::parse_str("(module)").unwrap();
    let result = engine.load_module(&wasm);
    assert!(result.is_ok());
}

// ============================================================================
// GameInstance Integration Tests (require WASM modules)
// ============================================================================

#[test]
fn test_game_instance_creation_empty_module() {
    let engine = WasmEngine::new().unwrap();
    let wasm = wat::parse_str(
        r#"
        (module
            (memory (export "memory") 1)
        )
    "#,
    )
    .unwrap();
    let module = engine.load_module(&wasm).unwrap();
    let linker = wasmtime::Linker::new(engine.engine());

    let result = GameInstance::<TestInput, ()>::new(&engine, &module, &linker);
    assert!(result.is_ok());
}

#[test]
fn test_game_instance_with_init_function() {
    let engine = WasmEngine::new().unwrap();
    let wasm = wat::parse_str(
        r#"
        (module
            (memory (export "memory") 1)
            (func (export "init"))
        )
    "#,
    )
    .unwrap();
    let module = engine.load_module(&wasm).unwrap();
    let linker = wasmtime::Linker::new(engine.engine());

    let mut game = GameInstance::<TestInput, ()>::new(&engine, &module, &linker).unwrap();
    let result = game.init();
    assert!(result.is_ok());
    // in_init should be false after init completes
    assert!(!game.state().in_init);
}

#[test]
fn test_game_instance_with_update_function() {
    let engine = WasmEngine::new().unwrap();
    let wasm = wat::parse_str(
        r#"
        (module
            (memory (export "memory") 1)
            (func (export "update"))
        )
    "#,
    )
    .unwrap();
    let module = engine.load_module(&wasm).unwrap();
    let linker = wasmtime::Linker::new(engine.engine());

    let mut game = GameInstance::<TestInput, ()>::new(&engine, &module, &linker).unwrap();
    let delta = 1.0 / 60.0;
    let result = game.update(delta);
    assert!(result.is_ok());
    assert_eq!(game.state().tick_count, 1);
    assert!((game.state().delta_time - delta).abs() < 0.0001);
}

#[test]
fn test_game_instance_update_increments_tick() {
    let engine = WasmEngine::new().unwrap();
    let wasm = wat::parse_str(
        r#"
        (module
            (memory (export "memory") 1)
            (func (export "update"))
        )
    "#,
    )
    .unwrap();
    let module = engine.load_module(&wasm).unwrap();
    let linker = wasmtime::Linker::new(engine.engine());

    let mut game = GameInstance::<TestInput, ()>::new(&engine, &module, &linker).unwrap();

    for i in 1..=5 {
        game.update(1.0 / 60.0).unwrap();
        assert_eq!(game.state().tick_count, i);
    }
}

#[test]
fn test_game_instance_update_accumulates_elapsed_time() {
    let engine = WasmEngine::new().unwrap();
    let wasm = wat::parse_str(
        r#"
        (module
            (memory (export "memory") 1)
            (func (export "update"))
        )
    "#,
    )
    .unwrap();
    let module = engine.load_module(&wasm).unwrap();
    let linker = wasmtime::Linker::new(engine.engine());

    let mut game = GameInstance::<TestInput, ()>::new(&engine, &module, &linker).unwrap();
    let delta = 0.016; // ~60fps

    game.update(delta).unwrap();
    game.update(delta).unwrap();
    game.update(delta).unwrap();

    assert!((game.state().elapsed_time - delta * 3.0).abs() < 0.0001);
}

#[test]
fn test_game_instance_with_render_function() {
    let engine = WasmEngine::new().unwrap();
    let wasm = wat::parse_str(
        r#"
        (module
            (memory (export "memory") 1)
            (func (export "render"))
        )
    "#,
    )
    .unwrap();
    let module = engine.load_module(&wasm).unwrap();
    let linker = wasmtime::Linker::new(engine.engine());

    let mut game = GameInstance::<TestInput, ()>::new(&engine, &module, &linker).unwrap();
    let result = game.render();
    assert!(result.is_ok());
}

#[test]
fn test_game_instance_set_input() {
    let engine = WasmEngine::new().unwrap();
    let wasm = wat::parse_str(
        r#"
        (module
            (memory (export "memory") 1)
        )
    "#,
    )
    .unwrap();
    let module = engine.load_module(&wasm).unwrap();
    let linker = wasmtime::Linker::new(engine.engine());

    let mut game = GameInstance::<TestInput, ()>::new(&engine, &module, &linker).unwrap();

    let input = TestInput { buttons: 0x00FF };

    game.set_input(0, input);
    assert_eq!(game.state().input_curr[0].buttons, 0x00FF);
}

#[test]
fn test_game_instance_set_input_invalid_player() {
    let engine = WasmEngine::new().unwrap();
    let wasm = wat::parse_str(
        r#"
        (module
            (memory (export "memory") 1)
        )
    "#,
    )
    .unwrap();
    let module = engine.load_module(&wasm).unwrap();
    let linker = wasmtime::Linker::new(engine.engine());

    let mut game = GameInstance::<TestInput, ()>::new(&engine, &module, &linker).unwrap();

    // Should not panic for invalid player index
    game.set_input(10, TestInput::default());
}

#[test]
fn test_game_instance_input_rotation() {
    let engine = WasmEngine::new().unwrap();
    let wasm = wat::parse_str(
        r#"
        (module
            (memory (export "memory") 1)
            (func (export "update"))
        )
    "#,
    )
    .unwrap();
    let module = engine.load_module(&wasm).unwrap();
    let linker = wasmtime::Linker::new(engine.engine());

    let mut game = GameInstance::<TestInput, ()>::new(&engine, &module, &linker).unwrap();

    // Set input for player 0
    let input1 = TestInput { buttons: 0x0001 };
    game.set_input(0, input1);

    // Call update (which rotates input_prev = input_curr)
    game.update(1.0 / 60.0).unwrap();

    // Previous should now have our input
    assert_eq!(game.state().input_prev[0].buttons, 0x0001);

    // Set new input
    let input2 = TestInput { buttons: 0x0002 };
    game.set_input(0, input2);

    // Current should have new input
    assert_eq!(game.state().input_curr[0].buttons, 0x0002);
}

#[test]
fn test_game_instance_store_access() {
    let engine = WasmEngine::new().unwrap();
    let wasm = wat::parse_str(
        r#"
        (module
            (memory (export "memory") 1)
        )
    "#,
    )
    .unwrap();
    let module = engine.load_module(&wasm).unwrap();
    let linker = wasmtime::Linker::new(engine.engine());

    let mut game = GameInstance::<TestInput, ()>::new(&engine, &module, &linker).unwrap();

    // Test mutable access
    game.state_mut().player_count = 4;
    assert_eq!(game.state().player_count, 4);

    // Test store access
    let _store = game.store();
    let _store_mut = game.store_mut();
}

#[test]
fn test_game_instance_save_state_basic() {
    let engine = WasmEngine::new().unwrap();
    let wasm = wat::parse_str(
        r#"
        (module
            (memory (export "memory") 1)
        )
    "#,
    )
    .unwrap();
    let module = engine.load_module(&wasm).unwrap();
    let linker = wasmtime::Linker::new(engine.engine());

    let mut game = GameInstance::<TestInput, ()>::new(&engine, &module, &linker).unwrap();

    // save_state returns entire WASM memory (1 page = 64KB)
    let result = game.save_state();
    assert!(result.is_ok());
    let snapshot = result.unwrap();
    assert_eq!(snapshot.len(), 65536); // 1 WASM page
}

#[test]
fn test_game_instance_load_state_basic() {
    let engine = WasmEngine::new().unwrap();
    let wasm = wat::parse_str(
        r#"
        (module
            (memory (export "memory") 1)
        )
    "#,
    )
    .unwrap();
    let module = engine.load_module(&wasm).unwrap();
    let linker = wasmtime::Linker::new(engine.engine());

    let mut game = GameInstance::<TestInput, ()>::new(&engine, &module, &linker).unwrap();

    // load_state requires exact memory size match
    let snapshot = vec![0u8; 65536]; // 1 WASM page
    let result = game.load_state(&snapshot);
    assert!(result.is_ok());
}

#[test]
fn test_game_instance_configure_session() {
    let engine = WasmEngine::new().unwrap();
    let wasm = wat::parse_str(
        r#"
        (module
            (memory (export "memory") 1)
        )
    "#,
    )
    .unwrap();
    let module = engine.load_module(&wasm).unwrap();
    let linker = wasmtime::Linker::new(engine.engine());

    let mut game = GameInstance::<TestInput, ()>::new(&engine, &module, &linker).unwrap();

    // Default values
    assert_eq!(game.state().player_count, 1);
    assert_eq!(game.state().local_player_mask, 1);

    // Configure for 4 players, only player 0 is local
    game.configure_session(4, 0b0001);
    assert_eq!(game.state().player_count, 4);
    assert_eq!(game.state().local_player_mask, 0b0001);

    // Configure for 2 players, both local
    game.configure_session(2, 0b0011);
    assert_eq!(game.state().player_count, 2);
    assert_eq!(game.state().local_player_mask, 0b0011);
}

#[test]
fn test_game_instance_configure_session_clamps_players() {
    let engine = WasmEngine::new().unwrap();
    let wasm = wat::parse_str(
        r#"
        (module
            (memory (export "memory") 1)
        )
    "#,
    )
    .unwrap();
    let module = engine.load_module(&wasm).unwrap();
    let linker = wasmtime::Linker::new(engine.engine());

    let mut game = GameInstance::<TestInput, ()>::new(&engine, &module, &linker).unwrap();

    // Try to set more than MAX_PLAYERS
    game.configure_session(100, 0xFFFF);
    assert_eq!(game.state().player_count, 4); // Clamped to MAX_PLAYERS
}

// ============================================================================
// WASM Memory Error Path Tests
// ============================================================================

#[test]
fn test_game_instance_load_state_size_mismatch() {
    let engine = WasmEngine::new().unwrap();
    let wasm = wat::parse_str(
        r#"
        (module
            (memory (export "memory") 1)
        )
    "#,
    )
    .unwrap();
    let module = engine.load_module(&wasm).unwrap();
    let linker = wasmtime::Linker::new(engine.engine());

    let mut game = GameInstance::<TestInput, ()>::new(&engine, &module, &linker).unwrap();

    // Try to load with wrong size (memory is 65536, we pass 100)
    let small_buffer = vec![0u8; 100];
    let result = game.load_state(&small_buffer);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("mismatch"));
}

#[test]
fn test_game_instance_load_state_no_memory() {
    let engine = WasmEngine::new().unwrap();
    // Module without memory export
    let wasm = wat::parse_str(
        r#"
        (module
            (func (export "init"))
        )
    "#,
    )
    .unwrap();
    let module = engine.load_module(&wasm).unwrap();
    let linker = wasmtime::Linker::new(engine.engine());

    let mut game = GameInstance::<TestInput, ()>::new(&engine, &module, &linker).unwrap();

    // Should fail because no memory is available
    let result = game.load_state(&[1, 2, 3, 4]);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("No memory"));
}

#[test]
fn test_game_instance_save_state_no_memory() {
    let engine = WasmEngine::new().unwrap();
    // Module without memory export
    let wasm = wat::parse_str(
        r#"
        (module
            (func (export "init"))
        )
    "#,
    )
    .unwrap();
    let module = engine.load_module(&wasm).unwrap();
    let linker = wasmtime::Linker::new(engine.engine());

    let mut game = GameInstance::<TestInput, ()>::new(&engine, &module, &linker).unwrap();

    // Should fail because no memory is available
    let result = game.save_state();
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("No memory"));
}

#[test]
fn test_game_instance_save_state_with_data() {
    let engine = WasmEngine::new().unwrap();
    // Module that has initialized data in memory
    let wasm = wat::parse_str(
        r#"
        (module
            (memory (export "memory") 1)
            ;; Initialize first 8 bytes with a pattern
            (data (i32.const 0) "\01\02\03\04\05\06\07\08")
        )
    "#,
    )
    .unwrap();
    let module = engine.load_module(&wasm).unwrap();
    let linker = wasmtime::Linker::new(engine.engine());

    let mut game = GameInstance::<TestInput, ()>::new(&engine, &module, &linker).unwrap();

    let result = game.save_state();
    assert!(result.is_ok());
    let snapshot = result.unwrap();
    assert_eq!(snapshot.len(), 65536); // Full memory
    assert_eq!(&snapshot[..8], &[1, 2, 3, 4, 5, 6, 7, 8]); // Check initialized data
}

#[test]
fn test_game_instance_load_state_restores_data() {
    let engine = WasmEngine::new().unwrap();
    let wasm = wat::parse_str(
        r#"
        (module
            (memory (export "memory") 1)
        )
    "#,
    )
    .unwrap();
    let module = engine.load_module(&wasm).unwrap();
    let linker = wasmtime::Linker::new(engine.engine());

    let mut game = GameInstance::<TestInput, ()>::new(&engine, &module, &linker).unwrap();

    // Create a snapshot with specific data
    let mut snapshot = vec![0u8; 65536]; // 1 page
    snapshot[0..4].copy_from_slice(&[0xDE, 0xAD, 0xBE, 0xEF]);

    let result = game.load_state(&snapshot);
    assert!(result.is_ok());

    // Verify data was restored by saving state again
    let restored = game.save_state().unwrap();
    assert_eq!(&restored[..4], &[0xDE, 0xAD, 0xBE, 0xEF]);
}

#[test]
fn test_game_instance_init_trap_propagates() {
    let engine = WasmEngine::new().unwrap();
    // Module with init that traps (unreachable instruction)
    let wasm = wat::parse_str(
        r#"
        (module
            (memory (export "memory") 1)
            (func (export "init")
                (unreachable)
            )
        )
    "#,
    )
    .unwrap();
    let module = engine.load_module(&wasm).unwrap();
    let linker = wasmtime::Linker::new(engine.engine());

    let mut game = GameInstance::<TestInput, ()>::new(&engine, &module, &linker).unwrap();

    // Trap should propagate as an error
    let result = game.init();
    assert!(result.is_err());
}

#[test]
fn test_game_instance_update_trap_propagates() {
    let engine = WasmEngine::new().unwrap();
    // Module with update that traps
    let wasm = wat::parse_str(
        r#"
        (module
            (memory (export "memory") 1)
            (func (export "update")
                (unreachable)
            )
        )
    "#,
    )
    .unwrap();
    let module = engine.load_module(&wasm).unwrap();
    let linker = wasmtime::Linker::new(engine.engine());

    let mut game = GameInstance::<TestInput, ()>::new(&engine, &module, &linker).unwrap();

    // Trap should propagate as an error
    let result = game.update(1.0 / 60.0);
    assert!(result.is_err());
}

#[test]
fn test_game_instance_render_trap_propagates() {
    let engine = WasmEngine::new().unwrap();
    // Module with render that traps
    let wasm = wat::parse_str(
        r#"
        (module
            (memory (export "memory") 1)
            (func (export "render")
                (unreachable)
            )
        )
    "#,
    )
    .unwrap();
    let module = engine.load_module(&wasm).unwrap();
    let linker = wasmtime::Linker::new(engine.engine());

    let mut game = GameInstance::<TestInput, ()>::new(&engine, &module, &linker).unwrap();

    // Trap should propagate as an error
    let result = game.render();
    assert!(result.is_err());
}

// ============================================================================
// Transform Matrix Tests
// ============================================================================

#[test]
fn test_transform_identity() {
    let transform = Mat4::IDENTITY;
    let point = Vec3::new(1.0, 2.0, 3.0);
    let transformed = transform.transform_point3(point);
    assert_eq!(transformed, point);
}

#[test]
fn test_transform_translation() {
    let transform = Mat4::from_translation(Vec3::new(10.0, 20.0, 30.0));
    let point = Vec3::ZERO;
    let transformed = transform.transform_point3(point);
    assert!((transformed.x - 10.0).abs() < 0.0001);
    assert!((transformed.y - 20.0).abs() < 0.0001);
    assert!((transformed.z - 30.0).abs() < 0.0001);
}

#[test]
fn test_transform_scale() {
    let transform = Mat4::from_scale(Vec3::new(2.0, 3.0, 4.0));
    let point = Vec3::new(1.0, 1.0, 1.0);
    let transformed = transform.transform_point3(point);
    assert!((transformed.x - 2.0).abs() < 0.0001);
    assert!((transformed.y - 3.0).abs() < 0.0001);
    assert!((transformed.z - 4.0).abs() < 0.0001);
}

#[test]
fn test_transform_rotation_90_deg_y() {
    let transform = Mat4::from_rotation_y(PI / 2.0);
    let point = Vec3::new(1.0, 0.0, 0.0);
    let transformed = transform.transform_point3(point);
    // Rotating (1, 0, 0) 90° around Y should give (0, 0, -1)
    assert!(transformed.x.abs() < 0.0001);
    assert!(transformed.y.abs() < 0.0001);
    assert!((transformed.z - (-1.0)).abs() < 0.0001);
}

#[test]
fn test_transform_combination() {
    // Scale, then rotate, then translate
    let scale = Mat4::from_scale(Vec3::splat(2.0));
    let rotate = Mat4::from_rotation_z(PI / 2.0);
    let translate = Mat4::from_translation(Vec3::new(5.0, 0.0, 0.0));

    // Combined transform (applied right-to-left)
    let transform = translate * rotate * scale;

    let point = Vec3::new(1.0, 0.0, 0.0);
    let transformed = transform.transform_point3(point);

    // (1, 0, 0) * 2 = (2, 0, 0)
    // Rotate 90° Z: (0, 2, 0)
    // Translate: (5, 2, 0)
    assert!((transformed.x - 5.0).abs() < 0.0001);
    assert!((transformed.y - 2.0).abs() < 0.0001);
    assert!(transformed.z.abs() < 0.0001);
}
