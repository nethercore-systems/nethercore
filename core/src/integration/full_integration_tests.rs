//! Combined Integration Tests

use crate::{
    rollback::{GameStateSnapshot, RollbackSession, RollbackStateManager},
    runtime::Runtime,
    test_utils::{TestConsole, TestInput},
};

use super::test_utils::*;

/// Full integration test: game lifecycle with rollback
#[test]
fn test_full_integration_lifecycle_with_rollback() {
    let (engine, linker) = create_test_engine();

    let wat = r#"
        (module
            (memory (export "memory") 1)
            (global $score (mut i32) (i32.const 0))
            (global $frame (mut i32) (i32.const 0))

            (func (export "init")
                (global.set $score (i32.const 0))
                (global.set $frame (i32.const 0))
            )

            (func (export "update")
                (global.set $frame (i32.add (global.get $frame) (i32.const 1)))
                ;; Score increases by frame number
                (global.set $score (i32.add (global.get $score) (global.get $frame)))
            )

            (func (export "render"))

            (func (export "save_state") (param $ptr i32) (param $max_len i32) (result i32)
                (if (i32.ge_u (local.get $max_len) (i32.const 8))
                    (then
                        (i32.store (local.get $ptr) (global.get $score))
                        (i32.store (i32.add (local.get $ptr) (i32.const 4)) (global.get $frame))
                        (return (i32.const 8))
                    )
                )
                (i32.const 0)
            )

            (func (export "load_state") (param $ptr i32) (param $len i32)
                (if (i32.ge_u (local.get $len) (i32.const 8))
                    (then
                        (global.set $score (i32.load (local.get $ptr)))
                        (global.set $frame (i32.load (i32.add (local.get $ptr) (i32.const 4))))
                    )
                )
            )
        )
    "#;

    let wasm = wat::parse_str(wat).unwrap();
    let module = engine.load_module(&wasm).unwrap();
    let mut game = new_test_game_instance(&engine, &module, &linker);
    let mut state_manager = RollbackStateManager::with_defaults();

    // Phase 1: Init
    game.init().unwrap();

    // Phase 2: Run 5 frames, save at frame 3
    let mut frame3_snapshot: Option<GameStateSnapshot> = None;
    for frame in 1..=5 {
        game.update(1.0 / 60.0).unwrap();
        game.render().unwrap();
        if frame == 3 {
            frame3_snapshot = Some(state_manager.save_state(&mut game, frame).unwrap());
        }
    }

    // Score after 5 frames: 1+2+3+4+5 = 15
    let snapshot_at_5 = state_manager.save_state(&mut game, 5).unwrap();

    // Phase 3: Rollback to frame 3
    state_manager
        .load_state(&mut game, frame3_snapshot.as_ref().unwrap())
        .unwrap();

    // Verify we're back at frame 3 state
    let restored_snapshot = state_manager.save_state(&mut game, 3).unwrap();
    assert_eq!(
        frame3_snapshot.as_ref().unwrap().checksum,
        restored_snapshot.checksum
    );

    // Phase 4: Re-simulate frames 4-5
    for _ in 4..=5 {
        game.update(1.0 / 60.0).unwrap();
        game.render().unwrap();
    }

    // Should arrive at same state as before rollback
    let final_snapshot = state_manager.save_state(&mut game, 5).unwrap();
    assert_eq!(snapshot_at_5.checksum, final_snapshot.checksum);
}

/// Integration test: multiplayer input with rollback
#[test]
fn test_full_integration_multiplayer_rollback() {
    let console = TestConsole;
    let mut runtime = Runtime::new(console);

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
    let game = new_test_game_instance(&engine, &module, &linker);

    runtime.load_game(game);
    runtime.init_game().unwrap();

    // Set up 4-player local session
    let session = RollbackSession::<TestInput, ()>::new_local(4, test_ram_limit());
    runtime.set_session(session);

    // Verify session is set
    assert!(runtime.session().is_some());
    assert_eq!(runtime.session().unwrap().local_players(), &[0, 1, 2, 3]);

    // Add inputs for all players
    for player in 0..4 {
        let input = TestInput {
            buttons: (player as u16) << 4,
            x: 0,
            y: 0,
        };
        runtime.add_local_input(player, input).unwrap();
    }

    // No events in local session
    let events = runtime.handle_session_events();
    assert!(events.is_empty());
}
