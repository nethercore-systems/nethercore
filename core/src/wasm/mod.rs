//! WASM runtime wrapper
//!
//! Provides abstractions over wasmtime for loading and executing game WASM modules.
//!
//! # Module Organization
//!
//! - [`camera`] - Camera state with view/projection matrix calculations
//! - [`draw`] - Draw commands and pending resource structures
//! - [`input`] - Player input state
//! - [`render`] - Render state, lighting, and init-time configuration
//! - [`state`] - Main game state structure
//!
//! # Key Types
//!
//! - [`WasmEngine`] - Shared WASM engine (one per application)
//! - [`GameInstance`] - Loaded and instantiated game
//! - [`GameState`] - Per-game mutable state stored in wasmtime Store

pub mod camera;
pub mod draw;
pub mod input;
pub mod render;
pub mod state;

use anyhow::{Context, Result};
use wasmtime::{Engine, Instance, Linker, Module, Store, TypedFunc};

// Re-export all public types from submodules
pub use camera::{CameraState, DEFAULT_CAMERA_FOV};
pub use draw::{DrawCommand, PendingMesh, PendingTexture};
pub use input::InputState;
pub use render::{InitConfig, LightState, RenderState, MAX_BONES};
pub use state::{GameState, MAX_PLAYERS, MAX_SAVE_SIZE, MAX_SAVE_SLOTS, MAX_TRANSFORM_STACK};

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
pub struct GameInstance {
    store: Store<GameState>,
    /// The WASM instance.
    /// Not directly used after initialization, but must be kept alive to maintain
    /// the lifetime of exported functions and memory references.
    #[allow(dead_code)]
    instance: Instance,
    init_fn: Option<TypedFunc<(), ()>>,
    update_fn: Option<TypedFunc<(), ()>>,
    render_fn: Option<TypedFunc<(), ()>>,
    save_state_fn: Option<TypedFunc<(u32, u32), u32>>,
    load_state_fn: Option<TypedFunc<(u32, u32), ()>>,
}

impl GameInstance {
    /// Create a new game instance from a module
    pub fn new(engine: &WasmEngine, module: &Module, linker: &Linker<GameState>) -> Result<Self> {
        let mut store = Store::new(engine.engine(), GameState::new());
        let instance = linker
            .instantiate(&mut store, module)
            .context("Failed to instantiate WASM module")?;

        // Get the memory export
        if let Some(memory) = instance.get_memory(&mut store, "memory") {
            store.data_mut().memory = Some(memory);
        }

        // Look up exported functions
        let init_fn = instance.get_typed_func::<(), ()>(&mut store, "init").ok();
        let update_fn = instance
            .get_typed_func::<(), ()>(&mut store, "update")
            .ok();
        let render_fn = instance
            .get_typed_func::<(), ()>(&mut store, "render")
            .ok();
        let save_state_fn = instance
            .get_typed_func::<(u32, u32), u32>(&mut store, "save_state")
            .ok();
        let load_state_fn = instance
            .get_typed_func::<(u32, u32), ()>(&mut store, "load_state")
            .ok();

        Ok(Self {
            store,
            instance,
            init_fn,
            update_fn,
            render_fn,
            save_state_fn,
            load_state_fn,
        })
    }

    /// Call the game's init function
    pub fn init(&mut self) -> Result<()> {
        self.store.data_mut().in_init = true;
        if let Some(init) = &self.init_fn {
            init.call(&mut self.store, ())
                .context("Failed to call init()")?;
        }
        self.store.data_mut().in_init = false;
        Ok(())
    }

    /// Call the game's update function
    pub fn update(&mut self, delta_time: f32) -> Result<()> {
        {
            let state = self.store.data_mut();
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
        let state = self.store.data_mut();
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

    /// Save game state to a buffer
    pub fn save_state(&mut self, buffer: &mut [u8]) -> Result<usize> {
        if let Some(save_state) = &self.save_state_fn {
            let memory = self
                .store
                .data()
                .memory
                .context("No memory export found")?;
            let ptr = 0u32; // Use start of memory for now (games should allocate)
            let max_len = buffer.len() as u32;

            let len = save_state
                .call(&mut self.store, (ptr, max_len))
                .context("Failed to call save_state()")?;

            let mem_data = memory.data(&self.store);
            let len = len as usize;
            if len <= buffer.len() && (ptr as usize + len) <= mem_data.len() {
                buffer[..len].copy_from_slice(&mem_data[ptr as usize..ptr as usize + len]);
                Ok(len)
            } else {
                anyhow::bail!("save_state returned invalid length")
            }
        } else {
            Ok(0)
        }
    }

    /// Load game state from a buffer
    pub fn load_state(&mut self, buffer: &[u8]) -> Result<()> {
        if let Some(load_state) = &self.load_state_fn {
            let memory = self
                .store
                .data()
                .memory
                .context("No memory export found")?;
            let ptr = 0u32;
            let len = buffer.len() as u32;

            // Copy buffer into WASM memory
            let mem_data = memory.data_mut(&mut self.store);
            if (ptr as usize + buffer.len()) <= mem_data.len() {
                mem_data[ptr as usize..ptr as usize + buffer.len()].copy_from_slice(buffer);
            } else {
                anyhow::bail!("Buffer too large for WASM memory");
            }

            load_state
                .call(&mut self.store, (ptr, len))
                .context("Failed to call load_state()")?;
        }
        Ok(())
    }

    /// Get mutable reference to the store
    pub fn store_mut(&mut self) -> &mut Store<GameState> {
        &mut self.store
    }

    /// Get reference to the store
    pub fn store(&self) -> &Store<GameState> {
        &self.store
    }

    /// Get mutable reference to game state
    pub fn state_mut(&mut self) -> &mut GameState {
        self.store.data_mut()
    }

    /// Get reference to game state
    pub fn state(&self) -> &GameState {
        self.store.data()
    }

    /// Set input for a player
    pub fn set_input(&mut self, player: usize, input: InputState) {
        if player < MAX_PLAYERS {
            self.store.data_mut().input_curr[player] = input;
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
        let state = self.store.data_mut();
        state.player_count = player_count.min(MAX_PLAYERS as u32);
        state.local_player_mask = local_player_mask;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use glam::{Mat4, Vec3};
    use std::f32::consts::PI;

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

        let result = GameInstance::new(&engine, &module, &linker);
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

        let mut game = GameInstance::new(&engine, &module, &linker).unwrap();
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

        let mut game = GameInstance::new(&engine, &module, &linker).unwrap();
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

        let mut game = GameInstance::new(&engine, &module, &linker).unwrap();

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

        let mut game = GameInstance::new(&engine, &module, &linker).unwrap();
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

        let mut game = GameInstance::new(&engine, &module, &linker).unwrap();
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

        let mut game = GameInstance::new(&engine, &module, &linker).unwrap();

        let input = InputState {
            buttons: 0x00FF,
            left_stick_x: 100,
            left_stick_y: -50,
            right_stick_x: 25,
            right_stick_y: -25,
            left_trigger: 200,
            right_trigger: 100,
        };

        game.set_input(0, input);
        assert_eq!(game.state().input_curr[0].buttons, 0x00FF);
        assert_eq!(game.state().input_curr[0].left_stick_x, 100);
        assert_eq!(game.state().input_curr[0].left_trigger, 200);
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

        let mut game = GameInstance::new(&engine, &module, &linker).unwrap();

        // Should not panic for invalid player index
        game.set_input(10, InputState::default());
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

        let mut game = GameInstance::new(&engine, &module, &linker).unwrap();

        // Set input for player 0
        let input1 = InputState {
            buttons: 0x0001,
            ..Default::default()
        };
        game.set_input(0, input1);

        // Call update (which rotates input_prev = input_curr)
        game.update(1.0 / 60.0).unwrap();

        // Previous should now have our input
        assert_eq!(game.state().input_prev[0].buttons, 0x0001);

        // Set new input
        let input2 = InputState {
            buttons: 0x0002,
            ..Default::default()
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

        let mut game = GameInstance::new(&engine, &module, &linker).unwrap();

        // Test mutable access
        game.state_mut().player_count = 4;
        assert_eq!(game.state().player_count, 4);

        // Test store access
        let _store = game.store();
        let _store_mut = game.store_mut();
    }

    #[test]
    fn test_game_instance_save_state_no_function() {
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
        let mut buffer = vec![0u8; 1024];

        // Should return Ok(0) when save_state is not exported
        let result = game.save_state(&mut buffer);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
    }

    #[test]
    fn test_game_instance_load_state_no_function() {
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

        // Should return Ok when load_state is not exported
        let result = game.load_state(&[1, 2, 3, 4]);
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
    fn test_game_instance_save_state_returns_invalid_length() {
        let engine = WasmEngine::new().unwrap();
        // Module with save_state that returns a length larger than the buffer
        let wasm = wat::parse_str(
            r#"
            (module
                (memory (export "memory") 1)
                (func (export "save_state") (param i32 i32) (result i32)
                    ;; Return more than the max_len provided
                    (i32.const 99999)
                )
            )
        "#,
        )
        .unwrap();
        let module = engine.load_module(&wasm).unwrap();
        let linker = Linker::new(engine.engine());

        let mut game = GameInstance::new(&engine, &module, &linker).unwrap();
        let mut buffer = vec![0u8; 100];

        // save_state should fail because returned length > buffer.len()
        let result = game.save_state(&mut buffer);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("invalid length"));
    }

    #[test]
    fn test_game_instance_save_state_oob_ptr() {
        let engine = WasmEngine::new().unwrap();
        // Module with save_state that writes past memory bounds
        // ptr=0 but length exceeds memory
        let wasm = wat::parse_str(
            r#"
            (module
                (memory (export "memory") 1)
                (func (export "save_state") (param i32 i32) (result i32)
                    ;; Return a length that would read past memory end at ptr=0
                    ;; Memory is 65536 bytes, return 100000
                    (i32.const 100000)
                )
            )
        "#,
        )
        .unwrap();
        let module = engine.load_module(&wasm).unwrap();
        let linker = Linker::new(engine.engine());

        let mut game = GameInstance::new(&engine, &module, &linker).unwrap();
        let mut buffer = vec![0u8; 100001];

        // Should fail due to out-of-bounds read
        let result = game.save_state(&mut buffer);
        assert!(result.is_err());
    }

    #[test]
    fn test_game_instance_load_state_too_large() {
        let engine = WasmEngine::new().unwrap();
        // Module with load_state
        let wasm = wat::parse_str(
            r#"
            (module
                (memory (export "memory") 1)
                (func (export "load_state") (param i32 i32))
            )
        "#,
        )
        .unwrap();
        let module = engine.load_module(&wasm).unwrap();
        let linker = Linker::new(engine.engine());

        let mut game = GameInstance::new(&engine, &module, &linker).unwrap();

        // Try to load more data than fits in memory (1 page = 65536 bytes)
        let large_buffer = vec![0u8; 100000];
        let result = game.load_state(&large_buffer);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("too large"));
    }

    #[test]
    fn test_game_instance_load_state_no_memory() {
        let engine = WasmEngine::new().unwrap();
        // Module without memory export
        let wasm = wat::parse_str(
            r#"
            (module
                (func (export "load_state") (param i32 i32))
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
        // Module without memory export but with save_state
        let wasm = wat::parse_str(
            r#"
            (module
                (func (export "save_state") (param i32 i32) (result i32)
                    (i32.const 10)
                )
            )
        "#,
        )
        .unwrap();
        let module = engine.load_module(&wasm).unwrap();
        let linker = Linker::new(engine.engine());

        let mut game = GameInstance::new(&engine, &module, &linker).unwrap();
        let mut buffer = vec![0u8; 1024];

        // Should fail because no memory is available
        let result = game.save_state(&mut buffer);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("No memory"));
    }

    #[test]
    fn test_game_instance_save_state_valid() {
        let engine = WasmEngine::new().unwrap();
        // Module that writes valid data at ptr and returns the length
        let wasm = wat::parse_str(
            r#"
            (module
                (memory (export "memory") 1)
                ;; Initialize first 8 bytes with a pattern
                (data (i32.const 0) "\01\02\03\04\05\06\07\08")
                (func (export "save_state") (param $ptr i32) (param $max_len i32) (result i32)
                    ;; Return 8 bytes written at ptr=0
                    (i32.const 8)
                )
            )
        "#,
        )
        .unwrap();
        let module = engine.load_module(&wasm).unwrap();
        let linker = Linker::new(engine.engine());

        let mut game = GameInstance::new(&engine, &module, &linker).unwrap();
        let mut buffer = vec![0u8; 100];

        let result = game.save_state(&mut buffer);
        assert!(result.is_ok());
        let len = result.unwrap();
        assert_eq!(len, 8);
        assert_eq!(&buffer[..8], &[1, 2, 3, 4, 5, 6, 7, 8]);
    }

    #[test]
    fn test_game_instance_load_state_valid() {
        let engine = WasmEngine::new().unwrap();
        // Module that just accepts load_state
        let wasm = wat::parse_str(
            r#"
            (module
                (memory (export "memory") 1)
                (func (export "load_state") (param $ptr i32) (param $len i32)
                    ;; Do nothing - just accept the call
                )
            )
        "#,
        )
        .unwrap();
        let module = engine.load_module(&wasm).unwrap();
        let linker = Linker::new(engine.engine());

        let mut game = GameInstance::new(&engine, &module, &linker).unwrap();

        let data = vec![0xDE, 0xAD, 0xBE, 0xEF];
        let result = game.load_state(&data);
        assert!(result.is_ok());
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
