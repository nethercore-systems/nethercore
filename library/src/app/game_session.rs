//! Game session lifecycle management

use emberware_core::app::{FRAME_TIME_HISTORY_SIZE, RuntimeError};
use emberware_core::console::Console;
use emberware_core::rollback::{SessionEvent, SessionType};
use emberware_z::library;
use std::path::Path;
use std::time::Instant;
use z_common::ZRom;

use super::App;

/// Load ROM and return WASM bytes
///
/// Supports both .ewz ROM files and raw WASM files.
fn load_rom_wasm(path: &Path) -> Result<Vec<u8>, RuntimeError> {
    if path.extension().and_then(|e| e.to_str()) == Some("ewz") {
        // Load from .ewz ROM file
        let ewz_bytes = std::fs::read(path)
            .map_err(|e| RuntimeError(format!("Failed to read .ewz ROM file: {}", e)))?;

        let rom = ZRom::from_bytes(&ewz_bytes)
            .map_err(|e| RuntimeError(format!("Failed to parse .ewz ROM: {}", e)))?;

        // Extract WASM code from ROM
        Ok(rom.code)
    } else {
        // Load raw WASM file (backward compatibility for development)
        let wasm = std::fs::read(path)
            .map_err(|e| RuntimeError(format!("Failed to read ROM file: {}", e)))?;
        Ok(wasm)
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
        // Clean up game session via ActiveGame
        if let Some(active_game) = &mut self.active_game {
            active_game.unload_game();
        }
        self.last_error = Some(error);
        self.mode = emberware_core::app::AppMode::Library;
        self.local_games = library::get_local_games(&library::ZDataDirProvider);
    }

    /// Handle session events from the rollback session
    ///
    /// Processes network events like disconnect, desync, and network interruption.
    /// Returns an error if a critical event occurs that should terminate the session.
    pub(super) fn handle_session_events(&mut self) -> Result<(), RuntimeError> {
        let active_game = match &mut self.active_game {
            Some(g) if g.has_game() => g,
            _ => return Ok(()),
        };

        // Poll remote clients for network messages (P2P sessions only)
        active_game.poll_remote_clients();

        // Get session events
        let events = active_game.handle_session_events();

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
        let active_game = match &mut self.active_game {
            Some(g) if g.has_game() => g,
            _ => {
                // Clear network stats when no session
                self.debug_stats.ping_ms = None;
                self.debug_stats.rollback_frames = 0;
                self.debug_stats.frame_advantage = 0;
                return;
            }
        };

        let session = match active_game.session_mut() {
            Some(s) => s,
            None => {
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
        let active_game = self
            .active_game
            .as_mut()
            .ok_or_else(|| RuntimeError("No active game".to_string()))?;

        let session = active_game
            .session_mut()
            .ok_or_else(|| RuntimeError("No game session".to_string()))?;

        // First, update input from InputManager
        if let Some(input_manager) = &self.input_manager {
            let console = session.runtime.console();

            // Get input for each local player and set it on the game
            // For now, we support 1 local player (keyboard/gamepad)
            let raw_input = input_manager.get_player_input(0);
            let z_input = console.map_input(&raw_input);

            if let Some(game) = session.runtime.game_mut() {
                game.set_input(0, z_input);
            }
        }

        // Check if frame controller allows running ticks
        let should_run = self.frame_controller.should_run_tick();
        let time_scale = self.frame_controller.time_scale();

        let tick_start = Instant::now();
        let (ticks, _alpha) = if should_run {
            // Apply time scale to the runtime's frame update
            session
                .runtime
                .frame_with_time_scale(time_scale)
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

        // Generate audio samples after rendering (only on confirmed frames)
        // Audio state is in rollback state, which was already advanced during update
        if did_render {
            // Get tick rate and sample rate before borrowing game mutably
            let tick_rate = session.runtime.tick_rate();
            let sample_rate = session
                .runtime
                .audio()
                .map(|a| a.sample_rate())
                .unwrap_or(emberware_z::audio::OUTPUT_SAMPLE_RATE);

            // Generate audio samples from rollback state
            let audio_buffer = if let Some(game) = session.runtime.game_mut() {
                // Clone sounds slice (contains Arcs, cheap to clone)
                let sounds: Vec<Option<emberware_z::audio::Sound>> =
                    game.console_state().sounds.clone();
                let rollback_state = game.rollback_state_mut();

                let mut buffer = Vec::new();
                emberware_z::audio::generate_audio_frame(
                    &mut rollback_state.audio,
                    &sounds,
                    tick_rate,
                    sample_rate,
                    &mut buffer,
                );
                Some(buffer)
            } else {
                None
            };

            // Push samples to audio output (separate borrow)
            if let Some(buffer) = audio_buffer {
                if let Some(audio) = session.runtime.audio_mut() {
                    audio.push_samples(&buffer);
                }
            }
        }

        // Check if game requested quit
        let quit_requested = session
            .runtime
            .game()
            .map(|g| g.state().quit_requested)
            .unwrap_or(false);

        Ok((!quit_requested, did_render))
    }

    /// Start a game by loading its WASM and initializing the runtime
    pub(super) fn start_game(&mut self, game_id: &str) -> Result<(), RuntimeError> {
        // Ensure active_game exists
        let active_game = self
            .active_game
            .as_mut()
            .ok_or_else(|| RuntimeError("Graphics not initialized".to_string()))?;

        // Find the game in local games
        let game = self
            .local_games
            .iter()
            .find(|g| g.id == game_id)
            .ok_or_else(|| RuntimeError(format!("Game not found: {}", game_id)))?;

        // Load ROM file (WASM bytes)
        let rom_bytes = load_rom_wasm(&game.rom_path)?;

        // Load the game via ActiveGame (handles all initialization)
        active_game
            .load_game(&rom_bytes, 1)
            .map_err(|e| RuntimeError(format!("Failed to load game: {}", e)))?;

        // Reset frame controller for new game session
        self.frame_controller.reset();

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
        // Ensure active_game exists
        let active_game = self
            .active_game
            .as_mut()
            .ok_or_else(|| RuntimeError("Graphics not initialized".to_string()))?;

        // Load ROM file (WASM bytes)
        let rom_bytes = load_rom_wasm(&path)?;

        // Load the game via ActiveGame (handles all initialization)
        active_game
            .load_game(&rom_bytes, 1)
            .map_err(|e| RuntimeError(format!("Failed to load game: {}", e)))?;

        // Reset frame controller for new game session
        self.frame_controller.reset();

        tracing::info!("Game started from path: {}", path.display());
        Ok(())
    }
}
