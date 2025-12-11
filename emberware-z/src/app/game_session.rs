//! Game session lifecycle management

use crate::console::EmberwareZ;
use crate::library;
use emberware_core::app::{session::GameSession, RuntimeError, FRAME_TIME_HISTORY_SIZE};
use emberware_core::console::Console;
use emberware_core::rollback::{SessionEvent, SessionType};
use emberware_shared::cart::ZDataPack;
use std::path::Path;
use std::time::Instant;

use super::App;

/// Dummy audio backend for resource processing (Z resource manager doesn't use audio)
pub(super) struct DummyAudio;
impl emberware_core::console::Audio for DummyAudio {
    fn play(
        &mut self,
        _handle: emberware_core::console::SoundHandle,
        _volume: f32,
        _looping: bool,
    ) {
    }
    fn stop(&mut self, _handle: emberware_core::console::SoundHandle) {}
    fn set_rollback_mode(&mut self, _rolling_back: bool) {}
}

/// Load WASM bytes and optional data pack from a ROM file path
///
/// Supports both .ewz ROM files and raw WASM files.
fn load_rom(path: &Path) -> Result<(Vec<u8>, Option<ZDataPack>), RuntimeError> {
    if path.extension().and_then(|e| e.to_str()) == Some("ewz") {
        // Load from .ewz ROM file
        let ewz_bytes = std::fs::read(path)
            .map_err(|e| RuntimeError(format!("Failed to read .ewz ROM file: {}", e)))?;

        let rom = emberware_shared::cart::z::ZRom::from_bytes(&ewz_bytes)
            .map_err(|e| RuntimeError(format!("Failed to parse .ewz ROM: {}", e)))?;

        // Extract WASM code and data pack from ROM
        Ok((rom.code, rom.data_pack))
    } else {
        // Load raw WASM file (backward compatibility for development)
        let wasm = std::fs::read(path)
            .map_err(|e| RuntimeError(format!("Failed to read ROM file: {}", e)))?;
        Ok((wasm, None))
    }
}

impl App {
    /// Handle a runtime error by transitioning back to library
    ///
    /// Called when the game runtime encounters an error (WASM panic, network
    /// disconnect, out of memory, etc). Transitions back to library and displays
    /// the error message to the user.
    pub(super) fn handle_runtime_error(&mut self, error: RuntimeError) {
        tracing::error!("Runtime error: {}", error);
        self.game_session = None; // Clean up game session
        self.last_error = Some(error);
        self.mode = emberware_core::app::AppMode::Library;
        self.local_games = library::get_local_games(&library::ZDataDirProvider);
    }

    /// Handle session events from the rollback session
    ///
    /// Processes network events like disconnect, desync, and network interruption.
    /// Returns an error if a critical event occurs that should terminate the session.
    pub(super) fn handle_session_events(&mut self) -> Result<(), RuntimeError> {
        let session = match &mut self.game_session {
            Some(s) => s,
            None => return Ok(()),
        };

        // Poll remote clients for network messages (P2P sessions only)
        session.runtime.poll_remote_clients();

        // Get session events
        let events = session.runtime.handle_session_events();

        // Clear network interrupted flag - will be set again if still interrupted
        self.debug_stats.network_interrupted = None;

        for event in events {
            match event {
                SessionEvent::Disconnected { player_handle } => {
                    tracing::warn!("Player {} disconnected", player_handle);
                    return Err(RuntimeError(format!(
                        "Player {} disconnected from session",
                        player_handle
                    )));
                }
                SessionEvent::Desync {
                    frame,
                    local_checksum,
                    remote_checksum,
                } => {
                    tracing::error!(
                        "Desync detected at frame {}: local={:#x}, remote={:#x}",
                        frame,
                        local_checksum,
                        remote_checksum
                    );
                    return Err(RuntimeError(format!(
                        "Desync detected at frame {} (states diverged)",
                        frame
                    )));
                }
                SessionEvent::NetworkInterrupted {
                    player_handle,
                    disconnect_timeout_ms,
                } => {
                    tracing::warn!(
                        "Network interrupted for player {}, disconnect in {}ms",
                        player_handle,
                        disconnect_timeout_ms
                    );
                    self.debug_stats.network_interrupted = Some(disconnect_timeout_ms);
                }
                SessionEvent::NetworkResumed { player_handle } => {
                    tracing::info!("Network resumed for player {}", player_handle);
                    self.debug_stats.network_interrupted = None;
                }
                SessionEvent::Synchronized { player_handle } => {
                    tracing::info!("Synchronized with player {}", player_handle);
                }
                SessionEvent::FrameAdvantageWarning { frames_ahead } => {
                    tracing::debug!("Frame advantage warning: {} frames ahead", frames_ahead);
                }
                SessionEvent::TimeSync { frames_to_skip } => {
                    tracing::debug!("Time sync: skip {} frames", frames_to_skip);
                }
                SessionEvent::WaitingForPlayers => {
                    tracing::trace!("Waiting for remote player input");
                }
            }
        }

        Ok(())
    }

    /// Update debug stats from the current session
    ///
    /// Populates network statistics in DebugStats from the rollback session.
    pub(super) fn update_session_stats(&mut self) {
        let session = match &self.game_session {
            Some(s) => s,
            None => {
                // Clear network stats when no session
                self.debug_stats.ping_ms = None;
                self.debug_stats.rollback_frames = 0;
                self.debug_stats.frame_advantage = 0;
                return;
            }
        };

        // Get session reference
        let rollback_session = match session.runtime.session() {
            Some(s) => s,
            None => {
                self.debug_stats.ping_ms = None;
                self.debug_stats.rollback_frames = 0;
                self.debug_stats.frame_advantage = 0;
                return;
            }
        };

        // Only show network stats for P2P sessions
        if rollback_session.session_type() != SessionType::P2P {
            self.debug_stats.ping_ms = None;
            self.debug_stats.rollback_frames = 0;
            self.debug_stats.frame_advantage = 0;
            return;
        }

        // Get stats from the first remote player
        let player_stats = rollback_session.all_player_stats();
        let local_players = rollback_session.local_players();

        // Find first remote player's stats
        for (idx, stats) in player_stats.iter().enumerate() {
            if !local_players.contains(&idx) {
                self.debug_stats.ping_ms = Some(stats.ping_ms);
                break;
            }
        }

        self.debug_stats.rollback_frames = rollback_session.total_rollback_frames();
        self.debug_stats.frame_advantage = rollback_session.frames_ahead();
    }

    /// Run one game frame (update + render)
    ///
    /// Returns true if the game is still running, false if it should exit.
    /// Returns (game_still_running, did_render_this_frame)
    pub(super) fn run_game_frame(&mut self) -> Result<(bool, bool), RuntimeError> {
        // First, update input from InputManager
        if let (Some(session), Some(input_manager)) = (&mut self.game_session, &self.input_manager)
        {
            let console = session.runtime.console();

            // Get input for each local player and set it on the game
            // For now, we support 1 local player (keyboard/gamepad)
            let raw_input = input_manager.get_player_input(0);
            let z_input = console.map_input(&raw_input);

            if let Some(game) = session.runtime.game_mut() {
                game.set_input(0, z_input);
            }
        }

        // Run the game frame (fixed timestep updates)
        let session = self
            .game_session
            .as_mut()
            .ok_or_else(|| RuntimeError("No game session".to_string()))?;

        // Check if frame controller allows running ticks
        let should_run = self.frame_controller.should_run_tick();

        let tick_start = Instant::now();
        let (ticks, _alpha) = if should_run {
            // Apply time scale by modifying the runtime's accumulated time
            // For now, we run at normal speed when not paused
            // Time scale affects visual smoothness, not tick rate
            session
                .runtime
                .frame()
                .map_err(|e| RuntimeError(format!("Game frame error: {}", e)))?
        } else {
            // When paused, don't run any ticks
            (0, 0.0)
        };
        let tick_elapsed = tick_start.elapsed();

        let did_render = if ticks > 0 {
            // Track game tick time for performance graph (average per tick if multiple ran)
            let tick_time_ms = tick_elapsed.as_secs_f32() * 1000.0 / ticks as f32;
            self.debug_stats.game_tick_times.push_back(tick_time_ms);
            while self.debug_stats.game_tick_times.len() > FRAME_TIME_HISTORY_SIZE {
                self.debug_stats.game_tick_times.pop_front();
            }

            // Clear previous frame's draw commands before generating new ones
            if let Some(game) = session.runtime.game_mut() {
                game.console_state_mut().clear_frame();
            }

            // Render the game ONCE per frame (even if multiple ticks ran due to slowdown)
            let render_start = Instant::now();
            session
                .runtime
                .render()
                .map_err(|e| RuntimeError(format!("Game render error: {}", e)))?;
            let render_elapsed = render_start.elapsed();
            let render_time_ms = render_elapsed.as_secs_f32() * 1000.0;

            // Track game render time
            self.debug_stats.game_render_times.push_back(render_time_ms);
            while self.debug_stats.game_render_times.len() > FRAME_TIME_HISTORY_SIZE {
                self.debug_stats.game_render_times.pop_front();
            }

            tracing::debug!(
                "Game tick: {} ticks, update={:.2}ms, render={:.2}ms, total={:.2}ms, buffer size: {}",
                ticks,
                tick_time_ms,
                render_time_ms,
                tick_time_ms + render_time_ms,
                self.debug_stats.game_tick_times.len()
            );

            // Track game tick times for FPS calculation
            let now = Instant::now();
            for _ in 0..ticks {
                self.game_tick_times.push(now);
                if self.game_tick_times.len() > FRAME_TIME_HISTORY_SIZE {
                    self.game_tick_times.remove(0);
                }
            }
            self.last_game_tick = now;

            true // Did render
        } else {
            false // No render
        };

        // Process audio commands after rendering
        // Use mem::take to avoid cloning - takes ownership and leaves empty vecs
        let (audio_commands, sounds) = if let Some(game) = session.runtime.game_mut() {
            let console_state = game.console_state_mut();
            (
                std::mem::take(&mut console_state.audio_commands),
                console_state.sounds.clone(), // sounds must be cloned (contains Arcs, shared with audio system)
            )
        } else {
            (Vec::new(), Vec::new())
        };

        if !audio_commands.is_empty() {
            if let Some(audio) = session.runtime.audio_mut() {
                audio.process_commands(&audio_commands, &sounds);
            }
        }

        // Check if game requested quit
        if let Some(game) = session.runtime.game_mut() {
            if game.state().quit_requested {
                return Ok((false, did_render));
            }
        }

        Ok((true, did_render))
    }

    /// Internal helper to initialize and start a game from WASM bytes and optional data pack
    fn initialize_game(
        &mut self,
        rom_bytes: Vec<u8>,
        data_pack: Option<ZDataPack>,
    ) -> Result<(), RuntimeError> {
        // Ensure WASM engine is available
        let wasm_engine = self
            .wasm_engine
            .as_ref()
            .ok_or_else(|| RuntimeError("WASM engine not initialized".to_string()))?;

        // Load the WASM module
        let module = wasm_engine
            .load_module(&rom_bytes)
            .map_err(|e| RuntimeError(format!("Failed to load WASM module: {}", e)))?;

        // Create a linker and register FFI functions
        let mut linker = wasmtime::Linker::new(wasm_engine.engine());

        // Register common FFI functions
        emberware_core::ffi::register_common_ffi(&mut linker)
            .map_err(|e| RuntimeError(format!("Failed to register common FFI: {}", e)))?;

        // Create the console instance
        let console = EmberwareZ::new();

        // Register console-specific FFI functions
        console
            .register_ffi(&mut linker)
            .map_err(|e| RuntimeError(format!("Failed to register Z FFI: {}", e)))?;

        // Create the game instance
        let game_instance = emberware_core::wasm::GameInstance::new(wasm_engine, &module, &linker)
            .map_err(|e| RuntimeError(format!("Failed to instantiate game: {}", e)))?;

        // Create the runtime
        let mut runtime = emberware_core::runtime::Runtime::new(console);
        runtime.load_game(game_instance);

        // Set data pack on console state (before init, so rom_* functions work)
        if let Some(data_pack) = data_pack {
            if let Some(game) = runtime.game_mut() {
                game.console_state_mut().data_pack = Some(std::sync::Arc::new(data_pack));
                tracing::info!("Data pack loaded with assets");
            }
        }

        // Initialize the game (calls game's init() function)
        runtime
            .init_game()
            .map_err(|e| RuntimeError(format!("Failed to initialize game: {}", e)))?;

        // Finalize debug registration (prevents further registration after init)
        if let Some(game) = runtime.game_mut() {
            game.store_mut()
                .data_mut()
                .debug_registry
                .finalize_registration();
        }

        // Reset frame controller for new game session
        self.frame_controller.reset();

        // Create resource manager
        let resource_manager = EmberwareZ::new().create_resource_manager();

        // Create the game session
        self.game_session = Some(GameSession::new(runtime, resource_manager));

        // Add built-in font texture to texture map (handle 0)
        // Add white fallback texture to texture map (handle 0xFFFFFFFF)
        if let (Some(session), Some(graphics)) = (&mut self.game_session, &self.graphics) {
            let font_texture_handle = graphics.font_texture();
            session
                .resource_manager
                .texture_map
                .insert(0, font_texture_handle);
            tracing::info!(
                "Initialized font texture in texture_map: handle 0 -> {:?}",
                font_texture_handle
            );

            let white_texture_handle = graphics.white_texture();
            session
                .resource_manager
                .texture_map
                .insert(u32::MAX, white_texture_handle);
            tracing::info!(
                "Initialized white texture in texture_map: handle 0xFFFFFFFF -> {:?}",
                white_texture_handle
            );
        }

        // Update render target resolution and window minimum size based on game's init config
        if let Some(session) = &self.game_session {
            if let Some(game) = session.runtime.game() {
                let z_state = game.console_state();
                let resolution_index = z_state.init_config.resolution_index as u8;

                // Update graphics render target to match game resolution
                if let Some(graphics) = &mut self.graphics {
                    graphics.update_resolution(resolution_index);

                    // Update window minimum size to match game resolution
                    if let Some(window) = &self.window {
                        let min_size =
                            winit::dpi::PhysicalSize::new(graphics.width(), graphics.height());
                        window.set_min_inner_size(Some(min_size));
                    }
                }
            }
        }

        Ok(())
    }

    /// Start a game by loading its WASM and initializing the runtime
    pub(super) fn start_game(&mut self, game_id: &str) -> Result<(), RuntimeError> {
        // Clear resources from previous game (clear-on-init pattern)
        // This handles crashes/failed init gracefully since the next game load clears stale state
        if let Some(graphics) = &mut self.graphics {
            graphics.clear_game_resources();
        }

        // Find the game in local games
        let game = self
            .local_games
            .iter()
            .find(|g| g.id == game_id)
            .ok_or_else(|| RuntimeError(format!("Game not found: {}", game_id)))?;

        // Load ROM file (WASM bytes and optional data pack)
        let (rom_bytes, data_pack) = load_rom(&game.rom_path)?;

        // Initialize and start the game
        self.initialize_game(rom_bytes, data_pack)?;

        tracing::info!("Game started: {}", game_id);
        Ok(())
    }

    /// Start a game directly from a file path (for debugging/development).
    ///
    /// This bypasses the library and loads a game directly from the given path.
    /// Supports both .ewz ROM files and raw .wasm files.
    pub(super) fn start_game_from_path(
        &mut self,
        path: std::path::PathBuf,
    ) -> Result<(), RuntimeError> {
        // Clear resources from previous game
        if let Some(graphics) = &mut self.graphics {
            graphics.clear_game_resources();
        }

        // Load ROM file (WASM bytes and optional data pack)
        let (rom_bytes, data_pack) = load_rom(&path)?;

        // Initialize and start the game
        self.initialize_game(rom_bytes, data_pack)?;

        tracing::info!("Game started from path: {}", path.display());
        Ok(())
    }
}
