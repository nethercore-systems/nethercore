//! Game instance implementation for loaded WASM modules

use anyhow::{Context, Result};
use wasmtime::{Instance, Linker, Module, Store, TypedFunc, Val};

use super::engine::WasmEngine;
use super::state::{DEFAULT_RAM_LIMIT, GameState, MAX_PLAYERS, WasmGameContext};
use crate::console::{ConsoleInput, ConsoleRollbackState};
use crate::debug::types::ActionParamValue;

/// A loaded and instantiated game
pub struct GameInstance<I: ConsoleInput, S: Send + Default + 'static, R: ConsoleRollbackState = ()>
{
    store: Store<WasmGameContext<I, S, R>>,
    /// The WASM instance.
    /// Not directly used after initialization, but must be kept alive to maintain
    /// the lifetime of exported functions and memory references.
    #[allow(dead_code)]
    instance: Instance,
    init_fn: Option<TypedFunc<(), ()>>,
    update_fn: Option<TypedFunc<(), ()>>,
    render_fn: Option<TypedFunc<(), ()>>,
    on_debug_change_fn: Option<TypedFunc<(), ()>>,
    /// Optional post_connect function for two-phase initialization.
    /// Called after NCHS handshake completes, before game loop starts.
    post_connect_fn: Option<TypedFunc<(), ()>>,
}

impl<I: ConsoleInput, S: Send + Default + 'static, R: ConsoleRollbackState> GameInstance<I, S, R> {
    /// Create a new game instance from a module with the fallback RAM limit (4MB)
    pub fn new(
        engine: &WasmEngine,
        module: &Module,
        linker: &Linker<WasmGameContext<I, S, R>>,
    ) -> Result<Self> {
        // Fallback for tests and tooling; prefer using ConsoleSpecs::ram_limit.
        Self::with_ram_limit(engine, module, linker, DEFAULT_RAM_LIMIT)
    }

    /// Create a new game instance from a module with specified RAM limit
    ///
    /// The RAM limit enforces how much WASM linear memory the game can use.
    /// This should match the console's `ConsoleSpecs::ram_limit`.
    ///
    /// # Arguments
    /// * `engine` - The WASM engine
    /// * `module` - The compiled WASM module
    /// * `linker` - The linker with FFI functions registered
    /// * `ram_limit` - Maximum linear memory in bytes (e.g., 8MB for Nethercore ZX)
    pub fn with_ram_limit(
        engine: &WasmEngine,
        module: &Module,
        linker: &Linker<WasmGameContext<I, S, R>>,
        ram_limit: usize,
    ) -> Result<Self> {
        let mut store = Store::new(engine.engine(), WasmGameContext::with_ram_limit(ram_limit));

        // Enable resource limiter to enforce memory constraints
        store.limiter(|state| state);

        let instance = linker
            .instantiate(&mut store, module)
            .map_err(|e| {
                eprintln!("WASM instantiation error: {:#?}", e);
                e
            })
            .context("Failed to instantiate WASM module")?;

        // Get the memory export
        if let Some(memory) = instance.get_memory(&mut store, "memory") {
            store.data_mut().game.memory = Some(memory);
        }

        // Look up exported functions
        let init_fn = instance.get_typed_func::<(), ()>(&mut store, "init").ok();
        let update_fn = instance.get_typed_func::<(), ()>(&mut store, "update").ok();
        let render_fn = instance.get_typed_func::<(), ()>(&mut store, "render").ok();
        let on_debug_change_fn = instance
            .get_typed_func::<(), ()>(&mut store, "on_debug_change")
            .ok();
        let post_connect_fn = instance
            .get_typed_func::<(), ()>(&mut store, "post_connect")
            .ok();

        Ok(Self {
            store,
            instance,
            init_fn,
            update_fn,
            render_fn,
            on_debug_change_fn,
            post_connect_fn,
        })
    }

    /// Call the game's init function
    pub fn init(&mut self) -> Result<()> {
        self.store.data_mut().game.in_init = true;
        if let Some(init) = &self.init_fn {
            init.call(&mut self.store, ()).map_err(|e| {
                // Extract more detailed error information from wasmtime
                let error_msg = format!("WASM init() failed: {:#}", e);
                eprintln!("{}", error_msg);
                anyhow::anyhow!(error_msg)
            })?;
        }
        self.store.data_mut().game.in_init = false;
        Ok(())
    }

    /// Call the game's post_connect function (two-phase initialization)
    ///
    /// This is called after NCHS handshake completes, when the game knows its
    /// player handle and can access player_handle()/is_connected() FFI functions.
    ///
    /// # Initialization Flow
    ///
    /// 1. `init()` - Basic setup (NO player_handle access)
    /// 2. NCHS handshake completes
    /// 3. `apply_session_config()` - Sets player_handle and random seed
    /// 4. `post_connect()` - Player-aware setup (CAN access player_handle)
    /// 5. Game loop begins
    pub fn post_connect(&mut self) -> Result<()> {
        if let Some(post_connect) = &self.post_connect_fn {
            post_connect.call(&mut self.store, ()).map_err(|e| {
                let error_msg = format!("WASM post_connect() failed: {:#}", e);
                eprintln!("{}", error_msg);
                anyhow::anyhow!(error_msg)
            })?;
        }
        Ok(())
    }

    /// Check if the game exports a post_connect function
    pub fn has_post_connect(&self) -> bool {
        self.post_connect_fn.is_some()
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
            update.call(&mut self.store, ()).map_err(|e| {
                let error_msg = format!(
                    "WASM update() failed at tick {}: {:#}",
                    self.store.data().game.tick_count,
                    e
                );
                eprintln!("{}", error_msg);
                anyhow::anyhow!(error_msg)
            })?;
        }
        // Rotate input state
        let state = &mut self.store.data_mut().game;
        state.input_prev = state.input_curr;
        Ok(())
    }

    /// Call the game's render function
    pub fn render(&mut self) -> Result<()> {
        if let Some(render) = &self.render_fn {
            render.call(&mut self.store, ()).map_err(|e| {
                let error_msg = format!("WASM render() failed: {:#}", e);
                eprintln!("{}", error_msg);
                anyhow::anyhow!(error_msg)
            })?;
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
    pub fn store_mut(&mut self) -> &mut Store<WasmGameContext<I, S, R>> {
        &mut self.store
    }

    /// Get reference to the store
    pub fn store(&self) -> &Store<WasmGameContext<I, S, R>> {
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

    /// Get mutable reference to console-specific FFI state
    pub fn console_state_mut(&mut self) -> &mut S {
        &mut self.store.data_mut().ffi
    }

    /// Get reference to console-specific FFI state
    pub fn console_state(&self) -> &S {
        &self.store.data().ffi
    }

    /// Get mutable reference to console-specific rollback state
    pub fn rollback_state_mut(&mut self) -> &mut R {
        &mut self.store.data_mut().rollback
    }

    /// Get reference to console-specific rollback state
    pub fn rollback_state(&self) -> &R {
        &self.store.data().rollback
    }

    /// Get split borrows: mutable FFI state and mutable rollback state
    ///
    /// This allows mutating both FFI state (e.g., tracker engine channel positions)
    /// and rollback state (e.g., audio playhead positions) during audio generation.
    pub fn ffi_and_rollback_mut(&mut self) -> (&mut S, &mut R) {
        let ctx = self.store.data_mut();
        (&mut ctx.ffi, &mut ctx.rollback)
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

    /// Call the game's on_debug_change function if it exists
    ///
    /// This is called when debug values are modified through the debug panel.
    /// Games can optionally export this function to react to debug value changes.
    pub fn call_on_debug_change(&mut self) {
        if let Some(func) = &self.on_debug_change_fn
            && let Err(e) = func.call(&mut self.store, ())
        {
            tracing::warn!("on_debug_change() failed: {}", e);
        }
    }

    /// Returns true if the game exports an on_debug_change function
    pub fn has_debug_change_callback(&self) -> bool {
        self.on_debug_change_fn.is_some()
    }

    /// Call a debug action by function name with arguments
    ///
    /// This is used to invoke WASM functions from the debug panel's action buttons.
    /// The function must be exported by the game.
    pub fn call_action(&mut self, func_name: &str, args: &[ActionParamValue]) -> Result<()> {
        let func = self
            .instance
            .get_func(&mut self.store, func_name)
            .ok_or_else(|| anyhow::anyhow!("Action function '{}' not exported", func_name))?;

        // Convert ActionParamValue to wasmtime::Val
        let vals: Vec<Val> = args
            .iter()
            .map(|arg| match arg {
                ActionParamValue::I32(v) => Val::I32(*v),
                ActionParamValue::F32(v) => Val::F32(v.to_bits()),
            })
            .collect();

        // Call with no expected results (fire and forget)
        func.call(&mut self.store, &vals, &mut [])
            .with_context(|| format!("Failed to call action '{}'", func_name))?;

        Ok(())
    }
}
