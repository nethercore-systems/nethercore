//! Console runner for game execution
//!
//! Provides a high-level abstraction for running games on any console.
//! The ConsoleRunner coordinates game execution and provides a simplified
//! interface for the library crate.

use std::sync::Arc;

use anyhow::Result;
use wasmtime::Linker;
use winit::window::Window;

use crate::{
    app::session::GameSession,
    console::{Console, Graphics, RawInput},
    ffi::register_common_ffi,
    rollback::{RollbackSession, SessionEvent},
    runtime::Runtime,
    wasm::{GameInstance, WasmEngine, WasmGameContext},
};

/// High-level game runner for any console type.
///
/// Owns all components needed to run a game:
/// - Console instance
/// - Graphics backend
/// - Audio backend
/// - WASM engine
/// - Game session (runtime + resource manager)
///
/// Provides a simplified interface for:
/// - Loading games
/// - Processing frames
/// - Handling input
/// - Rendering
pub struct ConsoleRunner<C: Console> {
    /// Graphics backend
    graphics: C::Graphics,
    /// Audio backend
    audio: C::Audio,
    /// WASM engine (shared, can be cloned for multiple games)
    wasm_engine: WasmEngine,
    /// Active game session (None if no game loaded)
    session: Option<GameSession<C>>,
    /// Cached console specs
    specs: &'static crate::console::ConsoleSpecs,
}

impl<C: Console> ConsoleRunner<C> {
    /// Create a new console runner.
    ///
    /// # Arguments
    /// * `console` - The console implementation (used for initialization, then stored in Runtime)
    /// * `window` - The window for graphics initialization
    ///
    /// # Errors
    /// Returns an error if graphics or audio initialization fails.
    pub fn new(console: C, window: Arc<Window>) -> Result<Self> {
        let graphics = console.create_graphics(window)?;
        let audio = console.create_audio()?;
        let wasm_engine = WasmEngine::new()?;
        let specs = C::specs();

        // Note: We don't store the console here because the Runtime takes ownership
        // when a game is loaded. The console is primarily needed for its specs
        // and for creating the graphics/audio/resource_manager.

        Ok(Self {
            graphics,
            audio,
            wasm_engine,
            session: None,
            specs,
        })
    }

    /// Get a mutable reference to the graphics backend.
    pub fn graphics_mut(&mut self) -> &mut C::Graphics {
        &mut self.graphics
    }

    /// Get a reference to the graphics backend.
    pub fn graphics(&self) -> &C::Graphics {
        &self.graphics
    }

    /// Get a mutable reference to the audio backend.
    pub fn audio_mut(&mut self) -> &mut C::Audio {
        &mut self.audio
    }

    /// Get mutable references to both graphics and session simultaneously.
    ///
    /// This enables operations that need both, which would otherwise fail
    /// the borrow checker due to the session borrowing from the runner.
    pub fn graphics_and_session_mut(&mut self) -> (&mut C::Graphics, Option<&mut GameSession<C>>) {
        (&mut self.graphics, self.session.as_mut())
    }

    /// Get a reference to the WASM engine.
    pub fn wasm_engine(&self) -> &WasmEngine {
        &self.wasm_engine
    }

    /// Get the console specs.
    pub fn specs(&self) -> &'static crate::console::ConsoleSpecs {
        self.specs
    }

    /// Check if a game is currently loaded.
    pub fn has_game(&self) -> bool {
        self.session.is_some()
    }

    /// Get a mutable reference to the active game session.
    pub fn session_mut(&mut self) -> Option<&mut GameSession<C>> {
        self.session.as_mut()
    }

    /// Get a reference to the active game session.
    pub fn session(&self) -> Option<&GameSession<C>> {
        self.session.as_ref()
    }

    /// Load a game from WASM bytes.
    ///
    /// # Arguments
    /// * `console` - Fresh console instance for the game
    /// * `wasm_bytes` - The compiled WASM code
    /// * `num_players` - Number of local players (1-4)
    ///
    /// # Errors
    /// Returns an error if the game fails to load or initialize.
    pub fn load_game(&mut self, console: C, wasm_bytes: &[u8], num_players: usize) -> Result<()> {
        // Load and validate the WASM module
        let module = self.wasm_engine.load_module(wasm_bytes)?;
        WasmEngine::validate_module_memory(&module, self.specs.ram_limit)?;

        // Create a linker and register FFI functions
        let mut linker: Linker<WasmGameContext<C::Input, C::State, C::RollbackState>> =
            Linker::new(self.wasm_engine.engine());

        // Register common FFI functions (input, random, save/load, etc.)
        register_common_ffi(&mut linker)?;

        // Register console-specific FFI functions
        console.register_ffi(&mut linker)?;

        // Create game instance with the linker
        let game = GameInstance::with_ram_limit(
            &self.wasm_engine,
            &module,
            &linker,
            self.specs.ram_limit,
        )?;

        // Create runtime (takes ownership of console)
        let mut runtime = Runtime::new(console);
        runtime.load_game(game);
        runtime.set_tick_rate(self.specs.tick_rates[self.specs.default_tick_rate]);

        // Create and set audio backend for the runtime
        // (separate from ConsoleRunner's audio which is used for resource loading)
        let audio = runtime.console().create_audio()?;
        runtime.set_audio(audio);

        // Create local rollback session
        let rollback_session = RollbackSession::new_local(num_players, self.specs.ram_limit);
        runtime.set_session(rollback_session);

        // Configure the game with session player info before init()
        // This ensures player_count() and local_player_mask() FFI return correct values
        let session_info = runtime
            .session()
            .map(|s| (s.player_config().num_players(), s.player_config().local_player_mask()));
        if let Some((player_count, local_mask)) = session_info {
            if let Some(game) = runtime.game_mut() {
                game.configure_session(player_count, local_mask);
            }
        }

        // Initialize console-specific FFI state before calling game init()
        // (e.g., set datapack for rom_* functions)
        runtime.initialize_console_state();

        // Initialize the game (calls init() export)
        runtime.init_game()?;

        // Create resource manager from console reference
        let resource_manager = runtime.console().create_resource_manager();

        // Set up game session
        let mut session = GameSession::new(runtime, resource_manager);

        // Process resources created during init
        session.process_pending_resources(&mut self.graphics, &mut self.audio)?;

        self.session = Some(session);
        Ok(())
    }

    /// Load a game with a pre-configured rollback session.
    ///
    /// Use this method when you need to specify a custom session type
    /// (sync-test, P2P, etc.) instead of the default local session.
    ///
    /// # Arguments
    /// * `console` - Fresh console instance for the game
    /// * `wasm_bytes` - The compiled WASM code
    /// * `session` - Pre-configured rollback session
    ///
    /// # Errors
    /// Returns an error if the game fails to load or initialize.
    pub fn load_game_with_session(
        &mut self,
        console: C,
        wasm_bytes: &[u8],
        session: RollbackSession<C::Input, C::State, C::RollbackState>,
    ) -> Result<()> {
        // Load and validate the WASM module
        let module = self.wasm_engine.load_module(wasm_bytes)?;
        WasmEngine::validate_module_memory(&module, self.specs.ram_limit)?;

        // Create a linker and register FFI functions
        let mut linker: Linker<WasmGameContext<C::Input, C::State, C::RollbackState>> =
            Linker::new(self.wasm_engine.engine());

        // Register common FFI functions (input, random, save/load, etc.)
        register_common_ffi(&mut linker)?;

        // Register console-specific FFI functions
        console.register_ffi(&mut linker)?;

        // Create game instance with the linker
        let game = GameInstance::with_ram_limit(
            &self.wasm_engine,
            &module,
            &linker,
            self.specs.ram_limit,
        )?;

        // Create runtime (takes ownership of console)
        let mut runtime = Runtime::new(console);
        runtime.load_game(game);
        runtime.set_tick_rate(self.specs.tick_rates[self.specs.default_tick_rate]);

        // Create and set audio backend for the runtime
        let audio = runtime.console().create_audio()?;
        runtime.set_audio(audio);

        // Set the provided rollback session
        runtime.set_session(session);

        // Configure the game with session player info before init()
        // This ensures player_count() and local_player_mask() FFI return correct values
        let session_info = runtime
            .session()
            .map(|s| (s.player_config().num_players(), s.player_config().local_player_mask()));
        if let Some((num_players, local_mask)) = session_info {
            if let Some(game) = runtime.game_mut() {
                game.configure_session(num_players, local_mask);
            }
        }

        // Initialize console-specific FFI state before calling game init()
        runtime.initialize_console_state();

        // Initialize the game (calls init() export)
        runtime.init_game()?;

        // Create resource manager from console reference
        let resource_manager = runtime.console().create_resource_manager();

        // Set up game session
        let mut game_session = GameSession::new(runtime, resource_manager);

        // Process resources created during init
        game_session.process_pending_resources(&mut self.graphics, &mut self.audio)?;

        self.session = Some(game_session);
        Ok(())
    }

    /// Unload the current game.
    pub fn unload_game(&mut self) {
        self.session = None;
    }

    /// Process input and add it to the game.
    ///
    /// # Arguments
    /// * `player` - Player index (0-3)
    /// * `raw_input` - Raw input from input manager
    pub fn add_input(&mut self, player: usize, raw_input: &RawInput) {
        if let Some(session) = &mut self.session {
            let input = session.runtime.console().map_input(raw_input);
            if let Some(game) = session.runtime.game_mut() {
                game.set_input(player, input);
            }
            // Also add to rollback session
            let _ = session.runtime.add_local_input(player, input);
        }
    }

    /// Run a single frame of the game.
    ///
    /// This runs fixed-timestep updates and returns the number of ticks
    /// that were executed plus the interpolation factor.
    ///
    /// # Returns
    /// * `Ok((ticks, interpolation))` - Number of ticks run and interpolation factor
    /// * `Err(...)` - If the game encounters an error
    pub fn update(&mut self) -> Result<(u32, f32)> {
        if let Some(session) = &mut self.session {
            session.runtime.frame()
        } else {
            Ok((0, 0.0))
        }
    }

    /// Render the current game state.
    ///
    /// This calls the game's render() function and executes draw commands.
    ///
    /// # Errors
    /// Returns an error if rendering fails.
    pub fn render(&mut self) -> Result<()> {
        if let Some(session) = &mut self.session {
            // Call game's render function
            session.runtime.render()?;

            // Execute accumulated draw commands
            session.execute_draw_commands(&mut self.graphics)?;
        }
        Ok(())
    }

    /// Begin a new graphics frame.
    pub fn begin_frame(&mut self) {
        self.graphics.begin_frame();
    }

    /// End the current graphics frame and present.
    pub fn end_frame(&mut self) {
        self.graphics.end_frame();
    }

    /// Handle window resize.
    pub fn resize(&mut self, width: u32, height: u32) {
        self.graphics.resize(width, height);
    }

    /// Poll remote clients (for networked sessions).
    pub fn poll_remote_clients(&mut self) {
        if let Some(session) = &mut self.session {
            session.runtime.poll_remote_clients();
        }
    }

    /// Handle and return session events.
    pub fn handle_session_events(&mut self) -> Vec<SessionEvent> {
        if let Some(session) = &mut self.session {
            session.runtime.handle_session_events()
        } else {
            Vec::new()
        }
    }

    /// Check if the game requested to quit.
    pub fn quit_requested(&self) -> bool {
        if let Some(session) = &self.session
            && let Some(game) = session.runtime.game()
        {
            return game.state().quit_requested;
        }
        false
    }
}
