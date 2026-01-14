//! Multi-Player Input Synchronization and Resource Limit Tests

use crate::{
    console::{Console, RawInput},
    rollback::RollbackSession,
    test_utils::{TestConsole, TestInput},
    wasm::{GameState, MAX_PLAYERS, MAX_SAVE_SIZE, MAX_SAVE_SLOTS},
};

use super::test_utils::*;

// ============================================================================
// Input Synchronization Tests
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
    let mut game = new_test_game_instance(&engine, &module, &linker);

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
    let mut game = new_test_game_instance(&engine, &module, &linker);

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
    let mut game = new_test_game_instance(&engine, &module, &linker);

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
    let mut session = RollbackSession::<TestInput, ()>::new_local(2, test_ram_limit());

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
// Resource Limit Enforcement Tests
// ============================================================================

/// Test console specs are correctly reported
#[test]
fn test_console_specs_limits() {
    let specs = TestConsole::specs();

    assert_eq!(specs.ram_limit, 16 * 1024 * 1024);
    assert_eq!(specs.vram_limit, 8 * 1024 * 1024);
    assert_eq!(specs.rom_limit, 32 * 1024 * 1024);
    assert_eq!(specs.cpu_budget_us, 4000);
}

/// Test save data slot limits
#[test]
fn test_save_data_slot_limits() {
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
