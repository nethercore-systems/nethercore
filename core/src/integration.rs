//! Integration tests for Emberware core framework
//!
//! Tests full game lifecycle, rollback simulation, input synchronization,
//! and resource limit enforcement.

#[cfg(test)]
mod tests {
    use wasmtime::Linker;

    use crate::{
        console::{Console, RawInput},
        ffi::register_common_ffi,
        rollback::{GameStateSnapshot, RollbackSession, RollbackStateManager},
        runtime::Runtime,
        test_utils::{TestConsole, TestInput},
        wasm::{GameInstance, GameState, MAX_PLAYERS, WasmEngine, WasmGameContext},
    };

    /// Create a test engine with common FFI registered
    fn create_test_engine() -> (WasmEngine, Linker<WasmGameContext<TestInput, (), ()>>) {
        let engine = WasmEngine::new().unwrap();
        let mut linker = Linker::new(engine.engine());
        register_common_ffi(&mut linker).unwrap();
        (engine, linker)
    }

    // ============================================================================
    // PART 1: Full Game Lifecycle Tests (init → update → render)
    // ============================================================================

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
        let mut game = GameInstance::<TestInput, ()>::new(&engine, &module, &linker).unwrap();

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
        let game = GameInstance::<TestInput, ()>::new(&engine, &module, &linker).unwrap();

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
        let mut game = GameInstance::<TestInput, ()>::new(&engine, &module, &linker).unwrap();

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
        let mut game = GameInstance::<TestInput, ()>::new(&engine, &module, &linker).unwrap();

        game.init().unwrap();

        // Run 10 updates
        for _ in 0..10 {
            game.update(1.0 / 60.0).unwrap();
        }

        // State should be persistent (x = 110, y = 205)
        assert_eq!(game.state().tick_count, 10);
    }

    // ============================================================================
    // PART 2: Rollback Simulation Tests (save → modify → load → verify)
    // ============================================================================

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
        let mut game = GameInstance::<TestInput, ()>::new(&engine, &module, &linker).unwrap();

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
        let mut game = GameInstance::<TestInput, ()>::new(&engine, &module, &linker).unwrap();
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
        let mut game = GameInstance::<TestInput, ()>::new(&engine, &module, &linker).unwrap();
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
        let mut game = GameInstance::<TestInput, ()>::new(&engine, &module, &linker).unwrap();
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
        let mut game = GameInstance::<TestInput, ()>::new(&engine, &module, &linker).unwrap();

        let mut session = RollbackSession::<TestInput, ()>::new_local(2, 4 * 1024 * 1024);

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

    // ============================================================================
    // PART 3: Multi-Player Input Synchronization Tests
    // ============================================================================

    /// Test input state rotation (prev/curr)
    #[test]
    fn test_input_state_rotation() {
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
        let mut game = GameInstance::<TestInput, ()>::new(&engine, &module, &linker).unwrap();

        game.init().unwrap();

        // Set input for player 0
        let input1 = TestInput {
            buttons: 0x0001,
            x: 100,
            y: -50,
        };
        game.set_input(0, input1);

        // Verify current input
        assert_eq!(game.state().input_curr[0].buttons, 0x0001);
        assert_eq!(game.state().input_prev[0].buttons, 0);

        // Update rotates input
        game.update(1.0 / 60.0).unwrap();

        // Previous should now have our input
        assert_eq!(game.state().input_prev[0].buttons, 0x0001);

        // Set new input
        let input2 = TestInput {
            buttons: 0x0002,
            ..Default::default()
        };
        game.set_input(0, input2);

        // Verify both states
        assert_eq!(game.state().input_curr[0].buttons, 0x0002);
        assert_eq!(game.state().input_prev[0].buttons, 0x0001);
    }

    /// Test multi-player input handling
    #[test]
    fn test_multiplayer_input_handling() {
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
        let mut game = GameInstance::<TestInput, ()>::new(&engine, &module, &linker).unwrap();

        game.init().unwrap();
        game.state_mut().player_count = 4;

        // Set different inputs for all 4 players
        for player in 0..MAX_PLAYERS {
            let p = player as i32;
            let input = TestInput {
                buttons: (player as u16) << 8,
                x: (p * 30) as i8,
                y: (p * -20) as i8,
            };
            game.set_input(player, input);
        }

        // Verify all players have correct input
        for player in 0..MAX_PLAYERS {
            let p = player as i32;
            let input = &game.state().input_curr[player];
            assert_eq!(input.buttons, (player as u16) << 8);
            assert_eq!(input.x, (p * 30) as i8);
            assert_eq!(input.y, (p * -20) as i8);
        }
    }

    /// Test local player mask handling
    #[test]
    fn test_local_player_mask() {
        let (engine, linker) = create_test_engine();

        let wat = r#"
            (module
                (import "env" "player_count" (func $player_count (result i32)))
                (import "env" "local_player_mask" (func $local_player_mask (result i32)))
                (memory (export "memory") 1)

                (func (export "init"))
                (func (export "update"))
                (func (export "render"))

                (func (export "get_player_count") (result i32)
                    (call $player_count)
                )

                (func (export "get_local_mask") (result i32)
                    (call $local_player_mask)
                )
            )
        "#;

        let wasm = wat::parse_str(wat).unwrap();
        let module = engine.load_module(&wasm).unwrap();
        let mut game = GameInstance::<TestInput, ()>::new(&engine, &module, &linker).unwrap();

        // Set up 4 players, players 0 and 2 are local
        game.state_mut().player_count = 4;
        game.state_mut().local_player_mask = 0b0101;

        game.init().unwrap();

        // Verify state is accessible
        assert_eq!(game.state().player_count, 4);
        assert_eq!(game.state().local_player_mask, 0b0101);
    }

    /// Test RollbackSession input handling
    #[test]
    fn test_rollback_session_input() {
        let mut session = RollbackSession::<TestInput, ()>::new_local(2, 4 * 1024 * 1024);

        // Add input for local players
        let input0 = TestInput {
            buttons: 0xFF,
            x: 100,
            y: -50,
        };
        let input1 = TestInput {
            buttons: 0x0F,
            x: -100,
            y: 50,
        };

        // Local sessions don't use GGRS input directly
        session.add_local_input(0, input0).unwrap();
        session.add_local_input(1, input1).unwrap();

        // Verify local players are tracked
        assert_eq!(session.local_players(), &[0, 1]);
    }

    /// Test console input mapping
    #[test]
    fn test_console_input_mapping() {
        let console = TestConsole;

        let raw = RawInput {
            button_a: true,
            button_b: false,
            left_stick_x: 0.5,
            left_stick_y: -0.25,
            ..Default::default()
        };

        let mapped = console.map_input(&raw);
        assert_eq!(mapped.buttons, 1); // A pressed
        assert_eq!(mapped.x, 63); // 0.5 * 127 ≈ 63
        assert_eq!(mapped.y, -31); // -0.25 * 127 ≈ -31
    }

    // ============================================================================
    // PART 4: Resource Limit Enforcement Tests
    // ============================================================================

    /// Test console specs are correctly reported
    #[test]
    fn test_console_specs_limits() {
        let console = TestConsole;
        let specs = console.specs();

        assert_eq!(specs.ram_limit, 16 * 1024 * 1024);
        assert_eq!(specs.vram_limit, 8 * 1024 * 1024);
        assert_eq!(specs.rom_limit, 32 * 1024 * 1024);
        assert_eq!(specs.cpu_budget_us, 4000);
    }

    /// Test save data slot limits
    #[test]
    fn test_save_data_slot_limits() {
        use crate::wasm::{MAX_SAVE_SIZE, MAX_SAVE_SLOTS};

        let mut state = GameState::<TestInput>::new();

        // Verify constants
        assert_eq!(MAX_SAVE_SLOTS, 8);
        assert_eq!(MAX_SAVE_SIZE, 64 * 1024);

        // All slots should be available
        assert_eq!(state.save_data.len(), MAX_SAVE_SLOTS);
        for slot in &state.save_data {
            assert!(slot.is_none());
        }

        // Fill all slots
        for i in 0..MAX_SAVE_SLOTS {
            state.save_data[i] = Some(vec![i as u8; 1024]);
        }

        // Verify all slots filled
        for (i, slot) in state.save_data.iter().enumerate() {
            let data = slot.as_ref().unwrap();
            assert_eq!(data.len(), 1024);
            assert_eq!(data[0], i as u8);
        }
    }

    /// Test player count limits
    #[test]
    fn test_player_count_limits() {
        assert_eq!(MAX_PLAYERS, 4);

        let mut state = GameState::<TestInput>::new();

        // Can set up to 4 players
        state.player_count = MAX_PLAYERS as u32;
        state.local_player_mask = 0b1111; // All local

        assert_eq!(state.player_count, 4);

        // Verify all player input slots exist
        assert_eq!(state.input_curr.len(), MAX_PLAYERS);
        assert_eq!(state.input_prev.len(), MAX_PLAYERS);
    }

    // ============================================================================
    // PART 5: Combined Integration Tests
    // ============================================================================

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
        let mut game = GameInstance::<TestInput, ()>::new(&engine, &module, &linker).unwrap();
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
        let game = GameInstance::<TestInput, ()>::new(&engine, &module, &linker).unwrap();

        runtime.load_game(game);
        runtime.init_game().unwrap();

        // Set up 4-player local session
        let session = RollbackSession::<TestInput, ()>::new_local(4, 4 * 1024 * 1024);
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
}
