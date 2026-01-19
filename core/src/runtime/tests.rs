//! Runtime tests

use wasmtime::Linker;

use crate::console::{Console, RawInput};
use crate::test_utils::{TestAudio, TestConsole, TestInput};
use crate::wasm::{GameInstance, WasmEngine};

use super::{Runtime, RuntimeConfig};

fn test_ram_limit() -> usize {
    TestConsole::specs().ram_limit
}

// ============================================================================
// RuntimeConfig Tests
// ============================================================================

#[test]
fn test_runtime_config_default() {
    let config = RuntimeConfig::default();
    assert_eq!(config.tick_rate, 60);
    assert_eq!(config.max_delta, std::time::Duration::from_millis(100));
    assert_eq!(config.cpu_budget, std::time::Duration::from_micros(4000));
}

// ============================================================================
// Runtime Creation Tests
// ============================================================================

#[test]
fn test_runtime_new() {
    let console = TestConsole;
    let runtime = Runtime::new(console);

    assert_eq!(runtime.tick_rate(), 60);
    assert!(runtime.game().is_none());
    assert!(runtime.session().is_none());
    assert!(runtime.audio().is_none());
}

#[test]
fn test_runtime_console_access() {
    assert_eq!(TestConsole::specs().name, "Test Console");
}

#[test]
fn test_runtime_set_tick_rate() {
    let console = TestConsole;
    let mut runtime = Runtime::new(console);

    runtime.set_tick_rate(30);
    assert_eq!(runtime.tick_rate(), 30);

    runtime.set_tick_rate(120);
    assert_eq!(runtime.tick_rate(), 120);
}

// ============================================================================
// Game Loading Tests
// ============================================================================

#[test]
fn test_runtime_load_game() {
    let console = TestConsole;
    let mut runtime = Runtime::new(console);

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
    let linker = Linker::new(engine.engine());
    let game =
        GameInstance::<TestInput, ()>::with_ram_limit(&engine, &module, &linker, test_ram_limit())
            .unwrap();

    runtime.load_game(game);
    assert!(runtime.game().is_some());
}

#[test]
fn test_runtime_init_game() {
    let console = TestConsole;
    let mut runtime = Runtime::new(console);

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
    let linker = Linker::new(engine.engine());
    let game =
        GameInstance::<TestInput, ()>::with_ram_limit(&engine, &module, &linker, test_ram_limit())
            .unwrap();

    runtime.load_game(game);
    let result = runtime.init_game();
    assert!(result.is_ok());
}

#[test]
fn test_runtime_init_no_game() {
    let console = TestConsole;
    let mut runtime = Runtime::new(console);

    // Should succeed even with no game loaded
    let result = runtime.init_game();
    assert!(result.is_ok());
}

// ============================================================================
// Session Tests
// ============================================================================

#[test]
fn test_runtime_set_session() {
    let console = TestConsole;
    let mut runtime = Runtime::<TestConsole>::new(console);

    let session = crate::rollback::RollbackSession::<TestInput, ()>::new_local(2, test_ram_limit());
    runtime.set_session(session);

    assert!(runtime.session().is_some());
    assert_eq!(runtime.session().unwrap().local_players().len(), 2);
}

#[test]
fn test_runtime_session_mut() {
    let console = TestConsole;
    let mut runtime = Runtime::<TestConsole>::new(console);

    let session = crate::rollback::RollbackSession::<TestInput, ()>::new_local(2, test_ram_limit());
    runtime.set_session(session);

    // Verify mutable access
    assert!(runtime.session_mut().is_some());
}

// ============================================================================
// Audio Tests
// ============================================================================

#[test]
fn test_runtime_set_audio() {
    let console = TestConsole;
    let mut runtime = Runtime::new(console);

    let audio = TestAudio {
        play_count: 0,
        stop_count: 0,
    };
    runtime.set_audio(audio);

    assert!(runtime.audio().is_some());
}

#[test]
fn test_runtime_audio_mut() {
    let console = TestConsole;
    let mut runtime = Runtime::new(console);

    let audio = TestAudio {
        play_count: 0,
        stop_count: 0,
    };
    runtime.set_audio(audio);

    // Verify mutable access
    assert!(runtime.audio_mut().is_some());
}

// ============================================================================
// Render Tests
// ============================================================================

#[test]
fn test_runtime_render_no_game() {
    let console = TestConsole;
    let mut runtime = Runtime::new(console);

    // Should succeed with no game
    let result = runtime.render();
    assert!(result.is_ok());
}

#[test]
fn test_runtime_render_with_game() {
    let console = TestConsole;
    let mut runtime = Runtime::new(console);

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
    let linker = Linker::new(engine.engine());
    let game =
        GameInstance::<TestInput, ()>::with_ram_limit(&engine, &module, &linker, test_ram_limit())
            .unwrap();

    runtime.load_game(game);
    let result = runtime.render();
    assert!(result.is_ok());
}

// ============================================================================
// Input Tests
// ============================================================================

#[test]
fn test_runtime_add_local_input_no_session() {
    let console = TestConsole;
    let mut runtime = Runtime::<TestConsole>::new(console);

    // Should succeed even without a session
    let result = runtime.add_local_input(
        0,
        TestInput {
            buttons: 0,
            x: 0,
            y: 0,
        },
    );
    assert!(result.is_ok());
}

#[test]
fn test_runtime_add_local_input_with_session() {
    let console = TestConsole;
    let mut runtime = Runtime::<TestConsole>::new(console);

    let session = crate::rollback::RollbackSession::<TestInput, ()>::new_local(2, test_ram_limit());
    runtime.set_session(session);

    // Local sessions don't use GGRS input, so this should succeed
    let result = runtime.add_local_input(
        0,
        TestInput {
            buttons: 1,
            x: 0,
            y: 0,
        },
    );
    assert!(result.is_ok());
}

// ============================================================================
// Session Events Tests
// ============================================================================

#[test]
fn test_runtime_handle_session_events_no_session() {
    let console = TestConsole;
    let mut runtime = Runtime::<TestConsole>::new(console);

    let events = runtime.handle_session_events();
    assert!(events.is_empty());
}

#[test]
fn test_runtime_handle_session_events_local_session() {
    let console = TestConsole;
    let mut runtime = Runtime::<TestConsole>::new(console);

    let session = crate::rollback::RollbackSession::<TestInput, ()>::new_local(2, test_ram_limit());
    runtime.set_session(session);

    // Local sessions don't produce events
    let events = runtime.handle_session_events();
    assert!(events.is_empty());
}

// ============================================================================
// Poll Remote Clients Tests
// ============================================================================

#[test]
fn test_runtime_poll_remote_clients_no_session() {
    let console = TestConsole;
    let mut runtime = Runtime::<TestConsole>::new(console);

    // Should not panic
    runtime.poll_remote_clients();
}

#[test]
fn test_runtime_poll_remote_clients_local_session() {
    let console = TestConsole;
    let mut runtime = Runtime::<TestConsole>::new(console);

    let session = crate::rollback::RollbackSession::<TestInput, ()>::new_local(2, test_ram_limit());
    runtime.set_session(session);

    // Should not panic (no-op for local sessions)
    runtime.poll_remote_clients();
}

// ============================================================================
// Test Console Implementation Tests
// ============================================================================

#[test]
fn test_console_specs() {
    let _console = TestConsole;
    let specs = TestConsole::specs();

    assert_eq!(specs.name, "Test Console");
    assert_eq!(specs.resolution, (320, 240)); // Fixed resolution
    assert_eq!(specs.tick_rates.len(), 2);
    assert_eq!(specs.ram_limit, 16 * 1024 * 1024); // Shared TestConsole has 16MB
}

#[test]
fn test_console_map_input() {
    let console = TestConsole;

    let raw = RawInput {
        button_a: true,
        button_b: false,
        ..Default::default()
    };
    let input = console.map_input(&raw);
    assert_eq!(input.buttons, 1);

    let raw = RawInput {
        button_a: false,
        button_b: true,
        ..Default::default()
    };
    let input = console.map_input(&raw);
    assert_eq!(input.buttons, 2);

    let raw = RawInput {
        button_a: true,
        button_b: true,
        ..Default::default()
    };
    let input = console.map_input(&raw);
    assert_eq!(input.buttons, 3);
}
