//! Game lifecycle methods: restart, run_game_frame, execute_draw_commands

use std::time::Instant;

use smallvec::SmallVec;

use crate::console::{Audio, AudioGenerator, Console, ConsoleResourceManager};

use super::super::{FRAME_TIME_HISTORY_SIZE, GameError, GameErrorPhase, RuntimeError};
use super::StandaloneApp;
use super::types::RomLoader;

impl<C, L> StandaloneApp<C, L>
where
    C: Console + Clone,
    C::Graphics: super::types::StandaloneGraphicsSupport,
    L: RomLoader<Console = C>,
{
    /// Restarts the game, reloading the ROM and resetting state
    pub(super) fn restart_game(&mut self) {
        self.error_state = None;

        // Try to reload ROM if not already loaded
        if self.loaded_rom.is_none() {
            match L::load_rom(&self.config.rom_path) {
                Ok(rom) => {
                    self.loaded_rom = Some(rom);
                }
                Err(_) => {
                    tracing::error!("Failed to reload ROM for restart");
                    self.should_exit = true;
                    return;
                }
            }
        }

        let rom = match &self.loaded_rom {
            Some(rom) => rom.clone(),
            None => {
                tracing::error!("No ROM loaded for restart");
                self.should_exit = true;
                return;
            }
        };

        if let Some(runner) = &mut self.runner {
            runner.unload_game();
        }

        let console = rom.console.clone();

        if let Some(runner) = &mut self.runner {
            if let Err(e) = runner.load_game(console, &rom.code, 1, &rom.game_id) {
                tracing::error!("Failed to restart game: {}", e);
                self.error_state = Some(GameError {
                    summary: "Restart Failed".to_string(),
                    details: format!("{:#}", e),
                    stack_trace: None,
                    tick: None,
                    phase: GameErrorPhase::Init,
                    suggestions: vec![
                        "The ROM file may be corrupted".to_string(),
                        "Try closing and reopening the player".to_string(),
                    ],
                });
                return;
            }

            if let Some(session) = runner.session_mut()
                && let Some(audio) = session.runtime.audio_mut()
            {
                let config = super::super::config::load();
                audio.set_master_volume(config.audio.master_volume);
            }
            if let Some(session) = runner.session() {
                self.capture.set_source_fps(session.runtime.tick_rate());
            }
        }

        self.next_tick = Instant::now();
        self.needs_redraw = true;
        tracing::info!("Game restarted successfully");
    }

    /// Runs a single game frame: processes input, advances simulation, renders
    ///
    /// Returns (game_running, did_render) or RuntimeError
    pub(super) fn run_game_frame(&mut self) -> Result<(bool, bool), RuntimeError> {
        let runner = self
            .runner
            .as_mut()
            .ok_or_else(|| RuntimeError("No runner".to_string()))?;

        let session = runner
            .session_mut()
            .ok_or_else(|| RuntimeError("No session".to_string()))?;

        // Get local player handles from session (e.g., [0] for host, [1] for joiner)
        // Use SmallVec to avoid heap allocation (max 4 local players)
        let local_players: SmallVec<[usize; 4]> = session
            .runtime
            .session()
            .map(|s| s.local_players().iter().copied().collect())
            .unwrap_or_else(|| smallvec::smallvec![0]);

        // Log once at startup (not every frame)
        static LOGGED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
        if !LOGGED.swap(true, std::sync::atomic::Ordering::Relaxed) {
            tracing::info!("run_game_frame: local_players = {:?}", local_players);
        }

        // Get inputs: from replay script or from input manager
        if let Some(ref executor) = self.replay_executor {
            // Replay mode: use script inputs
            if let Some(frame_inputs) = executor.current_inputs() {
                let console = session.runtime.console().clone();
                for (player_idx, bytes) in frame_inputs.iter().enumerate() {
                    let console_input = console.decode_replay_bytes(bytes);
                    if let Some(game) = session.runtime.game_mut() {
                        game.set_input(player_idx, console_input);
                    }
                    if let Err(e) = session.runtime.add_local_input(player_idx, console_input) {
                        tracing::error!(
                            "Failed to add replay input for player {}: {:?}",
                            player_idx,
                            e
                        );
                    }
                }
            }
        } else {
            // Normal mode: use input manager
            let all_inputs = self.input_manager.get_all_inputs();
            for &player_handle in local_players.iter() {
                let raw_input = all_inputs[player_handle];
                let console_input = session.runtime.console().map_input(&raw_input);

                if let Some(game) = session.runtime.game_mut() {
                    game.set_input(player_handle, console_input);
                }

                // Always add input - GGRS will handle synchronization
                if let Err(e) = session
                    .runtime
                    .add_local_input(player_handle, console_input)
                {
                    tracing::error!(
                        "Failed to add local input for handle {}: {:?}",
                        player_handle,
                        e
                    );
                }
            }
        }

        let should_run = self.frame_controller.should_run_tick();
        let time_scale = self.frame_controller.time_scale();

        // Sync frame control state to WASM context for FFI access
        // Only enabled for local games (FrameController auto-disables for netplay)
        if let Some(game) = session.runtime.game_mut() {
            let state = game.state_mut();
            state.debug_paused = self.frame_controller.is_paused();
            state.debug_time_scale = self.frame_controller.time_scale();
        }

        let tick_start = Instant::now();
        let (ticks, _alpha) = if should_run {
            session
                .runtime
                .frame_with_time_scale(time_scale)
                .map_err(|e| RuntimeError(format!("Game frame error: {}", e)))?
        } else {
            (0, 0.0)
        };
        let tick_elapsed = tick_start.elapsed();

        let did_render = if ticks > 0 {
            let tick_time_ms = tick_elapsed.as_secs_f32() * 1000.0 / ticks as f32;
            self.debug_stats.game_tick_times.push_back(tick_time_ms);
            while self.debug_stats.game_tick_times.len() > FRAME_TIME_HISTORY_SIZE {
                self.debug_stats.game_tick_times.pop_front();
            }

            if let Some(game) = session.runtime.game_mut() {
                C::clear_frame_state(game.console_state_mut());
            }

            let render_start = Instant::now();
            session
                .runtime
                .render()
                .map_err(|e| RuntimeError(format!("Render error: {}", e)))?;
            let render_time_ms = render_start.elapsed().as_secs_f32() * 1000.0;

            self.debug_stats.game_render_times.push_back(render_time_ms);
            while self.debug_stats.game_render_times.len() > FRAME_TIME_HISTORY_SIZE {
                self.debug_stats.game_render_times.pop_front();
            }

            let now = Instant::now();
            for _ in 0..ticks {
                self.game_tick_times.push_back(now);
                if self.game_tick_times.len() > FRAME_TIME_HISTORY_SIZE {
                    self.game_tick_times.pop_front();
                }
            }
            self.last_game_tick = now;

            true
        } else {
            false
        };

        // Process audio using the console's AudioGenerator
        // This handles both synchronous and threaded audio modes automatically
        if did_render {
            let tick_rate = session.runtime.tick_rate();
            let sample_rate = session
                .runtime
                .audio()
                .map(|a| a.sample_rate())
                .unwrap_or_else(C::AudioGenerator::default_sample_rate);

            let (game_opt, audio_opt) = session.runtime.game_and_audio_mut();
            if let (Some(game), Some(audio)) = (game_opt, audio_opt) {
                let (ffi_state, rollback_state) = game.ffi_and_rollback_mut();
                C::AudioGenerator::process_audio(
                    rollback_state,
                    ffi_state,
                    audio,
                    tick_rate,
                    sample_rate,
                );
            }
        }

        let quit_requested = session
            .runtime
            .game()
            .map(|g| g.state().quit_requested)
            .unwrap_or(false);

        Ok((!quit_requested, did_render))
    }

    /// Executes pending draw commands from the console state
    pub(super) fn execute_draw_commands(&mut self) {
        if let Some(runner) = &mut self.runner {
            let (graphics, session_opt) = runner.graphics_and_session_mut();
            if let Some(session) = session_opt
                && let Some(game) = session.runtime.game_mut()
            {
                let state = game.console_state_mut();
                session
                    .resource_manager
                    .execute_draw_commands(graphics, state);
            }
        }
    }

    /// Gets the clear color from console state
    pub(super) fn get_clear_color(&self) -> [f32; 4] {
        if let Some(runner) = &self.runner
            && let Some(session) = runner.session()
            && let Some(game) = session.runtime.game()
        {
            return C::clear_color_from_state(game.console_state());
        }
        [0.1, 0.1, 0.1, 1.0]
    }
}
