//! WASM runtime wrapper
//!
//! Provides abstractions over wasmtime for loading and executing game WASM modules.
//!
//! # Module Organization
//!
//! - [`state`] - Core game state structure (console-agnostic)
//!
//! # Key Types
//!
//! - [`WasmEngine`] - Shared WASM engine (one per application)
//! - [`GameInstance`] - Loaded and instantiated game
//! - [`GameState`] - Minimal core state (input, timing, RNG, saves)
//! - [`GameStateWithConsole`] - Wrapper combining core + console state

pub mod state;

use anyhow::{Context, Result};
use wasmtime::{Engine, Instance, Linker, Module, Store, TypedFunc};

use crate::console::ConsoleInput;

// Re-export public types from state module
pub use state::{GameState, GameStateWithConsole, MAX_PLAYERS, MAX_SAVE_SIZE, MAX_SAVE_SLOTS};

/// Shared WASM engine (one per application)
pub struct WasmEngine {
    engine: Engine,
}

impl WasmEngine {
    /// Create a new WASM engine with default configuration
    pub fn new() -> Result<Self> {
        let engine = Engine::default();
        Ok(Self { engine })
    }

    /// Get a reference to the underlying wasmtime engine
    pub fn engine(&self) -> &Engine {
        &self.engine
    }

    /// Load a WASM module from bytes
    pub fn load_module(&self, bytes: &[u8]) -> Result<Module> {
        Module::new(&self.engine, bytes).context("Failed to compile WASM module")
    }
}

// NOTE: WasmEngine intentionally does not implement Default.
// The WASM engine initialization is fallible (wasmtime::Engine::default() can fail
// on unsupported platforms or with invalid configuration). Using WasmEngine::new()
// returns Result<Self> which properly propagates initialization errors.

/// A loaded and instantiated game
/// A loaded and instantiated game
pub struct GameInstance<I: ConsoleInput, S: Send + Default + 'static> {
    store: Store<GameStateWithConsole<I, S>>,
    /// The WASM instance.
    /// Not directly used after initialization, but must be kept alive to maintain
    /// the lifetime of exported functions and memory references.
    #[allow(dead_code)]
    instance: Instance,
    init_fn: Option<TypedFunc<(), ()>>,
    update_fn: Option<TypedFunc<(), ()>>,
    render_fn: Option<TypedFunc<(), ()>>,
}

impl<I: ConsoleInput, S: Send + Default + 'static> GameInstance<I, S> {
    /// Create a new game instance from a module
    pub fn new(
        engine: &WasmEngine,
        module: &Module,
        linker: &Linker<GameStateWithConsole<I, S>>,
    ) -> Result<Self> {
        let mut store = Store::new(engine.engine(), GameStateWithConsole::new());
        let instance = linker
            .instantiate(&mut store, module)
            .context("Failed to instantiate WASM module")?;

        // Get the memory export
        if let Some(memory) = instance.get_memory(&mut store, "memory") {
            store.data_mut().game.memory = Some(memory);
        }

        // Look up exported functions
        let init_fn = instance.get_typed_func::<(), ()>(&mut store, "init").ok();
        let update_fn = instance.get_typed_func::<(), ()>(&mut store, "update").ok();
        let render_fn = instance.get_typed_func::<(), ()>(&mut store, "render").ok();

        Ok(Self {
            store,
            instance,
            init_fn,
            update_fn,
            render_fn,
        })
    }

    /// Call the game's init function
    pub fn init(&mut self) -> Result<()> {
        self.store.data_mut().game.in_init = true;
        if let Some(init) = &self.init_fn {
            init.call(&mut self.store, ())
                .context("Failed to call init()")?;
        }
        self.store.data_mut().game.in_init = false;
        Ok(())
    }

    /// Call the game's update function
    pub fn update(&mut self, delta_time: f32) -> Result<()> {
        {
            let state = &mut self.store.data_mut().game;
            state.delta_time = delta_time;
            state.elapsed_time += delta_time;
            state.tick_count += 1;
        }
        if let Some(update) = &self.update_fn {
            update
                .call(&mut self.store, ())
                .context("Failed to call update()")?;
        }
        // Rotate input state
        let state = &mut self.store.data_mut().game;
        state.input_prev = state.input_curr;
        Ok(())
    }

    /// Call the game's render function
    pub fn render(&mut self) -> Result<()> {
        if let Some(render) = &self.render_fn {
            render
                .call(&mut self.store, ())
                .context("Failed to call render()")?;
        }
        Ok(())
    }

    /// Save entire WASM linear memory to a vector (automatic snapshotting)
    ///
    /// This snapshots the entire WASM linear memory transparently. Games do not need
    /// to implement manual serialization - the entire memory is saved for rollback.
    /// Save entire WASM linear memory to a vector (automatic snapshotting)
    ///
    /// This snapshots the entire WASM linear memory transparently. Games do not need
    /// to implement manual serialization - the entire memory is saved for rollback.
    pub fn save_state(&mut self) -> Result<Vec<u8>> {
        let memory = self
            .store
            .data()
            .game
            .memory
            .context("No memory export found")?;
        let mem_data = memory.data(&self.store);
        Ok(mem_data.to_vec())
    }

    /// Load entire WASM linear memory from a snapshot (automatic snapshotting)
    ///
    /// Restores the entire WASM linear memory from a previous snapshot.
    /// This is the inverse of `save_state()`.
    pub fn load_state(&mut self, snapshot: &[u8]) -> Result<()> {
        let memory = self
            .store
            .data()
            .game
            .memory
            .context("No memory export found")?;
        let mem_data = memory.data_mut(&mut self.store);
        anyhow::ensure!(
            snapshot.len() == mem_data.len(),
            "Snapshot size mismatch: {} vs {}",
            snapshot.len(),
            mem_data.len()
        );
        mem_data.copy_from_slice(snapshot);
        Ok(())
    }

    /// Get mutable reference to the store
    pub fn store_mut(&mut self) -> &mut Store<GameStateWithConsole<I, S>> {
        &mut self.store
    }

    /// Get reference to the store
    pub fn store(&self) -> &Store<GameStateWithConsole<I, S>> {
        &self.store
    }

    /// Get mutable reference to game state
    pub fn state_mut(&mut self) -> &mut GameState<I> {
        &mut self.store.data_mut().game
    }

    /// Get reference to game state
    pub fn state(&self) -> &GameState<I> {
        &self.store.data().game
    }

    /// Get mutable reference to console-specific state
    pub fn console_state_mut(&mut self) -> &mut S {
        &mut self.store.data_mut().console
    }

    /// Get reference to console-specific state
    pub fn console_state(&self) -> &S {
        &self.store.data().console
    }

    /// Set input for a player
    pub fn set_input(&mut self, player: usize, input: I) {
        if player < MAX_PLAYERS {
            self.store.data_mut().game.input_curr[player] = input;
        }
    }

    /// Configure the session's player count and local player mask
    ///
    /// This should be called before `init()` to set up multiplayer state.
    /// The game can query these values via the `player_count()` and
    /// `local_player_mask()` FFI functions.
    ///
    /// # Arguments
    /// * `player_count` - Number of players in session (1-4)
    /// * `local_player_mask` - Bitmask of local players (bit N = player N is local)
    ///
    /// # Example
    /// ```ignore
    /// // 2 players, only player 0 is local (standard online play)
    /// game.configure_session(2, 0b0001);
    ///
    /// // 4 players, players 0 and 1 are local (2 local + 2 remote)
    /// game.configure_session(4, 0b0011);
    /// ```
    pub fn configure_session(&mut self, player_count: u32, local_player_mask: u32) {
        let state = &mut self.store.data_mut().game;
        state.player_count = player_count.min(MAX_PLAYERS as u32);
        state.local_player_mask = local_player_mask;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use glam::{Mat4, Vec3};
    use std::f32::consts::PI;

    use bytemuck::{Pod, Zeroable};

    #[repr(C)]
    #[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Pod, Zeroable)]
    struct TestInput {
        buttons: u16,
    }
    impl ConsoleInput for TestInput {}

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
        let linker = Linker::new(engine.engine());

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
        let linker = Linker::new(engine.engine());

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
        let linker = Linker::new(engine.engine());

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
        let linker = Linker::new(engine.engine());

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
        let linker = Linker::new(engine.engine());

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
        let linker = Linker::new(engine.engine());

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
        let linker = Linker::new(engine.engine());

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
        let linker = Linker::new(engine.engine());

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
        let linker = Linker::new(engine.engine());

        let mut game = GameInstance::<TestInput, ()>::new(&engine, &module, &linker).unwrap();

        // Set input for player 0
        let input1 = TestInput {
            buttons: 0x0001,
        };
        game.set_input(0, input1);

        // Call update (which rotates input_prev = input_curr)
        game.update(1.0 / 60.0).unwrap();

        // Previous should now have our input
        assert_eq!(game.state().input_prev[0].buttons, 0x0001);

        // Set new input
        let input2 = TestInput {
            buttons: 0x0002,
        };
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
        let linker = Linker::new(engine.engine());

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
        let linker = Linker::new(engine.engine());

        let mut game = GameInstance::new(&engine, &module, &linker).unwrap();

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
        let linker = Linker::new(engine.engine());

        let mut game = GameInstance::new(&engine, &module, &linker).unwrap();

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
        let linker = Linker::new(engine.engine());

        let mut game = GameInstance::new(&engine, &module, &linker).unwrap();

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
        let linker = Linker::new(engine.engine());

        let mut game = GameInstance::new(&engine, &module, &linker).unwrap();

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
        let linker = Linker::new(engine.engine());

        let mut game = GameInstance::new(&engine, &module, &linker).unwrap();

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
        let linker = Linker::new(engine.engine());

        let mut game = GameInstance::new(&engine, &module, &linker).unwrap();

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
        let linker = Linker::new(engine.engine());

        let mut game = GameInstance::new(&engine, &module, &linker).unwrap();

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
        let linker = Linker::new(engine.engine());

        let mut game = GameInstance::new(&engine, &module, &linker).unwrap();

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
        let linker = Linker::new(engine.engine());

        let mut game = GameInstance::new(&engine, &module, &linker).unwrap();

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
        let linker = Linker::new(engine.engine());

        let mut game = GameInstance::new(&engine, &module, &linker).unwrap();

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
        let linker = Linker::new(engine.engine());

        let mut game = GameInstance::new(&engine, &module, &linker).unwrap();

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
        let linker = Linker::new(engine.engine());

        let mut game = GameInstance::new(&engine, &module, &linker).unwrap();

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
}
