//! Rollback Simulation Tests (save → modify → load → verify)

use crate::rollback::{RollbackSession, RollbackStateManager};
use crate::test_utils::TestInput;

use super::test_utils::*;

/// Test basic save and load state functionality
///
/// The new save_state API snapshots entire WASM linear memory automatically.
/// State must be stored in memory (not globals) for rollback to work.
#[test]
fn test_rollback_save_load_basic() {
    let (engine, linker) = create_test_engine();

    // Store value at memory address 0 (not in a global)
    let wat = r#"
        (module
            (memory (export "memory") 1)

            (func (export "init")
                ;; Initialize value at address 0 to 0
                (i32.store (i32.const 0) (i32.const 0))
            )
            (func (export "update")
                ;; value += 10
                (i32.store (i32.const 0)
                    (i32.add (i32.load (i32.const 0)) (i32.const 10))
                )
            )
            (func (export "render"))

            (func (export "get_value") (result i32)
                (i32.load (i32.const 0))
            )
        )
    "#;

    let wasm = wat::parse_str(wat).unwrap();
    let module = engine.load_module(&wasm).unwrap();
    let mut game = new_test_game_instance(&engine, &module, &linker);

    game.init().unwrap();

    // Update a few times (value = 30)
    game.update(1.0 / 60.0).unwrap();
    game.update(1.0 / 60.0).unwrap();
    game.update(1.0 / 60.0).unwrap();

    // Save state - snapshots entire WASM memory
    let snapshot = game.save_state().unwrap();
    // Memory is 1 page = 64KB
    assert_eq!(snapshot.len(), 65536);

    // Update more (value = 60)
    game.update(1.0 / 60.0).unwrap();
    game.update(1.0 / 60.0).unwrap();
    game.update(1.0 / 60.0).unwrap();

    // Load saved state (value should be 30 again)
    game.load_state(&snapshot).unwrap();

    // Verify state was restored by saving again and comparing
    let snapshot2 = game.save_state().unwrap();
    assert_eq!(snapshot.len(), snapshot2.len());
    assert_eq!(snapshot, snapshot2);
}

/// Test rollback with RollbackStateManager
#[test]
fn test_rollback_state_manager() {
    let (engine, linker) = create_test_engine();

    let wat = r#"
        (module
            (memory (export "memory") 1)
            (global $counter (mut i32) (i32.const 0))

            (func (export "init"))
            (func (export "update")
                (global.set $counter (i32.add (global.get $counter) (i32.const 1)))
            )
            (func (export "render"))

            (func (export "save_state") (param $ptr i32) (param $max_len i32) (result i32)
                (if (i32.ge_u (local.get $max_len) (i32.const 4))
                    (then
                        (i32.store (local.get $ptr) (global.get $counter))
                        (return (i32.const 4))
                    )
                )
                (i32.const 0)
            )

            (func (export "load_state") (param $ptr i32) (param $len i32)
                (if (i32.ge_u (local.get $len) (i32.const 4))
                    (then
                        (global.set $counter (i32.load (local.get $ptr)))
                    )
                )
            )
        )
    "#;

    let wasm = wat::parse_str(wat).unwrap();
    let module = engine.load_module(&wasm).unwrap();
    let mut game = new_test_game_instance(&engine, &module, &linker);
    let mut state_manager = RollbackStateManager::with_defaults();

    game.init().unwrap();

    // Update 5 times
    for _ in 0..5 {
        game.update(1.0 / 60.0).unwrap();
    }

    // Save at frame 5
    let snapshot = state_manager.save_state(&mut game, 5).unwrap();
    assert_eq!(snapshot.frame, 5);
    assert!(!snapshot.is_empty());
    assert_ne!(snapshot.checksum, 0);

    // Update 5 more times
    for _ in 0..5 {
        game.update(1.0 / 60.0).unwrap();
    }
    assert_eq!(game.state().tick_count, 10);

    // Load state from frame 5
    state_manager.load_state(&mut game, &snapshot).unwrap();

    // Save again to verify state matches
    let snapshot2 = state_manager.save_state(&mut game, 5).unwrap();
    assert_eq!(snapshot.data, snapshot2.data);
    assert_eq!(snapshot.checksum, snapshot2.checksum);
}

/// Test that checksum detects state differences
///
/// State must be stored in memory (not globals) for rollback to work.
#[test]
fn test_rollback_checksum_detection() {
    let (engine, linker) = create_test_engine();

    // Store counter at memory address 0 (not in a global)
    let wat = r#"
        (module
            (memory (export "memory") 1)

            (func (export "init")
                ;; Initialize counter at address 0 to 0
                (i32.store (i32.const 0) (i32.const 0))
            )
            (func (export "update")
                ;; counter += 1
                (i32.store (i32.const 0)
                    (i32.add (i32.load (i32.const 0)) (i32.const 1))
                )
            )
            (func (export "render"))
        )
    "#;

    let wasm = wat::parse_str(wat).unwrap();
    let module = engine.load_module(&wasm).unwrap();
    let mut game = new_test_game_instance(&engine, &module, &linker);
    let mut state_manager = RollbackStateManager::with_defaults();

    game.init().unwrap();

    // Save at tick 0
    let snapshot1 = state_manager.save_state(&mut game, 0).unwrap();

    // Update and save at tick 1
    game.update(1.0 / 60.0).unwrap();
    let snapshot2 = state_manager.save_state(&mut game, 1).unwrap();

    // Checksums should be different (memory changed)
    assert_ne!(snapshot1.checksum, snapshot2.checksum);
    assert_ne!(snapshot1.data, snapshot2.data);
}

/// Regression test for GGRS sync-test snapshots.
///
/// StandaloneApp used to pre-seed `input_curr` before GGRS emitted
/// `SaveGameState`. During rollback replay, GGRS applies frame inputs at
/// `AdvanceFrame`, so a pre-frame snapshot must not depend on the current
/// frame's pre-seeded input. Only the previous input is needed to make
/// button_pressed/button_released deterministic when the frame is replayed.
#[test]
fn test_pre_frame_snapshot_canonicalizes_current_input() {
    let (engine, linker) = create_test_engine();

    let wat = r#"
        (module
            (memory (export "memory") 1)
            (func (export "init"))
            (func (export "update"))
            (func (export "render"))
        )
    "#;

    let wasm = wat::parse_str(wat).unwrap();
    let module = engine.load_module(&wasm).unwrap();
    let mut game = new_test_game_instance(&engine, &module, &linker);
    let mut state_manager = RollbackStateManager::with_defaults();

    game.init().unwrap();
    game.state_mut().input_prev[0] = TestInput {
        buttons: 0x0001,
        x: 0,
        y: 0,
    };
    game.state_mut().input_curr[0] = TestInput {
        buttons: 0x0010,
        x: 0,
        y: 0,
    };
    let snapshot_a = state_manager.save_state(&mut game, 7).unwrap();

    game.state_mut().input_curr[0] = TestInput {
        buttons: 0x0020,
        x: 0,
        y: 0,
    };
    let snapshot_b = state_manager.save_state(&mut game, 7).unwrap();

    assert_eq!(
        snapshot_a.checksum, snapshot_b.checksum,
        "pre-frame snapshots must not depend on pre-seeded current-frame input"
    );

    game.state_mut().input_prev[0] = TestInput {
        buttons: 0x0002,
        x: 0,
        y: 0,
    };
    let snapshot_c = state_manager.save_state(&mut game, 7).unwrap();

    assert_ne!(
        snapshot_b.checksum, snapshot_c.checksum,
        "previous input must still be part of rollback snapshots"
    );
}

#[test]
fn test_sync_test_advances_noop_game_with_staged_inputs() {
    let (engine, linker) = create_test_engine();

    let wat = r#"
        (module
            (memory (export "memory") 1)
            (func (export "init"))
            (func (export "update"))
            (func (export "render"))
        )
    "#;

    let wasm = wat::parse_str(wat).unwrap();
    let module = engine.load_module(&wasm).unwrap();
    let mut game = new_test_game_instance(&engine, &module, &linker);
    let config = crate::rollback::SessionConfig::sync_test_with_params(2, 0);
    let mut session =
        RollbackSession::<TestInput, ()>::new_sync_test(config, test_ram_limit()).unwrap();

    game.init().unwrap();

    for frame in 0..16 {
        for player in 0..2 {
            let input = TestInput {
                buttons: ((frame + player) & 1) as u16,
                x: frame as i8,
                y: player as i8,
            };
            game.set_input(player, input);
            session.add_local_input(player, input).unwrap();
        }

        let requests = session.advance_frame().unwrap();
        session
            .handle_requests_ordered(&mut game, requests, |game, inputs| {
                for (player_idx, (input, _status)) in inputs.iter().enumerate() {
                    game.set_input(player_idx, *input);
                }
                game.update(1.0 / 60.0)
                    .map_err(|e| crate::rollback::SessionError::Ggrs(e.to_string()))
            })
            .unwrap();
    }
}

/// Test rollback simulation with multiple save points
///
/// State must be stored in memory (not globals) for rollback to work.
#[test]
fn test_rollback_multiple_save_points() {
    let (engine, linker) = create_test_engine();

    // Store counter at memory address 0 (not in a global)
    let wat = r#"
        (module
            (memory (export "memory") 1)

            (func (export "init")
                ;; Initialize counter at address 0 to 0
                (i32.store (i32.const 0) (i32.const 0))
            )
            (func (export "update")
                ;; counter += 1
                (i32.store (i32.const 0)
                    (i32.add (i32.load (i32.const 0)) (i32.const 1))
                )
            )
            (func (export "render"))
        )
    "#;

    let wasm = wat::parse_str(wat).unwrap();
    let module = engine.load_module(&wasm).unwrap();
    let mut game = new_test_game_instance(&engine, &module, &linker);
    let mut state_manager = RollbackStateManager::with_defaults();

    game.init().unwrap();

    // Save snapshots at frames 0, 2, 4, 6, 8
    let mut snapshots = Vec::new();
    for frame in 0..10 {
        if frame % 2 == 0 {
            snapshots.push(state_manager.save_state(&mut game, frame).unwrap());
        }
        game.update(1.0 / 60.0).unwrap();
    }

    // All snapshots should have different checksums (memory changed each update)
    for i in 0..snapshots.len() {
        for j in (i + 1)..snapshots.len() {
            assert_ne!(
                snapshots[i].checksum, snapshots[j].checksum,
                "Snapshots {} and {} should have different checksums",
                i, j
            );
        }
    }

    // Load frame 4 snapshot and verify
    state_manager.load_state(&mut game, &snapshots[2]).unwrap();
    let restored = state_manager.save_state(&mut game, 4).unwrap();
    assert_eq!(snapshots[2].checksum, restored.checksum);
}

/// Test rollback with RollbackSession
#[test]
fn test_rollback_session_local() {
    let (engine, linker) = create_test_engine();

    let wat = r#"
        (module
            (memory (export "memory") 1)
            (func (export "init"))
            (func (export "update"))
            (func (export "render"))
        )
    "#;

    let wasm = wat::parse_str(wat).unwrap();
    let module = engine.load_module(&wasm).unwrap();
    let mut game = new_test_game_instance(&engine, &module, &linker);

    let mut session = RollbackSession::<TestInput, ()>::new_local(2, test_ram_limit());

    game.init().unwrap();

    // Advance frame through session
    let requests = session.advance_frame().unwrap();
    assert_eq!(requests.len(), 1);

    // Handle requests
    let advance_inputs = session.handle_requests(&mut game, requests).unwrap();
    assert_eq!(advance_inputs.len(), 1);
    assert_eq!(advance_inputs[0].len(), 2); // 2 players

    // Frame should have advanced
    assert_eq!(session.current_frame(), 1);
}
