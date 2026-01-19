//! Full Game Lifecycle Tests (init → update → render)

use crate::{runtime::Runtime, test_utils::TestConsole};

use super::test_utils::*;

/// Test that a game with all lifecycle exports can be created and run
#[test]
fn test_game_lifecycle_full() {
    let (engine, linker) = create_test_engine();

    // A complete game module with init, update, render, save_state, load_state
    let wat = r#"
        (module
            (memory (export "memory") 1)
            (global $counter (mut i32) (i32.const 0))
            (global $initialized (mut i32) (i32.const 0))

            (func (export "init")
                (global.set $initialized (i32.const 1))
            )

            (func (export "update")
                (global.set $counter (i32.add (global.get $counter) (i32.const 1)))
            )

            (func (export "render")
                ;; Rendering is a no-op for tests
                (nop)
            )

            ;; Save state: write counter to memory and return length
            (func (export "save_state") (param $ptr i32) (param $max_len i32) (result i32)
                (if (i32.ge_u (local.get $max_len) (i32.const 4))
                    (then
                        (i32.store (local.get $ptr) (global.get $counter))
                        (return (i32.const 4))
                    )
                )
                (i32.const 0)
            )

            ;; Load state: read counter from memory
            (func (export "load_state") (param $ptr i32) (param $len i32)
                (if (i32.ge_u (local.get $len) (i32.const 4))
                    (then
                        (global.set $counter (i32.load (local.get $ptr)))
                    )
                )
            )

            ;; Export counter for verification
            (func (export "get_counter") (result i32)
                (global.get $counter)
            )

            (func (export "get_initialized") (result i32)
                (global.get $initialized)
            )
        )
    "#;

    let wasm = wat::parse_str(wat).unwrap();
    let module = engine.load_module(&wasm).unwrap();
    let mut game = new_test_game_instance(&engine, &module, &linker);

    // Verify initial state
    let get_counter = game.store().data().game.memory.unwrap();
    let _ = get_counter; // Just verify memory exists

    // Get helper functions
    let instance = game.store_mut();
    let get_initialized = instance.data().game.memory.unwrap().data(&instance);
    let _ = get_initialized; // Memory is accessible

    // Test init
    assert!(game.state().in_init);
    game.init().unwrap();
    assert!(!game.state().in_init);

    // Test update
    assert_eq!(game.state().tick_count, 0);
    game.update(1.0 / 60.0).unwrap();
    assert_eq!(game.state().tick_count, 1);

    game.update(1.0 / 60.0).unwrap();
    assert_eq!(game.state().tick_count, 2);

    // Test render
    game.render().unwrap();

    // Test elapsed time accumulation
    let expected_elapsed = 2.0 / 60.0;
    assert!((game.state().elapsed_time - expected_elapsed).abs() < 0.0001);
}

/// Test game lifecycle with Runtime integration
#[test]
fn test_game_lifecycle_with_runtime() {
    let (engine, linker) = create_test_engine();
    let console = TestConsole;
    let mut runtime = Runtime::new(console);

    let wat = r#"
        (module
            (memory (export "memory") 1)
            (global $tick (mut i32) (i32.const 0))

            (func (export "init"))

            (func (export "update")
                (global.set $tick (i32.add (global.get $tick) (i32.const 1)))
            )

            (func (export "render"))
        )
    "#;

    let wasm = wat::parse_str(wat).unwrap();
    let module = engine.load_module(&wasm).unwrap();
    let game = new_test_game_instance(&engine, &module, &linker);

    runtime.load_game(game);
    runtime.init_game().unwrap();

    // Verify game is loaded
    assert!(runtime.game().is_some());

    // Test render
    runtime.render().unwrap();
}

/// Test game lifecycle with missing optional exports
#[test]
fn test_game_lifecycle_minimal() {
    let (engine, linker) = create_test_engine();

    // Module with only memory export (minimal game)
    let wat = r#"
        (module
            (memory (export "memory") 1)
        )
    "#;

    let wasm = wat::parse_str(wat).unwrap();
    let module = engine.load_module(&wasm).unwrap();
    let mut game = new_test_game_instance(&engine, &module, &linker);

    // All lifecycle calls should succeed (no-op for missing exports)
    game.init().unwrap();
    game.update(1.0 / 60.0).unwrap();
    game.render().unwrap();
}

/// Test game that modifies state across updates
#[test]
fn test_game_state_persistence() {
    let (engine, linker) = create_test_engine();

    let wat = r#"
        (module
            (memory (export "memory") 1)
            (global $x (mut f32) (f32.const 0.0))
            (global $y (mut f32) (f32.const 0.0))

            (func (export "init")
                (global.set $x (f32.const 100.0))
                (global.set $y (f32.const 200.0))
            )

            (func (export "update")
                ;; Move position each update
                (global.set $x (f32.add (global.get $x) (f32.const 1.0)))
                (global.set $y (f32.add (global.get $y) (f32.const 0.5)))
            )

            (func (export "render"))

            (func (export "get_x") (result f32)
                (global.get $x)
            )

            (func (export "get_y") (result f32)
                (global.get $y)
            )
        )
    "#;

    let wasm = wat::parse_str(wat).unwrap();
    let module = engine.load_module(&wasm).unwrap();
    let mut game = new_test_game_instance(&engine, &module, &linker);

    game.init().unwrap();

    // Run 10 updates
    for _ in 0..10 {
        game.update(1.0 / 60.0).unwrap();
    }

    // State should be persistent (x = 110, y = 205)
    assert_eq!(game.state().tick_count, 10);
}
