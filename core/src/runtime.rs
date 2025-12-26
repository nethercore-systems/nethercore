//! Game loop orchestration
//!
//! Manages the main game loop with fixed timestep updates
//! and variable render rate.

use std::time::{Duration, Instant};

use anyhow::Result;
use ggrs::{GgrsError, SessionState};

use crate::console::Console;
use crate::rollback::{RollbackSession, SessionEvent};
use crate::wasm::GameInstance;

/// Runtime configuration
#[derive(Debug, Clone)]
pub struct RuntimeConfig {
    /// Target tick rate in Hz
    pub tick_rate: u32,
    /// Maximum delta time clamp (prevents spiral of death)
    pub max_delta: Duration,
    /// CPU budget warning threshold per tick
    pub cpu_budget: Duration,
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            tick_rate: 60,
            max_delta: Duration::from_millis(100),
            cpu_budget: Duration::from_micros(4000), // 4ms at 60fps
        }
    }
}

/// Main runtime managing game execution
///
/// Generic over the console type to support different fantasy consoles
/// while sharing the core game loop and rollback infrastructure.
pub struct Runtime<C: Console> {
    /// The console implementation.
    /// Used for console-specific FFI state initialization and ownership of the console
    /// instance for the runtime's lifetime.
    console: C,
    config: RuntimeConfig,
    game: Option<GameInstance<C::Input, C::State, C::RollbackState>>,
    session: Option<RollbackSession<C::Input, C::State, C::RollbackState>>,
    audio: Option<C::Audio>,
    accumulator: Duration,
    last_update: Option<Instant>,
    tick_duration: Duration,
}

impl<C: Console> Runtime<C> {
    /// Create a new runtime for the given console
    pub fn new(console: C) -> Self {
        let config = RuntimeConfig::default();
        let tick_duration = Duration::from_secs_f64(1.0 / config.tick_rate as f64);

        Self {
            console,
            config,
            game: None,
            session: None,
            audio: None,
            accumulator: Duration::ZERO,
            last_update: None,
            tick_duration,
        }
    }

    /// Set the tick rate
    pub fn set_tick_rate(&mut self, tick_rate: u32) {
        self.config.tick_rate = tick_rate;
        self.tick_duration = Duration::from_secs_f64(1.0 / tick_rate as f64);
    }

    /// Get the tick duration (time per tick, inverse of tick rate)
    pub fn tick_duration(&self) -> Duration {
        self.tick_duration
    }

    /// Load a game instance
    pub fn load_game(&mut self, game: GameInstance<C::Input, C::State, C::RollbackState>) {
        self.game = Some(game);
        self.accumulator = Duration::ZERO;
        self.last_update = None;
    }

    /// Set the rollback session
    pub fn set_session(&mut self, session: RollbackSession<C::Input, C::State, C::RollbackState>) {
        self.session = Some(session);
    }

    /// Set the audio backend
    pub fn set_audio(&mut self, audio: C::Audio) {
        self.audio = Some(audio);
    }

    /// Initialize console-specific FFI state before calling game init()
    ///
    /// This allows the console to set up state that the game needs during
    /// initialization (e.g., datapack for rom_* functions).
    pub fn initialize_console_state(&mut self) {
        if let Some(game) = &mut self.game {
            self.console.initialize_ffi_state(game.console_state_mut());
        }
    }

    /// Initialize the loaded game
    pub fn init_game(&mut self) -> Result<()> {
        if let Some(game) = &mut self.game {
            game.init()?;
        }
        Ok(())
    }

    /// Add local input for a player
    ///
    /// Input should be added before calling `frame()` each render loop.
    pub fn add_local_input(
        &mut self,
        player_handle: usize,
        input: C::Input,
    ) -> Result<(), GgrsError> {
        if let Some(session) = &mut self.session {
            let frame = session.current_frame();
            let state = session.session_state();
            tracing::info!(
                "add_local_input: handle={}, frame={}, state={:?}",
                player_handle,
                frame,
                state
            );
            session.add_local_input(player_handle, input)?;
        }
        Ok(())
    }

    /// Poll remote clients (for P2P sessions)
    ///
    /// Should be called regularly, typically at the start of each frame.
    pub fn poll_remote_clients(&mut self) {
        if let Some(session) = &mut self.session {
            session.poll_remote_clients();
        }
    }

    /// Handle session events and return them for the application to process
    ///
    /// Should be called once per frame to get network events, desync warnings, etc.
    pub fn handle_session_events(&mut self) -> Vec<SessionEvent> {
        if let Some(session) = &mut self.session {
            session.handle_events()
        } else {
            Vec::new()
        }
    }

    /// Run a single frame (may include multiple ticks)
    ///
    /// Returns the number of ticks that were executed and the interpolation factor
    /// for rendering between the last two states.
    pub fn frame(&mut self) -> Result<(u32, f32)> {
        self.frame_with_time_scale(1.0)
    }

    /// Run a frame with a time scale modifier.
    ///
    /// Time scale affects how fast game time passes:
    /// - 1.0 = normal speed
    /// - 0.5 = half speed (slow motion)
    /// - 2.0 = double speed (fast forward)
    ///
    /// Returns the number of ticks that were executed and the interpolation factor
    /// for rendering between the last two states.
    pub fn frame_with_time_scale(&mut self, time_scale: f32) -> Result<(u32, f32)> {
        let now = Instant::now();

        // Calculate delta time
        let delta = if let Some(last) = self.last_update {
            let d = now - last;
            if d > self.config.max_delta {
                self.config.max_delta
            } else {
                d
            }
        } else {
            self.tick_duration
        };
        self.last_update = Some(now);

        // Apply time scale to delta before accumulating
        let scaled_delta = delta.mul_f32(time_scale.max(0.0));
        self.accumulator += scaled_delta;

        // Run fixed timestep updates
        let mut ticks = 0u32;

        // If we have a rollback session, use GGRS
        if let Some(session) = &mut self.session {
            // Check if P2P session is still synchronizing
            if let Some(state) = session.session_state() {
                if state != SessionState::Running {
                    // Session is still synchronizing - poll it but don't advance frames
                    session.poll_remote_clients();
                    // Reset accumulator to prevent catchup burst when session starts
                    self.accumulator = Duration::ZERO;
                    // Return 0 ticks and 0.0 interpolation (no game state to interpolate)
                    return Ok((0, 0.0));
                }
            }

            // For P2P sessions, only advance once per render frame to match input cadence
            // (we only add input once per run_game_frame call)
            let is_p2p = session.session_state().is_some();

            while self.accumulator >= self.tick_duration {
                let tick_start = Instant::now();

                // Advance GGRS frame and get requests
                let frame_before = session.current_frame();
                tracing::info!(
                    "advance_frame: about to advance, current_frame={}, state={:?}",
                    frame_before,
                    session.session_state()
                );
                let requests = session
                    .advance_frame()
                    .map_err(|e| anyhow::anyhow!("GGRS advance_frame failed: {}", e))?;

                // Handle all requests (SaveGameState, LoadGameState, AdvanceFrame)
                if let Some(game) = &mut self.game {
                    let advance_inputs = session
                        .handle_requests(game, requests)
                        .map_err(|e| anyhow::anyhow!("GGRS handle_requests failed: {}", e))?;

                    // Note: Audio rollback is automatic via ConsoleRollbackState
                    // Audio state is part of snapshot, no explicit mode tracking needed

                    // Execute each AdvanceFrame with its inputs
                    for inputs in advance_inputs {
                        // Set inputs in GameState for FFI access
                        // Each entry is (input, status) for one player
                        for (player_idx, (input, _status)) in inputs.iter().enumerate() {
                            game.set_input(player_idx, *input);
                        }
                        game.update(self.tick_duration.as_secs_f32())?;
                        ticks += 1;
                    }
                }

                self.accumulator -= self.tick_duration;

                // Check CPU budget
                let tick_time = tick_start.elapsed();
                if tick_time > self.config.cpu_budget {
                    log::warn!(
                        "Tick took {:?}, exceeds budget of {:?}",
                        tick_time,
                        self.config.cpu_budget
                    );
                }

                // For P2P sessions, only advance once per render frame
                // We only receive one input per run_game_frame call, so we can't
                // advance multiple times without new input
                if is_p2p {
                    // Clamp remaining accumulator to prevent runaway catchup
                    if self.accumulator > self.tick_duration {
                        self.accumulator = self.tick_duration;
                    }
                    break;
                }
            }
        } else {
            // No rollback session, run normally
            while self.accumulator >= self.tick_duration {
                let tick_start = Instant::now();

                if let Some(game) = &mut self.game {
                    game.update(self.tick_duration.as_secs_f32())?;
                }

                self.accumulator -= self.tick_duration;
                ticks += 1;

                // Check CPU budget
                let tick_time = tick_start.elapsed();
                if tick_time > self.config.cpu_budget {
                    log::warn!(
                        "Tick took {:?}, exceeds budget of {:?}",
                        tick_time,
                        self.config.cpu_budget
                    );
                }
            }
        }

        // Calculate interpolation factor for rendering
        let alpha = self.accumulator.as_secs_f32() / self.tick_duration.as_secs_f32();

        Ok((ticks, alpha))
    }

    /// Render the current frame
    pub fn render(&mut self) -> Result<()> {
        if let Some(game) = &mut self.game {
            game.render()?;
        }
        Ok(())
    }

    /// Get a reference to the loaded game
    pub fn game(&self) -> Option<&GameInstance<C::Input, C::State, C::RollbackState>> {
        self.game.as_ref()
    }

    /// Get a mutable reference to the loaded game
    pub fn game_mut(&mut self) -> Option<&mut GameInstance<C::Input, C::State, C::RollbackState>> {
        self.game.as_mut()
    }

    /// Get the current tick rate
    pub fn tick_rate(&self) -> u32 {
        self.config.tick_rate
    }

    /// Get the console
    pub fn console(&self) -> &C {
        &self.console
    }

    /// Get a reference to the rollback session
    pub fn session(&self) -> Option<&RollbackSession<C::Input, C::State, C::RollbackState>> {
        self.session.as_ref()
    }

    /// Get a mutable reference to the rollback session
    pub fn session_mut(
        &mut self,
    ) -> Option<&mut RollbackSession<C::Input, C::State, C::RollbackState>> {
        self.session.as_mut()
    }

    /// Get a reference to the audio backend
    pub fn audio(&self) -> Option<&C::Audio> {
        self.audio.as_ref()
    }

    /// Get a mutable reference to the audio backend
    pub fn audio_mut(&mut self) -> Option<&mut C::Audio> {
        self.audio.as_mut()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use nethercore_shared::NETHERCORE_ZX_RAM_LIMIT;
    use wasmtime::Linker;

    use crate::console::{Console, RawInput};
    use crate::test_utils::{TestAudio, TestConsole, TestInput};
    use crate::wasm::{GameInstance, WasmEngine};

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
        let game = GameInstance::<TestInput, ()>::new(&engine, &module, &linker).unwrap();

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
        let game = GameInstance::<TestInput, ()>::new(&engine, &module, &linker).unwrap();

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

        let session = crate::rollback::RollbackSession::<TestInput, ()>::new_local(
            2,
            NETHERCORE_ZX_RAM_LIMIT,
        );
        runtime.set_session(session);

        assert!(runtime.session().is_some());
        assert_eq!(runtime.session().unwrap().local_players().len(), 2);
    }

    #[test]
    fn test_runtime_session_mut() {
        let console = TestConsole;
        let mut runtime = Runtime::<TestConsole>::new(console);

        let session = crate::rollback::RollbackSession::<TestInput, ()>::new_local(
            2,
            NETHERCORE_ZX_RAM_LIMIT,
        );
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
        let game = GameInstance::<TestInput, ()>::new(&engine, &module, &linker).unwrap();

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

        let session = crate::rollback::RollbackSession::<TestInput, ()>::new_local(
            2,
            NETHERCORE_ZX_RAM_LIMIT,
        );
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

        let session = crate::rollback::RollbackSession::<TestInput, ()>::new_local(
            2,
            NETHERCORE_ZX_RAM_LIMIT,
        );
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

        let session = crate::rollback::RollbackSession::<TestInput, ()>::new_local(
            2,
            NETHERCORE_ZX_RAM_LIMIT,
        );
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
}
